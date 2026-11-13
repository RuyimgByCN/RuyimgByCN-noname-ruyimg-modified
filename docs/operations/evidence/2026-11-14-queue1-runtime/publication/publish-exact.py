import datetime,hashlib,json,pathlib,shutil,subprocess,sys,tempfile
root=pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
out=root/'docs/operations/evidence/2026-11-14-queue1-runtime/publication'
name,sha,old=sys.argv[1:]
assert name in ['code-batch-first','documentation-evidence-first']
assert len(sha)==len(old)==40 and all(c in '0123456789abcdef' for c in sha+old)
url='https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified.git'
ref='refs/heads/native-convergence'
meta=out/(name+'.json'); log=out/(name+'.log'); guard_out=out/(name+'-guard.py')
assert not any(p.exists() for p in [meta,log,guard_out])
def read(*argv): return subprocess.check_output(argv,cwd=root,text=True).strip()
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest() if p.is_file() else None
hook=root/read('git','rev-parse','--git-path','hooks/pre-push')
config=root/read('git','rev-parse','--git-path','config')
assert read('git','rev-parse','HEAD')==sha
assert not read('git','status','--porcelain=v1','--untracked-files=no')
assert read('git','ls-remote',url,ref).split()==[old,ref]
subprocess.run(['git','merge-base','--is-ancestor',old,sha],cwd=root,check=True)
old_hook=digest(hook); old_config=digest(config)
assert old_hook=='e349af1ec5ccc1742df7e7d91d387ed2ef086f7c864d8cf7372d0ef66dd49ca7'
assert old_config=='f862ba299c2bbd23c501ebeb0830f53caabc990a4da7e8005570470ff914bc25'
guard='''#!/usr/bin/env python3
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
'''%(url,sha,old,ref)
guard_out.write_text(guard)
row=[sha,sha,ref,old]
cases=[('exact',[url,url],[row],0),('wrong-remote-name',['origin',url],[row],1),('wrong-url',[url,url+'-wrong'],[row],1)]
for i,label in enumerate(['wrong-local-ref','wrong-local-sha','wrong-target','wrong-remote-parent']):
 wrong=list(row); wrong[i]=('refs/heads/main' if i==2 else '0'*40)
 cases.append((label,[url,url],[wrong],1))
cases.extend([('delete',[url,url],[['(delete)','0'*40,ref,old]],1),('multiple-updates',[url,url],[row,row],1),('empty',[url,url],[],1)])
checks=[]
for label,args,rows,want in cases:
 r=subprocess.run([sys.executable,str(guard_out),*args],input=''.join(' '.join(a)+'\n' for a in rows),capture_output=True,text=True)
 checks.append({'case':label,'expected_exit':want,'actual_exit':r.returncode,'stderr':r.stderr})
 assert r.returncode==want,label
record={'started_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'authorization':'Decision 25 and goal queue item 10','approved_url':url,'approved_sha':sha,'expected_remote_old':old,'target_ref':ref,'original_hook_sha256':old_hook,'persistent_config_sha256_before':old_config,'selftests':checks}
tmp=pathlib.Path(tempfile.mkdtemp(prefix='noname-q1-jiushi-exact-push-'))
try:
 h=tmp/'pre-push'; h.write_text(guard); h.chmod(0o755)
 argv=['git','-c','core.hooksPath='+str(tmp),'push','--porcelain',url,sha+':'+ref]
 record['argv']=argv
 print(' '.join(argv),flush=True)
 with log.open('wb') as stream: r=subprocess.run(argv,cwd=root,stdout=stream,stderr=subprocess.STDOUT)
 record.update(exit_code=r.returncode,finished_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),remote_after=read('git','ls-remote',url,ref))
finally:
 shutil.rmtree(tmp)
 record.update(original_hook_unchanged=digest(hook)==old_hook,persistent_git_configuration_unchanged=digest(config)==old_config,temporary_guard_removed=not tmp.exists())
 meta.write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n')
print(log.read_text()); print({k:v for k,v in record.items() if k not in ['selftests','argv']})
assert record.get('exit_code')==0 and record.get('remote_after','').split()==[sha,ref]
assert record['original_hook_unchanged'] and record['persistent_git_configuration_unchanged'] and record['temporary_guard_removed']
