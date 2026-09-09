// Diagnostic only. The included matcher is exact source text, not a reimplementation.
type Result<T> = std::result::Result<T, ()>;
const GROUP_LIMIT: usize = 65536;
const RECORD_COUNT_LIMIT: usize = 8191;
struct ObjectId([u8; 32]);
impl ObjectId { fn as_bytes(&self) -> &[u8; 32] { &self.0 } }
fn invalid() {}
fn put_u32(out: &mut Vec<u8>, n: usize) -> Result<()> {
    out.extend_from_slice(&u32::try_from(n).map_err(|_| ())?.to_le_bytes());
    Ok(())
}
mod telemetry {
    #[derive(Default)]
    pub struct PhysicalStorageReceipt {
        pub match_budget_skips: u64,
        pub seed_hash_bytes: u64,
        pub match_comparisons: u64,
        pub instruction_budget_skips: u64,
    }
}
mod exact {
    use super::*;
    include!("delta_record.rs");
}

// Return -1 for a declined candidate and -2 for an invalid FFI request/matcher error.
// Caller owns all buffers and ensures they do not overlap.
#[no_mangle]
pub unsafe extern "C" fn diagnostic_delta(
    base_id: *const u8, base: *const u8, base_len: usize,
    target: *const u8, target_len: usize, remaining: *mut usize,
    output: *mut u8, output_capacity: usize, counters: *mut u64,
) -> isize {
    if base_id.is_null() || base.is_null() || target.is_null() || remaining.is_null()
        || output.is_null() || counters.is_null() || !(1..=8192).contains(&base_len)
        || !(1..=8192).contains(&target_len) || output_capacity < target_len + 1 {
        return -2;
    }
    let mut id = [0;32];
    id.copy_from_slice(std::slice::from_raw_parts(base_id,32));
    let mut stats = telemetry::PhysicalStorageReceipt::default();
    let result = exact::delta_record(ObjectId(id), std::slice::from_raw_parts(base,base_len),
        std::slice::from_raw_parts(target,target_len), &mut *remaining, &mut stats);
    let values = [stats.match_budget_skips, stats.seed_hash_bytes, stats.match_comparisons, stats.instruction_budget_skips];
    std::ptr::copy_nonoverlapping(values.as_ptr(),counters,4);
    match result {
        Ok(Some(bytes)) => {
            if bytes.len() > output_capacity { return -2; }
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), output, bytes.len());
            bytes.len() as isize
        }
        Ok(None) => -1,
        Err(()) => -2,
    }
}
