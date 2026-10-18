# NC05 前端选将接线（2026-10-18）

范围：NC05/NC09「正式选将」的**前端消费面**——适配层与 Worker 暴露候选/选将，
玩家侧不再只有建局随机装配一条路径。内核导出面（`general_candidates_json` /
`assign_general`）由提交 `30d10e34` 落地，本轮只做前端接线与验收。

规则版本 v321 / 存档 v105 未变；WASM 产物为 web 目标（`build-native.mjs`）。

## 1. 缺口复现（`repro-before.log`）

采集基线 = 本轮改动前的 HEAD。四处 grep 计数全为 0：

| 位置 | 命令 | 结果 |
| --- | --- | --- |
| 适配层 | `git show HEAD:apps/web-client/src/worker/nativeBackend.ts \| grep -c 'general_candidates\|assign_general\|generalCandidates\|chooseGeneral'` | 0 |
| Worker 入口 | 同法查 `engine.native.worker.ts` | 0 |
| 共享协议 | 同法查 `types/engine.ts` | 0 |
| 主线程客户端 | 同法查 `engine/client.ts` | 0 |
| 前端全量 | `git grep -n 'general_candidates\|assign_general\|generalCandidates\|chooseGeneral' HEAD -- apps/web-client/src \| wc -l` | 0 |
| 旧 WASM 产物 | `grep -n 'general\|assign' .../native-pkg/engine_native_wasm.d.ts` | 无输出（旧产物只有 NC03 面） |

即：前端能改变座位技能的唯一路径是 `nativeCreate`/`new_with_packs_json` 的建局
随机装配，**无法选将**；且当时磁盘上的 wasm 产物早于导出面提交 `30d10e34`
（`git show 30d10e34^:...lib.rs | grep -c general_candidates_json` = 0，
`30d10e34:...` = 4）。

## 2. 已实现（本轮）

- **适配层**（`nativeBackend.ts`，+224 行）：`generalCandidates(handle)`、
  `chooseGeneral(handle, seat, name)`，以及可单测的
  `parseGeneralCandidatesJson` / `parseGeneralAssignmentJson`。
  - 候选逐项结构校验：`count` 整数、`candidates` 数组、每项
    `{def:非负整数, name:非空字符串, skills:字符串数组}`、`def` 不重复、
    `count` 与候选数一致；任一不符即抛错（不静默过滤成更小的候选集）。
  - 选将返回校验：`seat`/`name` 必须回显请求值、`revision` 整数、
    `state_hash` 为 hex；不一致即抛错（错座位不算成功）。
  - 失败路径：`assign_general` 抛出的原始文本（wasm-bindgen 抛的是 JS 字符串）
    原样带上下文重抛：`native: 选将失败(seat=…, name=…): assign_general 被拒绝: NoSuchGeneral`。
  - 旧产物缺导出：显式报错并给出重建命令
    （`执行 pnpm --dir apps/web-client build:native 重建 native-pkg`），
    半旧产物（只缺一个）同样整体拒绝，不降级为「无候选」。
- **缓存失效**：`invalidateSeatCaches(handle, seat)`；`chooseGeneral` 在
  **内核正常返回后、解析返回前**失效。根因是实测事实：
  `assign_general` **不推进 `revision`**（装配前后 `revision()` 相同），
  `(revision, seat)` 记忆化会命中装配前的旧候选；且装配改变共享状态
  （`state_hash` 变），故按句柄整体丢弃（该座位失效的超集）。选择面
  `open_choice_json` 逐次直读、无缓存。
- **Worker 入口**（`engine.native.worker.ts`）：新增 `generalCandidates` /
  `chooseGeneral` 两个消息分支；选将成功回 `{assignment, envelope}`，信封里
  该座位新技能集对界面可见。
- **协议与客户端**：`types/engine.ts` 增 `GeneralCandidate` /
  `GeneralCandidatesPayload` / `GeneralAssignment` / `ChooseGeneralPayload` 与
  两条 `WorkerRequest` 变体；`client.ts` 增 `generalCandidates()` /
  `chooseGeneral(seat, name)`（旧线后端会对该消息显式回
  `unknown message`，不静默成功）。
- **测试基建**：`ts-hook.mjs` 增解析钩子（纯增量：先按原样解析，失败才补
  `.ts`），使 Worker 入口这类使用 Vite 风格免扩展相对导入的生产模块能在
  node:test/冒烟里被真实导入。

## 3. 验收（命令与结果，全部在重建 WASM 之后）

| 命令 | 结果 | 日志 |
| --- | --- | --- |
| `node apps/web-client/scripts/build-native.mjs` | exit 0（48.25s）；新导出进 d.ts | `build-native.log`、`dts-exports.log` |
| `pnpm --dir apps/web-client typecheck` | exit 0（`vue-tsc --noEmit` 无输出） | `typecheck.log` |
| `pnpm --dir apps/web-client test` | **27/27 通过、0 失败**（新增 12 项） | `test.log` |
| `pnpm --dir apps/web-client smoke:native` | **32 项全 PASS、0 failed**（新增 9 项） | `smoke.log` |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | OK（Ran 1 test） | `line-gate-unittest.log` |
| `python3 -B scripts/check-source-lines.py` | `8259 files <= 2000 lines`，exit 0 | `line-gate.log` |

