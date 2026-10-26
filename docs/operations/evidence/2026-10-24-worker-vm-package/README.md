# Worker 安装完整 npack（VM 技能）第一阶段（2026-10-24）

核验基准 `7e28a93799a9c01794085e911a72a4faa5673883` 加未提交工作树；规则 **v322**、存档 **v105**。本批只改 **Worker 安装载荷解析**，不改规则/存档；是“完整 VM 包经正式安装入口进入 Worker”的第一段，**不是**全量 VM 包验收。

## 缺陷与修复

`engine-native-wasm/src/installed_packs.rs` 的 `installed_pack` 只认 `cards/characters/pile` 三个资产键：

- 完整 npack（带 `manifest` + `modules/trigger_index/skill_index`）会被判“缺少 cards/characters/pile 载荷”，Worker 无法安装服务端可执行的 VM-only 包。
- 即使放过载荷检查，也没有调用 `bridge_vm_skill_decls`，模块不会进入 `pack.vm_skills`，技能不会进入目录。

修复：

- 检测 `item.modules` 时，把整个 item 解析为 `content_schema::ContentPackage`，调用 `content_native_loader::bridge_vm_skill_decls`，把 `decls` 并入 `pack.vm_skills`。
- 允许 `modules` 作为独立合法载荷，不再要求 `cards/characters/pile`。
- `characters` 资产改为**独立于 cards** 桥接；新增 `bridge_generals_with_known(json, vm_names)`，把 VM 模块名视为已解析技能（不产生 host 行为），武将技能列表因此能引用 VM 技能。
- 服务端 `apps/game-server/src/native.rs` 同步改为 characters 独立桥接并使用已知 VM 名；vm-only / characters-only 包不再被静默丢掉武将。
- 保持包身份摘要（`installed:...:blake3(item)`）与混合包逻辑；`Cargo.toml` 增加 `content-schema` 依赖。
- `engine-native-wasm` 已用 `apps/web-client/scripts/build-native.mjs` 重建到真实 Worker 包（web 目标）。

## 验证

| 检查 | 结果 | 证据 |
|---|---|---|
| `cargo test -p engine-native-wasm` 完整 | **52 / 0 / 0**（10 target，含新增 3 项） | [日志](engine-native-wasm-full.log)、[计数](engine-native-wasm-counts.json)、[退出码](engine-native-wasm-full.exit) |
| `cargo test -p content-native-loader` 完整 | **32 / 0 / 0**（11 target） | [日志](loader-full.log)、[计数](loader-counts.json)、[退出码](loader-full.exit) |
| `cargo test -p game-server --test native_product_path` | **10 / 0 / 0**（含新增「npack + characters 服务端安装→选将→VM 技能」） | [日志](game-server-product.log)、[退出码](game-server-product.exit) |
| `cargo test -p game-server` 完整 | **330 / 0 / 0**（23 target） | [日志](game-server-full.log)、[计数](game-server-counts.json)、[退出码](game-server-full.exit) |
| Worker 真实 WASM 安装 VM npack + characters | 选将 `worker_vm_probe`，VM 技能使手牌 4→7；revision 6、64 hex hash | [探针](wasm-vm-package.mjs)、[日志](wasm-vm-package.log)、[退出码](wasm-vm-package.exit) |
| Worker smoke（重建后） | **47 / 47** | [日志](worker-smoke.log)、[退出码](worker-smoke.exit) |
| 前端 `pnpm test` / typecheck | **72 / 0**；typecheck exit 0 | [测试](frontend-test.log)、[typecheck](typecheck.log)、[退出码](typecheck.exit) |
| native/WASM 标准 + huanle 正式包轨迹 | 两段 `native_wasm_match=true`，exit 0 | [日志](cross-target.log)、[退出码](cross-target.exit) |

新增 Rust 测试（`crates/engine-native-wasm/tests/vm_package_worker.rs`）：

1. 用 `content-compiler` 编译一个 `phase_end` VM 包，序列化为 npack，并附 `characters` 资产；`engine_from_packs` 接受；技能 `worker_phase_end_phase_end` 进入目录，武将 `worker_vm_probe` 进入选将目录并被分配；推进到阶段结束后手牌增加，证明不只装载声明，VM 技能经武将真实执行。
2. **characters-only**（无 cards/modules）载荷仍可载入武将与 `builtin:护驾`，防止把「必须带 npack」当成新限制。
2. 只有 manifest、没有 `cards/characters/pile/modules` 的载荷仍被拒绝，防止放宽成“空包也接受”。

Node 探针使用本批新生成的 `worker-vm.npack.json`（1 个模块，`namespace=evidence`），真实加载 `apps/web-client/src/worker/native-pkg` 的 web 产物；这不是 host 包装层。

## 未完成与边界

