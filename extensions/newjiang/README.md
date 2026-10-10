# NEWJIANG 确定性移植（2026-08-03 接管批次）

上游 102 个技能已全部定义（103 个 def，含 dunyong_effect 辅助技能），
编译进 `newjiang.npack.json`（logic_hash=538e779c7016b8b8d90fba141d76b7a638061eab2eebb82fde2dc867b73994dc）。

覆盖等级（与既有包一致的分级）：
- 精确表达：beiyu、biancai、dckanghui、dcyiju、duanwan、dunyong、jingyi、kangli、
  lkzhongzhuang、mpbishi、mpjiusong、spqishe、yizu、weifeng2、duchi、spyishi、
  dcduliang(+dcduliang2 标记)、zhenbian、mpsijiu、yjshenwei(view_as)、dczhifeng(view_as)、
  lunshi(view_as)、dchuairen(view_as)、tenggu(dying 近似)、renjia(近似)、
  ciren/gnsheque/dcweijing/xiongjin/shiming/lkbushi 等（近似收益保留）。
- 近似表达（核心收益保留、复杂分支注明）：其余多数技能见各 .skill 文件注释。
- 空定义+注释（依赖 expansion/标记/动态转化等未建模原语）：mbaosi、mpmiaoxi、
  fuyu、zhanshi、duwang、cibei、gnjinfan、gnjinfan_gain、gongqiao、xianmou、
  sangu、biejun、spshidi、weifeng/weifeng3、mpsixiao、mpmaotao、baoxi、
  yj_yanyu、yjzhuiji、yanxie、yjqiaosi、qiqiao、yuliao、xinxhzhiyan 辅助等。

未建模缺口登记（需新原语，见 ABI-COORDINATION.md）：
- expansion 技能牌堆链（铃/刺/材/料）：gnjinfan/duwang/cibei/yuliao/qiqiao/yanxie
- 衍生卡 createCard2：gongqiao/tenggu 防具
- 按花色集合的临时禁用/状态：youtan/zhancai/zhenbian(花色)/qingbei/kousheng
- 双人同时二选一：dczhonge
- 拼点连续循环：dcchizei/dcxiujue/fuyu/zhanshi
