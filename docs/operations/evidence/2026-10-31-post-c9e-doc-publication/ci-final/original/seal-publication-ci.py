import datetime
import gzip
import hashlib
import json
from pathlib import Path
import zlib

ROOT=Path(__file__).resolve().parent
summary_path=ROOT/'verified-publication-ci-summary.json'
manifest_path=ROOT/'copy-manifest.json'
s=json.loads(summary_path.read_bytes())
m=json.loads(manifest_path.read_bytes())
state=json.loads((ROOT/'ci-stage/observation-state.json').read_bytes())
assert s['run_id']==41227614131 and s['head_sha']=='9f4b74236f97b0b9dc72388b6e08d310d2f26f64'
assert state['completed'] and not state.get('error') and len(s['jobs'])==10
assert len(s['watch_commands'])==1 and s['watch_commands'][0]['exit'] is not None
assert s['push']['push_exit']==0 and not s['push'].get('preservation_error')
assert all(path.startswith('docs/') for path in s['push']['committed_increment_paths'])
assert all(not job['detail_error'] and not job['log_error'] and job['raw_log'] for job in s['jobs'])
for job in s['jobs']:
    if job['conclusion']=='success':
        assert not job['failed_steps'] and not job['error_annotations']
        if 'rust_tests' in job:assert job['rust_tests']['failed']==0
    assert all(step['status']=='completed' for step in job['steps'])
    if job['name'] in ('build-windows-latest','desktop-windows-latest'):
        assert job['actual_checkout_step2']['status']=='completed'
        if job['conclusion']=='success':assert job['actual_checkout_step2']['conclusion']=='success'
    if job['name'].startswith('perf-') and job['conclusion']=='success':
        assert job['legacy_server_baseline_result_pass'] and len(job['legacy_server_baseline_metrics'])==4
        assert all(v['actual_us']<=v['budget_us'] for v in job['legacy_server_baseline_metrics'])
for artifact in s['artifacts']['artifacts']:
    assert artifact['workflow_run']['id']==s['run_id'] and artifact['workflow_run']['head_sha']==s['head_sha']
for item in m['files']:
    data=(ROOT/item['path']).read_bytes()
    assert len(data)==item['bytes'] and hashlib.sha256(data).hexdigest()==item['sha256'] and f'{zlib.crc32(data)&0xffffffff:08x}'==item['crc32']
    if 'gzip_original_identity' in item:
        raw=gzip.decompress(data);original=item['gzip_original_identity']
        assert len(raw)==original['bytes'] and hashlib.sha256(raw).hexdigest()==original['sha256'] and f'{zlib.crc32(raw)&0xffffffff:08x}'==original['crc32']
for prefix in ('initial','checkpoint'):
    envelope=json.loads((ROOT/'ci-stage'/(prefix+'-envelope.json')).read_bytes())
    for item in envelope['immutable_files']:
        data=(ROOT/item['path']).read_bytes();assert len(data)==item['bytes'] and hashlib.sha256(data).hexdigest()==item['sha256']
seal={'sealed_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'head_sha':s['head_sha'],'run_id':s['run_id'],'run_conclusion':s['run_conclusion'],'manifest_path':manifest_path.name,'manifest_bytes':manifest_path.stat().st_size,'manifest_sha256':hashlib.sha256(manifest_path.read_bytes()).hexdigest(),'summary_path':summary_path.name,'summary_bytes':summary_path.stat().st_size,'summary_sha256':hashlib.sha256(summary_path.read_bytes()).hexdigest(),'listed_current_files_verified':len(m['files']),'primary_copy_files_including_manifest_and_seal':len(m['files'])+2,'private_originals_excluded':len(s['private_originals']),'raw_log_bytes':s['raw_log_bytes'],'step_count':s['step_count'],'current_copy_authority':'Copylistedfiles+manifest+thisseal. Exactgzipbytes; nevernormalize originals. Datedinitial/checkpoint envelopes bindimmutable API/pushfiles. No fullsession/profile/cache included.','prelaunch_failure_provenance':'Exactfailedtoolcommand/result bounduniquetoolCallId, invalidcandidate reconstruction explicitlylabelled; supervisorauthorizedprivatecorrection, AST/compile/checkpointbodyunchanged/launchcount0controls passed beforeONEobserver/watch. No fallback/restart.','prelease_index_history':'fb->626 UNKNOWN beforeparentintentional9905gitadd/commit; nooldfbraw, notctimeonly. Pushwindowcfc raw/logical/flags/stagingstable; no Mainreads afterrelease.','source_scope':'9905docs/evidenceonly; source37+root46 unchanged atnew/parent/workingpushwindow. Cprivateblocked/source/golden/tarsunderdocs areevidence, notproductionimports ornewtestinventory.'}
with (ROOT/'artifact-index-seal.json').open('x') as f:json.dump(seal,f,indent=2);f.write('\n')
print(json.dumps(seal,indent=2))
print('SealSHA256',hashlib.sha256((ROOT/'artifact-index-seal.json').read_bytes()).hexdigest())
