# 37 来源覆盖三分：**12 已转换 / 20 有包零条目 / 5 无包**（2026-10-19，第 759 轮）

> 数据：[coverage.tsv](coverage.tsv)（`source, packages, entries, upstream_path, class`）
> 生成：`python3 -B scripts/report_source_coverage.py --write ...`
> 校验：`python3 -B scripts/report_source_coverage.py --check`（类别数漂移即 exit 1）

## 1 三分

| 类 | 条数 | 含义 |
|---|---:|---|
| **`converted`** | **12** | 包内**有卡/技/将条目** ⇒ 「规则差异」**可比对** |
| **`installed-empty`** | **20** | 包在（可安装），但**条目 = 0** |
| **`no-package`** | **5** | `L08 L11 L12 L14 L25` |
| **合计** | **37** | ✅ |

## 2 为什么这 20 个是「未转换」而不是「规则差异」

抽查三例，包内 `upstream_path` 指向**模式扩展**，条目却是 0：

| 来源 | `upstream_path` | 条目 |
|---|---|---:|
| **L21 武皇争霸** | `noname/extension/武皇争霸/extension.js` | 0 |
| **L28 雷霆万钧** | `noname/extension/雷霆万钧/extension.js` | 0 |
| **U02 boss-play** | `apps/core/extension/boss/extension.js` | 0 |

⇒ **上游有规则逻辑，转换包一条都没搬** ⇒ 属**覆盖缺口**（记在「来源覆盖」项），
**不属「规则差异」**（后者要求「已对照」）✅

## 3 三个「纯配置型」扩展连 `upstream_path` 都没有

`installed-empty` 里**3 条无 `upstream_path`**，它们的包内只有 `config.json`：

| 来源 | 包 | `config.json` 的键 | 性质 |
|---|---|---|---|
| **L07** | `omnisearch-upstream-liuli` | `css`, `background` | **搜索 UI** |
| **L23** | `desktop-adapt-upstream-liuli` | `device_profile`, `layout`, `tap_threshold_px`, `presets`, `emoji_bridge` | **桌面适配** |
| **U04** | `coin-upstream-noname` | `coin`, `coin_display_playpackconfig`, `snowFall`, `coinSnowSize` | **金币特效** |

⇒ **三者既无可转换条目，也无可追溯上游** ⇒ **是允许的例外集合**（单测固定为这 3 个）✅
⇒ **其余 17 条 `installed-empty` 都有 `upstream_path`** ⇒ **可按路径去搬**

## 4 对「规则差异」分母的影响

| 读法 | 分母 | 说明 |
|---|---:|---|
| 保守（现状） | 22–24 | 含未转换来源 |
| **本文主张** | **12** | 只算 `converted` |
| 不可取 | 32 | 把 ② 类算进来（无内容可比） |

⇒ **改分母是 D 类决策**（对外可见），**本文只提供证据，不单方面改** ⚠️
⇒ 见[签署表附 22](../2026-10-19-source-signoff-32/README.md)
