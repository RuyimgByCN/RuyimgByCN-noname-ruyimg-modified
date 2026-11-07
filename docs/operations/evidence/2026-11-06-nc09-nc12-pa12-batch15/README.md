# NC09 收口（非身份局 `MatchOutcome` 映射、`SkillView::view_as` 投影与房间随机盐密码哈希）、NC12 收口（追加式 `.wal.jsonl` 段、定期压缩与原生 `WalBlock` 签名）及 PA12 批⑮（`view_as_index` 桥接与 25 项具名 `Resolve*` 效果原语 tags `154..=178`）核验记录（2026-11-06）

- **核验日期**：2026-11-06
- **源码基准**：`f0b8ee605` 之上工作树（提交后记入 [原生重写进度](../../../architecture/native-progress.md)）
- **版本不变量（零增号）**：`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`（NGSA 107）、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`NETWORK_PROTOCOL_VERSION = 8`（WS8）

## 1. 变更范围与模块拆分

1. **子模块拆分（保持 Rust 非测试有效行 `<= 1,000`、总行 `<= 2,000`）**：
   - [`crates/content-native-loader/src/vm_view_as.rs`](../../../../crates/content-native-loader/src/vm_view_as.rs)：实现 `PackageViewAsEntry` 到 `VmViewAsSpec` 的装载桥接，支持单牌 `hand` / `hand_equip` 选牌、目标牌种（`Sha` / `Shan` / `Peach` / `Analeptic` / `Dismantle` / `Snatch` / `Duel` / `Lebusishu` / `Bingliangcunduan` / `TieSuoLianHuan` / `Wuxie` / `FireAttack`）、花色／颜色／类别／牌名过滤器、阶段窗口（`in_play_phase` / `in_response`）、`once_per_turn` / `uses_per_turn` / `uses_per_round` 限次与 `OwnerWounded` / `OwnerHpLe(2)` / `OwnerHandGeHp` 等前置条件。
   - [`crates/engine-native-kernel/src/snap/asks/shapes.rs`](../../../../crates/engine-native-kernel/src/snap/asks/shapes.rs)：从 [`snap/asks.rs`](../../../../crates/engine-native-kernel/src/snap/asks.rs) 拆出 `validate_choice_shape_for_ask` 与 `validate_semantic_options` 并接入 `find_player_vm_view_as` 响应窗校验，使 `asks.rs` 有效行降至约 655 行。
   - [`crates/engine-native-kernel/src/rvm_ops/accept_batch15.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/accept_batch15.rs)：承接批⑮ 25 项具名 `Resolve*` `EffectRequest` 变体的装载／执行前参数边界校验（`accept_batch15_effect`）。
   - [`crates/engine-native-kernel/src/rvm_ops/codec_ext3.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/codec_ext3.rs)：承接批⑮ tags `154..=178` 二进制编解码，与 `codec.rs`（`1..=70`）、`codec_ext.rs`（`71..=108`）、`codec_ext2.rs`（`109..=153` + `255` `u16` 扩展头）共同构成 178/178 允许效果 100% 往返闭合。
   - [`crates/engine-native-kernel/src/rvm_ops/commit_batch15.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_batch15.rs) 与 [`commit_batch15_rest.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_batch15_rest.rs)：承接批⑮ tags `154..=178` 共 25 项具名复合结算器的确定性权威落账。
   - [`crates/engine-native-content/src/content/vm_effects/batch15_resolvers.rs`](../../../../crates/engine-native-content/src/content/vm_effects/batch15_resolvers.rs)：实现批⑮ 25 项具名复合结算器的交互式选牌／选目标／选分支状态机（`step` / `active_ask` / `validate`）。

