# ADR 0395：shenhua 孙策制霸 + 张绣从俭（ABI 523-524）

本批完整移植 SHENHUA 孙策（`sunce`，制霸补齐江/魂姿闭包）并发布张绣「从俭」技能
定义；旧 JavaScript 仅作为语义来源，不进入运行时。SHENHUA npack 的
`rule_vm_abi_version` 提升至 524。

## ABI 523：`resolve_zhiba` / `resolve_zhiba_settle` — 制霸

- 主公技（`lord_only`）：新过滤器 `owner_alive_other_wu_play_phase_can_compare_owner`
  精确匹配其他吴势力角色的出牌阶段、双方均有手牌、当前回合角色为发起方；可选确认的
  观察者为发起方（`optional by event_player`）。
- 觉醒拒绝：以魂姿/界魂姿技能定义是否仍存在于拥有者为觉醒判定（旧 JS `storage.hunzi`
  等价）。已觉醒时先向拥有者开拒绝选择，拒绝则整次结束；未觉醒直接进入拼点。
- 拼点：在技能实例上记录待决发起方（`zhiba_compare` 状态，复用
  `state_blob` 标签机制），复用通用拼点选牌/展示/移牌续体；`validate_hand_compare_start`
  与 `hand_compare_commit_context` 为制霸场景（技能属于拼点目标方）增加授权分支，
  `commit_hand_compare` 按胜者路由到 `resolve_zhiba_settle`。
- 结算：仅当发起方未赢（平局或拥有者赢）时向拥有者开取牌选择，确认后获得仍处弃牌堆的
  两张拼点牌；结算或发起方获胜时清除实例状态。

## ABI 524：`resolve_drlt_congjian` — 从俭（张绣）

- 新过滤器 `owner_is_target_multi_target_trick_has_hand_equip`：`card_targeted` 时机、
  拥有者为目标之一、锦囊目标数大于一、拥有者有可弃置手牌/装备。
- 结算三段：从触发事件目标中选择一名其他存活目标（屏障重校验）→ 选择一张可弃置
  手牌/装备牌 → 经通用赠予管线给出并摸一张；所给为装备牌时摸两张（旧 JS 语义）。
- 张绣的「雄乱」依赖装备栏废除、判定区废除与目标用牌禁令等尚未落地的机制，因此本批
  只发布「从俭」技能定义，暂不把张绣加入选将池（沿用「只发布技能定义」惯例）。

## 打包与验证

- `zhiba.skill`、`drlt_congjian.skill` 加入 SHENHUA 包；`sunce` 加入
  `characters.json`（江/魂姿跨包既有定义闭包）。重建 npack 并同步
  `extensions/shenhua/`（ABI 524、97 技能、25 武将），重建可复现；同步更新包级契约
  断言（版本与索引计数）。
- Golden 覆盖：觉醒拒绝后继续拼点与发起方获胜不取牌；未觉醒无拒绝选择、发起方未赢时
  拥有者取回两张拼点牌；从俭目标选择排除自身、赠予摸牌与取消不动牌堆。新测试
  `original_shenhua_zhiba.rs`、`original_shenhua_drlt_congjian.rs` 全绿，36 个
  shenhua 测试二进制与 conformance 8/8 通过。
