import json,re,sys
from pathlib import Path
out=Path(__file__).parent;jobs=sys.argv[1];target=out/f'target-j{jobs}'
p=next((target/'cargo-timings').glob('cargo-timing-*.html'));s=p.read_text();decoder=json.JSONDecoder()
def data(name):return decoder.raw_decode(s.split('const '+name+' = ',1)[1])[0]
units=data('UNIT_DATA');concurrency=data('CONCURRENCY_DATA')
end=max(u['start']+u['duration'] for u in units)
ready_wait=0;single=0;active_integral=0
for a,b in zip(concurrency,concurrency[1:]+[{'t':end}]):
 dt=max(0,b['t']-a['t'])
 ready_wait+=dt if a['waiting'] else 0
 single+=dt if a['active']==1 else 0
 active_integral+=dt*a['active']
summary={'jobs':jobs,'end_s':end,'units':len(units),'sum_unit_s':sum(u['duration'] for u in units),'mean_active_units':active_integral/end,'ready_units_waiting_s':ready_wait,'single_active_s':single,'top_units':sorted(units,key=lambda u:-u['duration'])[:20],'product_units':[u for u in units if u['name'].startswith(('layerfs-','fs-benchmark'))]}
(out/f'j{jobs}-timing-data.json').write_text(json.dumps({'summary':summary,'units':units,'concurrency':concurrency},indent=2))
print(json.dumps(summary,indent=2))
