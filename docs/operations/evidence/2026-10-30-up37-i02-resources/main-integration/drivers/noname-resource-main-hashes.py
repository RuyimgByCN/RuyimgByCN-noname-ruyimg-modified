import hashlib,json,pathlib,subprocess,sys,os,time
ROOT=pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
OUT=ROOT/'docs/operations/evidence/2026-10-30-up37-i02-resources/main-integration'
OUT.mkdir(parents=True,exist_ok=True)
OLD=pathlib.Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-up37-assets-fg6ny846/evidence')
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def save(n,x): (OUT/n).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def snap(mode):
 old=json.loads((OLD/'immutable-before.json').read_text())
 missing=[p for p in old if not (ROOT/p).is_file()]
 assert not missing,missing
 protected={p:digest(ROOT/p) for p in sorted(old)}
 for d in ['content/original/ported','content/original/legacy-source','packages/extensions']:
  for p in (ROOT/d).rglob('*'):
   if p.is_file():protected[str(p.relative_to(ROOT))]=digest(p)
 tracked=subprocess.check_output(['git','ls-files','-z'],cwd=ROOT).decode().split('\0')
 source={p:digest(ROOT/p) for p in tracked if p and not p.startswith(('docs/','apps/web-client/dist/')) and (ROOT/p).is_file()}
 admission=json.loads((OUT.parent/'parent-apply-admission.json').read_text())
 runtime={p:digest(ROOT/p) for p in admission['applied_paths'] if not p.startswith('docs/')}
 source.update(runtime)
 wasm={str(p.relative_to(ROOT)):digest(p) for d in ['apps/web-client/src/wasm-pkg','apps/web-client/src/engine/native/wasm-pkg','apps/web-client/src/native-wasm-pkg'] for p in (ROOT/d).rglob('*') if p.is_file()}
 # discover generated WASM inputs from tracked/ignored actual directories
 for p in (ROOT/'apps/web-client').glob('**/*wasm*'):
  if p.is_file() and 'node_modules' not in p.parts and 'dist' not in p.parts:wasm[str(p.relative_to(ROOT))]=digest(p)
 core=ROOT.parent/'archive/upstream/noname/apps/core'
 m=json.loads((ROOT/'content/original/asset-manifest.json').read_text())
 assets={a['source_path']:digest(core/a['source_path']) for a in m['assets'] if a['source_category']=='core'}
 state={'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'protected':protected,'source':source,'wasm':wasm,'upstream_assets':assets,'runtime':runtime,'legacy_protected_count':len(old),'disk_free_bytes':__import__('shutil').disk_usage(ROOT).free,'CARGO_INCREMENTAL':os.environ.get('CARGO_INCREMENTAL')}
 save(mode+'-hashes.json',state)
 if mode=='before':
  assert all(runtime[p]==h for p,h in admission['applied_paths'].items() if p in runtime)
  save('source-admission.json',{'runtime_matches_admission':True,'runtime':runtime,'protected_count':len(protected),'legacy_protected_count':len(old),'source_count':len(source),'upstream_asset_count':len(assets),'wasm_inputs':wasm})
 else:
  b=json.loads((OUT/'before-hashes.json').read_text());result={k:{'before':len(b[k]),'after':len(state[k]),'equal':b[k]==state[k],'aggregate_sha256':hashlib.sha256(json.dumps(state[k],sort_keys=True,separators=(',',':')).encode()).hexdigest(),'changed':[p for p in set(b[k])|set(state[k]) if b[k].get(p)!=state[k].get(p)]} for k in ['protected','source','wasm','upstream_assets','runtime']};save('hash-comparison.json',result);assert all(x['equal'] for x in result.values()),result
 print(json.dumps({'mode':mode,'protected':len(protected),'source':len(source),'wasm':len(wasm),'upstream_assets':len(assets)}))
if __name__=='__main__':snap(sys.argv[1])
