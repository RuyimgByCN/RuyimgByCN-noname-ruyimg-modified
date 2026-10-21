//! Ordered StmtIr variants, starting with `EffectDraw`.

macro_rules! variants {
    ([$($tail:ident),*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail),*] [
            $($variants)*
    /// effect draw_cards(player, count)
    EffectDraw {
        /// Target expr.
        player: ExprIr,
        /// Count expression.
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect recover_hp(player, amount)
    EffectRecover {
        /// Player.
        player: ExprIr,
        /// Amount expression.
        amount: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect deal_damage(source, target, amount[, nature])
    EffectDamage {
        /// Source.
        source: ExprIr,
        /// Target.
        target: ExprIr,
        /// Amount expression.
        amount: ExprIr,
        /// Damage nature: 0=normal, 1=fire, 2=thunder.
        #[serde(default)]
        nature: u8,
        /// Span.
        span: Span,
    },
    /// effect emit_note(code)
    EffectNote {
        /// Code.
        code: i32,
        /// Span.
        span: Span,
    },
    /// effect change_max_hp(player, delta)
    EffectChangeMaxHp {
        /// Player.
        player: ExprIr,
        /// Delta expression.
        delta: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect add_mark(player, mark_def, delta)
    EffectAddMark {
        /// Player.
        player: ExprIr,
        /// Mark def id (literal).
        mark: u32,
        /// Delta literal.
        delta: i32,
        /// Span.
        span: Span,
    },
    /// effect remove_mark(player, mark_def, amount)
    EffectRemoveMark {
        /// Player.
        player: ExprIr,
        /// Mark def id.
        mark: u32,
        /// Amount.
        amount: i32,
        /// Span.
        span: Span,
    },
    /// effect discard_hand(player, count)
    EffectDiscardHand {
        /// Player.
        player: ExprIr,
        /// Count.
        count: i32,
        /// Span.
        span: Span,
    },
    /// effect discard_hand_equip(player, count): deterministically discard up to `count` cards from
    /// `player`'s hand then equipment (legacy he scope).
    EffectDiscardHandEquip {
        /// Player.
        player: ExprIr,
        /// Count.
        count: i32,
        /// Span.
        span: Span,
    },
    /// effect discard_selected(player)
    EffectDiscardSelected {
        /// Player whose invocation cards are discarded.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect discard_one_hand_or_equip(player)
    EffectDiscardOneHandOrEquip {
        /// Player who must choose and discard one card.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect discard_all_hand_and_equip(player)
    EffectDiscardAllHandAndEquip {
        /// Player whose hand and equipment cards are discarded.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect remove_all_skills(player)
    EffectRemoveAllSkills {
        /// Player losing all current skills.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect judge_gain_equipment_by_color(player)
    EffectJudgeGainEquipmentByColor {
        /// Player performing the judgment and gaining the equipment.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect reveal_discard_suit_gain_or_use(player, mark_def, base_count, max_mark)
    EffectRevealDiscardSuitGainOrUse {
        /// Player performing the choices.
        player: ExprIr,
        /// Mark definition id.
        mark: u32,
        /// Base reveal count.
        base_count: u8,
        /// Maximum mark count.
        max_mark: u8,
        /// Span.
        span: Span,
    },
    /// effect choose_discard_or_fire_damage_skip_phases(player, max_discard, damage)
    EffectChooseDiscardOrFireDamageSkipPhases {
        /// Current-turn player performing the choices.
        player: ExprIr,
        /// Maximum discard count.
        max_discard: u8,
        /// Fire damage amount.
        damage: u8,
        /// Span.
        span: Span,
    },
    /// effect draw_selected(player)
    EffectDrawSelected {
        /// Draw recipient.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect take_hand(from, to)
    EffectTakeHand {
        /// From player.
        from: ExprIr,
        /// To player.
        to: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect lose_hp(player, amount)
    EffectLoseHp {
        /// Player.
        player: ExprIr,
        /// Amount expression.
        amount: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect add_skill(player, skill_def)
    EffectAddSkill {
        /// Player.
        player: ExprIr,
        /// Skill definition id (literal).
        skill_def: u32,
        /// Span.
        span: Span,
    },
    /// effect add_bound_skill(player, skill_def, target)
    ///
    /// Add a skill whose instance state binds it to one living target.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectAddBoundSkill {
        /// Skill owner.
        player: ExprIr,
        /// Skill definition id (literal).
        skill_def: u32,
        /// Bound living target.
        target: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect remove_skill(skill_instance)
    EffectRemoveSkill {
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect refresh_skill(skill_instance)
    EffectRefreshSkill {
        /// Skill instance to refresh.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect tag_event_cards(skill_instance) — attach the skill's tag to the event cards.
    EffectTagEventCards {
        /// Skill instance whose definition id is used as the tag value.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect untag_event_cards(skill_instance) — remove the skill's tag from the event cards.
    EffectUntagEventCards {
        /// Skill instance whose definition id is used as the tag value.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect storage_set(skill_instance, key, value) — write to keyed content storage.
    EffectStorageSet {
        /// Skill instance whose storage is updated.
        skill: ExprIr,
        /// Storage key.
        key: ExprIr,
        /// Storage value.
        value: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect draw_all_event_targets(count) — draw `count` to the event player + each target.
    EffectDrawAllEventTargets {
        /// Count per player.
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect clanshenjun_reveal(self, event_card, self_skill) — reveal and
    /// gaintag every hand card sharing the event card's effective kind.
    EffectClanshenjunReveal {
        /// Skill owner.
        player: ExprIr,
        /// The card whose kind is matched.
        card: ExprIr,
        /// Skill instance whose definition id becomes the tag.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect clanshenjun_use(self, self_skill) — phase-end tagged-card use.
    EffectClanshenjunUse {
        /// Skill owner.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect clanyuzhi_reveal(self, self_skill) — round-start: choose one hand
    /// card, reveal + gaintag it, draw its name length (tagging the draws).
    EffectClanyuzhiReveal {
        /// Skill owner.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect clanyuzhi_round_end(self, self_skill) — round-end: discard gaintagged
    /// hand cards, then run the ambition check (lose 保族 or 1 thunder damage).
    EffectClanyuzhiRoundEnd {
        /// Skill owner.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect qieyi_phase_start(self, self_skill) — play-phase start: reset per-suit
    /// storage and privately view the top two draw-pile cards.
    EffectQieyiPhaseStart {
        /// Skill owner.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect qieyi_use_after(self, event_card, self_skill) — per-suit first use:
    /// reveal the top draw-pile card, gain it on same color/kind else put one card
    /// on top of the pile.
    EffectQieyiUseAfter {
        /// Skill owner.
        player: ExprIr,
        /// The card that was used.
        card: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect jianzhi_phase_end(self, self_skill) — end phase: choose judgment count
    /// (惬诣 fires this turn), run judgments, assign matching-suit judge cards, deal
    /// 1 thunder damage per non-matching judge card.
    EffectJianzhiPhaseEnd {
        /// Skill owner.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect choose_use_virtual_nanman(self, self_skill) — use a suitless, source-free
    /// virtual 南蛮入侵 through the ordinary card pipeline (phase-agnostic).
    EffectChooseUseVirtualNanman {
        /// Skill owner.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_qiluan(self, self_skill) — phase-end: draw Σ alive players'
    /// current-turn kill counts × (3 if the killer is the owner else 1).
    EffectResolveQiluan {
        /// Skill owner who draws.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect discard_all_own_judge(self, self_skill) — discard every card in the
    /// player's judgment zone.
    EffectDiscardAllOwnJudge {
        /// Player whose judgment zone is cleared.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect store_event_card_in_skill_pile(self, event_card, self_skill) — store the
    /// triggering event's used card (if in the discard pile) onto the owner's skill pile.
    EffectStoreEventCardInSkillPile {
        /// Skill owner who stores the card.
        player: ExprIr,
        /// The triggering event's used card.
        card: ExprIr,
        /// Skill instance (defines the pile).
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect move_card(card, to_zone, from_zone_or_zero)
    EffectMoveCard {
        /// Card instance.
        card: ExprIr,
        /// Destination zone id.
        to: ExprIr,
        /// Expected source zone id, or zero to infer.
        from: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect set_turn_state(current, phase_tag)
    EffectSetTurnState {
        /// Current player.
        current: ExprIr,
        /// Stable phase tag.
        phase: i32,
        /// Span.
        span: Span,
    },
    /// effect use_card_on(player, card, target)
    EffectUseCardOn {
        /// User.
        player: ExprIr,
        /// Card instance.
        card: ExprIr,
        /// Single target.
        target: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect enter_dying(player)
    EffectEnterDying {
        /// Player.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect reveal_card(card)
    EffectRevealCard {
        /// Card expression (e.g. `event_card`).
        card: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect use_card(player, card)
    EffectUseCard {
        /// User.
        player: ExprIr,
        /// Card expression.
        card: ExprIr,
        /// Span.
        span: Span,
    },
    /// if cond { then } else { else }
    If {
        /// Condition.
        cond: ExprIr,
        /// Then body.
        then_body: Vec<StmtIr>,
        /// Else body.
        else_body: Vec<StmtIr>,
        /// Span.
        span: Span,
    },
    /// loop n times { body } — bounded only
    Loop {
        /// Trip count (literal).
        times: u32,
        /// Body.
        body: Vec<StmtIr>,
        /// Span.
        span: Span,
    },
    /// let name = expr;
    Let {
        /// Local name.
        name: String,
        /// Value.
        value: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect add_mark(player, mark_def, delta_expression)
    ///
    /// Appended separately so existing package IR with a numeric `delta`
    /// remains deserializable.
    EffectAddMarkExpr {
        /// Player.
        player: ExprIr,
        /// Mark def id (literal).
        mark: u32,
        /// Delta expression.
        delta: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect remove_cards_by_kind(card_kind)
    EffectRemoveCardsByKind {
        /// Stable card-kind tag.
        kind: i32,
        /// Span.
        span: Span,
    },
    /// effect kill_player(player, killer)
    EffectKillPlayer {
        /// Victim.
        player: ExprIr,
        /// Killer, or zero when unattributed.
        killer: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect remove_mark(player, mark_def, amount_expression)
    ///
    /// Appended separately so existing package IR with a numeric `amount`
    /// remains deserializable.
    EffectRemoveMarkExpr {
        /// Player.
        player: ExprIr,
        /// Mark def id (literal).
        mark: u32,
        /// Amount expression.
        amount: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect draw_each_by_seat(first, second, count)
    EffectDrawEachBySeat {
        /// First participant.
        first: ExprIr,
        /// Second participant.
        second: ExprIr,
        /// Cards drawn by each participant.
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect draw_then_give_one(
    ///   player,
    ///   target,
    ///   count,
    ///   any_hand_equip|drawn_only|any_hand_equip_use_equipment
    /// )
    EffectDrawThenGiveOne {
        /// Drawer and giver.
        player: ExprIr,
        /// Recipient.
        target: ExprIr,
        /// Cards to draw.
        count: ExprIr,
        /// Restrict the gift to cards drawn by this effect.
        drawn_only: bool,
        /// Let the recipient optionally use the gifted card when it is equipment.
        use_given_equipment: bool,
        /// Span.
        span: Span,
    },
    /// effect draw_then_take_one(player, taker, count)
    EffectDrawThenTakeOne {
        /// Drawer and card owner.
        player: ExprIr,
        /// Player choosing and taking one card.
        taker: ExprIr,
        /// Cards to draw.
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect take_one_card(from, to, hand_equip|equip)
    EffectTakeOneCard {
        /// Card owner.
        from: ExprIr,
        /// Player choosing and taking one card.
        to: ExprIr,
        /// Restrict the choice to the equipment zone.
        equip_only: bool,
        /// Span.
        span: Span,
    },
    /// effect turn_over(player)
    EffectTurnOver {
        /// Player whose character card is toggled.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect remove_skill_by_def(player, skill_def)
    EffectRemoveSkillByDef {
        /// Skill owner.
        player: ExprIr,
        /// Stable definition to remove.
        skill_def: u32,
        /// Span.
        span: Span,
    },
    /// effect choose_discard_hand_add_skill_by_color(player, red_skill_def, black_skill_def)
    EffectChooseDiscardHandAddSkillByColor {
        /// Player making the optional discard.
        player: ExprIr,
        /// Skill granted after a red discard.
        red_skill_def: u32,
        /// Skill granted after a black discard.
        black_skill_def: u32,
        /// Span.
        span: Span,
    },
    /// effect reveal_top_gain_non_basic_or_swap_basic(player)
    EffectRevealTopGainNonBasicOrSwapBasic {
        /// Player revealing and possibly gaining the card.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect prevent_current_damage()
    EffectPreventCurrentDamage {
        /// Span.
        span: Span,
    },
    /// effect judge_cancel_current_sha_target_on_red(player, card, target)
    EffectJudgeCancelCurrentShaTargetOnRed {
        /// Player performing the judgment.
        player: ExprIr,
        /// Physical Sha currently resolving.
        card: ExprIr,
        /// Current Sha target.
        target: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect choose_draw_or_recover(player, draw_count, recover_count, allow_cancel)
    EffectChooseDrawOrRecover {
        /// Player making the choice and receiving the result.
        player: ExprIr,
        /// Cards drawn by the draw branch.
        draw_count: ExprIr,
        /// HP recovered by the recovery branch.
        recover_count: ExprIr,
        /// Whether declining both branches is legal.
        allow_cancel: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect choose_use_virtual_sha(player, distance_policy)
    EffectChooseUseVirtualSha {
        /// Player using the virtual Sha.
        player: ExprIr,
        /// Whether ordinary distance is ignored.
        ignore_distance: bool,
        /// Span.
        span: Span,
    },
    /// effect adjust_hand_to(player, count)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectAdjustHandTo {
        /// Player whose hand is adjusted.
        player: ExprIr,
        /// Exact target hand count.
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect gain_card_by_kind(player, card_kind)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGainCardByKind {
        /// Player gaining the card.
        player: ExprIr,
        /// Stable card-kind tag.
        kind: i32,
        /// Span.
        span: Span,
    },
    /// effect judge_recover_on_suit(player, suit, amount)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectJudgeRecoverOnSuit {
        /// Player performing the judgment and receiving any recovery.
        player: ExprIr,
        /// Stable suit tag: 1=spade, 2=heart, 3=club, 4=diamond.
        suit: u8,
        /// HP recovered when the effective judgment suit matches.
        amount: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect use_virtual_card_on(player, card_kind, target)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectUseVirtualCardOn {
        /// User.
        player: ExprIr,
        /// Stable logical card-kind tag.
        kind: i32,
        /// Single target.
        target: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect judge_damage_other_on_suit(player, suit, amount, nature)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectJudgeDamageOtherOnSuit {
        /// Player performing the judgment and dealing any resulting damage.
        player: ExprIr,
        /// Stable suit tag: 1=spade, 2=heart, 3=club, 4=diamond.
        suit: u8,
        /// Damage dealt when the effective judgment suit matches.
        amount: ExprIr,
        /// Damage nature: 0=normal, 1=fire, 2=thunder.
        nature: u8,
        /// Span.
        span: Span,
    },
    /// effect recast_selected(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRecastSelected {
        /// Player recasting the invocation's selected cards.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect disable_all_equip_slots(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDisableAllEquipSlots {
        /// Player whose currently enabled equipment slots are disabled.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect move_equipment_then_draw(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectMoveEquipmentThenDraw {
        /// Skill owner choosing the equipment and destination.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect choose_use_virtual_sha_or_draw(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseUseVirtualShaOrDraw {
        /// Player choosing a normal-distance, no-use-count virtual Sha or one card.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect discard_equipment_pair_with_armor(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDiscardEquipmentPairWithArmor {
        /// Skill owner choosing one equipment from each selected target.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect discard_target_equipment_gain_mount(player, target)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDiscardTargetEquipmentGainMount {
        /// Player choosing the equipment and receiving a discarded mount.
        player: ExprIr,
        /// Player whose equipment may be discarded.
        target: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect discard_target_card_gain_mount(player, target, equip|hand_equip|hand_equip_judge)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDiscardTargetCardGainMount {
        /// Player choosing the card and receiving a discarded mount.
        player: ExprIr,
        /// Player whose scoped card may be discarded.
        target: ExprIr,
        /// Canonical zone scope.
        scope: TargetCardScopeIr,
        /// Span.
        span: Span,
    },
    /// effect make_current_sha_target_unrespondable(source, card, target)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectMakeCurrentShaTargetUnrespondable {
        /// Current Sha user.
        source: ExprIr,
        /// Physical Sha currently resolving.
        card: ExprIr,
        /// Current Sha target.
        target: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect choose_discard_target_card(player, target, hand|equip|hand_equip|hand_equip_judge)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseDiscardTargetCard {
        /// Player choosing the card.
        player: ExprIr,
        /// Player whose scoped card may be discarded.
        target: ExprIr,
        /// Canonical zone scope.
        scope: TargetCardScopeIr,
        /// Span.
        span: Span,
    },
    /// effect judge_prevent_damage_lose_max_hp_on_nonheart(player, target)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectJudgePreventDamageLoseMaxHpOnNonHeart {
        /// Skill owner performing the judgment.
        player: ExprIr,
        /// Damaged player whose maximum HP is reduced on a non-heart result.
        target: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect choose_gain_top_cards_by_suit(player, count, suit)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseGainTopCardsBySuit {
        /// Player privately viewing the cards and choosing matching cards.
        player: ExprIr,
        /// Maximum number of draw-pile top cards viewed.
        count: u8,
        /// Stable suit tag: 1=spade, 2=heart, 3=club, 4=diamond.
        suit: u8,
        /// Span.
        span: Span,
    },
    /// effect choose_other_draw_by_hand_count(player, normal_count, empty_count, times)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseOtherDrawByHandCount {
        /// Player choosing a different living player on each iteration.
        player: ExprIr,
        /// Cards drawn when the selected target has at least one hand card.
        normal_count: ExprIr,
        /// Cards drawn when the selected target has no hand cards.
        empty_count: ExprIr,
        /// Number of independently cancellable target choices.
        times: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect choose_discard_one_then_gain_target_equipment(player, target, slot)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseDiscardOneThenGainTargetEquipment {
        /// Player who may discard one hand/equipment card as the cost.
        player: ExprIr,
        /// Player whose matching equipped cards are gained after the cost resolves.
        target: ExprIr,
        /// Stable equipment-slot tag: 1=weapon through 5=treasure.
        slot: u8,
        /// Span.
        span: Span,
    },
    /// effect choose_target_draw(player, count, other|male_other|female_other|any)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseTargetDraw {
        /// Player making the target choice.
        player: ExprIr,
        /// Cards drawn by the selected target.
        count: ExprIr,
        /// Canonical target scope.
        scope: DrawTargetScopeIr,
        /// Span.
        span: Span,
    },
    /// effect draw_by_play_phase_card_category_count(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDrawByPlayPhaseCardCategoryCount {
        /// Player whose play-phase history is queried and who draws the cards.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect choose_discard_one_for_sha_refund_and_draw_if_red(player, sha_user, card)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseDiscardOneForShaRefundAndDrawIfRed {
        /// Skill owner choosing the discard cost and receiving the possible draw.
        player: ExprIr,
        /// Player who used the currently resolving Sha.
        sha_user: ExprIr,
        /// Physical card carrying the logical Sha.
        card: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect queue_opening_extra_play_phase(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectQueueOpeningExtraPlayPhase {
        /// Player receiving an extra Play phase before Prepare.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect convert_current_sha_to_fire_lose_hp_if_damage(player, card)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectConvertCurrentShaToFireLoseHpIfDamage {
        /// Current Sha user.
        player: ExprIr,
        /// Physical Sha currently resolving.
        card: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect choose_add_current_sha_target(player, card)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseAddCurrentShaTarget {
        /// Current Sha user and chooser.
        player: ExprIr,
        /// Physical Sha currently resolving.
        card: ExprIr,
        /// Span.
        span: Span,
    },
    /// Choose hand Sha and store them in the current skill's public pile.
    EffectChooseStoreHandShaInSkillPile {
        /// Skill owner expression.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Spend one current-skill pile card to make a dying player use virtual Jiu.
    EffectChooseSpendSkillPileForDyingJiu {
        /// Skill owner expression.
        player: ExprIr,
        /// Dying player expression.
        dying: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Compare the active invocation's selected hand card with a target hand card and grant one
    /// of two helper skills.
    EffectChooseCompareSelectedHandCard {
        /// Skill owner expression.
        player: ExprIr,
        /// Other comparison participant.
        target: ExprIr,
        /// Bound helper definition granted on strict victory.
        win_skill_def: u32,
        /// Losing helper definition granted on tie or loss.
        lose_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Optionally append the current skill instance's bound target to a suspended card use.
    EffectChooseAddSkillBoundCardTarget {
        /// Card user and chooser.
        player: ExprIr,
        /// Physical card currently resolving.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Optionally pay a matching-color non-basic discard cost and grant a color-lock helper.
    EffectChooseDiscardMatchingEquipColorAddSkill {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Other player's current Play phase.
        target: ExprIr,
        /// Temporary helper definition.
        skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Use a virtual Sha from the current color-lock instance's stored source if no Sha was used.
    EffectUseVirtualShaFromCardColorLockSourceIfNoSha {
        /// Helper owner and retaliation target.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose one or two enhancements for the suspended exact card use.
    EffectChooseCurrentCardUseEnhancements {
        /// Current card user and chooser.
        player: ExprIr,
        /// Physical card currently resolving.
        card: ExprIr,
        /// Temporary exact-card-policy helper definition.
        helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Make exactly the suspended physical card use unrespondable through a temporary helper.
    EffectMakeCurrentCardUnrespondable {
        /// Current card user.
        player: ExprIr,
        /// Physical card currently resolving.
        card: ExprIr,
        /// Temporary exact-card-policy helper definition.
        helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Choose whether to draw or discard, then grant the corresponding temporary skill.
    EffectChooseDrawOrDiscardAddSkill {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Temporary skill granted by the draw branch.
        draw_skill_def: u32,
        /// Temporary skill granted by the discard branch.
        discard_skill_def: u32,
        /// Offer the cancellable plain-draw-one / draw-two / discard three-way contract.
        three_way: bool,
        /// Source span.
        span: Span,
    },
    /// Repeatedly use eligible cards from the triggering discard batch as Bingliang.
    EffectChooseDiscardedBlackBasicAsBingliang {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Discard a judgment card, then use virtual Sha and draw if it deals no damage.
    EffectChooseDiscardJudgeUseVirtualShaDrawIfNoDamage {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Judgment-phase player and Sha target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Record one selected card category and target, then discard the selected card.
    EffectDiscardSelectedRecordUniqueCategoryTarget {
        /// Skill owner and selected card owner.
        player: ExprIr,
        /// Selected other player.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose target draw/discard by the originating skill's current turn use count.
    EffectChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Affected player.
        target: ExprIr,
        /// Toggle linked state instead of only linking the owner.
        toggle_link: bool,
        /// Source span.
        span: Span,
    },
    /// Choose and reveal another player's hand card, remembering its logical category.
    EffectChooseRevealOtherHandCardRememberCategory {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Draw, give an exact number of hand/equipment cards, and record the phase target.
    EffectDrawThenGiveExactRecordPhaseTarget {
        /// Skill owner drawing and giving cards.
        player: ExprIr,
        /// Other player's current Play phase.
        target: ExprIr,
        /// Draw and gift count.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Clear the current skill's delayed Play-phase gift target.
    EffectClearPhaseGiftTarget {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Give the fully resolved Sha or its physical Shan sources, then optionally use Sha.
    EffectChooseGainResolvedShaOrShanThenOptionalUseSha {
        /// Skill owner and original Sha user.
        player: ExprIr,
        /// Physical Sha carrier bound to the event.
        sha_card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Skip the owner's current non-idle phase.
    EffectSkipCurrentPhase {
        /// Current phase owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Abort the remaining current resolution and finish the current phase without a skip record.
    EffectFinishCurrentPhaseAbortResolution {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Move the draw-pile top card into the current skill's public pile.
    EffectGainTopCardToSkillPile {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Spend one current-skill pile card and add a marked helper skill to another player.
    EffectChooseDiscardSkillPileCardAddMarkedSkill {
        /// Skill owner.
        player: ExprIr,
        /// Helper recipient.
        target: ExprIr,
        /// Helper skill definition.
        helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Gain one current hand card from each active-skill target in submitted order.
    EffectGainOneHandFromSelectedTargets {
        /// Skill owner and card recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose other targets up to the turn's skipped-phase count and damage each in order.
    EffectChooseOtherTargetsBySkippedPhaseCountDamage {
        /// Skill owner and damage source.
        player: ExprIr,
        /// Damage dealt to every selected target.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Reveal the owner's hand and, when its effective color is uniform, draw for the owner and
    /// an optional bounded set of other living players.
    EffectRevealUniformHandDrawSelfChooseOthers {
        /// Skill owner, hand owner, and target chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose another legal target for the suspended single-target Sha/ordinary trick. That
    /// player either gives one hand card and becomes the user, or becomes an extra target.
    EffectChooseCardUserOrAddTarget {
        /// Current card user and skill owner.
        player: ExprIr,
        /// Suspended physical card.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Force the damaged owner to discard one effective equipment card from hand/equipment,
    /// then reduce the bound pre-resolution damage by one.
    EffectChooseDiscardEquipmentReduceCurrentDamage {
        /// Damaged skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Discard one effective equipment card, then prevent the bound damage segment.
    EffectChooseDiscardEquipmentPreventCurrentDamage {
        /// Damaged skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// At another player's turn end, choose to move one compatible equipment card from that
    /// player to the owner, draw one card, or cancel.
    EffectChooseMoveTargetEquipmentOrDraw {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Other player whose turn ended and whose equipment may be moved.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Give all equipment cards to one selected player, who then either lets the owner recover
    /// by the captured count or damages up to that many players in attack range.
    EffectGiveAllEquipmentChooseRecoverOrDamage {
        /// Limited active skill owner.
        player: ExprIr,
        /// Equipment recipient and branch chooser.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Let the bound Sha damage target discard exactly the source's physical weapon range from
    /// hand; cancellation or an insufficient hand increases that damage by one instead.
    EffectChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
        /// Sha source and skill owner.
        player: ExprIr,
        /// Bound damage target and discard chooser.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose a living player with a different hand count. A player with fewer hand cards draws
    /// one; a player with more discards one deterministic random hand card.
    EffectChooseHandCountDifferentTargetDrawOrRandomDiscard {
        /// Skill owner and target chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Add to the pending draw count for the current draw phase. The authoritative draw entry
    /// consumes the accumulated bonus exactly once.
    EffectIncreaseCurrentDrawCount {
        /// Current draw-phase player.
        player: ExprIr,
        /// Nonnegative additional card count.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// During the owner's Prepare phase, choose any living player with discardable
    /// hand/equipment cards, discard the hand-minus-HP amount (at least one), then either
    /// discard the same number of the owner's cards as non-equipment cards discarded or let
    /// the target draw that many cards.
    EffectChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Draw one card, then sequentially choose and gain one card from the damaged player and
    /// that player's previous and next living seats in legacy order.
    EffectDrawThenChooseGainOneFromDamageTargetAndNeighbors {
        /// Damage source, skill owner, drawer, and chooser.
        player: ExprIr,
        /// Damaged player from the originating damage event.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Force a choice between preventing the current damage or losing one maximum HP and
    /// increasing that same pending damage.
    EffectChoosePreventDamageOrLoseMaxHpIncreaseDamage {
        /// Damage source, skill owner, and chooser.
        player: ExprIr,
        /// Nonnegative damage increase.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Make the other participant's card in the currently starting hand comparison be selected
    /// by authoritative deterministic RNG.
    EffectRandomizeCurrentHandCompareOpponent {
        /// Skill owner and one comparison participant.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Gain all highest-rank logical Sha among the triggering domain event's physical cards.
    EffectGainHighestRankShaFromEventCards {
        /// Skill owner receiving the cards.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Gain the legacy `zongshi` result card from a completed two-player hand comparison.
    EffectGainHandCompareResultCard {
        /// Skill owner receiving the card.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose whether to add or remove one target from the currently resolving basic/trick card.
    EffectChooseAddOrRemoveCurrentCardTarget {
        /// Card user, skill owner, and chooser.
        player: ExprIr,
        /// Physical event card.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose another player and both hand cards for a two-player comparison.
    EffectChooseHandCompare {
        /// First participant and skill owner.
        player: ExprIr,
        /// Helper granted on strict victory.
        win_skill_def: u32,
        /// Helper granted on tie or loss.
        lose_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Hand comparison whose strict-win branch grants the helper to the target
    /// instead of the chooser (ABI 678).
    EffectChooseHandCompareGrantTargetOnWin {
        /// First participant and skill owner.
        player: ExprIr,
        /// Helper granted to the **target** on the chooser's strict victory.
        win_grant_def: u32,
        /// Reserved for symmetry; unused in the grant flow.
        lose_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// 烈刃启动（shenhua 祝融）：以固定受伤害角色为目标启动通用拼点。ABI 531。
    EffectResolveLieren {
        /// Skill owner and comparing player.
        player: ExprIr,
        /// The damaged player and compare target.
        target: ExprIr,
        /// Helper granted on strict victory.
        win_skill_def: u32,
        /// Helper granted on tie or loss.
        lose_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// 屯田启动（shenhua 邓艾）：判定后非红桃判定牌移入本技能牌堆。ABI 533。
    EffectResolveTuntian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 凿险觉醒（shenhua 邓艾）：本技能牌堆至少 3 张时减 1 点体力上限并授予急袭。
    /// ABI 534。
    EffectResolveZaoxian {
        /// Skill owner.
        player: ExprIr,
        /// 急袭 definition granted on awakening.
        jixi_def: u32,
        /// 屯田 definition whose pile holds the 田 cards.
        tuntian_def: u32,
        /// Source span.
        span: Span,
    },
    /// 挑衅启动（shenhua 姜维）：出牌阶段限一次选择目标并推进二选一流程。ABI 537。
    EffectResolveTiaoxin {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 志继觉醒（shenhua 姜维）：准备阶段无手牌时摸两张牌、回复 1 点体力、
    /// 减 1 点体力上限并授予观星。ABI 538。
    EffectResolveZhiji {
        /// Skill owner.
        player: ExprIr,
        /// 观星 definition granted on awakening.
        guanxing_def: u32,
        /// Source span.
        span: Span,
    },
    /// 再起（shenhua 孟获）：摸牌阶段替换，翻亮 X 张并按红桃回复、其余获得。
    /// ABI 540。
    EffectResolveZaiqi {
        /// Skill owner.
        player: ExprIr,
        /// Extra revealed cards beyond the lost HP count.
        extra: u32,
        /// Source span.
        span: Span,
    },
    /// 求首（shenhua 孟获，主公技）：南蛮结算后目标死亡或受创目标数超过 3 时
    /// 蜀/群各摸一张。ABI 542。
    EffectResolveTwqiushou {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 征荣（shenhua 毌丘俭）：杀/伤害锦囊首个目标时从目标取一张手/装备入「荣」。
    /// ABI 545。
    EffectResolveZhengrong {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 鸿举（shenhua 毌丘俭）：觉醒技，「荣」≥3 时可选交换并获清俭。ABI 546。
    EffectResolveHongju {
        /// Skill owner.
        player: ExprIr,
        /// 清俭 definition granted on awakening.
        qingjian_def: u32,
        /// 征荣 definition whose pile holds the 荣 cards.
        rong_def: u32,
        /// Source span.
        span: Span,
    },
    /// 清俭（shenhua 毌丘俭）：出牌阶段限一次，弃一张手牌获一张「荣」并弃目标
    /// 装备/判定区一张牌。ABI 547。
    EffectResolveQingjian {
        /// Skill owner.
        player: ExprIr,
        /// 征荣 definition whose pile holds the 荣 cards.
        rong_def: u32,
        /// Source span.
        span: Span,
    },
    /// 雷击（shenhua 张角）：使用/打出闪后选目标判定。ABI 549。
    EffectResolveReleiji {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 黄天（shenhua 张角，主公技）：群势力角色出牌阶段可交一张闪/闪电。ABI 551。
    EffectResolveHuangtian {
        /// Skill owner.
        player: ExprIr,
        /// The giving Qun player (event player).
        actor: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 好施（shenhua 鲁肃）：摸牌阶段额外摸两张；摸牌阶段结束时给一半手牌。ABI 555。
    EffectResolveHaoshi {
        /// Skill owner.
        player: ExprIr,
        /// Zero = draw replacement, one = give-half settlement.
        stage: u32,
        /// Source span.
        span: Span,
    },
    /// 缔盟（shenhua 鲁肃）：出牌阶段限一次选两名其他角色交换手牌。ABI 556。
    EffectResolveDimeng {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 巧变（shenhua 张郃）：阶段开始时弃一张手牌并跳过该阶段；摸牌/出牌阶段
    /// 随后可选结算对应收益。ABI 559。
    EffectResolveQiaobian {
        /// Skill owner.
        player: ExprIr,
        /// Triggered phase: 0 judge, 1 draw, 2 play, 3 discard.
        phase: u8,
        /// Source span.
        span: Span,
    },
    /// 将当前正在结算的【杀】的指定目标从保存目标中移除。ABI 560。
    EffectExcludeSavedShaTarget {
        /// The Sha card currently resolving.
        card: ExprIr,
        /// Target to exclude.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 享乐（shenhua 刘禅）：使用者弃一张基本牌或此杀对持有者无效。ABI 560。
    EffectResolveXiangle {
        /// Skill owner (the Sha target).
        player: ExprIr,
        /// The Sha user.
        source: ExprIr,
        /// The Sha card.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 放权（shenhua 刘禅）：跳过本回合出牌阶段。ABI 560。
    EffectResolveFangquanSkip {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 放权（shenhua 刘禅）：结束阶段弃一张手牌并授额外回合。ABI 560。
    EffectResolveFangquanGrant {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 若愚（shenhua 刘禅，主公技）：最低体力觉醒。ABI 560。
    EffectResolveRuoyu {
        /// Skill owner.
        player: ExprIr,
        /// 激将 granted skill definition.
        jijiang_def: u32,
        /// Source span.
        span: Span,
    },
    /// 罪论（shenhua 诸葛瞻）：结束阶段揭示牌堆顶三张并按条件计数分派。ABI 563。
    EffectResolveZuilun {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 父荫（shenhua 诸葛瞻，锁定技）：本回合首次被手牌数不小于自己的其他角色
    /// 以【杀】指定为目标时此杀无效。ABI 563。
    EffectResolveFuyin {
        /// Skill owner (the Sha target).
        player: ExprIr,
        /// The Sha user.
        source: ExprIr,
        /// The Sha card.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 回合内禁用指定角色对另一角色使用牌。ABI 571。
    EffectBanUseAgainst {
        /// Banned card user.
        user: ExprIr,
        /// Banned target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 拒战（shenhua 严颜，转换技）。ABI 571。
    EffectResolveJuzhan {
        /// Skill owner.
        player: ExprIr,
        /// Event source.
        source: ExprIr,
        /// The Sha card.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 奇制（shenhua 王基）：用牌后弃非目标角色一张牌并令其摸一张。ABI 573。
    EffectResolveQizhi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 进趋（shenhua 王基）：出牌阶段结束时摸二并弃至奇制次数。ABI 573。
    EffectResolveJinqu {
        /// Skill owner.
        player: ExprIr,
        /// 奇制 skill definition whose per-turn count gates the discard.
        qizhi_def: u32,
        /// Source span.
        span: Span,
    },
    /// 荐相（shenhua 蒯良蒯越）：受指目标后选手牌最少者摸一张。ABI 574。
    EffectResolveJianxiang {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 审时（shenhua 蒯良蒯越，转换技）。ABI 574。
    EffectResolveShenshi {
        /// Skill owner.
        player: ExprIr,
        /// Event source (damage source for 阴).
        source: ExprIr,
        /// Trigger stage: 0 阳 active, 1 阴 damage, 4 turn-end tracked check.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 明任（shenhua 卢植）：首轮准备阶段摸二置「任」；结束阶段替换。ABI 576。
    EffectResolveMingren {
        /// Skill owner.
        player: ExprIr,
        /// Trigger stage: 0 place (prepare), 2 replace (end).
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 贞良（shenhua 卢植，转换技）。ABI 576。
    EffectResolveZhenliang {
        /// Skill owner.
        player: ExprIr,
        /// Event source (the used/responded card's player for 阳).
        source: ExprIr,
        /// The used/responded card for 阳.
        card: ExprIr,
        /// 明任 skill definition owning the「任」pile.
        mingren_def: u32,
        /// Trigger stage: 0 dispatch.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 溃诛（shenhua 孙亮）：弃牌阶段后按弃牌数选择摸牌或伤害。ABI 577。
    EffectResolveKuizhu {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 掣政（shenhua 孙亮）：出牌阶段结束弃置一名角色的一张牌。ABI 577。
    EffectResolveZhizheng {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 立军（shenhua 孙亮，主公技）：吴势力角色用杀后获牌并令其摸一张。ABI 577。
    EffectResolveLijun {
        /// Skill owner (the lord).
        player: ExprIr,
        /// The Wu card user.
        source: ExprIr,
        /// The used Sha card.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 良姻（shenhua 周妃）：失牌/获牌后双分支。ABI 579。
    EffectResolveLiangyin {
        /// Skill owner.
        player: ExprIr,
        /// 箜声 skill definition whose pile count gates the recover condition.
        kongsheng_def: u32,
        /// Trigger stage: 0 draw branch, 3 discard branch.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 箜声（shenhua 周妃）：准备阶段置箜、结束阶段回手/装备使用。ABI 579。
    EffectResolveKongsheng {
        /// Skill owner.
        player: ExprIr,
        /// Trigger stage: 0 place, 2 end dispatch.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 势邓艾「屯田/凿险」复合效果。ABI 543。
    EffectResolvePottuntian {
        /// Skill owner.
        player: ExprIr,
        /// Public DSL action (0..=2).
        action: u8,
        /// 势凿险 definition.
        zaoxian_def: u32,
        /// Source span.
        span: Span,
    },
        ] }
    };
}
pub(super) use variants;
