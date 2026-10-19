# NC08 第⑦层样本 56：`jieyin`（结姻，**标准面** / 孙尚香）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**0 项通过 / 2 项未验收（登记）** —— **行为面受阻于夹具/入口问题**，**不写成通过**
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-jieyin.js.txt](upstream-jieyin.js.txt)）

```js
jieyin: {
    enable: "phaseUse", filterCard: true, usable: 1, selectCard: 2,   // 弃两张手牌，限一次
    filterTarget(card, player, target) {
        if (!target.hasSex("male")) return false;      // 男性
        if (target.hp >= target.maxHp) return false;   // 已受伤
        if (target === player) return false;           // 非自己
        return true;
    },
    async content(event, trigger, player) {
        await player.recover();                        // 自己回复 1
        await event.target.recover();                  // 目标回复 1
    },
},
```

## 2 本轮状态：**未验收（如实登记，含诊断）**

**已满足的前置**（[diagnostic.txt](diagnostic.txt) 实测）：
`自己 hp=3 手牌=2 目标 hp=3 目标将槽=[Some(0), None]`

**受阻现象**：`ActivateSkill { skill }` 返回 **`Rejected(InvalidChoice)`** →
两个行为用例（双回复、门槛）均**未构成验收**，已改 `#[ignore]`（**不写成通过**）。

**根因待查（下一轮）**：疑点有二——
1. **将槽性别判定**：`slots_is_male` 在「非全空」时要求**任一槽为男性**；实测槽为 `[Some(0), None]`，
   需确认 `debug_assemble_general(1, <男性武将名>)` 是否真的把**男性**写入该槽；
2. **技能注册/入口面**：`ActivateSkill` 的 `InvalidChoice` 是否指向技能未按 `activate` 形态注册
   （对照：青囊/离间同形态可发动）。

## 3 原生实现引用（源码事实，**可核对**）

`content/standard/skills/jie_yin.rs`：注释「v202 对齐上游 `filterTarget`：**male + hp<maxHp + 非自己**；
上游无『已婚』『装备区有牌』条件，**旧登记撤销**」；`activate` 要求存活 + `hand.len() >= 2` +
存在「受伤的男性其他角色」（`slots_is_male` + `hp < max_hp`）；机器三阶段
`CardsAsk → TargetAsk → TargetRead`（弃两张 → 选目标 → 双方各回复 1）。

→ **源码事实层面**与上游三条判据一致；**行为面待补验收**。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **0 passed / 0 failed / 2 ignored**（未构成验收） | [diagnostic.txt](diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **查清 `ActivateSkill` 被拒的根因**（将槽性别 or 技能注册面），随后补行为验收
- [ ] `check` 评分与 `audio` 表现/策略面
- [ ] 样本 57+：其余来源/技能（标准面技能 **34 个样本 + 1 个未验收**；来源覆盖 **23/37**）
