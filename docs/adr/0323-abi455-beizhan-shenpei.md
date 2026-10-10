# ADR 0323：ABI 455 备战目标摸牌+授予技能原语 + SP2 审配

本批为 SP2 包审配（`sp_shenpei`）的「备战」（`beizhan`，2840635277）完整移植，
需要一个新的复合原语 `choose_target_draw_to_max_hp_cap_then_add_skill(player, cap, skill_def)`
（ABI 455）：选择任意存活目标，其摸牌至 min(5, 体力上限)，随后为其添加指定技能。

## 原语设计（ABI 455）

- DSL 表面：`choose_target_draw_to_max_hp_cap_then_add_skill(player, cap, skill_def)`
  —— `cap` 为正整数、`skill_def` 非零；编译为
  `effect_tag::CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_ADD_SKILL`（tag 366）。
- 引擎：`EffectRequest::ChooseTargetDrawMaxHpCapThenAddSkill { player, target, cap, skill, add_def }`。
  - `target == NONE`（VM 首发）：打开目标选择，载荷
    `[CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_ADD_SKILL_CUSTOM_CHOICE(0xDCA5_4550), player, cap, add_def]`，
    沿用 `ContinuationKind::Custom` 载荷魔法标记（B 段 `TWCHENGPU_CUSTOM_CHOICE` 先例，
    避免耗尽 0..=255 的 continuation tag 空间）。
  - 选择结算（`resolve_choice.rs` Custom+魔法守卫臂）：校验后重发同效果并携带所选目标；
    引擎按 `min(5, 体力上限) - 当前手牌` 摸牌（`DrawCardsFromSkill`），随后 `AddSkill` 授予。
- 无需新过滤器：待战子技能自身以既有 `owner_is_target_tied_max_hand` 过滤，以
  `add_skill(self, 4152430403)` 授予 yijiang 已移植的「自守」（`card_targets_self_only`）。

## 技能移植

「备战」（`beizhan`，2840635277）：
- `trigger phase_end owner_is_target optional`：结束阶段选择目标摸至 min(5, 体力上限)，
  `choose_target_draw_to_max_hp_cap_then_add_skill(self, 5, 4111010111)`。
- 「备战·待战」（`beizhan2`，4111010111，新派生技能）：
  - `trigger phase_begin owner_is_target_tied_max_hand { add_skill(self, 4152430403) }`
    —— 手牌并列最多时本回合获得「自守」（不能指定其他角色）。
  - `trigger phase_begin owner_is_target { remove_skill(self_skill) }` —— 同事件移除自身。

## 文档化近似

旧 JS 的 `target.drawTo(Math.min(5, target.maxHp))` 与引擎 `min(5, 体力上限)` 等价
（`choose_target_draw_to_max_hp_cap` 原有语义即按目标体力上限钳制）。「自守」的
`card_targets_self_only` 语义与旧 JS `zishou2` 一致。无行为偏差记录。

## 打包与验证

- `beizhan.skill` + `beizhan2` 加入 `content/original/ported/sp2/`；`sp_shenpei`
  （2498937845，男/群/3 体力，技能 [gangzhi 3573807347, beizhan 2840635277]）加入
  sp2 `characters.json`；catalog 标记 `beizhan` 为 `ported` 并新增 `beizhan2`。
- sp2 npack 重建（ABI 455，27 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/sp2/`（`package.npack.json` 与 `sp2.npack.json`）。
- 引擎测试全绿：engine-api 5、engine-core 54；26 个扩展包全新 store 导入成功。
- 旧 JavaScript 不进入 npack 或运行时。
