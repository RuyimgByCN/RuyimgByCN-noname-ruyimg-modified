# U05 wuxing 子包：真实宿主全生命周期验收（2026-10-08）

按同模板，对 wuxing 子包（`upstream-liuli/wuxing@1.0.0`，五行盘+生克
宿主技能）执行真实宿主全生命周期验收——**U05 子包验收第六例**（来源
计数仍为欢乐包 1/37）。

## 生命周期六步（`wuxing_real_host_full_lifecycle`，g1_wuxing.rs）

1. **独立安装+激活**：激活集含 `upstream-liuli/wuxing`。
2. **安装包→原生建局**：cards.json → C0 桥接 → 建局（五行盘入目录）。
3. **对局中卸载**：`revoke_slot(deactivate=true)`。
4. **在局续玩（lock_pinned）**：恢复+提交继续，五行盘仍可用。
5. **重装守卫**：同 npack 再 install → `RuntimeError::Revoked`。
6. **重新 import 被拒**。

## 验收口径

- **回归已加入**：`wuxing_real_host_full_lifecycle`（真实包，tempdir
  隔离 store）。
- **全量通过**：engine-native **393 passed / 0 failed / 1 ignored**；
  依赖包 **62/0**，DEPS_EXIT=0（无引擎代码改动，无 golden 变化）。
- **口径澄清**：U05 有转换包的子包（gwent/古剑/炉石/运筹/wuxing）
  已全部生命周期验收；swd/ow/xianjian/mtg/zhenfa **无转换包**——
  G1 逐子包转换为验收前置（未转换不可验）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（4 文件）。
