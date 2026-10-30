from pathlib import Path
import subprocess,os,json,time,re,sys,hashlib,shutil,fcntl,signal
ROOT=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
OUT=ROOT/'docs/operations/evidence/2026-10-30-pa09-desktop-native'
TARGET='/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage3.czy5e9gq/target'
INDEX=OUT/'recovery-command-results.json'
def run(name,cmd,extra=None):
 env=dict(os.environ,CARGO_TARGET_DIR=TARGET,CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true')
 if extra:env.update(extra)
 log=OUT/f'recovery-{name}.log'
 if log.exists():raise RuntimeError(f'Will not overwrite {log}')
 started=time.monotonic(); reason=None
 with Path('/tmp/noname-phase4-heavy-command.lock').open('a') as lock:
  fcntl.flock(lock,fcntl.LOCK_EX)
  free=shutil.disk_usage(ROOT).free
  if free<3*1024**3:raise RuntimeError(f'Free space below 3GiB: {free}')
  locked=time.monotonic()
  with log.open('w') as f:
   f.write(json.dumps({'command':cmd,'env':{k:env.get(k) for k in ['CARGO_TARGET_DIR','CARGO_INCREMENTAL','CARGO_BUILD_JOBS','RUSTFLAGS','NODE_OPTIONS']},'free_bytes_before':free})+'\n');f.flush()
   p=subprocess.Popen(cmd,cwd=ROOT,env=env,stdout=f,stderr=subprocess.STDOUT,start_new_session=True)
   while p.poll() is None:
    if shutil.disk_usage(ROOT).free<3*1024**3:
     reason='disk pressure below 3GiB: terminated owned process group';os.killpg(p.pid,signal.SIGTERM)
     try:p.wait(timeout=10)
     except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait()
     break
    time.sleep(2)
   seconds=time.monotonic()-locked
   f.write(json.dumps({'command':cmd,'exit':p.returncode,'locked_seconds':seconds,'stop_reason':reason})+'\n')
 text=log.read_text(errors='replace');matches=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)
 row={'name':name,'command':cmd,'exit':p.returncode,'log':log.name,'locked_seconds':seconds,'elapsed_including_lock_seconds':time.monotonic()-started,'env':{k:env.get(k) for k in ['CARGO_TARGET_DIR','CARGO_INCREMENTAL','CARGO_BUILD_JOBS','RUSTFLAGS','NODE_OPTIONS']},'free_bytes_after':shutil.disk_usage(ROOT).free,'rust_counts':[sum(int(x[i]) for x in matches) for i in range(3)] if matches else None,'stop_reason':reason}
 rows=json.loads(INDEX.read_text()) if INDEX.exists() else [];rows.append(row);INDEX.write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps(row),flush=True)
 if p.returncode:print(text[-8000:],flush=True)
 return p.returncode
if __name__=='__main__':sys.exit(run(sys.argv[1],sys.argv[2:]))
