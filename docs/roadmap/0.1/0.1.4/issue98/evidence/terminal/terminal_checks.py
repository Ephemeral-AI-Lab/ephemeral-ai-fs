import json,pathlib,subprocess,sys
root=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-workspace-admission');runs=pathlib.Path(__file__).parent
checks=[('native',['env','RUSTUP_TOOLCHAIN=1.85.1','CARGO_BUILD_JOBS=2','tools/test-fast.sh']),('large-spill',['cargo','+1.85.1','test','--locked','-j2','-p','layerfs-layerstack-store','--all-features','parallel_large_spill_matches_legacy_after_fresh_store_reopen','--','--ignored','--test-threads=1']),('doctests',['cargo','+1.85.1','test','--locked','-j2','--doc','-p','layerfs-layerstack-store','-p','layerfs-workspace','--all-features']),('fmt',['cargo','+1.96.0','fmt','--all','--check']),('clippy',['cargo','+1.96.0','clippy','--workspace','--locked','-j2','--','-D','warnings'])]
for name,cmd in checks:
 out=runs/('terminal-'+name)
 r=subprocess.run(['python3',str(runs/'locked.py'),str(out),*cmd],cwd=root)
 print(name,r.returncode,flush=True)
 if r.returncode:sys.exit(r.returncode)
(runs/'terminal-check-qualification.json').write_text(json.dumps(dict(status='PASS',source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),checks=[dict(name=n,command=c,result=json.loads((runs/('terminal-'+n)/'result.json').read_text())) for n,c in checks]),indent=2))
