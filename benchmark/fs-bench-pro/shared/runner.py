#!/usr/bin/env python3
"""Host-owned SQLite benchmarks; Docker runs daemon/FUSE/workloads only."""
from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import math
import os
import platform
import re
import shutil
from pathlib import Path
import statistics
import sys
import time
import uuid

HERE = Path(__file__).resolve().parent
BENCH = HERE.parent
REPO = BENCH.parent.parent
sys.path.insert(0, str(HERE))
import runtime

HOST_FAMILIES = ("payload_create_read", "dedup_workspace_reuse", "dedup_cross_file", "dedup_cdc_locality",
                 "edit_length_preserving", "edit_length_changing", "edit_canonical_chunk_count",
                 "init_namespace", "store_footprint", "tiny_file_churn",
                 "namespace_mutation", "directory_construction_traversal",
                 "workspace_change_locality", "dedup_branch_history", "git_tool_workflow",
                 "mixed_load_bearing", "workspace_reliability")
PRODUCT_TARGET_NS = 15_000_000_000
HISTORICAL_PRODUCT_TARGET_SCOPE = (
    "reporting-only historical 15-second family target; not a collection acceptance gate"
)


def performance_target_status(elapsed_ns):
    return "PASS" if elapsed_ns <= PRODUCT_TARGET_NS else "TARGET_MISS"


def issue47_assessment(selection, elapsed_ns):
    if selection.get("family") != "tiny_file_churn" or selection.get("case") not in (
            "tiny-bulk-create-100-mixed-v3", "tiny-bulk-delete-100-mixed-v3"):
        return None
    return {"contract": "issue47-mixed-v3", "target_ns": 1_000_000_000,
            "strict_less_than": True, "timer": "pure_call_sum_ns", "elapsed_ns": elapsed_ns,
            "status": "PASS" if elapsed_ns < 1_000_000_000 else "TARGET_MISS",
            "qualification": "performance-only; final independent proofs pending"}


TIMERS = {"workspace": "pure_call_sum_ns", "sdk": "edit_commit_ns",
          "namespace": "layerstack_init_ns", "store-footprint": "product_call_sum_ns"}


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def harness_identity():
    paths = [HERE / "runner.py", HERE / "runtime.py", BENCH / "verify-selected.py"]
    return digest({str(path.relative_to(BENCH)): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths})


def build_parser(include_modes=True):
    p = argparse.ArgumentParser(description="Full Docker/FUSE workloads; fast means one full sample, never reduced work.")
    p.add_argument("--family", required=True)
    p.add_argument("--topology", choices=("host-store",), default="host-store")
    p.add_argument("--host-binary", default=str(REPO / "target/release/fs-benchmark-pro"))
    p.add_argument("--source-arm", choices=("baseline", "candidate"), default="candidate")
    p.add_argument("--performance-rows", default="-")
    p.add_argument("--case")
    p.add_argument("--seed", type=int)
    p.add_argument("--repetition", type=int)
    p.add_argument("--setup", choices=("fresh", "clone"))
    if include_modes:
        mode = p.add_mutually_exclusive_group()
        mode.add_argument("--perf-fast", action="store_true")
        mode.add_argument("--perf-samples", type=int)
        mode.add_argument("--verification", action="store_true")
    p.add_argument("--prepare-only", action="store_true")
    p.add_argument("--smoke", action="store_true", help="Select the smallest registered supported case; performance/setup only")
    p.add_argument("--list", action="store_true", help="Print registered selections without running workloads")
    p.add_argument("--image", default=os.environ.get("LAYERFS_BENCH_IMAGE"))
    p.add_argument("--source", "--source-identity", dest="source")
    p.add_argument("--input", "--input-identity", dest="input")
    p.add_argument("--output", default=str(REPO / "benchmark-results" / "infra" / ("run-" + uuid.uuid4().hex[:12])))
    p.add_argument("--timeout", type=float, default=130, help="Outer performance command allowance; must exceed --product-timeout; historical 15-second pass target remains reporting-only unless collection-mode is unset")
    p.add_argument("--product-timeout", type=int, default=120, help="Workspace diagnostic product allowance in seconds; does not change historical 15-second reporting")
    p.add_argument("--setup-timeout", type=float, default=120)
    p.add_argument("--collection-mode", action="store_true",
                   help="Statistics collection: completed work is PASS regardless of the historical 15-second target; record that target as reporting-only")
    p.add_argument("--cpus", type=int, default=2)
    p.add_argument("--memory-mib", type=int, default=2048)
    return p


def _deadline(end):
    return runtime.Deadline(end)


def _command(argv, end, **kw):
    return runtime.run(argv, deadline=_deadline(end), **kw)


def _host_command_env(args, selection=None):
    image = (selection or {}).get("image") or getattr(args, "image", None)
    return {"LAYERFS_V013_IMAGE": image} if image else {}


def _text(value):
    return value.decode() if isinstance(value, bytes) else value


