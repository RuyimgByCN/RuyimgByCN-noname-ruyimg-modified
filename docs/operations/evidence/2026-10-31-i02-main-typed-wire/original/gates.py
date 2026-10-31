import sys,json
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent))
from run import E,R,run,snapshot
snapshot('gate-entry')
checks=[('line-unit',['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_source_lines.py']),('source-caps',['python3','-B','scripts/check-source-lines.py']),('layering-unit',['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_native_layering.py']),('layering-gate',['python3','-B','scripts/check-native-layering.py']),('strict-doc-lines',['python3','-B','scripts/check_doc_line_refs.py','--strict']),('local-links',['python3','-B','scripts/check_local_links.py','--all']),('preflight',['python3','-B','scripts/preflight.py'])]
failed=[]
for name,argv in checks:
 if run(name,argv):failed.append(name)
snapshot('gate-exit')
a=json.load(open(E/'gate-entry.json'));b=json.load(open(E/'gate-exit.json'));keys=['head','tree','indexSha256','indexEntries','headEntries','staged','trackedStatus','untrackedNames','sourceBinding','maintainedAndMarkdown','protectedSets','hooks'];comparison={k:a[k]==b[k] for k in keys};(E/'gate-comparison.json').write_text(json.dumps({'allEqual':all(comparison.values()),'fields':comparison,'failedGates':failed},indent=2));print(json.dumps({'allFrozenEqual':comparison,'failedGates':failed}),flush=True)
sys.exit(1 if failed or not all(comparison.values()) else 0)
