# ADR 0608：gwent 杜达元帅1/2 运行时 + ABI 643「取消使用牌」原语

> 承接 [ADR 0607](0607-huanle-shoulijian-abi642-gwent.md)「后续 1」：为 gwent 4 张特殊卡中
> 仍未具备运行时行为的 gw_dudayuanshuai1/2（杜达元帅）补全运行时，使 4 张特殊卡全部
> 具备运行时行为。核心交付为「取消使用牌」引擎原语（ABI 643，完整 8 文件 VM 驱动接线）。

## 背景

ADR 0607 完成 gwent 4 张特殊卡中的 2 张运行时（gw_wuyao→杀、gw_lang→酒 view_as），
gw_dudayuanshuai1/2 仅元数据登记，明确标记「需「取消使用牌」引擎原语，留后续」。本 ADR
补全该原语（ABI 643）并落地两卡运行时技能，另将上游 hupeng2/3/4 的机制归属写入注释。

## 上游语义（旧 JS 仅作语义证据，不进入运行时）

- **gw_dudayuanshuai1（小伙伴）**：`type: "special"`，`derivation: gw_zhuoertan`，
  `vanish`。hupeng4（`useCardToBefore` 触发，`event.player !== player`）：成为其他角色
  使用牌的目标时，可对它使用本牌；`content()` 中 `evt.cancel()` 取消该牌使用，
  `player.gain(evt.cards)` 获得对你使用的牌。
- **gw_dudayuanshuai2（捣蛋鬼）**：`type: "special"`。hupeng2 四个 mod（cardDiscardable/
  cardEnabled/cardUsable/cardRespondable 均 return false）禁使用/打出/弃置；hupeng3
  （`phaseEnd` 触发）：手牌含本牌则弃置本牌并随机弃置等量其他手牌。
- **gw_wuyao / gw_lang**：`autoViewAs: "sha"/"jiu"` + vanish（ADR 0607 已以 view_as 近似）。

## 做了什么

### ABI 643：`CancelIncomingCardUse` 原语（取消使用牌）

**效果**：成为其他角色使用牌的目标时，把持有者从当前牌结算目标中排除（取消该牌对持有者
生效），并将对你使用的实体牌移动到持有者手牌（获得对你使用的牌）。

**接线**（完整 8 文件 VM 驱动）：
| 文件 | 内容 |
|---|---|
| rule-api/effect.rs | `CancelIncomingCardUse { player, source, card, skill }` 变体 |
| rule-vm/bytecode.rs | `effect_tag::CANCEL_INCOMING_CARD_USE: u16 = 641` |
| rule-vm/vm.rs | pop 臂（player/source/card/skill）|
| engine-core/hash.rs | ABI 643 tag 编码 |
| engine-core/primitives/mod.rs | 调度臂 → `cancel_incoming_card_use` |
| engine-core/primitives/pack_sp_2.rs | 原语实现（见下）|
| content-schema/ir.rs | `StmtIr::EffectCancelIncomingCardUse { player, source, card }` |
| content-compiler/typecheck.rs | `"cancel_incoming_card_use(player, source, card)"` 臂 |
| content-compiler/codegen.rs | lowering → `Op::HostEffect(CANCEL_INCOMING_CARD_USE)` + ABI 643 |
| content-compiler/lib.rs | `ADDITIVE_RULE_VM_ABI_VERSION_643` + 判断链 |

**原语实现**（pack_sp_2.rs）：触发上下文为 `trigger card_targeted`——牌使用结算中、逐目标
触发。此刻杀牌已挂起 `ResumePlayedSha/ResumePlayedShaQueue`（`run_effects_with_saved_resume`
先设 resume 再跑 UseCard 效果），锦囊已挂起 `WuxieWindow` + resume_payload，两者均可经
`exclude_saved_card_target` 把持有者从当前牌结算目标中排除（杀走 `exclude_saved_sha_target`、
锦囊走 resume_payload 的 `remove_saved_target` 等分支）；排除成功后用
`gain_event_cards` 把 `card_source_cards(source, card)` 移动到手牌。不可排除的牌型
（如无中生有类全目标锦囊，`exclude_saved_card_target` 返回 Illegal）整效果视为未发动
（graceful no-op）。

**ABI**：`ABI-MAX: 643` + `RULE_VM_ABI_VERSION: 643`，`abort_bump_without_claim` 测试过。

### gwent 特殊卡 DSL（shahai_impl.skill）

```
// 杜达元帅1：成为其他角色使用牌的目标时，可以取消之并获得对你使用的牌。
skill gw_dudayuanshuai1_skill(3583584906) {
  trigger card_targeted owner_alive_targeted_by_other optional {
    effect cancel_incoming_card_use(self, event_source, event_card);
  }
}

// 杜达元帅2：结束阶段弃置两张手牌或装备牌（近似）。
skill gw_dudayuanshuai2_skill(3832623127) {
  trigger phase_end owner_is_target {
    effect discard_hand_equip_count(self, 2);
  }
}
```

**近似说明**（均写入 DSL 注释）：
- gw_dudayuanshuai1 上游为 useCardToBefore 取消整个 useCard 事件 + 获得牌；DSL 以
  card_targeted 逐目标取消（该牌对持有者不再生效）+ 获得实体牌近似——对其他目标的
  结算不受影响（更精细，属引擎语义内合理实现）。
- gw_dudayuanshuai2 的 per-card 禁使用/打出/弃置（hupeng2 四 mod）依赖按卡名/def 的
  card 门禁，引擎 modifier 选择器仅支持类别/花色（无自定义卡 def 选择器），该面省略；
  「结束阶段弃此牌并随机弃一张其他手牌」以弃置两张手牌或装备牌近似（含此牌本身 +
  随机一张其他手牌）。
- vanish（回合末消失）仍无引擎支持（ADR 0607 已登记）。

## 验证

| 检查 | 结果 |
|---|---|
| conformance | **4 测试全过**（+gwent_dudayuanshuai_abi643：编译 shahai_impl + 断言 rule_vm_abi_version ≥ 643 + 断言 gw_dudayuanshuai1/2 skill_def_id 在 skill_index）✅ |
| `abort_bump_without_claim` | **过**（ABI-MAX 643 = RULE_VM_ABI_VERSION 643）✅ |
| 失败集合 | **与基线一致**（零回归）✅ |
| `cargo check` | 无编译错误 ✅ |

**ABI 643 全链**：DSL `effect cancel_incoming_card_use(self, event_source, event_card)` →
codegen（player/source/card 三参 + HostEffect）→ VM pop → `EffectRequest::CancelIncomingCardUse`
→ hash（643 tag）→ `cancel_incoming_card_use` 原语（exclude_saved_card_target + gain_event_cards）。
端到端可编译可执行。

## 后续

1. **hupeng（呼朋）主技能**（gw_zhuoertan：弃一张牌并置入杜达元帅）与「杜达元帅」实体卡
   入牌流程：需实体自定义卡创建 + 置入手牌原语（createCard 等效），独立 ABI 644+ 工作。
2. **gw_dudayuanshuai2 per-card 禁弃置**：需按自定义卡 def 的 card 门禁选择器，独立工作。
3. **vanish 机制**（回合末消失）：需卡牌 vanish 引擎支持（ADR 0607 已登记，未变）。
4. **国战 12 锦囊结算**：仍为 ABI 644+ 待办（复用本模式）。
