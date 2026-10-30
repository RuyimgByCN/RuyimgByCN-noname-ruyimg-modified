from pathlib import Path
import os,sys,subprocess,json,time,re,shutil,signal
ROOT=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
OUT=ROOT/'docs/operations/evidence/2026-10-30-j2-main-integration/combined-acceptance'
TARGET='/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage3.czy5e9gq/target'
TMP=Path(Path('/tmp/b-j2-main-tmp.txt').read_text())
INDEX=OUT/'command-results.json'
def run(name,command,extra=None,cwd=ROOT):
 env=dict(os.environ,CARGO_TARGET_DIR=TARGET,CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true')
 if extra:env.update(extra)
 log=OUT/(name+'.log')
 if log.exists():raise RuntimeError('Refuse log overwrite: '+str(log))
 free=shutil.disk_usage(ROOT).free
 if free<3*1024**3:raise RuntimeError('STOP: less than3GiB free')
 argv=['python3','-B','/tmp/noname-run-serialized.py',*command]
 started=time.monotonic();stop=None
 with log.open('w') as f:
  f.write(json.dumps({'command':command,'launcher':argv,'cwd':str(cwd),'env':{k:env.get(k) for k in ['CARGO_TARGET_DIR','CARGO_INCREMENTAL','CARGO_BUILD_JOBS','RUSTFLAGS','NODE_OPTIONS']},'free_before':free})+'\n');f.flush()
  p=subprocess.Popen(argv,cwd=cwd,env=env,stdout=f,stderr=subprocess.STDOUT,start_new_session=True)
  while p.poll() is None:
   if shutil.disk_usage(ROOT).free<3*1024**3:
    stop='disk below3GiB; terminated only owned process group';os.killpg(p.pid,signal.SIGTERM)
    try:p.wait(timeout=10)
    except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait()
    break
   time.sleep(2)
  f.write(json.dumps({'runner_exit':p.returncode,'stop_reason':stop})+'\n')
 text=log.read_text(errors='replace');matches=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text);sentinels=[]
 for l in text.splitlines():
  try:
   v=json.loads(l)
   if isinstance(v,dict) and 'locked_seconds' in v:sentinels.append(v)
  except ValueError:pass
 row={'name':name,'command':command,'launcher':argv,'cwd':str(cwd),'exit':p.returncode,'log':log.name,'elapsed_including_wait':time.monotonic()-started,'locked_seconds':sentinels[-1]['locked_seconds'] if sentinels else None,'rust_counts':[sum(int(x[i]) for x in matches) for i in range(3)] if matches else None,'free_before':free,'free_after':shutil.disk_usage(ROOT).free,'stop_reason':stop}
 rows=json.loads(INDEX.read_text()) if INDEX.exists() else [];rows.append(row);INDEX.write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps(row),flush=True)
 paths=[Path(env['CARGO_TARGET_DIR'])/s for s in ['debug/incremental','release/incremental','wasm32-unknown-unknown/release/incremental']]
 audit={'name':name,'free_bytes':shutil.disk_usage(ROOT).free,'incremental_entries':{str(p):len(list(p.iterdir())) if p.exists() else 0 for p in paths},'deleted':[],'retained_reason':'Deps/binaries needed by this combined acceptance chain. Private generated outputs retained only until immediate runtime checks then archived and retired. Normal build:native removes .native-pkg-tmp in finally; main package is a shared normal asset.'}
 with (OUT/'post-command-cleanup.jsonl').open('a') as f:f.write(json.dumps(audit)+'\n')
 if p.returncode:print(text[-10000:],flush=True)
 return p.returncode
if __name__=='__main__':sys.exit(run(sys.argv[1],sys.argv[2:]))
