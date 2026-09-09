"""One fixed chronological FULL-vs-bounded-delta experiment. See protocol.md."""
import collections as C, csv, hashlib, json, pathlib, sqlite3, struct, sys, time
HERE=pathlib.Path(__file__).resolve().parent
SOURCE=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-structural/metadata/d')
sys.path.insert(0,str(SOURCE));import api as D;import codec
sys.path.insert(0,str(pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/tools')));import matcher

def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def leaf(b):return b[13:21]==b'LFS6INT\0' and b[23]==7

def edges(b):
    v=codec.value(b);magic=v[:8]
    if magic==b'LFS6FSR\0':return [v[84:116]]
    if magic==b'LFS6INT\0':
        if v[10]==7:return [ref for p in range(31,len(v),81) for ref in (v[p+17:p+49],v[p+49:p+81])]
        assert v[10]==8;return [v[p+8:p+40] for p in range(31,len(v),40)]
    if magic==b'LFS6NSP\0':
        if v[10]==1:return []
        assert v[10]==2;p=31;refs=[]
        for _ in range(int.from_bytes(v[13:15],'big')):
            n=int.from_bytes(v[p:p+2],'big');p+=2+n;refs.append(v[p:p+32]);p+=32
        assert p==len(v);return refs
    if magic==b'LFS4MAP\0':
        if v[10]==10:assert len(v)==93;return [v[61:93]]
        if v[10]==8:assert (len(v)-31)%40==0;return [v[p:p+32] for p in range(31,len(v),40)]
        assert v[10]==9 and (len(v)-31)%48==0;return [v[p+16:p+48] for p in range(31,len(v),48)]
    if magic==b'LFS4MET\0':
        assert v[10] in (9,10);p=31;refs=[]
        for _ in range(int.from_bytes(v[13:15],'big')):
            for _ in range(2):n=int.from_bytes(v[p:p+2],'big');p+=2+n
            if v[10]==9:assert v[p]==1;p+=1
            refs.append(v[p:p+32]);p+=32
        assert p==len(v);return refs
    assert magic in (b'LFS4CHK\0',b'LFS4LNK\0'),magic;return []

def leaf_ids(inv,root):
    if leaf(inv[root]):return [root]
    assert codec.role(inv[root])==b'LFS6INT\0';return [i for child in edges(inv[root]) for i in leaf_ids(inv,child)]

def keys(b):
    assert leaf(b) and len(b)<=8192
    result=tuple(int.from_bytes(b[p:p+8],'big') for p in range(44,len(b),81));assert 1<=len(result)<=100 and list(result)==sorted(set(result));return result

def chronology(inv,roots):
    seen=set();first={};cohorts=[];visiting=set();allkeys={i:keys(b) for i,b in inv.items() if leaf(b)};per_root=[]
    for step,root in enumerate(roots):
        cohort=[]
        def visit(i):
            if i not in inv or i in seen:return
            assert i not in visiting;visiting.add(i)
            for child in edges(inv[i]):visit(child)
            visiting.remove(i);seen.add(i);first[i]=step;cohort.append(i)
        visit(root)
        current=leaf_ids(inv,codec.value(inv[root])[84:116]);assert len(current)<=128
        per_root.append(current);cohorts.append(cohort)
    unrooted=sorted(set(inv)-seen)
    assert all(not leaf(inv[i]) for i in unrooted)
    for i in unrooted:first[i]=len(roots)-1;cohorts[-1].append(i)
    cohorts=[sorted(c,key=lambda i:(codec.role(inv[i]),i)) for c in cohorts]
    assert sum(map(len,cohorts))==len(inv) and len(first)==len(inv)
    return cohorts,per_root,allkeys,first,unrooted

def grouped(inv,cohort):
    result=[];pending=[];length=4
    for i in cohort:
        n=len(inv[i]);assert 0<n<=8192
        if pending and length+5+n>16384:result.append(pending);pending=[];length=4
        pending.append(i);length+=5+n
    if pending:result.append(pending)
    return result

def frame(records):
    ends=[];length=0
    for r in records:length+=len(r);ends.append(length)
    body=struct.pack('<I',len(records))+struct.pack('<'+'I'*len(records),*ends)+b''.join(records)
    assert len(body)<=16384;tag,data=codec.compress(body)
    return dict(body=body,codec=tag,data=data)

def closure(identity,records):
    seen=set();total=0;depth=0
    while True:
        assert identity not in seen and identity in records;seen.add(identity)
        r=records[identity];assert r[0] in (0,1)
        if r[0]==0:
            total+=len(r)-1;break
        total+=int.from_bytes(r[33:37],'little');identity=r[1:33];depth+=1
        assert depth<=16 and total<=131072
    assert total<=131072
    return depth,total

def decode_record(identity,records):
    closure(identity,records)
    r=records[identity]
    if r[0]==0:b=r[1:]
    else:
        baseid=r[1:33];base=decode_record(baseid,records)
        assert leaf(base) and len(base)<=8192
        b=matcher.replay(r,baseid,base);assert leaf(b)
    assert 0<len(b)<=8192 and codec.oid(b)==identity
    return b

def fixture(inv):
    baseid=next(i for i,b in inv.items() if leaf(b) and len(b)>100);base=inv[baseid]
    pos=100;target=base[:pos]+bytes([base[pos]^1])+base[pos+1:];targetid=codec.oid(target)
    record=b'\1'+baseid+struct.pack('<II',len(target),3)+b'\0'+struct.pack('<II',0,pos)+b'\1'+struct.pack('<I',1)+target[pos:pos+1]+b'\0'+struct.pack('<II',pos+1,len(target)-pos-1)
    ledger={baseid:b'\0'+base,targetid:record};assert decode_record(targetid,ledger)==target
    wrong=next((i,b) for i,b in inv.items() if codec.role(b)==b'LFS4MET\0')
    invalid=[('missing_base',{targetid:record}),('corrupt_base',{**ledger,baseid:b'\0'+base[:-1]+bytes([base[-1]^1])}),('wrong_role_base',{targetid:record[:1]+wrong[0]+record[33:],wrong[0]:b'\0'+wrong[1]}),('depth2_base',{**ledger,baseid:record})]
    for name,rows in invalid:
        try:decode_record(targetid,rows)
        except (AssertionError,KeyError):pass
        else:raise AssertionError(('accepted invalid',name))
    encoded,left,counters=matcher.delta(baseid,base,target,16*1024*1024);assert encoded is not None and matcher.replay(encoded,baseid,base)==target
    rejected,left,counters=matcher.delta(baseid,base,target,0);assert rejected is None and left==0
    return dict(status='PASS',rejected=[name for name,rows in invalid],matcher_roundtrip=True,zero_budget_fallback=True)

def assemble(groups,which):
    header=b'LFPACK\0\0'+struct.pack('<II',1,len(groups));offset=16+16*len(groups);directory=[]
    for g in groups:
        x=g[which];directory.append(struct.pack('<IIII',offset,len(x['data']),len(x['body']),x['codec']));offset+=len(x['data'])
    blob=header+b''.join(directory)+b''.join(g[which]['data'] for g in groups)
    assert len(blob)==offset<=262144 and 16+16*len(groups)+sum(len(g['full']['body']) for g in groups)<=262144
    return blob

def unpack(blob,positions):
    assert blob[:8]==b'LFPACK\0\0' and int.from_bytes(blob[8:12],'little')==1
    n=int.from_bytes(blob[12:16],'little');result={};expected=16+16*n
    for group in range(n):
        off,enc,dec,tag=struct.unpack_from('<IIII',blob,16+16*group);assert off==expected;expected+=enc;encoded=blob[off:off+enc]
        body=codec.decompress(encoded,dec) if tag else encoded;assert len(body)==dec
        count=struct.unpack_from('<I',body)[0];start=4+4*count;previous=start
        for k in range(count):
            end=start+struct.unpack_from('<I',body,4+4*k)[0];assert previous<end<=len(body)
            result[positions[group,k]]=body[previous:end];previous=end
        assert previous==len(body)
    assert expected==len(blob);return result

def main():
    started=time.monotonic();input_result=json.loads((SOURCE/'result.json').read_text());seals=json.loads((SOURCE/'manifest.json').read_text())
    for name,h in seals.items():assert sha(SOURCE/name)==h
    for name in ('cache.sqlite','result.json','ledger.csv','states.json'):assert not(HERE/name).exists(),('appendonly',name)
    inv,roots,content,scope,highwater=D.original_inventory();assert len(inv)==input_result['metadata_objects'] and len(roots)==54
    cohorts,rootleaves,allkeys,first,unrooted=chronology(inv,roots)
    tests=fixture(inv);records={};ledger=[];groups_ledger=[];counters=C.Counter();allpacks={'full':[],'delta':[]};locators=[];state_results=[];physical_bases=set();max_closure=0
    for step,cohort in enumerate(cohorts):
        previous=rootleaves[step-1] if step else [];remaining=16*1024*1024;trials=0;pending=[];full_pack_size=16;recordcount=0
        def flush():
            nonlocal pending,full_pack_size,recordcount
            if not pending:return
            packid=len(allpacks['full'])+1
            for name in ('full','delta'):allpacks[name].append((packid,assemble(pending,name)))
            for group,g in enumerate(pending):
                for ordinal,i in enumerate(g['ids']):locators.append((i,len(inv[i]),packid,group,ordinal))
            pending=[];full_pack_size=16;recordcount=0
        for groupids in grouped(inv,cohort):
            full_records=[b'\0'+inv[i] for i in groupids];candidates={};group_rows=[]
            for i in groupids:
                if not leaf(inv[i]):continue
                row=dict(object_id=i.hex(),step=step,base_id='',shared_keys=0,status='no_previous' if not previous else 'no_overlap',candidate_record_bytes=0,selected_record_bytes=0)
                targetkeys=allkeys[i];ts=set(targetkeys);rank=[]
                for old in previous:
                    oldkeys=allkeys[old]
                    if min(targetkeys[-1],oldkeys[-1])<max(targetkeys[0],oldkeys[0]):continue
                    overlap=len(ts.intersection(oldkeys))
                    if overlap:rank.append((-overlap,old))
                if rank:
                    overlap,origin=min(rank);row.update(base_id=origin.hex(),shared_keys=-overlap)
                    assert first[origin]<step and origin in records
                    if closure(origin,records)[0]>=16 or closure(origin,records)[1]+len(inv[i])>131072:row['status']='origin_closure_bound'
                    elif trials>=512 or remaining==0:row['status']='root_budget_exhausted'
                    else:
                        trials+=1;base=inv[origin];target=inv[i]
                        assert len(base)+len(target)<=16384 and len(base)<=8192 and len(target)<=8192 and leaf(base) and codec.oid(base)==origin and codec.oid(target)==i
                        candidate,remaining,counts=matcher.delta(origin,base,target,remaining);counters.update(counts);counters['matcher_trials']+=1
                        if candidate is None:row['status']='matcher_no_candidate'
                        else:
                            assert matcher.replay(candidate,origin,base)==target and len(candidate)<=len(target)+1
                            candidates[i]=candidate;row.update(status='candidate_pending_group',candidate_record_bytes=len(candidate))
                group_rows.append(row)
            full=frame(full_records);chosen=full;mixed_size=None;use_mixed=False
            if candidates:
                mixed=frame([candidates.get(i,b'\0'+inv[i]) for i in groupids]);mixed_size=len(mixed['data'])
                threshold=max(64,(len(full['data'])+7)//8)
                use_mixed=len(full['data'])-len(mixed['data'])>=threshold
                if use_mixed:chosen=mixed;counters['mixed_groups_selected']+=1
                else:counters['mixed_groups_rejected']+=1
            for i in groupids:
                selected=candidates[i] if use_mixed and i in candidates else b'\0'+inv[i]
                assert i not in records;records[i]=selected
                if selected[0]:
                    baseid=selected[1:33];assert first[baseid]<step;physical_bases.add(baseid);max_closure=max(max_closure,closure(i,records)[1])
            for row in group_rows:
                i=bytes.fromhex(row['object_id']);row['selected_record_bytes']=len(records[i])
                if i in candidates:row['status']='selected_DELTA' if records[i][0] else 'mixed_group_rejected'
                counters['status_'+row['status']]+=1;ledger.append(row)
            group=dict(ids=groupids,full=full,delta=chosen)
            if pending and (full_pack_size+16+len(full['body'])>262144 or len(pending)==256 or recordcount+len(groupids)>8191):flush()
            pending.append(group);full_pack_size+=16+len(full['body']);recordcount+=len(groupids)
            groups_ledger.append(dict(step=step,objects=len(groupids),candidate_objects=len(candidates),FULL_encoded_bytes=len(full['data']),MIXED_encoded_bytes=mixed_size,selected_encoded_bytes=len(chosen['data']),FULL_decoded_bytes=len(full['body']),selected_decoded_bytes=len(chosen['body']),mixed_selected=use_mixed))
        flush();state_results.append(dict(step=step,new_objects=len(cohort),previous_leaf_candidates=len(previous),trials=trials,match_budget_remaining=remaining))
        if(step+1)%20==0:print('encoded root',step+1,'/54; selected deltas',counters['status_selected_DELTA'],flush=True)
    assert len(records)==len(inv)==len(locators) and len(ledger)==len(allkeys)
    locations=C.defaultdict(dict)
    for i,n,p,g,k in locators:locations[p][g,k]=i
    physical={}
    for p,blob in allpacks['delta']:
        parsed=unpack(blob,locations[p]);assert all(parsed[i]==records[i] for i in parsed);physical.update(parsed)
    assert len(physical)==len(inv)
    reconstructed={i:decode_record(i,physical) for i in inv};assert reconstructed==inv
    for p,blob in allpacks['full']:
        assert all(r==b'\0'+inv[i] for i,r in unpack(blob,locations[p]).items())
    for step,root in enumerate(roots):
        table=codec.value(reconstructed[root])[84:116];got=D.decode_table(reconstructed,table);expected=D.decode_table(inv,table);assert got==expected
        directory_roots={v[9:41] for s,v in got if v[0]==2}
        for droot in directory_roots:assert D.decode_directory(reconstructed,droot)==D.decode_directory(inv,droot)
        state_results[step].update(verified=True,inodes=len(got),directories=len(directory_roots),inode_tuple_sha256=hashlib.sha256(b''.join(s+v for s,v in got)).hexdigest())
    db=sqlite3.connect(HERE/'cache.sqlite')
    db.execute('create table packs(layout text,pack_id integer,data blob,primary key(layout,pack_id)) without rowid')
    db.execute('create table locators(id blob primary key,canonical_length integer,pack_id integer,group_number integer,record_number integer) without rowid')
    db.execute('create table records(id blob primary key,first_step integer,kind integer,base_id blob,record blob) without rowid')
    for layout,packs in allpacks.items():db.executemany('insert into packs values(?,?,?)',[(layout,p,b) for p,b in packs])
    db.executemany('insert into locators values(?,?,?,?,?)',sorted(locators));db.executemany('insert into records values(?,?,?,?,?)',[(i,first[i],r[0],r[1:33] if r[0] else None,r) for i,r in sorted(records.items())]);db.commit();assert db.execute('pragma integrity_check').fetchone()[0]=='ok';db.close()
    with(HERE/'ledger.csv').open('x',newline='')as f:
        w=csv.DictWriter(f,fieldnames=list(ledger[0]));w.writeheader();w.writerows(ledger)
    (HERE/'groups.json').write_text(json.dumps(groups_ledger,indent=2)+'\n');(HERE/'states.json').write_text(json.dumps(state_results,indent=2)+'\n')
    summaries={layout:dict(packs=len(packs),pack_bytes=sum(len(b) for p,b in packs),pack_sha256={str(p):hashlib.sha256(b).hexdigest() for p,b in packs}) for layout,packs in allpacks.items()}
    for name,h in seals.items():assert sha(SOURCE/name)==h
    assert sha(codec.STORE)==input_result['source_sha256_after']
    result=dict(layouts=summaries,actual_pack_saving_vs_matched_FULL=summaries['full']['pack_bytes']-summaries['delta']['pack_bytes'],published_D_hashsorted_pack_bytes=input_result['pack_bytes'],difference_vs_published_D=input_result['pack_bytes']-summaries['delta']['pack_bytes'],metadata_objects=len(inv),groups=len(groups_ledger),canonical_bytes=sum(map(len,inv.values())),selected_full_objects=sum(r[0]==0 for r in records.values()),selected_delta_objects=sum(r[0]==1 for r in records.values()),selected_full_record_bytes=sum(len(r) for r in records.values() if r[0]==0),selected_delta_record_bytes=sum(len(r) for r in records.values() if r[0]),physical_dependency_bases=len(physical_bases),physical_dependency_base_record_bytes=sum(len(records[i]) for i in physical_bases),base_accounting='subset of selected objects FULL or DELTA; counted once',unrooted_carried_metadata_objects=len(unrooted),unrooted_roles=dict(C.Counter(codec.role(inv[i]).rstrip(b'\0').decode() for i in unrooted)),maximum_canonical_closure=max_closure,maximum_depth=max(closure(i,records)[0] for i in records),counters=dict(counters),verification=dict(all_objects_authenticated=len(reconstructed),verified_states=len(state_results),fixture=tests,canonical_inventory_byte_equal=True),scope=scope.hex(),highwater=highwater,source_manifest_sha256=sha(SOURCE/'manifest.json'),source_store_sha256_after=sha(codec.STORE),cache_sha256=sha(HERE/'cache.sqlite'),protocol_sha256=sha(HERE/'protocol.md'),script_sha256=sha(pathlib.Path(__file__)),matcher_hashes={p.name:sha(p) for p in (pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/tools')).iterdir() if p.is_file()},codec_calls=codec.codec_calls,max_codec_workspace_bytes=codec.max_workspace,elapsed_seconds=time.monotonic()-started,scope_note='Onefixeddepth16closure128KiBpolicy,samechronologicalgroups/packs;entireDcanonicalinventoryunchanged.NoStoreallocationorCommitclaim.')
    (HERE/'result.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:result[k] for k in ['actual_pack_saving_vs_matched_FULL','difference_vs_published_D','selected_full_objects','selected_delta_objects','physical_dependency_bases','maximum_canonical_closure','counters','elapsed_seconds']},indent=2),flush=True)
if __name__=='__main__':main()
