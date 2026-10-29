# NC08 第⑦层样本 123：**来源 U07（hearth）** 锦囊 `linghunzhihuo`（灵魂之火）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面）**，**未发现差异**（并行为面确认族内「确定性」口径）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`（`linghunzhihuo`）：

```js
linghunzhihuo: {
    type: "trick", cardnature: "fire", enable: true, filterTarget: true,
    content() {
        "step 0"; target.damage("fire");                          // 1 点火焰伤害
        "step 1"; var hs = player.getCards("h");
                  if (hs.length) player.discard(hs.randomGet());  // 使用者随机弃 1 张手牌
    },
},
```

## 2 验收结果（`nc08_upstream_hearth_linghunzhihuo.rs`）

| # | 要点 | 实测 | 结果 |
|---|---|---|---|
| 2 | **目标受 1 点火焰伤害** | 1 号 hp **4 → 3** | 通过 |
| 2 | **使用者弃 1 张手牌** | 0 号手牌 **3 → 1**（打出 −1、弃 1 ⇒ 净 **−2**） | 通过 |
| — | 族内口径确认 | **`询问=[]`** ⇒ 弃牌为**自动（确定性）**，上游为 `hs.randomGet()` 随机 | 确认 |

**原生实现引用**：`content/host/intrinsic.rs:996`（`CardKind::Linghunzhihuo` →
`HostBehavior::LinghunzhihuoCard`）；`content/host/step/skills_damage.rs:749`；
原生 `content/standard.rs:52`（「v40：…**灵魂之火**/照明弹/急耗卡（CardKind 三种 + 宿主行为）」）。

## 3 未完成 / 待办

- [ ] `cardnature: fire` 的**属性修饰面**（藤甲等载体）
- [ ] 弃牌**具体弃了哪张**（上游随机；原生确定性）
- [ ] hearth 最后 1 卡（视界术）
