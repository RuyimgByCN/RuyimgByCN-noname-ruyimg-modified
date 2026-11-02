# 恢复后继续 panic：同栈深多机应答错配（2026-11-02）

## 结论（一句话）

恢复件在同一栈深停着多台挂起机器时，`run` 旧实现按栈深顺序取第一台，把**已校验应答交给了没有提问的机器**；真正提出询问的机器随后读 `ctx.answer()` 命中 `expect("answer present on resume")`，WASM 报 `RuntimeError: unreachable`（`engine_native_kernel::machine::Ctx::answer`）。修复后应答只交给提出询问的那台（`Park::awaiting_answer`，运行时标记，不入快照/哈希，读档由 `snap::active_ask` 与开放选择唯一匹配重建）。

- 规则 **330**／存档 **NGSA 105**／ABI **758**／WS **6** **均未改变**；存档字节面、canonical hash 与既有 golden 不因本修复改写。
- 本次源码基线：`9f4b7423`（main）+ 工作树既有 I02／NC10 未提交改动 + 本修复。

## 反例与证据来源

- 诊断日志：[07-oracle-diagnostic.log](/private/tmp/noname-main36-product-host-20261101T070732Z/seven-legacy-20261101T091200Z/07-oracle-diagnostic.log)（2026-11-01，`park-1` → `restored-continuation` 主错误 `unreachable`；次级 `attempted to take ownership of Rust value while it was borrowed` 是 panic 后句柄借用未归还的清理噪声，**不是根因**）。
- 复现夹具：`park-1.ngsa`（同栈深两台机器：`OptionalGate{遗计}` 已确认、内层 `JiJi` 待自己的询问；`JiJie{机捷}` 正等 `jijie_give_target` 应答）。
- 2026-11-02 亲自复现（三处一致）：
  1. **native Rust**（byte 路径 `archive::decode_archive` + `Engine::restore`）：恢复后 `autoStep` → 存档逐字节等于 `native-park-1.ngsa`，随后 `legal_actions` 的候选探测在 `machine.rs` 的 `answer present on resume` panic（探测捕获后 `actions` 为空、`probe_faults` 增加）。
  2. **WASM**（旧产物 `7c2a9f26…`，3582372 B）：同一序列在 `legal_actions_json` 处 `RuntimeError: unreachable`，随后 `free()` 报借用错误（与诊断日志同型）。
  3. **修复后新产物**（`21d7d18f…`，3583299 B）：同一序列全绿，`legal_actions_json` 返回 3 条候选、`free()` 正常（见 `05-node-seven-case-park1.log`）。

## 修法（3 处源码 + 2 处测试）

1. `crates/engine-native-kernel/src/resolution.rs`：`Park` 增运行时字段 `awaiting_answer`；`Flow::WaitChoice` 时置真、恢复该机时清除；`run` 在**有已校验应答**时只唤醒 `awaiting_answer` 的机器（无应答时维持「同深度按挂起顺序恢复」原语义）。
2. `crates/engine-native/src/engine/snapshot.rs`：读档时用既有 N04-e 承接校验的唯一匹配重建 `awaiting_answer`（提出询问的那台认领应答）。
3. `crates/engine-native-kernel/src/snap/validation.rs`：新增结构不变量「有应答必须有开放选择」，拒绝无人认领的应答档（原子拒绝，原句柄/原局不变）。
4. 新增回归 `crates/engine-native/tests/nc10_answer_routing.rs`（`nc_rounds` 组）与 `crates/engine-native-wasm/tests/nc10_park_answer_routing.rs`。

**未做**：没有压制 panic、没有跳过案例、没有禁用 AI；`choice_source_skill` 的 `source_skill` 门控（裁决 27：仅 `OptionalGate` Own、producer=first-depth consumer）**未改**——`awaiting_answer` 不改变该字段的保守口径，公开投影输出与修复前一致。

## 修复前会失败（实测）

临时把 `run` 的唤醒条件改回「仅按栈深」：

- `nc10_answer_routing::restored_same_depth_answer_reaches_the_asking_machine` → 失败于「机捷取入的牌 23 必须交给所选角色 0」（应答被遗计内层吞掉）。
- `nc10_park_answer_routing::restored_same_depth_continuation_is_accepted_on_both_paths` → 复现 `machine.rs` 的 `answer present on resume` panic 与 `{"actions":[],"revision":3,"seat":1}`。

临时删掉新增的校验：`answer_without_open_ask_is_rejected_and_keeps_the_archive_atomic` → 失败于「有应答、无承接询问的档必须被拒」。恢复修复后 4/4 通过。

## 核验（2026-11-02，Asia/Taipei；`9f4b7423` + 工作树）

