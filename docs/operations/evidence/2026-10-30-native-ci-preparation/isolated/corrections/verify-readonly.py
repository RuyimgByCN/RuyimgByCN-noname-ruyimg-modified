import hashlib,json,os,subprocess
from pathlib import Path
root=Path.cwd();out=(root/'../evidence/corrections').resolve()
env={**os.environ,'GIT_OPTIONAL_LOCKS':'0','GIT_NO_LAZY_FETCH':'1'}
can=Path('/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname')
gitdir=Path(subprocess.check_output(['git','-C',str(can),'rev-parse','--absolute-git-dir'],env=env).decode().strip())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
modules=root/'apps/web-client/node_modules';lookup=root/'../archive'
assert not modules.exists() and not lookup.exists()
before=sha(gitdir/'index')
try:
    modules.symlink_to('/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client/node_modules',target_is_directory=True)
    lookup.symlink_to('/Users/ruyimg/Documents/Claude/sanguosha/archive',target_is_directory=True)
    command=['node','scripts/sync-original-content.mjs','--verify-only']
    with (out/'gate4-readonly-verify.log').open('w') as f:r=subprocess.run(command,env=env,stdout=f,stderr=subprocess.STDOUT)
    after=sha(gitdir/'index')
    logical=subprocess.run(['git','-C',str(can),'diff','--cached','--quiet','HEAD'],env=env)
    result={'command':command,'exit':r.returncode,'env':{'GIT_OPTIONAL_LOCKS':'0','GIT_NO_LAZY_FETCH':'1'},'canonical_index_before_sha256':before,'canonical_index_after_sha256':after,'canonical_index_bytes_unchanged_in_this_window':before==after,'canonical_logical_index_equals_HEAD':logical.returncode==0,'limits':'Does not erase earlier binary index refresh observation or prove attribution in that earlier concurrent window'}
    (out/'readonly-verify-result.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2),flush=True)
    assert r.returncode==0 and logical.returncode==0 and before==after
finally:
    for p in [modules,lookup]:
        if p.is_symlink():p.unlink()
