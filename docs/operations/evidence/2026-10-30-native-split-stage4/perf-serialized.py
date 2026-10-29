import pathlib,subprocess,sys,time,json,shutil,os,datetime,threading
base=pathlib.Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage4-9rhcfkw2')
E=pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-10-30-native-split-stage4')
processes=[]

def jobs():
 lines=subprocess.check_output(['ps','-axo','pid,ppid,comm'],text=True).splitlines()[1:];rows={}
 for line in lines:
  parts=line.strip().split(None,2)
  if len(parts)==3:rows[int(parts[0])]=(int(parts[1]),parts[2])
 return rows

def foreign(rows,own=None):
 descendants={os.getpid()}
 if own is not None:descendants.add(own)
 while True:
  larger=descendants|{pid for pid,(parent,_) in rows.items() if parent in descendants}
  if larger==descendants:break
  descendants=larger
 return {pid:command for pid,(_,command) in rows.items() if pid not in descendants and pathlib.Path(command).name in ('cargo','rustc','cargo-clippy','clippy-driver')}

def settle():
 idle=None
 while True:
  active=foreign(jobs())
  processes.append({'at':datetime.datetime.now().astimezone().isoformat(),'phase':'waiting-idle','foreign':active})
  if not active:
   if idle is None:idle=time.monotonic()
   if time.monotonic()-idle>=3:return
  else:idle=None
  (E/'performance-process-monitor.json').write_text(json.dumps(processes,indent=2)+'\n')
  time.sleep(1)

def build(phase,kind,attempt):
 settle();name=f'{phase}-{kind}-serialized-{attempt}';root=base/phase
 packages=['-p','engine-native'] if phase=='before' else ['-p','engine-native-kernel','-p','engine-native-content','-p','engine-native']
 cmd=[sys.executable,'-B','/tmp/noname-run-serialized.py','env','CARGO_TARGET_DIR='+str(base/('target-'+phase)),'CARGO_BUILD_JOBS=8','CARGO_INCREMENTAL=1','/usr/bin/time','-p','cargo','test','--offline',*packages,'--no-run']
 contaminated={}
 with (E/(name+'.log')).open('w') as log:
  log.write('$ '+repr(cmd)+'\ncwd='+str(root)+'\n');log.flush()
  p=subprocess.Popen(cmd,cwd=root,stdout=log,stderr=subprocess.STDOUT)
  while p.poll() is None:
   active=foreign(jobs(),p.pid);contaminated.update(active)
   processes.append({'at':datetime.datetime.now().astimezone().isoformat(),'phase':name,'foreign':active})
   time.sleep(.25)
 record={'name':name,'command':cmd,'exit':p.returncode,'cwd':str(root),'foreign_compiler_processes':contaminated,'clean_process_window':not contaminated}
 with (E/'commands.jsonl').open('a') as f:f.write(json.dumps(record)+'\n')
 (E/'performance-process-monitor.json').write_text(json.dumps(processes,indent=2)+'\n')
 print(json.dumps(record),flush=True)
 assert p.returncode==0
 return not contaminated

results={}
for phase in ['before','after']:
 for attempt in range(1,4):
  settle()
  target=base/('target-'+phase)
  assert target.parent==base and target.name in ('target-before','target-after')
  size=subprocess.check_output(['du','-sk',str(target)],text=True)
  with (E/'performance-cache-reclaims.jsonl').open('a') as f:f.write(json.dumps({'path':str(target),'du':size,'phase':phase,'attempt':attempt,'date':datetime.datetime.now().astimezone().isoformat()})+'\n')
  shutil.rmtree(target)
  cold=build(phase,'cold',attempt)
  root=base/phase;rel='crates/engine-native/src/content/standard.rs' if phase=='before' else 'crates/engine-native-content/src/content/standard.rs';p=root/rel;original=p.read_bytes()
  edit=json.loads((E/(phase+'-edit.json')).read_text());assert edit['old'] in original.decode()
  try:
   p.write_text(original.decode().replace(edit['old'],edit['new']))
   warm=build(phase,'warm',attempt)
  finally:p.write_bytes(original)
  assert p.read_bytes()==original
  results[phase]={'attempt':attempt,'cold_clean':cold,'warm_clean':warm,'source_restored':True,'target_kib':subprocess.check_output(['du','-sk',str(target)],text=True)}
  (E/'performance-serialized-results.json').write_text(json.dumps(results,indent=2)+'\n')
  if cold and warm:break
 else:raise RuntimeError('All performance attempts overlapped external compiler jobs')
