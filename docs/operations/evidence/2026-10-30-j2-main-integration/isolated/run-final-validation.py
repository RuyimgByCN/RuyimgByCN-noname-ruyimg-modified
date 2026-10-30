import json,pathlib,shutil,subprocess,sys,time
ROOT=pathlib.Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-j2-llgkp_le/worktree')
E=ROOT.parent/'evidence'/'theft-correction'
SER=['python3','-B','/tmp/noname-run-serialized.py','python3','-B',str(E/'space-guard.py')]
TARGET='/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-nc09-pe1axnwg/target-nc09'
GATE='/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-nc09-pe1axnwg/target-gate'
records=[]
def run(label,args,heavy=True,cwd=ROOT):
 command=(SER if heavy else [])+args;start=time.time()
 with (E/label).open('w') as log:r=subprocess.run(command,cwd=cwd,stdout=log,stderr=subprocess.STDOUT)
 records.append({'log':label,'command':command,'exit':r.returncode,'elapsed_seconds_including_wait':round(time.time()-start,3)})
 (E/'final-commands.json').write_text(json.dumps(records,indent=2)+'\n')
 if r.returncode:sys.exit(r.returncode)
def env(target):return ['env','CARGO_INCREMENTAL=0','CARGO_BUILD_JOBS=2','CARGO_TARGET_DIR='+target]
run('20-final-native-matrix.log',env(TARGET)+['J2_CROSS_NATIVE_OUT='+str(E/'native-matrix.json'),'cargo','test','--offline','-p','engine-native-wasm','--test','adr0786_j2_cross_target','--','--nocapture'])
run('10-final-native-three.log',env(TARGET)+['cargo','test','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native','--no-fail-fast'])
run('11-final-strict.log',env(GATE)+['RUSTFLAGS=-Dwarnings','cargo','check','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native','--all-targets'])
run('12-final-wasm-build.log',env(TARGET)+['wasm-pack','build','crates/engine-native-wasm','--target','web','--release','--no-opt','--out-dir','../../apps/web-client/src/worker/native-pkg'])
shutil.copytree(ROOT/'apps/web-client/src/worker/native-pkg',E/'wasm-pkg',dirs_exist_ok=True)
run('13-final-actual-wasm-matrix.log',['node','apps/web-client/scripts/adr0786-j2-cross-target.mjs',str(E/'native-matrix.json'),str(E/'wasm-pkg'),str(E/'wasm-matrix-result.json')],False)
run('14-final-theft-sdk.log',['node','--import','./scripts/ts-register.mjs','scripts/adr0786-j2-theft-sdk-smoke.mjs',str(E/'native-matrix.json')],False,ROOT/'apps/web-client')
run('15-final-author-sdk-lifecycle.log',['node','--import','./scripts/ts-register.mjs','scripts/adr0786-j2-active-smoke.mjs',str(E.parent/'author-pack')],False,ROOT/'apps/web-client')
run('16-final-worker-smoke.log',['npm','run','smoke:native'],False,ROOT/'apps/web-client')
run('17-final-workspace.log',env(TARGET)+['cargo','test','--offline','--workspace','--no-fail-fast'])
run('18-final-conformance.log',env(TARGET)+['cargo','run','--offline','-p','conformance'])
run('19-final-preflight.log',['env','CARGO_INCREMENTAL=0','CARGO_BUILD_JOBS=2','python3','-B','scripts/preflight.py'])
(E/'final-validation-complete.json').write_text(json.dumps({'complete':True,'commands':len(records),'exits':[r['exit'] for r in records]},indent=2)+'\n')
