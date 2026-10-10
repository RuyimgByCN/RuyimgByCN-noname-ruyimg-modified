# ADR 0361：ABI 485 匡扶弃装授杀复合 + huicui 潘凤

本批为 huicui 包潘凤（`re_panfeng`，2168487745）的「匡扶」（`xinkuangfu`，
3510695748）完整移植，需要 ABI 485 复合 `resolve_xinkuangfu(player)`。

## 原语设计（ABI 485）

- DSL 表面：`resolve_xinkuangfu(player)`；`trigger phase_play owner_is_target`。
  编译为 `effect_tag::RESOLVE_XINKUANGFU`（tag 417）。
- 引擎：`EffectRequest::ResolveXinkuangfu { player, target, sha_target, stage, skill }`；
  阶段机（Custom 续体 `RESOLVE_XINKUANGFU_CUSTOM_CHOICE`）：
  - 阶段 1：选装备目标（可为自己，需装备非空）；
  - 阶段 2：弃其装备区一张（`TargetCardScope::Equip`）；
  - 阶段 3：选【杀】目标 → `UseVirtualCardOn(player, Sha, target)`（无距离）→ 快照
    比对伤害；
  - 分支：自弃装备且受伤 → 摸两张；他弃装备且未受伤 → 弃两张手牌（阶段 4）。

## 文档化近似

【闪】响应窗口开启时分支跳过（与溃阵/魔狱同因）。

## 打包与验证

- `xinkuangfu.skill` 加入 huicui README 显式列表；`re_panfeng`（男/群/4 体力，技能
  [xinkuangfu]）加入 huicui `characters.json`；catalog 标记 `ported`。
- huicui npack 重建（ABI 485，34 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/huicui/`。
- 运行时测试 `original_huicui_xinkuangfu.rs`（1 用例全绿）：自弃装备 + 虚拟【杀】
  命中 + 摸两张。
- 旧 JavaScript 不进入 npack 或运行时。
