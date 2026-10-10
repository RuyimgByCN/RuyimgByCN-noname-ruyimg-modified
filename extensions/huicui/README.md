# HUICUI 确定性移植

`zangba`（臧霸）：4 体力魏势力完整武将，通过既有 `sp` 精确依赖复用 SP 包重写的
`rehengjiang`。每点受伤可令当前回合角色本回合手牌上限 -1；其弃牌阶段有弃牌时回合末
摸一张，否则摸本回合对其发动次数。临时修正与结算 helper 均在该回合结束后删除。

`dcgangu`：每回合限一次，其他角色失去体力后摸三张牌并失去一点体力；普通伤害不视为“失去体力”事件。

`llqshenwei`：摸牌阶段额外摸两张，手牌上限加二。

`dcxieshoux`：同名状态标记每层令手牌上限加一。

`dcyuxiang`：拥有者至其他角色的距离 -2；拥有者使用【杀】时，最终距离不大于 2 的目标不可响应。

`reshishou2`：准备阶段若没有“仓储”标记，强制失去一点体力；`recangchu` 后续移植须复用标记 ID `2058376721`。

`reshishou`（失守）：使用逻辑牌【酒】移去一枚“粮”；受到火焰伤害后按伤害点数移去“粮”；准备阶段没有“粮”时失去一点体力。

`recangchu`（仓储）：游戏开始时获得至多三个“粮”，手牌上限增加“粮”数；每回合限一次于回合外得牌后获得一个“粮”，死亡结算后将“粮”收缩至存活角色数。旧 `recangchu2`、`recangchu3` 的行为已内联进主技能，同时保留可独立执行的兼容定义。

`dcboxuan_rewrite`：旧 JS 明确声明的空替换技能；保留技能定义但不生成规则索引。

`qingman`（清蛮）：每个存活角色的回合结束时，将手牌补至该角色的空装备栏数。

`dccongshi`（从势）：一名角色使用装备牌并完成装备后，若其装备区牌数并列全场最多，拥有者强制摸一张牌。

`dcyingtu`（营图）：每回合限一次，上家或下家在其摸牌阶段外获得手牌后，可以获得其一张
手牌或装备牌，再将自己一张手牌或装备牌交给另一侧邻位；若交出装备牌且仍可合法使用，
受赠者立即使用。完整武将 `chengui` 与既有 `dccongshi` 闭包。

`dccunwei`（存畏）：成为锦囊牌的目标后，若是唯一目标则摸一张牌；否则有可弃置的手牌或装备牌时强制弃置一张。

`xixiu`（皙秀）：他人使用与自己装备同花色的牌指定自己时摸一张；只有一张装备时，该装备不能被他人弃置。

`chenjian`（陈见）：准备阶段可展示牌堆顶“3+陈见标记”张牌，按任意顺序执行弃牌后分配同花色展示牌、使用一张剩余展示牌；两项均执行后标记至多增加到 2，并重铸全部当前手牌。旧 JS 已重写为 DSL 与可恢复的类型化结算，不在运行时加载。

`tengyin`（滕胤）：3 体力吴势力完整武将，装配 `chenjian`、`xixiu`。

`dcshoucheng`（守成）：每回合限一次，一名非当前回合角色失去最后手牌后，可令其摸两张牌；
与 SHIJI 包的 `reshengxi` 组成完整蒋琬费祎武将。

`refuyuan`（扶援）：一名存活角色成为【杀】的目标后，若本回合没有成为过另一张红色实体牌
的目标，拥有者可令其摸一张牌；当前【杀】自身不进入“其他牌”判断。同一实体牌重复出现时仍
按旧 JS 的对象排除语义处理。陈登的 `reyingshui`、`rewangzu` 尚未重写，因此暂不发布武将。

`rezhixi`（扼睨）：锁定技，当你使用【杀】或普通锦囊牌时，你弃置一张手牌。以新增 bare 触发过滤器
`owner_source_sha_or_ordinary_trick`（无额外装备前置，镜像既有 `*_has_discardable_hand_equip` 变体的判定）
+ 既有 `discard_hand(self, 1)` 原语表达；旧 JavaScript 不进入运行时。

`jinglan`（惊澜）：锁定技，当你造成伤害后，按手牌数与体力值的关系三分支：大于弃四张手牌、等于弃一张
手牌或装备并回复一点、小于受到一点火焰伤害并摸五张。以既有 `owner_is_source` 过滤器 + DSL if/else
三分支表达；其中"弃一张手牌或装备"用新增原语 `discard_hand_equip`（ABI 350，跨栈：EffectRequest
`DiscardFromHandEquip` + opcode 322 + VM 解码 + StmtIr/codegen/typecheck + emit handler 镜像
`discard_from_hand`），HUICUI 包因此升至 ABI 350；旧 JavaScript 不进入运行时。

