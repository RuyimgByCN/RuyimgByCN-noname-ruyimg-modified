# U05 mtg 第三批：水没墓穴（规则 v102，2026-10-08）

mtg 子包第三批转换：水没墓穴（♠11 land 地）——换地后可弃一张摸二
（可取消，无牌空发）；地图效果锁定：手牌上限 -1 + 本弃牌阶段弃置过
牌则阶段结束摸一。前置：
[mtg 第二批](../2026-10-08-u05-mtg-batch2/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/杀海拾遗/card/mtg/）

- `list.js:3`：`["spade", 11, "mtg_shuimomuxue"]`（♠11 一张）；
  `translate.js:7-9`：水没墓穴——「你可以弃置一张牌并摸两张牌。地图
  效果：锁定技，你的手牌上限-1；若弃牌阶段弃置了至少一张牌，则在此
  阶段结束时摸一张牌」。
- `card.js:315-335`：content 换地 → `countDiscardableCards("he")` 有
  牌则 `chooseToDiscard("he")`（可取消）→ `result.bool` 弃则
  `draw(2)`；`skill.js mtg_shuimomuxue_skill`：mod maxHandcard -1 +
  trigger phaseDiscardEnd forced（event.cards 非空 → draw()）。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Shuimomuxue`（编码 123）+
  `HostBehavior::MtgShuimomuxueLand`（tag 86，HostBehavior::ALL 86 守
  卫过）两段机（换地→he 有牌问弃/空发→弃则摸二/取消无事，海岛同款
  双区弃牌口径）+ kernel intrinsic 臂 + PlayCard 地臂并入 + snap 派生
  臂。地图效果两钩：①手牌上限 -1（弃牌阶段 limit 计算处，
  current_land 直读，倾盆大雨/妄尊印同款）；②`TurnState.
  discard_phase_discarded` 计数（弃牌阶段开始清零、DiscardToLimit 机器
  弃牌累加、**PhaseEnd 离开弃牌阶段时**结算摸一并清零——上游
  phaseDiscardEnd forced 同口径；边界登记：计数仅含弃牌阶段上限弃牌
  机器的弃牌）。全套 archive/哈希接线（计数入存档与 canonical_hash）。
- **登记差异**：计数面仅弃牌阶段上限弃牌（上游 event.cards 同为该阶
  段弃牌集合，口径一致）；其余无。
- **转换包**：cards.json 增 ♠11 一张（现 3 张）+ 三哈希重算。

## 验收口径

- **回归已加入**：`mtg_shuimomuxue_land_effect`——①弃一摸二（存档
  往返）②有牌取消（保留原牌不摸）③空手空发无询问 ④地图效果（体力
  压 2：上限 2-1=1，手牌 2 触发 discard_to_limit 弃 1→阶段结束摸 1）
  ⑤换地覆盖（海岛→水没跨回合：盾牌格挡获甲不转换 1+1=2、不回血）。
- **全量通过**：engine-native **418 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 101→102 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（水没墓穴不在标准场景，纯版本位移）。
- **剩余**：mtg 其余 9 张地（依夏兰/废墟/幽林/沙漠绿洲 + 五镜像）逐
  批；zhenfa 14 张；ow/xianjian 武将管线。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（18 文件，相对 noname-next 根）。