2. **Part 1 — NC09 收口（[`native_view.rs`](../../../../apps/game-server/src/native_view.rs)、[`catalog.rs`](../../../../crates/engine-native-kernel/src/catalog.rs)、[`native_room_store.rs`](../../../../apps/game-server/src/native_room_store.rs)、[`room_native_actor.rs`](../../../../apps/game-server/src/room_native_actor.rs)）**：
   - **非身份局 `MatchOutcome` 映射**：在 [`crates/engine-api/src/lib.rs`](../../../../crates/engine-api/src/lib.rs) `MatchOutcome` 新增 `TeamWin { team: u8 }`，并在 [`apps/game-server/src/native_view.rs`](../../../../apps/game-server/src/native_view.rs) 将 `Winner::Versus(team)` 映射为 `MatchOutcome::TeamWin { team: team as u8 }`、`Winner::Doudizhu { lord_won }` 映射为 `GroupWin { group: "landlord" | "peasant" }`、`Winner::Boss { boss_won }` 映射为 `GroupWin { group: "boss" | "challengers" }`，消除非身份局终局回退为 `MatchOutcome::InProgress` 的缺口。
   - **`SkillView::view_as` 投影**：在 [`catalog.rs`](../../../../crates/engine-native-kernel/src/catalog.rs) `SkillBehavior` 上新增 `has_view_as` 与 `vm_view_as`，在 `Catalog::is_view_as_skill` 汇总 `s.behavior.has_view_as()` 与宿主视为技（`SKILL_WU_SHENG`、`SKILL_LONG_DAN`、`SKILL_GUO_SE`、`SKILL_QI_XI`、`SKILL_QING_GUO`、`SKILL_JI_JIU`、`SKILL_BOSS_JIUCHI`、`SKILL_BOSS_FURAN` 及扩展宿主视为技名），并在 [`native_view.rs`](../../../../apps/game-server/src/native_view.rs) 投影 `view_as: engine.catalog().is_view_as_skill(def)`。
   - **每房间随机盐密码哈希（`v2:{salt_hex}:{digest_hex}`）**：[`native_room_store.rs`](../../../../apps/game-server/src/native_room_store.rs) 保留 `password_hash`（v1 兼容），新增 `password_hash_with_salt`、`salted_password_hash`（`getrandom` 16 字节随机盐 + BLAKE3 `noname-native-room-pw-v2:` 域分离）与 `verify_password_hash`（自动识别 `v2:` 与旧版 v1 摘要，使用常数时间比较），并在 [`room_native_actor.rs`](../../../../apps/game-server/src/room_native_actor.rs) 接通创房一次哈希持久化与入房校验。

3. **Part 2 — NC12 收口（[`native_room_store.rs`](../../../../apps/game-server/src/native_room_store.rs)、[`durability.rs`](../../../../apps/game-server/src/room_native_actor/durability.rs)、[`persist.rs`](../../../../apps/game-server/src/persist.rs)、[`room_native_actor.rs`](../../../../apps/game-server/src/room_native_actor.rs)、[`timeout.rs`](../../../../apps/game-server/src/room_native_actor/timeout.rs)）**：
   - **追加式 `{room_id}.wal.jsonl` 段与周期性压缩检查点**：定义 `DEFAULT_WAL_CHECKPOINT_EVERY = 16` 与 `WalSegmentLine`；`RoomDurability::commit_step` 在创房首步（`rev == 0`）、每 16 步（`rev % 16 == 0`）、对局结束、故障注入打开或前一步写入失败恢复时执行完整两阶段原子检查点并清理 `.wal.jsonl`，其余中间步仅追加单行 JSONL 并 `sync_data`；`NativeRoomStore::scan` 启动恢复时按序回放所有 `revision > persisted.revision` 的有效段行（忽略崩溃导致的尾部半行）并重算 `content_blake3`。
   - **原生 `WalBlock` 链式签名**：[`persist.rs`](../../../../apps/game-server/src/persist.rs) 新增 `native_snapshot_state_hash(snapshot_hex)` 与 `RoomPersist::record_committed_native_step`，在 [`room_native_actor.rs`](../../../../apps/game-server/src/room_native_actor.rs) 与 [`timeout.rs`](../../../../apps/game-server/src/room_native_actor/timeout.rs) 替换原未使用占位 `_legacy_wal`，对每步已提交原生动作与超时动作累积签名并在达到 `block_every` 或对局结束时封存 `WalBlock`（经 `WalSigner::verify` 验签闭合）。

