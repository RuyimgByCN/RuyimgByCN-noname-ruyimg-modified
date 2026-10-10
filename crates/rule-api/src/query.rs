//! Typed read-only rule queries.
//!
//! Queries never emit effects or consume RuleRng.

use engine_api::{CardId, PlayerId, SkillId};
use serde::{Deserialize, Serialize};

/// Typed query request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryRequest {
    /// Distance between two players.
    Distance {
        /// From.
        from: PlayerId,
        /// To.
        to: PlayerId,
    },
    /// Attack range of a player.
    AttackRange {
        /// Player.
        player: PlayerId,
    },
    /// Whether a card can be used by a player now.
    CardUsable {
        /// Player.
        player: PlayerId,
        /// Card.
        card: CardId,
    },
    /// Whether `target` is a legal target of `card` used by `user`.
    TargetLegal {
        /// User.
        user: PlayerId,
        /// Card.
        card: CardId,
        /// Target.
        target: PlayerId,
    },
    /// Hand limit for a player.
    HandLimit {
        /// Player.
        player: PlayerId,
    },
    /// Remaining uses of a skill this window.
    SkillUsesRemaining {
        /// Skill instance.
        skill: SkillId,
    },
    /// Card attribute after modifiers.
    CardAttribute {
        /// Card.
        card: CardId,
        /// Attribute key (content-defined ordinal).
        attr: u32,
    },
    /// Identity / faction relation between two players.
    Relation {
        /// A.
        a: PlayerId,
        /// B.
        b: PlayerId,
    },
    /// Current HP of a player.
    Hp {
        /// Player.
        player: PlayerId,
    },
    /// Current hand size.
    HandCount {
        /// Player.
        player: PlayerId,
    },
    /// Number of cards in the player's equipment zone (batch 176 kamome 观星 count).
    EquippedCount {
        /// Player.
        player: PlayerId,
    },
    /// Number of currently empty equipment slots (weapon, armor, two mounts, treasure).
    EmptyEquipSlots {
        /// Player.
        player: PlayerId,
    },
    /// Max HP of a player.
    MaxHp {
        /// Player.
        player: PlayerId,
    },
    /// Resolve a player's zone by stable zone-kind tag.
    Zone {
        /// Zone owner (`NONE` for shared zones).
        player: PlayerId,
        /// 1=draw, 2=discard, 3=hand, 4=equip, 5=judge, 6=processing, 7=removed.
        kind: u8,
    },
    /// Current zone id of a card.
    CardZone {
        /// Card instance.
        card: CardId,
    },
    /// Current count of one stable player mark.
    MarkCount {
        /// Player.
        player: PlayerId,
        /// Mark definition id.
        mark: u32,
    },
    /// Number of living players.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    AliveCount,
    /// Number of cards a player has used or played as a response during the current turn.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CardActionCount {
        /// Player.
        player: PlayerId,
    },
    /// Number of distinct non-empty character groups among living players.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    AliveGroupCount,
    /// Whether a living player has strictly more current HP than every other living player.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    UniqueMaxCurrentHp {
        /// Player to compare.
        player: PlayerId,
    },
    /// Whether a living player would have strictly more current HP than every other living player
    /// if their current HP were `value`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    UniqueMaxCurrentHpAt {
        /// Player to compare.
        player: PlayerId,
        /// Hypothetical current HP for that player.
        value: i32,
    },
    /// Match-lifetime total damage points dealt by a player.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    LifetimeDamageDealt {
        /// Damage source to inspect.
        player: PlayerId,
    },
    /// Number of physical cards of one rule kind in the shared draw and discard piles.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    PublicCardKindCount {
        /// Stable rule card-kind tag.
        kind: u8,
    },
    /// Current round number capped by a content-supplied non-negative value.
    /// Appended to keep existing serialized enum ordinals stable.
    RoundNumberCapped {
        /// Inclusive upper bound.
        cap: u32,
    },
    /// Number of physical cards of one rule kind in living players' equipment zones.
    ///
    /// Appended to keep existing serialized query variant ordinals stable.
    FieldEquipmentKindCount {
        /// Stable rule card-kind tag.
        kind: u8,
    },
    /// Number of living wounded players, plus other living Wu players while `player` is the Lord
    /// and has enabled Guiming.
    ///
    /// Appended to keep existing serialized query variant ordinals stable.
    AliveWoundedOrGuimingWuCount {
        /// Skill owner whose Lord identity and Guiming state are inspected.
        player: PlayerId,
    },
    /// Number of cards in one player's hand/equipment region with the requested effective suit.
    ///
    /// Appended to keep existing serialized query variant ordinals stable.
    HandEquipSuitCount {
        /// Player whose cards are counted.
        player: PlayerId,
        /// Stable suit tag: 1=spade, 2=heart, 3=club, 4=diamond.
        suit: u8,
    },
    /// Current turn player's raw id (0 when no turn is in progress).
    ///
    /// Appended to keep existing serialized query variant ordinals stable.
    CurrentPhasePlayer,
    /// Number of cards in one player's hand with the requested effective suit.
    HandSuitCount {
        /// Player whose hand is counted.
        player: PlayerId,
        /// Stable suit tag: 1=spade, 2=heart, 3=club, 4=diamond.
        suit: u8,
    },
    /// Number of cards in one player's hand with the requested effective color.
    HandColorCount {
        /// Player whose hand is counted.
        player: PlayerId,
        /// Stable color tag: 1=red, 2=black.
        color: u8,
    },
    /// Boolean content flag on one owned skill definition, returned as 0/1.
    SkillContentFlag {
        /// Skill owner.
        player: PlayerId,
        /// Skill definition.
        skill_def: engine_api::SkillDefId,
        /// Flag bit in 0..64.
        bit: u8,
    },
    /// Number of cards in a player's skill pile (expansion) for one skill definition.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    SkillPileCount {
        /// Player whose skill pile is inspected.
        player: PlayerId,
        /// Skill definition id owning the pile.
        skill_def: u32,
    },
    /// Number of cards the player has drawn (gained into hand) during the current turn.
    /// Backed by `state.turn.card_gain_count`. Phase 3 history-count family.
    CardsDrawnThisTurn {
        /// Player to inspect.
        player: PlayerId,
    },
    /// Number of damage events the player has dealt as source during the current turn.
    /// Phase 3 history-count family.
    DamageDealtThisTurn {
        /// Damage source to inspect.
        player: PlayerId,
    },
    /// Whether the player currently owns (has) a skill with the given definition id.
    /// Phase 3 hasSkill-check family (mirrors upstream `player.hasSkill(...)`).
    HasSkill {
        /// Player to inspect.
        player: PlayerId,
        /// Skill definition id to look for.
        skill_def: engine_api::SkillDefId,
    },
    /// Rule-visible length of a card's localized name, matching upstream `get.cardNameLength`.
    /// Phase 3 card-name-length subcondition family.
    CardNameLength {
        /// Player perspective for effective-kind resolution.
        player: PlayerId,
        /// Card to inspect.
        card: engine_api::CardId,
    },
    /// Whether the current turn is in the Play phase (analogue of upstream
    /// `event.getParent("phaseUse")` for events that fire during the play phase). Phase 3 family.
    InPlayPhase,
    /// Whether one physical card carries a persistent content tag (gaintag).
    /// Implements upstream `card.hasGaintag(tag)`. Appended for ordinal stability.
    CardHasTag {
        /// Card to inspect.
        card: engine_api::CardId,
        /// Stable tag id (usually a skill definition id).
        tag: u32,
    },
    /// Value stored under `key` in one skill instance's keyed content storage, or 0 if absent.
    /// Implements upstream `player.getStorage(<skill>)[key]`. Appended for ordinal stability.
    StorageGet {
        /// Skill instance to inspect.
        skill: engine_api::SkillId,
        /// Storage key.
        key: u32,
    },
    /// Whether `key` is present in one skill instance's keyed content storage.
    /// Implements upstream `(key in player.getStorage(<skill>))`. Appended for ordinal stability.
    StorageHas {
        /// Skill instance to inspect.
        skill: engine_api::SkillId,
        /// Storage key.
        key: u32,
    },
    /// Number of proactive cards used by one player during the current round.
    /// Foundation for upstream `player.getRoundHistory("useCard").length`.
    /// Appended for ordinal stability.
    CardsUsedThisRound {
        /// Player to inspect.
        player: PlayerId,
    },}

/// Typed query response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryResponse {
    /// Integer result.
    Int(i32),
    /// Boolean result.
    Bool(bool),
    /// Relation tag.
    Relation(RelationTag),
    /// Query failed (invalid ids / depth / cycle).
    Error {
        /// Code.
        code: QueryErrorCode,
    },
    /// Number of cards in a player's skill pile (expansion) for one skill definition.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    SkillPileCount {
        /// Card count.
        count: i32,
    },
}

/// Identity / camp relation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationTag {
    /// Same camp (e.g. lord–loyalist).
    Ally,
    /// Enemy camp.
    Enemy,
    /// Self.
    Self_,
    /// Unknown (hidden identity).
    Unknown,
}

/// Query failure codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryErrorCode {
    /// Invalid entity id.
    InvalidId,
    /// Recursion / cycle detected.
    Cycle,
    /// Max depth exceeded.
    DepthExceeded,
    /// Not implemented yet.
    Unimplemented,
}

/// Provenance of a single modifier contribution (for debugger / explain).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModifierSource {
    /// Skill or rules module id.
    pub source_id: u32,
    /// Human key.
    pub label_key: String,
    /// Contribution value.
    pub value: i32,
}
