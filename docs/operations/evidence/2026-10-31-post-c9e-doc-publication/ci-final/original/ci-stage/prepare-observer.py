import ast
import datetime
import hashlib
import json
from pathlib import Path
import subprocess
import traceback

OUT=Path(__file__).resolve().parent
src=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-ci-34e83f05-i02-xs39v_py/ci-stage/observe-ci.py').read_text()
src=src.replace("SHA = '34e83f0550bb70ef4af6083697428de7be2f02f8'","SHA = '9f4b74236f97b0b9dc72388b6e08d310d2f26f64'").replace('RUN = 41214274772','RUN = 41227614131').replace('SCOPE_TIME + 53 * 60','SCOPE_TIME + 55 * 60').replace('SCOPE_TIME + 56 * 60','SCOPE_TIME + 58 * 60')
src=src.replace("'Checkpoint scope+17min; deadline warning scope+53min; harvest budget scope+56min, report before60min; no PID kill or watch restart'","'Initial checkpoint beforewatch; one timedcheckpoint scope+17min, then finitewait toscope+55min. No sleep/poll loop/signal/restart. Pendingwatchownership recorded ifbound.'")
src=src.replace('p = subprocess.run(argv, cwd=OUT, env=env, stdout=stdout, stderr=stderr, timeout=min(timeout, max(1, HARD_STOP_AT-time.time())))','p = subprocess.run(argv, cwd=OUT, env=env, stdout=stdout, stderr=stderr)')
src=src.replace("        name += '-transport-read-retry'","        STATE.setdefault('transport_retries', []).append({'source': name, 'endpoint': endpoint, 'reason': (OUT/(name+'.stderr')).read_text(errors='replace'), 'policy': 'One repeated identical read-only GET after recorded EOF/http2/502; no watchrestart or CLI switch'})\n        name += '-transport-read-retry'")
func='''def checkpoint(prefix):
    run = api(BASE + '/actions/runs/' + str(RUN), prefix+'-run')
    verify_run(run)
    jobs = api(BASE + '/actions/runs/' + str(RUN) + '/jobs?per_page=100', prefix+'-jobs')
    artifacts = api(BASE + '/actions/runs/' + str(RUN) + '/artifacts?per_page=100', prefix+'-artifacts')
    record = {'at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'run_id': RUN, 'head_sha': SHA, 'run_status': run['status'], 'run_conclusion': run['conclusion'], 'job_statuses': [{k:j.get(k) for k in ('id','name','status','conclusion','started_at','completed_at')} for j in jobs['jobs']], 'lease_release': json.loads((OUT.parent/'push-stage/lease-release.json').read_bytes()), 'artifact_count': artifacts['total_count']}
    paths = [OUT/(prefix+'-'+kind+'.'+stream) for kind in ('run','jobs','artifacts') for stream in ('stdout','stderr')]
    paths += [OUT.parent/'push-stage'/name for name in ('result.json','lease-release.json','scope.json','push-copy-manifest.json')]
    record['immutable_files'] = [{'path':str(p.relative_to(OUT.parent)), 'bytes':p.stat().st_size, 'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in paths]
    (OUT/(prefix+'-envelope.json')).write_text(json.dumps(record,indent=2)+'\\n')
    return record


def watch(name, env):
    argv = ['gh', 'run', 'watch', str(RUN), '--repo', REPO, '--exit-status', '--interval', '10', '--compact']
    entry = {'command': argv, 'name': name, 'http1': True}
    STATE['commands'].append(entry)
    with (OUT/(name+'.stdout')).open('xb') as stdout, (OUT/(name+'.stderr')).open('xb') as stderr:
        proc = subprocess.Popen(argv,cwd=OUT,env=env,stdout=stdout,stderr=stderr,stdin=subprocess.DEVNULL,start_new_session=True)
        STATE['watch_pid']=proc.pid
        STATE['watch_started_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat()
        STATE['cleanup']='Singlewatch running; owned by finite observer, no other watch'
        save()
        try:
            proc.wait(timeout=max(1,CHECKPOINT_AT-time.time()))
        except subprocess.TimeoutExpired:
            STATE['checkpoint_timed']=checkpoint('checkpoint')
            save()
            try:
                proc.wait(timeout=max(1,WATCH_STOP_AT-time.time()))
            except subprocess.TimeoutExpired:
                STATE['watch_bound_pending']=True
                STATE['watch_ownership']={'pid':proc.pid,'owner':'Parent continuation required; this is the SAME watch, do not restart','stdout':str(OUT/(name+'.stdout')),'stderr':str(OUT/(name+'.stderr')),'no_signal_sent':True}
        entry['exit']=proc.returncode
        if proc.returncode is None:
            STATE['cleanup']='Singlewatch STILL RUNNING; explicit pending ownership recorded; no signal/restart'
        else:
            STATE['cleanup']='Singlewatch naturally exited/reaped; no productcommands or targetcleanup performed'
        save()
    return entry
'''
a=src.index('def watch(');b=src.index('\ntry:\n    run = api',a);src=src[:a]+func+src[b:]
replacement="    STATE['checkpoint_initial']=checkpoint('initial')\n    run = json.loads((OUT/'initial-run.stdout').read_bytes())\n    initial_jobs = json.loads((OUT/'initial-jobs.stdout').read_bytes())\n    STATE['initial_jobs'] = [{'id': j['id'], 'name': j['name'], 'status': j['status'], 'conclusion': j['conclusion']} for j in initial_jobs['jobs']]\n    save()\n"
# Preserve the failed generated candidate by reproducing the original mistaken match.
a=src.index("    run = api(BASE + '/actions/runs/'");b=src.index("    if run['status'] != 'completed':",a);failed=src[:a]+replacement+src[b:]
(OUT/'observer-prelaunch-invalid.py.raw').write_text(failed)
try:
    ast.parse(failed)
    raise AssertionError('Expected original SyntaxError was not reproduced')
