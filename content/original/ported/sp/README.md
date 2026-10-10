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
| `spniluan`（逆乱） | `re_hansui` | 黑色手牌、装备牌或扩展牌当【杀】；本次未造成伤害才退还使用次数，Golden 已覆盖 |
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
| `rehengjiang`（横江） | `zangba` | 每点受伤可令当前回合角色本回合手牌上限 -1；其弃牌阶段有弃牌则回合末摸 1，否则摸本回合发动次数；HUICUI 臧霸闭包已发布 |
| `olxingshen`（邢神） | `ol_zhangchangpu` | 受到伤害后摸 1（上游 50% 概率摸 2 的随机分支确定性固定为 1），并按受伤点数增加「严教」展示计数标记、封顶 6 |
| `olbiluan`（避乱·OL） | `shixie` | 自己结束阶段可弃置 1 张手牌/装备，获得「避乱」状态并叠加当前存活势力数标记；「存在距你 ≤1 的其他角色」距离子门槛未建模 |
| `rebiluan`（避乱·新） | `dc_shixie` | 自己结束阶段可弃置 1 张手牌/装备，获得「避乱」状态并叠加 min(4, 存活角色数) 标记；「存在距你 ≤1 的其他角色」距离子门槛未建模 |
| `newzhennan`（新镇南） | `baosanniang` | 准备阶段可弃置两张牌并视为使用一张【南蛮入侵】；新增 `choose_use_virtual_nanman` 原语（ABI 604，走完整用牌管线）；限定技仅一次与南蛮受伤目标弃牌子技未建模 |
| `qiluan`（起乱） | `hetaihou` | 任意角色结束阶段按本回合各角色击杀数摸牌（自己击杀×3 他人×1，新增 `resolve_qiluan` 原语 ABI 605）；其他角色死亡且非你击杀时摸 1（新增 `owner_alive_other_source_death` 过滤器）；「本回合有角色使用过杀」过滤以击杀数为 0 时静默 no-op 近似 |
| `cihuai`（摧怀） | 独立技能 | 出牌阶段开始时若无手牌【杀】，可视为使用一张【杀】；新增 `owner_is_target_play_phase_hand_no_sha` 过滤器；展示手牌（信息暴露）省略 |
| `oldcihuai` / `oldcihuai2` / `oldcihuai3`（咄咄） | `old_machao` | 出牌阶段开始时若无手牌【杀】可视为使用【杀】；上游的临时技能授予/移除（oldcihuai2/3）被折叠进父技直接近似，两子技保持文档化 noop |
| `olfeiyang`（飞杨·OL） | 独立技能 | 准备阶段可弃两张牌并弃置判定区全部牌；新增 `discard_all_own_judge` 原语（ABI 606）；「可弃置手牌 ≥ 2」门槛与限定技仅一次以静默 no-op / 可选触发近似 |
| `ollixia`（礼下·OL） | `dc_shixie` | 其他角色结束阶段若不在你的攻击范围内，可选令自己摸 1 或对方摸 2，并获得 1 层「避乱」负距离标记；新增 `owner_alive_other_phase_end_player_outside_owner_range` 过滤器；上游目标选择以数值分支近似 |
| `lixia`（礼下） | `shixie` | 其他角色结束阶段若其攻击范围不包含你，可选令你或该角色各摸 1 张，并获 1 层「避乱」负距离标记；与 ollixia 共用过滤器与 rebiluan2 标记 |
| `kunfen`（困奋） | `sp_jiangwei` | 结束阶段开始时失去 1 点体力然后摸两张牌；storage.kunfen 置位后的可选模式未建模，恒按强制结算 |
| `zhuangshu_trick`（妆梳·锦囊） | 装备技能 | 判定阶段开始时可选跳过本回合判定阶段或弃牌阶段；上游单次三选以「可选确认 + 二选一」两步近似 |
| `olpianan`（骗南·OL） | `sp_sunce` | 游戏开始时与弃牌阶段结束后，弃置所有非【闪】手牌并补摸至体力值；「保留/优先【闪】」筛选与首轮每阶段结算未建模 |
| `ollinjie`（临界·OL） | `ol_diaochan` | 受伤后伤害来源须弃一张手牌，弃的牌为【杀】则你摸 1；新增 `resolve_ollinjie` 组合式原语（ABI 608/tag 664/Cont 265：来源弃牌与条件摸牌同一调度队列串行结算） |
| `zhuangshu_basic`（妆梳·基本） | 装备技能 | 受到伤害前可弃一张手牌/装备并防止此伤害；上游弃伤害点数张、排除妆梳·基本装备、「可弃 ≥ 点数」门槛均以弃 1 + 原语内静默跳过近似 |
| `olxiaoxi`（小西·OL） | `hansui` | 每轮开始时可选视为使用一张无距离限制的【杀】；「全场无人有过行动」过滤未建模（每轮都会触发） |
| `olniluan`（逆乱·OL） | `hansui` | 其他角色结束阶段且其体力大于你时，可选视为使用一张【杀】；「其本回合使用过杀」「必须对触发者使用」「黑色牌材质」条件未建模 |
| `gzshushen`（淑慎·国战） | `gz_ganfuren` | 每回复 1 点体力可选令一名其他角色摸 1（按回复点数重复）；「势力相同」过滤未建模，任意其他角色可选 |
| `dc_olshichou`（誓仇·DC） | `old_machao` | 你使用【杀】指定目标后，若你已受伤，可令一名其他角色也成为此【杀】的目标（上游至多受伤点数个，单次只加一个）；锁定技以可选触发近似 |
| `neifa_basic`（内伐·基本） | `yuantanyuanshang` 子技能 | 禁用锦囊/装备；使用【杀】后可额外指定一个目标；杀次数上限 + 自身标记数；本回合限制以技能存活期近似 |
| `neifa_nobasic`（内伐·非基本） | `yuantanyuanshang` 子技能 | 禁用基本牌；使用普通锦囊后可增加或减少一个目标 |
| `huamu`（灵杉） | `caoxiancaohua` | 使用牌结算完成后，若该牌进入弃牌堆，可将其置于武将牌上；新增 `store_event_card_in_skill_pile` 原语（ABI 607）；「与上一张全场用牌颜色交替」过滤与 gaintag 未建模 |
| `pxqiaoxian`（谯县） | 州郡技能 | 受伤后可选摸 1（damage + owner_is_target optional）；批量脚本误标为 audio-only，上游 skill.js:3195 实为真实规则体，已纠正 |
| `pxdunhuang`（敦煌） | 州郡技能 | 锁定，自己的【杀】被抵消后摸 1（sha_dodged + owner_source_sha_dodged）；批量脚本误标为 audio-only（上游 skill.js:1629 为真实规则体），已纠正 |
| `pxchenliu`（陈留） | 州郡技能 | 锁定，跳过弃牌阶段（modifier skip_discard_phase override 1，语义等价上游 phaseDiscardBefore cancel）；批量脚本误标为 audio-only（上游 skill.js:2581 为真实规则体），已纠正 |
| `pxyanzhou`（兖州） | 州郡技能 | 回复体力后可选摸 1（hp_recovered + owner_is_target optional）；「每回合限一次」以可选触发近似；批量脚本误标为 audio-only（上游 skill.js:2488 为真实规则体），已纠正 |
| `pxdianchi`（滇池） | 州郡技能 | 造成火焰伤害后可令受伤角色失去 1 点体力（damage + owner_source_fire optional）；「每回合限一次」以可选触发近似；批量脚本误标为 audio-only（上游 skill.js:1852 为真实规则体），已纠正 |
| `pxfuling`（涪陵） | 州郡技能 | 锁定，装备牌不能被其他角色弃置（modifier card_discard_protected(equip) override 1）；批量脚本误标为 audio-only（上游 skill.js:1817 为真实规则体），已纠正 |
| `pxwudu`（武都） | 州郡技能 | 锁定，使用牌无次数限制（modifier card_use_unlimited override 1，语义等价上游 cardUsable→Infinity）；批量脚本误标为 audio-only（上游 skill.js:2040 为真实规则体），已纠正 |
| `pxanding`（安定） | 州郡技能 | 锁定，因传导（铁索连环）造成的属性伤害防止之（damage_before + owner_is_target_chain_damage_linked → prevent_current_damage）；批量脚本误标为 audio-only（上游 skill.js:2909 为真实规则体），已纠正 |
| `pxzhuti`（朱提） | 州郡技能 | 属性伤害改为回复等量体力（damage_before 火焰/雷电双触发 → prevent + recover_hp(event_amount)）；「每回合限一次」以可选触发近似；批量脚本误标为 audio-only（上游 skill.js:2860 为真实规则体），已纠正 |
| `pxdaifang`（带方） | 州郡技能 | 属性【杀】不可被响应（modifier card_use_unrespondable(sha_elemental) override 1，语义等价上游 directHit 全场）；批量脚本误标为 audio-only（上游 skill.js:3054 为真实规则体），已纠正 |
| `pxpeixian`（沛县） | 州郡技能 | 普通锦囊不可被响应（modifier card_use_unrespondable(trick) override 1）；批量脚本误标为 audio-only（上游 skill.js:3162 为真实规则体），已纠正 |
| `pxningzhou`（宁州） | 州郡技能 | 【杀】不可被抵消（card_use_unrespondable(sha)）且杀伤害视为体力流失（damage_before owner_source_sha → prevent + lose_hp(event_player, event_amount)）；批量脚本误标为 audio-only（上游 skill.js:1829 为真实规则体），已纠正 |
| `xiaoxi`（小西） | `machao`/`hansui`/`pangde` | 游戏开始时可选视为使用一张【杀】（game_started owner_alive optional → choose_use_virtual_sha）；「每轮首名角色回合开始」以开局 GameStarted 事件近似；批量脚本误标为 audio-only（上游 skill.js:31011 为真实规则体），已纠正 |
| `zfengshi`（风石） | 独立技能 | 使用【杀】指定目标后可弃置其一张装备（card_targeted + 新增 owner_source_sha_target_alive_has_discardable_equip 过滤器 → choose_discard_target_card(self, event_player, equip)）；上游强制弃置以可选触发近似；批量脚本误标为 audio-only（上游 skill.js:34791 为真实规则体），已纠正 |
| `pxliangzhoux`（梁州） | 州郡技能 | 成为他人【杀】目标后弃置其一张手牌/装备（card_targeted owner_targeted_by_other_sha → choose_discard_target_card(self, event_source, hand_equip)）；批量脚本误标为 audio-only（上游 skill.js:1711 为真实规则体），已纠正 |
| `pxchengdu`（成都） | 州郡技能 | 对自己使用牌后可摸 1（card_used + 新增 owner_source_card_used_targets_self 过滤器）；「每回合限一次」以可选触发近似；批量脚本误标为 audio-only（上游 skill.js:2762 为真实规则体），已纠正 |
| `olmiaoxin`（妙心·OL） | `huanshujun` | 受到伤害后摸 2（damage owner_is_target forced → draw_cards(2)）；「温宜」次数联动分支未建模（olwenyi 标记不会出现），固定摸 2；批量脚本误标为 audio-only（上游 skill.js:4903 为真实规则体），已纠正 |
| `oltingji`（廷机·OL） | 独立技能 | 回合内指定未受伤其他角色为目标后令该牌不可被响应（card_targeted + 新增 owner_source_current_phase_card_targeted_other_healthy 过滤器 → make_current_card_unrespondable）；批量脚本误标为「需修改技原语」（上游 skill.js:42946 为触发技），已纠正 |
| `wuji`（武继） | `guanyinping` | 结束阶段开始时若本回合累计造成至少 3 点伤害则觉醒：失去「虎啸」、加 1 体力上限并回复 1（phase_end owner_is_target_turn_damage_at_least_three，与 oldwuji 同款）；场上【青龙偃月刀】获得未建模 |
| `mingshi`（名士） | 独立技能 | 受到伤害前若来源体力大于你可弃一张牌令伤害 -1（damage_before owner_is_target optional → if hp(event_source) > hp(self) → discard_hand_equip_count(1) + modify_current_damage(-1)）；「黑色牌」花色约束未建模；批量脚本误标为 audio-only（上游 skill.js:41702 为真实规则体），已纠正 |
| `pxliangzhou`（凉州） | 州郡技能 | 黑色手牌/装备当【杀】使用（view_as sha play_phase black from hand_equip）；「可打出响应」窗口未建模；批量脚本误标为 audio-only（上游 skill.js:1572 为真实规则体），已纠正 |
| `pxbeihai`（北海） | 州郡技能 | 获得后手牌摸至体力上限（skill_added + 新增 owner_skill_added_self_<def> 过滤器 → draw_cards(max_hp - hand_count)）；批量脚本误标为 audio-only（上游 skill.js:2203 为真实规则体），已纠正 |
| `pxyouzhou`（幽州） | 州郡技能 | 获得后体力上限 +1（skill_added + owner_skill_added_self → change_max_hp(1)）；批量脚本误标为 audio-only（上游 skill.js:2974 为真实规则体），已纠正 |
| `pxfengyi`（冯翊） | 州郡技能 | 获得后可回复 1（skill_added optional → if hp < max_hp → recover_hp(1)）；批量脚本误标为 audio-only（上游 skill.js:2922 为真实规则体），已纠正 |
| `pxhongnong`（弘农） | 州郡技能 | 获得后可回复 1（skill_added optional → recover_hp(1)）；「重铸装备」环节未建模（无选装备重铸原语）；批量脚本误标为 audio-only（上游 skill.js:2303 为真实规则体），已纠正 |
| `pxjizhou`（冀州） | 州郡技能 | 获得后从牌堆获得【万箭齐发】（skill_added → gain_card_by_kind(wanjian)）；批量脚本误标为 audio-only（上游 skill.js:1290 为真实规则体），已纠正 |
| `pxwuxiang`（武乡） | 州郡技能 | 获得后从牌堆获得【火攻】（skill_added → gain_card_by_kind(huogong)）；批量脚本误标为 audio-only（上游 skill.js:1274 为真实规则体），已纠正 |
| `pxquting`（句町） | 州郡技能 | 获得后可失去 1 体力并摸 3（skill_added optional → lose_hp(1) + draw_cards(3)）；批量脚本误标为 audio-only（上游 skill.js:1913 为真实规则体），已纠正 |
| `pxchangshan`（常山） | 州郡技能 | 获得后从牌堆获得两张【闪】（skill_added → gain_card_by_kind(shan) ×2）；批量脚本误标为 audio-only（上游 skill.js:1338 为真实规则体），已纠正 |
| `pxbeiping`（北平） | 州郡技能 | 获得后从牌堆获得两张【杀】（skill_added → gain_card_by_kind(sha) ×2）；批量脚本误标为 audio-only（上游 skill.js:3005 为真实规则体），已纠正 |
| `pxlean`（乐安） | 州郡技能 | 获得后从牌堆获得一张属性【杀】和一张【铁索连环】（skill_added → gain_card_by_kind(sha_fire) + gain_card_by_kind(tiesuo)）；批量脚本误标为 audio-only（上游 skill.js:2098 为真实规则体），已纠正 |
| `pxyanmen`（雁门） | 州郡技能 | 获得后从牌堆获得一张武器和一张进攻坐骑（skill_added → gain_card_by_kind(weapon) + gain_card_by_kind(horse_off)）；批量脚本误标为 audio-only（上游 skill.js:1117 为真实规则体），已纠正 |
| `pxlongxi`（陇西） | 州郡技能 | 获得后从牌堆获得一张防御坐骑（skill_added → gain_card_by_kind(horse_def)）；批量脚本误标为 audio-only（上游 skill.js:1987 为真实规则体），已纠正 |
| `pxguangling`（广陵） | 州郡技能 | 两张牌当【五谷丰登】使用（view_as wugu play_phase any from hand_equip up_to 2）；批量脚本误标为 audio-only（上游 skill.js:2475 为真实规则体），已纠正 |
| `pxnanfu`（南涪） | 州郡技能 | 获得后可失去 1 体力并视为使用【杀】（skill_added optional → lose_hp(1) + choose_use_virtual_sha）；批量脚本误标为 audio-only（上游 skill.js:1929 为真实规则体），已纠正 |
| `pxxiangyang`（襄阳） | 州郡技能 | 获得后可移动场上一张牌（skill_added optional → choose_move_field_card_to(self, self)，从其他角色场上传牌至自己）；批量脚本误标为 audio-only（上游 skill.js:1529 为真实规则体），已纠正 |
| `pxyangzhou`（扬州） | 州郡技能 | 获得后可令一名其他角色回复 1（skill_added optional → choose_other_recover(self, 1)）；「目标必须受伤」过滤未建模；批量脚本误标为 audio-only（上游 skill.js:2600 为真实规则体），已纠正 |
| `pxfanyang`（范阳） | 州郡技能 | 获得后可对攻击范围内一名角色造成 1 伤害（skill_added optional → choose_other_damage(self, 1)）；「攻击范围内」过滤未建模；批量脚本误标为 audio-only（上游 skill.js:3029 为真实规则体），已纠正 |
| `pxrunan`（汝南） | 州郡技能 | 获得后可令一名角色摸手牌上限张（skill_added optional → choose_other_draw_or_recover(self, hand_limit(self), 0, 1)）；批量脚本误标为 audio-only（上游 skill.js:3204 为真实规则体），已纠正 |
| `pxhefei`（合肥） | 州郡技能 | 获得后可弃至多两张牌并摸等量（skill_added optional → discard_hand_equip_count(1) + draw_cards(1)，「弃 2 摸 2」以弃 1 摸 1 近似）；批量脚本误标为 audio-only（上游 skill.js:2647 为真实规则体），已纠正 |
| `pxshangyong`（上庸） | 州郡技能 | 获得后可令一名其他角色摸 1 且自己摸 1（skill_added optional → choose_other_draw_or_recover(self,1,0,1) + draw_cards(self,1)，「各摸一张」以两段近似）；批量脚本误标为 audio-only（上游 skill.js:1503 为真实规则体），已纠正 |
| `pxtianshui`（天水） | 州郡技能 | 获得后可弃置一名角色一张手牌/装备并摸 2（skill_added optional → choose_other_discard_target_card(self, hand_equip) + draw_cards(self, 2)）；「仅手牌」「弃【杀】不摸」条件未建模；批量脚本误标为 audio-only（上游 skill.js:2003 为真实规则体），已纠正 |
| `pxbohai`（渤海） | 州郡技能 | 获得后可弃一张牌并摸 2（skill_added optional → discard_hand_equip_count(1) + draw_cards(2)）；「必须弃武器」约束未建模；批量脚本误标为 audio-only（上游 skill.js:1306 为真实规则体），已纠正 |
| `pxpengcheng`（彭城） | 州郡技能 | 获得后可令一名其他角色交给你一张牌并回复 1（skill_added optional → choose_other_give_one_hand_equip(self) + choose_other_recover(self, 1)）；批量脚本误标为 audio-only（上游 skill.js:2397 为真实规则体），已纠正 |
| `pxjiangxia`（江夏） | 州郡技能 | 获得后可令一名其他角色摸 2 并交给你一张牌（skill_added optional → choose_other_draw_or_recover(self,2,0,1) + choose_other_give_one_hand_equip(self)）；批量脚本误标为 audio-only（上游 skill.js:1541 为真实规则体），已纠正 |
| `pxjuchao`（居巢） | 州郡技能 | 获得后可弃置你与一名角色各一张牌（skill_added optional → choose_other_discard_target_card(self, hand_equip) + discard_hand_equip_count(self, 1)）；批量脚本误标为 audio-only（上游 skill.js:2672 为真实规则体），已纠正 |
| `pxnanyang`（南阳） | 州郡技能 | 获得后可弃一张牌并与一名角色各回复 1（skill_added optional → discard_hand_equip_count(1) + recover_hp(1) + choose_other_recover(1)）；「必须重铸锦囊」以弃任意一张近似；批量脚本误标为 audio-only（上游 skill.js:1467 为真实规则体），已纠正 |
| `pxwuwei`（武威） | 州郡技能 | 每回合首次使用伤害牌时可加一个目标（card_used + owner_source_damage_card_once_per_turn optional → choose_add_or_remove_current_card_target）；批量脚本误标为 audio-only（上游 skill.js:1673 为真实规则体），已纠正 |
| `pxwenxian`（温县） | 州郡技能 | 获得后加 1 体力上限并回复 1（skill_added → change_max_hp(1) + recover_hp(1)）；「获得【虚妄之冕】」卡牌创造未建模；批量脚本误标为 audio-only（上游 skill.js:2329 为真实规则体），已纠正 |
| `pxjingzhou`（荆州） | 州郡技能 | 获得后可交给一名角色一张手牌/装备（skill_added optional → choose_other_give_one_hand_equip(self)）；「任意张手牌」以一张近似；批量脚本误标为 audio-only（上游 skill.js:1433 为真实规则体），已纠正 |
| `pxzangke`（牂牁） | 州郡技能 | 受伤后可弃置你与伤害来源各一张牌（damage owner_is_target optional → discard_hand_equip_count(self, 1) + choose_other_discard_target_card(self, hand_equip)）；批量脚本误标为 audio-only（上游 skill.js:2816 为真实规则体），已纠正 |
| `pxwenxi`（闻喜） | 州郡技能 | 每回合首次使用某花色的牌后摸 1（card_used + 新增 owner_source_first_card_suit_this_turn 过滤器，5 处注册 + TriggerHistorySnapshot.owner_used_card_suit_history 字段 → draw_cards(1)）；批量脚本误标为 audio-only（上游 skill.js:2246 为真实规则体），已纠正 |
| `pxpingyuan`（平原） | 州郡技能 | 失去最后手牌后摸 1（cards_moved + 新增 owner_lost_hand_now_empty 过滤器，5 处注册 → draw_cards(1)）；「每回合限一次」以强制触发近似；批量脚本误标为 audio-only（上游 skill.js:1389 为真实规则体），已纠正 |
| `ol_shichou`（誓仇·OL） | `sp_machao` | 使用【杀】指定目标后可加一个目标（card_used owner_source_sha optional → choose_add_or_remove_current_card_target）；「至多已损失体力+1 个」以单次一个近似；批量脚本误标为 audio-only（上游 skill.js:32995 为真实规则体），已纠正 |
| `yongsi2`（雍嗣·弃） | `yuanshu` | 弃牌阶段开始时弃 1 张手牌/装备（phase_discard owner_is_target → discard_hand_equip_count(1)）；「弃势力数张」以弃 1 近似；批量脚本误标为 audio-only（上游 skill.js:43662 为真实规则体），已纠正 |
| `pxhandan`（邯郸） | 州郡技能 | 使用普通锦囊后可令一名角色获得此牌（card_used owner_source_ordinary_trick optional → choose_other_gain_event_cards(self)）；「非伤害锦囊」「实体牌进弃牌堆」过滤未建模；批量脚本误标为 audio-only（上游 skill.js:2260 为真实规则体），已纠正 |
| `pxjulu`（巨鹿） | 州郡技能 | 失去黑桃 2-9 牌后摸 1（cards_moved + 新增 owner_lost_spade_two_to_nine 过滤器，5 处注册 → draw_cards(1)）；「每回合限一次」以强制触发近似；批量脚本误标为 audio-only（上游 skill.js:1362 为真实规则体），已纠正 |
| `pxjiuyuan`（九原） | 州郡技能 | 使用【杀】可加一个目标（card_used owner_source_sha optional → choose_add_or_remove_current_card_target）；「任意个目标」以单次一个近似；批量脚本误标为 audio-only（上游 skill.js:1141 为真实规则体），已纠正 |
| `pxyuzhou`（豫州） | 州郡技能 | 获得后可横置自己并令一名其他角色回复 1（skill_added optional → add_mark(self, 2, 1) 铁索标记 + choose_other_recover(self, 1)）；「目标体力小于自己」过滤未建模；批量脚本误标为 audio-only（上游 skill.js:3087 为真实规则体），已纠正 |
| `pxyongzhou`（雍州） | 州郡技能 | 获得后可摸 2 并弃 1 张（skill_added optional → draw_cards(2) + discard_hand_equip_count(1)）；「弃至手牌上限」以弃 1 近似；批量脚本误标为 audio-only（上游 skill.js:2875 为真实规则体），已纠正 |

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
  content/original/ported/sp/spniluan.skill \
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
  content/original/ported/sp/rehengjiang.skill \
  content/original/ported/sp/rebaobian.skill \
  content/original/ported/sp/kunfen.skill \
  content/original/ported/sp/lixia.skill \
  content/original/ported/sp/zhuangshu_trick.skill \
  content/original/ported/sp/olpianan.skill \
  content/original/ported/sp/ollinjie.skill \
  content/original/ported/sp/zhuangshu_basic.skill \
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
新杀韩遂由 SP2 包登记，并通过依赖复用本包「逆乱」「违忤」形成完整武将闭包。
乐进的「骁果」按旧 JS 独立重写：弃任意手牌后，目标可自弃一张装备牌令乐进摸一张，
否则受到乐进造成的一点普通伤害；两个弃牌步骤均等待移动触发完整结算。
SP董卓的「横征」按旧 JS 重写为摸牌替换：体力为 1 或空手时可发动，将本次正常摸牌数
固定为 0，再从下一座开始顺时针逐名获得一张当前可获得的区域牌；不会翻面。
糜竺的「资援」已迁移为声明式主动技，并与既有「聚谷」组成完整武将闭包；旧 JS 仅作
行为基准，不进入运行时加载或执行链路。
步骘的「定叛」按身份局当前存活反贼数动态限制每回合发动次数；目标摸牌后选择由步骘
弃置其一张当前可弃装备，或一次收回全部装备并在移动触发结算后受到一点普通伤害。
「豹变」按旧 JS 固定顺序授予 SHENHUA「挑衅」、REFRESH「界咆哮」、SHENHUA「界神速」；
每次受伤只获得第一项尚未拥有的技能，不打开选择，供 XIANDING 新杀夏侯霸组成完整闭包。
