import os,sys,json,time,subprocess,shutil,re,datetime
from pathlib import Path
ROOT=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');E=Path(__file__).resolve().parent
name,*argv=sys.argv[1:];env=dict(os.environ,CARGO_TARGET_DIR='/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage3.czy5e9gq/target',CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true')
command=['python3','-B','/tmp/noname-run-serialized.py','/usr/bin/sandbox-exec','-f',str(E/'readonly.sb'),*argv]
if shutil.disk_usage(ROOT).free<3*1024**3:raise SystemExit('disk below3GiB')
started=time.monotonic();submittedAt=datetime.datetime.now().isoformat()
with (E/(name+'.log')).open('w') as log:result=subprocess.run(command,cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT)
text=(E/(name+'.log')).read_text();m=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',text)
row={'name':name,'argv':argv,'execution':command,'env':{k:env[k] for k in ['CARGO_TARGET_DIR','CARGO_INCREMENTAL','CARGO_BUILD_JOBS','CARGO_NET_OFFLINE']},'exit':result.returncode,'log':name+'.log','rustCounts':[sum(int(x[i]) for x in m) for i in range(3)] if m else None,'warningLines':sum(1 for line in text.splitlines() if line.startswith('warning:')),'diskFreeBytes':shutil.disk_usage(ROOT).free,'incrementalBytes':sum(p.stat().st_size for p in (Path(env['CARGO_TARGET_DIR'])/'debug/incremental').rglob('*') if p.is_file()),'finishedAt':datetime.datetime.now().isoformat(),'submittedAt':submittedAt,'secondsIncludingQueue':time.monotonic()-started}
for line in reversed(text.splitlines()):
 if line.startswith('{"command"'):
  try:row['wrapper']=json.loads(line);break
  except ValueError:pass
if row.get('wrapper'):row['queueSeconds']=max(0,row['secondsIncludingQueue']-row['wrapper']['locked_seconds'])
with (E/'commands.jsonl').open('a') as out:out.write(json.dumps(row)+'\n')
sys.exit(result.returncode)
