# NC05 选将 UI 面板接线（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC05/NC09**（[方案 §NC05](../../../architecture/native-convergence-plan.md#nc05)）的玩家选将界面切片
- 基线：`503ae703`（本轮改动前工作树 HEAD；适配层/Worker/Client 已由 `a1f60ed9`、`30d10e34` 落地）
- 规则版本 v321 / 存档 v105 未变；**本轮不改 Rust/WASM 导出面，未重建 `native-pkg`**（真实 WASM 用例直接使用工作树已有产物）
- 结论：**玩家界面已能显示候选（名称+技能）并点选**；成功应用刷新信封，失败显示错误文本。浏览器端到端与视觉验收仍未做，见 §5。

## 1 缺口复现（改动前，`repro-before.log`）

采集于改动前的工作树 HEAD `503ae703`。玩家界面是 Vue SFC（`views/` + `components/battle/`），
不是纯脚本渲染；对 `views/` 与 `components/` 的 5 个关键词 grep **全部 0 匹配**：

| 位置 | 命令 | 结果 |
| --- | --- | --- |
| 视图/组件是否引用候选 | `git grep -n 'generalCandidates' HEAD -- apps/web-client/src/views apps/web-client/src/components` | 0（exit 1） |
| 视图/组件是否调用选将 | `git grep -n 'chooseGeneral' HEAD -- ...` | 0（exit 1） |
| 是否出现候选类型 | `git grep -n 'GeneralCandidate' HEAD -- ...` | 0（exit 1） |
| 是否直连 WASM 导出 | `git grep -n 'general_candidates\|assign_general' HEAD -- ...` | 0（exit 1） |
| 全前端有引用的文件 | `git grep -l 'generalCandidates\|chooseGeneral' HEAD -- apps/web-client/src` | 仅 `engine/client.ts`、`types/engine.ts`、`worker/engine.native.worker.ts`、`worker/nativeBackend.ts` |

即：适配层/Worker/Client 已能选将，但玩家界面没有候选列表、没有 `chooseGeneral` 调用、
没有选将面板；座位卡片只消费数字 `character_def` 与技能对象（`types/engine.ts`），无法从界面选将。

## 2 已实现（本轮）

### 2.1 纯视图模型 / 提交控制器

新增 `src/engine/generalSelectModel.ts`（408 行，无 Vue/DOM 依赖）：

- `candidateRowsFromPayload`：严格校验 `{count, candidates}`（count 非负整数、逐项
  `def/name/skills`、`def` 不重复、count 与候选数一致），畸形即抛错，**不静默过滤成更小的
  候选集**；每项映射为 `{def, name, skills, skillLabel}`（无技能显示「无技能」）。
- `applyCandidatePayload`：成功 → `ready`/`empty`；畸形 → `error` 且清空列表。
  **空列表是合法状态**（面板明确显示「候选为空」），与「加载失败」区分。
- `failCandidateLoad`：保留 Worker 原始错误文本（如旧产物缺导出 + `build:native` 提示）。
- `seatProjectionFromView`：从刷新信封取目标座位的 `character_def`、技能 id 与
  **技能名**（原生窄壳按 `SkillView.name` 投影）；不伪造缺失字段。
- `submitGeneralSelection`：点击后的完整动作 —
  1. 同步进入 `pending`（`onPending` 回调，界面立即禁用重复点击）；
  2. 调用 `chooseGeneral(seat, name)`；
  3. 回包自证 `assignment.seat/name` 回显请求值、有 `state_hash`、有刷新信封 `view`；
  4. 成功后调用 `applyEnvelope(envelope)` 刷新主视图，并按新视图更新技能投影；
  5. 任一失败 → `selectionStatus: "rejected"` + 可显示错误，**保留点击前投影且不调用
     `applyEnvelope`**。
- 装配不推进 `revision`（内核事实），成功态额外记录 `state_hash`，界面展示它。
- 边界：Worker 适配层已有权威校验；这里再做一次渲染边界校验，防止旧 Worker/混装产物/
  手写桩把畸形数据送进模板。

### 2.2 选将面板组件

新增 `src/components/battle/GeneralSelectPanel.vue`（178 行）：

- 渲染候选按钮：`名称` + `技能`（`skillLabel`）+ `#def`；点击 `emit("choose", name)`。
- 状态展示：加载中、空候选、候选错误（`role="alert"`）、提交中（按钮禁用）。
- 成功展示：`已为 P{seat} 装配 {name}` + `state_hash` 前缀 + **刷新后视图技能**。
- 最近一次成功装配的候选高亮（`aria-pressed`）；「刷新候选」按钮重拉目录。
- 无新增依赖，沿用既有 CSS 变量与 `panel` 风格。

### 2.3 BattleView 接线

`src/views/BattleView.vue`（1000 → 1081 行，+81）：

- 本地原生路径 `connectAndStart()`：座位夹取后用 `setGeneralSelectSeat` 对齐面板座位，
  `start` 成功后拉取 `generalCandidates()`；失败留在面板里显示，不静默。
