"""Run the frozen full-venv proof; build the probe first. Owns the measurement lock."""
from pathlib import Path
from decimal import Decimal
import fcntl, hashlib, json, os, re, signal, subprocess, sys, tarfile, time

here = Path(__file__).resolve().parent
repo = here.parents[4]
plan = json.loads((here / "plan.json").read_text())
out = Path(sys.argv[1]).resolve()
assert not out.exists(), "fresh output directory required"
source = Path(plan["source"])
archive = Path(plan["archive"]["archive"])
binary = repo / "target/release/layerfs-directory-proof"

def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()

def save(name, value):
    (out / name).write_text(json.dumps(value, indent=2) + "\n")

def execute(argv, log, timeout, env=None):
    with log.open("x") as stream:
        process = subprocess.Popen(argv, stdout=stream, stderr=subprocess.STDOUT,
                                   env=env, start_new_session=True)
        try:
            return process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            return 124

with (Path(os.environ.get("TMPDIR", "/tmp")) / "layerfs-infra-measurement.lock").open("a") as lock:
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    out.mkdir()
    assert digest(archive) == plan["archive"]["sha256"], "archive identity changed"
    expected = {"."}
    for base, dirs, files in os.walk(source, followlinks=False):
        expected.update(str((Path(base) / name).relative_to(source)) for name in dirs + files)
    total = 0
    with tarfile.open(archive) as tar:
        members = tar.getmembers()
        assert {str(Path(m.name)) for m in members} == expected, "source entry set changed"
        assert len(members) == len(expected), "duplicate archive entries"
        for member in members:
            path = source / member.name
            stat = path.lstat()
            assert stat.st_mtime_ns == int(Decimal(member.pax_headers.get("mtime", str(member.mtime))) * 10**9), path
            assert stat.st_mode & 0o7777 == member.mode, path
            if member.issym():
                assert path.is_symlink() and os.readlink(path) == member.linkname, path
            elif member.isdir():
                assert path.is_dir() and not path.is_symlink(), path
            else:
                assert path.is_file() and not path.is_symlink(), path
                with tar.extractfile(member) as archived, path.open("rb") as native:
                    while True:
                        block = archived.read(1024 * 1024)
                        assert native.read(len(block)) == block, path
                        if not block:
                            assert native.read(1) == b"", path
                            break
                        total += len(block)
    save("input-custody.json", {"status": "PASS", "archive_sha256": digest(archive),
        "entries_including_root": len(expected), "regular_file_bytes": total,
        "coverage": "every source entry, full file bytes, modes, nsmtime and symlink targets against frozen archive"})
    identity = {"plan": plan, "source_commit": subprocess.check_output(["git", "-C", str(repo), "rev-parse", "HEAD"], text=True).strip(),
        "probe_sha256": digest(binary), "probe_source_sha256": digest(here / "probe/src/main.rs"),
        "probe_lock_sha256": digest(here / "probe/Cargo.lock"),
        "image": json.loads(subprocess.check_output(["docker", "image", "inspect", plan["image"]]))[0]}
    save("identity.json", identity)
    results = []
    for sample in range(1, plan["samples"] + 1):
        prefix = "sample-" + str(sample)
        store = out / prefix
        log = out / (prefix + ".log")
        argv = [str(binary), "workspace", str(archive), str(store), plan["image"]]
        save(prefix + "-command.json", {"argv": argv, "timeout_seconds": 480})
        start = time.monotonic()
        code = execute(argv, log, 480)
        text = log.read_text()
        names = re.findall(r"^container=(.+)$", text, re.M)
        row = {"sample": sample, "exit_code": code, "command_wall_seconds": time.monotonic() - start}
        for key, value in re.findall(r"^([a-z_]+)=([0-9.]+)$", text, re.M):
            row[key] = float(value)
        if code:
            for name in names:
                if name.startswith("layerfs-issue71-"):
                    owned = subprocess.run(["docker", "inspect", "--format", '{{index .Config.Labels "dev.layerfs.managed"}}', name], capture_output=True, text=True)
                    if owned.returncode == 0 and owned.stdout.strip() == "true":
                        subprocess.run(["docker", "rm", "-f", name], check=True)
            row["verification"] = "NOT_RUN_WORKLOAD_FAILED"
        else:
            assert "cleanup=removed_owned_container" in text
            assert len(names) == 1
            assert subprocess.run(["docker", "inspect", names[0]], capture_output=True).returncode != 0
            branch = re.search(r"^branch=(.+)$", text, re.M).group(1)
            verify = [str(binary), "verify", str(source), str(store / "store.sqlite")]
            save(prefix + "-verification-command.json", {"argv": verify, "VERIFY_BRANCH": branch, "timeout_seconds": 180})
            code = execute(verify, out / (prefix + "-verification.log"), 180, dict(os.environ, VERIFY_BRANCH=branch))
            row.update(branch=branch, verification_exit_code=code, verification="PASS" if code == 0 else "FAIL")
        results.append(row)
        save("results.json", {"status": "PASS" if code == 0 and sample == plan["samples"] else "IN_PROGRESS" if code == 0 else "FAIL", "samples": results})
        print(json.dumps(row), flush=True)
        if code:
            raise SystemExit(code)
