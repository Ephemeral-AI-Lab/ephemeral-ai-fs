//! Synthetic deterministic matcher checks only: no Store, encoding, or benchmark data.
#![allow(dead_code)]
use layerfs_layerstack_store::{Result, StoreError, PhysicalStorageReceipt};
mod telemetry { pub use layerfs_layerstack_store::PhysicalStorageReceipt; }
fn elapsed_ns(t: std::time::Instant) -> u64 { t.elapsed().as_nanos().try_into().unwrap() }
#[rustfmt::skip]
#[path = "../../../../../../../crates/layerfs-layerstack-store/src/objects/pack.rs"]
mod pack;
fn main() {
    use layerfs_content::file::extent_codec::encode_chunk_object;
    let mut state=0x12345678u32;
    let mut payload=(0..1024).map(|_| {state^=state<<13;state^=state>>17;state^=state<<5;state as u8}).collect::<Vec<_>>();
    let seed=(0..16).map(|x| x as u8).collect::<Vec<_>>();
    for start in [11,43,75,107,139] { payload[start..start+16].copy_from_slice(&seed); }
    let base=encode_chunk_object(&payload).unwrap();
    let target=encode_chunk_object(&payload[139..267]).unwrap();
    assert_eq!(base.len(),1045); assert_eq!(target.len(),149);
    let id=layerfs_content::identify_canonical(&base).unwrap().0;
    let mut work=16*1024*1024;
    let mut stats=PhysicalStorageReceipt::default();
    let candidate=pack::delta_record(id,&base,&target,&mut work,&mut stats).unwrap().unwrap();
    let pack::Record::Delta{instructions,count,output_length,..}=pack::record(&candidate).unwrap() else {panic!()};
    assert_eq!(pack::apply_delta(instructions,count,output_length,&base).unwrap(),target);
    // A valid witness program with the fifth seed offset; not an alternate matcher.
    let mut witness=vec![1]; witness.extend_from_slice(&21u32.to_le_bytes());witness.extend_from_slice(&target[..21]);
    witness.push(0);witness.extend_from_slice(&160u32.to_le_bytes());witness.extend_from_slice(&128u32.to_le_bytes());
    assert_eq!(pack::apply_delta(&witness,2,target.len(),&base).unwrap(),target);
    assert!(candidate.len()>41+witness.len());
    assert_eq!(stats.match_budget_skips,0);
    let mut budget=0;let mut exhausted=PhysicalStorageReceipt::default();
    assert!(pack::delta_record(id,&base,&target,&mut budget,&mut exhausted).unwrap().is_none());
    assert_eq!(exhausted.match_budget_skips,1);
    assert_eq!(exhausted.budget_skips,0); // Caller supplies generic event; not a terminal outcome.
    println!("{{\"status\":\"PASS\",\"scope\":\"synthetic exact-source matcher; no codec called\",\"base_canonical_bytes\":{},\"target_canonical_bytes\":{},\"current_delta_record_bytes\":{},\"valid_witness_record_bytes\":{},\"match_comparisons_count\":{},\"seed_hash_bytes\":{},\"exhausted_match_skips_count\":{},\"exhausted_inner_generic_skips_count\":{}}}",base.len(),target.len(),candidate.len(),41+witness.len(),stats.match_comparisons,stats.seed_hash_bytes,exhausted.match_budget_skips,exhausted.budget_skips);
}