`dcqizi`（泣梓）：锁定技，你不能对至其的距离大于2且正在进行濒死流程的角色使用【桃】。以新增 modifier
`tao_use_save_cancel_for_dying_beyond_distance`（ModifierTarget 枚举变体 + registry 双向映射 +
typecheck 白名单 + `query/mod.rs` 的 `tao_cancelled_for_dying_beyond_distance` 检查函数接入
`card_save_cancelled_without_hand_source`，在 modifier 生效时用 `base_distance > 2` 判定）表达；
旧 JavaScript 不进入运行时。


`dccuijin`（崔进，界沮授）：任意角色使用【杀】或【决斗】时，若使用者是你或你与其距离不超过 1，可弃一张手牌/装备使该牌伤害 +1（ABI 454 目标过滤）。`dc_yuejiu` 完整闭包。


`lianzhou`（链舟，蔡瑁张允）：准备阶段锁定横置自己，并选择任意名体力值相同且未横置的
其他角色横置（ABI 458 `choose_link_other_same_hp`，复用 MARK_CHAIN 横置管线）。
完整武将 `caimaozhangyun` 已发布（ADR 0327）：`jinglan`（静澜）+ `lianzhou` 组合闭包。

完整武将 `mengyou`（孟优）已发布（ADR 0346）：`manyi`（蛮裔，既有）+
`dcmanzhi`（蛮智，ABI 475 `resolve_dcmanzhi` 双分支：①目标交给你至多两张牌并视为
使用无距离【杀】；②获得其区域内至多两张牌并交回等量后摸一张；每分支每回合一次，
以技能 mark 记录）组合闭包；旧 JS 不进入运行时。

完整武将 `gongsunxiu`（公孙修）已发布（ADR 0348）：`dcgangu`（肝骨，既有）+
`dckuizhen`（溃阵，ABI 477 `resolve_dckuizhen` 目标选择 + 决斗使用 + `resolve_dckuizhen_resolve`
在 DuelResolved 事件中按胜负结算：落败获其手牌【杀】，获胜则发起者失去 1 点体力）
组合闭包；待结算决斗以技能 mark（count=发起者 id）记录；旧 JS 不进入运行时。

完整武将 `re_kanze`（阚泽）已发布（ADR 0351）：`xiashu`（下书，既有）+
`rekuanshi`（宽释，ABI 480 `resolve_rekuanshi` 结束阶段标记 + `resolve_rekuanshi_resolve`
受伤结算：被标记角色本回合第 2 次受伤时回复 1 点体力，每回合一次）组合闭包；
旧 JS 不进入运行时。

跨包闭包修复（ADR 0367）：`mengyou` 显式装配 XIANDING 的 `manyi` 与本包
`dcmanzhi`；`re_kanze` 显式装配 SP 的 `xiashu` 与本包 `rekuanshi`。HUICUI 清单同步
声明 `xianding`、`sp` 依赖，扩展只需放入包目录即可按依赖装载。

完整武将 `peiyuanshao`（裴元绍）已发布（ADR 0352）：`dcmoyu`（魔狱，ABI 481
`resolve_dcmoyu` 夺牌反击复合：获得目标 1（或 2）张手牌/装备/判定牌，其视为对你
使用无距离【杀】；受伤则本回合禁用本技，否则下次多拿一张；每目标每回合一次）
单技能闭包；旧 JS 不进入运行时。

完整武将 `dc_gaolan`（高览）已发布（ADR 0354）：`xizhen`（袭阵，ABI 482
`resolve_xizhen` 选目标视为使用【杀】或【决斗】）单技能闭包；响应代偿子技能文档化
省略；旧 JS 不进入运行时。

完整武将 `xingdaorong`（邢道荣）已发布（ADR 0356）：`xuxie`（虚泄，ABI 483
`resolve_xuxie` 上限牺牲 + 相邻弃牌/摸牌二选一 + `resolve_xuxie_add` 阶段末恢复）
单技能闭包；旧 JS 不进入运行时。

完整武将 `gaoxiang`（高翔）已发布（ADR 0359）：`dcchiying`（驰应，ABI 484
`resolve_dcchiying` 循环弃牌夺基复合：目标在其攻击范围内循环选人弃一张，停止后获得
所弃基本牌）单技能闭包；旧 JS 不进入运行时。

完整武将 `re_panfeng`（潘凤）已发布（ADR 0361）：`xinkuangfu`（匡扶，ABI 485
`resolve_xinkuangfu` 弃装授杀复合：弃自身/他人装备一张，视为使用无距离【杀】；自弃
受伤摸二、他弃未伤弃二）单技能闭包；旧 JS 不进入运行时。

完整武将 `zhangkai`（张闿）已发布（ADR 0362）：`dcxiangshu`（相术，ABI 486
`resolve_dcxiangshu` 秘密数字选择 + `resolve_dcxiangshu_lottery` 差值结算：|手牌数-
数字|≤1 获得其一张牌，==0 造成 1 点伤害）单技能闭包；弃牌保密步骤省略；旧 JS 不
进入运行时。

> 交付物：`package.npack.json` 与 `huicui.npack.json` 同源；详见 ABI-COORDINATION.md。
