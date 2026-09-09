"""Run the existing sealed original-oracle verifier with the structural reader."""
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

ROOT=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-structural')
sys.path.insert(0,str(ROOT/'combined'))
import store_api
SOURCE=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/verification/verify.py')
assert hashlib.sha256(SOURCE.read_bytes()).hexdigest()=='a660ba28254caa8db1c43504b0b7be838192c0a9132eb0800b551ffa598cab0d'
spec=importlib.util.spec_from_file_location('sealed_original_oracle_verifier',SOURCE)
verifier=importlib.util.module_from_spec(spec);spec.loader.exec_module(verifier)
assert verifier.physical is store_api
verifier.ROOT=ROOT;verifier.HERE=ROOT/'verification'
if __name__=='__main__':
    verifier.main()
