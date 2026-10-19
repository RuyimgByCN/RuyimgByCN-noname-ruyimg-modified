# NC08 第⑦层样本 155：**来源 U06（swd）** 法器 `xuanwuzhihuang`（玄武之璜）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**；规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面 · `global` 窗口）**，**未发现差异**
- 意义：**`global` 卡窗口的第 4 张验证**（神草结 ×2、朱雀之璋、玄武之璜）

## 1 上游源码（**逐字**）

```js
xuanwuzhihuang: { type: "jiqi", global: "g_xuanwuzhihuang" },   // 卡本体无 content
g_xuanwuzhihuang: {
    trigger: { source: "damageEnd" },                            // 本人造成伤害后
    forced: true,
    filter(event, player) {
        return player.countCards("h", "xuanwuzhihuang") > 0       // 须持有该卡
            && event.num > 0 && player.hp < player.maxHp;         // 且已受伤
    },
    content() { … player.recover(trigger.num); },                 // 回复伤害量
},
```

## 2 验收结果（`nc08_upstream_swd_xuanwu.rs`）

| # | 判据 | 实测 | 结果 |
|---|---|---|---|
| 1 | 原始伤害 | 1 号受伤 **1** 点 | ✅ |
| 2 | **回复伤害量（`forced`）** | 0 号（持有者）hp **+1** | ✅ |

**前置构造**：0 号先降到 `maxHp - 2` 以满足上游 `hp < maxHp` 条件。

## 3 未完成（登记）

- [ ] `event.num > 0` 的**非正伤害**分支（0 伤/负伤是否触发）
- [ ] `showCards` 表现面
