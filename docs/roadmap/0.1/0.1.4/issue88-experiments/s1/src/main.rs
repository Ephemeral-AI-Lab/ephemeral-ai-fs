//! S1: actual offline tree-engine origins; structural records only; no Store writes.
#![allow(dead_code)]
use layerfs_content::{CoreError, CoreResult, ObjectId};
use layerfs_content::object::access::ObjectStore;
use layerfs_content::tree::{batch::inode_table_apply_sorted, inode::{InodeId, InodeTableRoot, InodeTableCounters, visit_inode_table_entries}, directory::codec::decode_namespace_root};
use layerfs_layerstack_store::{Result, StoreError, PhysicalStorageReceipt};
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use std::{collections::BTreeMap, fs::{self,File}, io::{BufWriter,Write}, path::Path, time::Instant};
mod telemetry {pub use layerfs_layerstack_store::PhysicalStorageReceipt;}
fn elapsed_ns(t:Instant)->u64 {t.elapsed().as_nanos().try_into().unwrap()}
#[rustfmt::skip]
#[path="../../../../../../../crates/layerfs-layerstack-store/src/objects/pack.rs"]
mod pack;
const MAX_ENTRIES:usize=65536;
fn ro(p:&str)->Connection {Connection::open_with_flags(format!("file:{p}?mode=ro&immutable=1"),OpenFlags::SQLITE_OPEN_READ_ONLY|OpenFlags::SQLITE_OPEN_URI).unwrap()}
fn id(v:&[u8])->ObjectId {ObjectId::from_bytes(v).unwrap()}
fn bytes(c:&Connection,k:ObjectId)->Vec<u8>{c.query_row("SELECT canonical FROM objects WHERE id=?",[k.as_bytes().as_slice()],|r|r.get(0)).unwrap()}
struct EngineStore<'a>{c:&'a Connection,checkpoint:i64,emissions:u64}
impl ObjectStore for EngineStore<'_>{
 fn get(&self,k:ObjectId)->CoreResult<Vec<u8>>{
  let found:Option<Vec<u8>>=self.c.query_row("SELECT canonical FROM generated WHERE id=? UNION ALL SELECT canonical FROM objects WHERE id=? AND checkpoint<? LIMIT 1",params![k.as_bytes().as_slice(),k.as_bytes().as_slice(),self.checkpoint],|r|r.get(0)).optional().map_err(|_|CoreError::Io)?;
  found.ok_or(CoreError::MissingObject)
 }
 fn put(&mut self,c:&[u8])->CoreResult<ObjectId>{self.put_tree_origin(c.to_vec(),None)}
 fn put_tree_origin(&mut self,c:Vec<u8>,origin:Option<ObjectId>)->CoreResult<ObjectId>{
  let k=layerfs_content::identify_canonical(&c)?.0;
  self.c.execute("INSERT OR IGNORE INTO generated(id,canonical) VALUES(?,?)",params![k.as_bytes().as_slice(),c]).map_err(|_|CoreError::Io)?;
  let retained:Option<(Vec<u8>,i64,String)>=self.c.query_row("SELECT canonical,checkpoint,role FROM objects WHERE id=?",[k.as_bytes().as_slice()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(|_|CoreError::Io)?;
  if let Some((canonical,first,role))=retained{
   if canonical!=c{return Err(CoreError::IdentityMismatch)}
   if first>self.checkpoint{return Err(CoreError::InvalidRecord("future retained node emitted"))}
   if first==self.checkpoint && role=="inode_table_leaf"{
    let prior=origin.map(|x|x.to_bytes().to_vec());
    let existing:Option<Option<Vec<u8>>>=self.c.query_row("SELECT base FROM origins WHERE target=?",[k.as_bytes().as_slice()],|r|r.get(0)).optional().map_err(|_|CoreError::Io)?;
    if existing.is_some_and(|b|b!=prior){return Err(CoreError::InvalidRecord("ambiguous actual origin"))}
    self.c.execute("INSERT OR IGNORE INTO origins(target,base,checkpoint) VALUES(?,?,?)",params![k.as_bytes().as_slice(),prior,self.checkpoint]).map_err(|_|CoreError::Io)?;
   }
  }
  self.emissions+=1;Ok(k)
 }
}
fn extract(original:&Connection,index:&Connection,c:&Connection)->u64{
 let mut groups=BTreeMap::<(i64,usize),Vec<(ObjectId,i64,String)>>::new();
 let mut query=index.prepare("SELECT r.pack,r.grp,r.rec,r.id,l.first_retained_checkpoint,r.role FROM records r JOIN logical l ON l.id=r.id WHERE r.selected=1 AND r.role!='payload_chunk' ORDER BY r.pack,r.grp,r.rec").unwrap();
 let mut rr=query.query([]).unwrap();
 while let Some(r)=rr.next().unwrap(){
  let p:i64=r.get(0).unwrap();let g=usize::try_from(r.get::<_,i64>(1).unwrap()).unwrap();let n=usize::try_from(r.get::<_,i64>(2).unwrap()).unwrap();
  let entry=groups.entry((p,g)).or_default();assert_eq!(entry.len(),n,"structural group contains payload/unselected locator");
  entry.push((id(&r.get::<_,Vec<u8>>(3).unwrap()),r.get(4).unwrap(),r.get(5).unwrap()));
 }
 assert_eq!(groups.len(),9918);
 let mut q=original.prepare("SELECT pack_id,data FROM object_packs ORDER BY pack_id").unwrap();let mut rows=q.query([]).unwrap();
 let mut records=0u64;c.execute_batch("BEGIN").unwrap();
 while let Some(r)=rows.next().unwrap(){
  let p:i64=r.get(0).unwrap();let data:Vec<u8>=r.get(1).unwrap();assert!(data.len()<=pack::PACK_LIMIT);
  let n=pack::header(data[..16].try_into().unwrap(),data.len()).unwrap();let mut end=16+16*n;
  for g in 0..n{
   let e=pack::entry(data[16+16*g..32+16*g].try_into().unwrap(),n,data.len()).unwrap();assert_eq!(e.range.start,end);end=e.range.end;
   let Some(expected)=groups.remove(&(p,g)) else{continue};
   let decoded=pack::decode_group(e.clone(),data[e.range.clone()].to_vec()).unwrap();
   let cp=expected[0].1;assert!(expected.iter().all(|x|x.1==cp));
   c.execute("INSERT INTO groups VALUES(?,?,?,?,?,?)",params![p,i64::try_from(g).unwrap(),cp,i64::try_from(e.range.len()).unwrap(),i64::try_from(decoded.len()).unwrap(),blake3::hash(&data[e.range.clone()]).to_hex().to_string()]).unwrap();
   let mut seen=0;
   pack::visit_records(&decoded,false,|record,body|{
    let pack::Record::Full(canonical)=body else{panic!("structural baseline must be FULL")};
    let (k,checkpoint,role)=&expected[record];layerfs_content::authenticate_identity(canonical,*k)?;
    assert!(canonical.len()<=8192);c.execute("INSERT INTO objects VALUES(?,?,?,?,?,?,?)",params![k.as_bytes().as_slice(),canonical,role,p,i64::try_from(g).unwrap(),i64::try_from(record).unwrap(),checkpoint]).unwrap();seen+=1;Ok(())
   }).unwrap();assert_eq!(seen,expected.len());records+=seen as u64;
  }
  assert_eq!(end,data.len());
 }
 assert!(groups.is_empty());c.execute_batch("COMMIT").unwrap();assert_eq!(records,279724);records
}
fn reconstruct(c:&Connection,out:&Path)->u64{
 let mut roots=BTreeMap::new();let mut q=c.prepare("SELECT checkpoint,canonical FROM objects WHERE role='namespace_root' ORDER BY checkpoint").unwrap();
 for r in q.query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,Vec<u8>>(1)?))).unwrap(){let(cp,canonical)=r.unwrap();let root=decode_namespace_root(&canonical).unwrap();assert!(roots.insert(cp,root.inode_table_root).is_none());}
 assert_eq!(roots.len(),158);
 let mut writer=BufWriter::new(File::create_new(out.join("tree-reconstruction.csv")).unwrap());writeln!(writer,"checkpoint,old_entries_count,new_entries_count,delta_bindings_count,engine_emissions_count,roots_identical,origin_provenance").unwrap();
 for cp in 1..=157i64{
  // Target enumeration is observer input; engine reads may use only prior state
  // or nodes it emitted itself, never arbitrary future-retained nodes.
  struct All<'a>(&'a Connection);impl ObjectStore for All<'_>{fn get(&self,k:ObjectId)->CoreResult<Vec<u8>>{Ok(bytes(self.0,k))}fn put(&mut self,_:&[u8])->CoreResult<ObjectId>{panic!("readonly")}}
  let all=All(c);let collect=|root|{let mut entries=Vec::new();visit_inode_table_entries(&all,InodeTableRoot(root),&mut InodeTableCounters::default(),|page|{assert!(entries.len()+page.len()<=MAX_ENTRIES);entries.extend_from_slice(page);Ok(())}).unwrap();entries};let old=collect(roots[&(cp-1)]);let new=collect(roots[&cp]);
  assert!(old.len()<=MAX_ENTRIES && new.len()<=MAX_ENTRIES);
  let mut changes=Vec::new();let(mut i,mut j)=(0,0);
  while i<old.len() || j<new.len(){
   if j==new.len() || (i<old.len() && old[i].0<new[j].0){changes.push(Ok((old[i].0,None)));i+=1;}
   else if i==old.len() || new[j].0<old[i].0{changes.push(Ok((new[j].0,Some(new[j].1))));j+=1;}
   else{if old[i].1!=new[j].1{changes.push(Ok((new[j].0,Some(new[j].1))))};i+=1;j+=1;}
  }
  let change_count=changes.len();c.execute_batch("DELETE FROM generated; BEGIN").unwrap();
  let mut store=EngineStore{c,checkpoint:cp,emissions:0};
  let (result,_)=inode_table_apply_sorted(&mut store,InodeTableRoot(roots[&(cp-1)]),changes.into_iter()).unwrap();
  assert_eq!(result.0,roots[&cp],"exact tree reconstruction differs at {cp}");
  writeln!(writer,"{cp},{},{},{change_count},{},true,actual_offline_tree_engine_reconstruction",old.len(),new.len(),store.emissions).unwrap();
  c.execute_batch("COMMIT").unwrap();
 }
 writer.flush().unwrap();c.execute_batch("DELETE FROM generated").unwrap();157
}
fn encode(c:&Connection,out:&Path){
 let mut writer=BufWriter::new(File::create_new(out.join("groups.csv")).unwrap());
 writeln!(writer,"checkpoint,pack_id,group_number,records_count,A_encoded_bytes,B_encoded_bytes,selected_encoded_bytes,selected_DELTA_count,origin_present_count,eligible_origin_count,origin_missing_count,base_not_prior_count,base_candidate_not_FULL_count,raw_no_candidate_count,mixed_selected,decoded_target_bytes").unwrap();
 let mut frames=BufWriter::new(File::create_new(out.join("candidate-groups.bin")).unwrap());
 let mut frame_offset=0u64;let mut frame_index=BufWriter::new(File::create_new(out.join("candidate-frames.csv")).unwrap());writeln!(frame_index,"pack_id,group_number,frame_offset_bytes,frame_length_bytes,frame_blake3").unwrap();
 let mut sums=[0u64;13];let mut stats=PhysicalStorageReceipt::default();
 let mut groups=c.prepare("SELECT pack,grp,checkpoint,original_encoded,original_blake3 FROM groups ORDER BY checkpoint,pack,grp").unwrap();let mut rows=groups.query([]).unwrap();
 let mut last_pack=None;let mut remaining=16*1024*1024usize;let mut trials=0usize;
 while let Some(row)=rows.next().unwrap(){
  let p:i64=row.get(0).unwrap();let g:i64=row.get(1).unwrap();let cp:i64=row.get(2).unwrap();let original:i64=row.get(3).unwrap();let original_hash:String=row.get(4).unwrap();
  if last_pack!=Some(p){remaining=16*1024*1024;trials=0;last_pack=Some(p)}
  let mut rq=c.prepare("SELECT id,canonical,role FROM objects WHERE pack=? AND grp=? ORDER BY rec").unwrap();
  let objects=rq.query_map(params![p,g],|r|Ok((id(&r.get::<_,Vec<u8>>(0)?),r.get::<_,Vec<u8>>(1)?,r.get::<_,String>(2)?))).unwrap().map(|r|r.unwrap()).collect::<Vec<_>>();
  let mut deltas=Vec::new();let mut local=[0u64;6];
  for(k,canonical,role) in &objects{
   let mut delta=None;
   if role=="inode_table_leaf" && cp>0{
    let origin:Option<Vec<u8>>=c.query_row("SELECT base FROM origins WHERE target=?",[k.as_bytes().as_slice()],|r|r.get(0)).optional().unwrap().flatten();
    if let Some(origin)=origin{
     local[0]+=1;let base=id(&origin);
     let before:Option<i64>=c.query_row("SELECT checkpoint FROM objects WHERE id=? AND role='inode_table_leaf'",[base.as_bytes().as_slice()],|r|r.get(0)).optional().unwrap();
     if before.is_none_or(|before|before>=cp){local[3]+=1;}
     else if c.query_row("SELECT count(*) FROM candidate_full WHERE id=?",[base.as_bytes().as_slice()],|r|r.get::<_,i64>(0)).unwrap()==0{local[4]+=1;}
     else{local[1]+=1;if trials<512 && remaining>0{trials+=1;let b=bytes(c,base);delta=pack::delta_record(base,&b,canonical,&mut remaining,&mut stats).unwrap();}else{stats.match_budget_skips+=1;}
      if delta.is_none(){local[5]+=1;}
     }
    }else{local[2]+=1;}
   }
   deltas.push(delta);
  }
  let refs=objects.iter().map(|o|o.1.as_slice()).collect::<Vec<_>>();let mut control_stats=PhysicalStorageReceipt::default();
  let(control,_)=pack::encode_group(&refs,&[],&mut control_stats).unwrap();assert_eq!(control.bytes.len(),original as usize,"control codec mismatch");assert_eq!(blake3::hash(&control.bytes).to_hex().as_str(),original_hash,"control bytes differ");
  let mut trial_stats=PhysicalStorageReceipt::default();let(selected,mixed)=pack::encode_group(&refs,&deltas,&mut trial_stats).unwrap();
  let encoded=selected.bytes.len();frames.write_all(&selected.bytes).unwrap();writeln!(frame_index,"{p},{g},{frame_offset},{encoded},{}",blake3::hash(&selected.bytes).to_hex()).unwrap();frame_offset+=encoded as u64;let decoded=pack::decode_group(pack::GroupEntry{range:0..encoded,decoded_length:selected.decoded_length,codec:selected.codec,oversized:false},selected.bytes).unwrap();
  let mut admitted=0;pack::visit_records(&decoded,false,|record,body|{
   let(k,canonical,_)=&objects[record];let reconstructed=match body{pack::Record::Full(v)=>{c.execute("INSERT INTO candidate_full VALUES(?)",[k.as_bytes().as_slice()]).unwrap();v.to_vec()},pack::Record::Delta{base,output_length,instructions,count}=>{
    assert!(mixed);assert!(c.query_row("SELECT count(*) FROM candidate_full WHERE id=?",[base.as_bytes().as_slice()],|r|r.get::<_,i64>(0)).unwrap()>0);
    let before:i64=c.query_row("SELECT checkpoint FROM objects WHERE id=?",[base.as_bytes().as_slice()],|r|r.get(0)).unwrap();assert!(before<cp);
    admitted+=1;pack::apply_delta(instructions,count,output_length,&bytes(c,base))?
   }};assert_eq!(&reconstructed,canonical);layerfs_content::authenticate_identity(&reconstructed,*k)?;Ok(())
  }).unwrap();
  let b=if deltas.iter().any(Option::is_some){trial_stats.mixed_alternative_bytes.to_string()}else{"null".into()};
  let target_bytes=objects.iter().map(|x|x.1.len() as u64).sum::<u64>();
  writeln!(writer,"{cp},{p},{g},{},{},{b},{encoded},{admitted},{},{},{},{},{},{},{mixed},{target_bytes}",objects.len(),control.bytes.len(),local[0],local[1],local[2],local[3],local[4],local[5]).unwrap();
  sums[0]+=1;sums[1]+=objects.len()as u64;sums[2]+=control.bytes.len()as u64;sums[3]+=encoded as u64;sums[4]+=admitted;sums[5]+=target_bytes;
  for i in 0..6{sums[6+i]+=local[i]};sums[12]+=u64::from(mixed);
 }
 writer.flush().unwrap();frames.flush().unwrap();frame_index.flush().unwrap();assert_eq!(frame_offset,sums[3]);assert_eq!(sums[0],9918);assert_eq!(sums[2],78792537);assert_eq!(sums[5],94362335);
 let mut f=File::create_new(out.join("result.json")).unwrap();writeln!(f,"{{\"status\":\"PASS\",\"scope\":\"offline identical structural groups; no allocation or latency claim\",\"origin_provenance\":\"actual_offline_tree_engine_reconstruction\",\"historical_producer_origin_observed\":false,\"roots_verified_count\":157,\"groups_count\":{},\"authenticated_records_count\":{},\"A_encoded_bytes\":{},\"selected_encoded_bytes\":{},\"encoded_difference_bytes\":{},\"DELTA_selected_count\":{},\"canonical_bytes\":{},\"origin_present_count\":{},\"eligible_origin_count\":{},\"origin_missing_count\":{},\"base_not_prior_count\":{},\"base_candidate_not_FULL_count\":{},\"raw_no_candidate_count\":{},\"mixed_selected_groups_count\":{},\"matcher_budget_skips_count\":{},\"strict_prior_checkpoint\":true,\"candidate_FULL_base_closure\":true,\"max_delta_depth\":1,\"all_selected_records_authenticated\":true,\"allocation_bytes\":null,\"allocation_null_reason\":\"offline group encoding only\",\"cold_read_ns\":null,\"cold_read_null_reason\":\"no public read measurement in offline screen\"}}",sums[0],sums[1],sums[2],sums[3],sums[2]-sums[3],sums[4],sums[5],sums[6],sums[7],sums[8],sums[9],sums[10],sums[11],sums[12],stats.match_budget_skips).unwrap();
}
fn main(){
 let args=std::env::args().collect::<Vec<_>>();assert_eq!(args.len(),4,"original-store prior-analysis-index NEW-output-dir");
 let out=Path::new(&args[3]);fs::create_dir(out).unwrap();
 let original=ro(&args[1]);let index=ro(&args[2]);let c=Connection::open(out.join("structural.sqlite")).unwrap();
 c.execute_batch("PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF; PRAGMA cache_size=-8192; CREATE TABLE objects(id BLOB PRIMARY KEY,canonical BLOB,role TEXT,pack INTEGER,grp INTEGER,rec INTEGER,checkpoint INTEGER) WITHOUT ROWID;CREATE INDEX object_location ON objects(pack,grp,rec);CREATE TABLE groups(pack INTEGER,grp INTEGER,checkpoint INTEGER,original_encoded INTEGER,original_decoded INTEGER,original_blake3 TEXT,PRIMARY KEY(pack,grp));CREATE TABLE origins(target BLOB PRIMARY KEY,base BLOB,checkpoint INTEGER) WITHOUT ROWID;CREATE TABLE generated(id BLOB PRIMARY KEY,canonical BLOB) WITHOUT ROWID;CREATE TABLE candidate_full(id BLOB PRIMARY KEY) WITHOUT ROWID;").unwrap();
 let mut timing=BufWriter::new(File::create_new(out.join("phase-timings.csv")).unwrap());writeln!(timing,"phase,elapsed_ns,scope").unwrap();let t=Instant::now();extract(&original,&index,&c);writeln!(timing,"extraction,{},offline structural extraction authentication and spool writes",elapsed_ns(t)).unwrap();timing.flush().unwrap();eprintln!("S1 structural extraction authenticated");drop(original);drop(index);let t=Instant::now();reconstruct(&c,out);writeln!(timing,"tree_reconstruction,{},offline retained binding difference and exact engine roots",elapsed_ns(t)).unwrap();timing.flush().unwrap();eprintln!("S1 all157 offline roots identical");let t=Instant::now();encode(&c,out);writeln!(timing,"paired_encoding_validation,{},offline A plus candidate A/B plus reconstruction and output writes",elapsed_ns(t)).unwrap();timing.flush().unwrap();println!("S1 PASS: extraction, actual engine roots, chronological candidate FULL closure and paired group reconstruction");
}
