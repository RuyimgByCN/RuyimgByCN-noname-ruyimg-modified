import json,os,shutil,subprocess,time
from pathlib import Path
root=Path.cwd()
evidence=(root/'../evidence').resolve()
fixture=root/'.scratch/ci-dependencies'
fixture.mkdir(parents=True,exist_ok=True)
for name in ['package.json','pnpm-lock.yaml']:
    shutil.copyfile(root/'apps/web-client'/name,fixture/name)
pnpm='/Users/ruyimg/Library/pnpm/.tools/pnpm/9.15.9/bin/pnpm'
env={**os.environ,'XDG_CACHE_HOME':str(fixture/'cache'),'XDG_STATE_HOME':str(fixture/'state'),'XDG_CONFIG_HOME':str(fixture/'config'),'PNPM_HOME':str(fixture/'pnpm-home'),'CI':'true'}
command=[pnpm,'--dir',str(fixture),'install','--frozen-lockfile','--store-dir',str(fixture/'store')]
started=time.monotonic()
with (evidence/'pnpm-frozen-install.log').open('w') as output:
    result=subprocess.run(command,env=env,stdout=output,stderr=subprocess.STDOUT)
summary={'command':command,'pnpm_version':'9.15.9','exit':result.returncode,'seconds':time.monotonic()-started,'package_unchanged':(fixture/'package.json').read_bytes()==(root/'apps/web-client/package.json').read_bytes(),'lock_unchanged':(fixture/'pnpm-lock.yaml').read_bytes()==(root/'apps/web-client/pnpm-lock.yaml').read_bytes()}
(evidence/'pnpm-frozen-result.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2),flush=True)
assert result.returncode == 0
assert summary['package_unchanged'] and summary['lock_unchanged']
(root/'apps/web-client/node_modules').unlink()
(root/'apps/web-client/node_modules').symlink_to(fixture/'node_modules',target_is_directory=True)
with (evidence/'gate4-pnpm9-verify.log').open('w') as output:
    result=subprocess.run(['node','scripts/sync-original-content.mjs','--verify-only'],stdout=output,stderr=subprocess.STDOUT)
print('Gate4 with fresh pnpm9 dependencies:',result.returncode,flush=True)
assert result.returncode == 0
