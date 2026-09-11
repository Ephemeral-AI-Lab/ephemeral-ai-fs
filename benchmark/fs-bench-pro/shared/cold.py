"""Fixed namespace-100000 cold-source qualification; no operator cache policy."""
import ctypes
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import stat
import tempfile
import time

CONTRACT = "namespace-100000-cold-v2"
METHOD = "darwin-shared-mmap-invalidate-mincore-v1"
TARGET_NS = 2_700_000_000
FIXTURE_DIGEST = "6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e"
MAX_LAUNCH_GAP_NS = 1_000_000_000
METADATA_POLICY = {"contract": "namespace-fixture-metadata-v1", "file_mode": 0o640,
                   "directory_mode": 0o750, "mtime_ns": 1_700_000_000_000_000_000}


def applies(selection):
    return (selection.get("family", selection.get("family_id")) == "init_namespace"
            and selection.get("case", selection.get("scenario_id")) == "namespace-100000")


class Residency:
    """Darwin SDK mmap/mincore boundary; mappings never write fixture bytes."""
    def __init__(self):
        if platform.system() != "Darwin":
            raise OSError("cold source residency verification requires macOS")
        self.page_size = os.sysconf("SC_PAGE_SIZE")
        self.lib = ctypes.CDLL(None, use_errno=True)
        self.lib.mmap.restype = ctypes.c_void_p
        self.lib.mmap.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int,
                                 ctypes.c_int, ctypes.c_int, ctypes.c_int64]
        self.lib.msync.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int]
        self.lib.mincore.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p]
        self.lib.munmap.argtypes = [ctypes.c_void_p, ctypes.c_size_t]

    def check(self, fd, size, *, evict=False):
        if not size:
            return 0, 0
        # PROT_READ=1, MAP_SHARED=1; all constants from the Darwin SDK.
        address = self.lib.mmap(None, size, 1, 1, fd, 0)
        if address == ctypes.c_void_p(-1).value:
            raise OSError(ctypes.get_errno(), "cold mmap")
        pages = math.ceil(size / self.page_size)
        try:
            if evict:
                # Instantiate shared mappings before invalidation. No writes.
                for offset in range(0, size, self.page_size):
                    ctypes.c_ubyte.from_address(address + offset).value
                if self.lib.msync(address, size, 0x10 | 0x2):
                    raise OSError(ctypes.get_errno(), "cold msync")
            vector = (ctypes.c_ubyte * pages)()
            if self.lib.mincore(address, size, vector):
                raise OSError(ctypes.get_errno(), "cold mincore")
            return pages, sum(bool(value & 1) for value in vector)
        finally:
            if self.lib.munmap(address, size):
                raise OSError(ctypes.get_errno(), "cold munmap")

    def self_check(self):
        with tempfile.TemporaryFile() as file:
            file.write(b"cold residency positive control\n" * self.page_size)
            file.flush()
            os.fsync(file.fileno())
            size = file.tell()
            file.seek(0)
            file.read()
            pages, warm = self.check(file.fileno(), size)
            if not pages or not warm:
                raise OSError("mincore failed to detect known-warm source data")
            self.check(file.fileno(), size, evict=True)
            _, remaining = self.check(file.fileno(), size)
            if remaining:
                raise OSError("residency self-check could not evict source pages")
            return {"warm_pages_detected": warm, "cold_pages_remaining": remaining}


def _require_metadata(path, metadata, directory):
    expected_mode = METADATA_POLICY["directory_mode" if directory else "file_mode"]
    kind_matches = stat.S_ISDIR(metadata.st_mode) if directory else stat.S_ISREG(metadata.st_mode)
    if (not kind_matches or stat.S_IMODE(metadata.st_mode) != expected_mode
            or metadata.st_mtime_ns != METADATA_POLICY["mtime_ns"]):
        raise ValueError(f"cold source metadata mismatch: {path}")


def _metadata_inventory(root, files, budget):
    payload = root / "payload"
    # Check the root itself before traversing, including a symlinked payload root.
    _require_metadata(payload, payload.lstat(), True)
    actual = set()
    directories = [payload]
    for path in sorted(payload.rglob("*")):
        budget()
        metadata = path.lstat()
        directory = stat.S_ISDIR(metadata.st_mode)
        _require_metadata(path, metadata, directory)
        if directory:
            directories.append(path)
        else:
            actual.add(str(path.relative_to(root)))
    expected_directories = {root / parent for name in files for parent in Path(name).parents
                            if parent != Path(".")}
    if actual != set(files) or set(directories) != expected_directories:
        raise ValueError("cold source inventory differs from prepared fixture")
    return directories


