import hashlib,json,os,shutil,subprocess,time
from pathlib import Path
os.environ['GIT_OPTIONAL_LOCKS']='0'
root=Path.cwd();out=(root/'../evidence/corrections').resolve();steps=[]
def execute(name,args,env=None):
    start=time.monotonic()
    with (out/(name+'.log')).open('w') as f:r=subprocess.run(args,stdout=f,stderr=subprocess.STDOUT,env=env)
    steps.append({'name':name,'command':args,'exit':r.returncode,'seconds':time.monotonic()-start})
    (out/'original-input-validation.json').write_text(json.dumps(steps,indent=2)+'\n')
    print(name,r.returncode,flush=True);assert r.returncode==0
paths=json.loads(Path('../evidence/protected-before.json').read_text())
def hashes():return {p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in paths}
fixture=root/'.scratch/ci-correction-dependencies';lookup=root/'../archive';modules=root/'apps/web-client/node_modules'
assert not fixture.exists() and not lookup.exists() and not modules.exists()
try:
    before=hashes();assert before==paths
    fixture.mkdir(parents=True)
    for name in ['package.json','pnpm-lock.yaml']:shutil.copyfile(root/'apps/web-client'/name,fixture/name)
    env={**os.environ,'XDG_CACHE_HOME':str(fixture/'cache'),'XDG_STATE_HOME':str(fixture/'state'),'XDG_CONFIG_HOME':str(fixture/'config'),'PNPM_HOME':str(fixture/'pnpm-home'),'CI':'true'}
    execute('pnpm9-frozen',['/Users/ruyimg/Library/pnpm/.tools/pnpm/9.15.9/bin/pnpm','--dir',str(fixture),'install','--frozen-lockfile','--store-dir',str(fixture/'store')],env)
    for name in ['package.json','pnpm-lock.yaml']:assert (fixture/name).read_bytes()==(root/'apps/web-client'/name).read_bytes()
    modules.symlink_to(fixture/'node_modules',target_is_directory=True)
    lookup.symlink_to('/Users/ruyimg/Documents/Claude/sanguosha/archive',target_is_directory=True)
    execute('gate4-tests',['node','--test','scripts/test_original_content_order.mjs','scripts/test_original_resources.mjs'])
    execute('gate4-original-full-verify',['node','scripts/sync-original-content.mjs','--verify-only'])
    after=hashes();assert before==after
    (out/'original-input-hashes.json').write_text(json.dumps({'files':len(before),'sha256_list_before':hashlib.sha256(json.dumps(before,sort_keys=True).encode()).hexdigest(),'sha256_list_after':hashlib.sha256(json.dumps(after,sort_keys=True).encode()).hexdigest(),'same_as_previous_run_protected_input':True,'all_unchanged':before==after},indent=2)+'\n')
finally:
    for p in [modules,lookup]:
        if p.is_symlink():p.unlink()
    if fixture.exists():shutil.rmtree(fixture)
    (out/'original-input-cleanup.json').write_text(json.dumps({'owned_fixture_removed':not fixture.exists(),'owned_lookup_removed':not lookup.exists(),'owned_modules_removed':not modules.exists(),'disk_free_bytes':shutil.disk_usage(root).free},indent=2)+'\n')
