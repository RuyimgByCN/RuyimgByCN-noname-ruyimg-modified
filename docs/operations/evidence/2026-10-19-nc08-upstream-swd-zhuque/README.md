# NC08 第⑦层样本 154：**来源 U06（swd）** 法器 `zhuquezhizhang`（朱雀之璋）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**；规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面 · `global` 窗口）**，**未发现差异**
- 意义：**`global` 卡窗口的第 3 张验证**（前两张：神草结 ×2）

## 1 上游源码（**逐字**）

```js
zhuquezhizhang: { type: "jiqi", global: "g_zhuquezhizhang" },   // 卡本体无 content
g_zhuquezhizhang: {
    trigger: { player: "damageEnd" },                            // 本人受到伤害后
    forced: true,
    filter(event, player) {
        return event.source && event.source !== player && event.source.isAlive()
            && player.countCards("h", "zhuquezhizhang") > 0;      // 须持有该卡
    },
    content() { … trigger.source.damage("fire"); … },             // 对来源造成 1 点火伤
},
```

## 2 验收结果（`nc08_upstream_swd_zhuque.rs`）

| # | 判据 | 实测 | 结果 |
|---|---|---|---|
| 1 | 原始伤害 | 0 号受伤 **1** 点 | ✅ |
| 2 | **反伤（`forced`）** | **1 号（来源）受伤 1 点** | ✅ |

⇒ **持有者受伤后对来源造成 1 点火伤**，与上游 `forced` 语义一致（**无确认询问**）。

## 3 未完成（登记）

- [ ] `check` 的 AI 面（`get.damageEffect(source…) > 0`）
- [ ] `showCards` 的表现面
