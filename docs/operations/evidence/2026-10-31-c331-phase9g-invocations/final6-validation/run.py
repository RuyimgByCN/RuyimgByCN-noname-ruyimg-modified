from pathlib import Path
import subprocess,os,time,json,shutil,hashlib,datetime,traceback
r=Path(Path('/tmp/c331-phase7-root').read_text().strip());w=r/'worktree';e=Path(__file__).parent;source=json.loads((e/'source-manifest.json').read_text());clock=json.loads((e/'clock.json').read_text());cut=datetime.datetime.fromisoformat(clock['no_new_heavy_utc']).timestamp();sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();utc=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat();env=os.environ.copy();env.update(CARGO_TARGET_DIR='/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-efhmbq7e/target',CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2',GIT_OPTIONAL_LOCKS='0');results=[]
commands=[('full-native',['cargo','test','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native','--no-fail-fast']),('gate1b',['cargo','check','--offline','-p','engine-native','--all-targets']),('server-check',['cargo','check','--offline','-p','game-server','--all-targets']),('wasm-controls',['cargo','test','--offline','-p','engine-native-wasm','--test','c331_invocation_controls','--','--nocapture']),('focused',['cargo','test','--offline','-p','engine-native','--test','nc_rounds','c331_','--','--nocapture']),('golden',['cargo','test','--offline','-p','engine-native','--test','kernel','replay_scenarios_match_golden','--','--test-threads=1']),('source-cap-unit',['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_source_lines.py']),('source-caps',['python3','-B','scripts/check-source-lines.py']),('layering',['python3','-B','scripts/check-native-layering.py'])]
def bind():
 actual=[{'path':x['path'],'sha256':sha(w/x['path'])}for x in source['files']];assert actual==source['files'];return {'utc':utc(),'files':actual,'manifest_sha256':sha(e/'source-manifest.json')}
def save(): (e/'command-results.json').write_text(json.dumps(results,indent=2)+'\n')
try:
 for name,args in commands:
  if args[0]=='cargo'and time.time()>=cut:results.append({'name':name,'argv':args,'status':'not-run','reason':'heavy cutoff reached'});save();continue
  assert sha(Path('/tmp/noname-run-serialized.py'))=='d7f246f1fdbd4ec6d85deb22643d7ca8137be62539ebd5e75d49c779febb8e34';lock=Path('/tmp/noname-phase4-heavy-command.lock').stat();assert(lock.st_dev,lock.st_ino)==(16777231,620205209);assert shutil.disk_usage(r).free>=3*1024**3
  before=bind();(e/(name+'-source-before.json')).write_text(json.dumps(before,indent=2)+'\n');current=env.copy()
  if name=='gate1b':current.update(CARGO_TARGET_DIR=str(r/'target-strict'),RUSTFLAGS='-Dwarnings')
  if name=='focused':current.update(C331_PREDICATE_TRACE_DIR=str(e/'predicate-traces'))
  if name=='golden':current.update(C331_GOLDEN_TRACE_DIR=str(e/'current-final-traces'),C331_SEMANTIC_TRACE_DIR=str(e/'current-command-traces'))
  start=utc();tick=time.monotonic();log=e/(name+'.log')
  with log.open('w')as out:done=subprocess.run(['python3','/tmp/noname-run-serialized.py',*args],cwd=w,env=current,stdout=out,stderr=subprocess.STDOUT)
  after=bind();(e/(name+'-source-after.json')).write_text(json.dumps(after,indent=2)+'\n');text=log.read_text();summary=None
  for line in reversed(text.splitlines()):
   try:
    item=json.loads(line)
    if isinstance(item,dict)and item.get('command')==args and 'exit'in item:summary=item;break
   except json.JSONDecodeError:pass
  entry={'name':name,'argv':args,'status':'passed'if done.returncode==0 else'failed','exit':done.returncode,'start_utc':start,'end_utc':utc(),'seconds':time.monotonic()-tick,'target':current['CARGO_TARGET_DIR'],'RUSTFLAGS':current.get('RUSTFLAGS'),'source_before_sha256':sha(e/(name+'-source-before.json')),'source_after_sha256':sha(e/(name+'-source-after.json')),'log_sha256':sha(log),'runner_summary':summary};results.append(entry);save()
  if summary is None or summary['exit']!=done.returncode:raise RuntimeError('Runner/launch failure: missing or inconsistent serialized-runner completion; stop without fallback')
 (e/'runner-completion.json').write_text(json.dumps({'utc':utc(),'status':'all commands considered','commands':len(results),'owned_queue_empty':True},indent=2)+'\n')
except BaseException as error:
 (e/'runner-failure.json').write_text(json.dumps({'utc':utc(),'error':str(error),'traceback':traceback.format_exc(),'completed_commands':len(results),'queued_work':False},indent=2)+'\n');raise
