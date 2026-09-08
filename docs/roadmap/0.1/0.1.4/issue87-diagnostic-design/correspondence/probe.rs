// Appended by run.py to the pinned existing fixture; imports the real cursor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Reason { Memory, File, Operation, Descriptor }
#[derive(Debug)]
struct Observation {
    hints: [Option<ObjectId>; 4],
    limited: Option<Reason>,
    inherited: bool,
    grants: u8,
}

fn observe(
    cursor: &mut PredecessorCursor, memory: &Memory, start: u64, len: u32,
    sticky: &mut Option<Reason>, mut gate: impl FnMut() -> Result<(), Reason>,
) -> Observation {
    let inherited = cursor.counters().1 != 0;
    let mut denied = None;
    let mut grants = 0_u8;
    let hints = cursor.hints(memory, start, len, || match gate() {
        Ok(()) => { grants = grants.checked_add(1).unwrap(); true },
        Err(reason) => { assert!(denied.is_none()); denied = Some(reason); false }
    }).unwrap();
    if !inherited && cursor.counters().1 != 0 {
        *sticky = Some(denied.unwrap_or(Reason::Descriptor));
    }
    assert_eq!(cursor.counters().1 != 0, sticky.is_some());
    Observation { hints, limited: *sticky, inherited, grants }
}

fn diagnostic_reserve(available: bool, file: &mut u64, operation: &AtomicU64) -> Result<(), Reason> {
    // Preserve the original short-circuit order and exactly one fetch_update.
    if !available { return Err(Reason::Memory); }
    if *file + 131136 > 1024 * 1024 { return Err(Reason::File); }
    if operation.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |used| {
        used.checked_add(131136).filter(|sum| *sum <= 16 * 1024 * 1024)
    }).is_err() { return Err(Reason::Operation); }
    *file += 131136;
    Ok(())
}

