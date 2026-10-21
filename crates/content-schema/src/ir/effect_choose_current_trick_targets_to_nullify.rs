//! Ordered StmtIr variants, starting with `EffectChooseCurrentTrickTargetsToNullify`.

macro_rules! variants {
    ([$($tail:ident),*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail),*] [
            $($variants)*
    /// Choose one or more targets of the currently suspended non-multitarget trick and make the
    /// card ineffective for them.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseCurrentTrickTargetsToNullify {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Current card user.
        source: ExprIr,
        /// Current physical card carrier.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect judge_black_gain_repeat_protect_hand_limit(player)
    ///
    /// Appended for serialized ordinal stability.
    EffectJudgeBlackGainRepeatProtectHandLimit {
        /// Judged player and recipient of black results.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Give all physical Sha and Shan sources from the resolved Sha, then apply color rewards.
    ///
    /// Appended for serialized ordinal stability.
    EffectChooseGainAllResolvedShaShanThenColorRewards {
        /// Skill owner and original Sha user.
        player: ExprIr,
        /// Physical Sha carrier bound to the event.
        sha_card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Gain one hand card from each selected player and reduce the current draw by that count.
    ///
    /// Appended for serialized ordinal stability.
    EffectChooseOtherHandTargetsGainOneReduceDraw {
        /// Current Draw-phase player and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Discard one current Sha target card; non-basic cards make the target unrespondable and
    /// add one turn-scoped Sha-use bonus, while basic cards give the target the physical Sha.
    ///
    /// Appended for serialized ordinal stability.
    EffectChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
        /// Current Sha user and card chooser.
        source: ExprIr,
        /// Physical Sha carrier bound to the event.
        card: ExprIr,
        /// Current Sha target.
        target: ExprIr,
        /// Definition of the turn-scoped modifier skill and its matching mark.
        bonus_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve 趫猛's current-card target choice, target-card discard, and physical-type branch.
    ///
    /// Appended for serialized ordinal stability.
    EffectResolveDcqiaomeng {
        /// Current black-card user and skill owner.
        player: ExprIr,
        /// Physical card carrier bound to the event.
        card: ExprIr,
        /// Exact-card policy helper used by the trick-card branch.
        helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve 界铁骑's judgment, turn-scoped non-locked-skill suppression, and same-suit gift.
    ///
    /// Appended for serialized ordinal stability.
    EffectResolveRetieji {
        /// Current Sha user and skill owner.
        player: ExprIr,
        /// Physical Sha carrier bound to the event.
        card: ExprIr,
        /// Current Sha target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 界许褚「裸衣」's top-three reveal and optional draw replacement.
    ///
    /// Appended for serialized ordinal stability.
    EffectResolveNewReluoyi {
        /// Current Draw-phase player.
        player: ExprIr,
        /// Definition of the temporary Sha/Duel damage helper.
        buff_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve OL Huang Zhong's locked 没矢 placement or retrieval branch.
    ///
    /// Appended for serialized ordinal stability.
    EffectResolveRemoshi {
        /// Skill owner.
        player: ExprIr,
        /// Damaged player or player who lost equipment.
        target: ExprIr,
        /// Current physical Sha carrier for placement; NONE for retrieval.
        card: ExprIr,
        /// Presentation helper attached to a player holding 没矢 cards.
        helper_skill_def: u32,
        /// 0=place the physical Sha, 1=retrieve the target's 没矢 pile.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// Resolve 界全琮「邀名」's per-turn target and action choices.
    ///
    /// Appended for serialized ordinal stability.
    EffectResolveXinyaoming {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 界公孙渊「怀异」's reveal, color, target, and card choices.
    ///
    /// Appended for serialized ordinal stability.
    EffectResolveRehuaiyi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 界朱治「安国」 using the shared minimum-metric flow plus optional recast.
    ///
    /// Appended for serialized ordinal stability.
    EffectResolveReanguo {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Let the bound damage target discard `max(1, 4-distance)` hand cards; cancellation or
    /// insufficient cards increases that damage by one.
    ///
    /// Appended for serialized ordinal stability.
    EffectChooseTargetDiscardDistanceHandOrIncreaseDamage {
        /// Damage source and skill owner.
        player: ExprIr,
        /// Bound damage target and discard chooser.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose to draw one card, privately inspect up to two sampled hand cards from the ending
    /// player and gain one, or move one compatible equipment card.
    ///
    /// Appended for serialized ordinal stability.
    EffectChooseDrawViewTwoGainOrMoveEquipment {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Other player whose turn ended.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 备诛's private hand reveal, forced discard/gift branch, or ordered physical Sha
    /// queue.
    ///
    /// Appended for serialized ordinal stability.
    EffectResolveBeizhu {
        /// Skill owner.
        player: ExprIr,
        /// Selected other player.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve mobile 曹纯「缮甲」's draw, dynamic discard, and category rewards.
    ///
    /// Appended for serialized ordinal stability.
    EffectResolveMbshanjia {
        /// Skill owner.
        player: ExprIr,
        /// Number of equipment-zone cards lost so far this match.
        lost_equipment_count: ExprIr,
        /// Temporary distance-unlimited helper definition.
        distance_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve mobile 胡车儿「盗戟」's equipment gain, forced use, and weapon damage.
    ///
    /// Appended for serialized ordinal stability.
    EffectResolveDaoji {
        /// Skill owner.
        player: ExprIr,
        /// Selected other player.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve mobile 李遗「教化」's persistent category cycle and recipient choice.
    ///
    /// Appended for serialized ordinal stability.
    EffectResolveJiaohua {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve mobile 来敏「乱群」's private reveal and temporary restrictions.
    ///
    /// Appended for serialized ordinal stability.
    EffectResolveLuanqun {
        /// Skill owner.
        player: ExprIr,
        /// First-Sha target restriction helper definition.
        effect_skill_def: u32,
        /// Turn-long unrespondable helper definition.
        direct_hit_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect choose_give_two_different_colors_or_recover(owner, giver, target)
    ///
    /// Let `giver` optionally give exactly two current hand/equipment cards with different
    /// effective colors to `target`; if no valid pair is given, recover `target` by one HP.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseGiveTwoDifferentColorsOrRecover {
        /// Originating skill owner.
        owner: ExprIr,
        /// Player making the card choice and giving the cards.
        giver: ExprIr,
        /// Gift recipient or recovery target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_mbzhuguo(player, target)
    ///
    /// Resolve mobile 吴珂「助国」's capped hand adjustment, recovery branch, and optional
    /// distance/use-count-free physical Sha.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveMbzhuguo {
        /// Skill owner.
        player: ExprIr,
        /// Selected living hand-adjustment target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_mbanxian(player)
    ///
    /// Choose one physical Sha from the owner's first hand-discard event this turn, gain it,
    /// then use it when legal without ordinary distance or use-count limits.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveMbanxian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_mbjinzu(player, target, helper_skill_def)
    ///
    /// Resolve mobile 曹性「劲镞」's simultaneous hand-card reveal, extrema branches,
    /// active-use refresh, and turn-bound next-Sha helper.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveMbjinzu {
        /// Skill owner.
        player: ExprIr,
        /// Selected other living player.
        target: ExprIr,
        /// Hidden helper definition carrying target-specific next-Sha bonuses.
        helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect apply_mbjinzu_next_sha(player, card)
    ///
    /// Consume the hidden helper and mutate the currently suspended next Sha for matching
    /// targets.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectApplyMbjinzuNextSha {
        /// Sha user and helper owner.
        player: ExprIr,
        /// Currently used physical or view-as Sha source card.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect take_one_card(from, to, hand_equip_judge)
    ///
    /// Appended so the legacy hand/equipment-only IR shape remains stable.
    EffectTakeOneScopedCard {
        /// Player who owns the offered cards.
        from: ExprIr,
        /// Player choosing and gaining one card.
        to: ExprIr,
        /// Canonical source zones.
        scope: TargetCardScopeIr,
        /// Source span.
        span: Span,
    },
    /// effect take_one_event_target_hand(player)
    ///
    /// Move one of the event target's hand cards to the owner's hand.
    EffectTakeOneHandCard {
        /// Skill owner receiving the card.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_other_hand_store_skill_pile(player, target)
    ///
    /// Choose one of target's hand cards and place it into the owner's skill pile (ABI 567).
    EffectChooseOtherHandStoreSkillPile {
        /// Skill owner whose skill pile receives the card.
        player: ExprIr,
        /// Player whose hand card is chosen.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect gain_all_skill_pile_cards(player)
    ///
    /// Move all cards of the owner's skill pile into hand (ABI 569).
    EffectGainAllSkillPileCards {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect gain_other_skill_pile_cards(player, target, skill_def)
    ///
    /// Move all cards of `target`'s skill pile keyed by `skill_def` into `player`'s hand (ABI 600, 英逝 yingshi_die).
    EffectGainOtherSkillPileCards {
        /// Player whose hand receives the pile cards.
        player: ExprIr,
        /// Player whose skill pile is emptied.
        target: ExprIr,
        /// Skill definition id keying the source skill pile (u32 literal).
        skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_one_he_or_lose_hp(player, lose_amount)
    ///
    /// Choose to discard one hand/equipment card, or lose `lose_amount` HP (ABI 601, 庸肆 xinyongsi2).
    EffectChooseDiscardOneHeOrLoseHp {
        /// Player making the discard-or-lose choice.
        player: ExprIr,
        /// HP lost when declining to discard (u32 literal).
        lose_amount: u32,
        /// Source span.
        span: Span,
    },
    /// effect judge_spade_two_to_nine_draw_two_or_thunder_sha_twice(source, target)
    ///
    /// Target judges: spade 2-9 draws two cards, otherwise source uses two thunder Sha on them (ABI 602, 雷劫 releijie).
    EffectJudgeSpadeTwoToNineDrawTwoOrThunderShaTwice {
        /// Skill owner who uses the virtual thunder Sha.
        source: ExprIr,
        /// Target who is judged, draws, or receives the thunder Sha.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_source_confirm_owner_lose_hp(owner, source, amount)
    ///
    /// The death source confirms whether the owner loses `amount` HP (ABI 603, 随势 suishi2).
    EffectChooseSourceConfirmOwnerLoseHp {
        /// Owner who loses HP on confirmation.
        owner: ExprIr,
        /// Player who confirms the HP loss.
        source: ExprIr,
        /// HP lost on confirmation (u32 literal).
        amount: u32,
        /// Source span.
        span: Span,
    },
    /// effect reveal_hand(player)
    ///
    /// Publicly reveal the player's complete current hand.
    /// Appended for serialized ordinal stability.
    EffectRevealHand {
        /// Player whose hand is revealed.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect draw_linked_players(count)
    ///
    /// Draw for every currently linked living player in seat order.
    /// Appended for serialized ordinal stability.
    EffectDrawLinkedPlayers {
        /// Cards drawn by each linked player.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect reveal_top_gain_if_color(player, red|black)
    ///
    /// Publicly reveal the draw-pile top card, then gain it only when its printed color matches.
    /// Appended for serialized ordinal stability.
    EffectRevealTopGainIfColor {
        /// Player revealing and possibly gaining the card.
        player: ExprIr,
        /// Printed-color mask: `1` red or `2` black.
        color_mask: u8,
        /// Source span.
        span: Span,
    },
    /// effect spjungong_discard_or_lose_then_sha(player, target, skill)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectSpjungongDiscardOrLoseThenSha {
        /// 峻攻 owner; discarder and virtual Sha user.
        player: ExprIr,
        /// 峻攻 Sha target.
        target: ExprIr,
        /// Originating active skill instance.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect ban_active_until_turn_end(player, skill)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectBanActiveUntilTurnEnd {
        /// Skill owner.
        player: ExprIr,
        /// Active skill instance banned until the current turn ends.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_tw_daoji(player, target)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveTwDaoji {
        /// Skill owner.
        player: ExprIr,
        /// Selected other player.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect spwanwei_active(player, target)
    EffectSpwanweiActive {
        /// Skill owner.
        player: ExprIr,
        /// Target player.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect spwanwei_dying(player, target)
    EffectSpwanweiDying {
        /// Skill owner.
        player: ExprIr,
        /// Target player.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect give_all_hand_then_choose_use_one_or_damage(player, target)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGiveAllHandThenChooseUseOneOrDamage {
        /// Original hand owner.
        player: ExprIr,
        /// Gift recipient.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect suppress_non_locked_skills_until_turn_end(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectSuppressNonLockedSkillsUntilTurnEnd {
        /// Player whose non-locked skills are suppressed.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_recover_target(chooser, target, amount)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseRecoverTarget {
        /// Player deciding whether to recover the target.
        chooser: ExprIr,
        /// Recovery target.
        target: ExprIr,
        /// Recovery amount.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_category_then_target_give_matching_or_sha_unrespondable(player, card, target)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
        /// Current Sha user and skill owner.
        player: ExprIr,
        /// Current physical Sha.
        card: ExprIr,
        /// Current Sha target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_lose_hp_use_virtual_sha_by_basic_variants(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseLoseHpUseVirtualShaByBasicVariants {
        /// Skill owner and virtual Sha user.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spxizhan(player, phase_player, fangzong_skill_def)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpxizhan {
        /// Skill owner.
        player: ExprIr,
        /// Current-turn player.
        phase_player: ExprIr,
        /// 放纵 skill definition id.
        fangzong_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twfupan(player, damage_amount)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveTwfupan {
        /// Skill owner.
        player: ExprIr,
        /// Trigger damage amount.
        damage_amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spsongshu(player, phase_player, renku_skill_def, block_skill_def)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpsongshu {
        /// Skill owner.
        player: ExprIr,
        /// Draw-phase player.
        phase_player: ExprIr,
        /// Renku skill definition id.
        renku_skill_def: u32,
        /// Block skill definition id.
        block_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spcunsi(player, target)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpcunsi {
        /// Skill owner.
        player: ExprIr,
        /// Sha recipient.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_yuli(player, mode)
    ///
    /// ABI 409: mode 0 = own damage source (non-thunder → thunder replay, thunder → +1);
    /// mode 1 = own damage target (prevent and draw the damage amount).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveYuli {
        /// Skill owner.
        player: ExprIr,
        /// 0 = source branch, 1 = target branch.
        mode: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_tingwei(player, mode)
    ///
    /// ABI 410: after owner's sha names its first target, add 4 thunder marks, pick a
    /// target from the sha's targets (mode 0 opens the choice); the chosen target then
    /// picks one of four options or gets chained (mode 1 resolves the option).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveTingwei {
        /// Skill owner.
        player: ExprIr,
        /// 0 = select target stage, 1 = resolve target's option stage.
        mode: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xinmingshi(player, source)
    ///
    /// ABI 411: after the owner takes damage while holding the 谦 mark, the damage
    /// source must discard one hand/equip/judge card; a red discard heals the owner
    /// by one, a discard entering the discard pile is gained by the owner.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveXinmingshi {
        /// Skill owner.
        player: ExprIr,
        /// Damage source.
        source: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xunyi(player, mode)
    ///
    /// ABI 412: mode 0 selects the 义 target (gets the mark); mode 1 resolves the
    /// damage event — either member damaging draws one for the other, either member
    /// damaged discards one hand/equip for the other.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveXunyi {
        /// Skill owner.
        player: ExprIr,
        /// 0 = select target, 1 = damage event.
        mode: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_boming(player, target)
    ///
    /// ABI 413: give the active invocation's selected hand/equip card to the target
    /// (emits CardsGiven).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveBoming {
        /// Skill owner.
        player: ExprIr,
        /// Gift recipient.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_ejian(player, target)
    ///
    /// ABI 414: after the owner gifts a card, the recipient with another card of the
    /// same category picks: take 1 damage, or reveal hand and discard all same-category
    /// cards.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveEjian {
        /// Skill owner.
        player: ExprIr,
        /// Gift recipient.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_yiyong(player)
    ///
    /// ABI 415: after the owner takes Sha damage, gain the Sha's physical cards from
    /// the discard pile; if the source lives and the owner can Sha it (ignoring
    /// distance), counter with those cards (base damage 2 if the source lacks a weapon).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveYiyong {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_shanxie(player)
    ///
    /// ABI 416: play-phase once per turn — gain a weapon from the draw pile, or
    /// randomly take one from another player with a weapon.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveShanxie {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_sheyi(player, target)
    ///
    /// ABI 417: before another player takes damage (once per round), give at least
    /// max(1, hp) hand/equip cards to prevent it.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSheyi {
        /// Skill owner.
        player: ExprIr,
        /// Damage target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_tianyin(player)
    ///
    /// ABI 418: at end phase, gain the topmost draw-pile card of each category not
    /// yet used this phase.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveTianyin {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dbzhuifeng(player)
    ///
    /// ABI 419: lose 1 HP and use a virtual Juedou against one other player.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDbzhuifeng {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dbquedi(player, target, card)
    ///
    /// ABI 420: after the owner's single-target Sha/Juedou names its target, the
    /// owner picks: gain one target hand card / discard one basic to boost damage+1 /
    /// backwater (lose 1 max HP and do everything).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDbquedi {
        /// Skill owner.
        player: ExprIr,
        /// The single target.
        target: ExprIr,
        /// The Sha/Juedou card.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spyinju(player, target)
    ///
    /// ABI 421: the target picks — use a Sha against the owner, or get the skip mark
    /// (consumed by the helper skill next turn's prepare phase).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpyinju {
        /// Skill owner.
        player: ExprIr,
        /// Chosen target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spchijie(player, source, card)
    ///
    /// ABI 422: judgment; rank > 6 cancels the card's target on the owner.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpchijie {
        /// Skill owner.
        player: ExprIr,
        /// Card user.
        source: ExprIr,
        /// The card naming the owner.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spshangyi(player, target)
    ///
    /// ABI 423: reveal the target's hand and gain one of its cards.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpshangyi {
        /// Skill owner.
        player: ExprIr,
        /// Hand owner.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spjianyi(player)
    ///
    /// ABI 424: gain armor cards from the discard pile at another player's end phase.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpjianyi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_fyjianyu(player, a, b)
    ///
    /// ABI 425: mode 0 (b==0) grants the 喻 mark to the two active targets;
    /// mode 1 (b!=0) draws one for the marked target when a marked play-phase
    /// player targeted it.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveFyjianyu {
        /// Skill owner.
        player: ExprIr,
        /// Event source or zero.
        a: ExprIr,
        /// Event target or mode zero.
        b: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_liubing(player, card)
    ///
    /// ABI 426: the owner's first physical Sha in the play phase becomes diamond.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveLiubing {
        /// Skill owner.
        player: ExprIr,
        /// The physical Sha.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_heji(player, target, card)
    ///
    /// ABI 427: after another player's single-target Juedou/red Sha, optionally use
    /// a virtual Sha or Juedou against the original target.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveHeji {
        /// Skill owner.
        player: ExprIr,
        /// Original target.
        target: ExprIr,
        /// The triggering card.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twjiaohua(player, target)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveTwjiaohua {
        /// Skill owner.
        player: ExprIr,
        /// Player who drew the triggering cards.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twkujian(player, target, action)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveTwkujian {
        /// Skill owner.
        player: ExprIr,
        /// Gift recipient or event player.
        target: ExprIr,
        /// Zero gifts; one rewards use/response; two penalizes another loss.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twruilian(player, target, action)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveTwruilian {
        /// Skill owner.
        player: ExprIr,
        /// Selected role or triggering event player.
        target: ExprIr,
        /// Zero prompts; one records; two closes the tracked turn.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 济溯：判定阶段开始时跳过本次判定与摸牌阶段。
    ///
    /// Sets the pending judge skip and the SKIP_DRAW turn flag for the current player.
    EffectSkipJudgeAndDrawPhases {
        /// Current phase player.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect equip_weapon_from_draw_pile(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectEquipWeaponFromDrawPile {
        /// Equipment recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twjianwei(player, actor, stage)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveTwjianwei {
        /// Skill owner.
        player: ExprIr,
        /// Prepare-phase player.
        actor: ExprIr,
        /// Zero for the owner's prepare phase; one for another player's prepare phase.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twtanfeng(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveTwtanfeng {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twzhibian(player)
    EffectResolveTwzhibian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twyuyan(player, source, card)
    EffectResolveTwyuyan {
        /// Skill owner.
        player: ExprIr,
        /// Sha source.
        source: ExprIr,
        /// Physical Sha.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twguoyi(player, target, card, hand_lock_skill_def)
    EffectResolveTwguoyi {
        /// Skill owner and current card user.
        player: ExprIr,
        /// Current card target.
        target: ExprIr,
        /// Current card.
        card: ExprIr,
        /// Temporary hand-lock helper skill definition.
        hand_lock_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twchuhai(player, action)
    EffectResolveTwchuhai {
        /// Skill owner.
        player: ExprIr,
        /// Zero resolves success; one handles gained-card discard.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twyizhu(player, source, card, action)
    EffectResolveTwyizhu {
        /// Skill owner.
        player: ExprIr,
        /// Current card user.
        source: ExprIr,
        /// Current logical card.
        card: ExprIr,
        /// Zero plants pearls; one handles use; two clears discarded pearls.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twluanchou(player, helper_skill_def, action)
    EffectResolveTwluanchou {
        /// Skill owner.
        player: ExprIr,
        /// Shared helper skill definition.
        helper_skill_def: u32,
        /// Zero resets holders; one redirects current damage.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twxiawei(player, wangxing_skill_def, action)
    EffectResolveTwxiawei {
        /// Skill owner.
        player: ExprIr,
        /// Temporary 妄行 settlement helper definition.
        wangxing_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twbianfuren(player, target, helper_skill_def, action)
    EffectResolveTwbianfuren {
        /// Skill owner.
        player: ExprIr,
        /// Current event player.
        target: ExprIr,
        /// Helper skill definition.
        helper_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twfeiyi(player, action)
    EffectResolveTwfeiyi {
        /// Skill owner.
        player: ExprIr,
        /// Resolver action.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twchenzhen(player, target, helper_skill_def, record_skill_def, action)
    EffectResolveTwchenzhen {
        /// Skill owner.
        player: ExprIr,
        /// Current event player.
        target: ExprIr,
        /// First helper skill definition.
        helper_skill_def: u32,
        /// Second helper skill definition.
        record_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twqianju(player, target)
    EffectResolveTwqianju {
        /// Skill owner.
        player: ExprIr,
        /// Damaged player.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twqingxi(player, target, card)
    EffectResolveTwqingxi {
        /// Skill owner and Sha user.
        player: ExprIr,
        /// Current Sha target.
        target: ExprIr,
        /// Current physical Sha.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twzaoli(player)
    EffectResolveTwzaoli {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twzhengjian(player, target, helper_skill_def, action)
    EffectResolveTwzhengjian {
        /// Skill owner.
        player: ExprIr,
        /// Current Play-phase owner.
        target: ExprIr,
        /// 众斥 helper skill definition.
        helper_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twtianyu(player, target, helper_skill_def, action)
    EffectResolveTwtianyu {
        /// Skill owner.
        player: ExprIr,
        /// Current event target.
        target: ExprIr,
        /// 扬师 attack-range mark skill definition.
        helper_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twwujing(player, target_count, action)
    EffectResolveTwwujing {
        /// Skill owner.
        player: ExprIr,
        /// Declared card target count.
        target_count: ExprIr,
        /// Resolver action.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twshanghe(player)
    EffectResolveTwshanghe {
        /// Dying skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twwangchang(player, target, card, action)
    EffectResolveTwwangchang {
        /// Skill owner.
        player: ExprIr,
        /// Event card user.
        target: ExprIr,
        /// Triggering card.
        card: ExprIr,
        /// Resolver action.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twcaozhao(player, target, source, card, twfuzuan_def, twfeifu_def, action)
    EffectResolveTwcaozhao {
        /// Skill owner.
        player: ExprIr,
        /// Event target, or none for active/setup paths.
        target: ExprIr,
        /// Event source, or none outside 非服.
        source: ExprIr,
        /// Triggering card, or none outside 非服.
        card: ExprIr,
        /// 辅纂 skill definition.
        twfuzuan_skill_def: u32,
        /// 非服 skill definition.
        twfeifu_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twguohuai(player, target, source, card, mark_def, helper_def, action)
    EffectResolveTwguohuai {
        /// Skill owner.
        player: ExprIr,
        /// Trigger event player.
        target: ExprIr,
        /// Damage source.
        source: ExprIr,
        /// Triggering card carrier.
        card: ExprIr,
        /// 御章 mark definition.
        mark_def: u32,
        /// Temporary no-card helper definition.
        helper_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twchengpu(player, target, source, card, chunlao_def, action)
    EffectResolveTwchengpu {
        /// Skill owner.
        player: ExprIr,
        /// Event or selected target.
        target: ExprIr,
        /// Card user or damage source.
        source: ExprIr,
        /// Triggering Sha carrier.
        card: ExprIr,
        /// 醇醪 skill definition and pile key.
        chunlao_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twcaoang(player, target, source, card, helper_def, action)
    EffectResolveTwcaoang {
        /// Skill owner.
        player: ExprIr,
        /// Original Sha target or damage target.
        target: ExprIr,
        /// Sha user or damage source.
        source: ExprIr,
        /// Triggering Sha carrier.
        card: ExprIr,
        /// Distance helper skill and pile definition.
        helper_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twniujin(player, limited_skill_def, action)
    EffectResolveTwniujin {
        /// Skill owner.
        player: ExprIr,
        /// 挫锐 limited skill definition.
        limited_skill_def: u32,
        /// Zero resolves 挫锐; one resolves 裂围.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twmoukui(player, target, card)
    EffectResolveTwmoukui {
        /// Sha user and skill owner.
        player: ExprIr,
        /// Current Sha target.
        target: ExprIr,
        /// Exact Sha carrier.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twfuman(player, target, action)
    EffectResolveTwfuman {
        /// Skill owner and giver.
        player: ExprIr,
        /// Selected other recipient.
        target: ExprIr,
        /// Zero gives; one clears the event card; two clears moved event cards.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_cuijin(player, source, card, action)
    EffectResolveCuijin {
        /// Skill owner.
        player: ExprIr,
        /// Exact Sha user.
        source: ExprIr,
        /// Exact Sha carrier.
        card: ExprIr,
        /// Zero arms tracking; one clears it.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twqingkou(player, winner, card, action)
    EffectResolveTwqingkou {
        /// Skill owner and Duel initiator.
        player: ExprIr,
        /// Duel damage source on completion.
        winner: ExprIr,
        /// Exact materialized virtual Duel card.
        card: ExprIr,
        /// Zero starts the Duel; one applies the post-resolution reward.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twchenxun(player, target, card, action)
    EffectResolveTwchenxun {
        /// Skill owner and Duel initiator.
        player: ExprIr,
        /// Selected or resolved Duel target.
        target: ExprIr,
        /// Exact materialized Duel card.
        card: ExprIr,
        /// Resolver action, zero through four.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twfenxian(player, target, card, action)
    EffectResolveTwfenxian {
        /// Skill owner.
        player: ExprIr,
        /// Duel target.
        target: ExprIr,
        /// Exact materialized Duel card.
        card: ExprIr,
        /// Resolver action, zero through four.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect set_skill_content_flag(player, bit, value)
    EffectSetSkillContentFlag {
        /// Skill owner.
        player: ExprIr,
        /// Flag bit in 0..64.
        bit: u8,
        /// New flag value.
        value: bool,
        /// Source span.
        span: Span,
    },
    /// Give the active invocation's selected hand card to `target`, let that target optionally
    /// use the exact card, then draw one for the owner after a successful use. ABI 497.
    EffectGiveSelectedThenTargetMayUseDrawOwner {
        /// Skill owner and giver.
        player: ExprIr,
        /// Other living recipient.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Declare a logical card category, then let the damage target give one matching
    /// hand/equipment card or increase the bound damage by one. ABI 498.
    EffectChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
        /// Damage source and skill owner.
        player: ExprIr,
        /// Damage target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dcyingtu(player)
    ///
    /// After a card move, when a neighbor gained cards, gain one of their he cards and
    /// give one of the owner's to the other neighbor (equip is used by the recipient).
    /// ABI 487 (C 段 omp, `dcyingtu`). Appended for serialized ordinal stability.
    EffectResolveDcyingtu {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose a non-source other player, gain one deterministic-random hand/equipment card, and
    /// use it when it is equipment. ABI 499.
    EffectChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
        /// Damaged skill owner.
        player: ExprIr,
        /// Damage source excluded from the target choice.
        source: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_mobiledanshou(player, target). ABI 500.
    EffectResolveMobiledanshou {
        /// Skill owner.
        player: ExprIr,
        /// Player whose End phase is beginning.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_mbqingdao(player, source, card, damaged). ABI 503.
    EffectResolveMbqingdao {
        /// Skill owner.
        player: ExprIr,
        /// Resolved card user.
        source: ExprIr,
        /// Resolved card carrier.
        card: ExprIr,
        /// Whether the owner was damaged by the exact use.
        damaged: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_mbxiugeng(player, helper_def, hand_limit_def, stage). ABI 503.
    EffectResolveMbxiugeng {
        /// Skill owner or recorded helper owner.
        player: ExprIr,
        /// Temporary record helper definition.
        helper_def: u32,
        /// Persistent hand-limit helper definition.
        hand_limit_def: u32,
        /// Resolution stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_mbchenshe(player, dying, source). ABI 503.
    EffectResolveMbchenshe {
        /// Skill owner.
        player: ExprIr,
        /// Dying player.
        dying: ExprIr,
        /// Damage source.
        source: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_mbquanchong(player, helper_def). ABI 504.
    EffectResolveMbquanchong {
        /// Skill owner.
        player: ExprIr,
        /// Phase-begin penalty helper definition.
        helper_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_mbrenxing(player). ABI 504.
    EffectResolveMbrenxing {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_mbchizhang(player, card). ABI 505.
    EffectResolveMbchizhang {
        /// Skill owner.
        player: ExprIr,
        /// Current damage card.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_mbduanyang(player, target, card, mode). ABI 505.
    EffectResolveMbduanyang {
        /// Skill owner.
        player: ExprIr,
        /// Event player.
        target: ExprIr,
        /// Event card.
        card: ExprIr,
        /// Resolution mode.
        mode: u8,
        /// Source span.
        span: Span,
    },
    /// 势陆郁生「润微」复合。ABI 506.
    EffectResolveMbrunwei {
        /// Skill owner expression.
        player: ExprIr,
        /// Event or selected target expression.
        target: ExprIr,
        /// Resolver entry stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 势陆郁生「霜怀」复合。ABI 506.
    EffectResolveMbshuanghuai {
        /// Skill owner expression.
        player: ExprIr,
        /// Pending damage target expression.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 势孙峻「凶图」复合。ABI 507.
    EffectResolveMbxiongtu {
        /// Skill owner expression.
        player: ExprIr,
        /// Event or selected target expression.
        target: ExprIr,
        /// Resolver entry stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 势田丰「刚鲠」复合。ABI 508.
    EffectResolveMbganggeng {
        /// Skill owner expression.
        player: ExprIr,
        /// Selected or event target expression.
        target: ExprIr,
        /// Resolver stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 势田丰「死谏」复合。ABI 508.
    EffectResolveMbsijian {
        /// Skill owner expression.
        player: ExprIr,
        /// One-shot discard helper definition.
        helper_def: u32,
        /// Source span.
        span: Span,
    },
    /// 势杨弘「间计」复合。ABI 509.
    EffectResolveMbjianji {
        /// Skill owner expression.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 势杨弘「远谟」复合。ABI 509.
    EffectResolveMbyuanmo {
        /// Skill owner expression.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 势张燕「飞径」复合。ABI 510.
    EffectResolveMbfeijing {
        /// Skill owner expression.
        player: ExprIr,
        /// Original Sha target expression.
        target: ExprIr,
        /// Sha card expression.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 势张燕「骁戈」复合。ABI 510.
    EffectResolveMbxiaoge {
        /// Skill owner expression.
        player: ExprIr,
        /// Event target expression.
        target: ExprIr,
        /// Sha card expression.
        card: ExprIr,
        /// Resolver stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 势庞羲「蓄业」复合。ABI 514.
    EffectResolveMbxuye {
        /// Skill owner expression.
        player: ExprIr,
        /// Effect target expression.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 势庞羲「狂想」复合。ABI 514.
    EffectResolveMbkuangxiang {
        /// Skill owner expression.
        player: ExprIr,
        /// Exchange or event target expression.
        target: ExprIr,
        /// Resolver stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 势程普「督作」「避憾」复合。ABI 515.
    EffectResolvePotchengpu {
        /// Skill owner expression.
        player: ExprIr,
        /// Event target expression.
        target: ExprIr,
        /// Resolver stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 势臧洪「拒孤」复合。ABI 516.
    EffectResolvePotjugu {
        /// Skill owner expression.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 势曹真「伺锋」复合。ABI 517.
    EffectResolvePotsifeng {
        /// Skill owner expression.
        player: ExprIr,
        /// Expansion holder expression.
        target: ExprIr,
        /// Event card expression.
        card: ExprIr,
        /// Resolver stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 势陈群「法恩」「定品」复合。ABI 518.
    EffectResolvePotchenqun {
        /// Skill owner expression.
        player: ExprIr,
        /// Current card user, or unused for 定品.
        target: ExprIr,
        /// Zero resolves 法恩; one resolves 定品.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 势陈到「往烈」复合。ABI 519.
    EffectResolvePotwanglie {
        /// Skill owner expression.
        player: ExprIr,
        /// Armed or resolved card expression.
        card: ExprIr,
        /// Phase-local helper definition.
        helper_skill_def: u32,
        /// Resolver stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 势陈到「弘毅」复合。ABI 519.
    EffectResolvePothongyi {
        /// Skill owner expression.
        player: ExprIr,
        /// Resolver stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 势娄圭「灌沙」复合。ABI 520.
    EffectResolvePotguansha {
        /// Skill owner expression.
        player: ExprIr,
        /// Temporary hand-limit helper definition.
        helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// 势娄圭「急御」复合。ABI 520.
    EffectResolvePotjiyu {
        /// Skill owner expression.
        player: ExprIr,
        /// Resolver stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
        ] }
    };
}
pub(super) use variants;
