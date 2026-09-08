#!/usr/bin/env python3
"""Read-only native Git pack audit. Aggregates metadata; no re-encoding or raw dumps."""
import collections,hashlib,json,pathlib,re,subprocess,sys

def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def main(root,out):
    expected=set((root/'snapshot-object-ids.txt').read_text().splitlines())
    mappings=json.loads((root/'mapping.json').read_text())
    expected.update(r['git_commit'] for r in mappings)
    assert len(expected)==110081 and len(mappings)==157
    packdir=root/'snapshots.git/objects/pack';packs=list(packdir.glob('*.pack'));assert len(packs)==1
    pack=packs[0];idx=pack.with_suffix('.idx');before={p.name:sha(p) for p in [pack,idx,root/'mapping.json',root/'results.json',root/'snapshot-object-ids.txt']}
    assert pack.stat().st_size==51989900
    groups={};depths=collections.Counter();seen=set();success=False
    p=subprocess.Popen(['git','verify-pack','-v',str(idx)],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
    for line in p.stdout:
        f=line.split()
        if f and re.fullmatch('[0-9a-f]{40}',f[0]):
            assert len(f) in (5,7) and f[0] not in seen
            seen.add(f[0]);typ=f[1];depth=int(f[5]) if len(f)==7 else 0
            g=groups.setdefault(typ+('_delta' if depth else '_FULL'),{'objects_count':0,'packed_entry_bytes':0,'git_reported_size_bytes':0,'max_depth_count':0})
            g['objects_count']+=1;g['packed_entry_bytes']+=int(f[3]);g['git_reported_size_bytes']+=int(f[2]);g['max_depth_count']=max(g['max_depth_count'],depth);depths[depth]+=1
        elif line.rstrip().endswith(': ok'):success=True
    stderr=p.stderr.read();assert p.wait()==0,(p.returncode,stderr)
    assert success and seen==expected
    total=sum(g['packed_entry_bytes'] for g in groups.values())
    assert total+12+20==pack.stat().st_size
    after={p.name:sha(p) for p in [pack,idx,root/'mapping.json',root/'results.json',root/'snapshot-object-ids.txt']};assert before==after
    result={'status':'nativeverify-pack PASS and expectedmembership equality; derived aggregate','units':'packed_entry_bytes physical wirebytes incl entryheader/base reference; git_reported_size_bytes nativeoutputfield NOT assumed canonicaltargetbytes fordelta; countintegers','population':'existinghistorical157 matchedGitpack; all110081objects','snapshot':'retainedfinaldelta pack, currentread-only verification; no new packing or timing pair','provenance':{'path':str(pack),'file_sha256':before,'command':['git','verify-pack','-v',str(idx)],'git_version':subprocess.check_output(['git','--version'],text=True).strip()},'pack_file_bytes':pack.stat().st_size,'pack_header_bytes':12,'pack_trailer_bytes':20,'packed_entries_bytes':total,'by_type_and_representation':groups,'depth_histogram_count':dict(sorted(depths.items())),'expected_object_count':len(expected),'unchanged_hashes':True,'limitations':['Historicalconstruction/packing costs remain separate; currentverification duration not productperformance','Gitmetadata differs fromLayerFS; entrygroups not equalLFS canonicalroles','Depth/bases may dependonlaterselectedversions because finalbatchpack is not chronologicalpublicpath; candidate must not usefuturecontent at ack']}
    with (out/'git-pack-account.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
    print(json.dumps({'groups':groups,'max_depth':max(depths),'entries_bytes':total},indent=2))

if __name__=='__main__':main(*map(pathlib.Path,sys.argv[1:]))
