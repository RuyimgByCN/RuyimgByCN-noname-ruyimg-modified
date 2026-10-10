# ADR 0471 — shenhua 卢植【明任/贞良】(ABI 576)

- 状态:已接受
- 日期:2026-08-03
- 归属:C (omp)
- 关联 ADR:0466(蒯良蒯越)、0465(王基)
- ABI:576

## 目标

发布 shenhua 卢植完整武将:

- **明任**:游戏开始时(首轮准备阶段),你摸两张牌,然后将一张手牌置于武将牌上
  称为「任」;出牌阶段结束时,你可以用一张手牌替换「任」。
- **贞良**(转换技):
  - 阴:出牌阶段限一次,弃置一张与「任」同色的牌,对攻击范围内的一名角色造成
    1 点伤害。
  - 阳:回合外你使用或打出的牌结算完成后,若此牌与「任」同色,可令一名角色摸
    一张牌。

## 决策

### 明任 (ABI 576)

- 「任」映射为持有者的技能牌堆(明任 def 为堆键)。
- 首轮准备阶段强制触发 `resolve_mingren(player, 0)`:校验回合 1 且堆空 → 摸两张 →
  开置牌选择 → 移入手牌入堆。
- 结束阶段可选触发 `resolve_mingren(player, 2)`:堆非空且手牌非空 → 开替换选择 →
  新牌入堆、旧「任」回手。

### 贞良 (ABI 576)

- 单复合 `resolve_zhenliang(player, source, card, mingren_def, stage)` 三触发共用
  (active 阴 / card_used 阳 / card_responded 阳),DSL 显式携带明任 def(跨技能
  牌堆引用模式)。
- 转换技分支:技能状态 `ZL1<branch>` 每次发动翻转。
- 阴:开「弃一张与任同色牌」选择 → 开「攻击范围内角色」选择(距离查询) →
  `DealDamage`。
- 阳:回合外(card_used/card_responded,持有者非当前回合角色)且事件牌与「任」
  同色 → 开「任意角色」选择 → 摸一张。

## 接线

`ResolveMingren`(操作码 555)/`ResolveZhenliang`(556)→ vm 弹栈 → vm_host 注入 →
primitives 复合(牌堆/颜色助手)→ 续体标签 `0xDCDC_5760..5764` → hash 臂 530/531
→ DSL 词 `resolve_mingren(player, stage)`/`resolve_zhenliang(player, source, card,
mingren_def)` → IR/codegen/门控 `ADDITIVE_RULE_VM_ABI_VERSION_576`。

## 测试与发布

- Golden `original_shenhua_luzhi.rs` 4 项:契约(ABI 576/两触发一主动)、明任置牌与
  结束阶段替换、贞良阳回合外同色令目标摸一张、贞良阴弃同色牌伤害。全绿。
- 53 个 shenhua 测试文件零失败;engine-core 全量 0 失败;conformance 8/8 ALL PASS。
- npack:ABI 576、技能 136、角色 42(卢植 id 2818365014,群 3 体力 男)、触发 101、
  active 16、修正器 46、模块 117;`extensions/shenhua/{shenhua,package}.npack.json`
  同步。
- 顺带修复:jushou 旧测试的角色池增长回归(新角色强制准备阶段触发),拒绝分支
  扩展为按选择类型结算。

## 近似与遗留

- 「游戏开始时」以首轮准备阶段表达(回合 1 且「任」堆空)。
- 贞良阳的「结算完成后」以使用/打出事件触发(顺序近似)。
