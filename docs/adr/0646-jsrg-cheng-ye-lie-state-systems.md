# ADR 0646：jsrg 状态系统三族（城/访客/猎）切片章程

- 状态：**章程（2026-08-26 第七十七批·设计批，无代码）**——蓄谋族于切片 3b
  闭环后（ADR 0645 §5b），对第七十批「全包浅面终审」确认的三个自定义状态
  系统逐族普查成员、标注已可达面与缺口，作为后续批次章程。
- 关联：ADR 0645（深水区章程/§5b 蓄谋族）、ADR 0644（议事系统）、
  `docs/migration/upstream-1.11.5-delta.md` 第七十六批。

## 0. 蓄谋族闭环加注（2026-08-26）

上游 `addJudge({name:"xumou_jsrg"})` 全部置入点仅 3 处：jsrgeqian①、jsrgdaimou、
jsrgcangxiong。前两者已于第七十六批迁移（判定区 + XUMOU_TAG 实现，与 0645 §5b
原「ExpansionMap+query 合并」草案分歧：实走了 judge-zone 物理位，天然获得
每回合判定结算事件面，code 55 已透传）。jsrgcangxiong 需「弃置/他人得到」双复合
过滤器仅养一技、`xumou_jsrg_temp` 需按牌名的 cardEnabled 去重名册设施——均不达
3/3 立项门槛。**族在当前保真度闭环**；eqian② 的 xumou-origin 目标交互仍留债务。

## 1. 【猎】标记族（3 技，最低水温）

