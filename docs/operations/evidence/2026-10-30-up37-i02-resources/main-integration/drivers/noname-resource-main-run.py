import subprocess,os,json,time,pathlib,sys
root=pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');out=root/'docs/operations/evidence/2026-10-30-up37-i02-resources/main-integration';out.mkdir(parents=True,exist_ok=True)
env={**os.environ,'CARGO_INCREMENTAL':'0'}
commands=[('hash-before',['python3','-B','/tmp/noname-resource-main-hashes.py','before'],root),('pipeline',['node','--test','scripts/test_original_resources.mjs','scripts/test_original_content_order.mjs'],root),('resources-sync',['node','scripts/sync-original-content.mjs','--resources-only'],root),('resources-verify',['node','scripts/sync-original-content.mjs','--resources-only','--verify-only'],root),('full-sync-verify',['node','scripts/sync-original-content.mjs','--verify-only'],root),('frontend-test',['npm','test'],root/'apps/web-client'),('frontend-typecheck',['npm','run','typecheck'],root/'apps/web-client'),('frontend-build',['npm','run','build'],root/'apps/web-client')]
records=[]
for name,cmd,cwd in commands:
 actual=['python3','-B','/tmp/noname-run-serialized.py',*cmd];t=time.time()
 with (out/(name+'.log')).open('w') as log:p=subprocess.run(actual,cwd=cwd,env=env,stdout=log,stderr=subprocess.STDOUT)
 records.append({'name':name,'argv':actual,'cwd':str(cwd),'exit':p.returncode,'seconds':time.time()-t,'CARGO_INCREMENTAL':'0','disk_free_bytes':__import__('shutil').disk_usage(root).free})
 (out/'commands.json').write_text(json.dumps(records,indent=2)+'\n');print(name,p.returncode,flush=True)
 if p.returncode:sys.exit(p.returncode)
