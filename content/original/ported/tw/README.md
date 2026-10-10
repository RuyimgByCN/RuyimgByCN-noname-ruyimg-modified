# TW 确定性移植

源角色闭包修复（ADR 0369）：周处「除害」的获牌后弃牌触发折叠回 `twchuhai`，桥蕤
「穷技」的威牌使用/打出后摸牌触发折叠回 `twqiongji`。内部 helper 不再作为角色技能
发布，角色技能列表与旧 JavaScript 定义保持一致。

- `twjuyan`：对一名角色造成火焰伤害后摸一张牌，并令其减一点体力上限。
- `manji`：其他角色失去体力后，按双方结算后体力关系回复或摸牌。
- `gx_lingbaoxianhu`：大额伤害或死亡后增加体力上限并回复；当前仅发布技能定义，卡牌挂载随历史卡牌包迁移。
- `twyuanbu`：手牌上限和双向距离增加其他存活角色数，至多为四，查询时动态重算。
- `twshenxing`：装备区没有坐骑牌时，至其他角色的距离 -1 且手牌上限 +1。
- `twzhenglve3`：旧定义仅为 `charlotte` 临时状态位；保留技能定义但不生成规则索引。
- `old_twrouke`（柔克）：摸牌阶段外一次获得超过一张牌后，锁定摸一张牌；覆盖直接摸牌与区域移动获得牌。
- `jintao`（进讨）：使用【杀】无距离限制且次数上限 +1；出牌阶段第一张【杀】伤害 +1，第二张【杀】不可被响应。
- `twdianyi`（典仪）：自己的回合结束时，造成过伤害则弃置全部手牌，否则由权威选牌屏障将手牌精确调整至四张。
- `twyingji/twshanghe`（应机/商和）：回合外空手牌时摸一张并声明任意合法基本牌或锦囊牌；濒死时其他角色按座次各交一张手牌或装备牌，未收到【酒】则回复至一点体力。
- `twkaiji/twshepan`（开济/慑叛）：按整局曾濒死的不同角色数选择角色摸牌并检查非基本牌；成为其他角色牌的目标后摸牌或将其区域牌置顶，手牌相等时返还次数并可令当前牌无效。
- `twfuzuan/twchongqi/twfeifu`（复纂/宠齐/非服）：主动或伤害后切换任意角色的转换技状态；开局令全场获得非服，并可减一点体力上限令其他角色获得复纂；非服按转换状态在成为或发起单目标【杀】时令目标向使用者交牌，装备牌可由使用者立即装备。
- `twsaotao`（扫讨）：使用【杀】和普通锦囊牌不可被响应。
- `liuli_daxiaoqiao`（流离）：旧 JS 仅声明继承 STANDARD「流离」；本包以自己的稳定技能 ID
  发布相同的弃牌并转移【杀】目标规则，不执行继承脚本。
