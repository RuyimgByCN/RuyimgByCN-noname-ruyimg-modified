# ADR 0405：SHIJI 王甫赵累闭环（殉义，ABI 412）

本批发布 SHIJI 王甫赵累（`wangfuzhaolei`，4038165124，男/蜀/4 血）。技能
「殉义」（`xunyi`）完成闭环；旧 JavaScript 仅作语义来源。

## 决策

- **ABI 412 `resolve_xunyi(player, mode)`**：
  - mode 0：回合开始阶段（旧 JS 为开局 phaseBefore，文档化近似为第 1 回合
    phase_prepare）选择一名其他角色为“义”目标，其获得“义”标记
    （标记定义 ID = xunyi 技能 ID 2517009871）。一段选目标 choice
    （`ContinuationKind::Custom` + `RESOLVE_XUNYI_SELECT`）。
  - mode 1：damage 事件分流（拥有者存活时）——拥有者造成伤害 → 义目标摸 1；
    义目标造成伤害 → 拥有者摸 1；拥有者受伤 → 义目标强制弃 1 张手牌/装备
    （`ChooseDiscardOneHandOrEquip`）；义目标受伤 → 拥有者强制弃 1 张。
- 义标记者查找：存活角色中唯一持有义标记的非拥有者（同武将多实例边缘情况
  文档化）。
- 旧 JS 的 dieAfter 重选“义”目标未建模，文档化近似。

## 验证

- 新增 `original_shiji_xunyi.rs` 4 个行为测试：王甫造成伤害义目标摸牌、
  义目标受伤王甫弃牌、王甫受伤义目标弃牌、选目标流程（标记授予）。
- 打包：51 技能 / 12 武将，`rule_vm_abi_version` 502。
- 顺手补齐并行会话（omp，ABI 531 烈刃）缺失的 `choose_hand_compare` codegen
  闭合臂与 531 常量。
