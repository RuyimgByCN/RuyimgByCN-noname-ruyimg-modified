import os,sys,pathlib,subprocess,time,json,hashlib,shutil
root=pathlib.Path('/tmp/i02-phase1-ingress-repair-89364384');source=pathlib.Path('/tmp/i02-phase1-ab167599/source');name=sys.argv[1];argv=sys.argv[2:];env=os.environ.copy();env.update(CARGO_TARGET_DIR='/tmp/i02-phase1-ab167599/target',CARGO_BUILD_JOBS='2',CARGO_INCREMENTAL='0')
assert shutil.disk_usage(source).free>=3*1024**3,'free disk below 3GiB floor'
wrapper=pathlib.Path('/tmp/noname-run-serialized.py');assert hashlib.sha256(wrapper.read_bytes()).hexdigest()=='d7f246f1fdbd4ec6d85deb22643d7ca8137be62539ebd5e75d49c779febb8e34'
lock=pathlib.Path('/tmp/noname-phase4-heavy-command.lock').stat();assert (lock.st_dev,lock.st_ino)==(16777231,620205209)
cmd=['python3',str(wrapper),*argv];(root/f'evidence/{name}.argv.json').write_text(json.dumps({'argv':cmd,'cwd':str(source),'env':{k:env[k] for k in ['CARGO_TARGET_DIR','CARGO_BUILD_JOBS','CARGO_INCREMENTAL']},'started':time.time(),'free_disk_bytes_before':shutil.disk_usage(source).free,'memory_limiter':False,'profiles':'unchanged'},indent=2))
with (root/f'evidence/{name}.log').open('w') as f:r=subprocess.run(cmd,cwd=source,env=env,stdout=f,stderr=subprocess.STDOUT)
(root/f'evidence/{name}.result.json').write_text(json.dumps({'exit':r.returncode,'finished':time.time(),'free_disk_bytes_after':shutil.disk_usage(source).free},indent=2));print(name,r.returncode)
