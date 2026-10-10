# ADR 0628：gw_dudayuanshuai2 per-card 禁弃置门禁——结构性不可表达 + 弃牌域收窄

> 承接 [ADR 0627](0627-gwent-gwyewu-card-creation.md)：gwent（昆特牌）运行时收尾最后
> 一项。对 ADR 0608 登记的「gw_dudayuanshuai2 per-card 禁弃置门禁」给出**终结性评估**：
> 在杜达元帅以技能近似的模型下结构性不可表达；同时将 phase_end 近似从「弃两张手牌或
> 装备牌」收窄为「弃两张手牌」（更贴合上游）。

## 上游语义（旧 JS 仅作语义证据，不进入运行时）

杜达元帅2（捣蛋鬼，特殊卡）：①**你不能使用、打出或弃置此牌**；②结束阶段，若此牌在
你手牌中，你弃置之并随机弃置一张手牌。①由 hupeng2 的 cardDiscardable/cardEnabled/
cardUsable/cardRespondable 四个 mod 表达（按自定义卡 def 门禁）。

## 结构性评估（终结）

1. **模型前提**：本引擎将杜达元帅特殊卡整体以**技能近似**（gw_dudayuanshuai1_skill/
   gw_dudayuanshuai2_skill 即卡之运行时，hupeng 经 ABI 644 把技能置入目标手牌）——
   不存在实体卡，故「不能弃置此卡」无可门禁的物理对象。
2. **modifier 系统**：引擎的弃牌保护查询 `card_discard_protected_from` 与
   `card_gain_protected_from` 均按**卡牌类别/花色**选择器（`for_card_discard_protection
   (kind, suit)`）；modifier 无按自定义卡 def 的禁用门禁。即使实体卡存在，也需要扩展
   modifier 系统 + 弃牌管线查询，属大型改动。
3. **结论**：per-card 禁使用/打出/弃置面在技能近似模型下**结构性不可表达**，维持
   ADR 0608 登记、本 ADR 终结（不再挂起）；若未来将 gwent 特殊卡改为实体卡模型，
   需同步引入 def 门禁 modifier。

## 做了什么（近似收窄）

`shahai_impl.skill` gw_dudayuanshuai2_skill 的 phase_end：
- 原：`discard_hand_equip_count(self, 2)`（弃两张手牌或装备牌，玩家自选）；
- 改：`discard_hand(self, 2)`（**弃两张手牌**——上游两弃均为手牌：此牌本身 + 随机
  一张其他手牌；ADR 0628 收窄弃牌域，注释同步更新）。

仍为近似：弃牌由玩家选择（上游随机弃置）；「此牌本身」因技能模型无实体卡而并入
「两张手牌」计数。

## 验证

- `crates/devtools/tests/gwent_runtime.rs` 扩展：gw_dudayuanshuai2 phase_end 模块真实
  执行 → 正确发出 `DiscardFromHand{ player, count: 2 }`（与 hupeng / dudayuanshuai1 /
  gwyewu 同守卫，现覆盖全部四个 gwent 宿主效果）。
- conformance：shahai 三向量（impl_repro / dudayuanshuai_abi643 / hupeng_abi644）仍
  通过（内容改动只影响 phase_end 语句，`abi >= 643/644` 不变）。

**ABI 说明**：无 `RULE_VM_ABI_VERSION` bump——纯内容语句替换。

## 后续

- gwent 特殊卡若改为实体卡模型，需同步引入「按 def 的 card 门禁」modifier
  （ADR 0608 挂起项，本 ADR 终结评估）。
- gwent 运行时至此全部落地：gw_wuyao/gw_lang（view_as + vanish，ADR 0613）、
  gw_dudayuanshuai1/2（ADR 0608/0628）、hupeng（ADR 0612/0626）、gwyewu（ADR 0627）。
