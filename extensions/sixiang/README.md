# 四象确定性移植

| 武将 | 技能 | 状态 |
|---|---|---|
| `std_caozhen`（标曹真） | `stdsidi`（司敌） | 单技能武将，响应【杀】事件与可选摸牌 Golden 已覆盖 |
| `std_baosanniang`（标鲍三娘） | `stdduliang`（笃良） | 受伤后可摸 1，再按摸牌后的手牌/体力状态回复，Golden 已覆盖；其余技能未完成，暂不发布武将 |
| `std_mengda`（标孟达） | `stdzhuan`（赚） | 每回合首次受伤后摸三张，再由存活伤害来源获得一张手牌或装备牌 |
| `std_quyi`（标麴义） | `stdfuqi`（伏骑）、`stdjiaozi`（骄恣） | 双向距离 1 的整次【杀】不可响应；首次伤害加伤语义与多目标 Golden 已覆盖 |
| `std_niujin`（标牛金） | `stdcuorui`（挫锐） | 开局或杀死角色后可补至存活人数（至多 7）；低于阈值才询问，Golden 已覆盖 |
| `std_db_wenyang`（标文鸯） | `stdquedi`（却敌） | 手牌区或装备区内有效牌名为【杀】的牌可当【决斗】使用，复用完整决斗管线 |
| `std_lvfan`（标吕范） | `mbdiaodu`（调度）、`stddianfeng`（典封） | 完整武将；`mbdiaodu` 由始计包提供，四象包声明精确依赖 |
| `std_zhangyì`（标张翼） | `stdzhiyi`（执义） | 完整单技能武将；按角色记录本回合实际使用的【杀】，可使用不计次数的虚拟【杀】或摸一张牌 |
| `std_feiyi`（标费祎） | `stdtiaohe`（调和）、`stdqiansu`（谦素） | 完整武将；两名目标各弃一张装备，权威选择保证至少一张为防具且遵守弃置保护 |
| `std_zhonghui`（标钟会） | `stdxingfa`（兴伐） | 完整单技能武将；准备阶段精确检查手牌不少于体力，由拥有者选择一名其他角色造成 1 点普通伤害 |
| `std_wangyuanji`（标王元姬） | `stdqianchong`（谦冲）、`stdshangjian`（尚俭） | 完整武将；结束阶段按需读取本回合失牌溯源，从仍在弃牌堆的候选中取回一张 |
| `std_panglin`（标庞林） | `stdzhuying`（驻营）、`stdzhongshi`（忠事） | 完整武将；其他未横置角色将受普通伤害时可令其横置，随后复用横置状态差异加伤 |
| `std_simazhao`（标司马昭） | `stdzhaoxin`（昭心） | 完整单技能武将；准备阶段公开全部手牌，颜色一致时强制选择任意角色造成 1 点普通伤害 |
| `std_liuba`（标刘巴） | `stdduanbi`（锻币） | 完整单技能武将；结束阶段可弃全部手牌，强制选择两名不同角色并依座次各摸两张牌 |
| `std_jiakui`（标贾逵） | `stdzhongzuo`（忠佐）、`stdwanlan`（挽澜） | 完整武将；挽澜限定救援会交出全部手牌，等待移动触发结算完毕后将另一名濒死角色回复至 1 |
| `std_wangyun`（标王允） | `stdyunji`（运机）、`stdzongji`（纵计） | 完整武将；【杀】或【决斗】造成伤害后，可依次弃置受伤者与伤害来源各一张手牌或装备牌 |
| `std_dongyun`（标董允） | `stdbingzheng`（秉正）、`stdduliang`（笃良） | 完整武将；结束阶段可令任意一名角色自行弃置一张手牌或装备牌，移动触发结算后若其手牌数不等于体力值则拥有者失去 1 点体力 |
| `std_wuke`（标吴珂） | `stdanda`（谙达）、`stdzhuguo`（助国） | 完整武将；每轮一次在有存活伤害来源的濒死事件中，由来源选择交牌，否则令濒死角色回复 1 点体力 |
| `std_dingfeng`（标丁奉） | `stdduanbing`（短兵）、`stdfenxun`（奋迅） | 完整武将；弃置一张防具并绑定一名其他角色，该角色本回合视为在攻击范围内 |
| `std_liuzan`（标留赞） | `stdfenyin`（奋音） | 完整单技能武将；摸牌阶段可额外摸两张，本回合连续使用同色牌时强制弃一张牌 |
| `std_dc_yanghu`（标羊祜） | `stdmingfa`（明伐） | 完整单技能武将；伤害结算后若目标仍存活则暂时替换技能，绑定目标回复或死亡后恢复 |
| `std_sunyi`（标孙翊） | `stdzaoli`（躁厉） | 完整单技能武将；准备阶段弃置一个所选区域的全部牌，按弃牌数与已损体力摸牌后失去体力 |
| `std_liubiao`（标刘表） | `stdzishou`（自守）、`zongshi`（宗室）、`stdjujin`（聚进） | 完整武将；`zongshi` 由一将成名包提供，四象包声明精确依赖；受群势力来源伤害后可弃两张牌并回复 |
| `std_lijue`（标李傕） | `stdxiongsuan`（凶算） | 完整单技能武将；准备阶段体力并列全场最高时，强制选择至少一名同体力角色并依座次造成伤害 |
| `std_zhangfen`（标张奋） | `stdwanglu`（望橹） | 完整单技能武将；弃置场上一张可弃置装备，若为武器则其原装备者按攻击范围摸牌 |
| `std_ganfuren`（标甘夫人） | `stdzhijie`（智诫）、`stdshushenx`（淑慎） | 完整武将；每轮展示当前出牌阶段角色一张手牌并记录类别，其本阶段前三次使用同类别牌时各摸一张 |
| `std_wenyuan`（标文鸳） | `stdkengqiang`（铿锵）、`stdshangjue`（殇决） | 完整武将；伤害牌二选一强化，限定技濒死回满后可牺牲铿锵重置 |
| `std_huangfusong`（标皇甫嵩） | `stdtaoluan`（讨乱） | 完整单技能武将；其他角色结束阶段可交给其一张牌，再公开并弃置其所有【闪】 |
| `std_sunhao`（标孙皓） | `stdcanshi`（残蚀）、`chouhai`（仇海）、`guiming`（归命） | 完整主公武将；后两项由 SP 包提供，四象包声明精确依赖 |
| `std_xiahouba`（标夏侯霸） | `stdbaobian`（豹变） | 完整单技能武将；先失去体力，再令目标弃手牌，基本牌触发忽略距离但遵守次数的虚拟【杀】 |
| `std_mayunlu`（标马云禄） | `stdfengpo`（凤魄）、`mashu`（马术） | 完整武将；伤害提交前从【杀】双方选择并弃置一张手牌或装备，方片令该段伤害 +1 |

