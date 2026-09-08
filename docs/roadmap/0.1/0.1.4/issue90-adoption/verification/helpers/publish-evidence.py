import gzip,hashlib,json,pathlib,shutil
runs=pathlib.Path(__file__).parent
out=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue90-storage-adoption/docs/roadmap/0.1/0.1.4/issue90-adoption/verification')
out.mkdir()
patches={}
for source in sorted(runs.iterdir()):
 if not source.is_dir() or not (source/'result.json').exists():continue
 target=out/source.name;target.mkdir()
 for file in sorted(source.iterdir()):
  if not file.is_file():continue
  if file.name=='source.patch':
   data=file.read_bytes();(target/'source.patch.gz').write_bytes(gzip.compress(data,mtime=0));patches[source.name]=hashlib.sha256(data).hexdigest()
  else:shutil.copy2(file,target/file.name)
(out/'dirty-patch-sha256.json').write_text(json.dumps(patches,indent=2)+'\n')
shutil.copy2(runs/'final-identity.json',out/'identity.json')
helpers=out/'helpers';helpers.mkdir()
for name in ['run.py','build-runtime.py','run-live.py','run-live-ready.py','final-quality.py','identity.py','publish-evidence.py']:
 shutil.copy2(runs/name,helpers/name)
shutil.copytree(runs/'compatibility-probe',helpers/'compatibility-probe')
shutil.copytree(runs/'old-binary-01/probe',helpers/'initial-compatibility-probe')
for name in ['runtime-check-inputs','runtime-check-inputs-02','runtime-image-inputs']:
 shutil.copytree(runs/name,out/name)
# Exact frozen invocations are in each command/result/source receipt. Early
# python- wrapper stdin was not persisted; preserve that limitation explicitly.
(out/'README.md').write_text('''# Verification receipts\n\nFinal correctness/quality receipts bind clean source593f4ad018bf34b3f180baf66e1ae5cf40c36647.\n`identity.json` records exact source/tree, empty dirty patch, binaries, image,\ncompiler/dependency identities, host and topology. `manifest.json` hashes these\nretained files. Gzip patches decompress to the SHA256 values in\n`dirty-patch-sha256.json`; original local files remain unmodified.\n\nEvery failed/partial development attempt is retained. Early development\n`python3 -` commands did not persist wrapper stdin, and early dirty patches\ncover tracked files only; new test files were first fully sealed atc6e2d5901.\nTheir logs are diagnostic history, not exact-final-candidate qualification.\nThe final checks use clean committed source and directly named commands or the\nfrozen helper files here. No historical #88 Store/evidence was modified.\n\nThe initial old-code probe used Empty initialization. The final probe adds\n8192-byte Directory files, full older/newer reader checks, and observed native\nadmission before old-open rejection. Its separate Cargo.lock is retained; it\nis actual older source linked into a verifier, not an archived shipping binary.\n\nDocker's missing-socket attempts remain failures. The backend quit log identifies\nan explicit GUI /app/quit; runtime-check-final-02 and live-final-02 succeeded\nafter restoring Docker. No failed run was relabeled as passing.\n''')
files={str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.rglob('*')) if p.is_file()}
(out/'manifest.json').write_text(json.dumps(files,indent=2)+'\n')
print('files',len(files),'bytes',sum(p.stat().st_size for p in out.rglob('*') if p.is_file()))
