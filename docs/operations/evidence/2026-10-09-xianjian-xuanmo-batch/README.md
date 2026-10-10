# U05 xianjian 第三十二批切片（规则 v174，2026-10-09）

**实体发现基建首例**（createCard 缺口族的登记差异路径，台账待建排序③
首项以「实体池发现」差异方案落地）+ 夏侯瑾轩（pal_xiahoujinxuan，wei
3 男）玄墨（xuanmo）单技闭环；danqing 表现层登记不转。前置：
[离光批](../2026-10-09-xianjian-liguang-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js xuanmo/danqing）

- `xuanmo`：phaseUse usable:1——弃一张基本/装备/锦囊手牌置牌堆顶 →
  inpile randomGets(2) createCard 随机获得同类型 2 张。
- `danqing`：useCard 收集四花色 → phaseEnd chooseTarget getBuff(false)
  =**纯表现（无机制本体）登记不转**。

## 改动（分开陈述）

- **实体发现基建首例**（羊皮卷同款登记差异的通用化）：createCard 族
  以「牌堆+弃堆实体池随机发现该类型（yangpijuan_class 三分类）实体」
  替代凭空造牌；**登记差异**：上游 createCard 凭空造牌（无实体来源），
  原生从实体池发现；实体不足给余量。
- **宿主行为**：`XianjianXuanmo`（tag 195，phaseUse usable:1 主动：
  SelectCard 弃三类手牌（必选）→TargetChoice 置牌堆顶+实体池 RNG 洗
  牌发现同类型 2 张入手）——六表+ALL 194→195。无新状态字段。
- **登记不转**：danqing（纯表现）。
- **cards.json**：夏侯瑾轩行（wei 3 男）+ rehash 两步；**版本**
  173→174；a0 追加 195。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`xiahoujinxuan_xuanmo_discover_
  same_class`（弃锦囊置牌堆顶→实体池发现同类型（锦囊）2 张入手、不含
  置顶牌、离牌堆/弃堆）。
- **全量通过**：engine-native **505 passed / 0 failed / 1 ignored**
  （v173 基线 504 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 173→174 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。

## 剩余

- 夏侯瑾轩半闭环（xuanmo 闭环/danqing 登记不转）。xianjian 其余 8/36
  将（按缺口族）；实体发现基建可复用：苏媚 xiepan（随机装备）、绝世
  方机关牌族、dujiang 毒牌（仍需新牌种）；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（10 文件，相对 noname-next
根）。
