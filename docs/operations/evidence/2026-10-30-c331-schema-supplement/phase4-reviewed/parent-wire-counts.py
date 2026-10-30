from pathlib import Path
from collections import Counter
import hashlib,json,re
R=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-efhmbq7e/evidence/phase4')
O=Path('/tmp/noname-parent-c331-phase4-audit');O.mkdir(exist_ok=True)
def ident(p):
 b=p.read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
p=R/'final-products2/server-ws-wire.jsonl';counts=Counter();errors=[];lines=p.read_bytes().splitlines()
for n,line in enumerate(lines,1):
 try:
  outer=json.loads(line);inner=json.loads(outer['wire']);assert isinstance(inner,dict)
  counts[(outer['direction'],inner.get('type','<missing>'))]+=1
 except Exception as e:errors.append({'line':n,'error':str(e)})
wire={**ident(p),'lines':len(lines),'errors':errors,'direction_type_counts':[{'direction':k[0],'type':k[1],'count':v} for k,v in sorted(counts.items())]}
p=R/'affected-default-final2.log';text=p.read_text();rows=[]
for n,line in enumerate(text.splitlines(),1):
 m=re.search(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',line)
 if m: rows.append({'line':n,'status':m[1],'passed':int(m[2]),'failed':int(m[3]),'ignored':int(m[4]),'raw':line})
commands=json.loads((R/'final-results2.json').read_text());cmd=next(c for c in commands if c['label']=='affected-default-final2')
tests={**ident(p),'rows':rows,'totals':{k:sum(row[k] for row in rows) for k in ['passed','failed','ignored']},'command':cmd,'explicit_serial_test_arg':any('test-threads' in x for x in cmd['command']),'c331_results':[{'line':n,'raw':x} for n,x in enumerate(text.splitlines(),1) if re.match(r'test .*c331.* \.\.\. ',x)]}
assert len(lines)==3875 and not errors
assert tests['totals']=={'passed':2903,'failed':0,'ignored':76}
result={'qualification':'parent fresh read-only parsing of retained bytes; no producer execution or product experiments','wire':wire,'tests':tests}
(O/'wire-and-test-counts.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'output':str(O/'wire-and-test-counts.json'),'wire':wire,'test_totals':tests['totals'],'test_result_rows':len(rows),'c331_results':tests['c331_results']},indent=2))
