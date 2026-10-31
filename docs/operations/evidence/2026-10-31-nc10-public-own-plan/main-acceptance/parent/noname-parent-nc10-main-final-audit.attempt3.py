from pathlib import Path
import json,hashlib,re,subprocess,datetime,os
E=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/nc10-main-own-sfobj6i8');M=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
def j(p):return json.loads(p.read_bytes())
def h(p):
 z=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''):z.update(b)
 return z.hexdigest()
m=j(E/'manifest.json');art=j(E/'artifact-sha256.json');commands=j(E/'commands.json');before=j(E/'before.json');entry=j(E/'freeze-entry.json');end=j(E/'freeze-exit.json');product=j(E/'product-binding.json');results=[]
assert len(art)==714 and len(commands)==26
for rel,r in art.items():
 p=E/rel;assert p.is_file() and p.stat().st_size==r['bytes'] and h(p)==r['sha256'],rel
assert h(Path(m['finalReport']))==m['finalReportSha256']
assert h(E/'freeze-comparison.json')==m['freezeComparisonSha256'] and h(E/'index-provenance.json')==m['indexProvenanceSha256']
for p,sha in m['source6'].items():assert h(M/p)==sha
for c in commands:
 name=c['name'];assert c['source6_pre']==c['source6_post']==m['source6']
 assert c['exit']==(101 if name=='native-probe' else 0),(name,c['exit'])
 assert h(E/(name+'.stdout.log'))==c['stdout_sha256'] and h(E/(name+'.stderr.log'))==c['stderr_sha256']
 text=(E/(name+'.stdout.log')).read_text(errors='replace');groups=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text);counts=[sum(int(x[i]) for x in groups) for i in range(3)] if groups else None
 assert counts==c['counts'],(name,counts,c['counts']);assert c['disk_free_after']>3*1024**3
 assert c['environment']['CARGO_BUILD_JOBS']=='2' and c['environment']['CARGO_INCREMENTAL']=='0'
 results.append({'name':name,'exit':c['exit'],'counts':counts,'summaries':len(groups)})
assert next(r for r in results if r['name']=='native3')['counts']==[1719,0,75]
assert next(r for r in results if r['name']=='workspace')['counts']==[6922,0,77]
strict=next(c for c in commands if c['name']=='gate1b');assert strict['environment']['RUSTFLAGS']=='-Dwarnings' and strict['environment']['CARGO_TARGET_DIR']!=next(c for c in commands if c['name']=='workspace')['environment']['CARGO_TARGET_DIR']
assert 'ℹ pass 179' in (E/'frontend-tests.stdout.log').read_text()
preflight=(E/'preflight.stdout.log').read_text();assert len(re.findall(r'^✅ ',preflight,re.M))==21 and 'preflight 全部通过 ✅' in preflight
for key in ['head','tree','source6','index_sha256','index_entries','staged','status','tracked_hashes','hooks','protected','maintained_and_markdown']:assert entry[key]==end[key],key
assert len(end['tracked_hashes'])==44306 and len(end['maintained_and_markdown'])==14920
assert end['staged']=='' and entry['head']==m['head'] and entry['tree']==m['headTree']
assert h(E/'freeze-entry-index.raw')==h(E/'freeze-exit-index.raw')==end['index_sha256']
for group,rows in end['protected'].items():
 for p,sha in rows.items():assert h(M/p)==sha,(group,p)
