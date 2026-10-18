# NC08 第⑦层样本 2：`fz_liegong`（烈弓）+ `fz_xinkuanggu`（狂骨）（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层「上游行为」对照样本 2/3
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**由上游 JS 推导的 7 条规则点全部对齐**（无新差异登记）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js`（片段见
[upstream-snippets.js.txt](upstream-snippets.js.txt)）：

```js
lib.skill.fz_liegong = {
    trigger: { player: "useCardToPlayered" },          // 逐目标
    filter: function (event, player) {
        if (event.card.name != 'sha') return false;    // 仅【杀】
        var length = event.target.countCards('h');     // 目标手牌数
        return (length >= player.hp || length <= player.getAttackRange());
    },
    content: function () { trigger.getParent().directHit.push(trigger.target); },
};

lib.skill.fz_xinkuanggu = {
    trigger: { source: "damageSource" },               // 自己是伤害来源
    filter: function (event, player) {
        return get.distance(player, event.player) <= 1 && event.num > 0;
    },
    direct: true,
    content: function () { /* 选 recover_hp 或 draw_card */ },
};
```

## 2 由 JS 推导的规则点 → 原生验收（新增 `nc08_upstream_liegong_xinkuanggu.rs`，4 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| ② | 条件**第二支**「目标手牌 ≤ 攻击范围」 | 目标手牌 1 ≤ 攻击范围 1（且 < 出杀者体力 4）→ **直伤**且目标**未出【闪】**（手牌仍 1 张） | 通过 |
| ② | 条件**不满足**（2 < 体力 4 且 2 > 攻击范围 1） | **不**强制直伤（目标可正常响应） | 通过 |
| ⑤ | 狂骨要求 `distance ≤ 1` | 距离 2 的目标受伤 → **不触发**（体力不变） | 通过 |
| ⑦ | 狂骨二选一 | 选 `Tag(0)`（回复）→ 体力 +1；选 `Tag(1)`（摸牌）→ 手牌 +1 | 通过 |

（样本 1 已覆盖：烈弓「手牌 ≥ 体力」支与狂骨「距离 1」支——见
[2026-10-18-nc08-upstream-kurou](../2026-10-18-nc08-upstream-kurou/README.md) 与
`tests/l20_liegong_batch3.rs`。）

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 上游对照定向 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | **1161 passed / 0 failed / 2 ignored**（1157 → +4） | [engine-native-tests.log](engine-native-tests.log) |
| 行数门禁 | 8264 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 4 未完成 / 待办（NC08 第⑦层）

- [ ] 样本 3+：其余来源的同类「单技能语义对照」样本（当前 2/37 来源已覆盖）
- [ ] **逐命令对照工具**（固定上游版本/局面/随机输入与玩家选择 → 比较状态/事件/选择/随机流/续行）
- [ ] 样本 1 登记的**顺序差异**（`loseHp` 与 `draw` 先后）上游判定
- [ ] 表现/策略面（`audio`/`ai`/`mark`/`intro`）不在规则对照范围（边界登记）
