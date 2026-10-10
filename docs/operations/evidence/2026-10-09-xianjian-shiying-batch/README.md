# U05 xianjian 第二十九批切片（规则 v171，2026-10-09）

月琦（pal_yueqi，wei 3 女）适盈（shiying）单技闭环——**死亡取消型首
例**（v170 死亡前置窗口消费）；tianwu 表现层登记不转、liguang moveCard
基建暂缓。前置：
[长念批](../2026-10-09-xianjian-changnian-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js shiying/tianwu/liguang）

- `shiying`：global dieBefore limited——filter !storage && 他人死亡；
  content trigger.cancel()+awakenSkill+双方 maxHp/hp 置 3+clearSkills
  双方+弃 hej 全部+各摸 3。
- `tianwu`：useCardToBegin 单目标敌方 frequent——getDebuff+addTempSkill
  tianwu2（空壳）=**纯表现技登记不转**。
- `liguang`：phaseEnd direct——弃一张手牌+moveCard（移动场上装备/判定
  区牌）=moveCard 基建缺口暂缓。

## 改动（分开陈述）

- **状态**：`PlayerState.death_cancelled: bool`（死亡拦截标记——
  DeathMachine 直读跳过死亡执行，存活态保持）+ `shiying_used: bool`
  （limited 一次性）。存档 v32→**v33**。
- **基建**：DeathMachine step 开头直读拦截标记：置位则清除并
  `Flow::Done`（跳过 alive=false/清牌/移除技能——存活态保持，前置窗口
  技已重置体力；上游 dieBefore trigger.cancel 同口径）。
- **宿主行为**：`XianjianShiying`（tag 192，Death Before 窗口 forced：
  置一次性+死亡者拦截标记+双方 max_hp/hp 置 3+dying=false+clearSkills
  双方（SkillToggled 日志=DeathMachine 同款）+弃 hej 全部（装备
  revoke）+各摸 3）——六表+ALL 191→192。
- **登记不转/暂缓**：tianwu（表现层 noop 登记不转）、liguang
  （moveCard 基建暂缓）。
- **cards.json**：月琦行（wei 3 女）+ rehash 两步；**版本** 170→171；
  a0 追加 192。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`yueqi_shiying_cancel_death_and_
  reset`（他人濒死求桃全 Pass→死亡帧→适盈取消：存活+拦截标记消费+
  双方 3/3+一次性（持技者侧）+各摸 3+hej 清弃+clearSkills 双方）。
- **全量通过**：engine-native **502 passed / 0 failed / 1 ignored**
  （v170 基线 501 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 170→171 + 双字段入 canonical_hash——14 场景全
  mismatch，重录后 replay_scenarios 2/2。
- **测试坑（登记）**：适盈 filter 为「他人死亡」——持技者不得是被杀
  者（grant 打到死亡者身上 filter 恒 false）。

## 剩余

- xianjian 其余 10/36 将（柳梦璃 tianxian、南宫煌 zhaoyao/zhangmu、雪
  见 xshuangren、姜云凡/仙卿/罗招嫣/慕容紫英按缺口族）；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next
根）。
