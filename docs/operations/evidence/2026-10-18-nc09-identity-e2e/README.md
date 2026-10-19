# NC09 身份局端到端（**引擎侧**）：补齐登记项 `NC09-IDENTITY-E2E` 的一半（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC09**（[方案 §NC09](../../../architecture/native-convergence-plan.md#nc09)）
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**引擎侧 2 项通过**；**浏览器侧身份局端到端仍未完成**（如实分列）

## 1 背景（登记表原文）

> `NC09-IDENTITY-E2E`：终局胜负展示已接线（前端文案 44/0、Rust 映射 33/0），但端到端实测走的是
> **原型/单挑口径**（本地开局）。需以**身份局**开局跑到终局并检查胜负展示。

## 2 本轮完成（引擎侧）

新增 `crates/engine-native/tests/nc09_identity_end_to_end.rs`（2 项）：

| # | 判据 | 结果 |
|---|---|---|
| 1 | `Engine::new_identity(seed, 5)` 建局 → 身份局；确定性驱动 600 步（覆盖阶段/询问路径）→ 杀死主公 → 继续驱动 → **必须到达终局**，且结果属 `Winner::{LordSide, Rebels, Traitor}` | 通过 |
| 2 | 终局后**推进类命令被拒**（`MatchOver`），且**状态哈希不变** | 通过（同法先杀死主公以确保到达终局；已去除静默 SKIP 分支） |

**过程记录（诚实纪律）**：初版用例 2 用 `drive(4000)` 后**未到达终局**（被动驱动僵局），
代码里写的是 `SKIP` 返回；首份证据误写为「通过」→ **已更正**：用例 2 改为与用例 1 同法
（驱动 → 杀主公 → 驱动），并**删除静默 SKIP 分支**，改为 `assert!(outcome.is_some())`。
当前 [engine-side.txt](engine-side.txt) 中 **SKIP 行数 = 0**。

**口径说明（不夸大）**：
- 被动驱动（一律 Pass/确认）**不会**自然分出胜负 → 用 `debug_set_hp` + `debug_deal_damage`
  杀死主公把对局推到终局；本测试的对象是**终局结果判定与终局后稳定性**，
  **不是**「AI/玩家自然打完整局」（后者由 NC12 浏览器整局覆盖，口径为原型/单挑）。
- 座位数 5（身份局标准）；`Winner` 的其它变体（`Player`/`Faction`/`Versus`/`Doudizhu`）属其它模式。

## 2b 浏览器侧前置：WASM 身份局构造已加，前端接线**受阻于 wasm 包重建路径**

| 步骤 | 状态 |
|---|---|
| ① `NativeEngine::new_identity`（Rust/WASM 导出） | ✅ 已实现并提交（`f0118062`）；`engine-native-wasm` 定向全通过 |
| ② 前端接线（HomeView 模式选择 / `client.start` 的 `mode` / worker 分派 / `nativeBackend` 封装 / 协议类型） | ⚠️ **已写好但已回退**：补丁存于 [frontend-identity-wiring.patch](frontend-identity-wiring.patch)（153 行） |
| ③ wasm 包重建 | ❌ **未打通**：worker 从 `src/worker/native-pkg/engine_native_wasm.*` 导入，而
`pnpm --dir apps/web-client build:wasm`（`scripts/build-wasm.mjs`）输出到 **`src/wasm-pkg/engine_wasm.*`**（**不同包名/目录**）→ 重建后 `native-pkg` 仍**不含** `new_identity` → 类型检查报错 |

**处置（不破坏工作区）**：因 ② 依赖 ③，若把 ② 留在工作区会让 `vue-tsc --noEmit` **报错**
（`new_identity` 不存在）→ 故**把 ② 回退并把补丁存证**，待 ③ 打通后 `git apply` 该补丁即可。

**③ 的打通线索**：需确认 `native-pkg` 的生成方式（疑为另一条 wasm-pack 命令，`--out-dir
src/worker/native-pkg`，目标 crate `engine-native-wasm`）；`build-wasm.mjs` 的 `outDir`
指向 `src/wasm-pkg` 且产物名为 `engine_wasm.*`，与本包不一致。

## 3 未完成（**浏览器侧**身份局端到端）

- [ ] **打通 wasm 包重建路径**（见 §2b ③）→ 再 `git apply frontend-identity-wiring.patch`
- [ ] 真实构建产物 + WASM worker 下，以**身份局**开局跑到终局并检查胜负**展示**（前端文案）
- [ ] 与已完成的「浏览器整局（原型口径，319 步 / 3 局 3 胜）」并列记录，**不混记**

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[engine-side.txt](engine-side.txt) |
| 全量 engine-native | 见提交说明 | — |
| 行数门禁 | 见提交说明 | — |
