from pathlib import Path
import json,re,hashlib,datetime
r=Path(Path('/tmp/c331-phase7-root').read_text().strip());e=r/'evidence/phase9h-baseline-validation';c8=r/'evidence/phase9c';oldroot=c8/'c8-command-traces-v3';newroot=e/'current-command-traces';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();rows=[]
source=(r/'worktree/crates/engine-native/tests/replay_scenarios.rs').read_text();mismatches=re.findall(r'^([^\s]+) actual=([a-f0-9]{64}) expected=([a-f0-9]{64})',(e/'golden.log').read_text(),re.M)
def delta(a,b):
 i=next((i for i,(x,y)in enumerate(zip(a,b))if x!=y),min(len(a),len(b)))
 return {'character':i,'old':a[max(0,i-100):i+240],'new':b[max(0,i-100):i+240]}
def outcome(raw):
 if raw=='':return {'discriminant':'NoCommandOutcome'}
 m=re.fullmatch(r'Accepted \{ state_hash: \[([0-9, ]+)\] \}',raw)
 if m:
  values=[int(v.strip())for v in m[1].split(',')];assert len(values)==32 and all(0<=v<=255 for v in values)
  return {'discriminant':'Accepted','state_hash':values}
 if raw.startswith('Pending { choice: ')and raw.endswith(' }'):return {'discriminant':'Pending','complete_payload':raw}
 if raw.startswith('Rejected(')and raw.endswith(')'):return {'discriminant':'Rejected','complete_rejection_reason':raw}
 if raw=='Suspended':return {'discriminant':'Suspended'}
 raise AssertionError('Unparsed command outcome: '+raw)
for name,actual,expected in mismatches:
 olds=sorted((oldroot/name).glob('*.json'));news=sorted((newroot/name).glob('*.json'));assert [p.name for p in olds]==[p.name for p in news],name
 first=None;business=[];bindings=[];outcomes=[]
 for old,new in zip(olds,news):
  a=json.loads(old.read_text());b=json.loads(new.read_text());assert a['command']==b['command']and a['phase']==b['phase']and a['ruleset_version']==b['ruleset_version'],name
  keys=[k for k in ['core','events','rng','views','choices','referee']if a[k]!=b[k]]
  if keys:
   entry={'step':old.stem,'command':a['command'],'fields':keys,'differences':{k:delta(a[k],b[k])for k in keys},'old_json':str(old),'new_json':str(new)};business.append(entry)
   if first is None:first=entry
  oa=outcome(a['outcome_raw']);ob=outcome(b['outcome_raw']);assert oa['discriminant']==ob['discriminant'],(name,old.name,oa,ob)
  if oa['discriminant'] in ['Pending','Rejected','Suspended']:assert oa==ob,(name,old.name,oa,ob)
  outcomes.append({'step':old.stem,'discriminant':oa['discriminant'],'old_raw':a['outcome_raw'],'new_raw':b['outcome_raw'],'raw_equal':a['outcome_raw']==b['outcome_raw'],'parsed_old':oa,'parsed_new':ob,'all_nonhash_result_fields_equal':True})
  bindings.append({'step':old.stem,'old_path':str(old),'old_sha256':sha(old),'new_path':str(new),'new_sha256':sha(new),'old_snapshot_sha256':sha(old.with_suffix('.snapshot.txt')),'new_snapshot_sha256':sha(new.with_suffix('.snapshot.txt')),'old_archive_sha256':sha(old.with_suffix('.archive.ngsa')),'new_archive_sha256':sha(new.with_suffix('.archive.ngsa'))})
 fn=re.search(r'\(\s*"'+re.escape(name)+r'",\s*(scenario_\w+)',source).group(1);body=source[source.index('fn '+fn):];seed=int(re.search(r'Engine::new\((\d+)',body).group(1))
 status='business_equal_at_all_observed_commands_and_final'if first is None else'actual_business_difference_requires_source_disposition'
 rows.append({'scenario':name,'seed':seed,'function':fn,'source_versions':{'C8_base':'5fc0f77403d90fa4555156bf01722450cbe7bf3b','C8_overlay_manifest':'f5829f4023ee465e5e97fe6ad8f998b1480999cd727c348b310686dcdf72876e','current_source_manifest':str(e/'source-manifest.json'),'ruleset':331,'archive':106,'abi':758,'ws':6},'command_capture_count':len(olds),'paired_inputs_and_phase_identical':True,'original_expected_hash':expected,'current_actual_hash':actual,'first_business_difference':first,'business_differing_captures':business,'disposition':status,'captures':bindings,'outcomes':outcomes,'limits':'All raw fields retained. No complete scheduling/archive equality asserted; no unobserved transient/fuel or global rules/source equivalence; command rejection equality is not privacy evidence.'})
assert len(rows)==12
out={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'baseline_producer_reused_without_rerun':str(c8/'c8-observed3.log'),'baseline_manifest_sha256':'f5829f4023ee465e5e97fe6ad8f998b1480999cd727c348b310686dcdf72876e','current_red_log':str(e/'golden.log'),'compared_fields':['complete Core','complete log','RNG','all seat views','all projected choices','referee choice','command outcome discriminant; complete Pending payload; complete Rejected reason; Accepted hash retained separately'],'normalization':[],'golden_expectations_changed':False,'rows':rows}
(e/'golden-business-and-outcome-dispositions.json').write_text(json.dumps(out,indent=2,ensure_ascii=False)+'\n')
print(json.dumps([{'scenario':x['scenario'],'captures':x['command_capture_count'],'first_business':x['first_business_difference'],'outcome_discriminants_payloads_equal':True}for x in rows],ensure_ascii=False))
