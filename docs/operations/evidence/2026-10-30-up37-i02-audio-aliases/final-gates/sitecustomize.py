"""Record preflight child output without changing argv, output, or return codes."""
import json
import os
from pathlib import Path
import subprocess
import sys
import time

if sys.argv[0].endswith('/preflight.py') or sys.argv[0] == 'scripts/preflight.py':
    _run = subprocess.run
    _out = Path(os.environ['FINAL_GATES_OUT'])
    _sequence = 0
    def _recorded_run(*args, **kwargs):
        global _sequence
        _sequence += 1
        number = _sequence
        started = time.monotonic()
        completed = _run(*args, **kwargs)
        stem = f'preflight-{number:02d}'
        for stream in ['stdout', 'stderr']:
            value = getattr(completed, stream)
            if value is not None:
                (_out / f'{stem}.{stream}.log').write_text(value)
        with (_out / 'preflight-children.jsonl').open('a') as output:
            output.write(json.dumps({'sequence': number, 'argv': completed.args, 'exit': completed.returncode, 'seconds': time.monotonic() - started, 'stdout': f'{stem}.stdout.log', 'stderr': f'{stem}.stderr.log'}, ensure_ascii=False) + '\n')
        return completed
    subprocess.run = _recorded_run
