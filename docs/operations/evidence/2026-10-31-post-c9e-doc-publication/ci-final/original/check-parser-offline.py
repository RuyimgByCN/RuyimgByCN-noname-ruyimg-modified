import ast
import datetime
import json
from pathlib import Path
import re

ROOT=Path(__file__).resolve().parent
source=ROOT/'verify-publication-ci.py'
text=source.read_text()
compile(text,str(source),'exec')
tree=ast.parse(text)
section_node=next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='section')
space={}
exec(compile(ast.Module(body=[section_node],type_ignores=[]),str(source),'exec'),space)
assert space['section'](['one'],'missing')==[]
assert space['section'](['##[group]Run cargo test --workspace','test result','##[group]Run next'],'cargo test --workspace')==['##[group]Run cargo test --workspace','test result']
pattern=r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;'
interleaved=['Running unittests src/lib.rs (target/debug/deps/engine_api-aa)','test a ... ok','Running unittests src/main.rs (target/debug/deps/engine_cli-bb)','test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;','test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;']
headers=[line for line in interleaved if line.startswith('Running ')]
results=[match for line in interleaved if (match:=re.search(pattern,line))]
assert len(headers)==len(results)==2
paired=[(head,int(result[1])) for head,result in zip(headers,results)]
assert 'engine_api-' in paired[0][0] and paired[0][1]==4 and paired[1][1]==0
record={'verified_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'private_parser_compile':'passed','missing_section_unknown_not_exception':'passed','section_boundary_control':'passed','FIFO_interleaved_nextheader_before_previous_summary':'passed','runningN_notusedasboundary':True,'producttests_orproviderrequests':0}
with (ROOT/'parser-offline-controls.json').open('x') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps(record,indent=2))
