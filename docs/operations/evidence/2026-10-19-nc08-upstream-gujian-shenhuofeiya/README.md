# NC08 第⑦层样本 85：**来源 U05（古剑）** 机关牌 `shenhuofeiya`（神火飞鸦）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过（含行为面 · 额外目标）**，**未发现差异**
- **作用**：对修复清单**第 35 项**的**反向核对**（乾坤镖**缺** `he` 门槛；神火飞鸦**不应**有该门槛）

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:906`（`shenhuofeiya`）：

```js
shenhuofeiya: {
    type: "jiguan", enable: true, wuxieable: true,
    filterTarget(card, player, target) { return target !== player; },   // 只要求其他角色
    changeTarget(player, targets) {                    // 额外影响：距离 1（无 he 条件）
        game.filterPlayer(current => get.distance(targets[0], current, "pure") === 1, targets);
    },
    cardcolor: "red", cardnature: "fire",
    content() {
        "step 0"; var next = target.chooseToRespond({ name: "shan" });   // 须出【闪】
        "step 1"; if (…) target.damage("fire");                          // 否则 1 点火伤
    },
},
```

## 2 验收结果（`nc08_upstream_gujian_shenhuofeiya.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `filterTarget`：**只要求其他角色** | **门槛反向确认**：目标 `he` **无牌**时**仍受理**（`Accepted`）——与乾坤镖**不同** | 通过 |
| 2+3 | `changeTarget`（距离 1）+ 出【闪】否则火伤 | **行为面**：`hp [4,4,4,4] → [4,3,3,4]`（**首目标 1 号与距离 1 的 2 号各受 1 点火伤**）、使用者 hp 不变 | 通过 |

**原生实现引用**：`engine/play_card.rs:1684`（与乾坤镖**共用**目标校验分支）；`kernel/intrinsic.rs:253`
（映射 `GJ_SHENHUOFEIYA_CARD_KEY`）。

## 3 对第 35 项的意义

第 35 项要求给**乾坤镖**补 `he` 门槛，并**特别提示**神火飞鸦**不应**被误加该门槛。
本样本即该提示的**行为面证据**：修 35 项时若把校验合并，本样本的 `Accepted` 断言会**失败**。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] `wuxieable` 无懈面；`cardcolor`/`cardnature` 属性面
- [ ] 古剑包其余卡牌
