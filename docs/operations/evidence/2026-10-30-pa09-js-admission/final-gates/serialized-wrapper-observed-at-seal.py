import fcntl
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time

assert len(sys.argv) > 1, 'pass the command as argv'
with Path('/tmp/noname-phase4-heavy-command.lock').open('a') as lock:
    fcntl.flock(lock, fcntl.LOCK_EX)
    started = time.monotonic()
    result = subprocess.run(sys.argv[1:])
    print(json.dumps({'command': sys.argv[1:], 'exit': result.returncode, 'locked_seconds': time.monotonic() - started, 'disk_free_bytes': shutil.disk_usage(Path.cwd()).free}), flush=True)
    sys.exit(result.returncode)
