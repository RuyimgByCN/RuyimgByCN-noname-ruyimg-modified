# ADR 0344：ABI 473 拱坚共同目标弃牌夺杀复合 + sp2 朱儁

本批为 sp2 包朱儁（`zhujun`，2614075935）的「拱坚」（`gongjian`，3309202552）
完整移植，需要 ABI 473 新过滤器与复合。

## 原语设计（ABI 473）

- 新回合位掩码 `LAST_SHA_TARGETS`（turn_flag 39）：`remember_card_action` 在
  使用【杀】时记录其目标（与 LAST_CARD_TARGETS 同门）。
- 过滤器 `owner_source_sha_repeats_previous_sha_targets`：拥有者为来源、使用
  【杀】、`turn_fires == 0`（每回合一次）、当前【杀】目标与上一张【杀】目标相交。
- 复合 `resolve_gongjian(player)`（`EffectRequest::ResolveGongjian`，当前【杀】目标
  由 vm_host 捕获）：筛选共同目标（存活且有可弃 he）→ 逐目标打开 SelectCards
  （1..2 张，不可取消，`discardable_target_cards` 校验）→ 弃置并累计 → 全部完成后
  将累计中被弃置的【杀】移回拥有者手牌（Custom 续体 `RESOLVE_GONGJIAN_CUSTOM_CHOICE`）。

## 文档化近似

旧 JS 按每个目标（useCardToPlayered + isFirstTarget）触发；DSL 按整张【杀】使用
触发一次。

## 打包与验证

- `gongjian.skill` 加入 sp2 README 显式列表；`zhujun`（男/群/4 体力，技能
  [gongjian]）加入 sp2 `characters.json`；catalog 标记 `ported`。
- sp2 npack 重建（ABI 473，43 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/sp2/`。
- 运行时测试 `original_sp2_gongjian.rs`（3 用例全绿，真实 PlayCard 管线）：杀→A
  后再杀→A 弃 A 之【杀】并夺回；杀→A 后杀→B 无共同目标不触发；弃两张只夺回
  【杀】（桃留在弃牌堆）。
- 旧 JavaScript 不进入 npack 或运行时。
