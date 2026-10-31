from pathlib import Path
import hashlib,json,collections,math,subprocess,datetime
Q=Path('/tmp/nc10-fixed-reference-38de1942/evidence');M=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');F=Path('/tmp/nc10-own-36992565/evidence/final-v2');W=Path('/tmp/nc10-own-36992565/worktree')
def j(p):return json.loads(p.read_bytes())
def h(p):
 d=hashlib.sha256()
 with p.open('rb') as f:
  for x in iter(lambda:f.read(1024*1024),b''):d.update(x)
 return d.hexdigest()
manifest=j(Q/'manifest.json');summary=j(Q/'summary.json');binding=j(Q/'reference-binding.json');observed={}
base=(Q/'reference-baseline-ai.rs').read_text();measure=(Q/'measurement-ai.rs').read_text();candidate=(F/'source/crates/engine-native/src/ai.rs').read_text()
assert base.encode()==subprocess.check_output(['git','show','5fc0f774:crates/engine-native/src/ai.rs'],cwd=M)
start=measure.index('/// Public command advice;');end=measure.index('\nimpl Engine {',start)
helper_start=measure.index('pub fn projected_choice_action',start);helper_end=measure.index('\nthread_local!',helper_start)
assert measure[helper_start:helper_end].strip()==candidate[candidate.index('pub fn projected_choice_action'):candidate.index('\nimpl Engine {',candidate.index('pub fn projected_choice_action'))].strip()
normalized=measure[:start]+measure[end+1:]
prefix='''        if !MEASUREMENT_REFERENCE.with(|flag| flag.get()) {
            if let Some(choice @ ChoiceView::Own { .. }) = self.ai_session(seat).choice() {
                return Plan { seat, revision: self.revision(), action: projected_choice_action(choice).expect("Own advice"),
                    nodes_used: 0, budget_exhausted: false, timed_out: false, cancelled: planner.cancelled(), sim_faults: 0 };
            }
        }
'''
assert normalized.count(prefix)==1
normalized=normalized.replace(prefix,'').replace('measurement_reference_choice_action','projected_choice_action')
assert normalized==base,'reference normalization != full baseline'
for n in ['calibration','candidate-own-control','pilot']:
 r=j(Q/(n+'.result.json'));assert r['exit']==0 and r['source']['crates/engine-native/src/ai.rs']==binding['instrumented_ai_sha256']==h(Q/'measurement-ai.rs')
 assert r['binary_sha256']==h(Q/(n+'.binary'))
 for k,filename in [('stdout_sha256',n+'.stdout.log'),('stderr_sha256',n+'.stderr.log')]:assert r[k]==h(Q/filename)
 assert all(r['source'][p]==sha for p,sha in r['source_after'].items())
 assert r['environment']['CARGO_INCREMENTAL']=='0' and r['environment']['CARGO_BUILD_JOBS']=='2' and r['minimum_free_bytes']>3*1024**3
expected=j(Q/'calibration-expected.json');actual=j(Q/'calibration-actual.json');assert len(expected)==len(actual['rows'])==24
for a,b in zip(expected,actual['rows']):
 for k in ['input','plan','outcome','step']:assert a[k]==b[k]
actual_own=sum('choice: Some(Own {' in x['input'] for x in actual['rows']);assert actual_own==actual['own_calls']==4
# Compare the saved expected calibration against its original baseline experiment.
with Path('/tmp/nc10-own-quality-8ed2ce6f/evidence/baseline/submits.jsonl').open('rb') as f:
 original=[json.loads(f.readline()) for _ in range(24)]
assert original==expected
assert actual['authority_rng_unchanged_by_switch'] is True
prereg=j(Q/'preregistered.json');assert prereg==summary['preregistered'] and prereg['seeds']==[46000,46031] and prereg['games']==256
rows=[json.loads(x) for x in (Q/'pilot-games.jsonl').read_bytes().splitlines()];games={(x['seed'],x['challenger'],x['arm']):x for x in rows};assert len(games)==len(rows)==256
assert set(games)=={(s,c,a) for s in range(46000,46032) for c in range(4) for a in ['baseline','candidate']}
pergame=collections.defaultdict(collections.Counter);steps=collections.Counter();refcalls=collections.Counter();propcalls=collections.Counter();rawhash=hashlib.sha256();no_other=True
with (Q/'pilot-submits.jsonl').open('rb') as f:
 for line in f:
  rawhash.update(line);row=json.loads(line);key=(row['seed'],row['challenger'],row['arm']);assert key in games
  assert row['step']==steps[key] and row['after_revision']==row['step']+1
  assert row['reference']==(row['arm']=='baseline' or row['seat']!=row['challenger'])
  assert row['outcome'].startswith(('Accepted {','Pending {'))
  kind=row['outcome'].split(' ',1)[0];pergame[key][kind]+=1;steps[key]+=1
  (refcalls if row['reference'] else propcalls)[row['arm']]+=1
  assert 'sim_faults: 0' in row['plan'] and 'timed_out: false' in row['plan']
  assert 'choice: Some(Other {' not in row['input']
