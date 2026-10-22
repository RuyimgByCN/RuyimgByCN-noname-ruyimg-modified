# 原生引擎单机性能数据点（2026-10-19）

> **这份数据能证明什么、不能证明什么**
>
> * **能**：在本机、本次编译（`cargo test` debug 配置）下，双仓桥接建局的耗时量级，以及一次命令处理的耗时量级。
> * **不能**：**不能**作为「原生 vs 旧引擎」的对比结论——旧引擎（`engine-core`/`engine-ai`/`engine-wasm`）**没有在本机同一负载下测过**；
>   也不能作为 release 性能数字（debug 未开优化）。
> * 性能门槛（八项之一）要求**同设备、同工具链、同负载的前后数据**。本文**不满足**，故**不计入验收**。

## 采集命令

```bash
cargo test --offline -p engine-native --test t12_dual_diag -- --nocapture
```

## 原始输出（2026-10-19，本轮实测）

```text
[diag] 开始双仓桥接
[diag] 桥接完成: noname=113张 liuli=113张
[diag] 双仓建局开始
[diag] 双仓建局: 476.375µs cards=350
[diag] step 0: Accepted { state_hash: [189, 86, 124, 106, …] … }
[diag] step 1: Accepted { state_hash: [19, 125, 48, 195, …] … }
[diag] step 2: Pending { choice: … prompt: "discard_to_limit" … }
[diag] step 3: Rejected(ChoicePending) seat=0 elapsed=125ns hand_counts=[6, 4, 4]
```

## 读法（务必按此理解，勿外推）

| 观测 | 值 | 说明 |
|---|---|---|
| 双仓桥接 + 建局 | **476 µs** | 350 张牌的目录装配 + 状态初始化；**debug** 构建 |
| 一次命令处理（被拒的那次） | **125 ns** | 被拒路径极短，**不代表**正常命令耗时 |
| 无可用命令耗时数据 | — | 该诊断在遇到 `discard_to_limit` 选择窗时停止，未继续推进 |

## 要拿到可验收的性能证据，还缺什么

1. **同机跑旧引擎**（`engine-core`）同一负载的对照数据；
2. **release 构建**（`--release`）下的同口径数据；
3. **负载谱**：至少覆盖「建局 / N 步随机对局 / 存档-恢复往返 / WASM 侧」四类，
   并记录机器与工具链（CPU、内存、`rustc -V`、profile）；
4. 若要与历史基线比较，需**同一台机器**的历史数据——本仓库当前没有旧引擎的可运行环境。

⇒ 本条**保持待办**；本文只作为「原生侧有一个可复现的起点」记录。

---

## 追加：**WASM 侧**数据点（2026-10-19，Node 宿主，release 构建）

```bash
wasm-pack build crates/engine-native-wasm --target nodejs --release --no-opt --out-dir /tmp/wasm-pkg
node /tmp/wasm-perf.mjs
```

**实测（单次采样）**：

```json
{"initMs":1.637,"steps":400,"playMs":26.371,"perStepMs":0.0659,"viewMs":0.0319}
```

| 观测 | 值 | 说明 |
|---|---|---|
| 建局（`new NativeEngine(4, 4242n)`）| **1.64 ms** | release + wasm-bindgen，Node 宿主 |
| 自动对局 400 步 | **26.37 ms** | `auto_step` 含 AI 决策与引擎推进 |
| **单步** | **0.066 ms** | ≈ 15k 步/秒 |
| 座位视图 `view_json` | **0.032 ms** | 100 次平均 |

### 这仍然**不能**用于性能验收

* **debug/release 混用**：这里是 release，上文原生侧是 debug，**两者不可比**；
* **单次采样**，无重复与方差；
* **旧引擎没有同机同负载数据** ⇒ 无法给「提速/劣化」结论；
* 负载单一（4 人标准局、固定 seed）。

⇒ 性能门槛**仍为待办**。但这条给了一个**可复现的 wasm 侧基线**，供后续同机对照使用。
