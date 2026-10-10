# ADR 0613：gwent vanish 机制落地（gw_wuyao/gw_lang 回合末消失）

> 承接 [ADR 0607](0607-huanle-shoulijian-abi642-gwent.md)「后续 2」与
> [ADR 0612](0612-gwent-hupeng-abi644-choose-one-of-two-skills.md)「后续 2」：为
> gw_wuyao/gw_lang 两张特殊卡补全 vanish（回合结束后从手牌消失），关闭 gwent 卡片
> 生命周期（获得 → view_as 使用 → 回合末消失）。**无新增 ABI**（复用既有原语）。

## 上游语义（旧 JS 仅作语义证据，不进入运行时）

g_gw_yewu（global 技能）：`trigger: { player: "phaseAfter" }`，`silent`——取持有者手牌中
全部 gw_wuyao/gw_lang，`player.lose(cards).position = null` 移出游戏（vanish = 移除而非
弃置）。

## 做了什么

**DSL（shahai_impl.skill）**：gw_wuyao_skill(784509644) 与 gw_lang_skill(2836710318) 各加
一条 `trigger phase_end owner_is_target { effect remove_skill_by_def(self, <自身 def>); }`。

- 本引擎以技能近似特殊卡（卡之代理）；「回合末从手牌消失」以「回合末移除自身技能」表达。
- `remove_skill_by_def(player, def)` 为既有原语（sbyangwei/oldwuji 等先例），`phase_end
  owner_is_target` 为既有过滤器（shahai 内 tuteng5 同款）——**无需新 ABI、无 version bump**。
- 触发挂在技能本体上，仅当持有者拥有该技能实例时才会触发，移除语义自洽（无技能则无触发）。

**近似说明**（已写入 DSL 注释）：vanish 的移除对象为技能实例（卡之代理）；gwyewu（叶舞）
造卡入口——「弃一张手牌 + 随机弃敌人一张随机牌 + 同色重复 + 每弃敌方手牌获雾妖/每弃
敌方装备获狼」——仍为元数据登记（shahai_meta.skill），需「随机弃牌」引擎原语，独立
后续工作。

## 验证

| 检查 | 结果 |
|---|---|
| conformance | **4 测试全过**（shahai_impl 含 vanish 触发器编译通过）✅ |
| 失败集合 | **246 = 247 − 1**：唯一差异为 `standard_npack` 出集（并行代理修复标准包资产安装，本 ADR 仅改动 shahai_impl.skill，未触 content-runtime 与标准包）——零新增失败 ✅ |
| ABI | 不变（RULE_VM_ABI_VERSION 644，`abort_bump_without_claim` 继续过）✅ |

## 后续

1. **gwyewu（叶舞）造卡入口**：需「随机弃敌方一张随机牌 + 同色重复」引擎原语，独立工作。
2. **gw_dudayuanshuai2 per-card 禁弃置**：需按自定义卡 def 的 card 门禁选择器（ADR 0608 已登记）。
3. **国战 12 锦囊结算**：ABI 645+ 待办（复用 ADR 0608/0612 模式）。
