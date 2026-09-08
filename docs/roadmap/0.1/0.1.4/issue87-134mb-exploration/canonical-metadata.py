#!/usr/bin/env python3
"""Existing authenticated metadata only; no Store/payload decoding or encoding."""
import collections,csv,hashlib,json,pathlib,sqlite3,sys

def main(source,output):
    manifest=json.loads((source/'manifest.sha256.json').read_text())
    authenticated={}
    for name in ('groups.csv','group-role-composition.csv','object-roles.csv','roles-inventory.sqlite'):
        with (source/name).open('rb') as f: digest=hashlib.file_digest(f,'sha256').hexdigest()
        assert digest==manifest['files'][name]['sha256'],name
        authenticated[name]=digest
    role_sets=collections.defaultdict(set)
    components=collections.Counter()
    for r in csv.DictReader((source/'group-role-composition.csv').open()):
        role_sets[(r['pack_id'],r['group_number'])].add(r['role'])
        components[r['role']]+=int(r['canonical_bytes'])
    physical=collections.defaultdict(collections.Counter)
    envelopes=collections.defaultdict(collections.Counter)
    for r in csv.DictReader((source/'groups.csv').open()):
        roles=role_sets[(r['pack_id'],r['group_number'])]
        lane='payload_only' if roles=={'payload_chunk'} else 'payload_and_structural' if 'payload_chunk' in roles else 'structural_only'
        for key,column in [('groups_count',None),('encoded_bytes','encoded_bytes'),('decoded_bytes','decoded_bytes'),('records_count','record_count')]:
            value=1 if column is None else int(r[column]);physical[lane][key]+=value
        for family,selected in [('inode_tables',{'inode_table_leaf','inode_table_branch'}),('file_state_extent',{'FileState','extent_leaf','extent_branch'}),('inode_records',{'inode_record'})]:
            if roles & selected:
                label='exclusive' if roles<=selected else 'shared_with_other_roles'
                envelopes[family][label+'_groups_count']+=1
                envelopes[family][label+'_whole_encoded_bytes']+=int(r['encoded_bytes'])
    sizes=collections.defaultdict(collections.Counter)
    count=collections.Counter()
    for r in csv.DictReader((source/'object-roles.csv').open()):
        count[r['role']]+=1
        if r['role']=='extent_leaf':
            n=int(r['canonical_bytes']);assert (n-44)%40==0
            sizes['extent_entries'][(n-44)//40]+=1
    assert sum(v['encoded_bytes'] for v in physical.values())==295240878
    assert sum(components.values())==799525289
    p=source/'roles-inventory.sqlite'
    with sqlite3.connect(p.as_uri()+'?mode=ro&immutable=1',uri=True) as c:
        c.execute('pragma cache_size=-8192')
        refs=[dict(zip(['parent_role','reference_occurrences_count','older_reference_occurrences_count','same_checkpoint_reference_occurrences_count'],r)) for r in c.execute('SELECT p.role,count(*),sum(ch.first_retained_checkpoint<p.first_retained_checkpoint),sum(ch.first_retained_checkpoint=p.first_retained_checkpoint) FROM classified p JOIN edges e ON e.parent=p.id JOIN classified ch ON ch.id=e.child WHERE p.role IN ("inode_table_leaf","extent_leaf","inode_record","FileState","directory_map_branch") GROUP BY p.role')]
        fs_mapping=c.execute('SELECT count(*),count(DISTINCT e.child) FROM records r JOIN edges e ON e.parent=r.id WHERE r.role="FileState"').fetchone()
        inline=[]
        for threshold in (256,1024,4096):
            # A one-extent file cannot exceed its payload backing length. This
            # conservatively selects files even when source_offset is unavailable.
            row=c.execute('SELECT count(*),sum(fs.bytes+leaf.bytes),sum(payload.bytes),count(DISTINCT payload.id) FROM records fs CROSS JOIN edges f INDEXED BY edges_parent ON f.parent=fs.id CROSS JOIN records leaf INDEXED BY records_id ON leaf.id=f.child CROSS JOIN edges e INDEXED BY edges_parent ON e.parent=leaf.id CROSS JOIN records payload INDEXED BY records_id ON payload.id=e.child WHERE fs.role="FileState" AND leaf.role="extent_leaf" AND leaf.bytes=84 AND payload.role="payload_chunk" AND payload.bytes<=?',(threshold+21,)).fetchone()
            inline.append(dict(max_backing_payload_bytes=threshold,file_states_count=row[0],state_plus_leaf_canonical_bytes=row[1],backing_canonical_bytes_with_repeated_uses=row[2],distinct_backing_objects_count=row[3],status='derived candidate cohort; no claim these backing objects become deletable'))
    n=count['FileState'];extents=sum(int(k)*v for k,v in sizes['extent_entries'].items())
    assert fs_mapping==(n,n)
    old=components['FileState']+components['extent_leaf'];new_model=26*n+40*extents
    result=dict(source_directory=str(source),snapshot='post-verification authenticated retained metadata; first-retained chronology from prior analysis',status='derived measurements and explicitly hypothetical representation arithmetic',units='integer bytes/counts; no compressed per-object attribution',group_partition=dict(physical),role_canonical_bytes=dict(components),role_counts=dict(count),whole_group_envelopes=dict(envelopes),extent_entry_histogram=dict(sizes['extent_entries']),reference_chronology=refs,one_extent_inline_cohorts=inline,file_state_mapping={'file_states_count':fs_mapping[0],'distinct_mapping_roots_count':fs_mapping[1]},fused_leaf_model={'old_canonical_bytes':old,'proposed_header_bytes':26,'extent_entry_bytes':40,'derived_proposed_canonical_bytes':new_model,'derived_raw_difference_bytes':old-new_model,'eliminated_selected_objects_count':n,'status':'hypothetical new canonical format, unencoded; not compressed or allocated savings'},index_models={'current_objects_index_page_bytes':18837504,'full_id_bytes_once':366141*32,'hypothetical_dense_40byte_rows':366141*40,'dense_row_difference_before_framing':18837504-366141*40,'status':'representation arithmetic; mutable tail/fanout/checksums/publication costs excluded'},target={'baseline_acknowledgement_bytes':335552512,'allocation_goal_bytes':134221004,'required_reduction_bytes':335552512-134221004,'payload_only_encoded_floor_bytes':physical['payload_only']['encoded_bytes'],'required_payload_reduction_with_zero_everything_else_bytes':physical['payload_only']['encoded_bytes']-134221004},limitations=['Structural group envelopes overlap across candidate families; do not add them.','Raw canonical model savings are not compressed savings.','Final page statistics are post-verification and cannot be spliced into an exact pre-verification allocation equation.','Shared payload objects and FULL-base closure may survive inline files.','No original Store, payload bytes, alternative codec, replay or product optimization used.'])
    result['source_hashes']=authenticated
    with output.open('x') as f:json.dump(result,f,indent=2);f.write('\n')
    print(json.dumps({k:result[k] for k in ('group_partition','whole_group_envelopes','reference_chronology','fused_leaf_model','one_extent_inline_cohorts','target')},indent=2))
if __name__=='__main__':main(pathlib.Path(sys.argv[1]).resolve(),pathlib.Path(sys.argv[2]).resolve())
