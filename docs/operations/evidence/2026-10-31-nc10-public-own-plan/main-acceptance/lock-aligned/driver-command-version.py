import os,sys,json,hashlib,subprocess,datetime,time,shutil,tomllib
from pathlib import Path
E=Path('/tmp/nc10-main-lock-repair-0425dc78');R=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');OLD=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/nc10-main-own-sfobj6i8');TARGET=Path('/tmp/nc27-main-functional-458ed333/cargo-target')
ENV={**os.environ,'CARGO_TARGET_DIR':str(TARGET),'CARGO_BUILD_JOBS':'2','CARGO_INCREMENTAL':'0','CARGO_NET_OFFLINE':'true','GIT_OPTIONAL_LOCKS':'0','PYTHONDONTWRITEBYTECODE':'1'}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(n,x):(E/n).write_text(json.dumps(x,indent=2)+'\n')
def binding():
 entry=json.load(open(E/'entry.json'));assert sha(R/'Cargo.lock')==entry['MainCargoLockBefore'];assert all(sha(R/f)==h for f,h in entry['source6Before'].items());assert sha(E/'probe/src/main.rs')==entry['probeSourceSha256'];return {'source6':entry['source6Before'],'MainCargoLock':sha(R/'Cargo.lock'),'probeSource':sha(E/'probe/src/main.rs'),'privateLock':sha(E/'probe/Cargo.lock')}
