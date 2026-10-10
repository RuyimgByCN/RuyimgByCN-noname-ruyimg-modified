# ADR 0425 — shenhua 毌丘俭【征荣】【鸿举】【清俭】(ABI 545-547)

- 状态:已接受
- 日期:2026-08-02
- 归属:C (omp)
- 关联 ADR:0421(孟获)、0417(姜维)、0412(邓艾)
- ABI:545(征荣)、546(鸿举)、547(清俭)

## 目标

发布 shenhua 毌丘俭完整武将:

- **征荣**:当你使用【杀】或普通锦囊牌指定第一个目标后,若目标中有手牌数不小于你的
  角色,你可以将其中一名目标角色的一张牌置于你的武将牌上,称为「荣」。
- **鸿举**:觉醒技,准备阶段,若你的「荣」数量不小于 3,你可以将任意张手牌与等量
  「荣」交换,然后减 1 点体力上限,获得「清俭」。
- **清俭**:出牌阶段限一次,你可以弃置一张手牌,获得一张「荣」,然后弃置一名其他
  角色装备区或判定区内的一张牌。

## 决策

### 征荣 (ABI 545)

- 过滤器 `owner_source_first_damage_card_target_hand_gte_owner`:首个目标
  (`ctx.targets.first() == ctx.player`)、伤害标签牌(`is_sha || is_damage_trick`)、
  存在目标(手牌数 ≥ 持有者且有可弃手/装备)。
- 复合原语 `resolve_zhengrong(player, candidates, target, stage)`——vm 经
  `host.event_targets()` 注入事件目标(本次同步修正了 EngineVmHost 未实现 trait
  `event_targets` 的缺口,此前恒为空);续体:选目标 → 选其一张手/装备 → 移入
  「荣」技能牌堆。

### 鸿举 (ABI 546)

- 复用 `owner_phase_begin_skill_pile_gte_three_<def>`(荣 = 征荣技能牌堆)。
- 复合原语 `resolve_hongju(player, qingjian_def, rong_def, stage)`:荣 ≥ 3 且未持
  清俭时,可选交换(选等量手牌 ↔ 荣,MoveCards 双向)或跳过,随后 `ChangeMaxHp(-1)`
  + `AddSkill(清俭)`;已持有清俭幂等(觉醒一次性)。

### 清俭 (ABI 547)

- `active play_phase uses_per_turn 1` 承载「出牌阶段限一次」。
- 复合原语 `resolve_qingjian(player, rong_def, stage)` 四段:选「荣」→ 选手牌弃置
  → 选目标(有装备/判定区可弃牌)→ 选其装备/判定区牌弃置;同时「荣」移入持有者
  手牌。装备/判定区范围用既有 `TargetCardScope::EquipJudge`。

## 接线

`ResolveZhengrong`/`ResolveHongju`/`ResolveQingjian` 变体 → 操作码 494/495/496 →
vm 弹栈(征荣注入 event_targets)→ vm_host 注入技能实例 → primitives 三原语入口 →
自定义续体标签 `0xDCDC_5450/5460/5470` + resolve_choice.rs 各段解析器 →
hash 臂 509/510/511 → DSL 词 `resolve_zhengrong(player)`/`resolve_hongju(player,
qingjian_def, rong_def)`/`resolve_qingjian(player, rong_def)` → IR/codegen/门控链
`ADDITIVE_RULE_VM_ABI_VERSION_545/546/547`;过滤器四处登记。

## 测试与发布

- Golden `original_shenhua_guanqiujian.rs` 5 项:契约(ABI 547/触发/过滤/active)、
  征荣入荣(目标筛选+取牌)、鸿举三荣觉醒(跳过交换直接觉醒,上限 5→4+获清俭+荣留
  牌堆)、清俭四段全流程(荣入手/手牌弃置/武器弃置)、征荣非伤害牌不触发。全绿。
- 44 个 shenhua 二进制零失败;engine-core 全量 0 失败(唯 bingshi pot_huanjie 为
  Codex 20:23 重建包缺 qice/kanpo 定义所致,非本批次回归);conformance 8/8
  ALL PASS。
- npack:ABI 547、技能 116、角色 32(毌丘俭 id 500580390,魏 4 体力 男)、触发 76、
  模块 89、active 13;`extensions/shenhua/{shenhua,package}.npack.json` 三副本一致。

## 近似与遗留

- 征荣「伤害标签」以 `is_sha || is_damage_trick` 近似旧 JS `get.tag(card,
  "damage")`(闪电等延时锦囊不在目标指定路径,等价)。
- 鸿举交换以「选等量手牌」近似旧 JS 的 chooseToMove 自由分配,结算一致。
- 清俭目标范围为装备/判定区(旧 JS `ej`),不含手牌。