> **状态（2026-08-26 第七十八批·S1 已落地）**：政略①③、会猎觉醒、平戎「移去
> 一猎」均已迁移（ABI 仍 666，triggers 44→49）；新增过滤器 ×4：
> `owner_alive_event_player_lord` / `owner_source_damage_target_mark_positive_
> <def>_once`（turn_limit 登记）/ `owner_alive_marked_alive_above_two_<def>` /
> `owner_alive_someone_mark_positive_<def>`；choose_other_add_mark 负 delta 走
> 钳位+候选过滤。「猎」def=blake2b 合成 1167524380。遗留项后续批已全部收口：
> 政略①双授分支。
>
> **状态（2026-08-26 第八十一批·S3 已落地）**：额外回合经既有
> `extra_turn_queue` 消费管线承载（此前「引擎 grep 零命中」系搜索面误判——
> 生产端无通用 DSL 原语，消费端 turn.rs 完备）。新原语 `queue_extra_turn`
> （ABI 669 领号/tag 692）：入队 + PENDING 标记；pop 分支转 ACTIVE；
> continue_turn_advance 头部结算尾随失去体力 check（DAMAGE_DEALT==0）。
> 平戎自此完整（近似仅余死亡场景下的引擎默认回合顺序处理，已为终局口径）。
>
> **状态（2026-08-26 第八十二批·政略①双授分支落地）**：新原语
> `choose_others_add_mark(player, mark, delta, max_count, self_skill)`
> （ABI 670 领号/tag 693/Cont 280）+ 过滤器对
> `owner_alive_event_player_lord_{damaged,no_damage}`（DAMAGE_DEALT 回合聚合）。
> 政略剩余缺口仅②豁免值源。
>
> **状态（2026-08-26 第八十三批·政略② 豁免已落地）**：未新增 ValueSource——
> 引擎 per-target 查询钩子（sha_use_unlimited_for_target /
> card_distance_unlimited_for_target）天然携带 target 上下文，走 ModifierTarget
> 参数化族：`ShaUseUnlimitedTargetMark(u32)` / `CardDistanceUnlimitedTargetMark(u32)`
> （token `*_target_mark_<mark>`、override 1 门控、registry 尾追序数稳定）。
> zhenglve②自此真实建模；猎族可达面全部收口。
>
> **普查（2026-08-26 第八十四批）**：上游 `insertPhase()` 全树 57 处/26 文件；
> 首个包外消费者 **key/umi_lunhui 轮回** 落地（queue_extra_turn 泛化 check=0
> 形态+storage 绑定门控，key ABI 651→669）。剩余可复用面按包计数：offline 17、
> xianding 6、refresh/diy/tw/jsrg 各4（jsrg 已迁/特殊）、其余 ≤2；多数耦合
> chooseButtonTarget/转阶段排序等复合面，逐批按「体=queue+既有原语全绿」筛选。
> 
> **offline/xianding 复核登记（2026-08-26 第八十六批）**：两包 23 处逐块核验，
> **零「body=queue_extra_turn±既有原语」纯形候选**——
> | 文件(处) | 耦合点 | 判定 |
> |---|---|---|
> | offline_chunqiu(2) | 双人额外回合+技能互换 | 引擎级 |
> | offline_guansuo(2)/jiudin(1) | 亡者额定回合名单+座次相邻环判定 | 引擎级 |
> | offline_juedai zj_fuli(2) | 濒死 enable+recoverTo+`phaseList=["phaseUse"]` 相位裁剪 | 需 phase-list 级插入原语 |
> | offline_piracyE/jb/zy(7) | dieBegin 禁置、phaseUseBefore 强制跳相(skip(name))、phaseList+_noTurnOver | 引擎级 |
> | prettyDerby(2)/shiningGrils(1) | roundStart 行情钩子/onround 库挂载 | 引擎级 |
> | extra_offline(1，计数勘误：原记 2 系复扫混计) | sm_kulian 轮度竞赛奖励：「击杀数唯一最多」→ `insertPhase("sm_kulian")` 且 `phaseList=["phaseUse"]`；判定依赖 `game.getRoundHistory` 唯一最多 + initList 全局奖池 | 不可达：相位裁剪+轮度唯一最多竞赛系统（②∪③）|
> | xianding dcqijing(1) | 觉醒+座次重排(previous/lastPhasedPlayer) | 引擎级 |
> | xianding dcsbbizuo(1) | target 插回合+when 链+全局弃牌历史+∞ 分配盘 | 引擎级 |
> | xianding fengying(1) | 同左（尾钩） | **已迁(勘误)**：移植目录现存 active once_per_game + resolve_fengying 多路器(action0 全弃+deferred 入队 / action2 drawTo(maxHp) 子技)——第八十六批登记时未查移植实态，误列排队 |
> | xianding dcwumei(2) | phaseList/handler/全局历史深操 | 引擎级 |
>
> | **twjieqiu 装备栏族立项评估（2026-08-26 第九十四批）**：引擎底座完备——
> `disabled_equip_slots` 位段 + `disable_equip_slot/enable_equip_slot` fn +
> EquipSlotDisabled/Enabled 双事件（pack_sp_2.rs 同款）。缺的是编排层：
> **立项 resolve_twjieqiu 多路器**（照 resolve_fengying 形状）——
> action0：chooseTarget(other) → 全六槽 disable（其装备区现存牌入弃牌堆）→
> 目标摸 k 张（k=废除前装备区牌数，AddMark 打 GUILI_BIND 式回收标 delta=k）；
> action1：目标弃牌阶段结束时逐槽 enable(k 次)（消费 k 计数标）；
> action2：目标回合结束若仍有禁用栏（disabled bits !=0）→ owner 入队额外回合。
> 需领号 ABI 672 + Cont 282 + 一个新专用过滤器
> `owner_alive_event_target_mark_positive_discard_end_<mark>`（5 处注册）；
> **复核撤销（2026-08-26 第九十六批）**：通读上游全文后确认三处深水耦合，
立项撤消、判定升级为引擎级不可达——
①恢复侧依赖「本次弃牌阶段的实际弃牌张数」选择恢复哪些槽类别
（chooseButton 数量上限=min(transList.length,num)），而引擎 TriggerContext
对 PhaseEnded{Discard} 不携带本次弃牌卡片集合，Rust 侧无对应源数据；
②跨角色 buff 名单（markAuto 存 PlayerId 列表+onremove 生命周期）
超出单标记表达；
③额外回合入口由目标自身 chooseBool 决定并以 addTempSkill used(roundStart)
做每轮门，插件化入口需 when(phaseBegin)/phaseList 相位裁剪支持。
结论：twjieqiu 归入「装备栏编排子族」专项债，与 dcwumei/zj_fuli 同池。
>
> **dcwumei 无昧领号级立项书（2026-08-28 第一百零七批，§6 S3 终判）**：
> 上游全文核验后，本体远超「相位重排」——核心是**全场体力快照/回滚**：
> wake 子技 phaseBegin 记录所有角色 hp → phaseEnd 全员 changeHp 回滚至记录值；
> 另叠加 target 插相、owner 二段插相（phaseList=当前 phaseList 重演）、
> nexts 队列操纵、pushHandler 全局历史栈注入、_finished/untrigger 深度事件流
> 控制。依赖「选择性状态快照/恢复」设施（引擎 Transaction journal 已有基础，
> 但业务层部分回滚=新原语族），规模多批级。**S3 终判**：dcwumei 与 twjieqiu
> 终判归入「装备栏编排子族」专项池（终态：登记债，待选择性状态快照/恢复
> 设施立项后合并实现；不再单独排期）。
>
> **勘误（第一百零二批收尾核验）**：「政略②豁免值源」早在**第八十三批**已落地
> （`ShaUseUnlimitedTargetMark`/`CardDistanceUnlimitedTargetMark` 两变体 +
> zhenglve.skill 两条 modifier 一行式 + query 双接线，jsrg npack ABI 670 已含，
> 三处部署一致）——§1 遗留清单中该项为**过时描述，撤销**。政略①③②至此全部
> 真实建模。猎族可达面收敛完毕，余项仅凝寒改色层（已终判不可达）与政略②
> 反向豁免场景（改判：与凝寒同因，无触发源，不可达登记）。
>
> **改判（2026-08-28 第一百零二批·已落地）**：按简化的两支多路器实现——
> sub0（active 出牌阶段限一次）：选目标 → `disable_equip_slots` 全五槽
> （引擎自动将现有装备入弃牌堆+EquipSlotDisabled 事件）→ 目标摸 k 张
> （k=禁用前装备区牌数）→ BUFF 标记（合成 blake2b("twjieqiu_buff")=
> 385592780）；sub2（`turn_ended owner_alive_event_target_mark_positive_
> <mark>` optional，storage 每场一次锁）→ 仍余禁用栏则 owner 入队纯额外
> 回合（复用 queue_extra_turn）。领号 ABI 673/Cont 282 TwjieqiuSelect/
> 新过滤器 `owner_alive_event_target_mark_positive_<mark>`（5 处注册）。
> 残差状态（第一百零六批·终判登记）：①恢复槽类别 chooseButton 选择盘——残差
关闭登记。即时等量恢复已覆盖**状态语义**（禁用槽数量收敛一致）；剩余差异仅为
「玩家偏好表达」（选择恢复顺序权）。实现需 PhaseEnded{Discard} 携带弃牌集合
的引擎事件扩展 + 逐张交互盘 + 领号 ABI 674，投入/收益比失衡——除非未来出现
≥3 处同类「按弃牌量选类别恢复」消费者（当前仅 twjieqiu 一处），方可重开立项。
②used(roundStart) 每轮门收窄为每场一次——维持。
dcwumei 动态重排：终态并入 §6 相位裁剪原语族 S3（依赖 phaseList 级原语与
全局历史注入，已出具领号级立项书，终态登记债）。
>
> **S0/S1 落地（2026-08-26 第九十八批）**：`turn.extra_phase_queue:
> Vec<ExtraPhaseTurn{player,phases(u8 tags),no_turn_over}>` + continue_turn_advance
> 消费分支（位于 extra_turn_queue 之后；未列阶段以 SKIP_DRAW/PLAY/DISCARD 与
> pending_judge_skips 武装；Prepare 恒执行）；新原语
> `queue_extra_phases(player, phases_mask, self_skill)`（tag 695/ABI 672 领号；
> mask bit=1<<TurnPhaseTag）。消费者：offline zj_fuli（濒死 optional →
> recover_hp(2-hp)→draw 2→queue_extra_phases(Play)；limited→storage 一次性）。
> shiji tamo（座次重排盘）/ sm_kulian（轮度唯一最多竞赛判定）终态：登记债
> （归 §7 竞赛编排池，多批级）。
>
> | **diy nsjihui 集计晋升（2026-08-26 第九十五批）**：`cards_moved_discard_
> at_least_three` 过滤器(5处注册) + diy_meta 原位重写——每回合一次 storage 自锁
> + queue_extra_turn(check=0)；近似注记三条(目标限制子技省略/自锁收窄/确认噪声)。
> **twjieqiu 勘误**：完整语义含「按弃牌量选择恢复哪些栏」选择盘+跨角色 buff 名单
> +roundStart 门，判定为引擎级不可达，撤销第九十四批轻量化立项。
> | xianding guili(1) | 同左 | **已迁(第八十七批)**：choose_other_add_mark 绑定(GUILI_BIND=281977830 合成)+新过滤器 `owner_alive_event_target_mark_positive_<mark>_no_damage`+queue_extra_turn(check=0);近似=绑定以标记在场表达、整回合队列口径 |
> | **sp/sp2/mobile/onlyOL 筛迁（2026-08-26 第九十批）**：四包 8 处逐块核验,**零纯形,全部复合登记**——
> | sp olhuiqi(1) | 觉醒:本轮全局 useCard 目标集去重==3 且含己方;recover+addSkills+insertPhase | 需全局历史目标集合统计(①∪③) |
> | sp oldongdao(1) | 转换技 changeZhuanhuanji+target 插相 | 引擎级(转换态面) |
> | sp2 dccuichuan(1)/yijiao(1) | 空槽随机装牌 chooseUseTarget / chooseControl 加标 | 引擎级 |
> | mobile mbfengying(1)+xinlianpo(1) | 多源强制用牌 / 死亡装配派生技能白名单 else 插相 | 引擎级 |
> | onlyOL olduoqi(2) | 一号位 seatNum 判定+_status.olduoqi_record 全局数组+broadcast | 引擎级 |
>
> | **九单点包核验（2026-08-26 第九十一批）**：全部定级完毕，无纯形——
> | key umi_lunhui | — | **已迁（第八十四批）** |
> | extra lianpo | — | **已迁存证**（simayi 多路器） |
> | clan clankudu | recast 重铸原语+chooseCardTarget 双选+牌面差值数学 | 引擎级 |
> | bingshi mbquanchong | 同左 | **已迁(勘误·第九十三批)**：ported/bingshi/mb_chenzhi.skill 存 resolve_mbquanchong 多路器+effect 子技(phase_begin lose_hp 尾钩)；filter=discardable_hand_equip_once_per_round；上批误判因未查移植实态 |
> | huicui dczecai | addAdditionalSkills 叠加+getMax 判定插相 | 引擎级 |
> | shenhua fangquan2 | cost 弃一手牌+chooseTarget(notMe forced)→**target.insertPhase()** | **已迁（第九十二批）**：choose_others_insert_extra_turn(player, check, skill) ABI 671/tag 694/Cont 281 落地 |
> | shiji tamo | chooseToMove 全场座次重排 | 引擎级 |
> | sxrm sxrmmieguo | gainPlayerCard selectButton [1..3] 盘 | 引擎级 |
> | old oldmingjian | give 全手牌+cancel 回合+phaseList goto 跳转 | 引擎级 |
>
> ### §6. 相位裁剪原语族立项书（2026-08-26 第九十七批）
>
> 将 five 处 `phaseList/skip(name)/_noTurnOver` 消费者归并为单一引擎立项，
> 替代此前各处零散登记；核验证据同表格。
>
> **统一设计**：`GameState.turn.extra_phase_queue: Vec<(PlayerId, Vec<PhaseTag>, bool /*noTurnOver*/)>`
> （serde default；不影响既有包哈希），与既存 extra_turn_queue 同级编排：
> continue_turn_advance 在 pop extra_turn 之前优先弹出该队列，按给定
> phase 序列生成受限回合（仅列表内阶段依序执行，非全循环）。
>
> **DSL 入口**：`queue_extra_phases(player, mask, self_skill)` +
> `skip_next_named_phase(player, tag)`（后者承载 piracyE_zy 的 skip(name) 循环）。
> 前者 push (target, decoded list, false)；后者仅在目标下一回合生效单次跳过。
>
> **逐条终判论证**：
> | 消费者 | 所需 phaseList | 超出原语的残差 |
> |---|---|---|
> | offline_juedai zj_fuli | ["phaseUse"] + recoverTo/draw 前置 | recoverTo 可 change/recover 组合近似；主前置濒死 enable 需 chooseToUse 门 |
> | offline_piracyE_zy zylianji | 动态 list（=被 skip 后余下阶段）| 依赖 skip_next_named_phase 联动 |
> | xianding dcwumei | 动态重排+handler 全局历史注入 | 引擎级，不入首版 |
> | tw twjieqiu buff/end 双子技 | 全量禁用槽并存阶段的插相 | 终态：需跨角色 buff 名单设施（登记债，已并入装备栏编排池）|
> | extra_offline sm_kulian | ["phaseUse"]+_noTurnOver | 终态：引擎级不可达（轮度竞赛系统全局编排，第九十九批改判）|
>
> **里程碑**：S0 数据结构+queue 接线（领号预估 ABI 672 一枚）；S1 zj_fuli/sm_kulian
> 两个消费者接线；**S2 撤项（2026-08-28 第一百零一批）**：skip_remaining_phases
> 单原语的干净消费者仅 zylianji 1/3，且其本体需「分段互斥触发器家族（本阶段
> 用牌类别数 ≥1/≥2/≥3 三档）+ skip 剩余阶段 + 动态 phaseList 目标回合」三重
> 耦合，登记并入 §7 装备栏/竞赛编排池；**§6 相位裁剪原语族至此收口**（S0
> 队列基建 + S1 zj_fuli 消费者已交付，余项全部带证据归池）。
> | **diy/tw 筛迁（2026-08-26 第八十九批）** |
> | diy nsfuge(1) | storage 自锁一次性+纯 insertPhase | **已迁**（diy_meta 内晋升重写：queue_extra_turn(check=0)；diy ABI 478→669）；同文件 nspianwu(随机生成技能系统)/nsbaiyi(三张同类型判定分支)/nsjihui(currentPhase 记录+尾钩)复合登记 |
> | tw twrenxian(1) | give+定向 phaseList+cardEnabled 白名单限制 | 引擎级 |
> | tw twfuxi(1) | dying/disableEquipAfter choices 复合 | 引擎级 |
> | tw twjieqiu(1) | chooseToDiscard 链+roundStart temp | **已迁（第一百零二批）**：resolve_twjieqiu 多路器全链；恢复节奏即时等量恢复（104 批）|
> | tw twyingjia(1) | discard+target 联动 | 终态：登记债（discard+target 联动，多批级）|
>
> **第二消费者（2026-08-26 第八十五批）**：refresh/botu 博图（回合四花色齐→纯
> 额外回合）。复用 used_card_suit_history 位掩码（ABI≥232 opt-in 自动启用）+
> 新过滤器 `owner_alive_turn_used_all_suits`；offline 17 处深挖证实均为复合耦合
> （亡者回合绑定/技能互换/多源强制用牌），暂无纯形消费者。

