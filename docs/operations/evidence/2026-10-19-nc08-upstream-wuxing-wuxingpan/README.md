# NC08 第⑦层样本 140：**来源 U11（wuxing / noname-liuli）** 装备 `wuxingpan`（五行盘）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面）**，**未发现差异**
- **意义**：**wuxing 来源的首个行为面样本**（该来源此前仅有转换/归属层核对）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname-liuli/noname/extension/wuxing/extension.js`（**扩展对象格式**）：

```js
card:{
    wuxingpan:{ type:'equip', subtype:'equip5', skills:['wuxingpan_skill'], fullskin:true }
},
translate:{
    …, wuxingpan:'五行盘', wuxingpan_skill:'五行',
    wuxingpan_skill_info:'出牌阶段限一次，你可以永久改变一张手牌的五行属性',
},
help:{ '五行生克':'…每名角色在游戏开始时随机获得一个属性；牌堆中三分之一的牌会随机获得一个属性；
       当一名成为相克属性卡牌的目标时，须弃置一张牌；当一名角色成为相生的卡牌的目标时，须摸一张牌；
       金克木，金生水；木克土，木生火；水克火，水生木；火克金，火生土；土克水，土生金' }
```

## 2 验收结果（`nc08_upstream_wuxing_wuxingpan.rs`）

| # | 判据 | 实测 | 结果 |
|---|---|---|---|
| 1 | **装备（`equip5` 宝物位）** | 装备区 **0 → 1**，发动 `Accepted` | 通过 |
| 2 | **授予 `wuxingpan_skill`（五行）** | 技能候选命中 = **`["wuxingpan_skill"]`** | 通过 |

**原生实现引用**：`catalog.rs:946`（`CardKind::WuxingPan => "wuxingpan"`）；
包内 `equip_skills: ["wuxingpan_skill"]`。

## 3 系统面核对：**五行系统已实现**（源码逐字引用，2026-10-19）

| 面 | 原生实现（**逐字**） |
|---|---|
| **牌的五行属性** | `state.rs:32`「五行属性（G1 wuxing：**0=金 1=木 2=水 3=火 4=土**；None = 未启用）」；`state.rs:802`「**五行属性表**（牌 id → 属性 …）」 |
| **生克结算** | `behavior.rs:15`「**五行生克（wuxing）：成为有属性牌的目标时——被克弃一张手牌**」；`keys.rs:853`（`HostBehavior::ShengkeCheck => "host:shengke_check"`） |
| **五行盘技能** | `behavior.rs:19`「五行盘（wuxing）：**出牌阶段限一次——改变一张手牌的五行属性**」；`activation.rs:125`「五行盘：出牌阶段限一次（**阶段门控在命令边界**），**需手牌非空**」 |

⇒ **系统面（属性 + 生克 + 技能）均已在原生实现** ⇒ 本来源的待办**不是「未实现」**，
而是**行为面尚未逐面采样**（属性赋予、生克两支、技能改属性）。

## 4 未完成 / 待办（**行为面采样**）

- [ ] **属性赋予**：角色开局随机属性；牌堆 1/3 的牌随机属性
- [ ] **生克两支**：**相克 → 弃 1**；**相生 → 摸 1**（上游 `help` 明确两向）
- [ ] 技能：**限一次** + **永久改变一张手牌的五行属性**
