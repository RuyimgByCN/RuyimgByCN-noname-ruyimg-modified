# 逐包「保存恢复」矩阵：**13/13 包通过**（2026-10-19）

- 工作包：**NC08 / 八项第 5 项（保存恢复）**；规则版本 v322（工作树）/ 存档格式 v105
- 用例：`crates/engine-native/tests/nc08_save_restore_matrix.rs`
- 原始输出：[run.txt](run.txt)

## 1 判据（每包）

1. 建局并推进 **3 步**（制造非平凡状态）；
2. `save()` → `restore_with_extensions(&snap, &[&pack])` **成功**；
3. **`state_hash` 与保存前一致**；
4. 恢复后**仍可继续推进**。

## 2 结果

```
DBG 保存恢复矩阵：shahai-gujian-upstream-noname 往返 ✅（hash 一致、可继续推进）
… （共 13 行）
DBG 保存恢复矩阵：通过 13/13 个包
```

| 资产形态 | 覆盖 |
|---|---|
| `cards.json`（+ `character.json`） | ✅ 8 包 |
| 仅 `character.json` | ✅ `d3` |
| `pile.json` | ✅ `cardpile` ×2 |
| `cards.json` + 特殊前缀 | ✅ `wuxing`(liuli)、`huanle` |

## 3 意义

- 第 431/432 轮覆盖 **U01/U03/U05/U07/L04 五个来源**（各一包）；本轮扩到**全部 13 个可装载包**；
- 与[升级矩阵](../2026-10-19-upgrade-matrix/README.md)**同构**：升级验「错版被拒 + 回滚」，本表验「同版本往返 + 可继续」。

## 4 边界（**如实**）

1. **单进程往返**；**双进程接力**见 `t12-g5-batch2` 证据（8 包）；
2. 未覆盖**错版/缺包拒绝**（由 `nc08_upgrade_matrix` 覆盖）；
3. 未覆盖**长时反复 save/restore**（内存面见 `2026-10-19-product-memory`）。
