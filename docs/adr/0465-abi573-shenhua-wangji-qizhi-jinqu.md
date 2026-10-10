# ADR 0465 — shenhua 王基【奇制/进趋】(ABI 573)

- 状态:已接受
- 日期:2026-08-03
- 归属:C (omp)
- 关联 ADR:0454(严颜)、0453(诸葛瞻)
- ABI:573

## 目标

发布 shenhua 王基完整武将:

- **奇制**:你使用牌指定目标后,你可以弃置一名非目标角色的一张手牌/装备牌,然后
  令其摸一张牌;本回合奇制次数 +1。
- **进趋**:出牌阶段结束时,你可以摸两张牌,然后将手牌弃置至本回合奇制次数张。

## 决策

### 奇制 (ABI 573)

- `trigger card_used owner_is_source`(可选)→ `resolve_qizhi(player)`;vm 经
  `event_targets` 注入本牌目标,目标选择排除持有者与牌目标。
- 选目标 → 选其一张手牌/装备牌(续体 `0xDCDC_5730/5731`)→ 弃牌 + 目标摸一张 →
  计数 +1。
- 回合计数:技能状态 `QZ1<round u32><count u8>`(round 不匹配视为 0),与旧 JS
  `getHistory("custom")` 的每回合奇制次数一致。

### 进趋 (ABI 573)

- `trigger phase_end owner_is_target`(可选)→ `resolve_jinqu(player, qizhi_def)`;
  显式携带奇制 def(跨技能引用模式,同 `resolve_zaoxian` 的 tuntian_def)。
- 摸两张 → 手牌数减奇制次数为应弃数(可为 0)→ 续体 `0xDCDC_5732` 弃置校验。
- 弃置数在恢复时重算(手牌可能已变化),不一致拒绝。

## 接线

`ResolveQizhi`(操作码 551)/`ResolveJinqu`(552)→ vm 弹栈(qizhi 经 event_targets、
jinqu 携带 qizhi_def)→ vm_host 注入 → primitives 复合(计数助手
`qizhi_count`/`qizhi_increment`)→ 续体解析器 → hash 臂 526/527 → DSL 词
`resolve_qizhi(player)`/`resolve_jinqu(player, qizhi_def)` → IR/codegen/门控
`ADDITIVE_RULE_VM_ABI_VERSION_573`。

## 测试与发布

- Golden `original_shenhua_wangji.rs` 4 项:契约(ABI 573/双触发)、奇制弃非目标牌
  并令其摸一张且计数 +1、进趋摸二弃至 0、进趋计数 1 弃三留一。全绿。
- 51 个 shenhua 测试文件零失败;engine-core 全量 0 失败;conformance 8/8 ALL PASS。
- npack:ABI 573、技能 132、角色 40(王基 id 1051379043,魏 3 体力 男)、触发 94、
  active 14、修正器 46、模块 108;`extensions/shenhua/{shenhua,package}.npack.json`
  同步。

## 近似与遗留

- 旧 JS 奇制触发于「首目标指定」(isFirstTarget);引擎 `card_used` 每牌触发一次,
  等价。
- 进趋旧 JS 弃牌可选取消;本实现选择后强校验弃置数(与旧 JS 结算等价)。
- 操作码编号受并发编辑多次冲突后定稿 551/552(当前表尾 552)。
