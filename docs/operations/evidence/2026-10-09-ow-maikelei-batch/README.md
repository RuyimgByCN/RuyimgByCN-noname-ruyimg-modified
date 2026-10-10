# U05 ow 纵切批十五：麦克雷神枪（规则 v132，2026-10-09）

ow 纵切批十五：**麦克雷**（ow_maikelei，shu 4 男）——神枪（shenqiang，
伤害后触发锁定）单技闭环；**shanguang 暂缓**（shanguang2 全卡牌禁用
子技：mod cardEnabled/cardUsable/cardRespondable/cardSavable 全 false）
+ **tiandan 暂缓**（createCard 凭空造杀×N+主动跳过出牌/弃牌阶段）。
上游 25 将已闭环第 14 将。前置：
[ow 纵切批十四](../2026-10-09-ow-ana-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js shenqiang）

- trigger source damageEnd forced：event.card 为杀 &&
  _status.currentPhase===player → `player.getStat().card.sha--`（本回
  合出杀计数 -1=可再出一张杀）。**无次数上限**——每次杀伤害各回退一
  次（链式：连续命中多张杀依次解锁）。

## 改动（分开陈述）

- **代码已实现**：`WudianShenqiang`（tag 129，Damage after 窗口触发、
  optional:false=上游 forced 同口径）——filter（来源=持有者 && 未防止
  && 杀（实体牌口径与飞刃/飞弹一致）&& 自己回合）→ Act：
  `turn.sha_used_play_phase = false`（杀限次回退；land_sha_used 地图效
  果加成杀不回退，登记）。
- **登记差异**：转化声明杀（龙胆/目镜等）按实体牌 kind 判定——转化
  牌伤害不触发（与飞刃/飞弹既有口径一致）；land 加成杀的额外标记不回
  退（龙颅山峰不在标准场景）。
- **登记暂缓**：shanguang（全卡牌禁用子技）、tiandan（createCard+主
  动跳阶段）。
- **转换包**：cards.json 增 麦克雷（shu 4 男，shenqiang mapping+双暂
  缓 reason）+ npack 资产哈希重算。无新状态字段——存档 v9 不变。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`maikelei_shenqiang_sha_limit_
  refund`（第一张杀命中 4→3/回退后第二张杀合法 3→2/链式回退第三张杀
  合法 2→1——对齐上游每伤害一回退无上限语义）。
- **全量通过**：engine-native **453 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v131 基线 452 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 131→132 入 canonical_hash——14 场景全 mismatch
  （纯版本位移），重录后 replay_scenarios 2/2。
- **测试坑（登记）**：初版断言「回退仅一次」与上游语义相悖——神枪无
  次数上限，每次杀伤害各回退（链式），测试改为链式断言后即绿；无距离
  豁免（第二张杀仍受距离校验）。

## 剩余

- ow 其余 11 武将逐武将纵切；xianjian 36 武将；虚拟 useCard 原语
  （huoyu/youzhan 同源）、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
