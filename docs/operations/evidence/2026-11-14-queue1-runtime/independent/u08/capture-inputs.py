#!/usr/bin/env python3
"""Record the independent review's live source inputs without changing them."""
import datetime
import hashlib
import json
from pathlib import Path
import subprocess

BASE = Path(__file__).resolve().parent
REPO = BASE.parents[5]
UPSTREAM = REPO.parent / 'archive/upstream/noname'


def digest(path):
    data = path.read_bytes()
    return {'path': str(path), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}


def command(*args):
    return subprocess.check_output(args, cwd=REPO, text=True).strip()


source_paths = [
    'crates/content-native-loader/src/host_skill_bindings.rs',
    'crates/content-native-loader/src/decision34_pack9.rs',
    'crates/content-native-loader/src/decision34_pack9_data.rs',
    'crates/content-native-loader/src/locked_content.rs',
    'crates/engine-native-content/src/content/host/step/u08_batch_a.rs',
    'crates/engine-native-content/src/content/host/step/u08_batch_a_asks.rs',
    'crates/engine-native-content/src/content/host/step/u08_batch_d.rs',
    'crates/engine-native-content/src/content/host/step/u08_batch_d_asks.rs',
    'crates/engine-native-content/src/content/host/step/u08_batch_d_ext.rs',
    'crates/engine-native/src/engine/apply_command.rs',
    'crates/engine-native/src/engine/legal_actions.rs',
    'crates/engine-native-content/src/content/standard.rs',
    'crates/engine-native-kernel/src/archive.rs',
    'crates/engine-native-kernel/src/snap/restore.rs',
    'crates/engine-api/src/version.rs',
    'crates/engine-protocol/src/lib.rs',
    'packages/extensions/hero-kill-upstream-noname/assets/cards.json',
]
upstream_paths = [
    'apps/core/extension/英雄杀/character/skill.js',
    'apps/core/extension/英雄杀/character/character.js',
    'apps/core/noname/library/element/player.js',
    'apps/core/noname/game/index.js',
    'apps/core/noname/ui/click/index.js',
]
product_dirs = [
    'crates/engine-native-kernel', 'crates/engine-native-content',
    'crates/engine-native', 'crates/content-native-loader',
    'crates/engine-api', 'crates/engine-protocol',
]
all_product = sorted({p for d in product_dirs for p in (REPO / d / 'src').rglob('*.rs')})
row = {
    'recorded_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'head': command('git', 'rev-parse', 'HEAD'),
    'tracked_product_status': command('git', 'status', '--short', '--untracked-files=no', '--', 'crates', 'packages', 'apps', 'scripts'),
    'all_worktree_status': command('git', 'status', '--short'),
    'upstream_head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=UPSTREAM, text=True).strip(),
    'upstream_source_status': subprocess.check_output(['git', 'status', '--short', '--', *upstream_paths], cwd=UPSTREAM, text=True).strip(),
    'version_axes': {'rules': 333, 'archive': 107, 'content_schema': 5, 'rule_vm_abi': 759, 'network_protocol': 8},
    'source_inputs': [digest(REPO / p) for p in source_paths] + [digest(UPSTREAM / p) for p in upstream_paths],
    'product_rust_source_count': len(all_product),
    'product_rust_sources': [digest(p) for p in all_product],
    'codegraph_tools_available': False,
    'scope': '独立源码预查；作者原始采集尚未收到最终冻结通知，未认定为最终 cohort。',
    'not_executed': ['Cargo', 'browser', 'upstream or native case replay', 'full preflight', 'product changes', 'inventory changes', 'Git index/refs changes'],
}
out = BASE / 'input-context-first.json'
with out.open('x') as stream:
    json.dump(row, stream, ensure_ascii=False, indent=2)
    stream.write('\n')
inventory = json.loads((REPO / 'docs/migration/g0/inventory_status.json').read_text())['items']
keys = {k: {field: v.get(field) for field in ('source_id', 'category', 'id', 'upstream_ref', 'status', 'reviewer', 'review_date')}
        for k, v in inventory.items() if v.get('source_id') == 'U08' and v.get('id') in ('yixin', 'bolehuiyan', 'bolehuiyan2', 'bolehuiyan3', 'guifu')}
with (BASE / 'inventory-keys-at-start.json').open('x') as stream:
    json.dump(keys, stream, ensure_ascii=False, indent=2)
    stream.write('\n')
print(json.dumps({'head': row['head'], 'sources': len(row['source_inputs']), 'product_rust_sources': len(all_product), 'inventory_keys': len(keys)}, ensure_ascii=False))
