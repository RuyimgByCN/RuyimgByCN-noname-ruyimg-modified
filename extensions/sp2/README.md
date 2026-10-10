# SP2 确定性移植

武将 `xf_tangzi`（唐咨）：魏，4 体力，只授予组合后的父技能 `xinfu_xingzhao`；子技能不重复挂载，避免伤害通道叠加。

`juntun`：准备阶段体力上限 X 大于一时减一点体力上限，再摸 X 张牌。旧 JS 在减上限事件结算前读取 X。

`zhihu`：每回合限两次，对其他角色造成伤害后摸两张牌。

`chenggong`：一名角色使用目标数大于一的牌时，可令使用者摸一张牌。

`yanjiao2`：临时效果存在时，手牌上限 -1。

`redaoji2`：临时效果存在时，不能使用或打出【杀】。

`dcbenshi`：攻击范围 +1，使用【杀】时必须指定攻击范围内的所有其他角色。

`xiying2`：本回合内不能使用、打出或以牌救援；三条动作通道分别防御校验。

`dczhuiwang`：体力值不小于目标时，与该目标的最终距离视为 1。

`nhguizhu`：每回合限一次；任意角色进入濒死状态时，拥有者可摸两张牌。

`jiaojie`：红色手牌不计入手牌上限且不能在弃牌阶段弃置；黑色牌无距离和次数限制。

`huangkong`：回合外没有手牌时成为【杀】或锦囊牌目标，锁定摸两张牌。

`quanjiu`：实体【酒】视为【杀】，且由实体【酒】转化出的【杀】不计入使用次数。

`starzhonggu`：主公技；轮数不小于场上存活群势力角色数时摸牌数 +2，否则 -1。

`xianshuai`（先率）：每轮第一次有存活来源造成伤害后锁定摸一张牌；若伤害来源是拥有者，再令受伤角色受到一点无来源伤害。

`reliewei`：任意角色进入濒死时可摸一张；自己的回合内不限次数，否则每回合至多发动当前体力值次。

`recuorui`（摧锐）：限定技；出牌阶段选择一至当前体力值名有手牌的其他角色，按提交顺序依次
获得其一张当前手牌。目标上限、手牌状态和每次选择都由权威状态重检；与 `reliewei` 闭合发布
`re_niujin`。

`starsujun`：使用牌后，若剩余手牌中的基本牌与非基本牌数量相等，可摸两张牌。

`tanbei_effect1` / `tanbei_effect2`：`贪狈`两个选项的纯临时状态标记；旧定义仅含 `charlotte` 元数据。

`kuimang`（溃蟒）：一名角色死亡后，若拥有者曾对其造成过伤害，锁定摸两张牌。

`gangzhi`（刚直）：拥有者与其他角色之间造成的伤害改为等量体力流失；无来源伤害、自伤以及已离场来源对拥有者造成的伤害不转换。

`starjianyu`（翦羽）：其他角色在拥有者回合内失去装备区牌后，拥有者锁定摸一张牌；同一批移动不按失去的装备牌张数重复触发。

`xiongsuan`（凶算）：出牌阶段每回合限一次，弃置一张手牌并选择一名角色，对其造成 1 点普通伤害，然后摸三张牌；目标不是自己时再失去 1 点体力。主动技目标、手牌输入和次数均由包元数据声明并由引擎校验。

`nhxianshou`（仙授）：出牌阶段限一次，令一名角色摸牌；其未受伤时摸两张，否则摸一张。

`nhhuoqi`（活气）：出牌阶段限一次，弃置一张手牌或装备牌，令一名全场最低体力角色回复一点体力并摸一张牌。

`nhyinbing`（阴兵）：拥有者使用的【杀】造成伤害时改为等量体力流失；其他角色失去体力后，拥有者锁定摸一张牌。仅逻辑【杀】走转换通道，其他伤害保持不变。

`xinfu_xingzhao3`（兴棹伤害子技能）：全场无人受伤或至少四名存活角色受伤时，拥有者造成的伤害 +1。

