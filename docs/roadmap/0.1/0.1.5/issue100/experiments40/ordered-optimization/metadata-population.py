"""Independent field repetition census from sealed canonical D metadata caches."""
import collections,hashlib,json,sqlite3
from pathlib import Path
ROOT=Path(__file__).resolve().parent
SOURCES={53:Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-structural/metadata/d/cache.sqlite'),157:Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/metadata/cache.sqlite')}

def main():
 output={}
 for count,path in SOURCES.items():
  with path.open('rb') as f:seal=hashlib.file_digest(f,'sha256').hexdigest()
  db=sqlite3.connect(path.as_uri()+'?mode=ro&immutable=1',uri=True)
  values=collections.Counter();fields=[collections.Counter() for _ in range(4)];leaves=0;bytecount=0
  for identity,b in db.execute('select id,canonical from metadata'):
   if b[13:21]!=b'LFS6INT\0' or b[23]!=7:continue
   assert len(b)>=44 and (len(b)-44)%81==0;leaves+=1;bytecount+=len(b)
   for start in range(44,len(b),81):
    value=b[start+8:start+81];assert len(value)==73;values[value]+=1
    for i,part in enumerate((value[:1],value[1:9],value[9:41],value[41:73])):fields[i][part]+=1
  db.close();occurrences=sum(values.values())
  output[str(count)]=dict(source=str(path),sha256=seal,leaf_objects=leaves,canonical_leaf_bytes=bytecount,value_occurrences=occurrences,unique_values=len(values),copies_per_value=occurrences/len(values),raw_repeated_value_bytes=(occurrences-len(values))*73,unique_field_counts=dict(zip(['type','link_count','content_or_directory','attributes'],map(len,fields))),scope='Physical selected canonical leaf population; raw repetition is not compressed savings')
 (ROOT/'metadata-population.json').write_text(json.dumps(output,indent=2)+'\n');print(json.dumps(output,indent=2))

if __name__=='__main__':main()
