"""Reuse exact oracle decoder against every original stride3 state in actual copy."""
import argparse,hashlib,importlib.util,json,sys,time
from pathlib import Path
ROOT=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-structural')
BASE=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-comparison/layerfs')
FIXTURE=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-data/deepseek-stride3-218c1b81ed39c763ca22/fixture.json')
sys.path.insert(0,str(ROOT/'combined'));import store_api
SOURCE=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/verification/verify.py')
assert hashlib.sha256(SOURCE.read_bytes()).hexdigest()=='a660ba28254caa8db1c43504b0b7be838192c0a9132eb0800b551ffa598cab0d'
spec=importlib.util.spec_from_file_location('original_oracle_verifier',SOURCE);v=importlib.util.module_from_spec(spec);spec.loader.exec_module(v)
assert v.physical is store_api

def sha(path):
 with Path(path).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def main():
 parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,required=True);args=parser.parse_args();assert not args.output.exists()
 result=json.loads((ROOT/'combined/result.json').read_text())['results']['delta'];path=Path(result['path']);before=sha(path);assert before==result['sha256']
 assert sha(FIXTURE)=='3b2c12b682e4892837fc810969e4e90eaa8d756c0d6f738df50746ad478fb73e'
 states=json.loads(FIXTURE.read_text())['deepseek-stride3']['states'];original=json.loads((BASE/'deepseek-stride3/verification-result.json').read_text())['records']
 performance=json.loads((BASE/'deepseek-stride3/performance-result.json').read_text())['records']
 identities=json.loads((ROOT/'combined/identity-map.json').read_text());assert len(states)==len(original)==len(performance)==53
 assert [r['full157_index'] for r in states]==list(range(1,158,3))
 reader=store_api.StoreReader(path,cache_bytes=4*1024*1024,pack_cache_bytes=8*1024*1024);files=v.Files(reader);rows=[];start=time.monotonic()
 try:
  for state,receipt,produced in zip(states,original,performance):
   assert state['index']==receipt['index']==produced['index'] and receipt['identity']==produced['identity']
   assert state['oracle_sha256']==produced['oracle_sha256']==sha(state['oracle'])
   assert state['sha']==produced['sha'] and state['tree']==produced['tree']
   commit=bytes.fromhex(identities['commits'][receipt['identity']]);root=reader.db.execute('select root_id from commits where commit_id=?',(commit,)).fetchone();assert root
   expected=json.loads(Path(state['oracle']).read_text());observed=v.observe(reader,files,root[0])
   if expected!=observed:
    differing=sorted(k for k in set(expected)|set(observed) if expected.get(k)!=observed.get(k));raise AssertionError((state['index'],[(k,expected.get(k),observed.get(k)) for k in differing[:5]]))
   logical=sum(row[1] for row in observed.values());assert logical==state['logical_bytes']
   rows.append(dict(index=state['index'],full157_index=state['full157_index'],source_sha=state['sha'],commit_id=commit.hex(),paths=len(observed),logical_bytes=logical,oracle_sha256=state['oracle_sha256'],status='PASS'))
   if state['index']==1 or state['index']%10==0 or state['index']==53:print(f"PASS {state['index']}/53 (original{state['full157_index']}): {len(observed)}paths",flush=True)
 finally:reader.close()
 after=sha(path);assert before==after
 paths=sum(r['paths'] for r in rows);logical=sum(r['logical_bytes'] for r in rows);assert paths==306861 and logical==1676767835
 out=dict(status='PASS',scope='All53originalstride3oracles through actual offline structural reader; not public operation timing',states=len(rows),path_states=paths,logical_bytes=logical,unique_file_roots=len(files.summaries),unique_file_bytes_hashed=files.unique_logical_bytes,copy_sha256_before=before,copy_sha256_after=after,original_fixture_and_performance_oracle_seals='PASS',fixture_sha256=sha(FIXTURE),script_sha256=sha(Path(__file__)),reused_oracle_decoder_sha256=sha(SOURCE),reader_sha256=sha(ROOT/'combined/store_api.py'),elapsed_seconds=time.monotonic()-start,records=rows)
 args.output.write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({k:v for k,v in out.items() if k!='records'},indent=2))

if __name__=='__main__':main()
