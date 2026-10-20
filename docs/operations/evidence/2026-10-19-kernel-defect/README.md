# ⚠ **内核级缺陷档案**：无懈可击窗口中的悬空引用（2026-10-19）

> **这是本会话发现的最高优先级问题**：它不是功能缺失，而是**内核不变量被破坏**
> （模拟提交时引用了「已不在手牌里的牌」）。修复前**不应**把任何「全绿」当作完成证据。

- 复现用例：`crates/engine-native/tests/nc12_dangling_reference_repro.rs`（`#[ignore]` 的**缺陷守护**）
- 扫描用例：`crates/engine-native/tests/nc12_dangling_reference_probe.rs`
- 原始输出：[repro-run.txt](repro-run.txt)
- 状态：**未修复**（修复需改 `kernel/**`，本工作树中为**未跟踪文件** ⇒ 本代理不得修改）

## 1 复现参数（**确定性**）

| 项 | 值 |
|---|---|
| 内容档 | `shahai-swd-upstream-noname`（单包）|
| 座位 | **4** |
| 策略 | variant **0** / budget **8** / lookahead **1** |
| seed | **44170** |
| 触发步 | **第 10 步**（`rev=10`、`phase=Play`、**seat 1** 回答 **`choice_id=2`**）|
| 触发动作 | `Answer { choice_id: 2, selection: [Pass] }` |
| 故障 | 模拟提交触发内核不变量失败（`card … not in hand` 一类）|

## 2 故障现场（原始输出摘录）

```
[fault]   p1: hp=4/4 hand=[23, 15, 72, 10] equips=[] judge=[] off=[]
[fault]   frame[0] id=23 parent=None kind=CardUsed stage=Intrinsic
          data=CardUsed(CardUseData { card: 161, player: 1, targets: [0], nature: Normal, … })
[fault]   park[0] resume_depth=1 slot=Intrinsic { frame: 23 }
          snapshot=NullifyWindow { card: 161, source: 1, nullified: false, queue: [1, 2, 3, 0],
                                  asked: true, window_open: true, owns_disposal: true, … }
[sim-fault 0] PRE  rev=10 phase=Play action=Answer { choice_id: 2, selection: [Pass] }
[sim-fault 1] POST seat=1 rev=10 … table=[161] hands=[p0:hp=3/4 …]
assertion `left == right` failed: 内核悬空引用复现（seed 44170）——现场已打印；修复后本断言应保持 0
  left: 1   right: 0
```

## 3 读现场能确定什么 / 不能确定什么

**能确定**：

1. 故障发生在 **`NullifyWindow`（无懈可击询问窗口）** 仍 `window_open: true`、`asked: true` 时；
2. 该窗口的 `card = 161`、`source = 1`、**`queue = [1,2,3,0]`**（询问队列）、`owns_disposal: true`；
3. 此时**栈上只有一帧**（`frame[0] id=23 CardUsed`），`park[0]` 把它的 `snapshot` 换成了 `NullifyWindow`；
4. 模拟提交的是 **`Pass`**（不是选牌）⇒ **故障与「选的牌」无关**，更像是**窗口恢复/栈回填时**丢失了引用一致性。

**不能确定**（**未做**）：

- 具体是**哪张牌**被判成「不在手牌」（原始 panic 文案在 `machine.rs:408`，本轮输出只抓到断言原文）；
- `queue` 里 `[1,2,3,0]` 与 `targets: [0]` 的对应关系是否就是根因；
- 是否**只在 AI 模拟路径**触发（真实提交路径未测同一局面）。

## 4 修复时需要什么（**给接手者**）

1. **入口**：`content/host` 的 `NullifyWindow` 恢复逻辑 + `machine.rs:408` 的不变量检查；
2. **判据**：本用例**转为通过**（`left == 0`），且 `nc12_probe_scans_seeds_for_dangling_reference` 不再命中；
3. **回归**：修复后应把两条 `#[ignore]` **摘掉**，让它们成为常规回归；
4. **前置**：需要把 `kernel/**`（本工作树 `??`）纳入可修改范围 —— **这是本代理无法跨越的边界**。

## 5 严重性判断（**如实**）

| 维度 | 评估 |
|---|---|
| 是否影响真实对局 | **未知**：目前只在**AI 模拟提交**路径复现；**真实提交路径未测同一局面** |
| 是否影响存档/重放 | **未知**（未测）|
| 是否会让进程崩溃 | **否**：它是**不变量断言**（`panic` 在测试里被捕获为 `sim_faults`），产品路径**未观察崩溃** |
| 优先级 | **最高**：内核不变量是「其余验收可信」的前提 |

> **结论**：这是一个**已固定、可复现、根因未定位**的内核缺陷。**本会话未修复**（受未跟踪文件边界限制）。
