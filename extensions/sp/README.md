# SP 确定性移植

当前包只包含已完成语义对照和 Golden 的技能；未完整移植全部技能的武将不会提前写入
`character_index`。

`zhengnan`（征南）：其他角色死亡后可摸三张牌，再从尚未拥有的界武圣、当先、制蛮中获得
一项；`guansuo` 与已重写的 `xiefang` 一并形成完整武将闭包。

`kuangfu`（狂斧）：使用【杀】造成伤害后，可弃置目标的一张装备，或将兼容装备移动到自己的
装备区；同栏装备按标准替换规则进入弃牌堆。

`shanjia`（缮甲）：只累计本人本局使用的装备牌且封顶 7；出牌阶段开始可摸等量牌再强制
弃置等量手牌/装备，弃置装备区牌后可使用一张无距离限制但计通常次数的虚拟【杀】。

`jsp_guanyu`（SP 关羽）：魏，4 体力，通过 REFRESH 依赖复用 `new_rewusheng`，并授予
本包 `danji`。`danji` 觉醒后获得 STANDARD `mashu` 与本包 `nuzhan`。

| 技能 | 原角色 | 状态 |
|---|---|---|
| `gongao`（功獒） | `zhugedan` | `death` → 加 1 体力上限 → 回复 1，Golden 已覆盖 |
| `liewei`（裂围） | `fuhuanghou` | 击杀后可摸 3，Golden 已覆盖 |
| `zhuishe`（追摄） | `caiyang` | 使用【杀】次数上限 +1，modifier 查询已覆盖 |
| `weizhong`（威重） | `zhugedan` | 体力上限变化后按全场最少手牌状态摸 2/1，Golden 已覆盖 |
| `wangzun3` | 临时技能 | 手牌上限 -1 |
| `sanlve_skill` | 装备技能 | 手牌上限、攻击范围、【杀】次数上限各 +1 |
| `xuwangzhimian` | 装备技能 | 摸牌数 +2、手牌上限 -1 |
| `qingyi3` | 临时技能 | 使用所有牌无距离限制 |
| `zhanyi_trick` | 临时技能 | 使用所有牌无距离限制 |
| `zhanyi_basic_sha` | `zhanyi_basic` 子技能 | 手牌中的基本牌可当【杀】使用或打出 |
| `zhanyi_basic_tao` | `zhanyi_basic` 子技能 | 手牌中的基本牌可当【桃】使用，包括濒死救援 |
| `zhanyi_basic_jiu` | `zhanyi_basic` 子技能 | 手牌中的基本牌可当【酒】使用，包括濒死自救 |
| `zhidao2` | 临时技能 | 使用牌不能指定其他角色 |
| `rebiluan2` | 临时技能 | 同名状态标记修正其他角色计算与拥有者的距离，支持负值 |
| `xiefang`（协芳） | `sp_diaochan` | 至其他角色的距离减少场上存活女性角色数，查询时动态重算 |
| `zhuiji`（追姬） | `old_machao` | 体力值不小于目标时，与该目标的最终距离视为 1 |
| `yicong`（义从） | `gongsunzan` | 体力大于 2 时距离 -1；不大于 2 时其他角色至其距离 +1 |
| `xiandeng`（先登） | `quyi` | 【杀】次数上限 +1；本回合第一张【杀】无距离限制 |
| `guixiu`（闺秀） | `sp_daqiao` | 成为【杀】目标且手牌数小于体力值时，可摸 1 张牌 |
| `fenyue2` | 临时技能 | 不能使用或打出手牌；非手牌响应不受影响 |
| `olhexu`（和煦） | `ol_dongguiren` | 手牌上限增加手牌中非伤害牌的类别数（基本/锦囊/装备，去重） |
| `guilongzhanyuedao` | 装备技能 | 红色【杀】不可被响应 |
| `chouhai`（仇海） | `sunhao` | 没有手牌时受到的伤害 +1 |
| `xinfu_gongqing`（公清） | `panjun` | 来源攻击范围小于 3 时伤害封顶 1，大于 3 时伤害 +1 |
| `jugu`（巨贾） | `cuiyan` | 初始额外获得等同体力上限的手牌，手牌上限增加体力上限 |
| `zhuangshu_equip` | 装备技能 | 出牌阶段结束时将手牌补至手牌上限与 5 中的较小值 |
| `ranshang`（燃殇） | `chengpu` | 受到火焰伤害后，按伤害点数获得“燃”标记 |
| `ranshang2` | `chengpu` 的子技能 | 结束阶段按“燃”标记失去体力，超过 2 枚时再减 2 点体力上限并摸 2 |
| `hanyong`（悍勇） | `wutugu` | 受伤时使用南蛮、万箭或无属性黑桃杀可令当前牌基础伤害 +1；轮数不大于体力时获得“燃”，兀突骨闭包已发布 |
| `nuzhan`（怒斩） | `jsp_guanyu` 的觉醒衍生 | 单张锦囊转化的【杀】不计通常次数；单张装备转化的【杀】基础伤害 +1 |
| `danji`（单骑） | `jsp_guanyu` | 刘备不为主公且手牌多于体力时觉醒，减 1 点体力上限并获得马术、怒斩 |
| `jingong2` | `jingong` 的子技能 | 本回合未造成伤害时，结束阶段失去 1 点体力 |
| `shengxi`（生息） | `liushan` | 未造成伤害的回合进入弃牌阶段时，可摸 2 张牌 |
| `olzhennan2` | `olzhennan` 的分组技能 | 【南蛮入侵】不能选择拥有者 |
| `jiaozi`（骄恣） | `sp_machao` 等 | 手牌数为全场唯一最多时，造成或受到的伤害 +1 |
| `olbiri`（蔽日） | 独立技能 | 每回合首次有角色非因摸牌获得牌后锁定摸 1 |
| `shoucheng`（守成） | 独立技能 | 非当前回合角色失去手牌后空手时，可令该角色摸 1；按失牌角色逐次结算 |
| `shenxian`（甚贤） | `zhangxingcai` | 自己回合外，其他角色因弃置失去基本牌时可摸 1；每名角色的回合限一次，移动原因与使用/响应严格区分 |
| `rechouhai`（仇海） | `sunhao` | 没有手牌时，仅受到逻辑【杀】造成的伤害 +1 |
| `shulv`（熟虑） | `key_rei` | 手牌数大于体力时，出牌阶段限一次，弃置 1 张手牌或装备并摸 1 |
| `xinfu_zhanji`（展骥） | `xinfu_songsang` 的衍生技能 | 自己出牌阶段内，每次因其他效果摸牌后锁定再摸 1，且不会递归触发自身 |
| `xinfu_songsang`（送丧） | `re_jsp_pangtong` | 其他角色死亡后可发动一次；受伤回复 1，否则加 1 体力上限，随后永久获得「展骥」 |
| `xinfu_guolun`（过论） | `re_jsp_pangtong` | 出牌阶段限一次，暗选并展示目标一张手牌；可取消，或展示自己一张手牌/装备并交换入双方手牌，较小点数牌的原拥有者摸 1；SP庞统闭包已发布 |
| `xinfu_tushe`（图射） | `liuyan` | 使用有目标的非装备牌后，若手牌中没有基本牌，可摸目标数张牌 |
| `guofengyupao`（国风玉袍） | 装备技能 | 防具有效时，其他角色的普通锦囊不能指定拥有者；无视防具与防具无效通道可绕过 |
| `gzjili`（蒺藜） | `shamoke` | 当前全局回合内使用牌或打出响应牌的总数达到攻击范围时，可摸等量牌；沙摩柯武将闭包已发布 |
| `fuqi`（伏骑） | `quyi` | 使用牌时，响应者至拥有者的最终距离不大于 1 则不能响应；麴义武将闭包已发布 |
| `zhenlue`（缜略） | `sp_jiaxu`、`dc_sp_jiaxu` | 普通锦囊及其无懈链不可被响应；拥有者不能成为延时锦囊目标，Golden 已覆盖 |
| `qimenbagua`（奇门八卦） | 装备技能 | 防具有效时【杀】不能指定拥有者；无视防具可绕过，Golden 已覆盖 |
| `zniaoxiang`（鸟翔） | 独立技能 | 拥有者使用【杀】时每名目标需额外使用一张【闪】，Golden 已覆盖 |
| `spweiwu`（违忤） | `re_hansui` | 出牌阶段限一次，红色手牌或装备牌当无距离限制的【顺手牵羊】使用，Golden 已覆盖 |
| `yanzheng`（严整） | `jsp_caoren` | 手牌数大于体力值时，装备区牌可当【无懈可击】使用；与 `kuiwei` 组成完整武将闭包 |
| `kuiwei` / `kuiwei2`（溃围） | `jsp_caoren` | 结束阶段翻面并按场上武器数额外摸牌；下个摸牌阶段按当时武器数强制弃置，OFFLINE 武将闭包已发布 |
| `olyinji`（殷积） | `junk_liuyan` | 结束阶段体力不为全场唯一最高时，强制选择回复 1 点体力或增加 1 点体力上限 |
| `shenzhi`（神智） | `ganfuren` | 准备阶段可弃置全部手牌；冻结的弃牌数不小于当前体力值时回复 1 点体力 |
| `shushen`（淑慎） | `ganfuren` | 每回复 1 点体力均可独立选择另一名角色，令其强制选择摸 2 或回复 1；OFFLINE SP 甘夫人闭包已发布 |
| `zhenwei_three`（镇卫） | 统率三军诸葛瑾/文聘 | 存活拥有者令敌方计算到己方其他角色的距离 +1；多人稳定叠加 |
| `qiongshou`（穷守） | `huojun` | 游戏开始时若尚有可用装备栏，则废除全部可用栏位并摸 4；手牌上限 +4；霍峻另一技能未迁，不发布武将 |
| `sijian`（死谏） | `tianfeng` | 失去最后手牌后，可选择其他角色并弃置其一张手牌或装备牌；田丰闭包由 OFFLINE 包发布 |
| `wangong` / `wangong2`（挽弓） | `huangzu` | 使用基本牌后，下一张【杀】无距离与次数限制且基础伤害 +1；其他下一张牌只消费状态，黄祖闭包已发布 |
| `juyi`（举义） | `zhugedan` | 准备阶段体力上限大于存活人数时觉醒，摸体力上限张牌并获得 SHENHUA「崩坏」与 SP「威重」；诸葛诞闭包已发布 |
| `xiongyi`（雄异） | `mateng` | 限定技，出牌阶段令一至任意名角色各摸 3；结算后若自己体力值为全场最低则回复 1，OFFLINE 马腾闭包已发布 |
| `cunsi`（存嗣） | `mifuren` | 限定技，将全部手牌交给一名男性角色并授予「勇决」，然后翻面；糜夫人闭包已发布 |
| `yongjue`（勇决） | `cunsi` 衍生技能 | 每回合限一次，获得其他角色结算完成且未指定自己的实体【杀】 |
| `naman`（纳蛮） | `old_maliang` | 其他角色响应出的实体【杀】进入弃牌堆后可获得；自己的【杀】与虚拟牌不触发 |
| `fenxun` / `fenxun2`（奋迅） | `old_dingfeng` | 弃一张任意手牌/装备牌，将本回合到所选角色的距离视为 1 |
| `duanbing`（短兵） | `old_dingfeng` | 使用【杀】时可追加一名最终距离不大于 1 的合法目标 |
| `oldhuxiao`（虎啸） | `old_guanyinping` | 使用的【杀】被【闪】抵消后不计入使用次数；逻辑杀多目标时整次使用至多返还一次 |
| `xueji_old`（血祭） | `old_guanyinping` | 弃一张红色手牌/装备，对攻击范围内至多已损失体力名其他角色依座次各造成 1 点伤害并摸 1 |
| `oldwuji`（武继） | `old_guanyinping` | 结束阶段累计造成至少 3 点伤害时觉醒，失去虎啸、加 1 体力上限并回复 1 |
| `moukui`（谋溃） | `sp_mushun` | 使用【杀】指定目标后可取消、摸牌或弃目标牌；接受后仅该目标闪避同一张【杀】时反弃来源一张牌，COLLAB 武将闭包已发布 |
| `qiangwu`（枪舞） | `zhangxingcai`、`old_zhangxingcai` | 判定点数以下的【杀】无距离，以上的【杀】不计次数；SP/OLD 两个武将闭包均已发布 |
| `fenming`（奋命） | `chendong` | 结束阶段若横置，可依座次弃置每名有牌的横置角色一张手牌或装备；陈董闭包由 XIANDING 包发布 |
| `recanshi` / `recanshi2`（残蚀） | `sunhao` | 可按受伤角色/归命吴势力集合人数加摸；本回合使用杀或普通锦囊后自弃一张，孙皓闭包已发布 |
| `juesi`（决死） | `sp_pangde` | 弃置一张手牌【杀】，令攻击范围内有牌目标自弃一张；非【杀】且目标体力不低时经普通链路使用【决斗】，SP庞德闭包已发布 |
| `lizhan`（励战） | `sp_caoren` | 结束阶段可取消或选择任意名已受伤角色，依座次各摸一张 |
| `weikui` / `weikui2`（伪溃） | `sp_caoren` | 失去体力后按目标是否有【闪】弃牌或私密展示并尝试使用【杀】，回合内至该目标距离视为 1；SP曹仁闭包已发布 |
| `kaikang`（慷忾） | `caoang` | 距离 1 内角色成为【杀】目标后摸 1；非自身目标时交一张牌，受赠者可在移动触发后立即使用所获装备，曹昂闭包已发布 |
| `olcuorui`（挫锐） | `niujin` | 开局补至存活人数且至多八张；成为延时锦囊目标后跳过下个判定阶段，跨回合标记只登记一次，牛金闭包已发布 |
| `fengpo`（凤魄） | `mayunlu` | 本回合首次单目标【杀】与首次单目标【决斗】分别可发动；按目标当前方片手牌/装备数选择摸 X 加 1 伤或摸 1 加 X 伤，马云禄闭包已发布 |
| `hengzheng`（横征） | `sp_dongzhuo` | 体力为 1 或空手时可将本次摸牌固定为 0；从下一座起顺时针逐名获得一张当前可获得的手牌/装备/判定牌，移动触发逐次结算，SP董卓闭包已发布 |
| `ziyuan`（资援） | `mizhu` | 每个出牌阶段限一次，将有效点数之和恰为 13 的互异手牌一次性交给另一名角色；移动及嵌套触发结束后，该角色回复 1 点体力，糜竺闭包已发布 |
| `chongzhen`（冲阵） | `yy_zhaoyun`、`sp_zhaoyun` | 仅以名称含 `longdan`/`fanghun` 的视为技使用或打出【杀】【闪】时，可获得第一名对应角色一张手牌；燕幽赵云闭包已发布 |
| `hongde`（弘德） | `luji` | 单次获得或失去至少两张牌后，可令另一名角色摸一张牌；供 EXTRA「驭衡」精确依赖 |
| `bizheng`（弼政） | `dongyun` | 摸牌阶段结束可令另一名角色摸 2；随后双方手牌超过各自体力上限者依次弃置 2 |
| `xiashu`（霞姝） | `sunru` | 出牌阶段开始可交出全部手牌；目标展示非空手牌子集后，拥有者获得展示或未展示的一组 |
| `xinfu_guanchao`（观潮） | `yanjun` | 出牌阶段选择严格递增或递减；从第二张牌起每次仍满足全序列时摸 1，首次违例后本阶段失效 |
| `xinfu_guanwei`（观微） | `panjun` | 每回合限一次，一名角色同一出牌阶段使用至少两张且均为同一有效花色后，可弃 1 令其摸 2 并立即执行额外出牌阶段 |
| `lanjiang`（揽江） | `wuyan` | 结束阶段快照手牌不少于自己的角色，依座次逐一询问是否令自己摸 1；之后可对快照中当前手牌相等者造成 1 点伤害，若再有快照角色手牌更少则令其中一名摸 1 |
| `shuimeng`（水盟） | `yangxiu` | 出牌阶段结束时可拼点；严格获胜则对自己使用虚拟【无中生有】，否则由对方对自己使用虚拟【过河拆桥】 |

