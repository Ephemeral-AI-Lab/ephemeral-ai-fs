#!/usr/bin/env python3
"""Serialize experiment/build commands with the existing measurement lock."""
import fcntl,os,pathlib,subprocess,sys
with (pathlib.Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock').open('a') as lock:
    fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
    result=subprocess.run(sys.argv[2:],timeout=int(sys.argv[1]))
    sys.exit(result.returncode)
