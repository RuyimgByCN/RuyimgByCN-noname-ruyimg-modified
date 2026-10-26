import subprocess,sys,pathlib,json,datetime,time,os
out=pathlib.Path(__file__).parent
name=sys.argv[1];argv=sys.argv[2:]
env=os.environ.copy();env['PATH']='/Users/ruyimg/.cargo/bin:/usr/local/bin:'+env['PATH']
start=datetime.datetime.now().astimezone().isoformat();tick=time.monotonic()
with (out/(name+'.log')).open('w') as log:
 log.write('COMMAND '+repr(argv)+'\nSTART '+start+'\n');log.flush()
 proc=subprocess.run(argv,stdout=log,stderr=subprocess.STDOUT,env=env)
 end=datetime.datetime.now().astimezone().isoformat();log.write('\nEND '+end+'\nEXIT '+str(proc.returncode)+'\n')
meta=dict(command=argv,start=start,end=end,seconds=round(time.monotonic()-tick,3),exit=proc.returncode)
(out/(name+'.json')).write_text(json.dumps(meta,indent=2)+'\n');print(json.dumps(meta));raise SystemExit(proc.returncode)
