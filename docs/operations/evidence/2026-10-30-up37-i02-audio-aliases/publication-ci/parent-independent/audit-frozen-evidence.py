from pathlib import Path
import hashlib, json, re, tempfile

root = Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-ci-a1f72508-observation-gjutdrsj')
final = root / 'final-stage'
out = Path(tempfile.mkdtemp(prefix='noname-parent-ci41100475767-audit-'))
manifest = json.loads((final / 'copy-manifest.json').read_text())
assert manifest['manifest_excludes_itself'] is True
assert len(manifest['files']) == manifest['file_count_without_manifest'] == 146
for row in manifest['files']:
    path = root / row['path']
    assert path.resolve().is_relative_to(root.resolve())
    data = path.read_bytes()
    assert len(data) == row['bytes']
    assert hashlib.sha256(data).hexdigest() == row['sha256'], row['path']
run = json.loads((final / 'run-final-fetch.stdout').read_bytes())
assert (run['id'], run['head_sha'], run['run_attempt'], run['status'], run['conclusion']) == (
    41100475767, 'a1f7250815eee803d931301ab46207719cb9a15b', 1, 'completed', 'success')
job_list = json.loads((final / 'jobs-final-fetch.stdout').read_bytes())['jobs']
assert len(job_list) == 10 and len({j['id'] for j in job_list}) == 10
results = []
for job in job_list:
    assert (job['status'], job['conclusion'], job['run_id'], job['run_attempt']) == ('completed', 'success', run['id'], 1)
    detail = json.loads((final / f"job-{job['id']}-detail.stdout").read_bytes())
    assert (detail['id'], detail['run_id'], detail['conclusion']) == (job['id'], run['id'], 'success')
    log = (final / f"job-{job['id']}-raw-log.stdout").read_bytes()
    text = log.decode('utf-8')
    rust = [tuple(map(int, m)) for m in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', text)]
    steps = job['steps']
    results.append({'id':job['id'], 'name':job['name'], 'status':job['status'], 'conclusion':job['conclusion'],
                    'log_sha256':hashlib.sha256(log).hexdigest(), 'bytes':len(log), 'crlf':log.count(b'\r\n'),
                    'rust_summary_lines':len(rust), 'rust_summary_totals':tuple(sum(x[i] for x in rust) for i in range(3)),
                    'step_conclusions':{value:sum(s['conclusion']==value for s in steps) for value in ['success','skipped','failure',None]},
                    'step_records':steps})
by_name = {j['name']:j for j in results}
assert by_name['gates-ubuntu']['rust_summary_totals'] == (6894,0,77)
assert by_name['native-web']['rust_summary_totals'] == (1815,0,75)
for name in ['build-ubuntu-latest','build-macos-latest','build-windows-latest']:
    assert by_name[name]['rust_summary_totals'] == (838,0,0)
assert by_name['build-windows-latest']['crlf'] == 3821
assert by_name['desktop-windows-latest']['crlf'] == 2212
artifacts = json.loads((final / 'artifacts-final-fetch.stdout').read_bytes())['artifacts']
assert len(artifacts) == 3
parent = json.loads((final / 'parent-api-snapshot/artifacts.stdout').read_bytes())['artifacts']
def artifact_identity(items):
    return sorted((a['id'],a['name'],a['size_in_bytes'],a['digest'],a['expired'],a['workflow_run']['id'],a['workflow_run']['head_sha']) for a in items)
assert artifact_identity(artifacts) == artifact_identity(parent)
assert all(a['workflow_run']['id']==run['id'] and a['workflow_run']['head_sha']==run['head_sha'] and not a['expired'] for a in artifacts)
summary = {'manifest_digests_verified':146, 'manifest_self_excluded':True, 'run':{k:run[k] for k in ['id','head_sha','run_attempt','status','conclusion','updated_at']},
           'jobs':results,'artifacts':artifact_identity(artifacts), 'raw_log_bytes':sum(j['bytes'] for j in results),
           'jobs_total_not_unique_test_total':True, 'step_nulls_not_filled':True, 'no_network_or_builds':True,
           'old_watches_not_reclassified':True, 'source_root':str(root)}
(out/'independent-results.json').write_text(json.dumps(summary,indent=2)+'\n')
Path('/tmp/noname-parent-ci41100475767-audit-current.json').write_text(json.dumps({'out':str(out),'summary':'independent-results.json'},indent=2)+'\n')
print(json.dumps({'out':str(out),'manifest':146,'jobs':len(results),'log_bytes':summary['raw_log_bytes'],'result':'PASS'},indent=2))
