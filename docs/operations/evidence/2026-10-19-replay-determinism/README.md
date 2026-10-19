# 回放确定性（差异 #3「余随机代抽」的 RNG 面，2026-10-19）

- 工作包：**NC08 / 扩展台账**；规则版本 v322（工作树）/ 存档格式 v105
- 用例：`crates/engine-native/tests/nc08_replay_determinism_five_sources.rs`

## 背景

U07 差异 #3：「守卫弃牌上游一次性选 n 张——**原生首张自选 + 余随机代抽**」。
「随机」只要**由引擎 RNG 驱动**，在**同种子**下就应**完全可复现**（否则回放会漂移）。

## 结果（**4/4 通过**）

```
DBG 回放确定性：cardpile-upstream-noname 同 seed 一致=true；异 seed 不同=true
DBG 回放确定性：shahai-swd-upstream-noname 同 seed 一致=true；异 seed 不同=true
DBG 回放确定性：d3-upstream-noname 同 seed 一致=true；异 seed 不同=true
DBG 回放确定性：wudian-upstream-noname 同 seed 一致=true；异 seed 不同=true
```

| 包 | 同 seed 轨迹一致 | 异 seed 状态不同 |
|---|---|---|
| `cardpile-upstream-noname`（U03，`pile.json` 管线） | ✅ | ✅ |
| `shahai-swd-upstream-noname`（U05） | ✅ | ✅ |
| `d3-upstream-noname`（U01） | ✅ | ✅ |
| `wudian-upstream-noname`（U07） | ✅ | ✅ |

## 判据

1. **同 seed + 相同指令序列** ⇒ `state_hash` **必须相等**（回放可复现）；
2. **不同 seed** ⇒ `state_hash` **应不同**（**反向对照**，确认 seed 真的影响状态，避免「因为没随机所以一致」）。

## 结论

差异 #3 的「随机代抽」**不破坏回放确定性**：随机**由种子决定**，同种子完全可复现 ✅。
⇒ 该差异属「**与上游 RNG 流不可比**」，**不属**「回放不可复现」。

## 边界（**如实**）

1. **未验证**与上游 `Math.random` 流**一致**（不可能，上游无种子）；
2. 只跑 `NextPhase` 序列（**未触发**守卫弃牌的具体路径）——本用例验证的是**引擎级**确定性，
   **不是**「守卫弃牌在多次运行中选取同一张」的**单点**验证。
