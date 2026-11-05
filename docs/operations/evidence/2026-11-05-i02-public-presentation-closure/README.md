# I02 Phase 2+ 公开表现投影、资源绑定与对局自动语音完整实现与验收证据（2026-11-05）

## 1. 范围与授权依据

- **对应台账项**：`UP37-I02`（对局自动语音、生产 `presentation` 构造器、合格定义级资源绑定、本地一次消费）
- **授权与合同**：[裁决 30](../2026-10-31-i02-public-presentation-contract/user-decision30.json)、[proposal-v3.raw.txt](../2026-10-31-i02-public-presentation-contract/proposal-v3.raw.txt)、[proposal-v1.raw.txt](../2026-10-31-i02-public-presentation-contract/proposal-v1.raw.txt)
- **版本轴**：保持当前主树 **规则 333／NGSA 107／content schema 5／ABI 759／WS8** 不变（`presentation` 为易失公开增量投影，不进入权威状态、不进入 `canonical_hash`、不写入 NGSA 存档字节）。

## 2. 实施分步摘要（I02-1 ～ I02-5）

1. **I02-1 不可变合格定义与音频资产绑定**：
   - `crates/engine-api/src/presentation.rs`：扩展 `PresentationAssetCandidate`（携带 `size: u32`、`digest: String`），新增 `QualifiedSkillAssetId`、`QualifiedGeneralAssetBinding`、`QualifiedCardAssetBinding`、`PresentationCatalogLookup` trait。
   - `crates/engine-native-kernel/src/vocabulary/generals.rs`、`host.rs`、`card_skills.rs`、`catalog.rs`：在 `GeneralDef`、`SkillDef`、`CardDef` 与 `Catalog` 上携带不可变定义级合格语音绑定，实现 `PresentationCatalogLookup`。
   - `crates/engine-native-content/src/content/standard/cards.rs`：按 `content/original/asset-manifest.json` 校验存在性、大小与哈希，为内置基本牌与锦囊牌绑定 `male`/`female` 原生语音候选。
   - `crates/content-native-loader/src/presentation_binding.rs` 与 `presentation_binding/original.rs`：装载期为 SDK 包（`PackageAssetSource`）与 `upstream-noname`/`upstream-noname-liuli` 转换包（`OriginalAssetSource`）构建合格角色/阵亡/技能语音候选，严格隔离共享裸技能名跨包串音。
2. **I02-2 发生时点公开承诺与 `LogEvent::CardResponded`**：
   - `crates/engine-native-kernel/src/machine.rs`、`machine/presentation.rs`、`kernel/events.rs`、`resolution.rs`、`rvm_ops.rs`：
     - 在 `Ctx::log` 中按事件发生当刻（occurrence-time）捕获公开可见角色（暗置武将槽必须返回 `None`，防止通过表现层泄露暗将身份）与卡牌身份，附加 `OccurrencePresentation` 到 `LogEntry`（不入存档、不入状态摘要）。
     - 在 `Ctx::emit` 发出 `EventData::CardResponded` 时同步记录 `LogEvent::CardResponded { player, card }`，确保多目标 AOE（如南蛮入侵/万箭齐发）每个目标响应当刻即进入当前段日志切片，不被后续目标挂起窗口吞没。
3. **I02-3 易失 `Engine.presentation_range` 生命周期与公开批次投影器**：
   - `crates/engine-native/src/presentation/public_batch.rs`、`engine.rs`、`commands.rs`、`construction.rs`、`snapshot.rs`：
     - `Engine` 维护易失字段 `presentation_range: Option<(u32, usize, usize)>`（仅正式推进 `submit` 产生非空增量日志时记录；`new_with_catalog`、`from_snapshot`、`restore_bytes`、失败命令及 `simulation=true` AI 克隆一律清空为 `None`）。
     - `project_public_presentation_batch` 将当前提交段内的 `CardUsed`、`CardResponded`、`SkillTriggered`、`PlayerDied` 投影为 `PublicPresentationBatch { revision, entries }`，按稳定 `entry_index` 排序。