共享底座：「猎」= 上游 mark `jsrgzhenglve_mark`（marktext 猎）。引擎侧具名标记
已完备：`add_mark/remove_mark(player_expr, def, delta)` 支持跨角色表达式目标，
参数化 mark 过滤器有先例（reluanzhan_add 同型）。

| 技能 | 可零/低原语部分 | 缺口（阻断完整保真） |
|---|---|---|
| jsrgzhenglve 政略 | ③「对有猎者造成伤害后摸1+获伤害牌」（damage ctx.cards + 参数化 mark 过滤器）；②以 `card_use_unlimited/distance_to` 近似无门槛豁免 | ①「一号位的回合结束」需座次过滤器；②的「有猎的目标」条件不可表达（target-mark 条件值源缺）；①的无伤害分支至多两名 |
| jsrgpingrong 平戎 | 移去任意角色一个“猎”（choose_other_add_mark 原语负 delta 或 remove_mark 表达式）、每轮一次 storage 门槛 | **核心承诺是额外回合**——引擎无 ExtraTurn 子系统（grep 零命中），「回合后插入己方整回合」为 ADR 级；失去体力惩罚依赖其上 |
| jsrghuilie 会猎 | 觉醒后架（change_max_hp(-1) + add_skill ×2，wuji 同模板齐备） | 觉醒条件「有‘猎’的角色数>2」需跨存活角色的标记计数表达式（现仅 per-target mark_count）；「获得平戎/飞影」跨技授予 add_skill 可用 |