- 面板仅在 `!isOnline && store.connected && localClient.isNative()` 时显示；legacy
  `VITE_ENGINE_BACKEND=legacy` 回退路径不渲染原生选将面板。
- 点击处理：进入 pending → `localClient.chooseGeneral(seat, name)` → 成功
  `applyEnvelope(store, envelope)` + 事件日志 + 状态行；失败状态行也显示错误文本。
- 每个信封（开局/动作/读档）后 `syncGeneralSelectProjection()`，面板技能投影跟随主视图。
- 新对局开局与读档调用 `setGeneralSelectSeat` 重置最近装配/投影（同座位也重置）：
  引擎重建后不把上一局的「已装配」成功态留在界面上。
- `types/engine.ts` 的 `SkillView` 增加可选 `name?: string`（原生视图已有该字段；旧线可缺）。

## 3 测试与门禁

| 命令 | 结果 | 日志 |
| --- | --- | --- |
| `pnpm --dir apps/web-client typecheck` | exit 0（无输出） | [typecheck.log](typecheck.log) |
| `pnpm --dir apps/web-client test` | **41/41 通过、0 失败**（新增 14 项） | [test.log](test.log) |
| `pnpm --dir apps/web-client smoke:native` | **32/32 PASS、0 failed** | [smoke.log](smoke.log) |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | OK（Ran 1 test） | [line-gate-unittest.log](line-gate-unittest.log) |
| `python3 -B scripts/check-source-lines.py` | `8266 files <= 2000 lines`，exit 0 | [line-gate.log](line-gate.log) |

新增 `apps/web-client/scripts/general-select-ui.test.mjs`（398 行，14 项）：

1. 候选回包 → 渲染行（名称/技能/顺序，且不改写输入）；
2. 空列表 → 显式 `empty`，不是错误；
3. 畸形数据 12 类（null/数组/缺字段/count 类型与负值/不一致/项 null/def 负或非整数/
   name 空白/skills 非字符串数组/def 重复）→ 每类都 `error`、清空列表、显示错误；
4. 候选加载失败保留原始错误与 `build:native` 提示；
5. 重新加载保留旧候选并清错误；
6. 座位投影只取目标座位，提取技能名与 id；
7. 绑定新对局座位（换座位与同座位）清掉最近装配与投影、保留候选目录；
8. **点击成功**：`chooseGeneral(2, "甲")` 恰调用一次；先进入 pending；成功后应用
   `payload.envelope` 一次，技能投影变为 `["鬼道","天妒"]`；
9. **点击失败**：显示 `assign_general 被拒绝: NoSuchGeneral`；`applyEnvelope` 0 次；
   `assignment/lastStateHash` 为空、技能投影保持点击前；`selectionStatus` 非 accepted；
10. 失败后重试成功清除错误；
11. 错座位回包 → 拒绝且不应用信封（不误报成功）；
12. 缺刷新信封 → 拒绝且不误报成功；
13. 装配已生效但主视图刷新抛错 → 保持 `accepted`（内核确实成功）+ 显示刷新错误；
14. 空候选名在发起请求前拒绝。

**测试边界（不夸大）**：`.vue` 不能由 `node:test` 直接装载，本批测试覆盖
`BattleView` 调用的纯函数层（候选 → 视图模型、点击动作、刷新、错误），**不是** DOM
渲染或浏览器点击测试；Vue 模板绑定 / CSS / 真实 Worker + Vite 打包路径未在本文件覆盖。
接线本身由 [ui-wiring.log](ui-wiring.log) 的 grep（组件引用、`@click`、`emit("choose")`、
`localClient.chooseGeneral`、`applyEnvelope` 调用点）与 `typecheck` 证明。

## 4 玩家可见变化

- 本地原生对局开始后出现「选将」面板，列出内核目录全部候选及技能。
- 点击候选：按钮进入 pending（全部禁用）→ 成功后显示装配座位、武将名、`state_hash`、
  刷新后视图技能；主视图信封同时被应用（座位技能 / 统计 / 日志更新）。
- 失败（未知武将、越界座位、旧产物缺导出、畸形回包）在面板内以红色错误文本显示，
  不会出现「已装配」成功提示，也不会刷新主视图。

## 5 仍未完成（不写成已完成）

- [ ] **真实浏览器端到端**：Worker + Vite 打包路径、实际点击与渲染未验收；本轮仅
  `typecheck` + 纯函数测试 + Node 下真实 Worker 冒烟。
- [ ] **视觉验收**：候选面板在窄屏/长列表（真实内核 93 项）下的布局、可读性与滚动未做
  截图或人工确认。
- [ ] **联机房间协议选将的客户端入口**：服务端 `503ae703` 已落房间级选将，但
  `NetworkEngineClient` 尚无 `chooseGeneral`，面板在 `mode=online` 下隐藏。
- [ ] **候选未按座位门控**：内核 `general_candidates()` 仍是目录全量，未按赛前大厅/
  双将明置/信息隔离过滤；适配层与本面板如实透传。
