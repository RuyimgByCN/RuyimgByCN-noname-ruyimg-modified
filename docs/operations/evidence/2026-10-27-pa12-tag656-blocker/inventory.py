import collections
import json
import re
from pathlib import Path

values = collections.defaultdict(list)
text = Path('crates/rule-vm/src/bytecode/effect_tag.rs').read_text()
for name, number in re.findall(r'pub const\s+(\w+)\s*:\s*u16\s*=\s*(\d+)\s*;', text):
    values[int(number)].append(name)
for number, names in sorted(values.items()):
    if len(names) > 1:
        print('duplicate', number, ','.join(names))
print('tag 772 constants:', values.get(772, []))
for name, path, module in [
    ('sp', 'content/original/ported/sp/sp.npack.json', 'yingshi_die_death'),
    ('onlyOL', 'content/original/ported/onlyOL/onlyOL.npack.json', 'ol_pingkou_phase_end'),
]:
    data = json.loads(Path(path).read_text())
    entry = next(m for m in data['modules'] if m['name'] == module)
    print(name, 'package_version', data['manifest']['version'], 'abi', data['manifest']['rule_vm_abi_version'],
          'module', module, 'code', entry['code'])
