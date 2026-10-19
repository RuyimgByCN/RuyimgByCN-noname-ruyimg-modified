# NC08 第⑦层样本 90：**来源 U05（古剑）** 锦囊 `shihuifen`（石灰粉）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项未验收**（限制未生效，登记）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`（`shihuifen`）：

```js
shihuifen: {
    type: "trick", fullskin: true, filterTarget: true, global: "g_shihuifen",
    content() {
        "step 0";
        if (!_status.currentPhase?.isIn()) return;
        var next = _status.currentPhase.chooseToRespond({ name: "shan" });
        next.set("prompt2", "否则本回合无法对其他角色使用卡牌");
        "step 1";
        if (!result.bool) _status.currentPhase.addTempSkill("shihuifen", "phaseUseAfter");
    },
},
```

## 2 验收结果（`nc08_upstream_gujian_shihuifen.rs`）

| 观测点 | 实测 | 判读 |
|---|---|---|
| 发动（由**当前回合角色自己**使用） | `Accepted` | 可发动 ✅ |
| 闪的询问 | **`询问=[]`** | ✗ 上游应有 `chooseToRespond({name:"shan"})` |
| 限制 | 该角色随后对**其他角色**出【杀】**仍受理**（`Accepted`） | ✗ 限制未生效 |

**原生实现引用**：`content/host/step/gujian.rs:439`（`GujianShihuifenCard` 阶段机）；
`GujianFoodShihuifenBan => Flow::Done`（限制子技自身为空，执行应在出牌校验路径）。

## 3 夹具要点

- **`global` 子技不能由非当前回合方主动使用**（实测 `Rejected(NotYourTurn)`）→ 本样本改由
  **当前回合角色自己**使用；
- 对照：`jinlianzhu`（样本 87）同为 `global` 子技但**打断窗口可达且生效** ⇒ 本项须按卡定位。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **0 passed / 0 failed / 1 ignored** | [diagnostic.txt](diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 阶段机入口/stage 定位（为何无询问）
- [ ] 出牌校验路径实现限制 + `phaseUseAfter` 解除
- [ ] 古剑包其余卡牌