4. **I02-4 全宿主生产构造器接通（WASM / Worker / Desktop IPC / Game Server WS）**：
   - `crates/engine-native-wasm/src/session.rs`、`lib.rs`：新增零参数 `NativeSession::presentation_json()` 与导出 `native_presentation_json(handle)`；读档/构造清零，重复调用幂等，同键重放缓存完整 `presentation`。
   - `apps/web-client/src/worker/nativeBackend.ts`：Worker 在 `step` 收到 `Accepted`/`Pending` 后读取 `native_presentation_json` 并校验挂载到 `WorkerStepSuccess.presentation`；`init`/`loadArchive` 显式置 `undefined`。
   - `apps/desktop-client/src/native_envelope.rs`、`native_engine_host.rs`：桌面端 `accepted_from_outcome` 传入 `engine.public_presentation_batch()`；同键重放复用缓存信封。
   - `apps/game-server/src/room_native.rs`、`room_native_actor.rs`、`room_native_actor/timeout.rs`：权威服务端广播（含 AI 自动链、玩家命令、超时自动代答）统一汇总 `PublicPresentationEntry` 并构造 `PublicPresentationBatch`，重连/恢复基线快照置 `None`。
5. **I02-5 前端 `BattleAudioOwner` 与 `BattleView.vue` 接线**：
   - `apps/web-client/src/engine/battleAudioOwner.ts`：实现单实例 `BattleAudioOwner`，负责会话代际（`sessionGeneration`）、基线水位（`baselineRevision`）、严格入场校验、`Original` 与 `FrozenPackage` 资产大小/哈希校验、`(sessionGeneration, revision, entry_index)` 一次消费去重、有界播放队列（上限 8）、静音/音量持久化（`noname-next.battle-audio.muted`/`volume`）与自动播放解锁提示。
   - `apps/web-client/src/views/BattleView.vue`：接入 `BattleAudioOwner` 生命周期与顶部静音/音量/解锁控件；修复 `saveMatch()` 对 `ContentLock` 字段字典序差异的误判。

## 3. 验证命令与结果清单（首次运行与补跑分开留痕）

