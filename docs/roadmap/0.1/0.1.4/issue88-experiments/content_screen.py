#!/usr/bin/env python3
"""Offline framed-content screen. Only execute under the separately sealed contract.

Input JSONL, one unique target per line in chronological order:
  id, sha256, checkpoint, ordinal, source_path, offset, length, unit_kind, prior_id,
  prior_provenance (null iff prior_id is null).
`sha256` authenticates the exact encoded unit. Original ObjectId/Git identity and
prior correspondence authentication belong to the shared extractor and its seal.
No product Store or input is written. Output is an experimental container, not a
LayerFS Store; every frame, base and index byte is counted separately.
"""
import argparse
import csv
import ctypes as C
import hashlib
import json
import os
from pathlib import Path
import resource
import shutil
import sqlite3
import struct
import sys
import time

FILE_HEADER = struct.Struct("<8sII")
RECORD_HEADER = struct.Struct("<BBHIQQ32s")
MAGIC = b"LFS88PFX"


def sha256_file(path):
    digest = hashlib.sha256()
    with open(path, "rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


class Zstd:
    """Existing native codec, explicit bounded units and per-frame parameters."""
    def __init__(self, library, level=3, window_log=20):
        self.path = Path(library).resolve(strict=True)
        self.lib = C.CDLL(str(self.path))
        pointer, size, integer = C.c_void_p, C.c_size_t, C.c_int
        signatures = {
            "ZSTD_versionNumber": (C.c_uint, []),
            "ZSTD_createCCtx": (pointer, []), "ZSTD_createDCtx": (pointer, []),
            "ZSTD_freeCCtx": (size, [pointer]), "ZSTD_freeDCtx": (size, [pointer]),
            "ZSTD_CCtx_reset": (size, [pointer, integer]),
            "ZSTD_DCtx_reset": (size, [pointer, integer]),
            "ZSTD_CCtx_setParameter": (size, [pointer, integer, integer]),
            "ZSTD_DCtx_setParameter": (size, [pointer, integer, integer]),
            "ZSTD_CCtx_refPrefix": (size, [pointer, pointer, size]),
            "ZSTD_DCtx_refPrefix": (size, [pointer, pointer, size]),
            "ZSTD_compressBound": (size, [size]),
            "ZSTD_compress2": (size, [pointer, pointer, size, pointer, size]),
            "ZSTD_decompressDCtx": (size, [pointer, pointer, size, pointer, size]),
            "ZSTD_findFrameCompressedSize": (size, [pointer, size]),
            "ZSTD_getFrameContentSize": (C.c_ulonglong, [pointer, size]),
            "ZSTD_isError": (C.c_uint, [size]),
            "ZSTD_getErrorName": (C.c_char_p, [size]),
            "ZSTD_sizeof_CCtx": (size, [pointer]),
            "ZSTD_sizeof_DCtx": (size, [pointer]),
        }
        for name, (result, arguments) in signatures.items():
            function = getattr(self.lib, name)
            function.restype, function.argtypes = result, arguments
        self.version = self.lib.ZSTD_versionNumber()
        require(self.version == 10507, "contract requires native Zstd1.5.7")
        self.encoder, self.decoder = self.lib.ZSTD_createCCtx(), self.lib.ZSTD_createDCtx()
        require(self.encoder and self.decoder, "Zstd context allocation")
        self.level, self.window_log = level, window_log
        self.peak_encoder_bytes = self.peak_decoder_bytes = 0

    def checked(self, value):
        if self.lib.ZSTD_isError(value):
            raise ValueError(self.lib.ZSTD_getErrorName(value).decode())
        return value

    def compress(self, raw, prefix=None):
        self.checked(self.lib.ZSTD_CCtx_reset(self.encoder, 3))
        for parameter, value in ((100, self.level), (101, self.window_log),
                                 (200, 1), (201, 1), (202, 0), (400, 0)):
            self.checked(self.lib.ZSTD_CCtx_setParameter(self.encoder, parameter, value))
        self.checked(self.lib.ZSTD_CCtx_refPrefix(self.encoder, prefix, len(prefix) if prefix is not None else 0))
        output = C.create_string_buffer(self.checked(self.lib.ZSTD_compressBound(len(raw))))
        length = self.checked(self.lib.ZSTD_compress2(self.encoder, output, len(output), raw, len(raw)))
        self.peak_encoder_bytes = max(self.peak_encoder_bytes, self.lib.ZSTD_sizeof_CCtx(self.encoder))
        return output.raw[:length]

    def decompress(self, frame, expected_length, prefix=None):
        require(self.checked(self.lib.ZSTD_findFrameCompressedSize(frame, len(frame))) == len(frame),
                "trailing or concatenated frame bytes")
        require(self.lib.ZSTD_getFrameContentSize(frame, len(frame)) == expected_length,
                "frame content size")
        self.checked(self.lib.ZSTD_DCtx_reset(self.decoder, 3))
        self.checked(self.lib.ZSTD_DCtx_setParameter(self.decoder, 100, self.window_log))
        self.checked(self.lib.ZSTD_DCtx_refPrefix(self.decoder, prefix, len(prefix) if prefix is not None else 0))
        output = C.create_string_buffer(max(1, expected_length))
        length = self.checked(self.lib.ZSTD_decompressDCtx(self.decoder, output, expected_length, frame, len(frame)))
        require(length == expected_length, "reconstructed length")
        self.peak_decoder_bytes = max(self.peak_decoder_bytes, self.lib.ZSTD_sizeof_DCtx(self.decoder))
        return output.raw[:length]

    def close(self):
        if self.encoder:
            self.lib.ZSTD_freeCCtx(self.encoder)
            self.encoder = None
        if self.decoder:
            self.lib.ZSTD_freeDCtx(self.decoder)
            self.decoder = None


def read_exact(stream, offset, length):
    stream.seek(offset)
    data = stream.read(length)
    require(len(data) == length, "truncated input or frame")
    return data


def validate_file_header(stream):
    require(FILE_HEADER.unpack(read_exact(stream, 0, FILE_HEADER.size)) == (MAGIC, 1, RECORD_HEADER.size),
            "experimental container header")


def row_bytes(row):
    require(row["offset"] >= 0 and row["length"] >= 0, "negative source range")
    with open(row["source_path"], "rb") as source:
        raw = read_exact(source, row["offset"], row["length"])
    require(hashlib.sha256(raw).hexdigest() == row["sha256"], "input SHA256 mismatch")
    return raw


def index_create(path):
    index = sqlite3.connect(path)
    index.row_factory = sqlite3.Row
    index.execute("PRAGMA journal_mode=DELETE")
    index.execute("PRAGMA cache_size=-8192")
    index.execute("""CREATE TABLE objects (
      id TEXT PRIMARY KEY, sha256 TEXT NOT NULL, checkpoint INTEGER NOT NULL,
      ordinal INTEGER NOT NULL UNIQUE, length INTEGER NOT NULL,
      full_offset INTEGER NOT NULL, full_length INTEGER NOT NULL,
      selected_offset INTEGER NOT NULL, selected_length INTEGER NOT NULL,
      base_id TEXT, depth INTEGER NOT NULL, closure_bytes INTEGER NOT NULL)
    """)
    return index


def write_record(output, raw, frame, base_sha256, depth, closure_bytes):
    offset = output.tell()
    digest = hashlib.sha256(raw).digest()
    output.write(RECORD_HEADER.pack(bool(base_sha256), depth, 0, len(frame), len(raw), closure_bytes, digest))
    if base_sha256:
        output.write(bytes.fromhex(base_sha256))
    output.write(frame)
    return offset, RECORD_HEADER.size + 32 * bool(base_sha256) + len(frame)


def reconstruct(index, source, identifier, codec, max_depth, max_closure, stats):
    """No decoded-object cache. Walk one bounded single-base closure iteratively."""
    chain, seen = [], set()
    current = identifier
    while current is not None:
        require(current not in seen, "physical dependency cycle")
        seen.add(current)
        row = index.execute("SELECT * FROM objects WHERE id=?", (current,)).fetchone()
        require(row is not None, "missing physical base")
        chain.append(row)
        require(len(chain) <= max_depth + 1, "depth cap")
        current = row["base_id"]
    require(sum(row["length"] for row in chain) <= max_closure, "closure cap")
    prefix, previous = None, None
    for row in reversed(chain):
        header = read_exact(source, row["selected_offset"], RECORD_HEADER.size)
        kind, depth, reserved, encoded_length, output_length, closure, digest = RECORD_HEADER.unpack(header)
        require(reserved == 0 and kind in (0, 1), "record kind/reserved fields")
        require(encoded_length + RECORD_HEADER.size + 32 * kind == row["selected_length"], "record coverage")
        require(output_length == row["length"] and digest.hex() == row["sha256"], "record identity fields")
        require(depth == row["depth"] and closure == row["closure_bytes"], "record dependency counters")
        require(bool(kind) == (previous is not None), "record dependency kind")
        if kind:
            base_digest = read_exact(source, row["selected_offset"] + RECORD_HEADER.size, 32)
            require(base_digest == bytes.fromhex(previous["sha256"]), "base identity")
        require(depth == (previous["depth"] + 1 if previous else 0), "dependency depth")
        require(closure == output_length + (previous["closure_bytes"] if previous else 0), "dependency closure")
        frame = read_exact(source, row["selected_offset"] + RECORD_HEADER.size + 32 * kind, encoded_length)
        prefix = codec.decompress(frame, output_length, prefix)
        require(hashlib.sha256(prefix).digest() == digest, "reconstructed SHA256 mismatch")
        stats["frame_reads"] += 1
        stats["encoded_bytes"] += encoded_length + RECORD_HEADER.size + 32 * kind
        stats["decoded_bytes"] += output_length
        previous = row
    return prefix


def run(args):
    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=False)
    codec = Zstd(args.library, args.level, args.window_log)
    index = index_create(output / "index.sqlite")
    source_hash = sha256_file(args.manifest)
    contracts = {str(Path(path).resolve()): sha256_file(path) for path in args.contract}
    totals = dict(targets=0, target_bytes=0, prefix_candidates=0, prefix_candidate_frame_bytes=0,
                  full_frame_bytes=0, selected_frame_bytes=0, prefix_selected=0,
                  prefix_selected_target_bytes=0, full_selected=0, full_selected_target_bytes=0,
                  no_prior=0, base_size_limit=0, target_size_limit=0, closure_limit=0,
                  depth_limit=0, prefix_rejected=0, encode_full_ns=0, encode_prefix_ns=0,
                  candidate_base_reconstruct_ns=0, verification_ns=0)
    canonical = dict(chunk_targets=0, chunk_payload_bytes=0, chunk_canonical_bytes=0,
                     file_targets=0, file_payload_bytes=0, file_canonical_bytes=0)
    base_reads = dict(frame_reads=0, encoded_bytes=0, decoded_bytes=0)
    verification_reads = dict(frame_reads=0, encoded_bytes=0, decoded_bytes=0)
    fields = ["ordinal", "checkpoint", "id", "sha256", "length", "unit_kind", "canonical_id", "canonical_length", "prior_id", "prior_provenance",
              "outcome", "depth", "closure_bytes", "full_frame_bytes", "prefix_frame_bytes",
              "selected_frame_bytes", "full_container_offset", "selected_container_offset"]
    started = time.perf_counter_ns()
    minimum_free = shutil.disk_usage(output).free
    max_rss = 0
    def budget_check():
        nonlocal minimum_free, max_rss
        require(time.perf_counter_ns() - started <= 4 * 3600 * 1000000000, "four-hour wall cap")
        minimum_free = min(minimum_free, shutil.disk_usage(output).free)
        require(minimum_free >= 50 * 1024**3, "50GiB free disk reserve")
        rss = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
        max_rss = max(max_rss, rss if sys.platform == "darwin" else rss * 1024)
        require(max_rss <= 8 * 1024**3, "8GiB lifetime process peak RSS cap")
        require(sum(path.stat().st_size for path in output.iterdir() if path.is_file()) <= 32 * 1024**3,
                "32GiB output logical size cap")
    usage_before = resource.getrusage(resource.RUSAGE_SELF)
    try:
        with open(output / "full.frames", "x+b") as full, open(output / "selected.frames", "x+b") as selected, \
             open(output / "objects.csv", "x", newline="") as receipts, open(args.manifest) as manifest:
            for stream in (full, selected):
                stream.write(FILE_HEADER.pack(MAGIC, 1, RECORD_HEADER.size))
            writer = csv.DictWriter(receipts, fields)
            writer.writeheader()
            previous_checkpoint = 0
            for ordinal, line in enumerate(manifest):
                row = json.loads(line)
                require(row["ordinal"] == ordinal, "nonconsecutive ordinal")
                require(previous_checkpoint <= row["checkpoint"] <= 157, "checkpoint order/bounds")
                if row["checkpoint"] != previous_checkpoint or ordinal % 1024 == 0:
                    budget_check()
                previous_checkpoint = row["checkpoint"]
                require(row["length"] <= args.max_input_bytes, "input unit exceeds frozen cap")
                require(row.get("unit_kind") in ("chunk", "file"), "unit kind required")
                require(row["length"] <= (32768 if row["unit_kind"] == "chunk" else 262144), "unit size limit")
                require(args.arm != "S2" or row["unit_kind"] == "chunk", "S2 must retain chunk units")
                require(len(row["sha256"]) == 64 and len(bytes.fromhex(row["sha256"])) == 32, "SHA256 width")
                require(isinstance(row["id"], str) and row["id"], "missing source identity")
                require(index.execute("SELECT 1 FROM objects WHERE id=?", (row["id"],)).fetchone() is None,
                        "duplicate target identity")
                raw = row_bytes(row)
                kind = row["unit_kind"]
                if kind == "file":
                    require(row["id"] == "file:" + row["sha256"], "file raw-content index identity")
                    canonical_id = hashlib.sha256(b"LFS88FC\0" + len(raw).to_bytes(8, "big") + raw).hexdigest()
                    canonical_length = len(raw) + 16
                else:
                    canonical_id = row.get("existing_canonical_id")
                    require(isinstance(canonical_id, str) and row["id"] == "chunk:" + canonical_id,
                            "missing sealed extractor canonical identity")
                    require(row.get("canonical_length") == len(raw) + 21, "canonical chunk length")
                    canonical_length = len(raw) + 21
                canonical[kind + "_targets"] += 1
                canonical[kind + "_payload_bytes"] += len(raw)
                canonical[kind + "_canonical_bytes"] += canonical_length
                then = time.perf_counter_ns()
                full_frame = codec.compress(raw)
                totals["encode_full_ns"] += time.perf_counter_ns() - then
                then = time.perf_counter_ns()
                require(codec.decompress(full_frame, len(raw)) == raw, "FULL roundtrip")
                totals["verification_ns"] += time.perf_counter_ns() - then
                chosen, depth, closure, base = full_frame, 0, len(raw), None
                prefix_length = None
                prior = row.get("prior_id")
                if prior is None:
                    require(row.get("prior_provenance") is None, "provenance without prior")
                    outcome = "no_prior"
                else:
                    require(bool(row.get("prior_provenance")), "unproven prior")
                    require(prior.split(":", 1)[0] == row["unit_kind"], "cross-unit predecessor forbidden")
                    base = index.execute("SELECT * FROM objects WHERE id=?", (prior,)).fetchone()
                    require(base is not None and base["ordinal"] < ordinal, "future/unavailable prior")
                    # Frozen primary arms require prior checkpoint, not just earlier row.
                    require(base["checkpoint"] < row["checkpoint"], "same-checkpoint prior forbidden")
                    if len(raw) > args.prefix_max_bytes:
                        outcome = "target_size_limit"
                    elif base["length"] > args.prefix_max_bytes:
                        outcome = "base_size_limit"
                    elif base["depth"] + 1 > args.max_depth:
                        outcome = "depth_limit"
                    elif base["closure_bytes"] + len(raw) > args.max_closure_bytes:
                        outcome = "closure_limit"
                    else:
                        selected.flush()
                        then = time.perf_counter_ns()
                        prefix = reconstruct(index, selected, prior, codec, args.max_depth, args.max_closure_bytes, base_reads)
                        totals["candidate_base_reconstruct_ns"] += time.perf_counter_ns() - then
                        then = time.perf_counter_ns()
                        candidate = codec.compress(raw, prefix)
                        totals["encode_prefix_ns"] += time.perf_counter_ns() - then
                        then = time.perf_counter_ns()
                        require(codec.decompress(candidate, len(raw), prefix) == raw, "PREFIX roundtrip")
                        totals["verification_ns"] += time.perf_counter_ns() - then
                        prefix_length = len(candidate)
                        totals["prefix_candidates"] += 1
                        totals["prefix_candidate_frame_bytes"] += prefix_length
                        if prefix_length + 32 < len(full_frame):
                            chosen, depth = candidate, base["depth"] + 1
                            closure = base["closure_bytes"] + len(raw)
                            outcome = "prefix_selected"
                        else:
                            outcome = "prefix_rejected"
                        del prefix, candidate
                # Reconstruction seeks selected: always restore append position.
                selected.seek(0, os.SEEK_END)
                full_offset, full_size = write_record(full, raw, full_frame, None, 0, len(raw))
                selected_offset, selected_size = write_record(selected, raw, chosen,
                        base["sha256"] if outcome == "prefix_selected" else None, depth, closure)
                index.execute("INSERT INTO objects VALUES (?,?,?,?,?,?,?,?,?,?,?,?)", (
                    row["id"], row["sha256"], row["checkpoint"], ordinal, len(raw),
                    full_offset, full_size, selected_offset, selected_size,
                    prior if outcome == "prefix_selected" else None, depth, closure))
                totals["targets"] += 1
                totals["target_bytes"] += len(raw)
                totals["full_frame_bytes"] += len(full_frame)
                totals["selected_frame_bytes"] += len(chosen)
                totals[outcome] += 1
                if outcome == "prefix_selected":
                    totals["prefix_selected_target_bytes"] += len(raw)
                else:
                    totals["full_selected"] += 1
                    totals["full_selected_target_bytes"] += len(raw)
                writer.writerow({**{key: row.get(key) for key in fields if key in row},
                    "canonical_id": canonical_id, "canonical_length": canonical_length,
                    "outcome": outcome, "depth": depth, "closure_bytes": closure,
                    "full_frame_bytes": len(full_frame), "prefix_frame_bytes": prefix_length,
                    "selected_frame_bytes": len(chosen), "full_container_offset": full_offset,
                    "selected_container_offset": selected_offset})
                if totals["targets"] % 1024 == 0:
                    index.commit()
            index.commit()
            full.flush()
            selected.flush()
            validate_file_header(full)
            validate_file_header(selected)
            # Fixed no-decoded-cache read probe: every selected object exactly once
            # in ordinal order. OS cache remains uncontrolled and explicitly warm.
            then = time.perf_counter_ns()
            for number, row in enumerate(index.execute("SELECT id FROM objects ORDER BY ordinal")):
                if number % 1024 == 0:
                    budget_check()
                reconstruct(index, selected, row["id"], codec, args.max_depth, args.max_closure_bytes, verification_reads)
            totals["verification_ns"] += time.perf_counter_ns() - then
            # Fixed deterministic read probes: first target, largest target and
            # deepest selected chain. Repeated identities are probed only once.
            probe_ids = []
            for order in ("ordinal", "length DESC, ordinal", "depth DESC, ordinal"):
                identity = index.execute("SELECT id FROM objects ORDER BY " + order + " LIMIT 1").fetchone()
                if identity and identity[0] not in probe_ids:
                    probe_ids.append(identity[0])
            read_probes = []
            for identity in probe_ids:
                for mode in ("small_range", "full"):
                    fresh_codec = Zstd(args.library, args.level, args.window_log)
                    try:
                        stats = dict(frame_reads=0, encoded_bytes=0, decoded_bytes=0)
                        then = time.perf_counter_ns()
                        decoded = reconstruct(index, selected, identity, fresh_codec, args.max_depth, args.max_closure_bytes, stats)
                        returned = decoded[:min(4096, len(decoded))] if mode == "small_range" else decoded
                        elapsed = time.perf_counter_ns() - then
                        # Warm repeat has a one-object decoded cache, explicitly
                        # charged by its bytes and never used in encoding or the
                        # full authentication traversal.
                        then = time.perf_counter_ns()
                        warm = decoded[:min(4096, len(decoded))] if mode == "small_range" else decoded
                        warm_elapsed = time.perf_counter_ns() - then
                        require(warm == returned, "warm read equality")
                        read_probes.append(dict(id=identity, mode=mode, returned_bytes=len(returned),
                            fresh_elapsed_ns=elapsed, warm_elapsed_ns=warm_elapsed,
                            fresh_condition="fresh_decoder_empty_application_cache_os_cache_uncontrolled",
                            warm_condition="one_target_decoded_cache_os_cache_uncontrolled",
                            decoded_cache_bytes=len(decoded), fresh_work=stats,
                            warm_work=dict(frame_reads=0, encoded_bytes=0, decoded_bytes=0)))
                    finally:
                        fresh_codec.close()
        require(totals["targets"] > 0, "empty manifest")
        require(totals["targets"] == totals["full_selected"] + totals["prefix_selected"], "count partition")
        require(totals["target_bytes"] == totals["full_selected_target_bytes"] + totals["prefix_selected_target_bytes"], "byte partition")
        for filename, frames, base_count in (("full.frames", totals["full_frame_bytes"], 0), ("selected.frames", totals["selected_frame_bytes"], totals["prefix_selected"])):
            require((output / filename).stat().st_size == FILE_HEADER.size + totals["targets"] * RECORD_HEADER.size + base_count * 32 + frames,
                    "container byte conservation")
        require(sha256_file(args.manifest) == source_hash, "manifest changed during execution")
        require(all(sha256_file(path) == digest for path, digest in contracts.items()), "contract changed during execution")
        budget_check()
        index.close()
        index = None
        usage_after = resource.getrusage(resource.RUSAGE_SELF)
        files = {}
        for path in sorted(output.iterdir()):
            stat = path.stat()
            files[path.name] = {"logical_bytes": stat.st_size, "allocated_bytes": stat.st_blocks * 512,
                                "sha256": sha256_file(path)}
        result = {
            "schema": "issue88-content-screen-v1", "status": "PASS", "arm": args.arm,
            "scope": "offline encoding potential; experimental framed content and analysis index, not LayerFS allocation or public performance",
            "population": "unique chronological authenticated units in sealed shared extractor manifest; includes FULL fallbacks and physical base closure",
            "units": "integer bytes/ns/counts; JSON null means unattempted candidate",
            "manifest_sha256": source_hash,
            "contract_sha256": contracts,
            "settings": {"level": args.level, "window_log": args.window_log, "checksum": True,
                         "content_size": True, "dictionary_id": False, "threads": 0,
                         "max_depth": args.max_depth, "max_closure_bytes": args.max_closure_bytes,
                         "prefix_max_bytes": args.prefix_max_bytes, "max_input_bytes": args.max_input_bytes,
                         "base_policy": "single extractor-authenticated prior from strictly earlier checkpoint; no anchor substitution or future base",
                         "winner_policy": "prefix frame plus32-byte baseID strictly smaller than FULL frame; shared56-byte record header"},
            "identity": {"library_path": str(codec.path), "library_sha256": sha256_file(codec.path),
                         "zstd_version_number": codec.version, "tool_sha256": sha256_file(__file__)},
            "totals": totals,
            "canonical": {**canonical,
                "units": "counts and canonical/payload bytes, logical dimension not added to frames",
                "existing_canonical_authentication": "derived from exact sealed extractor canonical ID and every target raw SHA256/length roundtrip; canonical21-byte framing deterministic; no direct decoder BLAKE3 claim",
                "file_canonical_authentication": "SHA256 of LFS88FC-NUL + u64be payload length + reconstructed payload; experimental identity, not existing LayerFS format"},
            "framing": {"file_header_bytes": FILE_HEADER.size, "record_header_bytes": RECORD_HEADER.size,
                        "record_header_population": totals["targets"], "prefix_base_id_bytes": 32,
                        "prefix_base_id_population": totals["prefix_selected"]},
            "candidate_base_reads": base_reads,
            "verification_reads": {**verification_reads, "population": "every selected object once, all dependencies included; SHA256 verified",
                                   "decoded_object_cache": "none", "os_cache": "uncontrolled/warmed by generation; not cold IO"},
            "read_probes": read_probes,
            "resources": {"elapsed_ns": time.perf_counter_ns() - started,
                          "user_cpu_ns": round((usage_after.ru_utime - usage_before.ru_utime) * 1000000000),
                          "system_cpu_ns": round((usage_after.ru_stime - usage_before.ru_stime) * 1000000000),
                          "process_lifetime_maxrss_platform_units": usage_after.ru_maxrss,
                          "process_lifetime_peak_rss_bytes": max_rss,
                          "minimum_sampled_free_disk_bytes": minimum_free,
                          "peak_codec_encoder_bytes": codec.peak_encoder_bytes,
                          "peak_codec_decoder_bytes": codec.peak_decoder_bytes,
                          "process_read_write_bytes": None,
                          "process_read_write_unknown_reason": "not supplied by this portable in-process observer"},
            "files": files,
            "limitations": ["no complete LayerFS metadata/index/page layout in this container", "original allocation is not inferred from copied content",
                            "physical base records are selected earlier targets; shared bases not duplicated", "no product timing, migration or release claim"],
        }
        (output / "result.json").write_text(json.dumps(result, indent=2) + "\n")
        return result
    except Exception as error:
        (output / "failure.json").write_text(json.dumps({
            "schema": "issue88-content-screen-v1", "status": "FAIL", "arm": args.arm,
            "manifest_sha256": source_hash, "error_type": type(error).__name__,
            "error": str(error), "partial_totals": totals,
            "scope": "failed offline screen; preserve partial output, never use as complete result",
        }, indent=2) + "\n")
        raise
    finally:
        if index is not None:
            index.close()
        codec.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--arm", required=True, choices=["S2", "S3", "synthetic"])
    parser.add_argument("--library", required=True)
    parser.add_argument("--contract", action="append", required=True)
    parser.add_argument("--level", type=int, default=3)
    parser.add_argument("--window-log", type=int, default=20)
    parser.add_argument("--max-depth", type=int, default=4)
    parser.add_argument("--max-closure-bytes", type=int, default=1048576)
    parser.add_argument("--prefix-max-bytes", type=int, default=262144)
    parser.add_argument("--max-input-bytes", type=int, default=262144)
    args = parser.parse_args()
    require(args.level == 3 and args.window_log == 20, "frozen primary codec settings")
    require(args.max_depth == 4 and args.max_closure_bytes == 1048576, "frozen dependency bounds")
    require(args.prefix_max_bytes == args.max_input_bytes == 262144, "frozen unit bounds")
    result = run(args)
    print(json.dumps({"status": result["status"], "arm": result["arm"], "totals": result["totals"]}))


if __name__ == "__main__":
    main()
