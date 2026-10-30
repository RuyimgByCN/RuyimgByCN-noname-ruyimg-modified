from pathlib import Path
import os,json,subprocess,socket,time,uuid,blake3,urllib.request,copy,hashlib
ROOT=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');E=Path(__file__).resolve().parent;P=Path(json.loads((E/'product-root.json').read_text())['path']);out=P/'referee-external';out.mkdir(exist_ok=True);store=out/'rooms';store.mkdir(exist_ok=True)
original=json.loads((P/'real-ws/after-restart.json').read_text());compact=lambda obj:json.dumps(obj,ensure_ascii=False,separators=(',',':')).encode();digest=lambda data:blake3.blake3(data).hexdigest()
assert digest(compact(original['state']))==original['checksum'],'Python serializer must equal actual typed Rust state checksum before constructing fixture'
record=copy.deepcopy(original);record['room_id']=989;record['commit_id']=str(uuid.uuid4());token=str(uuid.uuid4());record['state']['members'].append({'connection_id':90001,'player_id':0,'seat':0,'role':'Referee','reconnect_token':token});record['checksum']=digest(compact(record['state']));candidate=json.dumps(record,ensure_ascii=False,indent=2).encode();journal={'version':1,'room_id':989,'commit_id':record['commit_id'],'candidate_hash':digest(candidate),'previous':None,'phase':'Committed','checksum':''};journal['checksum']=digest(compact(journal));(store/'989.json').write_bytes(candidate);(store/'989.json.journal').write_text(json.dumps(journal,ensure_ascii=False,indent=2))
(out/'trusted-seed-provenance.json').write_text(json.dumps({'scope':'trusted private test-store Referee membership, same canonical committed journal format; no public role-grant endpoint','originalActualRecordSha256':hashlib.sha256((P/'real-ws/after-restart.json').read_bytes()).hexdigest(),'changedStateFields':['members: append Referee only'],'allOtherStateFieldsExact':all(record['state'][k]==original['state'][k] for k in original['state'] if k!='members'),'archiveWalTimeoutPreserved':True,'noMetadataRemoved':True,'seedRoom':989},indent=2)+'\n')
s=socket.socket();s.bind(('127.0.0.1',0));port=s.getsockname()[1];s.close();base='http://127.0.0.1:'+str(port)
for d in ['native-store','legacy-store','legacy-extensions']:(out/d).mkdir(exist_ok=True)
env=dict(os.environ,GAME_SERVER_ADDR='127.0.0.1:'+str(port),NONAME_NATIVE_ROOMS='1',NONAME_NATIVE_ROOM_PACKAGES='',NONAME_NATIVE_PACKAGES_ROOT=str(ROOT/'packages/extensions'),NONAME_NATIVE_STORE_ROOT=str(out/'native-store'),NONAME_NATIVE_ROOM_STORE=str(store),NONAME_ORIGINAL_CONTENT_ROOT=str(ROOT/'content/original'),NONAME_CONTENT_STORE_ROOT=str(out/'legacy-store'),NONAME_EXTENSIONS_ROOT=str(out/'legacy-extensions'))
log=(out/'server.log').open('w');server=subprocess.Popen([str(P/'artifacts/game-server330-main')],cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT)
try:
 for i in range(200):
  try:
   with urllib.request.urlopen(base+'/native/packages') as response:loaded=json.load(response)
   break
  except Exception:assert server.poll() is None,'actual server exited';time.sleep(.05)
 else:raise AssertionError('readiness')
 before=json.loads((store/'989.json').read_text())['state'];res=subprocess.run(['node',str(E/'referee-command.mjs'),base,token,str(out/'report.json')],cwd=ROOT,env=env);assert res.returncode==0
 after=json.loads((store/'989.json').read_text())['state'];assert before['snapshot_hex']==after['snapshot_hex'];assert before['wal']==after['wal'];assert before['timeout']==after['timeout'];(out/'post-command-state.json').write_text(json.dumps({'passed':True,'snapshotExact':True,'walExact':True,'timeoutExact':True,'nativeImportedCount':len(loaded)},indent=2)+'\n')
finally:
 if server.poll() is None:server.terminate()
 try:server.wait(timeout=10)
 except subprocess.TimeoutExpired:server.kill();server.wait()
 log.close();(out/'owned-cleanup.json').write_text(json.dumps({'pid':server.pid,'exit':server.returncode,'waited':True,'port':port},indent=2)+'\n')