fn main() {
    let mut memory = Memory::default();
    let root = memory.file(1, 1);
    // Collision precedence: unavailable memory wins over full file/global caps;
    // a full file cap wins over a full operation cap. No later gate is executed.
    for (available, file_start, operation_start, expected) in [
        (false, 917952, 16654272, Reason::Memory),
        (true, 917952, 16654272, Reason::File),
        (true, 0, 16654272, Reason::Operation),
    ] {
        let mut original = PredecessorCursor::new(root);
        let mut observed = PredecessorCursor::new(root);
        let (mut file_a, mut file_b) = (file_start, file_start);
        let (op_a, op_b) = (AtomicU64::new(operation_start), AtomicU64::new(operation_start));
        let reads_before = memory.read_bytes.get();
        let a = original.hints(&memory, 0, 4, || available && reserve(&mut file_a, &op_a)).unwrap();
        let reads_a = memory.read_bytes.get() - reads_before;
        let reads_before = memory.read_bytes.get();
        let mut sticky = None;
        let b = observe(&mut observed, &memory, 0, 4, &mut sticky, || diagnostic_reserve(available, &mut file_b, &op_b));
        assert_eq!(a, b.hints);
        assert_eq!(b.limited, Some(expected));
        assert_eq!(b.grants, 0);
        assert!(!b.inherited);
        assert_eq!((file_a, op_a.load(Ordering::Relaxed), original.counters(), reads_a),
                   (file_b, op_b.load(Ordering::Relaxed), observed.counters(), memory.read_bytes.get()-reads_before));
        let after = observe(&mut observed, &memory, 4, 4, &mut sticky, || panic!("inherited exhaustion must not retry"));
        assert!(after.inherited);
        assert_eq!(after.grants, 0);
        assert_eq!(after.limited, Some(expected));
    }
    let mut cursor = PredecessorCursor::new(root);
    let empty = observe(&mut cursor, &memory, 4, 4, &mut None, || Ok(()));
    assert_eq!(empty.hints, [None; 4]);
    assert_eq!(empty.limited, None);

    // Partial hints survive an exhaustion transition during the same call.
    let big = memory.file(33, 128);
    let mut original = PredecessorCursor::new(big);
    let mut cursor = PredecessorCursor::new(big);
    let (mut file_a, mut file_b) = (4*131136, 4*131136);
    let (op_a, op_b) = (AtomicU64::new(0), AtomicU64::new(0));
    let a = original.hints(&memory, 508, 8, || reserve(&mut file_a, &op_a)).unwrap();
    let mut sticky = None;
    let partial = observe(&mut cursor, &memory, 508, 8, &mut sticky, || diagnostic_reserve(true, &mut file_b, &op_b));
    assert_eq!(partial.hints, a);
    assert_eq!(partial.hints.iter().flatten().count(), 1);
    assert_eq!(partial.limited, Some(Reason::File));
    assert_eq!(partial.grants, 3);
    assert_eq!((file_a, op_a.load(Ordering::Relaxed), original.counters()),
               (file_b, op_b.load(Ordering::Relaxed), cursor.counters()));

    // Exhaustion without a rejected callback uniquely means descriptor cap in
    // this pinned cursor. Unlimited reservation is synthetic discrimination only.
    let mut cursor = PredecessorCursor::new(big);
    for i in 0..4095 { cursor.hints(&memory, i*4, 4, || true).unwrap(); }
    let mut sticky = None;
    let partial = observe(&mut cursor, &memory, 4095*4, 8, &mut sticky, || Ok(()));
    assert_eq!(partial.hints.iter().flatten().count(), 1);
    assert_eq!(partial.limited, Some(Reason::Descriptor));

    // Complete empty correspondence is possible only at/past EOF for a valid
    // contiguous predecessor, including forward gaps and retained current nodes.
    let short = memory.file(1, 8);
    for first_start in 0..40 {
        for first_len in 1..9 {
            for second_start in first_start + first_len..48 {
                let mut cursor = PredecessorCursor::new(short);
                let first = cursor.hints(&memory, first_start, first_len as u32, || true).unwrap();
                assert_eq!(first.iter().all(Option::is_none), first_start >= 32);
                let second = cursor.hints(&memory, second_start, 7, || true).unwrap();
                assert_eq!(second.iter().all(Option::is_none), second_start >= 32);
                assert_eq!(cursor.counters().1, 0);
            }
        }
    }

    // Check proposed diagnostic tag fits existing in-memory accounting on this
    // compiler/target. This is a guard, never a cross-platform layout promise.
    #[allow(dead_code)]
    struct Before { prior_ids: [Option<ObjectId>; 4], first_span: Option<(u64,u32)>, has_predecessor: bool }
    #[allow(dead_code)]
    struct After { prior_ids: [Option<ObjectId>; 4], first_span: Option<(u64,u32)>, has_predecessor: bool, diagnostic: u8, triggered_reservation_grants: u8 }
    #[allow(dead_code)]
    struct Canonical { id: ObjectId, bytes: Vec<u8> }
    #[allow(dead_code)]
    struct AuthBefore(Canonical, Before);
    #[allow(dead_code)]
    struct AuthAfter(Canonical, After);
    assert_eq!(std::mem::size_of::<Before>(), std::mem::size_of::<After>());
    assert_eq!(std::mem::align_of::<Before>(), std::mem::align_of::<After>());
    assert_eq!(std::mem::size_of::<AuthBefore>(), std::mem::size_of::<AuthAfter>());
    assert_eq!(std::mem::align_of::<AuthBefore>(), std::mem::align_of::<AuthAfter>());
    // Exercise all legal prospective diagnostic tags/grants through the exact
    // existing fixed-width hint-frame slots. No Store/spill files are opened.
    for stop in 0_u8..=5 {
        for inherited in [false,true] {
            if inherited && stop < 2 { continue; }
            for grants in 0_u8..=7 {
                if inherited && grants != 0 { continue; }
                let mut encoded_hints = [0_u8;144];
                encoded_hints[13] = stop | (u8::from(inherited)<<3);
                encoded_hints[14] = grants;
                assert_eq!(encoded_hints[13]&7,stop);
                assert_eq!(encoded_hints[13]&8 != 0,inherited);
                assert_eq!(encoded_hints[14],grants);
                assert_eq!(encoded_hints[15],0);
                assert_eq!(32+8+encoded_hints.len(),184);
            }
        }
    }
    println!("{{\"all_checks_passed\":true,\"baseline_hints_size_bytes\":{},\"tagged_hints_size_bytes\":{},\"baseline_authenticated_size_bytes\":{},\"tagged_authenticated_size_bytes\":{},\"spill_hint_frame_bytes\":144,\"spill_row_overhead_bytes\":184,\"policy_counter_output_equivalence_checked\":true,\"partial_hints_not_terminal_failure\":true,\"complete_empty_requires_eof_check\":true}}", std::mem::size_of::<Before>(), std::mem::size_of::<After>(),std::mem::size_of::<AuthBefore>(),std::mem::size_of::<AuthAfter>());
}
