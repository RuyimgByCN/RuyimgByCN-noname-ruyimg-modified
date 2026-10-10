# ADR 0438 — shenhua 鲁肃【好施】【缔盟】(ABI 555-556)

- 状态:已接受
- 日期:2026-08-03
- 归属:C (omp)
- 关联 ADR:0435(于吉)、0431(张角)、0425(毌丘俭)
- ABI:555(好施)、556(缔盟)

## 目标

发布 shenhua 鲁肃完整武将:

- **好施**:摸牌阶段,你可以额外摸两张牌;摸牌阶段结束时,若你的手牌数大于 5,
  则将一半手牌(向下取整)交给手牌数最少的一名其他角色。
- **缔盟**:出牌阶段限一次,你可以选择两名其他角色并弃置 X 张牌(X 为这两名角色
  手牌数之差),然后交换其手牌。

## 决策

### 好施 (ABI 555)

- DSL 双触发:`phase_draw`(可选,摸牌替换 stage 0)+ `phase_draw_end`(强制,
  give-half stage 1),以 `resolve_haoshi(player, stage)` 显式分阶段。
- stage 0:复用 `set_draw_phase_fixed_count(当前 + 2)`(额外摸两张)。
- stage 1:手牌 > 5 时,选手牌数最少的一名其他角色(运行时计算,`min_by_key`),
  开 `floor(手牌/2)` 张选牌续体,移入该目标手牌;目标与数量在 Choice 恢复时
  重校验。

### 缔盟 (ABI 556)

- DSL `active play_phase uses_per_turn 1` + `resolve_dimeng(player)`(vm 经
  `selected_targets` 注入两目标)。
- 续体:选两名其他角色 → 计算手牌差 X → 若 X==0 直接交换;否则开 X 张
  手牌/装备弃置选择(数量与 X 相等)→ 弃置后整手交换(左右手牌互相 MoveCards)。
- 势鲁肃兵势变体(potdimeng)的 ≤3 差约束不适用,故不复用其引擎实现。

## 接线

`ResolveHaoshi`/`ResolveDimeng` 变体 → 操作码 509/510 → vm 弹栈(dimeng 注入
selected_targets)→ vm_host 注入技能实例 → primitives 入口 → 自定义续体
`0xDCDC_5550/5560` + resolve_choice.rs 解析器 → hash 臂 514/515 → DSL 词
`resolve_haoshi(player, stage)`/`resolve_dimeng(player)` → IR/codegen/门控链
`ADDITIVE_RULE_VM_ABI_VERSION_555/556`。

## 测试与发布

- Golden `original_shenhua_lusu.rs` 3 项:契约(ABI 556/双触发/active)、好施摸牌
  替换为 4 + 手牌>5 时给 floor/2 给手牌最少者、缔盟手牌差弃置 + 整手交换。全绿。
- 47 个 shenhua 二进制零失败;engine-core 全量 0 失败;conformance 8/8 ALL PASS。
- npack:ABI 556、技能 123、角色 35(鲁肃 id 56005897,吴 3 体力 男)、触发 80、
  active 14、模块 94;`extensions/shenhua/{shenhua,package}.npack.json` 三副本一致。

## 近似与遗留

- 好施「手牌数最少的一名角色」在触发时计算并锁定(旧 JS 于选择时过滤,等价)。
- 缔盟弃置范围手牌/装备(旧 JS `he`),交换前验证目标存活。
