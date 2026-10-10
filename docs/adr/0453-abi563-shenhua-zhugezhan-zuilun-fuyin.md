# ADR 0453 — shenhua 诸葛瞻【罪论/父荫】(ABI 563)

- 状态:已接受
- 日期:2026-08-03
- 归属:C (omp)
- 关联 ADR:0446(刘禅)、0445(张郃)
- ABI:563

## 目标

发布 shenhua 诸葛瞻完整武将:

- **罪论**:结束阶段,你揭示牌堆顶三张牌,然后按「本回合失去过牌 / 手牌数非最少 /
  本回合未受到过伤害」计数 0-3;0 获全部三张,1-2 将对应张数置于牌堆顶并获其余,
  3 全部置顶并选择一名其他角色与你一起失去 1 点体力。
- **父荫**(锁定技):其他角色以不小于你手牌数的【杀】指定你为目标,且本回合首次
  时,此【杀】对你无效。

## 决策

### 罪论 (ABI 563)

- 结束阶段可选触发 `resolve_zuilun(player)`:揭示牌堆顶三张(移入处理区,复用
  `move_one_card`);按计数分派:
  - 0:三张全部移入持有者手牌。
  - 3:三张移回牌堆顶,开「一名其他角色」选择,随后双方 `DealDamage{is_lose_hp}`
    各失去 1 点体力。
  - 1-2:开 SelectCards(min=max=计数)选择置顶牌,置顶其余获得。
- 「失去过牌」以 `turn.lost_cards` 或弃牌阶段旗标判定(弃牌阶段旗标恒可靠;
  `lost_cards` 依赖内容的回合失牌历史特性,测试经 ABI 405 过滤器族启用)。

### 父荫 (ABI 563)

- `trigger card_targeted`(强制)→ `resolve_fuyin(player, source, card)`:校验事件卡为
  【杀】、来源手牌数不小于持有者、本回合(轮次)未触发过(技能 `FY1` 轮次标记),
  经 ABI 560 `exclude_saved_sha_target` 将该杀保存目标移除。
- 旧 JS 亦覆盖【决斗】;引擎无决斗目标排除管线,此半幅文档化为近似。

## 接线

`ResolveZuilun`(操作码 537)/`ResolveFuyin`(538)两个变体 → vm 弹栈 → vm_host 注入
→ primitives 复合 → 续体标签 `0xDCDC_5630/5631` → hash 臂 522/523 → DSL 词
`resolve_zuilun(player)`/`resolve_fuyin(player, source, card)` → IR/codegen/门控
`ADDITIVE_RULE_VM_ABI_VERSION_563`。

操作码编号冲突修复:Codex 并行批次的 RESOLVE_RECHUHAI/XIONGSHU 曾占用 529/530(与
既有 `CHOOSE_OTHER_HAND_STORE_SKILL_PILE=529`、`GAIN_ALL_SKILL_PILE_CARDS=530`
冲突,导致旧效果分发到错误臂),本批顺带将其移至表尾 545/546;自身标签多次
碰撞后定稿 537/538(现表尾 546)。`SkillPileCount` 查询请求/响应变体被并发编辑
误移入响应枚举,已恢复请求变体并补缓存键臂。

## 测试与发布

- Golden `original_shenhua_zhugezhan.rs` 5 项:契约(ABI 563/双触发)、罪论计数 0
  全获、计数 2 置顶两张获一张、计数 3 置顶并一起失体力、父荫首次排除第二次不
  排除。全绿。
- 49 个 shenhua 测试文件零失败;engine-core 全量 0 失败;conformance 8/8 ALL PASS。
- npack:ABI 563、技能 129、角色 38(诸葛瞻 id 3998421601,蜀 3 体力 男)、触发 90、
  active 14、模块 104;`extensions/shenhua/{shenhua,package}.npack.json` 同步。

## 近似与遗留

- 父荫的【决斗】无效半幅未建模(引擎无决斗目标排除);「本回合首次」以轮次标记
  表达(回合内跨阶段一致)。
- 罪论「失去过牌」的判定依赖回合失牌历史特性;弃牌阶段旗标为恒可靠后备。
- 罪论置顶顺序与旧 JS `reverse` 不同(依次置顶,后放者最上),文档化近似。
