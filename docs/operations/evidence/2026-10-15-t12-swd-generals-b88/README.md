# T12 swd 武将面批 88：姜子牙（命符）

- 日期：2026-10-15
- 范围：一将 partial 转正——**77/79 名**（上批 76 将+1）；新增 HostBehavior tag 385
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（mingfu 逐段核验）

## 一将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd姜子牙 (wu hp3, 男) | 命符 mingfu（天命 tianlun defer） | SwdMingfu(tag 385)=ActivateSkill usable:1——SelectCard 梅花手牌（消费段校验）→ TargetChoice 任一其他存活 → 弃梅花牌+实体发现鬼幽结置其判定区（上游 mingfu prepare throw + addJudge("guiyoujie") 同口径；登记差异：实体发现替代 createCard，牌堆优先弃堆兜底） |

## 设计要点

- 鬼幽结实体发现沿用「牌堆优先、弃堆兜底」（羊皮卷/灵剑池同族）；判定区置入由既有 JudgePhaseMachine 结算（黑色判定=失去 1 体力+随机弃牌——既有基建）。
- **天命 tianlun 登记结构性缺口**：global judge 事件改判窗（用任意牌为判定对象换结果）——原生 JudgePhaseMachine 的鬼道改判队列只支持「黑手牌替换」，天命需任意区牌+全玩家判定区扫描，挂后续改判窗深化批。

## 验证

- **定向回归** `swd_generals_b88_mingfu`：正例（梅花弃牌→p1 判定区 +1 且 kind=Guiyoujie）；反例：无梅花 ActivateSkill 拒。
- **包装载**：77 将入册（cards.json 76→77）。
- **全量**：engine-native **800 passed / 0 failed / 0 ignored**（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **77/79 名**（剩余 2 将：车芸/端蒙）。
- 四项用户决策维持待用户裁决清单登记。