- `twdaoji`（盗戟）：弃置一张非基本牌，获得攻击范围内目标的一张手牌或装备牌；基本牌额外摸一张，装备牌可用则强制使用并对原持有者造成伤害。
- `twdanji`（单骑）：准备阶段手牌数大于体力值且场上没有刘备主公时觉醒，减一点体力上限并获得【马术】【怒斩】；此后每回合首次使用由实体牌转化的【杀】后摸一张牌。
- `twbaimei`（白眉）：没有手牌时，锁定防止属性伤害或普通锦囊牌造成的伤害。
- `twrangyi`（让驿）：将所有手牌交给一名其他角色；其可使用其中一张并归还其余牌，否则受到一点伤害。
- `zhengfu`（征服）：使用【杀】指定目标后声明一种牌的类别；目标交给你一张同类别牌，否则此【杀】不可被其响应。
- `twfenwu`（奋武）：结束阶段可失去一点体力并使用无距离限制的虚拟【杀】；本回合使用过至少两种基本牌变体时伤害加一。
- `twqingkou`（清口）：准备阶段可视为对一名其他角色使用虚拟【决斗】；该决斗的伤害来源摸一张牌，若为自己则跳过本回合判定和弃牌阶段。
- `twchenxun`（陈勋）：每轮可选择一名本轮未以此法选择过的角色并使用虚拟【决斗】；目标实际受到伤害则摸一张并可继续选择，否则失去一点体力。
- `twzenhui/xinjiaojin`（谮毁/骄矜，TW 孙鲁班）：复用通用用牌者转移/加目标管线；受到男性角色伤害前可弃一张有效装备牌防止整段伤害。
- `twfenxian`（焚险，幻黄盖）：目标选择以双方场上的实体牌当【决斗】使用，或令拥有者视为对其使用【火攻】；与既有 `twjuyan` 闭环武将。
- `twfupan`（复叛）：造成或受到伤害后按伤害点数摸牌并交出一张牌；对每名角色首次交牌摸二，再次交牌可造成伤害并永久停止向其交牌。
- `twjiaohua`（教化）：自己或体力值最低的其他角色摸牌后，可令其从牌堆或弃牌堆获得一张本次未摸到且本回合未以此法获得过类别的牌。
- `twkujian`（苦谏）：将至多三张手牌交给其他角色并追踪实体牌；其使用或打出时双方摸牌，因其他原因从手牌失去时双方各弃一张牌。
- `twruilian`（睿敛）：每轮选择一名角色，记录其回合内弃置牌的数量和类别；其回合结束时可选择一种记录类别，令其与你依次获得弃牌堆中的一张同类别牌。
- `twfujian`（负剑）：游戏开始或准备阶段无武器时，从牌堆随机装备一张武器牌；回合外失去武器后失去一点体力。
- `twjianwei`（剑威）：准备阶段可双向发起拼点；装备武器时以攻击范围增加自己的拼点点数，胜者按规则获得败者各区域牌或武器。
- `twtanfeng`（探锋）：准备阶段弃置一名其他角色区域内一张牌；其可将一张手牌或装备牌当有距离限制的【杀】使用，否则受到一点火焰伤害，存活后选择令技能拥有者跳过一个后续阶段。
- `twzhibian`（直辩）：出牌阶段开始可与其他角色拼点；未赢失去一点体力，赢后可移动其对应区域牌、回复一点体力，或弃置非基本牌执行两个可用选项。
- `twyuyan`（御严）：体力值更高的角色以实体【杀】指定你后，其须交给你一张点数更高的牌；无点数时改为非基本牌，否则取消你这一目标。
- `twguoyi`（果毅）：以【杀】或普通锦囊指定满足条件的其他角色后，令其选择本回合禁用手牌或弃牌；双条件同时满足或本回合两项均已选择时，在原牌完整结算后对原目标重放同名同属性牌。
- `twchuhai`（除害）：本回合令至少两名不同角色进入濒死后，于回合结束废除判定区并令其他角色依次交牌；每次获得他人牌后须弃置本批新获得牌中的一张。
- `twyizhu`（遗珠）：游戏开始及结束阶段摸至多四张牌并将两张手牌/装备牌随机插入牌堆顶部附近；其被其他角色用于单目标牌时可增加或替换目标，随后摸一张并解除实体牌追踪，进入弃牌堆也会解除追踪。
- `twluanchou/twgonghuan`（乱筹/共患）：出牌阶段选择两名角色重置为新的“共患”持有者；持有者每回合一次可代替体力值不高于自己的另一名持有者承受完整同源、同牌、同属性伤害，转移伤害不会再次连锁转移。
- `twxiawei/twqiongji`（狭威/穷技）：游戏开始将至多两张基本牌置为可如手牌使用或打出的“威”，回合开始清空；准备阶段可选择一至四并将牌堆顶对应数量加一置为“威”，回合结束弃置等量牌或减一点体力上限。每回合首次使用或打出“威”后摸一张牌，没有“威”时受到的伤害加一。
- `twwanwei/twyuejian`（挽危/约俭）：每回合一次响应最低体力角色受到的伤害，按目标与体力上限条件防止伤害并失去体力，或在当前回合结束阶段获得牌堆顶并使用可用的牌堆底牌；出牌阶段将超出手牌上限数量（至少一张）的牌自由分置牌堆顶或底，并按一、二、三张依次增加手牌上限、回复及增加体力上限。
- `twshengxi/twkuanji`（生息/宽济）：准备阶段从共享牌堆获得【调剂盐梅】，不足四种花色时按权威随机流补建一张；结束阶段若本回合用过牌且未造成伤害，可选择一张标准智囊并摸一张牌；自己的牌非因使用/打出进入弃牌堆后，每回合一次可将其中一张交给其他角色。
- `twmuyue/twchayi`（睦约/察异）：出牌阶段声明一种基本牌或锦囊牌、弃置一张牌并令任意角色从共享牌堆获得同名牌，弃牌同名时记录下次免弃；结束阶段令其他角色选择展示手牌或在下次用牌时弃牌，并记录其手牌数，到其下回合结束若手牌数变化则执行另一分支。
- `twqianju/twqingxi`（前驱乐/倾袭）：计算至其他角色距离时减去装备数；每回合一次对距离一以内角色造成伤害后，从共享牌堆装备一张可置入空槽的装备牌。每回合第一张【杀】的每名目标选择：令使用者摸至少一张、按装备数计的牌且该目标不可响应，或弃置自己的全部装备、再弃置等量使用者装备并令此【杀】对其伤害加一。
- `twzaoli`（躁厉）：出牌阶段只能使用或打出本回合进入手牌的实体牌；阶段开始弃置全部装备牌和至少一张非装备手牌并等量摸牌，再从共享牌堆按本次弃置装备的不同装备槽各补入一张，补入超过两张时失去一点体力。
- `twzhengjian/twzhongchi`（征建/众斥）：开局选择检查其他角色出牌阶段“未使用非基本牌”或“未获得牌”；不满足条件者交给你一张牌，之后可切换检查条件。累计至少游戏人数一半（向上取整）的不同交牌角色后回复两点体力，后续改为可对目标造成一点伤害，并令自己受到【杀】的伤害加一。
- `twzhenxi/twyangshi`（震袭/扬师）：每回合一次在【杀】指定其他角色后，弃置至多距离张手牌、移动其装备区或判定区的一张牌，或在体力条件满足时依次执行两项；受到伤害后，攻击范围未覆盖所有其他角色则永久加一，否则从共享牌堆获得一张【杀】。
- `zhenshan`（振赡）：每回合一次声明当前可使用或打出的基本牌，选择一名手牌数更少的其他角色交换全部手牌，再继续结算该无实体牌基本牌。
- `twfenghan/twcongji`（锋捍/从击）：每回合一次在【杀】或伤害类锦囊指定首个目标后，选择至多等于该牌目标数的角色各摸一张牌；自己的红色牌于回合外因弃置进入弃牌堆后，可选择其中任意张交给一名其他角色。
- `twxuhe`（虚吓，TW 张飞）：你使用的【杀】被目标闪避后，其选择受到你造成的 1 点伤害，或获得「虚吓·威吓」（本回合内其受到伤害 +2，其回合结束时移除）。注：旧 JS 由闪避目标选择分支，DSL 以技能拥有者选择近似（注于 .skill 文件内）。
- `twtijin`（替巾）：攻击范围内的其他角色使用单目标【杀】时可改为以自己为目标；该【杀】完整结算后，弃置使用者一张手牌或装备牌。
- `twmoukui`（谋溃，TW 伏完）：每个【杀】目标可取消、摸牌、弃目标牌或背水执行两项；背水目标未因该【杀】进入濒死时，在完整结算后反弃来源一张牌。
- `twfuman`（抚蛮，TW 马忠）：每个出牌阶段向每名其他角色各赠一张手牌并追踪；追踪牌动态视为【杀】，其使用/打出完整结算后按是否造成伤害摸一或两张。
- `cuijin`（催进，TW 乐就）：自己或攻击范围内角色使用【杀】时可弃一张牌令基础伤害加一；整张【杀】未造成伤害时摸一张并对使用者造成一点伤害。
- `twfengpo`（凤魄，TW 马云騄）：每次单目标【杀】或【决斗】观看目标手牌，按方片数摸牌或增伤；首次杀人后永久改按红色牌数。
- `twxingluan`（兴乱，TW 樊稠）：结束阶段展示牌堆顶六张牌，选择一个类别全部分配且每名角色至多三张；所得牌数不少于技能拥有者的获牌角色各失去一点体力。


