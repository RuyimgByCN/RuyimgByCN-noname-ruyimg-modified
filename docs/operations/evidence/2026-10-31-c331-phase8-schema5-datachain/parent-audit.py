from pathlib import Path
import hashlib,json,re,datetime,subprocess
E=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-phase7-ktqqj1fb/evidence/phase8');W=E.parent.parent/'worktree';M=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');P=Path(json.loads(Path('/tmp/noname-parent-c331-phase8-current.json').read_bytes())['root'])
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def j(p):return json.loads(p.read_bytes())
b=j(E/'results-bound.json');f=j(E/'final-freeze8-v2/source-manifest.json');pre=j(E/'pre-full2/source-manifest.json');full=j(E/'full-source/source-manifest.json');logs=[]
for ref in b['source_refs']:assert h(Path(ref['path']))==ref['sha256']
assert [(r['path'],r['sha256']) for r in f['files']]==[(r['path'],r['sha256']) for r in pre['files']]
for row in full['files']:assert h(W/row['path'])==row['sha256']
for row in b['logs']:
 path=E/row['log'];assert h(path)==row['sha256'];text=path.read_text(errors='replace')
 counts=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)
 total=[sum(int(r[i]) for r in counts) for i in range(3)] if counts else None
 assert total==row['test_counts'],(row['log'],total,row['test_counts'])
 records=[]
 for line in text.splitlines():
  try:v=json.loads(line)
  except (ValueError,TypeError):continue
  if isinstance(v,dict) and 'locked_seconds' in v and 'command' in v:records.append(v)
 assert records==row['runner_records'],row['log']
 logs.append({'path':row['log'],'counts':total,'results':len(counts),'exits':[r['exit'] for r in records]})
assert next(x for x in logs if x['path']=='full-native3.log')['counts']==[1773,0,75]
assert next(x for x in logs if x['path']=='full-native2.log')['counts']==[1764,1,75]
assert next(x for x in logs if x['path']=='initial-red.log')['counts']==[0,3,0]
assert 'can\'t open file' in (E/'full-native-first.log').read_text()
assert 'ℹ pass 183' in (E/'frontend-tests2.log').read_text() and 'ℹ fail 0' in (E/'frontend-tests2.log').read_text()
g=b['gate1b'];assert g['exit']==0 and g['RUSTFLAGS']=='-Dwarnings' and g['jobs']==2 and g['incremental']==0 and g['independent_target'].endswith('/target-strict')
assert all(r['exit']==0 for r in b['final_checks'])
for name,sha in b['C29approved_contract_hashes'].items():assert h(M/'docs/operations/evidence/2026-10-31-c331-runtime-contract'/name)==sha
assert subprocess.check_output(['git','diff','--cached','--name-only'],cwd=W)==b''
assert subprocess.check_output(['git','diff','--cached','--name-only'],cwd=M)==b''
v={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS','materialized_snapshot':str(P/'snapshot'),'full_current_source':len(full['files']),'final_increment_files':len(f['files']),'pre_full_and_final_source_sha_equal':True,'log_count':len(logs),'logs':logs,'gate1b':g,'C29_six_frozen_contract_sha':True,'Main_private_staging_empty':True,'qualification':'Fresh source/hash/raw-result parser, not new test execution. Worker protection assertions retained separately; not another31867fullscan. Private C8 datafoundation only; runtime consumers/fullC331 and Main integration blocked.'}
(P/'independent-results.json').write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n');print(json.dumps(v,ensure_ascii=False,indent=2))
