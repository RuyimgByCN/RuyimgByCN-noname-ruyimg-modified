# T12 swd 武将面批 90：端蒙（玄影/烈刃）——**79/79 将全入包**

- 日期：2026-10-15
- 范围：**末将**转正——**79/79 名**（上批 78 将+1）；新增 HostBehavior tag 387–388
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（xuanying 四子技/lieren 逐段核验）

## 末将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd端蒙 (shu hp4, 女) | 玄影 xuanying + 烈刃 lieren | **整将**：SwdXuanying(tag 387)=双面——①ActivateSkill（未横置）：TargetChoice 任一其他存活→横置自身+虚拟杀（无视距离由虚拟牌管线天然承担）②Damage optional（已横置受伤）：解除横置+对来源虚拟杀。SwdLieren(tag 388)=Damage optional——自己杀伤害后与受害者拼点两段（赢=e 获得其一张 he 手牌优先；确定性首张登记差异） |

## 设计要点

- **commit_linked 翻转语义**：原语按「翻转」提交（非置位）——机器不预置旗标只发事件（首版预置致双翻转被测试抓漏）。
- xuanying 触发面与主动面共用一个 step 臂，按「当前帧是否 Damage」分流（无帧=ActivateSkill 入口）。
- 登记差异：xuanying 闪响应面（横置代出闪）与失去牌反击面（回合外 use 子技）挂后续响应窗深化批；lieren 取牌确定性首张（上游 gainPlayerCard 自选）。

## 验证

- **定向回归** `swd_generals_b90_xuanying_lieren`：玄影主动面（横置断言+虚拟杀命中 8→7）；烈刃全链（杀 p1→确认窗→双段拼点→赢取牌断言）。
- **包装载**：**79 将全入包**（cards.json 78→79——swd 武将面全量闭合）。
- **全量**：engine-native **802 passed / 0 failed / 0 ignored**（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **79/79 名——swd 武将面 79 将全量入包**（部分转正将的缺技照既有 defer 登记逐批推进）。
- 四项用户决策维持待用户裁决清单登记。
