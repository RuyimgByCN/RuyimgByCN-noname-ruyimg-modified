# NC08 第⑦层样本 83：**来源 U05（运筹）** 武将技 `guihan`（归汉）——**已注册但效果为空**（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（登记型）：上游**确有**定义；原生**已注册但效果分支为空**

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`（`guihan`）：

```js
guihan: {
    enable: "chooseToUse", skillAnimation: "epic", limited: true,   // 限定技
    filter(event, player) {
        if (event.type !== "dying") return false;   // 濒死时
        if (player !== event.dying) return false;   // 自己濒死
        return true;
    },
    filterTarget(card, player, target) { return target.hasSex("male") && player !== target; },
    content() {
        "step 0"; player.awakenSkill(event.name); player.recover();   // 自己回复 1
        "step 1"; player.draw(2);                                     // 自己摸 2
        "step 2"; target.recover();                                   // 目标回复 1
        "step 3"; target.draw(2);                                     // 目标摸 2
    },
},
```

## 2 原生现状（**逐点核对**）

| 环节 | 现状 |
|---|---|
| 键与行为 | ✅ `vocabulary/host/keys.rs:44`：`HostBehavior::WudianGuihan => "host:guihan"`（运筹 `character.json` 中蔡文姬的 `ycnguihan` 的 `mapping` 正是 `host:guihan`） |
| 过滤器 | `filtering/common.rs:28` → **`false`**（作为**主动技**不随事件触发 → **方向正确**） |
| 工厂 | `factory.rs:231` → `(None, None)`（无目标/牌注入） |
| **效果分支** | **`step/wudian_status.rs:82` → `Flow::Done`（空实现）** ✗ |

⇒ **归汉的激活路径没有效果实现**（既未回复/摸牌，也未 `awakenSkill`）——
与「未实现」在**效果面**上等价，但成因不同：**已注册、效果为空**。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] **实现归汉效果**（回复/摸牌 + `awakenSkill` 限定技标记）→ 并入修复清单**第 32 项**（运筹未实现清单由三项增至**四项**）
- [ ] 行为面（无实现 → 无从验收）；`limited`（限定技）在原生侧的表示方式
