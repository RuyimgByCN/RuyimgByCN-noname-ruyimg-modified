//! Parsed AST (pre-typecheck).

use content_schema::{Span, TriggerChoiceActor};

/// Root.
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// Skills.
    pub skills: Vec<SkillAst>,
}

/// Skill.
#[derive(Debug, Clone)]
pub struct SkillAst {
    /// Name.
    pub name: String,
    /// Def id.
    pub def_id: u32,
    /// Locked skills cannot contain optional triggers.
    pub locked: bool,
    /// Only active while the owner has the Lord identity.
    pub lord_only: bool,
    /// Cannot be copied or gained by another skill.
    pub non_gainable: bool,
    /// Conversion skill whose branch can be toggled by generic rules.
    pub conversion: bool,
    /// Items.
    pub items: Vec<SkillItemAst>,
    /// Span.
    pub span: Span,
}

/// Skill body item.
#[derive(Debug, Clone)]
pub enum SkillItemAst {
    /// Trigger.
    Trigger(TriggerAst),
    /// Active skill.
    Active(ActiveAst),
    /// View-as conversion.
    ViewAs(ViewAsAst),
    /// Modifier.
    Modifier(ModifierAst),
}

/// Trigger.
#[derive(Debug, Clone)]
pub struct TriggerAst {
    /// Event.
    pub event: String,
    /// Filter.
    pub filter: String,
    /// Whether a player confirms this trigger before it executes.
    pub optional: bool,
    /// Player who confirms an optional trigger.
    pub choice_actor: TriggerChoiceActor,
    /// Body.
    pub body: Vec<StmtAst>,
    /// Span.
    pub span: Span,
}

/// Active skill (player-activated).
#[derive(Debug, Clone)]
pub struct ActiveAst {
    /// Window: play_phase | any
    pub window: String,
    /// Optional target-count contract.
    pub targets: Option<ActiveTargetsAst>,
    /// Optional authoritative per-turn activation limit.
    pub uses_per_turn: Option<u8>,
    /// Optional authoritative activation limit scoped to one Play phase.
    pub uses_per_play_phase: Option<u8>,
    /// Optional dynamic per-turn activation limit source.
    pub uses_per_turn_source: Option<String>,
    /// Optional owner-mark threshold required before activation.
    pub mark_at_least: Option<(u32, u32)>,
    /// Require the owner not to have this skill definition.
    pub owner_missing_skill: Option<u32>,
    /// Require the owner's hand count to exceed current HP.
    pub hand_gt_hp: bool,
    /// Require the owner's hand count to exceed maximum HP.
    pub hand_gt_max_hp: bool,
    /// Require the owner's current hand limit to be positive.
    pub hand_limit_positive: bool,
    /// Require at least one card in the owner's equipment zone.
    pub equip_nonempty: bool,
    /// Require at least one card in the owner's hand.
    pub hand_nonempty: bool,
    /// Require the owner to be face up.
    pub face_up: bool,
    /// Require the owner's hand to hold no card carrying this gaintag.
    pub hand_missing_gaintag: Option<u32>,
    /// Require the owner not to be linked.
    pub owner_unlinked: bool,
    /// Allow this active skill to resolve at most once during the match.
    pub once_per_game: bool,
    /// Optional card selection zone (`select_hand|select_he min max`).
    pub selection: Option<ActiveSelectionAst>,
    /// Body.
    pub body: Vec<StmtAst>,
    /// Span.
    pub span: Span,
}

