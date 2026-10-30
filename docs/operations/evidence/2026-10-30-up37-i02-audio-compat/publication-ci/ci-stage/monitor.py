import json
from pathlib import Path
import time

out = Path(__file__).resolve().parent
time.sleep(50)
state = json.loads((out / 'observation-state.json').read_text())
print(json.dumps({k: state[k] for k in ('run_status', 'run_conclusion', 'completed', 'finished_utc', 'error', 'cleanup', 'checkpoint_80min') if k in state}))
for name in ('watch-http1.stdout', 'watch.stdout'):
    path = out / name
    if path.exists():
        text = path.read_text(errors='replace')
        chunk = text.rsplit('JOBS\n', 1)[-1].split('ANNOTATIONS')[0]
        print(name + '\n' + chunk[-2000:])
        break
for name in ('watch-http1.stderr', 'watch.stderr', 'driver.stderr'):
    path = out / name
    if path.exists() and path.stat().st_size:
        print(name + '\n' + path.read_text(errors='replace')[-400:])
