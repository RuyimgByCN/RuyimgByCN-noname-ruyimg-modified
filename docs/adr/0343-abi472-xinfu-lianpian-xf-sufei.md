# ADR 0343：ABI 472 连篇共同目标摸牌授牌复合 + sp2 苏飞

本批为 sp2 包苏飞（`xf_sufei`，3980701921）的「连篇」（`xinfu_lianpian`，
3795462236）完整移植，需要 ABI 472 新过滤器与复合。

## 原语设计（ABI 472）

- 新回合位掩码 `LAST_CARD_TARGETS`（turn_flag 38）：`remember_card_action` 在
  CardUsed 时按 PlayerId-1 位记录上一张牌的目标（与 LAST_CARD_KIND/DEF 同能力门）。
- 过滤器 `owner_source_play_card_repeats_previous_targets`：拥有者为来源、处于自己
  出牌阶段、`turn_fires < 3`（每回合至多三次）、当前牌目标与上一张牌目标存在交集。
- 复合 `resolve_xinfu_lianpian(player)`（`EffectRequest::ResolveXinfuLianpian`，
  当前牌目标由 vm_host 捕获）：过滤共同目标（排除自己）→ 摸一张 → Custom 续体
  （`RESOLVE_XINFU_LIANPIAN_CUSTOM_CHOICE`）选择共同目标交付或取消保留。

## 文档化近似

旧 JS 按每个目标（useCardToPlayered）触发且含 triggeredTargets3 防重入守卫；DSL
按整张牌使用触发一次。

## 打包与验证

- `xinfu_lianpian.skill` 加入 sp2 README 显式列表；`xf_sufei`（男/吴/4 体力，技能
  [xinfu_lianpian]）加入 sp2 `characters.json`；catalog 标记 `ported`。
- sp2 npack 重建（ABI 472，42 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/sp2/`。
- 运行时测试 `original_sp2_xinfu_lianpian.rs`（3 用例全绿，真实 PlayCard 管线）：
  杀→A 后南蛮（A+B 共同目标）摸一赠 A；杀→A 后决斗→B 无共同目标不触发；取消
  保留所摸之牌。
- 旧 JavaScript 不进入 npack 或运行时。
