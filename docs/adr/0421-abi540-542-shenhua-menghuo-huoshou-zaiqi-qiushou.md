# ADR 0421 — shenhua 孟获【祸首】【再起】【求首】(ABI 540-542)

- 状态:已接受
- 日期:2026-08-02
- 归属:C (omp)
- 关联 ADR:0417(姜维)、0412(邓艾)、0401(祝融)
- ABI:540(再起)、541(祸首来源重定向)、542(求首)

## 目标

发布 shenhua 孟获完整武将:

- **祸首**(锁定):南蛮入侵对你无效;其他角色使用南蛮入侵造成伤害时,伤害来源视为你。
- **再起**:摸牌阶段,若你已受伤,你可以放弃摸牌,改为依次亮出牌堆顶 X 张牌
  (X = 已损失体力 + 1),每亮出一张红桃回复 1 点体力,然后获得其余亮出的牌。
- **求首**(主公技,锁定):其他角色使用的南蛮入侵结算完毕后,若目标中有角色死亡
  或造成伤害的目标数超过 3,所有存活蜀/群角色各摸一张牌。

## 决策

### 再起 (ABI 540)

- DSL `trigger phase_draw owner_is_target_draw_not_replaced optional`(复用庸肆摸牌
  替换过滤器),复合原语 `resolve_zaiqi(player, extra)`:以
  `set_draw_phase_fixed_count(0)` 弃摸牌,翻亮牌堆顶 X 张(移入 Processing +
  CardsRevealed),红桃移入弃牌堆并按张数 `RecoverHp`,其余移入持有者手牌。
- 旧 JS `zaiqixx` 的 X+1 以 `extra=1` 表达。

### 祸首 (ABI 541)

- 南蛮目标取消复用通用 `card_target_cancel(nanman) override 1`。
- 来源重定向:新修正器 `nanman_damage_source_as_owner` + `prepare_damage` 钩子——
  仅当伤害牌为南蛮实体牌(`effective_card_kind == TrickNanman`)时,将伤害来源改写
  为修正器持有者(存活且非原来源);其他伤害路径零影响。

### 求首 (ABI 542)

- 过滤器 `owner_alive_nanman_resolved_target_dead_or_damaged_gt_three`:
  `card_resolved` + 南蛮(kind 24)+ 目标死亡或受创目标数 > 3;受创目标以
  dispatch 的位掩码(`ctx.amount`)解码(旧 JS 按伤害总量计数,以目标数近似,文档化)。
- 复合原语 `resolve_twqiushou(player)`:所有存活 `player_group ∈ {蜀, 群}` 角色
  各摸一张;主公技以 `lord_only` 表达。

## 接线

`ResolveZaiqi`/`ResolveTwqiushou` 变体 → 操作码 490/491(表尾追加)→ vm 弹栈 →
vm_host 注入 → primitives 实现 → hash 臂 507/508 → DSL 词
`resolve_zaiqi(player, extra)`/`resolve_twqiushou(player)` → IR/codegen/门控链
`ADDITIVE_RULE_VM_ABI_VERSION_540/542`;修正器目标 `NanmanDamageSourceAsOwner`
在 modifier.rs/registry/typecheck 三处登记;过滤器四处登记。

## 测试与发布

- Golden `original_shenhua_menghuo.rs` 5 项:契约(ABI 542/lord_only/修正器暴露)、
  祸首南蛮伤害来源重定向、再起摸牌替换(红桃回复+其余获得+弃摸牌)、求首死亡分支
  (蜀/群各摸一、非蜀/群不摸)、求首单受创不触发。全绿。
- 43 个 shenhua 二进制零失败;engine-core 全量 0 失败;conformance 8/8 ALL PASS。
- npack:ABI 542、技能 113、角色 31(孟获 id 1715864006,蜀 4 体力 男)、触发 74、
  修正器 43、模块 86;`extensions/shenhua/{shenhua,package}.npack.json` 三副本一致。

## 近似与遗留

- 求首伤害阈值以「受创目标数」近似旧 JS 的伤害总量(多段伤害目标计数偏差)。
- 再起翻亮为一次性批量揭示(旧 JS 逐张展示),结算结果一致。
- 祸首来源重定向影响击杀归属等来源相关结算,与旧 JS customSource 语义一致。
