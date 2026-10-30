import subprocess, sys
from pathlib import Path
out = Path(__file__).parent
for name in ["sourcecap-unit", "sourcecap", "diffcheck"]:
    result = subprocess.run([sys.executable, "-B", str(out / "accept.py"), name])
    if result.returncode: raise SystemExit(result.returncode)
