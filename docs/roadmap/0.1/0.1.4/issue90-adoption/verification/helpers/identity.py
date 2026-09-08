import hashlib,json,pathlib,platform,subprocess,sys
root=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue90-storage-adoption')
runs=pathlib.Path(__file__).parent
sys.path.insert(0,str(root/'benchmark/fs-bench-pro/shared'))
import runner

def sha(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  for block in iter(lambda:f.read(1024*1024),b''):h.update(block)
 return h.hexdigest()
def out(cmd):return subprocess.check_output(cmd,text=True).strip()
files=[root/'target/debug'/name for name in ['layerfs','layerfs-daemon','layerfs-fuse','fs-benchmark-pro','layerfs-eval','issue90-compatibility-probe']]
for directory,pattern in [(root/'target/debug/deps','layerfs_layerstack_store-*'),(root/'target/debug/deps','live_docker-*')]:
 files += [p for p in directory.glob(pattern) if p.is_file() and '.' not in p.name and p.stat().st_mode&0o111]
image=json.loads((runs/'runtime-image-inputs/image.json').read_text())[0]
identity={
 'source':runner.source_build_args(),
 'source_dirty_patch_sha256':hashlib.sha256(subprocess.check_output(['git','diff','HEAD','--binary'],cwd=root)).hexdigest(),
 'host':{'platform':platform.platform(),'machine':platform.machine(),'cpu':out(['sysctl','-n','machdep.cpu.brand_string']),'memory_bytes':int(out(['sysctl','-n','hw.memsize'])),'logical_cpus':int(out(['sysctl','-n','hw.logicalcpu']))},
 'build':{'native_toolchain':out(['rustc','+1.85.1','-Vv']),'quality_toolchain':out(['rustc','+1.96.0','-Vv']),'native_profile':'debug; --locked; all-features for workspace tests','linux_profile':'release aarch64-unknown-linux-gnu, Rust1.85.1, Cargo jobs2; proxy FUSE','root_cargo_lock_sha256':sha(root/'Cargo.lock'),'schema7_sha256':sha(root/'crates/layerfs-layerstack-store/sql/schema/v7.sql')},
 'binaries':{str(p):sha(p) for p in files if p.exists()},
 'runtime_image':{'id':image['Id'],'tags':image['RepoTags'],'labels':image['Config'].get('Labels'),'architecture':image['Architecture'],'os':image['Os']},
 'older_probe':{'old_source_commit':out(['git','-C','/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue88-combined-control','rev-parse','HEAD']),'old_source_status':out(['git','-C','/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue88-combined-control','status','--porcelain']),'scope':'Actual older source linked into a separate verifier; 8192-byte file Directory imports, full old/new readback, schema7 native rejection unchanged, schema6 alternating writes. Not an archived shipping binary.','lock_sha256':sha(runs/'compatibility-probe/Cargo.lock'),'source_sha256':sha(runs/'compatibility-probe/src/main.rs')},
 'topology':{'store':'macOS host','sdk':'macOS host','daemon_fuse_workload':'Docker Linux arm64','live_container_cpus':2,'live_container_memory_bytes':2147483648,'live_container_pids':256},
}
assert identity['source']['LAYERFS_SOURCE_DIRTY']=='false'
assert image['Config']['Labels']['dev.layerfs.product-seal']==identity['source']['LAYERFS_PRODUCT_SEAL']
(runs/'final-identity.json').write_text(json.dumps(identity,indent=2)+'\n')
print(json.dumps(identity,indent=2))