另已完成独立技能 `stdshibei`（矢北）：首次受伤回复、移除自身实例实现限定一次，Golden 已覆盖。

`stdhuangong`（还宫）：失去装备区或判定区的最后一张牌后摸一张牌。

`stdtianshu`（天枢）：手牌上限增加存活角色势力数减一。

`stdqianchong`（谦冲）：装备区牌数为奇数时用牌无次数限制，为偶数（含 0）时用牌无距离限制。

`oldjuejing`（旧绝境）：摸牌阶段额定摸牌数增加已损失体力值，手牌上限 +2。

`stdqiansu`（谦素）：成为锦囊牌目标且装备区为空时，可摸一张牌。

`stdwuyan`（无言）：实体锦囊牌均按【无懈可击】使用，实体牌名保持不变。

`stdshanshen`（善身）：非拥有者造成其他角色死亡后，受伤的拥有者可回复 1 点体力。

`stdzhongyan`（忠言）：群势力角色死亡后，受伤的主公可回复 1 点体力。

`stdanran`（安然）：受到伤害后可摸递增的 1/2/3/4 张牌，此后保持为 4 张。

`stdyuqi`（隅泣）：任意角色受伤后，若其至拥有者的距离不大于拥有者当前体力，拥有者可令其摸一张牌。

