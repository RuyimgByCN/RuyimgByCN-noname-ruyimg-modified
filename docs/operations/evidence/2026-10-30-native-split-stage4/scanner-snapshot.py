import pathlib,subprocess,json,sys,hashlib
root=pathlib.Path.cwd(); evidence=pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-10-30-native-split-stage4')
names=['audit_upstream_skill_flags','check_guard_references','check_ledger_consistency','host_key_reachability','report_event_arms','report_filter_meta_mismatch','report_g14','report_phase_arms','report_phase_filters','report_registered_deviations','report_unimplemented_hosts','check_card_kind_compare','report_card_coverage','dump_skill_pair','report_defect_ledger']
phase=sys.argv[1]; data={}
if len(sys.argv)>2:names=sys.argv[2:]
for name in names:
 trace=evidence/(phase+'-'+name+'-inputs.json')
 code='''import sys,runpy,json,pathlib,atexit
reads=set()
def hook(event,args):
 if event=="open" and isinstance(args[0],str) and args[0].endswith(".rs"):
  reads.add(str(pathlib.Path(args[0]).resolve()))
sys.addaudithook(hook)
trace_path=pathlib.Path(sys.argv[1])
atexit.register(lambda: trace_path.write_text(json.dumps(sorted(reads),indent=2)+"\\n"))
sys.path.insert(0,str(pathlib.Path(sys.argv[2]).resolve().parent))
sys.argv=[sys.argv[2]]+sys.argv[3:]
runpy.run_path(sys.argv[0],run_name="__main__")
'''
 args=[]
 if name=='dump_skill_pair':args=['Jianxiong']
 if name=='audit_upstream_skill_flags':args=['jianxiong','Jianxiong']
 p=subprocess.run([sys.executable,'-B','-c',code,str(trace),'scripts/'+name+'.py',*args],capture_output=True,text=True)
 (evidence/(phase+'-'+name+'-final.log')).write_text(p.stdout+p.stderr)
 reads=json.loads(trace.read_text())
 data[name]={'exit':p.returncode,'inputs':{str(pathlib.Path(x).relative_to(root)):hashlib.sha256(pathlib.Path(x).read_bytes()).hexdigest() for x in reads if pathlib.Path(x).is_file() and pathlib.Path(x).is_relative_to(root)},'stdout':p.stdout,'stderr':p.stderr}
 trace.unlink()
snapshot=evidence/('scanners-'+phase+'.json')
if snapshot.exists():
 previous=json.loads(snapshot.read_text());previous.update(data);data=previous
snapshot.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
print({k:(v['exit'],len(v['inputs'])) for k,v in data.items()})
