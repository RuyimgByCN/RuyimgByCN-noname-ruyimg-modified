# C331 合并树产品面补跑（2026-11-02）

核验日期 2026-11-02 22:21–22:46（+08:00），执行者 Claude Code；rustc／cargo 1.95.0、Node v26.4.0、pnpm 10.34.3、wasm-pack 0.15.0、Google Chrome 156.0.8078.74，macOS 27.2。

## 树身份

- 运行时 main HEAD `6f98bfa8`，工作区干净（只有本证据目录未跟踪）。`f5e560db..6f98bfa8` 的非 `docs/` 路径只有根 `README.md`，因此源码、测试、包数据与 `f5e560db`（C331 phase7 合并提交，父 `a4483b64`＋`6d36f47a`）相同，下列结果适用于 `f5e560db` 的代码面。
- 版本：规则 331（`STANDARD_RULESET_VERSION`）／存档 NGSA 106（`ARCHIVE_FORMAT_VERSION`）／content schema 5／VM ABI 758／WS7（`NETWORK_PROTOCOL_VERSION`，[裁决 31](../2026-11-02-c331-ws7-boundary/README.md)追认）。
- 树身份、工具链与版本常量见 [00-tree-identity.txt](00-tree-identity.txt)。本次未改任何源码或跟踪产物；构建只写被忽略的 `src/worker/native-pkg/`、`src/wasm-pkg/`、`public/pkg/` 及 `/private/tmp` 私有目录，跟踪的 `apps/web-client/dist` 未触碰（[14](14-frontend-dist-build.log) 末尾 `git status` 为空）。

## 命令与结果

| 文件 | 命令 | 结果 |
| --- | --- | --- |
| [01](01-prebuild-artifacts.txt) | 重建前产物指纹 | native 4,289,679 B `1896724a…`（21:08 合并过程构建）；legacy 15,413,552 B `2b42008d…`（14:56） |
| [02](02-build-native.log) | `pnpm build:native` | exit 0；cargo 判定 Fresh，产物 4,289,679 B，sha256 `1896724a67e5f3fda52c5a18ea93c68829f936dc25e663de45119d235dfe3b3e` |
| [03](03-clean-private-wasm-build.log) | 全新私有 `CARGO_TARGET_DIR`＋`CARGO_INCREMENTAL=0` 下 `wasm-pack build … --release --no-opt` | exit 0（42.1 s）；与 02 **逐字节相同**，排除共享 target 的 Fresh 误判（mtime 陷阱）；私有 target 用后删除 |
| [04](04-frontend-test.log) | `pnpm test` | **449 passed／0 failed** |
| [05](05-typecheck.log) | `pnpm typecheck`（`vue-tsc --noEmit`） | exit 0 |
| [06](06-worker-smoke.log) | `pnpm smoke:native`（真实 Worker） | **82 passed／0 failed** |
| [07](07-native-wasm-cross-target.log) | `python3 -B scripts/check-native-wasm-cross-target.py` | `native_wasm_match: true`，20 步，final hash `59e0d98c…`，exit 0 |
| [08](08-cross-target-hash.log) | `bash scripts/cross-target-hash.sh` | `ALL CROSS-TARGET CHECKS PASSED`，exit 0；含 native conformance ALL PASS、legacy `build:wasm` 重建（15,485,070 B `ea6246ad…`）、`start_4_42`／`start_2_1` native 与 legacy WASM 相同、manifest chain=ok、skill-vm smoke |
| [09](09-desktop-client.log) | `cargo test --offline -p desktop-client --no-fail-fast` | lib 12／`native_engine_host` 6 passed，`native_planner_measurement` 1 ignored，0 failed |
| [10](10-seven-fixture-park.log) | `NC10_OWN_EVIDENCE=… cargo test --offline -p engine-native --test nc_rounds nc10_public_own_plan::complete_own_plan_cannot_depend_on_an_unseen_later_park -- --exact` | 1 passed；写出 331／106 的 `park-0`／`park-1` 夹具 |
| [11](11-seven-probe-build.log) | 私有探针 `cargo build --offline`（路径依赖指向本树，`Cargo.lock` 复制自 main） | exit 0；探针锁 43 包均在 main 锁内，无版本漂移 |
| [12](12-seven-native-reference.log) | 探针生成 native Rust 参照（Shan 四模式、两 park、Own 取消）与解码快照 | exit 0；10 份 `.ngsa` 头均为 NGSA／106／331 |
| [13](13-seven-node-oracle.log) | `node --import apps/web-client/scripts/ts-register.mjs seven-envelope-oracle.mjs <repo> <out> <out>` | exit 0：生产 `nativeBackend.ts`＋重建 WASM 的 7 案例完整信封；WASM 存档与 native **逐字节相同**；恢复只轮换 `auth_generation`；坏档原子；恢复后继续 accepted／pending |
| [14](14-frontend-dist-build.log) | `pnpm exec vite build --outDir /private/tmp/…/frontend-dist --emptyOutDir` | exit 0；dist 内 native WASM sha256 与 02 相同 |
| [15](15-seven-browser.log) | `node seven-archive-browser.mjs <dist> <Chrome> <out>`（真实 headless Chrome） | **UI 正向控制 5／正式协议 7／恢复后继续 7／坏档原子 7，exceptions 0，`realVueCreatedWorker: true`**；7 次预期坏档拒绝均为 `UnexpectedEof` |
| [16](16-strict-gate1b-and-wasm32.log) | 独立 `CARGO_TARGET_DIR` 下 `RUSTFLAGS="-D warnings" cargo check -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets`；`cargo check --target wasm32-unknown-unknown -p engine-native-wasm` | 两项 exit 0；私有 target 用后删除 |
| [17](17-preflight.log) | `python3 -B scripts/preflight.py`（写完本 README 主体与进度主稿条目后运行；本行在运行后追加，追加后另跑两项链接检查见下） | 21 项全部通过，exit 0 |
| [18](18-post-edit-link-checks.log) | `check_local_links.py --all` 与 `check_doc_line_refs.py --strict`（追加 17 行后） | 两项 exit 0 |