def records(output):
    result = []
    for line in _text(output).splitlines():
        start = line.find("{")
        if start >= 0:
            try:
                value = json.loads(line[start:])
                if isinstance(value, dict):
                    result.append(value)
            except json.JSONDecodeError:
                pass
    return result


def initialization_diagnostics(output):
    return [{"kind": "initialization-debug-text", "details": line}
            for line in _text(output).splitlines()
            if line.startswith(("layerfs-initialization-diagnostic-",
                                "layerfs-initialization-producer-",
                                "layerfs-initialization-commits-"))]


def source_build_args():
    def git(*argv):
        return _text(runtime.run(["git", "-C", str(REPO), *argv], deadline=runtime.Deadline.after(10)).stdout).strip()
    paths = sorted(p for root in (REPO / "crates", REPO / "tools", BENCH)
                   for p in root.rglob("*") if p.is_file() and p.suffix in (".rs", ".toml", ".sh", ".py", ".sql")
                   and "target" not in p.parts and "__pycache__" not in p.parts)
    paths += [REPO / "Cargo.toml", REPO / "Cargo.lock", BENCH / "Dockerfile.layerfs"]
    source = hashlib.sha256()
    product = hashlib.sha256()
    for path in paths:
        part = str(path.relative_to(REPO)).encode() + b"\0" + path.read_bytes()
        source.update(part)
        if "crates" in path.relative_to(REPO).parts:
            product.update(part)
    return {"LAYERFS_SOURCE_COMMIT": git("rev-parse", "HEAD"),
            "LAYERFS_SOURCE_TREE": git("rev-parse", "HEAD^{tree}"),
            "LAYERFS_SOURCE_DIRTY": "true" if git("status", "--porcelain") else "false", "LAYERFS_SOURCE_SEAL": source.hexdigest(),
            "LAYERFS_PRODUCT_SEAL": product.hexdigest(),
            "WORKLOAD_SOURCE_SHA256": hashlib.sha256((BENCH / "workload/main.rs").read_bytes()).hexdigest()}


def image_info(image, deadline):
    value = json.loads(_text(_command(["docker", "image", "inspect", image], deadline).stdout))[0]
    if value.get("Config", {}).get("Volumes"):
        raise ValueError("image-declared volumes are forbidden")
    return value


def mixed_fixture_info(args, host_identity, seed, deadline):
    # Cache only the untimed immutable oracle identity, never product/output state.
    key = {"binary_sha256": host_identity["binary_sha256"], "family": args.family,
           "case": args.case, "seed": seed, "profile": "tiny-bulk-mixed-v3"}
    path = HOST_ROOT / "fixture-identities" / (digest(key) + ".json")
    if path.exists():
        saved = json.loads(path.read_text())
        if saved.get("key") != key or saved.get("sha256") != digest(saved.get("fixture")):
            raise ValueError("mixed-v3 fixture identity cache mismatch")
        fixture = saved["fixture"]
    else:
        if getattr(args, "verification", False):
            raise ValueError("mixed-v3 proof requires the matching performance fixture identity cache")
        fixture = records(_command([args.host_binary, "infra-fixture-info", args.family, args.case, str(seed)], deadline).stdout)[-1]
        path.parent.mkdir(parents=True, exist_ok=True)
        staging = path.with_name(path.name + "." + uuid.uuid4().hex + ".tmp")
        try:
            staging.write_text(json.dumps({"key": key, "fixture": fixture, "sha256": digest(fixture)}, sort_keys=True))
            staging.chmod(0o444)
            staging.replace(path)
        finally:
            staging.unlink(missing_ok=True)
    if fixture.get("fixture_profile") != "tiny-bulk-mixed-v3" or not fixture.get("populated_manifest_sha256"):
        raise ValueError("mixed-v3 fixture custody missing")
    return fixture


