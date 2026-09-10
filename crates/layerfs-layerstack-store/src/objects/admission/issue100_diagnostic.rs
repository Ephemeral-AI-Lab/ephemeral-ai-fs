//! Original-fixture codec diagnostic; no Store/allocation/performance claim.
use super::*;
use layerfs_content::file::content;
use std::process::Command;

const EVIDENCE: &str = "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-evidence";
struct Encoded {
    id: ObjectId,
    raw_len: usize,
    frame: Box<[u8]>,
    base: Option<usize>,
    depth: usize,
    closure: usize,
    encoded_closure: usize,
}
fn original(commit: &str, path: &str) -> Option<Vec<u8>> {
    let output = Command::new("git")
        .arg(format!("--git-dir={EVIDENCE}/git/snapshots.git"))
        .args(["show", &format!("{commit}:{path}")])
        .output()
        .unwrap();
    if output.status.success() {
        Some(output.stdout)
    } else {
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("does not exist in"),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        None
    }
}
fn reconstruct(records: &[Encoded], index: usize) -> Vec<u8> {
    let mut chain = vec![index];
    while let Some(base) = records[*chain.last().unwrap()].base {
        assert!(base < *chain.last().unwrap());
        chain.push(base);
    }
    assert!(chain.len() <= 9);
    assert!(
        chain
            .iter()
            .map(|&i| records[i].raw_len + 23)
            .sum::<usize>()
            <= 512 * 1024
    );
    assert!(
        chain
            .iter()
            .map(|&i| records[i].frame.len() + 9 + usize::from(records[i].base.is_some()) * 32)
            .sum::<usize>()
            <= 256 * 1024
    );
    let mut raw: Option<Vec<u8>> = None;
    for &i in chain.iter().rev() {
        let node = &records[i];
        let next = pack::small_decompress(&node.frame, node.raw_len, raw.as_deref()).unwrap();
        assert_eq!(
            ObjectId::for_bytes(&content::encode_small(&next).unwrap()),
            node.id
        );
        raw = Some(next);
    }
    raw.unwrap()
}

