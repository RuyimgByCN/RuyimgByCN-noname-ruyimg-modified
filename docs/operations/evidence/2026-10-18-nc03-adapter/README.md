# NC03 前端适配层切片：根因、修复与回归（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC03**（[统一原生产品 API](../../../architecture/native-convergence-plan.md#nc03)）的**前端口切片**
- 反例来源：[2026-10-18 架构审查](../2026-10-18-architecture-audit/README.md)「原生前端转接层」（`adapter-probe.log`）与其中两条门禁（`typecheck.log`、`native-smoke.log`）
- 范围：只改 `apps/web-client`（适配层、冒烟、测试）与进度文档；**`crates/` 零改动**，不改 Rust 内核、不改规则基准与两仓台账
- 版本：规则 **v321**、存档 **v105**（均未变）；无 golden 重录
- 环境：`node v26.4.0`、`pnpm 10.34.3`、`vue-tsc`/`typescript ^5.7.3`（web-client 既有 devDependency）；命令工作目录 `noname-next/`
- 结论：**三个反例已修 + 正式回归已加 + 定向冒烟/测试/构建通过**；NC03 的其余面（内核权威动作查询、协议定义共享等）仍未完成，见 §6

## 1 改动文件

| 文件 | 动作 |
|---|---|
| [nativeBackend.ts](../../../../apps/web-client/src/worker/nativeBackend.ts) | 修三个反例：token 与候选值分离、`kind` 候选类型补齐、命令结果进入 `envelope.result` |
| [native-worker-smoke.mjs](../../../../apps/web-client/scripts/native-worker-smoke.mjs) | 目录数守护改为按包数据推导；新增真实 WASM 引擎的适配层 token 往返守护 |
| [extension-inventory.mjs](../../../../apps/web-client/scripts/extension-inventory.mjs) | 新增：包清单与客户端目录一致性的判定逻辑（供冒烟与回归共用） |
| [native-adapter.test.mjs](../../../../apps/web-client/scripts/native-adapter.test.mjs) | 新增：三个反例的正式回归（7 项，`node:test`） |
| [extension-inventory.test.mjs](../../../../apps/web-client/scripts/extension-inventory.test.mjs) | 新增：目录守护的失败路径回归（3 项，证明守护非恒真） |
| [ts-hook.mjs](../../../../apps/web-client/scripts/ts-hook.mjs)、[ts-register.mjs](../../../../apps/web-client/scripts/ts-register.mjs) | 新增：Node 装载钩子，让测试/冒烟直接 import `src` 下的 TS（CI 的 Node 20 无内建 type stripping） |
| [package.json](../../../../apps/web-client/package.json) | 新增 `test`；`smoke:native` 加 `--import` 注册钩子 |
| [native-progress.md](../../../architecture/native-progress.md) | NC03 行改为已修切片 + 剩余项；新增 NC03 核验小节 |

本轮**未**改动：[extensionCatalog.ts](../../../../apps/web-client/src/engine/extensionCatalog.ts)（客户端 15 项目录仍是产品事实，见 §5）、`docs/migration/upstream-extension-reconstruction.md`、规则基准、`crates/**`。

## 2 根因与修复（逐反例）

### 2.1 token 与候选值混用（反例 1）

- 旧行为：`readChoice` 的 token 对 `{card:42}` 这类载荷候选取**下标**（0），对裸数字候选取**值**；而 `resolveChoice` 一律按候选值 `findIndex`。于是候选值 42/99 时暴露 token 0/1，提交 token 0 会被当作值 0 查找 → `native: 选项值 0 不在候选`。同时 `deriveLegalActions` 合成的 `selections` 用的是候选值（`[42]`），与 token 语义相反，两个出口互相矛盾。
- 契约依据：窄壳 `answer()` 直取 `spec.options[idx]`，**回传面是索引**。当时位于 lib.rs 第268–287行；2026-10-30共享session抽取后，绑定仍委托同一[NativeSession::answer](../../../../crates/engine-native-wasm/src/session.rs)，本页测试结果仍仅属于2026-10-18快照。
- 修复：token **统一为候选下标**（`options.map((o, index) => ({ token: index, … }))`），候选值只进 `label_key`；`deriveLegalActions` 的 `selections` 改用首候选 token `[0]`；`resolveChoice` 按 token 直传索引，越界/非整数显式抛错（`native: 选项 token X 不在候选（0..N-1）`），不再静默命中别的候选。
- 验证：stub 回归 4 项 + 真实引擎（`discard_to_limit`，`{card:113}`）token 0 → `accepted`，提交载荷值 113 → 被拒（§4）。

### 2.2 引擎 `rejected` 被丢弃（反例 2）

