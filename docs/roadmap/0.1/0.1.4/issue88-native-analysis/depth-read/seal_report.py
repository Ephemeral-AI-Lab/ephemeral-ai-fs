#!/usr/bin/env python3
"""Seal additive depth-read finalization, preserving prior evidence and Git bytes."""
import pathlib,json,hashlib,shutil,sys

def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def write(p,v):
 with p.open('x') as f:json.dump(v,f,indent=2);f.write('\n')
def main(out,dest):
 assert out.is_dir() and not dest.exists() and not (out/'manifest.sha256.json').exists()
 root=out.parent;here=pathlib.Path(__file__).resolve().parent
 names=['issue88-depth-read-preparation-1','issue88-depth-read-builds-1','issue88-depth-read-cohort-1','issue88-depth-read-selection-supervisor-1','issue88-depth-read-campaign-1','issue88-depth-read-results-1']
 registry={n:{str(p.relative_to(root/n)):{'sha256':sha(p),'bytes':p.stat().st_size} for p in sorted((root/n).rglob('*')) if p.is_file()} for n in names}
 write(out/'artifacts.sha256.json',{'runs_root':str(root),'scope':'complete preparation/build/cohort/campaign/result evidence includingfailed prelaunch/buildlogs; originalhistoricalSPevidence remains inits sealedreport','directories':registry})
 sources=[p for p in here.rglob('*') if p.is_file() and not any(x in p.parts for x in ['target','__pycache__','published'])]
 sources += [here.parent/n for n in ['depth_custody.py','supervise_depth_preparation.py']]
 write(out/'analysis-sources.sha256.json',{str(p.relative_to(here.parent)):{'sha256':sha(p),'bytes':p.stat().st_size} for p in sorted(sources)})
 manifest={str(p.relative_to(out)):{'sha256':sha(p),'bytes':p.stat().st_size} for p in sorted(out.rglob('*')) if p.is_file()}
 write(out/'manifest.sha256.json',{'scope':'issue88terminalresearchreadreport; selfhash excluded','files':manifest})
 shutil.copytree(out,dest);(dest/'.gitattributes').write_text('* -text\n*.csv whitespace=cr-at-eol\n')
 for name,v in manifest.items():assert sha(dest/name)==v['sha256']
 print(json.dumps({'status':'PASS','manifest_sha256':sha(out/'manifest.sha256.json'),'artifact_directories':len(names),'artifact_files':sum(len(v) for v in registry.values())}))
if __name__=='__main__':main(*map(lambda x:pathlib.Path(x).resolve(),sys.argv[1:]))
