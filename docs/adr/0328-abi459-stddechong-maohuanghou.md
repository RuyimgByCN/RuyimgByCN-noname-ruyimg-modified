# ADR 0328：ABI 459 赠牌给触发角色原语 + SIXIANG 标毛皇后

本批为 SIXIANG 包标毛皇后（`std_maohuanghou`）的「得宠」（`stddechong`，2987704809）
完整移植，需要新的复合原语 `choose_give_hand_equip_to_event_player(player)`（ABI 459）：
选择任意张可弃置手牌/装备并交给触发事件角色。

## 原语设计（ABI 459）

- DSL 表面：`choose_give_hand_equip_to_event_player(player)`，编译为
  `effect_tag::RESOLVE_STDDECHONG`（tag 372）；vm_host 从触发上下文捕获接收者
  （`event_player`）。
- 引擎：`EffectRequest::ResolveStddechong { player, target, cards, skill }`；
  选择结算走 `ContinuationKind::Custom` + 载荷魔法标记
  `RESOLVE_STDDECHONG_CUSTOM_CHOICE`（0xDCE7_4590）：候选 = 拥有者可弃置手牌/装备，
  多选（1..=候选数，强制）；结算时按来源分区 `MoveCards` 移入接收者手牌。

## 技能移植

「得宠」（`stddechong`，2987704809）：
- `trigger phase_prepare owner_alive_not_target_hand_equip_nonempty`：
  `choose_give_hand_equip_to_event_player(self)` 赠牌 +
  `add_bound_skill(self, stddechong2, event_source)`。
- 「得宠·后续」（`stddechong2`，4229331568，新派生技能）：
  `trigger phase_discard owner_alive_skill_bound_target_is_event_player optional`，
  手牌不小于体力（`hand_count < hp` 的 else 分支）时 `deal_damage(self, event_source, 1)`，
  随后 `remove_skill(self_skill)`。

## 文档化近似

旧 JS 仅在目标手牌≥体力时询问是否伤害；DSL 以 optional 确认 + 体内条件守卫近似
（条件不满足时确认后空转）。

## 打包与验证

- `stddechong.skill`（含 `stddechong2`）加入 `content/original/ported/sixiang/`；
  标毛皇后（3720033842，女/魏/3 体力，技能 [stddechong 2987704809,
  stdyinzu 3812447605]）加入 sixiang `characters.json`；catalog 标记
  `stddechong` 为 `ported` 并新增 `stddechong2`。
- sixiang npack 重建（ABI 459，68 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/sixiang/`。
- 运行时测试 `original_sixiang_stddechong.rs` 全绿：赠牌入目标手牌 + 目标弃牌阶段
  拥有者确认后造成 1 点伤害。
- 旧 JavaScript 不进入 npack 或运行时。
