#!/usr/bin/env python3
"""Authenticate existing sealed manifests; explicit pre-verification Store substitution only."""
import hashlib,json,pathlib,sys

def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def main(runs,out):
 checked=[]
 for name,phase in [('full157-m45-1','verification'),('issue88-s1-full157-1','performance'),('issue88-s1-full157-1','verification')]:
  folder=runs/name;manifest=folder/(phase+'-manifest.json');entries=json.loads(manifest.read_text());files=[]
  for relative,expected in entries.items():
   path=folder/relative;substitution=None
   if name=='issue88-s1-full157-1' and phase=='performance' and relative=='deepseek-full/host-runtime/store.sqlite':
    path=runs/'issue88-s1-preverification-snapshot-1/store.sqlite';substitution='sealed pre-verification logical copy; no allocation substitution'
   actual=sha(path);assert actual==expected,(name,phase,relative,actual,expected)
   files.append({'relative':relative,'actual_path':str(path),'sha256':actual,'substitution':substitution})
  checked.append({'run':name,'phase':phase,'manifest_sha256':sha(manifest),'count':len(files),'files':files})
 original=runs/'full157-m45-1/deepseek-full/host-runtime/store.sqlite';assert sha(original)=='2036b98181dd3fa47788619896cb50daa27ac5c85ea8cc482c20d4b447573ed9'
 snap=runs/'issue88-s1-preverification-snapshot-1/store.sqlite';idx=runs/'issue88-s1-final-census-1/roles-inventory.sqlite';decoder=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3/docs/roadmap/0.1/0.1.4/issue87-analysis/roles/target/release/issue87-roles')
 result={'status':'PASS','scope':'existing final evidence manifests and preverification copy; no fresh benchmark or allocation measurement','manifests':checked,'snapshot_sha256':sha(snap),'inventory_sha256':sha(idx),'decoder_sha256':sha(decoder),'decoder_reuse':'existing issue87 canonical/record decoder; final-census-1.log records all366293 selected objects authenticated across4913 packs','analysis_index_binding':'parent-launched exact decoder invocation on separately sealed preverification copy; hashes bind completed output, not a claimed producer census receipt'}
 with out.open('x') as f:json.dump(result,f,indent=2);f.write('\n')
 print(json.dumps({'status':'PASS','counts':[(x['run'],x['phase'],x['count']) for x in checked]}))
if __name__=='__main__':main(*map(pathlib.Path,sys.argv[1:]))
