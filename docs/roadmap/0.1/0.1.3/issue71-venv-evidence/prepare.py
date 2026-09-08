"""Create an uncompressed input archive without changing source metadata."""
import hashlib
import json
import sys
import tarfile
from decimal import Decimal
from pathlib import Path

source = Path(sys.argv[1]).resolve(strict=True)
output = Path(sys.argv[2]).absolute()
assert source.is_dir()
assert not output.exists()
assert not output.parent.resolve(strict=True).is_relative_to(source)

def preserve_mtime(info):
    metadata = (source / info.name).lstat()
    info.pax_headers["mtime"] = str(Decimal(metadata.st_mtime_ns).scaleb(-9))
    return info

with tarfile.open(output, "x", format=tarfile.PAX_FORMAT) as archive:
    archive.add(source, arcname=".", filter=preserve_mtime)
with output.open("rb") as stream:
    digest = hashlib.file_digest(stream, "sha256").hexdigest()
print(json.dumps({"source": str(source), "archive": str(output), "bytes": output.stat().st_size, "sha256": digest}))