**切片路线**：S1 零新原语近似批（政略③+会猎觉醒近似计数降级注记或新表达式二选一）
→ S2 座次过滤器（若 S1 选真实路线）→ S3 额外回君子系统单独领号立项。
达 3/3 消费者门槛：政略/平戎/会猎（凝寒之外的第四技 jsrghuilie 计数同源复用）。

## 2. 【城】扩张堆族（2 技，中水温）

"> **状态（2026-08-26 第八十批·S2b 已落地）**：沙城②消耗迁移——新原语
> `city_consume`（ABI 668 领号/tag 691/Cont 279 CityConsume）；「本回合失去牌数」
> 经 `SkillPileRef.wants_lost_count` opt-in 门控的 per-player turn-flag 计数实现
> （热路径仅在城族堆存在时计数，其他包零哈希漂移）。近似：目标域宽化为「任一
> 本回合失去过牌的其他角色」；城牌选择确定性取最后一张。沙城自此完整（除 UI 层）。

> **状态（2026-08-26 第七十九批·S2a 已落地）**：沙城筑城迁移完成——新原语
> `place_deck_top_expansion(player, count, self_skill)`（ABI 667 领号/tag 690），
> 扩张堆按需创建并以技能 def 键控（skill_piles 惯例）。凝寒整体留债：♣杀全局
> 改色与「冰冻」伤害 nature 均为引擎缺位。S2b=消耗侧（移城令目标摸X≤5：需
> per-player lose 计数 + 城牌/目标双选续体）。

