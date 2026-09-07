"""Execute the predeclared twelve-run cohort using the existing serial runners."""
import argparse
import json
from pathlib import Path
import subprocess
import sys

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--repo', required=True, type=Path)
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('--declaration', required=True, type=Path)
parser.add_argument('--image', required=True)
args = parser.parse_args()
repo = args.repo.resolve()
output = args.output.resolve()
declaration = json.loads(args.declaration.read_text())
expected = [f'final-{tier}-{arm}-r{rep}' for tier in (100, 500)
            for rep in (1, 2, 3)
            for arm in (('layerfs', 'native') if rep % 2 else ('native', 'layerfs'))]
assert declaration['runs'] == expected, 'declaration must contain the exact alternating schedule'
assert not any((output / name).exists() for name in expected), 'never overwrite or resume a cohort'
sys.path.insert(0, str(repo / 'benchmark/fs-bench-pro/shared'))
import runner

source = runner.source_build_args()
for key in ('LAYERFS_SOURCE_SEAL', 'LAYERFS_PRODUCT_SEAL', 'WORKLOAD_SOURCE_SHA256'):
    assert source[key] == declaration['source_build_args'][key], 'source changed after declaration'
image = json.loads(subprocess.check_output(['docker', 'image', 'inspect', args.image]))[0]
assert image['Config']['Labels']['dev.layerfs.source-seal'] == source['LAYERFS_SOURCE_SEAL']
assert image['Config']['Labels']['dev.layerfs.product-seal'] == source['LAYERFS_PRODUCT_SEAL']
native = Path(__file__).with_name('run_native.py')
for name in expected:
    _, tier, arm, repetition = name.split('-')
    destination = output / name
    if arm == 'layerfs':
        command = [sys.executable, str(repo / 'benchmark/fs-bench-pro/shared/runner.py'),
                   '--topology', 'host-store', '--family', 'git_tool_workflow',
                   '--case', f'git-tool-{tier}-mixed-v4', '--seed', '1', '--setup', 'clone',
                   '--image', image['Id'], '--collection-mode', '--product-timeout', '300',
                   '--timeout', '310', '--setup-timeout', '600', '--output', str(destination)]
    else:
        # All three native runs use the exact case/source/fixture of the first
        # LayerFS run. Each native invocation creates independent mutable state.
        reference = output / f'final-{tier}-layerfs-r1' / 'perf.jsonl'
        command = [sys.executable, str(native), '--perf', str(reference),
                   '--output', str(destination)]
    with (output / f'{name}.log').open('x') as log:
        log.write(json.dumps(command) + '\n')
        log.flush()
        subprocess.run(command, cwd=repo, stdout=log, stderr=subprocess.STDOUT, check=True)
    print(f'completed {name}', flush=True)
