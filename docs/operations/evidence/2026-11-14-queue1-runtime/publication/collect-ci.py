from pathlib import Path
import datetime,hashlib,json,re,subprocess,sys,zipfile,io
label,run_id,sha=sys.argv[1:]
assert label in ['code-exact-ci-first','documentation-exact-ci-first']
base=Path('docs/operations/evidence/2026-11-14-queue1-runtime/publication')/label
initial=json.loads((base/'run-initial.json').read_text()); assert initial['id']==int(run_id) and initial['head_sha']==sha
prefix='repos/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/'+run_id
commands=[]
def api(endpoint,binary=False):
 argv=['gh','api','--allow-escape-sequences',endpoint]; r=subprocess.run(argv,capture_output=True)
 commands.append({'argv':argv,'exit_code':r.returncode})
 assert r.returncode==0,r.stderr.decode(errors='replace')
 return r.stdout if binary else json.loads(r.stdout)
def save(name,obj):
 p=base/name; assert not p.exists(),str(p)
 p.write_text(json.dumps(obj,ensure_ascii=False,indent=2)+'\n')
run=api(prefix); jobs=api(prefix+'/jobs?per_page=100'); artifacts=api(prefix+'/artifacts?per_page=100')
assert run['status']=='completed' and run['head_sha']==sha and run['head_branch']=='native-convergence'
assert run['id']==initial['id']; assert len(jobs['jobs'])==10
save('run-final.json',run); save('jobs-final.json',jobs); save('artifacts-final.json',artifacts)
raw=api(prefix+'/logs',binary=True); archive=base/'logs-first.zip'; assert not archive.exists(); archive.write_bytes(raw)
logs=base/'logs-first'; logs.mkdir()
with zipfile.ZipFile(io.BytesIO(raw)) as z:
 for item in z.infolist():
  if item.is_dir(): continue
  target=logs/item.filename
  assert target.resolve().is_relative_to(logs.resolve())
  target.parent.mkdir(parents=True,exist_ok=True); assert not target.exists(); target.write_bytes(z.read(item))
per_job=[]
for job in jobs['jobs']:
 roots=[f for f in logs.glob('*.txt') if f.stem==job['name'] or f.stem.endswith('_'+job['name'])]
 if roots:
  assert len(roots)==1,(job['name'],roots); files=roots
 else:
  files=sorted((logs/job['name']).glob('*.txt'))
 assert files,job['name']
 text='\n'.join(f.read_text(errors='replace') for f in files)
 clean=re.sub(r'\x1b\[[0-9;]*[A-Za-z]','',text)
 rows=[tuple(map(int,m)) for m in re.findall(r'test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',clean)]
 per_job.append({'job_id':job['id'],'name':job['name'],'status':job['status'],'conclusion':job['conclusion'],'started_at':job['started_at'],'completed_at':job['completed_at'],'logs':[str(f.relative_to(base)) for f in files],'rust_test_result_sections':len(rows),'rust_test_counts':dict(zip(['passed','failed','ignored','measured','filtered'],map(sum,zip(*rows)))) if rows else None,'setup_rustc_versions':sorted(set(re.findall(r'rustc (\d+\.\d+\.\d+) \([^\n)]+\)(?=\r?$)',clean,re.M))),'existing_tier_report_attempt_exits':[{'attempt':int(a),'exit':int(b)} for a,b in re.findall(r'=== attempt (\d+) exit=(\d+) ===',clean)],'new_jiushi_regression_passed':re.findall(r'test [^\n]*l29_jiushi_pending_confirmations_restore_and_match_direct_answers \.\.\. ok',clean)})
record={'at':datetime.datetime.now().astimezone().isoformat(),'head_sha':sha,'run_id':int(run_id),'run_attempt':run['run_attempt'],'run_status':run['status'],'conclusion':run['conclusion'],'commands':commands,'archive_sha256':hashlib.sha256(raw).hexdigest(),'archive_bytes':len(raw),'job_success_count':sum(j['conclusion']=='success' for j in jobs['jobs']),'job_failure_count':sum(j['conclusion']=='failure' for j in jobs['jobs']),'job_cancelled_count':sum(j['conclusion']=='cancelled' for j in jobs['jobs']),'jobs':per_job,'artifacts':[{'name':x['name'],'size_in_bytes':x['size_in_bytes'],'expired':x['expired'],'id':x['id']} for x in artifacts['artifacts']],'agent_dispatched_reruns':0,'scope':'Per-job test counts overlap between jobs and must not be added as unique product coverage. Build matrix uses --lib, full integration gates run on Linux. Installer upload does not establish installation/signing/notarization/device acceptance.'}
save('terminal-summary.json',record)
print({k:v for k,v in record.items() if k not in ['jobs','commands']})
print('jobs',[(j['name'],j['conclusion'],j['rust_test_counts'],j['setup_rustc_versions'],j['existing_tier_report_attempt_exits']) for j in per_job])
assert run['conclusion']=='success' and record['job_success_count']==10
