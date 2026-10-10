# ADR 0327：ABI 458 链舟同体力横置原语 + HUICUI 蔡瑁张允

本批为 HUICUI 包蔡瑁张允（`caimaozhangyun`）的「链舟」（`lianzhou`，3041401308）
完整移植，需要新的复合原语 `choose_link_other_same_hp(player)`（ABI 458）：
准备阶段选择任意名体力值与拥有者相同且未横置的其他角色并横置之。

## 原语设计（ABI 458）

- DSL 表面：`choose_link_other_same_hp(player)`，仅允许在 `phase_prepare` 触发中
  使用（typecheck 显式校验）；编译为 `effect_tag::RESOLVE_LIANZHOU`（tag 371）。
- 引擎：`EffectRequest::ResolveLianzhou { player, targets: Vec<PlayerId>, skill }`；
  无需 vm_host 上下文捕获（不依赖伤害/用牌上下文）。
- 选择结算走 `ContinuationKind::Custom` + 载荷魔法标记
  `RESOLVE_LIANZHOU_CUSTOM_CHOICE`（0xDCE3_4580）：候选 = 其他存活、未横置、
  体力值与拥有者相同；多目标选择（1..=候选数，强制）；结算时逐目标
  `AddMark(MARK_CHAIN, +1)` 横置。
- 拥有者自身横置由 DSL 侧 `add_mark(self, 2, 1)` 表达（复用 MARK_CHAIN 定义，
  先例：spfenming ABI 402）。

## 技能移植

「链舟」（`lianzhou`，3041401308）：
- `trigger phase_prepare owner_is_target locked`：未横置时先自链，再
  `choose_link_other_same_hp(self)`；无候选时原语自行空转。

## 文档化近似

旧 JS 过滤“存在同体力未横置角色”才触发；DSL 以固定触发 + 原语空转近似（行为等价）。

## 打包与验证

- `lianzhou.skill` 加入 `content/original/ported/huicui/`；蔡瑁张允
  （2957365815，男/群/4 体力，技能 [lianzhou 3041401308, jinglan 3782183327]）
  加入 huicui `characters.json`；catalog 标记 `lianzhou` 为 `ported`。
- huicui npack 重建（ABI 458，23 模块），部署至 `extensions/huicui/`；README 补充。
- 运行时测试 `original_huicui_lianzhou.rs` 全绿：自链 + 仅候选（同体力未横置）被
  横置、已横置角色不被重复选择。
- 旧 JavaScript 不进入 npack 或运行时。
