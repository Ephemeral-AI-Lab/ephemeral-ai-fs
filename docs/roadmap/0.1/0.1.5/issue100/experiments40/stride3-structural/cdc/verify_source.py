from shared import *
started=time.perf_counter_ns();rows=[]
assert len(records)==1998
for id in records:
 raw,info=decode(id);rows.append(dict(id=id,raw_sha256=hashlib.sha256(raw).hexdigest(),**info))
assert sha(STORE)==BEFORE
result=dict(status='PASS',objects=len(rows),store_sha256=BEFORE,records=rows,elapsed_ns=time.perf_counter_ns()-started)
(OUT/'source-native-verification.json').write_text(json.dumps(result,indent=2));print(json.dumps({k:v for k,v in result.items() if k!='records'}))
