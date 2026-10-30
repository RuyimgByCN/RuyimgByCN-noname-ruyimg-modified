import hashlib,json,os,subprocess,time
from pathlib import Path
root=Path.cwd()
evidence=(root/'../evidence').resolve()
steps=[]
def run(name,args):
    started=time.monotonic()
    with (evidence/(name+'.log')).open('w') as output:
        result=subprocess.run(args,stdout=output,stderr=subprocess.STDOUT)
    steps.append({'name':name,'command':args,'exit':result.returncode,'seconds':time.monotonic()-started})
    (evidence/'local-validation-results.json').write_text(json.dumps(steps,indent=2)+'\n')
    print(name,result.returncode,flush=True)
    return result.returncode

def protected():
    names=subprocess.check_output(['git','ls-files','-z','content/original','apps/web-client/src/generated','apps/web-client/public/original']).decode().split('\0')
    return {name:hashlib.sha256((root/name).read_bytes()).hexdigest() for name in names if name}

before=protected()
(evidence/'protected-before.json').write_text(json.dumps(before,indent=2)+'\n')
assert not (root/'apps/web-client/node_modules').exists()
assert not (root/'../archive').exists()
assert not (root/'content/original/assets').exists()
assert run('fresh-missing-dependency',['node','scripts/sync-original-content.mjs','--verify-only']) != 0
(root/'apps/web-client/node_modules').symlink_to('/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client/node_modules',target_is_directory=True)
assert run('fresh-missing-source',['node','scripts/sync-original-content.mjs','--verify-only']) != 0
(root/'../archive').symlink_to('/Users/ruyimg/Documents/Claude/sanguosha/archive',target_is_directory=True)
assert run('fresh-missing-assets',['node','scripts/sync-original-content.mjs','--verify-only']) != 0
assert run('resources-bootstrap',['node','scripts/sync-original-content.mjs','--resources-only']) == 0
assert run('resources-verify',['node','scripts/sync-original-content.mjs','--resources-only','--verify-only']) == 0
assert run('gate4-tests',['node','--test','scripts/test_original_content_order.mjs','scripts/test_original_resources.mjs']) == 0
assert run('gate4-full-verify',['node','scripts/sync-original-content.mjs','--verify-only']) == 0
after=protected()
(evidence/'protected-after.json').write_text(json.dumps(after,indent=2)+'\n')
assert before == after
print('Protected tracked files unchanged:',len(before),flush=True)
(evidence/'protected-result.json').write_text(json.dumps({'files':len(before),'unchanged':before==after},indent=2)+'\n')
