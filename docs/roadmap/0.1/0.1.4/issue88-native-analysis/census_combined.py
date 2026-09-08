#!/usr/bin/env python3
"""Run the frozen decoder once under the shared measurement lock and seal its exit."""
import argparse, fcntl, hashlib, json, os, pathlib, subprocess, time

def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('schedule', type=pathlib.Path)
    parser.add_argument('arm', choices=('control','candidate'))
    parser.add_argument('snapshot', type=pathlib.Path)
    parser.add_argument('output', type=pathlib.Path)
    args = parser.parse_args(); frozen=json.loads(args.schedule.read_text())
    binary=pathlib.Path(frozen['census']['binary']); assert sha(binary)==frozen['census']['sha256']
    snapshot=args.snapshot/'store.sqlite'; custody=args.snapshot/'custody.json'
    initial_custody=sha(custody)
    arm=next(a for a in frozen['order'] if a['arm']==args.arm)
    prior=json.loads(custody.read_text())
    assert prior['source']==str(pathlib.Path(arm['output'])/'deepseek-full/host-runtime/store.sqlite')
    assert prior['frozen_schedule_sha256']==sha(args.schedule)
    assert prior['status']=='PASS' and prior['snapshot_phase']=='final-pre-verification'
    assert sha(snapshot)==prior['copy_sha256']==prior['source_sha256']
    with (pathlib.Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        handles=subprocess.run(['lsof','-t','--',str(snapshot)],capture_output=True,text=True,timeout=30)
        assert handles.returncode==1 and not handles.stdout and not handles.stderr
        args.output.mkdir(); log=args.output/'census.log'; command=[str(binary),str(snapshot),str(args.output)]
        start=time.monotonic_ns()
        error=None
        with log.open('xb') as stream:
            try:
                proc=subprocess.run(command,stdout=stream,stderr=subprocess.STDOUT,timeout=14400)
                exit_code=proc.returncode
            except subprocess.TimeoutExpired as failure:
                error=str(failure); exit_code=124
        elapsed=time.monotonic_ns()-start
        inventory=args.output/'roles-inventory.sqlite'
        if sha(snapshot)!=prior['copy_sha256'] or sha(binary)!=frozen['census']['sha256'] or sha(custody)!=initial_custody:
            error='input/binary/custody changed during census'; exit_code=125
        result=dict(schema='issue88-combined-census-completion-v1',arm=args.arm,status='PASS' if exit_code==0 else 'FAIL',exit_code=exit_code,error=error,
                    command=command,decoder_binary=dict(path=str(binary),sha256=sha(binary)),log=dict(path=str(log),sha256=sha(log)),
                    snapshot=dict(path=str(snapshot),sha256=sha(snapshot)),inventory=dict(path=str(inventory),sha256=sha(inventory) if inventory.exists() else None),
                    snapshot_custody=dict(path=str(custody),sha256=sha(custody)),elapsed_ns=elapsed,
                    single_writer_append_only=True,single_writer_basis='Fresh unique run directory; one normal public host coordinator under shared exclusive measurement lock; no external Store openings; source unchanged; successful normal close and cleanup followed by lsof quiescence. Census opens only this closed logical snapshot RO immutable.')
        with (args.output/'completion.json').open('x') as stream:
            json.dump(result,stream,indent=2);stream.write('\n')
        assert sha(snapshot)==prior['copy_sha256'], 'decoder mutated snapshot'
        print(json.dumps(result));raise SystemExit(exit_code)

if __name__=='__main__':main()
