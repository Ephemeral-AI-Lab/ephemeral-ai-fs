from pathlib import Path
import collections,fcntl,gzip,hashlib,json,os,subprocess
r=Path(__file__).parent
root=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-workspace-admission')
doc=root/'docs/roadmap/0.1/0.1.4/issue98'
full=r/'terminal-full157'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def ref(p):return {'path':str(p),'sha256':sha(p)}
with (Path(os.environ['TMPDIR'])/'layerfs-infra-measurement.lock').open('a') as lock:
 fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
 assert subprocess.check_output(['git','diff','9cfb4be47','--','crates','tools','benchmark','Cargo.toml','Cargo.lock'],cwd=root)==b''
 perf=read(r/'terminal-benchmark/performance-ledger.json');proof=read(r/'terminal-benchmark/verification-ledger.json')
 assert len(perf)==198 and all(x['status']=='PASS' for x in perf)
 assert collections.Counter(x['status'] for x in proof)=={'PASS':226,'NOT_RUN_OPTIONAL':1}
 resource=read(full/'resources/resource-timing-summary.json');acc=read(full/'candidate-account/accounting-reconciliation.json');val=read(full/'candidate-validation/validation.json');inv=read(full/'candidate-proof/inventory-proof.json')
 assert read(full/'execution/completion.json')['status']=='PASS'
 assert resource['status']=='PASS' and not resource['invalid_counter_or_residual']
 assert acc['status']==val['status']==inv['status']=='PASS' and val['checkpoint_count']==158 and val['errors']==[]
 assert inv['all_references_valid'] and inv['all_selected_ids_authenticated'] and inv['native_dependencies_authenticated_by_completed_decoder']
 assert acc['sqlite']['page_size_bytes']['value']==4096 and acc['sqlite']['unexplained_page_residual_bytes']['value']==0
 for phase,count in [('performance',158),('verification',157)]:
  x=resource['arms']['candidate'][phase];assert x['status']==x['cleanup_status']=='PASS' and x['container_removed'] and x['checkpoint_rows_count']==count
  assert all(x['container_final_observed'][k]==0 for k in ['oom','oom_kill','swap_current'])
 allocation=acc['filesystem']['store_allocated_bytes']['value'];control=218116096;prior=184582144
 assert allocation<=prior and allocation/control<=.90
 representations=inv['inventory_checks']['physical_representation_counts']
 assert representations['NATIVE_PREFIX']==58306 and representations['NATIVE_FULL']==28106
 venv=read(r/'terminal-venv-02/results.json');assert venv['status']=='PASS' and len(venv['samples'])==3 and all(x['verification']=='PASS' for x in venv['samples'])
 report=read(r/'terminal-benchmark-report/report.json')
 assert report['status']=='INCOMPLETE' and len(report['errors'])==4 and all('git_tool_workflow' in e and 'prepared fixture content differs' in e for e in report['errors'])
 build=read(r/'terminal-build-qualification.json')
 gate={'status':'PASS_ISSUE98_QUALIFICATION','product_source':build['source'],'host':build['host'],'image':build['image_id'],'archived_binary':build['archived_binary'],'performance_pass':198,'routine_proof_pass':226,'optional_not_run':1,'native_pass':411,'native_ignored_run_separately':True,'strengthened_boundary_test_pass':True,'full157_performance_states':157,'full157_historical_proofs':157,'full157_validation_records':158,'venv_workloads_and_full_proofs':3,'storage':{'candidate_allocated_bytes':allocation,'control_allocated_bytes':control,'prior_candidate_allocated_bytes':prior,'reduction_percent':100*(1-allocation/control),'change_from_prior_bytes':allocation-prior,'candidate_logical_bytes':acc['filesystem']['database_logical_bytes']['value'],'page_size_bytes':4096,'native_prefix_count':58306,'native_full_count':28106,'unexplained_page_residual_bytes':0,'equal_retained_state_basis':'Both arms passed all historical file-state oracles under the unchanged frozen workload. Equality is not inferred from canonical object counts.'},'global_report_status':report.get('status'),'global_report_errors':report.get('errors',[]),'remaining':'FUSE extraction and necessary construction/encoding/publication costs remain; original historical regressions and four image-bound Git comparisons are not reclassified as passes.','references':{k:ref(p) for k,p in {'campaign':r/'terminal-benchmark/performance-ledger.json','proofs':r/'terminal-benchmark/verification-ledger.json','checks':r/'terminal-final-check-qualification.json','supplementals':r/'supplemental-qualification.json','full157':full/'execution/completion.json','resources':full/'resources/resource-timing-summary.json','account':full/'candidate-account/accounting-reconciliation.json','validation':full/'candidate-validation/validation.json','control_reuse':r/'terminal-preparation-02/control-reuse-applicability.json','venv':r/'terminal-venv-02/results.json','comparison':r/'terminal-benchmark-report/report.json'}.items()}}
 (r/'terminal-qualification.json').write_text(json.dumps(gate,indent=2)+'\n')
 e=doc/'evidence/terminal';e.mkdir();manifest=[]
 def copy(p):
  rel=p.relative_to(r);dst=e/rel;dst.parent.mkdir(parents=True,exist_ok=True);data=p.read_bytes();compressed=len(data)>100000 or p.suffix in ['.jsonl','.log']
  if compressed:dst=dst.with_name(dst.name+'.gz');dst.write_bytes(gzip.compress(data,mtime=0))
  else:dst.write_bytes(data)
  manifest.append({'raw':ref(p),'packaged':str(dst.relative_to(doc)),'packaged_sha256':sha(dst),'encoding':'gzip' if compressed else 'identity'})
 names=['terminal-qualification.json','terminal-build-qualification.json','terminal-check-qualification.json','terminal-final-check-qualification.json','terminal-declaration.json','supplemental-qualification.json','preparation-attempts.json','remaining-pipeline.json','terminal-venv-binary.json','terminal-venv.log','terminal-venv-02.log','terminal-benchmark-command.json','terminal-benchmark-result.json','terminal-benchmark/performance-ledger.json','terminal-benchmark/verification-ledger.json','terminal-benchmark/terminal-assessment.json','terminal-benchmark/declaration.json','terminal-benchmark/registry.jsonl','terminal-benchmark-report/report.json','terminal-benchmark-report/comparison.csv','terminal-benchmark-report/performance.csv','terminal-benchmark-report/verification.csv','terminal-full157/execution/completion.json','terminal-full157/resources/resource-timing-summary.json','terminal-full157/resources/field-contract.json','terminal-full157/candidate-account/accounting-reconciliation.json','terminal-full157/candidate-validation/validation.json','terminal-full157/candidate-proof/inventory-proof.json','terminal-full157/candidate-snapshot/custody.json','terminal-full157/candidate-census/completion.json']
 for n in names:copy(r/n)
 for directory in ['terminal-preparation','terminal-preparation-02','report-preparation','report-execution','terminal-native','terminal-large-spill','terminal-doctests','terminal-fmt','terminal-clippy','terminal-venv','terminal-venv-02','terminal-full157/execution']:
  for p in sorted((r/directory).iterdir()):
   if p.is_file() and p.suffix in ['.json','.py','.md','.log','.txt'] and p.name!='completion.json':copy(p)
 for directory in ['terminal-small-files','terminal-frequent-edits']:
  for p in sorted((r/directory).glob('*.json')):copy(p)
 for p in sorted(r.glob('supplemental-*/*')):
  if p.is_file() and p.suffix in ['.json','.log']:copy(p)
 for folder in [r/'terminal-benchmark/performance',r/'terminal-benchmark/verification']:
  for p in sorted(folder.rglob('*')):
   if p.is_file() and p.name in ['perf.jsonl','verification.json','failure.log']:copy(p)
 for n in ['locked.py','terminal_checks.py','terminal_benchmark.py','supplementals.py','finish_pipeline.py','terminal-venv-run.py','terminal-venv-run-02.py','run-pair.py','run-reversed-pair.py','run-spill-pair.py','run-combined-pair.py','profile.py','package_final.py']:copy(r/n)
 for n in ['review-focused-01.log','review-clippy-01.log','terminal-venv-build.log','terminal-venv-build-command.json','terminal-venv-command.json']:copy(r/n)
 raw_refs=[]
 for folder in [r/'terminal-benchmark',full/'candidate-run']:
  for p in sorted(folder.rglob('*')):
   if p.is_file() and p.name in ['perf.jsonl','verification.json','performance-result.json','verification-result.json','performance-summary.json','verification-summary.json','performance-manifest.json','verification-manifest.json']:raw_refs.append(ref(p))
 (e/'raw-references.json').write_text(json.dumps(raw_refs,indent=2)+'\n')
 (doc/'terminal-evidence-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
 print(json.dumps({'status':gate['status'],'packaged_files':len(manifest),'packaged_bytes':sum(p.stat().st_size for p in e.rglob('*') if p.is_file()),'storage':gate['storage']},indent=2))
