from pathlib import Path
import json, hashlib

root=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
e=root/'docs/operations/evidence/2026-10-30-up37-i02-audio-compat'
r=e/'main-acceptance'
out=e/'parent-independent'
out.mkdir(exist_ok=True)
index=json.loads((r/'artifact-sha256.json').read_text())
assert len(index)==64
for name,expected in index.items():
    assert hashlib.sha256((r/name).read_bytes()).hexdigest()==expected,name
admission=json.loads((e/'parent-main-source-admission.json').read_text())
for name,expected in admission['source14'].items():
    assert hashlib.sha256((root/name).read_bytes()).hexdigest()==expected,name
old=json.loads((e/'parent-before-asset-manifest.json').read_text())
new=json.loads((root/'content/original/asset-manifest.json').read_text())
assert len(old['assets'])==35752 and len(new['assets'])==35757
assert old['assets'][:12359]==new['assets'][:12359]
old_rows={x['path']:x for x in old['assets']};new_rows={x['path']:x for x in new['assets']}
assert len(old_rows)==len(old['assets']) and len(new_rows)==len(new['assets'])
removed=old_rows.keys()-new_rows.keys()
assert len(removed)==10
added=[new_rows[k] for k in new_rows.keys()-old_rows.keys() if k.startswith('audio-compat/')]
changed=[(old_rows[k],new_rows[str(Path(k).with_suffix('.m4a'))]) for k in removed]
assert all(k.endswith('.mp3') and k.startswith('audio-declared/') and old_rows[k]['kind']=='audio' for k in removed)
assert all(old_rows[k]==new_rows[k] for k in old_rows.keys() & new_rows.keys())
assert new_rows.keys()-old_rows.keys()=={x['path'] for x in added}|{b['path'] for a,b in changed}
assert len(added)==5 and all(x['path'].startswith('audio-compat/') and x['kind']=='audio' for x in added)
assert len(changed)==10 and all(a['kind']==b['kind']=='audio' and b['path'].startswith('audio-declared/') and b['source_category']=='compat' for a,b in changed)
assert sum(x['path'].startswith('audio-declared/') for x in new['assets'])==23393
old_aliases=[x for x in old['assets'] if x['path'].startswith('audio-declared/') and x['path'] not in {a['path'] for a,b in changed}]
new_aliases=[x for x in new['assets'] if x['path'].startswith('audio-declared/') and x['path'] not in {b['path'] for a,b in changed}]
assert old_aliases==new_aliases and len(old_aliases)==23383
new_bytes=0
for row in added+[b for a,b in changed]:
    data=(root/'content/original/assets'/row['path']).read_bytes()
    assert len(data)==row['size'] and hashlib.sha256(data).hexdigest()==row['sha256']
    if row in added:new_bytes+=len(data)
assert new_bytes==263579
commands=json.loads((r/'commands.json').read_text());assert len(commands)==17 and all(x['exit']==0 for x in commands)
assert json.loads((r/'original-stat-anomalies.json').read_text())==[]
vue=json.loads((r/'chrome-vue-result.json').read_text())
assert vue['passed'] and vue['actualProductionVue'] and vue['rawAudioCount']==9652 and len(vue['decoded'])==5
assert vue['exceptions']==[] and vue['normalResourceMocked'] is False and vue['originalRawFiveRetested'] is False
http=json.loads((r/'http-identities.json').read_text());assert len(http)==17
http_by={x['path']:x for x in http}
for row in http:
    if row['path'] in {'catalog.json','asset-manifest.json'}:data=(root/'content/original'/row['path']).read_bytes()
    else:data=(root/'content/original/assets'/row['path']).read_bytes()
    assert row['status']==200 and row['bytes']==len(data) and row['sha256']==hashlib.sha256(data).hexdigest()
for decoded in vue['decoded']:
    asset=new_rows[decoded['path']]
    assert decoded['status']==200 and decoded['sha256']==asset['sha256']==http_by[decoded['path']]['sha256']
    assert decoded['bytes']==0 # raw capture reads byteLength after decodeAudioData detaches input
    assert decoded['statistics']['frames']>0
    assert decoded['statistics']['channels']==2
    assert all(x['nonfinite']==0 and x['peak']>0 for x in decoded['statistics']['channelStatistics'])
    play=decoded['play']
    assert play['ended'] and play['error'] is None,play
assert len(vue['selectionAndRecovery'])==8
assert any(x['injected404'] for x in vue['routes'])
assert all(x['actual'].startswith('http://127.0.0.1:64033/') for x in vue['routes'])
summary={'result':'PASS','source14_exact':True,'artifact_digests_verified':64,'wrapper_commands':17,'all_exit_zero':True,'old_direct_prefix':12359,'unaffected_aliases_exact_order':23383,'changed_aliases':10,'new_backings':5,'new_backing_bytes':new_bytes,'new_manifest_rows':35757,'vue_full_decodes_and_ended':5,'vue_recovery_snapshots':8,'http_identities':17,'bytes_capture_limitation':'decoded[*].bytes is 0 because decodeAudioData detached the buffer after the pre-decode status/SHA/size assertion; actual full sizes independently bound by http-identities.json and manifest. Raw evidence retained.','no_new_media_or_build_execution':True,'fullpreflight_not_yet_run':True}
(out/'functional-independent-results.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2))