4. **Part 3 — PA12 批⑮（`view_as_index` 桥接 + 25 项具名 `Resolve*` 效果原语 tags `154..=178`）**：
   - **`view_as_index` 全链路桥接**：`Catalog::find_player_vm_view_as` 支持出牌阶段（[`play_rules.rs`](../../../../crates/engine-native/src/engine/play_rules.rs)、[`play_card.rs`](../../../../crates/engine-native/src/engine/play_card.rs)、[`play_sha.rs`](../../../../crates/engine-native/src/engine/play_sha.rs)、[`legal_actions.rs`](../../../../crates/engine-native/src/engine/legal_actions.rs)）、装备区实体牌转虚拟牌摘除（[`kernel/events.rs`](../../../../crates/engine-native-kernel/src/kernel/events.rs) `commit_card_used`）及四类双端响应窗口（【闪】[`kernel/sha.rs`](../../../../crates/engine-native-kernel/src/kernel/sha.rs) / [`aoe.rs`](../../../../crates/engine-native-kernel/src/kernel/aoe.rs)、【杀】[`duel.rs`](../../../../crates/engine-native-kernel/src/kernel/duel.rs) / [`aoe.rs`](../../../../crates/engine-native-kernel/src/kernel/aoe.rs) / [`combat_rules.rs`](../../../../crates/engine-native-kernel/src/kernel/combat_rules.rs)、濒死【桃】/【酒】[`dying.rs`](../../../../crates/engine-native-kernel/src/kernel/dying.rs) / [`combat_rules.rs`](../../../../crates/engine-native-kernel/src/kernel/combat_rules.rs)、【无懈可击】[`nullify.rs`](../../../../crates/engine-native-kernel/src/kernel/nullify.rs) 与快照校验侧 [`snap/asks.rs`](../../../../crates/engine-native-kernel/src/snap/asks.rs) / [`snap/asks/shapes.rs`](../../../../crates/engine-native-kernel/src/snap/asks/shapes.rs)），并在触发/转化后登记 `VmAux` 限次计数。
   - **25 项具名 `Resolve*` 效果原语（tags `154..=178`）**：
     - `154`: `ResolvePotweiyan`、`155`: `ResolvePotcaoshuang`、`156`: `ResolvePotlvyi`、`157`: `ResolvePotyuji`、`158`: `ResolvePotzhouyu`、`159`: `ResolveMbzhonghui`、`160`: `ResolveDddLiuhong`、`161`: `ResolveTwguohuai`、`162`: `ResolveDddXuelingyun`、`163`: `ResolvePottaishici`、`164`: `ResolveDddXujing`、`165`: `ResolveDddLiuba`、`166`: `ResolveDddLiuye`、`167`: `ResolveDddZhangkai`、`168`: `ResolveHiddenSkillPile`、`169`: `ResolveTwfuman`、`170`: `ResolveTwkujian`、`171`: `ResolveTwxiawei`、`172`: `ResolveCaoyi`、`173`: `ResolveDcSbLusu`、`174`: `ResolveDcSbZhouyu`、`175`: `ResolveMbxiongtu`、`176`: `ResolvePothongyi`、`177`: `ResolveSpyingjia`、`178`: `ResolveDddCaoshuang`。

## 2. 首次失败与修复记录（按 AGENTS.md 分开登记）

1. **`game-server` 全量首跑 2 处既有测试交互与修复**：
   - ① `native_ws_e2e::durability`（`apps/game-server/src/native_ws_e2e/durability.rs` 中 3 项持久性测试）：测试在故障注入恢复后或扩展包挂起询问（`open_choice`）时直接用 `std::fs::read` 读取 `{room}.json` 主文件断言最新 `revision`。在 `RoomDurability::commit_step` 增加 `last_step_failed` 恢复强制全量检查点与扩展包挂起询问检查点后，3 项测试全部通过，同时标准房非检查点步（`1..=15`）继续走 `.wal.jsonl` 增量追加并在第 16 步压缩。
   - ② `apps/game-server/tests/native_onlyol_general.rs`：前序提交 `f0b8ee605`（批⑭）补齐触发事件映射后，`onlyOL` `load_scoped` 的首个未桥接拒装原因由历史 `"game_started"` 推进至 `"modifier_index"`（未移植占位修正器）；同步测试断言子串为 `"modifier_index"` 后通过。
2. **原生三 crate 全量首跑 `batch15_resolvers::step` 非空续行落空问题与修复**：
   - 首跑 `engine-native --test misc` 时，`adr0786_j2_*`、`r05*` 与 `up37_r01_dcdichou` 共 43 项多阶段挂起测试失败。根因是 `batch15_resolvers::step` 在 `state` 非空（即第 2+ 阶段恢复）时未先检查 `effect` 是否属于批⑮ 的 25 个变体，导致排在后面的 `dcdichou`、`fenghe`、`danshou`、`pingkou`、`tiqi`、`baoshu`、`dcbihuo` 在恢复阶段被 `batch15_resolvers::step` 提前拦截并返回 `Some(Flow::Done)`。在 `batch15_resolvers::step` 入口增加 `let owner = batch15_owner(effect, ctx.core)?;` 守卫后，382/382 `misc` 测试一次全绿。
