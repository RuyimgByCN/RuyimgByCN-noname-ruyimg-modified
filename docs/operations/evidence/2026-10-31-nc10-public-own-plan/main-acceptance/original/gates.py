import sys,json
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent))
from verify import run,snapshot,O
checks=[('line-unit',['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_source_lines.py']),('source-caps',['python3','-B','scripts/check-source-lines.py']),('layering-unit',['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_native_layering.py']),('layering-gate',['python3','-B','scripts/check-native-layering.py']),('strict-doc-lines',['python3','-B','scripts/check_doc_line_refs.py','--strict']),('preflight',['python3','-B','scripts/preflight.py'])]
failures=[]
for name,argv in checks:
 if run(name,argv):failures.append(name)
snapshot('freeze-exit')
a=json.load(open(O/'freeze-entry.json'));b=json.load(open(O/'freeze-exit.json'))
fields=['head','tree','source6','index_sha256','index_entries','staged','status','tracked_hashes','hooks','protected','maintained_and_markdown']
comparison={key:a[key]==b[key] for key in fields};(O/'freeze-comparison.json').write_text(json.dumps({'fields':comparison,'failures':failures,'equal':all(comparison.values())},indent=2))
print(json.dumps({'frozenFieldsEqual':comparison,'failedGates':failures}),flush=True)
sys.exit(1 if failures or not all(comparison.values()) else 0)
