from pathlib import Path
import json,hashlib,struct,subprocess,os
R=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-efhmbq7e/evidence/phase4');ROOT=R.parent.parent
O=Path('/tmp/noname-parent-c331-phase4-audit');O.mkdir(exist_ok=True)
REPO=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
def sha(b):return hashlib.sha256(b).hexdigest()
def identity(p):
 if not p.exists():return {'path':str(p),'exists':False}
 b=p.read_bytes();return {'path':str(p),'exists':True,'bytes':len(b),'sha256':sha(b)}
def load(p):return json.loads(p.read_bytes())
def differences(a,b,path='$'):
 if a==b:return []
 if type(a)!=type(b):return [path+':type']
 if isinstance(a,dict):return [path+'.'+k+':presence' for k in a.keys()^b.keys()]+[d for k in a.keys()&b.keys() for d in differences(a[k],b[k],path+'.'+k)]
 if isinstance(a,list):
  if len(a)!=len(b):return [path+':length']
  return [d for i,(x,y) in enumerate(zip(a,b)) for d in differences(x,y,f'{path}[{i}]')]
 return [path]
def archive(h):
 b=bytes.fromhex(h);assert b[:4]==b'NGSA';fmt=struct.unpack_from('<H',b,4)[0];rule,n=struct.unpack_from('<QQ',b,6);assert len(b)==22+n+32
 payload_rule,revision,core_rule=struct.unpack_from('<QQQ',b,22);assert rule==payload_rule==core_rule
 return {'bytes':len(b),'sha256':sha(b),'magic':'NGSA','format':fmt,'rule_u64':str(rule),'revision':revision,'payload_bytes':n,'container':'plain NGSA hex, not a compression format','checksum_scope':'length/header consistency verified; no fresh BLAKE3 or semantic Native decode'}
def record(o):
 s=o['state'];w=s['wal'];base=archive(w['base_snapshot_hex']);entries=w['entries'];rows=[]
 for e in entries:
  variant=next(iter(e['action']));v=e['action'][variant];rows.append({'variant':variant,'choice_id':v.get('choice_id'),'selection':v.get('selection'),'actor':e['actor'],'dedup_id':e['dedup_id'],'expected_revision':e['expected_revision'],'revision':e['revision'],'state_hash_hex':bytes(e['state_hash']).hex()})
 return {'state_keys':list(s),'archive':archive(s['snapshot_hex']),'wal_base':base,'entries':rows,'timeout':s.get('timeout'),'receipt_count':len(s.get('command_receipts',[])),'wal_revision_contiguous':all(e['expected_revision']==base['revision']+i and e['revision']==base['revision']+i+1 for i,e in enumerate(entries))}
mp=R/'final-products2/server-manual-before-real-old-deadline.json';tp=R/'final-products2/server-real-timeout.json';m=load(mp);t=load(tp)
manual={'identity':identity(mp),'records':{k:record(m[k]) for k in ['accepted','after_old_deadline','final']},'state_comparisons':{k:differences(m['accepted']['state'],m[k]['state']) for k in ['after_old_deadline','final']},'stored_room_comparisons':{k:differences(m['accepted'],m[k]) for k in ['after_old_deadline','final']},'times':{k:m[k] for k in ['old_deadline','manual_committed_at','observed_after_old_deadline','new_choice_deadline']}}
a=manual['times'];assert a['manual_committed_at']<a['old_deadline']<=a['observed_after_old_deadline']<a['new_choice_deadline'];assert not any(manual['state_comparisons'].values())
timer={'identity':identity(tp),'records':{k:record(t[k]) for k in ['initial','expired']},'initial_snapshot_equals_initial_base':t['initial']['state']['snapshot_hex']==t['initial']['state']['wal']['base_snapshot_hex'],'base_identical_initial_expired':t['initial']['state']['wal']['base_snapshot_hex']==t['expired']['state']['wal']['base_snapshot_hex']}
assert timer['records']['initial']['wal_base']['revision']==10 and timer['records']['expired']['archive']['revision']==15
old=R/'old-wal-reader';i=load(old/'identity-final.json');source=R.parent/'phase3/server-ws-permissions-retry.json';raw=source.read_bytes();start,end=i['embedded_byte_range'];chunk=raw[start:end];assert sha(raw)==i['new_record_source_sha256'] and sha(chunk)==i['owned_copy_sha256']
inputrows=[]
for name in ['negative/331.json','startup3-negative/rooms/331.json']:
 p=old/name;b=p.read_bytes();assert b==chunk;inputrows.append(identity(p))
audit=load(old/'base-production-audit.json');prod=[]
for row in audit['files']:
 p=ROOT/'old330'/row['path'];current=p.read_bytes();base=subprocess.run(['git','-C',str(REPO),'cat-file','blob',audit['base']+':'+row['path']],check=True,capture_output=True,env={**os.environ,'GIT_OPTIONAL_LOCKS':'0'}).stdout
 assert sha(current)==row['old330_sha256']==row['base_sha256']==sha(base)
 prod.append({'path':row['path'],'bytes':len(current),'sha256':sha(current),'matches_actual_git_base':True})
binaries=[]
for p,key in [(old/'original330-wal-reader','reader_binary_sha256'),(ROOT/'target/old330/debug/game-server','production_server_binary_sha256')]:
 v=identity(p);v['recorded_sha256']=i[key]
 if v['exists']:v['matches_recorded']=v['sha256']==i[key];assert v['matches_recorded']
 binaries.append(v)
oldresult={'identity_final':identity(old/'identity-final.json'),'source_record':identity(source),'embedded_byte_range_zero_based_halfopen':[start,end],'embedded_bytes':len(chunk),'embedded_sha256':sha(chunk),'exact_input_copies':inputrows,'production_audit_identity':identity(old/'base-production-audit.json'),'production_file_count':len(prod),'production_files':prod,'production_recorded_differences':audit['differences'],'binaries':binaries,'reader_result':load(old/'reader-result.json'),'startup_result3':load(old/'startup-result3.json'),'harness':identity(old/'reader-harness.rs')}
assert oldresult['harness']['sha256']==i['harness_sha256']
for row in oldresult['startup_result3']:
 p=old/('positive/330.json' if row['case']=='positive' else 'negative/331.json');assert identity(p)['sha256']==row['input_sha256']
 row['fresh_input_identity']=identity(p);row['log_identity']=identity(Path(row['log']))
result={'manual':manual,'timer':timer,'old_reader':oldresult,'scope':['No experiments, producer execution, builds, or tests run.','Base is a compacted WAL starting checkpoint revision10, plain NGSA105; these retained cases do not exercise threshold-triggered rebase.','Complete stored state comparisons are exact with no excluded fields, including full archive hex, WAL, packages, control, receipts.','Rust frozen fixture equal_snapshot clears only auth_generation before complete StateSnapshot assert_eq; parent has not independently semantically decoded all Native snapshot fields or replayed WAL. Per-entry original variant, choice, hash and revision chains independently parsed here.']}
(O/'manual-timeout-oldreader.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'output':str(O/'manual-timeout-oldreader.json'),'manual_state_diff':manual['state_comparisons'],'timer_initial_base':timer['records']['initial']['wal_base'],'timer_final_revision':timer['records']['expired']['archive']['revision'],'old_production_count':len(prod),'binaries':binaries},indent=2))
