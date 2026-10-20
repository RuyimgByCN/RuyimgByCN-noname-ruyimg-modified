# 包 → 来源映射（**可复现**，2026-10-19）

> **脚本**：`python3 -B scripts/map_package_sources.py [--tsv F] [--check]`
> **数据**：[map.tsv](map.tsv)（来源 / 名称 / 包数 / 包列表）

## 1 结论

| 项 | 值 |
|---|---:|
| 台账来源 | **37** |
| `packages/extensions/` 包目录 | **41** |
| **映射到来源** | **32** ✅ |
| **未映射的包** | **0** ✅ |
| **无包的来源** | **5** |

**无包的 5 个**：`L08` 十周年UI、`L11` 太虚幻境、`L12` 如真似幻、`L14` 手杀ui、`L25` 祖安武将（G0 完成、G1 未做）。

⇒ **32 = 37 − 5** —— 「八项按来源」的分母来历**完全复现** ✅

## 2 为什么需要专门的脚本

**`boss`/`cardpile`/`coin` 各有两个同名来源**（noname 与 liuli）。
**按目录名匹配会把 liuli 的包记到 noname 上**：
`cardpile-upstream-liuli` → 错记 `U03`（应 `L04`）、`boss-play-upstream-liuli` → 错记 `U02`（应 `L03`）。

**消歧规则**（脚本内固化）：

1. **包名后缀 `-noname` / `-liuli`** 决定仓库侧，再用 `upstream_path` 的目录名匹配；
2. `upstream_path` 为 `None` 的 6 个包，用 `EXPLICIT` 表（包名 → 来源 ID）；
3. 匹配不到 ⇒ 报「未映射」，**不猜**。

## 3 多包来源

| 来源 | 包数 |
|---|---:|
| **U05 杀海拾遗** | **9** |
| **L03 boss** | **2**（`boss-play-upstream-liuli` + `boss-zhugeliang-upstream-liuli`）|
| 其余 30 个 | 各 1 |

⇒ **`boss-zhugeliang` 归属 `L03`** ✅ ⇒ **包数合计应为 41（非 40）**、**组合应为 12（非 11）**、**性能应为 14（非 13）** ⚠️
（**本会话不改验收表**，三处待改见[交接索引 §7.10](../2026-10-19-session-handover/README.md)）

## 4 建议

**在每个包的 `package.npack.json` 加 `source_id` 字段** ⇒ 从此不依赖目录名与后缀匹配，
**同名来源也不会再歧义**。包数据是他人维护的，本会话只记建议。