3. **复审边界巡检（3 处跨层一致性补强与回归固定）**：
   - ① [`snap/asks/shapes.rs`](../../../../crates/engine-native-kernel/src/snap/asks/shapes.rs) `is_tao_like` 补齐 `eval_dying_peach_target_canceled`、`eval_card_use_canceled`、`tao_futai2_banned`（真【桃】禁救门禁）与 `CardKind::Tianxianjiu`，与 [`kernel/combat_rules.rs`](../../../../crates/engine-native-kernel/src/kernel/combat_rules.rs) `is_tao_like` 保持双站点一致，消除濒死窗口同时持有被禁真【桃】与 `view_as` 救命牌时 `NGSA 107` 存读档的 `OpenChoiceMismatch`。
   - ② [`play_card.rs`](../../../../crates/engine-native/src/engine/play_card.rs) `CardKind::Jiedao` 分支移除提前 `card_use(...)` 返回，统一落入底部 `card_use_as(..., use_as)` 派发，使 `view_as jiedao` 转化牌正确携带 `use_as = Some(CardKind::Jiedao)` 启动 `JiedaoMachine`。
   - ③ [`batch15_resolvers.rs`](../../../../crates/engine-native-content/src/content/vm_effects/batch15_resolvers.rs) `ResolveHiddenSkillPile`（`action: 1 | 3`）在 `pile_skill_def` 未注册于 `catalog.skills` 时补充回退扫描 `owner` 名下非空牌堆（与 `commit_batch15_rest.rs` `pile_name` 回退 `executing` 技能名对齐），并在 [`pa12_batch15_view_as_and_resolvers.rs`](../../../../crates/engine-native/tests/pa12_batch15_view_as_and_resolvers.rs) 增加上述 3 项回归断言。

## 3. Ported `.skill` 装载桥接盘点

- `content/original/ported/**/*.skill` 共 **5,670** 个文件：
  - **通过 `compile_source_named` + `bridge_vm_skill_decls`**：**3,315 / 5,670**（较批⑭的 `3,079` 净增 **+236** 个 `.skill` 文件）。
  - **`INTENTIONAL_EMPTY_NOOP`（`modifier hand_limit add 0;`）严格拒装**：**1,540** 个（0 漏放）。
  - **触发事件与滤镜拒装（`filter_rejected`）**：**0** 个。

## 4. 全量门禁核验结果

| 门禁项 | 命令 | 结果 |
|---|---|---|
| 行数门禁单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | `Ran 2 tests ... OK` |
| 行数门禁 | `python3 -B scripts/check-source-lines.py` | `9146 files <= 2000 lines; Rust non-test files <= 1000 effective lines` |
| 原生分层棘轮 | `python3 -B scripts/check-native-layering.py` | `0 upward references <= 0` |
| `game-server` 全量测试（含 NC09/NC12 新增回归） | `cargo test --offline -p game-server` | **489 passed; 0 failed; 0 ignored**（33 个测试二进制） |
| 批⑮ 原生整局与盘点测试 | `cargo test --offline -p engine-native --test misc pa12_batch15_view_as_and_resolvers::` | **4 passed; 0 failed** |
| 旧线 `vm_host` 差分测试 | `cargo test --offline -p engine-core --test misc pa12_dsl_legacy_differential::` | **15 passed; 0 failed**（含 `pa12_batch15_legacy_vm_resolvers_match_native_expectation`） |
| 原生三 crate + loader 全量 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader` | **2,110 passed; 0 failed; 74 ignored**（三 crate `2,073/0/74`，loader `37/0/0`） |
| 独立 target Gate 1b `-D warnings` | `CARGO_TARGET_DIR=.../target/gate1b-check RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets` | **0 warnings, exit 0**（临时目录已清理） |
| WASM 目标检查 | `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` | **exit 0** |
| 全量 `preflight.py` | `python3 -B scripts/preflight.py` | **21/21 全部通过** |
