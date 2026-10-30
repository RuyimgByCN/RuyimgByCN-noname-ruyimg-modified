import hashlib,json,os,shutil,subprocess,time
from pathlib import Path
root=Path.cwd();out=(root/'../evidence/perf-profile').resolve();target=(root/'../target-perf').resolve()
assert not target.exists(),'target-perf already has an owner'
target.mkdir();owner={'task':'9375a4e3-45f5-467e-b135-7fef5948d472','root':str(root),'created':time.time()};(target/'owner.json').write_text(json.dumps(owner)+'\n')
(out/'target-owner.json').write_text(json.dumps(owner,indent=2)+'\n')
env=dict(os.environ)
for name in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER']:
    env.pop(name,None)
for name in list(env):
    if name.startswith('CARGO_PROFILE_'):env.pop(name)
env.update({'CARGO_TARGET_DIR':str(target),'CARGO_INCREMENTAL':'0','CARGO_BUILD_JOBS':'2','CARGO_TERM_COLOR':'never','GIT_OPTIONAL_LOCKS':'0','GIT_NO_LAZY_FETCH':'1'})
results=[]
tracked=['apps/benchmark-runner/Cargo.toml','apps/benchmark-runner/src/main.rs','apps/benchmark-runner/src/device_tiers.rs','Cargo.lock','Cargo.toml']
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
before={n:sha(root/n) for n in tracked};(out/'benchmark-source-sha256.json').write_text(json.dumps(before,indent=2)+'\n')
context={name:subprocess.check_output(cmd,text=True).strip() for name,cmd in {'rustc':['rustc','-Vv'],'cargo':['cargo','-V'],'machine':['uname','-a'],'cpu':['sysctl','-n','machdep.cpu.brand_string']}.items()}
context.update({'env_set':{n:env[n] for n in ['CARGO_TARGET_DIR','CARGO_INCREMENTAL','CARGO_BUILD_JOBS','CARGO_TERM_COLOR']},'source_head':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'same_source_and_workload_both_profiles':True,'no_warm_cache_copied':True})
(out/'measurement-context.json').write_text(json.dumps(context,indent=2)+'\n')
try:
 for profile in ['debug','release']:
    profile_args=['--release'] if profile=='release' else []
    command=['cargo','build','--offline','--locked','-j','2','-p','benchmark-runner',*profile_args]
    started=time.monotonic()
    with (out/(profile+'-build.log')).open('w') as f:r=subprocess.run(command,env=env,stdout=f,stderr=subprocess.STDOUT)
    row={'profile':profile,'build_argv':command,'build_exit':r.returncode,'build_seconds':time.monotonic()-started,'disk_free_after_build':shutil.disk_usage(root).free}
    for p in target.glob('**/incremental'):
        if p.is_dir():shutil.rmtree(p)
    (out/'diagnostic-results.json').write_text(json.dumps(results+[row],indent=2)+'\n')
    print(profile,'build exit',r.returncode,'seconds',row['build_seconds'],'free',row['disk_free_after_build'],flush=True)
    if r.returncode!=0:results.append(row);break
    binary=target/profile/'benchmark-runner';row['binary_sha256']=sha(binary);row['binary_bytes']=binary.stat().st_size
    row['runs']=[]
    for scenario in [['tiers'],['tier-report','server-baseline']]:
        argv=[str(binary),*scenario];start=time.monotonic()
        label='tiers' if scenario[0]=='tiers' else 'tier-report'
        with (out/(profile+'-'+label+'.log')).open('w') as f:run=subprocess.run(argv,env=env,stdout=f,stderr=subprocess.STDOUT)
        row['runs'].append({'argv':argv,'exit':run.returncode,'seconds':time.monotonic()-start,'log':profile+'-'+label+'.log'})
        print(profile,label,'exit',run.returncode,flush=True)
    results.append(row);(out/'diagnostic-results.json').write_text(json.dumps(results,indent=2)+'\n')
    # Each profile has separate outputs; keep only the logs and binary identity.
    shutil.rmtree(target/profile)
    print(profile,'outputs removed, free',shutil.disk_usage(root).free,flush=True)
finally:
    assert before=={n:sha(root/n) for n in tracked},'benchmark inputs changed'
    assert json.loads((target/'owner.json').read_text())==owner
    shutil.rmtree(target)
    (out/'cleanup.json').write_text(json.dumps({'target_removed':not target.exists(),'benchmark_source_unchanged':True,'disk_free_bytes':shutil.disk_usage(root).free,'binary_copies_retained':False},indent=2)+'\n')
