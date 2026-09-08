#!/usr/bin/env python3
"""Preserve exactly one closed, pre-verification logical snapshot per frozen arm."""
import argparse, fcntl, hashlib, json, os, pathlib, shutil, subprocess, time

def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('schedule', type=pathlib.Path)
    parser.add_argument('arm', choices=('control', 'candidate'))
    parser.add_argument('output', type=pathlib.Path)
    args = parser.parse_args()
    schedule = json.loads(args.schedule.read_text())
    arm = next(row for row in schedule['order'] if row['arm'] == args.arm)
    run = pathlib.Path(arm['output'])
    with (pathlib.Path(os.environ.get('TMPDIR', '/tmp')) / 'layerfs-infra-measurement.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        started = time.monotonic_ns()
        identity=json.loads((run/'identity.json').read_text())
        assert identity['host_identity']==arm['host'] and identity['image_id']==arm['image_id']
        result = json.loads((run / 'deepseek-full/performance-result.json').read_text())
        assert result['status'] == result['cleanup_status'] == 'PASS'
        assert result['container_removed'] is True
        assert [row['index'] for row in result['records']] == list(range(1, 158))
        assert not list(run.glob('verification*')) and not list((run / 'deepseek-full').glob('verification*')), 'verification already started'
        source = run / 'deepseek-full/host-runtime/store.sqlite'
        assert source.is_file() and not source.is_symlink()
        handles = subprocess.run(['lsof', '-t', '--', str(source)], capture_output=True, text=True, timeout=30)
        assert handles.returncode == 1 and not handles.stdout and not handles.stderr, handles
        sidecars = list(source.parent.glob('store.sqlite-*'))
        assert not sidecars, sidecars
        manifest = run / 'performance-manifest.json'
        sealed = json.loads(manifest.read_text())
        # Authenticate all performance artifacts before any verifier may add Branch metadata.
        for relative, digest in sealed.items():
            assert sha(run / relative) == digest, relative
        expected = sealed[str(source.relative_to(run))]
        ack = [v for v in result['records'][-1]['receipts'] if v['kind'] == 'storage-smoke-allocation']
        assert len(ack) == 1 and ack[0]['label'] == 'step-157'
        stat = source.stat()
        args.output.mkdir()
        copy = args.output / 'store.sqlite'
        with copy.open('xb') as sink, source.open('rb') as stream:
            shutil.copyfileobj(stream, sink, 1024 * 1024)
            sink.flush(); os.fsync(sink.fileno())
        assert sha(copy) == sha(source) == expected
        data = dict(status='PASS', source=str(source), source_sha256=expected,
                    copy_sha256=expected, snapshot_mode='ro-immutable',
                    snapshot_phase='final-pre-verification',
                    quiescence=dict(status='PASS', lsof_no_open_handles=True, performance_cleanup='PASS'),
                    primary_final_ack=ack[0], source_stat_allocated_bytes=stat.st_blocks * 512,
                    source_stat_logical_bytes=stat.st_size,
                    method='streamed logical copy; allocation not equivalent',
                    observer_copy_hash_ns=time.monotonic_ns()-started,
                    observer_effect='manifest hashing and logical copy warm caches after performance before verification; excluded from public elapsed',
                    performance_manifest_sha256=sha(manifest), frozen_schedule_sha256=sha(args.schedule))
        with (args.output / 'custody.json').open('x') as stream:
            json.dump(data, stream, indent=2); stream.write('\n')
        print(json.dumps(data))

if __name__ == '__main__':
    main()
