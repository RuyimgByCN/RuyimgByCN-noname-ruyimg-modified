# NC03 前端接线：合法动作改用内核权威查询（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC03**（[方案 §NC03](../../../architecture/native-convergence-plan.md#nc03)）前端接线切片
- 基线 HEAD：`3ee01345`（内核权威查询 `c30bc057` 及其后续）
- 规则版本 v321 / 存档 v105（**无规则与格式变化**：本切片只改「候选从哪来」）
- 结论：**前端已接权威查询 + Rust 导出面 5 项回归 + 前端 5 项回归 + 真实 WASM 冒烟 22/22 + cargo 全绿**；
  协议共享、`events`/`hash_hex`、AI 接线、候选分页仍待续

## 1 缺陷与复现（修复前）

前端自己拼候选，合法性定义散落在内核之外（`apps/web-client/src/worker/nativeBackend.ts@3ee01345`）：

```
$ git show 3ee01345:apps/web-client/src/worker/nativeBackend.ts | grep -n 'hand.slice\|deriveLegalActions'
259:function deriveLegalActions(
273:    for (const card of hand.slice(0, 8)) {     # 出牌候选按手牌前 8 张截断
282:        ResolveChoice: {                        # 开放选择只合成 1 个应答（恒取 options[0]）
340:    legal_actions: deriveLegalActions(handle, view, finished),
```

复现探针 `probe-before-authoritative.mjs`（桩窄壳，12 张手牌 + 3 候选开放选择）实测：

| 反例 | 实测输出 | 证据 |
|---|---|---|
| 12 张手牌 → 只有 8 个 `PlayCard` 候选（第 9–12 张永远点不到） | `play=8 cards=100,…,107` | [repro-before.log](repro-before.log) |
| 适配层从不调用权威查询 | `calls=["open_choice_json","open_choice_json"]` | 同上 |
| 3 个开放选择候选只合成 1 个 `ResolveChoice`（+1 取消） | `selections=[0]`、`cancelled=true` | 同上 |
| 从不产出 `ActivateSkill` 候选（主动技按钮恒空） | `kinds=["EndPhase","PlayCard"×8]` | 同上 |

探针是**修复前**的历史证据：接线后适配层要求桩提供权威查询导出，用同一桩重跑会显式报错（预期行为，见 §2 ③）。

## 2 接线实现

### ① 内核导出（`crates/engine-native-wasm/src/lib.rs`，1542 行）

WASM 导出面此前**没有** `legal_actions`（只有 view/choice/next_phase/play_card/answer/auto_step/save/restore），
按仓库既有导出方式补上：

| 导出 | 语义 |
|---|---|
| `NativeEngine::legal_actions_json(seat) -> Result<String, JsValue>` | `{seat, revision, actions}`；`actions` 逐项与 TS `LegalAction` 同形；座位越界报错 |
| `NativeEngine::activate_skill(actor, skill) -> Result<String, JsValue>` | 主动技发动（`skill` 取自权威候选）；内核 `GameAction::ActivateSkill` 只有技能 id，故窄壳不接目标 |
| `legal_actions_json_for(engine, seat)`（非 `wasm_bindgen`） | 宿主测试直接调用，避免走到 `JsValue` |
| `action_to_json`（私有投影） | `NextPhase→"EndPhase"`、`PlayCard`、`ActivateSkill`、`Answer→ResolveChoice`（**候选值按下标回投影**，与 `answer()` 直取 `spec.options[idx]` 同口径） |

同轮把 `outcome_json` 补上 `reason`（`RejectReason` 调试名，非 rejected 为 `null`）：伪造卡 id／越权座位
不再只是一个无因的 `rejected`。

### ② 前端适配层（`apps/web-client/src/worker/nativeBackend.ts`，586 行）

- 删除 `deriveLegalActions`（连同 `hand.slice(0, 8)` 与合成 `ResolveChoice`）；
- 新增 `authoritativeLegalActions(handle)`：调 `legal_actions_json(handle.observer)`，逐项走
  `parseLegalActionsJson` 结构校验（非 JSON／字段不符／未知动作形态一律报错）；
- 新增 `activateSkill` 命令面（`NativeDispatchReq` + Worker 分支）；传预选目标显式报错（目标由内核开放选择决定）；
- ③ 旧 wasm 产物（未重建 `native-pkg`）缺导出时**显式报错并给出 `build:native` 命令**，不静默回落到手牌拼接。

