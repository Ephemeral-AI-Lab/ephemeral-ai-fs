"""Explicit optional retained-history profiles; reuse the existing public importer."""
import argparse
import json

PROFILES = {'stride-1': ('deepseek-full', 1), 'stride-3': ('deepseek-stride3', 3),
            'stride-10': ('deepseek-stride10', 10)}


def registry():
    return [{'family': 'repository_history', 'profile': name, 'scenario': scenario,
             'optional': True, 'default_status': 'NOT_RUN_OPTIONAL',
             'full157_indices': sorted(set(range(1, 158, stride)) | {157}),
             'state_count': len(set(range(1,158,stride)) | {157})}
            for name, (scenario, stride) in PROFILES.items()]


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--family', choices=['repository_history'], default='repository_history')
    parser.add_argument('--profile', choices=PROFILES)
    parser.add_argument('--list', action='store_true')
    parser.add_argument('--self-check', action='store_true')
    args, remaining = parser.parse_known_args(argv)
    if args.list or args.self_check:
        from deepseek_ten import PROFILES as selections
        rows = registry()
        assert [r['state_count'] for r in rows] == [157,53,17]
        assert tuple(rows[1]['full157_indices']) == selections['deepseek-stride3'][0]
        assert tuple(rows[2]['full157_indices']) == selections['deepseek-stride10'][0]
        print(json.dumps(rows if args.list else {'status':'PASS','profiles':3,'states':[157,53,17]}))
        return 0
    if not args.profile:
        parser.error('execution requires explicit --profile; no history runs by default')
    if '--storage-smoke' in remaining:
        parser.error('select history only through --profile')
    import storage_smoke
    return storage_smoke.main(['--storage-smoke', PROFILES[args.profile][0], *remaining])


if __name__ == '__main__':
    raise SystemExit(main())
