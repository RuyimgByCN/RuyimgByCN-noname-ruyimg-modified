# NC08 第⑦层样本 156：**来源 U06（swd）** 法器 `qinglongzhigui`（青龙之圭）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**；规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项未通过**（**摸 2 生效、强制弃 1 未发生**）→ 登记**第 55 项**

## 1 上游源码（**逐字**）

```js
qinglongzhigui: { type: "jiqi", global: "g_qinglongzhigui" },   // 卡本体无 content
g_qinglongzhigui: {
    trigger: { player: "phaseBegin" },
    forced: true,
    filter(event, player) { return player.countCards("h", "qinglongzhigui") > 0; },
    content() {
        "step 0"; …player.draw(2);                    // 摸 2
        "step 1"; player.chooseToDiscard("he", true);  // **强制弃 1**
    },
},
```

## 2 实测（`nc08_upstream_swd_qinglong.rs`）

| 观测 | 实测 | 应然 |
|---|---|---|
| 牌堆 | **196 → 194** | −2 ✅（**摸 2 生效**） |
| 手牌 | **3 → 5** | 应 **3 → 4**（摸 2 弃 1 ⇒ 净 +1）❌ |
| 弃牌询问 | **`prompts = []`** | 应出现「强制弃 1」的选牌询问 ❌ |

## 3 诊断（第 55 项）

**已排除**：①持有条件成立（`filter` 只要求持有）②询问形态（用例同时处理 `SelectCards`/`Confirm`/`Pass`）
③阶段未开始（**摸 2 已证**窗口运行）。

**剩余候选**：
1. **强制弃牌分支未实现**（`chooseToDiscard(…, true)` 的 `true` 语义未被承载）；
2. **`step 1` 被跳过**（阶段机只执行首步，或多步窗口未串行）；
3. ~~`he` 区无牌可弃~~ —— **不成立**（实测手牌 5 张）。

**修法（先诊断）**：核对 `g_qinglongzhigui` 的宿主实现是否声明了「**先摸后弃**」两步；
若只有摸牌步，按上游补 `step 1`。

**验收**：本用例（当前 `#[ignore]`）应能断言 **牌堆 −2** 且 **手牌净 +1**。
