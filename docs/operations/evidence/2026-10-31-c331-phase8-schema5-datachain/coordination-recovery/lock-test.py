import sys,json,time
from pathlib import Path
p=Path(sys.argv[1]);tag=sys.argv[2]
with p.open("a") as f:f.write(json.dumps({"tag":tag,"event":"start","t":time.monotonic()})+"\n")
time.sleep(0.2)
with p.open("a") as f:f.write(json.dumps({"tag":tag,"event":"end","t":time.monotonic()})+"\n")