### ③ 缓存／性能策略

查询一次 = 内核 `Engine::legal_actions`（克隆校验） + JSON 投影 + JS↔wasm 边界。前端策略：

1. **按需触发**：只在构建信封时查询（`start`/命令/`observer`/`autoStep` 等用户动作驱动），不在渲染循环里逐帧调用；
2. **按 `(handle, revision, seat)` 记忆化**（`WeakMap`）：同一修订号重复构建信封不重复查询；修订号只在命令被接受时前进，
   被拒绝的命令不改变候选，故缓存安全；
3. **读档失效**：`restore` 会把引擎换成另一份状态（修订号可能相同），`nativeDispatch` 显式 `invalidateLegalActions`。

本机实测（Apple M3 Max，web release `--no-opt` 产物，5 人局、出牌阶段、6 候选）：

| 项 | 实测 |
|---|---|
| `legal_actions_json(0)` 单次 | **0.415 ms**（30 次均值） |
| 适配层信封（冷，含 1 次查询） | 0.558 ms |
| 适配层信封（同修订号，命中缓存） | **0.051 ms** |

内核侧上轮在 Rust 侧测得 1.62 ms/次（不同构建与调用面，不作跨口径比较）——[query-cost.log](query-cost.log)、
[内核查询证据](../2026-10-18-nc03-legal-actions/README.md)。

## 3 回归

### Rust（新增 `crates/engine-native-wasm/tests/nc03_legal_actions_export.rs`，5 项）

| 验收项 | 用例 | 结果 |
|---|---|---|
| 导出面存在且可调用（编译期取 `NativeEngine::legal_actions_json` / `activate_skill` 函数指针） | `export_surface_exists_and_returns_kernel_actions` | 通过 |
| 返回非空、`revision` 与该候选成立的状态一致、座位越界报错 | 同上 | 通过 |
| **候选逐个反向重建为内核动作并真实提交**（`Accepted`/`Pending`，绝不 `Rejected`） + 射程外目标不出现 | `every_projected_action_is_submittable` | 通过 |
| 候选不是手牌拼接：9 张【杀】全部出现（>8，无截断）；12 张【闪】时**没有**出牌候选 | `candidates_are_not_hand_padding_nor_truncated_at_eight` | 通过 |
| 开放选择候选的下标可回查候选值、逐个可提交、非观察者无候选 | `open_choice_candidates_round_trip_option_indices` | 通过 |
| 反向探针：伪造卡 id／越权座位被内核拒绝（`CardNotInHand`），候选不含伪造值 | `forged_actions_are_rejected_by_kernel` | 通过 |

### 前端（新增 `apps/web-client/scripts/native-legal-actions.test.mjs`，5 项；沿用 `node:test` + `scripts/ts-hook.mjs`）

| 验收项 | 用例 | 结果 |
|---|---|---|
| 候选逐项等于内核返回（12 张手牌 / 内核只给 3 张 → 不得补齐） | `候选逐项等于内核返回：不按 hand.slice(0,8) 拼接` | 通过 |
| 内核不给即不出现（play 阶段不补 `EndPhase`；只给取消时不补 `options[0]`；非观察者空） | `无合成：内核不给 EndPhase/应答候选就不出现` | 通过 |
| 查询按 `(handle, revision)` 记忆化，`restore` 后失效 | `查询按 (handle, revision) 记忆化，读档后失效` | 通过 |
| 旧产物缺导出／畸形返回显式报错（不回落） | `旧 wasm 产物缺导出／畸形返回 → 显式报错，不回落拼接` | 通过 |
| 候选逐项有命令面并可提交；伪造卡 id／越权座位被拒且 `reason` 可见 | `内核候选逐项可提交：命令面覆盖每种候选，伪造动作的拒绝原因可见` | 通过 |

上轮 `native-adapter.test.mjs` 的桩同步改为「扮演内核给候选」（6 项仍全绿）；冒烟新增 7 项真实 WASM 验收：
候选与适配层逐字节一致、抽样候选在同一保存点的新引擎上**逐条被接受**、伪造卡被拒且 `reason=CardNotInHand`、
座位越界报错。

## 4 验证结果（本机命令实测）