def resolve_selection(args, deadline):
    if args.topology != "host-store":
        raise ValueError("Docker-owned SQLite is prohibited; use host-store")
    if getattr(args, "_selection", None):
        return args._selection
    if not args.image:
        raise ValueError("select a built Linux image with --image or LAYERFS_BENCH_IMAGE; use shared/runner.py --build-image separately")
    if (not 1 <= args.cpus <= 8 or args.memory_mib <= 0
            or not math.isfinite(args.timeout) or args.timeout <= 0
            or args.product_timeout <= 0 or args.product_timeout >= args.timeout
            or not math.isfinite(args.setup_timeout) or args.setup_timeout <= 0):
        raise ValueError("invalid resource/budget selection")
    if getattr(args, "perf_samples", None) is not None and args.perf_samples <= 0:
        raise ValueError("--perf-samples must be a positive integer")
    info = image_info(args.image, deadline)
    identity = info.get("Config", {}).get("Labels", {}) or {}
    source = identity.get("dev.layerfs.source-seal")
    if not source:
        raise ValueError("image lacks source seal")
    if platform.system() != "Darwin":
        raise ValueError("host Store qualification requires macOS + Docker Desktop")
    if args.family not in HOST_FAMILIES:
        raise ValueError("family is not admitted to host-store execution")
    host_identity = json.loads(Path(args.host_binary + ".identity.json").read_text())
    if runtime.file_sha256(args.host_binary) != host_identity["binary_sha256"]:
        raise ValueError("host binary seal mismatch; rebuild with --build-host")
    if host_identity["LAYERFS_PRODUCT_SEAL"] != identity.get("dev.layerfs.product-seal"):
        raise ValueError("host and Linux image product seals differ")
    source = host_identity["LAYERFS_SOURCE_SEAL"]
    result = _command([args.host_binary, "infra-list", args.family]
                      + ([args.case] if args.case else []), deadline)
    rows = [row for row in records(result.stdout) if row.get("family_id") == args.family]
    if not rows:
        raise ValueError("unknown or archival family: " + args.family)
    if args.list:
        return {"rows": rows, "source_identity": source, "image": info["Id"]}
    if not args.case and args.smoke:
        rows = [row for row in rows if row.get("supported", True) and row.get("smoke_supported", True)]
        if not getattr(args, "verification", False):
            rows = [row for row in rows if not row.get("proof_only")]
        rows.sort(key=lambda r: (r.get("fixture_bytes") or 0, r.get("tier") or 0, r["scenario_id"]))
        if rows:
            args.case = rows[0]["scenario_id"]
        else:
            raise ValueError("no bounded low-tier smoke case: explicit large/proof selections are not run automatically")
    matches = [row for row in rows if row.get("scenario_id") == args.case]
    if len(matches) != 1:
        raise ValueError("select exactly one registered --case (or --smoke for performance)")
    row = matches[0]
    if not row.get("supported", True):
        raise ValueError(row.get("unsupported_reason", "historical/unsupported selection"))
    if args.seed is not None and args.repetition is not None:
        raise ValueError("choose seed or inherited repetition, not both")
    inherited = row.get("inherited", row.get("route") == "sdk" or args.family == "edit_length_changing_capped")
    if inherited and args.seed is not None:
        raise ValueError("inherited cases use --repetition, not --seed")
    if not inherited and args.repetition is not None:
        raise ValueError("this case uses --seed, not --repetition")
    seed = args.repetition if args.repetition is not None else (args.seed if args.seed is not None else 1)
    if not row.get("seed_min", 1) <= seed <= row.get("seed_max", 3):
        raise ValueError("invalid registered seed/repetition")
    fresh = row.get("setup_policy") == "fresh-output"
    if fresh and args.setup == "clone":
        raise ValueError("initialization requires a fresh output Store; clone is not applicable")
    setup = "fresh-output" if fresh else (args.setup or "clone")
    input_recipe = {"family": args.family, "case": args.case, "seed": 1 if inherited else seed,
                    "source": source, "recipe": row}
    fixture_info = None
    if row.get("fixture_profile") == "tiny-bulk-mixed-v3":
        fixture_info = mixed_fixture_info(args, host_identity, seed, deadline)
        input_recipe["fixture"] = fixture_info
    input_identity = digest(input_recipe)
    if args.source and args.source != source:
        raise ValueError("selected source identity does not match image")
    if args.input and args.input != input_identity:
        raise ValueError("selected input identity does not match recipe")
    selection = {**row, "family": args.family, "case": args.case, "seed": seed,
                 "repetition": args.repetition, "source_identity": source,
                 "input_identity": input_identity, "setup_identity": setup,
                 "image": info["Id"], "runtime_image": info["Id"],
                 "product_identity": identity.get("dev.layerfs.product-seal"),
                 "harness_identity": harness_identity(), "environment": {"os": info.get("Os"), "architecture": info.get("Architecture"),
                     "container_cpus": args.cpus, "container_memory_mib": args.memory_mib,
                     "topology": args.topology, "host_cpu_capped": False},
                 "verification_supported": row.get("verification_supported", True)}
    if fixture_info is not None:
        selection["fixture_info"] = fixture_info
    selection["source_arm"] = args.source_arm
    selection["timer"] = TIMERS.get(row.get("route"))
    selection["product_execution_allowance_seconds"] = args.product_timeout
    selection["topology"] = args.topology
    selection.update(host_executor=host_identity, image_source_identity=identity.get("dev.layerfs.source-seal"),
                     host_environment={"os": platform.system(), "architecture": platform.machine(), "cpu_count": os.cpu_count()})
    args._selection = selection
    return selection


HOST_ROOT = REPO / "benchmark-results/host-store"


