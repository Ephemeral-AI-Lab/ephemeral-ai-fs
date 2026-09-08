//! Offline inventory only. Reuses the product's exact wire decoder by source inclusion.
#![allow(dead_code)]
use layerfs_content::file::{extent::ExtentNodeV3, extent_codec::*};
use layerfs_content::tree::{directory::codec::*, inode::codec::*, metadata::codec::*};
use layerfs_content::{ObjectId, decode_bytes_object, identify_canonical};
use layerfs_layerstack_store::{Result, StoreError};
mod telemetry {
    pub use layerfs_layerstack_store::PhysicalStorageReceipt;
}
fn elapsed_ns(t: std::time::Instant) -> u64 {
    t.elapsed().as_nanos().try_into().unwrap()
}
#[rustfmt::skip]
#[path = "../../../../../../../crates/layerfs-layerstack-store/src/objects/pack.rs"]
mod pack;
use rusqlite::{Connection, params};
use std::collections::{BTreeMap, VecDeque};
fn role(c: &[u8]) -> std::result::Result<&'static str, layerfs_content::CoreError> {
    if c[4] == layerfs_content::ObjectKind::Directory as u8 {
        layerfs_content::decode_object(c)?;
        return Ok("other_supported_encoding");
    }
    let v = decode_bytes_object(c)?;
    Ok(match v.get(..8).unwrap_or_default() {
        b"LFS4CHK\0" => {
            decode_chunk_payload(v)?;
            "payload_chunk"
        }
        b"LFS4FSR\0" => {
            decode_namespace_root(c)?;
            "namespace_root"
        }
        b"LFS4INO\0" => {
            decode_inode_record(c)?;
            "inode_record"
        }
        b"LFS4DIR\0" => {
            decode_directory_state(c)?;
            "directory_state"
        }
        b"LFS4LNK\0" => {
            decode_symlink(c)?;
            "symlink_state"
        }
        b"LFS4INT\0" => match decode_inode_table_node(c)? {
            InodeTableNodeV1::Leaf(_) => "inode_table_leaf",
            _ => "inode_table_branch",
        },
        b"LFS4NSP\0" => match decode_directory_node(c)? {
            DirectoryNodeV1::Leaf { .. } => "directory_map_leaf",
            _ => "directory_map_branch",
        },
        b"LFS4MET\0" => match decode_metadata_node(c)? {
            MetadataNodeV1::Leaf { .. } => "metadata_map_leaf",
            _ => "metadata_map_branch",
        },
        b"LFS4MAP\0" => {
            if decode_file_state(c).is_ok() {
                "FileState"
            } else {
                match decode_node(c)? {
                    ExtentNodeV3::Leaf { .. } => "extent_leaf",
                    _ => "extent_branch",
                }
            }
        }
        _ => "unknown",
    })
}
struct Cache {
    groups: BTreeMap<(i64, usize), (pack::Version, Vec<u8>)>,
    order: VecDeque<(i64, usize)>,
    bytes: usize,
}
impl Cache {
    fn get(&mut self, db: &Connection, p: i64, g: usize) -> (pack::Version, Vec<u8>) {
        if let Some(v) = self.groups.get(&(p, g)) {
            return v.clone();
        }
        let b: Vec<u8> = db
            .query_row("select data from object_packs where pack_id=? and length(data)<=?", [p,(pack::CANONICAL_LIMIT+41) as i64], |r| {
                r.get(0)
            })
            .unwrap();
        let h = pack::versioned_header(b[..16].try_into().unwrap(), b.len()).unwrap();
        assert!(g < h.group_count);
        let e = pack::versioned_entry(b[16 + 16 * g..32 + 16 * g].try_into().unwrap(), h, b.len()).unwrap();
        let v = pack::decode_group(e.clone(), b[e.range.clone()].to_vec()).unwrap();
        while self.bytes + v.len() > 32 * 1024 * 1024 {
            let k = self.order.pop_front().unwrap();
            self.bytes -= self.groups.remove(&k).unwrap().1.len();
        }
        self.bytes += v.len();
        self.order.push_back((p, g));
        self.groups.insert((p, g), (h.version, v.clone()));
        (h.version, v)
    }
    fn base(&mut self, db: &Connection, id: ObjectId) -> (Vec<u8>, i64, usize) {
        let (p,g,r,l):(i64,usize,usize,usize)=db.query_row("select pack_id,group_number,record_number,canonical_length from objects where object_id=?",[id.as_bytes().as_slice()],|x|Ok((x.get(0)?,x.get::<_,i64>(1)? as usize,x.get::<_,i64>(2)? as usize,x.get::<_,i64>(3)? as usize))).unwrap();
        let (version, v) = self.get(db, p, g);
        assert_eq!(version, pack::Version::Legacy, "legacy DELTA base must be legacy FULL");
        let mut base = None;
        pack::visit_records(&v, false, |i, rec| {
            if i == r {
                match rec {
                    pack::Record::Full(c) => base = Some(c.to_vec()),
                    _ => panic!("base is not FULL"),
                }
            }
            Ok(())
        })
        .unwrap();
        let c = base.unwrap();
        assert_eq!(c.len(), l);
        layerfs_content::authenticate_identity(&c, id).unwrap();
        (c, p, g)
    }
    // Census glue only: product pack parsers, static decoder and canonical codec
    // own byte interpretation. At most four PREFIX edges, never general reads.
    fn native(&mut self, db: &Connection, target_pack: i64, rec: pack::NativeRecord<'_>, edges: usize, raw_sum: usize)
        -> (Vec<u8>, usize, usize, Option<(ObjectId, i64, usize, usize, &'static str)>) {
        let (raw_length, frame, base) = match rec {
            pack::NativeRecord::Full { raw_length, frame } => (raw_length, frame, None),
            pack::NativeRecord::Prefix { raw_length, base, frame } => (raw_length, frame, Some(base)),
        };
        let raw_sum = raw_sum.checked_add(raw_length).unwrap();
        assert!(raw_sum <= 1_048_576);
        let mut closure_edges = edges;
        let mut closure_raw = raw_sum;
        let mut provenance = None;
        let prefix = base.map(|id| {
            assert!(edges < 4, "native PREFIX depth exceeds four");
            let (p,g,r,l):(i64,i64,i64,i64)=db.query_row("select pack_id,group_number,record_number,canonical_length from objects where object_id=?",[id.as_bytes().as_slice()],|x|Ok((x.get(0)?,x.get(1)?,x.get(2)?,x.get(3)?))).unwrap();
            assert!(p > 0 && p < target_pack, "native base must be selected in an earlier immutable pack");
            let (g,r,l) = (usize::try_from(g).unwrap(),usize::try_from(r).unwrap(),usize::try_from(l).unwrap());
            let (version, group) = self.get(db,p,g);
            let (canonical, kind) = match version {
                pack::Version::Legacy => {
                    let (c,_,_) = self.base(db,id);
                    let payload = decode_chunk_payload(decode_bytes_object(&c).unwrap()).unwrap();
                    assert!(payload.len() <= pack::NATIVE_RAW_LIMIT);
                    closure_edges = edges + 1;
                    closure_raw = raw_sum.checked_add(payload.len()).unwrap();
                    (c,"FULL")
                }
                pack::Version::Native => {
                    let n = u32::from_le_bytes(group[..4].try_into().unwrap()) as usize;
                    assert!((1..=pack::RECORD_COUNT_LIMIT).contains(&n));
                    let range = pack::native_record_range(n,group.get(4..4+4*n).unwrap(),group.len(),r).unwrap();
                    let parsed = pack::native_record(&group[range]).unwrap();
                    let kind = if matches!(&parsed,pack::NativeRecord::Full {..}) {"NATIVE_FULL"} else {"NATIVE_PREFIX"};
                    let (c,depth,total,_) = self.native(db,p,parsed,edges+1,raw_sum);
                    closure_edges=depth; closure_raw=total;
                    (c,kind)
                }
            };
            assert!(closure_raw <= 1_048_576);
            assert_eq!(canonical.len(),l);
            layerfs_content::authenticate_identity(&canonical,id).unwrap();
            let payload=decode_chunk_payload(decode_bytes_object(&canonical).unwrap()).unwrap().to_vec();
            assert!(payload.len() <= pack::NATIVE_RAW_LIMIT);
            provenance=Some((id,p,g,r,kind));
            payload
        });
        let raw=pack::native_decompress(frame,raw_length,prefix.as_deref()).unwrap();
        let canonical=encode_chunk_object(&raw).unwrap();
        assert_eq!(canonical.len(),raw_length+21);
        (canonical,closure_edges,closure_raw,provenance)
    }

}
fn main() {
    let a: Vec<_> = std::env::args().collect();
    assert_eq!(a.len(), 3, "database output-directory");
    let out = std::path::Path::new(&a[2]);
    let db = Connection::open_with_flags(
        format!("file:{}?mode=ro&immutable=1", a[1]),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    )
    .unwrap();
    assert!(!out.join("roles-inventory.sqlite").exists(), "new inventory only");
    let idx = Connection::open(out.join("roles-inventory.sqlite")).unwrap();
    idx.execute_batch("PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF; PRAGMA cache_size=-16384; PRAGMA temp_store=FILE; CREATE TABLE records(id BLOB,bytes INTEGER,role TEXT,pack INTEGER,grp INTEGER,rec INTEGER,kind TEXT,record_bytes INTEGER,selected INTEGER,base BLOB,base_pack INTEGER,base_group INTEGER); CREATE TABLE edges(parent BLOB,child BLOB,use_label TEXT); CREATE TABLE roots(id BLOB,source TEXT); CREATE TABLE groups(pack INTEGER,grp INTEGER,encoded INTEGER,decoded INTEGER,records INTEGER,full_bytes INTEGER,delta_bytes INTEGER,copy_bytes INTEGER,insert_frame_bytes INTEGER,literal_bytes INTEGER,delta_header_bytes INTEGER,codec TEXT); CREATE TABLE packs(pack INTEGER,bytes INTEGER,groups INTEGER); CREATE TABLE pack_versions(pack INTEGER,version INTEGER); CREATE TABLE native_records(pack INTEGER,grp INTEGER,rec INTEGER,raw_bytes INTEGER,frame_bytes INTEGER,header_bytes INTEGER,prefix_edges INTEGER,closure_raw_bytes INTEGER,base_kind TEXT); CREATE TABLE native_groups(pack INTEGER,grp INTEGER,full_header_bytes INTEGER,full_frame_bytes INTEGER,prefix_header_bytes INTEGER,prefix_frame_bytes INTEGER,canonical_bytes INTEGER); BEGIN;").unwrap();
    for table in ["layers", "commits", "workspace_stages"] {
        let mut q = db.prepare(&format!("select root_id from {table}")).unwrap();
        for r in q.query_map([], |r| r.get::<_, Vec<u8>>(0)).unwrap() {
            idx.execute("insert into roots values(?,?)", params![r.unwrap(), table])
                .unwrap();
        }
    }
    let mut cache = Cache {
        groups: BTreeMap::new(),
        order: VecDeque::new(),
        bytes: 0,
    };
    let mut q = db
        .prepare("select pack_id,length(data),data from object_packs order by pack_id")
        .unwrap();
    let mut rows = q.query([]).unwrap();
    let mut packs = 0;
    while let Some(row) = rows.next().unwrap() {
        let p: i64 = row.get(0).unwrap();
        let length:i64=row.get(1).unwrap();
        assert!((1..=(pack::CANONICAL_LIMIT+41) as i64).contains(&length));
        let b: Vec<u8> = row.get(2).unwrap();
        let h = pack::versioned_header(b[..16].try_into().unwrap(), b.len()).unwrap();
        let n = h.group_count;
        idx.execute("insert into pack_versions values(?,?)",params![p, if h.version == pack::Version::Native {2} else {1}]).unwrap();
        let mut pack_records = 0;
        let mut end = 16 + 16 * n;
        idx.execute(
            "insert into packs values(?,?,?)",
            params![p, b.len() as i64, n as i64],
        )
        .unwrap();
        for g in 0..n {
            let e =
                pack::versioned_entry(b[16 + 16 * g..32 + 16 * g].try_into().unwrap(), h, b.len()).unwrap();
            assert_eq!(e.range.start, end);
            end = e.range.end;
            let v = pack::decode_group(e.clone(), b[e.range.clone()].to_vec()).unwrap();
            let mut stats = [0usize; 7];
            let mut native_stats = [0usize; 5];
            let mut num = 0;
            let mut emit = |r:usize,c:Vec<u8>,kind:&str,base:Option<Vec<u8>>,bp:Option<i64>,bg:Option<usize>,record_len:usize| {
 let(id,_)=identify_canonical(&c)?;let role=role(&c)?;
 let loc:Option<(i64,usize,usize,usize)>=db.query_row("select pack_id,group_number,record_number,canonical_length from objects where object_id=?",[id.as_bytes().as_slice()],|x|Ok((x.get(0)?,x.get::<_,i64>(1)? as usize,x.get::<_,i64>(2)? as usize,x.get::<_,i64>(3)? as usize))).optional().unwrap();
 let selected=loc==Some((p,g,r,c.len()));if let Some((lp,lg,lr,ll))=loc {if(lp,lg,lr)==(p,g,r){assert_eq!(ll,c.len());}}
 idx.execute("insert into records values(?,?,?,?,?,?,?,?,?,?,?,?)",params![id.as_bytes().as_slice(),c.len() as i64,role,p,g as i64,r as i64,kind,record_len as i64,selected,base,bp,bg.map(|v|v as i64)]).unwrap();
 if selected {for child in layerfs_content::object::references::referenced_objects(&c)? {let label=if role=="metadata_map_leaf"{"metadata_value"}else if role=="inode_record" {let inode=decode_inode_record(&c)?;if child==inode.metadata_root{"metadata"}else{"content"}}else{"graph"};idx.execute("insert into edges values(?,?,?)",params![id.as_bytes().as_slice(),child.as_bytes().as_slice(),label]).unwrap();}}

                Ok::<(),StoreError>(())
            };
            if h.version == pack::Version::Legacy {
                pack::visit_records(&v,e.oversized,|r,rec| {
                    num+=1;
                    let (c,kind,base,bp,bg,record_len)=match rec {
                        pack::Record::Full(c)=>{stats[0]+=1+c.len();(c.to_vec(),"FULL",None,None,None,1+c.len())},
                        pack::Record::Delta{base,output_length,instructions,count}=>{let (c,bp,bg)=cache.base(&db,base);let target=pack::apply_delta(instructions,count,output_length,&c)?; let len=41+instructions.len();stats[1]+=len;stats[5]+=41;let mut off=0;for _ in 0..count{let op=instructions[off];off+=1;if op==0{stats[2]+=9;off+=8;}else{let l=u32::from_le_bytes(instructions[off..off+4].try_into().unwrap())as usize;stats[3]+=5;stats[4]+=l;off+=4+l;}}assert_eq!(off,instructions.len());(target,"DELTA",Some(base.to_bytes().to_vec()),Some(bp),Some(bg),len)}
                    };
                    emit(r,c,kind,base,bp,bg,record_len)?;
                    Ok(())
                }).unwrap();
            } else {
                num=u32::from_le_bytes(v[..4].try_into().unwrap()) as usize;
                assert!((1..=pack::RECORD_COUNT_LIMIT).contains(&num));
                let directory=v.get(4..4+4*num).unwrap();
                // Validate the entire directory once. Sequential traversal below
                // uses those authenticated boundaries, not an O(N^2) point parser.
                pack::native_record_range(num,directory,v.len(),0).unwrap();
                let area_start=4+directory.len();
                let mut start=area_start;
                for r in 0..num {
                    let end=area_start+u32::from_le_bytes(directory[4*r..4*r+4].try_into().unwrap()) as usize;
                    let rec=pack::native_record(&v[start..end]).unwrap();
                    let (raw_length,frame_length,header_length,kind)=match &rec {
                        pack::NativeRecord::Full{raw_length,frame} => {native_stats[0]+=5; native_stats[1]+=frame.len(); (*raw_length,frame.len(),5,"NATIVE_FULL")},
                        pack::NativeRecord::Prefix{raw_length,frame,..} => {native_stats[2]+=37; native_stats[3]+=frame.len(); (*raw_length,frame.len(),37,"NATIVE_PREFIX")},
                    };
                    let (c,depth,total,dependency)=cache.native(&db,p,rec,0,0);
                    native_stats[4]+=c.len();
                    idx.execute("insert into native_records values(?,?,?,?,?,?,?,?,?)",params![p,g as i64,r as i64,raw_length as i64,frame_length as i64,header_length,depth as i64,total as i64,dependency.map(|v|v.4)]).unwrap();
                    let base=dependency.map(|v|v.0.as_bytes().to_vec());
                    emit(r,c,kind,base,dependency.map(|v|v.1),dependency.map(|v|v.2),end-start).unwrap();
                    start=end;
                }
                idx.execute("insert into native_groups values(?,?,?,?,?,?,?)",params![p,g as i64,native_stats[0] as i64,native_stats[1] as i64,native_stats[2] as i64,native_stats[3] as i64,native_stats[4] as i64]).unwrap();
            }
            assert_eq!(v.len(),4+4*num+stats[0]+stats[1]+native_stats[..4].iter().sum::<usize>());
            assert_eq!(stats[1],stats[2]+stats[3]+stats[4]+stats[5]);
            pack_records+=num;
            idx.execute(
                "insert into groups values(?,?,?,?,?,?,?,?,?,?,?,?)",
                params![
                    p,
                    g as i64,
                    e.range.len() as i64,
                    v.len() as i64,
                    num as i64,
                    stats[0] as i64,
                    stats[1] as i64,
                    stats[2] as i64,
                    stats[3] as i64,
                    stats[4] as i64,
                    stats[5] as i64,
                    format!("{:?}", e.codec)
                ],
            )
            .unwrap();
        }
        assert_eq!(end, b.len());
        assert!(pack_records <= pack::RECORD_COUNT_LIMIT);
        packs += 1;
        if packs % 500 == 0 {
            eprintln!("packs {packs}");
        }
    }
    idx.execute_batch("COMMIT; CREATE INDEX records_id ON records(id); CREATE INDEX records_base ON records(base); CREATE INDEX record_location ON records(pack,grp,rec); CREATE INDEX edges_parent ON edges(parent); CREATE INDEX edges_child ON edges(child);").unwrap();
    let selected: i64 = idx
        .query_row("select count(*) from records where selected=1", [], |r| {
            r.get(0)
        })
        .unwrap();
    let count: i64 = db
        .query_row("select count(*) from objects", [], |r| r.get(0))
        .unwrap();
    assert_eq!(selected, count);
    let canonical_bytes:i64=idx.query_row("select coalesce(sum(bytes),0) from records where selected=1",[],|r|r.get(0)).unwrap();
    let source_bytes:i64=db.query_row("select coalesce(sum(canonical_length),0) from objects",[],|r|r.get(0)).unwrap();
    assert_eq!(canonical_bytes,source_bytes);
    // These are spillable analysis sets, not a product index. UNION visits each
    // graph node once, and dependency closure includes native intermediate PREFIX
    // objects as well as final FULL anchors; dropping intermediates is invalid.
    idx.execute_batch("CREATE UNIQUE INDEX selected_id ON records(id) WHERE selected=1;
      CREATE TABLE logical_retained(id BLOB PRIMARY KEY) WITHOUT ROWID;
      INSERT INTO logical_retained WITH RECURSIVE walk(id) AS (SELECT id FROM roots UNION SELECT e.child FROM edges e JOIN walk w ON e.parent=w.id) SELECT id FROM walk;
      CREATE TABLE required_objects(id BLOB PRIMARY KEY) WITHOUT ROWID;
      INSERT INTO required_objects WITH RECURSIVE walk(id) AS (SELECT id FROM logical_retained UNION SELECT r.base FROM records r JOIN walk w ON r.id=w.id WHERE r.selected=1 AND r.base IS NOT NULL) SELECT id FROM walk;
      CREATE VIEW physical_base_only AS SELECT r.* FROM records r JOIN required_objects q ON r.id=q.id LEFT JOIN logical_retained l ON r.id=l.id WHERE r.selected=1 AND l.id IS NULL;
      CREATE VIEW selected_outside_required AS SELECT r.* FROM records r LEFT JOIN required_objects q ON r.id=q.id WHERE r.selected=1 AND q.id IS NULL;
      CREATE VIEW physical_unselected AS SELECT * FROM records WHERE selected=0;
      CREATE VIEW dependency_edges AS SELECT r.id,r.pack,r.grp,r.rec,r.selected,r.kind,r.base,b.pack AS base_pack,b.grp AS base_group,b.rec AS base_record,b.kind AS base_kind,b.role AS base_role,b.bytes AS base_canonical_bytes FROM records r JOIN records b ON r.base=b.id AND b.selected=1;
      CREATE VIEW dependency_fan_in AS SELECT base,base_kind,base_role,count(*) AS physical_references,sum(selected) AS selected_references FROM dependency_edges GROUP BY base,base_kind,base_role;
      CREATE TABLE file_content_objects(id BLOB PRIMARY KEY) WITHOUT ROWID;
      INSERT INTO file_content_objects WITH RECURSIVE walk(id) AS (SELECT e.child FROM edges e JOIN records p ON p.id=e.parent AND p.selected=1 AND p.role='inode_record' JOIN records c ON c.id=e.child AND c.selected=1 AND c.role='FileState' WHERE e.use_label='content' UNION SELECT e.child FROM edges e JOIN walk w ON e.parent=w.id) SELECT id FROM walk;
      CREATE VIEW file_payload AS SELECT r.* FROM records r JOIN file_content_objects f ON r.id=f.id WHERE r.selected=1 AND r.role='payload_chunk';").unwrap();
    let missing:i64=idx.query_row("select count(*) from required_objects q left join records r on q.id=r.id and r.selected=1 where r.id is null",[],|r|r.get(0)).unwrap();
    assert_eq!(missing,0,"retained/dependency closure must have selected locators");
    let bad:i64=idx.query_row("select count(*) from native_records n join records r using(pack,grp,rec) where r.role!='payload_chunk' or n.prefix_edges>4 or n.closure_raw_bytes>1048576",[],|r|r.get(0)).unwrap();
    assert_eq!(bad,0);
    eprintln!("authenticated {selected} selected objects across {packs} packs");
}
use rusqlite::OptionalExtension;
#[test]
fn roles_do_not_inspect_user_prefix() {
    let c = encode_chunk_object(b"LFS4INO\0not an inode").unwrap();
    assert_eq!(role(&c).unwrap(), "payload_chunk");
    assert!(pack::header(&[0; 16], 32).is_err());
}

#[test]
fn native_inventory_reconstructs_selected_prior_and_counts_closure() {
    // In-memory analysis fixture only; run under the same serialized test gate.
    let db=Connection::open_in_memory().unwrap();
    db.execute_batch("CREATE TABLE object_packs(pack_id INTEGER PRIMARY KEY,data BLOB); CREATE TABLE objects(object_id BLOB PRIMARY KEY,pack_id INTEGER,group_number INTEGER,record_number INTEGER,canonical_length INTEGER);").unwrap();
    let raw=b"bounded canonical native census base".to_vec();
    let canonical=encode_chunk_object(&raw).unwrap();
    let id=ObjectId::for_bytes(&canonical);
    let full_frame=pack::native_compress(&raw,None).unwrap();
    let full=pack::native_encode_record(raw.len(),None,&full_frame).unwrap();
    let blob=pack::assemble_native(&[pack::native_group(&[&full]).unwrap()]).unwrap();
    db.execute("insert into object_packs values(1,?)",[blob]).unwrap();
    db.execute("insert into objects values(?,1,0,0,?)",params![id.as_bytes().as_slice(),canonical.len() as i64]).unwrap();
    let target=b"bounded canonical native census target".to_vec();
    let frame=pack::native_compress(&target,Some(&raw)).unwrap();
    let record=pack::native_encode_record(target.len(),Some(id),&frame).unwrap();
    let mut cache=Cache{groups:BTreeMap::new(),order:VecDeque::new(),bytes:0};
    let (result,depth,total,base)=cache.native(&db,2,pack::native_record(&record).unwrap(),0,0);
    assert_eq!(result,encode_chunk_object(&target).unwrap());
    assert_eq!(depth,1);
    assert_eq!(total,raw.len()+target.len());
    assert_eq!(base.unwrap().4,"NATIVE_FULL");
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cache.native(&db,1,pack::native_record(&record).unwrap(),0,0);
    })).is_err(),"same-pack dependency must fail");
}
