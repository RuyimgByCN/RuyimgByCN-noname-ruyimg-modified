import argparse, datetime, hashlib, json, os, pathlib, re, shutil, subprocess, tempfile
p=argparse.ArgumentParser();p.add_argument('sha');p.add_argument('old');p.add_argument('label');a=p.parse_args()
assert re.fullmatch('[0-9a-f]{40}',a.sha) and re.fullmatch('[0-9a-f]{40}',a.old)
assert re.fullmatch('[a-z0-9-]+',a.label)
root=pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
ev=root/'docs/operations/evidence/2026-11-13-queue0-gates/publication';ev.mkdir(exist_ok=True)
url='https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified.git';ref='refs/heads/native-convergence'
record=ev/(a.label+'.json');log=ev/(a.label+'.log')
assert not record.exists() and not log.exists()
def read(argv): return subprocess.check_output(argv,cwd=root,text=True).strip()
assert read(['git','rev-parse','HEAD'])==a.sha
assert read(['git','ls-remote',url,ref]).split()==[a.old,ref]
assert subprocess.run(['git','merge-base','--is-ancestor',a.old,a.sha],cwd=root).returncode==0
common=pathlib.Path(read(['git','rev-parse','--git-common-dir']));common=common if common.is_absolute() else root/common
original=common/'hooks/pre-push';original_bytes=original.read_bytes()
configuration=read(['git','config','--local','--list'])
hook='''#!/usr/bin/env python3
import sys
expected_url = %r
expected_sha = %r
expected_old = %r
expected_ref = %r
rows = [line.split() for line in sys.stdin.read().splitlines()]
allowed = sys.argv[1:] == [expected_url, expected_url] and rows == [[expected_sha, expected_sha, expected_ref, expected_old]]
if not allowed:
    print("Exact publication guard rejected unexpected URL, SHA, ref, or remote parent", file=sys.stderr)
sys.exit(0 if allowed else 1)
'''%(url,a.sha,a.old,ref)
meta={'started_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'authorization':'Decision 25 and queue item 0','approved_url':url,'approved_sha':a.sha,'expected_remote_old':a.old,'target_ref':ref,'original_hook_sha256':hashlib.sha256(original_bytes).hexdigest(),'selftests':[]}
with tempfile.TemporaryDirectory(prefix='noname-queue0-exact-push-') as directory:
    guard=pathlib.Path(directory)/'pre-push';guard.write_text(hook);guard.chmod(0o700)
    (ev/(a.label+'-guard.py')).write_text(hook)
    good=f'{a.sha} {a.sha} {ref} {a.old}\n'
    cases=[('exact',url,url,good,0),('wrong-remote-name','other',url,good,1),('wrong-url',url,url+'/other',good,1),('wrong-local-ref',url,url,good.replace(a.sha+' ', 'refs/heads/main ',1),1),('wrong-local-sha',url,url,f'{a.sha} '+('1'*40)+f' {ref} {a.old}\n',1),('wrong-target',url,url,good.replace(ref,'refs/heads/main'),1),('wrong-remote-parent',url,url,good.replace(a.old,'2'*40),1),('delete',url,url,f'(delete) '+('0'*40)+f' {ref} {a.old}\n',1),('multiple-updates',url,url,good+good,1),('empty',url,url,'',1)]
    for label,name,location,stdin,expected in cases:
        proc=subprocess.run([str(guard),name,location],input=stdin,capture_output=True,text=True,cwd=root)
        assert proc.returncode==expected,(label,proc.returncode,proc.stderr)
        meta['selftests'].append({'case':label,'expected_exit':expected,'actual_exit':proc.returncode,'stderr':proc.stderr})
    command=['git','-c','core.hooksPath='+directory,'push','--porcelain',url,a.sha+':'+ref]
    meta['argv']=command
    with log.open('wb') as stream: proc=subprocess.run(command,cwd=root,stdout=stream,stderr=subprocess.STDOUT)
    meta['exit_code']=proc.returncode
meta['finished_at']=datetime.datetime.now(datetime.timezone.utc).isoformat()
meta['remote_after']=read(['git','ls-remote',url,ref])
meta['original_hook_unchanged']=original.read_bytes()==original_bytes
meta['persistent_git_configuration_unchanged']=read(['git','config','--local','--list'])==configuration
meta['temporary_guard_removed']=not pathlib.Path(directory).exists()
record.write_text(json.dumps(meta,ensure_ascii=False,indent=2)+'\n')
print(log.read_text(errors='replace'))
print(json.dumps({k:v for k,v in meta.items() if k not in ['argv','selftests']},ensure_ascii=False,indent=2))
assert proc.returncode==0 and meta['remote_after'].split()==[a.sha,ref]
assert meta['original_hook_unchanged'] and meta['persistent_git_configuration_unchanged']
