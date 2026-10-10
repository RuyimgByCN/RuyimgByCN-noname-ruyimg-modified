# ADR 0607：欢乐卡牌 shoulijian（手里剑）ABI 642 + gwent 卡牌机制移植

> 承接 [ADR 0605](0605-extension-migration-audit.md)：为欢乐卡牌 shoulijian 实现运行时结算
> （自定义基本牌），并移植杀海拾遗 gwent 4 张特殊卡牌机制。

## 背景

ADR 0605 审计确认 8 个扩展的 420/420 角色已移植，但欢乐卡牌 shoulijian（手里剑）与
gwent 4 张特殊卡仅完成元数据登记（翻译 + 空技能占位），无运行时行为。本 ADR 补全这两块。

## 做了什么

### ABI 642：`ResolveShoulijian` 原语（欢乐卡牌手里剑）

**效果**：出牌阶段对一名其他角色使用【手里剑】，其选择一项——弃置一张装备牌，或受到
你造成的 1 点伤害。

**接线**（完整 8 文件 VM 驱动）：
| 文件 | 内容 |
|---|---|
| rule-api/effect.rs | `ResolveShoulijian { player, target, skill }` 变体 |
| rule-vm/bytecode.rs | `effect_tag::RESOLVE_SHOULIJIAN: u16 = 640` |
| rule-vm/vm.rs | pop 臂（player/target/skill）|
| engine-core/hash.rs | ABI 642 tag 编码 |
| engine-core/primitives/mod.rs | 调度臂 → `resolve_shoulijian` |
| content-schema/ir.rs | `StmtIr::EffectResolveShoulijian` |
| content-compiler/typecheck.rs | `"resolve_shoulijian(player, target)"` 臂 |
| content-compiler/codegen.rs | lowering → `Op::HostEffect(RESOLVE_SHOULIJIAN)` + ABI 642 |
| content-compiler/lib.rs | `ADDITIVE_RULE_VM_ABI_VERSION_642` + 判断链 |

**原语实现**（pack_sp_2.rs）：复用 `open_xiaoguo_target_equipment_or_damage`——目标有装备
则开「弃装备或受伤害」choice（Xiaoguo 续体），无装备则直接受 1 伤害。

**DSL**（huanle_meta.skill）：`skill shoulijian(2612562977) { active play_phase targets 1 1 other { effect resolve_shoulijian(self, event_source); } }`。

**ABI**：`ABI-MAX: 642` + `RULE_VM_ABI_VERSION: 642`，`abort_bump_without_claim` 测试过。

### gwent 4 张特殊卡

上游 4 张均为 `type: "special"` 特殊卡：
- **gw_wuyao（雾妖）**：`autoViewAs: "sha"` → DSL `gw_wuyao_skill(784509644) { view_as sha play_phase any from hand }`
- **gw_lang（狼）**：`autoViewAs: "jiu"` → DSL `gw_lang_skill(2836710318) { view_as jiu play_phase any from hand }`
- **gw_dudayuanshuai1/2（杜达元帅）**：`derivation: gw_zhuoertan` 派生特殊卡（cancel 事件 +
  禁弃置机制），机制复杂，保持元数据登记（translations.json 已登记）。

**近似说明**：上游为卡牌 autoViewAs + vanish（回合末消失）；引擎 view_as 是技能属性且无
卡牌 vanish 机制，故以技能级 view_as 近似（vanish 省略）。gw_dudayuanshuai 的 cancel 事件
需「取消他人使用牌」引擎原语，留后续。

## 验证

| 检查 | 结果 |
|---|---|
| conformance | **4 测试全过**（+huanle_shoulijian_abi642 + shahai_impl_repro）✅ |
| `abort_bump_without_claim` | **过**（ABI-MAX 642 = RULE_VM_ABI_VERSION 642）✅ |
| 失败集合 | **247 = 247 双向空**（身份局零回归）✅ |
| warnings | **59** ✅ |

**ABI 642 全链**：DSL `effect resolve_shoulijian(self, event_source)` → codegen → VM pop →
`EffectRequest::ResolveShoulijian` → hash（642 tag）→ `resolve_shoulijian` 原语（复用
Xiaoguo「弃装备或伤害」choice）。端到端可编译可执行。

## 后续

1. **gw_dudayuanshuai1/2**：派生特殊卡（cancel 他人使用牌 + 禁弃置）——需「取消使用牌」
   引擎原语，留后续。
2. **vanish 机制**：gw_wuyao/gw_lang 的「回合末消失」——需卡牌 vanish 引擎支持，留后续。
3. **国战 12 锦囊结算**（gz_haolingtianxia 等）：同需新 CardKind + choice 原语，与本次
   shoulijian 同模式，可复用 ABI 642 接线模式（ABI 643+）。
