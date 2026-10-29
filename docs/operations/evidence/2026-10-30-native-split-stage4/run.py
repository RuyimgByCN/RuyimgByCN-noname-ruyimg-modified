import subprocess,sys,time,json,os,pathlib,datetime
E=pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-10-30-native-split-stage4')
name,cwd,*cmd=sys.argv[1:]
start=time.monotonic()
with (E/(name+'.log')).open('w') as f:
 f.write('$ '+repr(cmd)+'\ncwd='+cwd+'\n');f.flush()
 p=subprocess.run(cmd,cwd=cwd,stdout=f,stderr=subprocess.STDOUT)
record={'name':name,'cwd':cwd,'command':cmd,'exit':p.returncode,'seconds':round(time.monotonic()-start,3),'date':datetime.datetime.now().astimezone().isoformat()}
with (E/'commands.jsonl').open('a') as f:f.write(json.dumps(record)+'\n')
print(json.dumps(record),flush=True)
sys.exit(p.returncode)
