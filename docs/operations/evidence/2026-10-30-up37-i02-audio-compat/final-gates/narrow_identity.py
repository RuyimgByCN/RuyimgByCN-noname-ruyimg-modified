import pathlib,json,hashlib,subprocess,os,sys,time,shutil,tempfile,datetime
ROOT=pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');OUT=pathlib.Path(__file__).resolve().parent;CORE=ROOT.parent/'archive/upstream/noname/apps/core';ENV={**os.environ,'GIT_OPTIONAL_LOCKS':'0','GIT_NO_LAZY_FETCH':'1','CARGO_INCREMENTAL':'0','CARGO_BUILD_JOBS':'2'}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(n,x):(OUT/n).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def stat_id(p):
 s=p.stat();return {'dev':s.st_dev,'inode':s.st_ino,'size':s.st_size,'mtime_ns':s.st_mtime_ns}
def snap():
 a=json.loads((OUT.parent/'parent-main-source-admission.json').read_text());old_a=json.loads((ROOT/'docs/operations/evidence/2026-10-30-adr0787-timeout/parent-source-admission.json').read_text());ci=json.loads((ROOT/'docs/operations/evidence/2026-10-30-native-ci-corrections/parent-source-admission.json').read_text());frozen=json.loads((ROOT/'docs/operations/evidence/2026-10-30-adr0787-timeout/main-acceptance/phase2/frozen-extra-sha256.json').read_text());paths=set(old_a['runtime_paths_applied'])|set(ci['paths'])|set(frozen)|{'apps/web-client/src/views/CollectionView.vue','scripts/original-source-ast.mjs','scripts/original-content-order.mjs','.pi-lens.json','.gitattributes','content/original/catalog.json','.github/actions/prepare-original-content/action.yml','.github/actions/prepare-original-content/liuli-longxue.patch','Cargo.lock','Cargo.toml'};paths-=set(a['source14'])
 state={'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'index':subprocess.check_output(['git','diff','--cached','--name-only'],cwd=ROOT,text=True).splitlines(),'source14':{p:sha(ROOT/p) for p in a['source14']},'protected':{p:sha(ROOT/p) for p in sorted(paths)},'guard':json.loads((ROOT/'.pi-lens.json').read_text()),'modules':{str(p.relative_to(ROOT)):sha(p) for d in ['apps/web-client/src/worker/native-pkg','apps/web-client/src/wasm-pkg','apps/web-client/public/pkg'] for p in (ROOT/d).rglob('*') if p.is_file()},'dist':{str(p.relative_to(ROOT)):sha(p) for p in (ROOT/'apps/web-client/dist').rglob('*') if p.is_file()},'frozen_packages_locks_diff':hashlib.sha256(subprocess.check_output(['git','diff','HEAD','--','content/original/ported','content/original/legacy-source','packages/extensions','Cargo.lock'],cwd=ROOT)).hexdigest(),'upstream_head':subprocess.check_output(['git','-C',str(CORE.resolve()),'rev-parse','HEAD'],text=True).strip()}
 assert state['source14']==a['source14'];assert state['head']==a['base'];assert not state['index'];g=state['guard'];assert g['autofix']['enabled']==g['format']['enabled']==g['actionableWarnings']['autoFix']['enabled']==False
 for f,expected in [('apps/web-client/src/worker/native-pkg/engine_native_wasm_bg.wasm','2292d7ab1c04b05724e49eb750962bb78af119dac80011524b18574742ed189b'),('apps/web-client/src/wasm-pkg/engine_wasm_bg.wasm','2b1cc44e511484c804f10640429b32075fa525227f5d07468bdf5f9dc1fab355')]:assert state['modules'][f]==expected
 old_code=subprocess.check_output(['git','show','HEAD:scripts/original-audio-resources.mjs'],cwd=ROOT,text=True);new_code=(ROOT/'scripts/original-audio-resources.mjs').read_text();chunk=lambda text:text[text.index('export function resolveAudioDeclarations'):text.index('export function declaredAudioAssets')];assert chunk(old_code)==chunk(new_code);state['global_owner_resolver_sha256']=hashlib.sha256(chunk(new_code).encode()).hexdigest()
 return state

# Copied snapshot seam from main-acceptance/accept.py; no acceptance entry/finalizer runs.
def snapshot(filename):
 state=snap()
 prior=json.loads((OUT.parent/'main-acceptance/before.json').read_text())
 prior_after=json.loads((OUT.parent/'main-acceptance/after.json').read_text())
 assert prior==prior_after, 'historical functional identity mismatch'
 assert [len(state[k]) for k in ['source14','protected','modules','dist']]==[14,75,18,37]
 for k in ['source14','modules','dist','global_owner_resolver_sha256']:
  assert state[k]==prior[k], 'historical frozen identity mismatch: '+k
 docs_delta={p:{'functional_before':h,'current':state['protected'][p]} for p,h in prior['protected'].items() if state['protected'][p]!=h}
 assert all(p.startswith('docs/') for p in docs_delta), docs_delta
 state['accepted_prior_parent_docs_delta']=docs_delta
 state['source14_aggregate_sha256']=hashlib.sha256(json.dumps(state['source14'],sort_keys=True,separators=(',',':')).encode()).hexdigest()
 assert state['source14_aggregate_sha256']=='2dd4b54caf8ade53b66c71c45d3e16453c764de8fbb2ad7ad657d78f7ccf6bea'
 state['index_sha256']=sha(ROOT/subprocess.check_output(['git','rev-parse','--git-path','index'],cwd=ROOT,text=True).strip())
 state['tracked_diff_sha256']=hashlib.sha256(subprocess.check_output(['git','diff','HEAD','--binary','--','.',':!docs/operations/evidence'],cwd=ROOT,env=ENV)).hexdigest()
 docs=subprocess.check_output(['git','ls-files','-z','--','*.md'],cwd=ROOT,env=ENV).decode().split('\0')
 state['maintained_docs']={p:sha(ROOT/p) for p in docs if p and 'docs/operations/evidence/' not in p}
 state['entry_fixed_untracked_source_paths']=sorted(str(p.relative_to(ROOT)) for p in (ROOT/'content/original-audio-compat').rglob('*') if p.is_file())
 state['tracked_status']=subprocess.check_output(['git','status','--porcelain=v1','--untracked-files=no'],cwd=ROOT,env=ENV,text=True).splitlines()
 index=json.loads((OUT.parent/'main-acceptance/artifact-sha256.json').read_text())
 state['functional_artifacts']={p:sha(OUT.parent/'main-acceptance'/p) for p in index}
 assert len(index)==64 and state['functional_artifacts']==index
 state['functional_artifact_index_sha256']=sha(OUT.parent/'main-acceptance/artifact-sha256.json')
 state['admission_sha256']=sha(OUT.parent/'parent-main-source-admission.json')
 state['independent_results_sha256']=sha(OUT.parent/'parent-independent/functional-independent-results.json')
 state['free_disk_bytes']=shutil.disk_usage(ROOT).free
 state['captured_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat()
 assert state['free_disk_bytes']>=3*1024**3
 save(filename,state)
 if filename!='narrow-entry.json':
  before=json.loads((OUT/'narrow-entry.json').read_text())
  comparison={k:state[k]==before[k] for k in before if k not in ['free_disk_bytes','captured_utc']}
  save('source-comparison.json',{'entry':'narrow-entry.json','exit':filename,'equal':comparison,'all_equal':all(comparison.values()),'no_staged_files':not state['index']})
  assert all(comparison.values()), comparison
 return state

if __name__=='__main__':
 snapshot(sys.argv[1])
