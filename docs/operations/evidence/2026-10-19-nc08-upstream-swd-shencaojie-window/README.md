# NC08 第⑦层样本 150：**来源 U06（swd）** `shencaojie`（神草结）的 **`global` 窗口定位**（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面 · 窗口定位）**，**未发现差异**
- **意义**：第 384 轮交接**待办（`global` 卡窗口逐卡定位）的首张**——并确立了**可复用的定位方法**

## 1 上游源码（**逐字**）

`.../swd/skill.js:224` 的子技 **`g_shencaojie`**：

```js
g_shencaojie: {
    trigger: { source: "damageBegin", player: "damageBegin" },   // 伤害开始时（来源或受击者）
    direct: true,
    filter(event, player) {
        if (get.type(event.card) !== "trick") return false;      // 须为锦囊造成的伤害
        if (player.hasCard("shencaojie")) return true;           // 须持有神草结
        return false;
    },
    content() { player.chooseToUse(…, trigger.player, -1).targetRequired = true; },
},
```

卡片本体（`card.js:247`）：`global: "g_shencaojie"`；`content` 为
**`if (evt.player === player) evt.num--; else evt.num++;`**（**修改该次伤害的计数**）。

## 2 验收结果（`nc08_upstream_swd_shencaojie_window.rs`）

| # | 观测点 | 实测 | 结果 |
|---|---|---|---|
| 1 | **窗口是否出现** | 锦囊伤害时出现 **`trigger_confirm`**（`[Confirm]`、`allow_cancel=true`） | 通过 |
| 2 | **效果（`num++`）** | 回答 `Confirm`（即使用）后 **1 号共受 2 点**（未使用时应为 1 点）⇒ **伤害 +1 生效** | 通过 |

**原生实现引用**：宿主行为（swd 机器牌机；`shencaojie` 的 `global` 窗口）。

## 3 **可复用的定位方法（本轮确立）**

1. 从上游 `card.js` 取该卡的 **`global: "g_xxx"`**；
2. 在 **`skill.js`** 找到 **`g_xxx`** 子技的 **`trigger`** ⇒ 即**窗口事件**（本例 `damageBegin` + 锦囊）；
3. 构造该事件（本例：**用一张造成伤害的锦囊**打目标）；
4. 观察窗口形态与**效果**。

**夹具要点（**金莲珠教训的直接应用**）**：窗口形态为 **`trigger_confirm`（Confirm-only）**——
**回答 `Confirm` 才是「使用」**；若按「空选/取消」应答会**静默放弃**（首轮即如此，误判为「卡不可用」）。

## 4 未完成（**待办**）

- [ ] 其余 `global` 卡窗口逐卡定位（11 张在目录：趁火打劫/金莲珠/石灰粉/器族五件/神秘果/朱fang神石…）
- [ ] 神草结的 `targetRequired` 目标面与「来源为使用者时 `num--`」的反向分支
