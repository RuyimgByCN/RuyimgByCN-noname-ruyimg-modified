import argparse,datetime,json,os,pathlib,shlex,subprocess,sys,time
p=argparse.ArgumentParser(); p.add_argument("name"); p.add_argument("--cwd"); p.add_argument("--env",action="append",default=[]); p.add_argument("argv",nargs=argparse.REMAINDER); a=p.parse_args()
root=pathlib.Path("/Users/ruyimg/Documents/Claude/sanguosha/noname-next"); out=root/"docs/operations/evidence/2026-11-14-queue1-runtime/U06"
argv=a.argv[1:] if a.argv[:1]==["--"] else a.argv
assert argv and "/" not in a.name and ".." not in a.name
log=out/(a.name+".log"); meta=out/(a.name+".json")
assert not log.exists() and not meta.exists(), "Do not overwrite an earlier run"
env=dict(os.environ); env["CARGO_INCREMENTAL"]="0"
for entry in a.env:
 key,value=entry.split("=",1); env[key]=value
cwd=root/a.cwd if a.cwd else root
record={"argv":argv,"command":shlex.join(argv),"cwd":str(cwd),"env_overrides":{"CARGO_INCREMENTAL":"0",**dict(entry.split("=",1) for entry in a.env)},"source_head":subprocess.check_output(["git","rev-parse","HEAD"],cwd=root,text=True).strip(),"tracked_status":subprocess.check_output(["git","status","--porcelain=v1","--untracked-files=no"],cwd=root,text=True),"start_utc":datetime.datetime.now(datetime.timezone.utc).isoformat()}
start=time.monotonic(); print(record["command"],flush=True)
with log.open("wb") as stream:
 result=subprocess.run(argv,cwd=cwd,env=env,stdout=stream,stderr=subprocess.STDOUT)
record.update(exit_code=result.returncode,duration_seconds=round(time.monotonic()-start,3),end_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
meta.write_text(json.dumps(record,ensure_ascii=False,indent=2)+"\n")
lines=log.read_text(errors="replace").splitlines(); print("\n".join(lines[-18:])); print("exit",result.returncode,"log",log,flush=True)
print(subprocess.check_output(["df","-h",str(root)],text=True))
sys.exit(result.returncode)