采集口径说明：`build-native.log` 采于测试基建（`ts-register.mjs` 未使用导入清理）
最后一次微调之前；该改动只影响 node:test 的模块装载钩子，**不进入 WASM 产物**
（产物哈希见 `dts-exports.log`）。`typecheck`/`test`/`smoke`/两条门禁的日志按最终
源码重采（第二轮，全部 exit 0，结果与首轮一致：27/27、32 项全 PASS）。

新增回归 `apps/web-client/scripts/native-general-select.test.mjs`（12 项）覆盖：

1. 候选解析正常形态（且不改写输入）；
2. 畸形候选逐项报错：非 JSON、`null`、数组、缺 `count`/`candidates`、`count`
   类型错、项缺 `name`/`skills`、`name` 空串、`def` 非整数/负数、`skills` 非
   字符串数组、`def` 重复、`count` 与候选数不一致；
3. 选将返回与请求不一致（错座位/错名字/缺字段/非 hex）报错；
4. 旧产物缺导出（全缺/半缺）→ 报 `build:native`，且不调用任何内核方法；
5. 选将失败保留原始错误文本、且不改缓存（状态未变）；
6. 选将成功：桩下 `revision` 不变仍必须重查内核、技能集与候选随之变化；
7. 选将已生效但返回畸形：报错同时保持缓存失效；
8. 候选查询只读、不触发其它内核调用；
9. **真实 WASM**：候选非空且逐项过校验、按名字装配后视图技能集 == 候选技能
   集、装配回满；
10. **真实 WASM**：装配不推进 `revision`，两座位不同装配给出不同
   64 位 hex `state_hash`；
11. **真实 WASM**：未知武将（`NoSuchGeneral`）与越界座位（`座位越界（9 ≥ 4）`）
    报错且视图逐字段不变、失败后仍可选将；
12. **真实 WASM**：记忆化（同修订号只查一次）与选将后强制重查
    （打桩计数 `legal_actions_json`，1 → 2）。

冒烟新增 9 项（`smoke.log`）：5 项适配层 × 真实内核（候选 93、技能集变化、
`state_hash` 形态、未知武将/越界拒绝）+ 4 项**真实 Worker 协议**
（`engine.native.worker.ts` 的 `onmessage` → `generalCandidates` 回包 93 项、
`chooseGeneral` 回包含新技能集与 `state_hash`、两类失败回 `ok:false` 且错误
文本可读）。Node 下补 `self` 即可跑真实 Worker 入口（wasm-bindgen 的 init 已用
字节完成，浏览器路径仍走 URL，不受影响）。

## 4. 仍未完成（不因本轮记为完成）

- **选将 UI 未接线**：`BattleView`/Lobby 尚无选将面板；本轮只到
  「适配层 + Worker + 协议 + 客户端」这一层，玩家还看不到候选列表。
- **候选不含座位**：内核 `general_candidates()` 是目录全量，尚未按座位门控
  （赛前大厅阶段、双将明置与选将信息隔离）；适配层如实透传，不假装过滤过。
- **双将/明置选择**未建模（`revealed_generals` 只置 `[true,false]`）。
- 真实浏览器端到端（Worker + Vite 打包路径）未验收，只在 Node 下跑真实 Worker
  模块；平台档位与热切换仍属 NC05 其它待办。
- 选将发生在对局中时与开放选择/回放的一致性未定义（内核不校验是否存在挂起
  选择），登记为待办而非「已支持」。

## 5. 复跑

```bash
cd noname-next
node apps/web-client/scripts/build-native.mjs
pnpm --dir apps/web-client typecheck
pnpm --dir apps/web-client test
pnpm --dir apps/web-client smoke:native
python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'
python3 -B scripts/check-source-lines.py
```

## 6. 改动文件（行数门禁：单文件 ≤ 2000 行）

| 文件 | 行数 | 说明 |
| --- | --- | --- |
| `apps/web-client/src/worker/nativeBackend.ts` | 813 | 候选/选将适配 + 校验 + 缓存失效 |
| `apps/web-client/src/worker/engine.native.worker.ts` | 108 | 两条消息分支 |
| `apps/web-client/src/types/engine.ts` | 318 | 协议类型与 `WorkerRequest` 变体 |
| `apps/web-client/src/engine/client.ts` | 240 | 主线程 `generalCandidates`/`chooseGeneral` |
| `apps/web-client/scripts/native-general-select.test.mjs` | 469 | 新增回归 12 项 |
| `apps/web-client/scripts/native-worker-smoke.mjs` | 407 | 冒烟 +9 项（含真实 Worker 协议） |
| `apps/web-client/scripts/ts-hook.mjs` | 70 | 解析钩子（免扩展相对导入） |
| `apps/web-client/scripts/ts-register.mjs` | 16 | 注册解析钩子 |
