# ADR 0724：N08 表内技能授予、动态资格与卸装效果

日期：2026-09-16
状态：已实施（engine-native）
关联：[0720](0720-native-judgement-order-equip-qualification-v6.md)、[0722](0722-native-n05-active-judgement-matrix-reshuffle.md)、进度文档 N08

## 独立复核更正（2026-09-22）

debug_grant_general 的确会授予目录条目中的全部技能，但“目录条目完整”未经证明。
当前张角配置为鬼道+天妒，与本地旧张角雷击/鬼道/黄天不符；多个标准武将也缺技能。
本批证明组合与动态资格机制，不证明完整官方整将迁移。

当前反例、优先级与证据见 [进度主稿](../architecture/native-progress.md)。以下保留当批设计/记录，不作为当前完整验收结论。

## 背景与决定

进度文档 N08 剩余三项：动态技能资格（技能随状态启停）、完整武将授予（武将级技能集）、
装备卸装效果终止（卸装后射程/效果即时失效）。本批以最小内容 API + 行为级回归交付。

### 1. 完整武将授予（`debug_grant_general`）

内容层新增武将表 `STANDARD_GENERALS`（张角 = 鬼道 + 天妒），引擎新增按武将名一次性
授予全部技能的 `debug_grant_general`。回归 `general_bundle_grant_full_skill_set`：
单名玩家持完整武将技能集，同一判定流内完成 改判（自出黑牌）→ Finalized 恰一次 →
天妒 Confirm 获得——武将两个技能在同一流程协同，事件与守恒逐项断言。

### 2. 动态技能资格（启/停 + 死亡）

引擎新增 `debug_set_skill_enabled`（切换已授予技能的 `enabled`，未授予即 panic）。
资格判定的两条动态路径均有回归：

- **启**（`skill_disabled_gate_excludes_rewriter`）：鬼道授予但 disabled → 不进改判
  资格队列，判定静默完成（无询问、无 Rewritten、Finalized 恰一次）；
  重新 enable 后下一轮判定改判询问恢复（第二条乐 → gui_dao_rewrite 询问重现）。
- **死亡**（`dead_skill_holder_excluded_from_rewrite`）：改判者判定前死亡，即使手持
  黑牌也被资格队列排除（kernel 询问循环先查存活再查黑牌），判定静默完成。

体力（hp）门控：当前内容无体力条件技能实例，诚实记录为「无内容实例」，
不虚构验收；死亡/启停两条机制路径均已锁定。

### 3. 装备卸装效果终止（`weapon_replacement_terminates_old_range`）

标准规则无独立「卸装」动作；替换装备即卸装旧装备（`commit_equip`：旧牌进弃牌堆恰一份、
`Equipped.replaced` 记旧牌）。本回归用**行为级证据**证明旧武器效果即时终止（射程回落）：

射程1 够不到距离2 → 装 weapon2（射程2）可达 → 守方 +1 馬 使距离 3 →
替换 weapon3（射程3）可达、weapon2 弃置恰一份、`Equipped{replaced: weapon2}` →
再替换 weapon2b（射程2）后 **距离 3 再次不可达**（IllegalTargets，非杀数上限）——
若旧 weapon3 射程未终止（max 语义下残留在装备区），射程不可能回落到 2。

## 影响

- 测试 138 项（native_kernel 80、boundary_snapshot 53、场景 2、源码 2、调试 1），净增 4。
- 规则版本不变（6）；golden 不变。`STANDARD_GENERALS` 为内容静态表（无需版本递增：
  授予路径为测试/工具 API，不改对局语义）。
- N08 剩余：更多武将/技能内容迁移（按武将表扩条目）、armor 等无效果装备的语义落地；
  N09 分配/跨目标对照、N04 检查点段关系、N05 负体力救援、N06 重放事件对照继续。
