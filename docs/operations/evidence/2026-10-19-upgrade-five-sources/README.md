# 按来源的「升级」验收（L04/U01/U03/U05/U07，2026-10-19）

- 工作包：**NC08 / 八项验收第 2 项**；规则版本 v322（工作树）/ 存档格式 v105
- 用例：`crates/engine-native/tests/nc08_upgrade_five_sources.rs`

## 判据（**每个来源一个真实包**）

1. **v1 建局并 `save()`**；
2. **构造 v2**（仅 `def.version + 1`）并 `restore_with_extensions(&snap, &[&v2])`
   ⇒ **必须 `Err(SnapshotError::RuleVersionMismatch)`**（**错版拒绝，不静默兼容**）；
3. **用 v1 恢复** ⇒ 成功且 **`state_hash` 与保存前一致**。

## 结果（**5/5 通过**）

| 包（来源） | 错版拒绝 | 回滚恢复 |
|---|---|---|
| **`cardpile-upstream-liuli`（L04）** | ✅ | ✅ |
| **`d3-upstream-noname`（U01）** | ✅ | ✅ |
| **`cardpile-upstream-noname`（U03）** | ✅ | ✅ |
| **`shahai-swd-upstream-noname`（U05）** | ✅ | ✅ |
| **`wudian-upstream-noname`（U07）** | ✅ | ✅ |

## 实现要点

- **`pile.json` 包（U03/L04）** 需先经 `content::pile::extension_pack_from_pile` 构包再参与升级验收
  （与 card/general 管线不同）；
- **v2 只改 `def.version`**，内容不变 ⇒ 拒绝**确实来自内容锁的版本绑定**，而非内容哈希差异。

## 边界

- 只验证「**版本号变化**」；**同版本内容漂移**由 `nc08_upgrade_skill_mapping`（未登记 mapping 桥接即拒）覆盖；
- 未覆盖「**包被删除/降级后旧存档的行为**」——属另一面。
