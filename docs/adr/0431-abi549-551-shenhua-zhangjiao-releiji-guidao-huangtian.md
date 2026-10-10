# ADR 0431 — shenhua 张角【雷击】【鬼道】【黄天】(ABI 549-551)

- 状态:已接受
- 日期:2026-08-03
- 归属:C (omp)
- 关联 ADR:0425(毌丘俭)、0421(孟获)、0417(姜维)
- ABI:549(雷击)、550(鬼道)、551(黄天)

## 目标

发布 shenhua 张角完整武将:

- **雷击**:当你使用或打出【闪】时,你可以令一名其他角色判定;黑桃 → 对其 2 点雷电
  伤害;梅花 → 你回复 1 点体力并对其 1 点雷电伤害。
- **鬼道**:当一名角色的判定牌生效前,你可以打出一张黑色手牌或装备牌替换之。
- **黄天**(主公技):其他群势力角色的出牌阶段限一次,其可以将一张【闪】或【闪电】
  交给你。

## 决策

### 雷击 (ABI 549)

- 过滤器 `owner_responded_shan`:`card_responded` + 持有者为响应者 + `card_kind ==
  Shan`。
- 复合原语 `resolve_releiji(player, target, stage)`:选目标 → 翻亮牌堆顶判定牌(留在
  Processing)→ 按座次(自目标起)征询判定改写者(见鬼道)→ 结算分支(黑桃 2 雷伤 /
  梅花回 1 + 1 雷伤)。判定分支以旧 JS 梅花/黑桃语义为准,红桃/方块无效果。

### 鬼道 (ABI 550)

- 新修正器 `judgment_rewrite_black_hand_equip`:黑色手/装备牌进入判定改写候选。
  `can_rewrite_judgment` 与 `judgment_rewrite_card_matches` 增加黑色通道;改写候选
  复用既有 `judgment_rewrite_cards`(含响应取消/用牌取消过滤)。
- 雷击的自定义判定流程直接征询改判者(座次序,Choice 恢复时重校验),不经过延时
  锦囊队列;旧 JS 对任意判定可改判,延时锦囊路径沿用既有判定改写机制。

### 黄天 (ABI 551)

- 过滤器 `owner_alive_other_qun_play_phase_can_give_owner`(其他群势力角色出牌阶段
  且持有闪/闪电),加入 `once_per_round` 守卫(旧 JS usable 1,每回合一次);
  `by event_player` 将选择权交给群势力角色(制霸模式)。
- 复合原语 `resolve_huangtian(player, actor, stage)`:群势力角色选一张闪/闪电
  (可取消),移入持有者手牌。

## 接线

`ResolveReleiji`/`ResolveHuangtian` 变体 → 操作码 503/504 → vm 弹栈 → vm_host
注入 → primitives 入口 → 自定义续体 `0xDCDC_5490/5510` + resolve_choice.rs
解析器(雷击改判子流程、黄天赠送)→ hash 臂 512/513 → DSL 词
`resolve_releiji(player)`/`resolve_huangtian(player, actor)` → IR/codegen/门控链
`ADDITIVE_RULE_VM_ABI_VERSION_549/551`;过滤器与修正器四处登记。

## 测试与发布

- Golden `original_shenhua_zhangjiao.rs` 6 项:契约(ABI 551/lord_only/修正器暴露)、
  雷击黑桃 2 雷伤、雷击梅花回 1 + 1 雷伤、鬼道黑牌改判后按黑桃分支结算、黄天赠送
  闪给主公、黄天无闪/闪电不触发。全绿。
- 45 个 shenhua 二进制零失败;engine-core 全量 0 失败;conformance 8/8 ALL PASS。
- npack:ABI 551、技能 119、角色 33(张角 id 114402602,群 3 体力 男)、触发 78、
  修正器 44、模块 91;`extensions/shenhua/{shenhua,package}.npack.json` 三副本一致。

## 近似与遗留

- 雷击判定改判征询为自定义流程,延时锦囊判定仍走既有队列改写;直接判定的其他
  技能(八卦等)不征询改判者(旧 JS 亦允许,文档化近似)。
- 黄天每回合一次以 once_per_round 近似旧 JS usable 1。
- 雷击判定牌在改判前后均置入弃牌堆,与旧 JS 一致。
