#!/usr/bin/env python3
import hashlib,json,pathlib,sqlite3,sys,time,statistics
from content_screen import Zstd,read_exact,RECORD_HEADER,reconstruct,validate_file_header,sha256_file

def main(runs,out):
    out.mkdir(exist_ok=False);results=[]
    for arm in ['s2','s3']:
        folder=runs/f'issue88-{arm}-b9abb13-1';d=json.loads((folder/'result.json').read_text())
        for n,info in d['files'].items():assert sha256_file(folder/n)==info['sha256']
        index=sqlite3.connect((folder/'index.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True);index.row_factory=sqlite3.Row
        ids=[]
        for order in ['ordinal','length DESC,ordinal','closure_bytes DESC,ordinal']:
            row=index.execute('SELECT * FROM objects ORDER BY '+order+' LIMIT 1').fetchone();ids.append(row)
        with (folder/'full.frames').open('rb') as full,(folder/'selected.frames').open('rb') as selected:
            validate_file_header(full);validate_file_header(selected)
            for label,row in zip(['first','largest','largest_closure'],ids):
                for mode in ['small_range','full']:
                    for repeat in range(3):
                        for repr in (['FULL','selected'] if repeat%2==0 else ['selected','FULL']):
                            codec=Zstd(d['identity']['library_path']);stats={'frame_reads':0,'encoded_bytes':0,'decoded_bytes':0};t=time.perf_counter_ns()
                            if repr=='selected':raw=reconstruct(index,selected,row['id'],codec,4,1048576,stats)
                            else:
                                header=read_exact(full,row['full_offset'],56);kind,depth,res,encoded,length,closure,digest=RECORD_HEADER.unpack(header)
                                assert kind==depth==res==0 and length==closure==row['length'] and encoded+56==row['full_length']
                                frame=read_exact(full,row['full_offset']+56,encoded);raw=codec.decompress(frame,length);assert hashlib.sha256(raw).digest()==digest
                                stats={'frame_reads':1,'encoded_bytes':56+encoded,'decoded_bytes':length}
                            answer=raw[:min(4096,len(raw))] if mode=='small_range' else raw;elapsed=time.perf_counter_ns()-t
                            assert hashlib.sha256(raw).hexdigest()==row['sha256'];codec.close()
                            results.append({'arm':arm,'representative':label,'mode':mode,'repeat':repeat,'representation':repr,'elapsed_ns':elapsed,'returned_bytes':len(answer),'unit_bytes':len(raw),'depth':row['depth'],'closure_bytes':row['closure_bytes'],'work':stats})
        index.close()
    d={'status':'PASS','scope':'fixed matched offline decoder read costs; fresh applicationcache, OSuncontrolled; notpubliclatency orstatisticalgeneralization','units':'integerns/bytes/counts','rows':results}
    (out/'read-costs.json').write_text(json.dumps(d,indent=2)+'\n')
    for arm in ['s2','s3']:
        for label in ['first','largest','largest_closure']:
            pair={r:statistics.median(x['elapsed_ns'] for x in results if x['arm']==arm and x['representative']==label and x['mode']=='small_range' and x['representation']==r) for r in ['FULL','selected']}
            print(arm,label,pair)
if __name__=='__main__':main(*map(pathlib.Path,sys.argv[1:]))
