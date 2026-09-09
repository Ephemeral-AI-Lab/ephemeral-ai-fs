import json,collections
from pathlib import Path
p=Path(__file__).parent;r=json.loads((p/'attribution.json').read_text());caps=collections.defaultdict(collections.Counter);cross=collections.defaultdict(collections.Counter);families=collections.defaultdict(collections.Counter);depth=collections.defaultdict(collections.Counter);kinds=collections.defaultdict(collections.Counter)
for x in r['rows']:
 depth[str(x['depth'])].update(objects=1,frame_bytes=x['frame_bytes'])
 kinds[str(x['kind'])+' / '+x['git_category']].update(objects=1,frame_bytes=x['frame_bytes'],git_entry_bytes=x['git_entry_bytes'])
 if x.get('full_path_class')=='prior_small_structurally_capped':
  key='+'.join(sorted({v for c in x['prior_small_checks'] for v in c['limits']}));caps[key].update(objects=1,frame_bytes=x['frame_bytes']);cross[key+' / '+x['git_category']].update(objects=1,frame_bytes=x['frame_bytes'],git_entry_bytes=x['git_entry_bytes']);families[x['path']].update(objects=1,frame_bytes=x['frame_bytes'])
result=dict(cap_classes=dict(caps),cap_git_cross=dict(cross),small_depths=dict(depth),kind_git_classes=dict(kinds),largest_capped_families=sorted((dict(path=k,**v) for k,v in families.items()),key=lambda x:-x['frame_bytes'])[:25])
checks=[c for row in r['rows'] if row.get('full_path_class')=='prior_small_structurally_capped' for c in row['prior_small_checks']]
maxima=dict(maximum_capped_prior_chain={k:max(c[k] for c in checks) for k in ['depth','canonical_closure','encoded_closure']},maximum_git_depth_same_small_objects=max(x['git_depth'] for x in r['rows']))
(p/'cap-maxima.json').write_text(json.dumps(maxima,indent=2))
assert sum(v['objects'] for v in caps.values())==1528 and sum(v['frame_bytes'] for v in caps.values())==9254960
(p/'cap-summary.json').write_text(json.dumps(result,indent=2));print(json.dumps(dict(cap_classes=result['cap_classes'],kind_git_classes=result['kind_git_classes']),indent=2))
