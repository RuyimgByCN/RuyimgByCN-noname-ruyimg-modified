# 2026-11-09 进度核对扫描（Batch 24–26 之后）

- 核对人：Claude Code；时间 2026-11-09 11:19–12:00（CST）
- HEAD：`1a7d11fc3`（纯文档；最后代码提交 `cf9c6619a`），非文档路径干净（仅 `.pi-lens.json` 未跟踪）；规则 333／NGSA 107／content schema 5／ABI 759／WS8
- 范围：核对 `704eb0d7e..1a7d11fc3` 共 14 个提交（Batch 24 裁决 33、裁决 34 前端面板、PA09 macOS；Batch 25 裁决 34 子技／战役；Batch 26 裁决 34 余 9 来源及占位标识符修正）

## 实跑命令与结果

| 命令 | 结果 | 原始输出 |
| --- | --- | --- |
| `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --no-fail-fast` | exit 0；**2219 passed / 0 failed / 6 ignored**（17 段）；剩余 6 个均为手工延迟／AI 质量／状态增长／悬挂引用探针，规则探针 0 ignore | [three-crates.log](three-crates.log)、[忽略清单](three-crates-ignored.txt) |
| `cargo test --offline --workspace --no-fail-fast` | exit 0；**7611 passed / 0 failed / 8 ignored**（178 段） | [workspace.log](workspace.log) |

测试全绿只说明代码与它自己的测试一致；下面的抽查针对的是测试所断言的行为是否等于上游。

## 裁决 33（苍螭之璧）

抽查 `filtering/swd.rs`、`host/asks.rs`、`step/swd_equipment.rs`：回合开始且手持苍螭之璧时，选 1–3 名存活角色横置／重置，与上游 `杀海拾遗/card/swd/skill.js` 的 `g_cangchizhibi`（`phaseBegin`、`chooseTarget([1, 3])`、`link()`）一致。本项可以接受。

## 裁决 34：抽查发现的问题

裁决 34 要求 deferred 渲染面与单机战役脚本「逐来源实现，不得以数据桥接契约收口」。上游 37 个来源合计约 86 万行 JS（如十周年UI 78,879 行 JS＋22,638 行 CSS、太虚幻境 114,092 行、3D精选 `character/skill.js` 6,368 行）。Batch 24–26 的对应实现是 `extensionSurfaceRuntime.ts`（1,128 行）、`ExtensionSurfacePanel.vue`（930 行）、`decision34.rs`、`decision34_pack9.rs`、`decision34_closure.rs`、`decision34_pack9{,_data}.rs` 等约 3,000 行。抽查结果：

1. **Spine 骨骼动画是示意图**：`ExtensionSurfacePanel.vue` 的 `drawSpineSkeletalCanvas()` 在 2D canvas 上画一个圆、两条线和一行 `Spine:<clip>#<seq>` 文字；`triggerSpineSkinClip()` 只轮换一个字符串和三个坐标数。`apps/web-client/package.json` 未引入任何 Spine 运行库，也没有加载 `.skel`／`.atlas`。
2. **U01 3D精选 64 个技能**：`decision34_pack9.rs::resolve_u01_ddd_skill` 只为 8 个技能名写了分支（其中 6 个两两共用 3 段逻辑），其余全部落入 `_ => ctx.start_draw(owner, 1)`。有分支的也与上游不符：上游 `dddxiaxing`（侠行）是「每轮限一次，濒死或回合开始时把一张牌置于牌堆顶，视为对方使用【桃】／【酒】，获得『侠』；移去『侠』视为使用【杀】／【闪】」，原生实现是设标记、可选横置一名角色、摸 1 张牌。`decision34_pack9_closure.rs` 断言的正是这一自造行为。
3. **L11 太虚幻境战役**：`decision34.rs::resolve_taixu_expedition_step` 只把层数加 1、设祝福标记、对第一个存活的其他角色造成 1 点伤害。
4. **L08 十周年UI**：上游 `_usecard`／`_chongzhu` 等是界面动画钩子；原生把它们实现成「触发时摸 1 张牌」。`L08_DECADE_CHARACTERS` 列出的 9 个武将（`decade_caocao` 等）在上游十周年UI 目录中检索不到（0 命中），是凭空生成的。
5. **未接入产品路径**：`decision34_pack9_extension_pack`／`decision34_pack9_for_coexistence`／`decision34_closure_extension_pack` 只被 `crates/engine-native/tests/` 调用，没有接进任何 `packages/extensions/*` 正式包或产品路径；Batch 26 首个提交还带有占位标识符，后由 `cf9c6619a` 替换。
6. **「60 FPS」**：headless Chrome 帧率测的是上述示意面板，不能代表上游渲染面的性能。

## 结论

- 签署表与扩展台账写的「37/37 来源仓内 7 维 100% ✅」不成立：功能与规则差异两维的测试断言的是原生自造行为，而非上游语义。裁决 34 视为**未实现**，Batch 24–26 的裁决 34 代码只能当作脚手架，需按上游逐项重做或删除。
- 整包八项签署仍为 **0/37**，且原因不只是移动端真机未接入。
- 裁决 33、发布与 CI 记录本次未发现问题。PA09：Batch 24 的 macOS 对照显示条件①（IPC 往返 p50 2ms 高于 Worker 0ms）与②（单座顺序规划线程池慢于直调）未满足，按裁决维持 WASM 默认，需用户看数据再定。
- PA08／PA11／NC12 真机、安装包签名公证、第六阶段仍未完成。
