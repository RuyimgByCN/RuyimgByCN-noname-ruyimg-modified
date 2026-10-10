# U05 xianjian 第二十四批切片（规则 v165，2026-10-09）

雪见（pal_xuejian，shu 3 女）神暮（shenmu）+毒刺（duci）双技闭环；
xshuangren（双刃）武器技能转移缺口登记暂缓。前置：
[诚信补完批](../2026-10-09-xianjian-chengxin-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js shenmu/duci/xshuangren）

- `shenmu`：global dying check 询问式——亮全部手牌（纯表现不迁移）→
  弃全部红手牌→濒死者 recover 1 + draw(红牌数)。
- `duci`：player loseEnd direct——失去装备区牌（cards original=="e"）
  → chooseTarget 距 1 内其他角色 → damage 1。
- `xshuangren`：loseEnd 武器失去→置武将牌上+addAdditionalSkill 获得该
  武器技能；dieBegin 弃置——**武器技能转移复合缺口暂缓**。

## 改动（分开陈述）

- **宿主行为**：`XianjianShenmu`（tag 182，global Dying 确认门=濒死救
  援通道第三例：弃全部红手牌→濒死者 recover 1+摸等量；**登记差异**：
  check 询问简化为红手牌有无 filter）+ `XianjianDuci`（tag 183，
  CardsDiscarded direct：弃牌含装备 kind 时询问距离 1 内一名其他角色
  受 1 伤害（chooseTarget 取消=放弃）；**登记差异**：仅弃置路径触发
  （CardsDiscarded 两发射点），被获得/替换不触发=上游 loseEnd 全失去
  面收窄）——六表+ALL 181→183。无新状态字段。
- **登记暂缓**：xshuangren（武器技能转移复合缺口，mapping null+reason）。
- **cards.json**：雪见行（shu 3 女）+ rehash 两步；**版本** 164→165；
  a0 追加 182/183。

## 验收口径

- **回归已加入**：g1_wudian 2 例——`xuejian_shenmu_red_discard_rescue`
  （他人濒死确认→弃全部红手牌（黑牌保留）→濒死者回复 1 摸等量）+
  `xuejian_duci_equip_discard_damage`（弃牌阶段弃装备→距离 1 目标受 1
  伤→装备入弃堆；弃基本牌对照不触发）。
- **全量通过**：engine-native **495 passed / 0 failed / 1 ignored**
  （v164 基线 493 +2）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 164→165 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。
- **测试坑（登记）**：①濒死救援量验算——压 1 血再伤 1（hp=0 濒死），
  recover 1 后恰脱离；压 0 血伤 1 会到 -1，recover 1 后仍濒死（续求桃
  链）②神暮「弃全部红」断言=无红残留而非手牌清空（黑牌保留）。

## 剩余

- xianjian 其余 13/36 将（柳梦璃 runxin 可做；猎空属 ow 来源另算）；
  xshuangren 武器技能转移、changnian 死亡拦截、响应链、createCard、
  Recover 前置窗口等缺口族与长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（10 文件，相对 noname-next
根）。
