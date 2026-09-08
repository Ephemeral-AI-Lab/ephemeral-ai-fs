import subprocess,sys
for cmd in [
 ['cargo','+1.96.0','fmt','--all','--check'],
 ['cargo','+1.96.0','clippy','--workspace','--locked','--','-D','warnings'],
 ['git','diff','--check'],
]:
 print('RUN',repr(cmd),flush=True)
 p=subprocess.run(cmd,timeout=180)
 if p.returncode:sys.exit(p.returncode)
print('PASS exact-candidate formatting, warning-denying Clippy, diff check')