def acquire(prepared, sample_root, deadline):
    receipt = dict(contract=CONTRACT, method=METHOD, status="UNVERIFIED",
                   sample_root=str(sample_root), started_ns=time.monotonic_ns(),
                   fixture_digest=prepared.get("fixture", {}).get("fixture_digest"),
                   files_checked=0, logical_bytes=0, allocated_bytes=0,
                   pages_checked=0, resident_pages=0, errors=[],
                   metadata_validation={**METADATA_POLICY, "status": "UNVERIFIED",
                                        "files_checked": 0, "directories_checked": 0})
    def budget():
        if time.monotonic() >= deadline:
            raise TimeoutError("cold acquisition allowance exhausted")
    try:
        backend = Residency()
        receipt["backend_self_check"] = backend.self_check()
        root = Path(prepared["host_root"])
        manifest_bytes = (root / "host-cache.json").read_bytes()
        receipt["fixture_manifest_sha256"] = hashlib.sha256(manifest_bytes).hexdigest()
        files = {name: value for name, value in json.loads(manifest_bytes)["files"].items()
                 if name.startswith("payload/")}
        receipt["expected_pages"] = sum(math.ceil(value["bytes"] / backend.page_size) for value in files.values())
        directories = _metadata_inventory(root, files, budget)
        if len(files) != 100_000 or len(directories) != 1001:
            raise ValueError("cold source inventory differs from registered fixture")
        for name, expected in sorted(files.items()):
            budget()
            with (root / name).open("rb") as source:
                before = os.fstat(source.fileno())
                _require_metadata(root / name, before, False)
                digest = hashlib.sha256()
                while block := source.read(1024 * 1024):
                    budget()
                    digest.update(block)
                if (before.st_size != expected.get("bytes")
                        or digest.hexdigest() != expected.get("sha256")):
                    raise ValueError("cold source content differs from prepared fixture")
                os.fsync(source.fileno())
                backend.check(source.fileno(), before.st_size, evict=True)
                after = os.fstat(source.fileno())
                _require_metadata(root / name, after, False)
                if (before.st_size, before.st_mtime_ns) != (after.st_size, after.st_mtime_ns):
                    raise ValueError("cold source changed during acquisition")
        # Whole-input second pass: no payload reads or faults during observation.
        for name in sorted(files):
            budget()
            with (root / name).open("rb") as source:
                metadata = os.fstat(source.fileno())
                _require_metadata(root / name, metadata, False)
                receipt["metadata_validation"]["files_checked"] += 1
                pages, resident = backend.check(source.fileno(), metadata.st_size)
                receipt["files_checked"] += 1
                receipt["logical_bytes"] += metadata.st_size
                receipt["allocated_bytes"] += metadata.st_blocks * 512
                receipt["pages_checked"] += pages
                receipt["resident_pages"] += resident
        # Recheck directories after content validation/invalidation. Entry changes
        # also change parent mtimes; none of these checks fault payload pages.
        for directory in directories:
            budget()
            _require_metadata(directory, directory.lstat(), True)
            receipt["metadata_validation"]["directories_checked"] += 1
        receipt["metadata_validation"]["status"] = "VERIFIED"
        if (receipt["fixture_digest"] != FIXTURE_DIGEST
                or receipt["logical_bytes"] != 500_000_000
                or receipt["resident_pages"] != 0):
            raise ValueError("cold source identity or zero-residency check failed")
        receipt["status"] = "VERIFIED_COLD"
    except (OSError, ValueError, KeyError, TypeError) as error:
        receipt["errors"].append(str(error))
    receipt["finished_ns"] = time.monotonic_ns()
    receipt["wall_ns"] = receipt["finished_ns"] - receipt["started_ns"]
    return receipt


