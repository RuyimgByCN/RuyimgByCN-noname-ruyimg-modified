# O4 wasm-opt 对比收口（2026-10-16，D7）

## 任务来源与口径

[进度主稿 O4](../../../architecture/native-progress.md) + [D7](../2026-10-16-d4-d8-decisions/README.md)：接受离线内嵌闭包后原始 **14.5MB**；补装 wasm-opt 对比 `--no-opt` 与各级优化后关闭。

环境（detached worktree `noname-next-wt-perf`，独立 `CARGO_TARGET_DIR`）：

| 项 | 值 |
|---|---|
| 测量 git | `8c7a96146d417a9060be27122ccc9446006d65d3` |
| wasm-pack | **0.15.0**（`--target nodejs --release --no-opt`） |
| wasm-opt | **binaryen 123**（`npm install binaryen@123`） |
| rustc | 1.95.0 / wasm32-unknown-unknown |
| Node | v26.4.0 |
| 并发干扰 | 同机另有 `noname-next-wt-client`；测量开始 load≈53，opt 期间峰值≈61；计时交替取中位数（5 轮×6 级） |

构建耗时：engine-wasm **154s**；engine-native-wasm **22s**。

## 体积（原始 / gzip -9 / brotli q11）

完整行见 [sizes/summary.txt](sizes/summary.txt)。

### engine-wasm（内嵌 standard 全依赖闭包，产品离线体积）

| 配置 | 原始 | gzip | brotli |
|---|---:|---:|---:|
| no-opt | **15,176,936（14.47 MB）** | 3,305,071（3.15 MB） | 2,289,100（2.18 MB） |
| -O1 | 14,311,946 | 3,130,494 | 2,175,271 |
| -O2 | 14,229,566 | 3,111,477 | 2,225,299 |
| -O3 | 14,103,243 | 3,085,735 | 2,210,388 |
| -Os | 14,094,332 | 3,085,792 | 2,208,236 |
| -Oz | **14,026,494（13.37 MB）** | **3,083,523** | 2,209,120 |

相对 no-opt：`-Oz` 原始约 **-7.6%**；gzip 约 **-6.7%**；brotli 约 **-3.5%**（`-Os` brotli 略优于 `-Oz`）。

### engine-native-wasm（窄壳，无内嵌闭包）

| 配置 | 原始 | gzip | brotli |
|---|---:|---:|---:|
| no-opt | 1,814,404（1.73 MB） | 476,194 | 361,231 |
| -O1 | 1,637,938 | **441,525** | **333,883** |
| -O2 | 1,624,547 | 458,382 | 344,535 |
| -O3 | 1,582,891 | 458,328 | 345,718 |
| -Os | 1,580,783 | 460,165 | 347,216 |
| -Oz | **1,571,878** | 459,334 | 346,184 |

原生窄壳上 `-O1` 压缩体积最优；更高档原始更小但 gzip/brotli 反而回升。

## 加载/执行耗时（中位数，5 轮交替）

见 [timings/engine-wasm-median.txt](timings/engine-wasm-median.txt)、[timings/engine-native-wasm-median.txt](timings/engine-native-wasm-median.txt)。

| engine-wasm | require | loadEmbedded | startGame | autoStep×80 |
|---|---:|---:|---:|---:|
| no-opt | 12 ms | **105 ms** | **11 ms** | **43 ms** |
| -O1 | 11 ms | 106 ms | 10 ms | 43 ms |
| -O2 | 11 ms | 159 ms | 37 ms | 74 ms |
| -O3 | 11 ms | 158 ms | 37 ms | 70 ms |
| -Os | 11 ms | 159 ms | 37 ms | 70 ms |
| -Oz | 11 ms | 158 ms | 37 ms | 67 ms |

| engine-native-wasm | require | autoStep×80 |
|---|---:|---:|
| no-opt / -O1…-Oz | 2–3 ms | 11–13 ms |

结论：engine-wasm 上 `-O2` 及以上相对 no-opt **加载与步进明显变慢**（约 +50% load、+55% autoStep）；`-O1` 与 no-opt 耗时同量级。engine-native-wasm 各级耗时无显著差异。

## 功能回归（行为一致）

- engine-wasm：`loadEmbeddedStandard` → `startGame(4,4242)` → `autoStep×80` → `observerJson`；**30/30**（5×6）`smoke=ok`；各级 **stateHash 全同** `1a0c2bae…351c7`——[smoke/engine-wasm.txt](smoke/engine-wasm.txt)、[smoke/hash-consistency.txt](smoke/hash-consistency.txt)。
- engine-native-wasm：既有 `scripts/native_wasm_smoke.mjs` **11/11**，5×6 全 PASS——[smoke/engine-native-wasm.txt](smoke/engine-native-wasm.txt)。

## 交付选择与关闭条件

- **D7**：接受 14.5MB 离线体积——本批确认 no-opt 原始 **14.47 MB**，与裁决一致。
- **默认交付仍为 `--no-opt`**（与现网 `Cargo.toml` `wasm-opt = false` 及构建脚本一致）：更高档体积收益有限，且 `-O2+` 损害 Node 加载/执行中位数。
- 若产品后续要挤压缩体积，优先评估 **`-O1`**（体积约 -5.7%，耗时不劣化）；不建议默认开 `-O2/-Oz`。
- 浏览器 UI 回归仍归 T13 产品面；本批仅 Node 全链路。

**O4 关闭**：wasm-opt 对比与功能一致性已回填。
