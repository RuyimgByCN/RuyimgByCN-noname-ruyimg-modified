# NC08 第⑦层样本 59：`jiuyuan`（救援，**标准面** / 孙权）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（**源码事实级**）+ 1 项**同族设计差异引用**（第 3 例）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-jiuyuan.js.txt](upstream-jiuyuan.js.txt)）

```js
jiuyuan: {
    trigger: { target: "taoBegin" },                 // 自己成为【桃】的目标
    zhuSkill: true, forced: true,                    // 主公技 + 锁定
    filter(event, player) {
        if (event.player === player) return false;
        if (!player.hasZhuSkill("jiuyuan")) return false;   // 主公资格
        if (event.player.group !== "wu") return false;      // 用桃者为吴势力
        return true;
    },
    async content(event, trigger, player) { trigger.baseDamage++; },   // 回复量 +1
},
```

## 2 验收结果（`nc08_upstream_jiuyuan.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2+3 | `hasZhuSkill` + `group !== "wu"` + `baseDamage++` | 原生 `dying.rs`：`has_skill(dying, JIU_YUAN)` + `responder != dying` + `is_lord(dying)` + `Faction::Wu` → **`amount = 2`**（`recover(dying, amount, Some(tao))`） | 通过 |
| 1 | `zhuSkill` | **同族设计差异引用（第 3 例）**：主公技资格用 `is_lord`，非身份局为真（`state.rs` 注释「无身份体系的模式持技即可」）→ **原型模式下救援亦可生效**（与 `hujia`/`jijiang` 同族，已核对为有意设计） | 通过（**同族引用**） |

**原生实现引用**：`kernel/dying.rs:160`。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] **回复 2 点的行为面**（需构造「主公濒死 + 吴势力角色持桃救援」场景）
- [ ] `audio` 表现面
- [ ] 样本 60+：其余来源/技能（标准面技能 **37 个样本**；来源覆盖 **23/37**）
