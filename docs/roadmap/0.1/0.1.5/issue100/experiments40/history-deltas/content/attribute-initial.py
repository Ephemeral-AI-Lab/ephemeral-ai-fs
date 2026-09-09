"""All current75398SmallContent identities versus fixed full157Git inventory."""
import sys
sys.dont_write_bytecode=True
import collections,functools,hashlib,json,struct,time
from pathlib import Path
ROOT=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157');OUT=Path(__file__).resolve().parent;STORE=ROOT/'combined/D-B-CDC.sqlite';GIT_ROWS=ROOT/'git-baseline/verify-pack.txt'
sys.path.insert(0,str(ROOT/'combined'))
from store_api import StoreReader,fr,sha
started=time.perf_counter_ns();before=sha(STORE);gitseal=sha(GIT_ROWS);assert before=='20bc3581f29ecab3168712b8972b2eb24898f92d98e94d402657e953048efa6b'.replace('2b2eb','2eb')
identity=json.loads(Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1/identity.json').read_text());states=identity['fixtures']['deepseek-full']['states']
first={};facts={};sizes={};modes={};paths={};prev={};manifestseals={}
for s in states:
 step=s['index'];manifest=Path(s['input'])/'manifest.tsv';assert sha(manifest)==s['input_seal']['manifest.tsv'];manifestseals[str(step)]=sha(manifest);current={}
 for line in manifest.read_text().splitlines():
  mode,oid,size,path=line.split('\t');size=int(size);current[path]=(oid,size,mode);sizes[oid]=size;modes.setdefault(oid,mode)
  if oid not in first:first[oid]=step;facts[oid]=[];paths[oid]=bytes.fromhex(path).decode('utf8','backslashreplace')
  if first[oid]==step:
   prior=prev.get(path);fact=dict(prior_git_oid=prior[0],prior_raw_bytes=prior[1],prior_mode=prior[2]) if prior else dict(prior_git_oid=None)
   if fact not in facts[oid]:facts[oid].append(fact)
 prev=current
 del s['input_seal']
del identity,states,prev,current
git={}
for line in GIT_ROWS.read_text().splitlines():
 f=line.split()
 if len(f) in (5,7) and len(f[0])==40 and f[1]=='blob':git[f[0]]=dict(entry_bytes=int(f[3]),depth=int(f[5]) if len(f)==7 else 0,base=f[6] if len(f)==7 else None)
assert len(git)==75929 and set(git)==set(first)
@functools.lru_cache(None)
def available(oid):return max(first[oid],available(git[oid]['base']) if git[oid]['base'] else 0)
reader=StoreReader(STORE,cache_bytes=4*1024*1024,pack_cache_bytes=2*1024*1024)
small_ids=[];packs=[]
for p,b in reader.db.execute('select pack_id,data from object_packs order by pack_id'):
 if b[:8]==fr.MAGIC:
  count=fr.u32(b,12);assert fr.u32(b,8)==102;packs.append(dict(pack=p,bytes=len(b),objects=count,directory_bytes=4*count,header_bytes=16))
small_packs={r['pack'] for r in packs}
small_ids=sorted((id for id,loc in reader.loc.items() if loc[0] in small_packs),key=lambda id:reader.loc[id][:3])
assert len(small_ids)==75398
rows=[];seen=set();groups=collections.defaultdict(collections.Counter);physical={};git_to_id={}
for id in small_ids:
 v,record,_=reader._physical(id);assert v==3;kind,raw,base,frame=fr.record(record);canonical=reader.read_canonical(id);data=canonical[23:];assert len(data)==raw
 oid=hashlib.sha1(b'blob '+str(raw).encode()+b'\0'+data).hexdigest();assert oid in git and oid not in seen;seen.add(oid);git_to_id[oid]=id.hex()
 g=git[oid];gc='Git_FULL' if not g['base'] else 'Git_DELTA_future_closure' if available(g['base'])>first[oid] else 'Git_DELTA_same_closure' if available(g['base'])==first[oid] else 'Git_DELTA_earlier_closure';lc='LFS_FULL' if kind==0 else 'LFS_DELTA'
 actualrecord=len(record)-8
 groups[lc+' / '+gc].update(objects=1,frame_bytes=len(frame),compact_record_bytes=actualrecord,compact_record_directory_bytes=actualrecord+4,git_entry_bytes=g['entry_bytes'])
 r=dict(id=id.hex(),git_oid=oid,kind=kind,base=base.hex() if base else None,raw_bytes=raw,frame_bytes=len(frame),record_bytes=actualrecord,legacy_record_bytes=len(record),git_entry_bytes=g['entry_bytes'],git_category=gc,git_depth=g['depth'],first_step=first[oid],path=paths[oid],prior_facts=facts[oid],pack=reader.loc[id][0]);rows.append(r);physical[r['id']]=r
 assert reader.cache_bytes<=4*1024*1024 and reader.pack_bytes<=2*1024*1024
reader.close()
@functools.lru_cache(None)
def closure(id):
 r=physical[id]
 if not r['base']:return (0,r['raw_bytes']+23,r['legacy_record_bytes'])
 d,c,e=closure(r['base']);assert physical[r['base']]['pack']<=r['pack'];return(d+1,c+r['raw_bytes']+23,e+r['legacy_record_bytes'])
full_facts=collections.defaultdict(collections.Counter)
for r in rows:
 depth,canonical,encoded=closure(r['id']);assert depth<=8 and canonical<=524288 and encoded<=262144;r['depth']=depth;r['canonical_closure']=canonical;r['encoded_closure']=encoded
 if r['kind']:continue
 fs=r['prior_facts'];classes=set();details=[]
 for f in fs:
  prior=f['prior_git_oid']
  if prior is None:cat='initial_no_path' if r['first_step']==1 else 'new_path'
  elif f['prior_mode'] not in ('100644','100755'):cat='prior_nonregular'
  elif f['prior_raw_bytes']==0:cat='prior_empty'
  elif prior not in git_to_id:cat='prior_not_SmallContent'
  else:
   pd,pc,pe=closure(git_to_id[prior]);limits=[]
   if pd+1>8:limits.append('depth')
   if pc+r['raw_bytes']+23>524288:limits.append('canonical_bytes')
   if pe+41>=262144:limits.append('encoded_bytes_necessary')
   cat='prior_small_structurally_capped' if limits else 'prior_small_within_structural_caps'
   details.append(dict(prior_id=git_to_id[prior],depth=pd,canonical_closure=pc,encoded_closure=pe,limits=limits))
  classes.add(cat)
 label=next(iter(classes)) if len(classes)==1 else 'mixed_prior_path_facts';full_facts[label].update(objects=1,frame_bytes=r['frame_bytes'],record_bytes=r['record_bytes']);r['full_path_class']=label;r['prior_small_checks']=details
remaining=set(git)-seen;outside=collections.defaultdict(collections.Counter);remainingrows=[]
for oid in sorted(remaining):
 cat='symlink' if modes[oid]=='120000' else 'empty' if sizes[oid]==0 else 'large_regular' if sizes[oid]>=131072 else 'unexpected'
 assert cat!='unexpected';outside[cat].update(objects=1,git_entry_bytes=git[oid]['entry_bytes'],raw_bytes=sizes[oid]);remainingrows.append(dict(git_oid=oid,category=cat,raw_bytes=sizes[oid],git_entry_bytes=git[oid]['entry_bytes'],path=paths[oid],first_step=first[oid]))
summary=dict(small_objects=len(rows),frame_bytes=sum(r['frame_bytes'] for r in rows),compact_record_bytes=sum(r['record_bytes'] for r in rows),compact_directory_bytes=sum(p['directory_bytes'] for p in packs),compact_pack_header_bytes=sum(p['header_bytes'] for p in packs),compact_pack_bytes=sum(p['bytes'] for p in packs),git_same_object_entry_bytes=sum(r['git_entry_bytes'] for r in rows),git_all_blob_entry_bytes=sum(g['entry_bytes'] for g in git.values()),max_depth=max(r['depth'] for r in rows))
assert summary['frame_bytes']+summary['compact_record_bytes']-summary['frame_bytes']+summary['compact_directory_bytes']+summary['compact_pack_header_bytes']==summary['compact_pack_bytes']==58979700
assert sha(STORE)==before and sha(GIT_ROWS)==gitseal
result=dict(scope='Current final full157copy all75398SmallContent identity-attributed to SAME Git blobs. Opposing FULL/DELTA assignments reconciled; no encoding/forecast. Priorpath cap facts are necessary structural eligibility diagnostics, not recorded delivered hints or fallback decisions.',store_sha256=before,git_rows_sha256=gitseal,script_sha256=sha(Path(__file__)),protocol_sha256=sha(OUT/'protocol.md'),canonical_cache_limit=4194304,pack_cache_limit=2097152,manifest_seals=manifestseals,summary=summary,groups=dict(groups),full_prior_classes=dict(full_facts),remaining_git_classes=dict(outside),remaining_git_blobs=remainingrows,rows=rows,elapsed_ns=time.perf_counter_ns()-started)
(OUT/'attribution.json').write_text(json.dumps(result,indent=2));print(json.dumps({k:result[k] for k in ['summary','groups','full_prior_classes','remaining_git_classes','elapsed_ns']},indent=2))
