#!/usr/bin/env python3
"""Synthetic native-Zstd capability check, NOT a LayerFS savings sample."""
import hashlib
import json
from pathlib import Path
import random
import shutil
import subprocess
import tempfile

zstd = shutil.which("zstd")
if not zstd:
    raise SystemExit("native zstd CLI required; no package installation attempted")
version = subprocess.check_output([zstd, "--version"], text=True).strip()
generator = random.Random(87)
initial = generator.randbytes(256 * 1024)
versions = [initial]
for index in range(8):
    changed = bytearray(versions[-1])
    start = (index + 1) * 16384
    changed[start:start + 256] = generator.randbytes(256)
    versions.append(bytes(changed))

with tempfile.TemporaryDirectory(prefix="issue87-synthetic-prefix-") as temporary:
    folder = Path(temporary)
    files = []
    for index, data in enumerate(versions):
        path = folder / f"synthetic-{index}"
        path.write_bytes(data)
        files.append(path)
    rows = []
    for index in range(1, len(files)):
        row = {"synthetic_version": index, "synthetic_payload_bytes": len(versions[index]),
               "mutated_payload_bytes_from_prior": 256, "target_sha256": hashlib.sha256(versions[index]).hexdigest()}
        for name, base in [("full", None), ("initial_prefix", files[0]), ("previous_prefix", files[index-1])]:
            encoded = folder / f"{name}-{index}.zst"
            flags = [] if base is None else [f"--patch-from={base}"]
            subprocess.run([zstd, "--single-thread", "-3", "-q", "-f", *flags, str(files[index]), "-o", str(encoded)], check=True)
            decoded = subprocess.check_output([zstd, "-d", "-q", *flags, str(encoded), "-c"])
            assert decoded == versions[index]
            row[name + "_frame_bytes"] = encoded.stat().st_size
        rows.append(row)
    # The fixture is deliberately cumulative-edit data, not sampled source files.
    # A native prefix codec should exploit it and decode with the exact base.
    assert all(row["previous_prefix_frame_bytes"] < row["full_frame_bytes"] for row in rows)
    assert rows[-1]["previous_prefix_frame_bytes"] < rows[-1]["initial_prefix_frame_bytes"]

print(json.dumps({
    "status": "synthetic capability/self-check; not measured LayerFS opportunity",
    "source_population": "9 generated 262144-byte byte arrays; random seed87; 256 new bytes at a new offset each version",
    "codec": version,
    "codec_scope": "native CLI; not LayerFS static workspace, grouping, framing, admission, allocation or timing",
    "units": "integer bytes and counts; Zstd frame bytes exclude all future Store/base/index framing",
    "policy": "level3, single-thread; CLI --patch-from controls its own patch window settings",
    "hypothesis_checked": "an authenticated prior version can serve as native prefix; immediate prior avoids repeatedly encoding cumulative edits against the initial version",
    "all_roundtrips_verified": True,
    "retained_evidence_accessed": False,
    "rows": rows,
}, indent=2))
