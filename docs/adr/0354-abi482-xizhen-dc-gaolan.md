# ADR 0354：ABI 482 袭阵视杀/决斗复合 + huicui 高览

本批为 huicui 包高览（`dc_gaolan`，3939433103）的「袭阵」（`xizhen`，
4003133285）完整移植，需要 ABI 482 复合 `resolve_xizhen(player)`。

## 原语设计（ABI 482）

- DSL 表面：`resolve_xizhen(player)`；`trigger phase_play owner_is_target`。
  编译为 `effect_tag::RESOLVE_XIZHEN`（tag 410）。
- 引擎：`EffectRequest::ResolveXizhen { player, skill }`；阶段机（Custom 续体
  `RESOLVE_XIZHEN_CUSTOM_CHOICE`）：选目标（其他存活）→ 分支 [1=【杀】, 2=【决斗】]
  → `UseVirtualCardOn(player, chosen, target)`。

## 文档化近似

旧 JS 的响应代偿子技能（你被要求响应时，标记目标回复 1 点体力且你摸 1-2 张）依赖
「被要求响应」事件，DSL 无此事件，文档化省略。

## 打包与验证

- `xizhen.skill` 加入 huicui README 显式列表；`dc_gaolan`（男/群/4 体力，技能
  [xizhen]）加入 huicui `characters.json`；catalog 标记 `ported`。
- huicui npack 重建（ABI 482，30 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/huicui/`。
- 运行时测试 `original_huicui_xizhen.rs`（2 用例全绿）：【杀】分支命中 1 点；
  【决斗】分支正常结算无悬挂选择。
- 旧 JavaScript 不进入 npack 或运行时。
