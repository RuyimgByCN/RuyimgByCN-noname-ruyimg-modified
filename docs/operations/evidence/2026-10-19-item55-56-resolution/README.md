# 第 55/56 项**结案（非缺陷）**＋第 57 项登记（2026-10-19）

- 工作包：**NC08**；规则版本 v322（工作树）/ 存档格式 v105
- 结论：**第 55、56 项是「用例步数不足」的误判，实现本身正确**；同时**发现第 57 项真实缺陷**

## 1 关键事实（此前被忽略）

`EventData::Phase { begin: true }` **只在「回合起点的判定阶段」发射**：

```rust
// kernel/events.rs:753-757 与 :789-793（两处均为回合起点）
ctx.emit(EventData::Phase { player: next, phase: Phase::Judge, begin: true });
```

⇒ 建局时**第 1 轮的判定开始已经发过**，所以：

| 做法 | 结果 |
|---|---|
| 放牌后推进 16–32 步、停在 `Play`/`End`（**旧用例**） | **永远看不到阶段开始事件** ⇒ 0→0（**误判为派发缺陷**）|
| 推进到 **`round >= 2`** 的判定阶段（**新用例**） | **20 步**即到达 ⇒ 窗口正常触发 ✅ |

## 2 第 56 项（黄麟之琮）**已结案**

```
DBG 黄麟之琮：推进 20 步至 round 2
DBG 黄麟之琮：护甲 0 → 1（应 +1）
test upstream_huanglinzhicong_gains_hujia_on_phase_begin ... ok
```

⇒ **实现正确**：无护甲持有者在阶段开始 **hujia +1** ✅。`#[ignore]` 已移除。

## 3 第 55 项（青龙之圭）**已结案**

```
DBG 青龙之圭：询问=["discard_to_limit@obs0"]；牌堆 196 → 194（应 −2）；手牌 3 → 4（摸 2 弃 1 ⇒ 净 +1）
test upstream_qinglongzhigui_draw2_discard1 ... ok
```

⇒ **实现正确**：**摸 2 + 强制弃 1**（净 +1）✅。`#[ignore]` 已移除。

## 4 第 57 项（**新发现，真实缺陷**）：白兽之琥**永不触发**

```
DBG 白兽之琥：回合=14 阶段=Judge；窗口 prompt=None
```

推进到 **第 14 回合**的判定阶段仍**无窗口** ⇒ 与 55/56 的结论相反，**这是真实缺陷**。

**根因（源码级）**：

| 位置 | 内容 |
|---|---|
| `content/host/metadata/swd.rs:360` | `SwdBaishouzhihuHeld => TriggerMeta { event: EventKind::PhaseEnd, … }` |
| `content/host/filtering/swd.rs:409-421` | `EventData::Phase { player, begin, .. } if *player == owner && *begin && …` |

⇒ **触发元数据注册的是 `PhaseEnd`**，而**过滤器要求 `begin == true`**；
引擎的 `Phase` 事件**只有** `begin: true`（回合起点判定阶段）⇒ **两者永不匹配** ⇒ 技能**永不入队**。

**修法（受阻）**：把 `event` 改为 `EventKind::PhaseBegin`（并建议按上游 `discardEnd` 补「阶段名＝弃牌」判定，
避免当前「任意阶段开始都触发」的过宽口径）。**`content/host/metadata/` 未跟踪** ⇒ 与核心修复清单同因受阻。

## 5 方法学（**重要**）

1. **「事件已发射」≠「事件会在你观察的时刻发射」**：`Phase{begin:true}` 是**低频事件**（每轮一次），
   短步数用例**必然错过**；
2. **「同族通过/不通过」是最好的分辨器**：55/56 修好后 **57 仍失败** ⇒ 说明 57 的根因**不同**
   （元数据与过滤器自相矛盾），而不是同一个「派发缺陷」；
3. **撤回此前结论**：第 413–443 轮曾判定「宿主派发 `Phase{begin:true}` 失效 / 必须插桩」——
   **该结论错误**，实际是**用例驱动不足**。