/// Declarative player-target input for an active skill.
#[derive(Debug, Clone)]
pub struct ActiveTargetsAst {
    /// Minimum target count.
    pub min: u8,
    /// Maximum target count.
    pub max: u8,
    /// Whether the skill owner may be selected.
    pub allow_self: bool,
    /// Whether every selected target must tie for the lowest current HP.
    pub lowest_hp: bool,
    /// Whether every selected target must tie for the highest current HP.
    pub highest_hp: bool,
    /// Whether every selected target must currently be unlinked.
    pub unlinked: bool,
    /// Whether every selected target must currently be linked.
    pub linked: bool,
    /// Optional owner mark whose count caps the selected-player count.
    pub max_by_owner_mark: Option<u32>,
    /// Cap the selected-player count by the owner's current lost HP.
    pub max_by_owner_lost_hp: bool,
    /// Cap the selected-player count by the owner's current HP.
    pub max_by_owner_hp: bool,
    /// Whether every selected target must currently be wounded.
    pub wounded: bool,
    /// Whether every selected target must be male or double-sex.
    pub male: bool,
    /// Whether every selected target's current HP must be at least the owner's.
    pub hp_at_least_owner: bool,
    /// Whether every selected target's current HP must be greater than the owner's.
    pub hp_gt_owner: bool,
    /// Whether every selected target's current HP must be at most the owner's.
    pub hp_lte_owner: bool,
    /// Optional exclusive lower bound for every selected target's current HP.
    pub hp_gt: Option<u8>,
    /// Whether every selected target must have more hand cards than the owner.
    pub hand_gt_owner: bool,
    /// Whether every selected target must have at most as many hand cards as the owner.
    pub hand_lte_owner: bool,
    /// Require exactly two targets whose hand-card counts differ by at most three.
    pub pair_hand_difference_lte_three: bool,
    /// Optional exclusive upper bound for every selected target's hand-card count.
    pub hand_lt: Option<u8>,
    /// Whether every selected target must have at least one hand card.
    pub hand_nonempty: bool,
    /// Whether the owner and every selected target must each have a hand card to compare.
    pub can_compare: bool,
    /// Optional source-free card kind every selected target must be able to use on the owner.
    pub can_use_on_owner: Option<String>,
    /// Require the second selected target to be able to use a source-free Duel on the first.
    pub second_can_use_juedou_on_first: bool,
    /// Require two equipment-bearing targets with at least one discardable armor.
    pub equipment_pair_with_armor: bool,
    /// Require every selected target to currently have an equipped card.
    pub has_equip: bool,
    /// Require every selected target to have equipment the skill owner may currently discard.
    pub has_discardable_equip: bool,
    /// Require every selected target to have equipment the skill owner may currently gain.
    pub has_gainable_equip: bool,
    /// Require every selected target to have a hand/equipment card the owner may currently gain.
    pub has_gainable_hand_equip: bool,
    /// Require every selected target to be inside the skill owner's attack range.
    pub in_attack_range: bool,
    /// Require the skill owner to be inside every selected target's attack range.
    pub owner_in_target_attack_range: bool,
    /// Require every selected target to have at least one hand/equipment card.
    pub hand_equip_nonempty: bool,
    /// Require every selected target to have a hand/equipment card the skill owner may discard.
    pub has_discardable_hand_equip: bool,
    /// Reject a target already recorded by this skill in the current turn.
    pub unused_by_skill_this_turn: bool,
    /// Reject a target already recorded by this skill in the current play phase.
    pub unused_by_skill_this_play_phase: bool,
    /// Require the selected-player count to equal the selected-card count.
    pub match_selected_card_count: bool,
    /// Require selected cards to match the sole target's equipment-card count.
    pub selected_cards_match_target_equip_count: bool,
}

/// Declarative card input for an active skill.
#[derive(Debug, Clone)]
pub struct ActiveSelectionAst {
    /// Zone name.
    pub zone: String,
    /// Minimum count.
    pub min: u8,
    /// Maximum count.
    pub max: u8,
    /// Whether the activation may be cancelled at the selection barrier.
    pub allow_cancel: bool,
    /// Whether every selected card must have the same effective suit.
    pub same_suit: bool,
    /// Whether every selected card must have the same effective color.
    pub same_color: bool,
    /// Whether selected cards must have pairwise-distinct logical categories.
    pub distinct_categories: bool,
    /// Whether selected cards must have pairwise-distinct effective suits.
    pub distinct_suits: bool,
    /// Optional card-kind filter.
    pub filter: Option<String>,
    /// Reject a logical card category already recorded by this skill in the current turn.
    pub unused_category_by_skill_this_turn: bool,
    /// Reject a logical card category already recorded by this skill in the current play phase.
    pub unused_category_by_skill_this_play_phase: bool,
    /// Reject a logical card category used or discarded by the owner in the current turn.
    pub unused_category_by_owner_history_this_turn: bool,
    /// Require selected cards' effective numbers to total exactly this value.
    pub sum_effective_number_exact: Option<u16>,
}

