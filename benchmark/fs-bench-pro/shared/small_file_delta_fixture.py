"""Three files, 30 one-line changes; inputs for the existing FUSE importer."""
import hashlib
import json
from pathlib import Path

CASE = "small-file-delta-10x30-v1"
SIZES = tuple(n * 1024 for n in (4, 8, 12, 16, 20, 24, 28, 32, 40, 56))


def line(index, number):
    value = hashlib.sha256(f"{CASE}/{index}/{number}".encode()).hexdigest()[:36]
    return (f'export const v{number:04d} = "{value}";'.ljust(63) + "\n").encode()


def states():
    files = {f"source{i:02}.ts": b"".join(line(i, j) for j in range(size // 64))
             for i, size in enumerate(SIZES)}
    yield dict(files)
    for step in range(1, 31):
        i = (step - 1) % 10
        name = f"source{i:02}.ts"
        n = SIZES[i] // 64
        data = bytearray(files[name])
        if step <= 10:
            offset = ((17 * (i + 1)) % n) * 64 + 22
            digits = b"0123456789abcdef"
            for pos in range(offset, offset + 8):
                data[pos] = digits[(digits.index(data[pos]) + 1) % 16]
        elif step <= 20:
            offset = 64 * (n // 2)
            data[offset:offset] = line(i, 9000 + i)
        else:
            offset = 64 * (n // 4)
            del data[offset:offset + 64]
        files[name] = bytes(data)
        yield dict(files)


def manifest(files):
    return {
        name.encode().hex(): ("100644", hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest(), len(data))
        for name, data in files.items()
    }


def oracle(files):
    return {name.encode().hex(): ["100644", len(data), hashlib.sha256(data).hexdigest()]
            for name, data in files.items()}


def build(parent):
    root = Path(parent) / CASE
    root.mkdir(parents=True, exist_ok=True)
    initial = root / "initial"
    initial.mkdir(exist_ok=True)
    expected_files = {}

    def write(path, data):
        expected_files[str(path.relative_to(root))] = hashlib.sha256(data).hexdigest()
        if path.exists():
            if path.read_bytes() != data:
                raise ValueError(f"fixture differs: {path}")
        else:
            path.write_bytes(data)

    def encoded(tree):
        return "".join(f"{m}\t{o}\t{s}\t{p}\n" for p, (m, o, s) in sorted(tree.items())).encode()

    previous = None
    result = []
    for step, files in enumerate(states()):
        tree = manifest(files)
        proof = root / f"oracle-{step}.json"
        write(proof, (json.dumps(oracle(files), sort_keys=True) + "\n").encode())
        if step == 0:
            import os
            for name, data in files.items():
                write(initial / name, data)
                (initial / name).chmod(0o644)
                os.utime(initial / name, (1_000_000_000, 1_000_000_000))
            initial.chmod(0o755)
            os.utime(initial, (1_000_000_000, 1_000_000_000))
        else:
            folder = root / f"step-{step}"
            (folder / "blobs").mkdir(parents=True, exist_ok=True)
            write(folder / "previous.tsv", encoded(previous))
            write(folder / "manifest.tsv", encoded(tree))
            for name, data in files.items():
                key = name.encode().hex()
                if tree[key] != previous[key]:
                    write(folder / "blobs" / tree[key][1], data)
            result.append({"index": step, "input": str(folder), "oracle": str(proof)})
        previous = tree
    return {CASE: {"input": str(root), "initial_oracle": str(root / "oracle-0.json"),
                                "states": result, "file_sha256": expected_files}}


def self_check():
    import tempfile
    history = list(states())
    assert history == list(states()) and len(history) == 31
    assert tuple(map(len, history[0].values())) == SIZES
    assert tuple(map(len, history[-1].values())) == SIZES
    assert sum(SIZES) == 245760
    assert max(len(v) for h in history for v in h.values()) == 57408
    for step, (old, new) in enumerate(zip(history, history[1:]), 1):
        i = (step - 1) % 10
        name = f"source{i:02}.ts"
        assert [p for p in old if old[p] != new[p]] == [name]
        n = SIZES[i] // 64
        before, after = old[name], new[name]
        if step <= 10:
            offset = 64 * ((17 * (i + 1)) % n) + 22
            assert [p for p in range(len(before)) if before[p] != after[p]] == list(range(offset, offset + 8))
            assert all(int(chr(b), 16) == (int(chr(a), 16) + 1) % 16
                       for a, b in zip(before[offset:offset+8], after[offset:offset+8]))
        elif step <= 20:
            offset = 64 * (n // 2)
            assert after == before[:offset] + line(i, 9000 + i) + before[offset:]
        else:
            offset = 64 * (n // 4)
            assert before[offset:offset+64] == line(i, n // 4)
            assert after == before[:offset] + before[offset+64:]
        assert all(len(v) % 64 == 0 and all(len(l) == 64 for l in v.splitlines(keepends=True)) for v in new.values())
    with tempfile.TemporaryDirectory() as folder:
        fixture = build(folder)
        assert build(folder) == fixture
        for step, files in enumerate(history):
            assert json.loads((Path(folder) / CASE / f"oracle-{step}.json").read_text()) == oracle(files)
    print("PASS: ten files, thirty fixed edits, 31 complete fixture oracles")


if __name__ == "__main__":
    self_check()
