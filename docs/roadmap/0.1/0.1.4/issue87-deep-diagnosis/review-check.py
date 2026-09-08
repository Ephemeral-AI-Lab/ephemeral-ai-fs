#!/usr/bin/env python3
"""Product-free checks for review counterexamples; no SQLite or payload access."""
import importlib.util
from pathlib import Path

module = Path(__file__).resolve().parent.parent / 'issue87-analysis/trajectory.py'
spec = importlib.util.spec_from_file_location('old_trajectory', module)
trajectory = importlib.util.module_from_spec(spec)
spec.loader.exec_module(trajectory)

# Reproduce the old reporter's latent duplicate-phase acceptance. This is a
# defect witness, not a regression assertion approving the behavior.
phases = trajectory.stats([
    {'kind':'storage-smoke-phase','phase':'commit','elapsed_ns':7},
    {'kind':'storage-smoke-phase','phase':'commit','elapsed_ns':11},
])
assert len(phases) == 1 and phases['commit']['elapsed_ns'] == 11

# A shared ObjectId can have two initially-missing attempts with one winner.
# Attempt-race override must not replace the unique target's admitted outcome.
attempts = [('same-target', 32789, 'DELTA admitted'),
            ('same-target', 32789, 'admission race')]
assert len(attempts) == 2 and len({row[0] for row in attempts}) == 1
assert sum(row[1] for row in attempts) == 65578
assert {row[0]: row[1] for row in attempts} == {'same-target':32789}

# Prefix cardinality gives equality only after inclusion is established.
logical, selected = {'a','b'}, {'a','b'}
assert logical <= selected and len(logical) == len(selected)
assert logical == selected
assert len({'a','b'}) == len({'a','c'}) and {'a','b'} != {'a','c'}

# For n no-hint targets from known new-payload sizes, sorted extrema are sharp.
lengths, n = [21,64,512,32789], 2
assert sum(sorted(lengths)[:n]) == 85
assert sum(sorted(lengths)[-n:]) == 33301
print('Review checks PASS: duplicate-phase defect reproduced; attempt/unique race counterexample; prefix-equality premise; sharp byte bounds.')