except SyntaxError as exc:
    diagnostic={'recorded_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'phase':'Prelaunchast validation; observerlaunchcount0/watchlaunchcount0','argv':['python3','-B','- (generator through bash heredoc)'],'candidate_provenance':'Byte reconstruction using the same owned generator transformations and erroneous search from original tool call; original tool error remains in tool transcript, not a fullsession export','exception_type':type(exc).__name__,'message':exc.msg,'line':exc.lineno,'offset':exc.offset,'text':exc.text,'formatted_exception':''.join(traceback.format_exception_only(type(exc),exc)),'supervisor_direction':'Correct private generator, preserve failure, unique outertry match +compile/selfcheck before ONEoriginalwatch; actuallaunch/runtimefailure muststop, nofallback'}
    assert exc.lineno==121 and exc.msg=='invalid syntax'
    (OUT/'observer-prelaunch-failure.json').write_text(json.dumps(diagnostic,indent=2)+'\n')
checkpoint_ast=ast.dump(ast.parse(func).body[0],include_attributes=False)
needle="\ntry:\n    run = api(BASE + '/actions/runs/'"
assert src.count(needle)==1
outer=src.index(needle);a=outer+len('\ntry:\n');b=src.index("    if run['status'] != 'completed':",a);corrected=src[:a]+replacement+src[b:]
tree=ast.parse(corrected);compile(corrected,str(OUT/'observe-ci.py'),'exec')
assert ast.dump(next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='checkpoint'),include_attributes=False)==checkpoint_ast
assert len([n for n in tree.body if isinstance(n,ast.Try)])==1
assert not (OUT/'observe-ci.py').exists() and not (OUT/'observer-launch.json').exists() and not (OUT/'watch.stdout').exists()
assert 'while proc.poll' not in corrected and 'sleep(' not in corrected and 'killpg' not in corrected
(OUT/'observer-prelaunch-validation.json').write_text(json.dumps({'validated_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'compile_ast_passed':True,'unique_outertry_match':True,'checkpoint_body_ast_unchanged':True,'observer_launch_count_before_validation':0,'watch_launch_count_before_validation':0,'invalid_candidate_sha256':hashlib.sha256(failed.encode()).hexdigest(),'corrected_candidate_sha256':hashlib.sha256(corrected.encode()).hexdigest(),'no_sleep_poll_loop_or_signal':True},indent=2)+'\n')
(OUT/'observe-ci.py').write_text(corrected)
with (OUT/'observer.stdout').open('xb') as a,(OUT/'observer.stderr').open('xb') as b:proc=subprocess.Popen(['python3','-B',str(OUT/'observe-ci.py')],stdout=a,stderr=b,start_new_session=True)
(OUT/'observer-launch.json').write_text(json.dumps({'pid':proc.pid,'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'observer_launch_count':1,'one_original_watch_policy':True,'validated_script_sha256':hashlib.sha256(corrected.encode()).hexdigest()},indent=2)+'\n')
print('Corrected/compiledofflinevalidation passed; ONEoriginalobserver launched PID',proc.pid)
