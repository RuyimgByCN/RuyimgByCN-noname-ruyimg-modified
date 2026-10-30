import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
OUT = Path(__file__).resolve().parent

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, env={**os.environ, 'GIT_OPTIONAL_LOCKS': '0'}, text=True).strip()

def snapshot(name):
    audio = json.loads((OUT.parent / 'parent-main-source-admission.json').read_text())
    ci = json.loads((ROOT / 'docs/operations/evidence/2026-10-30-native-ci-corrections/parent-source-admission.json').read_text())
    a = json.loads((ROOT / 'docs/operations/evidence/2026-10-30-adr0787-timeout/parent-source-admission.json').read_text())
    prior = json.loads((OUT.parent / 'main-acceptance/before.json').read_text())
    a_expected = {p: ci['paths'].get(p, h) for p, h in a['runtime_paths_applied'].items()}
    ci4 = ['.github/workflows/ci.yml', '.github/actions/prepare-original-content/action.yml', '.github/actions/prepare-original-content/liuli-longxue.patch', 'docs/operations/native-ci.md']
    docs = ['docs/architecture/native-progress.md', 'docs/architecture/native-remaining-ledger.md', 'docs/operations/evidence/2026-10-30-native-ci-corrections/README.md', str(OUT.parent.relative_to(ROOT) / 'README.md')]
    paths = {
        'source9': audio['paths'],
        'ci4': {p: prior['protected'][p] for p in ci4},
        'A53': a_expected,
        'modules': prior['modules'],
        'tracked_dist': {p: digest(ROOT / p) for p in git('ls-files', 'apps/web-client/dist').splitlines()},
        'parent_docs': {p: digest(ROOT / p) for p in docs},
        'guard': {'.pi-lens.json': digest(ROOT / '.pi-lens.json')},
    }
    value = {'time': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'head': git('rev-parse', 'HEAD'), 'index_paths': git('diff', '--cached', '--name-only').splitlines(), 'index_sha256': digest(ROOT / git('rev-parse', '--git-path', 'index')), 'status_porcelain': git('status', '--porcelain=v1', '--untracked-files=all').splitlines(), 'versions': {'rule': 330, 'archive': 105, 'ABI': 758, 'WS': 6, 'asset_schema': 2}, 'groups': {}, 'admission_mismatches': {}}
    for group, expected in paths.items():
        actual = {p: digest(ROOT / p) for p in expected}
        value['groups'][group] = actual
        value['admission_mismatches'][group] = [p for p, h in expected.items() if actual[p] != h]
    value['guard_value'] = json.loads((ROOT / '.pi-lens.json').read_text())
    (OUT / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')
    assert value['head'] == audio['base'] == 'da7e4c019461a6c467a9dd2de849f3b1a7494b86'
    assert not value['index_paths']
    assert all(not v for v in value['admission_mismatches'].values())
    g = value['guard_value']
    assert g['autofix']['enabled'] is g['format']['enabled'] is g['actionableWarnings']['autoFix']['enabled'] is False
    assert len(value['groups']['source9']) == 9 and len(value['groups']['A53']) == 53
    if name != 'narrow-entry.json':
        entry = json.loads((OUT / 'narrow-entry.json').read_text())
        same = {k: entry[k] == value[k] for k in ['head', 'index_paths', 'index_sha256', 'groups', 'guard_value', 'versions']}
        (OUT / 'narrow-comparison.json').write_text(json.dumps(same, indent=2) + '\n')
        assert all(same.values()), same
    print(json.dumps({'snapshot': name, 'counts': {k: len(v) for k,v in value['groups'].items()}, 'head': value['head'], 'index_empty': not value['index_paths']}), flush=True)

if __name__ == '__main__':
    import sys
    snapshot(sys.argv[1])
