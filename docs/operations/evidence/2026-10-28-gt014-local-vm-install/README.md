# 2026-10-28 GT-014：本地 VM 正式安装全生命周期（NC02／NC05／NC09）

范围：GT-014（规范 §6.2 与第 13 节；本轮指令领取顺序第 2 项）。`apps/web-client/src/engine/bundledNativePackages.ts` 此前硬性拒绝含 VM 模块的包，含 VM 的作者产物只能手工预置 localStorage（[NC02 浏览器证据](../2026-10-27-nc02-browser-vm-trigger/README.md)即如此取证）。本批建立**正式导入入口**并走完 §6.2 全链：正式安装 → 启用 → 开局 → 真实触发 → 存读档 → 升级／回退／停用／卸载，补齐包校验与错误分支，并与服务端核对同包行为。**未改规则／存档／ABI 版本，未改 Worker／WS 消息结构与包格式**。

## 1. 基准

[起点](start-time.txt)约 20:45（NC07 批 `5eec1e9e` 之后）；[HEAD](source-head.txt) `5eec1e9e`；[改动前状态](status-before-commit.txt)。磁盘余量 170+ GB。

## 2. 设计与替代方案

- **正式导入入口 `importExtensionPackage(json)`**（`extensionStore.ts`）：作者产物（完整 npack JSON，可含 VM 模块与卡牌/武将/牌堆资产）经校验后写入本地存储，无需重编译客户端、不依赖内置目录。同包同版本重复导入须内容一致（否则拒绝），其他版本按升级/切换处理（旧版本副本进 `history` 供回退与旧局恢复）；校验失败不改写存储；已撤销不可重装；对局冻结期拒绝。
- **模块结构校验 `validateVmModules`**（`bundledNativePackages.ts`，并入 `validateNativePackage`）：模块名非空唯一、`code` 为非空 0–255 字节、`local_count/max_stack/entry_pc/caps_mask` 非负整数、`entry_pc < code.len`（镜像 rule-vm `verify.rs` 的同界）、模块指纹 32 字节、`skill_index`/`trigger_index` 的引用一致（模块存在、入口在界内、技能 id 已登记）。**不**在 TS 复刻 blake3 规范序列化——模块/包 logic_hash 与 ABI 窗口仍由开局时同一 Rust 校验器裁决（避免客户端与内核漂移），错误分支见 §4 表。
- **内置路径**：`bundledNativePackage` 由「拒绝含 VM 模块的包」改为「含模块的包整包装入（npack + 资产合并）并走同一校验」；当前 15 个内置目录包 `modules: []`，载荷仍为 `manifest + 资产`、逐字不变（有测试守护）。
- **产品 UI**（`ExtensionsView.vue`）：扩展管理页新增「导入扩展包」面板（粘贴/文件选择 + 导入按钮），包列表追加「（导入）」行，显示版本/状态并提供启停/回退/卸载；导入包的「安装/升级」按钮禁用（升级须导入新版本）。
- **夹具**：`emit_vm_fixture.rs` 参数化（版本、摸牌数；缺省输出与既有 `vm-trigger` 夹具[逐字节一致](game-server-gt014-targeted.log)），新增 `vm-trigger-v2`（同包身份 `official/vm-fixture`，v0.1.1，阶段结束摸 2 张）——升级/回退有**可观察的行为差异**（Δ1↔Δ2）。
- 替代方案（不采用）：把夹具塞进内置目录（需重编译，违背 §6.2）；导入期在 TS 计算 logic_hash（需复刻规范序列化，易漂移）；导入期经 Worker 校验（需新增 Worker 请求类型＝协议改动，须另行授权，见 §6）。

## 3. 改动文件