| # | 命令 | 结果 | 日志 |
| --- | --- | --- | --- |
| 1 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native` | **1726 passed / 0 failed / 75 ignored**（17 个测试二进制），exit 0 | [01-three-crates.log](01-three-crates.log) |
| 2 | `cargo test --offline --workspace` | **6946 passed / 0 failed / 77 ignored**（169 个测试二进制），exit 0 | [02-workspace.log](02-workspace.log) |
| 3 | `RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native --all-targets` | exit 0，0 warning（Gate 1b strict） | [03-strict-gate1b.log](03-strict-gate1b.log) |
| 4 | `pnpm build:native`（web-client） | exit 0；`engine_native_wasm_bg.wasm` = 3583299 B，sha256 `21d7d18ff7cd97c9f83372de0f148fbd369f71917f0ad7b599d7a234872d7ab7` | [04-build-native-wasm.log](04-build-native-wasm.log) |
| 5 | `node 05-node-seven-case-park1.mjs`（新产物，park-1 全序列） | 全 OK；`saved == native-park-1.ngsa`；`legal_actions_json` 3 条；`free` OK | [05-node-seven-case-park1.log](05-node-seven-case-park1.log)、[脚本](05-node-seven-case-park1.mjs) |
| 6 | `pnpm build:wasm`（legacy） | exit 0 | [06-build-legacy-wasm.log](06-build-legacy-wasm.log) |
| 7 | `pnpm test`（前端） | **441 passed / 0 failed** | [07-frontend-tests.log](07-frontend-tests.log) |
| 8 | `pnpm build`（`vue-tsc --noEmit` + `vite build`） | exit 0；dist 含新 `engine_native_wasm_bg-D64YKIj6.wasm` 3583.30 kB | [08-frontend-build.log](08-frontend-build.log) |
| 9 | `pnpm smoke:native`（真实 Worker） | **82 passed / 0 failed** | [09-worker-smoke.log](09-worker-smoke.log) |
| 10 | `python3 -B scripts/preflight.py` | 全部通过，exit 0 | [10-preflight.log](10-preflight.log) |
| 11 | `cargo test --offline -p desktop-client`（Tauri IPC 宿主） | 12 + 6 passed / 0 failed（`native_engine_host.rs` 6 项） | [11-tauri-ipc.log](11-tauri-ipc.log) |
| 12 | `cargo run --offline -q -p conformance` | ALL PASS，exit 0 | [12-conformance.log](12-conformance.log) |
| 13 | `python3 -B scripts/check-native-wasm-cross-target.py` | `native_wasm_match: true`，exit 0 | [13-cross-target.log](13-cross-target.log) |
| 14 | `bash scripts/cross-target-hash.sh` | `ALL CROSS-TARGET CHECKS PASSED`，exit 0（native 与 legacy wasm start hash 逐字节相同） | [16-cross-target-hash.log](16-cross-target-hash.log) |
| 15 | `cargo test --offline -p engine-native-wasm` | **81 passed / 0 failed**（22 个测试二进制，含新增 `nc10_park_answer_routing` 2 项） | [17-engine-native-wasm.log](17-engine-native-wasm.log) |
| 16 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` + `python3 -B scripts/check-source-lines.py` | OK；9022 文件 ≤2000 行、Rust 非测试 ≤1000 有效行 | 见「全量核验」 |

### 7 个浏览器案例（之前未执行完的那批）

- 先重建 `native-pkg` 与 `dist`，再用**当前生产 `nativeBackend.ts` + 重建 WASM** 重新生成七案例 oracle（`seven-envelope-oracle.mjs`，7/7 通过，`park-1` 的 `continued` 摘要 `26b4d25b…` 与 native Rust 修复路径逐步一致）。
- 然后真实 headless Chrome（`/Applications/Google Chrome.app`）跑 `seven-archive-browser.mjs`：**UI 正向控制 5／正式协议案例 7／恢复后继续 7／malformed 原子检查 7，exceptions 0，`realVueCreatedWorker: true`**；7 次预期坏档拒绝均为 `UnexpectedEof`。
- `park-1` 的 `continued` 存档与 native 路径逐字节一致；`park-0`／`shan-*`／`own-cancel` 的既有 native 引用未被本修复改写。
- 摘要：[14-seven-oracle-summary.json](14-seven-oracle-summary.json)、[15-browser-summary.json](15-browser-summary.json)。完整产物（含截图）在私有临时目录 `/private/tmp/noname-park-fix-20261102/`。
- 收尾：浏览器案例要用「当前源码重建的 dist」。跑完后已把 **git 跟踪**的 `apps/web-client/dist` 五个文件恢复为 HEAD 原状，并删除本次构建产生、未被跟踪的 6 个哈希产物，工作树不残留 dist 改动（重建命令与产物哈希见 [08-frontend-build.log](08-frontend-build.log)）。

## 提交范围

尚未提交；档案清单、混合 WIP 档的拆分建议与建议提交讯息见[提交范围提案](commit-scope-proposal.md)。

## 与上游等价

本修复只改**执行位置（哪台挂起机器领取应答）**，不改规则语义、事件载荷、随机流或存档字段：

- 规则 330／存档 105／ABI 758／WS6 不变；`canonical_hash` 字段集不变（`awaiting_answer` 不入哈希/快照）。
- 17 个测试二进制 1726/0、workspace 169 个二进制 6946/0、conformance ALL PASS、native↔wasm 轨迹 `native_wasm_match: true`、legacy 与 native start hash 相同。
- 唯一轨迹变化只出现在「同栈深多机 + 有待应答」这一此前必然 panic/串答的状态：修复前该状态无法继续，修复后按「应答归提问者」推进（`park-1` 新摘要 `26b4d25b…`）。
- 未覆盖/未做：Tauri 安装包构建（`cargo tauri build`）未跑；真实设备/性能未测；`choice_source_skill` 的 source 门控语义按裁决 27 保持不动。