- [ ] **双将/明置选择**未建模（`revealed_generals` 仍为 `[true,false]`）。
- [ ] **原生观察者投影未暴露 `character_def`**：`nativeObserverToSnapshot` 当前把
  `character_def` 固定为 `0`（技能 id 亦为占位），因此选将后座位立绘/名字不会变化；
  本面板用刷新信封中的**技能名**展示「视图已变化」，不能据此宣称座位卡片已同源刷新。
- [ ] 选将发生在对局中时与开放选择/回放的一致性仍未定义（NC05 既有待办）。
- [ ] 未重建 WASM；若后续改动 Rust 导出面，需按 `pnpm --dir apps/web-client build:native`
  重建并复核 32 项冒烟与真实 WASM 用例。

## 6 改动文件与行数（单文件 ≤ 2000）

| 文件 | 行数 | 说明 |
| --- | --- | --- |
| `apps/web-client/src/engine/generalSelectModel.ts` | 409 | 新增：候选校验收窄、座位投影、提交控制器 |
| `apps/web-client/src/components/battle/GeneralSelectPanel.vue` | 178 | 新增：选将面板（候选/空/错误/成功/刷新） |
| `apps/web-client/src/views/BattleView.vue` | 1086 | +86：拉候选、点击接线、信封后同步投影、新对局/读档重置 |
| `apps/web-client/scripts/general-select-ui.test.mjs` | 402 | 新增：14 项 UI 面板回归（纯函数层） |
| `apps/web-client/src/types/engine.ts` | 320 | +2：`SkillView.name?` |
| `docs/architecture/native-progress.md` | 423 | 仅 NC05 行更新；该行在提交前被并行提交 `de6a9b48` 一并纳入（见下） |
| `docs/operations/evidence/2026-10-18-nc05-general-select-ui/*` | — | 本 README + 日志 + SHA256SUMS |

> 提交说明：共享工作区里并行开发者在 2026-10-18 22:48 的提交 `de6a9b48`
> （NC08 上游对照）把本轮对 `native-progress.md` 的 NC05 行改动一并 `git add` 走了；
> 该行内容已随 `de6a9b48` 进入历史，本轮自己的提交因此不含 `docs/architecture/` 文件，
> 只包含上表中的源码/测试/证据文件。

## 7 复跑

```bash
cd noname-next
pnpm --dir apps/web-client typecheck
pnpm --dir apps/web-client test
pnpm --dir apps/web-client smoke:native
python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'
python3 -B scripts/check-source-lines.py
```

工作区存在并行开发者的未提交改动；以上结果只对应本轮采集时的工作树（日志内记录命令与
退出码），不代表其他人的改动已验收。

本目录日志与 README 的 SHA-256 见 [SHA256SUMS](SHA256SUMS)。

## 主 agent 独立复核（2026-10-18）

| 复核项 | 命令 | 结果 |
|---|---|---|
| 型别检查 | `pnpm --dir apps/web-client typecheck` | **exit 0** |
| 前端测试 | `pnpm --dir apps/web-client test` | **41/41 pass / 0 fail**（与报告一致） |
| 原生冒烟 | `pnpm --dir apps/web-client smoke:native` | **exit 0**，`{"total":32,"failed":0}` |
| 行数门禁 | `python3 -B scripts/check-source-lines.py` | 8266 文件 ≤ 2000 行 |
| 提交范围 | `git show --stat f5fd1cf1` / `0c483c3b` | 其自有文件（源码/测试/证据） |

### 并行协作事件（如实记录）

- 本切片报告：其在 HEAD 被并行提交前移后执行 `git commit --amend --no-edit`，**误改到父 agent 的
  NC08 提交**（原 `fd1d1afd` → 现 `b73df256`），使该提交额外包含 7 个 NC05 UI 证据文件。
- **父 agent 复核**：`b73df256` 的**源码/测试改动与本提交消息完整无缺**（`nc08_prepare_phase_mapping.rs`
  67 行、`native-progress.md` NC08 行、`2026-10-18-nc08-prepare-phase/README.md` 均在），
  仅额外携带对方证据文件；未做 reset/rebase，无内容丢失。按「不重写历史、保留他人变更」处理：
  **只记录，不修历史**。
- 教训（供后续 agent）：共享工作区中 **禁止 `--amend`**（HEAD 可能已被并行提交前移）；
  需要补证据时应新建提交。

### 仍未完成（沿用其清单，未验收项）

1. 真实浏览器端到端（Worker+Vite 打包、实际点击/渲染）与视觉验收；
2. 联机房间协议选将的**客户端入口**（`NetworkEngineClient` 无 `chooseGeneral`，online 模式面板隐藏）；
3. 候选**按座位门控**（内核仍目录全量）、双将/明置未建模；
4. 原生观察者投影 `character_def` 仍为 0（技能 id 占位）→ 选将后立绘/名字不随动；
5. 对局中选将与开放选择/回放的一致性未定义；
6. 若改 Rust 导出面需 `build:native` 重建并复核冒烟。
