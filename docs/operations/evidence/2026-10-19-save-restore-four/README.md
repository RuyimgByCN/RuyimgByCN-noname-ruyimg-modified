# 保存恢复补充：四来源单进程往返（2026-10-19）

- 工作包：**NC08 / 八项验收第 5 项**；规则版本 v322（工作树）/ 存档格式 v105
- 用例：`crates/engine-native/tests/nc08_save_restore_four_sources.rs`

## 结果

| 包（来源） | 结果 | 说明 |
|---|---|---|
| **`d3-upstream-noname`（U01）** | ✅ **往返成功** | `state_hash` 一致 + 恢复后可继续推进 |
| **`wudian-upstream-noname`（U07）** | ✅ **往返成功** | 同上 |
| `cardpile-upstream-noname`（U03） | ⏭ **跳过** | 资产为 **`pile.json`**（**牌堆补丁包**），**不在 card/general 管线** |
| `cardpile-upstream-liuli`（L04） | ⏭ **跳过** | 同上 |

## 判据

1. `e.save()` 成功；
2. `Engine::restore_with_extensions(&snap, &[&pack])` 成功；
3. **恢复后 `state_hash()` 与保存前一致**；
4. 恢复后**可继续推进**（`NextPhase` 不报错）。

## 边界（**如实**）

1. **单进程往返**，**弱于** U05 的**双进程接力**（`t12-g5-batch2`）；
2. **`cardpile`（U03/L04）未覆盖**——其资产是 `pile.json`（牌堆补丁），需按**该管线**单独写用例；
3. 未覆盖「**错版/缺包拒绝**」（U05 接力批已有）。

## 结论

**U01/U07 的保存恢复证据补齐**（单进程档）⇒ 五来源签署表中「保存恢复」由 **1/5 → 3/5**；
**U03/L04 仍缺**（需 `pile.json` 管线用例）。
