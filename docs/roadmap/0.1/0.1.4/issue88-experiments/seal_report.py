#!/usr/bin/env python3
"""Seal existing issue88 evidence without modifying the source artifact directories."""
import hashlib,json,pathlib,shutil,sys

def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def main(runs,docs):
 report=runs/'issue88-report-1'
 artifacts={}
 for folder in sorted(runs.glob('issue88-*')):
  if not folder.is_dir() or folder==report:continue
  artifacts[folder.name]={str(p.relative_to(folder)):{'sha256':sha(p),'bytes':p.stat().st_size} for p in sorted(folder.rglob('*')) if p.is_file()}
 registry={'scope':'existing offline/public S1 evidence and additive analysis snapshots; relative paths under declared runs root; no source directories modified','runs_root':str(runs),'artifacts':artifacts}
 with (report/'artifacts.sha256.json').open('x') as f:json.dump(registry,f,indent=2);f.write('\n')
 for name in ['findings.md','review.md','final-review-addendum.md','contract-v1.md','public-s1-contract.md','full-history-continuation.md','read-cost-contract.md']:
  shutil.copyfile(docs/name,report/name)
 sources={str(p.relative_to(docs)):{'sha256':sha(p),'bytes':p.stat().st_size} for p in sorted(docs.rglob('*')) if p.is_file() and not any(x in ['target','__pycache__','published'] for x in p.relative_to(docs).parts)}
 with (report/'reporter-sources.sha256.json').open('x') as f:json.dump({'scope':'source files at report seal; executed binaries separately authenticated, not inferred from reporting revision','files':sources},f,indent=2);f.write('\n')
 manifest={p.name:{'sha256':sha(p),'bytes':p.stat().st_size} for p in sorted(report.iterdir()) if p.is_file()}
 with (report/'manifest.sha256.json').open('x') as f:json.dump({'scope':'final issue88 S1/S2/S3 report; manifest self-hash excluded','files':manifest},f,indent=2);f.write('\n')
 dest=docs/'published';dest.mkdir(exist_ok=False)
 for p in report.iterdir():
  if p.is_file():shutil.copyfile(p,dest/p.name)
 print(json.dumps({'status':'PASS','artifact_directories':len(artifacts),'artifact_files':sum(map(len,artifacts.values())),'manifest_sha256':sha(report/'manifest.sha256.json')}))
if __name__=='__main__':main(*map(pathlib.Path,sys.argv[1:]))
