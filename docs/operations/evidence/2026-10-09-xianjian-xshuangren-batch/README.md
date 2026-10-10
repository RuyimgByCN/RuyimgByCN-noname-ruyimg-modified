# U05 xianjian 第三十八批切片（规则 v180，2026-10-09）

雪见（pal_xuejian）双刃（xshuangren）+双刃·殁（xshuangren2）闭环——
**雪见升全闭环**（离场区基建第三次消费）。前置：
[破云批](../2026-10-09-xianjian-poyun-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js xshuangren/xshuangren2）

- `xshuangren`：player loseEnd——失去 equip1（武器）时：旧 mark 弃置
  → 新武器置武将牌上（storage mark）+addAdditionalSkill 获得该武器技
  能。
- `xshuangren2`：player dieBegin——弃置 storage 武器牌。

## 改动（分开陈述）

- **宿主行为**：`XianjianXshuangren`（tag 203，CardsDiscarded forced：
  filter 弃牌含武器 kind（equip_slot=Weapon）→ Act 弃堆回收武器置离场
  区+grant_equip_skills 武器技能随迁）+ `XianjianXshuangren2`（tag
  204，Death 前置窗口 forced：死亡时弃置离场区全部牌[通用区清空=登记
  差异]）——六表+ALL 202→204。无新状态字段（off_court 复用）。
- **登记差异**：①仅弃置路径触发（CardsDiscarded），被替换/被获得不触
  发=上游 loseEnd 全失去面收窄 ②武器技能随迁按 grant_equip_skills 全
  量（上游 addAdditionalSkill 同义）③殁清空离场区全部=通用区共用语义。
- **cards.json**：雪见 xshuangren mapping null→host；追加 xshuangren2
  常驻实例登记 + rehash 两步；**版本** 179→180；a0 追加 203/204。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`xuejian_xshuangren_offcourt_and_
  death_discard`（弃牌阶段弃手牌武器→离场区收武器（非弃堆）→死亡→
  殁清离场区+武器入弃堆+死亡照常）。
- **全量通过**：engine-native **513 passed / 0 failed / 1 ignored**
  （v179 基线 512 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 179→180 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。
- **测试坑（登记）**：弃牌阶段压牌验算——弃牌量=手牌-体力上限（非固
  定 1）。

## 剩余

- 雪见升全闭环。xianjian 其余 3/36 将（南宫煌 zhangmu、柳梦璃
  tianxian、慕容紫英 qianfang——均按缺口族暂缓中）；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（10 文件，相对 noname-next
根）。