`stdqingman`（清蛮）：每回合结束时，将手牌补至自身空装备栏数与 3 中的较小值。

`stdshuyong`（姝勇）：其他角色在其回合内连续使用两张同名牌时，拥有者可摸一张牌；装备牌按具体牌名比较，普通/火/雷【杀】均视为【杀】。

`stdjiaozi`（骄恣）：每回合首次造成伤害时，若手牌数为全场最多（允许并列），令此伤害 +1。

`stdshiren`（士仁）：每回合限一次，成为其他角色【杀】的目标后，可摸两张牌并交给使用者一张手牌或装备牌。

`stdfuqi`（伏骑）：拥有者使用【杀】或成为【杀】目标时，若另一方至拥有者的距离为 1，则整次【杀】的所有目标均不可响应。

`stdquedi`（却敌）：可将手牌区或装备区内有效牌名为【杀】的实体牌当【决斗】使用。

`stdyunji`（运机）：手牌或装备区中的装备牌可当【借刀杀人】使用。

`stdduanbing`（短兵）：攻击范围始终为 1；每回合首次由【杀】造成的伤害 +1。

`stdyinzu`（荫族）：每名存活的技能拥有者各提供一层全场攻击范围光环；被查询角色手牌数大于体力值时每层 +1，否则每层 -1。

`stdzhongshi`（忠事）：伤害来源与目标的横置状态不同时，来源造成的伤害 +1。

`stdzhuying`（驻营）：其他存活且未横置的角色即将受到普通伤害时，拥有者可令其横置；
属性伤害、拥有者自己受伤及已经横置的目标均不询问。

`stdhuoshui`（祸水）：每名存活的技能拥有者各提供一层受伤光环；判定区有牌的其他角色受到的伤害每层 +1。

`stdzhongzuo`（忠佐）：一名角色的回合结束时，若拥有者本回合造成或受到过伤害，当前回合角色与拥有者依座次各摸一张牌；体力流失不计为伤害。

`stddingyi`（定仪）：一名角色进入结束阶段时，若其装备区为空，由该角色决定是否发动；接受后该角色摸一张牌。

`stddianfeng`（典封）：一名角色本次确实从装备区失去至少一张牌且结算后装备区为空时，拥有者可摸一张牌；本来就没有装备或仅有其他区域移动不会触发。

`stdxingfa`（兴伐）：准备阶段手牌数不少于当前体力且存在其他存活角色时可发动；确认后选择一名其他角色，由拥有者对其造成 1 点普通伤害。

`stdzhaoxin`（昭心）：准备阶段有手牌时强制公开全部手牌；若其有效颜色一致，拥有者强制选择任意一名存活角色（包括自己）并对其造成 1 点普通伤害。

`stdduanbi`（锻币）：结束阶段有手牌时可发动；确认后弃置所有手牌。若场上至少有两名存活角色，再强制选择两名不同角色（可包括自己），依座次各摸两张牌。

`stdwanlan`（挽澜）：限定技。另一名角色进入濒死时，若拥有者有手牌可选择发动；交给其全部当前手牌，
待卡牌移动及嵌套触发全部结算后，将其回复至 1 点体力。没有手牌、自己濒死或已经发动过时不会询问。

`stdzongji`（纵计）：逻辑【杀】或【决斗】造成伤害后，若存活的伤害来源与受伤角色合计至少有一张
手牌或装备牌，存活拥有者可发动；先从受伤角色、再从伤害来源各选择并弃置一张合法牌。任一方没有
合法牌时跳过该次选择，普通伤害、其他牌造成的伤害、无来源伤害或参与者死亡时均不询问。

`stdbingzheng`（秉正）：拥有者进入结束阶段时，若至少一名存活角色可以自行弃置一张手牌或装备牌，
可选择发动。确认后拥有者选择目标，再由目标强制选择并弃置自己的一张合法牌；等待该次移动和嵌套
触发全部结算后重新读取目标手牌数与体力值，不相等则仍存活的拥有者失去 1 点体力。

