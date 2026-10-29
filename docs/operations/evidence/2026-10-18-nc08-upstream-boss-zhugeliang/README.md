# NC08 第⑦层样本 40：来源 `boss-zhugeliang`（T11 Boss 诸葛亮族 / 赫拉西斯 `hlss_*`）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（含 2 项差异登记）；**来源覆盖 21 → 22/37**
- **来源包**：`packages/extensions/boss-zhugeliang-upstream-liuli`（1 将 / 5 技映射）

## 1 上游源码（归档快照，逐字引用）

`hlss_xiangxing`（[upstream-xiangxing.js.txt](upstream-xiangxing.js.txt)）：

```js
hlss_xiangxing: {
    unique: true, juexingtx: true,
    init: function(player) {
        player.storage.hlss_xiangxing = 3;          // 开局 3 枚星
        player.storage.hlss_xiangxing_count = 0;
        player.addSkill('hlss_xiangxing3');
    },
    mark: true, intro: { content: '当前有#枚星' },
    group: 'hlss_xiangxing_hp',
    trigger: { player: ['damageEnd', 'loseHpEnd'] },   // 受伤/失体后累计
    forced: true,
    …
},
```

官方文本（[upstream-xiangxing-info.js.txt](upstream-xiangxing-info.js.txt)）：**锁定技，游戏开始时，你获得 3 枚星；
你的体力值始终为满；每当你累计扣减 3 点体力，你失去一枚星，并造成 3 点雷属性伤害，随机分配给其他角色；
当你失去全部星后，你失去此技能。**

## 2 验收结果（`nc08_upstream_boss_zhugeliang.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| — | 包映射 | 包内 **1 将 / 5 技**，且 5 技**均映射到宿主行为键**（`host:…`） | 通过 |
| 1 | `init` 置 3 枚星 + `damageEnd`/`loseHpEnd` | 原生状态字段 `hlss_stars` / `hlss_count` 与官方文本一致（3→0、累计计数） | 通过 |
| 1 | 官方文本「**随机分配**给其他角色」 | **差异①**：原生按**座位序确定性近似**（「随机 → 确定性」族） | 通过（**差异登记**） |
| 2 | `group: 'hlss_xiangxing_hp'`（任意体力变化后回满） | **差异②**：原生在 `kernel/damage.rs::hlss_light_refill` 于**伤害/失去体力提交点**回满（先判实例 `enabled` 再置 `hp = max_hp`）→ 调用点/口径不同 | 通过（**差异登记**） |

**原生实现引用**：`content/host/keys.rs:239`（`HostBehavior::HlssXiangxing`）、
`state.rs:247–252`（`hlss_stars`/`hlss_count`/`hlss_damage`）、`kernel/damage.rs:1032`（`hlss_light_refill`）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] 差异①（随机分配 → 确定性）随该族一并评估
- [ ] 差异②（回满调用点）是否覆盖「非伤害类体力变化」（上游 `group` 面更宽）
- [ ] 该来源其余 4 技（`hlss_yueyin`/`yueyin2`/`fengqi`/`gaiming`）的逐点样本（本轮为**来源级**首个样本）
- [ ] 样本 41+：其余来源（当前 **22/37**）
