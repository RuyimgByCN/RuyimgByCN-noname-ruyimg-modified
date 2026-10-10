# ADR 0352：ABI 481 魔狱夺牌反击复合 + huicui 裴元绍

本批为 huicui 包裴元绍（`peiyuanshao`，3982852073）的「魔狱」（`dcmoyu`，
3259808437）完整移植，需要 ABI 481 复合 `resolve_dcmoyu(player)`。

## 原语设计（ABI 481）

- DSL 表面：`resolve_dcmoyu(player)`；`trigger phase_play owner_is_target`。
  编译为 `effect_tag::RESOLVE_DCMOYU`（tag 409）。
- 引擎：`EffectRequest::ResolveDcmoyu { player, skill }`；阶段机（Custom 续体
  `RESOLVE_DCMOYU_CUSTOM_CHOICE`）：
  - 选目标（hej 非空且无「已用」mark；mark key = 目标 id）；
  - 获得 1（+1 若 `DCMOYU_ADD_MARK` 已置）张 hej（`discardable_target_cards` 确定性
    取前几张）→ 消耗 add 标记 → `UseVirtualCardOn(target, Sha, owner)`；
  - 伤害快照比对：受伤 → `BanActiveUntilTurnEnd`（本回合禁用）+ 目标 mark；
    未受伤 → 置 `DCMOYU_ADD_MARK`（下次多拿一张）+ 目标 mark。

## 文档化近似

【闪】响应窗口开启时分支跳过（与溃阵同因）；每次随机取牌以确定性取前几张近似。

## 打包与验证

- `dcmoyu.skill` 加入 huicui README 显式列表；`peiyuanshao`（男/群/4 体力，技能
  [dcmoyu]）加入 huicui `characters.json`；catalog 标记 `ported`。
- huicui npack 重建（ABI 481，29 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/huicui/`。
- 运行时测试 `original_huicui_dcmoyu.rs`（2 用例全绿）：受伤分支（获得 1 张 +
  虚拟【杀】命中 + 本回合禁用 + 无 add 标记）；未受伤分支（持有【闪】闪避后无
  额外伤害）。
- 旧 JavaScript 不进入 npack 或运行时。
