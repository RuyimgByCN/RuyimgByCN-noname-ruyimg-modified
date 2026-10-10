# ADR 0445：SHIJI 佯解 / 清雨 + 糜夫人闭环（ABI 430 / 431）

本批发布 SHIJI 朱儁「佯解」（`xinyangjie`，ABI 430）与糜夫人「清雨」
（`qingyu`，ABI 431）；糜夫人（`sp_mifuren`，2496519082，女/蜀/3 血）完成闭环，
作为完整武将进入选将池。旧 JavaScript 仅作语义来源。

## 决策

- **ABI 430 `resolve_xinyangjie(player, target, card, target_card, source, stage, skill)`**：
  出牌阶段限一次（DSL `active targets 1 1 other can_compare`）与一名其他角色拼点，
  **点数小者胜**（引擎首个“小者胜”拼点变体——winner 按点数反转，复用展示/
  弃牌堆提交与 `EmitHandCompareFinished` 事件）。拼点失败时可选一名其他角色
  （在拼点目标攻击范围内，`dcchiying_in_range` 近似旧 JS `canUse`）视为对拼点
  目标使用虚拟火【杀】（`UseVirtualCardOn` + `CardKind::ShaFire`）。三段 choice
  状态机（拥有者拼点牌 → 目标拼点牌 → 火杀来源，可取消）。
- **ABI 431 `resolve_qingyu(player, action, xuancun_def, mbyongjue_def, target, skill)`**：
  使命技，三个 DSL trigger（`damage_before` / `phase_prepare` / `dying`，
  均 `owner_is_target`）汇聚一个原语，使命状态以技能实例 `state_blob`（"QY1"+status）
  记录，唤醒后三分支均失效：
  - action 0 defend：受伤前可弃 2 张 he 牌（`discardable_hand_equip_cards`，
    不足 2 张不开选择）→ 续体 `DiscardCards` + `PreventDamage{current_damage_id}`。
  - action 1 achieve：准备阶段满血且手牌 < 当前轮数（`turn.round + 1` 近似
    旧 JS `roundNumber`）→ 唤醒并授予【选存】（`xuancun`，既有纯 DSL 草稿：
    其他角色结束阶段手牌小于体力时其摸至多两张，`min(2, hp-hand)` 以 1/2 近似）。
  - action 2/3 fail：濒死 → 唤醒 + 减 1 点体力上限（`ChangeMaxHp`）+
    选一名其他角色授予【勇决】（`mbyongjue`，既有纯 DSL 草稿：他人回合首杀
    结算后获弃牌堆实体牌；唯一其他角色时直接授予）。

## 验证

- 新增 `original_shiji_xinyangjie.rs` 3 个行为测试（小者胜无报复、失败选火杀来源、
  失败取消）与 `original_shiji_qingyu.rs` 5 个行为测试（防伤弃牌、不足两张 no-op、
  使命成功获选存且唤醒后 defend 失效、唯一目标直接授勇决、多人选目标）。
- 打包：77 技能 / 23 武将，`rule_vm_abi_version` 502（max 语义）。
- conformance 3/3 通过，无 golden 漂移；engine-core 40 个套件全绿
  （`original_clan_draw_to` 栈溢出为 E 段 Hermes 活跃领域，未动）。
- 顺带协作补位：势魏延（ABI 558）codegen 缺失常量/重复导入、`LoseHp` 变体、
  `RecoverHp.source` 字段等并行会话中间态修复。
