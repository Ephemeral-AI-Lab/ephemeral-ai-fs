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


if __name__=='__main__':
    test_selection_resume()
    print('PASS complete registry, single-family isolation, explicit CLI selection, phase/family resume isolation')
