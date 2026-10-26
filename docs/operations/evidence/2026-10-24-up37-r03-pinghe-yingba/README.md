# UP37-R03 冯河/英霸旧线部分修复与 `has_skill` 编译器缺陷（2026-10-24）

核验基准 `7e28a93799a9c01794085e911a72a4faa5673883` 加未提交工作树；规则 **v322**、存档 **v105**。本批只改旧线内容/编译器，**原生同名技能仍未注册**；明确是**部分完成**，不把旧线定向通过当作 native/上游等价。

## 完成内容

1. **`has_skill(player, <u32>)` 无法编译（根因修复）**
   - `content-compiler/src/typecheck/expressions.rs` 的查询参数只有 `mark_count/skill_pile_count/skill_content_flag` 特判 u32；`has_skill` 第二参落入通用表达式检查，被 `ExprAst::U32` 拒绝（“u32 literal is only valid for an identifier argument”），与 ADR 0632 的 DSL 形态矛盾。
   - 增加 `has_skill` 第二参 u32 特判，按 `mark_count` 同样的 `i32::try_from`/`ExprIr::U32` 处理。
   - 新增 `content-compiler/src/tests/has_skill_query_u32.rs`：`has_skill(self, 2187983153)` 可以编译并登记 `phase_end` trigger。

2. **英霸 `yingba`**
   - 目标先 `lose_hp 1`；若仍存活，再 `change_max_hp -1`、`add_skill 3219705664`、`add_mark 1`；最后自己 `change_max_hp -1`。
   - 效果内检查目标 `max_hp > 1`；**但选择阶段仍可点到上限 ≤1 的目标**，只是不产生效果（未完成选择过滤）。
   - 平定的跨角色手牌上限计数仍是 `owner_skill_mark` 近似。

3. **冯河 `pinghe`**
   - 交牌后 `has_skill(self, 2187983153)` 判断**拥有**（`HasSkill` 查实例，不查 enabled），来源存活则 `add_skill 3219705664` + `add_mark 1`。
   - 因此“暂时失效但未移除”仍给标记；移除后不给。测试覆盖有/无英霸两分支。
   - **重铸分支未实现**：仍强制 `choose_other_give_one_hand_equip`，没有“交给目标或重铸”的二选一。
   - 手牌上限基数=已损失体力仍未建模。

4. **包版本**
   - `content/original/ported/shiji/shiji.npack.json` 由 **0.1.0 → 0.1.1**；`scripts/rebuild-pack.sh` 支持 `PACK_VERSION` 环境变量（默认仍 0.1.0），metadata 增加 0.1.1 changelog。
   - 新 logic hash `[179,149,115,23,230,222,…]`（`b3957317…`）；旧 0.1.0 的 hash 不同。按现有内容锁机制，旧局应被**显式拒绝恢复**；本批未做 0.1.0 存档迁移/拒绝的专项实测。
   - 包内 `trigger_index/active_index/modifier_index/skill_index` 条目数不变（各 0 removed / 0 added）；`asset_manifest/asset_paths` 完全不变；仅 `pinghe/yingba` 模块与 manifest logic hash 变化。

## 验证

| 检查 | 结果 | 证据 |
|---|---|---|
| `cargo test -p engine-native` 完整（旁证未受影响） | **1548 / 0 / 71**（316 target） | [日志](engine-native-full.log)、[计数](engine-native-counts.json)、[退出码](engine-native-full.exit) |
| `cargo test -p content-compiler` 完整 | **534 / 0 / 0**（3 target，含新增 1 项） | [日志](content-compiler-full.log)、[计数](content-compiler-counts.json)、[退出码](content-compiler-full.exit) |
| `cargo test -p engine-core` 完整 | **3782 / 0 / 1**（171 target，含新增 2 项） | [日志](engine-core-full.log)、[计数](engine-core-counts.json)、[退出码](engine-core-full.exit) |
| `cargo test -p engine-core --test original_merged_shiji` | **104 / 0 / 0** | [日志](shiji-targeted.log)、[退出码](shiji-targeted.exit) |
| `cargo test -p engine-wasm` | **2 / 0 / 0** | [日志](engine-wasm-full.log)、[计数](engine-wasm-counts.json)、[退出码](engine-wasm-full.exit) |
| 重建 `engine-wasm/pkg-node` + Node 冒烟 | wasm-pack exit 0；`nc12-old-wasm-bench` exit 0 | [构建](engine-wasm-node-build.log)、[冒烟](engine-wasm-smoke.log) |
| 完整 preflight（最终） | **20/20 通過、0 失敗／跳過** | [日志](preflight-final.log)、[退出码](preflight-final.exit) |
| `sync-original-content --verify-only` | exit 0（最终复跑） | [首次](sync-verify.log)、[最终](sync-verify-final.log) |

定向新增测试：
- `yingba_loses_hp_then_max_hp_and_grants_mark_to_living_target`：目标 4/4 → 3/3，获得技能+1 标记，自己上限 3。
- `pinghe_branch_and_mark_only_with_yingba_owned`：有英霸时来源获得标记/技能；移除英霸后不再获得；两次都经过目标选择→交牌选择。
- `has_skill_accepts_literal_skill_def`：编译器接受 u32 技能定义。

## 失败与边界（未隐藏）

- **防伤时机是真实缺陷（未修）**：直接 `DealDamage` 与真实 `PlayCard` 杀两条路径都在**到达冯河选择时体力已减少**（4→3），而 `prevented_damage_ids` 仍为 `[1]`；说明 `damage_before` 触发里的 `prevent_current_damage` 在该嵌套选择窗口来不及阻止已提交伤害。该测试因此**不**断言防伤结果；这是待独立修复的引擎时序问题，不能据此宣称“冯河防伤已验收”。
- **重铸分支缺失**：上游“交给目标或重铸”仍只实现前者。DSL 无现成可选目标/重铸混合原语；需要新增效果或两阶段选择设计，不能用一个近似冒充。
- **目标过滤缺失**：上限 ≤1 的目标仍可被选择，但效果内 `if 1 < max_hp(target)` 使其无效果；与上游 `filterTarget` 不等价。
- **动态手牌上限缺失**：冯河“手牌上限基数=已损失体力”未建模。
- **旧局兼容**：0.1.0 → 0.1.1 未提供迁移；按内容锁应拒绝而非静默接受新逻辑，但本批没有实际拿 0.1.0 存档做恢复反例。
- **原生**：`engine-native` 没有同名冯河/英霸注册；本批不能计入 native/37 来源等价。

## 版本与 golden

- 规则 **v322**、存档 **v105** 未变；没有改 golden 期望值来消除失败。
- `original_shiji_zjjuxiang::saved_other_player_can_trigger_damage_then_max_hp_draw` 原先假设包版本固定 0.1.0；已将测试内编译包的 `manifest.version` 同步为 committed npack 的 0.1.1 后再计算 logic hash。这是版本语义绑定，不是放宽断言。
