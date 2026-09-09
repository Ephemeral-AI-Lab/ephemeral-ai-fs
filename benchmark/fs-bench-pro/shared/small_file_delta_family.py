"""Single-case exploratory family using the established storage-smoke lifecycle."""
import argparse
import json
import storage_smoke


def main(argv):
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--family", choices=["small_file_delta_smoke"], required=True)
    parser.add_argument("--case", choices=["small-file-delta-10x30-v1"], default="small-file-delta-10x30-v1")
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--self-check", action="store_true")
    args, remaining = parser.parse_known_args(argv)
    if args.list:
        print(json.dumps({"family": args.family, "case": args.case, "case_count": 1,
                          "files": 10, "commits": 30, "initial_bytes": 245760}))
        return 0
    if args.self_check:
        from small_file_delta_fixture import self_check
        self_check()
        return 0
    return storage_smoke.main(["--storage-smoke", args.case, *remaining])