- 旧行为：`next_phase`/`play_card`/`answer`/`auto_step` 的返回值被直接丢弃，`nativeEnvelope` 里 `result: null` 是常量 → 引擎拒绝对界面不可见。
- 修复：新增 `parseCommandResult`（`outcome_json` 或 autoStep 步骤摘要）与 `commandOutcome`（取 `accepted`/`pending`/`rejected`/`suspended`），四个命令分支把解析结果原样放进 `EngineEnvelope.result`；`observer`/`save`/`status` 无命令返回时仍为 `null`（不臆造）。
- 验证：stub 回归断言 `result.outcome === "rejected"`（resolveChoice 与 playCard 两路）；旧代码同一场景实测 `result is null`（§4 探针）。

### 2.3 `ChoiceOption Kind(105)` 被归零 + `typecheck` exit 2（反例 3）

- 旧行为：`optionValue` 的参数类型与实现都不含 `kind`，`{kind:105}` 落到 `?? 0` → 标注 `"0"`；同一处类型遗漏让 `nativeBackend.ts(198,56)` 触发 `TS2345`，`pnpm --dir apps/web-client typecheck` exit 2。
- 修复：候选形状集中为 `ChoiceOptionJson`/`ChoiceOptionPayload`（含 `kind`），`optionValue` 收 `kind`，新增 `optionLabel` 输出带字段名的标注（`card:42`/`player:3`/`tag:7`/`kind:105`/`confirm`/`pass`），未知形状标 `unknown` 而不是退化成 0。
- 验证：`typecheck` exit 0；回归断言 `optionValue({kind:105}) === 105`、`label_key === "kind:105"`。

## 3 冒烟守护去硬编码（`extension_catalog_15`）

- 旧断言：`dirs.length === 15`，硬编码 2026-10-10 的「15 个转换包」，实测在盘包 **41**，故 exit 1。
- 新判定（[extension-inventory.mjs](../../../../apps/web-client/scripts/extension-inventory.mjs)，期望值全部由数据推导）：
  1. `extension_package_inventory`：逐目录读 `<dir>/package.npack.json`，要求可解析且 manifest 含 `namespace`/`package_id`/`version`，且 `(namespace, package_id)` 包键唯一；缺 npack 的目录只计数不上判（详情里给出 `no_npack`）。
  2. `extension_catalog_consistent`：客户端可发现目录 `EXTENSION_CATALOG`（**直接 import** `src/engine/extensionCatalog.ts`，不是正则复制）的每一项都能回查到在盘包，且 namespace/包 ID/版本逐字段一致；未进目录的在盘包只计数（`uncovered`）。
- 为什么不是恒真：`extension-inventory.test.mjs` 用临时目录 fixture 断言坏 JSON、缺字段、重复包键、目录项与包元数据不一致、目录项无对应包这五类输入**都会失败**；本轮只统计不失败的 `uncovered` 是产品面差额（见 §5），不是隐藏失败。

## 4 实测