- 本 fixture 现已带 `characters` 资产并完成选将/技能执行；但该资产是测试 JSON，不代表现有扩展包都能通过桥接。
- **完整 `shiji` npack 仍被拒绝**：桥接器对 `cards_given` 等事件名未映射（增量 3 严格面），Worker 不能加载全部现有 VM 包；需要逐事件扩展或包声明降级，不能把本 fixture 当全量。
- 安装/启停/升级/卸载/原锁恢复的正式产品流程未在本批重跑；本批覆盖 Worker 解析入口、真实 wasm 加载与服务端 `native_product_path` 定向回归。
- 未改规则/存档版本，未改 golden，未改人工 DSL。

## 2026-10-24 追加：版本升级与回滚

- `NativeContent` 新增 `activate_version(namespace, package_id, version)` 与 `rollback()`；导入新版本会自动激活，显式 activate 幂等，rollback 恢复上一 active 集。
- 新增 `native_upgrade_rollback`：v1 安装 → v2 导入自动激活为 0.2.0 → 显式 activate → rollback 回 0.1.0。
- `game-server` 完整见 [native-upgrade.log](native-upgrade.log)。
- 追加 `revoke_package(namespace, package_id, reason)`：撤销槽后内容不可装载、不可再激活；保留安装副本供审计（停用式卸载）。
- 追加 **物理卸载** `PackageStore::uninstall_slot` / `NativeContent::uninstall_package`：移除 active 引用、清空回滚历史、删除 `packages/<ns>__<id>` 全部版本；有依赖方时先失败、不改动；崩溃在 active 写入与删目录之间只留无害孤儿。新增 `physical_uninstall_removes_active_slot_and_versions`（卸载→不可装载/不列出→重新导入可恢复）。
- `content-runtime` **34/0/0**、`game-server` **350/0/0**。
- **第三方免重编译定向**：服务端 `vm_package_with_characters_installs_loads_and_runs` 与 Worker `vm_only_npack_loads_skills_and_generals` 均在测试时编译新包、经正式 npack 载荷装载并真实触发技能/选将；客户端未重新构建。日志见 [server](third-party-server.log)、[worker](third-party-worker.log)。新增 `uninstall_and_disable_reject_active_dependents_without_mutation`：有 active 依赖方时卸载/停用必须失败且保留安装副本；先卸依赖方后才能卸 base。追加**停用/启用**：`PackageStore::deactivate_slot` / `NativeContent::disable_package` 仅移除 active 引用、保留安装副本与回滚历史；`enable_package` 重新激活。新增 `disable_then_enable_keeps_installed_copy`。新增 `content_lock_rejects_restoring_after_package_upgrade`：v1 存档后升级 v2，旧档恢复必须因内容锁拒绝，不得静默按新包恢复。原锁**拒绝**已验收；原锁**恢复**（用冻结副本恢复旧局）仍待。新增正式 HTTP 生命周期入口（无鉴权、与现有 `/native/*` 同级）：`POST /native/packages/import`、`/activate`、`/disable`、`/uninstall`；底层生命周期行为由 `native_upgrade_rollback` 覆盖，路由为薄封装。正式 UI 已加：设置页「原生扩展包」面板可配置服务端 URL、刷新、扫描安装、激活、停用、卸载；调用上述正式 HTTP 入口。新增 `real_http_route_lists_and_disables_package`：在临时 TCP 端口启动真实 axum router，用原始 HTTP/1.1 请求验证 GET 列表 200 与 POST 停用 204；路由注册与状态码已验收。
- 浏览器 UI E2E 已做（mock API）：真实生产构建 + headless Chrome 点击，列出包并依次触发 disable/activate/import/uninstall；见 [ui-e2e/README.md](ui-e2e/README.md) 与截图。真实服务端 + 浏览器同链已跑：临时启动真实 game-server + 生产前端 + headless Chrome，列出 huanle-cards，依次 disable/activate/import/uninstall，卸载后列表为空；杀进程重启后包重新导入。详见 [ui-e2e-real/README.md](ui-e2e-real/README.md)。**完整 VM 武将包已跑同链**：`vm-fixture` 真实服务端导入、`/native/games` 选将并跑完、浏览器面板 disable/activate/import/uninstall；见 [ui-e2e-vm/README.md](ui-e2e-vm/README.md)。

## 2026-10-24 追加：冻结副本原锁恢复（既有能力核对）

- `native_frozen_packages::legacy_exact_reference_migrates_disabled_old_version_and_upgrade_leaves_original_copy` 已覆盖：旧精确引用迁移、升级后保留原副本、旧版本安装目录删除后仍用冻结副本恢复、错版拒绝。
- 本项不是新实现；从 NC02 待办移入已验收。正式 UI/HTTP 生命周期与第三方免重编译全流程仍是剩余项。
