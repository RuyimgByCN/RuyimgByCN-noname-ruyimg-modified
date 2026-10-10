# U05 杀海拾遗·运筹子包：真实宿主全生命周期验收（2026-10-08）

按欢乐包/gwent/古剑/炉石同模板，对 U05 运筹子包（`upstream-noname/
shahai-yunchou@1.0.0`，23 卡切片）执行真实宿主全生命周期验收——**U05
子包验收第五例**（来源计数仍为欢乐包 1/37）。

## 生命周期六步（`yunchou_real_host_full_lifecycle`，g1_shahai_yunchou.rs）

1. **独立安装+激活**：激活集含 `upstream-noname/shahai-yunchou`。
2. **安装包→原生建局**：安装产物 cards.json → C0 桥接 → 建局（23 卡
   目录）。
3. **对局中卸载**：`revoke_slot(deactivate=true)`。
4. **在局续玩（lock_pinned）**：恢复+提交继续，运筹卡仍可用。
5. **重装守卫**：同 npack 再 install → `RuntimeError::Revoked`。
6. **重新 import 被拒**。

## 验收口径

- **回归已加入**：`yunchou_real_host_full_lifecycle`（真实包，tempdir
  隔离 store）。
- **全量通过**：engine-native **392 passed / 0 failed / 1 ignored**；
  依赖包 **62/0**，DEPS_EXIT=0（无引擎代码改动，无 golden 变化）。
- **边界**：运筹规则差异（草船借箭/舌战群儒简化等）已在 v43/v44 与
  逐技对照批三登记；本验收=宿主能力面；U05 整包剩余=其余六子包
  （swd/ow/xianjian/mtg/zhenfa/wuxing）同模板续验 + 角色/技能/模式/
  配置面。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（4 文件）。
