# NC12/NC09 真实浏览器：**完整对局到终局**（headless Chrome）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC12**（浏览器档位）+ **NC09**（真实客户端）
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**在真实浏览器内推进一整局至「终局」**（319 次「自动一步」、rev 0→319、**控制台零错误**）

## 1 方法（可复跑）

```bash
(cd apps/web-client/dist && python3 -m http.server 8123 &)
node apps/web-client/scripts/nc12-browser-fullgame.mjs http://127.0.0.1:8123/ \
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" <outdir> 400
```

`apps/web-client/scripts/nc12-browser-fullgame.mjs`（新增）：CDP 驱动 headless Chrome →
点击「本地开局」→ 循环点击「自动一步」（**脚本节流 120 ms/次**，避免把节流当作引擎耗时）
→ 记录点击次数、`rev` 终值、墙钟、终局标记与最终正文。

## 2 实测（[fullgame.log](fullgame.log)、[fullgame-report.json](fullgame-report.json)）

| 指标 | 值 |
|---|---|
| 点击「自动一步」次数 | **319** |
| 引擎修订号 | **rev 0 → 319**（正文显示「**终局 · rev 319**」） |
| **终局标记** | **`ended_marker: true`**（界面出现「终局」） |
| 墙钟 | 54.0 s（**含脚本 120 ms/次节流**，非引擎耗时） |
| **控制台错误** | **0**（`console_errors: []`，整局无异常/`console.error`） |
| 局面规模（节选） | 「► 第 14 轮 · P3 · 出牌」 |

**读法**：真实浏览器中，**整局**（含选将面板、阶段推进、自动一步、投影刷新）从 rev 0 跑到 **rev 319 终局**，
全程无控制台错误 → 浏览器档位的「**可完整对局**」这一项有实测证据（此前只到「可交互」）。

## 3 口径边界（不得外推）

- **headless Chrome + 本机静态服务**；不含真实网络、不含 GPU 合成与真实输入设备；
- 墙钟含**脚本节流**（120 ms/次）→ **不能当作引擎或 UI 的单步耗时**；单步耗时应用
  `rev` 增量与引擎侧基准（见 [NC12 基线](../2026-10-18-nc12-baseline/README.md)）；
- 「终局」判据是界面文本出现「终局」；**未**断言胜负结果与引擎 `outcome` 的一致性（登记待办）；
- 单局、单档（默认原生后端）；未做多局稳定性与内存曲线。

## 4 未完成 / 待办

- [ ] 终局**胜负**与引擎 `outcome` 的一致性断言（界面文本 → 投影 → 引擎）
- [ ] 多局连续（稳定性/内存曲线）与长时浏览器负载
- [ ] 浏览器内**动画帧/输入响应**指标（需真实渲染路径与 `requestAnimationFrame` 采样）
- [ ] 移动（Android/iOS）与桌面（Tauri）档位；多浏览器（Safari/Firefox）；真实网络
- [ ] 旧线前端同口径完整对局对照

## 5 追加：**多局连续**（3 局）与终局显示观察（2026-10-19 同轮）

同一脚本连续运行 3 局（[multigame.log](multigame.log)）：

| 局 | 点击次数 | rev 终值 | 墙钟 | 终局标记 | 控制台错误 |
|---|---:|---:|---:|---|---|
| 1 | 319 | 319 | 56.9 s | ✅ | **0** |
| 2 | 319 | 318 | 66.2 s | ✅ | **0** |
| 3 | 301 | 300 | 69.1 s | ✅ | **0** |

**判读**：**3/3 局均到终局**、全程零控制台错误 → 浏览器档位在**多局连续**下稳定。
第 3 局步数与 rev 与前两局不同（**脚本未固定种子** → 每局局面不同）→ 记录为「不同局面亦能终局」。

**观察（登记为待办，不写成缺陷结论）**：终局时界面正文只显示「**终局 · rev N**」，
**未显示胜负/胜者**；而引擎侧投影已含 `outcome`（见
[NC09 投影字段证据](../2026-10-18-nc09-projection-fields/README.md)）→ 属**前端渲染面**待接线
（需进一步核对界面是否有胜负展示位）。

### 仍未完成（更新）

