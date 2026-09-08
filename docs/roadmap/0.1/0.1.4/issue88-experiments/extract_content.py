#!/usr/bin/env python3
"""Frozen source manifests to shared S2/S3 targets. Exact product CDC, no encoder."""
import hashlib,json,pathlib,sqlite3,subprocess,sys,time

def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def main(run,tool,out):
    out.mkdir(exist_ok=False);db=sqlite3.connect(out/'extraction.sqlite')
    db.executescript('pragma cache_size=-8192; create table blobs(id text primary key,path text,sha text,size integer,chunks text); create table selected(arm text,id text,checkpoint integer,primary key(arm,id));')
    p=subprocess.Popen([str(tool)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
    d=json.loads((run/'deepseek-full/performance-result.json').read_text());prev={};counts={'s2':0,'s3':0};totalbytes={'s2':0,'s3':0};start=time.monotonic_ns()
    files={a:(out/(a+'-targets.jsonl')).open('x') for a in counts}
    def blob(oid,source,seals):
        known=db.execute('select path,sha,size,chunks from blobs where id=?',(oid,)).fetchone()
        if known:return known[0],known[1],known[2],json.loads(known[3])
        path=source/'blobs'/oid;raw=path.read_bytes();digest=hashlib.sha256(raw).hexdigest()
        assert digest==seals['blobs/'+oid]
        assert hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==oid
        p.stdin.write(str(path)+'\n');p.stdin.flush();line=p.stdout.readline();assert line!=''
        chunks=[]
        for word in line.strip().split(';'):
            if not word:continue
            ident,offset,length=word.split(',');offset=int(offset);length=int(length)
            chunks.append({'id':ident,'offset':offset,'length':length,'sha256':hashlib.sha256(raw[offset:offset+length]).hexdigest()})
        assert sum(c['length'] for c in chunks)==len(raw)
        db.execute('insert into blobs values(?,?,?,?,?)',(oid,str(path),digest,len(raw),json.dumps(chunks)))
        return str(path),digest,len(raw),chunks
    def emit(arm,unit,path,prior,cp,kind):
        ident=kind+':'+unit['id']
        if db.execute('select 1 from selected where arm=? and id=?',(arm,ident)).fetchone():return
        priorid=None if prior is None else prior[0]+':'+prior[1]['id']
        if priorid:
            found=db.execute('select checkpoint from selected where arm=? and id=?',(arm,priorid)).fetchone()
            if not found or found[0]>=cp:priorid=None
        row={'id':ident,'sha256':unit['sha256'],'checkpoint':cp,'ordinal':counts[arm],'source_path':path,'offset':unit['offset'],'length':unit['length'],'prior_id':priorid,'prior_provenance':None if priorid is None else 'samepath previous checkpoint; first overlapping prior chunk or wholefile; only selected earlier checkpoint','unit_kind':kind,'canonical_length':unit['length']+21 if kind=='chunk' else None,'existing_canonical_id':unit['id'] if kind=='chunk' else None}
        files[arm].write(json.dumps(row,separators=(',',':'))+'\n');counts[arm]+=1;totalbytes[arm]+=unit['length'];db.execute('insert into selected values(?,?,?)',(arm,ident,cp))
    for s in d['records']:
        if __import__('shutil').disk_usage(out).free<50*1024**3:raise RuntimeError('free disk reserve')
        source=pathlib.Path(s['input']);mp=source/'manifest.tsv';assert sha(mp)==s['input_seal']['manifest.tsv'];cur={}
        for line in mp.read_text().splitlines():
            mode,oid,size,pathhex=line.split('\t');cur[pathhex]=(mode,oid,int(size))
        for pathhex,(mode,oid,size) in sorted(cur.items()):
            if mode=='120000':continue
            if prev.get(pathhex)==(mode,oid,size):continue
            path,digest,length,chunks=blob(oid,source,s['input_seal']);assert size==length
            old=prev.get(pathhex);prior=None
            if old and old[0]!='120000':
                prior=db.execute('select path,sha,size,chunks from blobs where id=?',(old[1],)).fetchone();assert prior,'prior source blob absent'
            oldchunks=[] if prior is None else json.loads(prior[3])
            for c in chunks:
                pc=next((x for x in oldchunks if x['offset']<c['offset']+c['length'] and x['offset']+x['length']>c['offset']),None)
                emit('s2',c,path,None if pc is None else ('chunk',pc),s['index'],'chunk')
            if length<=262144:
                unit={'id':digest,'sha256':digest,'offset':0,'length':length}
                pr=None if prior is None or prior[2]>262144 else ('file',{'id':prior[1]})
                emit('s3',unit,path,pr,s['index'],'file')
            else:
                eligible_oldchunks=oldchunks if prior is not None and prior[2]>262144 else []
                for c in chunks:
                    pc=next((x for x in eligible_oldchunks if x['offset']<c['offset']+c['length'] and x['offset']+x['length']>c['offset']),None)
                    emit('s3',c,path,None if pc is None else ('chunk',pc),s['index'],'chunk')
        prev=cur;db.commit()
        assert time.monotonic_ns()-start<4*3600*10**9
    for f in files.values():f.close()
    p.stdin.close();assert p.wait()==0
    summary={'status':'PASS','scope':'authenticated source blobs; exact existing FastCDC and canonicalchunkID; firstglobalCAS and strict priorcheckpoint base','counts':counts,'payload_bytes':totalbytes,'source_blobs_count':db.execute('select count(*) from blobs').fetchone()[0],'elapsed_ns':time.monotonic_ns()-start,'binary_sha256':sha(tool),'input_performance_sha256':sha(run/'deepseek-full/performance-result.json'),'manifest_sha256':{a:sha(out/(a+'-targets.jsonl')) for a in counts},'limitations':['Does not claim originalproducer hint selection; explicit offline samepath/span candidate policy','S2 excludes5metadata-valuechunks;S3 includes emptywholefiles; sourceblobpaths stay immutable','S3large fallbacksameCDCunits, chunk/fileidentity domains separate','sourcecanon structure regenerations excluded, payloadscreen only']}
    db.close();(out/'extraction-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
if __name__=='__main__':main(*map(pathlib.Path,sys.argv[1:]))
