"""Call the cached product CDC scanner; keep its frozen boundary check."""
import ctypes
from pathlib import Path

_lib = ctypes.CDLL(str(Path(__file__).with_name('libdiagnostic_cdc.dylib')))
_lib.diagnostic_cdc.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t), ctypes.c_size_t]
_lib.diagnostic_cdc.restype = ctypes.c_size_t

def lengths(data: bytes) -> list[int]:
    capacity = len(data) // 8192 + 2
    output = (ctypes.c_size_t * capacity)()
    count = _lib.diagnostic_cdc(data, len(data), output, capacity)
    assert count <= capacity
    result = list(output[:count])
    assert sum(result) == len(data) and all(0 < size <= 32768 for size in result)
    return result

if __name__ == '__main__':
    state = 0x9e3779b97f4a7c15
    data = bytearray()
    for _ in range(100_000):
        state ^= (state << 7) & ((1 << 64) - 1)
        state ^= state >> 9
        state ^= (state << 8) & ((1 << 64) - 1)
        data.append(state & 255)
    assert lengths(bytes(data)) == [16396, 17093, 16413, 20273, 19016, 10809]
    assert lengths(b'') == []
    print('Product frozen CDC boundary and empty-input checks passed')
