# U05 杀海拾遗·炉石子包：真实宿主全生命周期验收（2026-10-08）

按欢乐包/gwent/古剑同模板，对 U05 炉石子包（`upstream-noname/
shahai-hearth@1.0.0`，17 卡切片）执行真实宿主全生命周期验收——**U05
子包验收第四例**（来源计数仍为欢乐包 1/37）。

## 生命周期六步（`hearth_real_host_full_lifecycle`，g1_hearth.rs）

1. **独立安装+激活**：激活集含 `upstream-noname/shahai-hearth`。
2. **安装包→原生建局**：安装产物 cards.json → C0 桥接 → 建局（17 卡
   目录）。
3. **对局中卸载**：`revoke_slot(deactivate=true)`。
4. **在局续玩（lock_pinned）**：恢复+提交继续，炉石卡仍可用。
5. **重装守卫**：同 npack 再 install → `RuntimeError::Revoked`。
6. **重新 import 被拒**。

## 验收口径

- **回归已加入**：`hearth_real_host_full_lifecycle`（真实包，tempdir
  隔离 store）。
- **全量通过**：engine-native **391 passed / 0 failed / 1 ignored**；
  依赖包 **62/0**，DEPS_EXIT=0（无引擎代码改动，无 golden 变化）。
- **边界**：炉石规则差异（失去体力/随机抽取/确定性近似等）已在逐技
  对照系列与批三登记；本验收=宿主能力面；U05 整包剩余=其余七子包
  （yunchou/swd/ow/xianjian/mtg/zhenfa/wuxing）同模板续验 + 角色/
  技能/模式/配置面。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（4 文件）。