def _host_acquire(args, selection, deadline):
    sdk = selection.get("route") == "sdk"
    host_env = _host_command_env(args, selection)
    fixture_command = [args.host_binary, "infra-fixture-info", selection["family"], selection["case"], str(selection["seed"])]
    fixture = selection.get("fixture_info") or records(_command(fixture_command, deadline, env=host_env).stdout)[-1]
    native = selection["setup_identity"] == "fresh-output"
    compatibility = {"contract": "sdk-edit-prepared-store-cache-v1" if sdk else "layerfs-canonical-v5-workspace-fixture-v1",
        "fixture": fixture, "schema_sha256": selection["host_executor"]["schema_sha256"]}
    if not sdk and selection.get("route") not in ("namespace", "store-footprint"):
        compatibility["seed"] = selection["seed"]
    if native:
        compatibility.update(family=selection["family"], case=selection["case"])
    if selection["family"] == "git_tool_workflow":
        # The Git reference oracle is case-specific even when the Store fixture is shared.
        compatibility["case"] = selection["case"]
    key = digest(compatibility)
    fresh = selection["setup_identity"] == "fresh"
    root = HOST_ROOT / ("fixtures" if native else "prepared") / key
    if fresh:
        root = HOST_ROOT / "samples" / ("prepare-" + uuid.uuid4().hex)
    hit = root.exists()
    if not hit:
        root.parent.mkdir(parents=True, exist_ok=True)
        staging = HOST_ROOT / "samples" / ("prepare-" + uuid.uuid4().hex)
        staging.parent.mkdir(parents=True, exist_ok=True)
        try:
            if sdk:
                staging.mkdir()
                prepared = records(_command([args.host_binary, "sdk-edit-prepare", str(staging / "payload"),
                                             str(selection["fixture_bytes"])], deadline, env=host_env).stdout)[-1]
                (staging / "payload/branch-id").write_text(prepared["branch_id"])
                qualifications = ["family\tcase\tplan\tinitial\texpected\tfile\tmap\tinitial_count\tfinal_count\tdigest\n"]
                for family in ("edit_length_preserving", "edit_length_changing", "edit_canonical_chunk_count"):
                    listed = records(_command([args.host_binary, "infra-list", family], deadline, env=host_env).stdout)
                    for row in listed:
                        if row.get("fixture_bytes") == selection["fixture_bytes"] and row.get("supported", True):
                            output = _command([args.host_binary, "sdk-edit-qualify", str(staging / "payload"),
                                               prepared["branch_id"], family, row["scenario_id"]], deadline, env=host_env).stdout
                            qualifications.append(_text(output))
                (staging / "qualification.tsv").write_text("".join(qualifications))
                (staging / "manifest.json").write_text(json.dumps({
                    "schema": "fs-bench-infra-prepared-v1",
                    "input_qualification_sha256": runtime.file_sha256(staging / "qualification.tsv")
                }))
            else:
                _command([args.host_binary, "infra-prepare", selection["family"], selection["case"], str(selection["seed"]), str(staging)], deadline, env=host_env)
            (staging / "host-owner.json").write_text(json.dumps({"owner": runtime.OWNER}))
            if not native:
                master = staging / "payload/store.sqlite"
                # Preparation process has exited. Validate a disposable copy before protecting the master.
                checked = staging / "checked.sqlite"
                runtime.closed_store_copy(master, checked, deadline=_deadline(deadline))
                checked.unlink()
                master.rename(staging / "store.sqlite")
                (staging / "store.sqlite").chmod(0o444)
            files = runtime.host_tree_identity(staging, _deadline(deadline))
            manifest = {"compatibility": compatibility, "producer": selection["host_executor"], "created_ns": time.time_ns(),
                        "files": files, "data_bytes": sum(item.get("bytes", 0) for item in files.values())}
            (staging / "host-cache.json").write_text(json.dumps(manifest, sort_keys=True))
            for path in staging.rglob("*"):
                if path.is_file() and not path.is_symlink() and not (native and path.is_relative_to(staging / "payload")):
                    path.chmod(path.stat().st_mode & ~0o222)
            staging.rename(root)
        except BaseException:
            if staging.exists():
                (staging / "host-owner.json").write_text(json.dumps({"owner": runtime.OWNER}))
                runtime.remove_host_owned(staging)
            raise
    manifest = json.loads((root / "host-cache.json").read_text())
    if manifest["compatibility"] != compatibility or json.loads((root / "host-owner.json").read_text()).get("owner") != runtime.OWNER:
        raise ValueError("host prepared cache compatibility/content mismatch")
    # ponytail: owned native inputs trust their prepared recipe; sampled verification
    # detects selected content faults. Recreate this disposable cache if it is modified.
    if not native and runtime.host_tree_identity(root, _deadline(deadline)) != manifest["files"]:
        raise ValueError("host prepared Store content mismatch")
    removed = [] if fresh else runtime.evict_host_cache(HOST_ROOT, root)
    if native:
        fixture = json.loads((root / "fixture.json").read_text())
    return {"image": selection["image"], "host_root": str(root), "cache_key": key, "cache_hit": hit,
            "one_shot": fresh, "producer": manifest["producer"], "compatibility": compatibility,
            "fixture": fixture, "data_bytes": manifest["data_bytes"], "evicted": removed,
            "input_validation": "owned-prepared-recipe" if native else "full-master-identity"}


