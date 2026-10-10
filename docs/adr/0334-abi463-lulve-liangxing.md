# ADR 0334：ABI 463 掠卤目标二选一原语 + SP2 梁兴

本批为 SP2 包梁兴（`liangxing`）的「掠卤」（`lulve`，3019895260）完整移植，
需要新的复合原语 `resolve_lulve(player)`（ABI 463）：选择一名手牌数更少的其他角色，
由该角色二选一（交全部手牌并令拥有者翻面 / 自己翻面并视为对拥有者使用【杀】）。

## 原语设计（ABI 463）

- DSL 表面：`resolve_lulve(player)`；编译为 `effect_tag::RESOLVE_LULVE`（tag 379）。
- 引擎：`EffectRequest::ResolveLulve { player, target, branch, skill }`；
  三段选择走 `ContinuationKind::Custom` + `RESOLVE_LULVE_CUSTOM_CHOICE`
  （0xDCEF_4630）：拥有者选目标（手牌 0 < 目标 < 拥有者）→ 目标选分支
  （选择观察者为目标）→ 应用。
- 分支 0：整手 `MoveCards` 移给拥有者 + `TurnOver(拥有者)`；
  分支 1：`TurnOver(目标)` + `UseVirtualCardOn(目标, Sha, 拥有者)`。

## 技能移植

「掠卤」（`lulve`，3019895260）：
- `trigger phase_play owner_is_target`：`resolve_lulve(self)`（无候选时原语空转）。

## 打包与验证

- `lulve.skill` 加入 `content/original/ported/sp2/`（README 显式列表补充）；梁兴
  （2344865342，男/群/4 体力，技能 [lulve 3019895260, lxzhuixi 2387115210]）加入
  sp2 `characters.json`；catalog 标记 `lulve` 为 `ported`。
- sp2 npack 重建（ABI 463，33 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/sp2/`。
- 运行时测试 `original_sp2_lulve.rs` 双分支全绿：分支 0（目标整手移交 + 拥有者翻面）、
  分支 1（目标翻面 + 视为使用【杀】）。
- 旧 JavaScript 不进入 npack 或运行时。
