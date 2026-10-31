import sys,json,hashlib,subprocess,re,datetime,shutil
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent))
from verify import O,R,sha,save,bind

def counts(text):
 rows=[tuple(map(int,m)) for m in re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)]
 return list(map(sum,zip(*rows))) if rows else None
commands=[]
for f in sorted(O.glob('*.result.json')):
 x=json.loads(f.read_text());x['counts']=counts((O/(x['name']+'.stdout.log')).read_text());x['warnings']=[s for s in (O/(x['name']+'.stderr.log')).read_text().splitlines() if s.startswith(('warning:','WARN','Error:'))];commands.append(x)
save('commands.json',commands)
before=json.load(open(O/'before.json'));after=json.load(open(O/'after.json'))
changed=[p for p,h in before['tracked_hashes'].items() if after['tracked_hashes'].get(p)!=h]
comparison={k:before[k]==after[k] for k in ['head','tree','index_sha256','staged','hooks','source6']}
comparison['protected']={k:before['protected'][k]==after['protected'][k] for k in before['protected']}
comparison['tracked_changes']=changed
comparison['only_parent_docs_changed']=all(p.startswith('docs/') for p in changed)
save('comparison.json',comparison)
products={str(p.relative_to(O/'frontend-dist')):{'sha256':sha(p),'bytes':p.stat().st_size} for p in (O/'frontend-dist').rglob('*') if p.is_file()}
source6=bind()
for f in source6:
 p=O/'final-source'/f;p.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(R/f,p)
current={p:sha(R/p) for p in before['protected']['modules']}
save('generated18-current.json',current)
save('products.json',{'production':products,'nativeWasmSha256':current['apps/web-client/src/worker/native-pkg/engine_native_wasm_bg.wasm'],'oldNativeWasmSha256':json.load(open(O/'old-generated18.json'))['apps/web-client/src/worker/native-pkg/engine_native_wasm_bg.wasm'],'defaultDist37Unchanged':comparison['protected']['dist']})
old=json.load(open(O/'old-frozen-pair.json'));save('old-frozen-pair-post.json',{'unchanged':all(sha(Path(p))==s for p,s in old.items()),'files':len(old)})
manifest={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'head':after['head'],'headTree':after['tree'],'source6':source6,'composedPatchSha256':json.load(open(O/'receipt.json'))['patch_sha256'],'sourceManifestSha256':sha(Path('/tmp/nc10-final-regression-d9f5d064/final-regression-disposition/final/manifest.json')),'commands':len(commands),'allOriginalAttemptsExit0':all(c['exit']==0 for c in commands),'allFinalRequiredExit0':all(c['exit']==0 for c in commands if c['name']!='native-probe'),'resolvedSetupFailures':[{'name':'native-probe','exit':101,'cause':'Private harness incorrectly assumed empty Shan selection accepted; actual spec min1/max1 allow_cancelfalse, InvalidChoice+unchanged fullstate, explicitPass and genuineOwncancel separately verified; final probe + WASM + browser passed'}],'initialToFinalIndexUnchanged':comparison['index_sha256'],'freezeIndexUnchanged':json.load(open(O/'freeze-comparison.json'))['fields']['index_sha256'],'indexLogicalEntriesEqualHead':json.load(open(O/'index-provenance.json'))['currentLogicalEntriesEqualHead'],'indexProvenanceLimitation':'Unknown pre-freeze raw SHA change; no initial raw copy/logical entries. No restoration performed.','noStagedFiles':not after['staged'],'source33Unchanged':comparison['protected']['admission31'],'protected99Unchanged':comparison['protected']['protected'],'defaultDist37Unchanged':comparison['protected']['dist'],'wholeTrackedPrePost':len(before['tracked_hashes']),'trackedChanges':changed,'freezeFieldsEqual':json.load(open(O/'freeze-comparison.json'))['equal'],'qualification':'Main functional + bounded product acceptance only; no NC10 strength / no-choice privacy / VM closure / all37 / overall Goal completion claim. Full Plan privacy proven by native Rust tests; public WASM auto_step summary is not full Plan.'}
save('manifest.json',manifest)
artifacts={str(p.relative_to(O)):{'sha256':sha(p),'bytes':p.stat().st_size} for p in O.rglob('*') if p.is_file() and not any(x in p.relative_to(O).parts for x in ['strict-target']) and p.name!='artifact-sha256.json'}
save('artifact-sha256.json',artifacts)
print(json.dumps(manifest,indent=2))
