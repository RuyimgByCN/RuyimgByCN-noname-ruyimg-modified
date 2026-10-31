from pathlib import Path
import json,hashlib,re,datetime,tarfile
M=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');O=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/nc10-main-own-sfobj6i8');N=Path('/tmp/nc10-main-lock-repair-0425dc78');I=Path('/tmp/i02-phase1-ingress-repair-89364384/unicode-acceptance-checkpoint');C=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-phase7-ktqqj1fb/evidence/phase9')
def j(p):return json.loads(p.read_bytes())
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def pkgs(p):return [{k:json.loads(v) for k,v in re.findall(r'^(name|version|source|checksum) = ("[^"\n]*")$',b,re.M)} for b in re.split(r'^\[\[package\]\]\s*$',p.read_text(),flags=re.M)[1:]]
def dump(name,v):Path('/tmp/noname-parent-'+name+'.json').write_text(json.dumps({'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),**v},ensure_ascii=False,indent=2)+'\n')
# Corrective Main dependency proof. This does not revise the original cross-dependency run.
nm=j(N/'manifest.json');na=j(N/'artifact-sha256.json')
for rel,r in na.items():assert h(N/rel)==r['sha256'] and (N/rel).stat().st_size==r['bytes'],rel
mainpkgs=pkgs(M/'Cargo.lock');newpkgs=pkgs(N/'probe/Cargo.lock');external=[p for p in newpkgs if p.get('source')];assert len(external)==30 and all(p in mainpkgs for p in external)
oldexternal=[p for p in pkgs(O/'probe/Cargo.lock') if p.get('source')];drift=[p for p in oldexternal if p not in mainpkgs];assert len(drift)==21
assert h(N/'probe/src/main.rs')==h(O/'probe/src/main.rs')==nm['probeSourceSha256']
assert h(N/'MainCargo.lock.before')==h(M/'Cargo.lock')==nm['bindingAfter']['MainCargoLock']
assert h(N/'binary/nc10-main-lock-acceptance-probe')==nm['binarySha256']
for key,file in [('lockVerificationSha256','lock-verification.json'),('comparisonSha256','comparison.json'),('commandsSha256','commands.json'),('commandSourceBindingsSha256','command-source-bindings.json')]:assert h(N/file)==nm[key]
for cmd in j(N/'commands.json'):
 assert cmd['exit']==0 and '--offline' in cmd['argv']
 assert cmd['source6AndLockBefore']['source6']==cmd['source6AndLockAfter']['source6']==nm['bindingAfter']['source6']
 assert cmd['source6AndLockBefore']['MainCargoLock']==cmd['source6AndLockAfter']['MainCargoLock']==h(M/'Cargo.lock')
 assert cmd['environment']['CARGO_INCREMENTAL']=='0' and cmd['environment']['CARGO_BUILD_JOBS']=='2'
for r in j(N/'comparison.json')['cases']:
 name=r['case'];assert (N/f'native-{name}.ngsa').read_bytes()==(O/f'wasm-{name}.ngsa').read_bytes()==(O/f'native-{name}.ngsa').read_bytes()
assert len(j(N/'comparison.json')['cases'])==7
for p,sha in nm['bindingAfter']['source6'].items():assert h(M/p)==sha
assert h(O/'artifact-sha256.json')==j(N/'entry.json')['oldArtifactIndexSha256']
dump('nc10-main-lock-audit',{'status':'PASS','artifacts_fresh_sha_verified':len(na),'exact_Main_external_packages':30,'original_drift_external_packages':drift,'new_binary':nm['binarySha256'],'full_archive_cases':7,'original_source_exact':True,'Main_source6_and_lock_current_exact':True,'qualification':'Retained original sidecheck network/unknown launcher and original21 dependency drift. New recorded offline locked run supplies exact Main-lock native consumer comparison; no original evidence rewritten.'})
# Private I02 source/patch identities and independently parsed final logs; no Main integration.
im=j(I/'manifest.json');oldI=Path('/tmp/i02-phase1-ab167599/acceptance-checkpoint');oldim=j(oldI/'manifest.json');oldsrc=oldim['sources'] if 'sources' in oldim else None
for p,sha in im['sources'].items():assert h(I/'source'/p)==h(Path('/tmp/i02-phase1-ab167599/source')/p)==sha,p
assert len(im['sources'])==30 and len(im['repair_files'])==11
assert h(I/'full.patch')==im['full_patch_sha256'] and h(I/'repair.patch')==im['repair_patch_sha256']
vi=j(I/'evidence/validation-index-final.json');assert vi['all_invalid_cases']==80 and vi['prior_invalid_corpus_preserved']==47
front=(I/'evidence/unicode-final-full-frontend.log').read_text();assert re.search(r'(?:ℹ|#)\s+pass\s+441\b',front)
rs=(I/'evidence/unicode-final-api-protocol.log').read_text();rows=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',rs);counts=[sum(int(r[i]) for r in rows) for i in range(3)];assert counts==[12,0,0],counts
for name in ['unicode-final-api-protocol','unicode-final-full-frontend','unicode-final-typecheck']:
 rec=j(I/'evidence'/(name+'.argv.json'));result=j(I/'evidence'/(name+'.result.json'));assert result['exit']==0 and rec['env']['CARGO_INCREMENTAL']=='0' and rec['env']['CARGO_BUILD_JOBS']=='2' and result['free_disk_bytes_after']>=3*1024**3
for name,needle in [('red-unicode-surrogate-2.log','repository_lone_high'),('red-rust.log','repository_bom')]:assert needle in (I/'evidence'/name).read_text()
protection=j(I/'evidence/unicode-protection-after.json');assert protection['main_index_empty'] and protection['private_index_empty'] and protection['prior_artifacts_and_owned225f_fixture_unchanged']
Iindex={str(p.relative_to(I)):{'sha256':h(p),'bytes':p.stat().st_size} for p in sorted(I.rglob('*')) if p.is_file()};dump('i02-ingress-repair-audit',{'status':'PASS_PRIVATE_FOUNDATION_ONLY','sources':30,'repair_files':11,'frontend_passed':441,'Rust_API_protocol':counts,'shared_negative_cases':80,'full_patch':im['full_patch_sha256'],'new_parent_fresh_index':Iindex,'qualification':'Hashes compared against frozen source manifest; command exits/raw counts independently parsed. Other evidence files newly indexed by parent, not falsely described as compared with an absent worker artifact index. No native full/GUI/WASM/producer/Main acceptance.'})
# C9 retained failure checkpoint, not an accepted foundation.
ci=j(C/'artifact-index.json')
for r in ci['files']:assert h(C/r['path'])==r['sha256'] and (C/r['path']).stat().st_size==r['bytes'],r['path']
cm=j(C/'full-source/source-manifest.json');snap=Path(j(Path('/tmp/noname-parent-c9-final-current.json'))['snapshot'])
for r in cm['files']:assert h(snap/r['path'])==r['sha256']
inc=j(C/'final-freeze9/source-manifest.json')
with tarfile.open(C/'final-freeze9/changed-source.tar.gz') as t:
 expected={r['path']:r['sha256'] for r in inc['files']};assert len(expected)==42
 for r in t.getmembers():assert hashlib.sha256(t.extractfile(r).read()).hexdigest()==expected[r.name]
raw=(C/'full-native.log').read_text();rows=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw);counts=[sum(int(r[i]) for r in rows) for i in range(3)];assert counts==[1774,8,75],counts
assert j(C/'results-bound.json')['full_native']=={'passed':1774,'failed':8,'ignored':75,'exit':101}
dump('c9-final-audit',{'status':'INTEGRITY_PASS_RULE_ACCEPTANCE_BLOCKED','sources':191,'increment':42,'artifact_sha_verified':len(ci['files']),'full_native_counts':counts,'full_native_exit':101,'snapshot':str(snap),'qualification':'8 original failures preserved, preHP/Before/Begin1..4/SubmitHp and callback migration incomplete; no Main merge or C9 acceptance.'})
print('PASS: Main lock-aligned probe; private I02 ingress repair; C9 failure checkpoint integrity only')
