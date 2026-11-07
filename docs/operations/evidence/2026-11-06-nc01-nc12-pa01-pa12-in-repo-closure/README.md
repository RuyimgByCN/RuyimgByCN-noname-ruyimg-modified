# Batch 20：`NC08-BLOCKED-FIXES`、`NC01–NC12`、`PA01–PA12` 及 `todo-register.json`（`99/99`）仓内全量收口证据（2026-11-06）

- **核验日期**：2026-11-06
- **基线提交**：`8f7e8b394`（Batch 19）
- **版本常量（零变更）**：
  - `STANDARD_RULESET_VERSION = 333`（`crates/engine-native-content/src/content/standard.rs`）
  - `ARCHIVE_FORMAT_VERSION = 107`（`crates/engine-native-kernel/src/archive.rs`）
  - `RULE_VM_ABI_VERSION = 759`（`crates/rule-vm/src/lib.rs`）
  - `NETWORK_PROTOCOL_VERSION = 8`（`crates/protocol/src/lib.rs`）
  - `CONTENT_SCHEMA_VERSION = 5`（`crates/content-schema/src/lib.rs`）
- **14 Golden Hashes**：零改动（14/14 全部通过）
- **平台与客户端线索**：覆盖 `engine-native-wasm`（`wasm32-unknown-unknown` 编译门禁与 wasm 桥接）、`game-server` 产品 API/超时/扩展生命周期矩阵（`nc02_nc05_extension_lifecycle_matrix`、`nc03_nc09_product_api_and_timeout_matrix`）、跨目标 7 模式确定性验证（`nc01_nc08_multi_mode_matrix`、`nc08_cross_target_real`），以及浏览器/Tauri WKWebView/iOS/Android 模拟器档位与物理真机外设边界登记。

## 1. Part A — 5 项 `NC08-BLOCKED-FIXES` 与 `NC12-FUZZ-LONG-RUN` 悬空引用修复（零 Golden 位移）

| 序号 | 缺陷 / 差异项 | 修复文件与机制 | 回归测试与验证结果 |
|---|---|---|---|
| A.1 | `SwdZhongxinghujiaCard` 同帧先装后弃悬空引用（`NC12` seed `44170` step `10`） | `crates/engine-native-content/src/content/host/step/swd_lingjian_cards.rs`：记录 `let equipped = pool.first().copied();` 并在随机弃置手牌前从 `hand` 过滤 `Some(*c) != equipped`，防止延迟 `Equipped` 帧完成前将同一 `CardId` 先弃入 `discard` 再写入装备栏造成双重驻留 | 解除 `crates/engine-native/tests/nc12_dangling_reference_repro.rs` 中 `nc12_repro_seed_44170_step_10` 的 `#[ignore]`，实跑通过（`1 passed; 0 failed`） |
| A.2 | `Rangjie`（让节）分支 `1`（移动场上一张牌）漏摸 1 张牌 | `crates/engine-native-content/src/content/host/step/skills_cards.rs`：`SelectNature` 分支 `1` 改为置 `self.target = Some(1)` 并先调用 `ctx.start_draw(self.owner, 1)` 进入 `DrawSettle`；`DrawSettle` 若 `self.target == Some(1)` 再清零 `self.target`、收集场上 `equips`/`judge` 并发起 `"rangjie_move_card"` `WaitChoice` | 解除 `crates/engine-native/tests/nc08_upstream_rangjie.rs` 中 `upstream_rangjie_move_branch_must_draw_one` 与 `crates/engine-native/tests/nc08_registered_defect_guards.rs` 中 `guard_rangjie_branch1_draws` 的 `#[ignore]`，全部通过 |
| A.3 | `WuxingPan`（`ChangeCardNature`）缺 `usable: 1` 每回合限一次限制 | `crates/engine-native-content/src/content/host.rs`：在 `host_skill_def` 的 `once_per_turn` `matches!` 列表加入 `HostBehavior::ChangeCardNature` | 更新 `crates/engine-native/tests/nc08_upstream_wuxing_wuxingpan.rs` 断言同一出牌阶段第二次发动返回 `IllegalAction("skill already used this turn")`，`3 passed; 0 failed` |
| A.4 | `JiZhi`（集智）仅对 `Wuzhong` 以外的部分锦囊触发 | `crates/engine-native-content/src/content/standard/skills/ji_zhi.rs`：扩充 `JiZhiBehavior::filter` 匹配全部普通锦囊（含 `Wuzhong`、`Taoyuan`、`Wugu`、`Jiedao` 等非延时锦囊） | 更新 `crates/engine-native/tests/nc08_upstream_jizhi.rs` 断言黄月英打出【无中生有】后手牌净增 2 张（`hand_before + 2`），且 `huangyueying` 既有测试与 14 golden 100% 通过 |
| A.5 | `GangLie`（刚烈）非红桃判定缺「伤害来源弃置 2 张手牌或受 1 点伤害」选择窗 | `crates/engine-native-content/src/content/standard/skills/gang_lie.rs` 与 `crates/engine-native-kernel/src/snap/asks.rs`：当 `!self.asked` 且判定结果非红桃、来源存活且手牌 `>= 2` 时，置 `self.asked = true` 并向 `self.source` 打开 `SelectCards`（`prompt: "discard_to_limit"`, `min: 2, max: 2, allow_cancel: true`）；选 2 张手牌则经 `discard_from_hand` 弃置并发出 `EventData::CardsDiscarded` 免伤（可正常触发【连营】等失牌时机），取消或手牌 `< 2` 则受 1 点普通伤害（可正常触发 `DamageBefore` 等时机；标准 golden 场中来源手牌 `< 2` 直接走受伤害分支，14 golden 零位移） | 更新 `crates/engine-native/tests/nc08_upstream_ganglie.rs` 覆盖弃 2 免伤、取消受 1 伤、弃 2 触发【连营】及取消触发 `DamageBefore` VM 技四分支，全部通过 |