| 文件 | 改动 |
|---|---|
| `apps/web-client/src/engine/bundledNativePackages.ts` | `validateVmModules` 结构校验并入 `validateNativePackage`；`bundledNativePackage` 含模块包整包装入；接口补 `skill_index/trigger_index` |
| `apps/web-client/src/engine/extensionStore.ts` | 新增 `importExtensionPackage`；`upgradeExtension` 对导入包给出精确错误；头部注释更新 |
| `apps/web-client/src/views/ExtensionsView.vue` | 导入面板（粘贴/文件）、导入行渲染、按导入/目录区分按钮可用性 |
| `apps/web-client/scripts/native-vm-install.test.mjs`（新增） | 导入入口 7 项测试（安装/错误分支/升级回退/停用卸载撤销/冻结/内置包不变） |
| `apps/web-client/scripts/native-worker-smoke.mjs` | VM 段改为经正式入口：`runVmTrigger`/`replayFromSave` helper + 8 项生命周期检查 |
| `apps/web-client/scripts/fixtures/vm-trigger-v2/`（新增） | 0.1.1 夹具（npack + 武将资产） |
| `apps/game-server/examples/emit_vm_fixture.rs` | 参数化 `[版本] [摸牌数]`；缺省输出与既有夹具逐字节一致 |
| `apps/game-server/tests/gt014_vm_import_parity.rs`（新增） | 同包同变异的服务端导入核对（2 测） |
| `docs/operations/evidence/2026-10-28-gt014-local-vm-install/`（新增） | 本目录（含浏览器验证脚本与截图） |

## 4. 同包核对（client 导入 / Worker 开局 / 服务端导入）

夹具 `official/vm-fixture@0.1.0` + 变异集（客户端见 `native-vm-install.test.mjs`，Worker 见 smoke 既有拒绝检查，服务端见 [gt014_vm_import_parity](game-server-gt014-targeted.log)）：

| 变异 | 本地导入 | Worker 开局 | 服务端导入 |
|---|---|---|---|
| 模块 `local_count+1`（破坏模块指纹） | 接受（结构层无法判） | 拒 `module … logic_hash mismatch` | 拒（同错因） |
| `entry_pc = code.len` | 拒（入口或预算字段无效） | 拒（不可达） | 拒 `invalid bytecode: entry_pc out of range` |
| 触发索引引用不存在模块 | 拒（不存在的 VM 模块） | — | 拒 `logic_hash mismatch` |
| 篡改触发事件名 | 接受（结构层） | 拒 `logic_hash mismatch` | 拒 `logic_hash mismatch` |
| `package_id = "../x"` | 拒（身份或逻辑指纹无效） | — | 拒 `invalid package_id` |
| 同版本不同内容（合法重算指纹） | 拒「同版本包内容冲突」 | — | 拒 `version 0.1.0 already installed with different content` |
| 合法 v0.1.1 | 接受 → 新局 Δ2 | 通过 | 接受 → 装载 v0.1.1 字节码 |

已知差异（如实登记）：破坏指纹的两类变异（`local_count+1`、篡改触发事件）在本地**导入期**被接受、开局期由 Rust 校验器拒绝；服务端导入期即拒。导入期深层校验需 Worker 新增校验请求或主线程加载 WASM，属协议/产品决策——**须另行授权**（§6）。

## 5. 验证（2026-10-28 21:19–21:28，HEAD `5eec1e9e` + 本批工作树）