```
node apps/web-client/scripts/build-native.mjs          # wasm-pack 重建 native-pkg（release, web）
pnpm --dir apps/web-client typecheck
pnpm --dir apps/web-client test
pnpm --dir apps/web-client smoke:native
pnpm --dir apps/web-client build
cargo test --offline -p engine-native-wasm --no-fail-fast
cargo test --offline -p engine-native-wasm --test nc03_legal_actions_export
python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| WASM 重建（含新导出） | exit 0（42.7 s，wasm-pack 装 wasm-bindgen） | [build-native.log](build-native.log) |
| 前端 typecheck | exit 0 | [typecheck.log](typecheck.log) |
| 前端 `node:test` | **15 passed / 0 failed**（新增 5） | [web-tests.log](web-tests.log) |
| 真实 WASM 冒烟 | **22 PASS / 0 FAIL**（原 15 + 新 7） | [smoke-native.log](smoke-native.log) |
| 前端生产构建 | exit 0（`vite build` 721 ms） | [build-web.log](build-web.log) |
| 窄壳定向 | **5 passed / 0 failed** | [cargo-test-targeted.log](cargo-test-targeted.log) |
| `cargo test -p engine-native-wasm` | **25 passed / 0 failed**（lib 20 + 集成 5） | [cargo-test-engine-native-wasm.log](cargo-test-engine-native-wasm.log) |
| 行数门禁 | 8235 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 5 未完成 / 待办（NC03）

- [ ] **AI 接线**：`ai.rs::enumerate_candidates_variant` 仍镜像规则（克隆校验代价 → 需先有非克隆校验入口）；
- [ ] **协议共享**：Rust↔TS 动作 JSON 仍是手写两份（窄壳投影 + `LegalAction` 联合类型），未做自动校验/生成；
- [ ] `events` / `hash_hex` 仍为空占位；
- [ ] 候选空间上限 256 与目标组合上限（每牌最多 8 个单目标 + 1 个双目标）：超大局面需分页/分步接口；
- [ ] 主动技候选**未展开技能内部选择**：界面因此只能发动无目标技能，技能内的目标/选牌仍靠后续开放选择应答；
- [ ] `RevealGeneral`（国战明置）不在权威查询候选内，窄壳也不投影（不给客户端不可提交的动作）；
- [ ] 真实 Worker/浏览器端到端（当前冒烟在 Node 里跑 web 目标 pkg）、多选/取消/过期选择的浏览器路径；
- [ ] **构建管线未接 `native-pkg`**（本轮实测发现的既有缺口，非本切片引入）：`native-pkg` 为 gitignore 产物，
  而 `apps/web-client/scripts/build-desktop.mjs` 只跑 `build:wasm`（旧线）+ `build`，`apps/desktop-client/tauri.conf.json`
  的 `beforeBuildCommand` 还是 `pnpm --dir web-client build:desktop`（相对 `apps/desktop-client` 无此目录）；
  干净检出上原生 worker 的 `./native-pkg/engine_native_wasm.js` 无法解析。本切片按既有手工 `build:native` 流程验证
  （[build-native.log](build-native.log)），管线补建（并把 `wasm-pack` 纳入 CI 工具链或给 `build-native.mjs` 加
  cargo+wasm-bindgen 回退）列为待办；
- [ ] 客户端管理面仍只覆盖 15/41 个在盘包（沿用上轮登记）。

## 6 证据清单

| 文件 | 内容 |
|---|---|
| [repro-before.log](repro-before.log) | 修复前反例实测（4 项 PASS = 缺陷成立） |
| [probe-before-authoritative.mjs](probe-before-authoritative.mjs) | 修复前探针脚本（历史证据，重跑预期报缺导出） |
| [query-cost.log](query-cost.log) / [probe-query-cost.mjs](probe-query-cost.mjs) | 权威查询与缓存命中代价实测 |
| [source-evidence.log](source-evidence.log) | 修复前源码行号依据 + 修复后接线点 |
| [build-native.log](build-native.log)、[build-web.log](build-web.log) | WASM 重建、前端生产构建 |
| [typecheck.log](typecheck.log)、[web-tests.log](web-tests.log)、[smoke-native.log](smoke-native.log) | 前端三门禁 |
| [cargo-test-targeted.log](cargo-test-targeted.log)、[cargo-test-engine-native-wasm.log](cargo-test-engine-native-wasm.log) | Rust 定向与包级全量 |
| [source-lines.log](source-lines.log) | 行数门禁 |