## 2. Part B — `NC01` & `NC08` 七模式全矩阵（`identity`, `kingdoms`, `versus`, `boss`, `doudizhu`, `brawl`, `single`）

- 在 `crates/engine-native/src/engine/construction.rs` 新增 `Engine::new_kingdoms`、`Engine::new_brawl` 与 `Engine::new_single` 公开构造器，补齐 7 种核心模式的统一开局入口。
- 新增 `crates/engine-native/tests/nc01_nc08_multi_mode_matrix.rs`（接入 `crates/engine-native/tests/groups/nc08.rs`）并扩充 `crates/engine-native/tests/nc08_cross_target_real.rs`：
  - `nc01_nc08_seven_modes_opening_invariants_and_card_conservation`：验证全部 7 种模式的开局座位/将槽/身份不变量、卡牌守恒与随机流推进；
  - `nc01_nc08_seven_modes_same_priority_trigger_order_and_mid_choice_archive_roundtrip`：验证全部 7 种模式下同角色同优先级非静默触发技（【大雾】+【狂风】）在 `TurnEndWindow` 均打开 `"trigger_order"` 玩家选序窗、选择窗中途 `encode_archive`/`decode_archive`/`restore` 字节往返及双实例逐步哈希恒等；
  - `native_all_seven_modes_cross_target_and_restore_trajectories`：验证 7 种模式在跨目标对照下的 `state_hash` 与存档恢复步进轨迹一致性。

## 3. Part C — `NC02` & `NC05` 扩展宿主生命周期案例 2/3、3/3 与 `.sig.json` 签名验真

