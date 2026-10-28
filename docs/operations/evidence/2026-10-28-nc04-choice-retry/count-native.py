import re
from pathlib import Path

log = (Path(__file__).parent / 'native-full-final.log').read_text()
results = re.findall(r'^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', log, re.M)
if not results or not log.rstrip().endswith('exit 0'):
    raise SystemExit('missing completed suite results')
counts = [sum(int(row[i]) for row in results) for i in range(3)]
print(f'engine-native: {counts[0]} passed / {counts[1]} failed / {counts[2]} ignored; {len(results)} test targets')
