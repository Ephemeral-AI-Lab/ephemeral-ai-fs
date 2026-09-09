"""Reconcile complete selected SmallContent graphs against authenticated current records."""
import collections,json,sqlite3
from pathlib import Path
OUT=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-structural/content')
at=json.loads(Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/content/attribution.json').read_text());old={r['git_oid']:r for r in at['rows']};results={}
for mode in ['forward-small','reverse-small','git-small','git-all','git-small-bounded']:
 db=sqlite3.connect((OUT/(mode+'.sqlite')).as_uri()+'?mode=ro&immutable=1',uri=True);classes=collections.defaultdict(collections.Counter);families=collections.defaultdict(collections.Counter)
 for oid,base,n,size in db.execute('select oid,base,raw,length(frame) from records'):
  if oid not in old:continue
  r=old[oid];before=r['record_bytes']+4;after=size+5+32*bool(base);key=('DELTA' if r['base'] else 'FULL')+' -> '+('DELTA' if base else 'FULL');classes[key].update(objects=1,before=before,after=after,saved=before-after)
  if r.get('full_path_class')=='prior_small_structurally_capped':classes['subset: prior capped FULL'].update(objects=1,before=before,after=after,saved=before-after)
  families[r['path']].update(objects=1,before=before,after=after,saved=before-after)
 db.close();primary=[v for k,v in classes.items() if not k.startswith('subset:')];assert sum(v['objects'] for v in primary)==75398;assert sum(v['before'] for v in primary)==58979700-26608
 results[mode]=dict(classes=dict(classes),net_saved=sum(v['saved'] for v in primary),largest_savings=sorted(families.items(),key=lambda x:-x[1]['saved'])[:20],largest_regressions=sorted(families.items(),key=lambda x:x[1]['saved'])[:20])
(OUT/'comparison.json').write_text(json.dumps(results,indent=2));print(json.dumps({m:{k:v for k,v in r.items() if k in ('classes','net_saved')} for m,r in results.items()},indent=2))
