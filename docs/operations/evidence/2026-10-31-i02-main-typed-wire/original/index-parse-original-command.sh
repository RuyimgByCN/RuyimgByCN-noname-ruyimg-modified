python3 - <<'PY'
from pathlib import Path
import json,shutil
E=Path('/tmp/i02-main-typed-wire-functional-33d21ba5');session=Path('/Users/ruyimg/.pi/agent/sessions/--Users-ruyimg-Documents-Claude-sanguosha--/2026-10-29T12-17-50-921Z_01a18798-5949-73c1-8642-9d0f55dc485a/df47e19c-73c6-4cdd-a45e-5437618a62f2/run-0/session.jsonl');found={}
for line in session.open():
 j=json.loads(line);m=j.get('message',{});content=m.get('content',[])
 if not isinstance(content,list):continue
 for c in content:
  if c.get('type')=='toolCall' and c.get('name')=='bash':
   command=c.get('arguments',{}).get('command','')
   if 'def parse(p):' in command and "before-index.raw" in command:found['index-parse-original-command.sh']=command
   if 'def masked(p):' in command and 'actual-main-js-load-binding' in command:found['index-mask-and-js-load-original-command.sh']=command
  if m.get('role')=='toolResult' and c.get('type')=='text' and 'seal.py", line 13' in c.get('text','') and 'AssertionError' in c.get('text',''):found['seal-attempt1.stderr.log']=c['text']
for name,data in found.items():(E/name).write_text(data+'\n')
assert len(found)==3,found.keys();print('original metadata tool commands/failure retained',list(found))
p=E/'seal.py';s=p.read_text();s=s.replace("assert all(comparison[k] for k in fields);assert all(v", "assert all(comparison[k] for k in fields if k!='indexSha256');rawdiff=json.load(open(E/'index-raw-field-diff.json'));assert rawdiff['changedEntriesCount']==33 and set(rawdiff['changedFields'])=={'ctimeSeconds','ctimeNanos'} and not rawdiff['behaviorSignificantEntryChanges'] and rawdiff['sameExtensions'] and rawdiff['sameVersion'] and rawdiff['bodyEqualAfterMaskingOnlyEntryCtime'] and rawdiff['gateEntryExitRawExact'];assert all(v")
s=s.replace("'rawEqual':True,'logicalEqualHead'", "'rawEqual':before['indexSha256']==after['indexSha256'],'rawFrozenGateEqual':True,'ctimeOnlyChangedEntries':33,'mutationOrigin':'Unknown; no attribution or index repair','fieldDiffSha256':sha(E/'index-raw-field-diff.json'),'logicalEqualHead'")
s=s.replace("'MainIndexRawEqual':True,'MainIndexLogicalEqualHead'", "'MainIndexRawEqual':False,'MainIndexRawFrozenGateEqual':True,'indexRawLimitation':'Initial to pre-gate raw difference confined to ctime fields on33 entries; all OID/path/mode/stage/flags/extensions/other bytes unchanged; writer unknown. No repair.','indexRawFieldDiffSha256':sha(E/'index-raw-field-diff.json'),'MainIndexLogicalEqualHead'")
s=s.replace("'allOriginalCommandsExit0':True,'nativeRustCounts'", "'allRecordedProducerAndGateCommandsExit0':True,'metadataToolAttempts':[{'name':'seal-attempt1','exit':1,'cause':'Initial-to-final raw index equality assertion failed; retained raw copies/traceback/source. Independently parsed ctime-only33 entry change; explicit parent conditional disposition, no product test failure or rerun.'}],'nativeRustCounts'")
p.write_text(s)
PY
python3 -B /tmp/i02-main-typed-wire-functional-33d21ba5/seal.py
