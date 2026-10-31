import datetime,hashlib,json,re,sys
from pathlib import Path
R=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');O=Path('/tmp/nc27-main-final-v3-8e1fad00')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def load(n):return json.loads((O/n).read_text())
def save(n,x):(O/n).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
commands=[]
for f in sorted(O.glob('*.result.json')):
 x=json.loads(f.read_text());name=x['name'];text=(O/(name+'.stdout.log')).read_text();err=(O/(name+'.stderr.log')).read_text()
 for c in ['stdout','stderr']:assert sha(O/(name+'.'+c+'.log'))==x[c+'_sha256']
 x['wrapper_record']=json.loads(text.splitlines()[-1]);assert x['wrapper_record']['exit']==x['exit'] and x['wrapper_record']['command']==x['argv'];assert x['minimum_free_bytes']>=3*1024**3 and not x['disk_stop']
 x['warning_lines']=[l for l in (text+'\n'+err).splitlines() if l.startswith(('warning:','warning[','[WARN]')) or 'ExperimentalWarning:' in l]
 commands.append(x)
expected=['workspace','conformance','build-native','build-legacy','actual-native-source-final','actual-legacy-sink','worker-smoke','frontend-post-wasm','legacy-cross','native-cross','vite-post-wasm','lineunit','sourcecaps','strictrefs','preflight']
byname={x['name']:x for x in commands};pending=[n for n in expected if n not in byname];failed=[x['name'] for x in commands if x['exit'] and x['name']!='actual-native-source']
gate_invariants={};docs_package_invariants={}
if (O/'gate-exit.json').exists():
 a,b=load('gate-entry.json'),load('gate-exit.json');gate_invariants={k:a[k]==b[k] for k in a if k not in ['utc','free_disk_bytes']}
 if not all(gate_invariants.values()) or b['staged']:failed.append('gate-input-invariants')
 assert len(b['admission31'])==33 and len(b['modules'])==18 and len(b['dist'])==37 and len(b['protected'])==99
 c,d=load('gate-entry-supplement.json'),load('gate-exit-supplement.json');docs_package_invariants={k:c[k]==d[k] for k in c if k!='utc'}
 if not all(docs_package_invariants.values()):failed.append('gate-docs-package-invariants')
 assert len(c['docs_markdown'])>0
 if 'preflight' in byname and byname['preflight']['exit']==0:
  text=(O/'preflight.stdout.log').read_text();assert len([l for l in text.splitlines() if l.startswith('✅ ')])==21 and 'preflight 全部通过 ✅' in text and not any(l.startswith('❌') for l in text.splitlines())
 if 'lineunit' in byname and byname['lineunit']['exit']==0:assert 'Ran 2 tests' in (O/'lineunit.stderr.log').read_text()
 save('gate-comparison.json',{'invariants':gate_invariants,'docs_and_packages':docs_package_invariants,'all_equal':all(gate_invariants.values()) and all(docs_package_invariants.values()),'docs_markdown_count':len(c['docs_markdown'])})
front=(O/'frontend-post-wasm.stdout.log').read_text();frontcounts={k:int(re.search(r'^ℹ '+k+r' (\d+)$',front,re.M).group(1)) for k in ['tests','pass','fail','cancelled','skipped','todo']};assert frontcounts['tests']==frontcounts['pass']==179 and frontcounts['fail']==0
worker=(O/'worker-smoke.stdout.log').read_text();assert '{"total":82,"failed":0}' in worker
native=load('actual-native-source.json');assert native['passed'] and native['saveBeforeAfterIdentical'] and native['source_skill']=='遗计' and native['otherHidden']
legacy=load('legacy-wasm-sink-smoke.json');assert legacy['passed'] and legacy['steps']==120 and legacy['pendingChoicesObserved']==33
products=load('generated18-products.json');assert len(products)==18
for p,x in products.items():assert sha(R/p)==x['after_sha256']
module=load('bundle-module-table.json');assert len(module['sourceModules'])==35
for p,x in module['sourceModules'].items():assert sha(R/'apps/web-client'/p)==x['sourceSha256']
for p,x in module['chunks'].items():assert sha(O/'frontend-dist'/p)==x['outputSha256']
wasm_outputs={sha(p) for p in (O/'frontend-dist').rglob('*.wasm')}
assert products['apps/web-client/src/worker/native-pkg/engine_native_wasm_bg.wasm']['after_sha256'] in wasm_outputs
assert products['apps/web-client/src/wasm-pkg/engine_wasm_bg.wasm']['after_sha256'] in wasm_outputs
receipts=[json.loads(l) for l in (O/'build-child-argv.jsonl').read_text().splitlines()];assert len(receipts)==2 and all(x['changedKeys']==['CARGO_TARGET_DIR'] and x['afterTarget']=='/tmp/nc27-main-functional-458ed333/cargo-target' for x in receipts)
assert '--no-opt' in receipts[0]['args'] and '--no-default-features' in receipts[1]['args'] and '--no-opt' not in receipts[1]['args']
summary={'sealed_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'Requested MainV3 checks PASS; parent review required; overall Goal remains BLOCK' if not pending and not failed else 'Incomplete final gate qualification; overall Goal remains BLOCK','source_admission':load('admission.json'),'inherited_V2':load('v2-inherited-binding.json'),'functional_phase':load('functional-phase.json'),'workspace':load('workspace-parsed.json'),'frontend_post_wasm':frontcounts,'worker_smoke':{'total':82,'failed':0},'actual_native':native,'actual_legacy':legacy,'generated18':products,'production':load('versions-and-products.json'),'commands':commands,'pending_commands':pending,'unexpected_failed_commands':failed,'retained_harness_failure':'actual-native-source exit1 before metadata reads; unrequested input==post-restore assertion invalid because auth_generation increments; exact original script/log/result retained; corrected requested scope passes','gate_invariants':gate_invariants,'docs_package_invariants':docs_package_invariants,'no_staged_files':not load('before.json')['staged'] and (not load('gate-exit.json')['staged'] if (O/'gate-exit.json').exists() else not load('post-impact.json')['staged']),'target':'/tmp/nc27-main-functional-458ed333/cargo-target','notes':['Functional phase status is the immutable historical snapshot taken while awaiting DocsReady; top-level pending/failed lists and final-gate results describe the sealed final status.','Counts overlap: native/source/typed-sink tests are included in workspace6917; rerun cross tests and frontend179 are not added to workspace or historical totals.','Existing compiler/wasm-pack/Node warnings retained; no warning/source fix or relaxed gate.','Preflight itself captures child output internally; raw full-preflight outer stdout/stderr retained, not undisclosed child logs.','No GUI/Tauri release/platform/upstream-equivalence/global Goal completion claim.']}
save('summary.json',summary)
index={p.name:{'bytes':p.stat().st_size,'sha256':sha(p)} for p in sorted(O.iterdir()) if p.is_file() and p.name!='artifact-sha256.json'};save('artifact-sha256.json',index)
print(json.dumps({'status':summary['status'],'commands':len(commands),'pending':pending,'unexpected_failures':failed,'index_files':len(index),'index_sha256':sha(O/'artifact-sha256.json')},indent=2))
