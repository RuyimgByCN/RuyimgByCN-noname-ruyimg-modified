# UP37-R10 元数据子集核验（2026-10-24）

核验基准 `7e28a93799a9c01794085e911a72a4faa5673883` 加未提交工作树；规则 **v322**。本批只覆盖 **R10 的显示/元数据子集**，不包含 AI 估值或行为等价；不把本批当作 R10 完成。

## 内容

新增 `crates/engine-core/tests/up37_r10_metadata.rs`，直接读取 `content/original/catalog.json`：

1. `tw_zhugejun` / `dc_zhugejun` 的 `names = 诸葛|均`、`clans = [琅琊诸葛氏]` 在 I01 同步后保留。
2. `wufan` 的技能引用 `tianyun` / `wfyuyan` 均有 `resolved_skills`，不是悬空引用。
3. `stardangchen`（荡尘）描述使用最终上游口径：「令一名其他角色交给你任意张牌」与「此牌额外结算一次」。
4. `twrouke`（柔克）描述保留「直到下轮游戏开始」的期限。
5. `spyishi`（义释）描述保留「获得其装备区内的一张牌」的奖励；`hm_tuntian`（屯犬）显示名与重置来源保留。

## 结果

`cargo test --offline -p engine-core --test up37_r10_metadata`：

- **5 passed / 0 failed / 0 ignored**，exit 0；见[日志](tests.log)、[退出码](tests.exit)。
- `cargo test --offline -p engine-core` 完整 **3787 / 0 / 1**（172 target）；见[完整日志](engine-core-full.log)、[计数](engine-core-counts.json)、[退出码](engine-core-full.exit)。

## 未完成与边界

- **不是 R10 全量**：应机/共护/自矜/勇剑/化伤等 AI 估值、随机源、装备/判定牌有限值仍未验证，也未接入 Planner。
- 已补 TW柔克期限、TW义释提示、汉末屯犬显示名/来源；TW扶蜀伤害层数等其余显示项仍未逐一验证。
- 元数据通过不代表规则行为、组合、保存恢复或 37 来源整包验收。
- 未改规则/存档版本，未改 golden。

## 2026-10-24 追加：AI 内容调参有限值

- `ContentTuning::weight_of`：NaN/Inf 席位权重回退 1.0，有限值仍夹在 [0.1,10.0]。
- `threat_of_skills`：非有限威胁度回退 1.0。
- `benefit`：非有限权重/结果不产生 NaN/Inf；有限值仍按 `qzCf` 相乘。
- 新增 `nc10_ai_bounds::content_tuning_rejects_non_finite_values`；`engine-native` 完整 **1550 / 0 / 71**（316 target），见 [日志](engine-native-ai-finite.log)。
- 这是 R10 AI 有限值要求的一部分；R10 的应机/共护/自矜/勇剑/化伤估值与 Planner 行为仍未完成。

- 新增 `nc10_ai_bounds::zero_deadline_returns_timed_out_plan_without_blocking`：`deadline_ms=0` 时计划必须 `timed_out`、修订号与引擎一致，且不阻塞（<2s 测试上限）。

## 2026-10-25 AI 主动技候选（R10 子项）

- `engine-native` AI 变体 ≥2 现在把 `legal_actions` 中的 `ActivateSkill` 纳入候选（最多 8 个），此前 AI 从不考虑主动技。
- 变体 0/1 均加 `variant_*_does_not_include_active_skill_candidates` 守门；新增 `variant_two_includes_granted_active_skill_candidates`：授予「仁德」并给一张杀后，候选含 `ActivateSkill`；`variant_two_can_choose_granted_active_skill`：空手、授予「不臣」且自身体力 1 时，`plan_action` 实际选择 `ActivateSkill`。
- `engine-native` **1555/0/71**（316 target）；未改变 `variant 0/1` 行为。
- `variant_zero_does_not_include_active_skill_candidates`：变体 0 仍不进入主动技候选，保证历史行为不变。
- 技能威胁估值（2026-10-25）：变体 2 的敌方体力损失按目标技能威胁度放大（`threat_of_skills`，钳制 0.1–10），新增 `variant_two_weights_damage_by_enemy_skill_threat`：高威胁目标掉血得分高于低威胁目标；威胁度缺省 1 时仍与旧分值一致。
- 未完成：N01 旧线 AI、Planner 尾延迟、更多技能质量维度。