def run(name,argv):
 before=binding();assert shutil.disk_usage(E).free>=3*1024**3;actual=['python3','-B','/tmp/noname-run-serialized.py',*argv];start=time.monotonic()
 save(name+'.running.json',{'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'argv':actual,'binding':before})
 with (E/(name+'.stdout.log')).open('wb') as out,(E/(name+'.stderr.log')).open('wb') as err:p=subprocess.run(actual,cwd=R,env=ENV,stdout=out,stderr=err)
 after=binding();x={'name':name,'argv':argv,'actualArgv':actual,'cwd':str(R),'exit':p.returncode,'seconds':time.monotonic()-start,'environment':{k:ENV[k] for k in ['CARGO_TARGET_DIR','CARGO_BUILD_JOBS','CARGO_INCREMENTAL','CARGO_NET_OFFLINE','GIT_OPTIONAL_LOCKS']},'source6AndLockBefore':before,'source6AndLockAfter':after,'stdoutSha256':sha(E/(name+'.stdout.log')),'stderrSha256':sha(E/(name+'.stderr.log')),'freeDiskBytesAfter':shutil.disk_usage(E).free};save(name+'.result.json',x);print(json.dumps({'name':name,'exit':p.returncode,'seconds':x['seconds']}),flush=True);return p.returncode

def verify_lock():
 main=tomllib.loads((E/'MainCargo.lock.before').read_text())['package'];private=tomllib.loads((E/'probe/Cargo.lock').read_text())['package']
 def key(p):return (p['name'],p['version'],p.get('source'),p.get('checksum'))
 allowed={key(p) for p in main if p.get('source')};chosen=[p for p in private if p.get('source')];drift=[p for p in chosen if key(p) not in allowed]
 metadata=json.loads((E/'metadata.stdout.log').read_text().splitlines()[0]);meta_external={(p['name'],p['version'],p['source']) for p in metadata['packages'] if p.get('source')};lock_external={(p['name'],p['version'],p['source']) for p in chosen};assert meta_external==lock_external
 locals_bad=[p for p in private if not p.get('source') and p['name']!='nc10-main-lock-acceptance-probe' and key(p) not in {key(q) for q in main}]
 x={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'mainLockSha256':sha(E/'MainCargo.lock.before'),'privateLockSha256':sha(E/'probe/Cargo.lock'),'externalCount':len(chosen),'everyExternalExactMemberOfMain':not drift,'metadataExternalSetEqualsPrivateLock':meta_external==lock_external,'externalPackages':[{'name':p['name'],'version':p['version'],'source':p['source'],'checksum':p['checksum']} for p in chosen],'externalDrift':drift,'unexpectedLocalChanges':locals_bad,'newRootOnly':[p for p in private if p['name']=='nc10-main-lock-acceptance-probe'],'mainPackagesPrunedCount':len(main)-len(private)+1};save('lock-verification.json',x);assert not drift and not locals_bad,x;shutil.copy2(E/'probe/Cargo.lock',E/'privateCargo.lock.verified');print(json.dumps({'lockVerified':True,'externalCount':len(chosen),'privateLockSha256':x['privateLockSha256']}),flush=True)
if __name__=='__main__':
 mode=sys.argv[1]
 if mode=='metadata':
  code=run('metadata',['cargo','metadata','--offline','--format-version','1','--manifest-path',str(E/'probe/Cargo.toml')]);assert code==0;verify_lock()
 elif mode=='probe':
  x=json.load(open(E/'lock-verification.json'));assert x['everyExternalExactMemberOfMain'];assert sha(E/'probe/Cargo.lock')==x['privateLockSha256']
  for name,args in [('native-generate',[]),('canonical-compare',['compare'])]:
   code=run(name,['cargo','run','--offline','--locked','--manifest-path',str(E/'probe/Cargo.toml'),'--',str(E),*args])
   if code:sys.exit(code)
 elif mode=='seal':
  binding();names=['park-0','park-1','own-cancel','shan-auto','shan-direct','shan-cancel','shan-pass'];results=[]
  for name in names:
   fresh=E/f'native-{name}.ngsa';wasm=E/f'wasm-{name}.ngsa';old=E/f'old-native/native-{name}.ngsa';assert fresh.read_bytes()==wasm.read_bytes()==old.read_bytes(),name
   results.append({'case':name,'newNativeSha256':sha(fresh),'preservedActualWasmSha256':sha(wasm),'oldCrossDependencyNativeSha256':sha(old),'allBytesEqual':True})
  new_json=json.load(open(E/'native-probe-results.json'));old_json=json.load(open(E/'old-native/native-probe-results.json'));assert new_json==old_json
  assert (E/'native-shan-direct.ngsa').read_bytes()==(E/'native-shan-auto.ngsa').read_bytes();assert (E/'shan.ngsa').read_bytes()==(E/'old-native/shan.ngsa').read_bytes();assert (E/'shan-choice.txt').read_bytes()==(E/'old-native/shan-choice.txt').read_bytes()
  exe=TARGET/'debug/nc10-main-lock-acceptance-probe';entry=json.load(open(E/'entry.json'));assert sha(Path(entry['oldBinary']['path']))==entry['oldBinary']['sha256'];assert sha(OLD/'artifact-sha256.json')==entry['oldArtifactIndexSha256'];assert sha(Path(json.load(open(OLD/'manifest.json'))['finalReport']))==entry['oldFinalReportSha256']
  binary=E/'binary/nc10-main-lock-acceptance-probe';binary.parent.mkdir(exist_ok=True);shutil.copy2(exe,binary)
  commands=[json.load(open(E/(n+'.result.json'))) for n in ['metadata','native-generate','canonical-compare']];assert all(c['exit']==0 for c in commands)
  running=[l for c in commands for l in (E/(c['name']+'.stderr.log')).read_text().splitlines() if 'Running `' in l];assert len(running)==2 and all(str(exe) in l for l in running)
  save('commands.json',commands);save('comparison.json',{'cases':results,'fullNativeReplyStateHashEventJsonEqualOld':True,'directAutoShanFullArchiveEqual':True,'sourceHarnessUnchanged':True,'normalization':'None for native/WASM/old output equality; unchanged source explicitly validates restore auth_generation+1 only.','actualExecutedBinary':str(exe),'actualRunningLines':running,'binarySha256':sha(exe),'binaryCopySha256':sha(binary),'oldBinaryUnchanged':True,'old714AndReportUnchanged':True})
  save('manifest.json',{'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS exact Main-lock native probe and canonical7 comparisons','bindingAfter':binding(),'lockVerificationSha256':sha(E/'lock-verification.json'),'comparisonSha256':sha(E/'comparison.json'),'commandsSha256':sha(E/'commands.json'),'probeSourceSha256':sha(E/'probe/src/main.rs'),'probeManifestSha256':sha(E/'probe/Cargo.toml'),'driverSourceSha256':sha(E/'driver.py'),'actualExecutedBinary':str(exe),'binarySha256':sha(exe),'source6BeforeAfterExact':True,'MainLockBeforeAfterExact':True,'oldEvidenceUnchanged':True,'originalLimitationRetained':'Original21 external-version drift + sidecheck/index-update provenance is preserved. Old probe is cross-dependency compatibility evidence; this rerun supplies Main-lock consumer proof.','noMainOrIndexWrites':True,'noStagedFilesAtOriginalSeal':True})
  files={str(f.relative_to(E)):{'sha256':sha(f),'bytes':f.stat().st_size} for f in E.rglob('*') if f.is_file() and f.name!='artifact-sha256.json'};save('artifact-sha256.json',files);print(json.dumps({'status':'PASS','cases':len(results),'binarySha256':sha(exe),'manifestSha256':sha(E/'manifest.json'),'artifactIndexSha256':sha(E/'artifact-sha256.json')}),flush=True)
