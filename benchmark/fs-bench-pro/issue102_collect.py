#!/usr/bin/env python3
"""Alternate sealed arms using the existing collector; append access qualifications."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import time
import datetime
import os
import threading
import uuid

import issue54_collect as collect
from issue54_collect import runner
from shared.repository_history import registry as histories

ROOT = collect.REPO
DECLARATION = ROOT / 'docs/roadmap/0.1/0.1.5/issue104/mandatory-campaign.json'
PREPARATION = ROOT / 'docs/roadmap/0.1/0.1.4/issue91-campaign/verification-preparation-r4.json'


def select_rows(inventory, family):
    if family not in (*runner.HOST_FAMILIES, 'historical_access'):
        raise ValueError('one registered family is required')
    return [(i,r) for i,r in enumerate(r for f in runner.HOST_FAMILIES for r in inventory[f]['rows'])
            if r['family_id'] == family]


def completed(values, family, arm, phase, case):
    return any(v.get('family') == family and v.get('arm') == arm and
               v.get('phase') == phase and v.get('case') == case for v in values)


def streamed_run(argv, cwd=None):
    if '--list' in argv:
        return subprocess.run(argv,cwd=cwd or ROOT,text=True,capture_output=True)
    started=time.monotonic()
    folder=LOG_ROOT/uuid.uuid4().hex
    folder.mkdir(parents=True,exist_ok=False)
    collect._write(folder/'command.json',argv)
    print('COMMAND_START',datetime.datetime.now(datetime.timezone.utc).isoformat(),json.dumps(argv),'evidence='+str(folder),flush=True)
    proc=subprocess.Popen(argv,cwd=cwd or ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,
                          env={**os.environ,'PYTHONUNBUFFERED':'1'})
    parts=[[],[]]
    def drain(pipe,index):
        with (folder/('stdout.log' if index==0 else 'stderr.log')).open('x') as log:
            for line in pipe:
                parts[index].append(line);log.write(line);log.flush()
                print(line,end='',flush=True)
    threads=[threading.Thread(target=drain,args=(proc.stdout,0)),threading.Thread(target=drain,args=(proc.stderr,1))]
    for thread in threads:thread.start()
    while True:
        try:proc.wait(timeout=15);break
        except subprocess.TimeoutExpired:print('COMMAND_PROGRESS',round(time.monotonic()-started,3),'seconds',str(folder),flush=True)
    for thread in threads:thread.join()
    receipt={'returncode':proc.returncode,'wall_seconds':time.monotonic()-started,'finished_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
    collect._write(folder/'exit.json',receipt)
    print('COMMAND_END',json.dumps(receipt),str(folder),flush=True)
    return subprocess.CompletedProcess(argv,proc.returncode,''.join(parts[0]),''.join(parts[1]))


def main():
    global LOG_ROOT
    p=argparse.ArgumentParser(description=__doc__)
    for arm in ('control','candidate'):
        p.add_argument('--'+arm+'-image',required=arm=='candidate')
        p.add_argument('--'+arm+'-binary',required=arm=='candidate')
    p.add_argument('--control-inapplicable',type=Path)
    p.add_argument('--family',required=True,choices=(*runner.HOST_FAMILIES,'historical_access'))
    p.add_argument('--campaign',type=Path,default=DECLARATION)
    p.add_argument('--access-store')
    p.add_argument('--access-fixture',type=Path)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--inventory-only',action='store_true')
    p.add_argument('--resume',action='store_true')
    p.add_argument('--retry-case')
    p.add_argument('--verification-only',action='store_true')
    p.add_argument('--performance-only',action='store_true')
    p.add_argument('--applicability',type=Path)
    args=p.parse_args()
    if args.verification_only and args.performance_only:p.error('choose one phase')
    if (args.verification_only or args.performance_only) and (not args.applicability or not args.applicability.is_file()):
        p.error('verification-only resume requires retained applicability evidence')
    if not args.control_inapplicable and not (args.control_image and args.control_binary):
        p.error('qualified control or retained inapplicability evidence required')
    if args.family=='historical_access' and not (args.access_fixture and args.access_store):
        p.error('historical_access requires separately bound fixture and Store')
    if args.output.exists() and not args.resume:p.error('existing evidence requires --resume')
    args.output.mkdir(parents=True,exist_ok=True)
    LOG_ROOT=args.output/'commands';LOG_ROOT.mkdir(exist_ok=True)
    collect._run=streamed_run
    declaration=json.loads(args.campaign.read_text())
    arms={};inventory={}
    for arm in ('control','candidate'):
        if arm=='control' and args.control_inapplicable:continue
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
        arms[arm]=config;inventory[arm]={family:listed for family,(_,listed) in selected.items()}
    collect._write(args.output/('inventory-'+uuid.uuid4().hex+'.json'),inventory)
    if args.inventory_only:return 0
    ledger=args.output/'ledger.jsonl'
    values=[json.loads(l) for l in ledger.read_text().splitlines()] if ledger.exists() else []
    started=time.monotonic()
    selected=select_rows(inventory['candidate'],args.family)
    fixture=json.loads(args.access_fixture.read_text()) if args.family=='historical_access' else None
    ids=[r['scenario_id'] for _,r in selected] if fixture is None else [r['id'] for r in fixture['cases']]
    if args.retry_case and args.retry_case not in ids:p.error('retry case is outside selected family')
    print('FAMILY_START',datetime.datetime.now(datetime.timezone.utc).isoformat(),args.family,
          'cases='+json.dumps(ids),'arms='+json.dumps(list(arms)),'evidence='+str(args.output),flush=True)
    if args.control_inapplicable:
        print('CONTROL_INAPPLICABLE',args.control_inapplicable.read_text(),flush=True)
    def retain(arm,phase,case,result):
        value={'arm':arm,'phase':phase,**result,'family':args.family,'case':case,
               'recorded_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'treatment':'promoted-uncompacted'}
        with ledger.open('a') as f:f.write(json.dumps(value,sort_keys=True)+'\n');f.flush()
        values.append(value)
        print('CASE_END',args.family,arm,phase,case,result.get('status'),'timer='+str(result.get('timer')),
              'elapsed_ns='+str(result.get('elapsed_ns')),'evidence='+str(result.get('receipt')),flush=True)
    for index,row in selected:
        family=args.family;case=row['scenario_id']
        if args.retry_case and case!=args.retry_case:continue
        for arm in (('control','candidate') if index%2==0 else ('candidate','control')):
            if arm not in arms:continue
            config=arms[arm]
            prior=[v for v in values if v.get('family')==family and v.get('case')==case and v.get('arm')==arm]
            if not args.retry_case and not args.verification_only and not args.performance_only and any(v['phase']=='verification' for v in prior):
                print('RETAINED',family,arm,case,'original receipts in ledger',flush=True);continue
            if args.performance_only and not args.retry_case:
                latest_perf=next((v for v in reversed(prior) if v['phase']=='performance'),None)
                if latest_perf and latest_perf.get('status')=='PASS' and latest_perf.get('retained_verification'):
                    print('RETAINED_PERFORMANCE',latest_perf.get('receipt'),flush=True);continue
            if args.verification_only and not args.retry_case:
                latest_proof=next((v for v in reversed(prior) if v['phase']=='verification'),None)
                if latest_proof and latest_proof.get('status')=='PASS':
                    print('RETAINED_VERIFICATION',latest_proof.get('receipt'),flush=True);continue
            attempt=sum(v['phase']=='verification' for v in prior)+1
            output=args.output/arm/('attempt-'+str(attempt))
            try:
                if case==declaration.get('long_test_exclusion'):
                    retain(arm,'verification',case,collect.verify_row(config,family,row,output,None));continue
                original_performance=None
                if args.verification_only and not row['proof_only']:
                    original_performance=next((v for v in reversed(prior) if v['phase']=='performance' and v.get('status')=='PASS'),None)
                    if original_performance is None:raise ValueError('no passing performance to retain')
                    print('RETAINED_PERFORMANCE',original_performance['receipt'],flush=True)
                if not row['proof_only'] and not args.verification_only:
                    result,_=collect.collect_row(config,family,row,output)
                    if args.performance_only:
                        original_proof=next((v for v in reversed(prior) if v['phase']=='verification' and v.get('status')=='PASS'),None)
                        if original_proof is None:raise ValueError('performance-only recollection requires a retained passing proof')
                        result.update(retained_verification=original_proof['receipt'],retained_verification_sha256=original_proof['receipt_sha256'],
                                      applicability=str(args.applicability),applicability_sha256=collect._sha256(args.applicability))
                    if args.retry_case or args.performance_only or not completed(values,family,arm,'performance',case):retain(arm,'performance',case,result)
                    if args.performance_only:continue
                    identities=result.get('identities')
                    if not identities:
                        retain(arm,'verification',case,{'status':'INCOMPLETE','error':'no performance identities'});continue
                else:
                    resolve=runner.build_parser().parse_args(['--family',family,'--case',case,'--repetition' if row.get('route')=='sdk' else '--seed','1',
                        '--setup','fresh' if row['setup_policy']=='fresh-output' else 'clone','--image',config.image,'--host-binary',config.host_binary,
                        '--product-timeout','300','--timeout','310','--setup-timeout','600','--source-arm',config.source_arm])
                    resolve.verification=True
                    identities=runner.resolve_selection(resolve,time.monotonic()+30)
                result=collect.verify_row(config,family,row,output,identities)
                if original_performance:
                    result.update(retained_performance=original_performance['receipt'],
                                  retained_performance_sha256=original_performance['receipt_sha256'],
                                  applicability=str(args.applicability),applicability_sha256=collect._sha256(args.applicability))
                retain(arm,'verification',case,result)
            except Exception as error:
                retain(arm,'verification',case,{'status':'INCOMPLETE','error':type(error).__name__+': '+str(error)})
    if fixture:
        config=arms['candidate']
        for case in fixture['cases']:
            cid=case['id']
            if args.retry_case and cid!=args.retry_case:continue
            if not args.retry_case and completed(values,args.family,'candidate','verification',cid):continue
            previous=None
            attempt=sum(v.get('case')==cid and v.get('phase')=='verification' for v in values)+1
            for mode in ('performance','verification'):
                output=args.output/'historical_access'/('attempt-'+str(attempt))/(cid+'-'+mode)
                command=[sys.executable,str(runner.HERE/'runner.py'),'--family','historical_access','--case',cid,
                    '--fixture',str(args.access_fixture),'--mode',mode,'--store',args.access_store,
                    '--image',config.image,'--host-binary',config.host_binary,'--output',str(output)]
                if previous:command+=['--performance',str(previous/'result.json')]
                proc=streamed_run(command)
                result=json.loads((output/'result.json').read_text()) if (output/'result.json').exists() else {'status':'INCOMPLETE'}
                result.update(returncode=proc.returncode,receipt=str(output/'result.json'))
                if proc.returncode:result['status']='FAIL'
                retain('candidate',mode,cid,result);previous=output
    latest={ (v['arm'],v['phase'],v['case']):v for v in values if v.get('family')==args.family }
    cold_samples = {}
    # Resume cannot trust an older ledger's PASS or cached improvement summary.
    for key, value in latest.items():
        if key[1:] != ('performance', 'namespace-100000') or args.family != 'init_namespace':
            continue
        raw = [json.loads(line) for line in Path(value['receipt']).read_text().splitlines()]
        samples = [item for item in raw if item.get('kind') == 'sample']
        if len(samples) != 1:
            raise ValueError('cold qualification requires one original sample receipt')
        sample = samples[0]
        assessment = runner.cold.assess(sample)
        latest[key] = {**value, 'status': assessment['status'], 'cold_qualification': assessment,
                       'elapsed_ns': assessment['eligible_elapsed_ns']}
        cold_samples[key[0]] = sample
    cold_pair = None
    if 'control' in cold_samples and 'candidate' in cold_samples:
        index = next(i for i, row in selected if row['scenario_id'] == 'namespace-100000')
        declared_order = ['baseline', 'candidate'] if index % 2 == 0 else ['candidate', 'baseline']
        cold_pair = runner.cold.compare(cold_samples['control'], cold_samples['candidate'], order=declared_order)
    expected_perf=11 if fixture else sum(not r['proof_only'] for _,r in selected)
    expected_proof=11 if fixture else sum(r['scenario_id']!=declaration.get('long_test_exclusion') for _,r in selected)
    failed=[v for v in latest.values() if v.get('status') not in ('PASS','NOT_RUN_OPTIONAL')]
    slow=[v for v in latest.values() if v.get('historical_product_target_status')=='TARGET_MISS']
    counts={phase:sum(v['phase']==phase and v.get('status')!='NOT_RUN_OPTIONAL' for v in latest.values()) for phase in ('performance','verification')}
    outcome='FAIL' if failed or slow else 'PASS'
    if cold_pair and cold_pair['status'] != 'ELIGIBLE_COLD_PAIR':outcome='FAIL'
    if counts!={'performance':expected_perf*len(arms),'verification':expected_proof*len(arms)}:outcome='INCOMPLETE'
    summary={'family':args.family,'performance_completed':counts['performance'],'performance_expected':expected_perf*len(arms),
        'cold_pair':cold_pair,
        'verification_completed':counts['verification'],'verification_expected':expected_proof*len(arms),'failures':len(failed),
        'slow_cases':len(slow),'wall_seconds':time.monotonic()-started,'outcome':outcome,'evidence':str(args.output)}
    collect._write(args.output/(args.family+'-summary-'+uuid.uuid4().hex+'.json'),summary)
    print('Family | Performance completed/expected | Verification completed/expected | Failures | Slow cases | Wall time | Outcome | Evidence path',flush=True)
    print(f"{args.family} | {counts['performance']}/{summary['performance_expected']} | {counts['verification']}/{summary['verification_expected']} | {len(failed)} | {len(slow)} | {summary['wall_seconds']:.3f}s | {outcome} | {args.output}",flush=True)
    print('FAMILY_END',datetime.datetime.now(datetime.timezone.utc).isoformat(),json.dumps(summary),flush=True)
    return int(outcome!='PASS')


if __name__=='__main__':raise SystemExit(main())
