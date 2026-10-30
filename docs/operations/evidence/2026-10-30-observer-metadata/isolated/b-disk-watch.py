import subprocess,os,signal,shutil,time,json
from pathlib import Path
out=Path(__file__).resolve().parent
for _ in range(3600):
 free=shutil.disk_usage(out).free
 if (out/'b-handoff-complete').exists():break
 if free<3*1024**3:
  table=[line.strip().split(None,2) for line in subprocess.check_output(['ps','-Ao','pid,ppid,command']).decode().splitlines()[1:] if line.strip()]
  roots={int(pid) for pid,ppid,cmd in table if '../evidence/run-b.py b-' in cmd and 'python' in cmd.lower()}
  descendants=set(roots)
  for _ in range(12):
   descendants.update(int(pid) for pid,ppid,cmd in table if int(ppid) in descendants)
  stopped=[]
  for pid in sorted(descendants-roots,reverse=True):
   try:os.kill(pid,signal.SIGTERM);stopped.append(pid)
   except ProcessLookupError:pass
  (out/'b-disk-stop.json').write_text(json.dumps({'free':free,'stopped_own_children':stopped,'time':time.time()},indent=2)+'\n')
  break
 time.sleep(1)
