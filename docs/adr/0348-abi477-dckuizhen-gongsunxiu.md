# ADR 0348：ABI 477 溃阵决斗分支复合 + huicui 公孙修

本批为 huicui 包公孙修（`gongsunxiu`，3377163409）的「溃阵」（`dckuizhen`，
2730814674）完整移植，需要 ABI 477 双阶段复合。

## 原语设计（ABI 477）

- 第一阶段 `resolve_dckuizhen(player)`：`trigger phase_play owner_is_target`（每回合
  一次）；目标过滤（手牌数 ≥ 你 或 体力 ≥ 你）→ 以 `AddMark(owner,
  DCKUIZHEN_PENDING_MARK, target.raw())` 记录发起者（mark count 存 id）→
  `UseVirtualCardOn(target, Juedou, owner)`。决斗的响应窗口由游戏循环驱动。
- 第二阶段 `resolve_dckuizhen_resolve(player, winner, loser)`：`trigger duel_resolved
  owner_alive`；按待结算 mark 过滤本次决斗，结算后移除 mark：`loser == owner`（落败
  受伤）→ 获得发起者所有手牌【杀】；`winner == owner`（获胜）→ 发起者失去 1 点体力。
  DuelResolved 上下文 `player = winner`、`source = loser`（vm_host 捕获）。

## 打包与验证

- `dckuizhen.skill` 加入 huicui README 显式列表；`gongsunxiu`（男/群/4 体力，技能
  [dckuizhen]）加入 huicui `characters.json`；catalog 标记 `ported`。
- huicui npack 重建（ABI 477，26 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/huicui/`。
- 运行时测试 `original_huicui_dckuizhen.rs`（2 用例全绿）：落败分支（无手牌被决斗
  伤害，获得目标手牌【杀】）；获胜分支（以【杀】应答，目标受决斗伤害并再失去
  1 点体力）。
- 旧 JavaScript 不进入 npack 或运行时。
