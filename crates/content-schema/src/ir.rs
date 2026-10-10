//! Rule IR — typed intermediate representation after name/type check.

use crate::TriggerChoiceActor;
use serde::{Deserialize, Serialize};

/// Span in source (byte offsets).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Span {
    /// Start byte.
    pub start: u32,
    /// End byte.
    pub end: u32,
}

/// Top-level compiled unit before codegen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleModuleIr {
    /// Module name.
    pub name: String,
    /// Skills in module.
    pub skills: Vec<SkillIr>,
}

/// One skill definition IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillIr {
    /// Skill name.
    pub name: String,
    /// Stable skill def id (from source or assigned).
    pub def_id: u32,
    /// Locked/passive skill marker.
    pub locked: bool,
    /// Only active while the owner has the Lord identity.
    #[serde(default)]
    pub lord_only: bool,
    /// Cannot be copied or gained by another skill.
    #[serde(default)]
    pub non_gainable: bool,
    /// Conversion skill whose branch can be toggled by generic rules.
    #[serde(default)]
    pub conversion: bool,
    /// Triggers.
    pub triggers: Vec<TriggerIr>,
    /// Active (on-demand) skills.
    pub actives: Vec<ActiveIr>,
    /// View-as conversions.
    pub view_as: Vec<ViewAsIr>,
    /// Passive modifiers.
    pub modifiers: Vec<ModifierIr>,
    /// Source span.
    pub span: Span,
}

/// Trigger IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerIr {
    /// Event name: damage, card_used, phase_end, …
    pub event: String,
    /// Filter: owner_is_target | owner_is_source | always
    pub filter: String,
    /// Whether the trigger opens a player confirmation.
    pub optional: bool,
    /// Player who confirms an optional trigger.
    #[serde(default)]
    pub choice_actor: TriggerChoiceActor,
    /// Body statements.
    pub body: Vec<StmtIr>,
    /// Span.
    pub span: Span,
}

/// Active skill IR (player-activated, e.g. out-of-turn play phase).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveIr {
    /// Activation window: play_phase | any | response
    pub window: String,
    /// Optional target-count contract.
    pub targets: Option<ActiveTargetsIr>,
    /// Optional authoritative per-turn activation limit.
    pub uses_per_turn: Option<u8>,
    /// Optional authoritative activation limit scoped to one Play phase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses_per_play_phase: Option<u8>,
    /// Optional dynamic per-turn activation limit source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses_per_turn_source: Option<String>,
    /// Optional owner-mark threshold required before activation: `(mark definition, count)`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark_at_least: Option<(u32, u32)>,
    /// Require the owner not to have this skill definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_missing_skill: Option<u32>,
    /// Require the owner's hand count to exceed current HP.
    #[serde(default)]
    pub hand_gt_hp: bool,
    /// Require the owner's hand count to exceed maximum HP.
    #[serde(default)]
    pub hand_gt_max_hp: bool,
    /// Require the owner's current hand limit to be positive.
    #[serde(default)]
    pub hand_limit_positive: bool,
    /// Require at least one card in the owner's equipment zone.
    #[serde(default)]
    pub equip_nonempty: bool,
    /// Require at least one card in the owner's hand.
    #[serde(default)]
    pub hand_nonempty: bool,
    /// Require the owner to be face up.
    #[serde(default)]
    pub face_up: bool,
    /// Require the owner's hand to hold no card carrying this gaintag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hand_missing_gaintag: Option<u32>,
    /// Require the owner not to be linked.
    #[serde(default)]
    pub owner_unlinked: bool,
    /// Allow this active skill to resolve at most once during the match.
    #[serde(default)]
    pub once_per_game: bool,
    /// Optional card selection zone.
    pub selection: Option<ActiveSelectionIr>,
    /// Body statements.
    pub body: Vec<StmtIr>,
    /// Span.
    pub span: Span,
}

/// Declarative player-target input for an active skill.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveTargetsIr {
    /// Minimum target count.
    pub min: u8,
    /// Maximum target count.
    pub max: u8,
    /// Whether the skill owner may be selected.
    pub allow_self: bool,
    /// Whether selected targets must tie for the lowest current HP.
    #[serde(default)]
    pub lowest_hp: bool,
    /// Whether selected targets must tie for the highest current HP.
    #[serde(default)]
    pub highest_hp: bool,
    /// Whether selected targets must currently be unlinked.
    #[serde(default)]
    pub unlinked: bool,
    /// Whether selected targets must currently be linked.
    #[serde(default)]
    pub linked: bool,
    /// Optional owner mark whose count caps the selected-player count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_by_owner_mark: Option<u32>,
    /// Cap the selected-player count by the owner's current lost HP.
    #[serde(default)]
    pub max_by_owner_lost_hp: bool,
    /// Cap the selected-player count by the owner's current HP.
    #[serde(default)]
    pub max_by_owner_hp: bool,
    /// Whether selected targets must currently be wounded.
    #[serde(default)]
    pub wounded: bool,
    /// Whether selected targets must be male or double-sex.
    #[serde(default)]
    pub male: bool,
    /// Whether selected targets must have at least the owner's current HP.
    #[serde(default)]
    pub hp_at_least_owner: bool,
    /// Whether selected targets must have greater current HP than the owner.
    #[serde(default)]
    pub hp_gt_owner: bool,
    /// Whether selected targets must have at most the owner's current HP.
    #[serde(default)]
    pub hp_lte_owner: bool,
    /// Optional exclusive lower bound for selected targets' current HP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hp_gt: Option<u8>,
    /// Whether selected targets must have more hand cards than the owner.
    #[serde(default)]
    pub hand_gt_owner: bool,
    /// Whether selected targets must have at most as many hand cards as the owner.
    #[serde(default)]
    pub hand_lte_owner: bool,
    /// Require exactly two targets whose hand-card counts differ by at most three.
    #[serde(default)]
    pub pair_hand_difference_lte_three: bool,
    /// Optional exclusive upper bound for selected targets' hand-card count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hand_lt: Option<u8>,
    /// Require every selected target to have at least one hand card.
    #[serde(default)]
    pub hand_nonempty: bool,
    /// Require the owner and selected targets to each have at least one hand card.
    #[serde(default)]
    pub can_compare: bool,
    /// Optional source-free card kind every selected target must be able to use on the owner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_use_on_owner: Option<String>,
    /// Require the second selected target to be able to use a source-free Duel on the first.
    #[serde(default)]
    pub second_can_use_juedou_on_first: bool,
    /// Require exactly two targets with discardable equipment and at least one discardable armor.
    #[serde(default)]
    pub equipment_pair_with_armor: bool,
    /// Require every selected target to currently have an equipped card.
    #[serde(default)]
    pub has_equip: bool,
    /// Require every selected target to have equipment the skill owner may currently discard.
    #[serde(default)]
    pub has_discardable_equip: bool,
    /// Require every selected target to have equipment the skill owner may currently gain.
    #[serde(default)]
    pub has_gainable_equip: bool,
    /// Require every selected target to have a hand/equipment card the owner may gain.
    #[serde(default)]
    pub has_gainable_hand_equip: bool,
    /// Require every selected target to be inside the skill owner's attack range.
    #[serde(default)]
    pub in_attack_range: bool,
    /// Require the skill owner to be inside every selected target's attack range.
    #[serde(default)]
    pub owner_in_target_attack_range: bool,
    /// Require every selected target to have at least one hand/equipment card.
    #[serde(default)]
    pub hand_equip_nonempty: bool,
    /// Require every selected target to have a hand/equipment card the owner may discard.
    #[serde(default)]
    pub has_discardable_hand_equip: bool,
    /// Reject targets already recorded by this skill in the current turn.
    #[serde(default)]
    pub unused_by_skill_this_turn: bool,
    /// Reject targets already recorded by this skill in the current play phase.
    #[serde(default)]
    pub unused_by_skill_this_play_phase: bool,
    /// Require selected-player and selected-card counts to match.
    #[serde(default)]
    pub match_selected_card_count: bool,
    /// Require selected cards to match the sole target's equipment-card count.
    #[serde(default)]
    pub selected_cards_match_target_equip_count: bool,
}

/// Declarative card input for an active skill.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveSelectionIr {
    /// Zone name.
    pub zone: String,
    /// Minimum count.
    pub min: u8,
    /// Maximum count.
    pub max: u8,
    /// Whether the activation may be cancelled at the selection barrier.
    #[serde(default)]
    pub allow_cancel: bool,
    /// Whether every selected card must have the same effective suit.
    #[serde(default)]
    pub same_suit: bool,
    /// Whether every selected card must have the same effective color.
    #[serde(default)]
    pub same_color: bool,
    /// Whether selected cards must have pairwise-distinct logical categories.
    #[serde(default)]
    pub distinct_categories: bool,
    /// Whether selected cards must have pairwise-distinct effective suits.
    #[serde(default)]
    pub distinct_suits: bool,
    /// Optional card-kind filter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    /// Reject logical card categories already recorded by this skill in the current turn.
    #[serde(default)]
    pub unused_category_by_skill_this_turn: bool,
    /// Reject logical card categories already recorded by this skill in the current play phase.
    #[serde(default)]
    pub unused_category_by_skill_this_play_phase: bool,
    /// Reject logical card categories used or discarded by the owner in the current turn.
    #[serde(default)]
    pub unused_category_by_owner_history_this_turn: bool,
    /// Require selected cards' effective numbers to total exactly this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sum_effective_number_exact: Option<u16>,
}

/// View-as IR: treat matching source cards as another kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewAsIr {
    /// As kind: sha|shan|tao|jiu
    pub as_kind: String,
    /// Window: play_phase | any | response
    pub window: String,
    /// Filter string.
    pub filter: String,
    /// Optional source zone; omitted means hand.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_zone: Option<String>,
    /// Optional authoritative per-turn activation limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses_per_turn: Option<u8>,
    /// Whether ordinary card distance checks are ignored.
    #[serde(default)]
    pub ignore_distance: bool,
    /// Whether a red source makes the converted Sha unrespondable.
    #[serde(default)]
    pub red_unrespondable: bool,
    /// Whether a black source avoids consuming the ordinary Sha use count.
    #[serde(default)]
    pub black_no_use_count: bool,
    /// Whether this converted Sha always avoids consuming the ordinary Sha use count.
    #[serde(default)]
    pub no_use_count: bool,
    /// Make each target lacking a hand card of the source suit unable to respond.
    #[serde(default)]
    pub target_lacks_source_suit_unrespondable: bool,
    /// Refund the ordinary Sha use count after this exact converted Sha deals no damage.
    #[serde(default)]
    pub refund_use_count_if_no_damage: bool,
    /// Minimum number of physical source cards; one by default.
    #[serde(default = "default_one")]
    pub min_cards: u8,
    /// Maximum number of physical source cards; one by default.
    #[serde(default = "default_one")]
    pub max_cards: u8,
    /// Require exactly max(1, current HP) physical source cards.
    #[serde(default)]
    pub owner_hp_min_one_cards: bool,
    /// Minimum physical source cards = 1 + lifetime basic-card uses.
    #[serde(default)]
    pub owner_basic_uses_plus_one_cards: bool,
    /// Each rule of this skill usable once per turn (per-rule bitmap).
    #[serde(default)]
    pub once_per_turn_per_branch: bool,
    /// Require every selected source card to have the same effective suit.
    #[serde(default)]
    pub same_suit: bool,
    /// Optional logical card nature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nature: Option<String>,
    /// Optional effect applied when multiple source cards are selected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multi_card_bonus: Option<String>,
    /// Conversion branch required for a source-free family rule.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<u8>,
    /// Whether this branch may be used only once in each round.
    #[serde(default)]
    pub once_per_round: bool,
    /// Branch selected after a successful conversion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_branch: Option<u8>,
    /// Declarative source-free conversion cost.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<String>,
    /// Skill-definition mark consumed by the conversion cost.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark_cost: Option<u32>,
    /// Cards drawn after the converted card is used or responded with.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub post_draw: u8,
    /// Span.
    pub span: Span,
}

fn default_one() -> u8 {
    1
}

fn is_zero_u8(value: &u8) -> bool {
    *value == 0
}

/// Modifier IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModifierIr {
    /// Target channel: distance_from | attack_range | hand_limit
    pub target: String,
    /// Optional card selector for card-aware modifier targets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// Combine mode: add | override | min | max
    pub mode: String,
    /// Optional query-time value source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_source: Option<String>,
    /// Integer value.
    pub value: i32,
    /// Span.
    pub span: Span,
}