def assess(row):
    """Recompute eligibility from evidence, never saved PASS/eligible booleans."""
    acquisition = row.get("cold_acquisition") or {}
    records = row.get("records", [])
    samples = [r for r in records if "layerstack_init_ns" in r]
    reasons = []
    identity = row.get("identities", {})
    if not applies(identity) or identity.get("timer") != "layerstack_init_ns":
        reasons.append("receipt is outside the registered cold operation")
    if len(samples) != 1:
        reasons.append("missing or duplicate operation receipt")
    sample = samples[0] if len(samples) == 1 else {}
    if (acquisition.get("contract") != CONTRACT or acquisition.get("method") != METHOD
            or acquisition.get("status") != "VERIFIED_COLD" or acquisition.get("errors") != []
            or acquisition.get("files_checked") != 100_000
            or acquisition.get("logical_bytes") != 500_000_000
            or acquisition.get("resident_pages") != 0
            or not isinstance(acquisition.get("pages_checked"), int)
            or acquisition.get("pages_checked", 0) < 98_998
            or acquisition.get("pages_checked") != acquisition.get("expected_pages")
            or acquisition.get("backend_self_check", {}).get("warm_pages_detected", 0) <= 0
            or acquisition.get("backend_self_check", {}).get("cold_pages_remaining") != 0
            or acquisition.get("fixture_digest") != FIXTURE_DIGEST
            or len(acquisition.get("fixture_manifest_sha256", "")) != 64):
        reasons.append("cold source acquisition is unverified")
    metadata = acquisition.get("metadata_validation")
    if (not isinstance(metadata, dict)
            or any(metadata.get(key) != value for key, value in METADATA_POLICY.items())
            or metadata.get("status") != "VERIFIED"
            or metadata.get("files_checked") != 100_000
            or metadata.get("directories_checked") != 1001):
        reasons.append("cold source metadata validation is unverified")
    finish = acquisition.get("finished_ns")
    launch = row.get("product_command_started_ns")
    if (type(finish) is not int or type(launch) is not int
            or not 0 <= launch - finish <= MAX_LAUNCH_GAP_NS
            or not acquisition.get("sample_root")
            or acquisition.get("sample_root") != row.get("setup", {}).get("sample_root")):
        reasons.append("cold acquisition is stale or bound to another sample")
    if (sample.get("fixture_digest") != FIXTURE_DIGEST
            or sample.get("regular_files") != 100_000
            or sample.get("scanned_files") != 100_000
            or sample.get("logical_bytes") != 500_000_000
            or sample.get("scanned_bytes") != 500_000_000
            or sample.get("fixture_cache_profile") != "reused-first-sample-uncontrolled"):
        reasons.append("operation/fixture receipt mismatch")
    reads = sample.get("initialization_disk_read_bytes")
    allocated = acquisition.get("allocated_bytes")
    if (type(reads) is not int or type(allocated) is not int
            or allocated < 500_000_000 or reads < allocated):
        reasons.append("operation read volume does not corroborate cold acquisition")
    if (row.get("cold_diagnostic_environment")
            or any(r.get("kind") == "initialization-debug-text" for r in records)):
        reasons.append("nonce/profile diagnostics cannot qualify")
    elapsed = sample.get("layerstack_init_ns")
    if (type(elapsed) is not int or elapsed <= 0
            or row.get("completion_status") != "COMPLETE"
            or row.get("cleanup", {}).get("status") != "PASS"
            or row.get("resources", {}).get("oom_kill_delta") != 0
            or row.get("resources", {}).get("swap_current_bytes") != 0
            or sample.get("process_t0_swaps") != 0 or sample.get("process_t1_swaps") != 0):
        reasons.append("operation completion/resource/cleanup evidence is incomplete")
    return dict(contract=CONTRACT, qualification_eligible=not reasons,
                status="INELIGIBLE" if reasons else ("PASS" if elapsed <= TARGET_NS else "TARGET_MISS"),
                target_ns=TARGET_NS, eligible_elapsed_ns=None if reasons else elapsed,
                reasons=reasons)


def enforce(row):
    if applies(row.get("identities", {})):
        result = assess(row)
        row.update(cold_qualification=result, qualification_eligible=result["qualification_eligible"],
                   product_target_ns=TARGET_NS,
                   historical_product_target_status="NOT_APPLICABLE_COLD_CONTRACT")
        if row.get("completion_status") == "COMPLETE":
            row["status"] = result["status"]
    return row


def compare(control, candidate, *, order):
    """Performance-only pair report. Ineligible/incompatible pairs have no gain."""
    left, right = assess(control), assess(candidate)
    reasons = left["reasons"] + right["reasons"]
    a, b = control.get("identities", {}), candidate.get("identities", {})
    if not applies(a) or not applies(b) or order not in (["baseline", "candidate"], ["candidate", "baseline"]):
        reasons.append("missing declared cold pair/order")
    if a.get("source_arm") != "baseline" or b.get("source_arm") != "candidate":
        reasons.append("pair does not contain baseline and candidate")
    for identity in (a, b):
        seals = [identity.get("source_identity"), identity.get("product_identity"),
                 identity.get("host_executor", {}).get("binary_sha256")]
        if any(not isinstance(seal, str) or len(seal) != 64 for seal in seals):
            reasons.append("pair lacks source/product/binary custody")
    starts = [control.get("product_command_started_ns"), candidate.get("product_command_started_ns")]
    if all(type(start) is int for start in starts):
        observed = ["baseline", "candidate"] if starts[0] < starts[1] else ["candidate", "baseline"]
        if starts[0] == starts[1] or observed != order:
            reasons.append("pair differs from declared arm order")
    for key in ("family", "case", "seed", "timer", "harness_identity", "topology", "environment", "host_environment"):
        if a.get(key) is None or a.get(key) != b.get(key):
            reasons.append("pair mismatch: " + key)
    for key in ("WORKLOAD_SOURCE_SHA256", "schema_sha256", "rust_toolchain"):
        x, y = a.get("host_executor", {}), b.get("host_executor", {})
        if not x.get(key) or x.get(key) != y.get(key):
            reasons.append("pair mismatch: " + key)
    for key in ("fixture_manifest_sha256", "method", "contract"):
        x, y = control.get("cold_acquisition", {}), candidate.get("cold_acquisition", {})
        if not x.get(key) or x.get(key) != y.get(key):
            reasons.append("pair mismatch: " + key)
    reduction = None if reasons else left["eligible_elapsed_ns"] - right["eligible_elapsed_ns"]
    return dict(contract=CONTRACT, status="INELIGIBLE" if reasons else "ELIGIBLE_COLD_PAIR",
                reduction_ns=reduction,
                reduction_percent=None if reasons else 100 * reduction / left["eligible_elapsed_ns"],
                reasons=reasons, order=order, admission_eligible=False)
