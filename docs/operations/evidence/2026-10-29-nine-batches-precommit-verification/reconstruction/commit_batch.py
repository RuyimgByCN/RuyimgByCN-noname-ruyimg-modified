import subprocess,json,sys,re
from pathlib import Path
from rebuild_common import EVIDENCE
n=int(sys.argv[1]);label=sys.argv[2];version=sys.argv[3];out=Path(__file__).parent/f'{n:02}'
paths=subprocess.check_output(['git','diff','--name-only','-z']).decode().strip('\0').split('\0');paths=[p for p in paths if p]
e='docs/operations/evidence/'+EVIDENCE[n-1];paths.append(e)
extra=json.loads(sys.argv[4]) if len(sys.argv)>4 else [];paths+=extra
subprocess.run(['git','add','--',*paths],check=True)
with (out/'staged-whitespace.log').open('w') as f:
 r=subprocess.run(['git','diff','--cached','--check'],stdout=f,stderr=subprocess.STDOUT);f.write(f'EXIT_CODE: {r.returncode}\n')
msg=f'Reconstructed forward from baseline 95f51867 against the frozen nine-batch snapshot. This is reconstruction validation, not an original historical source snapshot.\n\nEvidence: {e}/\nReconstruction logs: reconstruction-logs/{n:02} (outside product tree).\nRules v{version}; archive v105; native ABI 753.'
subprocess.run(['git','commit','-m',label,'-m',msg],check=True)
head=subprocess.check_output(['git','rev-parse','HEAD']).decode().strip();(out/'commit.txt').write_text(head+'\n')
subprocess.run(['git','show','--stat','--oneline','HEAD'],stdout=(out/'commit-stat.txt').open('w'),check=True)
records=[]
for log in out.glob('*.log'):
 s=log.read_text(errors='replace');counts=[tuple(map(int,m)) for m in re.findall(r'test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored',s)]
 if counts:records.append({'log':log.name,'passed_failed_ignored':list(map(sum,zip(*counts)))})
(out/'test-counts.json').write_text(json.dumps(records,indent=2))