/// Statement IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StmtIr {
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
    /// 势邓艾「急袭」记录/结算效果。ABI 543。
    EffectResolvePotjixi {
        /// Skill owner.
        player: ExprIr,
        /// Event target for record actions.
        target: ExprIr,
        /// Public DSL action (0..=2).
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 势陈矫「策断」复合效果。ABI 544。
    EffectResolvePotceduan {
        /// Skill owner.
        player: ExprIr,
        /// Selected active target.
        target: ExprIr,
        /// Public DSL action (0..=1).
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 势太史慈「酣战/战烈/振锋」复合效果。ABI 548。
    EffectResolvePottaishici {
        /// Skill owner.
        player: ExprIr,
        /// Selected/event target.
        target: ExprIr,
        /// Public DSL action (0..=5).
        action: u8,
        /// 酣战 definition.
        hanzhan_def: u32,
        /// 战烈 definition.
        zhanlie_def: u32,
        /// Source span.
        span: Span,
    },
    /// 势钟会「俟资/挟志/狱难/克昌」复合效果。ABI 553。
    EffectResolveMbzhonghui {
        /// Skill owner.
        player: ExprIr,
        /// Event participant.
        target: ExprIr,
        /// Public DSL action (0..=7).
        action: u8,
        /// Event amount.
        amount: ExprIr,
        /// 俟资 definition.
        mbsizi_def: u32,
        /// 克昌 definition.
        mbkechang_def: u32,
        /// Source span.
        span: Span,
    },
    /// 势辛宪英「诫节」复合效果。ABI 554。
    EffectResolvePotjiejie {
        /// Skill owner.
        player: ExprIr,
        /// Current Play-phase actor.
        actor: ExprIr,
        /// 诫节已用 helper definition.
        used_def: u32,
        /// Suit-unlimited helper definitions in spade/heart/club/diamond order.
        suit_defs: [u32; 4],
        /// 清识 definition.
        qingshi_def: u32,
        /// Source span.
        span: Span,
    },
    /// 势辛宪英「清识」复合效果。ABI 554。
    EffectResolvePotqingshi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 势魏延【忠傲/壮誓/饮战/狂骨】复合效果。ABI 558。
    EffectResolvePotweiyan {
        /// Skill owner.
        player: ExprIr,
        /// Event target.
        target: ExprIr,
        /// Bound card.
        card: ExprIr,
        /// Public DSL action 0..=10.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 势周瑜【炽沄/焰洄/焚涛/雄姿】复合效果。ABI 561。
    EffectResolvePotzhouyu {
        /// Skill owner.
        player: ExprIr,
        /// Event target.
        target: ExprIr,
        /// Bound card.
        card: ExprIr,
        /// Public DSL action 0..=7.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 势吕壹【隳律/颂谳/势施】复合效果。ABI 564。
    EffectResolvePotlvyi {
        /// Skill owner.
        player: ExprIr,
        /// Event actor/source.
        target: ExprIr,
        /// Triggering physical card.
        card: ExprIr,
        /// Public DSL action 0..=9.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 势曹爽【典易/奢权/渐专/溺尊】复合效果。ABI 568。
    EffectResolvePotcaoshuang {
        /// Skill owner.
        player: ExprIr,
        /// Event source/target.
        target: ExprIr,
        /// Triggering physical card.
        card: ExprIr,
        /// Public DSL action 0..=11.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// Choose another living player and deal normal damage to that player.
    EffectChooseOtherDamage {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Damage amount.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose another living player, then choose and discard one of that player's cards.
    EffectChooseOtherDiscardTargetCard {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Canonical target-card zone scope.
        scope: TargetCardScopeIr,
        /// Source span.
        span: Span,
    },
    /// Gain the topmost card of one rule kind from the discard pile only.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGainDiscardCardByKind {
        /// Player gaining the card.
        player: ExprIr,
        /// Stable card-kind tag.
        kind: i32,
        /// Source span.
        span: Span,
    },
    /// Reveal all hand cards and, when their effective color is uniform, choose any living
    /// player and deal normal damage to that player.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRevealUniformHandChooseDamage {
        /// Skill owner, hand owner, damage source, and chooser.
        player: ExprIr,
        /// Damage amount.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose and gain one matching card from the discard pile.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseGainDiscardCardByKind {
        /// Player choosing and gaining the card.
        player: ExprIr,
        /// Stable card-kind tag.
        kind: i32,
        /// Source span.
        span: Span,
    },
    /// Draw one card for each supplied divisor of the player's match-lifetime card-action count.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDrawForLifetimeCardActionDivisors {
        /// Player whose action history is inspected and who draws the cards.
        player: ExprIr,
        /// Three positive divisors checked independently.
        divisors: [u32; 3],
        /// Source span.
        span: Span,
    },
    /// Force the owner to choose one definition they do not yet own, then grant it.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseAddMissingSkill {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Three candidate skill definitions.
        skill_defs: [u32; 3],
        /// Source span.
        span: Span,
    },
    /// Force a player to choose a fixed number of distinct living targets, then each draws.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseTargetsDraw {
        /// Player making the target choice.
        player: ExprIr,
        /// Cards drawn by each selected target.
        count: ExprIr,
        /// Exact number of distinct targets.
        target_count: u8,
        /// Canonical target scope.
        scope: DrawTargetScopeIr,
        /// Source span.
        span: Span,
    },
    /// Gain every card captured by the triggering event that is still in the discard pile.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGainEventCards {
        /// Player gaining the cards.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_draw_to_max_hp_cap(player, cap)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseTargetDrawToMaxHpCap {
        /// Player making the target choice.
        player: ExprIr,
        /// Inclusive cap applied to the selected target's maximum HP.
        cap: u32,
        /// Source span.
        span: Span,
    },
    /// effect choose_other_draw_owner_lost_hp_turn_over(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseOtherDrawOwnerLostHpTurnOver {
        /// Skill owner making the target choice and supplying the lost-HP draw count.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect judge_recover_other_on_suit(judge_player, recover_player, suit, amount)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectJudgeRecoverOtherOnSuit {
        /// Player performing the judgment.
        judge_player: ExprIr,
        /// Player receiving any recovery.
        recover_player: ExprIr,
        /// Stable suit tag: 1=spade, 2=heart, 3=club, 4=diamond.
        suit: u8,
        /// HP recovered when the effective judgment suit matches.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect force_others_use_sha_on_nearest_or_lose_hp(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectForceOthersUseShaOnNearestOrLoseHp {
        /// Skill owner excluded from the ordered forced-use queue.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect restore_character_state(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRestoreCharacterState {
        /// Player whose character card is restored to unlinked and face-up.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect discard_all_hand_equip_judge(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDiscardAllHandEquipJudge {
        /// Player whose hand, equipment, and judgment zones are emptied.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_target_card_resolve_current_sha(source, card, target)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseDiscardTargetCardResolveCurrentSha {
        /// Current Sha user and chooser.
        source: ExprIr,
        /// Physical Sha currently resolving.
        card: ExprIr,
        /// Current Sha target whose card may be discarded.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect use_virtual_card_on_selected_targets(player, card_kind)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectUseVirtualCardOnSelectedTargets {
        /// Virtual card user.
        player: ExprIr,
        /// Stable card-kind tag.
        kind: i32,
        /// Source span.
        span: Span,
    },
    /// effect deal_damage_ban_active_if_dying(source, target, amount, skill)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDealDamageBanActiveIfDying {
        /// Damage source.
        source: ExprIr,
        /// Damage target.
        target: ExprIr,
        /// Base damage amount.
        amount: ExprIr,
        /// Active skill instance banned when this damage enters dying.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_reclaim_equipment_draw_if_range_reduced(player, target, skill)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseTargetReclaimEquipmentDrawIfRangeReduced {
        /// Skill owner and equipment-card chooser.
        player: ExprIr,
        /// Other player reclaiming one equipped card into their own hand.
        target: ExprIr,
        /// Originating skill instance.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_use_virtual_card(player, card_kind, skill)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseUseVirtualCard {
        /// Virtual card user.
        player: ExprIr,
        /// Stable card-kind tag.
        kind: i32,
        /// Originating skill instance.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect increase_current_sha_base_damage(player, card, amount)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectIncreaseCurrentShaBaseDamage {
        /// Current Sha user.
        player: ExprIr,
        /// Physical card backing the current Sha.
        card: ExprIr,
        /// Amount added to the saved base damage.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect skip_next_play_phase(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectSkipNextPlayPhase {
        /// Current turn player whose upcoming Play phase is skipped.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_other_draw_or_recover(player, draw_count, recover_count, times)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseOtherDrawOrRecover {
        /// Player choosing a different living player on each iteration.
        player: ExprIr,
        /// Cards drawn by the selected target's draw branch.
        draw_count: ExprIr,
        /// HP recovered by the selected target's recovery branch.
        recover_count: ExprIr,
        /// Number of independently cancellable target choices.
        times: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_hand_equip_then_remove_self_skill(player, count)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseDiscardHandEquipThenRemoveSelfSkill {
        /// Player making the forced card choice.
        player: ExprIr,
        /// Requested discard count before capping to available cards.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect draw_selected_targets(count)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDrawSelectedTargets {
        /// Cards drawn by each active-skill target.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect recover_if_min_current_hp(player, amount)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRecoverIfMinCurrentHp {
        /// Potential recovery recipient.
        player: ExprIr,
        /// Maximum HP recovered.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect increase_current_card_base_damage(player, card, amount)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectIncreaseCurrentCardBaseDamage {
        /// Current damaging-card user.
        player: ExprIr,
        /// Physical card backing the current use.
        card: ExprIr,
        /// Amount added to the saved base damage.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect lose_hp_then_damage_other_lose_max_hp_if_dying(player, target, amount, skill)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectLoseHpThenDamageOtherLoseMaxHpIfDying {
        /// Skill owner who first loses HP and later sources the damage.
        player: ExprIr,
        /// Other player damaged after the owner's dying window resolves.
        target: ExprIr,
        /// Shared HP-loss and damage amount.
        amount: ExprIr,
        /// Originating active skill instance.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect recover_selected_targets(amount)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRecoverSelectedTargets {
        /// HP recovered by each active-skill target.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect record_selected_play_phase_category(skill)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRecordSelectedPlayPhaseCategory {
        /// Originating active skill instance.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect take_target_hand_then_repeat_until_max(player, target, skill)
    ///
    /// The first target is mandatory and every later larger-hand target choice is optional.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectTakeTargetHandThenRepeatUntilMax {
        /// Skill owner choosing and gaining cards.
        player: ExprIr,
        /// Mandatory target selected by the active-skill command.
        target: ExprIr,
        /// Originating active skill instance.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_draw_or_increase_current_card_damage_or_both_mark(
    /// player, card, draw_count, damage_amount, mark_def, mark_count)
    ///
    /// The third branch performs both earlier branches and then adds the requested mark.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
        /// Current card user and skill owner.
        player: ExprIr,
        /// Physical card backing the current use.
        card: ExprIr,
        /// Cards drawn by the first and third branches.
        draw_count: ExprIr,
        /// Base damage added by the second and third branches.
        damage_amount: ExprIr,
        /// Stable mark definition added by the third branch.
        mark_def: u32,
        /// Mark count added by the third branch.
        mark_count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect draw_then_choose_give_or_discard_hand(player, draw_count, payment_count)
    ///
    /// Draw first, then choose to give the exact hand-card count to another living player or
    /// discard that many hand cards. Appended to keep existing serialized enum ordinals stable.
    EffectDrawThenChooseGiveOrDiscardHand {
        /// Skill owner drawing and paying cards.
        player: ExprIr,
        /// Cards drawn before the branch choice.
        draw_count: ExprIr,
        /// Exact hand-card payment count when available.
        payment_count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect gain_target_equipment(player, target, slot)
    ///
    /// Gain every card currently occupying the requested equipment slot on `target`.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGainTargetEquipment {
        /// Player gaining the equipment.
        player: ExprIr,
        /// Player whose matching equipment is gained.
        target: ExprIr,
        /// Stable equipment-slot tag (`1` weapon through `5` treasure).
        slot: u8,
        /// Source span.
        span: Span,
    },
    /// effect discard_all_target_equipment(actor, target)
    ///
    /// Discard every equipment card that `actor` may currently discard from `target`.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDiscardAllTargetEquipment {
        /// Player causing the discard.
        actor: ExprIr,
        /// Player whose discardable equipment is discarded.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect increase_current_sha_target_damage(source, card, target, amount)
    ///
    /// Increase only the specified target's damage for the currently resolving Sha.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectIncreaseCurrentShaTargetDamage {
        /// Current Sha user.
        source: ExprIr,
        /// Physical card backing the current Sha.
        card: ExprIr,
        /// Current Sha target.
        target: ExprIr,
        /// Additional damage for this target.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_draw_then_owner_if_below_max_hp(player, count)
    ///
    /// Choose any living player to draw `count` cards, then draw one for the chooser if the
    /// selected target's hand remains below their maximum HP.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseTargetDrawThenOwnerIfBelowMaxHp {
        /// Skill owner making the target choice.
        player: ExprIr,
        /// Cards drawn by the selected target.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_bound_source_attack_range_damage(player, source, amount)
    ///
    /// Let `player` choose one living player in the bound `source` player's attack range, then
    /// deal normal damage from `source` to the selected player.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseBoundSourceAttackRangeDamage {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Bound damage source whose attack range limits the target choice.
        source: ExprIr,
        /// Damage amount.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_draw_max_hp_cap_then_discard_excess(player, cap)
    ///
    /// Choose any living player with positive maximum HP. That target draws the lesser of their
    /// current maximum HP and `cap`, then discards excess hand cards down to that same
    /// freshly-evaluated limit after nested draw triggers finish.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseTargetDrawMaxHpCapThenDiscardExcess {
        /// Skill owner making the target choice, including a dead owner resolving a death trigger.
        player: ExprIr,
        /// Inclusive upper bound applied to the selected target's maximum HP.
        cap: u32,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_draw_to_max_hp_cap_then_add_skill(player, cap, skill_def)
    ///
    /// Choose any living player with positive maximum HP. That target draws the lesser of their
    /// current maximum HP and `cap`, then gains the skill defined by `skill_def` after the draw
    /// finishes. ABI 455 (C 段 omp, `beizhan`). Appended to keep existing ordinals stable.
    EffectChooseTargetDrawMaxHpCapThenAddSkill {
        /// Skill owner making the target choice.
        player: ExprIr,
        /// Inclusive upper bound applied to the selected target's maximum HP.
        cap: u32,
        /// Skill definition granted to the selected target after the draw.
        skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_heart_redirect_current_damage(player, draw_skill_def)
    ///
    /// In a `damage_before` trigger, discard a heart hand card, choose another living player and
    /// redirect the current damage to them (cancel + re-deal with the same source/amount/nature/
    /// card), granting `draw_skill_def` to the new target. ABI 456 (C 段 omp, `tianxiang`).
    EffectResolveTianxiang {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Skill granted to the redirect target after the damage settles.
        draw_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_heart_cancel_damage_branch(player)
    ///
    /// In a `damage_before` trigger, discard a heart hand card, choose another living player,
    /// cancel the current damage, then the owner chooses: the target takes 1 damage from the
    /// damage source and draws min(5, lost HP), or loses 1 HP and gains the discarded heart card.
    /// ABI 457 (C 段 omp, `retianxiang`).
    EffectResolveRetianxiang {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_link_other_same_hp(player)
    ///
    /// In the owner's prepare phase, choose any number of other living players with the same HP
    /// as the owner who are not linked, and link them. ABI 458 (C 段 omp, `lianzhou`).
    EffectResolveLianzhou {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_give_hand_equip_to_event_player(player)
    ///
    /// Choose any number of discardable hand/equipment cards and give them to the triggering
    /// event player. ABI 459 (C 段 omp, `stddechong`).
    EffectResolveStddechong {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect discard_hand_equip_count(player, count)
    ///
    /// Choose exactly `count` discardable hand/equipment cards and discard them (generic
    /// discard-cost primitive). ABI 460 (C 段 omp, `yangzhong`).
    EffectDiscardHandEquipCount {
        /// Player discarding the cards.
        player: ExprIr,
        /// Number of cards to discard.
        count: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_jieliang_gain(player)
    ///
    /// When the current phase player discards cards during their discard phase, choose one
    /// of those cards still in the discard pile and gain it. ABI 461 (C 段 omp, `jieliang`).
    EffectResolveJieliangGain {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_other_gain_event_cards(player)
    ///
    /// Choose another living player; they gain the triggering event's used cards. ABI 462
    /// (C 段 omp, `stdjujian`).
    EffectResolveStdjujian {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_lulve(player)
    ///
    /// Choose another player with fewer hand cards; they choose: give all hand cards and flip
    /// the owner, or flip themselves and use a virtual Sha on the owner. ABI 463 (C 段 omp,
    /// `lulve`).
    EffectResolveLulve {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dctingxian(player)
    ///
    /// When the owner's Sha is resolving, draw (equipment count + 1) cards and exclude up to
    /// that many of the Sha's targets. ABI 464 (C 段 omp, `dctingxian`).
    EffectResolveDctingxian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_other_add_skill(player, skill_def)
    ///
    /// Choose another living player and grant them the given skill definition. ABI 465
    /// (C 段 omp, `hfjieying`).
    EffectResolveHfjieying {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Temp skill definition granted to the recipient.
        grant_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dcnuchen(player)
    ///
    /// Reveal one of a target's hand cards, then discard same-suit cards for equal damage
    /// or gain the target's same-suit hand. ABI 466 (C 段 omp, `dcnuchen`).
    EffectResolveDcnuchen {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dcdanji(player, skill)
    ///
    /// Awakening resolution: remove the skill, lose 1 maxHp, then recover+draw to the new
    /// maxHp and gain mashu + dcnuchen. ABI 466 (C 段 omp, `dcdanji`).
    EffectResolveDcdanji {
        /// Skill owner.
        player: ExprIr,
        /// Awakening skill instance to remove.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dczhiheng(player)
    ///
    /// Discard any number of hand/equip cards, draw the same count, plus one if every hand
    /// card was discarded. ABI 467 (C 段 omp, `dczhiheng`).
    EffectResolveDczhiheng {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_stdgaobian(player)
    ///
    /// At another player's end phase, when exactly one player took damage this turn, that
    /// player uses a virtual Sha or resets the owner's Anran counter. ABI 468
    /// (C 段 omp, `stdgaobian`).
    EffectResolveStdgaobian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spjiedao(player)
    ///
    /// Before the owner's first damage of the turn, optionally boost the damage by X and
    /// discard X hand/equip cards. ABI 469 (C 段 omp, `spjiedao`).
    EffectResolveSpjiedao {
        /// Skill owner and damage source.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_sibian(player)
    ///
    /// At the draw phase, reveal the top 4 pile cards, gain the max/min-numbered ones and
    /// give the rest to a minimum-hand player; the draw is skipped. ABI 470
    /// (C 段 omp, `sibian`).
    EffectResolveSibian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect reveal_choose_category_distribute_self_threshold_hp_loss(player, reveal_count, recipient_cap)
    ///
    /// Reveal draw-pile cards, choose one card category, distribute every matching card with a
    /// per-recipient cap, then make recipients whose share is at least the owner's lose 1 HP.
    /// ABI 491 (TW `twxingluan`).
    EffectRevealChooseCategoryDistributeSelfThresholdHpLoss {
        /// Skill owner.
        player: ExprIr,
        /// Number of cards to reveal.
        reveal_count: u8,
        /// Maximum number of distributed cards each recipient may receive.
        recipient_cap: u8,
        /// Source span.
        span: Span,
    },
    /// effect choose_other_random_gain_region_then_draw_compare_color_lose_hp(player)
    ///
    /// Choose another living player with a card in hand/equipment/judgment, gain one
    /// authoritative-random card from those regions, draw one, then lose 1 HP when the two
    /// cards have different colors. ABI 492 (DIY `nsdingzhou`).
    EffectChooseOtherRandomGainRegionThenDrawCompareColorLoseHp {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_refuhai(player)
    ///
    /// Collect every other living player's private binary choice in seat order, reveal the
    /// results together, and draw for the initial equal run. ABI 493 (mobile `refuhai`).
    EffectResolveRefuhai {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_gain_target_hand_ceil_half_difference(player, target)
    ///
    /// Discard the active invocation's selected hand card, then visibly gain half (rounded up)
    /// of the target's post-cost hand-count advantage. ABI 494 (offline `nsyangwu`).
    EffectChooseGainTargetHandCeilHalfDifference {
        /// Skill owner.
        player: ExprIr,
        /// Other target whose hand was larger before paying the cost.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_gzjinfa(player, target)
    ///
    /// Pay the selected hand/equipment card, then let the target give equipment or lose one
    /// gainable card; a spade gift makes the target use a virtual Sha. ABI 495.
    EffectResolveGzjinfa {
        /// Skill owner.
        player: ExprIr,
        /// Other target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wn_qifeng(player, target)
    ///
    /// Use a distance/count-free virtual Sha on the ending-turn player with damage equal to the
    /// number of cards the owner lost this turn. ABI 496.
    EffectResolveWnQifeng {
        /// Skill owner.
        player: ExprIr,
        /// Ending-turn player.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 杨彪「举讹」 from the current End-phase player. ABI 583.
    EffectResolveDcjue {
        /// 结算「厥」的角色。
        player: ExprIr,
        /// 结束阶段角色。
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 阮籍「昭文/酒遁」. ABI 584.
    EffectResolveRuanji {
        /// Skill owner.
        player: ExprIr,
        /// Event source, or NONE.
        source: ExprIr,
        /// Triggering card, or NONE.
        card: ExprIr,
        /// Public action: 0 昭文 setup, 1 red-card draw, 2 酒 rewrite, 3 酒遁 response.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 崔琰毛玠「征辟」. ABI 585.
    EffectResolveZhengbi {
        /// Skill owner.
        player: ExprIr,
        /// 征辟目标。
        target: ExprIr,
        /// Public action stage.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 崔琰毛玠「奉迎」. ABI 585.
    EffectResolveFengying {
        /// Skill owner.
        player: ExprIr,
        /// 奉迎辅助技能定义。
        helper_def: ExprIr,
        /// Public action stage.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect set_draw_phase_bottom(player)：寸目（许攸）当前摸牌阶段从牌堆底摸牌。ABI 589。
    EffectSetDrawPhaseFromBottom {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect draw_cards_from_bottom(player, count)：寸目（许攸）从牌堆底摸牌。ABI 589。
    EffectDrawFromBottom {
        /// Player drawing.
        player: ExprIr,
        /// Card count.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_chenglve(player, stage)：成略（许攸）转换技状态机。ABI 589。
    EffectResolveChenglve {
        /// Skill owner.
        player: ExprIr,
        /// Choice barrier stage: 0 flip+draw, 1 discard, 2 commit suits.
        stage: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_shicai(player, card, stage)：恃才（许攸）放置牌堆顶状态机。ABI 589。
    EffectResolveShicai {
        /// Skill owner.
        player: ExprIr,
        /// 使用牌。
        card: ExprIr,
        /// Choice barrier stage: 0 check+order, 1 place+draw.
        stage: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_sbbiyue(player)：闭月（界貂蝉）结束阶段摸牌。ABI 599。
    EffectResolveSbbiyue {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_sxrmqiewang(player, event_player)：窃王（sxrm 严畯）距离内摸牌。ABI 596。
    EffectResolveSxrmqiewang {
        /// Skill owner.
        player: ExprIr,
        /// 受伤角色。
        event_player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_sxrmqianliu(player, stage)：潜流（sxrm 伏皇后）牌堆底四张。ABI 596。
    EffectResolveSxrmqianliu {
        /// Skill owner.
        player: ExprIr,
        /// 使用者。
        source: ExprIr,
        /// 0=取牌堆底判定.
        stage: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_gzbuqu(player, delta, stage)：旧不屈（旧周泰）创牌状态机。ABI 591。
    EffectResolveGzbuqu {
        /// Skill owner.
        player: ExprIr,
        /// 本次体力变化量（负数 = 流失）。
        delta: ExprIr,
        /// 0=置入创牌并判定免死.
        stage: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_gzbuqu_recover(player, stage)：旧不屈恢复移除创牌。ABI 591。
    EffectResolveGzbuquRecover {
        /// Skill owner.
        player: ExprIr,
        /// 0=选择移除, 1=移除+救回判定.
        stage: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xiongluan(player, target, stage)：雄乱（张绣）限定技状态机。ABI 590。
    EffectResolveXiongluan {
        /// Skill owner.
        player: ExprIr,
        /// 限定目标。
        target: ExprIr,
        /// 1=施加限制（废除装备栏与判定区、目标禁用手牌）。
        stage: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_twjuxiang(player, event_player, stage)：踞襄（张绣）主公技状态机。ABI 590。
    EffectResolveTwjuxiang {
        /// Skill owner.
        player: ExprIr,
        /// 触发角色（其他群势力角色出牌阶段）。
        event_player: ExprIr,
        /// 0=装备牌选择, 1=置入或获得.
        stage: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 傅干「巧谏」. ABI 586.
    EffectResolveQiaojian {
        /// Skill owner.
        player: ExprIr,
        /// 巧谏目标。
        target: ExprIr,
        /// 巧谏辅助技能定义。
        helper_def: ExprIr,
        /// Public action stage.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 傅干「析察」. ABI 586.
    EffectResolveXicha {
        /// Skill owner.
        player: ExprIr,
        /// 事件来源。
        source: ExprIr,
        /// 触发牌。
        card: ExprIr,
        /// Public action stage.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 胡遵「斩涛 / 安境」. ABI 587.
    EffectResolveHuzun {
        /// Skill owner.
        player: ExprIr,
        /// Damage source for 斩涛; none for 安境.
        source: ExprIr,
        /// Damage card for 斩涛; none for 安境.
        card: ExprIr,
        /// 0=斩涛, 1=安境.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 诸葛梦雪「寄春 / 寒英」. ABI 588.
    EffectResolveZhugemengxue {
        /// Skill owner.
        player: ExprIr,
        /// 0=寄春, 10=寒英.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 柏灵筠「灵慧 / 黠策」. ABI 593.
    EffectResolveBailingyun {
        /// Skill owner.
        player: ExprIr,
        /// 0=灵慧, 10=造成伤害后黠策, 11=受到伤害后黠策.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 威关银屏「绍威」. ABI 594.
    EffectResolveDcshaowei {
        /// Skill owner.
        player: ExprIr,
        /// 0=失牌花色分支，1=回复后恢复装备栏.
        action: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 威关银屏「涤仇」. ABI 594.
    EffectResolveDcdichou {
        /// Skill owner.
        player: ExprIr,
        /// 0=回合开始，10=回合结束.
        action: ExprIr,
        /// Temporary modifier skill definition.
        helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 凌操「独锋」. ABI 595.
    EffectResolveDcdufeng {
        /// Skill owner.
        player: ExprIr,
        /// 0=choose costs, 1=draw and grant turn modifiers.
        action: ExprIr,
        /// Temporary modifier skill definition.
        helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect skip_next_discard_phase(player)
    ///
    /// Skip the target's discard phase this turn. ABI 471 (C 段 omp, `spfengyin`).
    EffectSkipNextDiscardPhase {
        /// Target whose discard phase is skipped.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spfengyin(player, target)
    ///
    /// Give one Sha from the owner's hand to the target and skip its play + discard
    /// phases. ABI 471 (C 段 omp, `spfengyin`).
    EffectResolveSpfengyin {
        /// Skill owner.
        player: ExprIr,
        /// Other player whose turn is being sealed.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xinfu_lianpian(player)
    ///
    /// When the owner uses a card sharing a target with the previous card this play phase,
    /// draw one and optionally give it to a shared target. ABI 472 (C 段 omp,
    /// `xinfu_lianpian`).
    EffectResolveXinfuLianpian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_gongjian(player)
    ///
    /// For every target shared with the previous Sha this turn, discard 1-2 of its
    /// hand/equip cards, then gain the discarded Sha cards. ABI 473 (C 段 omp,
    /// `gongjian`).
    EffectResolveGongjian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xinfu_qinguo(player)
    ///
    /// After using an equipment card, optionally use a virtual Sha against another player.
    /// ABI 474 (C 段 omp, `xinfu_qinguo`).
    EffectResolveXinfuQinguo {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dcmanzhi(player)
    ///
    /// At Prepare/End, choose one of two branches: give (target hands over up to two cards
    /// and uses a virtual Sha), or gain (gain up to two zone cards, give back equal, draw).
    /// Each branch once per turn via skill marks. ABI 475 (C 段 omp, `dcmanzhi`).
    EffectResolveDcmanzhi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xinxingluan(player)
    ///
    /// After a play-phase card use, once per turn, choose one of three rank-6 branches.
    /// ABI 476 (C 段 omp, `xinxingluan`).
    EffectResolveXinxingluan {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dckuizhen(player)
    ///
    /// Choose a target with >= hand or >= hp; the target uses a Duel on the owner; on
    /// owner damage gain the target's hand Shas, else the target loses 1 HP. ABI 477
    /// (C 段 omp, `dckuizhen`).
    EffectResolveDckuizhen {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dckuizhen_resolve(player, winner, loser)
    ///
    /// DuelResolved outcome for 溃阵: owner lost -> gain the initiator's hand Shas;
    /// owner won -> the initiator loses 1 HP. ABI 477 (C 段 omp, `dckuizhen`).
    EffectResolveDckuizhenResolve {
        /// Skill owner.
        player: ExprIr,
        /// Duel winner.
        winner: ExprIr,
        /// Duel loser.
        loser: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_nsdingzhou(player)
    ///
    /// Gain one of a target's hand/equip/judge cards (first, deterministic), draw one,
    /// lose 1 HP if the colors differ. ABI 478 (C 段 omp, `nsdingzhou`).
    EffectResolveNsdingzhou {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_sxrmlanjiao(player)
    ///
    /// Loop: view two of a target's hand cards; the target may lose 1 HP and draw to
    /// retry; on stop the owner gains the last viewed cards. ABI 479 (C 段 omp,
    /// `sxrmlanjiao`).
    EffectResolveSxrmlanjiao {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_rekuanshi(player)
    ///
    /// End-phase target choice marking the watched player. ABI 480 (C 段 omp,
    /// `rekuanshi`).
    EffectResolveRekuanshi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_rekuanshi_resolve(player, damaged)
    ///
    /// When the watched player takes their second damage this turn, recover 1 HP.
    /// ABI 480 (C 段 omp, `rekuanshi`).
    EffectResolveRekuanshiResolve {
        /// Skill owner.
        player: ExprIr,
        /// The damaged (watched) player.
        damaged: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dcmoyu(player)
    ///
    /// Gain one (or two) of a target's hand/equip/judge cards; the target uses a virtual
    /// Sha; on owner damage ban the skill this turn, else the next gain is bigger. ABI 481
    /// (C 段 omp, `dcmoyu`).
    EffectResolveDcmoyu {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xizhen(player)
    ///
    /// At play-phase start, choose a target and use a virtual Sha or Duel on it.
    /// ABI 482 (C 段 omp, `xizhen`).
    EffectResolveXizhen {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xuxie(player)
    ///
    /// Lose 1 maxHp at play start; adjacent (distance <= 1) players either each discard
    /// one card or each draw one. ABI 483 (C 段 omp, `xuxie`).
    EffectResolveXuxie {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xuxie_add(player)
    ///
    /// Restore 1 maxHp at play end when someone's maxHp exceeds the owner's.
    /// ABI 483 (C 段 omp, `xuxie`).
    EffectResolveXuxieAdd {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dcchiying(player)
    ///
    /// Let the selected target repeatedly discard cards from distinct players in its attack
    /// range, then gain the discarded basic cards. ABI 484 (C 段 omp, `dcchiying`).
    EffectResolveDcchiying {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_jsrgchiying(player, target)
    ///
    /// Every other player in the target's attack range discards one hand/equipment card in
    /// seat order. If the discarded basic-card count does not exceed the target's HP, the
    /// target gains the discarded basic cards. ABI 501 (`jsrgchiying`).
    EffectResolveJsrgchiying {
        /// Skill owner.
        player: ExprIr,
        /// Active-skill target whose attack range determines the discarders.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xinkuangfu(player)
    ///
    /// Discard one equipment of a chosen player (self allowed), use a virtual Sha, then
    /// branch on damage: self-equip + damage -> draw 2; other-equip + no damage ->
    /// discard 2 hand cards. ABI 485 (C 段 omp, `xinkuangfu`).
    EffectResolveXinkuangfu {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dcxiangshu(player, target)
    ///
    /// At another player's play start (hand >= hp), choose a secret number 0-5; at their
    /// play end, resolve |hand - number|: <=1 gain one card, ==0 deal 1 damage.
    /// ABI 486 (C 段 omp, `dcxiangshu`).
    EffectResolveDcxiangshu {
        /// Skill owner.
        player: ExprIr,
        /// The observed player.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dcxiangshu_lottery(player, target)
    ///
    /// Second phase: the number-guess payoff at the observed player's play end.
    /// ABI 486 (C 段 omp, `dcxiangshu`).
    EffectResolveDcxiangshuLottery {
        /// Skill owner.
        player: ExprIr,
        /// The observed player.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_shuangxiong(player)
    ///
    /// Draw-phase replacement: judge, gain the judgment card, record its color, and arm the
    /// opposite-color hand-card view-as Duel for the turn. ABI 511 (C 段 omp, `shuangxiong`).
    EffectResolveShuangxiong {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_zhijian(player)
    ///
    /// Play-phase: give one selected hand equipment card to another player who can equip it,
    /// then draw one. ABI 512 (C 段 omp, `zhijian`).
    EffectResolveZhijian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_guzheng(player, event_player)
    ///
    /// After another player's discard phase: return one card they discarded into the discard
    /// pile and optionally take the rest. ABI 513 (C 段 omp, `guzheng`).
    EffectResolveGuzheng {
        /// Skill owner.
        player: ExprIr,
        /// The player whose discard phase ended.
        event_player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_zhiba(player, actor)
    ///
    /// Lord skill: a Wu-faction player may start a hand comparison with the owner; the owner
    /// may refuse when awakened and may take both comparison cards when not winning.
    /// ABI 523 (C 段 omp, `zhiba`).
    EffectResolveZhiba {
        /// Skill owner and lord.
        player: ExprIr,
        /// The Wu-faction comparison initiator.
        actor: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_drlt_congjian(player)
    ///
    /// As a target of a multi-target trick, give one hand/equipment card to another target
    /// and draw. ABI 524 (C 段 omp, `drlt_congjian`).
    EffectResolveDrltCongjian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_nzry_feijun(player)
    ///
    /// Play-phase: discard one hand/equipment card, then either a hand-more player gives the
    /// owner one card or an equip-more player discards one equipment; arms 兵略 for a first
    /// time target. ABI 526 (C 段 omp, `nzry_feijun`).
    EffectResolveNzryFeijun {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_drlt_yongsi(player, stage)
    ///
    /// Draw-phase replacement with the living faction count, and play-phase-end settlement
    /// by turn damage. ABI 527 (C 段 omp, `drlt_yongsi`).
    EffectResolveDrltYongsi {
        /// Skill owner.
        player: ExprIr,
        /// Zero = draw replacement, one = play-phase-end settlement.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_drlt_weidi(player)
    ///
    /// Lord skill: at the start of the owner's Discard phase, give up to
    /// min(needed, other-Qun count) hand cards to equal other Qun players.
    /// ABI 528 (C 段 omp, `drlt_weidi`).
    EffectResolveDrltWeidi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect give_selected_then_target_choose_discard_same_suit_or_lose_hp(player, target)
    ///
    /// Give the single active-skill hand-card input to `target`. After the movement and nested
    /// triggers finish, that target chooses either to reveal their hand and discard every
    /// discardable hand/equipment card matching the given card's physical suit, or lose 1 HP.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
        /// Skill owner giving the selected hand card.
        player: ExprIr,
        /// Other living recipient who makes the follow-up choice.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect gain_equipment_from_draw_pile(player)
    ///
    /// Move the topmost equipment card in the shared draw pile into `player`'s hand.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGainEquipmentFromDrawPile {
        /// Player gaining the equipment card.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect give_all_hand_recover_to_one(player, target)
    ///
    /// Give every current hand card from `player` to `target`, then recover `target` to exactly
    /// one HP after the movement and all nested triggers finish.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGiveAllHandRecoverToOne {
        /// Living skill owner giving every hand card.
        player: ExprIr,
        /// Other living dying recipient.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(player)
    ///
    /// Let `player` choose any living target able to discard one of their own hand/equipment
    /// cards. That target chooses and discards exactly one such card. After the movement and all
    /// nested triggers finish, `player` loses one HP when the target's hand count differs from
    /// their HP. Appended to keep existing serialized enum ordinals stable.
    EffectChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
        /// Living skill owner making the target choice.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_give_one_hand_equip_or_recover(owner, giver, target)
    ///
    /// Let `giver` optionally give one current hand/equipment card to `target`; if no card is
    /// given, recover `target` by one HP. `owner` identifies the originating skill owner.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseGiveOneHandEquipOrRecover {
        /// Originating skill owner.
        owner: ExprIr,
        /// Player making the card choice and giving the card.
        giver: ExprIr,
        /// Gift recipient or recovery target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect replace_skill_with_bound_after_resolution_if_target_alive(
    ///     player, skill, skill_def, target
    /// )
    ///
    /// After the current effect, trigger, and dying/save resolution finishes, replace `skill`
    /// with a target-bound helper only when `target` is still alive. Appended to keep existing
    /// serialized enum ordinals stable.
    EffectReplaceSkillWithBoundAfterResolutionIfTargetAlive {
        /// Skill owner.
        player: ExprIr,
        /// Skill instance being replaced.
        skill: ExprIr,
        /// Target-bound helper skill definition.
        skill_def: u32,
        /// Target whose life state is checked after the resolution barrier.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(player)
    ///
    /// Force `player` to choose a nonempty hand or equipment zone, discard every
    /// discardable card in that zone, draw the discarded count plus current lost HP after
    /// movement triggers finish, then lose 1 HP after draw triggers finish. Appended to keep
    /// existing serialized enum ordinals stable.
    EffectChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_hand_equip_then_recover(player, count, amount)
    ///
    /// Force `player` to choose and discard exactly `count` current hand/equipment cards,
    /// then recover `amount` HP after movement triggers finish. Appended to keep existing
    /// serialized enum ordinals stable.
    EffectChooseDiscardHandEquipThenRecover {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Exact discard count.
        count: ExprIr,
        /// Maximum HP recovered after the discard resolves.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_targets_damage(player, min_count, max_count, scope, amount)
    ///
    /// Force `player` to choose a bounded number of distinct living targets in `scope`, then
    /// deal `amount` normal damage to each in seat order. Appended to keep existing serialized
    /// enum ordinals stable.
    EffectChooseTargetsDamage {
        /// Player making the target choice and dealing the damage.
        player: ExprIr,
        /// Minimum number of targets.
        min_count: u8,
        /// Maximum number of targets.
        max_count: u8,
        /// Canonical target scope.
        scope: DrawTargetScopeIr,
        /// Damage dealt to each selected target.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect discard_target_equipment_draw_weapon_range(player, target)
    ///
    /// Force `player` to choose and discard one current discardable equipment card from
    /// `target`; after movement triggers finish, a discarded weapon makes `target` draw cards
    /// equal to its physical attack range. Appended for serialized ordinal stability.
    EffectDiscardTargetEquipmentDrawWeaponRange {
        /// Skill owner choosing the equipment card.
        player: ExprIr,
        /// Living target whose equipment is discarded and who may draw.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_reveal_hand_damage_if_sha(player, amount)
    ///
    /// Force `player` to choose a living target with hand cards, publicly reveal that target's
    /// complete current hand, then deal normal damage from `player` iff at least one revealed
    /// card is an effective Sha for the target. Appended for serialized ordinal stability.
    EffectChooseTargetRevealHandDamageIfSha {
        /// Skill owner choosing the target and dealing damage.
        player: ExprIr,
        /// Damage amount when the revealed hand contains an effective Sha.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_other_give_one_hand_equip(player)
    ///
    /// Force `player` to choose another living player, then give that target exactly one current
    /// hand/equipment card. Appended for serialized ordinal stability.
    EffectChooseOtherGiveOneHandEquip {
        /// Skill owner choosing the target and card.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_reveal_hand_card_bind_category_target(player, target)
    ///
    /// Force `player` to choose and publicly reveal one current hand card from `target`, then
    /// remember both that target and the card's logical category for the current Play phase.
    /// Appended for serialized ordinal stability.
    EffectChooseRevealHandCardBindCategoryTarget {
        /// Skill owner choosing and revealing the card.
        player: ExprIr,
        /// Current Play-phase player whose hand card is revealed.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect recover_to_max_then_optional_exchange_skill(player, removable_skill_def)
    ///
    /// Consume the originating limited skill, recover `player` to max HP, then, after recovery
    /// and nested triggers finish, optionally exchange `removable_skill_def` for a fresh instance
    /// of the originating skill. Appended for serialized ordinal stability.
    EffectRecoverToMaxThenOptionalExchangeSkill {
        /// Skill owner recovering and choosing the exchange.
        player: ExprIr,
        /// Skill definition sacrificed when the limited skill is restored.
        removable_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect reveal_hand_discard_kind(player, card_kind)
    ///
    /// Publicly reveal the player's current hand, then discard every discardable hand card whose
    /// effective kind matches `card_kind`. Appended for serialized ordinal stability.
    EffectRevealHandDiscardKind {
        /// Player revealing and discarding cards.
        player: ExprIr,
        /// Stable logical card-kind tag.
        kind: i32,
        /// Source span.
        span: Span,
    },
    /// effect choose_move_target_equipment_and_or_draw(player, target)
    ///
    /// Let the owner cancel, draw one, move one compatible equipment card from `target`, or
    /// perform both available actions in move-then-draw order.
    /// Appended for serialized ordinal stability.
    EffectChooseMoveTargetEquipmentAndOrDraw {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Other player whose turn ended.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(
    /// player, target)
    ///
    /// Let a direct-Sha damage target discard up to the source's current attack range from
    /// hand/equipment or increase that damage by one. A physical weapon forces both branches
    /// and is discarded afterward.
    /// Appended for serialized ordinal stability.
    EffectChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
        /// Skill owner and Sha source.
        player: ExprIr,
        /// Bound Sha damage target and chooser.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_draw_or_recast_add_skill(player, draw_skill_def, recast_skill_def)
    ///
    /// Let the player cancel, draw one and gain `draw_skill_def`, or recast one hand/equipment
    /// card and gain `recast_skill_def`.
    /// Appended for serialized ordinal stability.
    EffectChooseDrawOrRecastAddSkill {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Temporary skill granted before drawing.
        draw_skill_def: u32,
        /// Temporary skill granted before recasting.
        recast_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect choose_gain_sha_for_opening_extra_play(player)
    ///
    /// During the originating skill's turn-opening Play phase, let the owner gain the first Sha
    /// from the draw pile or discard pile and arm its phase-end consequence.
    /// Appended for serialized ordinal stability.
    EffectChooseGainShaForOpeningExtraPlay {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect finish_opening_extra_play_sha_gain(player)
    ///
    /// Clear the originating skill's armed exact-card policy and damage the owner when they dealt
    /// no damage since accepting the gain.
    /// Appended for serialized ordinal stability.
    EffectFinishOpeningExtraPlayShaGain {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect reveal_target_hand_gain_shan_armor_or_draw_two(player, target)
    ///
    /// Publicly reveal the target's hand. Give every effective Shan and physical armor to the
    /// skill owner; if none match, the owner draws two.
    /// Appended for serialized ordinal stability.
    EffectRevealTargetHandGainShanArmorOrDrawTwo {
        /// Skill owner and recipient.
        player: ExprIr,
        /// Other living player whose hand is revealed.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect give_all_hand(player, target)
    ///
    /// Give every current hand card from `player` to `target`.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGiveAllHand {
        /// Living player giving every hand card.
        player: ExprIr,
        /// Other living recipient.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect gain_card_if_discard(player, card)
    ///
    /// Move `card` from the shared discard pile into `player`'s hand if it is still there.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGainCardIfDiscard {
        /// Living recipient.
        player: ExprIr,
        /// Physical event card.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_draw_or_discard_target_card_record_sha(player, card, target)
    ///
    /// Let the current Sha user cancel, draw one, or discard one target hand/equipment card.
    /// Accepting either branch records the exact Sha/target for a later dodge consequence.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseDrawOrDiscardTargetCardRecordSha {
        /// Sha user and chooser.
        player: ExprIr,
        /// Exact resolving Sha.
        card: ExprIr,
        /// Current Sha target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect reveal_hand_to_player(player, observer)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRevealHandToPlayer {
        /// Player whose current hand is revealed.
        player: ExprIr,
        /// Player authorized to see that hand.
        observer: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_optional_discard_target_card(
    ///     player,
    ///     target,
    ///     hand|equip|hand_equip|hand_equip_judge,
    /// )
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseOptionalDiscardTargetCard {
        /// Player choosing the card.
        player: ExprIr,
        /// Player whose scoped card may be discarded.
        target: ExprIr,
        /// Canonical zone scope.
        scope: TargetCardScopeIr,
        /// Source span.
        span: Span,
    },
    /// effect reveal_hand_choose_discard_color(player)
    ///
    /// Publicly reveal the player's current hand, then let that player choose one effective
    /// red/black color present and discard every discardable hand card of that color. A single
    /// present color resolves without a choice. Appended for serialized ordinal stability.
    EffectRevealHandChooseDiscardColor {
        /// Player revealing, choosing, and discarding.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_target_cards_recover_same_category(player, target, count, amount)
    ///
    /// Force `player` to discard up to `count` current hand/equipment cards from `target`.
    /// Recover `amount` HP after movement triggers only when exactly `count` cards were
    /// discarded and their effective categories for `target` are identical. Appended for
    /// serialized ordinal stability.
    EffectChooseDiscardTargetCardsRecoverSameCategory {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Player whose cards are discarded.
        target: ExprIr,
        /// Maximum discard count.
        count: ExprIr,
        /// Maximum HP recovered after a matching discard.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_scskuiji_discard(player, target, count): chooser picks up to `count`
    /// distinct-effective-suit cards from its own and `target`'s hands combined; all are discarded.
    EffectChooseScskuijiDiscard {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Other player whose hand also participates.
        target: ExprIr,
        /// Maximum number of cards chosen.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_pskuiji_discard(player, target, count): like `choose_scskuiji_discard`, but
    /// when owner and target discard different counts, the under-discarder loses 1 HP.
    EffectChoosePskuijiDiscard {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Other player whose hand also participates.
        target: ExprIr,
        /// Maximum number of cards chosen.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_compare_selected_hand_card_second_failure_seat_damage(
    ///     player, target, draw_count, failure_count, lose_hp, damage
    /// )
    ///
    /// The active selection supplies the owner's comparison card. Strict victory chooses between
    /// drawing and discarding one target hand/equipment card. At the configured non-win
    /// threshold, the owner loses HP and damages the shorter seat path between the first two
    /// failed targets. Appended for serialized ordinal stability.
    EffectChooseCompareSelectedHandCardSecondFailureSeatDamage {
        /// Skill owner and first comparison participant.
        player: ExprIr,
        /// Other comparison participant.
        target: ExprIr,
        /// Strict-win draw count.
        draw_count: ExprIr,
        /// Non-win threshold.
        failure_count: ExprIr,
        /// HP loss at the threshold.
        lose_hp: ExprIr,
        /// Damage per chosen seat-path player.
        damage: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_other_recover(player, amount)
    ///
    /// Force `player` to choose another living player, then recover that target's HP.
    /// Appended for serialized ordinal stability.
    EffectChooseOtherRecover {
        /// Player making the forced target choice.
        player: ExprIr,
        /// Maximum HP recovered by the selected target.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect enable_equip_slot(player, slot, source)
    ///
    /// Restore one disabled copy of an equipment slot for `player`, attributed to `source`.
    /// Appended for serialized ordinal stability.
    EffectEnableEquipSlot {
        /// Player whose equipment slot is restored.
        player: ExprIr,
        /// Stable slot tag (`1` weapon through `5` treasure).
        slot: ExprIr,
        /// Player causing the restoration.
        source: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect target_choose_suit_gain_owner_hand_damage_if_mismatch(player, target)
    ///
    /// `target` chooses a suit, gains one current hand card of `player`, then takes 1 damage
    /// from `player` when that card's owner-effective suit differs. Appended for serialized
    /// ordinal stability.
    EffectTargetChooseSuitGainOwnerHandDamageIfMismatch {
        /// Skill owner whose hand card is gained.
        player: ExprIr,
        /// Other living player making both choices.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect judge_source_discard_two_hand_or_damage(player, source)
    ///
    /// `player` judges. On a non-heart result, the living `source` may discard exactly two
    /// hand cards; otherwise `player` deals 1 normal damage to `source`. Appended for
    /// serialized ordinal stability.
    EffectJudgeSourceDiscardTwoHandOrDamage {
        /// Damaged skill owner performing the judgment.
        player: ExprIr,
        /// Damage source paying the discard or taking damage.
        source: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect give_selected_recover_after_phase_total_two(player, target)
    ///
    /// Give the active invocation's selected hand cards to `target`; when the current Play-phase
    /// total first reaches two cards, recover `player` by 1 after movement triggers. Appended
    /// for serialized ordinal stability.
    EffectGiveSelectedRecoverAfterPhaseTotalTwo {
        /// Skill owner giving selected hand cards.
        player: ExprIr,
        /// Other living recipient.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_other_hand_targets_gain_one_replace_draw(player)
    ///
    /// During the owner's Draw phase, choose one or two other players with hand cards, gain one
    /// hand card from each in selection order, and replace the normal draw. Appended for
    /// serialized ordinal stability.
    EffectChooseOtherHandTargetsGainOneReplaceDraw {
        /// Skill owner, target chooser, and card chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect distribute_top_two_cards(player)
    ///
    /// Move the draw-pile top two cards into public processing, then repeatedly assign a
    /// nonempty subset to any living player until all are distributed. Appended for serialized
    /// ordinal stability.
    EffectDistributeTopTwoCards {
        /// Skill owner choosing every subset and recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect judge_make_current_sha_target_unrespondable_on_red(player, card, target)
    ///
    /// Appended for serialized ordinal stability.
    EffectJudgeMakeCurrentShaTargetUnrespondableOnRed {
        /// Player performing the judgment.
        player: ExprIr,
        /// Physical Sha currently resolving.
        card: ExprIr,
        /// Current Sha target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_basic_then_target_discard_equip_or_damage(player, target)
    ///
    /// Appended for serialized ordinal stability.
    EffectChooseDiscardBasicThenTargetDiscardEquipOrDamage {
        /// Skill owner paying the optional basic-card cost and dealing damage.
        player: ExprIr,
        /// End-phase player discarding equipment or taking damage.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_current_sha_target_discard_redirect_to_owner(owner, source, card, target)
    ///
    /// Appended for serialized ordinal stability.
    EffectChooseCurrentShaTargetDiscardRedirectToOwner {
        /// Skill owner receiving the redirected Sha.
        owner: ExprIr,
        /// Current Sha user.
        source: ExprIr,
        /// Physical Sha currently resolving.
        card: ExprIr,
        /// Current target paying the optional discard cost.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_gain_bottom_card(player)
    ///
    /// Appended for serialized ordinal stability.
    EffectChooseTargetGainBottomCard {
        /// Skill owner revealing and giving the bottom card.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect judge_black_gain_repeat(player)
    ///
    /// Appended for serialized ordinal stability.
    EffectJudgeBlackGainRepeat {
        /// Judged player and recipient of black results.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect use_virtual_juedou_second_on_first()
    ///
    /// Appended for serialized ordinal stability.
    EffectUseVirtualJuedouSecondOnFirst {
        /// Source span.
        span: Span,
    },
    /// effect recover_hp_as(source, player, amount)
    ///
    /// Appended for serialized ordinal stability.
    EffectRecoverAs {
        /// Recovery source.
        source: ExprIr,
        /// Player recovering HP.
        player: ExprIr,
        /// Amount expression.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_redirect_current_sha(player, source, card)
    ///
    /// Appended for serialized ordinal stability.
    EffectChooseDiscardRedirectCurrentSha {
        /// Current Sha target and skill owner paying the discard cost.
        player: ExprIr,
        /// Current Sha user.
        source: ExprIr,
        /// Physical Sha currently resolving.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect judge_set_skill_mark_to_rank(player, skill)
    ///
    /// Appended for serialized ordinal stability.
    EffectJudgeSetSkillMarkToRank {
        /// Player performing the judgment.
        player: ExprIr,
        /// Originating skill instance whose definition id is used as the mark id.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_one_from_each_linked(player, skill)
    ///
    /// Appended for serialized ordinal stability.
    EffectChooseDiscardOneFromEachLinked {
        /// Skill owner choosing cards.
        player: ExprIr,
        /// Originating skill instance.
        skill: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect disable_skill_by_def_until_turn_end(player, skill_def)
    ///
    /// Appended for serialized ordinal stability.
    EffectDisableSkillByDefUntilTurnEnd {
        /// Skill owner.
        player: ExprIr,
        /// Skill definition to disable temporarily.
        skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect target_discard_then_duel_if_non_sha_and_hp_gte_owner(player, target)
    ///
    /// Appended for serialized ordinal stability.
    EffectTargetDiscardThenDuelIfNonShaAndHpGteOwner {
        /// Skill owner and virtual Duel user.
        player: ExprIr,
        /// Other player choosing and discarding one hand/equipment card.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_target_lose_hp_discard_hand_sha_if_basic(player)
    ///
    /// Appended for serialized ordinal stability.
    EffectChooseTargetLoseHpDiscardHandShaIfBasic {
        /// Skill owner and virtual Sha user.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_optional_targets_draw(player, count, min_count, max_count, scope)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseOptionalTargetsDraw {
        /// Player making the target choice.
        player: ExprIr,
        /// Cards drawn by each selected target.
        count: ExprIr,
        /// Minimum number of distinct targets.
        min_count: u8,
        /// Maximum number of distinct targets.
        max_count: u8,
        /// Canonical target scope.
        scope: DrawTargetScopeIr,
        /// Source span.
        span: Span,
    },
    /// effect reveal_target_hand_sha_or_discard_bind(player, target, helper_skill_def)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRevealTargetHandShaOrDiscardBind {
        /// Skill owner and virtual Sha user.
        player: ExprIr,
        /// Selected other player.
        target: ExprIr,
        /// Temporary target-bound helper definition.
        helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect draw_remembered_hand_cards(player, count)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDrawRememberedHandCards {
        /// Player drawing and remembering the cards.
        player: ExprIr,
        /// Number of cards to draw.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect discard_remembered_hand_cards(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDiscardRememberedHandCards {
        /// Player discarding remembered cards still in hand.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Gain one topmost draw-pile card from each selector group whose paired divisor matches the
    /// player's match-lifetime card-action count.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGainDrawCardsForLifetimeCardActionDivisors {
        /// Player whose history is inspected and who gains the cards.
        player: ExprIr,
        /// Three positive divisors checked independently.
        divisors: [u32; 3],
        /// Three stable reward-group tags paired with the divisors.
        groups: [u8; 3],
        /// Source span.
        span: Span,
    },
    /// During the originating turn-opening extra Play phase, lose 1 HP and then gain the
    /// topmost matching discard-pile card. Owning the supplied skill definition makes the
    /// action mandatory; otherwise it is optional.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectLoseHpGainDiscardCardForOpeningExtraPlay {
        /// Skill owner, HP-loss target, and card recipient.
        player: ExprIr,
        /// Stable card-kind tag.
        kind: i32,
        /// Owning this definition makes the action mandatory.
        mandatory_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Choose and gain one physical card the player lost from hand/equipment this turn that
    /// still remains in the discard pile.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseGainOwnLostDiscardCardThisTurn {
        /// Skill owner, chooser, and card recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose one equipment or delayed-trick card from another living player's field and move it
    /// to a fixed living destination.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseMoveFieldCardTo {
        /// Skill owner and card chooser.
        player: ExprIr,
        /// Fixed destination.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Draw cards, then optionally distribute up to a bounded number of current hand cards among
    /// other living players.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDrawThenOptionallyDistributeHandCards {
        /// Skill owner, drawer, chooser, and giver.
        player: ExprIr,
        /// Cards drawn before distribution choices begin.
        draw_count: ExprIr,
        /// Maximum total number of hand cards that may be distributed.
        give_cap: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Give the selected hand cards, then optionally repeat for distinct other recipients and
    /// recover once when the activation's total first reaches two.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGiveSelectedThenContinueDistinctRecoverAtTwo {
        /// Skill owner and giver.
        player: ExprIr,
        /// First active-skill target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose one living other player tied for the highest positive mark, judge that target, and
    /// kill them unless the physical result is Tao or Taoyuan.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
        /// Dead skill owner and chooser.
        player: ExprIr,
        /// Mark definition used to rank living other players.
        mark_def: u32,
        /// Source span.
        span: Span,
    },
    /// Choose a zone priority, gain one deterministic-random card from every other living
    /// player's first nonempty preferred zone in seat order, then turn over.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
        /// Skill owner, chooser, and card recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose and gain one hand/equipment/judgment card from each other living player in seat
    /// order, then turn over.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseGainCardFromEachOtherThenTurnOver {
        /// Skill owner, chooser, and card recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Grant a skill until the current turn ends.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectAddSkillUntilTurnEnd {
        /// Skill recipient.
        player: ExprIr,
        /// Skill definition to grant.
        skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Consume marks and resolve ordered all-other damage/discard/turn-over semantics.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveMassDamageDiscardTurnOver {
        /// Skill owner and damage source.
        player: ExprIr,
        /// Mark definition consumed.
        mark_def: u32,
        /// Mark count consumed.
        mark_count: u32,
        /// Damage dealt to every other player.
        damage_amount: u32,
        /// Maximum hand cards discarded by every other player.
        hand_count: u32,
        /// Source span.
        span: Span,
    },
    /// Consume marks, damage every other living player, then make each discard a bounded
    /// number of hand/equipment cards.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveMassDamageDiscardHandEquip {
        /// Skill owner and damage source.
        player: ExprIr,
        /// Mark definition consumed.
        mark_def: u32,
        /// Mark count consumed.
        mark_count: u32,
        /// Damage dealt to every other player.
        damage_amount: u32,
        /// Maximum hand/equipment cards discarded by every other player.
        discard_count: u32,
        /// Source span.
        span: Span,
    },
    /// Record the selected target for a target-only turn uniqueness contract.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRecordActiveTarget {
        /// Skill owner.
        player: ExprIr,
        /// Selected target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose and pay either one discardable physical weapon or one HP.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseDiscardWeaponOrLoseHp {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve a mark-parity target action and optional clear-all mass damage.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveMarkParityControl {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Mark definition controlling the effect.
        mark_def: u32,
        /// Clear-all option threshold.
        mass_threshold: u32,
        /// Normal damage amount.
        damage_amount: u32,
        /// Source span.
        span: Span,
    },
    /// Discard selected targets' equipment, then choose one selected survivor to damage.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSelectedEquipmentDiscardThenChooseDamage {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Damage amount.
        damage_amount: u32,
        /// Damage nature tag.
        nature: u8,
        /// Source span.
        span: Span,
    },
    /// Replace the pending normal draw with a top-card distinct-suit selection.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectReplaceDrawWithDistinctSuitSelection {
        /// Draw-phase player.
        player: ExprIr,
        /// Top-card reveal count.
        count: u8,
        /// Source span.
        span: Span,
    },
    /// Inspect a target hand and choose one heart card to discard or put on the draw pile.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectInspectTargetHandChooseHeartDiscardOrTop {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Inspected active-skill target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve the complete limited Yeyan choice and ordered fire damage.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveYeyan {
        /// Skill owner and damage source.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose whether every living player loses or recovers one HP, or cancel.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseAllLoseHpOrRecover {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Initialize or exchange the current skill's public Qixing pile.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveQixing {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Whether to initialize the pile from the draw pile before exchanging.
        initialize: bool,
        /// Source span.
        span: Span,
    },
    /// Spend cards from a Qixing pile and grant one source-bound weather helper per target.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveQixingWeather {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Qixing skill definition owning the public pile.
        qixing_skill_def: u32,
        /// Temporary target-owned helper skill definition.
        helper_skill_def: u32,
        /// Maximum number of cards and targets selected.
        max_targets: u8,
        /// Source span.
        span: Span,
    },
    /// Toggle Longnu, pay the matching HP cost, draw, and grant the phase helper.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveNzryLongnu {
        /// Skill owner.
        player: ExprIr,
        /// Red-card-to-fire-Sha helper.
        fire_skill_def: u32,
        /// Trick-card-to-thunder-Sha helper.
        thunder_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Force the owner to choose another living unlinked player and link them.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseLinkOther {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose and discard four distinct-suit cards across the owner and target hands, then
    /// resolve the owner-card-count branch.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDrltPoxi {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Inspected active-skill target.
        target: ExprIr,
        /// Temporary hand-limit helper definition.
        hand_limit_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve one Jieying lifecycle branch.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDrltJieying {
        /// Jieying owner.
        player: ExprIr,
        /// Player carried by the triggering event.
        event_player: ExprIr,
        /// Stable 营 mark definition.
        mark: u32,
        /// 0 = ensure mark, 1 = transfer, 2 = settle.
        mode: u8,
        /// Source span.
        span: Span,
    },
    /// Resolve one Duorui lifecycle branch.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDrltDuorui {
        /// Skill owner.
        player: ExprIr,
        /// Damaged target or cleanup event player.
        target: ExprIr,
        /// Target-side cleanup helper definition.
        helper_skill_def: u32,
        /// 0 = acquire, 3 = cleanup.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// Resolve one Zhiti restoration trigger.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDrltZhiti {
        /// Skill owner.
        player: ExprIr,
        /// Event player.
        event_player: ExprIr,
        /// Event source.
        event_source: ExprIr,
        /// Event scalar.
        event_amount: ExprIr,
        /// 0 = hand comparison, 1 = Duel, 2 = damage.
        mode: u8,
        /// Source span.
        span: Span,
    },
    /// Resolve one Shen Simayi shared branch.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveShenSimayi {
        /// Skill owner.
        player: ExprIr,
        /// Event player.
        event_player: ExprIr,
        /// Event scalar.
        event_amount: ExprIr,
        /// Stable 忍 mark definition.
        mark: u32,
        /// Package branch 0..=4.
        mode: u8,
        /// Source span.
        span: Span,
    },
    /// Resolve Cai Wenji's Beige cost and one four-suit judgment.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveBeige {
        /// Beige owner.
        player: ExprIr,
        /// Damaged player.
        target: ExprIr,
        /// Damage source.
        source: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect damage_then_discard_all_judge(player)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDamageThenDiscardAllJudge {
        /// Player damaged and whose judgment zone is eventually emptied.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect replace_draw_gain_top_two_add_skills_if_different_colors(player, first_skill_def, second_skill_def)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
        /// Draw-phase player and card recipient.
        player: ExprIr,
        /// First temporary skill definition.
        first_skill_def: u32,
        /// Second temporary skill definition.
        second_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect choose_gain_target_card_alternate_zone_then_target_draw(player, target)
    ///
    /// Gain one current hand/equipment card from `target`, excluding the zone already used by
    /// this skill in the current Play phase, then let `target` draw one card. Appended to keep
    /// existing serialized enum ordinals stable.
    EffectChooseGainTargetCardAlternateZoneThenTargetDraw {
        /// Skill owner gaining the card.
        player: ExprIr,
        /// Other living target losing one card, then drawing.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Mark a living player's next judgment phase to be skipped.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectSkipNextJudgePhase {
        /// Player whose next judgment phase is skipped.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose and discard one hand/equipment card from either endpoint of the current damage;
    /// increase that damage when the card's owner-effective suit matches.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseDiscardDamageEndpointCardIncreaseIfSuit {
        /// Damage source, skill owner, and chooser.
        player: ExprIr,
        /// Current damage target.
        target: ExprIr,
        /// Stable suit tag (`1` spade through `4` diamond).
        suit: u8,
        /// Damage added after the discard resolves.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose and gain one hand card from the first player bound to the current event.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectTakeOneEventTargetHand {
        /// Player choosing and gaining the card.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose an extra Draw or Play phase after End, or execute both when used-card suits reach HP.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseEndExtraDrawOrPlayByUsedSuits {
        /// Current turn player and skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Lose HP equal to the bound damage, double that damage, then replace this skill.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectLoseHpDoubleCurrentDamageThenReplace {
        /// Damage source, HP-loss target, and skill owner.
        player: ExprIr,
        /// Permanent replacement skill definition.
        replacement_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve one reusable hidden skill-pile action.
    ///
    /// Actions: 0 initialize two hidden top cards; 1 reveal one and exclude the current card
    /// target; 2 exchange hand cards with hidden pile cards; 3 reveal one and draw the number of
    /// revealed pile cards; 4 draw for triggering lost cards whose suits match revealed cards.
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveHiddenSkillPile {
        /// Skill owner.
        player: ExprIr,
        /// Definition that owns the expansion pile.
        pile_skill_def: u32,
        /// Stable action tag.
        action: u8,
        /// Current event card; ignored by actions other than target exclusion.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Enable or clear the current skill as the turn's non-locked-skill suppression source.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectSetNonLockedSkillSuppression {
        /// Skill owner and current turn player.
        player: ExprIr,
        /// Whether to enable (`true`) or clear (`false`) suppression.
        enabled: bool,
        /// Source span.
        span: Span,
    },
    /// Resolve EXTRA 帝力/驭衡 and its derived fate-line actions.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDiliYuheng {
        /// Skill owner.
        player: ExprIr,
        /// Stable action tag.
        action: u8,
        /// Bound card for card-use or equipment actions.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Discard the active invocation's selected hand/equipment cards, then draw that many cards
    /// plus one if every current hand card was selected.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDiscardSelectedDrawAllHandBonus {
        /// Skill owner and draw recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose another player for 国战英魂's lost-HP draw/discard branch.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseOtherYinghun {
        /// Wounded skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose another player for 弼政's draw and post-draw over-limit discards.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseOtherBizheng {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 霞姝's give-all, reveal-partition, and gain-partition choices.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseOtherXiashu {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve one 观潮 card rank against the preceding Play-phase card and invalidate the mode
    /// after the first monotonicity violation.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveGuanchaoRank {
        /// Skill owner.
        player: ExprIr,
        /// Current used card.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 观微's discard, draw-two, and immediate extra Play phase.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveGuanwei {
        /// Skill owner.
        player: ExprIr,
        /// Player whose Play phase ended.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 揽江's snapshotted sequential choices.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveLanjiang {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 秉壹's full-hand reveal and conditional draws.
    EffectResolveRebingyi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Queue 博图's round-bounded extra turn.
    EffectResolveRebotu {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 旧缔盟's pair selection/swap or Play-end discard.
    EffectResolveOldimeng {
        /// Skill owner.
        player: ExprIr,
        /// Zero starts the active swap; one settles the Play-end discard.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// Resolve 界结姻 using the active invocation's selected card.
    EffectResolveRejieyin {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 仁恤's ordered two-target card transfer and draw checks.
    EffectResolveReanxu {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 决堰's equipment-slot choice and permanent/temporary branch benefit.
    EffectResolveDrltJueyan {
        /// Skill owner.
        player: ExprIr,
        /// Permanent +1 Sha-use helper.
        sha_skill_def: u32,
        /// Temporary +3 Sha-use helper.
        weapon_skill_def: u32,
        /// Unlimited-distance helper.
        mount_skill_def: u32,
        /// Hand-limit +3 helper.
        armor_skill_def: u32,
        /// 界集智 helper.
        treasure_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve 界集智's draw and optional basic-card discard.
    EffectResolveRejizhi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 诫训's snapshotted draw/discard counts and terminal replacement.
    EffectResolveJiexun {
        /// Skill owner.
        player: ExprIr,
        /// 复难 enhancement helper.
        funan_helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Grant 问卦's active helper to every living player.
    EffectResolveWenguaSetup {
        /// 问卦 owner.
        player: ExprIr,
        /// Global active helper.
        helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve one 问卦 helper activation.
    EffectResolveWengua {
        /// Active actor.
        player: ExprIr,
        /// 问卦 main skill.
        main_skill_def: u32,
        /// Per-Play-phase recipient marker.
        marker_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve 新安国's target-first and owner-second minimum-state actions.
    EffectResolveXinanguo {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 国战三版刚烈's enemy choice and judgment.
    EffectResolveGanglieThree {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 忠义 activation, damage bonus, or round cleanup.
    EffectResolveZhongyi {
        /// Skill owner.
        player: ExprIr,
        /// Stable action tag.
        action: u8,
        /// Temporary helper definition.
        helper_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Discard every equipped card in one slot.
    EffectDiscardEquipmentSlot {
        /// Equipment owner.
        player: ExprIr,
        /// Stable equipment-slot tag.
        slot: u8,
        /// Source span.
        span: Span,
    },
    /// Resolve old 安恤's unequal-hand two-target transfer.
    EffectResolveOldAnxu {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Move active invocation cards into the originating skill's expansion pile.
    EffectStoreSelectedInSkillPile {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// End an identity match for the player's camp when a skill pile reaches a threshold.
    EffectEndMatchForCampIfSkillPileAtLeast {
        /// Camp representative.
        player: ExprIr,
        /// Definition owning the checked expansion pile.
        pile_skill_def: u32,
        /// Minimum pile size.
        count: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve old 寝情's target discard/draw sequence.
    EffectResolveOldQinqing {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve old 明鉴's hand transfer and inserted play-only phase.
    EffectResolveOldMingjian {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve old 旋风's virtual-Sha or distance-one damage branch.
    EffectResolveOldXuanfeng {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Discard two cards to prevent the bound damage for 剑柔.
    EffectResolveJianrou {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve old 贿生's reveal and source choice.
    EffectResolveOldHuisheng {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve old 解烦's prompted Sha or bound recovery.
    EffectResolveOldJiefan {
        /// Skill owner.
        player: ExprIr,
        /// Engine-owned stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// Resolve 彻骨's cancellation and discard branches.
    EffectResolveChegu {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 归心's lord-skill acquisition or faction change.
    EffectResolveJunkguixin {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 谶纬's alternating face-down-card branches.
    EffectResolveYaoChenwei {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Offer one additional current-Sha target within a final-distance cap.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseAddCurrentShaTargetWithinDistance {
        /// Current Sha user and chooser.
        player: ExprIr,
        /// Physical Sha currently resolving.
        card: ExprIr,
        /// Inclusive final-distance cap.
        max_distance: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Refund the currently saved proactive Sha's ordinary use count.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectRefundCurrentShaUse {
        /// Current Sha user.
        player: ExprIr,
        /// Physical Sha currently resolving.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Damage each selected active target, then let that target draw, in seat order.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectDamageThenDrawSelectedTargets {
        /// Damage source.
        source: ExprIr,
        /// Damage dealt to each target.
        damage: ExprIr,
        /// Cards drawn by each target afterward.
        draw: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 游敌's target-owned discard and conditional counter-gain.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveYoudi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 狂斧's equipment selection and move/discard branch.
    EffectResolveKuangfu {
        /// Skill owner.
        player: ExprIr,
        /// Damaged target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 缮甲's draw, forced discard, and optional virtual Sha.
    EffectResolveShanjia {
        /// Skill owner.
        player: ExprIr,
        /// Lifetime equipment-use count.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 新涯角's reveal, recipient, and category-mismatch discard.
    EffectResolveNewYajiao {
        /// Skill owner.
        player: ExprIr,
        /// Card just used or responded with.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 定品's target judgment and color-dependent result.
    EffectResolveDingpin {
        /// Skill owner.
        player: ExprIr,
        /// Selected target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Choose living recipients for 破虏, increase its stable activation mark once, then make
    /// every selected recipient draw the previous mark plus one card in seat order.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseRepoluTargetsDraw {
        /// Skill owner and chooser, who may already be dead.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 过论's hidden target-hand selection, optional owner-card exchange, and draw.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveXinfuGuolun {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Selected other player.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Discard one card for the current Sha refund/red draw, restoring a skill on equal ranks.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
        /// Skill owner choosing the discard cost and receiving the possible draw.
        player: ExprIr,
        /// Player who used the currently resolving Sha.
        sha_user: ExprIr,
        /// Physical card carrying the logical Sha.
        card: ExprIr,
        /// Skill definition restored only when the paid card and Sha have equal ranks.
        restore_skill_def: u32,
        /// Source span.
        span: Span,
    },
    /// Resolve Beige's cost, judgment, rewrite window, and final suit branch with configurable
    /// Heart recovery and Diamond draw counts.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveBeigeConfigured {
        /// Beige owner.
        player: ExprIr,
        /// Damaged player.
        target: ExprIr,
        /// Damage source.
        source: ExprIr,
        /// Heart recovery amount.
        heart_recover_amount: ExprIr,
        /// Diamond draw count.
        diamond_draw_count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Let a dead 追忆 owner optionally choose a living non-killer recipient.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveZhuiyi {
        /// Dead skill owner and choice observer.
        player: ExprIr,
        /// Credited killer, or no player for a source-free death.
        killer: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve 界镇军 without changing the legacy `rezhenjun` host effect or continuation stages.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDecadezhenjun {
        /// Skill owner and chooser.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve SP 骁果 without changing Standard `stdxiaoguo`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpXiaoguo {
        /// Skill owner and chooser.
        player: ExprIr,
        /// End-phase player.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Replace the pending normal draw by gaining one card from each other living player.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectChooseGainCardFromEachOtherReplaceDraw {
        /// Skill owner, chooser, and card recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Give the active invocation's selected hand cards, then recover its target after movement
    /// events and nested triggers finish.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectGiveSelectedThenRecoverTarget {
        /// Skill owner and giver.
        player: ExprIr,
        /// Selected living recipient.
        target: ExprIr,
        /// Recovery amount.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Active selection: give chosen cards to the target, then the chooser recovers
    /// (ABI 679).
    EffectGiveSelectedThenRecoverSelf {
        /// Chooser and giver.
        player: ExprIr,
        /// Gift recipient.
        target: ExprIr,
        /// Chooser recovery amount.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Third-party hand comparison with winner-vs-loser virtual Sha (ABI 680).
    EffectChooseHandCompareBetween {
        /// Skill owner and originating observer.
        player: ExprIr,
        /// First comparison participant (picks the opponent).
        first: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Watch and re-order the top `count` cards one by one (ABI 681).
    EffectGuanxingTop {
        /// Viewer.
        player: ExprIr,
        /// Number of top cards.
        count: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Falu-marked guanxing: N = non-zero count among four declared marks (ABI 683).
    EffectGuanxingTopByFaluMarks {
        /// Viewer.
        player: ExprIr,
        /// Four falu mark defs.
        marks: [u32; 4],
        /// Source span.
        span: Span,
    },
    /// Choose a giver who stashes one hand card to the chooser, then recovers (ABI 682).
    EffectChooseOtherGiveThenRecover {
        /// Chooser and gift recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve SP 定叛's draw, target branch, equipment movement, and delayed damage.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDingpan {
        /// Skill owner and possible damage source.
        player: ExprIr,
        /// Selected equipment-bearing target and branch chooser.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Resolve OFFLINE 贪赂's frozen HP-gap branch, gift, damage, and delayed discard.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveTanlu {
        /// Skill owner, gift recipient, and possible damage source.
        player: ExprIr,
        /// Current-turn player making every forced choice.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// Judge repeatedly while results match the first judgment's effective color.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectJudgeSameColorGainRepeat {
        /// Judged player and successful-card recipient.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
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
    /// 兵势孙韶「筑墼」复合。ABI 521.
    EffectResolveMbzhuji {
        /// Skill owner expression.
        player: ExprIr,
        /// Resolver stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_potlibing(player, card, stage). ABI 522.
    EffectResolvePotlibing {
        /// Skill owner.
        player: ExprIr,
        /// Used or resolved card.
        card: ExprIr,
        /// Zero handles resolution; one handles use.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_potpoxi(player). ABI 522.
    EffectResolvePotpoxi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_potfuan(player, card, stage, range_helper_def, yinxian_def). ABI 525.
    EffectResolvePotfuan {
        /// Skill owner.
        player: ExprIr,
        /// Movement event card.
        card: ExprIr,
        /// Resolution stage.
        stage: u8,
        /// Attack-range helper skill definition.
        range_helper_def: u32,
        /// 引弦 helper skill definition.
        yinxian_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_potyinxian(player, card, stage). ABI 525.
    EffectResolvePotyinxian {
        /// Skill owner.
        player: ExprIr,
        /// Movement event card.
        card: ExprIr,
        /// Resolution stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 势桓阶「共谋」「正朔」复合。ABI 529.
    EffectResolvePothuanjie {
        /// Skill owner expression.
        player: ExprIr,
        /// Exchange target expression; ignored while opening the choice.
        target: ExprIr,
        /// Resolver stage.
        stage: u8,
        /// 奇策 skill definition.
        qice_def: u32,
        /// 看破 skill definition.
        kanpo_def: u32,
        /// Source span.
        span: Span,
    },
    /// 势孙綝「逆固」复合。ABI 532.
    EffectResolvePotnigu {
        /// Skill owner expression.
        player: ExprIr,
        /// Resolver stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 势孙綝「戮连」复合。ABI 532.
    EffectResolvePotlulian {
        /// Skill owner expression.
        player: ExprIr,
        /// Triggering physical card.
        card: ExprIr,
        /// Resolver stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 势鲁肃「缔盟」复合。ABI 535.
    EffectResolvePotdimeng {
        /// Skill owner expression.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 势鲁肃「好施」复合。ABI 536.
    EffectResolvePothaoshi {
        /// Skill owner expression.
        player: ExprIr,
        /// Zero grants at End phase; one clears at the next turn start.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 势董昭「妙略」复合。ABI 539.
    EffectResolveSpmiaolve {
        /// Skill owner expression.
        player: ExprIr,
        /// Zero initializes; one handles damage.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 势董昭「迎驾」复合。ABI 539.
    EffectResolveSpyingjia {
        /// Skill owner expression.
        player: ExprIr,
        /// Zero resets, one records, two settles.
        action: u8,
        /// Extra-turn draw helper definition.
        helper_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spyajun(player, less_def, stage)
    ///
    /// ABI 428: 压军 — at Play-phase begin with a card gained this turn, compare
    /// a gained card against another player; on win optionally place the
    /// comparison card on top of the draw pile, on loss add a hand-limit mark.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpyajun {
        /// Skill owner.
        player: ExprIr,
        /// 戒 hand-limit mark definition.
        less_def: u32,
        /// Continuation stage.
        stage: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spzundi(player, target)
    ///
    /// ABI 429: 尊帝 — judge after discarding a hand card to another player;
    /// black: that player draws 3; red: that player moves a field card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpzundi {
        /// Skill owner.
        player: ExprIr,
        /// Judge beneficiary.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xinyangjie(player, target)
    ///
    /// ABI 430: 佯解 — compare against another player with the SMALLER rank
    /// winning; on failure choose another player to use a virtual fire Sha on
    /// the comparison target.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveXinyangjie {
        /// Skill owner.
        player: ExprIr,
        /// Comparison target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_qingyu(player, action, xuancun_def, mbyongjue_def)
    ///
    /// ABI 431: 清雨 — mission skill: defend by discarding 2 he cards before
    /// damage (action 0); on Prepare phase healthy with hand below the round
    /// number the mission succeeds and grants 选存 (action 1); on dying the
    /// mission fails, lowering max HP and granting 勇决 to a chosen player
    /// (actions 2/3).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveQingyu {
        /// Skill owner.
        player: ExprIr,
        /// Public action 0..=3.
        action: u8,
        /// 选存 definition.
        xuancun_def: u32,
        /// 勇决 definition.
        mbyongjue_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_zhengsu(player, mode, stage)
    ///
    /// ABI 432: 整肃 system — mode 0 垒金 rank-ascending tracking, mode 1
    /// 变阵 same-suit, mode 2 明志 distinct-suit discards; judged at
    /// discard-phase end with a draw-or-recover reward for grantees + self.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveZhengsu {
        /// Skill owner.
        player: ExprIr,
        /// Mode 0 垒金 / 1 变阵 / 2 明志.
        mode: u8,
        /// Stage 0 track, 1 judge.
        stage: u8,
        /// Triggering card (mode 0 stage 0).
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_houfeng(player, target, mode, stage)
    ///
    /// ABI 433: 侯封 / 燕集 — choose a 整肃 type and grant it to the target
    /// (mode 0 侯封 with grantor record; mode 1 燕集 for self).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveHoufeng {
        /// Skill owner.
        player: ExprIr,
        /// Grant target.
        target: ExprIr,
        /// Mode 0 侯封 / 1 燕集.
        mode: u8,
        /// Stage 0 choose type, 1 commit grant.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_mbdiancai(player, actor)
    ///
    /// ABI 434: 点才 — at another player's Play-phase end, if that actor lost at
    /// least as many cards this turn as their HP and holds fewer cards than their
    /// max HP, they draw up to their max HP.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveMbdiancai {
        /// Skill owner.
        player: ExprIr,
        /// The play-phase actor whose End just fired.
        actor: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_rechuhai(player, target, card, target_card, stage)
    ///
    /// ABI 435: 战海 — duty skill: draw one and compare with rank bonus from
    /// missing equipment; on win reveal the target's hand and gain one pile card
    /// per category; on loss with rank below 7 the mission fails; with more than
    /// two equipment the mission succeeds and swaps 乡害 for 彰名.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveRechuhai {
        /// Skill owner.
        player: ExprIr,
        /// Comparison target.
        target: ExprIr,
        /// Owner's chosen comparison card.
        card: ExprIr,
        /// Target's chosen comparison card.
        target_card: ExprIr,
        /// Continuation stage 0..=3.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xiongshu(player, target, card, action, stage)
    ///
    /// ABI 436: 凶竖 — discard X cards (X = skill mark) at another player's
    /// Play-phase start, reveal one of their hand cards, predict whether they
    /// will use it; at that phase's end a correct prediction deals 1 damage,
    /// a wrong one lets the owner gain the card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveXiongshu {
        /// Skill owner.
        player: ExprIr,
        /// Play-phase actor / prediction target.
        target: ExprIr,
        /// Chosen shown card.
        card: ExprIr,
        /// Prediction 0 会使用 / 1 不会使用.
        action: u8,
        /// Continuation stage 0..=3.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_jianhui(player, source, target)
    ///
    /// ABI 437: 奸回 — after damage, as source drawing one when the damaged
    /// player matches the last one; as target forcing the source to discard a
    /// hand/equip card when the source matches the last one.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveJianhui {
        /// Skill owner.
        player: ExprIr,
        /// Damage source.
        source: ExprIr,
        /// Damage target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spqiai(player, target)
    ///
    /// ABI 439: 弃哀 — give a selected non-basic card to another player; at
    /// full HP draw two, else the recipient chooses between the owner
    /// recovering 1 or drawing two.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpqiai {
        /// Skill owner.
        player: ExprIr,
        /// Gift recipient.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spshanxi(player, target, stage)
    ///
    /// ABI 440: 擅袭 — choose a player without the 檄 mark and give it to them
    /// (globally unique; other players' marks are cleared).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpshanxi {
        /// Skill owner.
        player: ExprIr,
        /// Chosen mark recipient.
        target: ExprIr,
        /// Stage 0 choose target, 1 commit mark.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spshanxi_suoming(player, actor, stage)
    ///
    /// ABI 441: 索命 — a recovering actor with the 檄 mark gives the owner two
    /// cards or loses 1 HP (loses HP directly when they cannot give).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpshanxiSuoming {
        /// Skill owner.
        player: ExprIr,
        /// The recovering actor with the 檄 mark.
        actor: ExprIr,
        /// Stage 0 open give choice, 1 commit.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_tongduo(player, target, card, stage)
    ///
    /// ABI 442: 通夺 — when the owner becomes the single target of another
    /// player's card (once per turn), choose a player with cards to recast one.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveTongduo {
        /// Skill owner.
        player: ExprIr,
        /// Chosen recast actor.
        target: ExprIr,
        /// Chosen recast card.
        card: ExprIr,
        /// Stage 0 choose actor, 1 choose card, 2 commit recast.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_yuanqing(player)
    ///
    /// ABI 443: 怨清 — at Play-phase end, for each category used this phase,
    /// randomly take one matching card from the discard pile into the 仁库.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveYuanqing {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_shuchen(player, dying, renku_def)
    ///
    /// ABI 444: 疏陈 — when another player enters dying (once per round) and the
    /// 仁库 holds more than three cards, gain them all and recover that player.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveShuchen {
        /// Skill owner.
        player: ExprIr,
        /// The dying player.
        dying: ExprIr,
        /// 怨清 definition whose pile is the 仁库.
        renku_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_duanbi(player, stage)
    ///
    /// ABI 445: 断笔 — limited skill: when the total hand count exceeds twice the
    /// living player count, choose any number of players; each discards
    /// min(3, floor(hand/2)); afterwards one player gains up to three of those
    /// discarded cards still in the discard pile (authoritative random).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDuanbi {
        /// Skill owner.
        player: ExprIr,
        /// Stage 0 open discard, 2 open gain target, 3 commit gain.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_zaoli(player, mark)
    ///
    /// ABI 446: 躁厉 — at turn start with 躁厉 marks, discard at least one card
    /// (up to all) and draw mark+discarded; losing 1 HP when the total exceeds
    /// two. Play phase only allows hand cards gained this turn (modifier).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveZaoli {
        /// Skill owner.
        player: ExprIr,
        /// 躁厉 mark count at activation.
        mark: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_guying(player, actor, card, stage)
    ///
    /// ABI 447: 孤影 — after the owner loses one hand/equip/judge card (once per
    /// turn, another player's phase), that actor chooses between randomly giving
    /// the owner one gainable card, or the owner taking back the lost card (if
    /// still in the discard pile).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveGuying {
        /// Skill owner.
        player: ExprIr,
        /// Current-phase actor making the choice.
        actor: ExprIr,
        /// The lost card.
        card: ExprIr,
        /// Stage 0 open two-choice, 1 commit branch.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect modify_current_damage(player, delta)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectModifyCurrentDamage {
        /// Skill owner requesting the modification.
        player: ExprIr,
        /// Delta applied to the current damage amount (may be negative).
        delta: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_potyuji(player, target, card, action)
    ///
    /// Appended for serialized ordinal stability (ABI 570).
    EffectResolvePotyuji {
        /// Skill owner or beneficiary.
        player: ExprIr,
        /// Event actor/target.
        target: ExprIr,
        /// Triggering physical card.
        card: ExprIr,
        /// Public action 0..=8.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_rehengjiang(player, actor, action, hand_limit_def, draw_def)
    ///
    /// Appended for serialized ordinal stability (ABI 572).
    EffectResolveRehengjiang {
        /// Skill owner.
        player: ExprIr,
        /// Turn actor for settlement.
        actor: ExprIr,
        /// Public action 0..=1.
        action: u8,
        /// Temporary hand-limit modifier definition.
        hand_limit_def: u32,
        /// Temporary draw-settlement definition.
        draw_def: u32,
        /// Source span.
        span: Span,
    },
    /// Permanently grant the first configured skill definition the player does not own.
    /// Appended for serialized ordinal stability (ABI 575).
    EffectAddFirstMissingSkill {
        /// Skill owner.
        player: ExprIr,
        /// Three ordered candidate skill definitions.
        skill_defs: [u32; 3],
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 糜夫人【存嗣】and the deferred【闺秀】recovery. ABI 578.
    /// Appended for serialized ordinal stability.
    EffectResolveDccunsi {
        /// Skill owner.
        player: ExprIr,
        /// Selected recipient of【勇决】.
        target: ExprIr,
        ///【闺秀】definition checked by the deferred recovery.
        guixiu_def: u32,
        ///【勇决】definition granted to the target.
        yongjue_def: u32,
        /// Public stage 0; stage 1 is engine-owned deferred settlement.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// Bind every target of the exact currently resolving Sha to this skill. ABI 580.
    /// Appended for serialized ordinal stability.
    EffectRecordCurrentShaAllTargets {
        /// Sha user.
        player: ExprIr,
        /// Exact logical Sha carrier.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_give_or_discard_hand(player, payment_count, scope)
    ///
    /// Force the player to give exactly `payment_count` hand cards to one living target in
    /// `scope`, or discard that many hand cards. ABI 581.
    /// Appended for serialized ordinal stability.
    EffectChooseGiveOrDiscardHand {
        /// Skill owner choosing and paying cards.
        player: ExprIr,
        /// Exact hand-card payment count.
        payment_count: ExprIr,
        /// Recipient target scope.
        scope: DrawTargetScopeIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_hand_equip_then_recover_target(chooser, target, count, amount)
    ///
    /// Let `chooser` cancel or discard exactly `count` current hand/equipment cards, then
    /// recover `target` by `amount`. ABI 582. Appended for serialized ordinal stability.
    EffectChooseDiscardHandEquipThenRecoverTarget {
        /// Skill owner and chooser.
        chooser: ExprIr,
        /// Living wounded recovery target.
        target: ExprIr,
        /// Exact discard count.
        count: ExprIr,
        /// Maximum HP recovered after the discard resolves.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_hand_range(player, min, max)
    ///
    /// Force a choice of discardable hand cards within the inclusive range. ABI 596.
    /// Appended for serialized ordinal stability.
    EffectChooseDiscardHandRange {
        /// Player choosing and discarding cards.
        player: ExprIr,
        /// Inclusive minimum selection count.
        min: u32,
        /// Inclusive maximum selection count.
        max: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dcctjiuxian(player, event_player, action)
    ///
    /// Resolve XIANDING 陈泰「矜贤」. ABI 597. Appended for serialized ordinal stability.
    EffectResolveDcctjiuxian {
        /// Skill owner.
        player: ExprIr,
        /// Damaged Duel target for the recovery branch; `self` for activation.
        event_player: ExprIr,
        /// 0=start active branch, 2=start damage recovery branch.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_bianyue(player, target, action)
    ///
    /// Resolve XIANDING 卞玥「庇族/无胁」. ABI 598. Appended for serialized ordinal stability.
    EffectResolveBianyue {
        /// Skill owner.
        player: ExprIr,
        /// Event player placeholder; the target is selected by the effect when needed.
        target: ExprIr,
        /// 0=庇族 active branch, 10=无胁 End-phase branch.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_zhupeilan(player, target, card, helper_def, action)
    ///
    /// Resolve XIANDING 朱佩兰「辞虑/痛悼」. ABI 600. Appended for serialized ordinal stability.
    EffectResolveZhupeilan {
        /// Skill owner.
        player: ExprIr,
        /// Event source or target placeholder.
        target: ExprIr,
        /// Triggering trick carrier, or none for 痛悼.
        card: ExprIr,
        /// Package-only helper carrying exact-card policies; zero for 痛悼.
        helper_skill_def: u32,
        /// 0=辞虑 targeted, 1=prevent tracked damage, 2=tracked card resolved,
        /// 10=痛悼 dying branch.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dc_sb_zhangxiu(player, target, action)
    ///
    /// Resolve XIANDING 势张绣「附袭/豪义」. ABI 601. Appended for ordinal stability.
    EffectResolveDcSbZhangxiu {
        /// Skill owner.
        player: ExprIr,
        /// Other Play-phase player, or placeholder for 豪义.
        target: ExprIr,
        /// 0=附袭, 10=豪义.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dc_sb_zhouyu(player, source, card, action)
    ///
    /// Resolve XIANDING 新杀谋周瑜「融火/英谋」. ABI 602. Appended for ordinal stability.
    EffectResolveDcSbZhouyu {
        /// Skill owner.
        player: ExprIr,
        /// Triggering card user, or placeholder.
        source: ExprIr,
        /// Triggering or tracked card, or placeholder.
        card: ExprIr,
        /// 0/1=融火, 10/11/20=英谋.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dc_sb_lusu(player, source, card, action)
    ///
    /// Resolve XIANDING 新杀谋鲁肃「明势/盟谋」. ABI 603. Appended for ordinal stability.
    EffectResolveDcSbLusu {
        /// Skill owner.
        player: ExprIr,
        /// Triggering transfer source or tracked Sha user.
        source: ExprIr,
        /// Triggering or tracked card.
        card: ExprIr,
        /// 0=明势, 10/11/20/21=盟谋.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_zhangjian(player, card, grant_skill_def, action)
    ///
    /// Resolve XIANDING 张臶「技能A/技能B」. ABI 604. Appended for ordinal stability.
    EffectResolveZhangjian {
        /// Skill owner.
        player: ExprIr,
        /// Triggering damage card, or placeholder.
        card: ExprIr,
        /// Granted skill definition for 技能B.
        grant_skill_def: u32,
        /// 0=技能A, 1=技能B.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_zhugeruoxue(player, target, card, action)
    ///
    /// Resolve XIANDING 诸葛若雪「琼英/暖惠」. ABI 605. Appended for ordinal stability.
    EffectResolveZhugeruoxue {
        /// Skill owner.
        player: ExprIr,
        /// Target or placeholder.
        target: ExprIr,
        /// Field card or placeholder.
        card: ExprIr,
        /// 0=琼英, 10=暖惠.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_caoyi(player, card, helper_skill_def, action)
    ///
    /// Resolve XIANDING 曹轶「蜜饴/寅君」. ABI 606. Appended for ordinal stability.
    EffectResolveCaoyi {
        /// Skill or helper owner.
        player: ExprIr,
        /// Triggering card or placeholder.
        card: ExprIr,
        /// Package-only 蜜饴 helper definition.
        helper_skill_def: u32,
        /// 0, 10, 11, 20, or 21.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_malingli(player, card, helper_skill_def, action)
    ///
    /// Resolve XIANDING 马伶俐「硝引/花火」. ABI 607. Appended for ordinal stability.
    EffectResolveMalingli {
        /// Skill or helper owner.
        player: ExprIr,
        /// Triggering card or placeholder.
        card: ExprIr,
        /// Package-only helper definition.
        helper_skill_def: u32,
        /// Public action 0, 10, or 20.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_nsshuaiyan(player, recovered_player)
    ///
    /// Resolve archived extension 玩点论杀 费诗「率言」. ABI 608.
    EffectResolveNsshuaiyan {
        /// Skill owner.
        player: ExprIr,
        /// Other player who recovered and chooses the branch.
        recovered_player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wdb_hanlong(player, target, card, helper_skill_def, action)
    ///
    /// Resolve archived extension 玩点论杀 韩龙「伺机/刺酋」. ABI 609.
    EffectResolveWdbHanlong {
        /// Skill owner.
        player: ExprIr,
        /// Current damage target or placeholder.
        target: ExprIr,
        /// Current damage card or placeholder.
        card: ExprIr,
        /// Package-only 刺酋濒死 helper definition.
        helper_skill_def: u32,
        /// Public action 0 or 1.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wdb_liufu(player, target, card, zhucheng_skill_def, action)
    ///
    /// Resolve archived extension 玩点论杀 刘馥「筑城/夺气」. ABI 610.
    EffectResolveWdbLiufu {
        /// Skill owner.
        player: ExprIr,
        /// Sha user/current Play-phase player or placeholder.
        target: ExprIr,
        /// Current Sha or placeholder.
        card: ExprIr,
        /// Stable 筑城 definition owning the public pile.
        zhucheng_skill_def: u32,
        /// Public action 0, 1, or 2.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wdb_liuyan(player, card, action)
    ///
    /// Resolve archived extension 玩点论杀 刘焉「绝道/割据」. ABI 611.
    EffectResolveWdbLiuyan {
        /// Skill owner.
        player: ExprIr,
        /// Selected active cost or placeholder.
        card: ExprIr,
        /// Public action 0 or 1.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wdb_liuzan(player, target, helper_skill_def, action)
    ///
    /// Resolve archived extension 玩点论杀 留赞「亢音」. ABI 612.
    EffectResolveWdbLiuzan {
        /// Skill owner.
        player: ExprIr,
        /// Selected other player or continuation placeholder.
        target: ExprIr,
        /// Stable definition of the temporary modifier helper.
        helper_skill_def: u32,
        /// Public action 0; engine-owned continuations 1..=3.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wdb_tianyu(player, target, card, action)
    ///
    /// Resolve archived extension 玩点论杀 田豫「诱战」. ABI 613.
    EffectResolveWdbTianyu {
        /// Skill owner.
        player: ExprIr,
        /// Target of the incoming Sha.
        target: ExprIr,
        /// Incoming Sha card.
        card: ExprIr,
        /// Public action 0; engine-owned continuations use later stages.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wdb_xizhenxihong(player, target, card, action)
    ///
    /// Resolve archived extension 玩点论杀 习珍习宏「负仇」. ABI 614.
    EffectResolveWdbXizhenxihong {
        /// Skill owner.
        player: ExprIr,
        /// Incoming Sha source for action 0; unused for action 1.
        target: ExprIr,
        /// Incoming Sha for action 0; unused for action 1.
        card: ExprIr,
        /// 0=become Sha target; 1=turn-ended settlement.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wdb_yangyi(player, target, card, action)
    ///
    /// Resolve archived extension 玩点论杀 杨仪「筹度/立断」. ABI 615.
    EffectResolveWdbYangyi {
        /// Skill owner.
        player: ExprIr,
        /// Chosen 调兵遣将 user for 筹度; gained-card recipient for 立断.
        target: ExprIr,
        /// Selected 筹度 cost or the single equipment gained for 立断.
        card: ExprIr,
        /// 0=筹度; 1=立断.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wdb_zaozhirenjun(player, target, card, action)
    ///
    /// Resolve archived extension 玩点论杀 枣祗任峻. ABI 616.
    EffectResolveWdbZaozhiRenjun {
        /// Skill owner.
        player: ExprIr,
        /// Selected target, if any.
        target: ExprIr,
        /// Current card carrier, if any.
        card: ExprIr,
        /// Public action 0=粮策; 1=坚壁.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dddxianxi(player, card, action)
    ///
    /// Resolve archived extension 3D精选 韩当「险袭」. ABI 617.
    EffectResolveDddXianxi {
        /// Skill owner and Sha user.
        player: ExprIr,
        /// Current physical Sha carrier.
        card: ExprIr,
        /// 0=choose extra targets; 1=settle undamaged extra targets.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dddxiaheng(player, action)
    ///
    /// Resolve archived extension 3D精选 周处「侠横」. ABI 618.
    EffectResolveDddXiaheng {
        /// Skill owner.
        player: ExprIr,
        /// Public action 0 starts the forced settlement.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_ddd_liuchong(player, actor, card, action)
    ///
    /// Resolve archived extension 3D精选 刘宠「精彀/末焱」. ABI 619.
    EffectResolveDddLiuchong {
        /// Skill owner.
        player: ExprIr,
        /// Event actor.
        actor: ExprIr,
        /// Triggering physical card, if any.
        card: ExprIr,
        /// 0=末焱轮首; 1=末焱受伤增伤; 2=精彀武器结算.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_ddd_duanbing(player, target, card, action)
    ///
    /// Resolve archived extension 3D精选 丁奉「短兵」. ABI 621.
    EffectResolveDddDuanbing {
        /// Skill owner.
        player: ExprIr,
        /// Damage target for the transfer branch, otherwise none.
        target: ExprIr,
        /// Damage card carrier for the transfer branch, otherwise none.
        card: ExprIr,
        /// 0=start active branch; 1=transfer Bingliang after card damage.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_ddd_wuzhi(player, card, action)
    ///
    /// Resolve archived extension 3D精选 吴质「灵涌/虚孝」. ABI 622.
    EffectResolveDddWuzhi {
        /// Skill owner.
        player: ExprIr,
        /// Judgment or selected physical card, otherwise none.
        card: ExprIr,
        /// 0=灵涌判定；10=虚孝弃置牌处理。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_ddd_xujing(player, source, card, action)
    ///
    /// Resolve archived extension 3D精选 许靖「虚誉/实荐」. ABI 623.
    EffectResolveDddXujing {
        /// Skill owner.
        player: ExprIr,
        /// Current card user, otherwise none.
        source: ExprIr,
        /// Current physical card, otherwise none.
        card: ExprIr,
        /// 0/1/2=虚誉发动/债务/清理；10/11/12=实荐赠牌/后续/清理。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_ddd_caomao(player, action)
    ///
    /// Resolve archived extension 3D精选 曹髦「太思/驱车」. ABI 624.
    EffectResolveDddCaomao {
        /// Skill owner.
        player: ExprIr,
        /// 0=太思；10=驱车主动入口。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_ddd_xinxianying(player, actor, action)
    ///
    /// Resolve archived extension 3D精选 辛宪英「洞察」. ABI 625.
    EffectResolveDddXinxianying {
        /// Skill owner.
        player: ExprIr,
        /// Current turn player.
        actor: ExprIr,
        /// 0=双方置鉴；1=回合末依次取鉴。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_ddd_xianglang(player, actor, action)
    ///
    /// Resolve archived extension 3D精选 向朗「恰化/腹笥/拓籍」. ABI 626.
    EffectResolveDddXianglang {
        /// Skill owner.
        player: ExprIr,
        /// Current turn player, borrower, or none.
        actor: ExprIr,
        /// 0=恰化；1=腹笥全局初始化；2=拓籍借牌后结算。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 于禁「整军」复合（ABI 627）。
    EffectResolveDddYujin {
        /// Skill owner.
        player: ExprIr,
        /// Player whose HP/cards changed.
        actor: ExprIr,
        /// Public action: 0=HP change, 1=cards moved.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 刘晔「察势/齐策」复合（ABI 628）。
    EffectResolveDddLiuye {
        /// Skill owner.
        player: ExprIr,
        /// Current Play-phase player or card user.
        actor: ExprIr,
        /// Current used card, or none for phase actions.
        card: ExprIr,
        /// 0=察势布置，1=察势命中，2=齐策准备，3=齐策解锁，4=察势过期。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 鲍三娘「枝连/赍剑」复合（ABI 629）。
    EffectResolveDddBaosanniang {
        /// Skill owner.
        player: ExprIr,
        /// 0=枝连回合末，1=赍剑主动技。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 甄宓「淼形/浮世」复合（ABI 630）。
    EffectResolveDddZhenji {
        /// Skill owner.
        player: ExprIr,
        /// 0=淼形起手拆组，1=淼形摸牌阶段后平衡，2=浮世整手交换。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 赵昂「奋计」（ABI 631）。
    EffectResolveDddZhaoang {
        /// Skill owner.
        player: ExprIr,
        /// 0=摸牌阶段，1=弃牌阶段。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 刘巴「丰政/玉律」（ABI 632）。
    EffectResolveDddLiuba {
        /// Skill owner or global-helper user.
        player: ExprIr,
        /// Current turn player when relevant.
        actor: ExprIr,
        /// 0=丰政转化，1=轮首分配，2=安装全局技，3=玉律起始，4=玉律响应。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 蹇硕「分野/逝潮」（ABI 633）。
    EffectResolveDddJianshuo {
        /// Skill owner.
        player: ExprIr,
        /// Active comparison target or damage source when relevant.
        target: ExprIr,
        /// 0=分野，1=逝潮准备阶段，2=逝潮伤害前。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 管宁「游学/承经」（ABI 634）。
    EffectResolveDddGuanning {
        /// Skill owner.
        player: ExprIr,
        /// Current round/turn actor when relevant.
        actor: ExprIr,
        /// 0=游学轮首，1=承经回合末，2=承经主动使用。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 曹爽「专摄/危秋」（ABI 635）。
    EffectResolveDddCaoshuang {
        /// Skill owner.
        player: ExprIr,
        /// Current actor or recovering player.
        actor: ExprIr,
        /// Current used card, or none.
        card: ExprIr,
        /// 0=赠牌，1=同名牌使用，2=回合末，3=危秋。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 薛灵芸「涟洏/暗织」（ABI 636）。
    EffectResolveDddXuelingyun {
        /// Skill owner.
        player: ExprIr,
        /// Current actor.
        actor: ExprIr,
        /// Event card.
        card: ExprIr,
        /// Public action tag.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 刘宏「失兴/党锢/覆宗」（ABI 637）。
    EffectResolveDddLiuhong {
        /// Skill owner.
        player: ExprIr,
        /// Current turn actor, changed player, or dead 群 character.
        actor: ExprIr,
        /// Public action tag.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 夏侯玄「朗怀/玄论」（ABI 638）。
    EffectResolveDddXiahouxuan {
        /// Skill owner.
        player: ExprIr,
        /// Public action tag.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 张闿「劫行/拜泪」（ABI 639）。
    EffectResolveDddZhangkai {
        /// Skill owner.
        player: ExprIr,
        /// Current actor or event player.
        actor: ExprIr,
        /// Event card.
        card: ExprIr,
        /// Public action tag.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 梁习「仝御」（ABI 640）。
    EffectResolveDddLiangxi {
        /// Skill owner.
        player: ExprIr,
        /// Current actor or event player.
        actor: ExprIr,
        /// Event card.
        card: ExprIr,
        /// Public action tag.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// Guozhan: spend a 珠联璧合 mark to draw two cards (ABI 641, ADR 0604).
    EffectGuozhanUseZhulianBihe {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 欢乐卡牌【手里剑】使用结算（ABI 642, ADR 0607）。
    EffectResolveShoulijian {
        /// Card user.
        player: ExprIr,
        /// Card target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 取消使用牌（ABI 643, ADR 0608）：成为其他角色使用牌的目标时，取消该牌对当前
    /// 角色生效并获取对其使用的实体牌（gwent 杜达元帅1 特殊卡）。
    EffectCancelIncomingCardUse {
        /// Card holder being targeted.
        player: ExprIr,
        /// Card user whose use is cancelled.
        source: ExprIr,
        /// The used card being cancelled.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 择一加技能（ABI 644, ADR 0612）：玩家在两个技能定义间二选一，将所选技能置入
    /// 目标（gwent 呼朋 hupeng 择一杜达元帅1/2）。
    EffectChooseOtherGainOneOfTwoSkills {
        /// Chooser (skill owner).
        player: ExprIr,
        /// Recipient of the chosen skill.
        target: ExprIr,
        /// First skill definition option.
        first_def: u32,
        /// Second skill definition option.
        second_def: u32,
        /// Source span.
        span: Span,
    },
    /// 叶舞（gwent）随机弃牌链：宿主原语内以种子 RNG 循环，并造卡（雾妖/狼 以技能
    /// 近似）。ABI 645 (ADR 0627).
    EffectGwyewuRandomDiscardChain {
        /// Skill owner (self).
        player: ExprIr,
        /// 雾妖 skill definition (gained per enemy hand card discarded).
        wuyao_def: u32,
        /// 狼 skill definition (gained per enemy equip card discarded).
        lang_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_ollinjie(player, source) — 临界 (sp/ollinjie): the damage source is
    /// forced to discard one hand card; if the discarded card is effectively a Sha, the
    /// owner draws one. ABI 608.
    EffectResolveOllinjie {
        /// Skill owner who draws if the discarded card is a Sha.
        player: ExprIr,
        /// Damage source forced to discard one hand card.
        source: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect discard_hand_expr(player, count) — forced hand discard with an
    /// expression-driven count (e.g. `7 - mark_count(self, def)`). Compiles to the
    /// same HostEffect op as literal-count `discard_hand`; no ABI bump required.
    EffectDiscardHandExpr {
        /// Discarding player.
        player: ExprIr,
        /// Number of cards (clamped to the available hand).
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect discard_hand_equip_expr(player, count) — forced hand-or-equip discard
    /// with an expression-driven count. Same HostEffect op as `discard_hand_equip`.
    EffectDiscardHandEquipExpr {
        /// Discarding player.
        player: ExprIr,
        /// Number of cards (clamped to the available hand+equip).
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect choose_target_gain_max_hp(player, self_skill) — 寿春 (sp/pxshouchun):
    /// the owner selects any living player (self included); that player gains 1 max hp.
    /// ABI 646.
    EffectChooseTargetGainMaxHp {
        /// Choosing player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect gain_target_hand_card(player, self_skill) — 肃州 (sp/pxsizhou):
    /// the owner selects another living player, then gains one of that player's
    /// hand cards. ABI 647.
    EffectGainTargetHandCard {
        /// Gaining player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect choose_clan_not_max_hand_limit_gain(player, self_skill) — 穆荫
    /// (clan/clanmuyin): select a same-clan-or-self player whose hand limit is
    /// not the field max; that player gains +1 hand limit (persistent).
    /// ABI 648.
    EffectChooseClanNotMaxHandLimitGain {
        /// Choosing player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect extra_resolve_current_trick(player, card, self_skill) — 额外结算
    /// family (clanfenshi): queue one extra effect pass for the suspended
    /// trick. ABI 649.
    EffectExtraResolveCurrentTrick {
        /// Owning player.
        player: ExprIr,
        /// Suspended trick source card.
        card: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect place_hand_card_to_expansion(player, self_skill) — 置入
    /// (expansion P2, ADR 0641): place one selected hand card onto the skill
    /// pile. ABI 650.
    EffectPlaceHandCardToExpansion {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect gain_from_skill_pile(player, self_skill) — 取用
    /// (expansion P1, ADR 0641): select one card from own skill pile
    /// into hand. ABI 651.
    EffectGainFromSkillPile {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect discard_skill_pile(player, self_skill) — 弃置自身技能堆全部牌
    /// (expansion S1-B, ADR 0641 §8.2)。ABI 684。
    EffectDiscardSkillPile {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Skill definition owning the pile (u32 literal).
        pile_def: u32,
        /// Span.
        span: Span,
    },
    /// effect place_top_deck_to_expansion(player, self_skill, pile_def, count)
    /// — 置入 (expansion P2-var, ADR 0641 §8.7)。ABI 685。
    EffectPlaceTopDeckToExpansion {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Skill definition owning the pile (u32 literal).
        pile_def: u32,
        /// Number of cards.
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect choose_one_from_skill_pile_discard(player, self_skill, pile_def)
    /// — 选堆内单卡弃置 (expansion S1-B, ADR 0641 §8.7)。ABI 686。
    EffectChooseOneFromSkillPileDiscard {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Skill definition owning the pile (u32 literal).
        pile_def: u32,
        /// Span.
        span: Span,
    },
    /// effect choose_target_hand_card_to_pile(player, self_skill, pile_def)
    /// — 置入他人手牌 (expansion A1, ADR 0641 §8.7)。ABI 688。
    EffectChooseTargetHandCardToPile {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Skill definition owning the pile (u32 literal).
        pile_def: u32,
        /// Span.
        span: Span,
    },
    /// effect rearrange_pile_to_deck_top(player, self_skill, pile_def, count)
    /// — 堆重排 (chooseToMove family, ADR 0641 §8.8)。ABI 690。
    EffectRearrangePileToDeckTop {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Skill definition owning the pile (u32 literal).
        pile_def: u32,
        /// Number of cards to rearrange.
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_qianlong(player, self_skill) — 专属状态机 (批 241)。
    EffectResolveQianlong {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_jsrgzuozhan(player, self_skill) — 专属状态机 (批 242)。
    EffectResolveJsrgzuozhan {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_twmouzhi(player, self_skill, mode, event_card) — 谋制
    /// 专属状态机 (批 245)。
    EffectResolveTwmouzhi {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// 0 = damage_before check/prevent, 1 = damage record.
        mode: ExprIr,
        /// Damage event card (0 if none).
        event_card: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_twdaigui(player, self_skill, stage, count) — 待归多段
    /// 状态机 (批 247, slice 1)。
    EffectResolveTwdaigui {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Machine stage (0 = entry/target selection, 2 = reveal bottom cards).
        stage: ExprIr,
        /// Number of revealed cards (target count bound at stage 2).
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_twmitu(player, self_skill, stage, count) — 迷途多段
    /// 拼点链状态机 (批 249, slice 1)。
    EffectResolveTwmitu {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Machine stage (0 = entry/target selection).
        stage: ExprIr,
        /// Reserved argument (upstream symmetry).
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_qiantun(player, self_skill) — 谦吞拼点族状态机 (批 252)。
    EffectResolveQiantun {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_yangming(player, self_skill) — 养名循环拼点状态机
    /// (批 253)。
    EffectResolveYangming {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_canyun(player, self_skill) — 残殒装备比较状态机
    /// (批 255)。
    EffectResolveCanyun {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_hengwu(player, self_skill, event_card) — 横骛花色计数
    /// 状态机 (批 256)。
    EffectResolveHengwu {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Used/responded card.
        event_card: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_biaozhao3(player, self_skill) — 表召·归状态机 (批 257)。
    EffectResolveBiaozhao3 {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_biaozhao2(player, self_skill, event_player) — 表召·应
    /// 状态机 (批 259)。
    EffectResolveBiaozhao2 {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// The discarder.
        event_player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_dcniji(player, self_skill, mode, event_card) — 逆击
    /// 状态机 (批 260)。
    EffectResolveDcniji {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// 0 = targeted (draw+tag), 1 = phase end (discard tagged).
        mode: ExprIr,
        /// The targeting card (mode 0).
        event_card: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_dcmoukui(player, self_skill, stage, target) — 谋溃
    /// 状态机 (批 261)。
    EffectResolveDcmoukui {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Machine stage (0 = entry choice, 3 = discard pick).
        stage: ExprIr,
        /// The sha target.
        target: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_zhongyan(player, self_skill, stage) — 忠言状态机
    /// (批 262)。
    EffectResolveZhongyan {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Machine stage (0 = entry/target selection).
        stage: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_chumei(player, self_skill, stage) — 狐魅状态机
    /// (批 263)。
    EffectResolveChumei {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Machine stage (0 = entry/target selection, 2 = option choice).
        stage: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_mbbeiming(player, self_skill, stage) — 孛明状态机
    /// (批 269)。
    EffectResolveMbbeiming {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Machine stage (0 = entry/target selection).
        stage: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect reset_linked_selected_targets(source) — 断索主动原语
    /// (批 270)：重置所选全部连环角色，再各造成 1 点火焰伤害。
    EffectResetLinkedSelectedTargets {
        /// Damage source (skill owner).
        source: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_duoduan(player, source, card) — 度断状态机
    /// (批 272)：重铸一张牌后选一项——使用者摸 2 且此杀对 owner 无效，
    /// 或使用者弃一张牌且 owner 不可响应此杀。
    EffectResolveDuoduan {
        /// Sha target holding the skill.
        player: ExprIr,
        /// Sha user.
        source: ExprIr,
        /// Physical Sha card.
        card: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_qiangyong(player, target) — 羌勇状态机 (批 278)：
    /// 【杀】造成伤害时弃置受伤角色 X 张牌（X=本回合用杀数），
    /// 其无手牌则此伤害 +1。
    EffectResolveQiangyong {
        /// Damage source (skill owner).
        player: ExprIr,
        /// Wounded player.
        target: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_dcqiaolve(player) — 巧略状态机 (批 286)：先弃自己
    /// 一张牌，再取事件目标一张随机手牌（弃牌代价补全面）。
    EffectResolveDcqiaolve {
        /// Skill owner.
        player: ExprIr,
        /// Event target.
        target: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_twxiongjun(player) — 雄军状态机 (批 287)：owner 造成
    /// 伤害后，每名拥有〖雄军〗的角色各摸一张牌。
    EffectResolveTwxiongjun {
        /// Damage source (skill owner).
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_yixiang(player) — 义襄状态机 (批 288)：每名角色回合
    /// 限一次，成为使用牌目标时随机获得牌堆中一张自己没有的基本牌。
    EffectResolveYixiang {
        /// Skill owner.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_xinfu_qiai(player) — 求援状态机 (批 291)：限定技，濒死
    /// 时其他角色依座次各交给 owner 一张牌（手+装备，强制，跳过无牌者）。
    EffectResolveXinfuQiai {
        /// Dying skill owner receiving the cards.
        player: ExprIr,
        /// Span.
        span: Span,
    },
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
}

/// Canonical living-player scope used by typed target-choice effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawTargetScopeIr {
    /// Any other living player.
    Other,
    /// Any other living male or double-sex player.
    MaleOther,
    /// Any other living female or double-sex player.
    FemaleOther,
    /// Any living player, including the chooser.
    Any,
    /// Any living player tied for the fewest hand cards.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    MinHand,
    /// Any living player whose current HP equals the chooser's current HP.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    SameHp,
    /// Any living player with no hand cards.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    HandEmpty,
    /// Any living wounded player, including the chooser.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    Wounded,
    /// Any other living player whose hand count does not exceed the chooser's.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    HandLteChooser,
}

impl DrawTargetScopeIr {
    /// Stable Rule VM payload tag.
    pub const fn tag(self) -> u8 {
        match self {
            Self::Other => 0,
            Self::MaleOther => 1,
            Self::FemaleOther => 2,
            Self::Any => 3,
            Self::MinHand => 4,
            Self::SameHp => 5,
            Self::HandEmpty => 6,
            Self::Wounded => 7,
            Self::HandLteChooser => 8,
        }
    }
}

/// Canonical target-card zone scope used by typed Rule IR effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetCardScopeIr {
    /// Equipment only.
    Equip,
    /// Hand and equipment.
    HandEquip,
    /// Hand, equipment, and judgment zones.
    HandEquipJudge,
    /// Equipment and judgment zones only (legacy `ej`, no hand).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EquipJudge,
    /// Hand only.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    Hand,
}

impl TargetCardScopeIr {
    /// Stable Rule VM payload tag.
    pub const fn tag(self) -> i32 {
        match self {
            Self::Equip => 0,
            Self::HandEquip => 1,
            Self::HandEquipJudge => 2,
            Self::EquipJudge => 5,
            Self::Hand => 4,
        }
    }
}

/// Expression IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExprIr {
    /// Integer literal.
    Int(i32),
    /// self (skill owner).
    SelfPlayer,
    /// Current skill instance.
    SelfSkill,
    /// event_player.
    EventPlayer,
    /// event_source (damage source, card user, first ActivateSkill target).
    EventSource,
    /// event_card (the card instance bound to the event, e.g. used card).
    EventCard,
    /// event_amount (damage amount bound to a damage trigger).
    EventAmount,
    /// Local variable.
    Local(String),
    /// Binary op.
    BinOp {
        /// op: + - * == <
        op: String,
        /// Left.
        left: Box<ExprIr>,
        /// Right.
        right: Box<ExprIr>,
    },
    /// query distance(a,b) etc.
    Query {
        /// Query name.
        name: String,
        /// Args.
        args: Vec<ExprIr>,
    },
    /// choose(opt0, opt1, ...) — single-select over integer option tokens.
    /// Pauses at HOST_CHOICE; evaluates to the chosen token on resume.
    Choose {
        /// Option token expressions.
        options: Vec<ExprIr>,
    },
    /// event_target_count (declared target count bound to a card event).
    /// Appended to keep existing serialized enum ordinals stable.
    EventTargetCount,
    /// event_card_count (number of physical cards captured from the triggering event).
    /// Phase 3 card-list-iteration family.
    EventCardCount,
    /// turn_damaged_player_count (distinct players damaged during this turn).
    /// Appended to keep existing serialized enum ordinals stable.
    TurnDamagedPlayerCount,
    /// Unsigned stable identifier literal used by identifier-bearing queries.
    /// Appended to keep existing serialized enum ordinals stable.
    U32(u32),
    /// choose_range(min, max) — single-select over an inclusive non-negative
    /// integer range. Pauses at HOST_CHOICE_RANGE and evaluates to the chosen
    /// token on resume.
    /// Appended to keep existing serialized enum ordinals stable.
    ChooseRange {
        /// Inclusive lower bound.
        min: Box<ExprIr>,
        /// Inclusive upper bound.
        max: Box<ExprIr>,
    },
    /// min(a, b) — element-wise minimum of two integer expressions.
    /// Appended to keep existing serialized enum ordinals stable.
    Min {
        /// First operand.
        left: Box<ExprIr>,
        /// Second operand.
        right: Box<ExprIr>,
    },
}
