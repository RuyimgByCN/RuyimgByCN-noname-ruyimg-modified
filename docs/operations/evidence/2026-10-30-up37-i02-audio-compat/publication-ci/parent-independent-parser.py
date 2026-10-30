from pathlib import Path
import json, hashlib, gzip, zlib, re, tempfile
root=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-ci-777bf339-publication-y7r1p6p5')
m=json.loads((root/'copy-manifest.json').read_text())
assert len(m['files'])==86 and m['manifest_excludes_itself']
for row in m['files']:
    data=(root/row['path']).read_bytes()
    assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['sha256']
    assert f'{zlib.crc32(data):08x}'==row['crc32']
run=json.loads((root/'ci-stage/run-final.stdout').read_text())
assert (run['id'],run['head_sha'],run['run_attempt'],run['status'],run['conclusion'])==(41114612841,'777bf33969e5ecc04b63971adab26be32f12fd28',1,'completed','failure')
jobs=json.loads((root/'ci-stage/jobs-final.stdout').read_text())['jobs']
assert len(jobs)==10 and sum(j['conclusion']=='success' for j in jobs)==9
assert [j['name'] for j in jobs if j['conclusion']=='failure']==['desktop-windows-latest']
steps=[s for j in jobs for s in j['steps']]
assert len(steps)==130 and sum(s['conclusion']=='skipped' for s in steps)==8 and not any(s['conclusion'] is None for s in steps)
assert [(j['id'],s['number']) for j in jobs for s in j['steps'] if s['conclusion']=='failure']==[(124265455086,7)]
producer=json.loads((root/'verified-publication-ci-summary.json').read_text())
by_id={j['id']:j for j in producer['jobs']};records=[]
for j in jobs:
    detail=json.loads((root/f"ci-stage/job-{j['id']}-detail.stdout").read_text())
    assert (detail['id'],detail['run_id'],detail['conclusion'])==(j['id'],run['id'],j['conclusion'])
    data=gzip.decompress((root/f"ci-stage/job-{j['id']}-raw-log.stdout.gz").read_bytes())
    expected=by_id[j['id']]
    assert len(data)==expected['log_bytes'] and hashlib.sha256(data).hexdigest()==expected['log_sha256']
    assert f'{zlib.crc32(data):08x}'==expected['log_crc32'] and data.count(b'\r\n')==expected['log_crlf_count']
    text=data.decode('utf-8');rust=[tuple(map(int,x)) for x in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)]
    totals=[sum(x[i] for x in rust) for i in range(3)]
    if j['name']=='gates-ubuntu':assert totals==[6894,0,77]
    if j['name']=='native-web':assert totals==[1815,0,75]
    if j['name'].startswith('build-'):assert totals==[838,0,0]
    if j['name']=='desktop-windows-latest':
        assert 'audio compat fixed provenance/recipe/tool binding differs' in text
        assert 'original-audio-resources.mjs:59:11' in text and 'Process completed with exit code 1' in text
    records.append({'id':j['id'],'name':j['name'],'conclusion':j['conclusion'],'log_bytes':len(data),'log_sha256':hashlib.sha256(data).hexdigest(),'log_crc32':f'{zlib.crc32(data):08x}','crlf':data.count(b'\r\n'),'rust_totals':totals})
artifacts=json.loads((root/'ci-stage/artifacts-final.stdout').read_text())['artifacts']
assert {x['name'] for x in artifacts}=={'installer-macos-latest','installer-ubuntu-latest'}
assert all(x['workflow_run']['id']==run['id'] and x['workflow_run']['head_sha']==run['head_sha'] and not x['expired'] for x in artifacts)
out=Path(tempfile.mkdtemp(prefix='noname-parent-ci41114612841-audit-'))
result={'result':'PASS','meaning':'Exact registration of9successful jobs and1realWindowsresourceprepare failure; not a green CI result','manifest_files_verified':86,'run':{k:run[k] for k in ['id','head_sha','run_attempt','status','conclusion','updated_at']},'jobs':records,'steps':{'total':130,'success':121,'failure':1,'skip':8,'null':0},'artifacts':[{'id':a['id'],'name':a['name'],'bytes':a['size_in_bytes'],'digest':a['digest']} for a in artifacts],'warnings_ignored_not_cleared':True,'failure_root_cause_not_proved_by_original_runner_bytes':True,'no_network_tests_or_builds':True}
(out/'independent-results.json').write_text(json.dumps(result,indent=2)+'\n')
Path('/tmp/noname-parent-ci41114612841-audit-current.json').write_text(json.dumps({'out':str(out)},indent=2)+'\n')
print(json.dumps({'out':str(out),'manifest':86,'jobs':'9success/1failure','steps':result['steps']},indent=2))