- [ ] **终局胜负展示**：界面未显示胜者/胜负（引擎投影已有 `outcome`）→ 前端渲染待接线
- [ ] 终局胜负与引擎 `outcome` 的**一致性断言**（需先有展示位或用 worker 协议读取）
- [ ] 固定种子下的**可重复**多局对照（当前脚本未固定种子）
- [ ] 浏览器长时负载（内存曲线）、动画帧/输入响应
- [ ] 移动（Android/iOS）与桌面（Tauri）档位；多浏览器（Safari/Firefox）；真实网络
- [ ] 旧线前端同口径完整对局对照

## 6 终局胜负展示：**已接线并实测通过**（2026-10-19 同轮）

§5 登记的「界面不显示胜负」缺口已修复，链路三处：

| 层 | 变更 |
|---|---|
| WASM 壳（`crates/engine-native-wasm/src/lib.rs`） | 视图 `outcome` 由固定 `"finished"` 改为**下发胜负类型**：`LoyalistWin`/`RebelWin`/`RenegadeWin`/`SingleWin{player}`/`GroupWin{group}`；未建模的模式胜者（对决/斗地主/Boss）**保持 `"finished"`**（不猜测语义） |
| 前端（`BattleView.vue`） | 新增 `outcomeText()`：映射为「主公方胜 / 反贼胜 / 内奸胜 / 单挑胜者 P# / 势力胜（xx）」；未知形态**如实显示原始 JSON** |
| 数据源 | 读 `env.view.outcome`（`EngineEnvelope` 无顶层 `outcome` 字段——typecheck 首轮即报错纠正） |

**实测**（重建 WASM + 前端后，同脚本整局）：

```
RESULT scenario=browser_fullgame tier=web {"clicks":319,"final_revision":318,"ended_marker":true,"console_errors":[]}
终局行: 终局 · 单挑胜者 P4 · rev 319
```

→ 终局行现为「**终局 · <胜负> · rev N**」；本局经「本地开局」进入的是**原型/单挑口径**的对局，
故引擎给出 `Winner::Player(3)` → 显示「单挑胜者 P4」，**与引擎语义一致**（未硬编码身份局文案）。

### 仍未完成（更新）

- [ ] 身份局路径的胜负文案实测（需以身份局开局：`new_identity` 路径或联机房间）
- [ ] 终局胜负与引擎 `outcome` 的**自动化一致性断言**（当前为人工读日志）
- [ ] 固定种子可重复多局、浏览器长时负载、动画帧/输入响应
- [ ] 移动（Android/iOS）与桌面（Tauri）档位；多浏览器；真实网络；旧线前端同口径完整对局

## 7 文案映射**抽为模块 + 自动化断言**（2026-10-19 同轮）

§6 的 `outcomeText` 原先内联在 `BattleView.vue`（无法单测）→ 抽为
`apps/web-client/src/engine/outcome.ts`，并新增 `scripts/outcome-text.test.mjs`：

| 断言 | 期望 |
|---|---|
| `{LoyalistWin:null}` / `{RebelWin:null}` / `{RenegadeWin:null}` | 主公方胜 / 反贼胜 / 内奸胜 |
| `{SingleWin:{player:4}}` | 单挑胜者 P4 |
| `{GroupWin:{group:"shu"}}` | 势力胜（shu） |
| `"finished"` / `null` | 已结束（胜负类型未建模） / 胜负未知 |
| **未知形态** `{SomethingNew:1}` | `已结束（{"SomethingNew":1}）`（**如实显示，不猜测语义**） |

**结果**：客户端测试 **44 passed / 0 failed**（[client-tests.log](client-tests.log)，原 41 + 本 3）；
`typecheck` exit 0；`vite build` exit 0；重建后浏览器整局复测：
`319 次点击 / rev 319 / ended_marker=true / 控制台零错误`，终局行仍为「**终局 · 单挑胜者 P4 · rev 319**」
（[fullgame-outcome2.log](fullgame-outcome2.log)）。

### 仍未完成（更新）

- [ ] **身份局端到端**：以身份局开局跑到终局并核对文案（本地开局为原型/单挑口径；
      身份局需联机房间路径或新增身份局开局入口）
- [ ] 引擎 `outcome` → 协议形态的**Rust 侧**自动化断言（当前 Rust 侧只有编译期映射，无断言测试）
- [ ] 固定种子可重复多局、浏览器长时负载、动画帧/输入响应
- [ ] 移动（Android/iOS）与桌面（Tauri）档位；多浏览器；真实网络；旧线前端同口径完整对局
