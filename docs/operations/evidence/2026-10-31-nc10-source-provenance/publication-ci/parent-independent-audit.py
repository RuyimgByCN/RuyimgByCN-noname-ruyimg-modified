from pathlib import Path
from collections import Counter
import json,hashlib,gzip,zlib,re,tempfile,datetime
R=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-ci-5fc0f774-publication-z03yx4in');H='5fc0f77403d90fa4555156bf01722450cbe7bf3b';ID=41183155464
load=lambda p:json.loads(p.read_bytes())
sha=lambda b:hashlib.sha256(b).hexdigest()
m=load(R/'copy-manifest.json');assert len(m['files'])==92 and m['manifest_excludes_itself'];assert sha((R/'copy-manifest.json').read_bytes())=='8f7eec2708f01641d7cc35dfda78dbc24222bcf858b09675bac74009c2af7b38'
for x in m['files']:
 b=(R/x['path']).read_bytes();assert len(b)==x['bytes'] and sha(b)==x['sha256'] and f'{zlib.crc32(b):08x}'==x['crc32'],x['path']
run=load(R/'ci-stage/run-final.stdout');assert (run['id'],run['head_sha'],run['run_attempt'],run['status'],run['conclusion'])==(ID,H,1,'completed','success')
jobs=load(R/'ci-stage/jobs-final.stdout')['jobs'];assert len(jobs)==10 and all(j['conclusion']=='success' for j in jobs)
steps=[s for j in jobs for s in j['steps']];conclusions=dict(Counter(s['conclusion'] for s in steps));assert conclusions=={'success':128,'skipped':2} and len(steps)==130
producer=load(R/'verified-publication-ci-summary.json');by_id={x['id']:x for x in producer['jobs']};records=[]
for j in jobs:
 detail=load(R/f"ci-stage/job-{j['id']}-detail.stdout");assert (detail['id'],detail['run_id'],detail['conclusion'])==(j['id'],ID,'success');assert detail['steps']==j['steps']
 compressed=(R/f"ci-stage/job-{j['id']}-raw-log.stdout.gz").read_bytes();raw=gzip.decompress(compressed);expected=by_id[j['id']];text=raw.decode();assert len(raw)==expected['log_bytes'] and sha(raw)==expected['log_sha256'] and f'{zlib.crc32(raw):08x}'==expected['log_crc32'] and raw.count(b'\r\n')==expected['log_crlf_count']
 rust=[tuple(map(int,x)) for x in re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)];totals=[sum(x[i] for x in rust) for i in range(3)]
 if j['name']=='gates-ubuntu':assert totals==[6917,0,77] and len(rust)==164 and re.search(r'# pass\s+17\b',text)
 if j['name']=='native-web':assert totals==[1825,0,75] and '# tests 179' in text and '# pass 179' in text
 if j['name'].startswith('build-'):assert totals==[853,0,0]
 metrics=[]
 for metric in expected.get('metrics',[]):
  assert metric['actual_us']<=metric['budget_us'];assert metric['name'] in text;metrics.append(metric)
 if j['name']=='desktop-windows-latest':
  for n in [7,10,11]:assert next(s for s in detail['steps'] if s['number']==n)['conclusion']=='success'
 records.append({'id':j['id'],'name':j['name'],'conclusion':j['conclusion'],'log_bytes':len(raw),'log_sha256':sha(raw),'log_crc32':f'{zlib.crc32(raw):08x}','crlf':raw.count(b'\r\n'),'rust_totals':totals,'metrics':metrics})
assert sum(x['log_bytes'] for x in records)==4754975
artifacts=load(R/'ci-stage/artifacts-final.stdout')['artifacts'];assert {a['name'] for a in artifacts}=={'installer-macos-latest','installer-windows-latest','installer-ubuntu-latest'}
assert all(a['workflow_run']['id']==ID and a['workflow_run']['head_sha']==H and not a['expired'] for a in artifacts)
push=load(R/'push-stage/result.json');scope=load(R/'push-stage/scope.json');assert push['accepted_commit']==H and push['expected_old_remote']=='4dd0b286b995fe414128b6c52eb83a4d0d85f6cd' and push['push_exit']==0 and push['push_attempts']==1;assert push['before']==push['after'];assert len(push['committed_increment_paths'])==477 and len(push['before']['accepted_file_sha256'])==477
assert push['before']['original_hook_sha256']=='e349af1ec5ccc1742df7e7d91d387ed2ef086f7c864d8cf7372d0ef66dd49ca7';assert push['before']['core_hooksPath_exit']==1 and not push['before']['core_hooksPath_stdout']
assert len(scope['selftests'])==8 and all(t['exit']==t['expected'] for t in scope['selftests']);guard=(R/'push-stage/scoped-pre-push.raw.txt').read_bytes();assert sha(guard)=='ba0dbbd4e1d9d34fe6534aa0a716460a90ca9c29c2bafba8c59cccc1f8aa3d96'
refs=push['remote_after_refs'];assert refs['refs/heads/native-convergence']==H and refs['refs/heads/main']=='47347c6608d651660ae9459e59eb529b3cef277f'
assert len(producer['watch_commands'])==1 and producer['watch_commands'][0]['exit']==0
# Negative digest check proves corruption would not satisfy the retained identity.
assert sha((R/'ci-stage/run-final.stdout').read_bytes()+b'!')!=next(x['sha256'] for x in m['files'] if x['path']=='ci-stage/run-final.stdout')
O=Path(tempfile.mkdtemp(prefix='noname-parent-ci41183155464-audit-'))
out={'verified_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'result':'PASS','manifest_entries_verified':92,'primary_files_including_manifest_and_seal':94,'run':{k:run[k] for k in ['id','head_sha','run_attempt','status','conclusion','updated_at']},'jobs':records,'steps':{'total':130,**conclusions,'failure':0,'null':0},'artifacts':[{'id':a['id'],'name':a['name'],'bytes':a['size_in_bytes'],'digest':a['digest']} for a in artifacts],'push':{'exit':0,'attempts':1,'changed_paths':477,'before_after_equal':True,'guard_sha256':sha(guard),'guard_selftests':8,'remote_after_refs':refs,'lease':load(R/'push-stage/lease-release.json')},'watch':producer['watch_commands'],'warnings_ignored_not_erased':True,'warning_count_producer':producer['warning_line_count'],'negative_digest_check':True,'qualification':'Fresh retained bytes/API/detail/gzip/hash/CRC/count verification only; no producer/tests/build/network or currentMain snapshot. All actualWindowssteps green, no installer download/install/signing/device/defaultswitch claim. Live independent API fetch recorded separately.'}
(O/'independent-results.json').write_text(json.dumps(out,indent=2)+'\n');Path('/tmp/noname-parent-ci41183155464-audit-current.json').write_text(json.dumps({'out':str(O)},indent=2)+'\n');print(json.dumps({'result':'PASS','out':str(O),'run':ID,'jobs':10,'steps':out['steps'],'artifacts':len(artifacts)},indent=2))
