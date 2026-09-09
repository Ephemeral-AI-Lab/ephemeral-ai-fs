"""Product-free check of selected original states and their transition/oracle seals."""
import argparse
import hashlib
import json
from pathlib import Path
from deepseek_ten import INDICES, PROFILES
from storage_smoke import CASES, MANIFEST_SHA, SOURCE_TIP, seal


def check(fixture_path, data):
    assert INDICES == (1, 18, 36, 53, 70, 88, 105, 122, 140, 157)
    assert len(PROFILES['deepseek-stride3'][0]) == 53
    assert PROFILES['deepseek-stride3'][0] == tuple(range(1, 158, 3))
    assert CASES['deepseek-full'] == ['deepseek-full']
    fixture = json.loads(fixture_path.read_text())
    assert len(fixture) == 1
    profile, selected = next(iter(fixture.items()))
    indices, scenario, _ = PROFILES[profile]
    assert selected['scenario'] == scenario and selected['full157_indices'] == list(indices)
    assert selected['manifest_sha256'] == MANIFEST_SHA
    raw = (data/'checkpoint-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest() == MANIFEST_SHA
    original = json.loads(raw)
    assert original['tip'] == SOURCE_TIP and len(original['checkpoints']) == 157
    states = selected['states']
    assert len(states) == len(indices)
    previous = b''
    oracle_entries = 0
    for ordinal, (state, full_index) in enumerate(zip(states, indices), 1):
        source = original['checkpoints'][full_index-1]
        assert state['index'] == ordinal and state['full157_index'] == full_index
        assert all(state[k] == source[k] for k in ('sha', 'tree', 'manifest_sha256', 'logical_bytes', 'files'))
        folder = Path(state['input'])
        assert seal(folder) == state['input_seal']
        manifest = (folder/'manifest.tsv').read_bytes()
        assert manifest == (data/'inputs'/source['sha']/'manifest.tsv').read_bytes()
        assert (folder/'previous.tsv').read_bytes() == previous
        oracle = data/'oracles'/(source['sha']+'.json')
        assert Path(state['oracle']) == oracle
        assert hashlib.sha256(oracle.read_bytes()).hexdigest() == state['oracle_sha256']
        oracle_entries += len(json.loads(oracle.read_text()))
        previous = manifest
    return {'status': 'PASS', 'profile': profile, 'states': len(states),
            'full157_indices': list(indices), 'original_oracle_path_states': oracle_entries,
            'fixture_sha256': hashlib.sha256(fixture_path.read_bytes()).hexdigest()}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fixture', type=Path, required=True)
    parser.add_argument('--data', type=Path, default=Path('/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data'))
    args = parser.parse_args()
    print(json.dumps(check(args.fixture, args.data), indent=2))
