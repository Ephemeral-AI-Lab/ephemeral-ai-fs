#!/usr/bin/env python3
"""Audit measured offline artifacts and emit honest per-treatment comparisons."""
import csv,hashlib,json,pathlib,sqlite3,sys

def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def main(runs,out):
 result={'status':'PASS','scope':'offline encoded potential; no combined/full allocated Store claim','units':'integerbytes/ns/counts; explicit ratios','screens':{}}
 s1dir=runs/'issue88-s1-bb36062-1';s1=json.loads((s1dir/'result.json').read_text());rows=list(csv.DictReader((s1dir/'groups.csv').open()))
 assert len(rows)==9918 and sum(int(r['records_count']) for r in rows)==279724
 assert sum(int(r['A_encoded_bytes']) for r in rows)==s1['A_encoded_bytes']
 assert sum(int(r['selected_encoded_bytes']) for r in rows)==s1['selected_encoded_bytes']
 assert (s1dir/'candidate-groups.bin').stat().st_size==s1['selected_encoded_bytes']
 assert len(list(csv.DictReader((s1dir/'tree-reconstruction.csv').open())))==157
 result['screens']['S1']=s1
 for arm in ['S2','S3']:
  p=runs/f'issue88-{arm.lower()}-b9abb13-1';d=json.loads((p/'result.json').read_text());t=d['totals'];assert d['status']=='PASS'
  for n,info in d['files'].items():assert sha(p/n)==info['sha256'] and (p/n).stat().st_size==info['logical_bytes']
  for cp,h in d['contract_sha256'].items():assert sha(pathlib.Path(cp))==h
  assert sha(pathlib.Path(d['identity']['library_path']))==d['identity']['library_sha256']
  assert d['manifest_sha256']==sha(runs/('issue88-content-inputs-2/'+arm.lower()+'-targets.jsonl'))
  assert t['targets']==t['full_selected']+t['prefix_selected']
  c=sqlite3.connect((p/'index.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True)
  assert c.execute('select count(*) from objects').fetchone()[0]==t['targets']
  assert c.execute('select count(*) from objects o join objects b on o.base_id=b.id where b.checkpoint>=o.checkpoint').fetchone()[0]==0
  maxdepth,maxclosure=c.execute('select max(depth),max(closure_bytes) from objects').fetchone();assert maxdepth<=4 and maxclosure<=1048576;c.close()
  full=d['files']['full.frames']['logical_bytes'];selected=d['files']['selected.frames']['logical_bytes']
  assert full==16+56*t['targets']+t['full_frame_bytes']
  assert selected==16+56*t['targets']+32*t['prefix_selected']+t['selected_frame_bytes']
  result['screens'][arm]={'status':'PASS','targets_count':t['targets'],'raw_payload_bytes':t['target_bytes'],'control_complete_frame_bytes':full,'candidate_complete_frame_bytes':selected,'encoded_difference_bytes':full-selected,'prefix_selected_count':t['prefix_selected'],'max_edges':maxdepth,'max_closure_payload_bytes':maxclosure,'analysis_index_bytes':d['files']['index.sqlite']['logical_bytes'],'analysis_index_scope':'joint control/candidate diagnostic index, not productmetadata; do notchargeasbotharmsindependentproductionindexes','resources':d['resources'],'codec_work_ns':{k:v for k,v in t.items() if k.endswith('_ns')},'canonical':d['canonical'],'read_probes':d['read_probes'],'artifact_result_sha256':sha(p/'result.json')}
 s2=result['screens']['S2'];s3=result['screens']['S3'];result['granularity_increment']={'S3_fewer_frame_bytes_than_S2':s2['candidate_complete_frame_bytes']-s3['candidate_complete_frame_bytes'],'S3_more_raw_payload_bytes':s3['raw_payload_bytes']-s2['raw_payload_bytes'],'interpretation':'small encodedgain vsnewcanonicalunits; wrapper/index regeneration unmeasured; no combinedclaim'}
 with out.open('x') as f:json.dump(result,f,indent=2);f.write('\n')
 print(json.dumps(result['granularity_increment']))
if __name__=='__main__':main(*map(pathlib.Path,sys.argv[1:]))
