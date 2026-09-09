"""Bounded Python binding to the existing Rust metadata COPY/INSERT matcher."""
import ctypes as C
import hashlib
import struct
from pathlib import Path

_lib = C.CDLL(str(Path(__file__).with_name('libdiagnostic_delta.dylib')))
_lib.diagnostic_delta.argtypes = [C.c_void_p,C.c_void_p,C.c_size_t,C.c_void_p,C.c_size_t,C.POINTER(C.c_size_t),C.c_void_p,C.c_size_t,C.POINTER(C.c_uint64)]
_lib.diagnostic_delta.restype = C.c_ssize_t

def delta(base_id, base, target, remaining):
    assert len(base_id) == 32 and 0 < len(base) <= 8192 and 0 < len(target) <= 8192 and remaining >= 0
    budget = C.c_size_t(remaining)
    output = C.create_string_buffer(len(target)+1)
    counters = (C.c_uint64*4)()
    length = _lib.diagnostic_delta(base_id,base,len(base),target,len(target),C.byref(budget),output,len(output),counters)
    assert length != -2, 'invalid request or matcher error'
    assert length == -1 or 0 < length <= len(target)+1
    names = ('match_budget_skips','seed_hash_bytes','match_comparisons','instruction_budget_skips')
    return (None if length == -1 else output.raw[:length]), budget.value, dict(zip(names,counters))

def replay(record, base_id, base):
    assert record[:1] == b'\1' and record[1:33] == base_id
    size, count = struct.unpack_from('<II', record, 33)
    assert 0 < size <= 8192 and 0 < count <= 8191
    pos = 41
    output = bytearray()
    for _ in range(count):
        kind = record[pos]; pos += 1
        if kind == 0:
            start, length = struct.unpack_from('<II',record,pos); pos += 8
            assert length > 0 and start+length <= len(base)
            output.extend(base[start:start+length])
        else:
            assert kind == 1
            length = struct.unpack_from('<I',record,pos)[0]; pos += 4
            assert length > 0 and pos+length <= len(record)
            output.extend(record[pos:pos+length]); pos += length
        assert len(output) <= size
    assert pos == len(record) and len(output) == size
    return bytes(output)

if __name__ == '__main__':
    base = b''.join(hashlib.sha256(i.to_bytes(4,'little')).digest() for i in range(128))
    identity = hashlib.sha256(base).digest()
    target = b'new-prefix'+base[20:2048]+b'changed-value'+base[2060:]
    encoded, remaining, stats = delta(identity,base,target,16*1024*1024)
    assert encoded is not None and len(encoded) < len(target)
    assert replay(encoded,identity,base) == target and remaining < 16*1024*1024
    rejected, remaining, stats = delta(identity,base,target,0)
    assert rejected is None and remaining == 0 and stats['match_budget_skips'] == 1
    same, _, _ = delta(identity,base,base,16*1024*1024)
    assert same is not None and replay(same,identity,base) == base
    print('PASS existing matcher: insert/copy roundtrip, identical input, zero-budget fallback')
