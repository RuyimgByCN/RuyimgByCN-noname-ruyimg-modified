import json
from pathlib import Path
import time

out = Path(__file__).resolve().parent
wait = 50
print('Local monitor wait50s; same single watch, no API request', flush=True)
time.sleep(wait)
path = out / 'observation-state.json'
if path.exists():
    state = json.loads(path.read_bytes())
    print(json.dumps({key: state.get(key) for key in ('run_id', 'head_sha', 'elapsed_seconds', 'cleanup', 'run_status', 'run_conclusion', 'completed', 'job_count', 'pending_jobs', 'error', 'finished_utc', 'checkpoint_15min', 'watch_deadline_reached')}, indent=2))
    if state.get('jobs'):
        print('Collected jobs:', [(j['name'], j['status'], j['conclusion']) for j in state['jobs']])
for name in ('watch.stdout', 'watch.stderr', 'observer.stderr'):
    p = out / name
    if p.exists():
        data = p.read_text(errors='replace')
        print(name, 'bytes', p.stat().st_size, '\n', data[-2200:])