`stdzhuguo`（助国）：出牌阶段限一次，选择一名当前手牌数少于 2 的存活角色（可以是自己），
令其摸牌至两张。目标手牌阈值由 `hand_lt 2` 声明并在主动技提交时重新验证。

`stdanda`（谙达）：每轮限一次，一名角色在有存活伤害来源的情况下进入濒死时，存活拥有者
可以发动。确认后由伤害来源决定是否交给濒死角色一张当前手牌或装备牌；取消或没有牌时，
濒死角色回复 1 点体力。技能拥有者、Choice 观察者和目标分别重新验证。

`stdfenxun`（奋迅）：出牌阶段限一次，选择一名其他存活角色并弃置手牌区或装备区内的一张
物理防具牌；通用目标绑定技能令该角色视为在拥有者攻击范围内，并在拥有者回合结束时移除。

`stdfenyin`（奋音）：摸牌阶段可额外摸两张；接受后，本回合每次使用与紧邻上一张用牌有效
颜色相同的牌时，必须选择并弃置一张当前手牌或装备牌。无色牌不参与红黑颜色比较。

`stdmingfa`（明伐）：出牌阶段可选择一名当前体力值大于 1 的角色（包括自己）并对其造成
1 点普通伤害。等待该次伤害、嵌套触发及濒死求桃全部结算后，若目标仍存活，则用绑定该
目标的临时技能替换〖明伐〗；该目标之后回复体力或死亡时，恢复〖明伐〗。

`stdzaoli`（躁厉）：锁定技。拥有者的准备阶段，若手牌区和装备区至少一处存在可弃置牌，
只有一处非空时直接选择该区，两处均非空时由拥有者强制选择一处；弃置该区全部可弃置牌，
待卡牌移动及嵌套触发结算后，按实际弃牌数与此时已损失体力值之和摸牌；待摸牌触发结算后
失去 1 点体力。

`stdzishou`（自守）：拥有者进入出牌阶段时可摸存活角色势力数张牌，然后跳过本次出牌阶段。
旧 JavaScript 的 `await draw` 后 `cancel` 顺序保留为同一触发体内的摸牌、跳过阶段两步效果。

`stdjujin`（聚进）：主公受到群势力角色造成的伤害后，若当前至少有两张可弃手牌或装备牌，
可精确选择并弃置两张；待移动触发与嵌套选择完成后回复 1 点体力。成本在 Choice 恢复时重验。

`stdxiongsuan`（凶算）：锁定技。准备阶段若拥有者当前体力并列全场最高，必须从所有与其
当前体力相等的存活角色中选择至少一名（可选择自己），再按座次由拥有者对每名目标造成
1 点普通伤害；目标范围、技能和存活状态在 Choice 恢复时重验。

`stdwanglu`（望橹）：出牌阶段限一次，选择一名（可为自己）拥有至少一张拥有者可弃置装备的
存活角色，由拥有者选择并弃置其一张装备牌。若该牌是实体武器，等待弃置移动及嵌套触发全部
结算后，令原装备者摸该武器攻击范围张牌（异常的零范围按一张处理）；其他装备不摸牌。

`stdmutao`（募讨）：准备阶段若场上至少一名存活角色有手牌，拥有者可发动；确认后强制选择
一名有手牌的存活角色（可为自己），公开其全部当前手牌。若其中至少一张牌对该角色的有效牌名
为【杀】，拥有者对其造成 1 点普通伤害；目标、手牌和逻辑牌名在 Choice 恢复时重验。

`stdyimou`（毅谋）：受到正数伤害后，若拥有者有手牌或装备牌且场上存在其他存活角色，可以
发动。确认后强制选择一名其他存活角色，再从自己的当前手牌/装备区选择一张牌，通过统一
`Give` 移动管线交给该角色；技能、目标、卡牌归属和区域在两个 Choice 屏障处分别重验。
至此标鲍信的两个技能均已重写，整将以 4 体力群势力进入 SIXIANG 包。

`stdshushenx`（淑慎）：每回合结束时，若拥有者于本回合累计获得至少三张实体牌且场上存在
其他存活角色，可以选择一名其他角色令其回复 1 点体力。累计获得数由共享卡牌入手路径按玩家
记录，并仅在加载相关过滤器时启用。

