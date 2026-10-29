# 2026-10-29 GT-014：导入期深层校验（Worker `validatePackage` + 未签名标记）

范围：[裁决登记](../2026-10-28-user-decisions/README.md)第 12 项——授权新增主线程与 Worker 之间的包校验请求，复用 Rust 校验器在**导入时**完成模块／包 logic_hash 与 ABI 窗口校验，使本地导入与服务端导入在导入期的拒绝行为一致；**不做本地验签**（暂缓，先定客户端信任模型），UI 把本地导入包标为未签名。NC02／NC05／GT-014。**规则 v326、存档 v105、ABI 753、WS 协议均不变**（Worker 消息类型新增属本项授权）。开始 [start-time.txt](start-time.txt)，结束 [end-time.txt](end-time.txt)，源码基准 HEAD `95f51867`（[source-head.txt](source-head.txt)，B8 与 v326 批同工作树之上）。

## 1. 反例（GT-014 证据 §4 已登记的差异）

同一夹具 `vm-fixture@0.1.0`、同一变异集下：`local_count+1`（破坏模块指纹）与篡改触发事件名两类变异此前在本地**导入期被接受**、开局时才被 Rust 校验器拒绝；服务端导入期即拒。本批把拒绝提前到导入期。

## 2. 设计与改动

| 层 | 改动 |
|---|---|
| `engine-native-wasm/src/installed_packs.rs` | 新增 `validate_pack_json(pack_json)`：对单个安装载荷执行与 `packs_from_json` 单项分派完全相同的逐包校验（manifest 身份、`ContentPackage::validate()`＝包/模块 logic_hash 与结构、`bridge_vm_skill_decls`＝rule-vm 装载期 verify + 宿主 ABI 窗口、cards/characters/pile 桥接），只校验不建局；跨包依赖解析仍在开局 |
| `engine-native-wasm/src/lib.rs` | `#[wasm_bindgen] pub fn validate_pack_json` 导出 |
| `types/engine.ts` | `WorkerRequest` 新增 `{ type: "validatePackage"; packJson }`（不需对局句柄） |
| `engine.native.worker.ts` + `nativeBackend.ts` | `validatePackage` 在句柄检查前分派到 `nativeValidatePack`；拒绝经既有 catch 以 `ok:false` 带 Rust 原始错因回包 |
| `engine/client.ts` | `EngineClient.validatePackage(packJson): Promise<void>` |
| `engine/extensionStore.ts` | 导入拆为 `prepareImport`（结构层+前置条件）→（深层校验）→ `commitImport`（重读存储复核前置条件后提交）；新增 `importExtensionPackageValidated(json, deepValidate)`；`InstalledExtensionRecord.signature: "unsigned"`（仅本地导入记录）；`importExtensionPackage` 保留为结构层入口（既有测试/烟测用） |
| `views/ExtensionsView.vue` | 「导入」按钮走 `importExtensionPackageValidated`（按需连接校验 Worker，页面内复用；进行中显示「校验中…」）；行标题加「未签名」徽标；说明文案补深层校验与未签名口径 |
| `scripts/native-worker-smoke.mjs` | 新增 9 项 `worker_validate_package_*` 检查 |
| `scripts/native-vm-install.test.mjs` | 新增 4 测：深层通过才提交并标未签名；深层拒绝不改写存储且不留版本副本；结构层/前置拒绝不触发深层校验；异步校验期间冻结的提交期复核 |

替代方案：主线程直接加载 WASM 校验（不取——大体积 WASM 上主线程，且裁决明确走 Worker 请求）；在 TS 重写规范序列化与哈希（明确禁止）；提交后开局再拒（现状，即本批要消除的差异）。

## 3. 与服务端导入核对（同夹具同变异集）

[wasm-validator-parity.log](wasm-validator-parity.log)（`GT014_WASM_REJECT`）对 [game-server-parity.log](game-server-parity.log)（`GT014_SERVER_REJECT`）逐条对照：六类共享变异全部两端拒绝、错因同族（`logic_hash mismatch`／`entry_pc out of range`／身份字段无效；仅前缀 `npack:` vs `package:` 与身份错误措辞属不同层文案）。WASM 侧另验 ABI 窗口：真实 onlyOL 0.1.5（要求 758）导入期即拒 `rule_vm_abi host=753 package=758`。

## 4. 验证（2026-10-29 00:50–01:10，HEAD `95f51867` + 本批工作树，规则 v326／存档 v105／ABI 753）

| 命令 | 结果 | 日志 |
|---|---|---|
| `cargo test -p engine-native-wasm --no-fail-fast` | **71/0/0**（基线 68+3：`gt014_import_validation_parity` 合法通过/六变异拒/ABI 窗口拒） | [engine-native-wasm.log](engine-native-wasm.log) |
| `cargo test -p game-server --no-fail-fast` | **362/0/0**（含 `gt014_vm_import_parity` 2/0） | [game-server.log](game-server.log) |
| `cargo check --workspace --all-targets` | exit 0，0 错 | [workspace-check.log](workspace-check.log) |
| Gate 1b（`-D warnings`） | exit 0 | [gate1b.log](gate1b.log) |
| `build:native`（WASM 从本源码重建，含新导出） | exit 0 | [build-native.log](build-native.log) |
| `smoke:native`（真实 Worker） | **81/0**（72+9：`validatePackage` 六变异+ABI+合法+不触碰当前局） | [smoke-native.log](smoke-native.log) |
| `pnpm --dir apps/web-client test` | **127/0**（123+4） | [web-test.log](web-test.log) |
| `pnpm --dir apps/web-client typecheck` | 0 错 | [web-typecheck.log](web-typecheck.log) |
| headless Chrome 产品路径（vite 构建到**临时 outDir** `/tmp/nn-gt014-dist`，不触碰跟踪的 `dist/`） | **4/4**：两类指纹变异经 UI 导入即拒（alert 显示 Rust 错因、localStorage 逐字不变）；合法导入行标题含「未签名」、状态「已安装」、按钮不滞留「校验中…」；浏览器异常 0 | [vite-build.log](vite-build.log)、[browser-product-path.log](browser-product-path.log)、[screenshot.png](screenshot.png) |
| 行数门禁 | **8658** 文件 ≤2000 行 | [source-lines.log](source-lines.log) |

产品路径驱动 [browser-import-deep-validation.mjs](browser-import-deep-validation.mjs)（复用 10-28 GT-014 驱动的 CDP 骨架）。

## 5. 兼容性与未覆盖项

- Worker／WS 消息：新增 `validatePackage` 请求类型（裁决第 12 项授权）；旧线 Worker 收到该类型按既有 `unknown message` 拒绝。规则／存档／ABI／golden 不变。
- 存量影响：此前已导入的记录无 `signature` 字段（可选字段，UI 不显示徽标）；`importExtensionPackage` 结构层入口行为不变（现有测试全部通过）。
- **本地验签未做**（按裁决暂缓：先定客户端信任模型——密钥持有、分发与吊销）；导入期校验为单包（跨包依赖解析仍在开局）；混合包与真实第三方作者产物仍属 NC05。
