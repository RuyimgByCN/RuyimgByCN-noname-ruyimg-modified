import os,sys,json,time,subprocess,pathlib,shutil
root=pathlib.Path(__file__).resolve().parent
name,*argv=sys.argv[1:]
env=dict(os.environ,CARGO_TARGET_DIR='/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage3.czy5e9gq/target',CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2')
with (root/(name+'.log')).open('w') as out:
 result=subprocess.run(['python3','-B','/tmp/noname-run-serialized.py',*argv],env=env,stdout=out,stderr=subprocess.STDOUT)
record={'name':name,'argv':argv,'exit':result.returncode,'disk_free_bytes':shutil.disk_usage('.').free,'finished_unix':time.time()}
with (root/'commands.jsonl').open('a') as out:out.write(json.dumps(record)+'\n')
sys.exit(result.returncode)
