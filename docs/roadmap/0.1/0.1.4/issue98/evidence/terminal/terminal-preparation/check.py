"""Small read-only preparation guard check; no workloads or Store reads."""
import importlib.util
import json
from pathlib import Path
import tempfile

path = Path(__file__).with_name('prepare.py')
spec = importlib.util.spec_from_file_location('prepare', path)
p = importlib.util.module_from_spec(spec)
spec.loader.exec_module(p)
with tempfile.TemporaryDirectory() as directory:
    artifact = Path(directory)/'receipt.json'
    artifact.write_text('{}')
    receipt = p.ref(artifact)
    p.check_ref(receipt)
    artifact.write_text('{"changed":true}')
    try:
        p.check_ref(receipt)
    except AssertionError:
        pass
    else:
        raise AssertionError('changed custody accepted')
old = json.loads((p.OLD/'schedule.frozen.json').read_text())
p.check_ref(old['control_reuse_applicability'])
reuse = json.loads(Path(old['control_reuse_applicability']['path']).read_text())
assert reuse['original_control_arm'] == old['order'][0]
for reference in reuse['references'].values():
    p.check_ref(reference)
for filename in ('prepare.py', 'run_frozen.py'):
    compile(path.with_name(filename).read_text(), filename, 'exec')
print('PASS: changed custody rejected; original control references authenticate; scripts compile')
