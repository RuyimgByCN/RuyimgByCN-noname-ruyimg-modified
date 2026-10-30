import pathlib,subprocess,json,hashlib,os,shutil,time
r=pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');o=r/'docs/operations/evidence/2026-10-30-up37-i02-resources/main-integration'
def save(n,x):(o/n).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
for name in ['noname-resource-main-run.log','noname-resource-main-gates.log']:
 p=pathlib.Path('/tmp')/name
 if p.exists():shutil.copy2(p,o/name)
records=[]
for name in ['commands.json','gate-commands.json','http-commands.json','retry-real-http/http-commands.json']:
 records.extend([{**x,'index':name} for x in json.loads((o/name).read_text())])
for name,code in [('real-http-stage.log',1),('real-http-retry-stage.log',1),('real-http-retry-02-stage.log',0),('finalize.log',0),('hash-after.log',0)]:records.append({'index':name,'exit':code,'wrapper':'/tmp/noname-run-serialized.py','details':'Exact argv, exit and locked duration are preserved in the final JSON line of this log.'})
save('command-index.json',records)
processes=subprocess.check_output(['ps','-axo','pid,ppid,command'],text=True).splitlines();owned=[p for p in processes if ('Google Chrome' in p and 'main-integration/' in p) or ('/tmp/noname-resource-server-329' in p and 'python' not in p and 'bash' not in p)]
assert not owned,owned
assert subprocess.check_output(['git','diff','--cached','--name-only'],cwd=r)==b''
assert subprocess.check_output(['git','diff','--name-only','--','apps/web-client/dist'],cwd=r)==b''
save('final-audit.json',{'owned_active_chrome_or_servers':owned,'initial_failed_runner_orphan_chrome_pid':34152,'initial_orphan_terminated_and_profile_removed':True,'native_room_stores_removed':not pathlib.Path('/tmp/noname-resource-http-main-329').exists(),'source_target_created':False,'borrowed_binary_and_serialization_wrapper_preserved':True,'A330_cache_untouched':True,'no_staged_files':True,'tracked_dist_exact_HEAD':True,'CARGO_INCREMENTAL':'0','disk_free_bytes':shutil.disk_usage(r).free,'retained_product_bytes':sum(p.stat().st_size for p in (o/'retry-real-http/happy/validated-dist').rglob('*') if p.is_file())})
# Remove only own temporary drivers/logs after keeping the exact sources and outputs.
shutil.copy2('/tmp/noname-resource-audit.py',o/'drivers/noname-resource-audit.py')
own_names=['noname-resource-main-hashes.py','noname-resource-main-run.py','noname-resource-url-audit.mjs','noname-resource-browser-audio.mjs','noname-resource-browser-errors.mjs','noname-resource-helper-review.mjs','noname-resource-real-http.py','noname-resource-real-http-retry.py','noname-resource-main-gates.py','noname-resource-finalize.py','noname-resource-audit.py','noname-resource-main-run.log','noname-resource-main-gates.log']
for name in own_names:
 p=pathlib.Path('/tmp')/name
 if p.is_file():p.unlink()
inventory={str(p.relative_to(o)):hashlib.sha256(p.read_bytes()).hexdigest() for p in o.rglob('*') if p.is_file() and p.name not in ['artifact-sha256.json','audit.log']};save('artifact-sha256.json',inventory)
print(json.dumps({'artifact_count':len(inventory),'no_staged_files':True,'owned_processes':0,'disk_free_bytes':shutil.disk_usage(r).free}))
