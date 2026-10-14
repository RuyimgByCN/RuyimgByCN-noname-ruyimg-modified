# T13 客户端产品面：engine-native wasm 窄壳 + Node 冒烟（2026-10-14）

## 任务来源

[T13](../../../architecture/native-progress.md) 产品面余项「客户端 UI」的内核侧切片：
此前 O4/T13 跨平台的 wasm 产物均为**旧线 `engine_wasm` 壳**（engine-core），原生引擎
无 wasm 面。本批新建 **`engine-native-wasm`**（workspace 第 20 成员）——native 引擎
的 wasm_bindgen 窄壳，并按 O4 harness 口径完成 Node 冒烟（**浏览器 UI 回归需真实
浏览器 harness，仍归后续批**）。

## 交付

- **`engine-native-wasm`**：窄 API 而非通用 JSON 协议——旧线投影 schema 与原生
  不兼容，为避免给原生全类型补 serde，壳暴露最小驱动面：
  - `new(players, seed)` 标准目录建局；`set_planner(nodes, deadline_ms)` 预算参数化；
  - `view_json(seat)`：`PlayerView` 手写 JSON 投影（信息隔离面：自己手牌具体 id/
    他人仅数量/牌库仅数量）；`open_choice_json(seat)`：N10 选择可见性口径
    （观察者本人含选项、他人仅存在面）；纯只读投影，不进权威状态与哈希；
  - `next_phase/play_card/answer`（选项索引直传）窄动作；
  - `auto_step`：挂起选择自动应答，否则当前座位 `BoundedPlanner` 规划并应用
    （game-server `/native` 同款）；
  - `save_hex/restore_hex`：NGSA v65 字节十六进制往返。
- **wasm 可移植性**（engine-native/src/ai.rs 单点）：`Instant::now` 在
  wasm32-unknown-unknown 无时间源 panic——规划 deadline 起点改 cfg 切换：
  wasm 下 `None`（**deadline 静默失效=纯节点预算面**，壳契约登记）；原生目标
  语义不变（724/0/0 全绿复跑）。

## 冒烟实测（Node v26.4.0，wasm-pack 0.15.0 --target nodejs --no-opt）

[smoke-final.json](smoke-final.json)（[运行输出](smoke-stderr.log)，脚本
`scripts/native_wasm_smoke.mjs`）：

| 断言 | 结果 |
|---|---|
| 标准目录建局（ruleset v278，4 人） | PASS |
| 座位视图隔离（own=4 张具体 id / 他人 hidden / 牌库仅数量 108） | PASS ×3 |
| auto_step 自动对局健康推进（**180 步终局**，30 次选择自动应答，0 悬挂） | PASS |
| 存档往返（第 61 步存档 → 新实例恢复视图逐字节一致） | PASS |
| 同 seed 重放一致（4242 双实例逐步比对 60 步零分叉） | PASS |
| 双实例确定性（seed 777 独立 200 步视图一致，T13 口径复用） | PASS |
| 选择可见性（own 选项面与 other 存在面双证） | PASS |

**11/11 通过**。冒烟中发现并修正：smoke 断言的 off-by-one（`steps===60` 时点
实为 61 步后状态）——非引擎缺陷（逐步比对定位为零分叉）。

## 登记边界

- 扩展包装载面（对等旧线 loadContentBundle）暂缓：原生包装载走 npack/包存储
  二进制面，wasm 侧接 C0 装载器归后续批；本壳 `new` 限标准目录。
- `view_json` 的 `outcome` 暂投为 `"finished"` 布尔面（胜者细节未投影）。
- web-client worker 切换（engine_wasm → 本壳）与浏览器 UI 回归（真实浏览器
  harness）归产品面后续批；旧线 UI 消费的投影 schema 不兼容，切换需 UI 层适配。
- wasm 下规划 deadline 失效（无时间源），节点预算（默认 64）仍生效。

## 实测记录

| 项 | 结果 |
|---|---|
| engine-native 全量 | **724/0/0**（ai.rs cfg 改动后复跑，原生语义不变） |
| Gate 1b | engine-native 与 engine-native-wasm 双 target `--all-targets` exit 0 |
| wasm 冒烟 | **11/11**（上表） |