重新生成包：

```bash
cargo run -q -p content-cli -- pack \
  content/original/ported/sp/kuangfu.skill \
  content/original/ported/sp/shanjia.skill \
  content/original/ported/sp/gongao.skill \
  content/original/ported/sp/liewei.skill \
  content/original/ported/sp/olcuorui.skill \
  content/original/ported/sp/fengpo.skill \
  content/original/ported/sp/chongzhen.skill \
  content/original/ported/sp/hongde.skill \
  content/original/ported/sp/bizheng.skill \
  content/original/ported/sp/xiashu.skill \
  content/original/ported/sp/xinfu_guanchao.skill \
  content/original/ported/sp/xinfu_guanwei.skill \
  content/original/ported/sp/lanjiang.skill \
  content/original/ported/sp/zhuishe.skill \
  content/original/ported/sp/weizhong.skill \
  content/original/ported/sp/wangzun3.skill \
  content/original/ported/sp/sanlve_skill.skill \
  content/original/ported/sp/xuwangzhimian.skill \
  content/original/ported/sp/qingyi3.skill \
  content/original/ported/sp/zhanyi_trick.skill \
  content/original/ported/sp/zhanyi_basic_sha.skill \
  content/original/ported/sp/zhanyi_basic_tao.skill \
  content/original/ported/sp/zhanyi_basic_jiu.skill \
  content/original/ported/sp/olwuniang2.skill \
  content/original/ported/sp/tuifeng3.skill \
  content/original/ported/sp/zhidao2.skill \
  content/original/ported/sp/rebiluan2.skill \
  content/original/ported/sp/xiefang.skill \
  content/original/ported/sp/zhuiji.skill \
  content/original/ported/sp/yicong.skill \
  content/original/ported/sp/xiandeng.skill \
  content/original/ported/sp/guixiu.skill \
  content/original/ported/sp/fenyue2.skill \
  content/original/ported/sp/olhexu.skill \
  content/original/ported/sp/guilongzhanyuedao.skill \
  content/original/ported/sp/chouhai.skill \
  content/original/ported/sp/xinfu_gongqing.skill \
  content/original/ported/sp/jugu.skill \
  content/original/ported/sp/zhuangshu_equip.skill \
  content/original/ported/sp/ranshang.skill \
  content/original/ported/sp/ranshang2.skill \
  content/original/ported/sp/jingong2.skill \
  content/original/ported/sp/shengxi.skill \
  content/original/ported/sp/olzhennan2.skill \
  content/original/ported/sp/jiaozi.skill \
  content/original/ported/sp/olbiri.skill \
  content/original/ported/sp/shoucheng.skill \
  content/original/ported/sp/shenxian.skill \
  content/original/ported/sp/rechouhai.skill \
  content/original/ported/sp/shulv.skill \
  content/original/ported/sp/xinfu_zhanji.skill \
  content/original/ported/sp/xinfu_songsang.skill \
  content/original/ported/sp/xinfu_guolun.skill \
  content/original/ported/sp/xinfu_tushe.skill \
  content/original/ported/sp/guofengyupao.skill \
  content/original/ported/sp/gzjili.skill \
  content/original/ported/sp/fuqi.skill \
  content/original/ported/sp/zhenlue.skill \
  content/original/ported/sp/qimenbagua.skill \
  content/original/ported/sp/zniaoxiang.skill \
  content/original/ported/sp/spweiwu.skill \
  content/original/ported/sp/yanzheng.skill \
  content/original/ported/sp/kuiwei.skill \
  content/original/ported/sp/olyinji.skill \
  content/original/ported/sp/shushen.skill \
  content/original/ported/sp/shenzhi.skill \
  content/original/ported/sp/zhenwei_three.skill \
  content/original/ported/sp/qiongshou.skill \
  content/original/ported/sp/sijian.skill \
  content/original/ported/sp/wangong.skill \
  content/original/ported/sp/juyi.skill \
  content/original/ported/sp/danji.skill \
  content/original/ported/sp/nuzhan.skill \
  content/original/ported/sp/xiongyi.skill \
  content/original/ported/sp/hanyong.skill \
  content/original/ported/sp/moukui.skill \
  content/original/ported/sp/cunsi.skill \
  content/original/ported/sp/yongjue.skill \
  content/original/ported/sp/naman.skill \
  content/original/ported/sp/fenxun.skill \
  content/original/ported/sp/duanbing.skill \
  content/original/ported/sp/oldhuxiao.skill \
  content/original/ported/sp/xueji_old.skill \
  content/original/ported/sp/oldwuji.skill \
  content/original/ported/sp/qiangwu.skill \
  content/original/ported/sp/fenming.skill \
  content/original/ported/sp/recanshi.skill \
  content/original/ported/sp/juesi.skill \
  content/original/ported/sp/lizhan.skill \
  content/original/ported/sp/weikui.skill \
  content/original/ported/sp/kaikang.skill \
  content/original/ported/sp/xisheng.skill \
  content/original/ported/sp/xiaoguo.skill \
  content/original/ported/sp/hengzheng.skill \
  content/original/ported/sp/ziyuan.skill \
  content/original/ported/sp/dingpan.skill \
  content/original/ported/sp/shuimeng.skill \
  --characters content/original/ported/sp/characters.json \
  --metadata content/original/ported/sp/metadata.json \
  --out content/original/ported/sp/sp.npack.json \
  --package-id sp
```

