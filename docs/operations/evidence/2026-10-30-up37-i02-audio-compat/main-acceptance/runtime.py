import pathlib,json,hashlib,subprocess,socket,time,shutil
from accept import ROOT,OUT,ENV,save,sha
paths=json.loads((OUT/'paths.json').read_text());private=pathlib.Path(paths['private']);runtime=private/'runtime-store';runtime.mkdir();binary=pathlib.Path('/tmp/noname-resource-server-329');wk_source=pathlib.Path('/tmp/noname-i02-all-codecs.LXnAxI/wk-probe');assert sha(binary)=='9ef4b49544bbd35640caf3abf240714af4cd3e0fc56a711c1f5ad502ce0d3489';assert sha(wk_source)=='e68c7cdede30d50d087ba704f5b1200c677d30a65d6aace5a47e3730f92d2108';(private/'bin').mkdir();wk_binary=private/'bin/wk-probe';shutil.copy2(wk_source,wk_binary)
with socket.socket() as sock:sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
assert port!=7700;base=f'http://127.0.0.1:{port}'
for d in ['empty-extensions','empty-packages']:(runtime/d).mkdir()
env={**ENV,'GAME_SERVER_ADDR':f'127.0.0.1:{port}','NONAME_ORIGINAL_CONTENT_ROOT':str(ROOT/'content/original'),'NONAME_CONTENT_STORE_ROOT':str(runtime/'legacy-store'),'NONAME_NATIVE_STORE_ROOT':str(runtime/'native-store'),'NONAME_NATIVE_ROOM_STORE':str(runtime/'rooms'),'NONAME_EXTENSIONS_ROOT':str(runtime/'empty-extensions'),'NONAME_NATIVE_PACKAGES_ROOT':str(runtime/'empty-packages'),'NONAME_NATIVE_ROOMS':'false'}
serverlog=(OUT/'rust-server.log').open('x');server=subprocess.Popen([str(binary)],cwd=ROOT,env=env,stdout=serverlog,stderr=subprocess.STDOUT);wk=None;info={'pid':server.pid,'base':base,'binary':str(binary),'binary_sha256':sha(binary),'source_build_head':'255bbfbd7f6916ee77bd29799826a225bd10871f','source_versions':'329/105/758/WS5 retained static ServeDir only; not current330WS/room acceptance','actual_frontend':'main a1f7250815eee803d931301ab46207719cb9a15b + reviewedcompat14','env':{k:v for k,v in env.items() if k.startswith(('NONAME_','GAME_SERVER_'))},'wk_binary':str(wk_binary),'wk_sha256':sha(wk_binary),'no_compile':True};save('rust-runtime.json',info);children=[]
try:
 for i in range(100):
  if server.poll() is not None:raise RuntimeError('retained Rust server exited')
  try:
   import urllib.request
   if urllib.request.urlopen(base+'/health',timeout=1).read()==b'ok':break
  except Exception:time.sleep(.05)
 else:raise RuntimeError('Rust startup timeout')
 selection=json.loads((OUT/'selection.json').read_text());http=[]
 for relative in ['catalog.json','asset-manifest.json']:
  with urllib.request.urlopen(base+'/original-content/'+relative,timeout=5) as response:data=response.read();row={'path':relative,'status':response.status,'bytes':len(data),'mime':response.headers.get('Content-Type'),'sha256':hashlib.sha256(data).hexdigest()}
  assert data==(ROOT/'content/original'/relative).read_bytes();http.append(row)
 for a in selection['backing']+selection['allTen']:
  with urllib.request.urlopen(base+'/original-content/assets/'+a['path'],timeout=5) as response:data=response.read();row={'path':a['path'],'status':response.status,'bytes':len(data),'mime':response.headers.get('Content-Type'),'sha256':hashlib.sha256(data).hexdigest()}
  assert row['status']==200 and row['bytes']==a['size'] and row['sha256']==a['sha256'];http.append(row)
 save('http-identities.json',http)
 with (OUT/'chrome-vue.log').open('x') as log:p=subprocess.run(['node',str(OUT/'vue-e2e.mjs'),base],cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=200)
 children.append({'name':'actual-production-Vue','argv':['node',str(OUT/'vue-e2e.mjs'),base],'exit':p.returncode});save('runtime-commands.json',children);assert p.returncode==0,(OUT/'chrome-vue.log').read_text()
 # Existing retained helper and decoder functions; no original raw decode repeated.
 common=(OUT/'pcm-common.js').read_text();probe=common+'''
async function runProbe(config,emit){
 const report=row=>emit(JSON.stringify(row));report({event:'runtime',userAgent:navigator.userAgent,mode:'current Main derived five HTTP/full PCM/HTML only; original raw not re-decoded'});
 for(const asset of config.assets){const decoded=await fetchDecode(asset,config.base);const play=await playMuted(asset,config.base);report({event:'result',path:asset.path,http:decoded.http,statistics:decoded.statistics,play});}
 report({event:'done',count:config.assets.length,stopped:false});
}
''';(OUT/'probe.js').write_text(probe);save('wk-config.json',{'base':base+'/original-content','assets':selection['selected'],'probeSHA':sha(OUT/'probe.js')})
 with (OUT/'wk-driver.log').open('x') as log:wk=subprocess.Popen([str(wk_binary),str(OUT)],cwd=OUT,env=env,stdout=log,stderr=subprocess.STDOUT);info['wk_pid']=wk.pid;save('rust-runtime.json',info);code=wk.wait(timeout=150)
 children.append({'name':'retained-WebKit-five','argv':[str(wk_binary),str(OUT)],'exit':code});save('runtime-commands.json',children);assert code==0,code
 rows=[json.loads(line) for line in (OUT/'wk.jsonl').read_text().splitlines()];wkrows=[r for r in rows if r.get('event')=='result'];assert len(wkrows)==5;assert any(r.get('event')=='done' and r['count']==5 for r in rows)
 prior_path=pathlib.Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-audio-compat-o2uoxbq1/evidence/browser/wk.jsonl');prior=[json.loads(line) for line in prior_path.read_text().splitlines()];prior_results=[r for r in prior if r.get('event')=='result'];chrome=json.loads((OUT/'chrome-vue-result.json').read_text());checks=[]
 for i,(w,c) in enumerate(zip(wkrows,chrome['decoded'])):
  assert w['path']==c['path']==selection['selected'][i]['path'];assert w['http']['sha256']==c['sha256'];assert w['play']['ok'] and w['play']['final']['ended'];a=w['statistics'];b=prior_results[i]['candidate']['statistics'];assert a['frames']==b['frames'] and a['sampleRate']==b['sampleRate']==48000 and a['channels']==b['channels']==2;assert [r['sha256'] for r in a['channelStatistics']]==[r['sha256'] for r in b['channelStatistics']];assert c['statistics']['frames']-a['frames']==2112;checks.append({'path':w['path'],'main_WK_full_PCM_equal_prior_derived':True,'prior_WK_raw_derived_equality_inherited_not_redecoded':True,'WK_ended':True,'Chrome_Vue_ended':c['play']['ended'],'chrome_extra_frames':2112,'chrome_extra_ms_at48k':44})
 save('main-media-acceptance.json',{'passed':True,'actualMainProductionVue':True,'role':selection['character']['id'],'Vue_m4a_count':5,'raw_audio_count':9652,'Chrome_full_decode_controls_ended':5,'WK_full_decode_controls_ended':5,'Vue_real404_error_recovery_and_character_isolation':True,'HTTP_identity_count':len(http),'checks':checks,'original_raw5_not_redecoded':True,'private_original_WK_PCM_proof':str(prior_path),'cross_decoder_PCM_equality_claimed':False,'OS_HID_or_physical_listening_claimed':False})
finally:
 if wk and wk.poll() is None:wk.terminate();wk.wait(timeout=10)
 server.terminate()
 try:server.wait(timeout=10)
 except subprocess.TimeoutExpired:server.kill();server.wait()
 serverlog.close();shutil.rmtree(runtime);info['server_exit']=server.returncode;info['server_waited']=True;info['wk_exit']=wk.returncode if wk else None;info['stores_removed']=not runtime.exists();save('rust-runtime.json',info)
print('actual Main Vue5 ended/decode/error recovery; retained WK5 ended/fullPCM; no raw5 decode repeated')
