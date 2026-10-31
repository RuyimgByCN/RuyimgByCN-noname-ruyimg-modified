import sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent))
from verify import run,O,R
commands=[('formal-trajectories',['python3','-B',str(O/'trajectories.py')]),('conformance',['cargo','run','--offline','-p','conformance']),('legacy-crosshash',['python3','-B',str(O/'legacy-crosshash.py')])]
for name,argv in commands:
 if run(name,argv):sys.exit(1)
