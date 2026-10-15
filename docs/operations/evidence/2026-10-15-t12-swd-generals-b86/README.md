# T12 swd 武将面批 86：疆梧（义绝/荡平）

- 日期：2026-10-15
- 范围：整将转正——**75/79 名**（上批 74 将+1）；新增 HostBehavior tag 381–382
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}` + `character/refresh/skill.js`（yijue 刷新版；dangping）

## 整将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd疆梧 (shu hp4, 男) | 义绝 yijue + 荡平 dangping | **整将**：SwdYijue(tag 381)=ActivateSkill usable:1——目标选拼点牌→己方选牌→比点：赢=目标 `fengyin_active`（本回合基本/锦囊失效）+受 1 伤；输=目标已受伤回 1。SwdDangping(tag 382)=Damage optional（每回合限一次）——触发帧捕获受害者→弃一张 he→受害者距离 ≤1 的另一名角色受 1 伤 |

## 设计要点

- **拼点机复用 CompareTargetPick/ComparePlayerPick 两段**（府邸愁心同族）；拼点牌入弃堆。
- 义绝赢面用 `fengyin_active` 旗标（Boss 封冻同源——PlayCard 校验已含 h 位置封印面）。
- 荡平受害者从触发帧读（上游 `trigger.player` 同口径）；邻近按 `computed_distance(victim, p) <= 1`。
- 每回合限一次用 limits once_per_turn（上游 addTempSkill dangping2 换回合等价）。

## 验证

- **定向回归** `swd_generals_b86_yijue_dangping`：义绝赢（p1 受 1 伤 4→3+封印断言）；荡平全链（杀 p1→确认窗→弃牌→目标窗 p2 候选断言→p2 受 1 伤）。
- **包装载**：75 将入册（cards.json 74→75）。
- **全量**：engine-native **798 passed / 0 failed / 0 ignored**（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **75/79 名**（剩余 4 将：车芸/端蒙/尉迟嫣红/姜子牙）。
- 四项用户决策维持待用户裁决清单登记。
