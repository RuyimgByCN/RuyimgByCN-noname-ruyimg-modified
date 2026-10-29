from rebuild_common import *
for p in ['Cargo.toml','crates/engine-native/Cargo.toml','crates/engine-core/Cargo.toml','crates/engine-native/tests/t8_rvm_bridge.rs','crates/engine-native/tests/t12_g5_relays.rs','crates/engine-native/tests/g1_huanle/huanle.rs','scripts/check-native-wasm-cross-target.py']:copy(p)
for row in json.loads((SNAP/'snapshot-manifest.json').read_text()):
 p=row['path']
 if '/tests/groups/' in p:copy(p)
evidence(7)
p='docs/operations/self-check-guide.md';s=Path(p).read_text();f=(SNAP/p).read_text();paragraph=next(l for l in f.splitlines() if l.startswith('engine-native 与 engine-core'));s=s.replace('当前基线、失败和忽略项见',paragraph+'\n\n当前基线、失败和忽略项见');Path(p).write_text(s)

p='docs/architecture/native-progress.md';s=Path(p).read_text().replace('--test replay_scenarios','--test kernel replay_scenarios::');paragraph=next(l for l in (SNAP/p).read_text().splitlines() if l.startswith('**追加（2026-10-29 15:40'));Path(p).write_text(s.rstrip()+'\n\n'+paragraph+'\n')
