#!/usr/bin/env python3
"""Contract-fixed synthetic preflight; root serial execution slot required."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import random
import sqlite3
import subprocess
import tempfile
from types import SimpleNamespace
from content_screen import RECORD_HEADER, Zstd, reconstruct, run, validate_file_header


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", required=True)
    parser.add_argument("--extractor", required=True)
    parser.add_argument("--contract", action="append", required=True)
    parser.add_argument("--output", required=True)
    options = parser.parse_args()
    destination = Path(options.output)
    destination.mkdir(parents=True, exist_ok=False)
    generated = random.Random(88).randbytes(262145)
    with tempfile.TemporaryDirectory(prefix="issue88-content-selfcheck-") as directory:
        root = Path(directory)
        rows, seen, snapshots = [], {}, []
        for checkpoint, length, edited in [(1, 32768, False), (2, 32768, True),
                                          (3, 262143, False), (4, 262143, True),
                                          (5, 262145, False), (6, 262143, False),
                                          (7, 32768, False)]:
            raw = generated[:length]
            if edited:
                at = length // 2
                raw = raw[:at] + b"Z" * 64 + raw[at + 64:]
            source = root / f"input-{checkpoint}"
            source.write_bytes(raw)
            if length <= 262144:
                units = [dict(id="file:" + hashlib.sha256(raw).hexdigest(),
                              unit_kind="file", offset=0, length=length)]
            else:
                text = subprocess.check_output([options.extractor], input=str(source) + "\n", text=True)
                units = []
                for item in text.strip().split(";"):
                    identity, offset, size = item.split(",")
                    units.append(dict(id="chunk:" + identity, unit_kind="chunk", offset=int(offset),
                                      length=int(size), existing_canonical_id=identity, canonical_length=int(size) + 21))
                assert sum(unit["length"] for unit in units) == length
                assert all(unit["length"] <= 32768 for unit in units)
            snapshot = [unit["id"] for unit in units]
            for unit in units:
                if unit["id"] in seen:
                    continue
                data = raw[unit["offset"]:unit["offset"] + unit["length"]]
                prior = snapshots[-1][0] if edited else None
                row = {**unit, "sha256": hashlib.sha256(data).hexdigest(), "checkpoint": checkpoint,
                       "ordinal": len(rows), "source_path": str(source), "prior_id": prior,
                       "prior_provenance": "synthetic samefile previous checkpoint" if prior else None}
                seen[unit["id"]] = row
                rows.append(row)
            snapshots.append(snapshot)
        assert snapshots[2] == snapshots[5] and snapshots[0] == snapshots[6]
        duplicate = root / "separate-file-same-content"
        duplicate.write_bytes(generated[:32768])
        duplicate_id = "file:" + hashlib.sha256(duplicate.read_bytes()).hexdigest()
        assert duplicate_id == snapshots[0][0] and duplicate_id in seen
        manifest = destination / "manifest.jsonl"
        manifest.write_text("".join(json.dumps(row) + "\n" for row in rows))
        args = SimpleNamespace(output=str(destination / "screen"), library=options.library, level=3, window_log=20,
                               manifest=str(manifest), arm="synthetic", max_depth=4, contract=options.contract,
                               max_closure_bytes=1048576, prefix_max_bytes=262144, max_input_bytes=262144)
        result = run(args)
        assert result["totals"]["prefix_selected"] == 2
        assert result["canonical"]["file_targets"] == 4
        assert result["canonical"]["chunk_payload_bytes"] == 262145
        index = sqlite3.connect(destination / "screen/index.sqlite")
        index.row_factory = sqlite3.Row
        codec = Zstd(options.library)
        checks = []
        try:
            with open(destination / "screen/selected.frames", "rb") as frames:
                validate_file_header(frames)
                try:
                    validate_file_header(io.BytesIO(b"BADMAGIC" + b"\0" * 8))
                except ValueError:
                    checks.append("malformed container header rejected")
                else:
                    raise AssertionError("malformed container header accepted")
                for checkpoint, ids in enumerate(snapshots, 1):
                    stats = dict(frame_reads=0, encoded_bytes=0, decoded_bytes=0)
                    actual = b"".join(reconstruct(index, frames, identity, codec, 4, 1048576, stats) for identity in ids)
                    assert actual == (root / f"input-{checkpoint}").read_bytes()
                for bad_kind in ("wrong_base", "truncated_frame", "concatenated_frames"):
                    base = generated[:32768]
                    edited = base[:16384] + b"Z" * 64 + base[16448:]
                    encoded = codec.compress(edited, base)
                    try:
                        if bad_kind == "wrong_base":
                            decoded = codec.decompress(encoded, len(edited), b"\0" * len(base))
                            assert hashlib.sha256(decoded).digest() == hashlib.sha256(edited).digest(), "wrong reconstructed hash"
                        elif bad_kind == "truncated_frame":
                            codec.decompress(encoded[:-1], len(edited), base)
                        else:
                            codec.decompress(encoded + encoded, len(edited), base)
                    except (ValueError, AssertionError):
                        checks.append(bad_kind + " rejected")
                    else:
                        raise AssertionError(bad_kind + " accepted")
        finally:
            index.close()
            codec.close()
        bad = dict(rows[0], prior_id="file:future", prior_provenance="invalid")
        (destination / "bad.jsonl").write_text(json.dumps(bad) + "\n")
        args.manifest, args.output = str(destination / "bad.jsonl"), str(destination / "expected-failure")
        try:
            run(args)
        except ValueError as error:
            assert "future/unavailable" in str(error)
            assert json.loads((destination / "expected-failure/failure.json").read_text())["status"] == "FAIL"
        else:
            raise AssertionError("future base accepted")
        assert RECORD_HEADER.size == 56
        summary = {"status": "PASS", "checks": checks + ["all synthetic raw/canonical identities and roundtrips",
                   "FULL/PREFIX conservation", "localized64byte edits", "typed recurrence/cross-file identity",
                   "262143->262145->262143 wholefile/CDC transition", "future base rejected with preserved failure"],
                   "read_probes": result["read_probes"], "totals": result["totals"],
                   "scope": "synthetic content-route boundary check; no public API, OS cold or product timing claim",
                   "temporary_inputs": "deleted normally; synthetic generator and hashes retained"}
        (destination / "result.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps({"status": "PASS", "checks": summary["checks"]}))


if __name__ == "__main__":
    main()
