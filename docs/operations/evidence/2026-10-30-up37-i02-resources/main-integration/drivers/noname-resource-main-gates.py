import subprocess,os,json,time,pathlib,shutil
root=pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');out=root/'docs/operations/evidence/2026-10-30-up37-i02-resources/main-integration';env={**os.environ,'CARGO_INCREMENTAL':'0'}
cmds=[('fullpreflight',['python3','-B','scripts/preflight.py']),('sourcecaps-selftests',['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_source_lines.py']),('sourcecaps',['python3','-B','scripts/check-source-lines.py']),('links-all',['python3','-B','scripts/check_local_links.py','--all'])]
rec=[]
for name,cmd in cmds:
 actual=['python3','-B','/tmp/noname-run-serialized.py',*cmd];t=time.time()
 with (out/(name+'.log')).open('w') as log:p=subprocess.run(actual,cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT)
 rec.append({'name':name,'argv':actual,'cwd':str(root),'exit':p.returncode,'seconds':time.time()-t,'CARGO_INCREMENTAL':'0','disk_free_bytes':shutil.disk_usage(root).free});(out/'gate-commands.json').write_text(json.dumps(rec,indent=2)+'\n');print(name,p.returncode,flush=True)
