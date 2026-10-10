# ADR 0454 — shenhua 严颜【拒战】(ABI 571)

- 状态:已接受
- 日期:2026-08-03
- 归属:C (omp)
- 关联 ADR:0453(诸葛瞻)、0446(刘禅)
- ABI:571

## 目标

发布 shenhua 严颜完整武将【拒战】(转换技):

- 阴:你成为其他角色【杀】的目标后,与其各摸一张牌,然后其本回合内不能再对你
  使用牌。
- 阳:你使用【杀】指定一名角色为目标后,获得其一张牌,然后你本回合内不能再对其
  使用牌。

## 决策

### 回合用牌禁用(新引擎能力)

- 回合态 `turn.use_ban_pairs: Vec<(PlayerId, PlayerId)>`(被禁使用者 → 被禁目标),
  回合边界随 `clear_per_player_card_counts` 清空;非空时进入 canonical hash。
- 新修正器 `card_target_cancel_owner_turn_banned(any)`:当 (当前使用者, 持有者)
  在禁用对中时取消该牌对持有者的指定(在 `card_target_cancelled_for_logical_kind`
  与公开桃路径 `card_target_cancelled_for_kind` 中均求值)。
- DSL 效果 `ban_use_against(user, target)`:登记禁用对(双方存活校验、去重)。

### 拒战复合 (ABI 571)

- 双触发 `card_targeted`(受杀,阴)+ `card_used`(用杀,阳),共用
  `resolve_juzhan(player, source, card, targets, stage)`。
- 转换技分支:技能状态 `JZ1<branch>`(4 字节)每次发动翻转 0/1。
- 阴:双方各摸一张 + `ban_use_against(source, player)`。
- 阳:目标取声明目标中非持有者者(vm 经 `event_targets` 注入,修正触发上下文
  `selected_targets` 为空的缺口);开「获得目标一张手牌/装备牌」选择,结算后
  `ban_use_against(player, target)`;目标无牌可获时仅登记禁用。

## 接线

`BanUseAgainst`(操作码 547)/`ResolveJuzhan`(548)→ vm 弹栈 → vm_host 注入 →
primitives 复合 → 续体标签 `0xDCDC_5710` → hash 臂 524/525 → DSL 词
`ban_use_against(user, target)`/`resolve_juzhan(player, source, card)` → 修正器词
`card_target_cancel_owner_turn_banned`(枚举尾 + registry 双向词 + typecheck 例外)→
IR/codegen/门控 `ADDITIVE_RULE_VM_ABI_VERSION_571`。

## 测试与发布

- Golden `original_shenhua_yanyan.rs` 4 项:契约(ABI 571/双触发/一修正器)、阴分支
  双摸牌+禁用来源、阳分支获牌+禁用己方、修正器取消被禁使用者对持有者的牌。全绿。
- 50 个 shenhua 测试文件零失败;engine-core 全量 0 失败;conformance 8/8 ALL PASS。
- npack:ABI 571、技能 130、角色 39(严颜 id 1871209985,蜀 4 体力 男)、触发 92、
  active 14、修正器 46、模块 106;`extensions/shenhua/{shenhua,package}.npack.json`
  同步。

## 近似与遗留

- 「本回合内不能再使用牌」以回合禁用对表达,回合边界清除(与旧 JS 临时技能至
  回合结束一致);判定牌/濒死救援等不受禁。
- 阳分支目标取首个非持有者声明目标(旧 JS 单目标杀);多目标杀仅首个目标受益。
- 阴分支旧 JS 的触发者受禁语义精确复刻。