| 技能 | 内容 | 缺口 |
|---|---|---|
| jsrgshacheng 沙城 | ①开局牌堆顶两张入武将牌上（addToExpansion）；②杀结算后移一张“城”令目标摸 X（X=本回合失去过的牌数≤5） | 扩张区置牌原语仅有手牌版（place_hand_card_to_expansion）；牌堆顶→expansion 需变体；“本回合失去过的牌数”历史查询无（turn flag 代理可行但计数口径要逐步验证）；移去指定的“城”需 expansion 选择续体 |
| jsrgninghan 凝寒 | ①所有角色♣【杀】视为冰【杀】；②冰冻伤害来源实体牌置入该角色的“城” | ♣花色 view_as 全局作用域：VIEW_AS 是技能自带转化而非全局改写，需要「per-color 视为」全局 modifier（0645 台账 poyue 同缺口已登记 1 例）；冰冻 nature 判定 + 来源实体牌捕获可行（damage ctx 有 card） |

**切片路线**：视§1 完成度再排；堆操作双变体（deck-top→expansion、指定移除）
可与 §3 共用扩张区选择续体设计。
>
> **凝寒终判（2026-08-28 第一百批）**：两半互锁，整体升级为引擎级不可达——
> ①「所有角色手牌中♣【杀】视为冰【杀】」需要**全局花色改写层**（effective
> suit/kind 查询面为 owner 局部，无全局改写挂点；per-color 全局 view-as 已在
> §6 台账挂账两例未达门槛）；②「冰冻伤害」在本引擎无产生源（DamageNature
> 仅有 normal/fire/thunder；且无任何已建模卡牌产生 ice 属性伤害）——②离开①
> 的冰冻判定即为死代码；「置入城」目标桶同时依赖 §2 沙城的扩张堆基建
> （fengying 式 action 多路器可承载），但触发源空缺使该半无意义。
> 状态由「中水温近似可迁」改判为「引擎级不可达（改色层+冰冻 nature 子系统）」，
> 归 §7 同池登记。

