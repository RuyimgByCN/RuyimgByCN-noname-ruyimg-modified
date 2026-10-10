# ADR 0763：N08 挂账简化项收尾——顺手牵羊/连营弃牌路径/激将直结算

日期：2026-09-22（第六十四批）
状态：已采纳
上游：[0759](0759-native-n08-zhugeliang-luxun.md) / [0760](0760-native-n08-liubei-machao.md)

## 独立复核更正（2026-09-22）

本批顺手牵羊、连营与激将切片存在且测试通过；但“全部简化项闭环/标准包完整迁移”超出证据。
STANDARD_GENERALS 中曹操缺护驾、甄姬缺倾国等多项；张角仍是鬼道+天妒组合；洛神整链一次确认、顺手/拆桥取首牌等简化也仍保留。
应以逐将逐技能规则表验收，不能用 26 个非样例名称计完整交付。

当前反例、优先级与证据见 [进度主稿](../architecture/native-progress.md)。以下保留当批设计/记录，不作为当前完整验收结论。

## 决策

1. **顺手牵羊（新牌种 `CardKind::Snatch`，目录 +4 张）**：获得距离 1
   的一名其他存活角色一张牌——手牌优先、其次装备（各区首张确定性
   取牌，Dismantle 同款约定）；获得进**来源手牌**（与拆桥弃置相异）。
   校验：距离 > 1 / 目标无牌 / **谦逊持有者**（谦逊对位闭环）→
   `IllegalTargets`。结算走无懈窗口包裹（与拆桥同款），机器快照/存档
   （tag 38）/哈希/restore 六处同步。
2. **连营弃牌路径**：新增 `EventData::CardsDiscarded { player, cards }`
   （`EventKind::CardsDiscarded`）——弃牌阶段机器（`discard_to_limit`
   应答消费）与拆桥弃牌路径在弃置后发出；连营 filter 增加该事件类别
   （`extra_events` 注册）。顺手的夺取同时发出 `CardsMoved`（目标失去
   手牌对连营可见）。
3. **激将直结算**：激将升级为「视为使用」完整语义——helper 确认后借杀
   上桌面 → 刘备选定目标（候选按距离 ≤ 射程过滤；无合法目标或弃权则
   借牌归还 helper）→ 内嵌 `ShaResolveMachine`（source = 刘备）委托
   结算（OptionalGate 同款 inner 委托与快照递归；机器改为手工 fork）。
   取消原「获得进手牌」简化项。

## 版本与验收

- 规则版本 23 → 24（目录加牌 + 事件变体），14 golden 全量重录。
- `liubei_jijiang_borrows_sha_from_shu_helper`（改造）：确认 → 目标
  选择 → 直结算 1 伤；helper 拒绝牌不动；无蜀候选 InvalidChoice。
- `snatch_steals_card_within_distance`：手牌/装备两路径、谦逊拒绝、
  距离拒绝。
- `luxun_lianying_triggers_on_forced_discard`：拆桥弃最后手牌与顺手
  夺牌均触发连营摸回。
- 全量 **210 passed / 0 failed**（native_kernel 131）。

## 后果

N08 挂账简化项全部闭环：激将借牌直结算、顺手牵羊（谦逊对位）、
连营弃牌路径。N08 尾部仅余平台侧工作；引擎侧标准包语义迁移完成。
