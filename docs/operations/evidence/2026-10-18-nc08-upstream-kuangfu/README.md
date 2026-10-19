# NC08 第⑦层样本 42：`kuangfu`（狂斧，**标准面** / 潘凤）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（含 1 项差异登记）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-kuangfu.js.txt](upstream-kuangfu.js.txt)）

```js
kuangfu: {
    audio: "xinkuangfu",
    trigger: { source: "damageSource" },
    forced: true,
    filter(event, player) {
        if (player.hasSkill("stdkuangfu_used")) return false;
        return player.isPhaseUsing() && event.card && event.card.name === "sha"
            && event.player !== player && event.player.isIn();
    },
    async content(event, trigger, player) {
        player.addTempSkill("stdkuangfu_used", "phaseChange");
        if (trigger.player.hp < player.hp) { await player.draw(2); }
        else { await player.loseHp(); }
    },
},
```

## 2 验收结果（`nc08_upstream_kuangfu.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `trigger.player.hp < player.hp` → `draw(2)` | 自己 4 血、目标 2 血（受 1 伤后仍存活）→ 打出【杀】后手牌**净 +1**（打出 −1、摸 2），目标 hp=1 | 通过 |
| 2 | 否则 → `loseHp()` | 自己 2 血、目标 4 血 → 打出【杀】后**自己失去 1 点体力**（2 → 1） | 通过 |
| 1 | `stdkuangfu_used` + `phaseChange` | **差异登记**：原生以「当前回合 + 出牌阶段」判定表达每阶段一次，并自述与上游 temp 技能**等价** | 通过（**差异登记**） |

**原生实现引用**：`content/standard/skills/kuang_fu.rs`（filter 判 `Damage` + 来源/目标/未防止/
牌为【杀】+ 回合与阶段；`step` 按 `target.hp < owner.hp` 分流）。

## 3 夹具经验（写入证据）

上游 filter 含 **`event.player.isIn()`**（目标**存活**），原生 `step` 亦有
`if !alive { Flow::Done }` → **夹具必须让目标在受伤后仍存活**，否则触发被跳过。
初版把目标设为 1 血（受 1 伤即死）→ 实测 `hand 1 → 0`（只打出了【杀】、未摸牌）→ 修正为 2 血后通过。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log)、[branch-diagnostic.txt](branch-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 差异（每阶段一次的表示方式）口径判定
- [ ] `ai.halfneg`/`audio` 表现与策略面
- [ ] 样本 43+：其余来源/技能（标准面技能 **21 个样本**；来源覆盖 **23/37**）
