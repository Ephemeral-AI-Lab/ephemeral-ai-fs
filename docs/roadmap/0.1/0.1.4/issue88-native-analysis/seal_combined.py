#!/usr/bin/env python3
"""Seal complete experiment/report artifacts once, preserving published bytes."""
import argparse, hashlib, json, pathlib, shutil

def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream,'sha256').hexdigest()

def save(path,data):
    with path.open('x') as stream:
        json.dump(data,stream,indent=2);stream.write('\n')

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('report',type=pathlib.Path);p.add_argument('published',type=pathlib.Path);a=p.parse_args()
    root=a.report.parent;here=pathlib.Path(__file__).resolve().parent
    assert not a.published.exists() and not (a.report/'manifest.sha256.json').exists()
    folders=sorted(d for d in root.glob('issue88-SP-*') if d.is_dir() and d!=a.report)
    folders += [root/'issue88-P-custody-1']
    registry={str(d):{str(f.relative_to(d)):{'sha256':sha(f),'bytes':f.stat().st_size} for f in sorted(d.rglob('*')) if f.is_file()} for d in folders}
    save(a.report/'artifacts.sha256.json',{'scope':'Combined fresh control/candidate smokes, reads, full157, one snapshot/census perarm, custody/validation/accounting/resources; failed logs retained; original performance Store identity explicitly uses snapshot after verifier metadata additions','directories':registry})
    save(a.report/'reporter-sources.sha256.json',{str(f.relative_to(here)):{'sha256':sha(f),'bytes':f.stat().st_size} for f in sorted(here.rglob('*')) if f.is_file() and not any(x in f.parts for x in ['target','__pycache__','published'])})
    manifest={str(f.relative_to(a.report)):{'sha256':sha(f),'bytes':f.stat().st_size} for f in sorted(a.report.rglob('*')) if f.is_file()}
    save(a.report/'manifest.sha256.json',{'scope':'final combined experiment report; self-hash excluded','files':manifest})
    shutil.copytree(a.report,a.published)
    (a.published/'.gitattributes').write_text('* -text\n*.csv whitespace=cr-at-eol\n')
    for name,entry in manifest.items():assert sha(a.published/name)==entry['sha256']
    print(json.dumps({'status':'PASS','manifest_sha256':sha(a.report/'manifest.sha256.json'),'directories':len(folders),'files':sum(len(v) for v in registry.values())}))

if __name__=='__main__':main()