## 3. 【访客】武将牌族（4 技，深水区 / 高温）

成员：jsrgyingmen/jsrgpingjian 与 sbyingmen/sbpingjian 手杀变体。

机制本质（上游 storage 存**武将名**字符串数组）：从 `_status.characterlist`
随机抽武将置于武将牌上 → createCard(name,"","") 伪卡；pingjian 检测
「正在使用的技能是否为某访客武将也拥有的技能」（get.character(name).skills.includes）
并按命中访客发奖；sbpingjian 进一步把访客技能注入 invisibleSkills 并委托触发
（triggerInvisible 中继）。引擎现状：characterlist 抽样池、按武将名生成伪卡、
invisible-skill 注入均不存在。**评估：非单批可解；sb 变体的触发委托已达引擎级
新子系统**。处理：登记为永久高温债，除非出现 ≥2 个其他引用同一底座的机制族
再行立项；优先序排最后。

## 4. 优先序与纪律

1. **§1 猎族 S1**（**已落地**：第九十八批政略①③/会猎/平戎全链，见 §1 状态注记）
2. §1 S2 座次过滤器（零 ABI）→ §1 S3 额外回合（领号级）
3. §2 城族两变体（一套扩张区选择续体养两技即立项）
4. §3 访客族维持登记，不排期。
5. **§6 相位裁剪池终判合并**：dcwumei/twjieqiu 恢复类别选择盘两项终态并入本节，与 §7 装备栏编排子族合并追踪——单一入口「选择性状态快照/恢复设施」+「chooseButton-from-名单设施」双前置，领号与切片划分见 §6 S3 立项书。
5. **§6 相位裁剪原语族**（**收口**：S0 队列基建 + S1 zj_fuli 已落地；**S2 已落地（第一百一十九批）**：skip_remaining_phases(tag 697/ABI 674 领号) + 新过滤器 `owner_alive_play_end_used_categories_positive` + zylianji 连计近似接线；S3 dcwumei 领号级立项书已出具,终态登记债——与 §7 合并追踪,不再重复立项）。
6. **§8 gaintag 区系统立项评估（2026-08-28 第一百一十七批）**：gaintag 消费者
   跨 12+ 包（collab 单包 41 处那兔系+clan/diy/bingshi/jsrg/huicui/key/old/
   extra/newjiang/onlyOL/shenhua 零散）,**达 3/3 门槛**。能力分解:
   ①per-card 标签（content_card_tags 既有）②「hasCards with gaintag」查询
   ③组合判定（扑克对子/三条/顺子,缺,引擎级）。
   **状态（2026-08-28 第一百一十八批·S1 已落地）**：
   参数化过滤器 `owner_source_hand_gaintag_absent_<tag>`
   （OwnerSourceHandGaintagAbsent(u32),5 处注册,零领号）——持有者用牌时
   手牌不含该 gaintag 卡则触发。消费者 natuxing 兴门接线（unrespondable
   触发器从无条件收紧为 gaintag-absent 门控,兴 tag=blake2b("natuxing")=
   297116519）。测试 original_collab_natuxing_gate 2/2。S2 组合判定
   子系统维持单独评估。**§8 S1 收口**。
