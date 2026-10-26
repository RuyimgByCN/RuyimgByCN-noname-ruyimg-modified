# B3 PA07 原型半提交收口：Worker 侧 SAB 镜像与 perf=1 分段插桨入库（2026-10-26）

- 执行时间：2026-10-26 **20:22:20（领取）→ 20:28:08 +0800**（调查在 B1 补跑等待期 20:05 起只读进行；均为实际 `date '+%F %T %z'` 输出，早于本批提交）
- 工作包：[台帐「当前阻断」B3](../../../architecture/native-remaining-ledger.md#当前阻断2026-10-26-扫描按优先级)（P2）
- 源码基准：`HEAD = 26d4b2d2aa1342038c887d6e9b759f86da7eafd7`（领取时 `git rev-parse HEAD`）；暂存树 `git write-tree` = `67af82077da3c9d1ffcf2571c2a94abd8c8e5294`（= HEAD + 本批 4 文件，即提交后的干净内容，验证在其导出上进行）
- 执行者：ZCode A 道

**一句话结论**：已提交的 PA07 主线程侧 `client.ts`（`f7b8e46d`）所依赖的 Worker 侧 SAB 镜像（`stepTimings.ts` 的 `setSabMirror`/`mirrorToSab`、`engine.native.worker.ts` 的 init 接线、`types/engine.ts` 的 `latencySab` 字段）以及 `18edc033` 推迟入库的 `nativeBackend.ts` perf=1 `stepMark` 四处插桨，本批按行级拆出并提交；同文件里他人的身份局 `packsJson`、公开事件投影、`eventCursor` 等 hunk 原样留在工作树未暂存。在暂存树导出的真实构建上：`latency=sab` 60 次真实点击 **60/60 提前解析**（`sabDebug seq=63 early=60`），perf=1 Worker 四段分段 n=60 全部非空，两轮 console_errors 0。

## 1 核对「工作树内容 = 证据所述内容」

| 证据所述（[PA07 原型](../2026-10-26-pa07-mitigation-prototype/README.md)、[PA08-2](../2026-10-26-pa08-2-wkwebview/README.md)、`18edc033` 提交说明） | 工作树未提交内容 | 归属 |
|---|---|---|
| `stepTimings.mirrorToSab`：回包 JSON 写入主线程创建的 SAB，`[8..]` 数据、`[0]` 序号、`[4]` 长度，seq 最后发布；超容量静默跳过 | `stepTimings.ts` +31：`SAB_DATA_OFFSET=8`、`Int32Array(buffer,0,2)`（[0]=seq、[1]=len），先 `store(len)` 再 `store(seq+1)`；`postWithStepTimings` 发回包前 `mirrorToSab(resp)` | **B3（PA07）** |
| init 消息携带 `latencySab`，worker 收到后设镜像 | `engine.native.worker.ts`：import `setSabMirror`；init 分支 `setSabMirror(msg.latencySab ?? null)` | **B3（PA07）** |
| `WorkerRequest.init.latencySab?: SharedArrayBuffer` | `types/engine.ts` +5 | **B3（PA07）** |
| `18edc033`：「`nativeBackend.ts` 的 stepMark 分段随事件投影道工作树暂缓入库」 | `nativeBackend.ts`：`import { stepMark }`，`nativeEnvelope` 内 `envelope_enter`/`view_json(raw.length)`/`view_parse`/`envelope_done` 四处 | **B3（PA08 perf=1 挂钩）** |
| —（不属 B3） | `engine.native.worker.ts`：`nativeCreateIdentity(..., msg.packsJson)`；`nativeBackend.ts`：`new_identity_with_packs_json`、`eventCursor`、`readPublicEvents`/`PUBLIC_EVENT_LABELS`、`hash_hex: state_hash_hex()`、读档游标归零、文档注释改写 | 他人在途（身份局带包、公开事件投影——B2 家族），**不暂存** |

## 2 行级拆分方式

四个文件中两个（`stepTimings.ts`、`types/engine.ts`）的全部差异都是 B3，整文件暂存；另两个与他人 hunk 同文件：以 `git show HEAD:<file>` 为底，只套用 B3 的改动生成目标内容，`git hash-object -w` + `git update-index --cacheinfo` 写入索引（不触碰工作树）。暂存 diff 见 [b3-staged.patch](b3-staged.patch)（4 文件 +48/−3）；`git diff`（未暂存）在这两个文件只剩他人 hunk（`engine.native.worker.ts` 1 处、`nativeBackend.ts` +55/−7）。`nativeEnvelope` 为放置 `envelope_done` 把字面量 return 改为 `const envelope … return envelope`，字段保持 HEAD 版本（`events: []`、`hash_hex: ""`）；事件投影作者在新 HEAD 上继续时只剩其字段改动。

## 3 验证（暂存树导出 `/tmp/b3-head`，= 提交后干净 HEAD 内容）

导出相对暂存树的偏离仅为**不入 git 的构建产物/依赖**：`apps/web-client/node_modules` 软链到真实树、复制 gitignored 的 `src/worker/native-pkg`（10-25 构建）与 `src/wasm-pkg`。构建产物输出到独立目录 `/tmp/b3-dist`，未触碰被跟踪的 `apps/web-client/dist/`。

| 命令（在导出的 `apps/web-client` 下） | 结果 |
|---|---|
| `pnpm test` | **48 passed / 0 failed**（干净树的测试数；工作树 80 含他人未提交测试） |
| `pnpm typecheck`（vue-tsc） | 0 错误 |
| `pnpm exec vite build --outDir /tmp/b3-dist --emptyOutDir` | ✓ built（engine.native.worker / engine.worker / 两个 wasm / index） |
| `node --import ./scripts/ts-register.mjs scripts/web-frame-latency.mjs --dist /tmp/b3-dist --out … --mode identity --players 4 --seed 43 --driver click --max-steps 60`（headless Chrome，perf=1 由驱动固定开启） | `initLatencySab=false`；60 次 CDP 真实点击到 rev 60；**Worker 分段 n=60 全非空**：dispatch p50 0.20 ms、view_json p50 0 / max 0.3 ms、view_parse、adapt p50 0.10 ms、view_json_chars p50 824；console_errors 0（[summary](runs/perf-only/summary.json)、[逐点击](runs/perf-only/interactions.csv)） |
| 同上 + `--coop --latency sab` | **`initLatencySab=true`，`sabDebug={"seq":63,"early":60}`**——60 次点击的回包全部经 SAB 镜像在 rAF 内提前解析；rev 60、console_errors 0；transfer p50 16.0→14.8 ms（headless 60 Hz 下与 PA07 有头结论一致：机制通，端到端 i2p 33→32.8 ms 无改善）（[summary](runs/latency-sab/summary.json)、[逐点击](runs/latency-sab/interactions.csv)） |

真实工作树（含他人在途）：`pnpm --dir apps/web-client test` **80/0/0**、`typecheck` 通过；`check-source-lines.py` 8612 文件通过、`check_local_links.py --all` 坏链 0、`check-adr-index.py` 通过。

## 4 边界与遗留

- 本批只入库 PA07 Worker 侧 + perf=1 插桨，**不改变**任何缺省行为：无 `latency=sab` 时 `setSabMirror(null)`、`mirrorToSab` 直接返回；perf 关闭时 `stepMark` 立即返回。
- PA07 本身的「端到端 i2p 未改善」待甄别状态不变（台帐 PA07 行）；本批不是性能结论。
- `nativeBackend.ts`/`engine.native.worker.ts` 中他人的身份局带包与公开事件投影改动仍为未提交（B2），由其执行者按证据核对后提交。
- 冒烟为 headless 60 Hz 一次 60 步，用于「机制可工作」判定，不替代 PA08 有头 120 Hz 全局基线。
