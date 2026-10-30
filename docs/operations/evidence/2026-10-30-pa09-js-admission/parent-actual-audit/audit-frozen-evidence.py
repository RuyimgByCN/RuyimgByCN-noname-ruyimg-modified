from pathlib import Path
import json, hashlib, tempfile, collections, subprocess, os, socket, datetime
root=Path('/tmp/pa09-webview-5b7ad3ab')
e=root/'evidence'
v=json.loads((e/'actual-webview-report.json').read_text())
manifest=json.loads((e/'actual-freeze-manifest.json').read_text())
for name,expected in manifest['source_and_bundle_files_sha256'].items():
    assert hashlib.sha256((root/name).read_bytes()).hexdigest()==expected,name
for name,expected in manifest['evidence_sha256'].items():
    assert hashlib.sha256((e/name).read_bytes()).hexdigest()==expected,name
calls=v['rawCalls'];trace=v['trace']
assert len(calls)==49
assert collections.Counter(c['command'] for c in calls)=={'native_engine_open':4,'native_engine_request':41,'native_engine_close':4}
assert [c['index'] for c in calls]==list(range(49))
assert all(c['settled'] is True for c in calls)
raw_lines=[json.loads(x) for x in (e/'raw-ipc.ndjson').read_text().splitlines()]
assert raw_lines==[x for x in trace if x['type'] in {'invoke-sent','invoke-reply','invoke-error'}]
pending=[json.loads(x) for x in (e/'pending-counts.ndjson').read_text().splitlines()]
assert len(pending)==128 and max(x['pendingCount'] for x in pending)==8
assert all(len(x['pendingIDs'])==x['pendingCount']<=8 for x in pending)
admission=[json.loads(x) for x in (e/'transport-admission.ndjson').read_text().splitlines()]
assert admission==[x for x in trace if x['type'] in {'transport-accepted','transport-rejected'}]
print('phase counts',dict(collections.Counter(x['phase'] for x in calls)))
print('admission phases',dict(collections.Counter((x['phase'],x['type']) for x in admission)))
burst=[x for x in calls if x['command']=='native_engine_request' and x['phase']=='burst128-status']
assert [x['args']['request']['id'] for x in burst]==list(range(2,10))
assert all(x['ok'] and x['response']['ok'] and x['response']['id']==x['args']['request']['id'] for x in burst)
rejected=[x for x in admission if x['phase']=='burst128-status' and x['type']=='transport-rejected']
assert len(rejected)==120 and all('not submitted' in x['error'] for x in rejected)
assert {x['request']['id'] for x in rejected}==set(range(10,130))
opens=[x['response'] for x in calls if x['command']=='native_engine_open']
assert len(set(opens))==4
closes=[x['args']['resourceId'] for x in calls if x['command']=='native_engine_close']
assert collections.Counter(closes)==collections.Counter(opens)
opening=v['opening']['resourceId'];active=v['active']['resourceId'];fresh=v['active']['freshResourceId']
production=[x for x in calls if x['command']=='native_engine_request' and '-retired-probe' not in x['phase']]
assert not [x for x in production if x['args']['resourceId']==opening]
assert len([x for x in production if x['args']['resourceId']==active and x['phase']=='active-dispose'])==1
assert active!=fresh
assert all(x['tailUnchanged'] for x in rejected)
assert trace[415]['type']=='old-disposed'
assert trace[415]['activeSettled'] is False and trace[415]['pending']==[]
# Inspect identity explicitly through the original invoke call, not fixture summary booleans.
late=trace[422]
assert late['type']=='invoke-reply'
old_call=calls[late['call']]
assert old_call['args']['resourceId']==active and old_call['response']['ok'] is True
assert old_call['args']['request']['id']==2
assert not [x for x in trace if x['type']=='client-delivery' and x.get('transport')==trace[415]['transport'] and x['index']>415]
for source in manifest['source3']:
    p=Path('/tmp/pa09-js-bound-4306bc00')/source['path']
    assert hashlib.sha256(p.read_bytes()).hexdigest()==source['sha256']
# Join every actual raw invoke with its unique sent and terminal trace event.
assert [x['index'] for x in trace]==list(range(len(trace)))
for call in calls:
    sent=[x for x in trace if x['type']=='invoke-sent' and x['call']==call['index']]
    end=[x for x in trace if x['type'] in {'invoke-reply','invoke-error'} and x['call']==call['index']]
    assert len(sent)==len(end)==1
    assert sent[0]['command']==end[0]['command']==call['command']
    assert sent[0]['args']==call['args'] and sent[0]['phase']==call['phase']
    assert end[0]['sentPhase']==call['phase'] and sent[0]['index']<end[0]['index']
    assert (end[0]['type']=='invoke-reply')==call['ok']
    assert end[0].get('response')==call.get('response') and end[0].get('error')==call.get('error')
assert sum(x['type'] in {'invoke-sent','invoke-reply','invoke-error'} for x in trace)==98
assert pending==next(x['counts'] for x in trace if x['type']=='pending-counts')
assert all(x['attempt']==i+1 and x['pendingIDs']==list(range(2,min(i+3,10))) for i,x in enumerate(pending))
assert len([x for x in trace if x['phase']=='burst128-status' and x['type']=='caller-rejected'])==120
assert len([x for x in trace if x['phase']=='burst128-status' and x['type']=='caller-resolved'])==8
micro=[x for x in v['assertions'] if 'settled' in x]
assert len(micro)==1 and micro[0]['settled']==120 and micro[0]['pending']==list(range(2,10)) and micro[0]['passed']
assert len(v['assertions'])==33 and all(x['passed'] for x in v['assertions'])
for rid in opens:
    close=[x for x in calls if x['command']=='native_engine_close' and x['args']['resourceId']==rid]
    probes=[x for x in calls if x['command']=='native_engine_request' and x['args']['resourceId']==rid and x['args']['request']['id']==900001]
    assert len(close)==len(probes)==1 and close[0]['ok'] and not probes[0]['ok'] and close[0]['index']<probes[0]['index']
