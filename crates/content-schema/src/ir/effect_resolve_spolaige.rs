//! Ordered StmtIr variants, starting with `EffectResolveSpolaige`.

macro_rules! variants {
    ([$($tail:ident),*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail),*] [
            $($variants)*
    /// effect resolve_spolaige(player) — 霸业觉醒状态机 (批 293)：全局第二
    /// 次濒死结算后，失去〖西向〗获得〖朔北〗，摸/回复至体力上限。
    EffectResolveSpolaige {
        /// Awakening skill owner.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_xiangshu(player, target) — 襄戍状态机 (批 309)：限
    /// 定技，令一名受伤角色回复 X 点体力并摸 X 张牌（X=min(5, 本回合造成
    /// 伤害数)）。
    EffectResolveXiangshu {
        /// Skill owner.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_nhlundao(player, source) — 论道状态机 (批 308)：受
    /// 伤后若来源存活且手牌数不同，手牌多则摸 1，否则弃来源一张牌。
    EffectResolveNhlundao {
        /// Skill owner (damage target).
        player: ExprIr,
        /// Damage source.
        source: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_sanshou(player) — 三首状态机 (批 307)：亮牌堆顶三
    /// 张，若存在全新类型的牌则取消本次伤害。
    EffectResolveSanshou {
        /// Skill owner (damage target).
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_starcuxia_weiming(player, source, variant) — 粗遐/
    /// 未命孪生状态机 (批 305)：使用牌者随机弃 1 手牌，条件仍成立则
    /// owner 摸 1。
    EffectResolveStarcuxiaWeiming {
        /// Skill owner (the targeted player).
        player: ExprIr,
        /// The card user who must discard.
        source: ExprIr,
        /// 0 = starcuxia, 1 = starweiming.
        variant: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_zhongjie(player) — 张介状态机 (批 304)：死亡时选一
    /// 名其他角色，其 +1 体力上限、回复 1 体力、摸一张牌。
    EffectResolveZhongjie {
        /// Deceased skill owner (chooser).
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_dchaoxian(player) — 朝献状态机 (批 303)：限定技，
    /// 弃牌堆全部点数为 3 的牌洗入牌堆底，再收齐其他角色手牌中的 3 点牌。
    EffectResolveDchaoxian {
        /// Skill owner.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_dczhee(player, mode) — 闪电挂判定状态机 (批 302)：
    /// 从牌堆（入场/开局）或弃牌堆（结束阶段）检索【闪电】挂为判定牌。
    EffectResolveDczhee {
        /// Skill owner.
        player: ExprIr,
        /// 0 = draw pile, 1 = discard pile.
        mode: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_olqiaozhi(player, stage) — 巧织状态机 (批 296)：弃
    /// 一张牌，亮牌堆顶两张并获得其中一张（所得打「巧织」标记）。
    EffectResolveOlqiaozhi {
        /// Skill owner.
        player: ExprIr,
        /// Machine stage.
        stage: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_olsuyi(player, target) — 俗义状态机 (批 295)：出牌
    /// 阶段限一次，牌堆检索目标可用的装备牌令其装备，再对其视为使用杀。
    EffectResolveOlsuyi {
        /// Skill owner (virtual Sha user).
        player: ExprIr,
        /// Chosen target.
        target: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_olzhijie(player) — 直谏状态机 (批 294)：任意角色结束
    /// 阶段，选射程内一名角色对其造成 1 点伤害。
    EffectResolveOlzhijie {
        /// Skill owner (damage source).
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_olwenren(player) — 温仁状态机 (批 292)：出牌阶段限
    /// 一次，任选多名目标，每名目标无手牌摸 1、手牌数不大于 owner 再摸 1。
    EffectResolveOlwenren {
        /// Skill owner whose hand count is the comparison baseline.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_dingce(player, source) — 定策状态机 (批 281)：受到
    /// 伤害后弃自己一张手牌，再令伤害来源弃一张手牌（owner 代选）。
    EffectResolveDingce {
        /// Wounded player (skill owner).
        player: ExprIr,
        /// Damage source.
        source: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_meiyong(player) — 姝勇状态机 (批 277)：使用/打出
    /// 【杀】时获得一名其他角色 hej 区一张牌，然后其摸 1。
    EffectResolveMeiyong {
        /// Skill owner.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_mbzhenann(player) — 镇南状态机 (批 276)：成为锦囊
    /// 目标（目标数>max(1,使用者体力)）时弃一张牌并对任意角色造成1点伤害。
    EffectResolveMbzhenann {
        /// Skill owner.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_friendbiwei(player, target) — 鄙位状态机 (批 275)：
    /// 弃手牌中点数最大的一张，令目标弃置所有点数≥该牌的手牌。
    EffectResolveFriendbiwei {
        /// Skill owner.
        player: ExprIr,
        /// Chosen other player.
        target: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_chulao(player) — 黜老状态机 (批 274)：弃一张牌后对
    /// 所选势力各异角色逐个由 owner 选弃其一张牌，黑桃摸 1 双向结算。
    EffectResolveChulao {
        /// Skill owner.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_dcanxu(player) — 暗蓄状态机 (批 273)：所选两名手牌数
    /// 不同角色中，手牌少者从多者处选得一张手牌，附带花色/相等结算。
    EffectResolveDcanxu {
        /// Skill owner.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_gaoyuan(player, source, card) — 告援状态机
    /// (批 271)：弃一张牌将当前【杀】转移给一名有「诤」标的合法目标。
    EffectResolveGaoyuan {
        /// Original target redirecting the Sha (skill owner).
        player: ExprIr,
        /// Sha user.
        source: ExprIr,
        /// Physical Sha card.
        card: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_shuaiyan(player, self_skill, stage) — 率言状态机
    /// (批 266)。
    EffectResolveShuaiyan {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Machine stage (0 = entry/target selection, 2 = give pick).
        stage: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_reliangyin(player, self_skill, mode) — 良姻状态机
    /// (批 268)。
    EffectResolveReliangyin {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// 0 = placed (draw), 1 = taken (discard).
        mode: u32,
        /// Span.
        span: Span,
    },
    /// effect resolve_zhengjian(player, self_skill, stage, event_player) —
    /// 诤荐状态机 (批 267)。
    EffectResolveZhengjian {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Machine stage (0 = grant target selection, 1 = use tracking,
        /// 2 = respond tracking, 3 = turn-begin settlement).
        stage: ExprIr,
        /// Card user (tracking arms).
        event_player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_qinmian(player, self_skill, mode, amount) — 勤勉状态机
    /// (批 265)。
    EffectResolveQinmian {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// 0 = draw+tag, 1 = end phase settlement.
        mode: ExprIr,
        /// Draw count (prepare: 1; damage: event amount).
        amount: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_chuanyun(player, self_skill, event_player, event_card) —
    /// 穿云状态机 (批 258)。
    EffectResolveChuanyun {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Sha target.
        event_player: ExprIr,
        /// The sha card.
        event_card: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_jsrgzuozhan_gain(player, self_skill, event_player) — 坐瞻
    /// 组技补牌状态机 (批 243)。
    EffectResolveJsrgzuozhanGain {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Dying player bound to the death event.
        event_player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_jsrgdaoren(player, self_skill) — 专属状态机 (批 240)。
    EffectResolveJsrgdaoren {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_duwang(player, self_skill) — 专属状态机 (批 239)。
    EffectResolveDuwang {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_quanjia(player, self_skill, source, target) — 专属状态机
    /// (批 237)。
    EffectResolveQuanjia {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Damage source (viewed player).
        source: ExprIr,
        /// Damage target.
        target: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_pxqixian(player, self_skill, 0, 0) — 专属状态机 (批 236)。
    EffectResolvePxqixian {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Machine stage.
        stage: ExprIr,
        /// Chosen kind / material / target.
        kind: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_dcbingji(player, self_skill, 0, 0) — 专属状态机 (批 235)。
    EffectResolveDcbingji {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Machine stage.
        stage: ExprIr,
        /// Chosen kind.
        kind: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect choose_hand_cards_to_skill_pile(player, self_skill, min_count,
    /// max_count) — 变张置入 (A0 置入面, ADR 0641 §8.12)。ABI 691。
    EffectChooseHandCardsToSkillPile {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Minimum number of cards to place.
        min_count: ExprIr,
        /// Maximum number of cards to place.
        max_count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect place_hand_cards_to_deck_top(player, self_skill, count)
    /// — 堆重排 (chooseToMove family, ADR 0641 §8.8)。ABI 689。
    EffectPlaceHandCardsToDeckTop {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Number of cards to select.
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect gain_whole_skill_pile(player, self_skill, pile_def)
    /// — 整取技能堆 (expansion S1-B 深化, ADR 0641 §8.7)。ABI 687。
    EffectGainWholeSkillPile {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Skill definition owning the pile (u32 literal).
        pile_def: u32,
        /// Span.
        span: Span,
    },
    /// ABI 652: choose one other alive player and adjust one of their marks.
    /// ABI 657: discard any number of basic cards and draw equal.
    EffectDiscardBasicDrawEqual {
        /// Initiating player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 655: start a debate among all other players.
    EffectStartDebateAllOthers {
        /// Initiating player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 656: start a debate with selected targets.
    EffectStartDebateTargets {
        /// Initiating player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 658 (ADR 0644 P3): pin debate opinions (self two, others random).
    EffectPinDebateOpinions {
        /// Pinning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 659 (ADR 0644 P3): align opinions of players damaged by the owner.
    EffectAlignDebateOpinions {
        /// Aligning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 660 (jsrg 影族): gain N shadow cards into hand.
    EffectGainShadowCards {
        /// Gaining player.
        player: ExprIr,
        /// Count.
        count: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 661 (势力族): change a player's faction/group.
    EffectChangeGroup {
        /// Player.
        player: ExprIr,
        /// Target group name literal (wei/shu/wu/qun/jin).
        group: String,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 662 (jsrg funi②): distribute N shadow cards among chosen players.
    EffectDistributeShadowCards {
        /// Distributing player.
        player: ExprIr,
        /// Total count.
        count: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 663 (jsrg yaoyan): register turn-end debate participants.
    EffectRegisterTurnEndDebate {
        /// Initiator.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 663 (jsrg yaoyan): resolve the registered turn-end debate.
    EffectResolveTurnEndDebates {
        /// Initiator.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 664 (势力族动态): choose a faction and change to it.
    EffectChooseGroup {
        /// Player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 675 (§7 技能夺取设施 S0): gain one of another player's gainable skills.
    EffectChooseGainOtherSkill {
        /// Player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 676 (§7 技能夺取设施 S1): lose one facility-gained skill.
    EffectLoseGainedSkill {
        /// Player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 677 (§7 技能夺取设施 S2): lose a shared skill, target draws 2.
    EffectResolvePotheyun {
        /// Player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 665 (jsrg 蓄谋族): place a card as a scheme card in the judge zone.
    EffectPlaceXumou {
        /// Player.
        player: ExprIr,
        /// Card.
        card: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 666 (jsrg 殆谋): move the draw-pile top card into the judge zone
    /// as a xumou (scheme) card.
    EffectPlaceDeckTopXumou {
        /// Player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 666 (jsrg 遏前①): choose one hand card and move it into the judge
    /// zone as a xumou (scheme) card; cancel means stop scheming.
    EffectChoosePlaceHandXumou {
        /// Player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 666 (jsrg 殆谋「我」分支): deterministically discard one tagged
    /// xumou card from own judge zone.
    EffectDiscardOneOwnXumou {
        /// Player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },    /// ABI 667 (jsrg 沙城 build): move the draw-pile top `count` cards face-up
    /// into the owner's expansion pile keyed by this skill def.
    EffectPlaceDeckTopExpansion {
        /// Player.
        player: ExprIr,
        /// Card count (u32 literal).
        count: u32,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },    /// ABI 668 (jsrg 沙城②): remove one City card and make a chosen other
    /// player who lost cards this turn draw min(5, lost count) cards.
    EffectCityConsume {
        /// Player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },    /// ABI 669 (猎族 S3): queue one extra turn for the player; a post-turn
    /// check applies lose-1-HP when no damage was dealt during it.
    EffectQueueExtraTurn {
        /// Player.
        player: ExprIr,
        /// Post-turn lose-HP check enabled (u32 literal 0/1).
        check: u32,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },    /// ABI 671 (shenhua 放权): choose one other player and enqueue an extra
    /// turn for them; `check` non-zero arms the post-turn lose-HP marker.
    EffectChooseOthersInsertExtraTurn {
        /// Player.
        player: ExprIr,
        /// Check flag (u32 literal 0/1).
        check: u32,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },    /// ABI 672 (§6 相位裁剪): enqueue a cropped extra turn for the player
    /// limited to the phases whose bits are set in `phases_mask`
    /// (bit = 1 << TurnPhaseTag discriminant).
    EffectQueueExtraPhases {
        /// Player.
        player: ExprIr,
        /// Phase mask (u32 literal).
        phases_mask: u32,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },    /// ABI 673 (tw twjieqiu): multi-action — sub 0 disables every equipment
    /// slot of the chosen target (equipped cards to discard pile), draws k
    /// cards (k = pre-disable equipment count) and marks the target;
    /// sub 2 enqueues a plain extra turn for the owner while disabled slots
    /// remain.
    EffectResolveTwjieqiu {
        /// Player.
        player: ExprIr,
        /// Target (event player for sub 1/2).
        target: ExprIr,
        /// Sub-action (u32 literal 0/2).
        subaction: u32,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },    /// ABI 674 (§6 S2): skip the current turn's remaining phases whose bits
    /// are set in `phases_mask` (Draw/Play/Discard).
    EffectSkipRemainingPhases {
        /// Player.
        player: ExprIr,
        /// Phase mask (u32 literal).
        phases_mask: u32,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },    /// ABI 670 (jsrg 政略①): choose 1..max_count other living players who each
    /// gain `delta` stacks of the given mark.
    EffectChooseOthersAddMark {
        /// Player.
        player: ExprIr,
        /// Mark definition raw id (u32 literal).
        mark: u32,
        /// Per-target delta (integer literal).
        delta: i32,
        /// Max targets (u32 literal).
        max_count: u32,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 653: resolve the jsrgchaozheng debate.
    EffectResolveJsrgchaozheng {
        /// Initiating player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 654: resolve the jsrgsheju two-player debate.
    EffectResolveJsrgsheju {
        /// Initiating player.
        player: ExprIr,
        /// Debate counterpart.
        counterpart: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// ABI 652: choose one other alive player and adjust one of their marks.
    EffectChooseOtherAddMark {
        /// Choosing player.
        player: ExprIr,
        /// Mark definition raw id (literal).
        mark: u32,
        /// Delta (literal).
        delta: i32,
        /// Span.
        span: Span,
    },
        ] }
    };
}
pub(super) use variants;
