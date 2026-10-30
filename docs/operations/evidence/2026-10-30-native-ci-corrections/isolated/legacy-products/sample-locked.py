import pathlib,subprocess,fcntl,json
p=pathlib.Path(__file__).resolve().parent
with pathlib.Path('/tmp/noname-phase4-heavy-command.lock').open('a') as lock:
 fcntl.flock(lock,fcntl.LOCK_EX)
 with (p/'profile-loop-before-locked.log').open('w') as f:
  child=subprocess.Popen([str(p/'before-cost-harness'),'loop'],stdout=f,stderr=subprocess.STDOUT)
  sample=subprocess.run(['sample',str(child.pid),'3','1','-file',str(p/'sample-before-locked.txt')],capture_output=True,text=True)
  (p/'sample-before-locked.log').write_text(sample.stdout+sample.stderr)
  result=child.wait()
 (p/'sample-before-locked.json').write_text(json.dumps({'sample_exit':sample.returncode,'loop_exit':result,'note':'Original sample-before.txt retained as exploratory unguarded capture; this capture holds global heavy lock throughout.'},indent=2))
