from rebuild_common import *
# The catalog's CardKind implementation stays in its original file until batch 9.
p='crates/engine-native/src/catalog.rs';s=Path(p).read_text();f=(SNAP/p).read_text();a=f.index('/// 内容卡牌的固有结算：');b=f.index('/// 目录：卡牌表',a);s=s.replace('/// 目录：卡牌表',f[a:b]+'/// 目录：卡牌表');s=s.replace('    pub vm_card_history: bool,','    pub vm_card_history: bool,\n    /// 内容层回调（内容目录构建时装入）。不入对局摘要。\n    pub hooks: ContentHooks,');s=s.replace('            vm_card_history,','            vm_card_history,\n            hooks: ContentHooks::NONE,');Path(p).write_text(s)
for p in ['crates/engine-native/src/content/mod.rs','crates/engine-native/src/content/hooks.rs','crates/engine-native/src/content/host/intrinsic.rs','crates/engine-native/src/kernel/intrinsic.rs','crates/engine-native/src/content/standard.rs','crates/engine-native/src/snap/restore.rs','crates/engine-native/src/engine/snapshot.rs','crates/engine-native/src/snap.rs','scripts/check-native-layering.py','scripts/test_check_native_layering.py','scripts/preflight.py','docs/architecture/architecture-evolution-plan.md']:copy(p)
p='crates/engine-native/src/content/host.rs';s=Path(p).read_text();s=s.replace('mod activation;','mod activation;\nmod asks;\nmod intrinsic;\npub(crate) use asks::active_ask_host;\npub(crate) use intrinsic::card_intrinsic;');Path(p).write_text(s)
p='crates/engine-native/src/snap/host_asks.rs';s=Path(p).read_text();s=s.replace('use super::*;','use super::*;\nuse crate::catalog::Catalog;\nuse crate::snap::*;\nuse crate::state::GameStateCore;',1).replace('pub(super) fn active_ask_host','pub(crate) fn active_ask_host');Path('crates/engine-native/src/content/host/asks.rs').write_text(s);Path(p).unlink()
p='crates/engine-native/src/snap/asks.rs';s=Path(p).read_text();s=s.replace('=> active_ask_host(', '=> (catalog.hooks.host_ask)(')
for name in ['ask','hand_card_options','other_player_options']:s=s.replace('pub(super) fn '+name+'(', 'pub(crate) fn '+name+'(')
Path(p).write_text(s)
p='.github/workflows/ci.yml';s=Path(p).read_text();f=(SNAP/p).read_text();a=f.index('      - name: engine-native kernel layer');b=f.index('      - name:',a+10);s=s.replace('          python3 scripts/check-source-lines.py\n','          python3 scripts/check-source-lines.py\n'+f[a:b]);Path(p).write_text(s)
evidence(8)

p='docs/operations/self-check-guide.md';s=Path(p).read_text();s=s.replace('全部 20 道门禁','全部 21 道门禁').replace('# 完整 20 项','# 完整 21 项').replace('# 实跑 18 项','# 实跑 19 项');paragraph=next(l for l in (SNAP/p).read_text().splitlines() if l.startswith('**第 21 项是'));s=s.replace('**第 20 项是',paragraph+'\n\n**第 20 项是');Path(p).write_text(s)

p='docs/architecture/native-progress.md';s=Path(p).read_text();paragraph=next(l for l in (SNAP/p).read_text().splitlines() if l.startswith('**追加（2026-10-29 16:50'));Path(p).write_text(s.rstrip()+'\n\n'+paragraph+'\n')
