import os,sys,subprocess,json,pathlib,time
root=pathlib.Path(__file__).resolve().parent
batch=sys.argv[1]; mode=sys.argv[2] if len(sys.argv)>2 else 'early'
out=root/batch; out.mkdir(exist_ok=True)
env=os.environ.copy(); env['CARGO_TARGET_DIR']=str(root.parent/'target')
cmds=[['cargo','check','--offline','--workspace','--all-targets'],['cargo','test','--offline','-p','content-compiler','--no-fail-fast'],['cargo','test','--offline','-p','engine-native','--no-fail-fast'],['python3','-B','-m','unittest','discover','-s','scripts','-p','test_check_source_lines.py'],['python3','-B','scripts/check-source-lines.py']]
if mode=='gate': cmds=[['cargo','check','--offline','-p','engine-native','--all-targets']];env['CARGO_TARGET_DIR']=str(root.parent/'gate-target');env['RUSTFLAGS']='-D warnings'
records=[]
for i,cmd in enumerate(cmds):
 start=time.time(); name=f'{mode}-{i:02d}'; text=' '.join(cmd)
 with (out/(name+'.log')).open('w') as f:
  f.write('RECONSTRUCTION VALIDATION (not historical evidence)\nCOMMAND: '+text+'\nCARGO_TARGET_DIR='+env['CARGO_TARGET_DIR']+'\nRUSTFLAGS='+env.get('RUSTFLAGS','')+'\n');f.flush()
  code=subprocess.call(cmd,stdout=f,stderr=subprocess.STDOUT,env=env)
  f.write('\nEXIT_CODE: '+str(code)+'\n')
 records.append(dict(command=text,exit_code=code,seconds=time.time()-start,log=name+'.log'))
 (out/(mode+'-results.json')).write_text(json.dumps(records,indent=2))
print(json.dumps(records))
