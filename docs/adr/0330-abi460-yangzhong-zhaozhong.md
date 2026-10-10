# ADR 0330：ABI 460 通用弃牌数量原语 + SP2 赵忠

本批为 SP2 包赵忠（`zhaozhong`）的「殃众」（`yangzhong`，4088525000）完整移植，
需要新的通用原语 `discard_hand_equip_count(player, count)`（ABI 460）：选择恰好
`count` 张可弃置手牌/装备并弃置（通用弃牌代价）。

## 原语设计（ABI 460）

- DSL 表面：`discard_hand_equip_count(player, count)`（count 为正整数）；编译为
  `effect_tag::DISCARD_HAND_EQUIP_COUNT`（tag 375）。
- 引擎：`EffectRequest::DiscardHandEquipCount { player, count, skill }`；
  选择结算走 `ContinuationKind::Custom` + 载荷魔法标记
  `DISCARD_HAND_EQUIP_COUNT_CUSTOM_CHOICE`（0xDCE9_4600）：候选 = 拥有者可弃置
  手牌/装备，恰好选 count 张（min=max=count，强制）；结算时 `DiscardCards` 弃置。
- 通用性：可作为任意“弃置 N 张牌”代价的通用原语。

## 技能移植

「殃众」（`yangzhong`，4088525000）：
- `trigger damage owner_is_source_or_target optional by event_source`：伤害来源确认后
  `discard_hand_equip_count(event_source, 2)` 弃两张牌，`lose_hp(event_player, 1)`
  令受伤角色失去 1 点体力。

## 打包与验证

- `yangzhong.skill` 加入 `content/original/ported/sp2/`（README 显式列表补充）；
  赵忠（3589727417，男/群/6 体力，技能 [yangzhong 4088525000,
  huangkong 2905812698]）加入 sp2 `characters.json`；catalog 标记
  `yangzhong` 为 `ported`。
- sp2 npack 重建（ABI 460，28 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/sp2/`。
- 运行时测试 `original_sp2_yangzhong.rs` 全绿：伤害来源确认后弃两张牌、
  受伤角色在 1 点伤害之外再失去 1 点体力。
- 旧 JavaScript 不进入 npack 或运行时。
