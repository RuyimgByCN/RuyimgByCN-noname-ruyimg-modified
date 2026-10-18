# NC08 第⑦层样本 16：`lingxue`（灵雪，U05 xianjian 武将包 / 星璇）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层——**首个 xianjian 武将包样本**
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项验收通过**（含 1 项源码事实固定）；**行为面（回复落地）本轮未验收，如实登记**

## 1 上游源码（归档快照，逐字引用；见 [upstream-lingxue.js.txt](upstream-lingxue.js.txt)）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js`：

```js
lingxue: {
    trigger: { player: "recoverEnd" },   // 自己回复体力结束后
    forced: true,                        // 锁定技
    content() { player.changeHujia(); }, // +1 点护甲
},
```

## 2 验收结果（`nc08_upstream_lingxue.rs`）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 3 | `content: changeHujia()` | 效果面**源码事实固定**：原生 `change_hujia(self.owner, 1)`；过滤臂绑定 `Recover` 且 `amount > 0` | 通过（源码事实） |
| 1 | `trigger: { player: "recoverEnd" }` | **未持技**者回复体力**不获得护甲**（技能门槛） | 通过 |
| 1+2+3 | 回复后 +1 护甲且无确认门 | **未验收（登记）** | — |

**行为面为何未验收（如实）**：夹具中 `debug_recover(0, 1)` 返回**接受**，但驱动到静止后
**体力未变**（回复事件经帧结算，本夹具未观察到落地）→ 该用例以 `#[ignore]` 守护，**不写成通过**。

**原生实现引用**：`content/host/filtering/xianjian.rs:407`（`Recover` 且 `amount > 0`）、
`content/host/step/xianjian_status.rs:6`（`ctx.change_hujia(owner, 1)`）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed / 1 ignored** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | 见提交说明 | — |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] 灵雪**行为面**夹具修复后补验收（回复事件的帧结算驱动）
- [ ] 样本 17+：其余来源（当前 **15/37 来源**）
- [ ] xianjian 其余 35 名武将的上游样本（本样本为 1/36）
- [ ] 逐命令对照工具

## 5 追加：行为面未验收的**根因已查明**（2026-10-19 同轮）

用最小探针（`Engine::new` + `debug_set_hp(0,2)` + `debug_recover(0,1)`）确认：

```
PROBE out=Accepted { state_hash: … } hp 2 -> 2 stack=0 choice=None
```

→ `debug_recover` **返回接受且状态摘要变化**，但**体力不变**、栈已清空（无待结算帧）。

**根因**：`debug_recover` 的语义是「**只发 `Recover` 事件、不施加体力变化**」——内核
`commit_recover_skills`（`kernel/events.rs:799`）只处理 `recoverEnd` 类**技能触发**，
体力变化按设计由**构造方**完成（`kernel.rs:38` 注释：「CardsDrawn/Recover/CardsMoved 等在构造时
已由助手完成区移动作」）。

**既有用例佐证（审计 13 处调用）**：`native_kernel/boss_nianshou.rs:33`、`boss_taotie.rs:124` 等
只用它**触发忍性/积财等 recoverEnd 技能**，**从不断言体力**——与上述语义一致，故不是引擎缺陷。

**结论与登记**：
- 灵雪行为面用例的**体力断言前提不成立**（已把 `#[ignore]` 原因改为该根因）；
- 要验收灵雪（`recoverEnd` 触发）须走**真实回复路径**（桃/技能回复）或新增「发事件 + 施加体力」
  的测试助手 → **登记为待办**；
- **命名/易误用风险登记**：`debug_recover` 名称暗示「回复」但只发事件，建议重命名或在文档注明
  （属测试面改动，待 host/kernel 拆分落定后统一处理）。
