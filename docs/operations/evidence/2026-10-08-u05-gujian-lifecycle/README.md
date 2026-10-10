# U05 杀海拾遗·古剑子包：真实宿主全生命周期验收（2026-10-08）

按欢乐包/gwent 同模板，对 U05 古剑子包（`upstream-noname/shahai-gujian
@1.0.0`，40/40 最全卡牌切片）执行真实宿主全生命周期验收——**U05 子包
验收第三例**（gwent 第二例；来源计数仍为欢乐包 1/37）。

## 生命周期六步（`gujian_real_host_full_lifecycle`，g1_shahai_gujian.rs）

1. **独立安装+激活**：激活集含 `upstream-noname/shahai-gujian`。
2. **安装包→原生建局**：安装产物 cards.json → C0 桥接 → 建局（40 卡
   目录）。
3. **对局中卸载**：`revoke_slot(deactivate=true)`。
4. **在局续玩（lock_pinned）**：恢复+提交继续，古剑卡仍可用。
5. **重装守卫**：同 npack 再 install → `RuntimeError::Revoked`。
6. **重新 import 被拒**。

## 验收口径

- **回归已加入**：`gujian_real_host_full_lifecycle`（真实包，tempdir
  隔离 store）。
- **全量通过**：engine-native **390 passed / 0 failed / 1 ignored**；
  依赖包 **62/0**，DEPS_EXIT=0（无引擎代码改动，无 golden 变化）。
- **边界**：古剑规则差异（失去体力原语 v77/随机抽取 v78/自选 v79 已
  消除项+保留登记项）已在逐技对照系列登记；本验收=宿主能力面；
  U05 整包剩余=其余八子包（hearth/yunchou/swd/ow/xianjian/mtg/
  zhenfa/wuxing）同模板续验 + 角色/技能/模式/配置面。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（4 文件）。
