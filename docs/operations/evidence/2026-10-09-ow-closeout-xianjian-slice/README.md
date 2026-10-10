# U05 ow 来源纵切收官 + xianjian 首切片（规则 v142，2026-10-09）

双段批：**① ow 来源纵切收官**——猎空（ow_liekong）双技登记 deferred
（区域离场结构性依赖：he 区牌入 storage 离场区+shanhui 回归含位置 "s"
的 createCard 重造——破坏牌守恒纪律，需专用隐藏区/离场区基建）后入包，
ow 包 24/24 武将全部入包（23 闭环 + 1 结构性暂缓）。**来源勘误**：此前
各批「上游 25 将」口径有误——character.js 实为 **24 武将**（与来源清
点 24 武将/107 技能一致），闭环 23/24。**② xianjian 首武将切片**——
新转换包 **shaai-xianjian-upstream-noname**（character 管线第二例）+
星璇（pal_xingxuan，qun 3 男）灵雪（lingxue）单技闭环；feizhua/leiyu
登记暂缓。前置：
[ow 纵切批二十四](../2026-10-09-ow-dva-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js lingxue +
character/ow/skill.js shanxian/shanhui）

- `lingxue`：trigger player recoverEnd forced——content changeHujia()
  （+1 护甲）。
- `shanxian/shanhui`：区域离场/回归（结构性依赖登记，不转换）。

## 改动（分开陈述）

- **① ow 收官**：cards.json 增 猎空（shu 3 女，shanxian/shanhui 双
  mapping null+结构性依赖 reason）+ npack 资产哈希重算。无代码变化、
  无版本递增（deferred 登记不动语义）。
- **② xianjian 首切片**：新转换包 shaai-xianjian-upstream-noname
  （manifest+assets/cards.json+package.npack.json 三件，ow 模板复制改
  标识+rehash）+ `XianjianLingxue`（tag 141，Recover 窗口触发
  optional:false=上游 forced 同口径）——filter（自己回复且量>0）→
  Act：change_hujia(+1)。
- **登记暂缓**：feizhua（useCard 杀单目标+纯距 1 内玩家询问换位）、
  leiyu（phaseEnd 雷域存储列表）。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`xingxuan_lingxue_recover_grants_
  hujia`（自回复吃桃→+1 甲/多次回复各 +1）。
- **全量通过**：engine-native **463 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v141 基线 462 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 141→142 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。
- **测试坑（登记）**：桃自回复 targets 为空（非 [self]——IllegalTargets
  一次修正）；Recover 载荷含 by_card（filter 模式 `..` 补齐）。

## 剩余

- ow 猎空 deferred（区域离场基建立项待排）；xianjian 其余 35 武将逐
  武将纵切；开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种
  阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（16 文件，相对 noname-next 根）。
