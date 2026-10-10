# ADR 0335：ABI 464 铤险摸牌+杀目标排除原语 + SP2 孙狼

本批为 SP2 包孙狼（`sunlang`）的「铤险」（`dctingxian`，3933341702）完整移植，
需要新的过滤器 `owner_source_sha_once_per_turn` 与复合原语
`resolve_dctingxian(player)`（ABI 464）。

## 原语设计（ABI 464）

- 过滤器 `owner_source_sha_once_per_turn`：`card_targeted` 事件中拥有者为使用者、
  牌为【杀】且本回合未发动（内建 once-per-turn）。
- 复合原语 `resolve_dctingxian(player)`：编译为 `effect_tag::RESOLVE_DCTINGXIAN`
  （tag 381）；vm_host 从 `card_targeted` 上下文捕获当前【杀】（`event_card`）与
  完整目标集（`event_targets`）；先 `DrawCards(装备数+1)`，再打开 1..=min(目标数,
  装备数+1) 的目标选择（`ContinuationKind::Custom` + `RESOLVE_DCTINGXIAN_CUSTOM_CHOICE`
  0xDCF1_4640）；结算时逐目标 `exclude_saved_sha_target` 令此【杀】对其无效。

## 技能移植

「铤险」（`dctingxian`，3933341702）：
- `trigger card_targeted owner_source_sha_once_per_turn`：`resolve_dctingxian(self)`。

## 文档化近似

旧 JS 于最后一个目标结算时触发（目标集已完整）；DSL 于首次目标指定时触发
（每回合一次），排除仍作用于同一【杀】的完整目标集（`card_targeted` 上下文携带
完整目标列表）。

## 打包与验证

- `dctingxian.skill` 加入 `content/original/ported/sp2/`（README 显式列表补充）；
  孙狼（4058623596，男/群/4 体力，技能 [dctingxian 3933341702,
  dcbenshi 3177558288]）加入 sp2 `characters.json`；catalog 标记
  `dctingxian` 为 `ported`。
- sp2 npack 重建（ABI 464，34 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/sp2/`。
- 运行时测试 `original_sp2_dctingxian.rs` 全绿：出牌阶段使用【杀】触发摸牌与排除
  选择；被排除目标不因该【杀】受到伤害。
- 旧 JavaScript 不进入 npack 或运行时。