同一代码面已由[进度扫描](../2026-11-02-progress-scan/README.md)实跑：行数门禁、三 crate 1903／0／75、workspace 7154／0／77、preflight 全过；本目录不重复这些命令。

## 7 个浏览器案例（331／106 夹具）

旧夹具与参照为 330／105，新 reader 严格拒绝 105，因此本轮在当前源码上重新生成，原 2026-11-01／11-02 产物未改：

1. `park-0`／`park-1`：由既有回归 `nc10_public_own_plan`（遗计确认门；`park-1` 另在同栈深挂起机捷）经环境变量写出，见 [10](10-seven-fixture-park.log)。
2. native 参照：沿用 2026-11-01 探针源码（`/private/tmp/noname-main36-products-20261101T062600Z/probe/src/main.rs`），只把路径依赖改指本树并新增 `snapshots` 模式——直接解码 `native-<case>.ngsa` 写 `snapshot-<case>.txt`，供 oracle 读取 `auth_generation`；WASM 与 native 的存档逐字节相等由 oracle 和浏览器脚本断言，强于原 `compare` 模式的解码相等。改动见 [main.rs.patch](seven-harness/main.rs.patch)。
3. oracle：只把头部断言 105／330 改为 106／331（[patch](seven-harness/seven-envelope-oracle.mjs.patch)）；浏览器脚本只改私有 Chrome profile 根目录（[patch](seven-harness/seven-archive-browser.mjs.patch)）。
4. 夹具、参照与解码快照在 [seven-fixtures-331-106/](seven-fixtures-331-106/)；完整 oracle 信封、浏览器结果和 Worker 收据（gzip）及两张截图在 [seven-results/](seven-results/)。

各案例 `next`／`continued` 摘要前 16 位：park-0 `61e31ab5…`／`0cc8c1ce…`；park-1 `b7559c79…`／`6815444c…`；own-cancel `0f3a8903…`／`4d76364d…`；shan-auto 与 shan-direct `f3aa9645…`／`1963f6ce…`；shan-cancel `f0345e71…`（被拒，revision 不变）／`f3aa9645…`；shan-pass `b589b0fc…`／`fd961f10…`。与 330 轮次（如 park-1 继续 `26b4d25b…`）不同是预期结果：规则版本进入状态摘要，且案例从 331／106 新夹具起步；本轮 native 与 WASM、Node 与浏览器逐步一致。

## golden 与 conformance

本次未改源码，conformance ALL PASS（legacy 金向量不变）；native 14 场 golden 在同一代码面的三 crate 全量中通过（见进度扫描 03），未重录任何期望值。

## 限制

- 只证明 `f5e560db` 代码面的本地产品面；不代表发布、exact-head CI、Tauri 安装包构建、真机或性能复测，也不关闭 C331 仍缺的 DeathFlow／PhaseBegin 等机制与 262／290 剩 28。
- 7 案例覆盖既有 NC10／park 路由场景，不是 C331 新机制的浏览器验收。
