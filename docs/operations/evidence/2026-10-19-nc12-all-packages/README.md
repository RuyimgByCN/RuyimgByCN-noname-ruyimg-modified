# 逐包效能跑批（全部 41 个包目录，2026-10-19）

- 工作包：**NC12 / 八项第 8 项（性能）**；规则版本 v322（工作树）/ 存档格式 v105
- 工具：`nc12_bench` **新增 `dir:<包目录名>` 档**（按 drop-in 目录名单包装载）
- 原始输出：[probe-all.txt](probe-all.txt)（41 行 `RESULT`）
- 设备/工具链：Apple M3 Max / macOS 27.0.1 / rustc 1.95.0 / `--release`

## 1 汇总：**22 个包可测 / 19 个包无卡牌或武将资产**

| 结果 | 数量 | 说明 |
|---|---:|---|
| `result=OK` | **22** | 有 `cards.json` / `character.json` / `pile.json` 之一 |
| `result=FAIL`（**环境失败**） | **19** | 资产目录里**没有**上述三种文件 ⇒ 属**配置/UI/VM/牌堆补丁类**包，**不适用**引擎级装载测量 |

## 2 可测包的装载数据（`probe`）

| 包 | cards | skills | 武将 | build_ns | load_ns |
|---|---:|---:|---:|---:|---:|
| `cardpile-upstream-liuli` | 237 | 0 | 0 | 671916 | 41075000 |
| `cardpile-upstream-noname` | 237 | 0 | 0 | 603167 | 41358458 |
| `shahai-swd-upstream-noname` | 214 | 32 | 79 | 943000 | 43384417 |
| `shahai-gujian-upstream-noname` | 164 | 4 | 0 | 771458 | 42659000 |
| `shahai-yunchou-upstream-noname` | 147 | 0 | 10 | 605708 | 44135375 |
| `shahai-gwent-upstream-noname` | 145 | 0 | 0 | 698708 | 39551541 |
| `shahai-hearth-upstream-noname` | 141 | 0 | 0 | 590750 | 40041708 |
| `wudian-upstream-noname` | 138 | 0 | 9 | 638542 | 42814958 |
| `shahai-mtg-upstream-noname` | 134 | 0 | 0 | 626833 | 43167083 |
| `shahai-zhenfa-upstream-noname` | 132 | 0 | 0 | 602166 | 59041542 |
| `huanle-upstream-noname` | 131 | 0 | 0 | 624917 | 41761875 |
| `wuxing-upstream-liuli` | 125 | 4 | 0 | 1024542 | 40845208 |
| `boss-zhugeliang-upstream-liuli` | 124 | 0 | 1 | 782209 | 40956208 |
| `d3-upstream-noname` | 124 | 0 | 1 | 785916 | 40597209 |
| `doomsday-brawl-upstream-liuli` | 124 | 0 | 0 | 597709 | 41047417 |
| `epic-cards-upstream-liuli` | 124 | 0 | 0 | 609125 | 43448500 |
| `hero-kill-upstream-noname` | 124 | 0 | 0 | 793375 | 42876833 |
| `longxue-upstream-liuli` | 124 | 0 | 1 | 799292 | 40578750 |
| `mark-supplement-upstream-liuli` | 124 | 0 | 0 | 608709 | 40633959 |
| `select-beautify-upstream-liuli` | 124 | 0 | 0 | 801792 | 42708541 |
| `shaai-ow-upstream-noname` | 124 | 0 | 24 | 623625 | 43773750 |
| `shaai-xianjian-upstream-noname` | 124 | 0 | 36 | 898209 | 43043875 |

## 3 **无卡牌/武将资产**的包（19 个，登记）

| 包 | 原因 |
|---|---|
| `ai-tuning-upstream-liuli` | 无 `cards.json` / `character.json` / `pile.json` |
| `boss-play-upstream-liuli` | 无 `cards.json` / `character.json` / `pile.json` |
| `boss-play-upstream-noname` | 无 `cards.json` / `character.json` / `pile.json` |
| `coin-upstream-liuli` | 无 `cards.json` / `character.json` / `pile.json` |
| `coin-upstream-noname` | 无 `cards.json` / `character.json` / `pile.json` |
| `desktop-adapt-upstream-liuli` | 无 `cards.json` / `character.json` / `pile.json` |
| `douzhuan-xingyi-upstream-liuli` | 无 `cards.json` / `character.json` / `pile.json` |
| `drag-read-upstream-liuli` | 无 `cards.json` / `character.json` / `pile.json` |
| `elo-sanction-upstream-liuli` | 无 `cards.json` / `character.json` / `pile.json` |
| `fool-plugin-upstream-liuli` | 无 `cards.json` / `character.json` / `pile.json` |
| `martial-upstream-liuli` | 无 `cards.json` / `character.json` / `pile.json` |
| `omnisearch-upstream-liuli` | 无 `cards.json` / `character.json` / `pile.json` |
| `qhly-upstream-liuli` | 无 `cards.json` / `character.json` / `pile.json` |
| `shousha-full-upstream-liuli` | 无 `cards.json` / `character.json` / `pile.json` |
| `shousha-mvp-upstream-liuli` | 无 `cards.json` / `character.json` / `pile.json` |
| `sj-settings-upstream-liuli` | 无 `cards.json` / `character.json` / `pile.json` |
| `skin-switch-upstream-liuli` | 无 `cards.json` / `character.json` / `pile.json` |
| `thunder-upstream-liuli` | 无 `cards.json` / `character.json` / `pile.json` |
| `update-guard-upstream-liuli` | 无 `cards.json` / `character.json` / `pile.json` |

## 4 判读（**如实**）

1. **22 个包**有引擎级装载与建局数据（此前 13 档）；
2. **19 个包**是**配置/UI/VM 类**（`ai-tuning`、`boss-play`、`coin`、`desktop-adapt`、`doomsday-brawl` 等）——它们**不产生卡牌/武将**，**引擎级性能无从测量**（这是**范围边界**，不是缺陷）；
3. 逐来源「性能」签署仍受限于：**无同负载前后对比**、**无产品级（server/浏览器）**数据。

## 5 复现

```bash
cd noname-next
for d in $(ls packages/extensions); do
  cargo run --offline --release -q -p engine-native --example nc12_bench -- probe tiers=dir:$d
done
```
