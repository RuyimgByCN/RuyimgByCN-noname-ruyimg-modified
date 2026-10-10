# U05 ow 纵切批十三：法老之鹰飞弹（规则 v130，2026-10-09）

ow 纵切批十三：**法老之鹰**（ow_falaozhiying，shu 3 女）——飞弹
（feidan，杀伤害后触发）单技闭环；**huoyu 暂缓**（limited+虚拟
useCard——chiyuxi 凭空用牌×2，youzhan 同款虚拟用牌原语缺口）+
**feiying 暂缓**（源定义不在 ow/skill.js 共享技能文件）。上游 25 将已
闭环第 12 将。前置：
[ow 纵切批十二](../2026-10-09-ow-yuanshi-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js feidan）

- trigger source damageAfter direct：he 非空 && event.card 为杀 && 场
  上存在「受伤者（event.player）距离 1 内的其他角色」。
- content：chooseToDiscard（1 张，可取消）→ 弃牌成功则目标按座位序逐
  个 damage()（无牌、来源=持有者、1 点）。
- mod targetInRange：杀无视距离（同飞刃写法）。

## 改动（分开陈述）

- **代码已实现**：`WudianFeidan`（tag 127，Damage after 窗口触发，
  optional:false=上游 direct 同口径）触发机——filter（来源=持有者 &&
  未防止有量 && 杀 && 受伤者存活 && 其距离 1 内存在其他存活者）→
  Act 建弹射目标（受伤者 computed_distance≤1 的其他存活者）与弃牌选
  项（he，可取消）→ SelectCard 读应答（取消=不弃不弹；弃 1 张→按座
  位序逐目标 emit 1 点无牌伤害，来源=持有者——上游 damage() 无参同口
  径）。杀无距离豁免：PlayCard 首目标+方天多目标循环（feiren 通用化
  为 feiren||feidan）。
- **登记差异/口径**：弹射目标自动全收（上游 targets=filterPlayer 全集，
  仅弃牌询问——同口径）；受伤者死亡不触发（上游 damageAfter 时点已死
  玩家不弹射的保守口径，登记）；弹射伤害 card=None（不触发飞弹自身/
  其他卡牌钩——递归天然断绝）。
- **登记暂缓**：huoyu（虚拟 useCard 原语缺口）、feiying（共享文件）。
- **转换包**：cards.json 增 法老之鹰（shu 3 女，feidan mapping+双暂
  缓 reason）+ npack 资产哈希重算。无新状态字段——存档 v9 不变。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`falaozhiying_feidan_splash_cancel_
  and_range`（杀 1 号弹射 0/2 号各受 1 含自己/放弃不弹/距离 2 杀可达/
  决斗伤害不触发）。
- **全量通过**：engine-native **451 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v129 基线 450 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 129→130 入 canonical_hash——14 场景全 mismatch
  （纯版本位移），重录后 replay_scenarios 2/2。
- **测试坑（登记）**：无（首跑即绿——触发机/距离豁免/放弃分支复用批
  九至批十二全部既有形态）。

## 剩余

- ow 其余 13 武将逐武将纵切；xianjian 36 武将；U07 duoqi/youzhan 暂缓
  （虚拟 useCard 原语与 huoyu 同源）；zhenfa 五种阵法。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（15 文件，相对 noname-next 根）。