`xz_xunxun`（兴棹的恂恂）：至少一名存活角色受伤时，摸牌阶段可观看牌堆顶四张牌，自由排列两张于牌堆顶、两张于牌堆底。

`xinfu_xingzhao2`（兴棹阶段子技能）：至少三名存活角色受伤时，锁定跳过判定阶段和弃牌阶段。

`xinfu_xingzhao`（兴棹）：组合上述三档能力；至少两人受伤时，每次有装备牌进入或离开自己的装备区便摸一张，替换装备因离开和进入各结算一次。

`refuqi`（伏骑）：使用牌时，拥有者至响应者的最终距离不大于 1 则该角色不能响应；`re_quyi` 与 SP 包的 `jiaozi` 组成完整武将闭包。

SP2 对 SP 声明精确包依赖，确保直接投放扩展目录时先加载 `jiaozi` 的唯一权威定义。

`nifu`（匿伏）：任意角色回合结束时，锁定将拥有者手牌精确调整至四张；超出部分由拥有者选择弃置。

`lxzhuixi`（追袭）：伤害来源与受伤角色的翻面状态不同时，伤害 +1；拥有者作为来源或受伤者时分别走来源侧、受伤侧修正通道，双方都拥有技能时可各自生效。

`langmie`（狼灭）：其他角色的出牌阶段结束时，若该阶段至少两张牌属于同一逻辑
基本/锦囊/装备类别，可摸一张牌；其他角色结束阶段开始时，若其本回合造成的伤害总量
大于 1，可弃置一张手牌或装备牌并对其造成 1 点普通伤害。阶段类别历史使用有界位集，
弃牌和后续伤害跨选择屏障依次结算。

```bash
cargo run -q -p content-cli -- pack \
  content/original/ported/sp2/juntun.skill \
  content/original/ported/sp2/zhihu.skill \
  content/original/ported/sp2/chenggong.skill \
  content/original/ported/sp2/yanjiao2.skill \
  content/original/ported/sp2/redaoji2.skill \
  content/original/ported/sp2/dcbenshi.skill \
  content/original/ported/sp2/xiying2.skill \
  content/original/ported/sp2/dczhuiwang.skill \
  content/original/ported/sp2/nhguizhu.skill \
  content/original/ported/sp2/jiaojie.skill \
  content/original/ported/sp2/huangkong.skill \
  content/original/ported/sp2/quanjiu.skill \
  content/original/ported/sp2/starzhonggu.skill \
  content/original/ported/sp2/xianshuai.skill \
  content/original/ported/sp2/reliewei.skill \
  content/original/ported/sp2/starsujun.skill \
  content/original/ported/sp2/tanbei_effect1.skill \
  content/original/ported/sp2/tanbei_effect2.skill \
  content/original/ported/sp2/kuimang.skill \
  content/original/ported/sp2/gangzhi.skill \
  content/original/ported/sp2/starjianyu.skill \
  content/original/ported/sp2/xiongsuan.skill \
  content/original/ported/sp2/nhxianshou.skill \
  content/original/ported/sp2/nhhuoqi.skill \
  content/original/ported/sp2/nhyinbing.skill \
  content/original/ported/sp2/xinfu_xingzhao3.skill \
  content/original/ported/sp2/xinfu_xingzhao.skill \
  content/original/ported/sp2/xinfu_xingzhao2.skill \
  content/original/ported/sp2/xz_xunxun.skill \
  content/original/ported/sp2/refuqi.skill \
  content/original/ported/sp2/nifu.skill \
  content/original/ported/sp2/lxzhuixi.skill \
  content/original/ported/sp2/langmie.skill \
  content/original/ported/sp2/recuorui.skill \
  --characters content/original/ported/sp2/characters.json \
  --metadata content/original/ported/sp2/metadata.json \
  --out content/original/ported/sp2/sp2.npack.json \
  --package-id sp2
```
