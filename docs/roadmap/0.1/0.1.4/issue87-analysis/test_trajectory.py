"""Small receipt parsing check; run with python3 test_trajectory.py."""
from trajectory import parsed_candidate, one, flat
assert parsed_candidate({'commit_receipt':'candidate: Some(CandidateStats { candidate_objects: 3, inserted_objects: 2, reused_objects: 1 })'}) == {'candidate_objects':3,'inserted_objects':2,'reused_objects':1}
assert parsed_candidate({}) == {}
assert flat({'physical':{'bytes':7}}) == {'physical_bytes':7}
assert one([{'kind':'allocation','bytes':4}],'allocation')['bytes']==4
try:
    one([{'kind':'allocation'},{'kind':'allocation'}],'allocation')
except AssertionError:
    pass
else:
    raise AssertionError('duplicate receipt accepted')
print('trajectory self-check PASS')
