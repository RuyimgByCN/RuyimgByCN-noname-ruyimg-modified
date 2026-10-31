from pathlib import Path
import json,hashlib,re,subprocess,tarfile,datetime
E=Path('/tmp/nc10-final-regression-d9f5d064/final-regression-disposition');F=E/'final';V=Path('/tmp/nc10-own-36992565/evidence/final-v2');W=Path('/tmp/nc10-own-36992565/worktree');M=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
def j(p):return json.loads(p.read_bytes())
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
m=j(F/'manifest.json');source={r['path']:r['sha256'] for r in m['files']};assert len(source)==6
for row in m['files']:
 p=row['path'];assert h(F/'source'/p)==row['sha256']==h(W/p)
 if row['baseline_sha256'] is not None:assert hashlib.sha256(subprocess.check_output(['git','show','5fc0f774:'+p],cwd=M)).hexdigest()==row['baseline_sha256']
for r in j(V/'manifest.json')['files']:assert h(F/'source'/r['path'])==r['sha256']
assert h(F/'candidate.patch')==m['composed_patch_sha256'];assert h(E/'semantic-delta.patch')==m['semantic_delta_sha256'];assert h(F/'source-overlay.tar.gz')==m['overlay_sha256'];assert h(F/'commands.json')==m['commands_sha256']
with tarfile.open(F/'source-overlay.tar.gz','r:gz') as t:
 members=t.getmembers();assert len(members)==6 and all(x.isfile() for x in members)
 for x in members:assert x.name in source and hashlib.sha256(t.extractfile(x).read()).hexdigest()==source[x.name]
commands=j(F/'commands.json');assert len(commands)==10;results=[]
for r in commands:
 n=r['name'];assert r['exit']==0 and r['source_before']==r['source_after']==source
 assert h(E/(n+'.stdout.log'))==r['stdout_sha256'] and h(E/(n+'.stderr.log'))==r['stderr_sha256'] and h(E/(n+'.result.json'))==r['result_sha256']
 assert r['environment']['CARGO_BUILD_JOBS']=='2' and r['environment']['CARGO_INCREMENTAL']=='0' and r['minimum_free_bytes']>3*1024**3
 text=(E/(n+'.stdout.log')).read_text(errors='replace');groups=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text);counts=[sum(int(x[i]) for x in groups) for i in range(3)] if groups else None
 if counts:assert counts[1]==0
 results.append({'name':n,'exit':r['exit'],'counts':counts,'argv':r['argv']})
strict=next(r for r in commands if r['name']=='gate1b');assert strict['environment']['RUSTFLAGS']=='-Dwarnings' and strict['environment']['CARGO_TARGET_DIR'].endswith('/gate1b-target')
assert next(r for r in results if r['name']=='native-final')['counts']==[1719,0,75]
assert next(r for r in results if r['name']=='sessions')['counts']==[6,0,0]
text=(E/'seed8211.stderr.log').read_text();assert 'steps=119 successful=119 suspended=0 simulation_faults=0 new_probe_faults=0 restored=true' in text
p='crates/engine-native/tests/ai_planner.rs';assert (E/'before-source'/p).read_bytes()==subprocess.check_output(['git','show','5fc0f774:'+p],cwd=M)
# Recheck exact preservation of the original failure and baseline logs against the old immutable evidence.
for old in ['seed8211.stdout.log','seed8211.stderr.log','seed8211.result.json','baseline-seed8211.stdout.log','baseline-seed8211.stderr.log','baseline-seed8211.result.json']:
 matches=list((E/'original-failure').glob(old));assert len(matches)==1 and h(matches[0])==h(Path('/tmp/nc10-own-36992565/evidence')/old)
assert subprocess.check_output(['git','diff','--cached','--name-only'],cwd=W)==b''
preservation=j(F/'preservation.json')
v={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS','source6_exact':True,'v2_five_unchanged':True,'commands':results,'overlay6_verified':True,'raw_original484_baseline_and119_failure_preserved':True,'seed8211_semantic_actual':'119success/0suspend/0simfault/0newprobe/fullrestore; P3checkedbytest; 119notexpectedpin','session_count_correction':6,'private_index_empty':True,'manifest_sha256':h(F/'manifest.json'),'composed_patch_sha256':h(F/'candidate.patch'),'qualification':'Retained finalprivate source/command/result parse, no test rerun/WASM rebuild/Main source import; notcompleteNC10strength/upstream.'};Path('/tmp/noname-parent-nc10-final-regression-audit.json').write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n');print(json.dumps(v,ensure_ascii=False,indent=2))
