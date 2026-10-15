# T12 swd 武将面批 89：车芸（仙匠）

- 日期：2026-10-15
- 范围：一将 partial 转正——**78/79 名**（上批 77 将+1）；新增 HostBehavior tag 386
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（cyxianjiang 逐段核验）

## 一将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd车芸 (wu hp3, 女) | 仙匠 cyxianjiang（巧械/神工 defer） | SwdCyxianjiang(tag 386)=CardUsed optional——自己使用牌指定唯一其他目标时：SelectCard 其装备区一张自己未持有（按牌种）的牌 → 实体发现同牌种牌入手 + Equipped 帧全托管装备（take_hand_card+同栏替换+grant_equip_skills——破茧同款托管面） |

## 设计要点

- **实体发现替代 createCard 凭空复制**（登记差异——羊皮卷/灵剑池同族）；本体留在原主。
- **「每回合每目标一次」登记收窄**：上游按 storage.cyxianjiang 逐目标记录，原生收敛为 limits once_per_turn（每回合一次，登记差异）。
- 装备入场走完整 Equipped 帧（同栏替换/替换牌入弃堆/装备技授予全托管）。

## 验证

- **定向回归** `swd_generals_b89_cyxianjiang`：正例（p0 杀 p1→确认窗→候选含 p1 武器→选中后 p0 装备区出现同牌种另一实体的断言+本体仍在 p1）。
- **包装载**：78 将入册（cards.json 77→78）。
- **全量**：engine-native **801 passed / 0 failed / 0 ignored**（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **78/79 名**（剩余 1 将：端蒙）。
- 四项用户决策维持待用户裁决清单登记。
