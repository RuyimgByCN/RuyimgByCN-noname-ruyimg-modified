# ADR 0339：ABI 468 高变受损者二选复合 + sixiang 标王沈

本批为 sixiang 包标王沈（`std_wangshen`，3312867416）的「高变」（`stdgaobian`，
4024217730）完整移植，需要 ABI 468 复合 `resolve_stdgaobian(player)`。

## 原语设计（ABI 468）

- DSL 表面：`resolve_stdgaobian(player)`；`trigger phase_end owner_alive_not_target`
  （其他角色结束阶段）。编译为 `effect_tag::RESOLVE_STDGAOBIAN`（tag 389）。
- 引擎：`EffectRequest::ResolveStdgaobian { player, chooser, skill }`；阶段机
  （Custom 续体 `RESOLVE_STDGAOBIAN_CUSTOM_CHOICE`）：
  - 判定本回合恰好一名存活角色受到伤害（`turn.player_received_damage`）；
  - 该角色二选：0 = 对一名其他角色视为使用一张【杀】（`UseVirtualCardOn`）；
    1 = 重置拥有者【安忍】计数（`RemoveMark` mark 221181016 全量）。

## 文档化近似

旧 JS 的【杀】取自本回合进入弃牌堆的具体【杀】实体（chooseToUse），DSL 以虚拟
【杀】表达（实体身份不影响结算）。

## 打包与验证

- `stdgaobian.skill` 加入 sixiang README 显式列表；`std_wangshen`（男/魏/3 体力，
  技能 [stdanran, stdgaobian]）加入 sixiang `characters.json`；catalog 标记
  `ported`。
- sixiang npack 重建（ABI 468，70 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/sixiang/`。
- 运行时测试 `original_sixiang_stdgaobian.rs`（3 用例全绿）：【杀】分支（虚拟杀命中
  1 点伤害）、重置分支（安忍计数清零）、双人受伤不触发。
- 旧 JavaScript 不进入 npack 或运行时。
