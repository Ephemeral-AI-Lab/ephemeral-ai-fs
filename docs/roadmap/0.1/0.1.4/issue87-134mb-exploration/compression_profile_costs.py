#!/usr/bin/env python3
"""Query pinned Zstd resource estimates only; never compress/decompress data."""
import ctypes
import ctypes.util
import json

class Parameters(ctypes.Structure):
    _fields_ = [(name, ctypes.c_uint) for name in
                ('windowLog', 'chainLog', 'hashLog', 'searchLog', 'minMatch', 'targetLength', 'strategy')]

def main():
    path = ctypes.util.find_library('zstd')
    assert path, 'installed Zstd library required'
    z = ctypes.CDLL(path)
    z.ZSTD_versionNumber.restype = ctypes.c_uint
    assert z.ZSTD_versionNumber() == 10507, 'estimates require producer Zstd1.5.7 version'
    z.ZSTD_getCParams.argtypes = [ctypes.c_int, ctypes.c_ulonglong, ctypes.c_size_t]
    z.ZSTD_getCParams.restype = Parameters
    z.ZSTD_estimateCCtxSize_usingCParams.argtypes = [Parameters]
    z.ZSTD_estimateCCtxSize_usingCParams.restype = ctypes.c_size_t
    z.ZSTD_compressBound.argtypes = [ctypes.c_size_t]
    z.ZSTD_compressBound.restype = ctypes.c_size_t
    z.ZSTD_isError.argtypes = [ctypes.c_size_t]
    z.ZSTD_isError.restype = ctypes.c_uint
    rows = []
    for label, level, size, window in [('current_group_cap', 1, 65536, 16),
                                     ('stronger_same_group_cap', 19, 65536, 16),
                                     ('aggressive_8MiB_frame', 19, 8388608, 23)]:
        p = z.ZSTD_getCParams(level, size, 0)
        p.windowLog = min(p.windowLog, window)
        context = z.ZSTD_estimateCCtxSize_usingCParams(p)
        bound = z.ZSTD_compressBound(size)
        assert not z.ZSTD_isError(context) and not z.ZSTD_isError(bound)
        assert context > 0 and bound >= size
        rows.append(dict(profile=label, level=level, decoded_frame_bytes=size,
                         parameters={name: getattr(p, name) for name, _ in p._fields_},
                         estimated_encoder_context_bytes=context,
                         maximum_encoded_output_bytes=bound,
                         input_plus_max_output_plus_context_bytes=size + bound + context))
    print(json.dumps({'status': 'library resource estimates; not allocated RSS or encoding measurements',
                      'library_path': path, 'library_version': '1.5.7',
                      'units': 'byte suffixes integer bytes; other numbers Zstd parameter/count values',
                      'population': 'three declared profiles, no data input, dictionary or worker threads',
                      'provenance': 'ZSTD_getCParams/ZSTD_estimateCCtxSize_usingCParams/ZSTD_compressBound',
                      'limitations': 'These estimates exclude application buffers, dictionaries, worker queues, decoder/read caches and SQLite. No compression or decompression performed.',
                      'profiles': rows}, indent=2))

if __name__ == '__main__':
    main()
