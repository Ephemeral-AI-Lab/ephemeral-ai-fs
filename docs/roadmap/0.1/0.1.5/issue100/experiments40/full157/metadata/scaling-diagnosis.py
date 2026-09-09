"""Read-only counts from sealed D cache/results; no encoding or policy changes."""
import hashlib,json,math,pathlib,sqlite3
HERE=pathlib.Path(__file__).resolve().parent
TEN=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-experiments/metadata')
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def main():
 result=json.loads((HERE/'result.json').read_text());assert sha(HERE/'cache.sqlite')==result['cache_sha256']
 db=sqlite3.connect((HERE/'cache.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True)
 leaves=branches=occurrences=canonical=0;values=set();pairs=set();keys=set()
 for identity,b in db.execute('select id,canonical from metadata where substr(canonical,14,8)=?',(b'LFS6INT\0',)):
  if b[23]==7:
   leaves+=1;n=int.from_bytes(b[26:28],'big');assert len(b)==44+81*n;occurrences+=n;canonical+=len(b)
   for pos in range(44,len(b),81):
    key=b[pos:pos+8];v=b[pos+8:pos+81];keys.add(key);values.add(v);pairs.add((key,v))
  else:branches+=1
 db.close()
 ten_roots=json.loads((TEN/'roots-D.json').read_text());ten=json.loads((TEN/'result-D.json').read_text())
 ten_occ=sum(r['inodes'] for r in ten_roots);ten_leaves=sum(math.ceil(r['inodes']/100) for r in ten_roots);ten_branches=sum(r['inodes']>100 for r in ten_roots)
 # All ten-stateconstructedtable nodes are distinct, so no physicalleaves disappear throughCAS.
 assert ten_leaves+ten_branches==ten['role_counts']['LFS6INT']==604
 unique=len(values);assert unique==89576 and len(keys)==52724 and occurrences==885543
 def cohort(occ,n):return dict(inline_occurrences=occ,unique_inode_values=n,copies_per_value=occ/n,repeated_occurrences=occ-n,inline_value_raw_bytes=occ*73,repeated_inline_value_raw_bytes=(occ-n)*73,fixedmembership_hybrid_raw_reduction_bytes=occ*41-n*98,additional_hybrid_CAS_rows=n)
 full_roots=json.loads((HERE/'roots.json').read_text())
 out=dict(full=cohort(occurrences,unique),ten=cohort(ten_occ,37289),full_unique_inode_value_pairs=len(pairs),full_distinct_inode_ids=len(keys),full_unique_leaf_objects=leaves,full_branch_objects=branches,full_leaf_canonical_bytes=canonical,full_logical_inode_occurrences=sum(x['inodes'] for x in full_roots),ten_unique_leaf_objects=ten_leaves,ten_branch_objects=ten_branches,inline_occurrence_growth=occurrences/ten_occ,unique_inode_value_growth=unique/37289,original_full_metadata_delta=dict(records=4673,canonical_bytes=26757164,selected_record_bytes=5571112,canonical_minus_record_bytes=26757164-5571112,FULL_record_minus_DELTA_record_bytes=26757164+4673-5571112,qualification='Beforewholegroupcompression;neitherquantityisallocatedsaving'),scope='Counts and fixedmembershiprawarithmetic only. Noalternativeencoding,policychange orStorewrite.',source_hashes={str(p):sha(p) for p in [HERE/'cache.sqlite',HERE/'result.json',HERE/'roots.json',TEN/'result-D.json',TEN/'roots-D.json']})
 assert sha(HERE/'cache.sqlite')==result['cache_sha256']
 (HERE/'scaling-diagnosis.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({k:out[k] for k in ['full','ten','full_logical_inode_occurrences','inline_occurrence_growth','unique_inode_value_growth']},indent=2))
if __name__=='__main__':main()
