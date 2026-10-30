import fcntl,json,os,pathlib,shutil,subprocess,time,sys
root=pathlib.Path(__file__).resolve().parent
cwd=root.parent/'worktree'
env=dict(os.environ,CARGO_INCREMENTAL='0',CARGO_TARGET_DIR=str(root.parent/'target'),CARGO_BUILD_JOBS='2')
commands=[
 ('p2-green-nc10', ['cargo','test','--offline','-p','engine-native','--test','nc_rounds','nc10_','--','--nocapture'],{}),
 ('p2-full-native3', ['cargo','test','--offline','-p','engine-native-kernel','-p','engine-native-content','-p','engine-native','--no-fail-fast'],{}),
 *[(f'p2-strict-{p}',['cargo','check','--offline','-p',p,'--all-targets'],{'RUSTFLAGS':'-D warnings'}) for p in ['engine-native-kernel','engine-native-content','engine-native']],
 ('p2-wasm-native-privacy',['cargo','test','--offline','-p','engine-native-wasm','--test','nc03_view_projection','--test','nc03_actor_authorization','--test','nc03_legal_actions_export'],{}),
 ('p2-wasm-build',['wasm-pack','build','crates/engine-native-wasm','--target','nodejs','--release','--no-opt','--out-dir',str(root/'nc10-node-pkg')],{}),
 ('p2-wasm-smoke',['node','scripts/native_wasm_smoke.mjs'],{'PKG':str(root/'nc10-node-pkg')})
]
final_native = len(sys.argv) > 1 and sys.argv[1] == 'final-native'
if final_native:
 commands=[(name.replace('p2-', 'p2-final-', 1),command,extra) for name,command,extra in commands[:2]]
index_prefix='final-native' if final_native else 'recovery'
results=[]
for name,command,extra in commands:
 if shutil.disk_usage(cwd).free < 3*1024**3:
  results.append({'name':name,'command':command,'exit':None,'blocked':'free space below 3 GiB before queue'});break
 with open('/tmp/noname-phase4-heavy-command.lock','a') as lock:
  fcntl.flock(lock,fcntl.LOCK_EX)
  if shutil.disk_usage(cwd).free < 3*1024**3:
   results.append({'name':name,'command':command,'exit':None,'blocked':'free space below 3 GiB after lock'});break
  start=time.monotonic()
  with (root/(name+'.log')).open('w') as out:
   run=subprocess.run(command,cwd=cwd,env=dict(env,**extra),stdout=out,stderr=subprocess.STDOUT)
   record={'name':name,'command':command,'env':dict(CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2',CARGO_TARGET_DIR=env['CARGO_TARGET_DIR'],**extra),'exit':run.returncode,'locked_seconds':time.monotonic()-start}
   print(json.dumps(record),file=out,flush=True)
  results.append(record)
 (root/(index_prefix+'-commands.json')).write_text(json.dumps(results,indent=2)+'\n')
(root/(index_prefix+'-commands.json')).write_text(json.dumps(results,indent=2)+'\n')
(root/(index_prefix+'-done.json')).write_text(json.dumps({'completed':len(results),'planned':len(commands),'results':results},indent=2)+'\n')
