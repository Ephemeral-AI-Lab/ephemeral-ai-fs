"""Bind an actual offline candidate reader to the existing original-state verifier."""
import argparse,hashlib,importlib.util,json,shutil,sys
from pathlib import Path

def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def load(name,path):
 spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

def main():
 parser=argparse.ArgumentParser();parser.add_argument('--history',type=int,choices=(53,157),required=True);parser.add_argument('--database',type=Path,required=True);parser.add_argument('--reader',type=Path,required=True);parser.add_argument('--expected-sha',required=True);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
 assert not args.output.exists() and sha(args.database)==args.expected_sha
 baseline=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-structural' if args.history==53 else '/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-structural')
 driverpath=baseline/'verification/verify.py' if args.history==53 else Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/verification/verify.py')
 # Seed the correct53/157 baseline module before a reused driver changes import paths.
 sys.path.insert(0,str(baseline/'combined'));__import__('store_api')
 # Load the original driver, then substitute the actual candidate reader explicitly.
 driver=load('ordered_original_driver',driverpath)
 reader=load('ordered_candidate_reader',args.reader)
 args.output.mkdir(parents=True);combined=args.output/'combined';combined.mkdir();verification=args.output/'verification';verification.mkdir()
 (combined/'result.json').write_text(json.dumps({'results':{'delta':{'path':str(args.database.resolve()),'sha256':args.expected_sha}}},indent=2)+'\n')
 shutil.copyfile(baseline/'combined/identity-map.json',combined/'identity-map.json');shutil.copyfile(args.reader,combined/'store_api.py')
 driver.ROOT=args.output
 if args.history==53:
  driver.store_api=reader;driver.v.physical=reader
 else:driver.physical=reader;driver.HERE=verification
 sys.argv=[str(driverpath),'--output',str(verification/'result.json')];driver.main()
 result=json.loads((verification/'result.json').read_text());assert result['status']=='PASS' and result['states']==args.history
 assert result['copy_sha256_before']==result['copy_sha256_after']==args.expected_sha
 # Explicit reader/module and driver hashes bind the substitution rather than relying on an import search path.
 seals=dict(history=args.history,reader_path=str(args.reader),reader_sha256=sha(args.reader),driver_path=str(driverpath),driver_sha256=sha(driverpath),wrapper_sha256=sha(Path(__file__)),database_sha256=sha(args.database),identity_map_sha256=sha(combined/'identity-map.json'),result_sha256=sha(verification/'result.json'))
 (args.output/'custody.json').write_text(json.dumps(seals,indent=2)+'\n')

if __name__=='__main__':main()
