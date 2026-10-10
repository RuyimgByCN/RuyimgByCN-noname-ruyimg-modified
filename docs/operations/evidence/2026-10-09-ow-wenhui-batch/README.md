# U05 xianjian 第五切片：温慧龙翔（规则 v146，2026-10-09）

xianjian 第五武将切片：**温慧**（pal_wenhui，wei 4 男）——龙翔
（longxiang，ShaResolve 内联钩）单技闭环；**huxi 暂缓**（chooseToUse
viewAs sha——无实体牌虚拟用牌+respondSha，虚拟用牌原语缺口）。xianj-
ian 包 5/36 将入包（5 闭环）。前置：
[xianjian 第四切片](../2026-10-09-ow-xia-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js longxiang）

- shaBegin（check 询问式但内容无选择面——上游 check 为 AI 姿态，强制
  内容）：filter 目标手牌数>自己手牌数 → 目标 discard(hs.randomGets(
  hs.length - player.countCards("h")))（随机弃差值张；杀在结算区不计
  入 countCards）。

## 改动（分开陈述）

- **代码已实现**：`WudianLongxiang`（tag 147 目录标记技）+ **ShaResolve
  内联钩**（`!asked` 分支闪弹钩后）——持技判定 WUDIAN_LONGXIANG_KEY+
  目标存活+目标手牌>自己手牌 → 目标 he 池 swap_remove 抽取式随机弃差
  值张（逐张日志；上游 randomGets(diff) 同口径）。**零新增机器字段**
  （无询问面；每杀至多一次由 !asked 分支天然保证）——存档 v16 不变。
- **登记差异**：上游 check=AI 姿态（内容强制），原生无条件执行同口
  径；杀在结算区不计入来源手牌（上游 countCards 同口径）。
- **登记暂缓**：huxi（虚拟用牌缺口）。
- **转换包**：cards.json 增 温慧（wei 4 男，longxiang mapping+huxi 暂
  缓 reason）+ npack 资产哈希重算。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`wenhui_longxiang_random_discard_
  diff`（4 张差值→随机弃 4/杀照常命中 4→3）。
- **全量通过**：engine-native **467 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v145 基线 466 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 145→146 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。
- **测试坑（登记）**：差值口径——杀在结算区不计入来源手牌（初版按
  「打出的杀仍算 1 张」写差值 3 断言失败，对齐上游 countCards 后为 4）。

## 剩余

- xianjian 其余 31 武将逐武将纵切；ow 猎空 deferred（离场区基建立项）；
  开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
