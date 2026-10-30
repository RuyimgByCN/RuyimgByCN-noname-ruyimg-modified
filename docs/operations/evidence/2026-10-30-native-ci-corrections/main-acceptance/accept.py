import pathlib,os,sys,json,hashlib,subprocess,time,shutil,datetime,re
ROOT=pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');OUT=ROOT/'docs/operations/evidence/2026-10-30-native-ci-corrections/main-acceptance';TARGET='/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-f7-stage3.czy5e9gq/target';PHASE=ROOT/'docs/operations/evidence/2026-10-30-adr0787-timeout/main-acceptance/phase2'
ENV={**os.environ,'GIT_OPTIONAL_LOCKS':'0','GIT_NO_LAZY_FETCH':'1','CARGO_INCREMENTAL':'0','CARGO_BUILD_JOBS':'2','CARGO_TARGET_DIR':TARGET,'CARGO_NET_OFFLINE':'true'}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(n,x):(OUT/n).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def snap():
 a=json.loads((OUT.parent/'parent-source-admission.json').read_text());old=json.loads((PHASE.parents[1]/'parent-source-admission.json').read_text());remain={p:h for p,h in old['runtime_paths_applied'].items() if p not in a['paths']};assert len(remain)==50
 protect=['.github/actions/prepare-original-content/action.yml','.github/actions/prepare-original-content/liuli-longxue.patch','scripts/sync-original-content.mjs','content/original/asset-manifest.json','content/original/catalog.json','apps/web-client/scripts/build-wasm.mjs','scripts/cross-target-hash.sh']+list(json.loads((PHASE/'frozen-extra-sha256.json').read_text()))
 state={'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'index':subprocess.check_output(['git','diff','--cached','--name-only'],cwd=ROOT,text=True).splitlines(),'source13':{p:sha(ROOT/p) for p in a['paths']},'A_remaining50':{p:sha(ROOT/p) for p in remain},'protected':{p:sha(ROOT/p) for p in protect},'guard':sha(ROOT/'.pi-lens.json'),'guard_value':json.loads((ROOT/'.pi-lens.json').read_text()),'native_modules':{str(p.relative_to(ROOT)):sha(p) for p in (ROOT/'apps/web-client/src/worker/native-pkg').rglob('*') if p.is_file()},'legacy_modules':{str(p.relative_to(ROOT)):sha(p) for d in ['apps/web-client/src/wasm-pkg','apps/web-client/public/pkg'] for p in (ROOT/d).rglob('*') if p.is_file()},'frozen_content_diff_sha':hashlib.sha256(subprocess.check_output(['git','diff','HEAD','--','content/original','packages/extensions'],cwd=ROOT)).hexdigest(),'tracked_dist_diff_sha':hashlib.sha256(subprocess.check_output(['git','diff','HEAD','--','apps/web-client/dist'],cwd=ROOT)).hexdigest()}
 assert state['source13']==a['paths'];assert state['A_remaining50']==remain;assert state['head']==a['base'];assert not state['index'];g=state['guard_value'];assert g['autofix']['enabled']==g['format']['enabled']==g['actionableWarnings']['autoFix']['enabled']==False
 expected=json.loads((PHASE/'generated-diff.json').read_text())['after'];assert all(expected[p]==h for p,h in state['native_modules'].items())
 return state
COMMANDS={
 'tcp':(['cargo','test','--offline','-p','game-server','--bin','game-server','native_ws_e2e::native_extensions_ws_slow_spectator_does_not_block_commands','--','--exact','--nocapture'],ROOT),
 'roster':(['cargo','test','--offline','-p','game-server','--test','nc09_native_actor','nc09_native_actor_join_command_snapshot','--','--exact','--nocapture'],ROOT),
 'workspace':(['cargo','test','--offline','--workspace'],ROOT),
 'all-targets':(['cargo','check','--offline','--workspace','--all-targets'],ROOT),
 'frontend-test':(['pnpm','test'],ROOT/'apps/web-client'),
 'typecheck':(['pnpm','typecheck'],ROOT/'apps/web-client'),
 'tiers':(['cargo','run','--offline','--release','-q','-p','benchmark-runner','--','tiers'],ROOT),
 'tier-report':(['cargo','run','--offline','--release','-q','-p','benchmark-runner','--','tier-report','server-baseline'],ROOT),
 'legacy-build':(['pnpm','build:wasm'],ROOT/'apps/web-client'),
 'cross-target':(['bash','scripts/cross-target-hash.sh'],ROOT),
 'worker':(['pnpm','smoke:native'],ROOT/'apps/web-client'),
 'sourcecaps-tests':(['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_source_lines.py'],ROOT),
 'sourcecaps':(['python3','-B','scripts/check-source-lines.py'],ROOT),
 'layering-tests':(['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_native_layering.py'],ROOT),
 'layering':(['python3','-B','scripts/check-native-layering.py'],ROOT),
 'formal':(['python3','-B',str(OUT/'formal.py')],ROOT),
 'vite':(['node',str(OUT/'vite-build.mjs')],ROOT/'apps/web-client')}
def run(name):
 cmd,cwd=COMMANDS[name];env=dict(ENV)
 if name in ['legacy-build','cross-target']:env['NODE_OPTIONS']='--require='+str(OUT/'target-preload.cjs')
 assert shutil.disk_usage(ROOT).free>=3*1024**3,'Disk below 3GiB; stopped'
 actual=['python3','-B','/tmp/noname-run-serialized.py',*cmd];started=time.monotonic();log=OUT/(name+'.log')
 print('START '+name,flush=True)
 with log.open('x') as f:result=subprocess.run(actual,cwd=cwd,env=env,stdout=f,stderr=subprocess.STDOUT)
 wall=time.monotonic()-started;text=log.read_text(errors='replace');last=json.loads(text.rstrip().splitlines()[-1]);rec={'name':name,'argv':actual,'cwd':str(cwd),'exit':result.returncode,'wall_seconds':wall,'locked_seconds':last['locked_seconds'],'queue_estimate_seconds':max(0,wall-last['locked_seconds']),'free_disk_bytes':shutil.disk_usage(ROOT).free,'log':log.name,'log_sha256':sha(log),'warning_lines':len(re.findall(r'^warning:',text,re.M)),'test_results':re.findall(r'test result: .*',text),'incremental_entries':[],'env':{k:env[k] for k in ['CARGO_TARGET_DIR','CARGO_INCREMENTAL','CARGO_BUILD_JOBS','GIT_OPTIONAL_LOCKS','GIT_NO_LAZY_FETCH']}}
 for d in ['debug/incremental','release/incremental','wasm32-unknown-unknown/release/incremental']:
  p=pathlib.Path(TARGET)/d
  if p.exists():rec['incremental_entries'] += [str(f) for f in p.iterdir()]
 idx=json.loads((OUT/'commands.json').read_text());idx.append(rec);save('commands.json',idx);print(json.dumps(rec,ensure_ascii=False),flush=True)
 assert rec['free_disk_bytes']>=3*1024**3,'Disk below 3GiB; stopped'
 return result.returncode
if __name__=='__main__':
 if sys.argv[1]=='entry':
  OUT.mkdir(exist_ok=True);assert not (OUT/'before.json').exists();state=snap();save('before.json',state);save('commands.json',[])
  for rel in state['legacy_modules']:
   dest=OUT/'old-legacy-modules'/rel;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(ROOT/rel,dest)
  save('entry.json',{'time':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source13':13,'A_remaining':50,'old_legacy_files':len(state['legacy_modules']),'setup_correction':'actual Cargo bin game-server; supervisor confirmed handoff noname-next-server typo','strict_native3':'unchanged path: prior A330 native3 -Dwarnings accepted; workflow only requires native3 strict, no global/game-server strict gate added','disk_free_bytes':shutil.disk_usage(ROOT).free,'deps_retention':'Stage3 debug/release/wasm deps needed for this continuous validation; CARGO_INCREMENTAL=0'})
  print('entry all hashes match; old legacy preserved')
 else:sys.exit(run(sys.argv[1]))
