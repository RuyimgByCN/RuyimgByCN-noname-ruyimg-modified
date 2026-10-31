from pathlib import Path
import hashlib,json,re,datetime,subprocess
E=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-phase7-ktqqj1fb/evidence/phase8/admission-repair');W=E.parents[2]/'worktree';P=Path(json.loads(Path('/tmp/noname-parent-c331-admission-repair-current.json').read_bytes())['root']);M=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
def j(p):return json.loads(p.read_bytes())
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
b=j(E/'results-bound.json');f=j(E/'final-freeze/source-manifest.json');full=j(E/'full-source/source-manifest.json');assert h(E/'final-freeze/source-manifest.json')==b['final_increment_sha256'];assert h(E/'full-source/source-manifest.json')==b['full_manifest_sha256']
for r in full['files']:assert h(W/r['path'])==r['sha256'],r['path']
logs=[]
for r in b['logs']:
 p=E/r['log'];assert h(p)==r['sha256'];text=p.read_text(errors='replace');groups=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text);counts=[sum(int(t[i]) for t in groups) for i in range(3)] if groups else None;assert counts==r['test_counts'],r['log'];records=[]
 for line in text.splitlines():
  try:v=json.loads(line)
  except ValueError:continue
  if isinstance(v,dict) and 'locked_seconds' in v and 'command' in v:records.append(v)
 assert records==r['runner_records'],r['log']
 for rec in records:assert rec['disk_free_bytes']>3*1024**3
 logs.append({'path':r['log'],'counts':counts,'wrapper_exits':[x['exit'] for x in records]})
assert next(x for x in logs if x['path']=='full-native.log')['counts']==[1773,0,75]
assert next(x for x in logs if x['path']=='four-packages.log')['counts']==[609,0,0]
assert next(x for x in logs if x['path']=='rust-red.log')['counts']==[1,2,0]
assert 'ℹ pass 186' in (E/'frontend-green2.log').read_text() and 'ℹ fail 0' in (E/'frontend-green2.log').read_text()
assert all(x['exit']==0 for x in b['final_checks']);g=next(x for x in b['final_checks'] if x['name']=='gate1b');assert g['RUSTFLAGS']=='-Dwarnings' and g['target'].endswith('/target-strict')
a=b['actual_sdk_wasm'];assert all(a[k] is True for k in ['actualCurrentWasm','actualSdkStore','originalTextToNative','positiveSchema5','positiveOriginalAssetOnly','unknownAssetManifestKeysPreserved']);assert len(a['results'])==11
for r in a['results']:assert all(r[k] is True for k in ['importRejected','directWasmRejected','admissionRejected','restoreRejected','storeHandleCatalogArchiveUnchanged'])
for r in a['oldFixtures']:assert r['restoreRuleStateEqual'] and r['postRestoreBadAdmissionAtomic']
for d in b['restore_generation_diff']:
 assert d['typed_fields_changed']==['auth_generation'] and d['before_generation']==0 and d['after_generation']==1 and d['all_other_fields_equal'] and d['state_hash_equal'] and d['view_equal']
 assert all(x['offset']==1573 or 1616<=x['offset']<1648 for x in d['byte_changes'])
assert b['no_instrumentation_remaining'] is True
# Production repair bytes are identical at checkpoint and final; only the SDK smoke gained positive restore checks.
checkpoint=j(E/'checkpoint15/source-manifest.json');before={r['path']:r['sha256'] for r in checkpoint['files']};changed=[r['path'] for r in f['files'] if before.get(r['path'])!=r['sha256']];assert changed==['apps/web-client/scripts/c331-admission-repair-smoke.mjs']
assert subprocess.check_output(['git','diff','--cached','--name-only'],cwd=W)==b''
assert subprocess.check_output(['git','diff','--cached','--name-only'],cwd=M)==b''
result={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS','fresh_current_source':len(full['files']),'increment_files':len(f['files']),'log_count':len(logs),'logs':logs,'SDK_bad_mutations':11,'actual_sdk_wasm_old3and4_positive':True,'restore_only_auth_generation_and_archive_checksum':True,'checkpoint_to_final_changes':changed,'independent_gate1b':g,'staging_empty':True,'qualification':'Fresh source/log/digest/result parse, no rerun/build/restore. Original invalid crossrestore raw-equality assertion preserved. CurrentWASM identity & completeartifactindex will be registered once producerseals; no wholeC331/Main acceptance.'};(P/'independent-results.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps(result,ensure_ascii=False,indent=2))