- 新增 `apps/game-server/tests/nc02_nc05_extension_lifecycle_matrix.rs`：
  - `nc02_nc05_case2_resource_ui_pack_signed_lifecycle_and_old_content_lock_restore`：使用 `content_runtime::sign::{generate_keypair, sign_package, verify_package}` 对资源/UI/配置型扩展包生成 `.sig.json` 证书并校验通过，验证篡改 `package.npack.json` 或不可信公钥在验签期即被拒绝，以及安装 → 启用 → 装载 → 升级 → 回滚 → 停用 → 卸载 → 旧 `ContentLock` 精确恢复全生命周期；
  - `nc02_nc05_case3_hybrid_two_pack_dependency_cross_ref_and_hot_uninstall_isolation`：验证双包跨包依赖（基础包 + 依赖包）缺前置拒激、对局进行中热卸载阻断与活局 `Arc<Catalog>` 隔离、以及旧 `ContentLock` 恢复。

## 4. Part D — `NC03` & `NC09` 产品 API 错误码全矩阵、7 模式裁判全信息投影与 30s/31s 超时 WAL 重放

- 新增 `apps/game-server/tests/nc03_nc09_product_api_and_timeout_matrix.rs`：
  - `nc03_nc09_protocol_error_code_matrix_idempotent_receipt_and_vm_fault_rollback`：覆盖 `RevisionMismatch`、`WrongPlayer`、`NoSuchChoice`、`InvalidChoice`、`IllegalAction`、同 `dedup_id` 幂等 `command_receipt` 重放，以及 `VmFault` 故障零部分状态泄漏回滚；
  - `nc03_nc09_seven_modes_referee_player_spectator_projection_matrix`：覆盖全部 7 种模式下 `Observer::Player`（仅见己方暗手牌与私有角色）、`Observer::Spectator`（全暗手牌遮蔽）与 `Observer::Referee`（全座位暗手牌、暗置将槽、私有身份与询问选项 100% 可见且只读不污染 `state_hash`）；
  - `nc03_nc09_30s_choice_timeout_31s_auto_reply_and_wal_store_replay`：验证 30s 选择窗时限下 29s 不误触发、31s 自动触发 AI 代答（`TimeoutAction`）推进 `revision`、将超时座位记入 `managed_seats`，并落盘增量 `.wal.jsonl` 与 `NativeRoomStore::scan` 崩溃重放恢复一致。

## 5. Part E — `NC04` VM 故障/回滚矩阵与 `NC10` 多帧非静止态续行栈暗牌脱敏

- 修复 `crates/engine-native/src/ai/redact.rs` 与 `crates/engine-native/src/ai/remap.rs`：
  - 在多帧非静止态续行栈（`parks.len() >= 2` 或 `stack.len() >= 2`）下，不再整体退化为跳过脱敏，而是仅钉住（pin）被不可重映射机器槽位（如 `Host(state.cards)`）引用的具体 `CardId`，同时继续对非观察者手牌、`core.deck`、敌方 `sxianjing_traps` 及非观察者 `vm_aux.{piles, storage, flags, drawn}`（含保守分支与作用域化 `:cards:{idx}` 槽位）执行洗牌重映射与擦除；
  - 在 `crates/engine-native/tests/nc10_ai_information_boundary.rs` 新增 `planner_is_invariant_under_multi_frame_non_quiescent_continuations`，证明在 `parks >= 2, stack >= 2`（含 `Sha` + `OptionalGate` 以及外层非空 `cards` 的 `Host`【让节】+ 内层 `Draw` `Rvm` 选择窗嵌套）挂起窗下改变敌方暗手牌、牌堆与私有 `vm_aux` 时 `plan_action` 决策与 `sim_faults == 0` 完全不变。
- 解除 `crates/engine-native/tests/nc04_vm_fault.rs` 中 `nc04_pause_and_resume` 的 `#[ignore]` 并补齐真实 `HOST_CHOICE` 挂起→存档解码恢复→双实例续行哈希一致断言；在 `crates/engine-native/tests/nc04_vm_resume.rs` 新增 `nc04_multi_window_fault_rollback_matrix`，覆盖 `Damage`、`PhaseBegin`、`PhaseEnd`、`TurnBegin` 四窗口 VM 故障原子回滚。

## 6. Part F — `NC06`, `NC07` & `PA04` 作用域化技能私有存储与同源持久化

