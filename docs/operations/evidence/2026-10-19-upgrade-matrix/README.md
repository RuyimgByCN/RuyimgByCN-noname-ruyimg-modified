# 逐包「升级」矩阵：**13/13 包通过**（2026-10-19）

- 工作包：**NC08 / 八项第 2 项（升级）**；规则版本 v322（工作树）/ 存档格式 v105
- 用例：`crates/engine-native/tests/nc08_upgrade_matrix.rs`
- 原始输出：[run.txt](run.txt)

## 1 判据（每包）

1. v1 建局并 `save()`；
2. **v2（仅 `def.version + 1`）恢复必须被拒** ⇒ `SnapshotError::RuleVersionMismatch`；
3. **v1 回滚恢复成功**且 `state_hash` 与保存前**一致**。

## 2 结果

```
DBG 升级矩阵：shahai-gujian-upstream-noname  v1→v2 拒版 ✅、回滚 ✅
DBG 升级矩阵：shahai-mtg-upstream-noname     v1→v2 拒版 ✅、回滚 ✅
DBG 升级矩阵：shahai-swd-upstream-noname     v1→v2 拒版 ✅、回滚 ✅
DBG 升级矩阵：shahai-hearth-upstream-noname  v1→v2 拒版 ✅、回滚 ✅
DBG 升级矩阵：shahai-yunchou-upstream-noname v2→v3 拒版 ✅、回滚 ✅
DBG 升级矩阵：shahai-zhenfa-upstream-noname  v1→v2 拒版 ✅、回滚 ✅
DBG 升级矩阵：shahai-gwent-upstream-noname   v1→v2 拒版 ✅、回滚 ✅
DBG 升级矩阵：huanle-upstream-noname         v1→v2 拒版 ✅、回滚 ✅
DBG 升级矩阵：wuxing-upstream-liuli          v1→v2 拒版 ✅、回滚 ✅
DBG 升级矩阵：wudian-upstream-noname         v2→v3 拒版 ✅、回滚 ✅
DBG 升级矩阵：d3-upstream-noname             v1→v2 拒版 ✅、回滚 ✅
DBG 升级矩阵：cardpile-upstream-noname       v1→v2 拒版 ✅、回滚 ✅
DBG 升级矩阵：cardpile-upstream-liuli        v1→v2 拒版 ✅、回滚 ✅
DBG 升级矩阵：通过 13/13 个包
```

| 资产形态 | 覆盖 |
|---|---|
| `cards.json`（+ `character.json`） | ✅ 8 包 |
| 仅 `character.json` | ✅ `d3` |
| `pile.json` | ✅ `cardpile` ×2 |
| `cards.json` + 特殊前缀 | ✅ `wuxing`(liuli)、`huanle` |

## 3 意义

- 第 433 轮只覆盖 **5 个来源**（各取一包）；本轮扩到**全部 13 个可装载包** ⇒ 「升级」项**按包覆盖完成**；
- 全部包都**拒绝了版本号 +1 的恢复** ⇒ 内容锁的**版本绑定**在所有资产形态上都生效（不是只在 cards 包生效）。

## 4 边界（**如实**）

1. 只验**版本号变化**；**同版本内容漂移**由 `nc08_upgrade_skill_mapping`（未登记 `mapping` 桥接即拒）覆盖；
2. 未覆盖「**包被删除/降级后旧存档的行为**」；
3. 未覆盖**跨格式**（如 cards→pile 的同名包替换）。