/// View-as: `view_as sha play_phase red;` or
/// `view_as shan response black from hand_equip uses_per_turn 1 ignore_distance;`.
#[derive(Debug, Clone)]
pub struct ViewAsAst {
    /// As kind.
    pub as_kind: String,
    /// Window.
    pub window: String,
    /// Filter.
    pub filter: String,
    /// Optional source zone; omitted means hand.
    pub source_zone: Option<String>,
    /// Optional authoritative per-turn activation limit.
    pub uses_per_turn: Option<u8>,
    /// Whether ordinary card distance checks are ignored.
    pub ignore_distance: bool,
    /// Whether a red source makes the converted Sha unrespondable.
    pub red_unrespondable: bool,
    /// Whether a black source avoids consuming the ordinary Sha use count.
    pub black_no_use_count: bool,
    /// Whether this converted Sha always avoids consuming the ordinary Sha use count.
    pub no_use_count: bool,
    /// Make each target lacking a hand card of the source suit unable to respond.
    pub target_lacks_source_suit_unrespondable: bool,
    /// Refund the ordinary Sha use count if this exact converted Sha deals no damage.
    pub refund_use_count_if_no_damage: bool,
    /// Minimum number of physical source cards; one by default.
    pub min_cards: u8,
    /// Maximum number of physical source cards; one by default.
    pub max_cards: u8,
    /// Require exactly max(1, current HP) physical source cards.
    pub owner_hp_min_one_cards: bool,
    pub owner_basic_uses_plus_one_cards: bool,
    pub once_per_turn_per_branch: bool,
    /// Require every selected source card to have the same effective suit.
    pub same_suit: bool,
    /// Optional logical card nature, currently `fire` for Sha.
    pub nature: Option<String>,
    /// Optional effect applied when more than one source card is selected.
    pub multi_card_bonus: Option<String>,
    /// Conversion branch required for a source-free family rule.
    pub branch: Option<u8>,
    /// Whether this branch may be used only once in each round.
    pub once_per_round: bool,
    /// Branch selected after a successful conversion.
    pub next_branch: Option<u8>,
    /// Declarative source-free conversion cost.
    pub cost: Option<String>,
    /// Skill-definition mark consumed by the conversion cost.
    pub mark_cost: Option<u32>,
    /// Cards drawn after the converted card is used or responded with.
    pub post_draw: u8,
    /// Span.
    pub span: Span,
}

/// Modifier.
#[derive(Debug, Clone)]
pub struct ModifierAst {
    /// Target.
    pub target: String,
    /// Optional card selector for card-aware modifier targets.
    pub selector: Option<String>,
    /// Mode.
    pub mode: String,
    /// Optional query-time value source.
    pub value_source: Option<String>,
    /// Value.
    pub value: i32,
    /// Span.
    pub span: Span,
}

/// Statement.
#[derive(Debug, Clone)]
pub enum StmtAst {
    /// effect name(args);
    Effect {
        /// Name.
        name: String,
        /// Args.
        args: Vec<ExprAst>,
        /// Span.
        span: Span,
    },
    /// if cond { } else { }
    If {
        /// Cond.
        cond: ExprAst,
        /// Then.
        then_body: Vec<StmtAst>,
        /// Else.
        else_body: Vec<StmtAst>,
        /// Span.
        span: Span,
    },
    /// loop n { }
    Loop {
        /// Times.
        times: u32,
        /// Body.
        body: Vec<StmtAst>,
        /// Span.
        span: Span,
    },
    /// let x = e;
    Let {
        /// Name.
        name: String,
        /// Value.
        value: ExprAst,
        /// Span.
        span: Span,
    },
}

/// Expression.
#[derive(Debug, Clone)]
pub enum ExprAst {
    /// Int.
    Int(i32),
    /// Unsigned identifier literal above the signed VM range.
    U32(u32),
    /// Ident.
    Ident(String),
    /// Call.
    Call {
        /// Name.
        name: String,
        /// Args.
        args: Vec<ExprAst>,
    },
    /// Binary.
    BinOp {
        /// Op.
        op: String,
        /// Left.
        left: Box<ExprAst>,
        /// Right.
        right: Box<ExprAst>,
    },
}
