# U05 ow 纵切批十二：源氏飞刃（规则 v129，2026-10-09）

ow 纵切批十二：**源氏**（ow_yuanshi，qun 3 男）——飞刃（feiren，锁定
被动三钩）单技闭环；**lianpo 暂缓**（源定义不在 ow/skill.js=共享技能
文件，feiying 同款）+ **zhanlong 暂缓**（limited+createCard 凭空造牌
3 杀+zhanlong2 临时技链）。上游 25 将已闭环第 11 将。前置：
[ow 纵切批十一](../2026-10-08-ow-luba-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js feiren）

- trigger source damageBegin forced：event.card 为杀 && 花色黑桃 &&
  event.notLink() → num++（伤害 +1；传导的二次结算为独立事件，
  notLink 排除）。
- mod targetInRange：杀恒可达（无视距离）。
- mod selectTarget：梅花杀（实体花色）且目标上限非无限时 range[1]++
  （额外目标 +1）。

## 改动（分开陈述）

- **代码已实现**：`DamageData.from_chain: bool` 新字段（铁索传导 emit
  置 true，其余构造默认 false；入事件编解码）——上游 notLink 的原生
  对应物（主伤害/传导伤害首次可区分）；`WudianFeiren`（tag 126，锁定
  被动无机器）+ **三钩位**：commit_damage 增幅段（黑桃杀 && !from_chain
  → +1；花色按实体牌——火/雷杀为黑桃同样命中）+ PlayCard 杀距离校验
  豁免（feiren_range_free 与哨卫/贴房绅视并列，首目标与方天多目标循
  环均豁免）+ max_targets 加项 feiren_extra（梅花杀 +1，实体花色、转
  化声明牌种不影响）。
- **登记暂缓**：lianpo（共享技能文件）；zhanlong（limited+createCard
  凭空造牌+zhanlong2 临时技链——与羊皮卷「牌堆发现」替代路径需单独
  语义评估）。
- **登记差异**：无新增（三钩均与上游一一对应；伤害 +1 为数值增幅不另
  发事件——星君眼同款既有口径）。
- **转换包**：cards.json 增 源氏（qun 3 男，feiren mapping + 双暂缓
  reason）+ npack 资产哈希重算。存档 **v9**（随批十一一并首发：TurnState
  两字段+DamageData.from_chain 入事件编解码）。

## 验收口径

- **回归已加入**：g1_wudian 2 例——`yuanshi_feiren_bonus_range_and_
  club_extra_target`（黑桃杀 +1 4→2/无视距离距离 2 可达+红杀无加成/
  梅花杀双目标可达+非梅花双目标仍拒；连弩解除限次）、
  `yuanshi_feiren_chain_damage_not_bonused`（铁索连 1、2 后黑桃火杀：
  主伤害 +1 4→2、传导不加成 4→3）。
- **全量通过**：engine-native **450 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v128 基线 448 +2）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 128→129 + DamageData.from_chain 入 canonical_hash
  事件流——14 场景全 mismatch（标准场景伤害 from_chain 恒 false，纯
  版本/零值字段位移），重录后 replay_scenarios 2/2。
- **测试坑（登记）**：飞刃无次数豁免——连出多杀需先装诸葛连弩；a0
  编码守卫样例 DamageData 字面量随新字段同步。

## 剩余

- ow 其余 14 武将（含法老之鹰 feidan 可闭环+huoyu 虚拟 useCard 暂缓、
  feiying 共享文件暂缓）逐武将纵切；xianjian 36 武将；U07 duoqi/
  youzhan 暂缓；zhenfa 五种阵法。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（16 文件，相对 noname-next 根）。
