import json,pathlib,hashlib
from accept import ROOT,OUT,CORE,save,sha,stat_id
old=json.loads((OUT.parent/'parent-before-asset-manifest.json').read_text());new=json.loads((ROOT/'content/original/asset-manifest.json').read_text());prior=json.loads((OUT/'adopted-resource-stats-before.json').read_text());assert new['assets'][:12359]==old['assets'][:12359]
for k in set(old)-{'assets','sources','trees'}:assert new[k]==old[k],k
assert new['sources']['core']==old['sources']['core'];assert new['sources']['skin']==old['sources']['skin'];assert set(new['sources'])==set(old['sources'])|{'compat'};assert new['sources']['compat']['head'] is None;assert new['sources']['compat']['root']=='content/original-audio-compat';assert new['trees'][:len(old['trees'])]==old['trees']
old_alias=[a for a in old['assets'] if a['path'].startswith('audio-declared/')];new_alias=[a for a in new['assets'] if a['path'].startswith('audio-declared/')];assert len(old_alias)==len(new_alias)==23393
unchanged=[];changed=[]
for a,b in zip(old_alias,new_alias):
 if a==b:unchanged.append(a)
 else:
  assert a['source_category']=='core' and b['source_category']=='compat';assert b['path']==str(pathlib.PurePosixPath(a['path']).with_suffix('.m4a'));assert b['source_path']==str(pathlib.PurePosixPath(a['source_path']).with_suffix('.m4a'));changed.append({'old':a,'new':b})
assert len(unchanged)==23383 and len(changed)==10
backing=[a for a in new['assets'] if a['source_category']=='compat' and not a['path'].startswith('audio-declared/')];assert len(backing)==5;assert sum(a['size'] for a in backing)==263579;assert len(new['assets'])==35757
anomalies=[];count=0
for a in old['assets'][:12359]+unchanged:
 source=CORE/a['source_path'];target=ROOT/'content/original/assets'/a['path'];now={'source':stat_id(source),'target':stat_id(target)};before=prior[a['path']]
 if now!=before:
  hs=sha(source);ht=sha(target);assert hs==ht==a['sha256'];anomalies.append({'path':a['path'],'before':before,'after':now,'source_sha256':hs,'target_sha256':ht})
 assert now['source']['size']==now['target']['size']==a['size'];count+=1
checks=[];compat=ROOT/'content/original-audio-compat'
for a in backing+[r['new'] for r in changed]:
 assert set(a)=={'kind','path','size','source_category','source_path','sha256'};input_file=compat/a['source_path'];target=ROOT/'content/original/assets'/a['path'];assert sha(input_file)==sha(target)==a['sha256'];assert input_file.stat().st_size==target.stat().st_size==a['size'];checks.append({'path':a['path'],'source_path':a['source_path'],'size':a['size'],'sha256':a['sha256'],'input_target_same_inode':stat_id(input_file)==stat_id(target)})
assert all(set(a)=={'kind','path','size','source_category','source_path','sha256'} for a in new['assets']);assert sum(a['kind']=='audio' and a['path']==a['source_path'] and a['source_category']=='core' for a in new['assets'])==9652
expected_aliases={a['path'] for a in new_alias};actual={'audio-declared/'+str(p.relative_to(ROOT/'content/original/assets/audio-declared')) for p in (ROOT/'content/original/assets/audio-declared').rglob('*') if p.is_file()};assert actual==expected_aliases
save('changed-ten-aliases.json',changed);save('bounded-compat-byte-checks.json',checks);save('original-stat-anomalies.json',anomalies);save('generated-roots-after.json',{p:stat_id(ROOT/'content/original/assets'/p) for p in ['audio','portraits','audio-declared','audio-compat']});result={'original_direct_objects_exact':12359,'original_stat_qualification_reused':count,'unaffected_alias_objects_order_exact':len(unchanged),'only_affected_aliases':len(changed),'new_backings':len(backing),'new_input_bytes':263579,'new_targeted_identity_checks':len(checks),'unaffected_rehashed_anomalies':len(anomalies),'raw_audio_count':9652,'alias_count':23393,'total_manifest_rows':35757,'original_raw5_decode_repeated':False,'generated_directory_metadata_relinked_as_authorized':True};save('bounded-resource-proof.json',result);print(json.dumps(result))
