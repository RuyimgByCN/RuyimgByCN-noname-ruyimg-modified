# 对局机器人（`legal_actions` 驱动）——**部分完成 + 诊断**（2026-10-19）

- 工作包：**NC08 / 产品级长程**；规则版本 v322（工作树）/ 存档格式 v105
- 用例：`apps/game-server/tests/long_run_bot.rs`（**已跟踪**；当前 `#[ignore]`）

## 1 做了什么

用 `engine-api` 提供的 **`LegalAction::to_command(actor, expected_revision)`** 把服务端下发的
**权威合法动作**直接转成命令，从而越过「只发 `EndPhase` 会卡在等待选择」的限制
（见[上一轮 soak](../2026-10-19-product-soak/README.md)）。

选步策略（**不是 AI**，只求推进）：优先 `PlayCard`/`ActivateSkill`/`UseSkillCard` →
`EndPhase` → `ResolveChoice{cancelled:true}`；**被拒后轮换**下一个合法动作。

## 2 实测（4 房间 × 40 轮 × 6 步 = 960 条）

```
DBG bot 引导：room 0 rev=2 legal_actions=6      （引导步：先发 EndPhase 换取 legal_actions）
RESULT bot rooms=4 rounds=40 per_round=6 submitted=960 accepted=14 snapshots=160
       max_revision=9 elapsed_ms=583 throughput_cmds_per_s=1644
```

| 判据 | 结果 |
|---|---|
| 引导步能拿到 `legal_actions` | ✅（6–7 条）|
| 机器人能把对局推进到 `rev > 2` | ✅（**rev=9**）|
| **继续推进** | ❌ **卡在 rev≈9**（960 条中仅 **14** 条被接受）|
| 房间始终可响应（160 次快照）| ✅ |

## 3 诊断（**未解决的部分**）

1. **`ResolveChoice` 的 `selections: Vec<u32>`**：当前只能发 `cancelled: true`，
   若某步必须选择具体项，则**无法构造合法选择** ⇒ 被拒；
2. **actor 归属**：`legal_actions` 是**按观察者**下发的，但**轮到谁行动**会变化；
   当前固定用 `player1` 作 `actor`，遇到「需要 player2 应答」的窗口时**必然被拒**；
3. 被拒后**轮换**能避免死磕，但**不能**在语义上纠正上面两条。

## 4 结论与待办

- **交付**：机器人骨架 + `legal_actions` 通路 + 引导步 + 房间探活（**可复用**）；
- **未完成**：**整局连续推进**。待办（按优先级）：
  - [ ] 解析 `ChoiceRequest`/`ResolveChoice` 的**合法 selections 集合**，构造精确选择；
  - [ ] 按**当前应行动者**设置 `actor`（可用 `ObserverSnapshot.current_player` 或 choice 的 observer）；
  - [ ] 完成后把本用例从 `#[ignore]` 打开，并把断言门槛提到 `max_revision >= 20`。

## 5 复现

```bash
cd noname-next
cargo test --offline -p game-server --test long_run_bot -- --ignored --nocapture
```