| 命令 | 结果 | 日志 |
|---|---|---|
| `pnpm --dir apps/web-client test` | **123/0/0**（116 + 新 7） | [web-test.log](web-test.log) |
| `pnpm --dir apps/web-client typecheck` | 0 错 | [web-typecheck.log](web-typecheck.log) |
| `pnpm --dir apps/web-client build:native` | exit 0（WASM 从本次源码重建，gitignored `src/worker/native-pkg/`） | [build-native.log](build-native.log) |
| `pnpm --dir apps/web-client smoke:native` | **72/0**（64 + 新 8，两次运行一致：[首跑](smoke-native-first-run.log)、[终跑](smoke-native.log)） | [smoke-native.log](smoke-native.log) |
| `cargo test --offline -p game-server --no-fail-fast` | **362/0/0**（360 + 新 2） | [game-server.log](game-server.log) |
| `cargo check --offline --workspace --all-targets` | exit 0（869 条既有警告；本批新文件 0 警告，新测试目标经 `#[allow(dead_code)]` 归零） | [workspace-check-tail.log](workspace-check-tail.log)、[gt014-targeted-recheck.log](gt014-targeted-recheck.log) |
| 行数门禁 | 单测 OK；**8656** 文件 ≤2000 行 | [source-line-tests.log](source-line-tests.log)、[source-lines.log](source-lines.log) |
| 浏览器产品路径（headless Chrome 141，`vite build` 独立 outDir `/tmp/gt014-dist`，未触碰已跟踪 dist/） | **12/12、0 异常** | [browser-vm-import-lifecycle.log](browser-vm-import-lifecycle.log)、[screenshot.png](screenshot.png)、[构建](vite-build.log) |

冒烟 GT-014 新增 8 检查：正式导入副本达开局、开局/身份局/存读档、升级保留历史与旧副本、**新局用新版本（Δ2）**、**旧档冻结旧版本（重放同哈希）**、旧档配新副本被 `RuleVersionMismatch` 拒、回退、停用后不在新局但旧档可恢复、卸载后旧档仍可从冻结副本恢复。浏览器 12 检查：空 store 起步 → UI 导入安装 → 启用入锁 → 选将 `vm_fixture` → 阶段结束 **4→5** → 存档含安装副本 → 读档重放同哈希 → UI 导入 v2 升级（历史/旧副本）→ 回退 → 三类拒绝（非法 JSON／`entry_pc` 越界／同版本冲突）后存储逐字不变 → 停用 → 卸载。

未重跑：engine-native、engine-native-wasm、engine-core、content-compiler、loader、rule-vm（本批未改这些 crate 的库代码；`emit_vm_fixture` 为 game-server example）。engine-native-wasm 的 VM-only 载荷形状由既有 `vm_package_worker.rs` 覆盖。

## 6. GT-014 映射与未覆盖项

| GT-014 要求 | 测试/证据 | 状态 |
|---|---|---|
| 作者新包经本地正式入口完成全生命周期 | `native-vm-install.test.mjs` 7 测、smoke 8 检查、浏览器 12 检查 | 已实现、定向+产品通过 |
| 不能只删拒绝检查而跳过校验 | `validateVmModules` + 冲突/撤销/冻结错误分支 + 浏览器三类拒绝不改写 | 定向通过 |
| 与服务端核对同包行为 | `gt014_vm_import_parity.rs` 2 测（§4 表） | 定向通过 |
| 升级／回退／停用／卸载不破坏旧局 | smoke + 浏览器（冻结副本恢复） | 定向+产品通过 |

未覆盖（交后续/裁决）：
1. **导入期深层校验**（模块/包 logic_hash、ABI 窗口）在客户端导入时不做，开局期由 Rust 校验器拒绝（§4 差异行）；补齐需 Worker 新增校验请求（协议改动）或主线程加载 WASM——**须另行授权**。
2. **签名信任链**：服务端 `import_extensions` 支持 `.sig.json`，本地导入入口不验签（客户端无密钥信任链）；第三方正式分发仍需服务端/签名路径（NC05）。
3. NC05「三类真实案例」只完成第一类（VM-only 导入）；混合包（cards+VM）、跨包依赖导入链、真实第三方作者产物（非测试夹具）未覆盖。
4. 浏览器验证的「选择文件」走粘贴路径等价替代；未做多浏览器/移动端。
5. 服务端 drop-in 与本地导入是两个正式入口（目录扫描 vs JSON 导入），行为已核对（§4），未做同一 HTTP 入口统一（t63 产品路径维持）。

结束时间见 [end-time.txt](end-time.txt)。
