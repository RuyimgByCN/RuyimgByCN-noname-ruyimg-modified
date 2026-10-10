# ADR 0332：ABI 461 劫粮获牌原语 + SP2 韩猛

本批为 SP2 包韩猛（`hanmeng`）的「劫粮」（`jieliang`，4227370894）完整移植，
需要新的复合原语 `resolve_jieliang_gain(player)`（ABI 461）：当前回合角色于弃牌
阶段弃牌后，选择一张仍处弃牌堆的弃牌获得。

## 原语设计（ABI 461）

- DSL 表面：`resolve_jieliang_gain(player)`；编译为 `effect_tag::RESOLVE_JIELIANG_GAIN`
  （tag 376）；vm_host 从 `cards_moved` 上下文捕获弃牌者（`event_source`）与
  本次移动牌（`event_cards`）。
- 引擎：`EffectRequest::ResolveJieliangGain { player, mover, cards, skill }`；
  内部校验：mover == 当前回合角色 && mover != 拥有者 && 当前阶段为弃牌阶段；
  过滤仍处弃牌堆的事件牌后打开可取消的单牌选择（`ContinuationKind::Custom` +
  `RESOLVE_JIELIANG_GAIN_CUSTOM_CHOICE` 0xDCEB_4610）；结算时 `MoveCards`
  移入拥有者手牌。
- 复用：`draw_count`/`hand_limit` 修饰符表达摸牌数与手牌上限削减（纯 DSL）。

## 技能移植

「劫粮」（`jieliang`，4227370894）：
- `trigger phase_draw owner_alive_not_target_hand_equip_nonempty optional`：
  `discard_one_hand_or_equip(self)` 弃一张牌，授予目标「劫粮·削减」
  （3795806305，`draw_count -1` + `hand_limit -1`，目标回合结束移除），
  并授予自己「劫粮·截获」（2999325050，`cards_moved` 触发
  `resolve_jieliang_gain(self)`）。

## 文档化近似

旧 JS 的 `trigger.num--` 以 `draw_count -1` 修饰符等价表达；摸牌数 >1 守卫未移植
（摸牌数为 1 时削为 0）。旧 JS 的 `phaseDiscard` 父事件判定以复合原语内部校验
（当前回合角色 + 弃牌阶段）等价表达。

## 打包与验证

- `jieliang.skill`（含 `jieliang_less`/`jieliang_gain`）加入
  `content/original/ported/sp2/`（README 显式列表补充）；韩猛（2358742012，
  男/群/4 体力，技能 [jieliang 4227370894, quanjiu 2358780202]）加入 sp2
  `characters.json`；catalog 标记 `jieliang` 为 `ported` 并新增两个派生技能。
- sp2 npack 重建（ABI 461，32 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/sp2/`。
- 运行时测试 `original_sp2_jieliang.rs` 全绿：目标摸牌阶段削减生效（获得
  `jieliang_less`）、目标弃牌阶段弃牌后拥有者截获该牌。
- 旧 JavaScript 不进入 npack 或运行时。
