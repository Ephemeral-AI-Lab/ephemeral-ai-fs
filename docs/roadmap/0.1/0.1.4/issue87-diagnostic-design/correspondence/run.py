#!/usr/bin/env python3
"""Extend the sealed synthetic fixture in a disposable crate; never touch a Store."""
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[5]
OLD = HERE.parent.parent / "issue87-deep-diagnosis/correspondence-probe"
PINNED = {
    ROOT / "crates/layerfs-content/src/file/rope/read.rs": "fef00f3337dd600a83febba3664d3f8ecfe383a26b9d46abffc4732e2ee394ed",
    ROOT / "crates/layerfs-layerstack-store/src/objects.rs": "b65cf3915d648a10f2b22a670d31d85d93861db94a4ab6eb5e7178dd0930db6f",
    OLD / "src/main.rs": "ac6993bcf3bd94a734dbec20e42042e3b879bef189a5082ccbaf456f518d55a1",
}
for path, expected in PINNED.items():
    assert hashlib.sha256(path.read_bytes()).hexdigest() == expected, path
with tempfile.TemporaryDirectory(prefix="issue87-correspondence-design-") as temporary:
    folder = Path(temporary)
    (folder / "src").mkdir()
    fixture = (OLD / "src/main.rs").read_text().split("fn main() {", 1)[0]
    (folder / "src/main.rs").write_text(fixture + (HERE / "probe.rs").read_text())
    manifest = (OLD / "Cargo.toml").read_text().replace(
        "../../../../../../crates/layerfs-content", str(ROOT / "crates/layerfs-content")
    )
    (folder / "Cargo.toml").write_text(manifest)
    (folder / "Cargo.lock").write_bytes((OLD / "Cargo.lock").read_bytes())
    environment = dict(os.environ, CARGO_TARGET_DIR="/tmp/issue87-correspondence-design-target")
    subprocess.run(["cargo", "run", "--offline", "--locked", "--quiet", "--manifest-path", str(folder / "Cargo.toml")], env=environment, check=True)
