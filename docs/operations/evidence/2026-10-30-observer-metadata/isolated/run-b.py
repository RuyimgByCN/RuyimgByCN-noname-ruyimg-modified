import sys,subprocess,os,json,time,shutil
from pathlib import Path
out=Path(__file__).resolve().parent
name=sys.argv[1]; command=sys.argv[2:]
free=shutil.disk_usage(Path.cwd()).free
if free<3*1024**3: raise SystemExit('STOP: free space <3 GiB')
env=os.environ.copy(); env.update(CARGO_TARGET_DIR='/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-referee-pbhzrko1/target',CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2')
record={'name':name,'command':command,'cwd':str(Path.cwd()),'env':{k:env[k] for k in ['CARGO_TARGET_DIR','CARGO_INCREMENTAL','CARGO_BUILD_JOBS']},'free_before':free,'start':time.time()}
with (out/(name+'.log')).open('x') as log:
 result=subprocess.run(['python3','/tmp/noname-run-serialized.py',*command],env=env,stdout=log,stderr=subprocess.STDOUT)
record.update(exit=result.returncode,end=time.time(),free_after=shutil.disk_usage(Path.cwd()).free)
(out/(name+'.json')).write_text(json.dumps(record,indent=2)+'\n')
print(name,result.returncode)
sys.exit(result.returncode)