- 新增 `crates/engine-native-kernel/src/state/scoped_storage.rs`（由 `crates/engine-native-kernel/src/state.rs` 重导出）：
  - 提供 `SkillStorageScope::{Turn, Round, Match}`、`SkillStorageValue::{Int(i64), Flag(bool), CardList(Vec<CardId>), PlayerList(Vec<PlayerId>)}`、`SkillStorageKey` 及 `set_scoped_skill_field` / `get_scoped_skill_field` / `export_scoped_skill_storage` / `import_scoped_skill_storage`；
  - 同源承载于 `VmAux.flags`（`NGSA 107` 字节格式与 14 golden 零变化），随 `VmAux::clear_turn` / `clear_round` 自动按生命周期清理；
  - 在 `crates/engine-native-kernel/src/snap/validation.rs::validate_core` 补齐 `declared_kinds`、`declared_slots`、`card_natures`、`detached_cards` 及作用域化存储 `:cards:{idx}` / `:players:{idx}` 的越界引用校验；
  - 在 `crates/engine-native/tests/nc07_state_same_source.rs` 新增 `nc07_all_vm_aux_tables_and_core_maps_are_same_sourced`、`nc06_nc07_pa04_scoped_skill_storage_roundtrip_and_lifecycle` 与越界恢复拒绝断言。

## 7. Part G — `todo-register.json` 全量 `99/99` 收口与物理外设边界声明

- 更新 `docs/operations/evidence/2026-10-16-todo-register/todo-register.json`：原 17 项开放条目与 5 项追加条目全部完成仓内闭环并转为 `"resolved"`，同步追加 `NC01-NC12-IN-REPO-CLOSURE` 与 `PA01-PA12-IN-REPO-CLOSURE` 两项收口总账条目，`meta` 更新为 `item_count: 99, resolved_count: 99, open_count: 0, wholepack_count: 37`。
- **外部物理设备边界声明（非仓内代码缺口）**：中低端 Android/iOS 物理真机 ≥30 分钟温控/功耗/降频实测（`PA08` 真机扩面与 `PA11` 触发前置）及 Windows/Linux 桌面实机对比（`PA09` 跨 OS 切默认前置）受限于当前开发机未挂载物理手机/跨 OS 硬件（`adb devices` 空、`devicectl` 物理设备 unavailable），已在 `native-remaining-ledger.md` 与 `todo-register.json` 的 `resolution` 中明确标注为「仓内代码与模拟器/桌面 WebView 基线已闭环，真机硬件门禁随物理设备接入执行」。

## 8. 门禁验证命令与实测结果

```bash
python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'
python3 -B scripts/check-source-lines.py
python3 -B scripts/check-native-layering.py
cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader
cargo test --offline -p game-server
CARGO_TARGET_DIR=/tmp/noname-gate1b-check RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets && rm -rf /tmp/noname-gate1b-check
cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm
python3 -B scripts/preflight.py
```

- `test_check_source_lines.py`：2 tests OK。
- `check-source-lines.py`：9,187 files `<= 2,000` 总行数、Rust 非测试文件 `<= 1,000` 有效行全部通过。
- `check-native-layering.py`：0 越界通过。
- `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader`：**2,181 passed; 0 failed; 70 ignored**（三 crate **2,137/0/70**、loader **44/0/0**，14/14 golden 零位移通过，详见 [01-3crate-and-loader-tests.log](01-3crate-and-loader-tests.log)）。
- `cargo test --offline -p game-server`：**578 passed; 0 failed; 0 ignored**（35 个测试二进制，含新增 `nc02_nc05_extension_lifecycle_matrix` **2/0** 与 `nc03_nc09_product_api_and_timeout_matrix` **3/0**，详见 [02-game-server-tests.log](02-game-server-tests.log)）。
- Gate 1b 独立 target `-D warnings` 与 `wasm32-unknown-unknown` 编译门禁：0 警告通过。
- `python3 -B scripts/preflight.py`：**21/21 全部通过**。
