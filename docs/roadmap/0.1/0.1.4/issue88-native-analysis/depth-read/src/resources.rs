// Exact macOS resource primitives copied from the existing benchmark private helper.
// No public library export exists; no new resource sampling algorithm.
#[derive(Clone, Copy)]
struct ProcessResourceSnapshot {
    user_cpu_ns: u64,
    system_cpu_ns: u64,
    resident_bytes: u64,
    peak_resident_bytes: u64,
    physical_footprint_bytes: u64,
    disk_read_bytes: u64,
    disk_write_bytes: u64,
    context_switches: u64,
    swaps: u64,
    threads: u64,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[repr(C)]
#[derive(Default)]
struct NativeTimeval {
    seconds: i64,
    microseconds: i64,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[repr(C)]
#[derive(Default)]
struct NativeRusage {
    user_time: NativeTimeval,
    system_time: NativeTimeval,
    max_rss: i64,
    shared_memory: i64,
    unshared_data: i64,
    unshared_stack: i64,
    minor_faults: i64,
    major_faults: i64,
    swaps: i64,
    block_inputs: i64,
    block_outputs: i64,
    messages_sent: i64,
    messages_received: i64,
    signals: i64,
    voluntary_context_switches: i64,
    involuntary_context_switches: i64,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
unsafe extern "C" {
    fn getrusage(who: i32, usage: *mut NativeRusage) -> i32;
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn native_peak_rss_and_swaps() -> AnyResult<(u64, u64)> {
    native_peak_rss_and_swaps_for(0)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn native_peak_rss_and_swaps_for(who: i32) -> AnyResult<(u64, u64)> {
    let mut usage = NativeRusage::default();
    // SAFETY: getrusage writes exactly one native rusage C-layout structure.
    if unsafe { getrusage(who, std::ptr::from_mut(&mut usage)) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let peak = u64::try_from(usage.max_rss)?;
    #[cfg(target_os = "linux")]
    let peak = peak.checked_mul(1024).ok_or("Linux peak RSS overflow")?;
    Ok((peak, u64::try_from(usage.swaps)?))
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Default)]
struct DarwinRusageInfoV2 {
    uuid: [u8; 16],
    user_time: u64,
    system_time: u64,
    package_idle_wakeups: u64,
    interrupt_wakeups: u64,
    pageins: u64,
    wired_size: u64,
    resident_size: u64,
    physical_footprint: u64,
    process_start_time: u64,
    process_exit_time: u64,
    child_user_time: u64,
    child_system_time: u64,
    child_package_idle_wakeups: u64,
    child_interrupt_wakeups: u64,
    child_pageins: u64,
    child_elapsed_time: u64,
    disk_read_bytes: u64,
    disk_write_bytes: u64,
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Default)]
struct DarwinTaskInfo {
    virtual_size: u64,
    resident_size: u64,
    total_user: u64,
    total_system: u64,
    threads_user: u64,
    threads_system: u64,
    policy: i32,
    faults: i32,
    pageins: i32,
    cow_faults: i32,
    messages_sent: i32,
    messages_received: i32,
    mach_syscalls: i32,
    unix_syscalls: i32,
    context_switches: i32,
    thread_count: i32,
    running_threads: i32,
    priority: i32,
}

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Default)]
struct MachTimebaseInfo {
    numerator: u32,
    denominator: u32,
}

#[cfg(target_os = "macos")]
#[link(name = "proc")]
unsafe extern "C" {
    fn proc_pid_rusage(pid: i32, flavor: i32, buffer: *mut std::ffi::c_void) -> i32;
    fn proc_pidinfo(
        pid: i32,
        flavor: i32,
        argument: u64,
        buffer: *mut std::ffi::c_void,
        size: i32,
    ) -> i32;
    fn mach_timebase_info(info: *mut MachTimebaseInfo) -> i32;
}

#[cfg(target_os = "macos")]
fn process_resource_snapshot() -> AnyResult<ProcessResourceSnapshot> {
    const RUSAGE_INFO_V2: i32 = 2;
    const PROC_PIDTASKINFO: i32 = 4;
    let pid = i32::try_from(std::process::id())?;
    let mut usage = DarwinRusageInfoV2::default();
    let mut task = DarwinTaskInfo::default();
    let mut timebase = MachTimebaseInfo::default();
    // SAFETY: both calls target this process and receive correctly sized C-layout buffers.
    let usage_status =
        unsafe { proc_pid_rusage(pid, RUSAGE_INFO_V2, std::ptr::from_mut(&mut usage).cast()) };
    // SAFETY: PROC_PIDTASKINFO writes exactly one proc_taskinfo-compatible buffer.
    let task_bytes = unsafe {
        proc_pidinfo(
            pid,
            PROC_PIDTASKINFO,
            0,
            std::ptr::from_mut(&mut task).cast(),
            i32::try_from(std::mem::size_of::<DarwinTaskInfo>())?,
        )
    };
    // SAFETY: mach_timebase_info initializes the two-field C-layout structure.
    let timebase_status = unsafe { mach_timebase_info(std::ptr::from_mut(&mut timebase)) };
    if usage_status != 0
        || usize::try_from(task_bytes)? != std::mem::size_of::<DarwinTaskInfo>()
        || timebase_status != 0
        || timebase.denominator == 0
    {
        return Err(std::io::Error::last_os_error().into());
    }
    let to_nanoseconds = |value: u64| -> AnyResult<u64> {
        u64::try_from(
            u128::from(value)
                .checked_mul(u128::from(timebase.numerator))
                .ok_or("Mach CPU time overflow")?
                / u128::from(timebase.denominator),
        )
        .map_err(Into::into)
    };
    let (peak_resident_bytes, swaps) = native_peak_rss_and_swaps()?;
    Ok(ProcessResourceSnapshot {
        user_cpu_ns: to_nanoseconds(usage.user_time)?,
        system_cpu_ns: to_nanoseconds(usage.system_time)?,
        resident_bytes: usage.resident_size,
        peak_resident_bytes,
        physical_footprint_bytes: usage.physical_footprint,
        disk_read_bytes: usage.disk_read_bytes,
        disk_write_bytes: usage.disk_write_bytes,
        context_switches: u64::try_from(task.context_switches)?,
        swaps,
        threads: u64::try_from(task.thread_count)?,
    })
}
