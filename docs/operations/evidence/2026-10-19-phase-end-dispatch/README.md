# `PhaseEnd` 派发面探针（第 57 项影响面核对，2026-10-19）

- 工作包：**NC08**；规则版本 v322（工作树）/ 存档格式 v105

## 问题

第 57 项发现 `baishouzhihu` 的 `TriggerMeta` 注册 `EventKind::PhaseEnd`，而 filter 要求 `begin == true`。
需先确认：**`PhaseEnd` 是否整体不可用**（若是，则 14 个技能全是死技能，影响面远大于一个法器）。

## 探针结果

```
DBG PhaseEnd：目录注册 PhaseEnd 技能 14 个（其中进入 candidates 14 个）；
             candidates(PhaseEnd) 共 15；PhaseBegin 技能 40 → candidates 41
```

| 检查 | 结果 |
|---|---|
| 目录中注册 `PhaseEnd` 的技能 | **14 个** |
| 其中进入 `candidates(PhaseEnd)` | **14 / 14**（派发表完整）|
| 引擎是否发射 `begin: false` 的 `Phase` 帧 | **是**（`kernel/phase.rs:125`）|

⇒ **`PhaseEnd` 派发面正常**，14 个技能都**不是**死技能。

## 对第 57 项的结论

`baishouzhihu` 不触发是**单技能**的「元数据（`PhaseEnd`）与过滤器（`begin == true`）自相矛盾」，
**不是**引擎缺陷。修法仍只有一行（`event: EventKind::PhaseEnd` → `PhaseBegin`，并建议按上游 `discardEnd`
改判阶段名），但文件**未跟踪** ⇒ 受阻。
