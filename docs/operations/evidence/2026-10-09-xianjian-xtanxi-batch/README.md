# U05 xianjian 第三十四批切片（规则 v176，2026-10-09）

仙卿（pal_xianqing，qun 4 男）探息（xtanxi）单技闭环；xiaoyue 复合缺
口登记暂缓。前置：
[苏媚批](../2026-10-09-xianjian-sumei-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js xtanxi/xiaoyue）

- `xtanxi`：phaseUse usable:1 filterCard true——弃一张手牌→持有同名
  牌手牌的敌人中随机一名→亮牌 gainPlayerCard(visible) 随机获得其一张
  手牌（addExpose 亮牌纯表现不迁移；无同名牌持有者则牌照弃）。
- `xiaoyue`：global roundStart forced——随机红手牌视为使用+changeHujia
  ——roundStart 事件缺口+自动虚拟用牌复合缺口暂缓。

## 改动（分开陈述）

- **宿主行为**：`XianjianXtanxi`（tag 198，phaseUse usable:1 主动：
  SelectCard 弃牌（可取消=放弃）→持同（牌）种手牌的其他角色中随机一
  名→随机获得其一张手牌（CardsGiven 日志）；无人持有同名牌则牌照弃）
  ——六表+ALL 197→198。**登记差异**：同名牌判定按牌种（sha/shan 各
  一种实体=get.name 同口径）；亮牌表现不迁移。无新状态字段。
- **登记暂缓**：xiaoyue（roundStart 事件+自动虚拟用牌复合缺口）。
- **cards.json**：仙卿行（qun 4 男）+ rehash 两步；**版本** 175→176；
  a0 追加 198。

## 验收口径

- **回归已加入**：g1_wudian 2 例——`xianqing_xtanxi_same_name_steal`
  （弃杀→持杀者随机夺取一张+弃牌入弃堆）+
  `xianqing_xtanxi_no_holder_cancel`（弃无中无人持同名牌→牌照弃无获
  得）。
- **全量通过**：engine-native **509 passed / 0 failed / 1 ignored**
  （v175 基线 507 +2）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 175→176 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。
- **工程坑（登记）**：host.rs 批内多脚本交叠写入致 enum 变体缺失+key
  表重复臂+孤儿体（三处交叠损伤）——每脚本先重新读文件、写入前
  count 断言、脚本失败立即停（勿叠加）。

## 剩余

- 仙卿半闭环（xtanxi 闭环/xiaoyue 暂缓）。xianjian 其余 8/36 将按缺
  口族；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（10 文件，相对 noname-next
根）。
