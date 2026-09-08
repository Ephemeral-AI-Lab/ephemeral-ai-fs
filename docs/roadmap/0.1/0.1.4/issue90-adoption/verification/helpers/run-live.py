import json,os,pathlib,subprocess,sys
root=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue90-storage-adoption')
tag=json.loads((pathlib.Path(__file__).parent/'runtime-image-inputs/identity.json').read_text())['tag']
env=dict(os.environ,LAYERFS_LIVE_DOCKER='1',LAYERFS_LIVE_DOCKER_IMAGE=tag)
cmd=['cargo','+1.85.1','test','-p','layerfs-sdk','--all-features','--locked','--test','live_docker','--','--nocapture','--test-threads=1']
print(json.dumps({'command':cmd,'LAYERFS_LIVE_DOCKER':'1','LAYERFS_LIVE_DOCKER_IMAGE':tag}),flush=True)
sys.exit(subprocess.run(cmd,cwd=root,env=env,timeout=180).returncode)
