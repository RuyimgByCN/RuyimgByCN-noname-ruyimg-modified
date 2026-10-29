# NC08 第⑦层样本 99：**来源 U05（古剑）** 基本牌 `yunvyuanshen`（玉女元参）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`（`yunvyuanshen`）：

```js
yunvyuanshen: {
    type: "basic", enable: true, logv: false,
    filterTarget(card, player, target) { return !target.hasSkill("yunvyuanshen_skill"); },
    async content(event, trigger, player) {
        target.storage.yunvyuanshen_skill = game.createCard("yunvyuanshen");
        target.addSkill("yunvyuanshen_skill");        // 授予子技
        …
    },
},
```

## 2 验收结果（`nc08_upstream_gujian_yunvyuanshen.rs`）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `target.addSkill("yunvyuanshen_skill")` | **行为面**：发动后 **1 号子技候选命中 = `["gj_yunvyuanshen_listener"]`** ⇒ **子技确已授予** | 通过 |

**原生实现引用**：`content/host/step/gujian.rs:220`（食物式机器：
`CardKind::Yunvyuanshen => (GJ_YUNVYUANSHEN_LISTENER_KEY, GJ_FOOD_ROUNDS_DEFAULT)`）；
`vocabulary/host/constants.rs:136`（`GJ_YUNVYUANSHEN_LISTENER_KEY = "gj_yunvyuanshen_listener"`）；
`vocabulary/host/keys.rs:357`（`HostBehavior::GujianYunvyuanshen => "host:gujian_yunvyuanshen"`）；
`filtering/common.rs:625`（子技触发：**自己濒死**）。

## 3 方法学更正（**同日，第 8 次**）

初版用**猜测的 5 个候选名**（`yunvyuanshen_skill` 等）探测 → **全部未命中** ✗ → 一度判定「子技未授予」。
**根因**：原生用的是 **`gj_yunvyuanshen_listener`**（**listener** 而非 **skill**）——
**猜名探测会漏**。

**纪律（固化）**：判定「技能是否授予」时，**先读原生的键常量**（如 `constants.rs` 的
`*_LISTENER_KEY`/`*_KEY`），**再按该名探测**；**不要仅凭上游键猜名**。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **1 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 子技**效果**（自己濒死时触发，`filtering/common.rs:625`）的行为面
- [ ] `storage` 面（上游存入一张生成的牌）
- [ ] 古剑包其余卡牌
