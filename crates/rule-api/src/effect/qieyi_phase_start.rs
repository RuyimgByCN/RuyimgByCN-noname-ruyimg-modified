//! Ordered EffectRequest variants, starting with `QieyiPhaseStart`.

macro_rules! variants {
    ([$($tail:ident),*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail),*] [
            $($variants)*
    /// 惬诣 play-phase start: reset the per-suit first-use storage and privately view
    /// the top two draw-pile cards.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    QieyiPhaseStart {
        /// Skill owner.
        player: PlayerId,
        /// 惬诣 skill instance.
        skill: SkillId,
    },
    /// 惬诣 card-used: on the first use of a suit this turn, reveal the top draw-pile
    /// card; same color or same kind as the used card → gain it, else choose one
    /// hand/equip card to put on top of the pile.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    QieyiUseAfter {
        /// Skill owner.
        player: PlayerId,
        /// The card that was used.
        card: CardId,
        /// 惬诣 skill instance.
        skill: SkillId,
    },
    /// 谏直 phase-end: when 惬诣 fired this turn, choose up to that many judgments;
    /// matching-suit judge cards are assigned to players and any non-matching judge
    /// card deals 1 thunder damage to the owner.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JianzhiPhaseEnd {
        /// Skill owner.
        player: PlayerId,
        /// 谏直 skill instance.
        skill: SkillId,
    },
    /// Use a suitless, source-free virtual 南蛮入侵 through the ordinary card pipeline.
    ///
    /// Unlike `ChooseUseVirtualCard`, this variant is phase-agnostic (no phase-end
    /// history suppression): 南蛮 resolves against its complete legal target set
    /// immediately. Appended so existing serialized effect variant ordinals remain stable.
    ChooseUseVirtualNanman {
        /// Virtual card user.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 起乱 phase-end: draw `Σ alive players' current-turn kill counts × (3 if the killer is
    /// the owner else 1)` — 何太后「起乱」. Computes the total and draws it for the owner.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ResolveQiluan {
        /// Skill owner who draws.
        player: PlayerId,
        /// 起乱 skill instance.
        skill: SkillId,
    },
    /// Discard every card currently in the player's judgment zone.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    DiscardAllOwnJudge {
        /// Player whose judgment zone is cleared.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Store the triggering event's used card (if it ended in the discard pile) onto the
    /// owner's skill pile (武将牌). Implements 灵杉 (sp/huamu).
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    StoreEventCardInSkillPile {
        /// Skill owner who stores the card.
        player: PlayerId,
        /// The triggering event's used card.
        card: CardId,
        /// Originating skill instance (defines the pile).
        skill: SkillId,
    },
    /// 临界 (sp/ollinjie): after the owner takes damage, the damage source is forced to
    /// discard one hand card; if the discarded card is effectively a Sha, the owner draws one.
    ///
    /// The engine rechecks the discard options when this effect starts and revalidates the
    /// selected card at the choice barrier; the conditional draw is scheduled with the
    /// discard in the same queue. Appended for serialized ordinal stability.
    ResolveOllinjie {
        /// Skill owner who draws if the discarded card is a Sha.
        player: PlayerId,
        /// Damage source forced to discard one hand card.
        source: PlayerId,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
    /// 寿春 (sp/pxshouchun): the owner selects any living player (self included);
    /// the selected player gains 1 max hp. Appended for serialized ordinal stability.
    ChooseTargetGainMaxHp {
        /// Choosing player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 肃州 (sp/pxsizhou): the owner selects another living player, then gains one
    /// of that player's hand cards (the owner picks the card). Two-stage choice:
    /// target selection, then card selection. Appended for serialized ordinal stability.
    GainTargetHandCard {
        /// Gaining player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 穆荫 (clan/clanmuyin): the owner selects a living player who is the owner
    /// themself or shares a clan with the owner and whose hand limit is NOT the
    /// field maximum; that player permanently gains +1 hand limit.
    /// Appended for serialized ordinal stability.
    ChooseClanNotMaxHandLimitGain {
        /// Choosing player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 额外结算 (clanfenshi family, upstream effectCount++): queue one extra
    /// effect pass for the currently suspended trick. Must run inside the
    /// CardUsed trigger window (WuxieWindow continuation).
    ExtraResolveCurrentTrick {
        /// Owning player.
        player: PlayerId,
        /// Suspended trick source card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 置入 (expansion P2, ADR 0641): the owner selects one hand card (any kind)
    /// and places it onto their skill pile (武将牌上). Reuses the
    /// CommitStoreHandShaInSkillPile pile-insertion machinery.
    PlaceHandCardToExpansion {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 取用 (expansion P1, ADR 0641): the owner selects one card from their own
    /// skill pile (武将牌上) and gains it into hand.
    GainFromSkillPile {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 弃置 (expansion S1-B, ADR 0641 §8.2): move every card of the owner's own
    /// skill pile (武将牌上) into the discard pile.
    DiscardSkillPile {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Skill definition owning the pile (u32 literal; may be a sibling skill).
        pile_def: u32,
    },
    /// 置入 (expansion P2-var, ADR 0641 §8.7): move the top `count` cards of the
    /// draw pile into the owner's own (player, pile_def) skill pile.
    PlaceTopDeckToExpansion {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Skill definition owning the pile (u32 literal; may be a sibling skill).
        pile_def: u32,
        /// Number of top deck cards to place.
        count: u32,
    },
    /// 选堆内单卡弃置 (expansion S1-B, ADR 0641 §8.7): pick one card from the
    /// owner's own (player, pile_def) skill pile and discard it.
    ChooseOneFromSkillPileDiscard {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Skill definition owning the pile.
        pile_def: u32,
    },
    /// 置入他人手牌 (expansion A1, ADR 0641 §8.7): pick another player with hand
    /// cards, then one of their hand cards, into the (player, pile_def) pile.
    ChooseTargetHandCardToPile {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Skill definition owning the pile.
        pile_def: u32,
    },
    /// 堆重排 (chooseToMove family, ADR 0641 §8.8): select `count` cards from the
    /// own (player, pile_def) skill pile and place them on top of the draw pile.
    RearrangePileToDeckTop {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Skill definition owning the pile.
        pile_def: u32,
        /// Number of cards to rearrange.
        count: u32,
    },
    /// 专属状态机 (批 240): jsrgdaoren 蹈仁 — give one hand card to another
    /// player, then deal 1 damage to every player in both attack ranges.
    ResolveJsrgdaoren {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 专属状态机 (批 241): qianlong 潜龙 — after damage, view the top three
    /// deck cards and gain up to lost-HP count, the rest go to deck bottom.
    ResolveQianlong {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 专属状态机 (批 242): jsrgzuozhan 坐瞻 — at game start choose up to two
    /// other players; gain an attack-range mark equal to min(highest HP, 5).
    ResolveJsrgzuozhan {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 专属状态机 (批 243): jsrgzuozhan gain group face — after a marked ally
    /// dies, one surviving 坐瞻 ally gains up to X distinct-name basic cards
    /// from the discard pile.
    ResolveJsrgzuozhanGain {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Dying player bound to the death event.
        event_player: PlayerId,
    },
    /// 专属状态机 (批 245): twmouzhi 谋制 — mode 0: prevent same-color
    /// consecutive damage; mode 1: record the damage card's color.
    ResolveTwmouzhi {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// 0 = damage_before check/prevent, 1 = damage record.
        mode: u32,
        /// Damage event card (0 if none).
        event_card: u32,
    },
    /// 专属状态机 (批 247): twdaigui 待归 slice 1 — uniform-color hand gate,
    /// choose up to hand-count players, reveal deck-bottom cards.
    ResolveTwdaigui {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Machine stage (0 = entry/target selection, 2 = reveal).
        stage: u32,
        /// Number of revealed cards (target count bound at stage 2).
        count: u32,
    },
    /// 专属状态机 (批 249): twmitu 迷途 slice 1 — prepare phase, choose up to
    /// three players; each draws two 迷途-tagged cards and shows them.
    ResolveTwmitu {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Machine stage (0 = entry/target selection).
        stage: u32,
        /// Reserved.
        count: u32,
    },
    /// 专属状态机 (批 252): qiantun 谦吞 — once per play phase, another player
    /// shows hand cards (their only compare cards); compare decides which cards
    /// the owner takes.
    ResolveQiantun {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 专属状态机 (批 253): yangming 养名 — cyclic compare with one player;
    /// target win draws prior losses and owner recovers, otherwise repeat.
    ResolveYangming {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 专属状态机 (批 255): canyun 残殒 — discard one card, pick 1-2 players,
    /// compare equipment counts for recover/draw/lose-HP outcomes.
    ResolveCanyun {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 专属状态机 (批 256): hengwu 横骛 — after using/playing a card whose
    /// suit is absent from hand, draw per matching field equipment.
    ResolveHengwu {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Used/responded card (0 if none).
        event_card: u32,
    },
    /// 专属状态机 (批 257): biaozhao3 表召·归 — prepare phase with a 表 card:
    /// discard it, then optionally have one player draw up to the field's
    /// largest hand count (max 5) and recover 1.
    ResolveBiaozhao3 {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 专属状态机 (批 258): chuanyun 穿云 — after your Sha targets a player
    /// with equipment, randomly discard one of their equipped cards.
    ResolveChuanyun {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Sha target.
        event_player: PlayerId,
        /// The sha card.
        event_card: u32,
    },
    /// 专属状态机 (批 259): biaozhao2 表召·应 — when a discard matches the
    /// 表 card's suit and rank, give the 表 card to the discarder; owner loses
    /// 1 HP.
    ResolveBiaozhao2 {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// The discarder.
        event_player: PlayerId,
    },
    /// 专属状态机 (批 260): dcniji 逆击 — mode 0: targeted by a non-equipment
    /// card, draw and tag; mode 1: end phase, discard all tagged cards.
    ResolveDcniji {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// 0 = targeted, 1 = end phase.
        mode: u32,
        /// The targeting card (mode 0).
        event_card: u32,
    },
    /// 专属状态机 (批 261): dcmoukui 谋溃 — after your Sha targets a player,
    /// choose one or both: draw a card, or discard one of the target's cards.
    ResolveDcmoukui {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Machine stage (0 = entry choice, 3 = discard pick).
        stage: u32,
        /// The sha target.
        target: PlayerId,
    },
    /// 专属状态机 (批 262): zhongyan 忠言 — once per play phase, reveal the
    /// top three deck cards; a chosen player exchanges one for a hand card.
    ResolveZhongyan {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Machine stage (0 = entry/target selection).
        stage: u32,
    },
    /// 专属状态机 (批 263): chumei 狐魅 — during the play phase, choose a
    /// player whose HP is at most the owner's damage dealt this turn; they
    /// draw a card, give a card, or recover 1 HP.
    ResolveChumei {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Machine stage (0 = entry/target selection, 2 = option choice).
        stage: u32,
    },
    /// 专属状态机 (批 266): shuaiyan 率言 — at the discard phase with more
    /// than one hand card, another player gives the owner one of their cards.
    ResolveShuaiyan {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Machine stage (0 = entry/target selection, 2 = give pick).
        stage: u32,
    },    /// 专属状态机 (批 287): twxiongjun 雄军 — after the owner deals damage,
    /// every living player owning 雄军 draws one card.
    ResolveTwxiongjun {
        /// Damage source (skill owner).
        player: PlayerId,
    },
    /// 专属状态机 (批 288): yixiang 义襄 — randomly gain a basic card from
    /// the deck that is not already in the owner\'s hand.
    ResolveYixiang {
        /// Skill owner.
        player: PlayerId,
    },
    /// 专属状态机 (批 291): xinfu_qiai 求援 — once per game, while dying,
    /// every other player in seat order hands the owner one hand/equip card.
    ResolveXinfuQiai {
        /// Dying skill owner receiving the cards.
        player: PlayerId,
    },
    /// 专属状态机 (批 293): spolaige 觉醒 — after the match's second dying
    /// window, exchange 西向 for 朔北 and draw/recover up to max HP.
    ResolveSpolaige {
        /// Awakening skill owner.
        player: PlayerId,
    },
    /// 专属状态机 (批 294): olzhijie 直谏 — at any player's end phase, deal
    /// one damage to a chosen player within the owner's attack range.
    ResolveOlzhijie {
        /// Skill owner (damage source).
        player: PlayerId,
    },
    /// 专属状态机 (批 295): olsuyi 俗义 — once per play phase, search the
    /// deck for equipment the target can use, have them equip it, then use
    /// a virtual Sha against them.
    ResolveOlsuyi {
        /// Skill owner (virtual Sha user).
        player: PlayerId,
        /// Chosen target who receives and uses the equipment.
        target: PlayerId,
    },
    /// 专属状态机 (批 296): olqiaozhi 巧织 — discard one card, reveal the
    /// top two deck cards, gain one of them with the weaving gaintag.
    ResolveOlqiaozhi {
        /// Skill owner.
        player: PlayerId,
        /// Machine stage (0 = entry discard, 1 = pick from the two revealed).
        stage: u32,
    },
    /// 专属状态机 (批 302): dczhee 拓跋? — search the draw pile (entry) or
    /// discard pile (end phase) for a Lightning and hang it as owner's judge.
    ResolveDczhee {
        /// Skill owner.
        player: PlayerId,
        /// 0 = search draw pile, 1 = search discard pile.
        mode: u32,
    },
    /// 专属状态机 (批 303): dchaoxian 朝献 — once per game: shuffle every
    /// rank-3 card from the discard pile back into the deck (bottom) and
    /// collect all rank-3 hand cards from other players.
    ResolveDchaoxian {
        /// Skill owner.
        player: PlayerId,
    },
    /// 专属状态机 (批 304): zhongjie 张介 — while dying owner dies, they
    /// pick another player who gains 1 max hp, recovers, and draws.
    ResolveZhongjie {
        /// Deceased skill owner (chooser).
        player: PlayerId,
    },
    /// 专属状态机 (批 307): sanshou 三首 — reveal the top three deck cards;
    /// cancel the incoming damage if any revealed type-group is new.
    ResolveSanshou {
        /// Skill owner (damage target).
        player: PlayerId,
    },
    /// 专属状态机 (批 308): nhlundao 论道 — after taking damage, if the
    /// source is alive with a different hand count: owner draws when their
    /// hand is larger, else discards one of the source's cards.
    ResolveNhlundao {
        /// Skill owner (damage target).
        player: PlayerId,
        /// Damage source.
        source: PlayerId,
    },
    /// 专属状态机 (批 309): xiangshu 襄戍 — once per game at the end phase:
    /// a wounded target recovers X and draws X (X = min(5, damage dealt
    /// this turn)).
    ResolveXiangshu {
        /// Skill owner.
        player: PlayerId,
        /// Selected wounded recipients.
        targets: Vec<PlayerId>,
    },
    /// 专属状态机 (批 305): starcuxia/starweiming 孪生 — the card user
    /// randomly discards a hand card; the owner draws if the dominance
    /// condition still holds.
    ResolveStarcuxiaWeiming {
        /// Skill owner (the targeted player).
        player: PlayerId,
        /// The card user who must discard.
        source: PlayerId,
        /// 0 = starcuxia branch, 1 = starweiming branch.
        variant: u32,
    },
    /// 专属状态机 (批 292): olwenren 温仁 — once per play phase, every
    /// selected target draws 1 if empty-handed and 1 more if their hand
    /// count is not greater than the owner\'s.
    ResolveOlwenren {
        /// Skill owner whose hand count is the baseline.
        player: PlayerId,
        /// Selected targets in seat order.
        targets: Vec<PlayerId>,
        /// Owning skill instance.
        skill: SkillId,
    },
    /// 专属状态机 (批 286): dcqiaolve 巧略 — discard one of your cards first,
    /// then take a random hand card from the event target.
    ResolveDcqiaolve {
        /// Skill owner.
        player: PlayerId,
        /// Event target.
        target: PlayerId,
    },
    /// 专属状态机 (批 281): dingce 定策 — after taking damage, discard one of
    /// your hand cards, then the damage source discards one (owner chooses
    /// which); the same-color virtual-card branch is deferred (card absent).
    ResolveDingce {
        /// Wounded player (skill owner).
        player: PlayerId,
        /// Damage source.
        source: PlayerId,
    },
    /// 专属状态机 (批 278): qiangyong 羌勇 — when your Sha deals damage,
    /// discard X cards from the wounded player (X = Sha uses this turn);
    /// if they then have no hand cards this damage is increased by one.
    ResolveQiangyong {
        /// Damage source (skill owner).
        player: PlayerId,
        /// Wounded player.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 专属状态机 (批 277): meiyong 姝勇 — when using/playing a Sha, take one
    /// card from another player\'s hand/equip/judge area, then they draw one.
    ResolveMeiyong {
        /// Skill owner (the Sha user/responder).
        player: PlayerId,
    },
    /// 专属状态机 (批 276): mbzhennann 镇南 — when targeted by a trick whose
    /// target count exceeds max(1, the user's hp), discard one card and deal
    /// 1 damage to any character.
    ResolveMbzhenann {
        /// Skill owner (a target of the trick).
        player: PlayerId,
    },
    /// 专属状态机 (批 275): friendbiwei 鄙位 — discard the highest-rank hand
    /// card, then the target discards every hand card of rank >= it; if none
    /// were discarded the skill use is refunded.
    ResolveFriendbiwei {
        /// Skill owner.
        player: PlayerId,
        /// Chosen other player.
        target: PlayerId,
        /// The discarded cost card (selected via active card selection).
        card: CardId,
        /// Originating skill instance (for the use-count refund).
        skill: SkillId,
    },
    /// 专属状态机 (批 274): chulao 黜老 — discard one card, then for each
    /// selected target of a distinct group the owner picks one of their cards
    /// to discard; spade discards draw (owner from cost, target from pick).
    ResolveChulao {
        /// Skill owner.
        player: PlayerId,
        /// Selected targets (distinct groups, at least one).
        targets: Vec<PlayerId>,
    },
    /// 专属状态机 (批 273): dcanxu 暗蓄 — of two selected players with
    /// differing hand counts, the lower gains one card from the higher;
    /// non-spade gain draws the owner one, equalized hands recover owner one.
    ResolveDcanxu {
        /// Skill owner.
        player: PlayerId,
        /// Selected targets (exactly two).
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 专属状态机 (批 272): duoduan 度断 — once per turn when targeted by a
    /// Sha, recast one card, then pick: user draws two and the Sha misses you,
    /// or the user discards a card and you cannot respond.
    ResolveDuoduan {
        /// Sha target holding the skill.
        player: PlayerId,
        /// Sha user.
        source: PlayerId,
        /// Physical Sha card.
        card: CardId,
        /// Originating skill instance (gates the deferred cancel).
        skill: SkillId,
    },
    /// 专属状态机 (批 271): gaoyuan 告援 — when targeted by a Sha, discard one
    /// card to redirect the Sha to a legal target holding a 诤 mark.
    ResolveGaoyuan {
        /// Original target redirecting the Sha.
        player: PlayerId,
        /// Sha user.
        source: PlayerId,
        /// Physical Sha card.
        card: CardId,
    },
    /// 引擎内部提交 (批 271)：告援转移落盘（弃牌尾随后执行）。
    /// Rule VM bytecode never emits this directly.
    CommitGaoyuanRedirect {
        /// Original target that paid the discard.
        player: PlayerId,
        /// Sha user.
        source: PlayerId,
        /// Physical Sha card.
        card: CardId,
        /// Replacement target.
        target: PlayerId,
    },
    /// 主动原语 (批 270): duansuo 断索 — reset every selected linked target,
    /// then deal 1 fire damage to each (two passes, upstream order).
    ResetLinkedSelectedTargets {
        /// Damage source (skill owner).
        source: PlayerId,
        /// Selected linked targets.
        targets: Vec<PlayerId>,
    },
    /// 专属状态机 (批 269): mbbeiming 孛明 — at game start choose up to two
    /// players; each gains a weapon whose range equals their hand suit count.
    ResolveMbbeiming {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Machine stage (0 = entry/target selection).
        stage: u32,
    },
    /// 专属状态机 (批 267): zhengjian 诤荐 — mode 0: end phase, grant a 诤
    /// mark to one player; mode 1/2: a marked player's card use/play bumps
    /// their mark; mode 3: turn begin, marked players draw and marks clear.
    ResolveZhengjian {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Machine stage.
        stage: u32,
        /// Card user (tracking arm).
        event_player: PlayerId,
    },
    /// 专属状态机 (批 268): reliangyin 良姻（mobile 重印版）— mode 0: cards
    /// moved into an expansion, one player draws; mode 1: cards moved out,
    /// one player discards.
    ResolveReliangyin {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// 0 = placed (draw), 1 = taken (discard).
        mode: u32,
    },
    /// 专属状态机 (批 265): qinmian 勤勉 — mode 0: draw amount cards tagged
    /// 勤勉; mode 1: end phase, discard all tagged cards to damage the current
    /// turn player.
    ResolveQinmian {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// 0 = draw+tag, 1 = end phase settlement.
        mode: u32,
        /// Draw count.
        amount: u32,
    },
    /// 专属状态机 (批 239): duwang 度望 — at game start, place the first five
    /// non-Sha cards from the deck top onto the own skill pile.
    ResolveDuwang {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 专属状态机 (批 237): quanjia 拳甲 — randomly view (2+marks) cards of the
    /// damage source's hand; use a virtual Sha on them if any, else gain a mark.
    ResolveQuanjia {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Damage source (viewed player).
        source: PlayerId,
        /// Damage target.
        target: PlayerId,
    },
    /// 专属状态机 (批 236): pxqixian 祁县 — recast one equip card as a virtual
    /// Sha or Guohe used on another player.
    ResolvePxqixian {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Machine stage.
        stage: u32,
        /// Chosen kind / material / target depending on stage.
        kind: u32,
    },
    /// 专属状态机 (批 235): dcbingji 秉纪 authoritative machine.
    /// stage 0: validate + choose sha/tao; stage 1: pick target (sha) / self-use (tao);
    /// stage 2: materialize + play + record suit.
    ResolveDcbingji {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Machine stage.
        stage: u32,
        /// Chosen kind (1 = sha, 2 = tao); later stages carry the uniform suit tag.
        kind: u32,
    },
    /// 变张置入 (A0 置入面, ADR 0641 §8.12): select between `min_count` and
    /// `max_count` cards from the own hand and place them into the own
    /// originating skill pile.
    ChooseHandCardsToSkillPile {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance (pile def source).
        skill: SkillId,
        /// Minimum number of cards to place.
        min_count: u32,
        /// Maximum number of cards to place.
        max_count: u32,
    },
    /// 堆重排 (chooseToMove family, ADR 0641 §8.8): select `count` cards from the
    /// player's hand and place them on top of the draw pile (in selection order).
    PlaceHandCardsToDeckTop {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Number of cards to select.
        count: u32,
    },
    /// 整取 (expansion S1-B 深化, ADR 0641 §8.7): move every card of the owner's
    /// own (player, pile_def) skill pile into the owner's hand.
    GainWholeSkillPile {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Skill definition owning the pile.
        pile_def: u32,
    },
    /// 朝争议事结算（ABI 653，ADR 0644 P1）：全参与者亮首张手牌分组多数决。
    ResolveJsrgchaozheng {
        /// 发起者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 设居双人议事结算（ABI 654，ADR 0644 P1）：黑多数则双方各减 1 体力上限。
    ResolveJsrgsheju {
        /// 发动者。
        player: PlayerId,
        /// 议事对方。
        counterpart: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 玉蝉引擎（ABI 657）：弃任意张基本手牌并摸等量（选择续体）。
    DiscardBasicDrawEqual {
        /// 发动者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 发起议事·全员（ABI 655，ADR 0644 P2）：只发 DebateResolved 事件。
    StartDebateAllOthers {
        /// 发起者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 发起议事·指定目标（ABI 656，ADR 0644 P2）：选 1..N 其他角色后发事件。
    StartDebateTargets {
        /// 发起者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 议事意见钉死（ABI 658，ADR 0644 P3）：自己选 2 张手牌为双意见，
    /// 其他参与者意见随机钉死一张（debateShowOpinion 时机）。
    PinDebateOpinions {
        /// 参与者（钉死发起方）。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 议事意见对齐（ABI 659，ADR 0644 P3）：本回合受到过 player 伤害的
    /// 参与者意见对齐为 player 的意见。
    AlignDebateOpinions {
        /// 对齐发起者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 获得【影】（ABI 660，jsrg 影族）：生成 count 张影牌（黑桃 1 虚拟
    /// 基本牌）置入手牌。
    GainShadowCards {
        /// 获得者。
        player: PlayerId,
        /// 张数。
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 变更势力（ABI 661，jsrg/tw 势力族）：将 player 势力覆盖为指定势力。
    ChangeGroup {
        /// 变更者。
        player: PlayerId,
        /// 目标势力名（如 "wei"/"shu"/"qun"）。
        group: String,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 分发【影】（ABI 662，jsrg funi②）：选 1..N 角色共计 count 张影均分给予。
    DistributeShadowCards {
        /// 分发者。
        player: PlayerId,
        /// 总张数。
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 登记回合末议事（ABI 663，jsrg yaoyan）：逐角色选择后登记为发起者的
    /// 回合末议事参与者。
    RegisterTurnEndDebate {
        /// 发起者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 选择势力并变更（ABI 664，势力族动态）：选现存在且不同于自己的势力，
    /// 写入 group_overrides（chijie/延祚/wuchang 动态势力）。
    ChooseGroup {
        /// 变更者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// §7 技能夺取设施 S0（ABI 675）：选一名其他存活角色，再从其可获得技能
    /// 名单中获得一个（potyinhui 音洄增益臂）。
    ChooseGainOtherSkill {
        /// 获得者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// §7 技能夺取设施 S1（ABI 676）：从本设施获得清单中移除一个技能
    /// （potyinhui 音洄失去臂）。
    LoseGainedSkill {
        /// 移除者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// §7 技能夺取设施 S2（ABI 677）：选共享技能的角色，失去共同技能，
    /// 目标摸 2（potheyun 合韵）。
    ResolvePotheyun {
        /// 发动者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 蓄谋（ABI 665，jsrg 蓄谋族）：将一张牌作为蓄谋牌置入判定区
    /// （特殊延时虚拟牌，可重复）。
    PlaceXumou {
        /// 蓄谋者。
        player: PlayerId,
        /// 置入的实体牌。
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 回合末议事结算（ABI 663）：对登记名单发起议事并收尾。
    ResolveTurnEndDebates {
        /// 发起者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Select one other alive player and adjust one of their marks (ABI 652).
    ChooseOtherAddMark {
        /// Choosing player.
        player: PlayerId,
        /// Mark definition raw id.
        mark: u32,
        /// Delta (positive or negative).
        delta: i32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 蓄谋·牌堆顶（ABI 666，jsrg 殆谋）：将牌堆顶实体牌移入判定区作为蓄谋牌
    /// （上游 addJudge({name:"xumou_jsrg"}, get.cards())）。
    PlaceDeckTopXumou {
        /// 蓄谋者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 蓄谋·选牌（ABI 666，jsrg 遏前①）：选一张手牌移入判定区作为蓄谋牌
    /// （cancel = 停止蓄谋）。
    ChoosePlaceHandXumou {
        /// 蓄谋者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 弃置一张蓄谋牌（ABI 666，jsrg 殆谋「我」分支）：确定性弃置判定区
    /// 最后一张蓄谋牌（无 tagged 牌则静默 no-op）。
    DiscardOneOwnXumou {
        /// 弃置者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 筑城·堆顶置入扩张区（ABI 667，jsrg 沙城 build 子技）：将牌堆顶 count 张
    /// 实体牌明置移入自己的「城」扩张堆。
    PlaceDeckTopExpansion {
        /// 筑城者。
        player: PlayerId,
        /// 置入张数（常量）。
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 城族消耗（ABI 668，jsrg 沙城②）：移去一张「城」并令一名本回合失去过
    /// 牌的其他角色摸 min(5, 失去牌数) 张。
    CityConsume {
        /// 发动者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 额外回合（ABI 669，猎族 S3）：把玩家排入引擎 extra_turn_queue，
    /// 并登记尾随 check（额外回合内未造成伤害则失去 1 点体力）。
    QueueExtraTurn {
        /// 获得额外回合的玩家。
        player: PlayerId,
        /// 非零：登记尾随失去体力 check（平戎）；零：纯额外回合（海未轮回）。
        check: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 目标插相（ABI 671，shenhua 放权）：选择一名其他角色，将其排入
    /// extra_turn_queue；check 非零时同时登记尾随失去体力 check 标记。
    ChooseOthersInsertExtraTurn {
        /// 发动者。
        player: PlayerId,
        /// 非零：为目标额外回合登记尾随 check。
        check: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 相位裁剪额外回合（ABI 672，§6 相位裁剪原语族）：把玩家排入裁剪队列，
    /// 仅执行 phases_mask 中置位的阶段（bit = 1 << TurnPhaseTag 判别值）。
    QueueExtraPhases {
        /// 获得裁剪回合的玩家。
        player: PlayerId,
        /// 阶段位掩码（bit = 1 << TurnPhaseTag 判别值）。
        phases_mask: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 跳过本回合剩余阶段（ABI 674，§6 S2）：对 mask 置位的 Draw/Play/Discard
    /// 阶段置 SKIP 标志（仅本回合生效）。
    SkipRemainingPhases {
        /// 跳过者（须为当前回合角色）。
        player: PlayerId,
        /// 阶段位掩码（bit = 1 << TurnPhaseTag 判别值）。
        phases_mask: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 劫囚·装备栏编排（ABI 673，tw twjieqiu）：多路器——
    /// sub 0：选择一名其他角色，禁用其全部装备槽（现存装备入弃牌堆）并令其
    ///      摸 k 张（k=废除前装备区牌数），打上 BUFF 标记；
    /// sub 2：若目标仍有禁用槽，为持有者入队一个纯额外回合。
    ResolveTwjieqiu {
        /// 技能持有者。
        player: PlayerId,
        /// 被劫囚目标（sub 0 时忽略）。
        target: PlayerId,
        /// 子动作（0=废除+摸k，2=余栏判额外回合）。
        subaction: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 多目标授标（ABI 670，jsrg 政略①「至多两名」）：选择 1..max_count 名其他
    /// 存活角色，各获得 delta 点给定标记（cancel = 放弃授标）。
    ChooseOthersAddMark {
        /// 选择者（授标者）。
        player: PlayerId,
        /// Mark definition raw id.
        mark: u32,
        /// Delta per target.
        delta: i32,
        /// Max selectable targets.
        max_count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
}
pub(super) use variants;
