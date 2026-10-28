import collections
import json
import re
from pathlib import Path

root = Path(__file__).parent
artifacts = {}
for line in (root / 'test-artifacts-final.jsonl').read_text().splitlines():
    item = json.loads(line)
    executable = item.get('executable')
    if item.get('reason') == 'compiler-artifact' and executable:
        package = item['package_id'].split('#', 1)[0].rsplit('/', 1)[-1]
        artifacts[str(Path(executable).resolve())] = package
counts = collections.defaultdict(lambda: [0, 0, 0])
current = None
runs = 0
for line in (root / 'rust-full-final.log').read_text().splitlines():
    run = re.match(r'\s*Running .*\(([^)]+)\)$', line)
    doc = re.match(r'\s*Doc-tests ([\w-]+)$', line)
    if run:
        current = artifacts.get(str(Path(run.group(1)).resolve()))
        if current is None:
            raise SystemExit(f'unknown test executable: {run.group(1)}')
    elif doc:
        current = doc.group(1).replace('_', '-')
    else:
        result = re.match(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', line)
        if result:
            if current is None:
                raise SystemExit(f'test result without target: {line}')
            for i, value in enumerate(result.groups()):
                counts[current][i] += int(value)
            current = None
            runs += 1
if current is not None:
    raise SystemExit(f'missing result for {current}')
for package in sorted(counts):
    print(f'{package}: {counts[package][0]} passed / {counts[package][1]} failed / {counts[package][2]} ignored')
total = [sum(v[i] for v in counts.values()) for i in range(3)]
print(f'total: {total[0]} passed / {total[1]} failed / {total[2]} ignored; {runs} test targets')
