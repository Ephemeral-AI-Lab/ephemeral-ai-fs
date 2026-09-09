"""Lossless diagnostic SmallContent reframing; no product Store writes."""
import ctypes as C, hashlib, json, pathlib, sqlite3, struct, time
ROOT=pathlib.Path(__file__).resolve().parent
SRC=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1/deepseek-ten/host-runtime/store.sqlite')
EXPECTED='713e43e4f31a489c8eb347b953702ca97c33b17832fbdc0633018584308507f4'
MAGIC=b'LFDIAG\0\0'; OLD=b'LFPACK\0\0'
class Invalid(ValueError):pass
def check(ok,why='invalid framing'):
 if not ok:raise Invalid(why)
def sha(path):
 with pathlib.Path(path).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def u32(b,p):
 check(p>=0 and p+4<=len(b),'truncated integer');return struct.unpack_from('<I',b,p)[0]
def record(b):
 check(10<=len(b)<=192*1024,'record bounds')
 kind=b[0];raw=u32(b,1);frame_len=u32(b,5)
 check(kind in (0,1,2) and 1<=raw<131072 and 1<=frame_len<=135168,'record values')
 start=9+32*(kind!=0);check(len(b)==start+frame_len,'frame range')
 return kind,raw,b[9:41] if kind else None,b[start:]
def original(b):
 check(len(b)<=256*1024 and b[:8]==OLD and u32(b,8)==3,'old version')
 n=u32(b,12);check(1<=n<=256 and len(b)>16+16*n,'old directory')
 out=[];pos=16+16*n
 for g in range(n):
  p,e,d,c=struct.unpack_from('<IIII',b,16+16*g)
  check(p==pos and e==d and c==0 and p+e<=len(b),'old entry');pos+=e
  r=b[p:pos];record(r);out.append(r)
 check(pos==len(b),'old trailing bytes');return out
def encode(rs,version):
 check(version in (101,102),'new version')
 bodies=[r if version==101 else r[:1]+r[9:] for r in rs]
 p=16+4*len(rs);starts=[]
 for b in bodies:starts.append(p);p+=len(b)
 return MAGIC+struct.pack('<II',version,len(rs))+struct.pack('<'+'I'*len(rs),*starts)+b''.join(bodies)
def unpack(b,version,lengths):
 check(version in (101,102) and b[:8]==MAGIC and u32(b,8)==version,'diagnostic version')
 check(len(b)<=256*1024,'pack bound');n=u32(b,12)
 check(1<=n<=256 and len(lengths)==n and len(b)>16+4*n,'directory count')
 starts=[u32(b,16+4*g) for g in range(n)]+[len(b)]
 check(starts[0]==16+4*n and all(a<z for a,z in zip(starts,starts[1:])),'directory offsets')
 out=[]
 for g,(a,z) in enumerate(zip(starts,starts[1:])):
  check(0<=a<z<=len(b),'directory range');r=b[a:z]
  check(24<=lengths[g]<131072+23,'locator length');raw=lengths[g]-23
  if version==102:
   check(2<=len(r)<=192*1024 and r[0] in (0,1,2),'compact kind/range')
   frame_len=len(r)-1-32*(r[0]!=0)
   check(1<=frame_len<=135168,'compact frame bound')
   r=r[:1]+struct.pack('<II',raw,frame_len)+r[1:]
  kind,got,base,frame=record(r);check(got==raw,'locator raw mismatch');out.append(r)
 return out
def restore(rs):
 p=16+16*len(rs);entries=[]
 for r in rs:entries.append(struct.pack('<IIII',p,len(r),len(r),0));p+=len(r)
 return OLD+struct.pack('<II',3,len(rs))+b''.join(entries)+b''.join(rs)
def nodes_for(target,records,loc):
 nodes=[];seen=set();node=target;canonical=encoded=0;full=False
 while True:
  check(node not in seen,'dependency cycle');seen.add(node)
  check(node in records and node in loc,'missing base')
  r=records[node];kind,raw,base,frame=record(r);p,g,ordinal,length=loc[node]
  check(ordinal==0 and length==raw+23 and (not full or kind==0),'role/locator')
  canonical+=length;encoded+=len(r)
  check(canonical<=512*1024 and encoded<=256*1024,'closure bound')
  nodes.append(node)
  if base is None:break
  check(len(nodes)<=8,'depth bound');check(base in loc and loc[base][0]<=p,'base chronology')
  full=kind==1;node=base
 return nodes,canonical,encoded
