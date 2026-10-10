# U05 zhenfa 首切片：地载阵（规则 v110，2026-10-08）

杀海拾遗 zhenfa 子包（阵法牌，list 启用 13 张全 `mode:["guozhan"]`）首
切片：新转换包 shahai-zhenfa-upstream-noname + 地载阵（♦3/♠3/♥3 三
张）——所有非大势力角色各摸一张牌（大势力=已明置势力存活人数最大
者）。其余 5 种启用阵法（长蛇/天覆/七星/鸟翔/破阵决）登记国战队列/
围攻/座位重排依赖暂缓。前置：[mtg 第十批](../2026-10-08-u05-mtg-batch10/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/杀海拾遗/card/zhenfa/）

- `list.js` 启用 13 张：长蛇阵 ♦1/♣1、天覆阵 ♦2/♠2/♥2、地载阵
  ♦3/♠3/♥3、七星阵 ♦7/♣7/♠7、鸟翔阵 ♦11/♠11/♥11（风扬/云垂/蛇蟠/
  龙飞/虎翼/破阵决全注释停用）。
- `card.js dizaizhen`：type zhenfa、`mode:["guozhan"]`、enable 存在
  isNotMajor 玩家、filterTarget `target.isNotMajor()`、selectTarget -1
  （自动全选）、content `target.draw(false)`（逐目标摸 1）。
- `isMajor`（library/element/player.js:13359）：国战各势力存活人数最
  大者为大势力（identity!=unknown；forceMajor 技能标特例）；isNotMajor
  = 场上有大势力时「非大势力者」（含暗置）。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Dizaizhen`（编码 131）+ kernel
  `DizaizhenMachine`（无懈窗口包装 + 展开所有非大势力存活角色各摸 1 +
  本体牌离结算区入弃堆；Noop 快照形态——无询问）+
  `GameStateCore::is_major_faction`（state.rs：effective_faction 非空
  者按势力统计存活数、人数最大者并列同算为大；未明置/无主将不属大势
  力）+ PlayCard 门控（targets 空 + 存在非大者，上游 enable 同口径）。
  新转换包（地载阵 ×3；deferred 五种阵法 with 原因；npack 三哈希）。
- **登记差异**：大势力判定简化——effective_faction 推导（Kingdoms=
  明置势力、其余模式=主将势力），forceMajor 技能标特例未建模；队列
  （inline）/围攻/座位重排（swapSeat）体系未建模——长蛇/天覆/七星/
  鸟翔/破阵决登记依赖暂缓（结构性保留项同族）；上游 mode:["guozhan"]
  模式限定未建模（原型/国战均可出，大势力按 effective_faction 推导）。
- **暂缓登记**：deferred 五种（changshezhen/tianfuzhen/qixingzhen/
  niaoxiangzhen/pozhenjue）；zhenfa 13 张启用清单全覆盖（3 转换 + 10
  deferred 实体位/5 种）。

## 验收口径

- **回归已加入**：`zf_dizaizhen_major_faction_draw`——子局①（魏魏
  蜀：两魏并列最大均大、蜀非大独摸 1、本体离结算区）+ 子局②（全魏无
  非大者出牌被拒）；`zf_real_host_full_lifecycle` 六步（安装+激活→建
  局→对局中卸载→锁定副本续玩→重装 Revoked→import 被拒）。
- **全量通过**：engine-native **427 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 109→110 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（地载阵不在标准场景，纯版本位移）。
- **剩余**：zhenfa 五种阵法（队列/围攻依赖，随国战队列体系建模另评估）；
  ow/xianjian 武将管线；U07 其余 9 条技能声明。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（18 文件，相对 noname-next 根）。
