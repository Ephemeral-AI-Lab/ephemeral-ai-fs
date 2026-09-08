#!/usr/bin/env python3
"""Read the existing small receipt CSV only; never open a Store."""
import csv
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
SOURCES = {
    "crates/layerfs-content/src/file/rope/read.rs": "fef00f3337dd600a83febba3664d3f8ecfe383a26b9d46abffc4732e2ee394ed",
    "crates/layerfs-layerstack-store/src/objects.rs": "b65cf3915d648a10f2b22a670d31d85d93861db94a4ab6eb5e7178dd0930db6f",
}
for path, digest in SOURCES.items():
    assert hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == digest, path
source = HERE.parent / "issue87-analysis/published/checkpoint-trajectory.csv"
rows = list(csv.DictReader(source.open()))[1:]
assert [int(row["index"]) for row in rows] == list(range(1, 158))
reservation = 131136
last_slot = ((16 * 1024 * 1024) // reservation) * reservation
result = {
    "status": "derived from measured phase counters; source policy independently checked",
    "population": "157 performance checkpoints; Init excluded",
    "snapshot_phase": "original public Exec and Commit receipts; not current Store",
    "units": "integer bytes, integer counts, ordered checkpoint indices",
    "provenance": str(source.relative_to(ROOT)),
    "provenance_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
    "source_sha256": SOURCES,
    "per_metadata_read_reservation_bytes": reservation,
    "operation_limit_bytes": 16 * 1024 * 1024,
    "last_admissible_total_reservation_bytes": last_slot,
    "operation_slots_count": last_slot // reservation,
    "file_slots_count": (1024 * 1024) // reservation,
    "phases": {},
    "affected_unique_target_canonical_bytes": None,
    "affected_unique_target_canonical_bytes_unknown_reason": "Cursor exhaustion events precede CAS filtering; no persisted cursor-to-eligible-target mapping or byte-weighted terminal causes.",
}
for phase in ("exec", "commit"):
    prefix = phase + "_physical_storage_"
    blocked = [row for row in rows if int(row[prefix + "correspondence_budget_skips"])]
    assert all(int(row[prefix + "correspondence_reserved_bytes"]) == last_slot for row in blocked)
    assert all(int(row[prefix + "memory_budget_skips"]) == 0 for row in rows)
    result["phases"][phase] = {
        "exhausted_checkpoint_count": len(blocked),
        "every_exhausted_checkpoint_at_last_reservation_slot": True,
        "exhausted_cursor_count": sum(int(row[prefix + "correspondence_budget_skips"]) for row in rows),
        "reserved_bytes": sum(int(row[prefix + "correspondence_reserved_bytes"]) for row in rows),
        "descriptors_count": sum(int(row[prefix + "correspondence_descriptors"]) for row in rows),
        "nonexhausted_checkpoint_count": len(rows) - len(blocked),
        "zero_reservation_checkpoint_count": sum(int(row[prefix + "correspondence_reserved_bytes"]) == 0 for row in rows),
        "nonexhausted_with_nonzero_reservations": [{
            "index": int(row["index"]),
            "reserved_bytes": int(row[prefix + "correspondence_reserved_bytes"]),
        } for row in rows if row not in blocked and int(row[prefix + "correspondence_reserved_bytes"])],
    }
assert result["phases"]["exec"]["exhausted_checkpoint_count"] == 0
assert result["phases"]["commit"]["exhausted_checkpoint_count"] == 152
assert result["phases"]["commit"]["exhausted_cursor_count"] == 125254
print(json.dumps(result, indent=2))
