# 22 个有资产包的**逐包吞吐**：µs / 命令（2026-10-19）

- 工作包：**NC12 / 八项第 8 项（性能）**
- 工具：`nc12_bench drive tier=dir:<包> seats=4 budget=64 lookahead=1 variant=0 seed=42007`
- 原始输出：[run.txt](run.txt)

## 1 结果（**22/22 全部收敛、零拒绝** ✅）

| 包 | steps | accepted | **µs/命令** | 墙钟 ms |
|---|---:|---:|---:|---:|
| `shahai-zhenfa-upstream-noname` | 254 | 210 | **49.86** | 10.47 |
| `wudian-upstream-noname` | 213 | 178 | **50.91** | 9.06 |
| `huanle-upstream-noname` | 228 | 189 | **52.20** | 9.87 |
| `shahai-hearth-upstream-noname` | 214 | 178 | **52.62** | 9.37 |
| `hero-kill-upstream-noname` | 298 | 245 | **53.43** | 13.09 |
| `shahai-yunchou-upstream-noname` | 195 | 160 | **53.62** | 8.58 |
| `shahai-gujian-upstream-noname` | 253 | 211 | **53.78** | 11.35 |
| `d3-upstream-noname` | 298 | 245 | **53.95** | 13.22 |
| `doomsday-brawl-upstream-liuli` | 298 | 245 | **54.01** | 13.23 |
| `shahai-gwent-upstream-noname` | 156 | 131 | **54.20** | 7.10 |
| `shaai-xianjian-upstream-noname` | 298 | 245 | **54.60** | 13.38 |
| `shaai-ow-upstream-noname` | 298 | 245 | **54.73** | 13.41 |
| `select-beautify-upstream-liuli` | 298 | 245 | **54.90** | 13.45 |
| `mark-supplement-upstream-liuli` | 298 | 245 | **55.03** | 13.48 |
| `boss-zhugeliang-upstream-liuli` | 298 | 245 | **55.11** | 13.50 |
| `shahai-mtg-upstream-noname` | 292 | 237 | **55.19** | 13.08 |
| `longxue-upstream-liuli` | 298 | 245 | **55.22** | 13.53 |
| `epic-cards-upstream-liuli` | 298 | 245 | **56.17** | 13.76 |
| `cardpile-upstream-liuli` | 261 | 214 | **58.35** | 12.49 |
| `cardpile-upstream-noname` | 261 | 214 | **59.16** | 12.66 |
| `wuxing-upstream-liuli` | 164 | 137 | **62.45** | 8.56 |
| `shahai-swd-upstream-noname` | 268 | 223 | **185.47** | 41.36 |

**汇总**：包数 **22**；**不收敛/有拒绝 = 0** ✅；**µs/命令 中位 54.66**；范围 **49.86–185.47**。

## 2 判读（**如实**）

1. **中位 54.66 µs/命令**，**21 个包落在 49.86–62.45**（**极差 1.25×**）✅ ⇒ **内容包对每命令开销影响很小**；
2. ⚠️ **`shahai-swd` 是明显异常：185.47 µs/命令（中位的 3.4×）** —— **需跟进**（见 §3）；
3. ⚠️ **8 个包出现完全相同的 `steps=298 / accepted=245`**（`doomsday-brawl`/`shaai-xianjian`/`shaai-ow`/`select-beautify`/`mark-supplement`/`boss-zhugeliang`/`longxue`/`epic-cards`）——
   **这个巧合需要解释**（**疑似触到某个统一上限**，本次**未查**）；
4. **不是**优化前后对比、**不是**产品级（无 server/浏览器）。

## 3 待跟进（**两条**）

| # | 现象 | 已知 | 下一步 |
|---|---|---|---|
| **P1** | **`shahai-swd` 185.47 µs/命令**（中位 3.4×）| 上一轮（3 seed）**68.99 µs**；本轮 **185.47** ⇒ **波动 2.7×** ⚠️ | 用 3–5 seed 重测该包，确认是**冷启动**还是**真实慢** |
| **P2** | **8 包 `steps=298/accepted=245` 完全相同** | 未查 | 查 `drive` 的终止条件（是否有统一上限）|

## 4 口径与边界

- 指标：`墙钟(ns)/accepted` ⇒ µs/命令；**每包 1 seed**（**seed 间波动已在多 seed 证据里量化**）；
- **未控制** CPU 亲和、后台负载、首次运行预热；**跨机不可比**。
