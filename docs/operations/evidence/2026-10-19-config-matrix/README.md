# 逐包「配置」矩阵：**41/41 manifest 合格**（2026-10-19）

- 工作包：**NC08 / 八项第 3 项（配置）**；规则版本 v322（工作树）/ 存档格式 v105
- 用例：`crates/engine-native/tests/nc08_manifest_matrix.rs`
- 原始输出：[run.txt](run.txt)

## 1 判据（每个 `packages/extensions/*/`）

1. `package.npack.json` **可解析**，`manifest` 段齐全（`namespace` / `package_id` / `version` 非空）；
2. **`(namespace, package_id)` 全库唯一**（`package_id` 允许**跨 namespace 重名**——`cardpile`/`boss-play`/`coin`）；
3. **`namespace` 与目录名一致**（`*-upstream-noname` ⇒ `upstream-noname`；`*-upstream-liuli` ⇒ `upstream-liuli`）；
4. `assets/config.json`（若有）**可解析**且为**非空对象**。

## 2 结果

```
DBG 配置矩阵：目录 41 个、manifest 合格 41 个、含 config.json 25 个、问题 0 项
```

| 判据 | 结果 |
|---|---|
| 包目录数 | **41** |
| manifest 合格 | **41 / 41** ✅ |
| **带 `assets/config.json`** | **25** 个 ✅（全部可解析且非空）|
| `(namespace, package_id)` 重复 | **0** ✅ |
| namespace 与目录名不一致 | **0** ✅ |
| **问题总数** | **0** ✅ |

## 3 与既有配置验收的关系

| 验收 | 范围 | 内容 |
|---|---|---|
| `nc08_config_acceptance`（既有） | **19 个卡牌包** | **装载层面**：元数据交叉核对、`enabled` 开关 214→124、`init_hook` 实际取值（`swd_jiqi_held`/`wuxing_natures`）、3 种非法数据被拒 |
| **本矩阵（新增）** | **41 个包目录** | **元数据/配置文件形状**：manifest 齐全、`(ns,id)` 唯一、namespace 与目录名一致、`config.json` 可解析 |

⇒ 两者**互补**：前者验「装载行为」，后者验「**全部包**的元数据契约」。
**「配置」项覆盖由 19 包 → 41 包**。

## 4 边界（**如实**）

1. 只验**形状与唯一性**；**不验配置项语义**（各包语义差异写在[规则基准](../../architecture/native-rules-baseline.md)）；
2. `enabled` 开关的**运行时效果**仍由 `nc08_config_acceptance` 覆盖（19 包）；
3. 未覆盖**运行时改配置后持久化**（已登记待办）。
