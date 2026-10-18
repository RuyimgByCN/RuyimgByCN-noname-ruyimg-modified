# NC08 各层验收边界清单（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）：「验收分别覆盖单元/性质测试、内核集成、生产适配器、真实客户端、原生服务端、native/WASM、上游行为和异常输入；**各层报告自身边界，不合并成一个"完成率"**」
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**八层各自的实测数字与未覆盖边界已分列**（不汇总为单一完成率）

## 1 各层实测（本机本次源码，命令与日志见 [layers.log](layers.log) / [layers-client.log](layers-client.log)）

| 层 | 命令 | 实测 | 本层**未覆盖**边界 |
|---|---|---|---|
| ① 单元/性质测试 | `cargo test -p engine-native --lib` | **11 / 0** | lib 面单元测试；不含内容包行为 |
| ② 内核集成 | `cargo test -p engine-native --no-fail-fast`（含 examples） | **1153 / 0 / 2 ignored** | 14 golden 场景 + 各内容批；**不含**真实上游 JS 对照 |
| ③ 装载与包管理 | `cargo test -p content-native-loader -p content-runtime` | **65 / 0** | 装载/存储/锁/资产；不含 UI 安装向导 |
| ④ native/WASM | `cargo test -p engine-native-wasm` | **31 / 0** | 宿主（非 wasm32）下同实现路径；**wasm32 目标运行仅由产物冒烟覆盖** |
| ⑤ 生产适配器/服务端 | `cargo test -p game-server` | **87 / 0** | 原生房间/视图桥接/生命周期/双后端；**旧线房间能力对齐未完成** |
| ⑥ 真实客户端 | `pnpm --dir apps/web-client typecheck` / `test` / `smoke:native` | typecheck **exit 0**；node:test **27/27**；冒烟 **32/32** | **未启动真实浏览器**；Vite 打包路径端到端未验收；桌面（Tauri）未验收 |
| ⑦ 上游行为 | —（未开始） | **未验收** | 需上游 JS 侧可复跑 harness；当前只有旧线↔原生线对照（4 处开局差异已登记规则基准） |
| ⑧ 异常输入 | 受控反例用例（`nc07_state_same_source` 12 项、`nc08_cross_target` 4 项、`nc10_planner_pool` 队列满/过期等） | **16 / 0** | 覆盖篡改归档/越界引用/伪造动作/队列满/过期拒绝；**未做模糊测试与长时随机输入** |

行数门禁：8259 文件 ≤ 2000 行（[source-lines.log](source-lines.log)）。

## 2 边界声明（不得合并成一个"完成率"）

- 上述数字是**各层自身**的结果：例如 ② 全绿**不证明** ⑥ 的浏览器端到端，⑥ 的冒烟**不证明** wasm32 目标，④ 的宿主测试**不证明**真实 WASM 目标；
- **⑦ 上游行为对照未开始**——「旧线全绿」「golden 未变」「测试全绿」三者都不能推导上游等价（方案明确要求分别记录）；
- **平台档位**（Windows/macOS/Linux、Android/iOS、Web）除 macOS 本机外**均未取证**；
- 内容面 37 来源的整包验收**未完成**（NC11 复核结论：24 只是标签，13 来源 open，19/24 行内仍登记未迁移面）。

## 3 未完成 / 待办（NC08）

- [ ] **上游行为逐命令对照**（第 ⑦ 层，方案主体）：需上游 JS 侧 harness（固定上游版本/初始局面/随机输入与玩家选择）
- [ ] wasm32 目标的**运行与恢复**自动化（第 ④ 层）：需 wasm-pack/wasmtime 工具链
- [ ] 真实浏览器（Vite 打包路径）与桌面外壳端到端（第 ⑥ 层）
- [ ] 平台档位取证（Windows/Linux/Android/iOS/Web）
- [ ] 模糊测试/长时随机输入（第 ⑧ 层）
- [ ] 各层数字随批次推进**同步更新**（本清单为 2026-10-18 快照，读数前先 `git rev-parse HEAD`）
