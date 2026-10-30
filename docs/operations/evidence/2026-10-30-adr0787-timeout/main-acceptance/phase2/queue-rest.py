from pathlib import Path
import json,subprocess,sys
E=Path(__file__).resolve().parent
for name,argv in json.loads((E/'queue-rest.json').read_text()):
 print('START '+name,flush=True)
 code=subprocess.run(['python3','-B',str(E/'run.py'),name,*argv]).returncode
 print('END '+name+' exit='+str(code),flush=True)
 if code:sys.exit(code)
