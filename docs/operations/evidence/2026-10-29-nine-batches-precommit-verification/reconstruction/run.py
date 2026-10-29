import os,subprocess,json,sys,time
from pathlib import Path
root=Path(__file__).resolve().parent; batch=sys.argv[1]; tag=sys.argv[2]; commands=json.loads(sys.argv[3]);out=root/batch;out.mkdir(exist_ok=True)
env=os.environ.copy();env['CARGO_TARGET_DIR']=str(root.parent/('gate-target' if tag=='gate' else 'target'))
if tag=='gate':env['RUSTFLAGS']='-D warnings'
records=[]
for i,command in enumerate(commands):
 text=command if isinstance(command,str) else ' '.join(command);start=time.time();log=f'{tag}-{i:02d}.log'
 with (out/log).open('w') as f:
  f.write('RECONSTRUCTION VALIDATION (not historical evidence)\nCOMMAND: '+text+'\nCARGO_TARGET_DIR='+env['CARGO_TARGET_DIR']+'\nRUSTFLAGS='+env.get('RUSTFLAGS','')+'\n');f.flush()
  code=subprocess.call(command,shell=isinstance(command,str),env=env,stdout=f,stderr=subprocess.STDOUT)
  f.write('\nEXIT_CODE: '+str(code)+'\n')
 records.append(dict(command=text,exit_code=code,seconds=time.time()-start,log=log));(out/(tag+'-results.json')).write_text(json.dumps(records,indent=2))
