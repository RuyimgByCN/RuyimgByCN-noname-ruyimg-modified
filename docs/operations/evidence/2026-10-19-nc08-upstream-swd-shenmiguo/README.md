# NC08 第⑦层样本 160：**swd** `shenmiguo`（神秘果）（2026-10-19）

- 工作包：**NC08**；规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过**，**未发现差异**；**`global` 卡窗口第 8 张验证**

## 上游（`skill.js` `g_shenmiguo`，逐字摘录）

```js
g_shenmiguo: {
    trigger: { player: "useCardAfter" },        // 本人使用牌后
    direct: true,
    filter(event, player) {
        if (event.parent.name === "g_shenmiguo") return false;        // 防递归
        if (_status.currentPhase !== player) return false;            // 自己的阶段
        if (event.parent.parent.name !== "phaseUse") return false;    // 出牌阶段直接使用
        if (!event.targets || !event.card) return false;
        if (event.card.name === "shenmiguo") return false;
        … /* 五谷/机关/乐/兵/闪电 排除 */
    },
    content() { /* 用掉一颗神秘果，复制该牌结算 */ },
}
```

## 原生（`filtering/swd.rs:476-500` + `step/swd_equipment.rs:497-540`）

过滤「**自己在出牌阶段使用基本牌或非装备锦囊、且手持有神秘果**」；
生效方式为 **`turn.shenmiguo_used = true` + 追加一次 `CardUsed(神秘果)`**（**无询问**）。

## 判据与结果

| 判据 | 实测 | 结果 |
|---|---|---|
| 出牌阶段使用【杀】 | `PlayCard` ⇒ `Accepted` | ✅ |
| **神秘果被自动用掉** | **`turn.shenmiguo_used`：false → true** | ✅ |

```
DBG 神秘果：使用杀后 shenmiguo_used=true（应 true）
```

**判据强度说明**：`shenmiguo_used` **只由本窗口置位**（`step/swd_equipment.rs:515`），
不存在其它写入路径 ⇒ **不会因为其它功能误置位**（无假阳性）。

## 踩坑记录（**方法学**）

首版按**名字**取【杀】（`name.contains("sha") && contains("swd")`）⇒ 命中了 **`swd_shandian`（闪电）**
⇒ `Rejected(IllegalTargets)`。改为**按 `CardKind::Sha` 取牌**后通过。
⇒ 本会话第 N 次验证：**取牌必须按牌种，不能按名字**。

## 未完成（登记）

- [ ] 上游「**复制该牌结算**」的对等性（原生的 `CardUsed(神秘果)` + Restage 是否覆盖全部结算面）
- [ ] `turn.shenmiguo_used` 的**每回合一次**语义（跨回合重置）
