from pathlib import Path
import json,hashlib,math,datetime
R=Path('/tmp/pa09-release-fe278227');E=R/'evidence'
def read(n):return json.loads((E/n).read_bytes())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
I=read('evidence-index.json')
for r in I['files']:
 p=R/r['path'];assert p.is_file() and p.stat().st_size==r['bytes'] and sha(p)==r['sha256'],r['path']
assert sha(Path(I['report']))==I['report_sha256']
for v in ['candidate','serial']:
 b=read(v+'-binary.json');assert sha(Path(b['binary']))==b['binary_sha256'];assert b['profile']=='release'
for row in read('test-overlay-manifest.json'):
 p=Path(row['tree'])/row['only_changed_file'];assert p.stat().st_size==row['overlay_bytes'] and sha(p)==row['overlay_sha256']
cmd=read('command-matrix.json');assert len(cmd)==6 and all(c['exit']==0 and '--release' in c['argv'] and not c['extra_compiler_flags'] for c in cmd)
A=read('analysis.json');runs=[]
for order,v in enumerate(['candidate','serial','serial','candidate']):
 allrows=[json.loads(x) for x in (E/f'abba-{order}-{v}.raw.ndjson').read_text().splitlines()];meta=[x for x in allrows if x['kind']=='metadata'];assert len(meta)==1;meta=meta[0]['metadata'];assert meta['profile']=='release' and meta['nodes']==64 and meta['deadline_ms']==50 and meta['players']==4 and meta['seed']==42
 req=[x for x in allrows if x['kind']=='request'];assert len(req)==1687 and [x['request_id'] for x in req]==list(range(1,1688));auto=[x for x in req if x['request_type']=='autoStep'];assert len(auto)==1680
 measured=[x for x in auto if not x['warmup']];assert len(measured)==1400 and {x['game_index'] for x in measured}=={1,2,3,4,5};assert sum(x['accepted_jobs_delta'] for x in measured)==(625 if v=='candidate' else 0);assert sum(x['accepted_jobs_delta']>0 for x in measured)==(135 if v=='candidate' else 0)
 summary=[x for x in allrows if x['kind']=='summary'];assert len(summary)==1 and summary[0]['completed_games']==5
 for x in req:assert x['profile']=='release' and x['source_identity']==meta['source_identity'] and x['binary_sha256']==meta['binary_sha256']
 runs.append((req,measured))
base=[(x['request_id'],x['game_index'],x['step_index'],x['request_type'],x['hash_hex']) for x in runs[0][0]]
for req,_ in runs[1:]:assert [(x['request_id'],x['game_index'],x['step_index'],x['request_type'],x['hash_hex']) for x in req]==base

def stats(rows):
 out={'count':len(rows),'accepted_jobs':sum(x['accepted_jobs_delta'] for x in rows)}
 for field in ['roundtrip_ns','owner_ns']:
  v=sorted(x[field] for x in rows);out[field]={'p50':v[math.ceil(len(v)*.50)-1],'p95':v[math.ceil(len(v)*.95)-1],'p99':v[math.ceil(len(v)*.99)-1],'mean':sum(v)/len(v),'sum':sum(v),'min':v[0],'max':v[-1]}
 return out
pairs=[]
for ci,si in [(0,1),(3,2)]:
 c=runs[ci][1];s=runs[si][1];assert stats(c)==A['pairs'][len(pairs)]['total']['candidate'] and stats(s)==A['pairs'][len(pairs)]['total']['serial'];eligible=[j for j,x in enumerate(c) if x['accepted_jobs_delta']>0];cs=stats([c[j] for j in eligible]);ss=stats([s[j] for j in eligible]);assert cs==A['pairs'][len(pairs)]['eligible_after_full_hash_match']['candidate'] and ss==A['pairs'][len(pairs)]['eligible_after_full_hash_match']['serial_matched'];assert cs['roundtrip_ns']['p95']>ss['roundtrip_ns']['p95'] and stats(c)['roundtrip_ns']['p95']>stats(s)['roundtrip_ns']['p95'];assert {x['accepted_jobs_delta'] for x in c if x['accepted_jobs_delta']}=={3,4,7};pairs.append({'candidate':ci,'serial':si,'total':{'candidate':stats(c),'serial':stats(s)},'eligible':{'candidate':cs,'serial':ss}})
out={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS identity/analysis; NEGATIVE performance result','fresh_evidence_files':len(I['files']),'fresh_two_binary_hashes':True,'six_release_commands_success':True,'request_rows_each':1687,'full_coordinate_requestid_statehash_equality':True,'warmup_games':1,'measured_games':5,'measured_auto_steps_each':1400,'pairs':pairs,'qualification':'Read-only parent SHA/JSON/raw-row parser; no producer/test/build/network execution. One seed/load, four processes; no stable overall or eligible-subset benefit, no admission threshold/defaultswitch/Main integration or fullPA09 acceptance.'}
Path('/tmp/noname-parent-pa09-release-results.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({k:v for k,v in out.items() if k!='pairs'},indent=2))
