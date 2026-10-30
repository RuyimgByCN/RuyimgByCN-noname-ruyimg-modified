import os,subprocess,pathlib,json,hashlib,shutil,time,socket,urllib.request,sys
root=pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');out=root/'docs/operations/evidence/2026-10-30-up37-i02-resources/main-integration/retry-real-http';out.mkdir(exist_ok=True);runtime=pathlib.Path('/tmp/noname-resource-http-main-329');runtime.mkdir(exist_ok=False)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(n,x): (out/n).write_text(json.dumps(x,indent=2,ensure_ascii=False)+'\n')
assert sha(pathlib.Path('/tmp/noname-resource-server-329'))=='9ef4b49544bbd35640caf3abf240714af4cd3e0fc56a711c1f5ad502ce0d3489'
admission=json.loads((out.parents[1]/'parent-apply-admission.json').read_text());checks={p:{'actual':sha(root/p),'expected':h} for p,h in admission['applied_paths'].items()};assert all(v['actual']==v['expected'] for v in checks.values());save('seven-path-admission.json',checks)
products={str(p.relative_to(root/'apps/web-client/dist')):sha(p) for p in (root/'apps/web-client/dist').rglob('*') if p.is_file()};save('production-products-sha256.json',products)
shutil.copytree(root/'apps/web-client/dist',out/'happy/validated-dist');(out/'errors').mkdir()
# all generated module inputs, including JS glue and typings
modules={str(p.relative_to(root)):sha(p) for d in ['apps/web-client/src/worker/native-pkg','apps/web-client/src/wasm-pkg'] for p in (root/d).rglob('*') if p.is_file()};save('generated-module-inputs-before.json',modules)
with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
base=f'http://127.0.0.1:{port}';env={**os.environ,'CARGO_INCREMENTAL':'0','GAME_SERVER_ADDR':f'127.0.0.1:{port}','NONAME_ORIGINAL_CONTENT_ROOT':str(root/'content/original'),'NONAME_CONTENT_STORE_ROOT':str(runtime/'content-store'),'NONAME_EXTENSIONS_ROOT':str(runtime/'extensions'),'NONAME_NATIVE_PACKAGES_ROOT':str(root/'packages/extensions'),'NONAME_NATIVE_STORE_ROOT':str(runtime/'native-store'),'NONAME_NATIVE_ROOM_STORE':str(runtime/'rooms'),'NONAME_NATIVE_ROOMS':'true'}
for d in ['extensions','rooms']: (runtime/d).mkdir()
records=[];serverlog=(out/'rust-server.log').open('w');server=subprocess.Popen(['/tmp/noname-resource-server-329'],cwd=root,env=env,stdout=serverlog,stderr=subprocess.STDOUT)
info={'pid':server.pid,'binary':'/tmp/noname-resource-server-329','binary_sha256':sha(pathlib.Path('/tmp/noname-resource-server-329')),'build_head':'255bbfbd7f6916ee77bd29799826a225bd10871f','url':base,'env':{k:v for k,v in env.items() if k.startswith(('NONAME_','GAME_SERVER_'))},'rust_source_sha256':{str(p.relative_to(root)):sha(p) for p in (root/'apps/game-server/src').rglob('*.rs')},'no_rust_compilation':True};save('rust-process.json',info)
def run(name,args,cwd=root):
 t=time.time()
 with (out/(name+'.log')).open('w') as log:r=subprocess.run(args,cwd=cwd,env=env,stdout=log,stderr=subprocess.STDOUT)
 records.append({'name':name,'argv':args,'cwd':str(cwd),'exit':r.returncode,'seconds':time.time()-t});save('http-commands.json',records);assert r.returncode==0,(name,r.returncode)
try:
 for i in range(200):
  if server.poll() is not None:raise RuntimeError('server exited')
  try:
   if urllib.request.urlopen(base+'/health',timeout=1).read()==b'ok':break
  except Exception:time.sleep(.1)
 else:raise RuntimeError('server startup timeout')
 run('catalog-url-audit',['node','--import',str(root/'apps/web-client/scripts/ts-register.mjs'),'/tmp/noname-resource-url-audit.mjs',str(out)])
 run('helper-aux',['node','--import',str(root/'apps/web-client/scripts/ts-register.mjs'),'/tmp/noname-resource-helper-review.mjs',str(out)])
 run('browser-happy',['node','/tmp/noname-resource-browser-audio.mjs',str(root),str(out/'happy'),base])
 run('browser-errors',['node','/tmp/noname-resource-browser-errors.mjs',str(root),str(out/'errors'),str(out/'happy/validated-dist'),base])
 catalog=json.loads((root/'content/original/catalog.json').read_text());manifest=json.loads((root/'content/original/asset-manifest.json').read_text());assets={a['path']:a for a in manifest['assets']};http=[]
 def get(relative):
  with urllib.request.urlopen(base+'/original-content/'+relative) as r:data=r.read();row={'path':relative,'status':r.status,'bytes':len(data),'content_type':r.headers.get('Content-Type'),'sha256':hashlib.sha256(data).hexdigest()}
  assert data==(root/'content/original'/relative).read_bytes(),relative
  http.append(row);return row
 get('catalog.json');get('asset-manifest.json')
 for row in json.loads((out/'happy/browser-result.json').read_text())['matrix']:
  ch=next(c for c in catalog['characters'] if c['name']==row['name'])
  if ch['portrait']:get(ch['portrait'])
  for audio in row['audio']:
   relative=audio['src'].split('/original-content/')[1];h=get(relative);asset=assets[relative.removeprefix('assets/')];assert h['sha256']==audio['sha256']==asset['sha256'];assert h['bytes']==audio['byteLength']==asset['size']
 save('real-http-byte-evidence.json',{'base':base,'pid':server.pid,'requests':http,'audio_count':sum(r['path'].startswith('assets/audio/') for r in http),'actual_rust_servedir':True})
 after={str(p.relative_to(root)):sha(p) for d in ['apps/web-client/src/worker/native-pkg','apps/web-client/src/wasm-pkg'] for p in (root/d).rglob('*') if p.is_file()};save('generated-module-inputs-after.json',after);assert modules==after
finally:
 server.terminate()
 try:server.wait(timeout=10)
 except subprocess.TimeoutExpired:server.kill();server.wait()
 serverlog.close();info['stop_exit']=server.returncode;info['stopped']=True;info['runtime_store_bytes']=sum(p.stat().st_size for p in runtime.rglob('*') if p.is_file());shutil.rmtree(runtime);info['runtime_store_removed']=not runtime.exists();save('rust-process.json',info)
print('real HTTP and browser acceptance passed')
