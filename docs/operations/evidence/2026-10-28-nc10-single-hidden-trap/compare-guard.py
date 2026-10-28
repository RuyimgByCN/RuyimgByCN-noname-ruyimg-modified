import re
from pathlib import Path

root = Path(__file__).parent

def cases(name):
    result = {}
    for line in (root / name).read_text().splitlines():
        match = re.match(r'CASE (\d+) (\w+) equal=(true|false)', line)
        if match:
            result[(int(match[1]), match[2])] = match[3] == 'true'
    return result

before, after = cases('guard-clean-before.log'), cases('guard-final.log')
assert before.keys() == after.keys()
for kind in sorted({key[1] for key in before}):
    keys = [key for key in before if key[1] == kind]
    print(f'{kind}: {len(keys)} pairs; previously_equal_now_different={sum(before[k] and not after[k] for k in keys)}; baseline_different={sum(not before[k] for k in keys)}; final_different={sum(not after[k] for k in keys)}')
