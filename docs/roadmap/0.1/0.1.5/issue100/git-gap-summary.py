"""Reconcile existing pack evidence; no product execution or alternative encoding."""
import collections,json,sys
from pathlib import Path
root=Path(sys.argv[1]);output=Path(sys.argv[2]);assert not output.exists()
objects={};groups=collections.defaultdict(collections.Counter)
for line in (root/'git-pack-attribution-raw.txt').read_text().splitlines():
 f=line.split()
 if len(f) in (5,7) and len(f[0])==40 and f[1] in ('blob','tree','commit','tag'):
  base=f[6] if len(f)==7 else None
  objects[f[0]]=dict(kind=f[1],packed=int(f[3]),base=base,depth=int(f[5]) if base else 0)
  groups[f[1]+('_delta' if base else '_full')].update(objects=1,packed_bytes=int(f[3]))
first={};fixture=json.loads((root/'fixture.json').read_text())['deepseek-ten']
for state in fixture['states']:
 for line in (Path(state['input'])/'manifest.tsv').read_text().splitlines():
  mode,oid,size,path=line.split('\t');first.setdefault(oid,state['index'])
closure={}
def available(oid):
 if oid not in closure:closure[oid]=max(first[oid],available(objects[oid]['base']) if objects[oid]['base'] else 0)
 return closure[oid]
availability=collections.defaultdict(collections.Counter)
for oid,o in objects.items():
 if o['kind']!='blob' or not o['base']:continue
 when=available(o['base']);own=first[oid]
 category='earlier_checkpoint_closure' if when<own else 'same_checkpoint_closure' if when==own else 'later_checkpoint_required'
 availability[category].update(objects=1,packed_bytes=o['packed'])
old=json.loads((root/'git-pack-attribution.json').read_text())
assert dict(groups)==old['groups'] and dict(availability)==old['blob_delta_base_availability']
git=json.loads((root/'git/results.json').read_text());assert sum(o['packed'] for o in objects.values())+32==git['phases']['pack_delta']['storage']['pack_bytes']
arm=json.loads((root/'comparison-v014-v015.json').read_text())['arms'][1];c=arm['census']['counts']
lfs_content=c['pack_v2']['bytes']+c['pack_v3']['bytes'];lfs_metadata=c['pack_v1']['bytes'];lfs_total=arm['metrics']['final_allocated_bytes']
git_content=groups['blob_full']['packed_bytes']+groups['blob_delta']['packed_bytes'];git_metadata=groups['tree_full']['packed_bytes']+groups['tree_delta']['packed_bytes']+groups['commit_full']['packed_bytes'];git_total=git['phases']['pack_delta']['storage']['allocated_bytes']
buckets={}
for name,l,g in [('content',lfs_content,git_content),('metadata',lfs_metadata,git_metadata),('other',lfs_total-lfs_content-lfs_metadata,git_total-git_content-git_metadata)]:buckets[name]=dict(layerfs=l,git=g,gap=l-g)
assert sum(b['gap'] for b in buckets.values())==lfs_total-git_total
result=dict(scope='Byte-exact comparison of existing ten-state evidence, not an optimization forecast',buckets=buckets,layerfs_total=lfs_total,git_total=git_total,target_bytes=40000000,required_saving=lfs_total-40000000,git_groups=dict(groups),git_delta_availability=dict(availability),small_full_mapping=json.loads((root/'small-full-git-attribution.json').read_text())['groups'],boundary_example=json.loads((root/'online-candidate-example.json').read_text()))
with output.open('x') as f:json.dump(result,f,indent=2)
print(json.dumps(buckets,indent=2))
