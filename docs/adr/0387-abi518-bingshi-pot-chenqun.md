# ADR 0387: ABI 518 兵势陈群（法恩/定品）

- 状态：已完成
- 日期：2026-08-02
- ABI：518

旧 JavaScript 仅作为语义证据。`resolve_potchenqun` 在法恩技能实例中保存上一名全局用牌者及待执行的反向项：上一张牌由拥有者使用时，可令当前使用者摸一张或弃一张；下一张牌使用时先强制执行另一项并清除状态，同一事件仍可按上一名用牌者继续发动法恩。

定品复用权威回合失牌计数，结束阶段只允许选择并列失牌最多且失牌数大于零的存活角色。引擎新增与既有额外出牌阶段对称的额外摸牌阶段队列，完整进入 Draw 阶段、执行阶段触发与摸牌修饰，结束后按原回合锚点继续正常轮转。

门禁：势陈群 Golden 3/3、兵势全量 Golden、content-compiler 466/466、Rule VM、content-runtime、engine-api ABI 协调、26 个扩展包导入与 conformance 8/8 全部通过；`identity_4_42` 因兵势选将池新增陈群按规则重钉。全量 engine-core tests 另被 `original_shenhua_guzheng.rs` 调用私有 helper 的既有编译错误阻断，与本 ADR 无关。