当前 `character_index` 只包含技能闭包完整的沙摩柯、麴义、黄祖、诸葛诞、兀突骨、标甘夫人、
糜夫人、张星彩、孙皓、SP庞德、SP曹仁、曹昂、牛金与 SP庞统；
标甘夫人通过精确依赖复用 STANDARD 的「淑慎」，马腾由 OFFLINE 包复用本包的「雄异」。
联动包另通过精确依赖复用本包「谋溃」发布 SP穆顺。
限定包另通过精确依赖复用本包「奋命」发布陈董。
邹氏按上游定义为 3 体力、群势力且无技能，直接登记零技能角色，不制造空技能占位。
SP赵云通过精确 REFRESH 依赖复用已重写的「龙胆」，与本包「冲阵」形成完整武将闭包。
SP庞统的「过论」「送丧」均已重写并登记完整角色闭包。
乐进的「骁果」按旧 JS 独立重写：弃任意手牌后，目标可自弃一张装备牌令乐进摸一张，
否则受到乐进造成的一点普通伤害；两个弃牌步骤均等待移动触发完整结算。
SP董卓的「横征」按旧 JS 重写为摸牌替换：体力为 1 或空手时可发动，将本次正常摸牌数
固定为 0，再从下一座开始顺时针逐名获得一张当前可获得的区域牌；不会翻面。
糜竺的「资援」已迁移为声明式主动技，并与既有「聚谷」组成完整武将闭包；旧 JS 仅作
行为基准，不进入运行时加载或执行链路。
步骘的「定叛」按身份局当前存活反贼数动态限制每回合发动次数；目标摸牌后选择由步骘
弃置其一张当前可弃装备，或一次收回全部装备并在移动触发结算后受到一点普通伤害。