`stdzhijie`（智诫）：每轮限一次，一名角色进入出牌阶段且有手牌时，拥有者可以选择并公开
其一张当前手牌，记录该角色与牌的有效基本牌/锦囊牌/装备牌类别。该角色于这个出牌阶段
前三次使用同类别牌时，拥有者各摸一张；类别绑定在任意真实阶段切换时清除。至此标甘夫人
的两个技能均已重写，整将以 3 体力蜀势力进入 SIXIANG 包。

`stdkengqiang`（铿锵）：每回合限一次，拥有者使用逻辑【杀】或伤害类锦囊牌时可以发动，
确认后选择摸两张牌，或先失去 1 点体力再将当前挂起牌的伤害基数增加 1。当前牌和触发次数
均由权威状态绑定。

`stdshangjue`（殇决）：限定技。拥有者进入濒死状态时可以发动，先消费本技能并回复至体力
上限；回复事件及其嵌套触发全部完成后，若仍拥有「铿锵」，可以失去「铿锵」并重新获得一枚
未发动的「殇决」实例。至此标文鸳的两个技能均已重写，整将以 3 体力蜀势力进入 SIXIANG 包。

`stdjishe`（极奢）：出牌阶段可反复摸一张牌；每次发动都以该主动技本回合累计调用次数令
手牌上限减一，当前手牌上限降至零后不能继续发动，调用次数在新回合统一清零。

`stdwudu`（无度）：一名没有手牌的角色将受到伤害时可以发动，先防止当前挂起伤害，再令
拥有者减少 1 点体力上限。至此标岑昏的两个技能均已重写，整将以 3 体力吴势力进入 SIXIANG 包。

`stdtaoluan`（讨乱）：其他角色进入结束阶段时，若拥有者有手牌或装备牌，可以交给该角色
一张牌；移动与嵌套触发结算后公开该角色的当前手牌，并弃置其中所有有效牌名为【闪】的牌。
至此标皇甫嵩以 4 体力群势力进入 SIXIANG 包。

`stdcanshi`（残蚀）：摸牌阶段强制把摸牌数改为存活的受伤角色数且至少为一；主公身份下
「归命」生效时，其他存活吴势力角色也计入且不重复。随后直到回合结束，使用【杀】或普通
锦囊牌指定受伤角色后强制弃置一张当前可弃手牌或装备牌。`chouhai`、`guiming` 由 SP 包提供，
至此标孙皓以 5 体力吴势力进入 SIXIANG 包。

`stdshangjian`（尚俭）：结束阶段，若拥有者本回合从手牌区或装备区累计失去一至当前体力值
张牌，且其中至少一张实体牌仍在弃牌堆，可以发动并从这些唯一候选中获得一张。重复失去同一
张牌会重复计入失牌数，但候选只出现一次；失牌溯源仅在加载相关过滤器时启用。

`stdzuici`（罪辞）：拥有者受到有来源的伤害后，可以把伤害来源以外存活角色装备区或判定区
的一张牌移到伤害来源的对应区域。装备牌要求目标同槽空置且槽位可用，延时锦囊不能与目标
判定区内同类牌重复。至此标孙邵的两个技能均已重写，整将以 3 体力吴势力进入 SIXIANG 包。

`stdbaobian`（豹变）：拥有者进入出牌阶段且场上存在可自行弃置手牌的角色时可发动；选择
目标后先失去 1 点体力，再令目标弃置一张手牌。移动与嵌套触发全部结算后，若弃牌为基本牌且
双方仍存活，则视为对目标使用一张忽略距离、遵守通常次数限制的【杀】。

`stdfengpo`（凤魄）：拥有者以逻辑【杀】造成伤害前，若自己或受伤角色存在可弃置的手牌或
装备牌，可选择发动并从双方的合法候选中弃置一张。待弃牌移动及嵌套触发结算完毕后，若该牌
对原拥有者的有效花色为方片，则当前挂起伤害及其基数增加 1；火【杀】的连环伤害段同样适用。

