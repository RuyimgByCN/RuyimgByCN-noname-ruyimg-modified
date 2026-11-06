# PA12 批⑦（零升号子批）`rvm_ops/codec.rs` 五项效果编码补齐、`rvm_named.rs` 多词短语与 `once` 放弃不消费配额、NC08 24/24 扩展矩阵验收证据（2026-11-06）

- **核验日期**：2026-11-06
- **源码基线**：`d4ecc3847`（PA12 批⑥提交之上）
- **版本基线**：`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`（NGSA 107）、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`NETWORK_PROTOCOL_VERSION = 8`（WS8）——**零升号**（不改变任何既有快照/归档字节布局、不改变 golden 14 场摘要）。
- **授权依据**：[PA12④ 迁移计划](../../../architecture/native-dsl-migration-plan.md)顶部用户授权（2026-10-27 21:50）与[裁决登记](../2026-10-28-user-decisions/README.md)第 18 项（计划内零升号批次连续推进并本地入库）。

---

## 1. 变更范围与根因修复

### 1.1 `rvm_ops/codec.rs`：补齐 `accept_effect` 允许面中缺失的 5 个 `EffectRequest` 编码/解码标签（tags `27..=29`, `36..=37`）

- **问题**：[crates/engine-native-kernel/src/rvm_ops.rs](../../../../crates/engine-native-kernel/src/rvm_ops.rs#L60-L98) 的 `accept_effect` 允许 37 种 `EffectRequest` 变体，但 [crates/engine-native-kernel/src/rvm_ops/codec.rs](../../../../crates/engine-native-kernel/src/rvm_ops/codec.rs#L35-L212) 的 `enc_effects` / `dec_effects` 此前仅承载 32 种（tags `1..=26` 与 `30..=35`），遗漏以下 5 种已被批④-1 / 批④-3 / 批⑤-1 / 批⑤-3 接通允许面的变体：
  1. `EffectRequest::TurnOver { target }`（tag `27`）
  2. `EffectRequest::PreventDamage`（tag `28`）
  3. `EffectRequest::ModifyCurrentDamage { delta }`（tag `29`）
  4. `EffectRequest::SkipCurrentPhase`（tag `36`）
  5. `EffectRequest::StorageSet { target, key, value }`（tag `37`）
- **触发崩溃路径**：当技能模块在同一次 Halted 落账中先排队 `DiscardHandEquipCount(N>0)`（触发 ADR 0786 J1 提交期挂起 `pump_commit -> Flow::WaitChoice`）、后排队上述 5 种变体之一（例如真实 ported 技能 `sp/mingshi.skill`：`DiscardHandEquipCount { target: Source, count: 1 }` 后紧跟 `ModifyCurrentDamage { delta: -1 }`）时，若在 `rvm_discard_pick` 挂起窗口调用 `Engine::save()`，会经 `VmMachine::snapshot()` → `enc_effects(&self.commit_pending)` 落入 `_ => unreachable!("accept_effect 之外的变体不入队列")` panic。
- **修复**：在 [crates/engine-native-kernel/src/rvm_ops/codec.rs](../../../../crates/engine-native-kernel/src/rvm_ops/codec.rs#L94-L115) 与 [codec.rs](../../../../crates/engine-native-kernel/src/rvm_ops/codec.rs#L179-L210) 补齐 tags `27..=29` 与 `36..=37` 的对称编码与解码，使 `accept_effect` 允许的全部 37 种变体 100% 往返闭合。既往已编码变体（tags `1..=26`, `30..=35`）字节布局零变化。

### 1.2 `rvm_named.rs`：补齐 `targeted_by_other`、`other_dying`、`other_damage_before` / `other_damage` / `other_direct_damage_before` 多词短语

- **问题**：在 `owner_alive_targeted_by_other_*`（如 `sp/xiemu2.skill`、`xianding/dczhangji.skill`）、`owner_alive_other_dying_*`（如 `onlyOL/olbuyi.skill`）、`owner_alive_other_damage_before_*`（如 `key/shiroha_guying.skill`）中，域前缀被切分为 `owner_alive_`（`Atom::OwnerAlive`），剩余部分进入 `protect()` + `phrase()`。此前 `phrase()` 未收录 `"targeted_by_other"`、`"other_dying"`、`"other_damage_before"`、`"other_damage"`、`"other_direct_damage_before"`，导致 `"other"` 被单字词表回退为 `Atom::OtherAlive`（「场上存在其他存活角色」），使：
  - `owner_alive_targeted_by_other_black_card` 在持有者对**自己**使用黑色牌（如黑铁索连环）时误触发；
  - `owner_alive_other_dying_once_per_game` 在持有者**自己**进入濒死时误触发；
  - `owner_alive_other_damage_before_amount_ge_two_hand_nonempty` 在持有者**自己**受到 2 点伤害时误触发。
- **修复**：在 [crates/engine-native-kernel/src/rvm_named.rs](../../../../crates/engine-native-kernel/src/rvm_named.rs#L481-L492) 的 `phrase(key)` 中补齐：
  - `"targeted_by_other"` → `[Seed::Atom(Atom::InTargets), Seed::Atom(Atom::UserOther)]`
  - `"other_dying"` → `[Seed::Not, Seed::Atom(Atom::EvPlayer), Seed::Atom(Atom::Dying)]`
  - `"other_damage_before" | "other_damage"` → `[Seed::Not, Seed::Atom(Atom::EvPlayer), Seed::Atom(Atom::Damage)]`
  - `"other_direct_damage_before"` → `[Seed::Not, Seed::Atom(Atom::EvPlayer), Seed::Atom(Atom::Direct), Seed::Atom(Atom::Damage)]`
- **回归注意**：初版一并加入 `"other_phase_play"` 时，既有合成桥测 `t8_rvm_bridge::t8_skill_pile_then_nonempty_filter` 将 `owner_alive_other_phase_play_skill_pile_nonempty` 挂在 `trigger damage`（事件主玩家=受伤者=持有者 1，当前阶段=`Phase::Play`）上测试 `skill_pile_nonempty` 门控，导致 `!Atom::EvPlayer` 拦截该合成用例（首跑失败保留于 `03-3crate-tests-first-run-failed.log`）。本批只收敛上述已由真实 `.skill` 整局与旧线差分验证的三类多词短语，`t8_skill_pile_then_nonempty_filter` 恢复通过。

### 1.3 `PA12 §2 D` 遗留收口：`once` / `once_per_round` / `once_per_game` 在 `OptionalGate` 取消（Decline）时不消费限次配额

- **验证**：在 [crates/engine-native/tests/pa12_batch7_codec_named_and_once.rs](../../../../crates/engine-native/tests/pa12_batch7_codec_named_and_once.rs#L406-L520) 与 [crates/engine-core/tests/pa12_dsl_legacy_differential.rs](../../../../crates/engine-core/tests/pa12_dsl_legacy_differential.rs#L455-L560) 中，使用真实 ported 技能 `offline/liangfan2.skill`（`owner_is_target_once`）与 `onlyOL/olbuyi.skill`（`owner_alive_other_dying_once_per_game`）验证：
  - 当 `OptionalGate`（`trigger_confirm`）选择取消（`ChoiceOption::Pass`）时，`VmMachine` 不进入执行段，`vm_aux.once` 不写入 `(owner, skill, scope)`；
  - 同一回合/同一局后续再次满足触发条件时，仍正常弹出 `trigger_confirm`；确认发动后 `vm_aux.once` 写入配额，同回合/同局第三次不再触发；
  - 旧线 `vm_host` 同样只在模块实际执行并 `Halted` 落账时消费限次，两线语义完全一致，闭合 [PA12④ 计划](../../../architecture/native-dsl-migration-plan.md) §2 D「取消时 once 是否消费两线未验」遗留项。

### 1.4 NC08 扩展矩阵扩面：22 包 → **24/24 全规则扩展包**，并启用 `guard_prepare_phase_exists`

- [crates/engine-native/tests/nc08_save_restore_matrix.rs](../../../../crates/engine-native/tests/nc08_save_restore_matrix.rs#L21-L154) 与 [crates/engine-native/tests/nc08_upgrade_matrix.rs](../../../../crates/engine-native/tests/nc08_upgrade_matrix.rs#L20-L156) 的 `load(pkg)` 扩展支持 `characters.json` + `package.npack.json`（经 `content_schema::ContentPackage` + `bridge_vm_skill_decls` 装载 VM 规则扩展包），将 `sp2-upstream-noname` 与 `xianding-upstream-noname` 纳入矩阵，实现 `packages/extensions/` 下全部 **24/24** 含规则载荷扩展包的存档恢复与版本升级拒绝覆盖。
- [crates/engine-native/tests/nc08_registered_defect_guards.rs](../../../../crates/engine-native/tests/nc08_registered_defect_guards.rs#L403-L413) 解除 `guard_prepare_phase_exists` 的 `#[ignore]`（R05 已落地 `Phase::Prepare`）。

---

## 2. 首次失败与修复后重跑记录（按 AGENTS.md 分开留痕）

1. **定向测试首跑失败（测试构造缺陷，非内核缺陷）**：
   - `pa12_batch7_other_dying_and_once_decline_preserves_turn_and_game_quota` 首跑在 `crates/engine-native/tests/pa12_batch7_codec_named_and_once.rs:432` 断言 `left: "trigger_confirm", right: "dying_save"` 失败：因为 `DamageFlow` 在 `SubmitHp` 后先经过 `TargetDamage`（触发 1 号位自身的 `liangfan2` `trigger_confirm`），再进入 `DyingPending` 发出 `dying_save`。随后第二次运行因对 `dying_save` 非救援座提交 `vec![ChoiceOption::Pass]`（`dying_save` 的 `options` 仅含桃/酒 `ChoiceOption::Card`，放弃须传空 `Vec::new()`）被 `Rejected(InvalidChoice)` 而在无界 `while` 循环挂起（`task-4993` 超时终止）。修正为有界循环并在 `submit()` 内强制断言 `!matches!(out, CommandOutcome::Rejected(_))`、取消传 `Vec::new()`。
   - `pa12_batch7_targeted_by_other_xiemu2_and_dczhangji_reject_self_use` 首跑在 `crates/engine-native/tests/pa12_batch7_codec_named_and_once.rs:400` 断言 `left: 4, right: 6` 失败：因 `e.debug_place(0, "sha")` 取出的首张杀在 seed `997_103` 下为红色杀，不满足 `xiemu2` 的 `black_card` 条件。修正为从牌堆精确选取黑色（Spade/Club）杀。
2. **三 crate 全量首跑失败（`03-3crate-tests-first-run-failed.log`）**：
   - 命令：`cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native`
   - 结果：`misc` 组 `t8_rvm_bridge::t8_skill_pile_then_nonempty_filter` 1 项失败（`left: 107, right: 106`），原因见 §1.2。
   - 修复后重跑（`03b-3crate-tests-green.log`）：原生三 crate 全量 **1,998 passed; 0 failed; 74 ignored**（`engine-native-kernel` 4 + `engine-native-content` 4 + `engine-native --lib` 15 + 11 个集成测试组 1,975 = **1,998**，其中 `nc08_registered_defect_guards` ignored 由 50 降至 49）。
3. **`preflight.py` 首跑失败（`07-preflight-first-run-failed.log`）**：
   - 20/21 通过，仅 `本地链接（全部）` 报 `docs/operations/evidence/2026-11-02-park-answer-routing/README.md` 第 12 行指向 `/private/tmp/noname-main36-product-host-20261101T070732Z/.../07-oracle-diagnostic.log` 因系统清理 `/private/tmp` 失效。将该 `/private/tmp` 临时路径改为行内代码格式并指向同目录归档的 `05-node-seven-case-park1.log` 后重跑（`07b-preflight-green.log`），**21/21 全部通过**。

---

## 3. 验证命令与结果汇总

| 检查项 | 命令 | 结果 | 原始日志 |
|---|---|---|---|
| 行数门禁自测 + 全仓扫描 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py' && python3 -B scripts/check-source-lines.py` | 2 tests OK；**9,114 files <= 2000 lines; Rust non-test <= 1000 effective lines** | `06-lines-and-layering.log` |
| 分层棘轮检查 | `python3 -B scripts/check-native-layering.py` | **0 upward references <= 0** | `06-lines-and-layering.log` |
| PA12 批⑦ 原生整局测试 | `cargo test --offline -p engine-native --test misc pa12_batch7_codec_named_and_once::` | **5 passed; 0 failed** | `01-pa12-batch7-native.log` |
| PA12 批⑦ 旧线 `vm_host` 差分 | `cargo test --offline -p engine-core --test misc pa12_dsl_legacy_differential::` | **7 passed; 0 failed** | `02-pa12-batch7-legacy-diff.log` |
| NC08 24/24 存档恢复与升级矩阵 + 缺陷守卫 | `cargo test --offline -p engine-native --test nc08` | **78 passed; 0 failed; 49 ignored** | `03b-3crate-tests-green.log` |
| 原生三 crate 全量（首跑失败留痕） | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native` | `misc` 组 `t8_skill_pile_then_nonempty_filter` 1 败 | `03-3crate-tests-first-run-failed.log` |
| 原生三 crate 全量（修复后重跑） | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --no-fail-fast` | **1,998 passed; 0 failed; 74 ignored** | `03b-3crate-tests-green.log` |
| 独立 target Gate 1b `-D warnings` | `CARGO_TARGET_DIR=/tmp/noname-gate1b-pa12b7 RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets` | **exit 0（0 warnings）** | `04-gate1b-check.log` |
| WASM32 目标编译检查 | `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` | **exit 0** | `05-wasm32-check.log` |
| 收口 `preflight.py`（首跑失败留痕） | `python3 -B scripts/preflight.py` | 20/21 通过（`本地链接（全部）` 1 项 `/private/tmp` 过期链接失败） | `07-preflight-first-run-failed.log` |
| 收口 `preflight.py`（修复后重跑） | `python3 -B scripts/preflight.py` | **21/21 全部通过 ✅** | `07b-preflight-green.log` |
