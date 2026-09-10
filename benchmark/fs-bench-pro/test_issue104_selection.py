import json
import subprocess
import sys
from pathlib import Path
import issue102_collect as campaign


def test_selection_resume():
    declaration=json.loads(campaign.DECLARATION.read_text())
    rows=[json.loads(line) for line in (campaign.ROOT/'docs/roadmap/0.1/0.1.5/issue102/mandatory-registry.jsonl').read_text().splitlines()]
    inventory={family:{'rows':[r for r in rows if r['family_id']==family]} for family in campaign.runner.HOST_FAMILIES}
    campaign.collect.validate_campaign(declaration,{f:(v['rows'],v) for f,v in inventory.items()})
    for family in campaign.runner.HOST_FAMILIES:
        selected=campaign.select_rows(inventory,family)
        assert selected and all(r['family_id']==family for _,r in selected)
        assert len(selected)==sum(declaration['families'][family].values())
    assert campaign.select_rows(inventory,'historical_access')==[]
    try:campaign.select_rows(inventory,None)
    except ValueError:pass
    else:raise AssertionError('unselected full campaign admitted')
    values=[dict(family='a',arm='candidate',phase='performance',case='x',status='PASS')]
    assert campaign.completed(values,'a','candidate','performance','x')
    assert not campaign.completed(values,'a','candidate','verification','x')
    assert not campaign.completed(values,'b','candidate','performance','x')
    proc=subprocess.run([sys.executable,str(Path(campaign.__file__))],capture_output=True,text=True)
    assert proc.returncode==2 and '--family' in proc.stderr


def test_verification_only_preserves_passing_work():
    import contextlib,io,tempfile
    from unittest.mock import patch
    rows=[json.loads(line) for line in (campaign.ROOT/'docs/roadmap/0.1/0.1.5/issue102/mandatory-registry.jsonl').read_text().splitlines()]
    family='payload_create_read';cases=[r['scenario_id'] for r in rows if r['family_id']==family]
    def listing(config,f):
        members=[r for r in rows if r['family_id']==f]
        return members,{'rows':members,'source_identity':'original','image':'image'}
    with tempfile.TemporaryDirectory() as directory:
        root=Path(directory);evidence=root/'run';evidence.mkdir()
        (root/'applicability').write_text('verifier-only change, original product and performance unchanged')
        (root/'control').write_text('INAPPLICABLE')
        ledger=[dict(family=family,arm='candidate',phase=phase,case=case,status='PASS' if phase=='performance' or case==cases[0] else 'FAIL',receipt='original-'+case,receipt_sha256='original-hash')
                for case in cases for phase in ('performance','verification')]
        (evidence/'ledger.jsonl').write_text(''.join(json.dumps(v)+'\n' for v in ledger))
        argv=['collector','--family',family,'--resume','--verification-only','--applicability',str(root/'applicability'),
              '--control-inapplicable',str(root/'control'),'--candidate-binary','unused','--candidate-image','unused','--output',str(evidence)]
        with patch.object(sys,'argv',argv),patch.object(campaign.collect,'list_family',side_effect=listing), \
             patch.object(campaign.runner,'resolve_selection',return_value={}), \
             patch.object(campaign.collect,'collect_row',side_effect=AssertionError('passing performance rerun')), \
             patch.object(campaign.collect,'verify_row',return_value={'status':'PASS'}) as verify, \
             contextlib.redirect_stdout(io.StringIO()):
            assert campaign.main()==0
        assert verify.call_count==7
        after=[json.loads(l) for l in (evidence/'ledger.jsonl').read_text().splitlines()]
        assert after[:len(ledger)]==ledger
        assert len(after)==len(ledger)+7
        assert all(v['phase']=='verification' and v['case']!=cases[0] and v['retained_performance'].startswith('original-') for v in after[len(ledger):])


if __name__=='__main__':
    test_selection_resume()
    test_verification_only_preserves_passing_work()
    print('PASS complete registry, single-family isolation, explicit CLI selection, phase/family resume isolation')