observed['pilot-submits.jsonl']=rawhash.hexdigest();assert sum(steps.values())==84344==summary['actual_submit_rows']
computed={}
for arm in ['baseline','candidate']:
 selected=[(k,g) for k,g in games.items() if k[2]==arm];counts=collections.Counter();winner=collections.Counter();wins=0;values=[];seats={}
 for k,g in selected:
  assert g['finished'] and g['byte_restore'] and g['error'] is None and g['sim_faults']==0
  assert g['steps']==steps[k] and g['counts']==dict(pergame[k]);assert g['candidate_calls']+g['reference_calls']==g['steps']
  assert g['challenger_win']==(g['winner']==f'Player({g["challenger"]})')
  counts.update(pergame[k]);winner[g['winner']]+=1;wins+=g['challenger_win'];values.append(g['steps'])
 for c in range(4):
  gs=[g for k,g in selected if k[1]==c];seats[str(c)]={'games':len(gs),'wins':sum(x['challenger_win'] for x in gs),'finished':sum(x['finished'] for x in gs),'steps':sum(x['steps'] for x in gs)}
 values.sort();pct=[values[math.floor((len(values)-1)*p+0.5)] for p in [.5,.9]]+[values[-1]]
 s=summary['phases'][arm];assert s['wins']==wins and s['counts']==dict(counts) and s['per_seat']==seats and s['winner_counts']==dict(winner) and s['steps_p50_p90_max']==pct
 assert refcalls[arm]==s['reference_calls'] and propcalls[arm]==s['candidate_calls']
 computed[arm]={'games':len(selected),'wins':wins,'counts':dict(counts),'reference_calls':refcalls[arm],'candidate_calls':propcalls[arm],'per_seat':seats,'steps_p50_p90_max':pct}
paired=collections.Counter()
for seed in range(46000,46032):
 for c in range(4):
  b=games[seed,c,'baseline']['challenger_win'];p=games[seed,c,'candidate']['challenger_win'];paired['both' if b and p else 'baseline_only' if b else 'candidate_only' if p else 'neither']+=1
assert dict(paired)==summary['paired_wins']
archives=j(Q/'archives.json');assert len(archives)==256
archivehash={}
for a in archives:
 p=Q/f"final-{a['seed']}-{a['challenger']}-{a['arm']}.ngsa";assert p.stat().st_size==a['bytes'] and h(p)==a['sha256'] and p.read_bytes()[:4]==b'NGSA';archivehash[a['seed'],a['challenger'],a['arm']]=a['sha256']
for seed in range(46000,46032):
 assert len({(games[seed,c,'baseline']['winner'],games[seed,c,'baseline']['steps'],archivehash[seed,c,'baseline']) for c in range(4)})==1
for rel,digest in manifest['evidence_sha256'].items():
 if rel not in observed:observed[rel]=h(Q/rel)
 assert observed[rel]==digest,rel
for r in j(F/'manifest.json')['files']:assert h(W/r['path'])==r['sha256']
quality='crates/engine-native/tests/nc12_ai_strength_baseline.rs';assert (W/quality).read_bytes()==subprocess.check_output(['git','show','5fc0f774:'+quality],cwd=M)
assert subprocess.check_output(['git','diff','--cached','--name-only'],cwd=W)==b''
result={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS','fresh_evidence_hashes':len(observed),'manifest_sha256':h(Q/'manifest.json'),'normalized_reference_full5fc_equal':True,'candidate_helper_exact_v2':True,'calibration_rows':24,'calibration_actual_Own_calls':actual_own,'raw_submit_rows':sum(steps.values()),'results':computed,'paired_wins':dict(paired),'final_archive_hashes':len(archives),'correlated_baseline_seed_groups':32,'restored_frozen5_qualitytest_and_index_empty':True,'report_correction':'report says3 Own calibration choices; rawactual and parentcountare4. Minimumassert>=3doesnotmeanactual3. Raw/reportoriginalspreserved.','qualification':'Raw evidence parser, no producer/restore rerun; baseline exactwholeai source normalization and helper equality verified. Candidate reference measurement differs structurally from v2 outside Own earlyreturn; allactualactorinputs Own/None, no Other; boundedrouting source review stillneeded, no broadstrength/NC10acceptance.'}
Path('/tmp/noname-parent-nc10-fixed-reference-audit.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps(result,ensure_ascii=False,indent=2))