def _host_sample(prepared, selection, name, deadline):
    master = Path(prepared["host_root"])
    sample = HOST_ROOT / "samples" / name
    sample.mkdir(parents=True, exist_ok=False)
    (sample / "host-owner.json").write_text(json.dumps({"owner": runtime.OWNER}))
    (sample / "payload").mkdir()
    fixture = dict(prepared["fixture"])
    receipt = {"sample_root": str(sample), "prepared_root": str(master), "setup_mode": selection["setup_identity"]}
    if selection["setup_identity"] == "fresh-output":
        if (master / "input-qualification.tsv").exists():
            shutil.copyfile(master / "input-qualification.tsv", sample / "input-qualification.tsv")
        source = master / ("payload" if selection.get("route") in ("namespace", "store-footprint") else "payload/input")
        receipt.update(clone_method="not-applicable", fixture_reuse_method="host-prepared-source",
                       prepared_input_root=str(source), fresh_output_stores=[str(sample / "payload/store.sqlite"), str(sample / "work/store.sqlite")])
    else:
        receipt.update(runtime.closed_store_copy(master / "store.sqlite", sample / "payload/store.sqlite", deadline=_deadline(deadline)))
        fixture["branch_id"] = (master / "payload/branch-id").read_text().strip()
        (sample / "payload/branch-id").write_text(fixture["branch_id"])
        if selection.get("route") == "sdk":
            shutil.copyfile(master / "qualification.tsv", sample / "qualification.tsv")
    encoded = json.dumps(fixture, separators=(",", ":")) + "\n"
    (sample / "fixture.json").write_text(encoded)
    manifest = json.loads((master / "manifest.json").read_text())
    manifest.update(family_id=selection["family"], scenario_id=selection["case"], seed=selection["seed"],
                    fixture_receipt_sha256=hashlib.sha256(encoded.encode()).hexdigest())
    if selection.get("route") == "sdk":
        manifest["input_qualification_sha256"] = runtime.file_sha256(sample / "qualification.tsv")
    (sample / "manifest.json").write_text(json.dumps(manifest, separators=(",", ":")))
    (sample / "selection.tsv").write_text(f"{selection['family']}\t{selection['case']}\t{selection['seed']}\n")
    return receipt


def cgroup_snapshot(sample, deadline):
    command = _command(["docker", "exec", sample.id, "sh", "-c",
        "cat /sys/fs/cgroup/cpu.stat; printf 'memory_peak '; cat /sys/fs/cgroup/memory.peak; "
        "printf 'memory_current '; cat /sys/fs/cgroup/memory.current; "
        "printf 'swap_current '; cat /sys/fs/cgroup/memory.swap.current; "
        "cat /sys/fs/cgroup/memory.events"], deadline)
    result = {}
    for line in _text(command.stdout).splitlines():
        fields = line.split()
        if len(fields) == 2:
            result[fields[0]] = int(fields[1])
    return result


