from pathlib import Path
import subprocess,json,os,hashlib
S=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-10-30-pa09-js-admission/final-gates/serialized-wrapper-observed-at-seal.py');H=Path('/tmp/noname-run-serialized.py');L=Path('/tmp/noname-phase4-heavy-command.lock');P=Path(json.loads(Path('/tmp/noname-parent-serialization-recovery-current.json').read_bytes())['root']);raw=S.read_bytes();assert not H.exists() and not L.exists()
ps=subprocess.check_output(['ps','-axo','pid=,ppid=,comm=,command=']).decode();rows={}
for line in ps.splitlines():
 parts=line.strip().split(None,3)
 if len(parts)==4:rows[int(parts[0])]=(int(parts[1]),parts[2],parts[3])
ancestors={os.getpid()};parent=os.getppid()
while parent and parent not in ancestors:
 ancestors.add(parent);parent=rows.get(parent,(0,'',''))[0]
active=[{'pid':pid,'comm':row[1],'command':row[2]} for pid,row in rows.items() if pid not in ancestors and Path(row[1]).name in {'cargo','rustc','wasm-pack'}];assert not active,active
lf=subprocess.run(['/usr/sbin/lsof','-nP','+L1','-Fpcn'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=20);assert lf.returncode==0 and b'noname-phase4-heavy-command.lock' not in lf.stdout
fd=os.open(L,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600);os.close(fd)
with H.open('xb') as f:f.write(raw)
assert H.read_bytes()==raw
trace=P/'lock-test.jsonl';payload=P/'lock-test.py';payload.write_text('import sys,json,time\nfrom pathlib import Path\np=Path(sys.argv[1]);tag=sys.argv[2]\nwith p.open("a") as f:f.write(json.dumps({"tag":tag,"event":"start","t":time.monotonic()})+"\\n")\ntime.sleep(0.2)\nwith p.open("a") as f:f.write(json.dumps({"tag":tag,"event":"end","t":time.monotonic()})+"\\n")\n')
procs=[]
for tag in ['A','B']:
 out=(P/(tag+'.stdout')).open('wb');err=(P/(tag+'.stderr')).open('wb');procs.append((subprocess.Popen(['python3','-B',str(H),'python3','-B',str(payload),str(trace),tag],stdout=out,stderr=err),out,err))
for p,out,err in procs:assert p.wait(timeout=10)==0;out.close();err.close()
events=[json.loads(x) for x in trace.read_text().splitlines()];assert [x['event'] for x in events]==['start','end','start','end'];assert events[0]['tag']==events[1]['tag'] and events[2]['tag']==events[3]['tag'] and events[0]['tag']!=events[2]['tag']
r=subprocess.run(['python3','-B',str(H),'python3','-c','import sys;sys.exit(7)'],stdout=subprocess.PIPE,stderr=subprocess.PIPE);(P/'exit7.stdout').write_bytes(r.stdout);(P/'exit7.stderr').write_bytes(r.stderr);assert r.returncode==7 and json.loads(r.stdout)['exit']==7
st=L.stat();v={'status':'RESTORED exact archivedhelper; testsPASS','helper':str(H),'source':str(S),'sha256':hashlib.sha256(raw).hexdigest(),'lock':str(L),'lock_dev':st.st_dev,'lock_inode':st.st_ino,'two_concurrent_helpers_mutually_exclusive':True,'exit7_propagated':True,'before_active_heavy':active,'unlinked_old_lock_matches':0,'cause_original_missing':'unknown; no deletion by this recovery','first_probe_failure':'own ancestor shell command contained cargo literals; pre-mutation false positive, corrected by processcomm+ancestorPID identity','profile':'Originalargv/env/profile preserved; no new flags, no alternate lock; worker-owned3GiB monitoring unchanged'};(P/'restoration.json').write_text(json.dumps(v,indent=2)+'\n');print(json.dumps(v,indent=2))
