# ADR 0466 — shenhua 蒯良蒯越【荐相/审时】(ABI 574)

- 状态:已接受
- 日期:2026-08-03
- 归属:C (omp)
- 关联 ADR:0465(王基)、0454(严颜)
- ABI:574

## 目标

发布 shenhua 蒯良蒯越完整武将:

- **荐相**:你成为其他角色使用牌的目标后,令场上手牌数最少的一名角色摸一张牌。
- **审时**(转换技):
  - 阳:出牌阶段限一次,将一张牌交给一名手牌数最多的角色,然后对其造成 1 点
    伤害;若该角色因此死亡,可令一名角色将手牌摸至四张。
  - 阴:其他角色对你造成伤害后,观看该角色的手牌并交给其一张牌;其回合结束时
    若此牌仍在其手牌区或装备区,你将手牌摸至四张。

## 决策

### 荐相 (ABI 574)

- `trigger card_targeted owner_is_target`(可选)→ `resolve_jianxiang(player)`:
  运行时取全场手牌数最少者(可含持有者自己,与旧 JS `isMinHandcard` 一致)开
  目标选择,选定者摸一张。

### 审时 (ABI 574)

- 单复合 `resolve_shenshi(player, source, stage)` 三触发共用,DSL 显式分阶段
  (0 阳 active / 1 阴受伤 / 4 回合末追踪检查),避免触发来源歧义。
- 转换技分支:技能状态 `SS1<branch>` 每次发动翻转。
- 阳:开「交一张手牌/装备牌」选择 → 开「手牌最多者」选择 → 赠牌 + `DealDamage`
  → 若目标死亡开「任意角色」选择,目标手牌摸至四张。
- 阴:开「交一张牌」选择 → 赠予来源并记录追踪牌(技能状态
  `SS1<branch><source u32><card u32>`)→ `turn_ended`(PhaseEnded-End 事件,过滤
  `owner_alive` 使非自身回合亦可触发)校验追踪牌仍在其手/装备区 → 持有者摸至
  四张。
- 「观看手牌」为纯信息展示,引擎无独立观手事件,文档化为近似。

## 接线

`ResolveJianxiang`(操作码 553)/`ResolveShenshi`(554)→ vm 弹栈 → vm_host 注入 →
primitives 复合 → 续体标签 `0xDCDC_5740..5743` → hash 臂 528/529 → DSL 词
`resolve_jianxiang(player)`/`resolve_shenshi(player, source, stage)` → IR/codegen/
门控 `ADDITIVE_RULE_VM_ABI_VERSION_574`。

## 测试与发布

- Golden `original_shenhua_kuailiangkuaiyue.rs` 4 项:契约(ABI 574/三触发一主动)、
  荐相选手牌最少者摸一张、审时阳交牌伤害、审时阴追踪牌回合末摸至四张。全绿。
- 52 个 shenhua 测试文件零失败;engine-core 全量 0 失败;conformance 8/8 ALL PASS。
- npack:ABI 574、技能 134、角色 41(蒯良蒯越 id 1354173421,魏 3 体力 男)、触发
  97、active 15、修正器 46、模块 112;`extensions/shenhua/{shenhua,package}.npack.json`
  同步。

## 近似与遗留

- 「观看手牌」未建模(纯信息);「手牌数最多/最少」在触发时计算,与旧 JS 一致。
- 审时阳的死亡奖励仅覆盖直接死亡(旧 JS 亦仅触发者死亡时)。
