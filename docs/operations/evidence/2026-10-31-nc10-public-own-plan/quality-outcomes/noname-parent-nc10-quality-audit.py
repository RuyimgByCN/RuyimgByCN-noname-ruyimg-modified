from pathlib import Path
import hashlib,json,collections,subprocess,datetime,math
Q=Path('/tmp/nc10-own-quality-8ed2ce6f/evidence')
M=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
W=Path('/tmp/nc10-own-36992565/worktree')
F=Path('/tmp/nc10-own-36992565/evidence/final-v2')
def digest(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
 return h.hexdigest()
def load(path):return json.loads(path.read_bytes())
manifest=load(Q/'manifest.json');summary=load(Q/'summary.json');hashes=manifest['evidence_files_sha256'];seen={};results={}
assert digest(Path(manifest['report']))==manifest['report_sha256']
for phase in ['baseline','candidate']:
 D=Q/phase;r=load(D/'result.json');s=summary['phases'][phase]
 assert r['exit']==0 and r['argv']==['cargo','test','--offline','-p','engine-native','--test','nc_rounds','nc12_ai_strength_baseline_statistics','--','--nocapture','--test-threads=1']
 assert r['source_after']=={k:v for k,v in r['source'].items() if k in r['source_after']}
 assert r['environment']['CARGO_INCREMENTAL']=='0' and r['environment']['CARGO_BUILD_JOBS']=='2'
 assert r['environment']['CARGO_TARGET_DIR']=='/tmp/nc10-own-36992565/target' and r['minimum_free_bytes']>3*1024**3
 for key,n in [('stdout_sha256','stdout.log'),('stderr_sha256','stderr.log'),('benchmark_binary_sha256','benchmark-binary')]:assert digest(D/n)==r[key]
 for p,h in r['source'].items():
  if (D/'source'/p).is_file():assert digest(D/'source'/p)==h
  if p in ['crates/engine-native/src/ai.rs','crates/engine-native-wasm/src/session.rs','crates/engine-native/tests/groups/nc_rounds.rs']:
   if phase=='baseline':assert hashlib.sha256(subprocess.check_output(['git','show','5fc0f774:'+p],cwd=M)).hexdigest()==h
   else:assert digest(F/'source'/p)==h
 games={g['seed']:g for g in map(json.loads,(D/'games.jsonl').read_bytes().splitlines())}
 assert set(games)==set(range(45000,45200)) and len(games)==200
 counts=collections.Counter();perseed=collections.defaultdict(collections.Counter);steps=collections.Counter();winners=collections.Counter();h=hashlib.sha256();dedup=0
 with (D/'submits.jsonl').open('rb') as stream:
  for line in stream:
   h.update(line);row=json.loads(line);seed=row['seed'];kind=row['kind']
   assert seed in games and row['step']==steps[seed] and 0<=row['seat']<4
   assert row['expected_revision']==row['step'] and row['after_revision']==row['expected_revision']+1
   assert row['dedup']>dedup;dedup=row['dedup']
   assert isinstance(row['input'],str) and row['input'].startswith('PolicyInput')
   assert isinstance(row['plan'],str) and row['plan'].startswith('Plan')
   assert kind in ['Accepted','Pending'] and row['outcome'].startswith(kind+' ')
   counts[kind]+=1;perseed[seed][kind]+=1;steps[seed]+=1
 seen[phase+'/submits.jsonl']=h.hexdigest();assert h.hexdigest()==s['trace_sha256']==hashes[phase+'/submits.jsonl']
 for seed,g in games.items():
  assert g['finished'] is True and g['byte_restore'] is True and g['reasons']=={} and g['legacy_cumulative_sim_probe_faults']==0
  assert g['steps']==steps[seed]==sum(g['counts'].values()) and dict(perseed[seed])==g['counts'];winners[g['winner']]+=1
 assert counts['Accepted']==s['Accepted'] and counts['Pending']==s['Pending'] and sum(counts.values())==s['actual_submits']
 ordered=sorted(steps.values());pct=[ordered[math.floor((len(ordered)-1)*p+0.5)] for p in [.5,.9]]+[ordered[-1]]
 assert pct==s['steps_p50_p90_max'] and dict(winners)==s['winners']
 archives=load(D/'archives.json');assert len(archives)==200 and {x['seed'] for x in archives}==set(games)
 for a in archives:
  path=Path(a['path']);assert path==D/f"final-{a['seed']}.ngsa" and digest(path)==a['sha256'];assert path.read_bytes()[:4]==b'NGSA'
 results[phase]={'games':len(games),'counts':dict(counts),'actual_submits':sum(counts.values()),'steps_p50_p90_max':pct,'winners':dict(winners),'final_archive_hashes':len(archives),'source_and_binary_binding':True}
for rel,expected in hashes.items():
 p=Q/rel;assert p.is_file(),rel
 if rel not in seen:seen[rel]=digest(p)
 assert seen[rel]==expected,(rel,expected,seen[rel])
# The five restored candidate sources are independently compared with the sealed v2 copy.
files=load(F/'manifest.json')['files']
if isinstance(files,dict):items=[(p,h if isinstance(h,str) else h['sha256']) for p,h in files.items()]
else:items=[(x['path'],x['sha256']) for x in files]
for p,h in items:assert digest(W/p)==h==digest(F/'source'/p)
quality='crates/engine-native/tests/nc12_ai_strength_baseline.rs'
assert (W/quality).read_bytes()==subprocess.check_output(['git','show','5fc0f774:'+quality],cwd=M)
assert subprocess.check_output(['git','diff','--cached','--name-only'],cwd=W)==b''
assert (Q/'baseline/source'/quality).read_bytes()==(Q/'candidate/source'/quality).read_bytes()
a=load(Q/'preservation.json');assert all(a['restored_frozen5_and_quality_test'].values()) and a['private_staging']=='' and a['Main_held315_equal']
res={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS','manifest_sha256':digest(Q/'manifest.json'),'fresh_evidence_hashes':len(seen),'results':results,'restored_candidate_files':len(items),'quality_test_restored_to_Main5fc':True,'staging_empty':True,'qualification':'Independent raw retained-evidence parse, not a producer rerun or new execution/restore; NGSA decoded/restored in original Rust command; here bytes/header/hash only. Complete input/Plan retained as Rust Debug strings, not a new structured DTO. No strength/upstream-rule/fullNC10 acceptance.'}
Path('/tmp/noname-parent-nc10-quality-audit.json').write_text(json.dumps(res,ensure_ascii=False,indent=2)+'\n');print(json.dumps(res,ensure_ascii=False,indent=2))
