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
    groups: BTreeMap<(i64, usize), Vec<u8>>,
    order: VecDeque<(i64, usize)>,
    bytes: usize,
}
impl Cache {
    fn get(&mut self, db: &Connection, p: i64, g: usize) -> Vec<u8> {
        if let Some(v) = self.groups.get(&(p, g)) {
            return v.clone();
        }
        let b: Vec<u8> = db
            .query_row("select data from object_packs where pack_id=?", [p], |r| {
                r.get(0)
            })
            .unwrap();
        let n = pack::header(b[..16].try_into().unwrap(), b.len()).unwrap();
        assert!(g < n);
        let e = pack::entry(b[16 + 16 * g..32 + 16 * g].try_into().unwrap(), n, b.len()).unwrap();
        let v = pack::decode_group(e.clone(), b[e.range.clone()].to_vec()).unwrap();
        while self.bytes + v.len() > 32 * 1024 * 1024 {
            let k = self.order.pop_front().unwrap();
            self.bytes -= self.groups.remove(&k).unwrap().len();
        }
        self.bytes += v.len();
        self.order.push_back((p, g));
        self.groups.insert((p, g), v.clone());
        v
    }
    fn base(&mut self, db: &Connection, id: ObjectId) -> (Vec<u8>, i64, usize) {
        let (p,g,r,l):(i64,usize,usize,usize)=db.query_row("select pack_id,group_number,record_number,canonical_length from objects where object_id=?",[id.as_bytes().as_slice()],|x|Ok((x.get(0)?,x.get::<_,i64>(1)? as usize,x.get::<_,i64>(2)? as usize,x.get::<_,i64>(3)? as usize))).unwrap();
        let v = self.get(db, p, g);
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
    let idx = Connection::open(out.join("roles-inventory.sqlite")).unwrap();
    idx.execute_batch("PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF; PRAGMA cache_size=-16384; CREATE TABLE records(id BLOB,bytes INTEGER,role TEXT,pack INTEGER,grp INTEGER,rec INTEGER,kind TEXT,record_bytes INTEGER,selected INTEGER,base BLOB,base_pack INTEGER,base_group INTEGER); CREATE TABLE edges(parent BLOB,child BLOB,use_label TEXT); CREATE TABLE roots(id BLOB,source TEXT); CREATE TABLE groups(pack INTEGER,grp INTEGER,encoded INTEGER,decoded INTEGER,records INTEGER,full_bytes INTEGER,delta_bytes INTEGER,copy_bytes INTEGER,insert_frame_bytes INTEGER,literal_bytes INTEGER,delta_header_bytes INTEGER,codec TEXT); CREATE TABLE packs(pack INTEGER,bytes INTEGER,groups INTEGER); BEGIN;").unwrap();
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
        .prepare("select pack_id,data from object_packs order by pack_id")
        .unwrap();
    let mut rows = q.query([]).unwrap();
    let mut packs = 0;
    while let Some(row) = rows.next().unwrap() {
        let p: i64 = row.get(0).unwrap();
        let b: Vec<u8> = row.get(1).unwrap();
        let n = pack::header(b[..16].try_into().unwrap(), b.len()).unwrap();
        let mut end = 16 + 16 * n;
        idx.execute(
            "insert into packs values(?,?,?)",
            params![p, b.len() as i64, n as i64],
        )
        .unwrap();
        for g in 0..n {
            let e =
                pack::entry(b[16 + 16 * g..32 + 16 * g].try_into().unwrap(), n, b.len()).unwrap();
            assert_eq!(e.range.start, end);
            end = e.range.end;
            let v = pack::decode_group(e.clone(), b[e.range.clone()].to_vec()).unwrap();
            let mut stats = [0usize; 7];
            let mut num = 0;
            pack::visit_records(&v,e.oversized,|r,rec|{
 num+=1;
 let (c,kind,base,bp,bg,record_len)=match rec {
 pack::Record::Full(c)=>{stats[0]+=1+c.len();(c.to_vec(),"FULL",None,None,None,1+c.len())},
 pack::Record::Delta{base,output_length,instructions,count}=>{let (c,bp,bg)=cache.base(&db,base);let target=pack::apply_delta(instructions,count,output_length,&c)?; let len=41+instructions.len();stats[1]+=len;stats[5]+=41;let mut off=0;for _ in 0..count{let op=instructions[off];off+=1;if op==0{stats[2]+=9;off+=8;}else{let l=u32::from_le_bytes(instructions[off..off+4].try_into().unwrap())as usize;stats[3]+=5;stats[4]+=l;off+=4+l;}}assert_eq!(off,instructions.len());(target,"DELTA",Some(base.to_bytes().to_vec()),Some(bp),Some(bg),len)} };
 let(id,_)=identify_canonical(&c)?;let role=role(&c)?;
 let loc:Option<(i64,usize,usize,usize)>=db.query_row("select pack_id,group_number,record_number,canonical_length from objects where object_id=?",[id.as_bytes().as_slice()],|x|Ok((x.get(0)?,x.get::<_,i64>(1)? as usize,x.get::<_,i64>(2)? as usize,x.get::<_,i64>(3)? as usize))).optional().unwrap();
 let selected=loc==Some((p,g,r,c.len()));if let Some((lp,lg,lr,ll))=loc {if(lp,lg,lr)==(p,g,r){assert_eq!(ll,c.len());}}
 idx.execute("insert into records values(?,?,?,?,?,?,?,?,?,?,?,?)",params![id.as_bytes().as_slice(),c.len() as i64,role,p,g as i64,r as i64,kind,record_len as i64,selected,base,bp,bg.map(|v|v as i64)]).unwrap();
 if selected {for child in layerfs_content::object::references::referenced_objects(&c)? {let label=if role=="metadata_map_leaf"{"metadata_value"}else if role=="inode_record" {let inode=decode_inode_record(&c)?;if child==inode.metadata_root{"metadata"}else{"content"}}else{"graph"};idx.execute("insert into edges values(?,?,?)",params![id.as_bytes().as_slice(),child.as_bytes().as_slice(),label]).unwrap();}}
 Ok(())}).unwrap();
            assert_eq!(v.len(), 4 + 4 * num + stats[0] + stats[1]);
            assert_eq!(stats[1], stats[2] + stats[3] + stats[4] + stats[5]);
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
    eprintln!("authenticated {selected} selected objects across {packs} packs");
}
use rusqlite::OptionalExtension;
#[test]
fn roles_do_not_inspect_user_prefix() {
    let c = encode_chunk_object(b"LFS4INO\0not an inode").unwrap();
    assert_eq!(role(&c).unwrap(), "payload_chunk");
    assert!(pack::header(&[0; 16], 32).is_err());
}