assert len(end['protected']['admission31'])==33 and len(end['protected']['protected'])==99 and len(end['protected']['modules'])==18 and len(end['protected']['dist'])==37
changed_modules=[p for p,sha in before['protected']['modules'].items() if sha!=end['protected']['modules'][p]];assert changed_modules==['apps/web-client/src/worker/native-pkg/engine_native_wasm_bg.wasm']
assert before['index_sha256']!=end['index_sha256'] and m['initialToFinalIndexUnchanged'] is False
assert h(M/'apps/web-client/src/worker/native-pkg/engine_native_wasm_bg.wasm')==product['mainWasmSha256']==product['bundleAssetSha256']=='30e97f8aff0cc683e5d8bb3e2b122e023dba08260c45a3fbc0ebee630268c0bd'
assert h(E/'frontend-dist'/product['bundleWasmAsset'].lstrip('/'))==product['mainWasmSha256']
assert h(E/'frontend-dist/assets'/product['worker'])==product['workerSha256'] and product['servedSameWasm'] and product['productionAppReferencesWorker']
assert h(E/'browser-results.json')==product['browserResultSha256'] and h(E/'bundle-module-table.json')==product['bundleModuleTableSha256']
assert h(M/'apps/web-client/scripts/build-native.mjs')==product['mainOriginalBuildSha256'] and h(E/'build-child-argv.jsonl')==product['targetOverrideChildLogSha256']
browser=j(E/'browser-results.json');assert len(browser['reports'])==12 and browser['exceptions']==[] and browser['rejected']==[]
assert any(x['sha256']==product['mainWasmSha256'] and x['path']==product['bundleWasmAsset'] for x in browser['requested'])
wasm=j(E/'wasm-current-results.json');assert wasm['cases']==len(wasm['rows'])==7 and wasm['fullPlanExported'] is False
for row in wasm['rows']:
 n=row['name'];assert (E/f'native-{n}.ngsa').read_bytes()==(E/f'wasm-{n}.ngsa').read_bytes(),n
 assert row['other']['hidden'] is True
 assert row['accepted'] is (n!='shan-cancel')
assert (E/'native-shan-auto.ngsa').read_bytes()==(E/'native-shan-direct.ngsa').read_bytes()
a=j(E/'trajectory-native.json');b=j(E/'trajectory-wasm.json')
for k in ['direct','restore']:assert a[k]==b[k]
assert len(a['direct'])==40
package_a=j(E/'trajectory-package-native.json');package_b=j(E/'trajectory-package-wasm.json')
for k in ['direct','restore']:assert package_a[k]==package_b[k]
assert len(package_a['direct'])==20
cross=j(E/'cross-target-current-values.json');assert len(cross['hashes'])==2 and all(r['native']==r['wasm'] for r in cross['hashes'])
# A current read-only index probe validates logical entries; it does not explain the pre-freeze raw delta.
env={**os.environ,'GIT_OPTIONAL_LOCKS':'0'}
def git(*args):return subprocess.check_output(['git','--no-optional-locks',*args],cwd=M,env=env)
assert git('ls-files','--stage').decode()==end['index_entries'];assert git('diff','--cached','--name-only')==b''
assert end['hooks']['hooks/pre-push']=='e349af1ec5ccc1742df7e7d91d387ed2ef086f7c864d8cf7372d0ef66dd49ca7'
v={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS','fresh_artifacts':len(art),'source6_current_exact':True,'commands':results,'freeze_entry':entry['utc'],'freeze_exit':end['utc'],'frozen_tracked_hashes':len(end['tracked_hashes']),'frozen_maintained_and_markdown':len(end['maintained_and_markdown']),'all11_freeze_fields_equal':True,'source33_protected99_modules18_dist37_current':True,'changed_generated':changed_modules,'current_main_wasm':product['mainWasmSha256'],'wasm_adapter_cases':7,'browser_worker_cases':12,'trajectory_standard_steps':40,'trajectory_installed_package_steps':20,'legacy_crosshash_pairs':2,'index_caveat':'Unexplained raw pre-freeze difference retained; freeze rawcopy+logicalexact; allHEAD stage0/emptystaging corroborated. No speculative attribution or indexrepair.','qualification':'Independent fresh retainedartifact/source/product/protection/rawcounts audit, no producer rerun; not entire44306live3GBrescan. Privateprobeunusedtarget provenance addendum pending, not shipped/neededforacceptedproduct; no broadNC10strength/all37/platformacceptance.'};Path('/tmp/noname-parent-nc10-main-final-audit.json').write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n');print(json.dumps(v,ensure_ascii=False,indent=2))
