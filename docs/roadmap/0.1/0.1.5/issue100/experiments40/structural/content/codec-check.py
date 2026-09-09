"""Check requested zstd parameters reproduce stored same-base frames."""
from pathlib import Path
exec(Path(__file__).with_name('experiment.py').read_text().split('started=time.time();at=')[0])
sys.path.insert(0,'/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/combined')
from store_api import StoreReader,fr
r=StoreReader(Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/combined/D-B-CDC.sqlite'))
a=json.loads(ATTR.read_text())['rows'];byid={x['id']:x for x in a};counts=collections.Counter()
# Fixed every 100th physical record: not a codec search or selected-winner sample.
for row in a[::100]:
 _,rec,_=r._physical(bytes.fromhex(row['id']));kind,n,base,frame=fr.record(rec);target=raw(row['git_oid']);prefix=raw(byid[base.hex()]['git_oid']) if base else b''
 assert encode(target,prefix)==frame,(row['id'],kind)
 counts[str(kind)]+=1
r.close();proc.stdin.close();proc.wait();assert proc.returncode==0
(OUT/'codec-check.json').write_text(json.dumps(dict(status='PASS',sample='every100th source physical record',counts=dict(counts),objects=sum(counts.values())),indent=2));print(counts)
