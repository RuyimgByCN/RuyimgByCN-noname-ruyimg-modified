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
    /// Explicit schema5 charlotte trait.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charlotte: Option<bool>,
    /// Explicit schema5 persevere trait.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persevere: Option<bool>,
    /// Explicit schema5 forever trait.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forever: Option<bool>,
    /// Explicit schema5 temporary trait.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporary: Option<bool>,
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
    /// Optional exclusive lower bound for selected targets' maximum HP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_hp_gt: Option<u8>,
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

// Collect complete variants before declaring the enum; their order is part of the wire format.
mod effect_choose_current_trick_targets_to_nullify;
mod effect_draw;
mod effect_recover_to_max_then_optional_exchange_skill;
mod effect_resolve_mbzhuji;
mod effect_resolve_potjixi;
mod effect_resolve_spolaige;

macro_rules! collect_variants {
    ([$head:ident $(, $tail:ident)*] [$($variants:tt)*]) => {
        $head::variants! { [$($tail),*] [$($variants)*] }
    };
    ([] [$($variants:tt)*]) => {
        /// Statement IR.
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum StmtIr { $($variants)* }
    };
}
collect_variants! { [effect_draw, effect_resolve_potjixi, effect_recover_to_max_then_optional_exchange_skill, effect_choose_current_trick_targets_to_nullify, effect_resolve_mbzhuji, effect_resolve_spolaige] [] }

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
