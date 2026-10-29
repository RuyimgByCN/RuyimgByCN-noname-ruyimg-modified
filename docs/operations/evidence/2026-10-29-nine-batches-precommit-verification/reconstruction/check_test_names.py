from pathlib import Path
import re,json,hashlib
root=Path(__file__).resolve().parent
pairs=[('engine-native','06/test-lists-00.log','07/grouping-02.log'),('engine-core','06/test-lists-01.log','07/grouping-03.log')]
def names(path):
 out=[];prefix=None
 for line in path.read_text().splitlines():
  if 'Running unittests ' in line or 'Doc-tests ' in line:prefix=None
  m=re.search(r'Running tests/(\S+)\.rs ',line)
  if m:
   p=m[1];prefix='' if p.startswith('groups/') else Path(p).name+'::'
  elif prefix is not None and line.endswith(': test'):
   name=prefix+line[:-6]
   out.append(name)
 return sorted(out)
report=[]
for crate,old,new in pairs:
 a=names(root/old);b=names(root/new);extra=sorted(set(b)-set(a));missing=sorted(set(a)-set(b));assert not missing,(crate,missing);assert len(extra)==1,(crate,extra)
 b=[x for x in b if x not in extra];assert a==b,(crate,len(a),len(b));sha=hashlib.sha256(('\n'.join(a)+'\n').encode()).hexdigest();report.append(dict(crate=crate,before=len(a),after=len(b),added_registry=extra,sha256=sha))
(root/'07/test-name-conservation.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2))
