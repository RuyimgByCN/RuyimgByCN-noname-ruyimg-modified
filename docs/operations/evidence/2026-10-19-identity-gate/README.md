# 逐包**版本闸门（identity 级）**：**41/41 通过**（2026-10-19）

- 工作包：**NC08 / 八项第 2 项（升级）与第 5 项（保存恢复）的版本面**
- 用例：`crates/engine-native/tests/nc08_identity_gate_matrix.rs`
- 原始输出：[run.txt](run.txt)

## 1 为什么要做这一层

现有 [升级矩阵](../2026-10-19-upgrade-matrix/README.md) 与[保存恢复矩阵](../2026-10-19-save-restore-matrix/README.md)
只覆盖 **22 个「有引擎资产」的包**（cards/character/pile）。其余 **19 个包**（config/UI/VM 类）**没有引擎内容**，
无法建局 ⇒ 在「八项」里被记为「**适用面边界**」。

但**内容锁的版本闸门**并不依赖资产：它按 **`ExtensionDef.version`** 判定。因此可以用**每个包真实的
`package.npack.json`**（`namespace`/`package_id`/`version`）构造一个**空内容**的 `ExtensionPack`，
验证「**错版恢复被拒**」。

## 2 判据（每包）

1. 读 `package.npack.json` ⇒ `(namespace, package_id, version)`；版本按
   **`major*10000 + minor*100 + patch`** 映射为 `u32`（`"1.0.0"` ⇒ `10000`）；
2. 用该身份 + **空内容**构造 `ExtensionPack`（`identity-gate`）⇒ 建局 ⇒ `save()`；
3. **`version + 1` 恢复必须被拒** ⇒ `SnapshotError::RuleVersionMismatch`；
4. **`version` 回滚恢复成功**且 `state_hash` 与保存前**一致**。

## 3 结果

```
DBG identity 闸门：目录 41、通过 41、跳过 0、失败 0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 finished in 0.04s
```

| 判据 | 结果 |
|---|---|
| 包目录数 | **41** |
| **通过** | **41 / 41** ✅ |
| 跳过 | **0** |
| 失败 | **0** |

## 4 口径（**必须读**）

| 层级 | 含义 | 覆盖 |
|---|---|---|
| **asset 级** | 真实装载 cards/character/pile 内容，验证错版拒绝 + 回滚 | **22 个包**（[升级矩阵](../2026-10-19-upgrade-matrix/README.md)）|
| **identity 级（本表）** | 用**真实包身份** + **空内容**，只验证**版本闸门** | **41 个包（含 22）** |

⇒ **「41/41」不等于「41 个包都能升级/恢复」** ❗ —— 它等于「**41 个包的版本闸门都生效**」✅。
**按来源计**：`22 asset 级包` 覆盖 **13 个来源**；`19 identity 级包` 覆盖 **另外 19 个来源** ⇒
**32/32 来源都有「版本闸门」证据**，但**只有其中 13 个是 asset 级**。

## 5 边界

1. **未验证**空内容包在**真实安装/装载**路径下的行为（那属 `native.rs`/store 面，见[安装矩阵](../2026-10-19-install-matrix/README.md)）；
2. **未验证**「包被删除后旧存档」与「跨格式替换」；
3. `ExtensionDef.name` 需要 `&'static str` ⇒ 测试里用 `Box::leak`（**仅测试**，不影响产品）。