```bash
cargo run -q -p content-cli -- pack \
  content/original/ported/tw/cuijin.skill \
  content/original/ported/tw/gx_lingbaoxianhu.skill \
  content/original/ported/tw/jintao.skill \
  content/original/ported/tw/liuli_daxiaoqiao.skill \
  content/original/ported/tw/manji.skill \
  content/original/ported/tw/old_twrouke.skill \
  content/original/ported/tw/twbaimei.skill \
  content/original/ported/tw/twbianfuren.skill \
  content/original/ported/tw/twcaoang.skill \
  content/original/ported/tw/twcaozhao.skill \
  content/original/ported/tw/twcaoxiu.skill \
  content/original/ported/tw/twchengpu.skill \
  content/original/ported/tw/twchenxun.skill \
  content/original/ported/tw/twchenzhen.skill \
  content/original/ported/tw/twchuhai.skill \
  content/original/ported/tw/twdanji.skill \
  content/original/ported/tw/twdaoji.skill \
  content/original/ported/tw/twdianyi.skill \
  content/original/ported/tw/twfeiyi.skill \
  content/original/ported/tw/twfengpo.skill \
  content/original/ported/tw/twfenxian.skill \
  content/original/ported/tw/twfenwu.skill \
  content/original/ported/tw/twfuwan.skill \
  content/original/ported/tw/twfujian.skill \
  content/original/ported/tw/twfuman.skill \
  content/original/ported/tw/twfupan.skill \
  content/original/ported/tw/twguoyi.skill \
  content/original/ported/tw/twguohuai.skill \
  content/original/ported/tw/twjianwei.skill \
  content/original/ported/tw/twjiaohua.skill \
  content/original/ported/tw/twjuyan.skill \
  content/original/ported/tw/twkujian.skill \
  content/original/ported/tw/twluanchou.skill \
  content/original/ported/tw/twniujin.skill \
  content/original/ported/tw/twpuyangxing.skill \
  content/original/ported/tw/twqingkou.skill \
  content/original/ported/tw/twrangyi.skill \
  content/original/ported/tw/twruilian.skill \
  content/original/ported/tw/twsaotao.skill \
  content/original/ported/tw/twshenxing.skill \
  content/original/ported/tw/twsunyi.skill \
  content/original/ported/tw/twtanfeng.skill \
  content/original/ported/tw/twtianyu.skill \
  content/original/ported/tw/twzumao.skill \
  content/original/ported/tw/twwangcan.skill \
  content/original/ported/tw/twwangchang.skill \
  content/original/ported/tw/twwujing.skill \
  content/original/ported/tw/twxiawei.skill \
  content/original/ported/tw/twxingluan.skill \
  content/original/ported/tw/twxuhe.skill \
  content/original/ported/tw/twyizhu.skill \
  content/original/ported/tw/twyuanbu.skill \
  content/original/ported/tw/twyuyan.skill \
  content/original/ported/tw/twzhenglve3.skill \
  content/original/ported/tw/twzenhui.skill \
  content/original/ported/tw/twzhibian.skill \
  content/original/ported/tw/zhengfu.skill \
  content/original/ported/tw/zhenshan.skill \
  --characters content/original/ported/tw/characters.json \
  --metadata content/original/ported/tw/metadata.json \
  --out content/original/ported/tw/tw.npack.json \
  --package-id tw
```