| 序号 | 检查项 | 命令 | 结果 | 原始日志 |
|---|---|---|---|---|
| 01 | 行数门禁单元测试 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | 2/2 OK | [01-test-check-source-lines.raw.txt](01-test-check-source-lines.raw.txt) |
| 02 | 源码行数与 Rust 有效行门禁 | `python3 -B scripts/check-source-lines.py` | 9112 files <= 2000 lines; Rust <= 1000 effective lines | [02-check-source-lines.raw.txt](02-check-source-lines.raw.txt)、[02b-check-source-lines-final.raw.txt](02b-check-source-lines-final.raw.txt) |
| 03 | 原生三 crate 分层棘轮 | `python3 -B scripts/check-native-layering.py` | 0 upward references <= 0 | [03-check-native-layering.raw.txt](03-check-native-layering.raw.txt) |
| 04 | I02 六项 Rust 专项门禁（Gate 1～6） | `cargo test --offline -p engine-native --test misc i02_public_presentation_closure` | 6 passed; 0 failed | [04-i02-rust-gates.raw.txt](04-i02-rust-gates.raw.txt) |
| 05 | 宿主层 crate 回归测试 | `cargo test --offline -p content-native-loader -p engine-native-wasm -p desktop-client -p game-server` | 全部 0 failed | [05-host-crates-tests.raw.txt](05-host-crates-tests.raw.txt) |
| 06 | 原生三 crate 全量测试 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native` | **1,984 passed; 0 failed; 75 ignored**（17 段） | [06-three-crates-tests.raw.txt](06-three-crates-tests.raw.txt) |
| 07 | Gate 1b 独立 target `-D warnings` | `CARGO_TARGET_DIR=/private/tmp/noname-i02-gate1b RUSTFLAGS="-D warnings" cargo check -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets` | exit 0（0 warnings；私有缓存已按裁决 23 清理） | [07-gate1b-warnings-check.raw.txt](07-gate1b-warnings-check.raw.txt) |
| 08 | `wasm32-unknown-unknown` 编译检查 | `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` | exit 0 | [08-wasm32-check.raw.txt](08-wasm32-check.raw.txt) |
| 09 | 原生 WASM 重建 | `pnpm --dir apps/web-client build:native` | exit 0 | [09-web-build-native.raw.txt](09-web-build-native.raw.txt)、[09c-web-build-native-clean.raw.txt](09c-web-build-native-clean.raw.txt) |
| 10 | 前端单元测试（含首次失败与修复补跑） | `pnpm --dir apps/web-client test` | 首跑 `network-client.test.mjs` 因 AST 抽取 `handleEnvelope` 缺少 `typeof audioOwner` 守卫失败 1 项（保留于 [10-web-test.raw.txt](10-web-test.raw.txt)）；加守卫后补跑 **461/461 passed, 0 failed** | [10-web-test.raw.txt](10-web-test.raw.txt)、[10b-web-test-rerun.raw.txt](10b-web-test-rerun.raw.txt) |
| 11 | 前端 TypeScript 类型检查 | `pnpm --dir apps/web-client typecheck` | exit 0 | [11-web-typecheck.raw.txt](11-web-typecheck.raw.txt) |
| 12 | 真实原生 Worker smoke | `pnpm --dir apps/web-client smoke:native` | `{"total":82,"failed":0}` | [12-web-smoke-native.raw.txt](12-web-smoke-native.raw.txt) |
| 13 | `BattleAudioOwner` 专项单测 | `node --experimental-strip-types scripts/i02-battle-audio-owner.test.mjs` | 8/8 passed | [13-i02-battle-audio-owner.raw.txt](13-i02-battle-audio-owner.raw.txt)、[13b-i02-battle-audio-owner-rerun.raw.txt](13b-i02-battle-audio-owner-rerun.raw.txt) |
| 14 | Native↔WASM 跨目标轨迹 | `python3 -B scripts/check-native-wasm-cross-target.py` | `native_wasm_match: true` | [14-wasm-cross-target.raw.txt](14-wasm-cross-target.raw.txt) |
| 15 | 跨目标哈希门禁 | `bash scripts/cross-target-hash.sh` | `ALL CROSS-TARGET CHECKS PASSED` | [15-cross-target-hash.raw.txt](15-cross-target-hash.raw.txt) |
| 16 | Conformance 全套 | `cargo run --offline -p conformance` | `passed=19 ALL PASS` | [16-conformance.raw.txt](16-conformance.raw.txt) |
| 17 | 前端生产构建 `dist` | `pnpm --dir apps/web-client build` | exit 0 | [17-web-build-dist.raw.txt](17-web-build-dist.raw.txt)、[17b-web-build-dist-rerun.raw.txt](17b-web-build-dist-rerun.raw.txt) |
| 18 | 真实 Headless Chrome 产品回归（NC09） | `node scripts/nc09-browser-drive.mjs` | 全部通过，`console_errors: []` | [18-nc09-browser-drive.raw.txt](18-nc09-browser-drive.raw.txt) |
| 19 | 真实 Headless Chrome 对局音频生命周期驱动 | `node scripts/i02-browser-audio-drive.mjs` | 选将手势解锁、推进 14 步触发真实卡牌语音入队播放（`playedCuesCount: 1`）、静音切换、存读档基线水位更新（`sessionGeneration: 2 -> 3`，`baselineRevision: 14`，不重播历史语音）、`console_errors: []`（调优过程保留于 `19`～`19g`） | [19h-i02-browser-audio-drive-cues.raw.txt](19h-i02-browser-audio-drive-cues.raw.txt) |
| 20 | Workspace 全量测试 | `cargo test --offline --workspace --no-fail-fast` | **7,254 passed; 0 failed; 77 ignored**（175 段） | [20-workspace-tests.raw.txt](20-workspace-tests.raw.txt) |
| 21 | 收口全量 Preflight 门禁 | `python3 -B scripts/preflight.py` | **21/21 全部通过** | [21-preflight.raw.txt](21-preflight.raw.txt) |

## 4. 边界与剩余待办（按裁决 30 如实登记）

- **本批已完成**：裁决 30 批准的公开表现投影 `PublicPresentationBatch`、发生时点公开承诺（暗将绝不泄露）、定义级合格资源绑定（跨包同名技能隔离）、全宿主生产构造器（WASM/Worker/Desktop/Server）接通、`BattleAudioOwner` 一次消费与存读档/重连基线去重。
- **仍留待后续批次（不借本批关闭）**：
  1. C29 技能真实扣费/激活（`SkillActivated` vs `SkillTriggered` 精细语义）及 `provider`/虚拟牌完整矩阵；
  2. VM `KillPlayer` / `DeathMachine` 真实死亡语义（按裁决 30 §4.2 须待共享 `DeathFlow` 单独规则方案，本批不混入）；
  3. 其余 18 类非语音资源、234 条动态音频表达式、2487 条缺 backing 声明及 37 来源逐包八项验收。
