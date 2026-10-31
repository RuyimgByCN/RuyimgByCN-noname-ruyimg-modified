from pathlib import Path
import json,hashlib,tarfile,re,datetime
E=Path('/tmp/nc10-own-36992565/evidence');F=E/'final-v2'
def load(p):return json.loads(p.read_bytes())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
m=load(F/'manifest.json');assert m['base']=='5fc0f77403d90fa4555156bf01722450cbe7bf3b' and len(m['files'])==5
for r in m['files']:
 p=F/'source'/r['path'];assert p.stat().st_size==r['bytes'] and sha(p)==r['sha256'],r['path']
assert sha(F/'candidate.patch')==m['patch_sha256']=='316b2588119799597965595c4c2a65493a84d26158bdb0317366cbf187e2b3a6';assert sha(F/'source-overlay.tar.gz')==m['overlay_sha256'];assert sha(E/'delta-v2.patch')==m['incremental_delta_sha256']=='fa9f4f6db9e42e4d8e74d646da60d29c6200327e7a853b3dc35b7a2c59cb372e'
for name,key in [('commands.json','commands_sha256'),('counts.json','counts_sha256'),('products.json','products_sha256'),('full-worktree-manifest.json','full_manifest_sha256')]:assert sha(F/name)==m[key]
rows={r['path']:r['sha256'] for r in m['files']};seen=set()
with tarfile.open(F/'source-overlay.tar.gz','r:gz') as t:
 for e in t:
  if e.isdir():continue
  assert e.isfile();n=e.name.removeprefix('./').removeprefix('source/');assert n in rows and n not in seen;seen.add(n);assert hashlib.sha256(t.extractfile(e).read()).hexdigest()==rows[n]
assert seen==set(rows)
old=load(E/'checkpoint-20/manifest.json');oldrows={r['path']:r['sha256'] for r in old['source']};changed=[n for n in rows if rows[n]!=oldrows[n]];assert set(changed)=={'crates/engine-native/src/ai.rs','crates/engine-native/tests/nc10_public_own_plan.rs'}
commands=load(F/'commands.json');assert len(commands)==30
for c in commands:
 n=c['name'];assert sha(E/(n+'.stdout.log'))==c['stdout_sha256'];assert sha(E/(n+'.stderr.log'))==c['stderr_sha256'];assert sha(E/(n+'.result.json'))==c['result_sha256'];r=load(E/(n+'.result.json'));assert r['exit']==c['exit'];assert c['minimum_free_bytes']>3*1024**3
counts=load(F/'counts.json')
for name in ['native-full','native-full-fixtures','final-native','wasm-crate-full']:
 text=(E/(name+'.stdout.log')).read_text();rr=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text);tot=[sum(int(x[i]) for x in rr) for i in range(3)];assert tot==[counts[name]['passed'],counts[name]['failed'],counts[name]['ignored']],(name,tot)
assert [counts['final-native'][x] for x in ['passed','failed','ignored']]==[1718,1,75]
products=load(F/'products.json');assert len(products)==6
for r in products:assert Path(r['path']).stat().st_size==r['bytes'] and sha(Path(r['path']))==r['sha256']
prot=load(F/'Main-protection.json');assert not prot['all_non_doc_tracked_changes'] and not prot['all_non_doc_protected_changes'];assert all(prot[k] for k in ['index_bytes_equal','index_entries_equal','empty_staging','hooks_equal','old_archives_manifests_equal'])
d=load(E/'first-divergence.json');assert d['seed']==8211 and d['step_zero_based']==61 and d['seat']==1;assert 'Pass' in d['oldAction'] and '85' in d['newAction'];assert len(d['formal_consequences'])==2
assert hashlib.sha256((F/'candidate.patch').read_bytes()+b'!').hexdigest()!=m['patch_sha256']
out={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS bounded frozen-source/30command/log/count/product binding audit; ACCEPTANCE BLOCK','source_files':5,'delta_v2_files':changed,'commands':30,'products':6,'final_native_v1':[1718,1,75],'quality_assertion':'Preservedseed8211 119vs484 failure; firstdivergenceseat1Pass/Card85bothAccepted;200seedformaloutcomedistributionsstillpending','timeout':'Worker45min orchestrationtimeout aftersealed09:40source/report; not an additional producttest failure','full44303unchanged_files':'Inherited exact5fc/originalworkerGitblob audit; no parent3GB rescan, no currentworking audit duringactivequalitydiagnostics','qualification':'Read-only parentfrozen5/tar/manifest/30rawcommands/counts/6products checks only; no producer/test/build/JS/network execution. No Mainintegration/privacyallstates/NoCert19/quality/wholeGoal closure.'};Path('/tmp/noname-parent-nc10-own-final-audit.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