重新生成包：

```bash
cargo run -q -p content-cli -- pack \
  content/original/ported/sixiang/stdsidi.skill \
  content/original/ported/sixiang/stdshibei.skill \
  content/original/ported/sixiang/stdduliang.skill \
  content/original/ported/sixiang/stdhuangong.skill \
  content/original/ported/sixiang/stdtianshu.skill \
  content/original/ported/sixiang/stdqianchong.skill \
  content/original/ported/sixiang/oldjuejing.skill \
  content/original/ported/sixiang/stdqiansu.skill \
  content/original/ported/sixiang/stdwuyan.skill \
  content/original/ported/sixiang/stdshanshen.skill \
  content/original/ported/sixiang/stdzhongyan.skill \
  content/original/ported/sixiang/stdanran.skill \
  content/original/ported/sixiang/stdyuqi.skill \
  content/original/ported/sixiang/stdqingman.skill \
  content/original/ported/sixiang/stdshuyong.skill \
  content/original/ported/sixiang/stdjiaozi.skill \
  content/original/ported/sixiang/stdzhuan.skill \
  content/original/ported/sixiang/stdshiren.skill \
  content/original/ported/sixiang/stdfuqi.skill \
  content/original/ported/sixiang/stdcuorui.skill \
  content/original/ported/sixiang/stdquedi.skill \
  content/original/ported/sixiang/stdyunji.skill \
  content/original/ported/sixiang/stdduanbing.skill \
  content/original/ported/sixiang/stdyinzu.skill \
  content/original/ported/sixiang/stdzhongshi.skill \
  content/original/ported/sixiang/stdzhuying.skill \
  content/original/ported/sixiang/stdhuoshui.skill \
  content/original/ported/sixiang/stdzhongzuo.skill \
  content/original/ported/sixiang/stddingyi.skill \
  content/original/ported/sixiang/stddianfeng.skill \
  content/original/ported/sixiang/stdzhiyi.skill \
  content/original/ported/sixiang/stdtiaohe.skill \
  content/original/ported/sixiang/stdxingfa.skill \
  content/original/ported/sixiang/stdzhaoxin.skill \
  content/original/ported/sixiang/stdduanbi.skill \
  content/original/ported/sixiang/stdwanlan.skill \
  content/original/ported/sixiang/stdzongji.skill \
  content/original/ported/sixiang/stdbingzheng.skill \
  content/original/ported/sixiang/stdzhuguo.skill \
  content/original/ported/sixiang/stdanda.skill \
  content/original/ported/sixiang/stdfenxun.skill \
  content/original/ported/sixiang/stdfenyin.skill \
  content/original/ported/sixiang/stdmingfa.skill \
  content/original/ported/sixiang/stdzaoli.skill \
  content/original/ported/sixiang/stdzishou.skill \
  content/original/ported/sixiang/stdjujin.skill \
  content/original/ported/sixiang/stdxiongsuan.skill \
  content/original/ported/sixiang/stdwanglu.skill \
  content/original/ported/sixiang/stdmutao.skill \
  content/original/ported/sixiang/stdyimou.skill \
  content/original/ported/sixiang/stdshushenx.skill \
  content/original/ported/sixiang/stdzhijie.skill \
  content/original/ported/sixiang/stdkengqiang.skill \
  content/original/ported/sixiang/stdshangjue.skill \
  content/original/ported/sixiang/stdjishe.skill \
  content/original/ported/sixiang/stdwudu.skill \
  content/original/ported/sixiang/stdtaoluan.skill \
  content/original/ported/sixiang/stdcanshi.skill \
  content/original/ported/sixiang/stdbaobian.skill \
  content/original/ported/sixiang/stdshangjian.skill \
  content/original/ported/sixiang/stdzuici.skill \
  content/original/ported/sixiang/stdfengpo.skill \
  --characters content/original/ported/sixiang/characters.json \
  --metadata content/original/ported/sixiang/metadata.json \
  --out content/original/ported/sixiang/sixiang.npack.json \
  --package-id sixiang
```