#[test]
#[ignore = "uses immutable original ten-snapshot Git fixture; codec diagnostic only"]
fn issue100_actual_family_predecessor_diagnostic() {
    let output = Command::new("python3").args(["-c", r#"
import json,sys
p=sys.argv[1]
r=json.load(open(p+'/git/results.json'))
assert r['checkpoints']==10 and r['manifest_sha256']=='03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271'
m=json.load(open(p+'/git/mapping.json'))
assert [s['full157_index'] for s in m]==[1,18,36,53,70,88,105,122,140,157]
for s in m: print(s['index'],s['full157_index'],s['git_commit'],sep='\t')
"#, EVIDENCE]).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mapping = String::from_utf8(output.stdout).unwrap();
    println!("scope\tactual-three-family-codec-diagnostic\tno-product-allocation-claim\tmax_edges=8\tcanonical_closure=524288\tencoded_capacity=262144\tpack_headers_SQL_indexes_CDC_excluded\toracle_storage_excluded_from_active_closure");
    for path in [
        "scripts/snapshots/translation-prompt-v4/request-response.expected.json",
        "docs/config-catalog.zh.md",
        "packages/client/ui-primitives/src/icons/index.tsx",
    ] {
        for chained in [false, true] {
            let policy = if chained {
                "immediate-predecessor"
            } else {
                "retained-FULL-anchor"
            };
            let mut records: Vec<Encoded> = Vec::new();
            let mut prior: Option<usize> = None;
            let mut previous_len = 0;
            let (mut full_bytes, mut delta_bytes, mut encode_ns, mut decode_ns) = (0, 0, 0, 0);
            for row in mapping.lines() {
                let columns = row.split('\t').collect::<Vec<_>>();
                let Some(raw) = original(columns[2], path) else {
                    prior = None;
                    previous_len = 0;
                    continue;
                };
                if raw.is_empty() || raw.len() >= content::SMALL_LIMIT {
                    println!(
                        "ineligible\t{policy}\t{path}\t{}\t{}\tbytes={}\tCDC-or-empty",
                        columns[0],
                        columns[1],
                        raw.len()
                    );
                    prior = None;
                    previous_len = raw.len();
                    continue;
                }
                let canonical = content::encode_small(&raw).unwrap();
                assert_eq!(canonical.len(), raw.len() + 23);
                let id = ObjectId::for_bytes(&canonical);
                if let Some(index) = records.iter().position(|r| r.id == id) {
                    prior = Some(index);
                    previous_len = raw.len();
                    println!(
                        "CAS\t{policy}\t{path}\t{}\t{}\t{id:?}",
                        columns[0], columns[1]
                    );
                    continue;
                }
                let candidate = prior.map(|i| {
                    if chained {
                        i
                    } else {
                        records[i].base.unwrap_or(i)
                    }
                });
                let eligible = candidate.filter(|&i| {
                    records[i].depth < 8 && records[i].closure + canonical.len() <= 512 * 1024
                });
                let start = Instant::now();
                let prefix = eligible.map(|i| reconstruct(&records, i));
                let base_decode_ns = start.elapsed().as_nanos();
                decode_ns += base_decode_ns;
                let start = Instant::now();
                let mut encoder = pack::NativeEncoder::new_small().unwrap();
                let full = encoder.compress(&raw, None).unwrap();
                let full_len = full.len();
                let alternative = prefix
                    .as_deref()
                    .map(|base| encoder.compress(&raw, Some(base)).unwrap());
                drop(encoder);
                let encoding = start.elapsed().as_nanos();
                encode_ns += encoding;
                if let Some(frame) = &alternative {
                    assert_eq!(
                        pack::small_decompress(frame, raw.len(), prefix.as_deref()).unwrap(),
                        raw
                    );
                }
                assert_eq!(pack::small_decompress(&full, raw.len(), None).unwrap(), raw);
                let delta_len = alternative.as_ref().map_or(0, Vec::len);
                let selected = eligible.filter(|&i| {
                    delta_len + 32 < full_len
                        && records[i].encoded_closure + delta_len + 41 <= 256 * 1024
                });
                let (frame, depth, closure, encoded_closure) = if let Some(i) = selected {
                    let frame = alternative.unwrap();
                    delta_bytes += frame.len() + 41 + 16;
                    let size = frame.len() + 41 + records[i].encoded_closure;
                    (
                        frame,
                        records[i].depth + 1,
                        records[i].closure + canonical.len(),
                        size,
                    )
                } else {
                    full_bytes += full.len() + 9 + 16;
                    let size = full.len() + 9;
                    (full, 0, canonical.len(), size)
                };
                records.push(Encoded {
                    id,
                    raw_len: raw.len(),
                    frame: frame.into_boxed_slice(),
                    base: selected,
                    depth,
                    closure,
                    encoded_closure,
                });
                let index = records.len() - 1;
                let start = Instant::now();
                assert_eq!(reconstruct(&records, index), raw);
                let verify_ns = start.elapsed().as_nanos();
                decode_ns += verify_ns;
                println!("record\t{policy}\t{path}\tstep={}\tindex={}\tcommit={}\tid={id:?}\traw={}\tprevious_raw={previous_len}\tfull_frame={full_len}\tdelta_frame={delta_len}\tselected_delta={}\tdepth={depth}\tclosure={closure}\tencoded_capacity={encoded_closure}\tencode_ns={encoding}\tbase_decode_ns={base_decode_ns}\tverify_ns={verify_ns}", columns[0], columns[1], columns[2], raw.len(), selected.is_some());
                prior = Some(index);
                previous_len = raw.len();
            }
            println!("total\t{policy}\t{path}\tdistinct_objects={}\tFULL_records_and_directory={full_bytes}\tDELTA_records_and_directory={delta_bytes}\tretained_total={}\tencode_ns={encode_ns}\tdecode_ns={decode_ns}", records.len(), full_bytes + delta_bytes);
        }
    }
}
