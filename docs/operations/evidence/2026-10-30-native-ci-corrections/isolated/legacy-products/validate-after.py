import pathlib,subprocess,json,hashlib,os,shutil
p=pathlib.Path('../products').resolve(); target=pathlib.Path('../target').resolve(); env=os.environ.copy(); env.update(CARGO_TARGET_DIR=str(target),CARGO_INCREMENTAL='0')
def run(name,argv,locked=True):
 command=(['python3','/tmp/noname-run-serialized.py'] if locked else [])+argv
 with (p/(name+'.log')).open('w') as f:r=subprocess.run(command,env=env,stdout=f,stderr=subprocess.STDOUT)
 (p/(name+'.exit')).write_text(str(r.returncode)+'\n')
 with (p/'commands.jsonl').open('a') as f:f.write(json.dumps({'name':name,'argv':command,'exit':r.returncode,'disk_free':shutil.disk_usage(p).free})+'\n')
 for d in target.glob('**/incremental'):
  if d.is_dir():shutil.rmtree(d)
 return r.returncode
files=subprocess.check_output(['git','ls-files','-z'],text=True).split('\0')
identity={f:hashlib.sha256(pathlib.Path(f).read_bytes()).hexdigest() for f in files if f and (f.startswith(('crates/','apps/benchmark-runner/','content/')) or f in ['Cargo.toml','Cargo.lock'])}
(p/'after-source-sha256.json').write_text(json.dumps(identity,indent=2,sort_keys=True))
assert run('build-after',['cargo','build','--offline','--locked','--release','-j','2','-p','benchmark-runner'])==0
shutil.copyfile(target/'release/benchmark-runner',p/'after-benchmark-runner'); (p/'after-benchmark-runner').chmod(0o755)
(p/'after-binary.sha256').write_text(hashlib.sha256((p/'after-benchmark-runner').read_bytes()).hexdigest()+'\n')
deps=target/'release/deps'; args=['rustc','--edition=2021','-C','opt-level=3','-C','lto=true','-C','codegen-units=1','-L','dependency='+str(deps)]
for crate in ['engine_core','engine_api','content_compiler','serde_json']:
 libs=list(deps.glob('lib'+crate+'-*.rlib')); assert len(libs)==1,(crate,libs)
 args+=['--extern',crate+'='+str(libs[0])]
args += [str(p/'cost-harness.rs'),'-o',str(p/'after-cost-harness')]
assert run('harness-after-build',args)==0
run('profile-after',[str(p/'after-cost-harness')])
run('traces-after',[str(p/'after-cost-harness'),'traces'])
run('catalog-tests',['cargo','test','--offline','--locked','-j','2','-p','engine-core','--lib','content::standard_pack::tests'])
run('affected-tests',['cargo','test','--offline','--locked','-j','2','-p','engine-core','-p','engine-ai','-p','benchmark-runner','-p','conformance','--no-fail-fast'])
run('conformance',['cargo','run','--offline','--locked','-j','2','-p','conformance'])
(p/'validation-driver.done').write_text('done\n')