def execute_selected(args, *, deadline, verification=False):
    """No receipt files here: the caller publishes after this function cleans up."""
    started = time.monotonic_ns()
    selection = resolve_selection(args, deadline)
    if selection.get("proof_only") and not verification and not args.prepare_only:
        raise ValueError("proof-only case does not support performance")
    if verification and not selection["verification_supported"]:
        return {"status": "INCOMPLETE", "identities": selection, "omissions": ["proof cannot fit bounded verification"], "cleanup": {"status": "PASS", "not_started": True}}
    sample = None
    sample_name = None
    host_sample_path = None
    prepared = None
    result = {"status": "INCOMPLETE", "identities": selection, "checks": [], "omissions": [],
              "phase": "preparation",
              "sampled_paths_or_ranges": [], "reused_proof_identities": [],
              "resource_precision": "separate host process CPU/RSS/IO and container lifetime peak/command CPU"}
    work_end = deadline - 4
    try:
        setup_started = time.monotonic_ns()
        prepared = _host_acquire(args, selection, work_end)
        result["preparation"] = prepared
        if args.prepare_only:
            result["status"] = "PASS"
            return result
        name = "layerfs-infra-sample-" + uuid.uuid4().hex[:12]
        sample_name = name
        result["phase"] = "runtime-start"
        sample = runtime.start_sample(prepared["image"], name,
            {"family": selection["family"], "run": name}, deadline=_deadline(work_end),
            cpus=args.cpus, memory_bytes=args.memory_mib * 1024**2)
        result["phase"] = "sample-setup"
        result["environment_observation"] = sample.observation
        host_sample_path = HOST_ROOT / "samples" / name
        result["setup"] = _host_sample(prepared, selection, name, work_end)
        if selection["family"] == "git_tool_workflow":
            reference = Path(prepared["host_root"]) / "reference" / "input"
            if not reference.is_dir():
                raise RuntimeError("prepared Git reference tree is missing")
            runtime.install_tree(sample.name, reference, "/qualified/git-reference", _deadline(work_end))
            runtime.ensure_container_dir(sample.name, "/verification", _deadline(work_end))
            result["setup"]["git_reference_install"] = {
                "method": "docker-cp",
                "source": str(reference),
                "destination": "/qualified/git-reference",
                "host_data_sharing_mount": False,
            }
        result["preparation_wall_ns"] = time.monotonic_ns() - setup_started
        before = cgroup_snapshot(sample, work_end)
        run_started = time.monotonic_ns()
        result["phase"] = "product-command"
        command_end = min(work_end, time.monotonic() + (45 if verification else args.timeout))
        command_env = {"LAYERFS_V013_IMAGE": selection["image"],
                       "LAYERFS_BENCH_SOURCE_ARM": selection["source_arm"]}
        if not verification:
            command_env["LAYERFS_BENCH_PRODUCT_TIMEOUT_SECONDS"] = str(args.product_timeout)
        if args.performance_rows != "-":
            command_env["LAYERFS_SDK_EDIT_PERFORMANCE_ROWS"] = args.performance_rows
        if selection["family"] in ("dedup_cross_file", "dedup_cdc_locality"):
            command_env["LAYERFS_INITIALIZATION_DIAGNOSTIC_NONCE"] = selection["input_identity"][:16]
        if selection["family"] == "workspace_reliability":
            prepared_input = str(Path(prepared["host_root"]))
        else:
            prepared_input = result["setup"].get(
                "prepared_input_root", str(Path(prepared["host_root"]) / "payload/input")
            )
        command_env.update(LAYERFS_EXEC_TRANSPORT="daemon", LAYERFS_FUSE_TRANSPORT="daemon",
            LAYERFS_BENCH_WORKLOAD="/usr/local/bin/fs-benchmark-workload",
            LAYERFS_BENCH_PREPARED_INPUT=prepared_input,
            TMPDIR=str(host_sample_path))
        operation = ["infra-run", selection["family"], selection["case"], str(selection["seed"]),
                     "verify" if verification else "performance", str(host_sample_path), sample.id]
        command = _command([args.host_binary, *operation], command_end, env=command_env, output_limit=16 * 1024**2)
        result["command_wall_ns"] = time.monotonic_ns() - run_started
        result["records"] = records(command.stdout)
        result["records"].extend(initialization_diagnostics(command.stderr))
        for record in result["records"]:
            if record.get("kind") == "sampled-canonical-verification":
                result["sampled_paths_or_ranges"].extend(record["sampled_paths_or_ranges"])
                result["omissions"].append("sampled Workspace proof: no exhaustive namespace, full-file bytes, object census, aliases, or failure injection")
        timer, elapsed = _timer(result)
        if command.truncated:
            result["omissions"].append("selected command output exceeded the 16 MiB compact receipt limit")
            if not verification:
                raise RuntimeError("selected command output exceeded compact receipt limit")
        if not verification and elapsed is None and command.returncode == 0:
            raise RuntimeError(f"missing declared product timer: {timer}")
        result["phase"] = "resource-finalization"
        after = cgroup_snapshot(sample, work_end)
        result["resources"] = {"command_window_cpu_ns": (after["usage_usec"] - before["usage_usec"]) * 1000,
            "sample_container_lifetime_peak_bytes": after["memory_peak"],
            "memory_current_bytes": after["memory_current"], "swap_current_bytes": after["swap_current"],
            "oom_kill_delta": after.get("oom_kill", 0) - before.get("oom_kill", 0),
            "measurement_scope": "Linux daemon/FUSE container command window; host coordinator/Store process CPU/RSS/IO reported separately in records; host CPU is not container-capped"}
        result["status"] = "PASS" if command.returncode == 0 and result["records"] and not result["resources"]["oom_kill_delta"] else "FAIL"
        result["slow"] = result["command_wall_ns"] >= 5_000_000_000
        result["checks"] = [r for r in result["records"] if "verif" in str(r.get("kind", "")) or "proof" in str(r.get("kind", ""))]
        if result["status"] != "PASS":
            result["error"] = _text(command.stderr)[-8192:]
        else:
            result["phase"] = "complete"
            if not verification:
                result["completion_status"] = "COMPLETE"
                result["product_target_ns"] = PRODUCT_TARGET_NS
                historical = performance_target_status(elapsed)
                result["historical_product_target_status"] = historical
                result["historical_product_target_scope"] = HISTORICAL_PRODUCT_TARGET_SCOPE
                assessment = issue47_assessment(selection, elapsed)
                if assessment is not None:
                    result["issue47_assessment"] = assessment
                if getattr(args, "collection_mode", False):
                    result["status"] = "PASS"
                    if historical == "TARGET_MISS":
                        result["historical_product_target_note"] = (
                            f"complete product-call sum {elapsed} ns exceeds the historical 15-second target"
                        )
                else:
                    result["status"] = historical
                    if result["status"] == "TARGET_MISS":
                        result["error"] = f"complete product-call sum {elapsed} ns exceeds the 15-second target"
    except Exception as error:
        result["status"] = "TIMEOUT" if isinstance(error, TimeoutError) or "timeout" in str(error).lower() or "deadline" in str(error).lower() else "FAIL"
        failed_command = getattr(error, "result", None)
        detail = _text(failed_command.stderr)[-8192:] if failed_command is not None else ""
        result["error"] = (str(error) + "\n" + detail)[-8192:]
        if failed_command is not None:
            result.setdefault("records", records(failed_command.stdout))
            if result["phase"] == "product-command":
                result["command_wall_ns"] = failed_command.wall_ns
        if any(record.get("kind") == "product-time-budget-exceeded" for record in result.get("records", [])):
            result["status"] = "TIMEOUT"
        result["completion_status"] = "INCOMPLETE"
        result["slow"] = result["status"] == "TIMEOUT"
    finally:
        cleanup_started = time.monotonic_ns()
        try:
            if sample is not None:
                sample.remove(deadline=_deadline(deadline))
            elif sample_name is not None:
                remaining = _command(["docker", "container", "inspect", sample_name], deadline, check=False)
                if remaining.returncode == 0:
                    item = json.loads(_text(remaining.stdout))[0]
                    labels = item.get("Config", {}).get("Labels") or {}
                    if labels.get(runtime.OWNER_LABEL) != runtime.OWNER or labels.get("run") != sample_name:
                        raise RuntimeError("startup cleanup refused mismatched ownership")
                    _command(["docker", "rm", "--force", item["Id"]], deadline)
                elif "No such" not in _text(remaining.stderr):
                    raise RuntimeError("cannot confirm failed-start container cleanup")
            if host_sample_path is not None and host_sample_path.exists():
                runtime.remove_host_owned(host_sample_path)
            if prepared and selection["setup_identity"] != "fresh-output":
                master = Path(prepared["host_root"])
                manifest = json.loads((master / "host-cache.json").read_text())
                if runtime.host_tree_identity(master, _deadline(deadline)) != manifest["files"]:
                    raise RuntimeError("prepared host master changed during sample")
                result["prepared_master_unchanged"] = True
            if prepared and prepared.get("one_shot"):
                runtime.remove_host_owned(prepared["host_root"])
            result["cleanup"] = {"status": "PASS", "wall_ns": time.monotonic_ns() - cleanup_started}
        except Exception as error:
            result["cleanup"] = {"status": "FAIL", "error": str(error)[-2048:]}
            result["status"] = "INCOMPLETE"
        result["wall_ns"] = time.monotonic_ns() - started
    return result


