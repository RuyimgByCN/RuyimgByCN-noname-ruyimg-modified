# B2 批②：10-24 原生产品链路批（P1 四项 + 房间耐久/提交判定/双重故障/固定包副本 + 真 WS + 事件/hash + Worker VM 包 + 扩展交付/CI + NC10 核验 + 保存成本 + 跨目标）核对入库（2026-10-26）

- 执行时间：2026-10-26 **20:41:08 → 21:08 +08:00**（`HEAD = e6ba869a`，领取时实值；暂存树 `git write-tree` 验证基线 `23426e45`，最终提交树 `b461988c`——两者唯一差别是把 N01 服务端测试 `native_onlyol_general.rs` 移出本批，见 §4）；执行者 ZCode A 道
- 工作包：[台帐 B2](../../../architecture/native-remaining-ledger.md#当前阻断2026-10-26-扫描按优先级) 第二批。12 个 10-24 证据目录整体未入库、其代码与测试仍在工作树未提交；本批把它们按「工作树内容 = 证据所述内容 → 跑登记验证 → 只暂存该批」入库。

**一句话结论**：613 个文件（代码/测试/配置 79 + 12 个证据目录 534 个原始记录文件，约 24 MB）入库；每个证据项描述的机制在工作树源码中逐一核对到位；在暂存树导出（独立 target）上 `cargo check --offline --workspace --all-targets` 通过，game-server **357/0/0**、engine-native-wasm **52/0/0**、content-native-loader **32/0/0**、content-runtime **34/0/0**、engine-protocol **1/0/0**、engine-native **1552/7/71**（7 失败全部为既登记的 B5 未提交包数据，与本批无关），前端 **80/80**、vue-tsc 0 错、真实 WASM Worker 冒烟 **47/47**、扩展存储冒烟通过。两处非本批工作的异常按规则处理并登记：被跟踪的 `apps/web-client/dist/` 曾被本地构建误覆盖（已恢复到 HEAD），`apps/game-server/data/` 是 P1-4 锁机制留下的运行时目录（加入 .gitignore）。

## 1 批内各项与核对（工作树内容 = 证据所述内容）

| 证据目录（10-24） | 声称机制 | 工作树核对（`rg` 存在性） | 入库文件 |
|---|---|---|---|
| [p1-trust-boundaries-sync](../2026-10-24-p1-trust-boundaries-sync/README.md) §1 WASM 坏存档 | `hex_decode_bounded`、`restore_hex_checked`、`MAX_ARCHIVE_HEX_CHARS` | 均存在（engine-native-wasm/src/lib.rs） | engine-native-wasm lib + `tests/restore_boundary_handle.rs` |
| 同上 §2 慢 WS | `Outbound::try_send`、慢信号 `slow_marker`、writer 5 s 超时 | session.rs 8 处、room_native_actor.rs 4 处 | game-server session/actor + nc09_native_actor |
| 同上 §3 重连 pending | `sessionStorage` 持久化、`createdAt` 10 min TTL、`cancelPendingCommands`、BattleView「取消待确认」 | networkClient.ts 7/6/1 处、BattleView 4 处 | web-client networkClient/BattleView + `scripts/network-client.test.mjs` |
| 同上 §4 目录单写者 | `StoreWriterLock`、`.writer.lock`、`try_bootstrap` | 15/3 处；`try_bootstrap` 在 main.rs（证据写 lobby 拆分，实际落在 main.rs） | lobby.rs/main.rs/native_room_store |
| [native-room-durability](../2026-10-24-native-room-durability/README.md) | 独占临时文件、`sync_all`、原子替换、检查点回滚 | native_room_store/ 模块、nc09_native_durability.rs | 同上 + `tests/nc09_native_durability.rs` |
| [native-room-commit-decision](../2026-10-24-native-room-commit-decision/README.md) | `.json.journal` 的 `Prepared`/`Committed` | journal.rs 14 处、commit_tests.rs | native_room_store/journal.rs、commit_tests.rs |
| [native-room-double-fault](../2026-10-24-native-room-double-fault/README.md) | 停写/`decision_uncertain`、`command_receipts` | durability.rs 6 处、receipts.rs 16 处；native_ws_e2e `commit_crash`/`double_failure` | room_native_actor/durability.rs、room_native/receipts.rs、native_ws_e2e/ |
| [native-room-frozen-packages](../2026-10-24-native-room-frozen-packages/README.md) | `frozen_packages` 原局包副本 | game-server src 22 处 | native/frozen.rs、`tests/native_frozen_packages.rs`、`native_upgrade_rollback.rs` |
| [native-ws-delivery](../2026-10-24-native-ws-delivery/README.md) | 真 loopback WS、重连令牌、候选下标 token | native_ws_e2e.rs + 模块 | native_ws_e2e.rs/ |
| [native-events-hash](../2026-10-24-native-events-hash/README.md) | `public_event_code`、`public_event_codes_from`、WASM `state_hash_hex/log_len/public_event_codes_json`、`event_cursor`/`take_public_event_codes` | engine-native view.rs/views.rs、wasm lib 3 处、room_native.rs 12 处、engine-protocol `PublicEvent` | engine-native view.rs/engine/views.rs/accessors.rs、wasm `tests/events_hash_export.rs`、engine-protocol messages.rs |
| [worker-vm-package](../2026-10-24-worker-vm-package/README.md) | `installed_packs` 解析完整 npack、`bridge_vm_skill_decls`、`bridge_generals_with_known` | wasm installed_packs.rs 2/1 处；loader lib.rs 定义；game-server native.rs 2 处 | engine-native-wasm/src/installed_packs.rs、content-native-loader/src/lib.rs、`tests/vm_package_worker.rs`、游戏服 native.rs |
| [native-extension-delivery](../2026-10-24-native-extension-delivery/README.md) | 正式选将保留主公体力/装备技能、`build:desktop` 顺序、独立 `native-web` CI | engine-native general_select.rs、ci.yml `native-web` 2 处、tauri.conf.json `beforeBuildCommand` | engine-native engine/general_select.rs + nc05_general_select.rs、.github/workflows/ci.yml、apps/desktop-client/tauri.conf.json、web-client build-*.mjs、extensionStore.ts、ExtensionsView/SettingsView、NativePackagesPanel.vue、bundledNativePackages.ts、nativeMatchContent.ts、content-runtime store.rs + `tests/uninstall_dependency.rs`、wasm `tests/installed_package_payload.rs` |
| [nc10-ai-boundary](../2026-10-24-nc10-ai-boundary/README.md) | 无产品代码；nc10 测试核对（新增非有限值拒绝用例、planner_pool 改信号同步） | nc10_ai_bounds.rs +32、nc10_planner_pool.rs +14/−10 | 两测试文件 |
| [native-save-cost](../2026-10-24-native-save-cost/README.md) | `examples/save_cost.rs` | 存在 | 该示例 |
| [native-wasm-cross-target](../2026-10-24-native-wasm-cross-target/README.md) | `tests/nc08_cross_target_real.rs` + `scripts/check-native-wasm-cross-target.py` + wasm `tests/cross_target_package.rs` | 存在 | 三文件 |

`Cargo.lock` 只暂存 `engine-native-wasm` 包块（新增 blake3/content-compiler/content-schema 依赖，对应其 Cargo.toml）；`rule-api` 的 `serde_json` dev-dep 行留给批③（其 Cargo.toml 在批③）。

## 2 验证

### 2.1 真实工作树（含他人在途改动，共享 target 已重建）

`cargo test --offline -p game-server --no-fail-fast`（20:44:47–20:47:13）：**358 passed / 0 failed / 0 ignored**，25 个 target，exit 0（[摘要](worktree-game-server-summary.txt)；含批③的 `native_onlyol_general` 1 项）。

### 2.2 暂存树导出 `/tmp/b2-head`（= 提交后干净内容；独立 `CARGO_TARGET_DIR`）

导出相对暂存树只补了不入 git 的数据/依赖：`/tmp/archive` 软链到上游存档（engine-native 测试读上游源码）、9 个未跟踪 `shahai_meta_*.skill`（B4，engine-core 用，本批未跑）、`apps/web-client/node_modules` 软链、gitignored 的 `native-pkg`/`wasm-pkg` 复制。

| 命令 | 结果 | 原始 |
|---|---|---|
| `cargo check --offline --workspace --all-targets --keep-going` | **exit 0，Finished** | [汇总](staged-tree-cargo-summary.txt) |
| `cargo test --offline -p engine-native-wasm --no-fail-fast` | **52 / 0 / 0**（与 worker-vm-package 证据 52/0/0 一致） | [逐二进制](staged-tree-engine-native-wasm.txt) |
| `cargo test --offline -p content-runtime` | **34 / 0 / 0**（证据 33 + 新 `uninstall_dependency` 1） | [逐二进制](staged-tree-content-runtime.txt) |
| `cargo test --offline -p engine-protocol` | **1 / 0 / 0** | [逐二进制](staged-tree-engine-protocol.txt) |
| `cargo test --offline -p content-native-loader` | **32 / 0 / 0** | [逐二进制](staged-tree-content-native-loader.txt) |
| `cargo test --offline -p game-server --no-fail-fast` | **357 / 0 / 0**（树 `23426e45` 为 357/1/0，唯一失败 `native_onlyol_general::onlyol_requires_abi757…` 读不到批③的 onlyOL 未提交 npack；该文件已移出本批，最终树 `b461988c` 不含它） | [逐二进制](staged-tree-game-server.txt) |
| `cargo test --offline -p engine-native --no-fail-fast` | **1552 / 7 / 71**；7 失败 = `unknown skill fz_liegong/fz_xinkuanggu` + 技能清单基线 94，即台帐 **B5**（mark-supplement 包数据未提交），与本批文件无关（B1 补跑时 1546/7/71，+6 为本批 nc05/nc10/跨目标测试） | [逐二进制与失败](staged-tree-engine-native.txt) |
| web-client `pnpm test` | **80 passed / 0 failed**（含本批入库的 3 个测试文件 32 项） | 终端输出 |
| web-client `pnpm typecheck`（vue-tsc） | 0 错误 | 终端输出 |
| `node --import ./scripts/ts-register.mjs scripts/native-worker-smoke.mjs` | **47 checks / 0 failed**（与 events-hash/worker-vm 证据 47/47 一致） | 终端输出 |
| `node --import ./scripts/ts-register.mjs scripts/extension-store-smoke.mjs` | 通过（fail 0） | 终端输出 |
| `check-source-lines.py` / `check_local_links.py --all` / `check-adr-index.py` | 见提交前门禁（§5） | — |

**未重跑**：各证据里的 Chrome 真程序 SIGKILL/生产页面 smoke（需启动服务程序+有头 Chrome；Rust 侧 `native_ws_e2e` 的真进程中断/双重故障矩阵已包含在 game-server 全量中）与完整 `preflight.py`。这些原始日志随证据目录入库，本批不改写它们。

## 3 规则处理的两处异常（非本批工作，已登记）

1. **被跟踪 `apps/web-client/dist/` 被本地构建覆盖**：`.gitignore:22` 本就忽略 `apps/web-client/dist/`，被跟踪的 `index.html` + 4 个资产是 10-11 遗留；工作树里 4 个旧资产被删、`index.html` 指向新的（被忽略的）资产名。按「构建产物输出到独立目录，不覆盖被跟踪 dist」规则，以 `git checkout -- apps/web-client/dist` 恢复到 HEAD（本地新构建产物仍在磁盘、仍被忽略），不提交任何 dist 变化。
2. **`apps/game-server/data/native-rooms/.writer.lock`**：P1-4 单写者锁在测试/运行时创建的目录，无主、不应入库；本批在 `.gitignore` 追加 `apps/game-server/data/`。

## 4 范围外（留给后续批）

- 批③（UP37/engine-core）：`apps/game-server/tests/native_onlyol_general.rs`（N01 服务端测试，依赖 `content/original/ported/onlyOL` 未提交 npack/技能）、rule-api effect-split 33 项、engine-core 55、content-compiler 19、content-schema 3、rule-vm 4、content/original 75、scripts 的 effect-split/preflight 改动、`Cargo.lock` 的 rule-api 行。
- 批④（10-24 文档批）：`README.md`、overview/evolution-plan/rules-baseline/progress-history/self-check-guide、ADR INDEX、10-07/10-18 两个证据 README 注记、architecture-doc-audit/native-remaining-ledger/native-arch-perf-review/upstream-37-impact 证据目录。
- B4/B5 数据文件、`data/`（native-content/server-content 运行数据）、`extensions/shahai/README.md`。

## 5 提交前门禁

`python3 -B scripts/check-source-lines.py`（含本批新增 rust/ts 文件）通过、`python3 -B scripts/check_local_links.py --all` 坏链 0、`python3 -B scripts/check-adr-index.py` 通过；具体数字见提交信息与终端输出。
