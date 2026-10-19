# NC08 第⑦层样本 44：`hujia`（护驾，**标准面** / 曹操）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**4 项通过（含行为面）**（含 **2 项差异登记**，其中①为**原生更宽松**）
- **基准面**：`character/standard/skill.js`
- **样本类型**：**源码事实级**（本样本未做借闪的行为面，已登记待办）

## 1 上游源码（归档快照，逐字引用；见 [upstream-hujia.js.txt](upstream-hujia.js.txt)）

```js
hujia: {
    zhuSkill: true,                                     // 主公技
    trigger: { player: ["chooseToRespondBefore", "chooseToUseBefore"] },
    filter(event, player) {
        if (event.responded) return false;
        if (player.storage.hujiaing) return false;      // 防重入
        if (!player.hasZhuSkill("hujia")) return false; // 主公资格
        if (!event.filterCard({ name: "shan", isCard: true }, player, event)) return false;
        return game.hasPlayer(current => current !== player && current.group === "wei");
    },
    async content(event, trigger, player) { let current = player.next; while (true) { … } },
},
```

## 2 验收结果（`nc08_upstream_hujia.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `current.group === "wei"` + `let current = player.next` | 原生 `build_hujia_queue`：**下家起座位序** + `Faction::Wei`（源码事实） | 通过 |
| 1 | 需求牌为【闪】 | 原生护驾归属 **【杀】结算机器**（`ShaResolveMachine`，需求即【闪】） | 通过 |
| 2 | **行为面（借闪）** | 1 号持护驾（原型模式 `is_lord` 为真）、2 号装配**魏**势力武将并持【闪】→ 出现 **`hujia_respond`** 询问 → 2 号代出【闪】→ **【杀】被抵消**（1 号 hp 4 → 4，2 号手牌清空） | 通过 |
| 1 | `hasZhuSkill("hujia")` | **差异①（原生更宽松）**：原生用 `is_lord`，而 `is_lord` 在**非身份局返回 true**（`state.rs` 的 `None => true`）→ 原型/单挑模式下原生**可发动**而上游不可 | 通过（**差异登记**） |
| 1 | `player.storage.hujiaing` | **差异②**：原生 `hujia_active` **未检查**防重入旗标（是否由调用路径保证**未判定**，登记） | 通过（**差异登记**） |

**原生实现引用**：`kernel/sha.rs:293`（`hujia_active`）、`:302`（`build_hujia_queue`）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **4 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [x] **借闪行为面已验收**（`hujia_respond` 询问 → 魏势力代出【闪】→ 杀被抵消）
- [ ] 差异①口径（`is_lord` 在非身份局为真是否应改为「须身份局且为主公」）
- [ ] 差异②：核对调用路径是否已保证不重入
- [ ] `chooseToUseBefore` 面（上游亦覆盖主动使用【闪】的场景）
- [ ] 样本 45+：其余来源/技能（标准面技能 **23 个样本**；来源覆盖 **23/37**）
