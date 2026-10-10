# ADR 0455：SHIJI 周处闭环（战海 / 彰名，ABI 435）

本批发布 SHIJI 周处（`zhouchu`，2792396690，男/吴/4 血）。技能「战海」
（`rechuhai`，ABI 435）完成闭环；初始技能「乡害」（`xianghai`，既有纯 DSL：
装备牌视为【酒】+ 其他角色手牌上限各减一）与使命成功获得的「彰名」
（`zhangming`，纯 DSL）一并进入选将池。旧 JavaScript 仅作语义来源。

## 决策

- **ABI 435 `resolve_rechuhai(player, target, card, target_card, stage, skill)`**：
  职责技四段状态机：
  - stage 0：出牌阶段限一次（DSL `active targets 1 1 other can_compare`）——
    先摸一张（旧 JS `player.draw()`），再选自己的拼点牌；
  - stage 1：目标选拼点牌；
  - stage 2：提交——装备数 < 4 时拼点牌点数 + (4 - 装备数)（上限 13，旧 JS
    `trigger.num1 + num` 修正），大者胜；展示、双方拼点牌入弃牌堆、
    `EmitHandCompareFinished`；赢则展示目标手牌并按类别（基本/锦囊/装备）从
    牌堆各取一张获得（`first_shared_pile_card_by_category`，旧 JS
    `viewHandcards` + 每类 `get.cardPile`）；输且拼点牌原始点数 < 7 使命失败
    （blob "RCH1" 状态）；
  - stage 3（DSL `card_used owner_alive_equip_used` 触发）：使命未完成且装备
    数 > 2 → 回复至满、移除【乡害】并以【彰名】替换。
- **zhangming（彰名）**：纯 DSL `trigger card_used owner_source_club`（新过滤器，
  梅花牌使用）→ `make_current_card_unrespondable(self, event_card, 自身 def)`
  （`AddCardUsePolicySkill` 的辅助技能 def 传自身）。

## 验证

- 新增 `original_shiji_zhengsu.rs` 第 8 个测试：战海装备补点拼点赢 → 按类别从
  牌堆取两张获得（目标手牌构成基本+装备两类）。
- 打包：85 技能 / 26 武将，`rule_vm_abi_version` 502（max 语义）。
- conformance 3/3 通过，无 golden 漂移；engine-core 41 套件全绿
  （`original_clan_draw_to` 栈溢出为 E 段 Hermes 活跃领域，未动）。
- 顺带协作补位：兵势傅巽调试打印残留、兵势吕乂（ABI 563 起）dispatch/hash
  stub 臂、`QueryResponse::SkillPileCount` VM 处理臂等并行中间态。
