import argparse,datetime,json,pathlib,re,subprocess,time
p=argparse.ArgumentParser();p.add_argument('sha');p.add_argument('label');a=p.parse_args()
assert re.fullmatch('[0-9a-f]{40}',a.sha) and re.fullmatch('[a-z0-9-]+',a.label)
root=pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');out=root/'docs/operations/evidence/2026-11-13-queue0-gates/publication'/a.label
out.mkdir(exist_ok=False)
repo='RuyimgByCN/RuyimgByCN-noname-ruyimg-modified';workflow=401719763
last=None;start=time.monotonic()
for index in range(180):
    cmd=['gh','api',f'repos/{repo}/actions/runs?head_sha={a.sha}&per_page=100']
    proc=subprocess.run(cmd,text=True,capture_output=True)
    stamp=datetime.datetime.now(datetime.timezone.utc).isoformat()
    if proc.returncode:
        (out/f'{index:03}-api-error.json').write_text(json.dumps({'at':stamp,'argv':cmd,'exit_code':proc.returncode,'stderr':proc.stderr},indent=2)+'\n')
        print('API observation failed; will poll same SHA again',flush=True);time.sleep(20);continue
    data=json.loads(proc.stdout)
    (out/f'{index:03}-runs.json').write_text(json.dumps({'at':stamp,'argv':cmd,'data':data},indent=2)+'\n')
    matching=[run for run in data['workflow_runs'] if run['workflow_id']==workflow and run['head_sha']==a.sha]
    assert len(matching)<=1
    if not matching:
        state=('awaiting-ci',)
    else:
        run=matching[0]
        state=(run['id'],run['run_attempt'],run['status'],run['conclusion'])
        if run['status']=='completed':
            jobs_cmd=['gh','api',f'repos/{repo}/actions/runs/{run["id"]}/attempts/{run["run_attempt"]}/jobs?per_page=100']
            jobs_proc=subprocess.run(jobs_cmd,text=True,capture_output=True)
            assert jobs_proc.returncode==0,jobs_proc.stderr
            jobs=json.loads(jobs_proc.stdout)
            (out/'jobs.json').write_text(json.dumps(jobs,indent=2)+'\n')
            result={'verified_at':stamp,'source_head':a.sha,'run_id':run['id'],'attempt':run['run_attempt'],'workflow_id':workflow,'url':run['html_url'],'status':run['status'],'conclusion':run['conclusion'],'duration_observed_seconds':round(time.monotonic()-start,3),'jobs':[{'id':j['id'],'name':j['name'],'status':j['status'],'conclusion':j['conclusion']} for j in jobs['jobs']]}
            (out/'result.json').write_text(json.dumps(result,indent=2)+'\n')
            print(json.dumps(result,indent=2),flush=True)
            if run['conclusion']!='success':
                failed=subprocess.run(['gh','run','view',str(run['id']),'--repo',repo,'--attempt',str(run['run_attempt']),'--log-failed'],text=True,capture_output=True)
                (out/'failed-job-log.txt').write_text(failed.stdout+failed.stderr)
                raise SystemExit(2)
            raise SystemExit(0)
    if state!=last:print(stamp,state,flush=True);last=state
    time.sleep(20)
raise SystemExit('Observation limit reached; this does not terminate the remote run. Continue observing same SHA/run.')