| 命令 | 修复前 | 修复后 | 日志 |
|---|---|---|---|
| `pnpm --dir apps/web-client typecheck` | **exit 2**：`nativeBackend.ts(198,56)` TS2345（`{ kind: number }` 不符合 `optionValue` 参数） | **exit 0** | [typecheck-before.log](typecheck-before.log) / [typecheck.log](typecheck.log) |
| `pnpm --dir apps/web-client smoke:native` | **exit 1**：13 项 12 通过，`extension_catalog_15` 实际 41 | **exit 0：15/15** | [native-smoke-before.log](native-smoke-before.log) / [native-smoke.log](native-smoke.log) |
| `pnpm --dir apps/web-client test` | 无此命令（web-client 无测试框架） | **exit 0：10/10**（适配层 7 + 清单 3） | [adapter-tests.log](adapter-tests.log) |
| `pnpm --dir apps/web-client build` | — | **exit 0**（`vue-tsc --noEmit` + `vite build`，65 模块） | [build.log](build.log) |
| 反例探针（改动前源码，见 [probe-before-adapter.mjs](probe-before-adapter.mjs)） | **3/3 REPRODUCED** | 同探针在修复后源码上不再适用（回归由 `pnpm test` 承担） | [adapter-repro-before.log](adapter-repro-before.log) |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` + `python3 -B scripts/check-source-lines.py` | 8215 文件（审查时） | **1/1 单测通过；8222 文件 ≤ 2000 行** | [source-lines.log](source-lines.log) |

冒烟新增项实测明细（真实 WASM 引擎 + 真实适配层）：

```
PASS extension_package_inventory | packs=41 dirs=41 no_npack=0
PASS extension_catalog_consistent | catalog=15 uncovered=26
PASS adapter_choice_token_roundtrip | choice=1 options=5 label=card:113 token=0 submission=[0] outcome=accepted value_as_token_refused=true
```

修复前探针输出（`adapter-repro-before.log`，逐字）：

```
REPRODUCED: emitted card choice token 0 (label 42) cannot be consumed for card 42; legal action sends [42]: Error: native: 选项值 0 不在候选
REPRODUCED: engine outcome=rejected discarded; returned envelope.result is null
REPRODUCED: ChoiceOption Kind(105) becomes label 0
```

复跑方式：

```bash
cd noname-next
pnpm --dir apps/web-client typecheck
pnpm --dir apps/web-client smoke:native
pnpm --dir apps/web-client test
# 反例探针（可选）：git show <修复前提交>:apps/web-client/src/worker/nativeBackend.ts \
#   > apps/web-client/src/worker/nativeBackend.before.ts
#   node --import ./apps/web-client/scripts/ts-register.mjs \
#     docs/operations/evidence/2026-10-18-nc03-adapter/probe-before-adapter.mjs
```

## 5 口径与限制

- **token 语义收紧是接口变更**：`ChoiceEnvelope.options[].token` 由「载荷候选下标 / 裸数字候选值」的混合口径统一为**唯一下标**（裸数字与字符串候选的 token 也随之变为下标）；载荷候选的 `label_key` 由裸数字改为带字段名（`card:42` 等，裸数字/`confirm`/`pass` 保持原文字面）。当前仓库内除测试外没有消费 `choice.options[].token` / `label_key` 的界面代码（`grep` 只命中类型声明与适配层自身），因此无迁移；后续 NC03 固化协议时应把这条写进共享定义。若外部按值回传，现在会**显式报错**，不会静默错选。
- `pnpm test` 用 `node:test`（web-client 原本没有测试框架）：`.mjs` 测试通过 [ts-hook.mjs](../../../../apps/web-client/scripts/ts-hook.mjs) 直接 import `src` 下的 **TS 生产源码**，不复制一份 JS 复刻件；钩子用既有 `typescript` devDependency 逐文件转译，Node ≥20.6 可用（CI 的 Node 20 没有内建 type stripping，故未用 `--experimental-strip-types`）。
- 冒烟与适配层测试都使用**既有 WASM 产物** `src/worker/native-pkg`，本轮未重建 Rust、未启动浏览器；因此「真实引擎 token 往返」证明的是**当前产物 + 当前适配层**一致，不证明当前 Rust 源码已重新发布 WASM。
- 客户端可发现目录仍是 **15 项**，在盘包 **41**（`uncovered=26`）。本轮只把差额显式计量，未把 26 个包补进管理面：那需要逐包标题/指纹等元数据与产品入口，属 NC03/NC05 后续面。
- 未接入 CI：`.github/workflows/ci.yml` 正在被并行工作修改，本轮不动；`pnpm test` 本身无浏览器、无网络、无外部服务，接入只需在 gates job 增加 `pnpm install` + 本命令。
- `pnpm build` 会改写**已被 git 跟踪**的 `apps/web-client/dist/`（旧哈希产物被删、`index.html` 改写）。按「只提交自己的源码/文档改动」，本轮把这批构建产物恢复为 HEAD 状态、未纳入提交；是否更新 `dist/` 应由产品发布流程决定。

## 6 未完成 / 剩余项（NC03 整体）

- [ ] 内核**权威动作查询**：`deriveLegalActions` 仍在客户端用 `hand.slice(0,8)` 拼接 `PlayCard`、按阶段合成 `EndPhase`/`ResolveChoice`，UI 与 AI 各写一套规则的问题未解。
- [ ] Rust↔TS **协议定义共享或自动校验**（Accepted/Pending/Rejected/Fault、修订号、选择身份、稳定 token、大整数表示）。
- [ ] `events: []`、`hash_hex: ""` 仍是空占位，未接真实事件流与状态哈希。
- [ ] 穿过真实 Worker/浏览器的端到端验收：多选、取消、过期选择、错误不污染状态、正式安装扩展开局与同锁恢复。
- [ ] 客户端管理面覆盖 41/41 在盘包（现 15/41），含启用/升级/回退/卸载产品入口。
- [ ] 把 `pnpm test` 接入 CI 门禁（依赖本轮之外的 `ci.yml` 归属）。

## 7 工件清单

| 工件 | 说明 |
|---|---|
| [typecheck-before.log](typecheck-before.log) / [native-smoke-before.log](native-smoke-before.log) | 修复前原始失败输出（exit 2 / exit 1） |
| [typecheck.log](typecheck.log) / [native-smoke.log](native-smoke.log) / [adapter-tests.log](adapter-tests.log) / [build.log](build.log) / [source-lines.log](source-lines.log) | 修复后各命令完整输出与退出码 |
| [adapter-repro-before.log](adapter-repro-before.log) | 改动前源码的三反例探针输出（3/3 REPRODUCED） |
| [probe-before-adapter.mjs](probe-before-adapter.mjs) | 该探针源码（含复跑步骤；需临时导出修复前的 `nativeBackend.before.ts`） |
| [SHA256SUMS](SHA256SUMS) | 本目录工件的 SHA-256（不含清单自身），`shasum -a 256 -c SHA256SUMS` 可核验 |

本目录只证明上述命令在其执行时刻的输出；不证明整包验收、上游规则等价、平台验收或浏览器产品面。
