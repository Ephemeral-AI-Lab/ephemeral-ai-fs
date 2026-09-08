import json,pathlib,subprocess,sys
root=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue90-storage-adoption')
sys.path.insert(0,str(root/'benchmark/fs-bench-pro/shared'))
import runner
mode=sys.argv[1]
args=runner.source_build_args()
tag='layerfs-issue90:'+args['LAYERFS_SOURCE_SEAL'][:16]+('-check' if mode=='check' else '')
cmd=['docker','build','--progress=plain','--file',str(root/'benchmark/fs-bench-pro/Dockerfile.layerfs'),'--tag',tag,'--build-arg','CARGO_BUILD_JOBS=2']
if mode=='check':cmd+=['--target','runtime-check']
for key,value in sorted(args.items()):cmd+=['--build-arg',f'{key}={value}']
cmd.append(str(root))
folder=pathlib.Path(__file__).parent/(sys.argv[2] if len(sys.argv)>2 else 'runtime-'+mode+'-inputs')
folder.mkdir()
(folder/'identity.json').write_text(json.dumps({'source':args,'tag':tag,'command':cmd},indent=2))
subprocess.run(cmd,check=True,timeout=900)
(folder/'image.json').write_bytes(subprocess.check_output(['docker','image','inspect',tag]))
print('PASS',tag,flush=True)
