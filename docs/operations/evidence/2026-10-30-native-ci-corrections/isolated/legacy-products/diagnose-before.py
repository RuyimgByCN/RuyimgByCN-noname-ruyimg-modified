import subprocess, pathlib, json, shutil, time, fcntl
p=pathlib.Path('../products').resolve(); target=pathlib.Path('../target').resolve()
def run(name,argv):
    with (p/(name+'.log')).open('w') as f: r=subprocess.run(argv,stdout=f,stderr=subprocess.STDOUT)
    (p/(name+'.exit')).write_text(str(r.returncode)+'\n')
    return r.returncode
assert run('original-release-tier-report',['python3','/tmp/noname-run-serialized.py',str(p/'before-benchmark-runner'),'tier-report','server-baseline']) in [0,1]
deps=target/'release/deps'
args=['rustc','--edition=2021','-C','opt-level=3','-C','lto=true','-C','codegen-units=1','-L','dependency='+str(deps)]
for crate in ['engine_core','engine_api','content_compiler','serde_json']:
    libs=list(deps.glob('lib'+crate+'-*.rlib')); assert len(libs)==1,(crate,libs)
    args+=['--extern',crate+'='+str(libs[0])]
args += [str(p/'cost-harness.rs'),'-o',str(p/'before-cost-harness')]
assert run('harness-before-build',['python3','/tmp/noname-run-serialized.py']+args)==0
(p/'disk-after-harness-before.json').write_text(json.dumps({'free':shutil.disk_usage(p).free}))
assert run('profile-before',['python3','/tmp/noname-run-serialized.py',str(p/'before-cost-harness')])==0
assert run('traces-before',['python3','/tmp/noname-run-serialized.py',str(p/'before-cost-harness'),'traces'])==0
with pathlib.Path('/tmp/noname-phase4-heavy-command.lock').open('a') as lock, (p/'profile-loop-before.log').open('w') as f:
    fcntl.flock(lock, fcntl.LOCK_EX)
    child=subprocess.Popen([str(p/'before-cost-harness'),'loop'],stdout=f,stderr=subprocess.STDOUT)
    run('sample-before',['sample',str(child.pid),'3','1','-file',str(p/'sample-before.txt')]); result=child.wait()
(p/'profile-loop-before.exit').write_text(str(result)+'\n')