def _timer(row):
    declared = row.get("identities", {}).get("timer")
    for record in reversed(row.get("records", [])):
        keys = (declared,) if declared else ("layerstack_init_ns",) if row.get("identities", {}).get("route") == "namespace" else (
            "edit_commit_ns", "edit_commit_end_ns", "pure_call_sum_ns", "initialize_ns", "execution_ns", "complete_ns", "complete_lifecycle_ns")
        for key in keys:
            if isinstance(record.get(key), (float, int)):
                return key, record[key]
    if declared in (None, "pure_call_sum_ns", "product_call_sum_ns"):
        total = 0
        found = False
        for record in row.get("records", []):
            if record.get("kind") == "phase" and isinstance(record.get("elapsed_ns"), (float, int)):
                total += record["elapsed_ns"]
                found = True
        if found:
            return declared or "pure_call_sum_ns", total
    return declared or "unavailable", None


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    if "--family" in argv and argv[argv.index("--family") + 1:][:1] == ["small_file_delta_smoke"]:
        import small_file_delta_family
        return small_file_delta_family.main(argv)
    if "--deepseek-full" in argv:
        import storage_smoke
        return storage_smoke.main(["--storage-smoke", "deepseek-full", *[arg for arg in argv if arg != "--deepseek-full"]])
    if "--storage-smoke" in argv:
        import storage_smoke
        return storage_smoke.main(argv)
    if argv in (["--build-image"], ["--build-host"], ["--build-storage-smoke-image"]):
        lock_path = Path(os.environ.get("TMPDIR", "/tmp")) / "layerfs-infra-measurement.lock"
        with lock_path.open("a") as lock:
            try:
                fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError as error:
                raise RuntimeError("another benchmark owns the measurement lock") from error
            values = source_build_args()
            if argv == ["--build-host"]:
                binary = REPO / "target/release/fs-benchmark-pro"
                try:
                    result = runtime.run(["cargo", "+1.85.1", "build", "--locked", "--release", "-j2", "-p", "fs-benchmark-pro"],
                        deadline=runtime.Deadline.after(900), cwd=REPO, output_limit=1024**2)
                except runtime.CommandFailure as error:
                    print(_text(error.result.stderr), file=sys.stderr)
                    return error.result.returncode or 1
                version = re.search(r"pub const SCHEMA_VERSION:\s*i64\s*=\s*(\d+)",
                    (REPO / "crates/layerfs-layerstack-store/src/schema.rs").read_text())
                if version is None:
                    raise ValueError("active Store schema version missing")
                schema_path = REPO / f"crates/layerfs-layerstack-store/sql/schema/v{version.group(1)}.sql"
                identity = {**values, "binary_sha256": runtime.file_sha256(binary), "platform": platform.platform(), "rust_toolchain": "1.85.1", "schema_sha256": runtime.file_sha256(schema_path)}
                Path(str(binary) + ".identity.json").write_text(json.dumps(identity, sort_keys=True))
                print(binary)
                return 0
            tag = "layerfs-bench-infra:" + values["LAYERFS_SOURCE_SEAL"][:16]
            if argv == ["--build-storage-smoke-image"]:
                # The owner limits executable verification to three mounted smokes.
                values["LAYERFS_BUILD_SELF_CHECK"] = "0"
            try:
                result = runtime.build_image(REPO, tag, values, deadline=runtime.Deadline.after(900), jobs=2)
            except runtime.CommandFailure as error:
                print(_text(error.result.stderr)[-16384:], file=sys.stderr)
                return error.result.returncode or 1
        if result.returncode:
            print(_text(result.stderr)[-16384:], file=sys.stderr)
            return result.returncode
        print(tag)
        return 0
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.output == parser.get_default("output"):
        args.output = str(HOST_ROOT / "results" / ("run-" + uuid.uuid4().hex[:12]))
    if args.verification:
        parser.error("use the family verify.sh or verify-selected.py for bounded verification")
    if args.prepare_only and (args.perf_fast or args.perf_samples is not None):
        parser.error("preparation-only cannot also select a performance mode")
    # ponytail: one process lock serializes this benchmark; no concurrent resource-sensitive samples.
    lock_path = Path(os.environ.get("TMPDIR", "/tmp")) / "layerfs-infra-measurement.lock"
    with lock_path.open("a") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            parser.error("another benchmark owns the measurement lock")
        selection = resolve_selection(args, time.monotonic() + 20)
        if args.list:
            print(json.dumps(selection, sort_keys=True))
            return 0
        if args.prepare_only:
            result = execute_selected(args, deadline=time.monotonic() + args.setup_timeout)
            print(json.dumps(result, sort_keys=True))
            return 0 if result["status"] == "PASS" else 1
        if selection.get("proof_only"):
            parser.error("proof-only case cannot run performance")
        count = args.perf_samples or 1
        output = Path(args.output)
        output.mkdir(parents=True, exist_ok=True)
        path = output / "perf.jsonl"
        samples = []
        with path.open("x") as stream:
            def emit(row):
                stream.write(json.dumps(row, sort_keys=True, separators=(",", ":")) + "\n")
                stream.flush()
            emit({"kind": "header", "schema": "layerfs-perf-v1", "identities": selection,
                  "requested_samples": count, "full_workload": True, "cpus": args.cpus,
                  "product_target_ns": PRODUCT_TARGET_NS, "command_allowance_seconds": args.timeout,
                  "product_execution_allowance_seconds": args.product_timeout,
                  "collection_mode": bool(args.collection_mode),
                  "historical_product_target_scope": HISTORICAL_PRODUCT_TARGET_SCOPE,
                  "memory_mib": args.memory_mib, "resource_limit_scope": "Linux container only; host CPU not capped", "verification_status": "NOT_RUN"})
            for index in range(1, count + 1):
                row = execute_selected(args, deadline=time.monotonic() + args.setup_timeout + args.timeout + 10)
                row.update(kind="sample", sample_index=index)
                emit(row)
                samples.append(row)
                key, value = _timer(row)
                print(f"{args.family} {args.case} sample={index} {row['status']} {key}={value} historical_target={row.get('historical_product_target_status')} slow={row.get('slow', False)}", flush=True)
                if row["status"] not in ("PASS", "TARGET_MISS"):
                    (output / "failure.log").write_text(str(row.get("error", row.get("cleanup")))[:1024**2])
                    break
                if row["status"] == "TARGET_MISS" and not args.collection_mode:
                    (output / "failure.log").write_text(str(row.get("error", row.get("cleanup")))[:1024**2])
                    break
            valid = [row for row in samples if row["status"] == "PASS"]
            completed = [row for row in samples if row["status"] in ("PASS", "TARGET_MISS")]
            times = [value for row in completed if (value := _timer(row)[1]) is not None]
            if args.collection_mode:
                summary_status = "PASS" if len(valid) == count else "INCOMPLETE"
            else:
                summary_status = "PASS" if len(valid) == count else ("TARGET_MISS" if len(completed) == count else "INCOMPLETE")
            summary = {"kind": "summary", "requested": count, "attempted": len(samples), "valid": len(valid),
                       "completed": len(completed), "product_target_ns": PRODUCT_TARGET_NS,
                       "collection_mode": bool(args.collection_mode),
                       "historical_product_target_scope": HISTORICAL_PRODUCT_TARGET_SCOPE,
                       "status": summary_status, "verification_status": "NOT_RUN",
                       "timer": _timer(completed[0])[0] if completed else None,
                       "median_ns": statistics.median(times) if times else None,
                       "min_ns": min(times) if times else None, "max_ns": max(times) if times else None}
            emit(summary)
        return 0 if summary["status"] == "PASS" else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ValueError, RuntimeError, TimeoutError) as error:
        print(str(error), file=sys.stderr)
        raise SystemExit(1)
