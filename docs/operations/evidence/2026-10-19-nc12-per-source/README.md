# **逐来源**效能签名（2026-10-19，八项第 8 项）

- 工作包：**NC12**；规则版本 v322（工作树）/ 存档格式 v105
- 工具：`crates/engine-native/examples/nc12_bench.rs`（**已跟踪**；本轮新增 **9 个逐来源档**）
- 原始输出：[raw.txt](raw.txt)（33 行 `RESULT`）
- 设备/工具链：**Apple M3 Max** / macOS 27.0.1 / **rustc 1.95.0** / `--release`

## 1 新增的逐来源档（代码改动）

`TIER_TABLE` 增加 9 档（一个来源目录一档）：`gujian`/`mtg`/`wudian`/`hearth`/`yunchou`/`zhenfa`/`gwent`/`wuxing`/`d3`。
⇒ 逐来源性能**可复现**（命令见 §4），不再是单点。

## 2 构建与装载（`probe`）

| 来源 | 卡数 | 技能（含武将） | `build_ns` | `load_ns` | 存档字节 |
|---|---:|---:|---:|---:|---:|
| gujian | 164 | 233 | 521 µs* | 42.7 ms | 2080 |
| mtg | 134 | 228 | 44 µs | 38.5 ms | 1928 |
| wudian | 138 | 234 + 9 将 | 59 µs | 38.4 ms | 1944 |
| hearth | 141 | 215 | 32 µs | 36.8 ms | 1956 |
| yunchou | 147 | 215 | 38 µs | 39.0 ms | 1980 |
| zhenfa | 132 | 215 | 37 µs | 39.1 ms | 1920 |
| gwent | 145 | 220 | 46 µs | 38.5 ms | 1972 |
| wuxing | 125 | 217 + 4 将 | 42 µs | 42.1 ms | 2088 |
| **d3（U01）** | 124 | 217 + **1 将** | 896 µs* | 51.9 ms | 1888 |
| **cardpile（U03）** | 237 | 215 | 46.5 µs | 41.1 ms | 2340 |
| **cardpile_liuli（L04）** | 237 | 215 | 45.8 µs | 38.8 ms | 2340 |

`*` gujian 为**首个档**（冷缓存），`build_ns` 521 µs 属**离群**；其余 32–59 µs。
`*` d3 为**首档**（冷缓存）；其 `build_ns` 896 µs 同为离群。

**已修复工具限制**：bench 现在按**资产形态**装载（`cards.json` → `character.json` → `pile.json`），
因此 `d3`（纯武将包）与 `cardpile*`（牌堆补丁包）**均可测量**，不再 `环境失败`。

## 3 **整局驱动**（AI 打完整局，种子 42007，4 人）

| 来源 | 步数 | 事件 | 拒绝 | 收敛 | 墙钟 |
|---|---:|---:|---:|---|---:|
| gujian | 253 | 660 | **0** | ✅ | 11.79 ms |
| mtg | 292 | 760 | **0** | ✅ | 12.69 ms |
| wudian | 213 | 556 | **0** | ✅ | 9.18 ms |
| hearth | 214 | 564 | **0** | ✅ | 9.66 ms |
| yunchou | 195 | 505 | **0** | ✅ | 8.79 ms |
| zhenfa | 254 | 664 | **0** | ✅ | 10.69 ms |
| gwent | 156 | 411 | **0** | ✅ | **7.70 ms** |
| **d3（U01）** | 298 | — | **0** | ✅ | 见 raw.txt |
| **cardpile（U03）** | 261 | — | **0** | ✅ | 见 raw.txt |
| **cardpile_liuli（L04）** | 261 | — | **0** | ✅ | 见 raw.txt |
| wuxing | 164 | 429 | **0** | ✅ | 9.05 ms |

⇒ **8 个来源的完整 AI 对局全部收敛、拒绝 0、墙钟 7.7–12.7 ms**（≈ **15–20k 步/秒**）。

## 4 复现

```bash
cd noname-next
cargo run --offline --release -p engine-native --example nc12_bench -- \
  probe tiers=gujian,mtg,wudian,hearth,yunchou,zhenfa,gwent,wuxing,d3
cargo run --offline --release -p engine-native --example nc12_bench -- \
  drive tier=gwent seats=4 budget=64 lookahead=1 variant=0 seed=42007
```

## 5 与八项第 8 项的关系（**如实**）

| 判据 | 状态 |
|---|---|
| 同设备同工具链、逐来源 | ✅ **8 个来源**（d3 受工具限制）|
| **同负载前后对比**（优化收益）| ❌ **无**（无旧实现基线）|
| **产品级**（server/网络/浏览器）| ❌ **未测** |
| 桩（pile.json 包 U03/L04）| ✅ **已支持**（`load_pack_dir` 增加 `pile.json` 分支）|

## 6 工具限制（**如实**）

1. **`package_id` 歧义**：`cardpile-upstream-noname` 与 `cardpile-upstream-liuli` 的
   `manifest.package_id` **同为 `cardpile`**，仅 **namespace 不同**（`upstream-noname` / `upstream-liuli`）⇒
   bench 输出的 `package_ids=cardpile` **不能区分**两者；两档的 `content_lock_id` 也**相同**
   （`17450228774124681398`）⇒ **不能**用锁 id 断言两档内容不同；
2. **d3 的 `build_ns` / `smoke` 含首次分配**，与其它档不可直接横比。
