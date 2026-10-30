from pathlib import Path
import datetime
import hashlib
import json
import os
import subprocess

MAIN = Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
TEMP = Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T')
OUT = Path(__file__).parent
EVIDENCE = MAIN / 'docs/operations/evidence/2026-10-30-native-ci-corrections'
GROUPS = {
    'noname-ci-slow-ws-yn6e1n1c': ['apps/game-server/src/native_ws_e2e.rs', 'apps/game-server/src/native_ws_e2e/peer.rs', 'apps/game-server/tests/nc09_native_actor.rs'],
    'noname-native-ci-pml7niud': ['.github/workflows/ci.yml'],
    'noname-installer-unicode-a_in35gp': ['apps/desktop-client/tauri.conf.json', 'apps/desktop-client/tauri.linux.conf.json', 'apps/desktop-client/linux/main.desktop', 'apps/web-client/scripts/build-pipeline.test.mjs'],
    'noname-legacy-start-perf-0hholn9n': ['crates/engine-core/src/content/mod.rs', 'crates/engine-core/src/content/standard_pack.rs', 'crates/engine-core/src/effect/primitives/pack_tw_1/finish_turn_begin.rs', 'crates/engine-core/src/engine.rs'],
}
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

admission = json.loads((EVIDENCE / 'parent-source-admission.json').read_text())
before = json.loads((EVIDENCE / 'main-acceptance/before.json').read_text())
after = json.loads((EVIDENCE / 'main-acceptance/after.json').read_text())
rows = []
for directory, names in GROUPS.items():
    for name in names:
        current = sha(MAIN / name)
        reviewed = sha(TEMP / directory / 'worktree' / name)
        rows.append({'path': name, 'current_sha256': current, 'reviewed_sha256': reviewed, 'equal': current == reviewed})
source13 = {name: sha(MAIN / name) for name in admission['paths']}
checks = {'reviewed12': len(rows) == 12 and all(row['equal'] for row in rows), 'admission13': source13 == admission['paths'], 'functional_before13': source13 == before['source13'], 'functional_after13': source13 == after['source13'], 'A_remaining50': len(after['A_remaining50']) == 50 and all(sha(MAIN / name) == value for name, value in after['A_remaining50'].items())}
env = {**os.environ, 'GIT_OPTIONAL_LOCKS': '0', 'GIT_NO_LAZY_FETCH': '1'}
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=MAIN, env=env, text=True).strip()
index = subprocess.check_output(['git', 'diff', '--cached', '--name-only'], cwd=MAIN, env=env, text=True).splitlines()
checks['head'] = head == '38ab1678df9b4b7b991d3b3122b1d937fd2648a1'
checks['index_empty'] = not index
result = {'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'head': head, 'index': index, 'versions_from_unchanged_source_identity': '330/105/758/WS6', 'reviewed12': rows, 'current13': source13, 'checks': checks, 'all_equal': all(checks.values()), 'doc_overlap': 'native-ci.md is the thirteenth path; reviewer reads the combined profile and installer sections, not expected byte-identical to either isolated document', 'final_preflight': 'Separate native async run c735ec38-d18e-46b6-850c-9bbb9460aba2; no completion received at this check; not claimed21/21', 'read_only': True}
(OUT / 'identity-entry.json').write_text(json.dumps(result, indent=2) + '\n')
assert result['all_equal'], checks
print(json.dumps({'reviewed12': len(rows), 'current13': len(source13), 'A_remaining': 50, 'all_equal': True}))
