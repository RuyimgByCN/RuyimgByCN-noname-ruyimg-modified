# NC08 第⑦层样本 78：**来源 U05（杀海拾遗·运筹）** 武将技 `diyguhuo`（蛊惑·变体）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（含行为面）+ **1 项差异登记**（弃牌选择权）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`（`diyguhuo`）：

```js
diyguhuo: {
    trigger: { player: "phaseBegin" },        // 回合开始
    forced: true,
    filter(event, player) { return player.countCards("hej") > 0; },
    content() {
        "step 0"; player.draw(2);                                  // 摸 2
        "step 1";
        var next = player.discardPlayerCard(player, "hej", 2, true);   // 弃自己 hej 区 2 张
        next.filterButton = b => lib.filter.cardDiscardable(b.link, player);   // 排除不可弃
    },
},
```

## 2 验收结果（`nc08_upstream_yunchou_guhuo.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `draw(2)` + `discardPlayerCard(player,"hej",2,true)` | **行为面**：回合开始后 **判定区 1 → 0**、**装备区 1 → 0**（原生按「判定区→装备区→手牌」顺序各弃 1） | 通过 |
| 3 | `discardPlayerCard`（玩家自选）+ `cardDiscardable` | **差异登记**：上游**玩家自选 2 张**（并排除不可弃牌）↔ 原生**固定顺序自动弃** → 与「**取牌选择权**」族同族 | 通过（**差异登记**） |

**原生实现引用**：`content/host/step/yunchou.rs`（`YunchouDiyguhuo`：`draw_cards(owner,2)` →
`judge.first()` / `equips.first()` / 手牌首张 依次弃，装备走 `revoke`）。

## 3 夹具要点（**重要**）

- **引擎初态即 0 号判定阶段**（其「回合开始」已过）→ 须先推进到本轮摸牌阶段，再**驱动一整轮**
  等**下一次**回合开始（与天妒/观星同款教训）；
- **手牌数不可作判据**：驱动整轮会经过该角色自己的**弃牌阶段**（`discard_to_limit`），
  手牌变化被污染（实测 8 → 4）→ 只用**判定区/装备区**两个干净信号。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] `filterButton`（延迟锦囊不可弃）的对应行为
- [ ] **弃牌选择权**随「取牌选择权」族统一评估
- [ ] `ai.effect` 策略面
- [ ] 运筹其余武将技（纵火/烧营/归汉/精械/巧思/奸雄变体/悲歌变体）
