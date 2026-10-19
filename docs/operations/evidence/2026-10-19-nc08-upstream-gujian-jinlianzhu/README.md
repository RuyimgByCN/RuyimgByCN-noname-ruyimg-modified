# NC08 第⑦层样本 87：**来源 U05（古剑）** 锦囊 `jinlianzhu`（金莲珠）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过（含行为面 · 打断）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`（`jinlianzhu`）：

```js
jinlianzhu: {
    type: "trick", fullskin: true, filterTarget: true,
    global: "g_jinlianzhu",                        // 全局子技：可在触发时机打出
    content() {
        var evt = event.getParent(3)._trigger;     // 取出被触发的那个事件
        evt.cancel();                              // 取消它
        if (evt.source) evt.source.draw();         // 来源摸 1
    },
},
```

## 2 验收结果（`nc08_upstream_gujian_jinlianzhu.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `global: "g_jinlianzhu"`（打断窗口） | **行为面**：1 号对 2 号出【杀】时，0 号（持金莲珠）出现打断询问 **`gj_jinlianzhu_confirm`**（`[Confirm, Pass]`）→ 确认后**生效** | 通过 |
| 2 | `evt.cancel()` + `evt.source.draw()` | **行为面**：确认后 **2 号 hp `4 → 4`**（伤害被**取消**）、**1 号手牌 `3 → 3`**（打出【杀】−1、**来源摸 1** +1） | 通过 |

**原生实现引用**：`content/host/factory.rs`（`HostBehavior::JinlianzhuCancel`：目标集合取伤害来源）。

## 3 夹具要点（**新增**）

- **打断窗口的形态**是 **`Confirm`/`Pass`**（询问名 **`gj_jinlianzhu_confirm`**），**不是**「选牌」——
  按「选项含该牌」筛选会**误判为不可达**（初版即如此）；
- 打断需**当前回合为攻击方**（本样本推进到 1 号出牌阶段再出杀）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 可被取消的**其它事件类型**（上游 `getParent(3)._trigger` 泛化）
- [ ] `ai.result` 策略面
- [ ] 古剑包其余卡牌
