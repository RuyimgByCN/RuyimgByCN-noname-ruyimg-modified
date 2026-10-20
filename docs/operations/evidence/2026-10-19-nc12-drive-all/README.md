# 逐包**整局 AI** 跑批（22 个可测包，2026-10-19）

- 工作包：**NC12 / 八项第 8 项（性能）**；规则版本 v322（工作树）/ 存档格式 v105
- 工具：`nc12_bench drive tier=dir:<包目录名> seats=4 budget=64 lookahead=1 variant=0 seed=42007`
- 原始输出：[drive-all.txt](drive-all.txt)

- 设备/工具链：Apple M3 Max / macOS 27.0.1 / rustc 1.95.0 / `--release`

## 1 汇总

| 指标 | 值 |
|---|---|
| 包数 | **22** |
| **全部收敛且零拒绝** | **22/22** ✅ |
| 整局步数合计 | **5739** |
| 墙钟中位数 | **12.83 ms** |
| 墙钟最大 | **14.65 ms** |

## 2 逐包数据（按墙钟升序）

| 包 | 步数 | accepted | rejected | 收敛 | 事件 | 墙钟 ms |
|---|---:|---:|---:|---|---:|---:|
| `shahai-gwent-upstream-noname` | 156 | 131 | 0 | ✅ | 411 | 7.13 |
| `wuxing-upstream-liuli` | 164 | 137 | 0 | ✅ | 429 | 8.19 |
| `shahai-yunchou-upstream-noname` | 195 | 160 | 0 | ✅ | 505 | 8.63 |
| `wudian-upstream-noname` | 213 | 178 | 0 | ✅ | 556 | 8.91 |
| `shahai-hearth-upstream-noname` | 214 | 178 | 0 | ✅ | 564 | 9.23 |
| `huanle-upstream-noname` | 228 | 189 | 0 | ✅ | 588 | 9.7 |
| `shahai-zhenfa-upstream-noname` | 254 | 210 | 0 | ✅ | 664 | 10.62 |
| `shahai-gujian-upstream-noname` | 253 | 211 | 0 | ✅ | 660 | 11.05 |
| `shahai-mtg-upstream-noname` | 292 | 237 | 0 | ✅ | 760 | 12.57 |
| `cardpile-upstream-noname` | 261 | 214 | 0 | ✅ | 659 | 12.68 |
| `cardpile-upstream-liuli` | 261 | 214 | 0 | ✅ | 659 | 12.8 |
| `hero-kill-upstream-noname` | 298 | 245 | 0 | ✅ | 762 | 12.86 |
| `d3-upstream-noname` | 298 | 245 | 0 | ✅ | 762 | 12.87 |
| `doomsday-brawl-upstream-liuli` | 298 | 245 | 0 | ✅ | 762 | 13.17 |
| `mark-supplement-upstream-liuli` | 298 | 245 | 0 | ✅ | 762 | 13.17 |
| `boss-zhugeliang-upstream-liuli` | 298 | 245 | 0 | ✅ | 762 | 13.21 |
| `shaai-xianjian-upstream-noname` | 298 | 245 | 0 | ✅ | 762 | 13.26 |
| `select-beautify-upstream-liuli` | 298 | 245 | 0 | ✅ | 762 | 13.33 |
| `longxue-upstream-liuli` | 298 | 245 | 0 | ✅ | 762 | 13.36 |
| `epic-cards-upstream-liuli` | 298 | 245 | 0 | ✅ | 762 | 13.63 |
| `shaai-ow-upstream-noname` | 298 | 245 | 0 | ✅ | 762 | 14.58 |
| `shahai-swd-upstream-noname` | 268 | 223 | 0 | ✅ | 739 | 14.65 |

## 3 判读（**如实**）

1. **22 个可测包**逐一跑完整局 AI：**全部 `finished=true`、`rejected=0`** ✅；
2. 墙钟 **7.13–14.65 ms**（中位 **12.83 ms**）——同一机器、同一负载形状，可横向比较；
3. **不是**优化前后对比（无旧实现基线），**不是**产品级（无 server/浏览器）；
4. 19 个无卡牌/武将资产的包**不在此表**（见[逐包 probe 跑批](../2026-10-19-nc12-all-packages/README.md)）。

## 4 复现

```bash
cd noname-next
cargo run --offline --release -p engine-native --example nc12_bench -- \
  drive tier=dir:shahai-swd-upstream-noname seats=4 budget=64 lookahead=1 variant=0 seed=42007
```
