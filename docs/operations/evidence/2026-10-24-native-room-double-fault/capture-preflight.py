import importlib.util,sys,subprocess,pathlib,json,re,datetime,hashlib,time,contextlib
root=pathlib.Path.cwd();out=pathlib.Path('/tmp/noname-next-room-double-fault-20261024')
source=root/'scripts/preflight.py'
spec=importlib.util.spec_from_file_location('preflight',source);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
original=subprocess.run;checks=[];start=datetime.datetime.now().astimezone().isoformat();clock=time.monotonic()
with (out/'preflight-full-01.log').open('w',buffering=1) as summary,(out/'preflight-full-01-raw.log').open('w',buffering=1) as raw:
 summary.write('START '+start+'\n');raw.write('Original source '+str(source)+'\nInterpreter '+sys.executable+'\n')
 def captured(argv,*args,**kwargs):
  n=len(checks)+1;name=module.CHECKS[n-1][0];at=datetime.datetime.now().astimezone().isoformat();tick=time.monotonic()
  summary.write('START_CHECK '+str(n)+' '+name+' '+at+'\n');raw.write('\nCHECK '+str(n)+' '+name+'\nARGV '+repr(argv)+'\n')
  proc=original(argv,*args,**kwargs)
  raw.write('STDOUT\n'+(proc.stdout or '')+'\nSTDERR\n'+(proc.stderr or '')+'\nEXIT '+str(proc.returncode)+'\n')
  rec=dict(index=n,name=name,argv=argv,start=at,end=datetime.datetime.now().astimezone().isoformat(),seconds=round(time.monotonic()-tick,3),exit=proc.returncode)
  found=re.search(r'Ran (\d+) tests? in ([0-9.]+)s',(proc.stderr or '')+(proc.stdout or ''))
  if found:rec['unittest']={'executed':int(found[1]),'seconds':float(found[2]),'ok':bool(re.search(r'(?m)^OK\s*$',proc.stderr or ''))}
  checks.append(rec);summary.write('END_CHECK '+str(n)+' exit '+str(proc.returncode)+'\n');return proc
 module.subprocess.run=captured;sys.argv=[str(source)]
 try:
  with contextlib.redirect_stdout(summary),contextlib.redirect_stderr(summary):code=module.main()
 except BaseException as error:
  summary.write('EXCEPTION '+repr(error)+'\n');code=1
 summary.write('END '+datetime.datetime.now().astimezone().isoformat()+'\nEXIT '+str(code)+'\n')
meta=dict(command=[sys.executable,'-u','-B',str(source)],method='unmodified preflight.main full mode; subprocess.run transparently records original output; no repository script changes',start=start,end=datetime.datetime.now().astimezone().isoformat(),seconds=round(time.monotonic()-clock,3),source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),complete=len(checks)==len(module.CHECKS),configured=len(module.CHECKS),executed=len(checks),passed=sum(c['exit']==0 for c in checks),failed=sum(c['exit']!=0 for c in checks),skipped=0,exit=code,checks=checks)
(out/'preflight-full-01.json').write_text(json.dumps(meta,indent=2,ensure_ascii=False)+'\n');print(json.dumps({k:v for k,v in meta.items() if k!='checks'},ensure_ascii=False));raise SystemExit(code)
