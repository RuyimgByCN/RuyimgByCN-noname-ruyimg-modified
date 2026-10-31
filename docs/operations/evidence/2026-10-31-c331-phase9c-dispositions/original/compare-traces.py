from pathlib import Path
import json,re,hashlib,difflib
r=Path(Path('/tmp/c331-phase7-root').read_text().strip());e=r/'evidence/phase9c';oldroot=e/'c8-command-traces-v3';newroot=e/'checks-final/current-command-traces-v3';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();rows=[]
source=(r/'worktree/crates/engine-native/tests/replay_scenarios.rs').read_text();mismatches=re.findall(r'^([^\s]+) actual=([a-f0-9]{64}) expected=([a-f0-9]{64})',(e/'checks-final/golden-traces.log').read_text(),re.M)
def delta(a,b):
 i=next((i for i,(x,y)in enumerate(zip(a,b))if x!=y),min(len(a),len(b)))
 return {'character':i,'old':a[max(0,i-100):i+240],'new':b[max(0,i-100):i+240]}
def relations(text):
 return [{'id':int(m[0]),'parent_raw':m[1],'kind':m[2],'data':m[3]}for m in re.findall(r'Frame \{\s+id: (\d+),\s+parent: (None|Some\(\s*\d+,?\s*\)),\s+kind: (\w+),\s+data: (\w+)',text)]
for name,actual,expected in mismatches:
 olds=sorted((oldroot/name).glob('*.json'));news=sorted((newroot/name).glob('*.json'));assert [p.name for p in olds]==[p.name for p in news],name
 first=None;firstschedule=None;business_diff_steps=[];bindings=[]
 for old,new in zip(olds,news):
  a=json.loads(old.read_text());b=json.loads(new.read_text());assert a['command']==b['command']and a['phase']==b['phase']and a['ruleset_version']==b['ruleset_version'],name
  keys=[k for k in ['core','events','rng','views','choices','referee']if a[k]!=b[k]]
  if keys:
   entry={'step':old.stem,'command':a['command'],'fields':keys,'differences':{k:delta(a[k],b[k])for k in keys},'old_json':str(old),'new_json':str(new)};business_diff_steps.append(entry)
   if first is None:first=entry
  if firstschedule is None and a['resolution']!=b['resolution']:
   firstschedule={'step':old.stem,'old_json':str(old),'new_json':str(new),'difference':delta(a['resolution'],b['resolution']),'old_root_window_relations':relations(a['resolution']),'new_root_window_relations':relations(b['resolution'])}
  bindings.append({'step':old.stem,'old_path':str(old),'old_sha256':sha(old),'new_path':str(new),'new_sha256':sha(new)})
 fn=re.search(r'\(\s*"'+re.escape(name)+r'",\s*(scenario_\w+)',source).group(1);body=source[source.index('fn '+fn):];seed=int(re.search(r'Engine::new\((\d+)',body).group(1))
 status='business_equal_at_all_observed_commands_and_final'if first is None else'approved_individual_chain_unlink_order_change'
 if first is not None:assert name=='chain_fire_propagation'and first['fields']==['events'],(name,first)
 rows.append({'scenario':name,'seed':seed,'function':fn,'source_versions':{'C8_base':'5fc0f77403d90fa4555156bf01722450cbe7bf3b','C8_overlay_manifest':'f5829f4023ee465e5e97fe6ad8f998b1480999cd727c348b310686dcdf72876e','current_source_manifest':str(e/'pre-checks2/source-manifest.json'),'ruleset':331,'archive':106,'abi':758,'ws':6,'packages':'standard catalog from frozen base + exact overlay; scenario uses Engine::new, no extensions'},'command_capture_count':len(olds),'paired_command_inputs_identical':True,'original_expected_hash':expected,'current_actual_hash':actual,'first_business_difference':first,'business_differing_captures':business_diff_steps,'first_scheduling_difference':firstschedule,'disposition':status,'ready_for_independent_disposition_review':True,'unproved':'No global rule/source parity conclusion; native competition outside three migrated declarations remains incomplete; internal transients between command boundaries not normalized or equated.','source_cause':('Liuli game.js34767-34810: unlink each damaged player in own changeHp; collect/recheck siblings after parent completion. C29 v2 canonical damage/chain + accepted C9b chain correction. Parent damage record remains before changeHp per content.ts12092-12120.'if first else'No observed business delta: every complete core/RNG/event log/view/choice/referee capture is byte-equal. Raw scheduling/archives differ and remain preserved; no unknown-field normalization.'),'captures':bindings})
assert len(rows)==12
out={'baseline_positive_log':str(e/'c8-observed3.log'),'current_red_log':str(e/'checks-final/golden-traces.log'),'compared_fields':['complete GameStateCore Debug','full event log Debug','RuleRng state+calls','all player views','all projected choices','referee choice'],'normalization':[],'outcome_raw':'Captured unchanged in JSON; accepted state_hash excluded from business comparisons because it is precisely the disputed golden. No rejected-error equality used as privacy evidence.','rows':rows};(e/'golden-business-dispositions.json').write_text(json.dumps(out,indent=2,ensure_ascii=False)+'\n');print([(x['scenario'],x['seed'],x['command_capture_count'],x['disposition'],x['first_business_difference']['step']if x['first_business_difference']else None)for x in rows])
