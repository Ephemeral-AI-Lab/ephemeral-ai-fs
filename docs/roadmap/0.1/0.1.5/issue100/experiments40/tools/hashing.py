"""BLAKE3 for standalone diagnostics; uses the existing cached Rust dependency."""
import ctypes
from pathlib import Path

_lib = ctypes.CDLL(str(Path(__file__).with_name('libdiagnostic_hash.dylib')))
_lib.diagnostic_blake3.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p]
_lib.diagnostic_blake3.restype = None

def blake3(data: bytes) -> bytes:
    result = ctypes.create_string_buffer(32)
    _lib.diagnostic_blake3(data, len(data), result)
    return result.raw

if __name__ == '__main__':
    assert blake3(b'').hex() == 'af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262'
    assert blake3(b'abc').hex() == '6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85'
    print('BLAKE3 empty/abc known vectors passed')