identity=json.loads((e/'runtime-product-identity.json').read_text())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(Path(identity['binary']))==identity['packaged_binary_sha256']==manifest['packaged_binary_sha256']
build=json.loads((e/'native-build-result.json').read_text())
assert sha(Path(build['binary']))==identity['source_binary_sha256']==build['binary_sha256']
assert sha(e/'compile-time-tauri-config.json')==identity['compile_time_config_sha256']==build['config_sha256']
bundle=Path(identity['bundle']);files={str(p.relative_to(bundle)):sha(p) for p in sorted(bundle.rglob('*')) if p.is_file()}
assert files==identity['bundle_files']
assert hashlib.sha256(json.dumps(files,sort_keys=True).encode()).hexdigest()==identity['bundle_sha256']==manifest['bundle_sha256']
served=json.loads((e/'http-served-assets.json').read_text())
for row in served:
    name=row['path'].split('?')[0].lstrip('/') or 'index.html'
    assert sha(bundle/name)==row['sha256']==files[name] and (bundle/name).stat().st_size==row['bytes']
assert len(served)==5 and not any('worker' in row['path'] or '.wasm' in row['path'] for row in served)
loaded=json.loads((e/'bundle-module-source-proof.json').read_text())['loadedModulesBeforeTransforms']
for name,expected in loaded.items():assert sha(Path(name))==expected,name
copy=json.loads((e/'product-source-copy-proof.json').read_text())
for name,expected in copy['files'].items():
    assert sha(root/'product-web'/name)==expected
    assert sha(Path(copy['copied_from'])/name)==expected
baseline=json.loads((e/'native-source-baseline-proof.json').read_text())
assert len(baseline['files'])==3001 and not baseline['differing_from_a1f']
main=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
env={**os.environ,'GIT_OPTIONAL_LOCKS':'0','GIT_NO_LAZY_FETCH':'1'}
entries={}
for row in subprocess.check_output(['git','ls-tree','-r','-z',baseline['base']],cwd=main,env=env).split(b'\0'):
    if row:
        meta,name=row.split(b'\t',1);entries[name.decode()]=meta.split()[2].decode()
for row in baseline['files']:
    data=(Path('/tmp/pa09-js-bound-4306bc00')/row['path']).read_bytes()
    assert hashlib.sha256(data).hexdigest()==row['sha256']
    blob=hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()
    assert blob==row['base_blob']==entries[row['path']]
changed=subprocess.check_output(['git','diff','--name-only',baseline['base'],'777bf33969e5ecc04b63971adab26be32f12fd28'],cwd=main,env=env,text=True).splitlines()
assert not any(x.endswith('.rs') or x in {'Cargo.lock','Cargo.toml'} or (x.startswith('crates/') and x.endswith('Cargo.toml')) for x in changed)
cleanup=json.loads((e/'cleanup.json').read_text());owner=json.loads((e/'run-owner.json').read_text())
for row in cleanup['owned_state_archived_then_removed']:
    assert not Path(row['path']).exists() and sha(Path(row['archive']))==row['sha256']
for pid in [owner['driver_pid'],owner['app_pid']]:
    try:os.kill(pid,0)
    except ProcessLookupError:pass
    else:raise AssertionError('Owned PID still exists')
with socket.socket() as sock:
    sock.settimeout(1);assert sock.connect_ex(('127.0.0.1',19439))!=0
result={'result':'PASS','raw_calls':49,'raw_by_command':dict(collections.Counter(c['command'] for c in calls)),'burst_IPC_ids':list(range(2,10)),'burst_rejected_ids':list(range(10,130)),'maximum_pending':8,'pending_samples':128,'source_bundle_hashes_verified':len(manifest['source_and_bundle_files_sha256']),'evidence_hashes_verified':len(manifest['evidence_sha256']),'source3_exact':True,'own_resources_each_closed_once':True,'opening_no_request':True,'active_only_one_request':True,'late_success_after_dispose':True,'no_tests_or_GUI_repeated':True,'scope':'Same-event-turn JS admission and asynchronous bridge lifecycle only; no OS pressure/performance/PA09-six-condition claim','raw_to_trace_joins':49,'invoke_trace_events':98,'ndjson_exact':True,'native_baseline_blobs_verified':3001,'a1f_to_777_no_Rust_or_Cargo_delta':True,'binary_config_bundle_served_loaded_sources_verified':True,'cleanup_current_read_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'owned_pids_exited_port_19439_unbound_state_directories_absent':True,'main_movement':'Parent authorized compatibility commit777bf339, not candidate runtime integration'}
out=Path(tempfile.mkdtemp(prefix='noname-parent-pa09-webview-audit-'))
(out/'independent-results.json').write_text(json.dumps(result,indent=2)+'\n')
Path('/tmp/noname-parent-pa09-webview-audit-current.json').write_text(json.dumps({'out':str(out)},indent=2)+'\n')
print(json.dumps({'out':str(out),**result},indent=2))
