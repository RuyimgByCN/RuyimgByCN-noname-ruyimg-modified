from pathlib import Path
import os,sys,json,socket,subprocess,time,urllib.request,hashlib
ROOT=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');E=Path(__file__).resolve().parent;P=Path(json.loads((E/'product-root.json').read_text())['path']);out=P/'resources';out.mkdir(exist_ok=True)
s=socket.socket();s.bind(('127.0.0.1',0));port=s.getsockname()[1];s.close();base='http://127.0.0.1:'+str(port)
for d in ['legacy-extensions','native-store','rooms','legacy-store','happy','errors']:(out/d).mkdir(exist_ok=True)
env=dict(os.environ,GAME_SERVER_ADDR='127.0.0.1:'+str(port),NONAME_NATIVE_ROOMS='1',NONAME_NATIVE_ROOM_PACKAGES='',NONAME_NATIVE_PACKAGES_ROOT=str(ROOT/'packages/extensions'),NONAME_NATIVE_STORE_ROOT=str(out/'native-store'),NONAME_NATIVE_ROOM_STORE=str(out/'rooms'),NONAME_ORIGINAL_CONTENT_ROOT=str(ROOT/'content/original'),NONAME_CONTENT_STORE_ROOT=str(out/'legacy-store'),NONAME_EXTENSIONS_ROOT=str(out/'legacy-extensions'))
log=(out/'server.log').open('w');server=subprocess.Popen([str(P/'artifacts/game-server330-main')],cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT)
try:
 for i in range(200):
  try:
   with urllib.request.urlopen(base+'/native/packages') as response:packages=json.load(response)
   break
  except Exception:
   assert server.poll() is None,'actual server exited';time.sleep(.05)
 else:raise AssertionError('server never ready')
 assert len(packages)==41,len(packages)
 gets=[]
 for suffix in ['catalog.json','asset-manifest.json','assets/audio/die/star_dongyun.mp3']:
  with urllib.request.urlopen(base+'/original-content/'+suffix) as response:body=response.read();status=response.status
  disk=(ROOT/'content/original'/suffix).read_bytes();assert body==disk
  gets.append({'path':suffix,'httpStatus':status,'bytes':len(body),'sha256':hashlib.sha256(body).hexdigest(),'exactDiskBytes':True})
 (out/'actual-rust-http.json').write_text(json.dumps({'base':base,'pid':server.pid,'nativeImportedCount':len(packages),'packages':packages,'gets':gets,'nativeRoomSelectedPacks':[]},indent=2)+'\n')
 for name,args in [('happy',[str(ROOT),str(out/'happy'),base]),('errors',[str(ROOT),str(out/'errors'),str(P/'ui-dist'),base])]:
  helper='noname-resource-browser-'+('audio' if name=='happy' else 'errors')+'.mjs'
  with (out/(name+'.log')).open('w') as childlog:res=subprocess.run(['node',str(E/helper),*args],cwd=ROOT,env=env,stdout=childlog,stderr=subprocess.STDOUT)
  print('resource '+name+' exit='+str(res.returncode),flush=True);assert res.returncode==0,(out/(name+'.log')).read_text()
finally:
 if server.poll() is None:server.terminate()
 try:server.wait(timeout=10)
 except subprocess.TimeoutExpired:server.kill();server.wait()
 log.close();(out/'cleanup.json').write_text(json.dumps({'ownedServerPid':server.pid,'exit':server.returncode,'waited':True,'user7700Untouched':True},indent=2)+'\n')