6. **§7 技能夺取/竞赛子系统池**（2026-08-28 第九十九批改判）：sm_kulian（全局竞赛系统：roundStart 奖池 initList/sm_mabian 装备联动/addGlobalSkill/多维度唯一最多）、potyinhui/potheyun（addAdditionalSkills+refreshSkill+chooseButton 名单）同池；**轮度唯一最多原语不立项**（干净消费者 1/3）；与访客族同为登记债，approx 池抽样证明无隐藏纯形晋升。**状态（2026-08-28 第一百四十三批·设施 S0 已落地；同日第一百四十四批·S1 已落地）**：
  S1=`lose_gained_skill`（tag 699/Cont 284，领号 ABI 676）+GameState.facility_gained_skills 获得登记，potyinhui 升级为完整上游语义（失去可取消臂+获得链），approx→完整。
  **S2（2026-08-28 第一百四十五批·已落地）**：`resolve_potheyun`（tag 700/Cont 285，领号 ABI 677）三段链（共享角色→共同技能→失去+目标摸2），potheyun 合韵完整建模，approx→完整。`choose_gain_other_skill`（tag 698/Cont 283，领号 ABI 675）两段选择链（选其他角色→选其可获得技能名单→AddSkill；名单过滤 locked/non_gainable）全链 15 处落地；消费者 potyinhui 音洄增益臂接线（bingshi npack ABI 675），行为测试 2/2。残差：音洄「失去已获得技能」臂（需获得技能追踪=设施 S1，与选择性状态快照同池）、potheyun 合韵（同技能过滤+失去技能）、sm_kulian 竞赛系统——维持登记债。
- 单技能半解锁不铸原语；效果原语全链 15 处 + ABI-COORDINATION 领号；
  每批完成后回写本章程状态列并在 delta 日志记账。
