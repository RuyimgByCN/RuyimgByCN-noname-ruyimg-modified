# T12 swd 武将面批 84：淳于恒/凤天凌（补技）

- 日期：2026-10-15
- 范围：两 partial 补技——**名数不变 73/79**（补技不加将）；新增 HostBehavior tag 378–379
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`

## 两将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd淳于恒 (wei hp2, 男) | 绝情（builtin）+ **圣手 shengshou** + 虚影 | **整将补全**：SwdShengshou(tag 379)=锁定 marker——engine PlayCard viewAs 链新增臂：黑色手牌当草药（上游 viewAs caoyao 同口径；草药结算=kernel Caoyao 臂回 1）。cards.json shengsheng→shengshou 勘误（此前 skill 名与上游不符） |
| swd凤天凌 (shu hp4, 男) | 鬼眼（host:guiyan）+ **激昂 jiang** | **整将补全**：SwdJiang(tag 378)=CardUsed frequent——①自己使用红【杀】/【决斗】且唯一目标 ②成为他人红【杀】/【决斗】目标 → 摸 1（上游双面并集；登记差异：目标面按 CardUsed+targets 表达） |

## 设计要点

- **华人名勘误**：cards.json 原 `shengsheng` 技能名为误写（上游键=shengshou/圣手）——本批按上游修正并补 mapping。
- 激昂双面并集收敛为 CardUsed 单事件（`u.player==owner && targets.len()==1` ∨ `u.targets.contains(&owner)`）。

## 验证

- **定向回归** `swd_generals_b84_shengshou_jiang`：
  - 圣手正例：黑桃牌当草药→p1 回 1（1→2）；反例：红桃牌不转化被拒。
  - 激昂正例：自己红杀→摸 1（出 1 摸 1 净 1 张）；黑面由 filter 条件（红/决斗+目标面）承担。
- **全量**：engine-native **796 passed / 0 failed / 0 ignored**（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **73/79 名**（部分转正将的缺技补全；入包将数不变）。
- 四项用户决策维持待用户裁决清单登记。
