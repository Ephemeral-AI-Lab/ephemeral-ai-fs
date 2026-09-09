import collections,fcntl,gzip,hashlib,json,os,pathlib,shutil,subprocess
r=pathlib.Path(__file__).parent;root=r.parent/'layerfs-issue95';doc=root/'docs/roadmap/0.1/0.1.4/issue95';e=doc/'evidence';full=r/'terminal-full157'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def ref(p):return dict(path=str(p),sha256=sha(p))
with (pathlib.Path(os.environ['TMPDIR'])/'layerfs-infra-measurement.lock').open('a') as lock:
 fcntl.flock(lock,fcntl.LOCK_EX)
 assert subprocess.check_output(['git','diff','c48bb4903','--','crates','tools','benchmark','Cargo.toml','Cargo.lock'],cwd=root)==b''
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
 allocation=acc['filesystem']['store_allocated_bytes']['value'];control=218116096;prior=184586240
 assert allocation<=prior and allocation/control<=.90
 assert inv['inventory_checks']['physical_representation_counts']['NATIVE_PREFIX']==58306
 host=root/'target/release/fs-benchmark-pro';identity=read(pathlib.Path(str(host)+'.identity.json'));assert sha(host)==identity['binary_sha256']
 archive=r/'final-host';archive.mkdir();shutil.copy2(host,archive/host.name);shutil.copy2(str(host)+'.identity.json',archive/(host.name+'.identity.json'))
 gate=dict(status='PASS_FOCUSED_ISSUE95_QUALIFICATION',product_source=identity['LAYERFS_SOURCE_COMMIT'],host=identity,image=(r/'c2-image-id.txt').read_text().strip(),archived_binary=ref(archive/host.name),performance_pass=198,routine_proof_pass=226,optional_not_run=1,native_pass=410,native_ignored_run_separately=True,new_regression_checks=5,full157_performance_states=157,full157_historical_proofs=157,full157_validation_records=158,storage=dict(candidate_allocated_bytes=allocation,control_allocated_bytes=control,prior_candidate_allocated_bytes=prior,reduction_percent=100*(1-allocation/control),change_from_prior_bytes=allocation-prior,candidate_logical_bytes=acc['filesystem']['database_logical_bytes']['value'],page_size_bytes=4096,native_prefix_count=58306,native_full_count=28106,unexplained_page_residual_bytes=0,equal_retained_state_basis='Both arms passed every historical file-state oracle under the unchanged frozen157-state workload. Canonical object populations differ; equality is not inferred from object counts.'),remaining='Unique/scattered publication and encoding costs, mixed-file bounded-working-set misses, original performance gate regressions, and four historical Git fixture comparisons remain.',global_report_status='INCOMPLETE',references={k:ref(p) for k,p in dict(campaign=r/'terminal-benchmark/performance-ledger.json',proofs=r/'terminal-benchmark/verification-ledger.json',checks=r/'terminal-check-qualification.json',supplementals=r/'supplemental-qualification.json',full157=full/'execution/completion.json',resources=full/'resources/resource-timing-summary.json',account=full/'candidate-account/accounting-reconciliation.json',validation=full/'candidate-validation/validation.json',control_reuse=r/'terminal-preparation/control-reuse-applicability.json').items()})
 (r/'terminal-qualification.json').write_text(json.dumps(gate,indent=2)+'\n')
 e.mkdir();manifest=[]
 def copy(p,relative=None):
  rel=pathlib.Path(relative or p.relative_to(r));dst=e/rel;dst.parent.mkdir(parents=True,exist_ok=True)
  data=p.read_bytes();compressed=len(data)>100000 or p.suffix in ['.jsonl'] or p.name in ['profile.txt','report.json']
  if compressed:dst=dst.with_name(dst.name+'.gz');dst.write_bytes(gzip.compress(data,mtime=0))
  else:dst.write_bytes(data)
  manifest.append(dict(raw=ref(p),packaged=str(dst.relative_to(doc)),packaged_sha256=sha(dst),encoding='gzip' if compressed else 'identity'))
 names=['terminal-qualification.json','terminal-build-qualification.json','terminal-check-qualification.json','terminal-correctness-coverage.json','terminal-declaration.json','supplemental-qualification.json','attempts.json','measurement-lock-refusal.json','correctness-review.md','r26-diagnostics.json','terminal-diagnostics.json','terminal-diagnostics.csv','pair01/summary.json','pair01/ledger.json','terminal-benchmark-command.json','terminal-benchmark-result.json','terminal-benchmark/performance-ledger.json','terminal-benchmark/verification-ledger.json','terminal-benchmark/terminal-assessment.json','terminal-benchmark/declaration.json','terminal-benchmark/registry.jsonl','terminal-benchmark-report/report.json','terminal-benchmark-report/comparison.csv','terminal-benchmark-report/performance.csv','terminal-benchmark-report/verification.csv','terminal-full157/execution/completion.json','terminal-full157/resources/resource-timing-summary.json','terminal-full157/resources/field-contract.json','terminal-full157/candidate-account/accounting-reconciliation.json','terminal-full157/candidate-validation/validation.json','terminal-full157/candidate-proof/inventory-proof.json','terminal-full157/candidate-snapshot/custody.json','terminal-full157/candidate-census/completion.json']
 for n in names:copy(r/n)
 for directory in ['terminal-preparation','report-preparation','prework-refusal-repair-01','terminal-native','terminal-large-spill','terminal-doctests','terminal-fmt','terminal-clippy','t01-existing-collision','t02-comparison-reuse','t03-comparison-bound','c1-host']:
  for p in sorted((r/directory).iterdir()):
   if p.is_file() and p.suffix in ['.json','.py','.md','.log','.txt']:copy(p)
 for p in sorted((r/'terminal-full157/execution').iterdir()):
  if p.is_file() and p.name!='completion.json':copy(p)
 for glob in ['p0*-control/*','pair01/*-command.json','supplemental-*/*']:
  for p in sorted(r.glob(glob)):
   if p.is_file() and p.suffix in ['.json','.txt','.log']:copy(p)
 for folder in [r/'pair01',r/'terminal-benchmark/performance',r/'terminal-benchmark/verification']:
  for p in sorted(folder.rglob('*')):
   if p.name not in ['perf.jsonl','verification.json','failure.log']:continue
   if folder.name=='pair01' or any(x in p.parts for x in ['dedup_cdc_locality','dedup_cross_file']) or '.prework-refused-' in str(p):copy(p)
 for directory in ['terminal-small-files','terminal-frequent-edits']:
  for p in sorted((r/directory).glob('*.json')):copy(p)
 for n in ['extract.py','extract_terminal.py','summarize_pairs.py','summarize_pairs_v2.py','pairs.py','locked.py','retry_refused_proofs.py','terminal_checks.py','terminal_benchmark.py','supplementals.py','package.py']:copy(r/n,pathlib.Path('scripts')/n)
 raw_manifest=[]
 for folder in [r/'terminal-benchmark',full/'candidate-run']:
  for p in sorted(folder.rglob('*')):
   if p.is_file() and p.name in ['perf.jsonl','verification.json','performance-result.json','verification-result.json','performance-summary.json','verification-summary.json','performance-manifest.json','verification-manifest.json']:raw_manifest.append(ref(p))
 (e/'raw-references.json').write_text(json.dumps(raw_manifest,indent=2)+'\n')
 (doc/'evidence-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
 print(json.dumps(dict(status=gate['status'],packaged_files=len(manifest),packaged_bytes=sum(p.stat().st_size for p in e.rglob('*') if p.is_file()),storage=gate['storage']),indent=2))
