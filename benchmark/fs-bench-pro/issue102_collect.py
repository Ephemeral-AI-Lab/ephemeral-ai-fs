#!/usr/bin/env python3
"""Alternate sealed arms using the existing collector; append access qualifications."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import time

import issue54_collect as collect
from issue54_collect import runner
from shared.repository_history import registry as histories

ROOT = collect.REPO
DECLARATION = ROOT / 'docs/roadmap/0.1/0.1.5/issue102/mandatory-campaign.json'
PREPARATION = ROOT / 'docs/roadmap/0.1/0.1.4/issue91-campaign/verification-preparation-r4.json'


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for arm in ('control','candidate'):
        p.add_argument('--'+arm+'-image',required=True)
        p.add_argument('--'+arm+'-binary',required=True)
    p.add_argument('--access-store',required=True)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--inventory-only',action='store_true')
    args=p.parse_args()
    args.output.mkdir(parents=True,exist_ok=False)
    declaration=json.loads(DECLARATION.read_text())
    arms={}; inventory={}
    for arm in ('control','candidate'):
        config=argparse.Namespace(image=getattr(args,arm+'_image'),host_binary=getattr(args,arm+'_binary'),
            source_arm='baseline' if arm=='control' else 'candidate',campaign_spec=declaration,
            proof_preparation_spec=json.loads(PREPARATION.read_text()),proof_preparation_declaration=PREPARATION,
            listed_families={},proofs='all')
        selected={}
        for family in runner.HOST_FAMILIES:
            rows,listed=collect.list_family(config,family)
            config.listed_families[family]=listed;selected[family]=(rows,listed)
        collect.validate_campaign(declaration,selected)
        collect.validate_proof_preparation(config.proof_preparation_spec,selected)
        arms[arm]=config
        inventory[arm]={family:listed for family,(_,listed) in selected.items()}
    collect._write(args.output/'inventory.json',inventory)
    collect._write(args.output/'optional-history.json',histories())
    if args.inventory_only:return 0
    ledger=args.output/'ledger.jsonl'
    def retain(arm,phase,case,result):
        with ledger.open('a') as f:
            f.write(json.dumps({'arm':arm,'phase':phase,**result,'case':case},sort_keys=True)+'\n')
        print(f"{arm} {phase} {case} {result.get('status')}",flush=True)
    ordered=[r for f in runner.HOST_FAMILIES for r in inventory['candidate'][f]['rows']]
    for index,row in enumerate(ordered):
        family=row['family_id'];case=row['scenario_id']
        for arm in (('control','candidate') if index%2==0 else ('candidate','control')):
            config=arms[arm];output=args.output/arm
            try:
                if case==declaration.get('long_test_exclusion'):
                    retain(arm,'verification',case,collect.verify_row(config,family,row,output,None));continue
                if not row['proof_only']:
                    result,_=collect.collect_row(config,family,row,output)
                    retain(arm,'performance',case,result)
                    identities=result.get('identities')
                    if not identities:
                        retain(arm,'verification',case,{'status':'INCOMPLETE','error':'no performance identities'});continue
                else:
                    resolve=runner.build_parser().parse_args(['--family',family,'--case',case,'--seed','1',
                        '--verification','--setup','clone','--image',config.image,'--host-binary',config.host_binary,
                        '--product-timeout','300','--timeout','310','--setup-timeout','600','--source-arm',config.source_arm])
                    identities=runner.resolve_selection(resolve,time.monotonic()+30)
                result=collect.verify_row(config,family,row,output,identities)
                retain(arm,'verification',case,result)
            except Exception as error:
                retain(arm,'error',case,{'status':'INCOMPLETE','error':type(error).__name__+': '+str(error)})
    config=arms['candidate']
    fixture=json.loads((runner.BENCH/'families/historical_access/fixture.json').read_text())
    for case in fixture['cases']:
        previous=None
        for mode in ('performance','verification'):
            output=args.output/'historical_access'/(case['id']+'-'+mode)
            command=[sys.executable,str(runner.HERE/'runner.py'),'--family','historical_access','--case',case['id'],
                     '--mode',mode,'--store',args.access_store,'--image',config.image,'--host-binary',config.host_binary,'--output',str(output)]
            if previous:command+=['--performance',str(previous/'result.json')]
            started=time.monotonic_ns();proc=subprocess.run(command,capture_output=True,text=True)
            result=json.loads((output/'result.json').read_text()) if (output/'result.json').exists() else {'status':'INCOMPLETE'}
            result.update(external_wall_ns=time.monotonic_ns()-started,returncode=proc.returncode)
            if proc.returncode or result['external_wall_ns']>=15_000_000_000:result['status']='FAIL'
            retain('candidate',mode,case['id'],result);previous=output
    values=[json.loads(l) for l in ledger.read_text().splitlines()]
    summary={}
    for row in values:
        key=row['arm']+'/'+row['phase']+'/'+str(row.get('status'))
        summary[key]=summary.get(key,0)+1
    collect._write(args.output/'summary.json',summary)
    print(json.dumps(summary,sort_keys=True),flush=True)
    return int(any(r.get('status') not in ('PASS','NOT_RUN_OPTIONAL') for r in values))


if __name__=='__main__':raise SystemExit(main())