class Header(C.Structure):
 _fields_=[('content',C.c_ulonglong),('window',C.c_ulonglong),('block',C.c_uint),('type',C.c_uint),('size',C.c_uint),('dictid',C.c_uint),('checksum',C.c_uint),('reserved1',C.c_uint),('reserved2',C.c_uint)]
def setup_decoder():
 z=C.CDLL('/opt/homebrew/lib/libzstd.dylib')
 defs={'ZSTD_versionString':([],C.c_char_p),'ZSTD_isError':([C.c_size_t],C.c_uint),'ZSTD_getFrameHeader':([C.c_void_p,C.c_void_p,C.c_size_t],C.c_size_t),'ZSTD_findFrameCompressedSize':([C.c_void_p,C.c_size_t],C.c_size_t),'ZSTD_estimateDCtxSize':([],C.c_size_t),'ZSTD_initStaticDCtx':([C.c_void_p,C.c_size_t],C.c_void_p),'ZSTD_DCtx_reset':([C.c_void_p,C.c_int],C.c_size_t),'ZSTD_DCtx_setParameter':([C.c_void_p,C.c_int,C.c_int],C.c_size_t),'ZSTD_estimateDDictSize':([C.c_size_t,C.c_int],C.c_size_t),'ZSTD_initStaticDDict':([C.c_void_p,C.c_size_t,C.c_void_p,C.c_size_t,C.c_int,C.c_int],C.c_void_p),'ZSTD_decompress_usingDDict':([C.c_void_p,C.c_void_p,C.c_size_t,C.c_void_p,C.c_size_t,C.c_void_p],C.c_size_t)}
 for name,(args,res) in defs.items():f=getattr(z,name);f.argtypes=args;f.restype=res
 check(z.ZSTD_versionString()==b'1.5.7','decoder version')
 return z
