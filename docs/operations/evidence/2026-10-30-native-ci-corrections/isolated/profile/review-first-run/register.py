from pathlib import Path
import json,re
src=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-10-30-native-ci-preparation/first-publication')
out=Path('../evidence/perf-profile/review-first-run')
run=json.loads((src/'later-run.json').read_text());jobs=json.loads((src/'later-jobs.json').read_text());arts=json.loads((src/'later-artifacts.json').read_text());parent_sha=json.loads((src/'first-run-record-sha256.json').read_text())
assert run['id']==41053688083 and run['status']=='completed' and run['conclusion']=='failure'
assert run['head_sha']=='9dbc65530feb695688d3160dc38b6cf64ad39ecd'
assert jobs['total_count']==len(jobs['jobs'])==10
byname={j['name']:j for j in jobs['jobs']};assert len(byname)==10
assert all(j['run_id']==run['id'] and j['head_sha']==run['head_sha'] and j['status']=='completed' for j in jobs['jobs'])
fails=sorted(j['name'] for j in jobs['jobs'] if j['conclusion']=='failure')
assert fails==sorted(['gates-ubuntu','perf-ubuntu-latest','perf-macos-latest','desktop-windows-latest','desktop-ubuntu-latest'])
assert sum(j['conclusion']=='success' for j in jobs['jobs'])==5
assert arts['total_count']==len(arts['artifacts'])==1
art=arts['artifacts'][0];assert art['name']=='installer-macos-latest' and art['size_in_bytes']==32791200 and art['expired'] is False
assert art['workflow_run']['id']==run['id'] and art['workflow_run']['head_sha']==run['head_sha']
raws={};plain={}
strip=lambda s:re.sub(r'\x1b\[[0-?]*[ -/]*[@-~]','',s)
for name in ['gates-ubuntu','perf-ubuntu','perf-macos','desktop-windows','desktop-ubuntu','native-web','desktop-macos']:
 f=f'run41053688083-{name}-raw.log';lines=[strip(l) for l in (src/f).read_text().splitlines()];plain[name]='\n'.join(lines)
 assert run['head_sha'] in plain[name] and 'rustc 1.99.1' in plain[name]
 selections=[]
 for n,l in enumerate(lines,1):
  msg=re.sub(r'^\S+Z\s+','',l)
  if re.search(r'^rustc 1\.|^node: |^\+ pnpm 9|Installed package|##\[group\]Run (cargo run|cargo test|pnpm build|pnpm smoke|pnpm test)|panicked at|慢连接应|test result: FAILED|REGRESSION|result=FAIL|Process completed with exit code|error LGHT0311|invalid name|Bundling |Finished 2 bundles|Artifact.*(successfully|download URL)|^\{"total":82|^# (tests|pass|fail)|forced to run on Node.js 24',msg):selections.append({'line':n,'text':l})
 raws[name]={'file':str(src/f),'parent_recorded_sha256_not_recomputed':parent_sha[f],'selected_lines':selections,'conclusion':byname[name if name in byname else name+'-latest']['conclusion']}
for name in ['gates-ubuntu','native-web','desktop-windows','desktop-ubuntu','desktop-macos']:assert 'node: v20.20.2' in plain[name] and '+ pnpm 9.15.9' in plain[name]
assert 'native_extensions_ws_slow_spectator_does_not_block_commands' in plain['gates-ubuntu'] and 'Elapsed(())' in plain['gates-ubuntu'] and 'Process completed with exit code 101' in plain['gates-ubuntu']
skipped=[s['name'] for s in byname['gates-ubuntu']['steps'] if s['conclusion']=='skipped'];assert len([s for s in skipped if s.startswith('Gate ')])==4
for name in ['perf-ubuntu','perf-macos']:
 assert 'Run cargo run -q -p benchmark-runner -- tier-report server-baseline' in plain[name]
 assert len(re.findall(r'REGRESSION (start-hash-p50|snapshot-restore|observer-projection)',plain[name]))==3
 assert 'Process completed with exit code 1' in plain[name]
assert 'LGHT0311' in plain['desktop-windows'] and "code page '1252'" in plain['desktop-windows']
assert 'invalid name "无名杀-next"' in plain['desktop-ubuntu']
assert '{"total":82,"failed":0}' in plain['native-web'] and '# tests 156' in plain['native-web'] and '# pass 156' in plain['native-web'] and '# fail 0' in plain['native-web']
assert 'vue-tsc --noEmit && vite build' in plain['native-web']
assert 'native engine pkg (web) deployed' in plain['native-web'] and 'WASM package written' in plain['native-web']
assert 'Finished 2 bundles' in plain['desktop-macos'] and 'Final size is 32791200 bytes' in plain['desktop-macos']
initial=json.loads((src/'execution/result.json').read_text());published=json.loads((src/'decision25-scoped-execution/result.json').read_text());decision=json.loads((src.parent/'parent-user-publication-decision25.json').read_text())
assert initial['push_exit']==1 and published['push_exit']==0 and published['remote_after_refs']['refs/heads/native-convergence']==run['head_sha']
assert published['remote_before_refs']['refs/heads/main']==published['remote_after_refs']['refs/heads/main']=='47347c6608d651660ae9459e59eb529b3cef277f'
assert decision['decision']==25 and decision['no_force'] and decision['approved_branch']=='native-convergence'
errors={}
for f in ['later-runs.stderr','later-workflows.stderr','later-check-runs.stderr','later-check-suites.stderr','run41053688083-gates-ubuntu.stderr','run41053688083-perf-ubuntu.stderr','run41053688083-perf-macos.stderr']:
 errors[f]=(src/f).read_text()
summary={'registration_validated':True,'input_scope':'Saved API JSON and seven raw logs only; no network/Git/build/hash commands executed','run':{k:run[k] for k in ['id','html_url','head_sha','head_branch','event','status','conclusion','run_attempt','created_at','updated_at']},'jobs':[{'id':j['id'],'name':j['name'],'status':j['status'],'conclusion':j['conclusion'],'url':j['html_url'],'steps':j['steps']} for j in jobs['jobs']],'success_count':5,'failure_count':5,'artifact':art,'gates_skipped':skipped,'raw_evidence':raws,'observation_errors_not_CI_results':errors,'publication':{'decision':25,'initial_hook_push_exit':1,'scoped_push_exit':0,'remote_main_unchanged':True,'initial_count_zero_not_observed':published['actions_total_count']==0,'uncompressed_bytes_not_wire_estimate':decision['reachable_uncompressed_bytes_not_wire_estimate']},'parent_sha_manifest':str(src/'first-run-record-sha256.json'),'hash_policy':'Parent supplied SHA values cited; reviewer did not recompute hashes.','limitations':['Three build jobs assessed from job/step API only; no invented test totals.','No artifact downloaded or installed; no signing/notarization/runtime certification.','No A330 or subsequent local result is input to this registration.']}
(out/'registration.json').write_text(json.dumps(summary,indent=2,ensure_ascii=False)+'\n')
print(json.dumps({'validated':True,'run':run['id'],'head':run['head_sha'],'status':run['status'],'conclusion':run['conclusion'],'jobs':10,'success':5,'failure':5,'raw_logs':len(raws),'artifact':art['name'],'artifact_bytes':art['size_in_bytes'],'parent_sha_cited_not_recomputed':True},indent=2))