def decode(z,frame,length,base):
 check(frame[:4]==b'\x28\xb5\x2f\xfd' and len(frame)>=5 and frame[4]&0x1b==0,'zstd grammar')
 h=Header();check(z.ZSTD_getFrameHeader(C.byref(h),frame,len(frame))==0,'zstd header')
 check(h.content==length and h.window<=262144 and h.type==0 and h.dictid==0 and h.checksum==1,'zstd frame limits')
 check(z.ZSTD_findFrameCompressedSize(frame,len(frame))==len(frame),'zstd trailing bytes')
 n=z.ZSTD_estimateDCtxSize();check(n<=1024*1024,'context bound')
 memory=(C.c_ulonglong*((n+7)//8))();ctx=z.ZSTD_initStaticDCtx(memory,C.sizeof(memory));check(ctx,'static context')
 check(not z.ZSTD_isError(z.ZSTD_DCtx_reset(ctx,3)),'reset')
 check(not z.ZSTD_isError(z.ZSTD_DCtx_setParameter(ctx,100,18)),'window parameter')
 ddict=None;workspace=0
 if base is not None:
  n=z.ZSTD_estimateDDictSize(len(base),1);workspace=((n+7)//8)*8
  check(workspace+C.sizeof(memory)<=1024*1024,'dictionary workspace')
  dictionary=(C.c_ulonglong*((n+7)//8))();ddict=z.ZSTD_initStaticDDict(dictionary,C.sizeof(dictionary),base,len(base),1,1);check(ddict,'static dictionary')
 out=C.create_string_buffer(length);got=z.ZSTD_decompress_usingDDict(ctx,out,length,frame,len(frame),ddict)
 check(got==length,'zstd decompression');return out.raw,C.sizeof(memory)+workspace

def authenticate(target,records,loc,z,hashfn):
 nodes,canonical,encoded=nodes_for(target,records,loc);base=None;maxworkspace=0
 for node in reversed(nodes):
  kind,length,baseid,frame=record(records[node]);base,w=decode(z,frame,length,base)
  canonical_bytes=b'LFSO\x01'+struct.pack('>II',length+14,length+10)+b'LFS5SML\0\0\x01'+base
  check(len(canonical_bytes)==loc[node][3] and hashfn(b'layerfs/object/v2\0'+canonical_bytes)==node,'canonical authentication')
  maxworkspace=max(maxworkspace,w)
 return len(nodes)-1,canonical,encoded,maxworkspace

def main():
 start=time.monotonic();check(sha(SRC)==EXPECTED,'source identity')
 # Parent supplies a helper built solely from cached pinned BLAKE3 sources.
 import sys
 sys.path.insert(0,str(ROOT.parent/'tools'))
 from hashing import blake3 as hashfn
 helper=ROOT.parent/'tools'/'libdiagnostic_hash.dylib'
 check(hashfn(b'').hex()=='af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262','BLAKE3 vector')
 z=setup_decoder();db=sqlite3.connect(SRC.as_uri()+'?mode=ro&immutable=1',uri=True)
 check(db.execute('pragma user_version').fetchone()[0]==9,'source schema9')
 byloc={(p,g,r):(i,n) for i,n,p,g,r in db.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects')}
 loc={};records={};totals={3:0,101:0,102:0};packs=0;pack_hashes=[];examples={}
 for p,b in db.execute('select pack_id,data from object_packs order by pack_id'):
  if u32(b,8)!=3:continue
  rs=original(b);lengths=[];packs+=1;totals[3]+=len(b)
  for g,r in enumerate(rs):
   identity,length=byloc[p,g,0];lengths.append(length);records[identity]=r;loc[identity]=(p,g,0,length)
  hashes={'pack_id':p,'original_sha256':hashlib.sha256(b).hexdigest()}
  for version in (101,102):
   compact=encode(rs,version);back=unpack(compact,version,lengths)
   check(back==rs and restore(back)==b,'exact pack restoration');totals[version]+=len(compact)
   hashes[str(version)+'_sha256']=hashlib.sha256(compact).hexdigest()
   if version not in examples and len(rs)>2:examples[version]=(compact,lengths,rs)
  pack_hashes.append(hashes)
 check(packs==514 and len(records)==33217,'corpus counts')
 check(totals[3]-totals[101]==398604 and totals[101]-totals[102]==265736,'fixed saving')
 maximum=[0]*4
 for identity in records:
  facts=authenticate(identity,records,loc,z,hashfn);maximum=[max(a,b) for a,b in zip(maximum,facts)]
 tests=[]
 def reject(name,fn):
  try:fn()
  except (Invalid,struct.error):tests.append(name);return
  raise AssertionError('mutation accepted: '+name)
 def put(b,p,n):return b[:p]+struct.pack('<I',n)+b[p+4:]
 for v,(b,lengths,rs) in examples.items():
  reject(f'{v}:old_parser_refusal',lambda:original(b))
  reject(f'{v}:other_version_refusal',lambda:unpack(b,203-v,lengths))
  reject(f'{v}:old_magic_refusal',lambda:unpack(restore(rs),v,lengths))
  for name,mut in [('zero_count',put(b,12,0)),('huge_count',put(b,12,257)),('truncated_directory',b[:18]),('start_inside_directory',put(b,16,16)),('nonmonotonic_offsets',put(b,20,u32(b,16))),('end_outside_pack',put(b,20,len(b)+1)),('truncated_body',b[:-1])]:
   def validate_mut(mut=mut):
    recovered=unpack(mut,v,lengths)
    for r in recovered:kind,n,base,frame=record(r);check(z.ZSTD_findFrameCompressedSize(frame,len(frame))==len(frame),'frame truncation')
   reject(f'{v}:{name}',validate_mut)
  reject(f'{v}:invalid_locator_length',lambda:unpack(b,v,[23]+lengths[1:]))
  pos=u32(b,16);bad=b[:pos]+bytes([3])+b[pos+1:]
  reject(f'{v}:invalidkind',lambda:unpack(bad,v,lengths))
  recovered=unpack(b,v,[lengths[0]+1]+lengths[1:]) if v==102 else None
  if recovered:
   r=recovered[0];kind,n,base,frame=record(r)
   reject('102:wrong_derived_raw_length',lambda:decode(z,frame,n,None))
  else:reject('101:wrong_locator_length',lambda:unpack(b,v,[lengths[0]+1]+lengths[1:]))
 full=next(i for i,r in records.items() if r[0]==0);delta=next(i for i,r in records.items() if r[0]==2)
 r=records[full]
 reject('oversized_pack',lambda:unpack(MAGIC+struct.pack('<II',101,1)+b'x'*(256*1024),101,[24]))
 reject('zero_frame_length',lambda:record(put(r,5,0)))
 reject('oversized_frame_length',lambda:record(put(r,5,135169)))
 reject('extra_frame_bytes',lambda:record(r+b'x'))
 reject('bad_raw_length',lambda:record(put(r,1,131072)))
 reject('bad_frame_length',lambda:record(put(r,5,u32(r,5)+1)))
 wrong=next(i for i in records if i!=full)
 reject('wrong_selected_locator_identity',lambda:authenticate(wrong,{**records,wrong:r},{**loc,wrong:loc[full]},z,hashfn))
 dr=records[delta]
 reject('missing_corrupted_base',lambda:nodes_for(delta,{**records,delta:dr[:9]+b'\xff'*32+dr[41:]},loc))
 reject('valid_but_wrong_base',lambda:authenticate(delta,{**records,delta:dr[:9]+full+dr[41:]},loc,z,hashfn))
 reject('cycle',lambda:nodes_for(delta,{**records,delta:dr[:9]+delta+dr[41:]},loc))
 base=record(dr)[2]
 reject('future_base',lambda:nodes_for(delta,records,{**loc,base:(100000,*loc[base][1:])}))
 reject('kind1_delta_base',lambda:nodes_for(delta,{**records,delta:bytes([1])+dr[1:],base:bytes([2])+dr[1:]},loc))
 # Synthetic correctly framed nodes isolate depth/canonical/encoded guards without codec work.
 def synthetic(count,raw=1,frame=1):
  ids=[i.to_bytes(32,'little') for i in range(count)];rr={};ll={}
  for j,i in enumerate(ids):
   rr[i]=bytes([2 if j else 0])+struct.pack('<II',raw,frame)+(ids[j-1] if j else b'')+b'x'*frame
   ll[i]=(j+1,0,0,raw+23)
  return ids[-1],rr,ll
 for name,args in [('depth',(10,)),('canonical_closure',(5,131071,1)),('encoded_closure',(3,1,100000))]:reject(name,lambda args=args:nodes_for(*synthetic(*args)))
 # SQLite geometry only: complete disposable in-memory copies, identical compact baseline.
 geometry={}
 for version in (3,101,102):
  copy=sqlite3.connect(':memory:');db.backup(copy)
  if version!=3:
   for p,b in db.execute('select pack_id,data from object_packs'):
    if u32(b,8)==3:copy.execute('update object_packs set data=? where pack_id=?',(encode(original(b),version),p))
   copy.execute('pragma user_version='+str(9000+version));copy.commit()
  copy.execute('vacuum')
  check(copy.execute('pragma integrity_check').fetchone()==('ok',),'copy SQLite integrity')
  page_size=copy.execute('pragma page_size').fetchone()[0];page_count=copy.execute('pragma page_count').fetchone()[0]
  geometry[str(version)]={'logical_bytes':page_size*page_count,'pack_btree':dict(zip(('bytes','payload','unused'),copy.execute("select sum(pgsize),sum(payload),sum(unused) from dbstat where name='object_packs'").fetchone()))}
  copy.close()
 db.close();check(sha(SRC)==EXPECTED,'source unchanged')
 result={'scope':'Standalone lossless physical-format diagnostic; not product compatibility, rollback, cold reopen or benchmark qualification','source':str(SRC),'source_sha256_before_and_after':EXPECTED,'script_sha256':sha(__file__),'protocol_sha256':sha(ROOT/'protocol.md'),'packs':packs,'objects':len(records),'pack_bytes':totals,'saving_directory':398604,'additional_saving_lengths':265736,'total_saving':664340,'all_original_pack_bytes_exactly_restored':True,'every_selected_identity_and_dependency_authenticated':True,'max_depth':maximum[0],'max_canonical_closure':maximum[1],'max_original_encoded_closure':maximum[2],'max_static_decoder_dictionary_workspace':maximum[3],'negative_checks_passed':tests,'zstd_version':'1.5.7','zstd_sha256':sha('/opt/homebrew/lib/libzstd.dylib'),'blake3_helper':str(helper),'blake3_helper_sha256':sha(helper),'elapsed_seconds':time.monotonic()-start,'pack_hashes':pack_hashes,'offline_inmemory_sqlite_geometry':geometry}
 (ROOT/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='pack_hashes'},indent=2))
if __name__=='__main__':main()
