//! Typed primitive effect requests.
//!
//! Skills never mutate stores directly; they emit these requests to the scheduler.

use engine_api::{CardId, MarkDefId, PlayerId, SkillDefId, SkillId, Suit, ZoneId};
use serde::{Deserialize, Serialize};

fn no_skill() -> SkillId {
    SkillId::NONE
}

fn no_skill_def() -> SkillDefId {
    SkillDefId::NONE
}

/// Canonical target-card zones available to a typed choose/discard effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetCardScope {
    /// Equipment only.
    Equip,
    /// Hand and equipment.
    HandEquip,
    /// Hand, equipment, and judgment zones (legacy `hej`).
    HandEquipJudge,
    /// Judgment zone only.
    Judge,
    /// Equipment and judgment zones only (legacy `ej`, no hand).
    ///
    /// Appended for serialized ordinal and Rule VM payload-tag stability.
    EquipJudge,
    /// Hand only.
    ///
    /// Appended for serialized ordinal and Rule VM payload-tag stability.
    Hand,
}

impl TargetCardScope {
    /// Stable Rule VM payload tag.
    pub const fn tag(self) -> u8 {
        match self {
            Self::Equip => 0,
            Self::HandEquip => 1,
            Self::HandEquipJudge => 2,
            Self::Judge => 3,
            Self::EquipJudge => 5,
            Self::Hand => 4,
        }
    }

    /// Decode a stable Rule VM payload tag.
    pub const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::Equip),
            1 => Some(Self::HandEquip),
            2 => Some(Self::HandEquipJudge),
            3 => Some(Self::Judge),
            5 => Some(Self::EquipJudge),
            4 => Some(Self::Hand),
            _ => None,
        }
    }

    /// Whether the scope contains the hand zone.
    pub const fn includes_hand(self) -> bool {
        matches!(self, Self::HandEquip | Self::HandEquipJudge | Self::Hand)
    }

    /// Whether the scope contains the equipment zone.
    pub const fn includes_equip(self) -> bool {
        matches!(
            self,
            Self::Equip | Self::HandEquip | Self::HandEquipJudge | Self::EquipJudge
        )
    }

    /// Whether the scope contains the judgment zone.
    pub const fn includes_judge(self) -> bool {
        matches!(self, Self::HandEquipJudge | Self::Judge | Self::EquipJudge)
    }
}

/// Canonical living-player scopes available to typed target choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawTargetScope {
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
    /// Appended for stable serialized ordinals and Rule VM payload tags.
    MinHand,
    /// Any living player whose current HP equals the chooser's current HP.
    ///
    /// Appended for stable serialized ordinals and Rule VM payload tags.
    SameHp,
    /// Any living player with no hand cards.
    ///
    /// Appended for stable serialized ordinals and Rule VM payload tags.
    HandEmpty,
    /// Any living wounded player, including the chooser.
    ///
    /// Appended for stable serialized ordinals and Rule VM payload tags.
    Wounded,
    /// Any other living player whose hand count does not exceed the chooser's.
    ///
    /// Appended for stable serialized ordinals and Rule VM payload tags.
    HandLteChooser,
}

impl DrawTargetScope {
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

    /// Decode a stable Rule VM payload tag.
    pub const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::Other),
            1 => Some(Self::MaleOther),
            2 => Some(Self::FemaleOther),
            3 => Some(Self::Any),
            4 => Some(Self::MinHand),
            5 => Some(Self::SameHp),
            6 => Some(Self::HandEmpty),
            7 => Some(Self::Wounded),
            8 => Some(Self::HandLteChooser),
            _ => None,
        }
    }
}

fn no_player() -> PlayerId {
    PlayerId::NONE
}

fn no_card() -> CardId {
    CardId::NONE
}

fn no_suit() -> Suit {
    Suit::None
}

/// Damage nature tags for [`EffectRequest::DealDamage::nature`].
pub mod damage_nature {
    /// Ordinary damage.
    pub const NORMAL: u8 = 0;
    /// Fire damage (e.g. 火攻); 藤甲 +1.
    pub const FIRE: u8 = 1;
    /// Thunder damage (e.g. 闪电).
    pub const THUNDER: u8 = 2;
    /// Divine damage (神属性); elemental but does not propagate through iron chains.
    pub const KAMI: u8 = 3;
}

/// Stable selector groups for match-lifetime card-action rewards.
pub mod lifetime_card_reward_group {
    /// Any physical Sha nature or Shan.
    pub const SHA_OR_SHAN: u8 = 1;
    /// Tao or Jiu.
    pub const TAO_OR_JIU: u8 = 2;
    /// Duel or Wuzhong.
    pub const JUE_DOU_OR_WU_ZHONG: u8 = 3;
}

/// Provenance flags carried by card-linked damage.
pub mod damage_card_flag {
    /// The resolved Sha consumed a 酒 mark for its damage bonus.
    pub const JIU_BOOSTED: u8 = 1 << 0;
    /// This Sha was converted from ordinary to fire and must cost its user 1 HP after the whole
    /// use if any linked damage from it was committed.
    pub const CONVERTED_FIRE_LOSE_HP_AFTER_DAMAGE: u8 = 1 << 1;
    /// At least one damage segment belonging to the current card use was committed.
    pub const CARD_USE_DEALT_DAMAGE: u8 = 1 << 2;
    /// Draw for the originating skill after this exact Sha use if it commits no damage.
    pub const DRAW_IF_NO_DAMAGE: u8 = 1 << 3;
    /// This segment is elemental chain propagation rather than damage whose immediate parent is
    /// the originating card use.
    pub const CHAIN_PROPAGATED: u8 = 1 << 4;
    /// Damage belongs to the exact Sha used for old 解烦.
    pub const OLDJIEFAN: u8 = 1 << 5;
    /// The target was linked immediately before this damage segment committed.
    pub const TARGET_WAS_LINKED: u8 = 1 << 6;
    /// Damage belongs to a Sha forced by 备诛.
    pub const BEIZHU: u8 = 1 << 7;
}

/// Typed effect the Rule VM / content may request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectRequest {
    /// Draw `count` cards for `player`.
    DrawCards {
        /// Recipient.
        player: PlayerId,
        /// Number of cards.
        count: u32,
    },
    /// Move cards to a zone.
    MoveCards {
        /// Cards in order.
        cards: Vec<CardId>,
        /// Destination zone.
        to: ZoneId,
        /// Optional explicit source (validated by core).
        from: Option<ZoneId>,
    },
    /// Use a card (play-as-use semantics).
    UseCard {
        /// User.
        player: PlayerId,
        /// Card.
        card: CardId,
        /// Targets.
        targets: Vec<PlayerId>,
    },
    /// Use and consume a physical card from an equipment or judgment zone.
    UseFieldCard {
        /// User (which may differ from the field card's owner).
        player: PlayerId,
        /// Physical field card.
        card: CardId,
        /// Targets.
        targets: Vec<PlayerId>,
    },
    /// Deal damage.
    DealDamage {
        /// Source.
        source: PlayerId,
        /// Target.
        target: PlayerId,
        /// Amount.
        amount: u32,
        /// Lose-HP channel (skips armor etc. per rules).
        is_lose_hp: bool,
        /// Damage nature: 0=normal, 1=fire, 2=thunder, 3=divine (see `damage_nature`).
        #[serde(default)]
        nature: u8,
    },
    /// Recover HP.
    RecoverHp {
        /// Recovery source (`NONE` for unattributed recovery).
        #[serde(default = "no_player")]
        source: PlayerId,
        /// Target.
        player: PlayerId,
        /// Amount.
        amount: u32,
    },
    /// Change max HP.
    ChangeMaxHp {
        /// Target.
        player: PlayerId,
        /// Delta (may be negative).
        delta: i32,
    },
    /// Add skill instance from definition.
    AddSkill {
        /// Owner.
        player: PlayerId,
        /// Skill definition.
        skill_def: SkillDefId,
    },
    /// Remove skill instance.
    RemoveSkill {
        /// Skill instance.
        skill: SkillId,
    },
    /// Add / set mark stack.
    AddMark {
        /// Player.
        player: PlayerId,
        /// Mark def.
        mark: MarkDefId,
        /// Delta.
        delta: i32,
    },
    /// Remove mark stacks.
    RemoveMark {
        /// Player.
        player: PlayerId,
        /// Mark def.
        mark: MarkDefId,
        /// Amount to remove; 0 means all.
        amount: u32,
    },
    /// Reveal cards to observers.
    RevealCards {
        /// Cards.
        cards: Vec<CardId>,
        /// Recipients; empty = public.
        to: Vec<PlayerId>,
    },
    /// Request a player choice (pauses resolution).
    RequestChoice {
        /// Observer.
        observer: PlayerId,
        /// Min selections.
        min_count: u8,
        /// Max selections.
        max_count: u8,
        /// Allow cancel.
        allow_cancel: bool,
        /// Option tokens.
        option_tokens: Vec<u32>,
    },
    /// Enter dying.
    EnterDying {
        /// Player.
        player: PlayerId,
    },
    /// Kill player.
    KillPlayer {
        /// Victim.
        player: PlayerId,
        /// Killer if any.
        killer: PlayerId,
    },
    /// Set turn phase / current player (system).
    SetTurnState {
        /// Current player.
        current: PlayerId,
        /// Phase ordinal (maps to TurnPhaseTag in core).
        phase: u8,
    },
    /// Continue normal/extra turn selection after all End-phase triggers settle.
    ///
    /// Engine-owned; content does not emit this request.
    AdvanceTurn,
    /// Engine-internal: emit a DebateResolved event (ADR 0644 P2 resolvers only).
    EmitDebateResolved {
        /// Initiator.
        initiator: PlayerId,
        /// Red-opinion players.
        red: Vec<PlayerId>,
        /// Black-opinion players.
        black: Vec<PlayerId>,
        /// Result code.
        result: u8,
    },
    /// Emit a rule note event.
    EmitRuleEvent {
        /// Event code.
        code: u32,
        /// Payload.
        payload: Vec<u32>,
    },
    /// Discard up to `count` cards from hand (deterministic: hand zone order).
    DiscardFromHand {
        /// Player.
        player: PlayerId,
        /// Max cards to discard.
        count: u32,
    },
    /// Deterministically discard up to `count` cards from `player`'s hand, then equipment (he
    /// scope, in legacy seat/id order). Powers skills like 「惊澜」.
    ///
    /// Appended for serialized ordinal stability.
    DiscardFromHandEquip {
        /// Player.
        player: PlayerId,
        /// Max cards to discard.
        count: u32,
    },
    /// Discard an explicit set of cards owned by `player`.
    DiscardCards {
        /// Card owner.
        player: PlayerId,
        /// Selected cards, in command order.
        cards: Vec<CardId>,
    },
    /// Take the first card from `from`'s hand into `to`'s hand (deterministic).
    TakeFromHand {
        /// Source hand owner.
        from: PlayerId,
        /// Destination hand owner.
        to: PlayerId,
    },
    /// Record a logical response card after its cost has been paid.
    CardResponded {
        /// Responding player.
        player: PlayerId,
        /// Player whose card is being answered; none when no response relation exists.
        #[serde(default = "no_player")]
        respond_to: PlayerId,
        /// Physical card; none for a virtual response.
        card: CardId,
        /// Logical card-kind tag after view-as conversion.
        response_kind: u8,
    },
    /// Move every card of one rule kind from live play zones into the shared
    /// removed-from-game zone.
    RemoveCardsByKind {
        /// Stable rule card-kind tag.
        kind: u8,
    },
    /// Deal damage produced by a resolved logical Sha.
    ///
    /// Appended separately so card-specific late damage policies do not affect generic damage
    /// and existing serialized effect variant ordinals remain stable.
    DealShaDamage {
        /// Source.
        source: PlayerId,
        /// Target.
        target: PlayerId,
        /// Physical card carrying the logical Sha, or none for a cardless Sha.
        #[serde(default = "no_card")]
        card: CardId,
        /// Amount after ordinary Sha-use modifiers.
        amount: u32,
        /// Damage nature: 0=normal, 1=fire, 2=thunder.
        #[serde(default)]
        nature: u8,
        /// Card-use provenance flags; see [`damage_card_flag`].
        #[serde(default)]
        card_flags: u8,
    },
    /// Ask `player` to discard exactly one card from hand or equipment.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseDiscardOneHandOrEquip {
        /// Player making the forced choice.
        player: PlayerId,
    },
    /// Perform a judgment, then gain the topmost equipment from the draw pile
    /// for red or from the discard pile for black.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeGainEquipmentByColor {
        /// Player performing the judgment and gaining the equipment.
        player: PlayerId,
    },
    /// Reveal draw-pile cards, then perform the discard-by-suit gain option and/or
    /// use-one-revealed-card option in either order.
    ///
    /// The progress fields are engine-owned continuation state. Content emits them empty.
    /// Appended so existing serialized effect variant ordinals remain stable.
    RevealDiscardSuitGainOrUse {
        /// Player performing the choices.
        player: PlayerId,
        /// Mark whose current count expands the reveal and whose stack rewards both options.
        mark: MarkDefId,
        /// Base number of cards revealed.
        base_count: u8,
        /// Maximum counted/rewarded mark stacks.
        max_mark: u8,
        /// Revealed cards still tracked by this resolution.
        #[serde(default)]
        revealed: Vec<CardId>,
        /// Completed option bitset: 1=discard/gain, 2=use.
        #[serde(default)]
        completed: u8,
        /// Internal resume step: 0=choose/finish, 1=discard suit captured.
        #[serde(default)]
        step: u8,
        /// Step-specific stable scalar (currently the discarded card's effective suit tag).
        #[serde(default)]
        data: u32,
    },
    /// Choose the discard and/or fire-damage branches, applying their conditional phase skips.
    ///
    /// The progress fields are engine-owned continuation state. Content emits them empty.
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseDiscardOrFireDamageSkipPhases {
        /// Current-turn player performing the choices.
        player: PlayerId,
        /// Maximum cards discarded from the chosen target's hand/equipment zones.
        max_discard: u8,
        /// Fire damage dealt by the damage branch.
        damage: u8,
        /// Selected branch bitset: 1=discard, 2=damage.
        #[serde(default)]
        selected: u8,
        /// Internal resume step: 0=choose branches, 1=after discard.
        #[serde(default)]
        step: u8,
        /// Step-specific target.
        #[serde(default = "no_player")]
        target: PlayerId,
    },
    /// Draw cards while preserving the originating skill instance for trigger filters.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    DrawCardsFromSkill {
        /// Recipient.
        player: PlayerId,
        /// Number of cards.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Discard every card in `player`'s hand and equipment zones.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    DiscardAllHandAndEquip {
        /// Player losing all hand and equipment cards.
        player: PlayerId,
    },
    /// Open the deterministic multi-rescuer save queue for a dying player.
    ///
    /// Appended so forced dying-trigger choices can suspend this request without losing its
    /// continuation metadata.
    RequestDyingSave {
        /// Player currently dying.
        player: PlayerId,
        /// Damage source used if the player dies.
        source: PlayerId,
    },
    /// Draw for two players in seat order.
    ///
    /// `skill` is filled by the engine VM host so draw-trigger provenance remains exact.
    DrawEachBySeat {
        /// First participant.
        first: PlayerId,
        /// Second participant.
        second: PlayerId,
        /// Cards drawn by each participant.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Draw cards, then force the drawer to give one eligible hand/equipment card.
    ///
    /// The progress fields are engine-owned continuation state. Content emits them empty.
    DrawThenGiveOne {
        /// Drawer and giver.
        player: PlayerId,
        /// Recipient.
        target: PlayerId,
        /// Cards to draw.
        count: u32,
        /// Restrict the gift to cards drawn by this effect.
        drawn_only: bool,
        /// Let the recipient optionally use the gifted card when it is equipment.
        #[serde(default)]
        use_given_equipment: bool,
        /// Originating skill instance.
        skill: SkillId,
        /// Cards drawn by the initial stage.
        #[serde(default)]
        drawn: Vec<CardId>,
        /// False for draw stage, true for give stage.
        #[serde(default)]
        ready_to_give: bool,
    },
    /// Draw cards, then let another living player take one hand/equipment card.
    ///
    /// The progress field is engine-owned continuation state. Content emits it false.
    DrawThenTakeOne {
        /// Drawer and card owner.
        player: PlayerId,
        /// Player choosing and taking the card.
        taker: PlayerId,
        /// Cards to draw.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
        /// False for draw stage, true for take stage.
        #[serde(default)]
        ready_to_take: bool,
    },
    /// Let one player choose and take one card from another player's allowed zones.
    TakeOneCard {
        /// Card owner.
        from: PlayerId,
        /// Player choosing and taking the card.
        to: PlayerId,
        /// Restrict the choice to the equipment zone.
        equip_only: bool,
    },
    /// Toggle a player's character card between face-up and face-down.
    ///
    /// A face-down player skips their next whole turn and returns face-up.
    TurnOver {
        /// Player whose character card is toggled.
        player: PlayerId,
    },
    /// Remove every skill instance with one definition from a player.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    RemoveSkillByDef {
        /// Skill owner.
        player: PlayerId,
        /// Definition to remove.
        skill_def: SkillDefId,
    },
    /// Optionally discard one hand card, then add a skill selected by its effective color.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseDiscardHandAddSkillByColor {
        /// Player making the discard choice and receiving the skill.
        player: PlayerId,
        /// Skill added after discarding a red card.
        red_skill_def: SkillDefId,
        /// Skill added after discarding a black card.
        black_skill_def: SkillDefId,
    },
    /// Reveal the draw-pile top card, gain it when non-basic, or optionally
    /// replace a basic card with one hand card placed on top of the draw pile.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    RevealTopGainNonBasicOrSwapBasic {
        /// Player revealing, gaining, and optionally swapping the card.
        player: PlayerId,
    },
    /// Prevent the damage event whose pre-resolution trigger is currently running.
    ///
    /// Content emits `damage_id = 0`; the engine VM host binds the current event id.
    /// Appended so existing serialized effect variant ordinals remain stable.
    PreventDamage {
        /// Engine-owned damage event id.
        #[serde(default)]
        damage_id: u32,
    },
    /// Apply damage after its `damage_before` trigger queue has completed.
    ///
    /// Engine-owned; content bytecode cannot construct this request.
    /// Appended so existing serialized effect variant ordinals remain stable.
    CommitDamage {
        /// Engine-owned damage event id, or zero when no pre-trigger matched.
        damage_id: u32,
        /// Source.
        source: PlayerId,
        /// Target.
        target: PlayerId,
        /// Final amount after ordinary damage modifiers.
        amount: u32,
        /// Amount propagated through an elemental chain before target-side modifiers.
        base_amount: u32,
        /// Lose-HP channel.
        is_lose_hp: bool,
        /// Damage nature.
        nature: u8,
        /// Physical card that caused the damage, or none.
        #[serde(default = "no_card")]
        card: CardId,
        /// Logical card-kind tag, or zero when the damage was not card-linked.
        #[serde(default)]
        card_kind: u8,
        /// Card-use provenance flags; see [`damage_card_flag`].
        #[serde(default)]
        card_flags: u8,
    },
    /// Perform a judgment and, on a red result, exclude one target from the
    /// currently resolving Sha.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeCancelCurrentShaTargetOnRed {
        /// Player performing the judgment.
        player: PlayerId,
        /// Physical Sha whose target is being resolved.
        card: CardId,
        /// Target excluded on a red judgment.
        target: PlayerId,
    },
    /// Let a specified player choose between drawing cards and recovering HP.
    /// Full-HP players draw immediately because recovery is unavailable.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseDrawOrRecover {
        /// Player making the choice and receiving the result.
        player: PlayerId,
        /// Cards drawn by the draw branch.
        draw_count: u32,
        /// HP recovered by the recovery branch.
        recover_count: u32,
        /// Whether declining both branches is legal.
        allow_cancel: bool,
    },
    /// Damage linked to a logical card without reapplying card-specific source modifiers.
    /// Used internally for elemental chain propagation.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    DealCardDamage {
        /// Source.
        source: PlayerId,
        /// Target.
        target: PlayerId,
        /// Amount before target-side chain modifiers.
        amount: u32,
        /// Lose-HP channel.
        is_lose_hp: bool,
        /// Damage nature.
        nature: u8,
        /// Physical source card, or none.
        card: CardId,
        /// Logical card-kind tag.
        card_kind: u8,
        /// Card-use provenance flags; see [`damage_card_flag`].
        #[serde(default)]
        card_flags: u8,
    },
    /// Materialize a suitless virtual Sha and let its user choose legal targets.
    ///
    /// The card still traverses the ordinary use/response/damage pipeline and is
    /// moved to the removed zone after use. Appended so existing serialized
    /// effect variant ordinals remain stable.
    ChooseUseVirtualSha {
        /// Player using the virtual Sha.
        player: PlayerId,
        /// Whether range is ignored while target legality is otherwise preserved.
        ignore_distance: bool,
    },
    /// Use a virtual Sha with a base-damage bonus (国战克复中原：蜀势力目标出杀的
    /// 伤害值基数 +1). Appended for serialized ordinal stability.
    ChooseUseVirtualShaWithBonus {
        /// Player using the virtual Sha.
        player: PlayerId,
        /// Whether range is ignored while target legality is otherwise preserved.
        ignore_distance: bool,
        /// Base-damage bonus applied to the used Sha.
        damage_bonus: u32,
    },
    /// Adjust a player's hand to an exact count: draw when below it, otherwise
    /// force that player to choose the excess hand cards to discard.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    AdjustHandTo {
        /// Player whose hand is adjusted.
        player: PlayerId,
        /// Exact target hand count.
        count: u32,
    },
    /// Gain the topmost card of one rule kind, searching the draw pile before
    /// the discard pile.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    GainCardByKind {
        /// Player gaining the card.
        player: PlayerId,
        /// Stable rule card-kind tag.
        kind: u8,
    },
    /// Perform a judgment and recover HP when its effective suit matches.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeRecoverOnSuit {
        /// Skill owner credited as the recovery source.
        source: PlayerId,
        /// Player performing the judgment and receiving any recovery.
        player: PlayerId,
        /// Effective suit required for recovery.
        suit: Suit,
        /// HP recovered on a matching result.
        amount: u32,
    },
    /// Move cards as a voluntary transfer from one player to another.
    ///
    /// This engine-level primitive preserves the semantic movement reason for
    /// content triggers. Appended so existing serialized effect ordinals remain stable.
    GiveCards {
        /// Cards in transfer order.
        cards: Vec<CardId>,
        /// Recipient hand zone.
        to: ZoneId,
        /// Explicit source personal zone.
        from: ZoneId,
    },
    /// Materialize and use a suitless virtual card against one explicit target.
    ///
    /// The card traverses the ordinary use/nullification/resolution pipeline and
    /// is moved to the removed zone after use. Appended for ordinal stability.
    UseVirtualCardOn {
        /// User.
        player: PlayerId,
        /// Stable logical card-kind tag.
        kind: u8,
        /// Explicit target.
        target: PlayerId,
    },
    /// Emit the semantic completion of one Duel after its damage has resolved.
    ///
    /// Engine-owned; appended for serialized ordinal stability.
    EmitDuelResolved {
        /// Physical or materialized virtual card that carried the Duel.
        card: CardId,
        /// Player who used the Duel.
        initiator: PlayerId,
        /// Original Duel target.
        target: PlayerId,
        /// Participant who dealt Duel damage.
        winner: PlayerId,
        /// Participant who received Duel damage.
        loser: PlayerId,
    },
    /// Emit a completed judgment after every rewrite opportunity has settled.
    ///
    /// Engine-owned; appended for serialized ordinal stability.
    EmitJudgmentResolved {
        /// Player whose judgment completed.
        player: PlayerId,
        /// Physical card carrying the final judgment result.
        result_card: CardId,
        /// Final effective suit for the judged player.
        suit: Suit,
        /// Final rank.
        rank: u8,
        /// Delayed trick/equipment that caused the judgment, or `NONE`.
        cause_card: CardId,
    },
    /// Perform a judgment, then force the judging player to damage another
    /// living player when the effective suit matches.
    ///
    /// Appended for serialized ordinal stability.
    JudgeDamageOtherOnSuit {
        /// Player performing the judgment and dealing any resulting damage.
        player: PlayerId,
        /// Effective suit required for the target choice.
        suit: Suit,
        /// Damage dealt on a matching result.
        amount: u32,
        /// Stable damage nature tag.
        nature: u8,
    },
    /// Recast selected personal-zone cards, then draw the same count.
    ///
    /// Appended for serialized ordinal stability.
    RecastCards {
        /// Player recasting the cards and drawing replacements.
        player: PlayerId,
        /// Physical cards selected by the active invocation.
        cards: Vec<CardId>,
    },
    /// Disable the requested currently enabled equipment slots.
    ///
    /// Slot tags are stable (`1` weapon through `5` treasure). Equipment occupying a newly
    /// disabled slot is discarded. Appended for serialized ordinal stability.
    DisableEquipSlots {
        /// Affected player.
        player: PlayerId,
        /// Stable equipment-slot tags in deterministic request order.
        slots: Vec<u8>,
    },
    /// Let `player` move one equipment card to another legal living player, excluding
    /// `player` as the destination, then let the equipment's original owner draw one card.
    ///
    /// The card and destination choices are resolved against canonical state and revalidated
    /// at both choice barriers. Appended for serialized ordinal stability.
    MoveEquipmentThenDraw {
        /// Skill owner making both choices and forbidden as the destination.
        player: PlayerId,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
    /// Let `player` use a suitless virtual Sha without consuming the ordinary Sha-use count;
    /// if no legal use is selected, draw one card instead.
    ///
    /// The target choice still applies ordinary distance and target-legality rules. Appended for
    /// serialized ordinal stability.
    ChooseUseVirtualShaOrDraw {
        /// Player choosing between a legal virtual Sha use and drawing one card.
        player: PlayerId,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
    /// Discard one equipment card from each of two distinct selected players, requiring at
    /// least one selected card to be armor.
    ///
    /// Target and card choices are revalidated against canonical discard-protection rules at
    /// every barrier. Appended for serialized ordinal stability.
    DiscardEquipmentPairWithArmor {
        /// Skill owner choosing the two equipment cards.
        player: PlayerId,
        /// Two distinct living targets selected by the active invocation.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let `player` choose and discard one equipment card from `target`; if that card is a
    /// mount and remains in the discard pile, move it into `player`'s hand.
    ///
    /// The forced card choice and discard protection are revalidated at the continuation
    /// barrier. Appended for serialized ordinal stability.
    DiscardTargetEquipmentGainMount {
        /// Player choosing the equipment card and receiving a discarded mount.
        player: PlayerId,
        /// Living damaged player whose equipment may be discarded.
        target: PlayerId,
    },
    /// Let `player` choose and discard one card from `target` in a typed zone scope; if that
    /// card is a mount and remains in the discard pile, move it into `player`'s hand.
    ///
    /// Appended for serialized ordinal stability. ABI-33 packages keep using
    /// [`Self::DiscardTargetEquipmentGainMount`]; ABI-34 content uses this generalized form.
    DiscardTargetCardGainMount {
        /// Player choosing the card and receiving a discarded mount.
        player: PlayerId,
        /// Living damaged player whose card may be discarded.
        target: PlayerId,
        /// Canonical zones offered to the chooser.
        scope: TargetCardScope,
    },
    /// Discard concrete cards from another player's typed target-card scope.
    ///
    /// This preserves semantic `Discard` movement events for hand/equipment/judgment effects
    /// while reusing the canonical discard-protection query. Appended for ordinal stability.
    DiscardTargetCards {
        /// Player causing the discard.
        actor: PlayerId,
        /// Player who owns the selected cards.
        target: PlayerId,
        /// Concrete cards selected from the target.
        cards: Vec<CardId>,
        /// Canonical zones allowed by the effect.
        scope: TargetCardScope,
    },
    /// Make one target unable to respond to the currently resolving physical Sha.
    ///
    /// The request mutates only the saved in-flight Sha continuation. It never installs a
    /// persistent modifier, and is rejected when the source/card/target tuple does not match
    /// that continuation. Appended for serialized ordinal stability.
    MakeCurrentShaTargetUnrespondable {
        /// Sha user and skill owner.
        source: PlayerId,
        /// Physical Sha currently resolving.
        card: CardId,
        /// Current Sha target whose response window is disabled.
        target: PlayerId,
    },
    /// Record that one target fully dodged the currently resolving logical Sha.
    ///
    /// This engine-internal intent emits the typed `ShaDodged` domain event only after every
    /// required Shan has resolved. Appended for serialized ordinal stability.
    ShaDodged {
        /// Sha user.
        source: PlayerId,
        /// Target who dodged the Sha.
        target: PlayerId,
        /// Current logical Sha carrier (which may be a view-as source card), or `NONE` for a
        /// source-free virtual Sha.
        card: CardId,
    },
    /// Let `player` choose and discard one card from `target` in a typed zone scope.
    ///
    /// The forced choice and canonical discard protection are revalidated at the continuation
    /// barrier. Appended for serialized ordinal stability.
    ChooseDiscardTargetCard {
        /// Player choosing the card.
        player: PlayerId,
        /// Living player whose card may be discarded.
        target: PlayerId,
        /// Canonical zones offered to the chooser.
        scope: TargetCardScope,
    },
    /// Perform a judgment and, on a non-heart result, prevent the currently resolving damage
    /// and reduce the damaged player's maximum HP by one.
    ///
    /// Content emits `damage_id = 0`; the engine VM host binds the current pre-damage event.
    /// Appended for serialized ordinal stability.
    JudgePreventDamageLoseMaxHpOnNonHeart {
        /// Skill owner performing the judgment.
        player: PlayerId,
        /// Damaged player whose maximum HP may be reduced.
        target: PlayerId,
        /// Engine-owned damage event id.
        #[serde(default)]
        damage_id: u32,
    },
    /// Privately view draw-pile top cards, then optionally reveal and gain any matching-suit
    /// subset while leaving every unselected card on top in its original relative order.
    ///
    /// Appended for serialized ordinal stability.
    ChooseGainTopCardsBySuit {
        /// Player viewing and choosing cards.
        player: PlayerId,
        /// Maximum number of top cards viewed.
        count: u8,
        /// Required effective suit.
        suit: Suit,
    },
    /// Repeatedly let one player choose another living player, then draw for the selected
    /// target using one of two counts according to whether that target's hand is empty.
    ///
    /// Every iteration is independently cancellable. Appended for serialized ordinal stability.
    ChooseOtherDrawByHandCount {
        /// Player making each target choice.
        player: PlayerId,
        /// Cards drawn when the target has at least one hand card.
        normal_count: u32,
        /// Cards drawn when the target has no hand cards.
        empty_count: u32,
        /// Remaining independently cancellable choices.
        times: u32,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
    /// Optionally discard one hand/equipment card, then gain every equipped card in one slot
    /// from `target` that still exists when the cost resolves.
    ///
    /// The discard selection and target equipment are revalidated at the continuation barrier.
    /// Appended for serialized ordinal stability.
    ChooseDiscardOneThenGainTargetEquipment {
        /// Player choosing and paying the discard cost.
        player: PlayerId,
        /// Player whose matching equipment is gained.
        target: PlayerId,
        /// Stable equipment-slot tag (`1` weapon through `5` treasure).
        slot: u8,
    },
    /// Let one player choose a living target from a typed scope, then make that target draw.
    ///
    /// Appended for serialized ordinal stability.
    ChooseTargetDraw {
        /// Player making the target choice.
        player: PlayerId,
        /// Cards drawn by the selected target.
        count: u32,
        /// Canonical target scope.
        scope: DrawTargetScope,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
    /// Draw cards equal to the number of distinct logical card categories the player used in
    /// the current play phase (basic, trick, equipment).
    ///
    /// The bounded history is maintained by the engine only when loaded content requests it.
    /// Appended for serialized ordinal stability.
    DrawByPlayPhaseCardCategoryCount {
        /// Player whose play-phase history is queried and who draws the cards.
        player: PlayerId,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
    /// Refund the currently resolving proactive Sha's use count at most once, then let the
    /// skill owner draw if that Sha is effectively red.
    ///
    /// The current saved Sha continuation, source, and physical card are revalidated by the
    /// engine. Appended for serialized ordinal stability.
    RefundCurrentShaUseAndDrawIfRed {
        /// Skill owner receiving the possible draw.
        player: PlayerId,
        /// Player who used the currently resolving Sha.
        sha_user: PlayerId,
        /// Physical card carrying the logical Sha.
        card: CardId,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
    /// Force one hand/equipment-card discard, then refund the currently resolving proactive
    /// Sha's use count at most once and draw for the skill owner if that Sha is effectively red.
    ///
    /// The engine rechecks the cost options when this effect starts and revalidates the selected
    /// card at the choice barrier. The refund/draw follow-up is scheduled only after payment.
    /// Appended for serialized ordinal stability.
    ChooseDiscardOneForShaRefundAndDrawIfRed {
        /// Skill owner choosing and paying the discard cost.
        player: PlayerId,
        /// Player who used the currently resolving Sha.
        sha_user: PlayerId,
        /// Physical card carrying the logical Sha.
        card: CardId,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
    /// Queue one extra Play phase at the opening of the specified player's current turn.
    ///
    /// The engine executes queued phases before Prepare and gives every Play phase an independent
    /// Sha-use allowance and play-phase counters. Appended for serialized ordinal stability.
    QueueOpeningExtraPlayPhase {
        /// Player receiving the extra Play phase.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal turn-opening boundary. Appended for serialized ordinal stability.
    BeginTurn {
        /// Player whose turn begins.
        player: PlayerId,
    },
    /// Engine-internal continuation after all turn-begin triggers resolve.
    FinishTurnBegin {
        /// Current player.
        player: PlayerId,
    },
    /// Engine-internal transition after an opening extra Play phase ends.
    AdvanceOpeningExtraPlay {
        /// Current player.
        player: PlayerId,
    },
    /// Convert the currently saved ordinary Sha to fire and attach its post-use HP-loss rule.
    ///
    /// The source/card tuple must match the proactive Sha continuation. Appended for serialized
    /// ordinal stability.
    ConvertCurrentShaToFireLoseHpIfDamage {
        /// Sha user.
        player: PlayerId,
        /// Physical card carrying the logical Sha.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let the Sha user optionally add one legal living target to the currently saved Sha.
    ///
    /// The original target list, selected target, range, and card policies are revalidated at the
    /// choice barrier. Appended for serialized ordinal stability.
    ChooseAddCurrentShaTarget {
        /// Sha user and chooser.
        player: PlayerId,
        /// Physical card carrying the logical Sha.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal commit after the optional Sha target choice is revalidated.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    CommitAddCurrentShaTarget {
        /// Sha user.
        player: PlayerId,
        /// Physical card carrying the logical Sha.
        card: CardId,
        /// Newly selected target.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose one or more logical Sha from hand and store them in this skill's public pile.
    /// Appended for serialized ordinal stability.
    ChooseStoreHandShaInSkillPile {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose one card from this skill's public pile, discard it, then make the dying player use
    /// one suitless virtual Jiu on themselves. Appended for serialized ordinal stability.
    ChooseSpendSkillPileForDyingJiu {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Current dying player who will use Jiu.
        dying: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal commit of selected hand Sha into a skill pile.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    CommitStoreHandShaInSkillPile {
        /// Skill owner.
        player: PlayerId,
        /// Revalidated selected hand Sha.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal commit of one skill-pile card for a dying player's virtual Jiu.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    CommitSpendSkillPileForDyingJiu {
        /// Skill owner.
        player: PlayerId,
        /// Current dying player.
        dying: PlayerId,
        /// Selected skill-pile card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Compare the active invocation's selected hand card with one hand card chosen by `target`.
    /// The two cards are revealed and discarded; strict victory grants a target-bound helper,
    /// while a tie or loss grants the ordinary losing helper.
    /// Appended for serialized ordinal stability.
    ChooseCompareSelectedHandCard {
        /// Skill owner and first comparison participant.
        player: PlayerId,
        /// Other comparison participant.
        target: PlayerId,
        /// Owner's card captured from the active invocation's selected-card input.
        card: CardId,
        /// Helper skill granted on strict victory and bound to `target`.
        win_skill_def: SkillDefId,
        /// Helper skill granted on tie or loss.
        lose_skill_def: SkillDefId,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Add a skill whose versioned instance state binds it to one player.
    /// Appended for serialized ordinal stability.
    AddBoundSkill {
        /// Skill owner.
        player: PlayerId,
        /// Skill definition.
        skill_def: SkillDefId,
        /// Bound target encoded in the new instance state.
        bound_target: PlayerId,
    },
    /// Optionally append the current skill instance's bound target to the suspended one-target
    /// Sha or ordinary-trick use.
    /// Appended for serialized ordinal stability.
    ChooseAddSkillBoundCardTarget {
        /// Current card user and chooser.
        player: PlayerId,
        /// Physical card whose use is suspended in a CardUsed trigger.
        card: CardId,
        /// Originating target-bound skill instance.
        skill: SkillId,
    },
    /// Engine-internal commit of a selected bound extra target.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    CommitAddSkillBoundCardTarget {
        /// Current card user.
        player: PlayerId,
        /// Physical card whose use is suspended.
        card: CardId,
        /// Revalidated bound target.
        target: PlayerId,
        /// Originating target-bound skill instance.
        skill: SkillId,
    },
    /// Optionally discard one non-basic hand/equipment card whose color matches one of the
    /// chooser's equipment cards, then grant a color-lock helper to `target`.
    /// Appended for serialized ordinal stability.
    ChooseDiscardMatchingEquipColorAddSkill {
        /// Skill owner choosing and paying the cost.
        player: PlayerId,
        /// Other player's current Play phase receiving the helper.
        target: PlayerId,
        /// Temporary color-lock helper definition.
        skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Add or update a color-lock helper. Repeated grants merge color masks and replace the
    /// stored source with the latest grant.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    AddCardColorLockSkill {
        /// Player receiving the helper.
        player: PlayerId,
        /// Latest skill source used by the helper's follow-up.
        source: PlayerId,
        /// Bit 0 = red, bit 1 = black.
        color_mask: u8,
        /// Temporary helper definition.
        skill_def: SkillDefId,
    },
    /// If the helper owner used no logical Sha in the current Play phase, make the source stored
    /// in this skill instance use a virtual Sha on them without distance or use-count limits.
    /// Appended for serialized ordinal stability.
    UseVirtualShaFromCardColorLockSourceIfNoSha {
        /// Helper owner and retaliation target.
        player: PlayerId,
        /// Originating color-lock helper instance.
        skill: SkillId,
    },
    /// Force the user of the suspended one-target Sha or ordinary trick to choose one or two
    /// reusable enhancements for that exact card use.
    /// Appended for serialized ordinal stability.
    ChooseCurrentCardUseEnhancements {
        /// Current card user and chooser.
        player: PlayerId,
        /// Physical card whose use is suspended.
        card: CardId,
        /// Temporary helper definition that stores exact-card policies.
        helper_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Add or merge exact-card policies in a temporary helper skill.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    AddCardUsePolicySkill {
        /// Helper owner and card user.
        player: PlayerId,
        /// Physical card receiving the policies.
        card: CardId,
        /// Bit 0 = ignore armor, bit 1 = unrespondable, bit 2 = draw per damage event.
        policy_mask: u8,
        /// Temporary helper definition.
        skill_def: SkillDefId,
    },
    /// Force a choice between drawing one card and discarding one hand/equipment card, then
    /// grant the branch-specific temporary skill.
    /// Appended for serialized ordinal stability.
    ChooseDrawOrDiscardAddSkill {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Temporary skill granted before drawing one card.
        draw_skill_def: SkillDefId,
        /// Temporary skill granted before discarding one card.
        discard_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
        /// Offer the cancellable plain-draw-one / draw-two / discard three-way contract.
        #[serde(default)]
        three_way: bool,
    },
    /// Repeatedly let the owner use one black basic card from the triggering discard batch as
    /// a distance-unlimited Bingliang. Each iteration is optional and revalidated.
    /// Appended for serialized ordinal stability.
    ChooseDiscardedBlackBasicAsBingliang {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Physical cards captured from the triggering move batch by the engine VM host.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Optionally discard one judgment card from `target`, then make `player` use a source-free
    /// Sha on that target and draw one card if that exact Sha deals no damage.
    /// Appended for serialized ordinal stability.
    ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player whose judgment phase is beginning.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal commit of one fixed virtual Sha with a skill-attributed no-damage draw.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    UseVirtualShaDrawIfNoDamage {
        /// Virtual Sha user.
        player: PlayerId,
        /// Fixed Sha target.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Discard one selected hand/equipment card after atomically recording its logical category
    /// and the chosen target in the originating skill's turn-scoped uniqueness state.
    /// Appended for serialized ordinal stability.
    DiscardSelectedRecordUniqueCategoryTarget {
        /// Skill owner and card owner.
        player: PlayerId,
        /// Other player selected by the active skill.
        target: PlayerId,
        /// Selected physical cost card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let the owner choose whether the target draws or discards the originating skill's current
    /// turn use count, then link the owner if the target is wounded.
    /// Appended for serialized ordinal stability.
    ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Player affected by the branch.
        target: PlayerId,
        /// Toggle linked state instead of only linking the owner.
        #[serde(default)]
        toggle_link: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal finalization after the selected draw/discard branch and its nested triggers.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    FinishLinkIfTargetWounded {
        /// Player to link if currently unlinked.
        player: PlayerId,
        /// Player whose wounded state is tested after the branch.
        target: PlayerId,
        /// Toggle linked state instead of only linking the owner.
        #[serde(default)]
        toggle_link: bool,
        /// Originating skill instance for attribution.
        skill: SkillId,
    },
    /// Let `player` choose another living player with hand cards, choose one of those cards,
    /// reveal it publicly, and remember its logical category for the current Play phase.
    /// Appended for serialized ordinal stability.
    ChooseRevealOtherHandCardRememberCategory {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Draw cards, then give exactly `count` hand/equipment cards to the other current
    /// Play-phase player and record that player for a delayed phase consequence.
    /// Appended for serialized ordinal stability.
    DrawThenGiveExactRecordPhaseTarget {
        /// Skill owner drawing and giving cards.
        player: PlayerId,
        /// Other player's current Play phase.
        target: PlayerId,
        /// Draw and gift count.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
        /// Internal continuation flag set after the draw and nested draw triggers finish.
        ready_to_give: bool,
    },
    /// Clear this skill instance's delayed Play-phase gift target.
    /// Appended for serialized ordinal stability.
    ClearPhaseGiftTarget {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Emit the engine-owned boundary for one fully resolved logical Sha use.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    EmitShaResolved {
        /// Sha user.
        source: PlayerId,
        /// Physical Sha carrier or engine materialization.
        card: CardId,
        /// Complete declared target list.
        targets: Vec<PlayerId>,
        /// Physical Shan source cards in response order.
        response_cards: Vec<CardId>,
        /// Stable card-use flags carried to the post-resolution trigger context.
        card_flags: u8,
    },
    /// Choose a legal recipient, give that recipient the resolved Sha or its physical Shan
    /// sources still in discard, then optionally use one physical Sha source if the gift is red.
    /// Appended for serialized ordinal stability.
    ChooseGainResolvedShaOrShanThenOptionalUseSha {
        /// Skill owner and original Sha user.
        player: PlayerId,
        /// Physical Sha carrier.
        sha_card: CardId,
        /// Physical Shan source cards observed during this Sha use.
        shan_cards: Vec<CardId>,
        /// Original Sha targets, all excluded from receiving the gift.
        excluded_recipients: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal continuation after the gifted cards and their movement triggers finish.
    /// Appended for serialized ordinal stability; Rule VM bytecode never emits this directly.
    ChooseUseShaAfterResolvedGift {
        /// Original skill owner whose attack range constrains the follow-up Sha.
        owner: PlayerId,
        /// Gift recipient who may use a Sha.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Skip the owner's current non-idle phase and enter the next phase.
    /// Appended for serialized ordinal stability.
    SkipCurrentPhase {
        /// Current phase owner.
        player: PlayerId,
    },
    /// Choose up to the current turn's skipped-phase count of other players and deal damage to
    /// them in the submitted target order.
    /// Appended for serialized ordinal stability.
    ChooseOtherTargetsBySkippedPhaseCountDamage {
        /// Skill owner and damage source.
        player: PlayerId,
        /// Damage dealt to each selected target.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Publicly reveal the owner's current hand. If every card has the same effective color,
    /// let the owner choose up to the hand count of other living players, then draw one card for
    /// every chosen player and the owner in canonical seat order.
    /// Appended for serialized ordinal stability.
    RevealUniformHandDrawSelfChooseOthers {
        /// Skill owner, hand owner, and target chooser.
        player: PlayerId,
        /// Originating skill instance for validation and draw provenance.
        skill: SkillId,
    },
    /// Choose another legal target for a suspended single-target Sha/ordinary trick. That
    /// player may give one hand card and become the user, or become an additional target.
    /// Appended for serialized ordinal stability.
    ChooseCardUserOrAddTarget {
        /// Current card user, skill owner, and first chooser.
        player: PlayerId,
        /// Suspended physical card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal user transfer after the selected hand card and all nested movement
    /// triggers finish. Rule VM bytecode never emits this directly.
    /// Appended for serialized ordinal stability.
    CommitTransferSavedCardUser {
        /// Previous card user and skill owner.
        previous: PlayerId,
        /// New card user who paid the hand-card gift.
        new_user: PlayerId,
        /// Suspended physical card.
        card: CardId,
        /// Originating skill instance for provenance.
        skill: SkillId,
    },
    /// Choose and discard one effective equipment card from hand/equipment, then reduce the
    /// bound pre-resolution damage by one.
    /// Appended for serialized ordinal stability.
    ChooseDiscardEquipmentReduceCurrentDamage {
        /// Damaged skill owner and chooser.
        player: PlayerId,
        /// Engine-owned pre-resolution damage id.
        #[serde(default)]
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose and discard one effective equipment card from hand/equipment, then prevent the
    /// entire bound pre-resolution damage segment.
    ChooseDiscardEquipmentPreventCurrentDamage {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Engine-owned pre-resolution damage id.
        #[serde(default)]
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose to move one compatible equipment card from `target` to `player`, draw one card,
    /// or cancel. Appended for serialized ordinal stability.
    ChooseMoveTargetEquipmentOrDraw {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player whose turn just ended.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Give all of the owner's current equipment to `target`, then let that target choose the
    /// count-bound recover-or-damage branch. Appended for serialized ordinal stability.
    GiveAllEquipmentChooseRecoverOrDamage {
        /// Limited active skill owner.
        player: PlayerId,
        /// Recipient and later branch chooser.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal branch choice after the equipment gift and nested movement triggers.
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    ChooseRecoverOwnerOrDamageByGivenEquipment {
        /// Original equipment owner and possible recovery target.
        owner: PlayerId,
        /// Equipment recipient, branch chooser, recovery source, and damage source.
        recipient: PlayerId,
        /// Number of equipment cards captured before the gift.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let the target discard exactly the source's current physical weapon range from hand. If
    /// the target cannot or declines, increase the bound pre-resolution Sha damage by one.
    /// Appended for serialized ordinal stability.
    ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
        /// Sha source and originating skill owner.
        player: PlayerId,
        /// Bound damage target and discard chooser.
        target: PlayerId,
        /// Engine-owned pre-resolution damage id.
        #[serde(default)]
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose a living player whose hand count differs from the owner. The target draws one when
    /// lower, or discards one authoritative deterministic-random hand card when higher.
    /// Appended for serialized ordinal stability.
    ChooseHandCountDifferentTargetDrawOrRandomDiscard {
        /// Skill owner and target chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Increase the pending card count for the current draw phase. The draw-phase command
    /// consumes the accumulated value exactly once.
    /// Appended for serialized ordinal stability.
    IncreaseCurrentDrawCount {
        /// Current draw-phase player.
        player: PlayerId,
        /// Additional cards.
        count: u32,
    },
    /// During the owner's Prepare phase, choose a living target and discard the target's
    /// hand-minus-HP amount (at least one) from hand/equipment. The discarded non-equipment
    /// count becomes the optional self-discard cost; declining makes the target draw instead.
    /// Appended for serialized ordinal stability.
    ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw {
        /// Skill owner and target/card chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal follow-up after the target discard and its nested movement triggers.
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    ChooseSelfDiscardOrTargetDrawAfterTargetDiscard {
        /// Skill owner who may pay the discard cost.
        player: PlayerId,
        /// Original target who draws when the owner declines or cannot pay.
        target: PlayerId,
        /// Number of non-equipment cards discarded from the target.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Draw one card, then choose and gain one card in legacy seat order from the damaged
    /// player, that player's previous living seat, and that player's next living seat.
    /// Content emits `ready_to_choose = false`; the engine-owned second stage runs only after
    /// the draw and all nested draw triggers have finished.
    /// Appended for serialized ordinal stability.
    DrawThenChooseGainOneFromDamageTargetAndNeighbors {
        /// Damage source, skill owner, drawer, and card chooser.
        player: PlayerId,
        /// Player damaged by the originating damage event.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// False for the draw stage and true for the post-draw seat-list stage.
        #[serde(default)]
        ready_to_choose: bool,
    },
    /// Engine-internal sequential gain continuation. `selected = NONE` scans for the next
    /// eligible seat and opens a choice; a concrete card performs the gain and defers the next
    /// scan until all nested movement triggers finish. Rule VM bytecode never emits this.
    /// Appended for serialized ordinal stability.
    ContinueChooseGainOneFromDamageTargetAndNeighbors {
        /// Skill owner and card chooser.
        player: PlayerId,
        /// Snapshotted legacy order, including intentional duplicate seats.
        targets: Vec<PlayerId>,
        /// Current index in `targets`.
        index: u32,
        /// Selected card to gain, or NONE while scanning/opening the next choice.
        selected: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Force the damage source to choose between preventing the bound damage or losing one
    /// maximum HP and increasing that same damage by the requested amount.
    /// Content emits damage_id = 0; the engine VM host binds the current pre-damage event.
    /// Appended for serialized ordinal stability.
    ChoosePreventDamageOrLoseMaxHpIncreaseDamage {
        /// Damage source, skill owner, and chooser.
        player: PlayerId,
        /// Engine-owned pre-resolution damage id.
        #[serde(default)]
        damage_id: u32,
        /// Amount added to both the pending damage and its base amount.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-internal continuation that increases a still-bound pending damage after the
    /// maximum-HP loss and all nested max-HP-change triggers have resolved.
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    ContinueIncreaseBoundDamage {
        /// Original damage source.
        player: PlayerId,
        /// Engine-owned pre-resolution damage id.
        damage_id: u32,
        /// Amount added to both the pending damage and its base amount.
        amount: u32,
        /// Originating skill instance for provenance.
        skill: SkillId,
    },
    /// Engine-internal continuation that opens or resolves the second participant's hand-card
    /// selection after every `hand_compare_started` trigger has completed.
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    ContinueChooseCompareTargetHandCard {
        /// First comparison participant.
        player: PlayerId,
        /// Second comparison participant.
        target: PlayerId,
        /// Initially selected card of the first participant.
        card: CardId,
        /// Helper granted on strict victory.
        win_skill_def: SkillDefId,
        /// Helper granted on tie or loss.
        lose_skill_def: SkillDefId,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Make the other participant's card in the currently starting hand comparison be selected
    /// by authoritative deterministic RNG. Appended for serialized ordinal stability.
    RandomizeCurrentHandCompareOpponent {
        /// Skill owner and one comparison participant.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Emit the completed hand-comparison boundary after both physical cards have been moved.
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    EmitHandCompareFinished {
        /// First comparison participant.
        player: PlayerId,
        /// Second comparison participant.
        target: PlayerId,
        /// First participant's comparison card.
        player_card: CardId,
        /// Second participant's comparison card.
        target_card: CardId,
        /// Strict winner, or `NONE` for a tie.
        winner: PlayerId,
    },
    /// Gain every highest-rank logical Sha among the physical cards captured by the triggering
    /// domain event and still present in the discard pile. Appended for serialized stability.
    GainHighestRankShaFromEventCards {
        /// Skill owner receiving the cards.
        player: PlayerId,
        /// Physical event cards copied into the typed request by the Rule VM host.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Gain the legacy `zongshi` result card from a completed two-player hand comparison when
    /// that physical card is still in the discard pile. Appended for serialized stability.
    GainHandCompareResultCard {
        /// Skill owner receiving the card.
        player: PlayerId,
        /// First comparison participant.
        initiator: PlayerId,
        /// Second comparison participant.
        target: PlayerId,
        /// Strict winner, or `NONE` for a tie.
        winner: PlayerId,
        /// Physical comparison cards in `[initiator, target]` order.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose whether to add one legal target to, or remove one existing target from, the
    /// currently resolving basic or trick card. Appended for serialized stability.
    ChooseAddOrRemoveCurrentCardTarget {
        /// Card user, skill owner, and chooser.
        player: PlayerId,
        /// Physical card whose saved target list may be adjusted.
        card: CardId,
        /// Originating one-shot skill instance.
        skill: SkillId,
    },
    /// Choose another living player with hand cards, then let both participants choose one hand
    /// card for a comparison. Appended for serialized stability.
    ChooseHandCompare {
        /// First participant, skill owner, and chooser.
        player: PlayerId,
        /// Helper granted on strict victory.
        win_skill_def: SkillDefId,
        /// Helper granted on tie or loss.
        lose_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose one other living player for a hand comparison; on the chooser's strict win the
    /// **target** is granted `win_grant_def`, on tie or loss nothing happens. Appended for
    /// serialized stability (ABI 678, batch 166 tanhu).
    ChooseHandCompareGrantTargetOnWin {
        /// First participant, skill owner, and chooser.
        player: PlayerId,
        /// Helper granted to the target on the chooser's strict victory.
        win_grant_def: SkillDefId,
        /// Reserved for symmetry with `ChooseHandCompare`; unused in the grant flow.
        lose_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Active selection: give the chosen hand/equip cards to `target`, then the **chooser**
    /// recovers `amount` HP. Appended for serialized stability (ABI 679, batch 172 jun_renwang).
    GiveSelectedThenRecoverSelf {
        /// Chooser and giver; recovers after the gift.
        player: PlayerId,
        /// Gift recipient.
        target: PlayerId,
        /// Chosen cards (injected from the active selection).
        cards: Vec<CardId>,
        /// Recovery amount for the chooser.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Third-party hand comparison (ABI 680, batch 174 jun_zhaoshou): `first` picks an
    /// opponent (excluding `player`), both stash one hand card, and on a non-tie result the
    /// winner uses a distance-free virtual Sha against the loser.
    ChooseHandCompareBetween {
        /// Skill owner / originating observer.
        player: PlayerId,
        /// First comparison participant (also picks the opponent).
        first: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Watch the top `count` cards of the draw pile and re-order them one by one: each
    /// round the player chooses top-sequence or bottom-sequence for the current card
    /// (ABI 681, batch 176 guanxing family). Multi-round state lives in the continuation
    /// payload.
    GuanxingTop {
        /// Viewer.
        player: PlayerId,
        /// Number of top cards to watch and re-order.
        count: u32,
        /// Round cursor (0-based); carried through the continuation payload.
        round: u32,
        /// Flipped cards in draw order.
        flipped: Vec<CardId>,
        /// Cards placed into the top sequence (in pick order).
        top: Vec<CardId>,
        /// Cards placed into the bottom sequence (in pick order).
        bottom: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose one other living player with hand cards: they give one hand card to the
    /// chooser, then **they** recover 1 HP (ABI 682, batch 180 psdaohe 踏鹤).
    ChooseOtherGiveThenRecover {
        /// Chooser and gift recipient.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Watch the top N cards of the draw pile, where N is the number of non-zero stacks
    /// among the four declared falu marks; pick top/bottom per card as ABI 681 (batch 183
    /// xinfu_dianhua 花色化法箓).
    GuanxingTopByFaluMarks {
        /// Viewer.
        player: PlayerId,
        /// Four falu mark defs (spade, heart, club, diamond).
        marks: [SkillDefId; 4],
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose one other living player and deal fixed normal damage. Appended for serialized
    /// stability.
    ChooseOtherDamage {
        /// Skill owner, damage source, and chooser.
        player: PlayerId,
        /// Damage amount.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose another living player, then choose and discard one card in the supplied zone scope.
    /// Appended for serialized stability.
    ChooseOtherDiscardTargetCard {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Canonical zones offered for the selected target.
        scope: TargetCardScope,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Gain the topmost card of one rule kind from the discard pile only.
    /// Appended for serialized stability.
    GainDiscardCardByKind {
        /// Player gaining the card.
        player: PlayerId,
        /// Stable rule card-kind tag.
        kind: u8,
    },
    /// Reveal all hand cards and, when their effective color is uniform, force the owner to
    /// choose any living player and deal fixed normal damage. Appended for serialized stability.
    RevealUniformHandChooseDamage {
        /// Skill owner, hand owner, damage source, and chooser.
        player: PlayerId,
        /// Damage amount.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Force the player to choose and gain one matching card from the discard pile.
    /// Appended for serialized stability.
    ChooseGainDiscardCardByKind {
        /// Player choosing and gaining the card.
        player: PlayerId,
        /// Stable rule card-kind tag.
        kind: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Draw one card for each supplied divisor of the player's match-lifetime number of cards
    /// used or played as a response. Appended for serialized stability.
    DrawForLifetimeCardActionDivisors {
        /// Player whose action history is inspected and who draws the cards.
        player: PlayerId,
        /// Three positive divisors checked independently.
        divisors: [u32; 3],
    },
    /// Force the owner to choose and gain one candidate skill definition they do not yet own.
    /// Appended for serialized stability.
    ChooseAddMissingSkill {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Three distinct candidate skill definitions.
        skill_defs: [SkillDefId; 3],
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Force a player to choose a fixed number of distinct living targets, then each draws.
    /// Appended for serialized stability.
    ChooseTargetsDraw {
        /// Player making the target choice.
        player: PlayerId,
        /// Cards drawn by each selected target.
        count: u32,
        /// Exact number of distinct targets.
        target_count: u8,
        /// Canonical target scope.
        scope: DrawTargetScope,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Gain every card captured by the triggering event that remains in the discard pile.
    /// Appended for serialized stability.
    GainEventCards {
        /// Player gaining the cards.
        player: PlayerId,
        /// Stable event-card snapshot supplied by the engine host.
        cards: Vec<CardId>,
    },
    /// Force the chooser to select any living player, then draw that target up to the lesser of
    /// their maximum HP and a fixed cap. Appended for serialized stability.
    ChooseTargetDrawToMaxHpCap {
        /// Player making the target choice.
        player: PlayerId,
        /// Inclusive upper bound applied to the selected target's maximum HP.
        cap: u32,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
    /// Force the owner to choose another living player. The target draws cards equal to the
    /// owner's current lost HP, then turns over. Appended for serialized stability.
    ChooseOtherDrawOwnerLostHpTurnOver {
        /// Skill owner making the target choice.
        player: PlayerId,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
    /// Perform a judgment for one player and recover another player if the effective suit
    /// matches. Appended for serialized stability.
    JudgeRecoverOtherOnSuit {
        /// Recovery source, normally the skill owner.
        source: PlayerId,
        /// Player performing the judgment.
        judge_player: PlayerId,
        /// Player receiving any recovery.
        recover_player: PlayerId,
        /// Required effective judgment suit.
        suit: Suit,
        /// Recovery amount.
        amount: u32,
    },
    /// In clockwise seat order, force every other snapshotted living player to use a physical
    /// or view-as Sha on a nearest player, or lose one HP. Appended for serialized stability.
    ForceOthersUseShaOnNearestOrLoseHp {
        /// Skill owner excluded from the forced-use queue.
        player: PlayerId,
        /// Remaining players in the activation-time clockwise snapshot.
        remaining: Vec<PlayerId>,
        /// False only for the Rule VM's initial request; prevents an empty completed queue from
        /// being mistaken for a fresh activation.
        started: bool,
        /// Originating once-per-game skill instance.
        skill: SkillId,
    },
    /// Continue a deterministic multi-player dying-save queue. Engine-internal and appended for
    /// serialized stability.
    ContinueDyingSave {
        /// Player currently dying.
        player: PlayerId,
        /// Damage source used if the player dies.
        source: PlayerId,
        /// Remaining responders in clockwise order.
        responders: Vec<PlayerId>,
    },
    /// Restore a living character card to its normal state: unlinked and face-up.
    ///
    /// Appended for serialized stability.
    RestoreCharacterState {
        /// Player whose character card state is restored.
        player: PlayerId,
    },
    /// Force every card in a player's hand, equipment, and judgment zones into discard.
    ///
    /// Appended for serialized stability.
    DiscardAllHandEquipJudge {
        /// Player whose three public/private character zones are emptied.
        player: PlayerId,
    },
    /// Choose and discard one hand/equipment card from the current Sha target.
    ///
    /// If the selected card is equipment, that target cannot respond to this Sha. Otherwise,
    /// the target gains the physical Sha if it remains in the discard pile. Appended for
    /// serialized stability.
    ChooseDiscardTargetCardResolveCurrentSha {
        /// Current Sha user and card chooser.
        source: PlayerId,
        /// Physical Sha currently resolving.
        card: CardId,
        /// Current target whose card may be discarded.
        target: PlayerId,
    },
    /// Materialize one source-free virtual card and use it on the active skill's selected targets.
    ///
    /// Appended for serialized stability.
    UseVirtualCardOnSelectedTargets {
        /// Virtual card user.
        player: PlayerId,
        /// Stable logical card-kind tag.
        kind: u8,
        /// Distinct active-skill targets in submitted order.
        targets: Vec<PlayerId>,
    },
    /// Deal ordinary damage and ban one active skill for the current turn if this exact damage
    /// makes the target enter dying.
    ///
    /// Appended for serialized stability.
    DealDamageBanActiveIfDying {
        /// Damage source and skill owner.
        source: PlayerId,
        /// Damage target.
        target: PlayerId,
        /// Base damage amount.
        amount: u32,
        /// Active skill instance to ban.
        skill: SkillId,
    },
    /// 峻攻：弃置至多 `use_count + 1` 张牌或失去等量体力，然后对目标使用虚拟【杀】。
    ///
    /// 虚拟【杀】经普通结算队列，命中目标时由配套触发禁用该主动技至回合结束。
    SpjungongDiscardOrLoseThenSha {
        /// 峻攻 owner；弃牌/失体力者与虚拟【杀】使用者。
        player: PlayerId,
        /// 峻攻虚拟【杀】目标。
        target: PlayerId,
        /// 来源主动技实例。
        skill: SkillId,
    },
    /// 把技能实例的 `active_turn_banned` 置位，禁用该主动技直到当前回合结束。
    BanActiveUntilTurnEnd {
        /// 技能拥有者。
        player: PlayerId,
        /// 被禁用的主动技实例。
        skill: SkillId,
    },
    /// 挽危（主动阶段）：目标回复 player.hp 点体力，拥有者失去 player.hp 点体力。
    SpwanweiActive {
        /// 挽危拥有者。
        player: PlayerId,
        /// 回复体力的目标角色。
        target: PlayerId,
    },
    /// 挽危（濒死救援）：目标回复 max(player.hp + 1, 1 - target.hp) 体力，拥有者失去 player.hp 点体力。
    SpwanweiDying {
        /// 挽危拥有者。
        player: PlayerId,
        /// 处于濒死状态的目标角色。
        target: PlayerId,
    },
    /// Engine-owned prepared form of [`EffectRequest::DealDamageBanActiveIfDying`].
    ///
    /// Content bytecode cannot construct this request. Appended for serialized stability.
    CommitDamageBanActiveIfDying {
        /// Engine-owned damage event id, or zero when no pre-trigger matched.
        damage_id: u32,
        /// Damage source and skill owner.
        source: PlayerId,
        /// Damage target.
        target: PlayerId,
        /// Final amount after ordinary damage modifiers.
        amount: u32,
        /// Amount propagated through an elemental chain before target-side modifiers.
        base_amount: u32,
        /// Damage nature after damage-before rewrites.
        #[serde(default)]
        nature: u8,
        /// Active skill instance to ban if the target enters dying.
        skill: SkillId,
    },
    /// Choose one of the target's equipped cards and move it into that target's own hand.
    ///
    /// After nested movement triggers finish, the owner draws one card only if the number of
    /// living players inside the target's attack range decreased. Appended for serialized
    /// stability.
    ChooseTargetReclaimEquipmentDrawIfRangeReduced {
        /// Skill owner and equipment-card chooser.
        player: PlayerId,
        /// Other player reclaiming the selected equipment.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-owned committed equipment reclamation after the card choice is resolved.
    ///
    /// Content bytecode cannot construct this request. Appended for serialized stability.
    CommitTargetReclaimEquipmentDrawIfRangeReduced {
        /// Skill owner and equipment-card chooser.
        player: PlayerId,
        /// Player reclaiming the selected equipment.
        target: PlayerId,
        /// Selected card still expected in the target's equipment zone.
        card: CardId,
        /// Attack-range population captured before the equipment leaves play.
        before: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-owned post-movement range comparison for
    /// [`EffectRequest::ChooseTargetReclaimEquipmentDrawIfRangeReduced`].
    ///
    /// Content bytecode cannot construct this request. Appended for serialized stability.
    FinishTargetReclaimEquipmentDrawIfRangeReduced {
        /// Skill owner who may draw.
        player: PlayerId,
        /// Player whose attack-range population is compared.
        target: PlayerId,
        /// Attack-range population captured before the equipment left play.
        before: u32,
        /// Originating skill instance for draw attribution.
        skill: SkillId,
    },
    /// Use a suitless, source-free virtual Wanjian or Juedou through the ordinary card pipeline.
    ///
    /// Wanjian resolves against its complete legal target set immediately. Juedou opens a
    /// mandatory target choice. Appended for serialized stability.
    ChooseUseVirtualCard {
        /// Virtual card user.
        player: PlayerId,
        /// Stable card-kind tag; ABI 98 accepts Wanjian and Juedou.
        kind: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-owned tail that ends phase-history suppression after a phase-end virtual card
    /// finishes every nested choice and response.
    ///
    /// Content bytecode cannot construct this request. Appended for serialized stability.
    FinishPhaseEndVirtualCard,
    /// Increase the base damage of the currently suspended proactive Sha use.
    ///
    /// The source/card pair is revalidated against the saved continuation. Appended for
    /// serialized stability.
    IncreaseCurrentShaBaseDamage {
        /// Current Sha user.
        player: PlayerId,
        /// Physical card backing the current Sha.
        card: CardId,
        /// Amount added to the saved base damage.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Mark the current living player's upcoming Play phase to be skipped.
    ///
    /// The request is valid only before Play in that player's current turn. Appended for
    /// serialized stability.
    SkipNextPlayPhase {
        /// Current turn player.
        player: PlayerId,
    },
    /// Repeatedly let one player choose another living player, then force the selected target
    /// to choose between drawing cards and recovering HP.
    ///
    /// Every target choice is independently cancellable. Appended for serialized stability.
    ChooseOtherDrawOrRecover {
        /// Player making each target choice.
        player: PlayerId,
        /// Cards drawn by the selected target's draw branch.
        draw_count: u32,
        /// HP recovered by the selected target's recovery branch.
        recover_count: u32,
        /// Remaining independently cancellable target choices.
        times: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Force a player to discard the requested number of hand/equipment cards, capped by the
    /// currently available cards, then remove the originating one-shot skill.
    ///
    /// Cleanup is deferred until movement triggers and any nested choices finish. Appended for
    /// serialized stability.
    ChooseDiscardHandEquipThenRemoveSelfSkill {
        /// Player making the forced card choice.
        player: PlayerId,
        /// Requested discard count before capping to available cards.
        count: u32,
        /// Originating one-shot skill instance removed after the discard resolves.
        skill: SkillId,
    },
    /// Draw cards for the distinct living targets selected by the active-skill command.
    ///
    /// The VM snapshots `targets` from the authoritative active invocation. The engine resolves
    /// them in seat order. Appended for serialized stability.
    DrawSelectedTargets {
        /// Active-skill targets.
        targets: Vec<PlayerId>,
        /// Cards drawn by each target.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Recover HP only when the player is tied for the lowest current HP among living players.
    ///
    /// The condition is evaluated when the effect resolves, not while bytecode is emitted.
    /// Appended for serialized stability.
    RecoverIfMinCurrentHp {
        /// Potential recovery recipient.
        player: PlayerId,
        /// Maximum HP recovered.
        amount: u32,
    },
    /// Increase the base damage of the currently suspended damaging card use.
    ///
    /// ABI 104 accepts proactive Sha, Nanman, and Wanjian; ABI 229 also accepts Juedou.
    /// The source/card pair is revalidated against the saved continuation. Appended for
    /// serialized stability.
    IncreaseCurrentCardBaseDamage {
        /// Current card user.
        player: PlayerId,
        /// Physical card backing the current use.
        card: CardId,
        /// Amount added to the saved base damage.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Lose HP, reduce maximum HP only if that exact loss first enters dying, then damage another
    /// player after the complete dying/save window.
    ///
    /// Appended for serialized stability.
    LoseHpThenDamageOtherLoseMaxHpIfDying {
        /// Skill owner who loses HP and later sources the damage.
        player: PlayerId,
        /// Other player damaged after the owner's dying window.
        target: PlayerId,
        /// Shared HP-loss and damage amount.
        amount: u32,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Engine-owned prepared form of
    /// [`EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying`].
    ///
    /// Content bytecode cannot construct this request. Appended for serialized stability.
    CommitLoseHpThenDamageOtherLoseMaxHpIfDying {
        /// Skill owner whose HP loss is committed.
        player: PlayerId,
        /// Other player damaged after the owner's dying window.
        target: PlayerId,
        /// Shared HP-loss and damage amount.
        amount: u32,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Recover HP for the distinct living targets selected by the active-skill command.
    ///
    /// The VM snapshots `targets` from the authoritative active invocation and the engine
    /// resolves them in seat order. Appended for serialized stability.
    RecoverSelectedTargets {
        /// Skill owner credited as the recovery source.
        source: PlayerId,
        /// Active-skill targets.
        targets: Vec<PlayerId>,
        /// HP recovered by each target.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Record the selected card's effective category for the current play phase.
    ///
    /// Appended for serialized stability.
    RecordSelectedPlayPhaseCategory {
        /// Selected physical card, captured from the authoritative active invocation.
        card: CardId,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Take one hand card from the mandatory larger-hand target, then optionally repeat with
    /// freshly chosen larger-hand targets until the owner ties for the largest living hand.
    ///
    /// Appended for serialized stability.
    TakeTargetHandThenRepeatUntilMax {
        /// Skill owner choosing and gaining cards.
        player: PlayerId,
        /// Mandatory first target selected by the active-skill command.
        target: PlayerId,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Engine-owned scan for the next optional larger-hand target after movement triggers finish.
    ///
    /// Rule VM bytecode cannot construct this request. Appended for serialized stability.
    ContinueTakeLargerHandUntilMax {
        /// Skill owner choosing and gaining cards.
        player: PlayerId,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Choose between drawing cards, increasing the current card's base damage, or performing
    /// both and then adding a mark.
    ///
    /// Content emits the current event card and no skill id; the engine VM host binds the
    /// originating skill. Appended for serialized ordinal stability.
    ChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
        /// Current card user, skill owner, and chooser.
        player: PlayerId,
        /// Physical card backing the current use.
        card: CardId,
        /// Cards drawn by the first and third branches.
        draw_count: u32,
        /// Base damage added by the second and third branches.
        damage_amount: u32,
        /// Stable mark definition added by the third branch.
        mark_def: MarkDefId,
        /// Mark count added by the third branch.
        mark_count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Draw cards, wait for the draw's nested triggers, then choose either to give an exact
    /// number of hand cards to another living player or discard that many hand cards.
    ///
    /// Appended for serialized ordinal stability.
    DrawThenChooseGiveOrDiscardHand {
        /// Skill owner drawing and paying cards.
        player: PlayerId,
        /// Cards drawn before the branch choice.
        draw_count: u32,
        /// Exact hand-card payment count when enough cards remain.
        payment_count: u32,
        /// Originating skill instance.
        skill: SkillId,
        /// Internal continuation flag set after draw events and nested triggers finish.
        ready_to_choose: bool,
    },
    /// Gain every card currently equipped by `target` in one equipment slot.
    ///
    /// The target zone and card slot are revalidated when this request resolves. Appended for
    /// serialized ordinal stability.
    GainTargetEquipment {
        /// Player gaining the equipment.
        player: PlayerId,
        /// Player whose matching equipment is gained.
        target: PlayerId,
        /// Stable equipment-slot tag (`1` weapon through `5` treasure).
        slot: u8,
    },
    /// Discard every equipment card that `actor` may currently discard from `target`.
    ///
    /// The target's living state, equipment zone, and discard protection are revalidated when
    /// this request resolves. Appended for serialized ordinal stability.
    DiscardAllTargetEquipment {
        /// Player causing the discard.
        actor: PlayerId,
        /// Player whose discardable equipment is discarded.
        target: PlayerId,
    },
    /// Increase only one target's damage for the currently resolving physical Sha.
    ///
    /// The source/card/target tuple is revalidated against the saved proactive Sha continuation.
    /// Appended for serialized ordinal stability.
    IncreaseCurrentShaTargetDamage {
        /// Current Sha user.
        source: PlayerId,
        /// Physical card backing the current Sha.
        card: CardId,
        /// Current Sha target.
        target: PlayerId,
        /// Additional damage for this target.
        amount: u32,
    },
    /// Choose any living target to draw cards. After the draw and all nested triggers finish,
    /// the owner draws one card if that target's hand remains below their maximum HP.
    ///
    /// `target == NONE` is the VM-emitted target-choice stage. `ready_to_check` is set only by
    /// the engine after the selected target's draw has completed.
    /// Appended for serialized ordinal stability.
    ChooseTargetDrawThenOwnerIfBelowMaxHp {
        /// Skill owner and target chooser.
        player: PlayerId,
        /// Selected draw target, or `NONE` before the choice.
        target: PlayerId,
        /// Cards drawn by the selected target.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the target draw and its nested triggers have completed.
        ready_to_check: bool,
    },
    /// Choose one living player in a bound source's attack range, then deal normal damage from
    /// that source. Appended for serialized ordinal stability.
    ChooseBoundSourceAttackRangeDamage {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Skill-bound damage source.
        source: PlayerId,
        /// Damage amount.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose any living target with positive maximum HP. The target draws cards equal to the
    /// lesser of their current maximum HP and `cap`; after nested draw triggers finish, they
    /// discard excess hand cards down to the freshly evaluated limit.
    ///
    /// `target == NONE` is the VM-emitted target-choice stage. `ready_to_discard` is set only by
    /// the engine after the selected target's draw has completed. The chooser may already be dead
    /// so that death-trigger skills can finish. Appended for serialized ordinal stability.
    ChooseTargetDrawMaxHpCapThenDiscardExcess {
        /// Skill owner and target chooser.
        player: PlayerId,
        /// Selected draw target, or `NONE` before the choice.
        target: PlayerId,
        /// Inclusive upper bound applied to the target's maximum HP.
        cap: u32,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the draw and its nested triggers have completed.
        ready_to_discard: bool,
    },
    /// Choose any living target with positive maximum HP. The target draws cards equal to the
    /// lesser of their current maximum HP and `cap`; after the draw finishes, the target gains
    /// the skill from `add_def` (the DSL side adds/removes the temp skill on its own triggers).
    ///
    /// `target == NONE` is the VM-emitted target-choice stage; the choice resolver re-emits with
    /// the selected target. Appended for serialized ordinal stability. ABI 455 (C 段 omp,
    /// `beizhan`).
    ChooseTargetDrawMaxHpCapThenAddSkill {
        /// Skill owner and target chooser.
        player: PlayerId,
        /// Selected draw target, or `NONE` before the choice.
        target: PlayerId,
        /// Inclusive upper bound applied to the target's maximum HP.
        cap: u32,
        /// Originating skill instance.
        skill: SkillId,
        /// Skill definition granted to the selected target after the draw.
        add_def: SkillDefId,
    },
    /// Give one active-skill-selected hand card to another living player. After the movement and
    /// nested triggers complete, that recipient chooses either to reveal their current hand and
    /// discard every discardable hand/equipment card matching the given card's physical suit, or
    /// lose 1 HP.
    ///
    /// `ready_to_choose` is false only for the VM-emitted gift stage. The engine captures `suit`
    /// from `card` before moving it and schedules the true stage behind the movement barrier.
    /// Appended for serialized ordinal stability.
    GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
        /// Skill owner giving the selected card.
        player: PlayerId,
        /// Other living recipient who makes the follow-up choice.
        target: PlayerId,
        /// Exact active-skill hand-card input.
        card: CardId,
        /// Captured physical suit of the given card.
        suit: Suit,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the gift and its nested triggers have completed.
        ready_to_choose: bool,
    },
    /// Gain the topmost equipment card in the shared draw pile.
    ///
    /// This request never searches the discard pile. Appended for serialized ordinal stability.
    GainEquipmentFromDrawPile {
        /// Player gaining the equipment card.
        player: PlayerId,
    },
    /// Give every current hand card from `player` to `target`, then recover `target` to exactly
    /// one HP after the movement and all nested triggers finish.
    ///
    /// Content emits `ready_to_recover = false`; the engine-owned second stage runs behind the
    /// movement barrier. Appended for serialized ordinal stability.
    GiveAllHandRecoverToOne {
        /// Living skill owner giving every hand card.
        player: PlayerId,
        /// Other living dying recipient.
        target: PlayerId,
        /// Whether the gift and all nested movement triggers have completed.
        ready_to_recover: bool,
    },
    /// Choose a living target who can self-discard one hand/equipment card, let that target
    /// discard exactly one, then make the owner lose one HP if the target's hand count differs
    /// from their HP after all nested movement triggers finish.
    ///
    /// Content emits `target = NONE` and `ready_to_check = false`; the engine owns the target,
    /// card-choice, and post-movement stages. Appended for serialized ordinal stability.
    ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
        /// Living skill owner choosing the target.
        player: PlayerId,
        /// Selected target, or NONE before target selection.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the target's discard and all nested movement triggers have completed.
        ready_to_check: bool,
    },
    /// Let `giver` choose whether to give one current hand/equipment card to `target`; if the
    /// giver declines or has no eligible card, recover the target by one HP.
    ///
    /// `owner` is the originating skill owner and may differ from both choice observer and
    /// recipient. Appended for serialized ordinal stability.
    ChooseGiveOneHandEquipOrRecover {
        /// Living owner of the originating skill.
        owner: PlayerId,
        /// Living player who observes the card choice and gives the selected card.
        giver: PlayerId,
        /// Living recipient, normally the current dying player.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Replace an active skill with a target-bound helper after the current resolution barrier,
    /// but only if the target is still alive.
    ///
    /// Content emits `ready_to_replace = false`; the engine-owned second stage runs after
    /// damage triggers and any dying/save window. Appended for serialized ordinal stability.
    ReplaceSkillWithBoundAfterResolutionIfTargetAlive {
        /// Owner of both the replaced skill and the target-bound helper.
        player: PlayerId,
        /// Exact skill instance to replace.
        skill: SkillId,
        /// Helper skill definition to add.
        skill_def: SkillDefId,
        /// Target encoded into the helper instance when still alive.
        bound_target: PlayerId,
        /// Whether the current resolution barrier has completed.
        ready_to_replace: bool,
    },
    /// Force the skill owner to choose a nonempty hand or equipment zone, discard every
    /// discardable card there, draw the discarded count plus current lost HP after movement
    /// triggers finish, then lose 1 HP after draw triggers finish.
    ///
    /// `zone` is 0 before the zone branch, 1 for hand, or 2 for equipment.
    /// `ready_to_draw` is false until the discard and all nested movement triggers complete.
    /// Appended for serialized ordinal stability.
    ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
        /// Living skill owner and forced chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Chosen zone tag: 0 = unresolved, 1 = hand, 2 = equipment.
        zone: u8,
        /// Discarded-card count captured before the movement.
        discarded_count: u32,
        /// Whether the discard and its nested triggers have completed.
        ready_to_draw: bool,
    },
    /// Force a player to discard exactly `count` current hand/equipment cards, then recover
    /// `amount` HP after movement triggers and nested choices finish.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDiscardHandEquipThenRecover {
        /// Skill owner and forced chooser.
        player: PlayerId,
        /// Exact number of cards required.
        count: u32,
        /// Maximum HP recovered after the discard resolves.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Force a player to choose a bounded number of living targets in a canonical scope, then
    /// deal normal damage to each selected target in seat order.
    ///
    /// Appended for serialized ordinal stability.
    ChooseTargetsDamage {
        /// Player making the choice and dealing the damage.
        player: PlayerId,
        /// Minimum number of distinct targets.
        min_count: u8,
        /// Maximum number of distinct targets.
        max_count: u8,
        /// Canonical target scope.
        scope: DrawTargetScope,
        /// Damage dealt to each selected target.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Force the skill owner to choose and discard one current equipment card from a target;
    /// when the selected card is a weapon, the target draws its physical attack range after
    /// movement triggers finish.
    ///
    /// Appended for serialized ordinal stability.
    DiscardTargetEquipmentDrawWeaponRange {
        /// Skill owner choosing and causing the discard.
        player: PlayerId,
        /// Living target whose equipment is discarded and who may draw.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Force the skill owner to choose a living target with hand cards, publicly reveal that
    /// target's complete current hand, and deal normal damage when it contains an effective Sha.
    ///
    /// Appended for serialized ordinal stability.
    ChooseTargetRevealHandDamageIfSha {
        /// Skill owner choosing the target and dealing the damage.
        player: PlayerId,
        /// Damage dealt when the revealed hand contains an effective Sha.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Force the skill owner to choose another living player, then give that target exactly one
    /// current hand/equipment card.
    ///
    /// Appended for serialized ordinal stability.
    ChooseOtherGiveOneHandEquip {
        /// Skill owner choosing the target and card.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Force the skill owner to choose and publicly reveal one current hand card from the
    /// specified Play-phase player, then bind that player and the card's effective category to
    /// this skill instance until the phase changes.
    ///
    /// Appended for serialized ordinal stability.
    ChooseRevealHandCardBindCategoryTarget {
        /// Skill owner choosing and revealing the card.
        player: PlayerId,
        /// Current Play-phase player whose hand card is revealed.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Consume a limited skill, recover its owner to max HP, then optionally sacrifice another
    /// skill to restore a fresh instance of the limited skill after recovery fully resolves.
    ///
    /// Appended for serialized ordinal stability.
    RecoverToMaxThenOptionalExchangeSkill {
        /// Skill owner recovering and choosing the exchange.
        player: PlayerId,
        /// Originating limited skill instance; consumed in the first stage.
        skill: SkillId,
        /// Skill definition that may be sacrificed.
        removable_skill_def: SkillDefId,
        /// Originating definition captured before the instance is consumed.
        restore_skill_def: SkillDefId,
        /// Whether recovery and all nested triggers have completed.
        ready_to_choose: bool,
    },
    /// Publicly reveal a player's complete current hand, then discard every discardable hand
    /// card whose effective kind matches `kind`.
    ///
    /// Appended for serialized ordinal stability.
    RevealHandDiscardKind {
        /// Player revealing and discarding cards.
        player: PlayerId,
        /// Stable logical card-kind tag.
        kind: u8,
    },
    /// Let the skill owner cancel, draw one, move one compatible equipment card from `target`,
    /// or perform both available actions in move-then-draw order.
    ///
    /// Appended for serialized ordinal stability.
    ChooseMoveTargetEquipmentAndOrDraw {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player whose turn just ended.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let a bound direct-Sha damage target discard up to the source's current attack range
    /// from hand/equipment or increase that damage by one. When the source has a physical
    /// weapon, both branches are forced in discard-then-increase order and the weapon is
    /// discarded afterward.
    ///
    /// Content emits `damage_id = 0`; the engine VM host binds the current pre-damage event.
    /// Appended for serialized ordinal stability.
    ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
        /// Sha source and originating skill owner.
        player: PlayerId,
        /// Bound damage target and discard/branch chooser.
        target: PlayerId,
        /// Engine-owned pre-resolution damage id.
        #[serde(default)]
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let the player cancel, draw one and gain one temporary skill, or recast one
    /// hand/equipment card and gain another temporary skill.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDrawOrRecastAddSkill {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Temporary skill granted before drawing one card.
        draw_skill_def: SkillDefId,
        /// Temporary skill granted before recasting one card.
        recast_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let the owner gain the first Sha from the draw or discard pile during this originating
    /// skill's turn-opening extra Play phase.
    ///
    /// Appended for serialized ordinal stability.
    ChooseGainShaForOpeningExtraPlay {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Finish the originating turn-opening Sha-gain policy and apply its no-damage consequence.
    ///
    /// Appended for serialized ordinal stability.
    FinishOpeningExtraPlayShaGain {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Reveal another player's hand, gain every effective Shan and physical armor, or draw two
    /// when no card matches.
    ///
    /// Appended for serialized ordinal stability.
    RevealTargetHandGainShanArmorOrDrawTwo {
        /// Skill owner and recipient.
        player: PlayerId,
        /// Other living player whose hand is revealed.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Give every current hand card from `player` to `target`.
    ///
    /// Appended for serialized ordinal stability.
    GiveAllHand {
        /// Living player giving every hand card.
        player: PlayerId,
        /// Other living recipient.
        target: PlayerId,
    },
    /// Gain `card` if it is still in the shared discard pile.
    ///
    /// Appended for serialized ordinal stability.
    GainCardIfDiscard {
        /// Living recipient.
        player: PlayerId,
        /// Physical card to gain.
        card: CardId,
    },
    /// Let the Sha user cancel, draw one card, or discard one target hand/equipment card.
    /// Accepting either branch records this skill and target on the exact resolving Sha.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDrawOrDiscardTargetCardRecordSha {
        /// Sha user and chooser.
        player: PlayerId,
        /// Exact resolving Sha.
        card: CardId,
        /// Current Sha target.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Reveal `player`'s complete current hand only to `observer`.
    ///
    /// Appended for serialized ordinal stability.
    RevealHandToPlayer {
        /// Player whose hand is revealed.
        player: PlayerId,
        /// Living player authorized to see the hand.
        observer: PlayerId,
    },
    /// Let `player` optionally choose and discard one card from `target` in a typed zone scope.
    ///
    /// Cancelling is a successful no-op. The scope and card legality are revalidated at the
    /// continuation barrier. Appended for serialized ordinal stability.
    ChooseOptionalDiscardTargetCard {
        /// Player choosing the card.
        player: PlayerId,
        /// Living player whose card may be discarded.
        target: PlayerId,
        /// Canonical zones offered to the chooser.
        scope: TargetCardScope,
    },
    /// Publicly reveal `player`'s complete current hand, then let that player choose one
    /// effective red/black color present and discard every discardable hand card of that color.
    /// A single present color resolves without opening a choice.
    ///
    /// Appended for serialized ordinal stability.
    RevealHandChooseDiscardColor {
        /// Player revealing, choosing, and discarding.
        player: PlayerId,
    },
    /// Force `player` to discard up to `count` current hand/equipment cards from `target`.
    /// Recover `amount` HP after movement triggers only when exactly `count` cards were
    /// discarded and their effective categories for `target` are identical.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDiscardTargetCardsRecoverSameCategory {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Player whose cards are discarded.
        target: PlayerId,
        /// Maximum discard count.
        count: u32,
        /// Maximum HP recovered after a matching discard.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// `player` chooses up to `count` cards from its own and `target`'s hands combined, each of a
    /// distinct effective suit, then all chosen cards are discarded (split by owner). Powers
    /// 「试探」-style active skills.
    ///
    /// Appended for serialized ordinal stability.
    ChooseScskuijiDiscard {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player whose hand also participates.
        target: PlayerId,
        /// Maximum number of cards chosen (and discarded).
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Like `ChooseScskuijiDiscard`, but when the chooser and target discard different numbers of
    /// cards, the side that discarded fewer loses 1 HP (the legacy `rechouai` grant is a dangling
    /// no-op in upstream and is intentionally not modeled). Powers 「破试探」.
    ///
    /// Appended for serialized ordinal stability.
    ChoosePskuijiDiscard {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player whose hand also participates.
        target: PlayerId,
        /// Maximum number of cards chosen (and discarded).
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Compare the selected hand card with one unused target. A strict win chooses between
    /// drawing and discarding one target hand/equipment card; the configured non-win threshold
    /// loses HP, damages the shorter seat path between the first two failures, and bans the
    /// active skill for the turn.
    ///
    /// Appended for serialized ordinal stability.
    ChooseCompareSelectedHandCardSecondFailureSeatDamage {
        /// Skill owner and first comparison participant.
        player: PlayerId,
        /// Other comparison participant.
        target: PlayerId,
        /// Owner's selected hand card.
        card: CardId,
        /// Cards drawn after choosing the strict-win draw branch.
        draw_count: u32,
        /// Number of non-wins that activates the failure branch.
        failure_count: u32,
        /// HP lost when the failure threshold is reached.
        lose_hp: u32,
        /// Damage dealt to each chosen seat-path player.
        damage: u32,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Resume the comparison after `hand_compare_started` triggers finish.
    ///
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    ContinueChooseCompareTargetHandCardSecondFailureSeatDamage {
        /// Skill owner and first comparison participant.
        player: PlayerId,
        /// Other comparison participant.
        target: PlayerId,
        /// Owner's selected hand card.
        card: CardId,
        /// Cards drawn on the strict-win draw branch.
        draw_count: u32,
        /// Non-win threshold.
        failure_count: u32,
        /// HP loss at the threshold.
        lose_hp: u32,
        /// Damage per seat-path player.
        damage: u32,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Choose between drawing cards and discarding one target hand/equipment card.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDrawOrDiscardTargetCard {
        /// Living chooser.
        player: PlayerId,
        /// Other player whose card may be discarded.
        target: PlayerId,
        /// Draw-branch count.
        draw_count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Record one comparison non-win and, at the threshold, schedule HP loss followed by
    /// seat-path damage.
    ///
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    RecordCompareFailureSeatDamage {
        /// Skill owner.
        player: PlayerId,
        /// Current failed comparison target.
        target: PlayerId,
        /// Non-win threshold.
        failure_count: u32,
        /// HP loss at the threshold.
        lose_hp: u32,
        /// Damage per seat-path player.
        damage: u32,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Choose an equal-length seat path, or automatically use the shorter path, then damage
    /// its currently living players in order.
    ///
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    ChooseSeatPathDamage {
        /// Damage source and chooser.
        player: PlayerId,
        /// First failed comparison target.
        first: PlayerId,
        /// Second failed comparison target.
        second: PlayerId,
        /// Damage per player.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Damage the remaining snapshotted players one by one, resuming only after each dying/save
    /// interruption has completed.
    ///
    /// Rule VM bytecode never emits this directly. Appended for serialized ordinal stability.
    DealDamageEachInOrder {
        /// Damage source.
        source: PlayerId,
        /// Remaining targets in exact resolution order.
        targets: Vec<PlayerId>,
        /// Damage per target.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Remove every skill currently owned by one player.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    RemoveAllSkills {
        /// Player losing all current skills.
        player: PlayerId,
    },
    /// Force one player to choose another living player, then recover the selected target.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseOtherRecover {
        /// Player making the forced target choice.
        player: PlayerId,
        /// Maximum HP recovered by the selected target.
        amount: u32,
    },
    /// Restore one disabled copy of an equipment slot.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    EnableEquipSlot {
        /// Player whose equipment slot is restored.
        player: PlayerId,
        /// Stable slot tag (`1` weapon through `5` treasure).
        slot: u8,
        /// Player causing the restoration.
        source: PlayerId,
    },
    /// Let `target` choose a suit and one current hand card from `player`; move that card to
    /// `target`, then deal 1 damage from `player` if its owner-effective suit differs.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    TargetChooseSuitGainOwnerHandDamageIfMismatch {
        /// Skill owner whose hand card is gained.
        player: PlayerId,
        /// Other living player making both choices.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Perform Ganglie's judgment, then let a living damage source discard exactly two hand
    /// cards or take 1 normal damage from the judging player.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeSourceDiscardTwoHandOrDamage {
        /// Damaged player performing the judgment.
        player: PlayerId,
        /// Damage source paying the discard or taking damage.
        source: PlayerId,
    },
    /// Give selected hand cards and recover the giver when their current Play-phase gift total
    /// first reaches two.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    GiveSelectedRecoverAfterPhaseTotalTwo {
        /// Skill owner giving selected cards.
        player: PlayerId,
        /// Other living recipient.
        target: PlayerId,
        /// Selected hand cards in input order.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether movement and nested triggers have completed.
        ready_to_recover: bool,
    },
    /// Choose one or two other players with hand cards, gain one hand card from each in order,
    /// and replace the current Draw-phase draw.
    ///
    /// An empty `targets` list opens the target choice. A concrete `selected` card gains that
    /// card from `targets[index]`; `NONE` scans for the next card choice. Appended so existing
    /// serialized effect variant ordinals remain stable.
    ChooseOtherHandTargetsGainOneReplaceDraw {
        /// Current Draw-phase player and chooser.
        player: PlayerId,
        /// Snapshotted selected targets in choice order.
        targets: Vec<PlayerId>,
        /// Current target index.
        index: u32,
        /// Selected target hand card, or NONE while opening/scanning choices.
        selected: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Move the draw-pile top two cards into public processing and distribute all of them.
    ///
    /// `ready_to_choose = false` performs the top-card movement. The engine-owned stages use
    /// `cards` for the remaining processing cards, `selected` for the current subset, and
    /// `target` for its recipient. Appended so existing serialized effect variant ordinals
    /// remain stable.
    DistributeTopTwoCards {
        /// Skill owner choosing every subset and recipient.
        player: PlayerId,
        /// Remaining cards to distribute.
        cards: Vec<CardId>,
        /// Current selected subset.
        selected: Vec<CardId>,
        /// Current subset recipient, or NONE before target choice.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the top cards have reached processing.
        ready_to_choose: bool,
    },
    /// Perform a judgment and, on a red result, make one target unable to respond to the
    /// currently resolving Sha.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeMakeCurrentShaTargetUnrespondableOnRed {
        /// Player performing the judgment and using the Sha.
        player: PlayerId,
        /// Physical Sha whose target is being resolved.
        card: CardId,
        /// Current Sha target whose response window is disabled on success.
        target: PlayerId,
        /// Resolve the full 界铁骑 judgment, suppression, and same-suit gift branch.
        #[serde(default)]
        retieji: bool,
        /// Originating skill instance for the 界铁骑 branch.
        #[serde(default = "no_skill")]
        skill: SkillId,
        /// Whether judgment triggers have finished and the target choice may open.
        #[serde(default)]
        ready_to_choose: bool,
        /// Captured effective judgment suit tag for the 界铁骑 branch.
        #[serde(default)]
        judgment_suit: u8,
    },
    /// Let the skill owner optionally discard one basic hand card, then let the target discard
    /// one equipment card from hand/equipment or take 1 normal damage from the owner.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseDiscardBasicThenTargetDiscardEquipOrDamage {
        /// Skill owner paying the cost and dealing damage.
        player: PlayerId,
        /// End-phase player choosing the consequence.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let the current Sha target optionally discard one hand/equipment card, then redirect that
    /// target slot to the skill owner after the discard's nested triggers finish.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseCurrentShaTargetDiscardRedirectToOwner {
        /// Skill owner receiving the redirected Sha.
        owner: PlayerId,
        /// Current Sha user.
        source: PlayerId,
        /// Physical Sha currently resolving.
        card: CardId,
        /// Current target paying the discard cost.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// False opens the cost choice; true commits the redirect after the discard barrier.
        ready_to_redirect: bool,
    },
    /// Let the current Sha target discard one hand/equipment card and redirect that target slot
    /// to another legal player in the skill owner's attack range.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseDiscardRedirectCurrentSha {
        /// Current Sha target and skill owner paying the discard cost.
        player: PlayerId,
        /// Current Sha user.
        source: PlayerId,
        /// Physical Sha currently resolving.
        card: CardId,
        /// Chosen replacement target, or `NONE` before target selection.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Reveal the draw-pile bottom card, then force the skill owner to give it to a living
    /// player.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseTargetGainBottomCard {
        /// Skill owner and attributed giver.
        player: PlayerId,
        /// Revealed card in processing, or `NONE` before it is staged.
        card: CardId,
        /// Chosen recipient, or `NONE` before target selection.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Judge repeatedly while results are black, gaining each black result before optionally
    /// continuing.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeBlackGainRepeat {
        /// Judged player and black-card recipient.
        player: PlayerId,
        /// Black judgment card awaiting gain, or `NONE` before a new judgment.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the gained-card trigger chain has completed and the repeat choice may open.
        ready_to_choose: bool,
    },
    /// Make the active skill's second selected target use a suitless virtual Duel on the first
    /// without opening a Wuxie window.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    UseVirtualJuedouSecondOnFirst {
        /// Active-skill targets in submitted order.
        targets: Vec<PlayerId>,
        /// Materialized virtual card after its CardUsed trigger chain, or `NONE` initially.
        card: CardId,
    },
    /// Perform a judgment and replace the originating skill's same-id mark with its rank.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeSetSkillMarkToRank {
        /// Player performing the judgment and receiving the mark.
        player: PlayerId,
        /// Originating skill instance whose definition id is used as the mark id.
        skill: SkillId,
    },
    /// In seat order, choose and discard one hand/equipment card from every linked player.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseDiscardOneFromEachLinked {
        /// Skill owner choosing each discard.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Disable every instance of one skill definition for a player until the next turn begins.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    DisableSkillByDefUntilTurnEnd {
        /// Skill owner.
        player: PlayerId,
        /// Skill definition to disable temporarily.
        skill_def: SkillDefId,
    },
    /// Force `target` to discard one hand/equipment card, then let `player` use a virtual Duel
    /// on that target when the discarded card is not Sha and the target's HP is not lower.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    TargetDiscardThenDuelIfNonShaAndHpGteOwner {
        /// Skill owner and virtual Duel user.
        player: PlayerId,
        /// Other player choosing and discarding a card.
        target: PlayerId,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Choose a player with a discardable hand card, lose 1 HP, then make that target discard.
    /// A basic card produces a distance-free virtual Sha after movement triggers finish.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseTargetLoseHpDiscardHandShaIfBasic {
        /// Skill owner and virtual Sha user.
        player: PlayerId,
        /// Selected target, or NONE before target selection.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// 0=choose target, 1=choose discard after HP loss, 2=post-discard check.
        stage: u8,
        /// Effective category snapshot for the discarded card.
        discarded_basic: bool,
    },
    /// Optionally choose a bounded number of living targets, then each draws in seat order.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ChooseOptionalTargetsDraw {
        /// Player making the target choice.
        player: PlayerId,
        /// Cards drawn by each selected target.
        count: u32,
        /// Minimum number of distinct targets.
        min_count: u8,
        /// Maximum number of distinct targets.
        max_count: u8,
        /// Canonical target scope.
        scope: DrawTargetScope,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve the legacy Weikui HP cost and hand branch, then bind a temporary distance helper.
    ///
    /// Stage zero loses HP. Later stages run only after its complete dying window, then branch
    /// on effective Shan, attempt the virtual Sha, and conditionally grant the helper.
    /// Appended for serialized ordinal stability.
    RevealTargetHandShaOrDiscardBind {
        /// Skill owner, HP-loss payer, observer, and virtual Sha user.
        player: PlayerId,
        /// Selected other player.
        target: PlayerId,
        /// Temporary target-bound helper skill definition.
        helper_skill_def: SkillDefId,
        /// Originating active skill instance.
        skill: SkillId,
        /// Engine-owned stage: 0=HP loss, 1=branch, 2=virtual Sha, 3=helper grant.
        stage: u8,
    },
    /// Draw cards and remember their exact physical ids in the originating skill instance.
    ///
    /// Remembered ids are removed as soon as those cards leave the owner's hand.
    /// Appended for serialized ordinal stability.
    DrawRememberedHandCards {
        /// Skill owner and drawer.
        player: PlayerId,
        /// Number of cards to draw.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Discard every remembered card that remains discardable in the owner's hand, then clear
    /// the remembered set.
    ///
    /// Appended for serialized ordinal stability.
    DiscardRememberedHandCards {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Gain one topmost draw-pile card from each selector group whose paired divisor matches the
    /// player's match-lifetime card-action count.
    ///
    /// Appended for serialized ordinal stability.
    GainDrawCardsForLifetimeCardActionDivisors {
        /// Player whose history is inspected and who gains the cards.
        player: PlayerId,
        /// Three positive divisors checked independently.
        divisors: [u32; 3],
        /// Three stable reward-group tags paired with the divisors.
        groups: [u8; 3],
    },
    /// During the originating turn-opening extra Play phase, lose 1 HP and then gain the
    /// topmost matching discard-pile card. The action is forced while `mandatory_skill_def`
    /// remains owned and optional after that definition has been removed.
    ///
    /// Appended for serialized ordinal stability.
    LoseHpGainDiscardCardForOpeningExtraPlay {
        /// Skill owner, HP-loss target, and card recipient.
        player: PlayerId,
        /// Stable rule card-kind tag.
        kind: u8,
        /// Owning this definition makes the action mandatory.
        mandatory_skill_def: SkillDefId,
        /// Originating extra-phase skill instance.
        skill: SkillId,
    },
    /// Choose and gain one physical card the player lost from hand/equipment this turn that
    /// still remains in the discard pile.
    ///
    /// Appended for serialized ordinal stability.
    ChooseGainOwnLostDiscardCardThisTurn {
        /// Player choosing and gaining the card.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose one equipment or delayed-trick card from another living player's field and move
    /// it into `target`'s matching empty equipment slot or judgment area.
    ///
    /// Appended for serialized ordinal stability.
    ChooseMoveFieldCardTo {
        /// Skill owner and card chooser.
        player: PlayerId,
        /// Fixed living destination.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Draw cards, then optionally plan and give up to a bounded number of current hand cards to
    /// one or more other living players.
    ///
    /// Appended for serialized ordinal stability.
    DrawThenOptionallyDistributeHandCards {
        /// Skill owner, drawer, chooser, and giver.
        player: PlayerId,
        /// Cards drawn before distribution choices begin.
        draw_count: u32,
        /// Maximum total number of hand cards that may be distributed.
        give_cap: u32,
        /// Planned distinct hand cards, paired positionally with `targets`.
        cards: Vec<CardId>,
        /// Planned recipients, paired positionally with `cards`.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
        /// Internal flag set after draw events and nested triggers finish.
        ready_to_choose: bool,
        /// Internal flag requesting atomic validation and transfer of the complete plan.
        finish: bool,
    },
    /// Give the active invocation's selected hand cards, then optionally repeat for previously
    /// unused other recipients; recover once when the cumulative gift count first reaches two.
    ///
    /// Appended for serialized ordinal stability.
    GiveSelectedThenContinueDistinctRecoverAtTwo {
        /// Skill owner, chooser, giver, and recovery target.
        player: PlayerId,
        /// Current distinct gift recipient.
        target: PlayerId,
        /// Current hand cards to give.
        cards: Vec<CardId>,
        /// Recipients already used by this activation.
        used_targets: Vec<PlayerId>,
        /// Cards given by this activation before the current batch.
        total_given: u32,
        /// Originating active skill instance.
        skill: SkillId,
        /// Internal flag that opens the next optional card choice without giving a batch.
        continue_only: bool,
    },
    /// Let a dead skill owner choose one living other player tied for the highest positive mark,
    /// judge that target, and kill them unless the physical result is Tao or Taoyuan.
    ///
    /// Appended for serialized ordinal stability.
    ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
        /// Dead skill owner and chooser.
        player: PlayerId,
        /// Mark definition used to rank living other players.
        mark: MarkDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose a zone priority, gain one deterministic-random card from the first nonempty
    /// preferred zone of each other living player in seat order, then turn over.
    ///
    /// Appended for serialized ordinal stability.
    ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
        /// Skill owner, chooser, and card recipient.
        player: PlayerId,
        /// 0 = hand/equip/judge, 1 = equip/hand/judge, 2 = judge/hand/equip, 255 = choose.
        priority: u8,
        /// Engine-owned remaining targets in seat order.
        remaining_targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose and gain one card from every other living player's hand/equipment/judgment zones
    /// in seat order, then turn over.
    ///
    /// Appended for serialized ordinal stability.
    ChooseGainCardFromEachOtherThenTurnOver {
        /// Skill owner, chooser, and card recipient.
        player: PlayerId,
        /// Whether the stable target snapshot has been initialized.
        initialized: bool,
        /// Engine-owned remaining targets in seat order.
        remaining_targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Grant a skill only until the current turn ends.
    ///
    /// An already-owned permanent or temporary instance is left unchanged. Appended for
    /// serialized ordinal stability.
    AddSkillUntilTurnEnd {
        /// Skill recipient.
        player: PlayerId,
        /// Skill definition to grant.
        skill_def: SkillDefId,
    },
    /// Consume marks, damage every other living player in seat order, then make those players
    /// discard all equipment and a bounded number of hand cards before turning the owner over.
    ///
    /// `stage` and `index` are engine-owned state-machine fields. Appended for serialized
    /// ordinal stability.
    ResolveMassDamageDiscardTurnOver {
        /// Skill owner and damage source.
        player: PlayerId,
        /// Mark definition consumed at stage zero.
        mark: MarkDefId,
        /// Number of marks consumed.
        mark_count: u32,
        /// Normal damage dealt to each target.
        damage_amount: u32,
        /// Maximum hand cards each target must discard.
        hand_count: u32,
        /// 0 = initialize, 1 = damage, 2 = equipment, 3 = hand, 4 = turn over.
        stage: u8,
        /// Next target index within the stable target snapshot.
        index: u32,
        /// Stable other-player snapshot in seat order.
        targets: Vec<PlayerId>,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Consume marks, damage every other living player in seat order, then make those players
    /// discard a bounded number of cards from their combined hand/equipment zones.
    ///
    /// `stage` and `index` are engine-owned state-machine fields. Appended for serialized
    /// ordinal stability.
    ResolveMassDamageDiscardHandEquip {
        /// Skill owner and damage source.
        player: PlayerId,
        /// Mark definition consumed at stage zero.
        mark: MarkDefId,
        /// Number of marks consumed.
        mark_count: u32,
        /// Normal damage dealt to each target.
        damage_amount: u32,
        /// Maximum hand/equipment cards each target must discard.
        discard_count: u32,
        /// 0 = initialize, 1 = damage, 2 = discard.
        stage: u8,
        /// Next target index within the stable target snapshot.
        index: u32,
        /// Stable other-player snapshot in seat order.
        targets: Vec<PlayerId>,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Record one selected target for an active skill declaring
    /// `unused_by_skill_this_turn`.
    ///
    /// Appended for serialized ordinal stability.
    RecordActiveTarget {
        /// Active-skill owner.
        player: PlayerId,
        /// Selected target to reserve for the current turn.
        target: PlayerId,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Resolve a mark-parity target action, then optionally clear a large mark stack to damage
    /// every other living player in seat order.
    ///
    /// `stage`, `index`, `target`, and `targets` are engine-owned state-machine fields.
    /// Appended for serialized ordinal stability.
    ResolveMarkParityControl {
        /// Skill owner, chooser, and damage source.
        player: PlayerId,
        /// Mark definition controlling parity and the optional clear.
        mark: MarkDefId,
        /// Clear-all option is offered only above this count.
        mass_threshold: u32,
        /// Normal damage dealt by either damage branch.
        damage_amount: u32,
        /// 0 = choose target, 1 = parity action, 2 = discard card, 3 = clear option,
        /// 4 = mass damage.
        stage: u8,
        /// Next target index for mass damage.
        index: u32,
        /// Chosen parity-action target.
        target: PlayerId,
        /// Stable mass-damage target snapshot.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Discard all equipment from selected active-skill targets in seat order, then let the
    /// owner choose one surviving selected target to damage.
    ///
    /// `stage`, `index`, and `targets` are engine-owned state-machine fields. Appended for
    /// serialized ordinal stability.
    ResolveSelectedEquipmentDiscardThenChooseDamage {
        /// Skill owner, chooser, and damage source.
        player: PlayerId,
        /// Damage dealt to the final chosen target.
        damage_amount: u32,
        /// Damage nature tag.
        nature: u8,
        /// 0 = initialize, 1 = equipment discard, 2 = choose damage target.
        stage: u8,
        /// Next selected-target index.
        index: u32,
        /// Stable active-skill target snapshot.
        targets: Vec<PlayerId>,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Replace the pending normal draw with a public top-card reveal and an exact one-per-suit
    /// selection.
    ///
    /// `stage` and `cards` are engine-owned state-machine fields. Appended for serialized
    /// ordinal stability.
    ReplaceDrawWithDistinctSuitSelection {
        /// Draw-phase player and card recipient.
        player: PlayerId,
        /// Number of top cards revealed.
        count: u8,
        /// 0 = initialize, 1 = choose from the stable processing-zone cards.
        stage: u8,
        /// Stable revealed-card snapshot.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Inspect another player's hand, choose one heart card, then discard it or put it on top of
    /// the draw pile.
    ///
    /// `stage` and `card` are engine-owned state-machine fields. Appended for serialized ordinal
    /// stability.
    InspectTargetHandChooseHeartDiscardOrTop {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Inspected hand owner.
        target: PlayerId,
        /// 0 = reveal/choose heart, 1 = choose destination.
        stage: u8,
        /// Chosen heart card.
        card: CardId,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Resolve the limited Yeyan choice between one-point fire damage and the four-suit
    /// three-point fire allocation.
    ///
    /// `mode`, `stage`, `index`, `targets`, and `data` are engine-owned state-machine fields.
    /// Appended for serialized ordinal stability.
    ResolveYeyan {
        /// Skill owner, chooser, and damage source.
        player: PlayerId,
        /// 0 = small fire, 1 = four-suit big fire, 255 = choose mode.
        mode: u8,
        /// 0 = initialize, 1 = choose cost/targets, 2 = choose big-fire targets,
        /// 3 = choose allocation, 4 = deal damage in seat order.
        stage: u8,
        /// Next target index for ordered damage.
        index: u32,
        /// Stable selected-target snapshot in seat order.
        targets: Vec<PlayerId>,
        /// Stage-specific scalar: single-target damage or boosted target raw id.
        data: u32,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Let Qinyin's owner cancel or make every currently living player lose or recover one HP.
    ///
    /// Appended for serialized ordinal stability.
    ChooseAllLoseHpOrRecover {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating trigger skill instance.
        skill: SkillId,
    },
    /// Initialize or exchange the current skill's public Qixing pile.
    ///
    /// `stage`, `hand_cards`, and `pile_cards` are engine-owned state-machine fields.
    /// Appended for serialized ordinal stability.
    ResolveQixing {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Whether stage zero first initializes seven draw-pile cards.
        initialize: bool,
        /// 0 = initialize/open hand choice, 1 = open hand choice, 2 = commit exchange.
        stage: u8,
        /// Current hand cards selected for exchange.
        hand_cards: Vec<CardId>,
        /// Current Qixing pile cards selected for exchange.
        pile_cards: Vec<CardId>,
        /// Originating Qixing skill instance.
        skill: SkillId,
    },
    /// Spend Qixing pile cards and grant a source-bound helper to equal-count living targets.
    ///
    /// `stage`, `cards`, and `targets` are engine-owned state-machine fields. Appended for
    /// serialized ordinal stability.
    ResolveQixingWeather {
        /// Skill owner, chooser, and weather source.
        player: PlayerId,
        /// Qixing definition owning the public pile.
        qixing_skill_def: SkillDefId,
        /// Target-owned weather helper definition.
        helper_skill_def: SkillDefId,
        /// Maximum number of pile cards and targets.
        max_targets: u8,
        /// 0 = choose cards, 1 = apply helpers after discard.
        stage: u8,
        /// Selected Qixing pile cards.
        cards: Vec<CardId>,
        /// Selected living targets.
        targets: Vec<PlayerId>,
        /// Originating weather skill instance.
        skill: SkillId,
    },
    /// Toggle Longnu, pay its branch cost, draw, then grant the matching phase helper.
    ///
    /// Appended for serialized ordinal stability.
    ResolveNzryLongnu {
        /// Skill owner.
        player: PlayerId,
        /// Red-card-to-fire-Sha helper definition.
        fire_skill_def: SkillDefId,
        /// Trick-card-to-thunder-Sha helper definition.
        thunder_skill_def: SkillDefId,
        /// 0 = toggle/pay cost, 1 = draw/grant helper.
        stage: u8,
        /// Originating Longnu skill instance.
        skill: SkillId,
    },
    /// Force the skill owner to choose and link another currently unlinked living player.
    ///
    /// Appended for serialized ordinal stability.
    ChooseLinkOther {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose four distinct-suit cards across the owner and target hands, discard them, then
    /// resolve Poxi according to how many came from the owner.
    ///
    /// Appended for serialized ordinal stability.
    ResolveDrltPoxi {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Inspected active-skill target.
        target: PlayerId,
        /// Temporary hand-limit helper definition.
        hand_limit_skill_def: SkillDefId,
        /// 0 = choose cards, 1 = resolve the post-discard branch.
        stage: u8,
        /// Number of selected cards that belonged to the owner.
        own_count: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve one of Jieying's turn-begin, mark-transfer, or turn-end settlement branches.
    ///
    /// Appended for serialized ordinal stability.
    ResolveDrltJieying {
        /// Jieying owner.
        player: PlayerId,
        /// Player carried by the triggering event.
        event_player: PlayerId,
        /// Stable 营 mark definition.
        mark: MarkDefId,
        /// 0 = ensure mark, 1 = choose transfer, 2 = settle another marked player's turn.
        mode: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve Duorui's equipment-slot cost, skill borrowing, or expiry cleanup.
    ///
    /// Appended for serialized ordinal stability.
    ResolveDrltDuorui {
        /// Duorui owner for acquisition; helper owner for cleanup.
        player: PlayerId,
        /// Damaged target for acquisition; event player for cleanup.
        target: PlayerId,
        /// Target-side cleanup helper definition.
        helper_skill_def: SkillDefId,
        /// 0 = start, 1 = choose skill, 2 = acquire selected skill, 3 = cleanup.
        stage: u8,
        /// Selected target skill definition for stage 2.
        selected_skill_def: SkillDefId,
        /// Originating Duorui or helper skill instance.
        skill: SkillId,
    },
    /// Restore one disabled equipment slot when Zhiti's event condition is satisfied.
    ///
    /// Appended for serialized ordinal stability.
    ResolveDrltZhiti {
        /// Zhiti owner.
        player: PlayerId,
        /// Event player.
        event_player: PlayerId,
        /// Event source.
        event_source: PlayerId,
        /// Event scalar used to preserve a hand-comparison winner.
        event_amount: u32,
        /// 0 = hand comparison, 1 = Duel, 2 = received damage.
        mode: u8,
        /// Originating Zhiti skill instance.
        skill: SkillId,
    },
    /// Resolve Shen Simayi's shared mark, extra-turn, active, and draw branches.
    ///
    /// Appended for serialized ordinal stability.
    ResolveShenSimayi {
        /// Skill owner.
        player: PlayerId,
        /// Player carried by the triggering event.
        event_player: PlayerId,
        /// Scalar carried by the triggering event.
        event_amount: u32,
        /// Stable 忍 mark definition.
        mark: MarkDefId,
        /// 0=discard Renjie, 1=Lianpo, 2=Wansha, 3=Zhiheng, 4=Jizhi.
        mode: u8,
        /// Selected active cards or engine-owned continuation cards.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Selected legal targets for the chosen card at the final stage.
        #[serde(default)]
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve Cai Wenji's Beige cost, judgment, rewrite window, and final suit branch.
    ///
    /// Appended for serialized ordinal stability.
    ResolveBeige {
        /// Beige owner.
        player: PlayerId,
        /// Player damaged by Sha and judged by Beige.
        target: PlayerId,
        /// Damage source.
        source: PlayerId,
        /// 0=cost, 1=flip, 2=start rewrites, 3=apply suit branch, 4=continue rewrites.
        stage: u8,
        /// Current physical judgment result.
        #[serde(default = "no_card")]
        result_card: CardId,
        /// Current effective judgment suit.
        #[serde(default = "no_suit")]
        suit: Suit,
        /// Current physical judgment rank.
        #[serde(default)]
        rank: u8,
        /// Remaining judgment rewriters in seat order.
        #[serde(default)]
        rewriters: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose and pay either one discardable physical weapon from hand/equipment or one HP.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDiscardWeaponOrLoseHp {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Deal one ordinary self-damage, then discard the player's current judgment zone after the
    /// complete damage, dying, and save chain.
    ///
    /// Appended for serialized ordinal stability.
    DamageThenDiscardAllJudge {
        /// Player damaged and whose judgment zone is eventually emptied.
        player: PlayerId,
        /// Engine-owned continuation stage. Content emits false.
        #[serde(default)]
        ready_to_discard: bool,
    },
    /// Replace normal draw with two public top cards, gain them, then grant two temporary skills
    /// when their effective colors differ.
    ///
    /// Appended for serialized ordinal stability.
    ReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
        /// Draw-phase player and card recipient.
        player: PlayerId,
        /// First temporary skill definition.
        first_skill_def: SkillDefId,
        /// Second temporary skill definition.
        second_skill_def: SkillDefId,
        /// 0 = reveal, 1 = gain, 2 = grant skills.
        #[serde(default)]
        stage: u8,
        /// Stable revealed-card snapshot.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose and gain one target hand/equipment card from a Play-phase zone not yet used by
    /// this skill, then let that target draw one card.
    ///
    /// `selected = NONE` opens the card choice; a concrete card resumes it. Appended for
    /// serialized ordinal stability.
    ChooseGainTargetCardAlternateZoneThenTargetDraw {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other living player losing one card, then drawing.
        target: PlayerId,
        /// Chosen current target card, or NONE before the choice.
        selected: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Mark a living player's next judgment phase to be skipped.
    ///
    /// Appended for serialized ordinal stability.
    SkipNextJudgePhase {
        /// Player whose next judgment phase is skipped.
        player: PlayerId,
    },
    /// Choose and discard one hand/equipment card from either endpoint of the current damage,
    /// then increase that damage if the card's owner-effective suit matches.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDiscardDamageEndpointCardIncreaseIfSuit {
        /// Damage source, skill owner, and chooser.
        player: PlayerId,
        /// Current damage target.
        target: PlayerId,
        /// Bound pre-resolution damage id. Content emits zero; the VM host binds it.
        damage_id: u32,
        /// Stable suit tag (`1` spade through `4` diamond).
        suit: u8,
        /// Damage added after a matching discard.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose and gain one hand card from a player supplied by the triggering event.
    ///
    /// `from = NONE` is bound by the VM host to the first event target.
    /// Appended for serialized ordinal stability.
    TakeOneHandCard {
        /// Card owner whose hand card is taken.
        from: PlayerId,
        /// Recipient of the taken card.
        to: PlayerId,
    },
    /// 选目标一张手牌置入拥有者技能牌堆（ABI 567,储元 chuyuan 等）。
    ///
    /// Appended to keep existing serialized variant ordinals stable.
    ChooseOtherHandStoreSkillPile {
        /// Skill owner whose skill pile receives the card.
        player: PlayerId,
        /// Player whose hand card is chosen.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 获得本技能牌堆全部牌（移入手牌,ABI 569,登极 dengji 觉醒取回等）。
    ///
    /// Appended to keep existing serialized variant ordinals stable.
    GainAllSkillPileCards {
        /// Skill owner whose skill pile is emptied into hand.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 获得其他角色指定技能牌堆的全部牌（移入自己手牌）。ABI 600,英逝 yingshi_die。
    ///
    /// Appended to keep existing serialized variant ordinals stable.
    GainOtherSkillPileCards {
        /// Player whose hand receives the pile cards.
        player: PlayerId,
        /// Player whose skill pile (keyed by `skill_def`) is emptied.
        target: PlayerId,
        /// Skill definition id keying the source skill pile.
        skill_def: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 弃置一张手牌/装备牌或失去等量体力。ABI 601,庸肆 xinyongsi2。
    ///
    /// Appended to keep existing serialized variant ordinals stable.
    ChooseDiscardOneHeOrLoseHp {
        /// Player making the discard-or-lose choice.
        player: PlayerId,
        /// Amount of HP lost if the player declines to discard.
        lose_amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 目标判定：黑桃 2-9 则其摸两张牌，否则对其使用两张雷【杀】。ABI 602,雷劫 releijie。
    ///
    /// Appended to keep existing serialized variant ordinals stable.
    JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice {
        /// Skill owner who uses the virtual thunder Sha.
        source: PlayerId,
        /// Target who is judged, draws, or receives the thunder Sha.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 死亡来源选择是否令拥有者失去体力。ABI 603,随势 suishi2。
    ///
    /// Appended to keep existing serialized variant ordinals stable.
    ChooseSourceConfirmOwnerLoseHp {
        /// Owner who loses HP when the source confirms.
        owner: PlayerId,
        /// Player (e.g. the death source) who confirms the HP loss.
        source: PlayerId,
        /// Amount of HP the owner loses on confirmation.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// At the current turn's completed End phase, choose an extra Draw or Play phase, or
    /// execute both when the distinct suits of cards used this turn reach current HP.
    ///
    /// Appended for serialized ordinal stability.
    ChooseEndExtraDrawOrPlayByUsedSuits {
        /// Current turn player and skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Lose HP equal to the bound damage, double that damage, then permanently replace the
    /// originating skill after the HP-loss resolution finishes.
    ///
    /// Appended for serialized ordinal stability.
    LoseHpDoubleCurrentDamageThenReplace {
        /// Damage source, HP-loss target, and skill owner.
        player: PlayerId,
        /// Bound pre-resolution damage id.
        damage_id: u32,
        /// Original damage amount used by the HP-loss cost and initial damage increase.
        amount: u32,
        /// Originating skill instance to replace.
        skill: SkillId,
        /// Permanent replacement skill definition.
        replacement_skill_def: SkillDefId,
        /// Internal second stage, executed after the HP-loss resolution.
        ready_to_replace: bool,
    },
    /// Resolve a hidden skill-pile action shared by reveal, exchange, and matching-suit skills.
    ///
    /// Appended for serialized ordinal stability.
    ResolveHiddenSkillPile {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Definition that owns the expansion pile.
        pile_skill_def: SkillDefId,
        /// Stable action tag.
        action: u8,
        /// Current triggering card for target exclusion.
        card: CardId,
        /// Engine-owned continuation stage.
        #[serde(default)]
        stage: u8,
        /// Selected hand cards for an exchange.
        #[serde(default)]
        hand_cards: Vec<CardId>,
        /// Selected pile card(s).
        #[serde(default)]
        pile_cards: Vec<CardId>,
        /// Physical cards captured from the triggering movement event.
        #[serde(default)]
        event_cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Enable or clear the originating skill as the current turn's non-locked-skill suppression
    /// source.
    ///
    /// Appended for serialized ordinal stability.
    SetNonLockedSkillSuppression {
        /// Skill owner and current turn player.
        player: PlayerId,
        /// Whether to enable or clear suppression.
        enabled: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve EXTRA 帝力/驭衡 and its derived fate-line actions.
    ///
    /// Appended for serialized ordinal stability.
    ResolveDiliYuheng {
        /// Skill owner.
        player: PlayerId,
        /// Stable action tag.
        action: u8,
        /// Bound card for card-use or equipment actions.
        card: CardId,
        /// Physical cards captured from the triggering movement event.
        #[serde(default)]
        event_cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Finish 调剂盐梅 after its forced discard and any resulting triggers.
    ///
    /// Appended for serialized ordinal stability.
    FinalizeTiaojiyanmei {
        /// Card user.
        actor: PlayerId,
        /// First original target.
        left: PlayerId,
        /// Second original target.
        right: PlayerId,
        /// Card discarded by the target above the original average.
        card: CardId,
    },
    /// Open 调剂盐梅's forced discard after the lower-hand target's draw and triggers finish.
    ///
    /// Appended for serialized ordinal stability.
    BeginTiaojiyanmeiDiscard {
        /// Card user.
        actor: PlayerId,
        /// First original target.
        left: PlayerId,
        /// Second original target.
        right: PlayerId,
        /// Target whose original hand count was above the average.
        higher: PlayerId,
    },
    /// Discard selected hand/equipment cards and draw the same count, plus one when all current
    /// hand cards were selected.
    ///
    /// Appended for serialized ordinal stability.
    DiscardSelectedDrawAllHandBonus {
        /// Skill owner and draw recipient.
        player: PlayerId,
        /// Cards selected by the active invocation.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 国战英魂: choose another player, choose one lost-HP-scaled draw/discard branch,
    /// then perform its forced discard after draw triggers finish.
    ///
    /// Appended for serialized ordinal stability.
    ChooseOtherYinghun {
        /// Wounded skill owner and chooser.
        player: PlayerId,
        /// Selected other player, or `NONE` before target choice.
        target: PlayerId,
        /// Lost HP captured when target choice opens.
        count: u32,
        /// Branch tag: zero before choice, one for draw-X/discard-one, two for
        /// draw-one/discard-X.
        branch: u8,
        /// Whether the selected target's draw and nested triggers have completed.
        ready_to_discard: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 弼政: choose another player to draw two, then sequentially make each participant
    /// above their own maximum HP discard up to two hand/equipment cards.
    ///
    /// Appended for serialized ordinal stability.
    ChooseOtherBizheng {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Selected other player, or `NONE` before target choice.
        target: PlayerId,
        /// Engine-owned stage: zero chooses, one draws, two checks owner, three checks target.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 霞姝: give all hand cards to another player, let that player reveal a nonempty
    /// subset, then let the owner gain the revealed or unrevealed partition.
    ///
    /// Appended for serialized ordinal stability.
    ChooseOtherXiashu {
        /// Skill owner and target chooser.
        player: PlayerId,
        /// Selected other player, or `NONE` before target choice.
        target: PlayerId,
        /// Engine-owned stage: zero chooses target, one gives, two chooses reveal, three chooses
        /// partition.
        stage: u8,
        /// Revealed hand cards captured for the partition choice.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve one 观潮 card rank against the previous Play-phase card, drawing on a valid
    /// selected direction or invalidating that direction after a violation.
    ///
    /// Appended for serialized ordinal stability.
    ResolveGuanchaoRank {
        /// Skill owner and card user.
        player: PlayerId,
        /// Current used physical or materialized card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 观微: discard one hand/equipment card, let the completed Play-phase player draw
    /// two, then insert one immediate extra Play phase for that player.
    ///
    /// Appended for serialized ordinal stability.
    ResolveGuanwei {
        /// Skill owner and discard chooser.
        player: PlayerId,
        /// Player whose Play phase just ended.
        target: PlayerId,
        /// Engine-owned stage: zero chooses discard, one discards, two draws, three inserts.
        stage: u8,
        /// Selected discard card after the choice.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 揽江's snapshotted seat-order draw offers, optional equal-hand damage, and
    /// post-damage forced draw.
    ///
    /// Appended for serialized ordinal stability.
    ResolveLanjiang {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Current seat-order snapshot index.
        index: u32,
        /// Selected target for the current stage, or `NONE`.
        target: PlayerId,
        /// Players whose hand count met the entry threshold, in seat order.
        #[serde(default)]
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Reveal 秉壹 owner's full hand, choose up to hand-count living players to draw when all
    /// cards share a color, and additionally draw for the owner when all ranks also match.
    ///
    /// Appended for serialized ordinal stability.
    ResolveRebingyi {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Record one 博图 use for the current round and queue an extra turn.
    ///
    /// Appended for serialized ordinal stability.
    ResolveRebotu {
        /// Skill owner and extra-turn recipient.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 旧缔盟's legal pair choice and atomic hand swap, or its Play-end forced discard.
    ///
    /// Appended for serialized ordinal stability.
    ResolveOldimeng {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned stage: zero chooses first, one chooses second/swaps, two settles discard.
        stage: u8,
        /// First selected other player.
        first: PlayerId,
        /// Second selected other player.
        second: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 界结姻's selected-card target/placement choice and ordered HP comparison benefit.
    ResolveRejieyin {
        /// Skill owner.
        player: PlayerId,
        /// Selected male target, or `NONE` before target choice.
        target: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Selected hand/equipment card.
        card: CardId,
        /// Whether the selected card should enter the target's equipment zone.
        equip: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 仁恤's two-target card transfer and ordered draw checks.
    ResolveReanxu {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Player losing the selected card.
        first: PlayerId,
        /// Player gaining the selected card.
        second: PlayerId,
        /// Selected card.
        card: CardId,
        /// Whether the selected card came from the first target's hand.
        from_hand: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 决堰's equipment-slot choice and branch benefit.
    ResolveDrltJueyan {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Stable selected slot group (`1`, `2`, `3`, or `5`).
        slot: u8,
        /// Whether this is the first activation in the match.
        first: bool,
        /// Permanent +1 Sha-use helper.
        sha_skill_def: SkillDefId,
        /// Temporary +3 Sha-use helper.
        weapon_skill_def: SkillDefId,
        /// Unlimited-distance helper.
        mount_skill_def: SkillDefId,
        /// Hand-limit +3 helper.
        armor_skill_def: SkillDefId,
        /// 界集智 helper.
        treasure_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 界集智's draw and optional discard of the drawn basic card.
    ResolveRejizhi {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Drawn basic card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 诫训's target choice, snapshotted draw/discard counts, and terminal replacement.
    ResolveJiexun {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Chosen other player.
        target: PlayerId,
        /// Snapshotted diamond count.
        draw_count: u32,
        /// Snapshotted existing 诫训 mark count.
        discard_count: u32,
        /// 复难 enhancement helper.
        funan_helper_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Grant 问卦's global active helper to every living player.
    ResolveWenguaSetup {
        /// 问卦 owner.
        player: PlayerId,
        /// Active helper definition.
        helper_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve one 问卦 active-helper invocation.
    ResolveWengua {
        /// Active actor.
        player: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Chosen 问卦 owner.
        target: PlayerId,
        /// Selected hand/equipment card.
        card: CardId,
        /// Top (`0`) or bottom (`1`) branch.
        branch: u8,
        /// 问卦 main definition.
        main_skill_def: SkillDefId,
        /// Per-Play-phase recipient marker.
        marker_skill_def: SkillDefId,
        /// Originating helper skill instance.
        skill: SkillId,
    },
    /// Resolve 新安国's ordered minimum-hand/HP/equipment actions.
    ResolveXinanguo {
        /// Skill owner.
        player: PlayerId,
        /// Selected other target.
        target: PlayerId,
        /// Engine-owned stage (`0` start, `1` target, `2` owner).
        stage: u8,
        /// Current action index (`0` hand, `1` HP, `2` equipment).
        index: u8,
        /// Actions not already performed for the target.
        remaining: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 国战三版刚烈's enemy choice and judgment.
    ResolveGanglieThree {
        /// Skill owner.
        player: PlayerId,
        /// Selected enemy, or `NONE` before target choice.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 忠义 activation, damage bonus, or next-round cleanup.
    ResolveZhongyi {
        /// Skill owner.
        player: PlayerId,
        /// Stable action tag: 0=activate, 1=increase bound Sha damage, 2=cleanup.
        action: u8,
        /// Temporary helper definition.
        helper_skill_def: SkillDefId,
        /// Selected activation card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Discard every equipped card in one slot.
    DiscardEquipmentSlot {
        /// Equipment owner.
        player: PlayerId,
        /// Stable equipment-slot tag.
        slot: u8,
    },
    /// Resolve old 安恤's unequal-hand two-target transfer.
    ResolveOldAnxu {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Player with more hand cards.
        giver: PlayerId,
        /// Player with fewer hand cards.
        receiver: PlayerId,
        /// Selected hand card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Move active invocation cards into the originating skill's expansion pile.
    StoreSelectedInSkillPile {
        /// Skill owner.
        player: PlayerId,
        /// Selected hand cards.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// End an identity match for the player's camp when a skill pile reaches a threshold.
    EndMatchForCampIfSkillPileAtLeast {
        /// Camp representative.
        player: PlayerId,
        /// Definition owning the checked expansion pile.
        pile_skill_def: SkillDefId,
        /// Minimum pile size.
        count: u32,
    },
    /// Resolve old 寝情's target discard/draw sequence.
    ResolveOldQinqing {
        /// Skill owner.
        player: PlayerId,
        /// Selected target.
        target: PlayerId,
        /// Current lord.
        lord: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve old 明鉴's hand transfer and inserted play-only phase.
    ResolveOldMingjian {
        /// Skill owner.
        player: PlayerId,
        /// Selected recipient.
        target: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve old 旋风's virtual-Sha or distance-one damage branch.
    ResolveOldXuanfeng {
        /// Skill owner.
        player: PlayerId,
        /// Selected target.
        target: PlayerId,
        /// Materialized virtual Sha.
        card: CardId,
        /// Engine-owned stage.
        stage: u8,
        /// Branch tag: 1 virtual Sha, 2 direct damage.
        branch: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Discard two cards to prevent the bound damage for 剑柔.
    ResolveJianrou {
        /// Damaged skill owner.
        player: PlayerId,
        /// Engine-owned pre-resolution damage id.
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve old 贿生's reveal, source choice, and bound damage prevention.
    ResolveOldHuisheng {
        /// Damaged skill owner.
        player: PlayerId,
        /// Other living damage source.
        source: PlayerId,
        /// Engine-owned pre-resolution damage id.
        damage_id: u32,
        /// Engine-owned stage.
        stage: u8,
        /// Cards selected by the owner.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve old 解烦's prompted Sha and its bound damage replacement.
    ResolveOldJiefan {
        /// Skill owner and Sha user.
        player: PlayerId,
        /// Player being saved.
        dying: PlayerId,
        /// Current turn player who must be included as a Sha target.
        current: PlayerId,
        /// Selected physical or view-as Sha source.
        card: CardId,
        /// Engine-owned pre-resolution damage id.
        damage_id: u32,
        /// Engine-owned stage: 0 choose Sha, 1 use Sha, 2 replace its damage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 彻骨's target cancellation, branch, ordered target discards, and turn bonus.
    ResolveChegu {
        /// Skill owner and card user.
        player: PlayerId,
        /// Triggering physical card.
        card: CardId,
        /// Original card targets at stage zero; selected discard targets afterward.
        targets: Vec<PlayerId>,
        /// Cards discarded by earlier targets.
        cards: Vec<CardId>,
        /// Branch: 1 one target/many cards, 2 many targets/one card.
        branch: u8,
        /// Next selected target index.
        index: u8,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 归心's lord-skill acquisition or faction change.
    ResolveJunkguixin {
        /// Skill owner.
        player: PlayerId,
        /// Selected faction-change target.
        target: PlayerId,
        /// Engine-owned stage.
        stage: u8,
        /// Selected faction code at the commit stage.
        group: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 谶纬's alternating face-down-card branches.
    ResolveYaoChenwei {
        /// Skill owner.
        player: PlayerId,
        /// Selected player.
        target: PlayerId,
        /// Selected card.
        card: CardId,
        /// Conversion mode: 0 toggle owner's card, 1 gain and turn face-down.
        mode: u8,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let the current Sha user optionally add one legal target within a final-distance cap.
    ///
    /// Appended for serialized ordinal stability.
    ChooseAddCurrentShaTargetWithinDistance {
        /// Sha user and chooser.
        player: PlayerId,
        /// Physical card carrying the logical Sha.
        card: CardId,
        /// Inclusive final-distance cap.
        max_distance: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Refund the currently saved proactive Sha's ordinary use count at most once.
    ///
    /// Appended for serialized ordinal stability.
    RefundCurrentShaUse {
        /// Current Sha user.
        player: PlayerId,
        /// Physical card carrying the logical Sha.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Damage each selected active-skill target, then let that target draw, in seat order.
    ///
    /// Appended for serialized ordinal stability.
    DamageThenDrawSelectedTargets {
        /// Damage source.
        source: PlayerId,
        /// Active-skill targets.
        targets: Vec<PlayerId>,
        /// Damage dealt to each target.
        damage: u32,
        /// Cards drawn by each target afterward.
        draw: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 游敌's target-owned discard and conditional counter-gain.
    ///
    /// Appended for serialized ordinal stability.
    ResolveYoudi {
        /// Skill owner.
        player: PlayerId,
        /// Selected other player.
        target: PlayerId,
        /// Selected card for the current stage.
        card: CardId,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 狂斧's selected equipment and move/discard branch.
    ///
    /// Appended for serialized ordinal stability.
    ResolveKuangfu {
        /// Skill owner.
        player: PlayerId,
        /// Damaged target.
        target: PlayerId,
        /// Selected equipment card.
        card: CardId,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 缮甲's draw, forced discard, and optional virtual Sha.
    ///
    /// Appended for serialized ordinal stability.
    ResolveShanjia {
        /// Skill owner.
        player: PlayerId,
        /// Draw/discard count, capped by the engine at seven.
        count: u32,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 新涯角's reveal, recipient, and category-mismatch discard.
    ///
    /// Appended for serialized ordinal stability.
    ResolveNewYajiao {
        /// Skill owner.
        player: PlayerId,
        /// Card just used or responded with.
        source_card: CardId,
        /// Revealed draw-pile card.
        revealed: CardId,
        /// Selected recipient.
        target: PlayerId,
        /// Snapshot of the source card's basic/trick/equipment category.
        source_category: u8,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 定品's judgment and color-dependent result.
    ///
    /// Appended for serialized ordinal stability.
    ResolveDingpin {
        /// Skill owner.
        player: PlayerId,
        /// Selected judging target.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Discard the shared processing area, abort the remaining current resolution, and finish the
    /// current phase without recording it as skipped.
    ///
    /// Appended for serialized ordinal stability.
    FinishCurrentPhaseAbortResolution {
        /// Skill owner that requested the abort.
        player: PlayerId,
        /// Engine-owned stage: 0 abort/discard, 1 finish the phase after move triggers.
        stage: u8,
    },
    /// Move the draw-pile top card into the originating skill's public pile.
    GainTopCardToSkillPile {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose and discard one card from the originating skill's public pile, then add one mark of
    /// a helper skill to the target.
    ChooseDiscardSkillPileCardAddMarkedSkill {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player receiving the helper skill.
        target: PlayerId,
        /// Helper skill definition and mark id.
        helper_skill_def: SkillDefId,
        /// Selected pile card, or `NONE` before choice.
        selected: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Gain one current hand card from each active-skill target in submitted order.
    GainOneHandFromSelectedTargets {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Active-skill targets in submitted order.
        targets: Vec<PlayerId>,
        /// Current target index.
        index: u32,
        /// Selected hand card, or `NONE` before choice.
        selected: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Optionally choose living 破虏 recipients, increment its mark once, then draw in seat order.
    ///
    /// The chooser may be dead because one authoritative trigger fires on the owner's death.
    /// Appended for serialized ordinal stability.
    ChooseRepoluTargetsDraw {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance; its definition is also the stable mark id.
        skill: SkillId,
    },
    /// Resolve 过论's reveal, optional exchange, and lower-rank draw.
    ///
    /// `player_card` and `target_card` are `NONE` before the first choice and both populated
    /// only for the atomic exchange. Appended for serialized ordinal stability.
    ResolveXinfuGuolun {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player whose hand card is selected.
        target: PlayerId,
        /// Owner's selected hand/equipment card.
        player_card: CardId,
        /// Target's selected hand card.
        target_card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Force one hand/equipment-card discard, refund the currently resolving proactive Sha's
    /// use count at most once, draw for the skill owner if that Sha is effectively red, and
    /// restore one skill definition when the paid card and Sha have equal effective ranks.
    ///
    /// This is a distinct appended request so the original Longyin request and its serialized
    /// shape remain stable.
    ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
        /// Skill owner choosing and paying the discard cost.
        player: PlayerId,
        /// Player who used the currently resolving Sha.
        sha_user: PlayerId,
        /// Physical card carrying the logical Sha.
        card: CardId,
        /// Skill definition restored only when both effective ranks are equal.
        restore_skill_def: SkillDefId,
        /// Originating skill instance for draw-trigger provenance.
        skill: SkillId,
    },
    /// Resolve Beige's existing cost, judgment, and rewrite pipeline with content-supplied
    /// Heart recovery and Diamond draw counts.
    ///
    /// This is a distinct appended request so the original `ResolveBeige` serialized shape and
    /// stable hash remain unchanged.
    ResolveBeigeConfigured {
        /// Beige owner.
        player: PlayerId,
        /// Player damaged by Sha and judged by Beige.
        target: PlayerId,
        /// Damage source.
        source: PlayerId,
        /// 0=cost, 1=flip, 2=start rewrites, 3=apply suit branch, 4=continue rewrites.
        stage: u8,
        /// Current physical judgment result.
        #[serde(default = "no_card")]
        result_card: CardId,
        /// Current effective judgment suit.
        #[serde(default = "no_suit")]
        suit: Suit,
        /// Current physical judgment rank.
        #[serde(default)]
        rank: u8,
        /// Remaining judgment rewriters in seat order.
        #[serde(default)]
        rewriters: Vec<PlayerId>,
        /// Amount recovered by the damaged player on a Heart result.
        heart_recover_amount: u32,
        /// Number of cards drawn by the damaged player on a Diamond result.
        diamond_draw_count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let a dead 追忆 owner optionally choose one living recipient other than the killer,
    /// then recover that recipient and draw cards in the continuation.
    ///
    /// Appended for serialized ordinal stability.
    ResolveZhuiyi {
        /// Dead skill owner and choice observer.
        player: PlayerId,
        /// Credited killer, or `NONE` for a source-free death.
        killer: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 界镇军 through an independent state machine while preserving legacy `rezhenjun`.
    ///
    /// Only stage zero is constructible by Rule VM content. Stage one is an engine-owned
    /// continuation after the selected target cards and their nested triggers have resolved.
    /// Appended for serialized ordinal stability.
    ResolveDecadezhenjun {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Selected target, or `NONE` before target selection.
        #[serde(default = "no_player")]
        target: PlayerId,
        /// 0=choose target/cards, 1=offer one owner discard or let the target draw.
        #[serde(default)]
        stage: u8,
        /// Number of all-non-equipment target cards discarded; also the fallback draw count.
        #[serde(default)]
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve SP 骁果 through an independent state machine while preserving Standard 骁果.
    ///
    /// Only stage zero is constructible by Rule VM content. Stages one through three are
    /// engine-owned continuations that run after the preceding discard and all nested
    /// card-movement triggers have resolved.
    /// Appended for serialized ordinal stability.
    ResolveSpXiaoguo {
        /// Skill owner.
        player: PlayerId,
        /// End-phase player.
        target: PlayerId,
        /// 0=choose owner cost, 1=offer target equipment, 2=damage fallback, 3=owner draw.
        #[serde(default)]
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Replace the pending normal draw with choosing and gaining one card from every other
    /// living player's hand/equipment/judgment zones in clockwise order.
    ///
    /// Only the initial empty target list is constructible by Rule VM content. The engine owns
    /// the stable remaining-target snapshot across nested card-movement triggers.
    /// Appended for serialized ordinal stability.
    ChooseGainCardFromEachOtherReplaceDraw {
        /// Skill owner, chooser, and card recipient.
        player: PlayerId,
        /// Whether the stable target snapshot has been initialized.
        #[serde(default)]
        initialized: bool,
        /// Engine-owned remaining targets in owner-relative clockwise order.
        #[serde(default)]
        remaining_targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Give selected hand cards in one batch, then recover the recipient after every movement
    /// event and nested trigger has completed.
    ///
    /// Appended for serialized ordinal stability.
    GiveSelectedThenRecoverTarget {
        /// Skill owner and giver.
        player: PlayerId,
        /// Gift recipient and recovery target.
        target: PlayerId,
        /// Selected hand cards.
        cards: Vec<CardId>,
        /// Recovery amount.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
        /// Engine-owned stage flag set after the gift batch has moved.
        #[serde(default)]
        ready_to_recover: bool,
    },
    /// Resolve SP 定叛's draw, target branch, equipment movement, and delayed damage.
    ///
    /// Only stage zero is constructible by Rule VM content. Stages one and two are
    /// engine-owned tails after draw or equipment-movement triggers finish.
    /// Appended for serialized ordinal stability.
    ResolveDingpan {
        /// Skill owner and possible damage source.
        player: PlayerId,
        /// Selected equipment-bearing target and branch chooser.
        target: PlayerId,
        /// 0=draw, 1=choose branch, 2=deal delayed damage.
        #[serde(default)]
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve OFFLINE 贪赂's frozen HP-gap branch, gift, damage, and delayed discard.
    ///
    /// Only stage zero is constructible by Rule VM content. Later stages are engine-owned
    /// continuations after the target's branch or the complete damage chain.
    /// Appended for serialized ordinal stability.
    ResolveTanlu {
        /// Skill owner, gift recipient, and possible damage source.
        player: PlayerId,
        /// Current-turn player making every forced choice.
        target: PlayerId,
        /// Frozen absolute signed HP difference.
        #[serde(default)]
        amount: u32,
        /// 0=freeze and branch, 1=damage, 2=post-damage discard.
        #[serde(default)]
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Judge repeatedly while results match the first judgment's effective color, gaining each
    /// matching result before optionally continuing.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    JudgeSameColorGainRepeat {
        /// Judged player and matching-card recipient.
        player: PlayerId,
        /// Matching judgment card awaiting gain, or `NONE` before a new judgment.
        #[serde(default = "no_card")]
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the gained-card trigger chain has completed and the repeat choice may open.
        #[serde(default)]
        ready_to_choose: bool,
        /// First effective judgment color: 0=unset, 1=red, 2=black.
        #[serde(default)]
        color: u8,
    },
    /// Let a skill owner choose one or more current targets of another player's suspended
    /// non-multitarget trick, then make that trick ineffective for those targets.
    ///
    /// Appended for serialized ordinal stability.
    ChooseCurrentTrickTargetsToNullify {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Current trick user.
        source: PlayerId,
        /// Physical card carrying the logical trick.
        card: CardId,
        /// Originating limited skill instance.
        skill: SkillId,
    },
    /// Judge repeatedly while results are black, gaining each black result and excluding that
    /// concrete card from this player's ordinary discard-to-limit accounting for the turn.
    ///
    /// Appended for serialized ordinal stability.
    JudgeBlackGainRepeatProtectHandLimit {
        /// Judged player and black-card recipient.
        player: PlayerId,
        /// Black judgment card awaiting gain, or `NONE` before a new judgment.
        #[serde(default = "no_card")]
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the gained-card trigger chain has completed and the repeat choice may open.
        #[serde(default)]
        ready_to_choose: bool,
    },
    /// Give one legal recipient every physical card consumed by the resolved Sha, then let a
    /// red gift use Sha and let a black gift draw one card.
    ///
    /// Appended for serialized ordinal stability.
    ChooseGainAllResolvedShaShanThenColorRewards {
        /// Skill owner and original Sha user.
        player: PlayerId,
        /// Physical Sha carrier.
        sha_card: CardId,
        /// Physical Shan source cards observed during this Sha use.
        shan_cards: Vec<CardId>,
        /// Original Sha targets, all excluded from receiving the gift.
        excluded_recipients: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose up to the current draw count of other players, gain one hand card from each, and
    /// reduce rather than replace the pending normal draw.
    ///
    /// Appended for serialized ordinal stability.
    ChooseOtherHandTargetsGainOneReduceDraw {
        /// Current Draw-phase player and chooser.
        player: PlayerId,
        /// Snapshotted selected targets in choice order.
        targets: Vec<PlayerId>,
        /// Current target index.
        index: u32,
        /// Selected target hand card, or NONE while opening/scanning choices.
        selected: CardId,
        /// Draw count captured before the target choice.
        draw_count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose and discard one hand/equipment card from the current Sha target.
    ///
    /// A non-basic selected card makes that target unable to respond, grants the supplied
    /// turn-scoped modifier skill, and increments its matching mark. A basic selected card gives
    /// the target the physical Sha if it remains gainable. Appended for serialized stability.
    ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
        /// Current Sha user and card chooser.
        source: PlayerId,
        /// Physical Sha currently resolving.
        card: CardId,
        /// Current target whose card may be discarded.
        target: PlayerId,
        /// Turn-scoped modifier skill definition and matching mark definition.
        bonus_skill_def: SkillDefId,
    },
    /// Resolve 趫猛's target choice, target-card discard, and equipment/trick branch.
    ///
    /// Only stage zero is constructible by Rule VM content. Later stages are reconstructed from
    /// authoritative choices. Appended for serialized ordinal stability.
    ResolveDcqiaomeng {
        /// Current black-card user, skill owner, and chooser.
        player: PlayerId,
        /// Physical card whose use is suspended.
        card: CardId,
        /// Ordered target snapshot captured from the triggering card event.
        targets: Vec<PlayerId>,
        /// Selected target, or NONE before target selection.
        target: PlayerId,
        /// Selected target card, or NONE before card selection.
        selected: CardId,
        /// Exact-card policy helper definition for the trick-card branch.
        helper_skill_def: SkillDefId,
        /// 0=choose target, 1=choose card, 2=resolve selected card.
        #[serde(default)]
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 界许褚「裸衣」: reveal the draw-pile top three, optionally replace the
    /// normal draw, gain every basic card, Duel, and weapon, discard the rest, and grant
    /// the configured damage helper until the owner's next turn begins.
    ///
    /// Only stage zero is constructible by Rule VM content. Later stages are engine-owned
    /// continuations after reveal and choice resolution. Appended for serialized stability.
    ResolveNewReluoyi {
        /// Current Draw-phase player and card recipient.
        player: PlayerId,
        /// Temporary Sha/Duel damage helper definition.
        buff_skill_def: SkillDefId,
        /// 0=reveal, 1=choose whether to replace draw, 2=move cards, 3=grant helper.
        #[serde(default)]
        stage: u8,
        /// Stable revealed-card snapshot.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Whether the player accepted the draw replacement.
        #[serde(default)]
        accepted: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve OL Huang Zhong's locked 没矢 placement or retrieval branch.
    ///
    /// The public expansion pile is keyed by the originating skill definition. Appended for
    /// serialized ordinal stability.
    ResolveRemoshi {
        /// Skill owner.
        player: PlayerId,
        /// Damaged player or player who lost equipment.
        target: PlayerId,
        /// Current physical Sha carrier for placement; NONE for retrieval.
        card: CardId,
        /// Presentation helper attached to a player holding 没矢 cards.
        helper_skill_def: SkillDefId,
        /// 0=place the physical Sha, 1=retrieve the target's 没矢 pile.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 界全琮「邀名」's target, branch, and branch-specific card choices.
    ///
    /// Appended for serialized ordinal stability.
    ResolveXinyaoming {
        /// Skill owner.
        player: PlayerId,
        /// Selected target; NONE opens the target choice.
        target: PlayerId,
        /// 0=choose target/branch, 1=draw, 2=discard target hand, 3=target recast-like discard.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 界公孙渊「怀异」's staged reveal, discard, and gain flow.
    ///
    /// Appended for serialized ordinal stability. Content emits stage zero with the remaining
    /// progress fields empty; the engine owns later stages.
    ResolveRehuaiyi {
        /// Skill owner.
        player: PlayerId,
        /// 0=reveal/choose color, 1=choose targets, 2=gain one card from each target.
        stage: u8,
        /// Selected effective color bit: 1=red, 2=black, 0=not selected.
        color: u8,
        /// Number of hand cards discarded by the selected-color branch.
        discarded_count: u32,
        /// Stable selected targets in seat order.
        targets: Vec<PlayerId>,
        /// Current target index for stage two.
        index: u32,
        /// Number of cards actually gained so far.
        gained_count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 界朱治「安国」's ordered minimum-hand/HP/equipment actions and recast.
    ///
    /// Appended for serialized ordinal stability. The engine owns progress after stage zero.
    ResolveReanguo {
        /// Skill owner.
        player: PlayerId,
        /// Selected other target.
        target: PlayerId,
        /// Engine-owned stage (`0` start, `1` target, `2` owner).
        stage: u8,
        /// Current action index (`0` hand, `1` HP, `2` equipment).
        index: u8,
        /// Actions not yet actually performed.
        remaining: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let the bound damage target discard `max(1, 4-distance)` hand cards or increase the
    /// current damage by one.
    ///
    /// Appended for serialized ordinal stability.
    ChooseTargetDiscardDistanceHandOrIncreaseDamage {
        /// Damage source and skill owner.
        player: PlayerId,
        /// Bound damage target and discard chooser.
        target: PlayerId,
        /// Engine-owned damage event id.
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose to draw one card, privately inspect up to two sampled target hand cards and gain
    /// one, or move one compatible target equipment card.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDrawViewTwoGainOrMoveEquipment {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player whose turn ended.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 备诛's forced discard/gift branch or its deterministic queue of physical Sha.
    ///
    /// Content emits stage zero with an empty card queue; later stages are engine-owned.
    /// Appended for serialized ordinal stability.
    ResolveBeizhu {
        /// Skill owner.
        player: PlayerId,
        /// Selected other player.
        target: PlayerId,
        /// 0=start, 1=offer a draw-pile Sha after the forced discard, 2=use queued Sha.
        stage: u8,
        /// Shuffled activation-time physical Sha snapshot for stage two.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve mobile 曹纯「缮甲」's draw, dynamic discard, and category rewards.
    ///
    /// Content emits stage zero. Later stages are engine-owned so discard movement triggers
    /// finish before the temporary distance modifier and optional virtual Sha are offered.
    /// Appended for serialized ordinal stability.
    ResolveMbshanjia {
        /// Skill owner.
        player: PlayerId,
        /// Match-lifetime equipment-zone loss count captured on activation.
        lost_equipment_count: u32,
        /// Temporary all-card distance-unlimited helper definition.
        distance_skill_def: SkillDefId,
        /// 0=draw, 1=discard, 2=grant distance reward, 3=offer virtual Sha.
        #[serde(default)]
        stage: u8,
        /// Reward bits captured before discard: bit 0=no basic, bit 1=no trick.
        #[serde(default)]
        reward_mask: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve mobile 胡车儿「盗戟」's equipment gain, forced use, and weapon damage.
    ///
    /// Content emits stage zero. Later stages are engine-owned so gain and equipment-use
    /// triggers finish before the next step.
    /// Appended for serialized ordinal stability.
    ResolveDaoji {
        /// Skill owner.
        player: PlayerId,
        /// Selected other player.
        target: PlayerId,
        /// Gained physical equipment card after stage zero.
        card: CardId,
        /// 0=choose equipment, 1=use if legal, 2=deal weapon damage.
        #[serde(default)]
        stage: u8,
        /// TW variant: gain from hand/equipment, draw for basic, use any equipment then damage.
        #[serde(default)]
        tw: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve mobile 李遗「教化」's category and recipient choices.
    ///
    /// Content emits category zero and no target. Later calls are engine-owned.
    /// Appended for serialized ordinal stability.
    ResolveJiaohua {
        /// Skill owner.
        player: PlayerId,
        /// Logical category bit: 1=basic, 2=trick, 4=equipment; zero starts the choice.
        #[serde(default)]
        category: u8,
        /// Selected living recipient; none starts the target choice.
        #[serde(default = "no_player")]
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve mobile 来敏「乱群」's private selections, reveal, gains, and helpers.
    ///
    /// Content emits stage zero with empty snapshots. Later calls are engine-owned.
    /// Appended for serialized ordinal stability.
    ResolveLuanqun {
        /// Skill owner.
        player: PlayerId,
        /// First-Sha target restriction helper definition.
        effect_skill_def: SkillDefId,
        /// Turn-long unrespondable helper definition.
        direct_hit_skill_def: SkillDefId,
        /// 0=begin private selections, 1=reveal and apply results.
        #[serde(default)]
        stage: u8,
        /// Seat-ordered players who selected a card.
        #[serde(default)]
        participants: Vec<PlayerId>,
        /// One selected hand card per participant.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let `giver` give exactly two differently colored hand/equipment cards to `target`; if
    /// the giver declines or has no valid pair, recover the target by one HP.
    ///
    /// `owner` identifies the originating skill owner and may differ from the choice observer.
    /// Appended for serialized ordinal stability.
    ChooseGiveTwoDifferentColorsOrRecover {
        /// Living owner of the originating skill.
        owner: PlayerId,
        /// Living player who chooses and gives the cards.
        giver: PlayerId,
        /// Living recipient, normally the current dying player.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve mobile 吴珂「助国」's capped hand adjustment and optional follow-up Sha.
    ///
    /// Content emits stage zero. Stage one runs after the draw and nested movement triggers.
    /// Appended for serialized ordinal stability.
    ResolveMbzhuguo {
        /// Skill owner.
        player: PlayerId,
        /// Selected hand-adjustment target.
        target: PlayerId,
        /// 0=adjust hand; 1=offer follow-up target after a draw.
        #[serde(default)]
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve mobile 曹性「暗弦」's discarded-Sha gain and forced follow-up use.
    ///
    /// Content emits stage zero with the current movement event's physical cards. Stage one
    /// runs after the gain and nested movement triggers.
    /// Appended for serialized ordinal stability.
    ResolveMbanxian {
        /// Skill owner, card recipient, and Sha user.
        player: PlayerId,
        /// Current movement event cards at stage zero.
        cards: Vec<CardId>,
        /// Selected physical Sha after stage zero.
        #[serde(default = "no_card")]
        card: CardId,
        /// 0=choose and gain one discarded Sha; 1=force its use when legal.
        #[serde(default)]
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve mobile 曹性「劲镞」 and its turn-bound next-Sha helper.
    ///
    /// Content emits stage zero from the active skill with one selected owner hand card.
    /// Stage one evaluates the jointly revealed cards. Stage two consumes the helper on the
    /// owner's next Sha. Stages three and four finalize post-discard refresh, with four also
    /// granting the helper.
    /// Appended for serialized ordinal stability.
    ResolveMbjinzu {
        /// Active owner or next-Sha helper owner.
        player: PlayerId,
        /// Selected other player during active resolution.
        #[serde(default = "no_player")]
        target: PlayerId,
        /// Owner-selected card followed by target-selected cards during active resolution.
        cards: Vec<CardId>,
        /// Currently suspended Sha source card during stage two.
        #[serde(default = "no_card")]
        card: CardId,
        /// Hidden helper definition used by active stages.
        #[serde(default = "no_skill_def")]
        helper_skill_def: SkillDefId,
        /// 0=start; 1=evaluate reveals; 2=apply next Sha; 3=refresh; 4=refresh and grant.
        #[serde(default)]
        stage: u8,
        /// Originating active or helper skill instance.
        skill: SkillId,
    },
    /// Let one player choose and gain one card from another player's typed zone scope.
    ///
    /// Appended for serialized ordinal stability. Unlike the legacy [`Self::TakeOneCard`],
    /// this request supports the judgment zone and uses the shared gain-protection query.
    TakeOneScopedCard {
        /// Living player who owns the offered cards.
        from: PlayerId,
        /// Living player choosing and gaining the card.
        to: PlayerId,
        /// Canonical zones offered to the chooser.
        scope: TargetCardScope,
    },
    /// Publicly reveal one player's complete current hand.
    ///
    /// Appended for serialized ordinal stability.
    RevealHand {
        /// Player whose hand is revealed.
        player: PlayerId,
    },
    /// Draw `count` cards for every currently linked living player in seat order.
    ///
    /// Appended for serialized ordinal stability.
    DrawLinkedPlayers {
        /// Cards drawn by each linked player.
        count: u32,
    },
    /// Publicly reveal the draw-pile top card, then gain it when its printed color matches.
    /// A nonmatching card enters the discard pile.
    ///
    /// Appended for serialized ordinal stability.
    RevealTopGainIfColor {
        /// Living player revealing and possibly gaining the card.
        player: PlayerId,
        /// Printed-color mask: `1` red or `2` black.
        color_mask: u8,
    },
    /// Suppress one player's non-locked skills until the current turn ends.
    ///
    /// Appended for serialized ordinal stability.
    SuppressNonLockedSkillsUntilTurnEnd {
        /// Living player being suppressed.
        player: PlayerId,
    },
    /// Let one player cancel or recover a specified target.
    ///
    /// Appended for serialized ordinal stability.
    ChooseRecoverTarget {
        /// Living player making the choice.
        chooser: PlayerId,
        /// Living wounded recovery target.
        target: PlayerId,
        /// Maximum HP recovered.
        amount: u32,
    },
    /// Give every hand card to another player, who may use one of those exact cards; return the
    /// other gifted cards before that use, or deal one normal damage when the offer is declined.
    ///
    /// Appended for serialized ordinal stability.
    GiveAllHandThenChooseUseOneOrDamage {
        /// Original hand owner and damage source.
        player: PlayerId,
        /// Gift recipient and optional card user.
        target: PlayerId,
        /// Exact gifted cards captured at stage zero.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Selected legal targets for the gifted card at the final stage.
        #[serde(default)]
        targets: Vec<PlayerId>,
        /// Engine-owned stage: zero gives, one chooses a card, two chooses targets, three chooses
        /// a target pair.
        #[serde(default)]
        stage: u8,
        /// Selected gifted card after stage one.
        #[serde(default = "no_card")]
        card: CardId,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Choose one logical card category, then let the current Sha target give the source one
    /// matching hand/equipment card or make that Sha unrespondable for the target.
    ///
    /// Appended for serialized ordinal stability.
    ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
        /// Current Sha user and skill owner.
        player: PlayerId,
        /// Current physical Sha.
        card: CardId,
        /// Current Sha target.
        target: PlayerId,
        /// Originating trigger skill instance.
        skill: SkillId,
    },
    /// Choose targets for an ignore-distance virtual Sha, then lose 1 HP and use it. Its base
    /// damage is increased when the user played multiple distinct basic-card variants this turn.
    ///
    /// Appended for serialized ordinal stability.
    ChooseLoseHpUseVirtualShaByBasicVariants {
        /// Skill owner and virtual Sha user.
        player: PlayerId,
        /// Materialized virtual Sha after target selection begins.
        #[serde(default = "no_card")]
        card: CardId,
        /// Selected legal Sha targets at the post-cost stage.
        #[serde(default)]
        targets: Vec<PlayerId>,
        /// Zero chooses targets; one uses the saved virtual Sha after the HP-loss barrier.
        #[serde(default)]
        stage: u8,
        /// Captured virtual Sha damage bonus.
        #[serde(default)]
        damage_bonus: u32,
        /// Originating trigger skill.
        skill: SkillId,
    },
    /// 嬉战：弃置一张牌或失去体力，再按花色对回合角色视为用牌，并暂时禁用放纵。
    ///
    /// Appended for serialized ordinal stability.
    ResolveSpxizhan {
        /// Skill owner.
        player: PlayerId,
        /// Current-turn player (phase begin event player).
        phase_player: PlayerId,
        /// Skill definition to temporarily disable (放纵).
        fangzong_skill_def: SkillDefId,
    },
    /// 复叛：按伤害点数摸牌，交出一张牌，并按目标的累计状态结算奖励或伤害。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwfupan {
        /// Skill owner.
        player: PlayerId,
        /// Damage amount captured by the trigger.
        damage_amount: u32,
        /// Selected card between the card and target choice stages.
        #[serde(default = "no_card")]
        card: CardId,
        /// Selected recipient between the gift and confirmation stages.
        #[serde(default = "no_player")]
        target: PlayerId,
        /// Engine-owned stage.
        #[serde(default)]
        stage: u8,
        /// Originating trigger skill instance.
        skill: SkillId,
    },
    /// 颂蜀：将当前摸牌阶段摸牌数置零，令摸牌角色从仁库技能牌堆选牌，并临时禁止指定其他角色。
    ///
    /// Appended for serialized ordinal stability.
    ResolveSpsongshu {
        /// Skill owner (仁库所在角色).
        player: PlayerId,
        /// Draw-phase player.
        phase_player: PlayerId,
        /// Skill definition whose public pile is the renku (割卜).
        renku_skill_def: SkillDefId,
        /// Temporary “cannot target others” skill definition.
        block_skill_def: SkillDefId,
    },
    /// 教化（TW）：摸牌后令摸牌角色获得一张本次未获得类别的牌。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwjiaohua {
        /// Skill owner.
        player: PlayerId,
        /// Player who drew the triggering cards.
        target: PlayerId,
        /// Physical cards captured by the draw event.
        event_cards: Vec<CardId>,
        /// Originating trigger skill instance.
        skill: SkillId,
    },
    /// 苦谏（TW）：赠予并追踪实体牌，或结算其使用/打出及其他失去分支。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwkujian {
        /// Skill owner.
        player: PlayerId,
        /// Gift recipient or player who used/lost tracked cards.
        target: PlayerId,
        /// Selected or event physical cards.
        cards: Vec<CardId>,
        /// Zero gifts; one rewards use/response; two penalizes another loss.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Move cards with an engine-owned semantic reason.
    ///
    /// Appended separately so existing `MoveCards` producers keep their wire shape and ordinals.
    MoveCardsWithReason {
        /// Cards in order.
        cards: Vec<CardId>,
        /// Destination zone.
        to: ZoneId,
        /// Optional explicit source (validated by core).
        from: Option<ZoneId>,
        /// Authoritative movement cause.
        reason: engine_api::CardMoveReason,
    },
    /// 睿敛（TW）：选择本轮关注角色、记录其弃牌，并在其回合结束按类别获得弃牌堆牌。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwruilian {
        /// Skill owner.
        player: PlayerId,
        /// Selected role or triggering event player.
        target: PlayerId,
        /// Physical cards captured by a discard event.
        #[serde(default)]
        event_cards: Vec<CardId>,
        /// Zero prompts for a target; one records a discard; two closes the tracked turn; three gains.
        action: u8,
        /// Selected category for action three: 1 basic, 2 trick, 3 equipment.
        #[serde(default)]
        category: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 济溯（collab 叶诗文）：判定阶段开始时跳过本次判定与摸牌阶段，随后视为使用虚拟【杀】。
    ///
    /// Sets the pending judge skip and the SKIP_DRAW turn flag for the current player.
    /// Appended for serialized ordinal stability.
    SkipJudgeAndDrawPhases {
        /// Current phase player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Randomly equip one physical weapon from the shared draw pile when the slot is empty.
    ///
    /// Appended for serialized ordinal stability.
    EquipWeaponFromDrawPile {
        /// Equipment recipient.
        player: PlayerId,
    },
    /// 剑威（TW）：双向发起拼点并按胜负结算区域牌或武器。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwjianwei {
        /// Skill owner.
        player: PlayerId,
        /// Comparison initiator.
        actor: PlayerId,
        /// Comparison opponent.
        #[serde(default = "no_player")]
        target: PlayerId,
        /// Initiator's selected comparison card.
        #[serde(default = "no_card")]
        card: CardId,
        /// Comparison winner for settlement.
        #[serde(default = "no_player")]
        winner: PlayerId,
        /// Zero starts owner's branch; one starts another player's branch; two starts the
        /// comparison after both participants are fixed; three settles the finished comparison.
        #[serde(default)]
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 探锋（TW）：弃置目标区域牌后令其选择以牌当【杀】，否则受伤并跳过阶段。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwtanfeng {
        /// Skill owner.
        player: PlayerId,
        /// Selected other player.
        #[serde(default = "no_player")]
        target: PlayerId,
        /// Selected physical card used as Sha.
        #[serde(default = "no_card")]
        card: CardId,
        /// Resolver stage.
        #[serde(default)]
        stage: u8,
        /// Selected future phase tag for the skip branch.
        #[serde(default)]
        phase: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 直辩（TW）：发起拼点并结算移动对应区域牌、回复或背水。
    ResolveTwzhibian {
        /// Skill owner.
        player: PlayerId,
        /// Comparison opponent.
        #[serde(default = "no_player")]
        target: PlayerId,
        /// Comparison winner; NONE is a tie.
        #[serde(default = "no_player")]
        winner: PlayerId,
        /// Resolver stage.
        #[serde(default)]
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 御严（TW）：令当前 Sha 使用者交牌，否则取消拥有者这一目标。
    ResolveTwyuyan {
        /// Skill owner and current Sha target.
        player: PlayerId,
        /// Current Sha source.
        source: PlayerId,
        /// Current physical Sha carrier.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 果毅（TW）：目标选择禁用手牌或弃牌，并在满足条件时登记原牌重放。
    ResolveTwguoyi {
        /// Skill owner and current card user.
        player: PlayerId,
        /// Current card target making the branch choice.
        target: PlayerId,
        /// Current physical or materialized card.
        card: CardId,
        /// Temporary helper skill that disables hand-card use and responses.
        hand_lock_skill_def: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Replay a virtual card on its original targets after the source card has fully settled.
    ReplayVirtualCardOnTargets {
        /// Card user.
        player: PlayerId,
        /// Stable logical card-kind tag.
        kind: u8,
        /// Original targets in declaration order.
        targets: Vec<PlayerId>,
        /// Skill provenance used to prevent recursive replay.
        skill: SkillId,
    },
    /// 除害（TW）：完成使命后的判定区废除/座次收牌，或弃置本批从他人获得的牌。
    ResolveTwchuhai {
        /// Skill owner.
        player: PlayerId,
        /// Zero resolves mission success; one discards one triggering gained card.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Engine-owned seat queue index.
        index: u32,
        /// Physical cards captured by the triggering movement.
        event_cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 遗珠（TW）：埋置实体牌、修改他人的单目标用牌，或清除弃牌堆中的追踪牌。
    ResolveTwyizhu {
        /// Skill owner.
        player: PlayerId,
        /// Current card user for target adjustment.
        source: PlayerId,
        /// Current logical card.
        card: CardId,
        /// Zero plants pearls; one handles their use; two clears discarded pearls.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Physical cards captured by the triggering event.
        event_cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 乱筹/共患（TW）：重置共享技能持有者，或将当前伤害完整转移给技能拥有者。
    ResolveTwluanchou {
        /// Skill owner.
        player: PlayerId,
        /// Shared helper skill definition.
        helper_skill_def: u32,
        /// Zero resets the two selected holders; one redirects current damage.
        action: u8,
        /// Active invocation targets.
        targets: Vec<PlayerId>,
        /// Current damage source.
        source: PlayerId,
        /// Current damage target.
        target: PlayerId,
        /// Current damage amount.
        amount: u32,
        /// Current damage nature.
        nature: u8,
        /// Current damage card.
        card: CardId,
        /// Current logical card kind.
        card_kind: u8,
        /// Current card provenance flags.
        card_flags: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 天香（old/shenhua 小乔）：弃一张红桃手牌、选择一名其他角色，将当前伤害
    /// 重定向（取消后按同源/同量/同属性/同牌重新结算）并为其附加摸牌技能。
    ///
    /// VM 首发（`target == NONE`）时由 vm_host 从 damage_before 上下文捕获
    /// source/amount/nature/card；选择结算后经 `ResolveTianxiang` 重发应用。
    /// ABI 456（C 段 omp）。
    ResolveTianxiang {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Selected redirect target, or `NONE` before the target choice.
        target: PlayerId,
        /// Heart hand card discarded as the cost, or `NONE` before the card choice.
        discard_card: CardId,
        /// Current damage source (host-captured).
        source: PlayerId,
        /// Current damage amount (host-captured).
        amount: u32,
        /// Current damage nature (host-captured).
        nature: u8,
        /// Current damage card (host-captured).
        card: CardId,
        /// Current logical card kind (host-captured).
        card_kind: u8,
        /// Current card provenance flags (host-captured).
        card_flags: u8,
        /// Skill granted to the redirect target after the damage settles.
        draw_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 界天香（shenhua 小乔）：弃一张红桃手牌、选择一名其他角色、取消当前伤害，
    /// 由拥有者二选一：目标受伤害来源 1 点伤害后摸 min(5, 已损失体力) 张牌，或
    /// 目标失去 1 点体力后获得弃置的红桃牌。
    ///
    /// VM 首发（`target == NONE`）时由 vm_host 捕获 damage_before 上下文的伤害来源；
    /// 选择结算后经 `ResolveRetianxiang` 重发应用。ABI 457（C 段 omp）。
    ResolveRetianxiang {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Selected target, or `NONE` before the target choice.
        target: PlayerId,
        /// Heart hand card discarded as the cost, or `NONE` before the card choice.
        discard_card: CardId,
        /// Owner-chosen branch: 0 = damage-1-from-source + draw, 1 = lose-1-HP + gain card.
        branch: u8,
        /// Current damage source (host-captured).
        source: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 链舟（huicui 蔡瑁张允）：准备阶段选择任意名体力值与拥有者相同且未横置的
    /// 其他角色并横置之。
    ///
    /// `targets` 为空是 VM 首发（打开目标选择）；选择结算后重发应用横置。ABI 458。
    ResolveLianzhou {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Selected link targets (empty until the choice resolves).
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 得宠（sixiang 标毛皇后）：其他角色准备阶段，将任意张可弃置手牌/装备交给
    /// 该角色；其下回合弃牌阶段开始时若手牌数不小于体力值，拥有者可对其造成 1 点
    /// 伤害（后续阶段经 bound 技能表达）。
    ///
    /// `cards` 为空是 VM 首发（打开选牌）；选择结算后重发赠牌。ABI 459。
    ResolveStddechong {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Recipient (the triggering prepare-phase player, host-captured).
        target: PlayerId,
        /// Selected hand/equipment cards to give (empty until the choice resolves).
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 通用弃牌数量代价（`yangzhong` 殃众等）：选择恰好 `count` 张可弃置手牌/装备
    /// 并弃置。
    ///
    /// `count == 0` 是 VM 首发（打开选牌）；选择结算后重发弃置。ABI 460。
    DiscardHandEquipCount {
        /// Player discarding the cards.
        player: PlayerId,
        /// Number of cards to discard.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 劫粮（sp2 韩猛）：当前回合角色于弃牌阶段弃牌后，选择一张仍处弃牌堆的弃牌
    /// 获得。
    ///
    /// VM 首发时由 vm_host 捕获弃牌者（`event_source`）与本次移动牌（`event_cards`）。
    /// ABI 461。
    ResolveJieliangGain {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Mover of the discarded cards (host-captured).
        mover: PlayerId,
        /// Cards moved by the triggering event (host-captured).
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 举荐（sixiang 标徐庶）：使用【无懈可击】后，选择一名其他角色获得该牌。
    ///
    /// `target == NONE` 是 VM 首发（打开目标选择）；选择结算后重发获牌。
    /// `cards` 由 vm_host 从使用事件捕获。ABI 462。
    ResolveStdjujian {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Selected recipient, or `NONE` before the target choice.
        target: PlayerId,
        /// Used cards captured from the triggering event (host-captured).
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 掠卤（sp2 梁兴）：选择一名手牌数更少的其他角色，由其二选一：交全部手牌并
    /// 令拥有者翻面，或自己翻面并视为对拥有者使用【杀】。
    ///
    /// `target == NONE` 打开目标选择；`branch == 2` 时由目标选择分支；0/1 应用。
    /// ABI 463。
    ResolveLulve {
        /// Skill owner.
        player: PlayerId,
        /// Selected other player, or `NONE` before the target choice.
        target: PlayerId,
        /// Owner-chosen branch: 0 = give-all + flip owner, 1 = flip target + virtual Sha.
        branch: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 铤险（sp2 孙狼）：使用【杀】时（每回合一次）摸装备数+1 张牌，并令此【杀】
    /// 对至多等量目标无效。
    ///
    /// VM 首发时由 vm_host 捕获当前【杀】（`event_card`）与目标（`event_targets`）。
    /// ABI 464。
    ResolveDctingxian {
        /// Skill owner.
        player: PlayerId,
        /// The resolving Sha carrier (host-captured).
        card: CardId,
        /// Targets of the resolving Sha (host-captured).
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 界营（sp2 韩馥）：结束阶段选择一名其他角色授予「界营」临时技能；其回合内
    /// 使用单目标【杀】时可增加一个目标。
    ///
    /// `target == NONE` 是 VM 首发（打开目标选择）；选择结算后重发授技。ABI 465。
    ResolveHfjieying {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Selected recipient, or `NONE` before the target choice.
        target: PlayerId,
        /// Temp skill definition granted to the recipient.
        grant_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 怒嗔（sp2 新杀SP关羽）：出牌阶段限一次，令一名有手牌的其他角色展示一张手牌，
    /// 然后你选择弃置其展示花色的至少一张手牌/装备牌对其造成等量伤害，或获得其所有
    /// 该花色手牌。
    ///
    /// `target == NONE` 是 VM 首发（打开目标选择）；选择结算后重发。ABI 466。
    ResolveDcnuchen {
        /// Skill owner.
        player: PlayerId,
        /// Chosen target, or `NONE` before the target choice.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 单骑（sp2 新杀SP关羽）：觉醒结算——移除本技、减 1 点体力上限，然后回复并摸牌
    /// 至（新）体力上限，获得【马术】与【怒嗔】。
    ///
    /// 数量按减上限之后的 (maxHp - hp) 顺序求值，与旧 JS 一致。ABI 466。
    ResolveDcdanji {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 制衡（collab 经典孙权）：出牌阶段限一次，弃置任意张手牌/装备牌，摸等量牌；
    /// 若弃置了所有手牌则额外摸一张。
    ///
    /// VM 首发后打开弃牌选择；选择结算后由续体完成弃置与摸牌。ABI 467。
    ResolveDczhiheng {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 高变（sixiang 标王沈）：其他角色结束阶段，若本回合恰好一名角色受到伤害，该
    /// 角色选择一项：视为使用一张【杀】，或重置标王沈【安忍】的摸牌计数。
    ///
    /// `chooser`（受伤害角色）为分支与目标选择的观察者；VM 首发时 `chooser == NONE`。
    /// ABI 468。
    ResolveStdgaobian {
        /// Skill owner.
        player: PlayerId,
        /// The unique damaged player making the choice, or `NONE` before resolution.
        chooser: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 劫刀（sp2 忙牙长）：本回合首次造成伤害且自身受伤时，可选择弃置 X 张手牌/
    /// 装备牌并使本次伤害 +X（X ∈ 1..受伤体力数）。
    ///
    /// 阶段机：选 X → 增幅伤害 → 弃 X 张 he。ABI 469。
    ResolveSpjiedao {
        /// Skill owner and damage source.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 思变（sp2 张温）：摸牌阶段开始时，改为展示牌堆顶 4 张牌，获得其中点数最大与
    /// 最小的牌，其余牌交给一名手牌数最少的角色（无则置入弃牌堆），并以
    /// `draw_phase_fixed_count(0)` 跳过本次摸牌。
    ///
    /// `resolver == NONE` 是 VM 首发；选择结算后由续体完成分配。ABI 470。
    ResolveSibian {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 跳过本回合目标角色的弃牌阶段（SKIP_DISCARD 标记；出牌阶段后由回合机消费）。
    /// ABI 471。
    SkipNextDiscardPhase {
        /// Target whose discard phase is skipped.
        player: PlayerId,
    },
    /// 封印（collab SP伏完）：其他角色准备阶段且其体力不小于拥有者时，拥有者可交给
    /// 其一张【杀】，令其跳过本回合的出牌阶段与弃牌阶段。
    ///
    /// VM 首发时由 vm_host 捕获 `event_player`。ABI 471。
    ResolveSpfengyin {
        /// Skill owner.
        player: PlayerId,
        /// The other player whose turn is being sealed.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 连篇（sp2 苏飞）：出牌阶段使用的牌与上一张使用的牌存在共同目标时，摸一张牌，
    /// 并可将之交给一名共同目标角色（除自己外）。
    ///
    /// `targets` 为当前牌的（宿主捕获的）目标；`NONE` 目标过滤在复合体内完成。ABI 472。
    ResolveXinfuLianpian {
        /// Skill owner.
        player: PlayerId,
        /// Targets of the currently used card (host-captured).
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 拱坚（sp2 朱儁）：本回合使用的【杀】与上一张【杀】存在共同目标时，对每个共同
    /// 目标弃置 1-2 张手牌/装备牌，然后获得其中被弃置的【杀】。
    ///
    /// `targets` 为当前【杀】的（宿主捕获的）目标；阶段机逐目标弃牌后统一夺【杀】。
    /// ABI 473。
    ResolveGongjian {
        /// Skill owner.
        player: PlayerId,
        /// Targets of the currently used Sha (host-captured).
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Reveal cards, choose one category, distribute all matching cards up to a per-recipient cap,
    /// then make recipients with at least the owner's share lose 1 HP. ABI 491.
    RevealChooseCategoryDistributeSelfThresholdHpLoss {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Number of draw-pile cards to reveal.
        reveal_count: u8,
        /// Maximum matching cards each recipient may receive.
        recipient_cap: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose another player with a region card, gain one authoritative-random card, draw one,
    /// and lose 1 HP when the gained and drawn cards have different colors. ABI 492.
    ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve mobile 卫温诸葛直「浮海」's seat-ordered private binary choices. ABI 493.
    ResolveRefuhai {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Discard the selected active-skill hand card, then visibly gain half (rounded up) of the
    /// target's post-cost hand-count advantage. ABI 494.
    ChooseGainTargetHandCeilHalfDifference {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other target.
        target: PlayerId,
        /// Exact selected heart hand card paid as the cost.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the discard and its nested triggers have completed.
        ready_to_choose: bool,
    },
    /// Resolve offline 文钦「矜伐」 after capturing the active skill's selected cost card. ABI 495.
    ResolveGzjinfa {
        /// Skill owner.
        player: PlayerId,
        /// Other target.
        target: PlayerId,
        /// Selected owner hand/equipment card paid as the cost.
        cost_card: CardId,
        /// Equipment card given by the target, if any.
        given_card: CardId,
        /// Resolution stage: 0 pays cost, 1 opens target choice, 2 checks the follow-up Sha.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve offline 杨秋「齐锋」 using the owner's current-turn lost-card count. ABI 496.
    ResolveWnQifeng {
        /// Skill owner and virtual Sha user.
        player: PlayerId,
        /// Player whose turn is ending.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 勤国（sp2 吕岱）：使用装备牌后，可视为对一名其他角色使用一张【杀】。
    ///
    /// `card` 为本次使用的（宿主捕获的）装备牌。ABI 474。
    ResolveXinfuQinguo {
        /// Skill owner.
        player: PlayerId,
        /// The equipment card just used (host-captured).
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 蛮智（huicui 孟优）：准备/结束阶段二选一：①令一名其他角色交给你至多两张牌，
    /// 其视为使用一张无距离限制的【杀】；②你获得一名其他角色区域内的至多两张牌，
    /// 交给其等量牌后摸一张。每分支每回合一次（技能 mark 记录）。ABI 475。
    ResolveDcmanzhi {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 心星乱（sp2 樊稠）：出牌阶段使用牌后每回合一次，三选一：①观看牌堆中两张
    /// 点数为 6 的牌并获得其中一张（无则摸六张）；②获得一名其他角色装备/判定区的
    /// 一张点数为 6 的牌；③令一名其他角色弃置一张点数为 6 的手牌或交给你一张牌。
    /// ABI 476。
    ResolveXinxingluan {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 溃阵（huicui 公孙修）：出牌阶段限一次，选择一名手牌数不小于你或体力值不小于
    /// 你的其他角色，其视为对你使用一张【决斗】；若你因此受到伤害，你获得其所有
    /// 手牌中的【杀】，否则其失去 1 点体力。ABI 477。
    ResolveDckuizhen {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 溃阵结算（huicui 公孙修）：在 DuelResolved 事件中依据决斗结果执行——拥有者
    /// 落败（受伤）则获得决斗发起者所有手牌【杀】，获胜则发起者失去 1 点体力。
    /// ABI 477（第二阶段）。
    ResolveDckuizhenResolve {
        /// Skill owner.
        player: PlayerId,
        /// Duel winner (host-captured `event_player`).
        winner: PlayerId,
        /// Duel loser (host-captured `event_source`).
        loser: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 定州（diy 蔡瑁）：出牌阶段限一次，选择一名其他角色，随机获得其一张手牌/
    /// 装备/判定区牌，摸一张牌，若花色颜色不同则失去 1 点体力。ABI 478。
    ResolveNsdingzhou {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 揽娇（sxrm 嗔曹操）：出牌阶段限一次（每目标每回合一次），循环查看目标两张
    /// 手牌，目标可失去 1 点体力并摸一张令拥有者重新选择；停止后拥有者获得最后
    /// 查看的两张牌。ABI 479。
    ResolveSxrmlanjiao {
        /// Skill owner.
        player: PlayerId,
        /// Chosen target, or `NONE` before the target choice.
        target: PlayerId,
        /// Loop stage: 0 = target choice, 1 = view + rechoose choice.
        stage: u8,
        /// Currently viewed cards (carried across the rechoose loop).
        view: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 宽释（huicui 阚泽）：结束阶段选择一名其他角色标记；其后该角色于本回合内第
    /// 二次受到伤害后回复 1 点体力（每回合一次）。ABI 480。
    ResolveRekuanshi {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 魔狱（huicui 裴元绍）：出牌阶段限一次（每目标每回合一次），获得目标 1（或 2）
    /// 张手牌/装备/判定牌，其视为对你使用一张无距离限制的【杀】；若你因此受伤，
    /// 本回合禁用本技，否则下次多拿一张。ABI 481。
    ResolveDcmoyu {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 袭阵（huicui 高览）：出牌阶段开始时，选择一名其他角色，视为对其使用一张
    /// 【杀】或【决斗】。ABI 482。
    ResolveXizhen {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 虚泄（huicui 邢道荣）：出牌阶段开始时失去 1 点体力上限；对距离 ≤1 的相邻
    /// 角色二选一：弃各一张牌，或各摸一张。ABI 483。
    ResolveXuxie {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 驰应（huicui 高翔）：出牌阶段限一次，选择一名体力不大于你的其他角色；其循环
    /// 选择攻击范围内一名未选过的角色并弃置其一张牌，停止后其获得所弃的全体基本牌。
    /// ABI 484。
    ResolveDcchiying {
        /// Skill owner.
        player: PlayerId,
        /// Chosen target (the discarding player), or `NONE` before the target choice.
        target: PlayerId,
        /// Loop stage: 1 = target choice, 2 = pick a player in range, 3 = pick a card.
        stage: u8,
        /// Accumulated discarded card ids.
        accum: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 驰应（江山如故高翔）：目标攻击范围内除技能拥有者外的角色依座次各弃一张牌；
    /// 若其中基本牌数不大于目标体力，目标获得仍在弃牌堆的这些基本牌。ABI 501。
    ResolveJsrgchiying {
        /// Skill owner.
        player: PlayerId,
        /// Active-skill target whose range determines the forced discarders.
        target: PlayerId,
        /// Whether the fixed seat-ordered discarder list has already been captured.
        started: bool,
        /// Remaining forced discarders, captured before the first discard.
        remaining: Vec<PlayerId>,
        /// Cards discarded by this resolution.
        accum: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 匡扶（huicui 潘凤）：出牌阶段限一次，选择一名角色（可为自身）并弃置其一件
    /// 装备，视为使用一张无距离限制的【杀】；若弃自身装备且此杀造成伤害则摸两张，
    /// 若弃他人装备且此杀未造成伤害则弃两张手牌。ABI 485。
    ResolveXinkuangfu {
        /// Skill owner.
        player: PlayerId,
        /// Chosen equipment target, or `NONE` before the target choice.
        target: PlayerId,
        /// The Sha target (after the equipment discard).
        sha_target: PlayerId,
        /// Resolution stage: 1 = target choice, 2 = equipment pick, 3 = Sha target.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 相术（huicui 张闿）：他人出牌阶段开始（其手牌数 ≥ 体力值）时选择秘密数字
    /// 0-5；其出牌阶段结束时按 |其手牌数 - 数字| 结算：≤1 获得其一张牌，==0 对其
    /// 造成 1 点伤害。ABI 486。
    ResolveDcxiangshu {
        /// Skill owner.
        player: PlayerId,
        /// The other player whose play phase is being observed.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 相术结算（第二阶段）：按数字差结算获得/伤害。
    ResolveDcxiangshuLottery {
        /// Skill owner.
        player: PlayerId,
        /// The observed player.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 虚泄结算：出牌阶段结束时若存在体力上限大于你的角色，恢复 1 点体力上限。
    /// ABI 483（第二阶段）。
    ResolveXuxieAdd {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 宽释结算：被标记角色受伤后，若为本回合第 2 次受伤则回复 1 点体力。
    /// ABI 480（第二阶段）。
    ResolveRekuanshiResolve {
        /// Skill owner.
        player: PlayerId,
        /// The damaged (watched) player.
        damaged: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 狭威/穷技（TW）：维护“威”牌堆、妄行代价与空堆伤害增量。
    ResolveTwxiawei {
        /// Skill owner.
        player: PlayerId,
        /// Temporary 妄行 settlement helper definition.
        wangxing_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 挽危/约俭（TW）：处理最低体力伤害保护、回合末牌堆操作与约俭置顶置底收益。
    ResolveTwbianfuren {
        /// Skill owner.
        player: PlayerId,
        /// Current damage target, or NONE outside 挽危伤害结算.
        target: PlayerId,
        /// 挽危临时技能或约俭手牌上限技能定义.
        helper_skill_def: u32,
        /// Zero handles 挽危 damage; one handles its end-phase effect; two handles 约俭.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected physical cards.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 生息/宽济（TW）：获得调剂盐梅/智囊，或转赠非使用进入弃牌堆的牌。
    ResolveTwfeiyi {
        /// Skill owner.
        player: PlayerId,
        /// Zero prepare Shengxi; one end Shengxi; two Kuanji.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected card.
        card: CardId,
        /// Physical cards carried by the trigger event.
        event_cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 睦约/察异（TW）：声明牌名获牌及察异的延迟分支切换。
    ResolveTwchenzhen {
        /// Skill owner or helper owner.
        player: PlayerId,
        /// Current event player when relevant.
        target: PlayerId,
        /// 睦约免弃技能或察异弃牌技能定义.
        helper_skill_def: u32,
        /// 察异延迟记录技能定义.
        record_skill_def: u32,
        /// Zero Muyue; one Chayi setup; two use-card discard; three delayed switch.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Declared physical card kind tag.
        card_kind: u8,
        /// Selected physical card.
        card: CardId,
        /// Branch/state byte.
        branch: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 前驱乐/倾袭（TW）：伤害后补装备，或结算首杀目标的装备选择。
    ResolveTwqianju {
        /// Skill owner.
        player: PlayerId,
        /// Damaged player.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 倾袭（TW）：首杀目标选择摸牌直击或双方弃装备并加伤。
    ResolveTwqingxi {
        /// Sha user and skill owner.
        player: PlayerId,
        /// Current Sha target.
        target: PlayerId,
        /// Current physical Sha.
        card: CardId,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Number of target equipment cards discarded by the branch.
        count: u8,
        /// Selected source equipment cards.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 躁厉（TW）：弃牌摸牌并按原装备槽补入装备。
    ResolveTwzaoli {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected discard cards.
        cards: Vec<CardId>,
        /// Distinct equipment-slot tags to refill.
        slots: Vec<u8>,
        /// Number of cards discarded by the skill.
        count: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 征建/众斥（TW）：选择惩罚条件、收牌切换并在累计后觉醒。
    ResolveTwzhengjian {
        /// Skill owner.
        player: PlayerId,
        /// Current Play-phase owner.
        target: PlayerId,
        /// 众斥伤害加值技能定义.
        helper_skill_def: u32,
        /// Zero setup; one phase-end settlement.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected physical card.
        card: CardId,
        /// Choice bitfield.
        branch: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 震袭/扬师（TW）：弃手牌、移动场上牌，或按攻击范围成长/获得【杀】。
    ResolveTwtianyu {
        /// Skill owner.
        player: PlayerId,
        /// Current Sha target for 震袭; selected move destination in the final stage.
        target: PlayerId,
        /// 扬师 attack-range mark skill definition.
        helper_skill_def: u32,
        /// Zero for 震袭; one for 扬师.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected field card.
        card: CardId,
        /// 震袭 branch bitfield: discard = 1, move = 2.
        branch: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 锋捍/从击（TW）：按牌目标数选择摸牌，或转赠事件中的红色弃牌。
    ResolveTwwujing {
        /// Skill owner.
        player: PlayerId,
        /// Declared target count for 锋捍.
        target_count: u8,
        /// Zero for 锋捍; one for 从击.
        action: u8,
        /// Cards carried by the triggering movement event.
        event_cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 存嗣：拥有者翻面，从牌堆/弃牌堆取一张【杀】交给目标。
    ///
    /// Appended for serialized ordinal stability.
    ResolveSpcunsi {
        /// Skill owner.
        player: PlayerId,
        /// Recipient of the Sha.
        target: PlayerId,
    },
    /// 商和（TW）：濒死时其他角色依次交牌；无人交【酒】则回复至一点体力。
    ResolveTwshanghe {
        /// Dying skill owner.
        player: PlayerId,
        /// Zero initializes seat order; one continues an existing queue.
        stage: u8,
        /// Remaining contributors in authoritative seat order.
        remaining: Vec<PlayerId>,
        /// Whether any contributed card was logically Jiu for the dying owner.
        got_jiu: bool,
        /// Originating limited skill instance.
        skill: SkillId,
    },
    /// 开济/慑叛（TW 王昶）的目标选择、置顶与当前目标排除。
    ResolveTwwangchang {
        /// Skill owner.
        player: PlayerId,
        /// Card user for 慑叛; NONE for 开济.
        target: PlayerId,
        /// Triggering physical/virtual card carrier.
        card: CardId,
        /// Selected target-region card for 慑叛置顶.
        selected_card: CardId,
        /// Zero for 开济; one for 慑叛.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected 开济 draw targets.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 辅纂/宠齐/非服（TW 曹肇）的转换技切换、全场授予与单目标【杀】赠牌。
    ResolveTwcaozhao {
        /// Skill owner.
        player: PlayerId,
        /// Event target or selected player.
        target: PlayerId,
        /// Event source for 非服.
        source: PlayerId,
        /// Triggering or selected physical card.
        card: CardId,
        /// Selected conversion skill definition.
        selected_skill_def: u32,
        /// 辅纂 definition granted by 宠齐.
        twfuzuan_skill_def: u32,
        /// 非服 definition granted by 宠齐.
        twfeifu_skill_def: u32,
        /// Zero 辅纂; one 宠齐; two 非服.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 精策/御章（TW 郭淮）的摸牌蓄策、受伤分支与跳阶段结算。
    ResolveTwguohuai {
        /// Skill owner.
        player: PlayerId,
        /// Trigger event player.
        target: PlayerId,
        /// Damage source.
        source: PlayerId,
        /// Triggering card carrier.
        card: CardId,
        /// 御章 mark definition.
        mark_def: u32,
        /// Temporary no-card helper skill definition.
        helper_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 疠火/醇醪（TW 程普）的改火代价、置醇、增伤与救援结算。
    ResolveTwchengpu {
        /// Skill owner.
        player: PlayerId,
        /// Event or selected target.
        target: PlayerId,
        /// Card user or damage source.
        source: PlayerId,
        /// Triggering Sha carrier.
        card: CardId,
        /// 醇醪 skill definition and pile key.
        chunlao_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected field/hand/equipment card.
        selected_card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 孝廉（TW 曹昂）的单目标杀替换与“马”牌距离结算。
    ResolveTwcaoang {
        /// Skill owner.
        player: PlayerId,
        /// Original Sha target or damage target.
        target: PlayerId,
        /// Sha user or damage source.
        source: PlayerId,
        /// Triggering Sha carrier.
        card: CardId,
        /// Distance helper skill and pile definition.
        helper_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 挫锐/裂围（TW 牛金）的限定补牌、判定区废除与击杀奖励。
    ResolveTwniujin {
        /// Skill owner.
        player: PlayerId,
        /// 挫锐 limited skill definition.
        limited_skill_def: SkillDefId,
        /// Zero resolves 挫锐; one resolves 裂围.
        action: u8,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 谋溃（TW 伏完）的摸牌、弃目标牌与背水结算。
    ResolveTwmoukui {
        /// Sha user and skill owner.
        player: PlayerId,
        /// Current Sha target.
        target: PlayerId,
        /// Exact Sha carrier.
        card: CardId,
        /// Engine-owned stage; zero opens the branch choice.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 抚蛮（TW 马忠）的实体赠牌与技能实例追踪。
    ResolveTwfuman {
        /// Skill owner and card giver.
        player: PlayerId,
        /// Selected other recipient.
        target: PlayerId,
        /// Selected physical hand card.
        cards: Vec<CardId>,
        /// Zero gives selected cards; one clears the event card; two clears moved event cards.
        action: u8,
        /// Engine-owned stage; zero performs the gift.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 催进（TW 乐就）的精确【杀】结算追踪。
    ResolveCuijin {
        /// Skill owner.
        player: PlayerId,
        /// Exact Sha user.
        source: PlayerId,
        /// Exact Sha carrier.
        card: CardId,
        /// Zero arms tracking; one clears it after resolution.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 清口（TW 冯习）的精确虚拟【决斗】结算。
    ResolveTwqingkou {
        /// Skill owner and Duel initiator.
        player: PlayerId,
        /// Duel damage source on completion; ignored while starting the Duel.
        winner: PlayerId,
        /// Exact materialized virtual Duel card; ignored while starting the Duel.
        card: CardId,
        /// Zero starts and tracks the virtual Duel; one applies its post-resolution reward.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 陈勋（幻刘封）的逐轮虚拟【决斗】链。
    ResolveTwchenxun {
        /// Skill owner and Duel initiator.
        player: PlayerId,
        /// Selected Duel target; none while opening a target choice.
        target: PlayerId,
        /// Exact materialized Duel card at post-resolution boundaries.
        card: CardId,
        /// Zero starts a round; one rewards and continues; two penalizes; three uses the
        /// selected target; four opens the next optional target choice.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 焚险（幻黄盖）的目标分支与实体牌转化结算。
    ResolveTwfenxian {
        /// Skill owner, except action four where this is the Duel user.
        player: PlayerId,
        /// Selected skill target, except action four where this is the Duel target.
        target: PlayerId,
        /// Selected field card for the Duel branch.
        card: CardId,
        /// Zero opens branches; one uses Huogong; two opens field cards; three opens Duel
        /// targets; four uses the selected field card as Duel.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Set or clear one persistent content flag on the originating skill instance.
    SetSkillContentFlag {
        /// Skill owner.
        player: PlayerId,
        /// Flag bit in 0..64.
        bit: u8,
        /// New flag value.
        value: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Give one selected non-equipment hand card, let the recipient optionally use it, and draw
    /// one for the owner after a successful use. ABI 497 (`yuanlve`).
    GiveSelectedThenTargetMayUseDrawOwner {
        /// Skill owner and giver.
        player: PlayerId,
        /// Other living recipient.
        target: PlayerId,
        /// Exact selected card.
        card: CardId,
        /// Selected targets for the recipient's use.
        targets: Vec<PlayerId>,
        /// Resolution stage: 0 gift, 1 offer use, 2 commit use.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Declare one logical card category; the damage target gives one matching hand/equipment
    /// card or the bound damage is increased by one. ABI 498 (`jytongbei`).
    ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
        /// Damage source and skill owner.
        player: PlayerId,
        /// Damage target.
        target: PlayerId,
        /// Bound pre-resolution damage id.
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 应图（huicui 陈珪）：牌移动后，若上家或下家因此获得手牌（非其摸牌阶段），
    /// 获得其一张 he；将一张 he 交给其对位角色，若所给为装备牌则其使用之。ABI 487。
    /// Appended to keep existing serialized enum ordinals stable.
    ResolveDcyingtu {
        /// Skill owner.
        player: PlayerId,
        /// The neighbor who gained cards (target of the gain).
        target: PlayerId,
        /// Resolution stage: 1 = gain card pick, 2 = give card pick.
        stage: u8,
        /// Moved card ids from the triggering event (host-captured).
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose another living player other than the damage source, gain one deterministic-random
    /// gainable hand/equipment card, reveal it, and use it when it is an equipment card. ABI 499.
    ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
        /// Damaged skill owner.
        player: PlayerId,
        /// Damage source excluded from the target choice; may be `NONE`.
        source: PlayerId,
        /// Randomly gained equipment awaiting ordinary use.
        card: CardId,
        /// Resolution stage: 0 = choose target; 1 = use gained equipment if still legal.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve Mobile Danshou from the exact number of cards `target` used on `player` this turn:
    /// draw one at zero, otherwise optionally discard that many cards to deal one damage. ABI 500.
    ResolveMobiledanshou {
        /// Skill owner.
        player: PlayerId,
        /// Player whose End phase is beginning.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Emit the shared end boundary for one fully resolved logical card use. Engine-only; ABI 503.
    EmitCardResolved {
        /// Card user.
        source: PlayerId,
        /// Physical carrier or materialized virtual card.
        card: CardId,
        /// Effective card-kind tag captured at use time.
        kind: u8,
        /// Complete declared targets.
        targets: Vec<PlayerId>,
        /// Targets damaged by this exact card use.
        damaged_targets: Vec<PlayerId>,
        /// Physical materials consumed by this logical use.
        #[serde(default)]
        source_cards: Vec<CardId>,
        /// Whether all physical materials originated in the user's hand.
        #[serde(default)]
        all_sources_were_hand: bool,
    },
    /// 清蹈：after another player's damage card resolves against the owner, choose the branch
    /// determined by whether this owner was damaged. ABI 503.
    ResolveMbqingdao {
        /// Skill owner.
        player: PlayerId,
        /// Card user.
        source: PlayerId,
        /// Resolved card carrier.
        card: CardId,
        /// Whether this owner received damage from the exact use.
        damaged: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 修耕：choose up to two players and record their hand counts, or settle one recorded
    /// player's next draw phase. ABI 503.
    ResolveMbxiugeng {
        /// Skill owner for stage 0; recorded player for stage 1.
        player: PlayerId,
        /// Temporary record helper skill definition.
        helper_def: SkillDefId,
        /// Persistent hand-limit mark skill definition.
        hand_limit_def: SkillDefId,
        /// 0 = choose and record targets; 1 = settle the helper at draw-phase start.
        stage: u8,
        /// Selected players for stage 2; empty in VM-authored requests.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 陈赦：sequentially discard from owner, dying player, and damage source; three cards of one
    /// suit recover the dying player to max HP and remove this skill. ABI 503.
    ResolveMbchenshe {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player currently dying.
        dying: PlayerId,
        /// Damage source, or `NONE`.
        source: PlayerId,
        /// 0 = capture roles; positive values resume after one or more discards.
        stage: u8,
        /// Remaining role players in original order (duplicates are intentional).
        roles: Vec<PlayerId>,
        /// Physical suit tags of cards discarded so far.
        suits: Vec<u8>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 权宠：discard all hand/equipment cards, queue an extra turn, and arm its HP penalty.
    /// ABI 504.
    ResolveMbquanchong {
        /// Skill owner and extra-turn recipient.
        player: PlayerId,
        /// One-shot phase-begin HP-loss helper definition.
        helper_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 任行：choose its draw or target-discard branch. ABI 504.
    ResolveMbrenxing {
        /// Skill owner.
        player: PlayerId,
        /// Zero chooses a branch; one discards from `target`.
        stage: u8,
        /// Selected discard target for stage one.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 鸱张：discard hand cards and lock matching response colors for the exact card. ABI 505.
    ResolveMbchizhang {
        /// Skill owner.
        player: PlayerId,
        /// Current damage-card carrier.
        card: CardId,
        /// Zero starts selection; one commits selected cards.
        stage: u8,
        /// Selected hand cards.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 断鞅：store lost Sha, use pile cards at phase end, or recast after tracked damage. ABI 505.
    ResolveMbduanyang {
        /// Skill owner.
        player: PlayerId,
        /// Damage target for mode two, otherwise event player.
        target: PlayerId,
        /// Current card carrier.
        card: CardId,
        /// Zero stores; one uses pile cards; two recasts after damage.
        mode: u8,
        /// Internal continuation stage.
        stage: u8,
        /// Event or selected physical cards.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 势陆郁生「润微」. ABI 506.
    ResolveMbrunwei {
        /// Skill owner.
        player: PlayerId,
        /// Selected recipient, or none before target selection.
        target: PlayerId,
        /// Resumable stage.
        stage: u8,
        /// Revealed count or selected color mask.
        value: u32,
        /// Snapshotted revealed or moved cards.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 势陆郁生「霜怀」. ABI 506.
    ResolveMbshuanghuai {
        /// Skill owner.
        player: PlayerId,
        /// Pending damage target.
        target: PlayerId,
        /// Resumable stage.
        stage: u8,
        /// Selected prevention or Tao branch.
        choice: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 势孙峻「凶图」. ABI 507.
    ResolveMbxiongtu {
        /// Skill owner.
        player: PlayerId,
        /// Selected target.
        target: PlayerId,
        /// Resumable stage.
        stage: u8,
        /// Revealed target hand card.
        card: CardId,
        /// Selected owner cards.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 势田丰「刚鲠」. ABI 508.
    ResolveMbganggeng {
        /// Skill owner.
        player: PlayerId,
        /// Selected or remembered target.
        target: PlayerId,
        /// Zero gives selected cards; one settles at turn end.
        stage: u8,
        /// Selected hand cards.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 势田丰「死谏」. ABI 508.
    ResolveMbsijian {
        /// Skill owner.
        player: PlayerId,
        /// Selected discard-debuff target.
        target: PlayerId,
        /// Resumable stage.
        stage: u8,
        /// Selected branch: discard, draw, or both.
        choice: u8,
        /// One-shot discard helper definition.
        helper_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 势杨弘「间计」. ABI 509.
    ResolveMbjianji {
        /// Skill owner.
        player: PlayerId,
        /// First comparison participant or winner.
        source: PlayerId,
        /// Second comparison participant or loser.
        target: PlayerId,
        /// Resumable stage.
        stage: u8,
        /// Secretly selected owner hand card.
        card: CardId,
        /// Comparison cards selected by the two participants.
        cards: Vec<CardId>,
        /// Selected comparison participants or later damage recipients.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 势杨弘「远谟」. ABI 509.
    ResolveMbyuanmo {
        /// Skill owner.
        player: PlayerId,
        /// Owner of the moved field card.
        target: PlayerId,
        /// Resumable stage.
        stage: u8,
        /// Selected field card.
        card: CardId,
        /// Attack-range population before movement or draw amount.
        value: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 势张燕「飞径」. ABI 510.
    ResolveMbfeijing {
        /// Skill owner and Sha user.
        player: PlayerId,
        /// Original Sha target.
        target: PlayerId,
        /// In-flight Sha.
        card: CardId,
        /// Resumable stage.
        stage: u8,
        /// Chosen seat path or discarded-card owners.
        targets: Vec<PlayerId>,
        /// Cards selected by path roles.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve 势张燕「骁戈」. ABI 510.
    ResolveMbxiaoge {
        /// Skill owner.
        player: PlayerId,
        /// Damage target or original Sha target.
        target: PlayerId,
        /// Bound Sha.
        card: CardId,
        /// Zero prevents extra-target damage; one follows a single-target Sha with Duel.
        stage: u8,
        /// Complete resolved Sha targets.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 双雄（shenhua 颜良文丑）：摸牌阶段可选改为判定，获得判定牌并记录判定颜色，
    /// 本回合非该颜色手牌可当【决斗】使用。ABI 511。
    ResolveShuangxiong {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 直谏（shenhua 张昭张纮）：出牌阶段限一次，将一张手牌区装备牌交给一名可装备
    /// 该牌的其他角色，然后摸一张。ABI 512。
    ResolveZhijian {
        /// Skill owner and giver.
        player: PlayerId,
        /// Exact selected equipment card from hand.
        card: CardId,
        /// Choice barrier stage: 0 choose target, 1 commit give.
        stage: u8,
        /// Chosen recipient.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 固政（shenhua 张昭张纮）：其他角色弃牌阶段结束后，归还其一张某阶段弃置且仍处
    /// 弃牌堆的牌，并可选获得其余弃牌。ABI 513。
    ResolveGuzheng {
        /// Skill owner.
        player: PlayerId,
        /// The player whose discard phase ended.
        event_player: PlayerId,
        /// Choice barrier stage: 0 choose return card, 1 choose take-rest.
        stage: u8,
        /// Snapshot of the discard-phase lost cards (player id + card id pairs).
        cards: Vec<u32>,
        /// Card chosen to return.
        return_card: CardId,
        /// Whether the owner takes the remaining phase discards.
        take_rest: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势庞羲「蓄业」：伤害后摸牌并在新达手牌上限时置顶一张区域牌。ABI 514。
    ResolveMbxuye {
        /// Skill owner.
        player: PlayerId,
        /// Damage target, or the owner for a 狂想-derived invocation.
        target: PlayerId,
        /// Engine-owned resumable stage.
        stage: u8,
        /// Whether the target already tied for maximum hand size before drawing.
        was_max: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势庞羲「狂想」：交换整手、追踪交换牌耗尽并派生蓄业。ABI 514。
    ResolveMbkuangxiang {
        /// Skill owner.
        player: PlayerId,
        /// Exchange target or role whose received cards were exhausted.
        target: PlayerId,
        /// Zero exchanges, one invokes 蓄业, two clears tracking.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势程普「督作」「避憾」复合。ABI 515。
    ResolvePotchengpu {
        /// Skill owner.
        player: PlayerId,
        /// Selected recipient, damage target, or hand-discard role.
        target: PlayerId,
        /// Zero/one resolve 督作; two/three resolve 避憾.
        stage: u8,
        /// Bound damage id for 避憾.
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势臧洪「拒孤」：摸二并按本回合发动次数强制弃牌。ABI 516。
    ResolvePotjugu {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势曹真「伺锋」分配、逐牌消耗与阶段末结算。ABI 517。
    ResolvePotsifeng {
        /// Skill owner.
        player: PlayerId,
        /// Expansion holder or event card user.
        target: PlayerId,
        /// Resolved card for color comparison.
        card: CardId,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势陈群「法恩」「定品」复合。ABI 518。
    ResolvePotchenqun {
        /// Skill owner.
        player: PlayerId,
        /// Current card user, or unused for 定品.
        target: PlayerId,
        /// Zero resolves 法恩; one resolves 定品.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势陈到「往烈」选牌、精确牌结算与阶段清理。ABI 519。
    ResolvePotwanglie {
        /// Skill owner.
        player: PlayerId,
        /// Armed or resolved physical card.
        card: CardId,
        /// Phase-local self-target-only helper definition.
        helper_skill_def: SkillDefId,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势陈到「弘毅」初始/伤害加标记及准备、结束阶段配对结算。ABI 519。
    ResolvePothongyi {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势娄圭「灌沙」全牌替换与本回合手牌上限增益。ABI 520。
    ResolvePotguansha {
        /// Skill owner.
        player: PlayerId,
        /// Temporary hand-limit helper definition.
        helper_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势娄圭「急御」异类获牌、耗尽检测与最多两次重置。ABI 520。
    ResolvePotjiyu {
        /// Skill owner.
        player: PlayerId,
        /// Selected hand card for stage zero.
        card: CardId,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 兵势孙韶「筑墼」同花色弃牌、装备检索使用与收益选择。ABI 521。
    ResolveMbzhuji {
        /// Skill owner.
        player: PlayerId,
        /// Choice value supplied by the engine-owned continuation.
        choice: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 兵势夏侯霸「砺兵」循环增益、伤害牌加伤与重置。ABI 522。
    ResolvePotlibing {
        /// Skill owner.
        player: PlayerId,
        /// Used or resolved physical card.
        card: CardId,
        /// Zero handles card resolution; one handles card use.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 兵势夏侯霸「破袭」逐张强制使用弃置的实体杀。ABI 522。
    ResolvePotpoxi {
        /// Skill owner and Sha user.
        player: PlayerId,
        /// Current movement event cards, or remaining cards after one forced use.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 兵势张任「伏暗」降攻击范围、记花色与回合结束追击。ABI 525。
    ResolvePotfuan {
        /// Skill owner.
        player: PlayerId,
        /// Movement event card.
        card: CardId,
        /// Resolution stage.
        stage: u8,
        /// Helper skill that lowers the owner's attack range.
        range_helper_def: SkillDefId,
        /// 引弦 helper skill definition granted on suit match.
        yinxian_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 兵势张任「引弦」最远目标杀增伤与回合外用牌加攻击范围。ABI 525。
    ResolvePotyinxian {
        /// Skill owner.
        player: PlayerId,
        /// Movement event card.
        card: CardId,
        /// Resolution stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 驭雳（神马超，ABI 409）：mode 0 造成伤害前非雷改雷、已雷则 +1
    /// （prevent + 同源同量同牌雷属性重放）；mode 1 自己受伤前取消并摸伤害点数张。
    ///
    /// Appended for serialized ordinal stability.
    ResolveYuli {
        /// Skill owner.
        player: PlayerId,
        /// Zero = source branch, one = target branch.
        mode: u8,
        /// Current damage source.
        source: PlayerId,
        /// Current damage target.
        target: PlayerId,
        /// Current damage amount.
        amount: u32,
        /// Current damage nature.
        nature: u8,
        /// Current damage card.
        card: CardId,
        /// Current logical card kind.
        card_kind: u8,
        /// Current card provenance flags.
        card_flags: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 霆威（神马超，ABI 410）：mode 0 杀指定首目标后加 4 霆标记并选目标；
    /// mode 1 目标四选一（非锁失效/交装备/该杀对其伤害+1/随机弃牌）或取消横置。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTingwei {
        /// Skill owner.
        player: PlayerId,
        /// Zero = select target stage, one = resolve target option stage.
        mode: u8,
        /// The sha whose targets are considered.
        card: CardId,
        /// The sha's targets at trigger time.
        targets: Vec<PlayerId>,
        /// Chosen target at stage one.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 新名士（shiji 孔融，ABI 411）：受伤后（有谦标记）来源强制弃一张
    /// 手牌/装备/判定区牌；弃红色牌拥有者回复 1，弃牌进入弃牌堆则拥有者获得。
    ///
    /// Appended for serialized ordinal stability.
    ResolveXinmingshi {
        /// Skill owner.
        player: PlayerId,
        /// Damage source.
        source: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 殉义（shiji 王甫赵累，ABI 412）：mode 0 选择“义”目标（获义标记）；
    /// mode 1 伤害事件分流——拥有者/义目标任一造成伤害另一人摸 1、
    /// 任一受伤另一人强制弃 1 张手牌/装备。
    ///
    /// Appended for serialized ordinal stability.
    ResolveXunyi {
        /// Skill owner.
        player: PlayerId,
        /// Zero = select target, one = damage event.
        mode: u8,
        /// Current damage source.
        source: PlayerId,
        /// Current damage target.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 博明（shiji 许靖，ABI 413）：把主动技选中的手牌赠给目标。
    ///
    /// Appended for serialized ordinal stability.
    ResolveBoming {
        /// Skill owner.
        player: PlayerId,
        /// Gift recipient.
        target: PlayerId,
        /// Selected gift card (filled by host from the active selection).
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 恶荐（shiji 许靖，ABI 414）：赠牌后目标若另有同类牌则二选一——
    /// 受 1 点伤害 / 展示手牌并弃所有同类牌。
    ///
    /// Appended for serialized ordinal stability.
    ResolveEjian {
        /// Skill owner.
        player: PlayerId,
        /// Gift recipient.
        target: PlayerId,
        /// The gifted card (from the CardsGiven event).
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 异勇（shiji 王双，ABI 415）：受伤后获得造成伤害的杀实体牌，可反击
    /// （来源无武器则反击杀伤害基数 2）。
    ///
    /// Appended for serialized ordinal stability.
    ResolveYiyong {
        /// Skill owner.
        player: PlayerId,
        /// The Sha's physical card at damage time.
        card: CardId,
        /// Damage source.
        source: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 善械（shiji 王双，ABI 416）：出牌阶段限一次，牌堆有武器则获得，
    /// 无则随机夺一名有武器角色的武器。
    ///
    /// Appended for serialized ordinal stability.
    ResolveShanxie {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 舍医（shiji 蔡贞姬，ABI 417）：其他角色受伤前（每轮一次）交至少
    /// max(1, hp) 张手牌/装备牌防伤。
    ///
    /// Appended for serialized ordinal stability.
    ResolveSheyi {
        /// Skill owner.
        player: PlayerId,
        /// Damage target.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 天荫（shiji 蔡贞姬，ABI 418）：结束阶段获得牌堆顶每个本阶段未用
    /// 类别牌的第一张。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTianyin {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 追锋（shiji 标文鸯，ABI 419）：失 1 点体力并对一名其他角色视为
    /// 使用【决斗】。
    ///
    /// Appended for serialized ordinal stability.
    ResolveDbzhuifeng {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 曲迪（shiji 标文鸯，ABI 420）：单目标杀/决斗指定后三选一——
    /// 获得目标一张手牌 / 弃一张基本牌令此牌伤害+1 / 背水减 1 上限并全执行。
    ///
    /// Appended for serialized ordinal stability.
    ResolveDbquedi {
        /// Skill owner.
        player: PlayerId,
        /// The single target.
        target: PlayerId,
        /// The Sha/Juedou card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 隐据（shiji 辛毗，ABI 421）：目标二选一——对拥有者使用【杀】/
    /// 获得跳过标记。
    ///
    /// Appended for serialized ordinal stability.
    ResolveSpyinju {
        /// Skill owner.
        player: PlayerId,
        /// Chosen target.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 持节（shiji 辛毗，ABI 422）：判定点数 > 6 取消该牌目标。
    ///
    /// Appended for serialized ordinal stability.
    ResolveSpchijie {
        /// Skill owner.
        player: PlayerId,
        /// Card user.
        source: PlayerId,
        /// The card naming the owner.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 上议（shiji 江庆，ABI 423）：展示目标手牌并获得其中一张。
    ///
    /// Appended for serialized ordinal stability.
    ResolveSpshangyi {
        /// Skill owner.
        player: PlayerId,
        /// Hand owner.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 简议（shiji 江庆，ABI 424）：其他角色结束阶段获得弃牌堆中的防具牌。
    ///
    /// Appended for serialized ordinal stability.
    ResolveSpjianyi {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 谏喻（shiji 费祎，ABI 425）：mode 0 授予 2 名目标喻标记；
    /// mode 1 触发时目标摸 1。
    ///
    /// Appended for serialized ordinal stability.
    ResolveFyjianyu {
        /// Skill owner.
        player: PlayerId,
        /// Event source（mode 1）或零（mode 0）。
        source: PlayerId,
        /// Event target（mode 1）或零（mode 0）。
        target: PlayerId,
        /// Selected mark recipients for mode 0.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 流冰（shiji 吴懿，ABI 426）：出牌阶段首次实体杀改为方块。
    ///
    /// Appended for serialized ordinal stability.
    ResolveLiubing {
        /// Skill owner.
        player: PlayerId,
        /// The physical Sha.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 合击（shiji 吴懿，ABI 427）：他人单目标决斗/红杀后对原目标虚拟杀/决斗。
    ///
    /// Appended for serialized ordinal stability.
    ResolveHeji {
        /// Skill owner.
        player: PlayerId,
        /// Original target.
        target: PlayerId,
        /// The triggering card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 制霸（shenhua 孙策，主公技）：其他吴势力角色出牌阶段限一次可与拥有者拼点；
    /// 拥有者未赢时可选择获得两张拼点牌，已觉醒时可先拒绝。ABI 523。
    ResolveZhiba {
        /// Skill owner and lord.
        player: PlayerId,
        /// The Wu-faction player initiating the comparison.
        actor: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 制霸结算（拼点完成后）：拥有者未赢时可选择获得两张拼点牌。ABI 523。
    ResolveZhibaSettle {
        /// Skill owner and lord.
        player: PlayerId,
        /// The Wu-faction comparison initiator.
        actor: PlayerId,
        /// Strict winner of the finished comparison, or NONE for a tie.
        winner: PlayerId,
        /// First participant's comparison card.
        player_card: CardId,
        /// Second participant's comparison card.
        target_card: CardId,
        /// Choice barrier stage: 0 owner accept choice, 1 commit gain.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 从俭（shenhua 张绣）：成为多目标锦囊牌的目标后，交给其中一名其他目标一张
    /// 手牌/装备牌，然后摸一张（所给为装备牌则摸两张）。ABI 524。
    ResolveDrltCongjian {
        /// Skill owner.
        player: PlayerId,
        /// The trick's targets captured at trigger time.
        targets: Vec<PlayerId>,
        /// Card chosen to give.
        card: CardId,
        /// Chosen recipient.
        target: PlayerId,
        /// Choice barrier stage: 0 choose recipient, 1 choose card, 2 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 飞军（shenhua 王平）：出牌阶段限一次弃一张牌，按可用性选“手牌数大于你者交给
    /// 你一张牌”或“装备数大于你者弃置一张装备牌”分支；目标首次被选时置兵略旗标。
    /// ABI 526。
    ResolveNzryFeijun {
        /// Skill owner.
        player: PlayerId,
        /// Chosen target.
        target: PlayerId,
        /// Card the target chose to give or discard.
        card: CardId,
        /// Branch: 0 = hand-give, 1 = equip-discard.
        branch: u8,
        /// Choice barrier stage: 0 branch, 1 target, 2 target card, 3 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 庸肆（shenhua 袁术）：摸牌阶段摸牌数改为势力数；出牌阶段结束时按本回合是否
    /// 造成伤害：未造成伤害摸至体力值，否则授予既有庸肆弃牌阶段状态。ABI 527。
    ResolveDrltYongsi {
        /// Skill owner.
        player: PlayerId,
        /// Zero = draw-phase replacement, one = play-phase-end settlement.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 伪帝（shenhua 袁术，主公技）：弃牌阶段开始把至多 min(需弃置数, 其他群势力
    /// 角色数) 张手牌分别交给等量其他群势力角色。ABI 528。
    ResolveDrltWeidi {
        /// Skill owner.
        player: PlayerId,
        /// Cards chosen to give.
        cards: Vec<CardId>,
        /// Chosen recipients, paired with cards by index.
        targets: Vec<PlayerId>,
        /// Choice barrier stage: 0 cards, 1 targets, 2 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势桓阶「共谋」「正朔」复合。ABI 529。
    ///
    /// Appended for serialized ordinal stability.
    ResolvePothuanjie {
        /// Skill owner.
        player: PlayerId,
        /// Exchange target; NONE while opening the target choice.
        target: PlayerId,
        /// Zero opens 共谋 target choice, one commits 共谋, two resolves 正朔.
        stage: u8,
        /// 奇策 definition temporarily granted to the owner.
        qice_def: SkillDefId,
        /// 看破 definition temporarily granted to the exchange target.
        kanpo_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 烈刃（shenhua 祝融）：以固定受伤害角色为目标启动通用拼点选牌续体；胜者获得
    /// 绑定辅助技能。ABI 531。
    ResolveLieren {
        /// Skill owner and comparing player.
        player: PlayerId,
        /// The damaged player and compare target.
        target: PlayerId,
        /// Helper granted on a strict win.
        win_skill_def: SkillDefId,
        /// Helper granted on a tie or loss.
        lose_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势孙綝「逆固」：依次收取攻击范围内角色可选的一张牌，并记录拒绝人数；
    /// 后续伤害前消耗一次记录使伤害 +1。ABI 532。
    ResolvePotnigu {
        /// Skill owner.
        player: PlayerId,
        /// Current giver during the staged gift sequence.
        target: PlayerId,
        /// Selected gift card, or NONE for a refusal.
        card: CardId,
        /// Stable giver order captured at activation.
        targets: Vec<PlayerId>,
        /// Current giver index.
        index: u8,
        /// Refusal count accumulated so far.
        declined: u8,
        /// Zero starts, one commits one answer, two advances, three buffs damage, four clears.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势孙綝「戮连」：记录手牌来源的完整用牌，结算后横置目标、摸牌并处理
    /// “成势”火焰伤害。ABI 532。
    ResolvePotlulian {
        /// Skill owner.
        player: PlayerId,
        /// Physical carrier of the logical card use.
        card: CardId,
        /// Forced fire-damage target after the choice barrier.
        target: PlayerId,
        /// Zero records card use, one settles, two commits fire damage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势鲁肃「缔盟」：交换两名角色的全部手牌，并结算拥有者的弃牌/摸牌代价选择。
    /// ABI 535。
    ResolvePotdimeng {
        /// Skill owner.
        player: PlayerId,
        /// Exactly two selected players.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势鲁肃「好施」：结束阶段授权其他角色借用拥有者手牌；拥有者下个回合
    /// 开始时清除授权。内部动作 2 结算借牌耗尽手牌后的至三张补牌。ABI 536。
    ResolvePothaoshi {
        /// Skill owner and physical hand owner.
        player: PlayerId,
        /// Authorized borrower for internal refill settlement; none for content actions.
        target: PlayerId,
        /// Zero opens authorization choice, one clears grants, two settles one empty-hand use.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势董昭「妙略」：初始获得两张【瞒天过海】，受伤后选择摸两张
    /// 或获得一张智囊。ABI 539。
    ResolveSpmiaolve {
        /// Skill owner.
        player: PlayerId,
        /// Zero initializes derivation cards; one opens the post-damage choice.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势董昭「迎驾」：记录本回合同名锦囊，回合结束时弃手牌并为目标
    /// 插入摸两张的额外回合。ABI 539。
    ResolveSpyingjia {
        /// Skill owner.
        player: PlayerId,
        /// Zero resets turn history, one records a trick use, two opens settlement.
        action: u8,
        /// Temporary draw helper granted to the extra-turn target.
        helper_def: SkillDefId,
        /// Triggering physical card, if any.
        card: CardId,
        /// Triggering logical card kind.
        card_kind: u8,
        /// Extra-turn target during internal settlement.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Internal continuation for 【瞒天过海】 after a physical gain/give move.
    ResolveMantianguohai {
        /// Card user.
        player: PlayerId,
        /// Zero advances the per-target trick queue; one advances the give-back queue.
        stage: u8,
        /// Remaining independent-trick queue payload or remaining give targets.
        payload: Vec<u32>,
    },
    /// 屯田（shenhua 邓艾）：回合外失去牌后判定；判定牌不为红桃时移入本技能
    /// 牌堆（田）。ABI 533。
    ResolveTuntian {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 凿险（shenhua 邓艾）：准备阶段本技能牌堆至少 3 张时觉醒，减 1 点体力
    /// 上限并授予急袭。ABI 534。
    ResolveZaoxian {
        /// Skill owner.
        player: PlayerId,
        /// 急袭 definition granted on awakening.
        jixi_def: SkillDefId,
        /// 屯田 definition whose skill pile stores the 田 cards.
        tuntian_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 挑衅（shenhua 姜维）：出牌阶段限一次，选择一名攻击范围含持有者的其他角色，
    /// 其选择对持有者使用一张杀，或令持有者弃置其一张牌。ABI 537。
    ResolveTiaoxin {
        /// Skill owner.
        player: PlayerId,
        /// The challenged player; NONE while opening the target choice.
        target: PlayerId,
        /// Choice barrier stage: 0 open target, 1 open sha-or-refuse, 2 open sha source,
        /// 3 play the sha, 4 open discard card, 5 commit the discard.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 志继（shenhua 姜维）：觉醒技，准备阶段无手牌时摸两张牌、回复 1 点体力、
    /// 减 1 点体力上限并获得观星。ABI 538。
    ResolveZhiji {
        /// Skill owner.
        player: PlayerId,
        /// 观星 definition granted on awakening.
        guanxing_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 再起（shenhua 孟获）：摸牌阶段替换——已受伤时弃摸牌并翻亮牌堆顶 X 张
    /// （X=已损失体力+extra），红桃每张回复 1 点体力，其余获得。ABI 540。
    ResolveZaiqi {
        /// Skill owner.
        player: PlayerId,
        /// Extra revealed cards beyond the lost HP count.
        extra: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 求首（shenhua 孟获，主公技）：南蛮结算后目标死亡或受创目标数超过 3 时，
    /// 所有存活蜀/群角色各摸一张牌。ABI 542。
    ResolveTwqiushou {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势邓艾「屯田/凿险」：维护蓄力上限与蓄力标记，分发红桃牌并按消耗阈值
    /// 从弃牌堆获得锦囊。ABI 543。
    ResolvePottuntian {
        /// Skill owner.
        player: PlayerId,
        /// 0 = gain charge, 1 = expand full capacity, 2 = choose spend, 3 = commit spend.
        action: u8,
        /// Number of charge points committed by action 3.
        amount: u8,
        /// Selected recipients for the heart-card distribution.
        targets: Vec<PlayerId>,
        /// 势凿险 definition used for the threshold rewards.
        zaoxian_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势邓艾「急袭」：记录本回合用牌目标，回合结束弃当前角色一张牌后，
    /// 对任意名已记录角色使用无距离限制的虚拟【顺手牵羊】。ABI 543。
    ResolvePotjixi {
        /// Skill owner.
        player: PlayerId,
        /// Event/current turn player depending on action.
        target: PlayerId,
        /// 0 = reset, 1 = record target, 2 = start, 3 = choose targets, 4 = use trick.
        action: u8,
        /// Selected targets for action 4.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势陈矫「策断」：收集目标攻击范围内角色展示的手牌，并使用多牌【杀】。ABI 544。
    ResolvePotceduan {
        /// Skill owner.
        player: PlayerId,
        /// Active target.
        target: PlayerId,
        /// 0 = collect reveals, 1 = draw after the tracked Sha dealt damage.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势太史慈「酣战/战烈/振锋」状态机。ABI 548。
    ResolvePottaishici {
        /// Skill owner.
        player: PlayerId,
        /// Selected active target, or the current event player when unused.
        target: PlayerId,
        /// 0 = turn reset, 1 = collect discarded Sha, 2 = launch 战烈,
        /// 3 = 酣战, 4 = 振锋, 5 = tracked 战烈 draw;
        /// 6 is the internal post-draw 酣战 continuation.
        action: u8,
        /// Event cards supplied by the VM host for action 1.
        cards: Vec<CardId>,
        /// 酣战 definition whose X source may be rewritten.
        hanzhan_def: SkillDefId,
        /// 战烈 definition whose X source and marks are authoritative.
        zhanlie_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 征荣（shenhua 毌丘俭）：出牌阶段外，杀/伤害锦囊首个目标结算时，从一名
    /// 手牌数不小于持有者的目标角色取一张手牌/装备移入本技能牌堆「荣」。ABI 545。
    ResolveZhengrong {
        /// Skill owner.
        player: PlayerId,
        /// Event target candidates from the triggering card use.
        candidates: Vec<PlayerId>,
        /// The chosen target; NONE while opening the target choice.
        target: PlayerId,
        /// Choice barrier stage: 0 open target, 1 open card, 2 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 鸿举（shenhua 毌丘俭）：觉醒技，准备阶段「荣」≥3 时可选交换「荣」与等量
    /// 手牌、减 1 点体力上限并授予清俭。ABI 546。
    ResolveHongju {
        /// Skill owner.
        player: PlayerId,
        /// 清俭 definition granted on awakening.
        qingjian_def: SkillDefId,
        /// 征荣 definition whose pile holds the 荣 cards.
        rong_def: SkillDefId,
        /// Choice barrier stage: 0 open swap-or-skip, 1 open hand selection, 2 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 清俭（shenhua 毌丘俭）：出牌阶段限一次，弃一张手牌获得一张「荣」，
    /// 弃一名其他角色装备/判定区一张牌。ABI 547。
    ResolveQingjian {
        /// Skill owner.
        player: PlayerId,
        /// 征荣 definition whose pile holds the 荣 cards.
        rong_def: SkillDefId,
        /// Choice barrier stage: 0 open 荣 selection, 1 open hand discard, 2 open target,
        /// 3 open ej card, 4 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 雷击（shenhua 张角）：使用/打出闪后选一名其他角色判定，黑桃 2 点雷电伤害，
    /// 梅花持有者回 1 点体力并对目标 1 点雷电伤害。ABI 549。
    ResolveReleiji {
        /// Skill owner.
        player: PlayerId,
        /// The judged target; NONE while opening the target choice.
        target: PlayerId,
        /// Choice barrier stage: 0 open target, 1 judge and apply.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 黄天（shenhua 张角，主公技）：其他群势力角色出牌阶段限一次可将一张闪/闪电
    /// 交给持有者。ABI 551。
    ResolveHuangtian {
        /// Skill owner (lord).
        player: PlayerId,
        /// The giving Qun player.
        actor: PlayerId,
        /// Choice barrier stage: 0 open card choice, 1 commit the give.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势钟会「俟资/挟志/狱难/克昌」状态机。ABI 553。
    ResolveMbzhonghui {
        /// Skill owner.
        player: PlayerId,
        /// Event participant.
        target: PlayerId,
        /// Public action 0..=7; action 8 commits the charge choice.
        action: u8,
        /// Event amount or committed charge count.
        amount: u32,
        /// 俟资 definition.
        mbsizi_def: SkillDefId,
        /// 克昌 definition.
        mbkechang_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 好施（shenhua 鲁肃）：摸牌阶段额外摸两张；摸牌阶段结束时手牌>5 则选手牌数
    /// 最少角色给 floor(手牌/2) 张。ABI 555。
    ResolveHaoshi {
        /// Skill owner.
        player: PlayerId,
        /// Choice barrier stage: 0 draw replacement, 1 open target, 2 open cards, 3 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 缔盟（shenhua 鲁肃）：出牌阶段限一次，选两名其他角色，弃置 X 张牌
    /// （X=两者手牌差）后交换其手牌。ABI 556。
    ResolveDimeng {
        /// Skill owner.
        player: PlayerId,
        /// The two chosen targets; NONE while opening the target choice.
        targets: Vec<PlayerId>,
        /// Choice barrier stage: 0 open targets, 1 open discard, 2 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势辛宪英「诫节」：当前回合角色展示手牌，持有者选择花色并结算对应分支。ABI 554。
    ResolvePotjiejie {
        /// Skill owner.
        player: PlayerId,
        /// Current Play-phase actor.
        actor: PlayerId,
        /// 诫节已用 helper definition.
        used_def: SkillDefId,
        /// Four suit-unlimited helper definitions in spade/heart/club/diamond order.
        suit_defs: [SkillDefId; 4],
        /// 清识 definition used by the unique-maximum bonus.
        qingshi_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势辛宪英「清识」：选择角色，按阵营关系共同摸牌或双方弃牌。ABI 554。
    ResolvePotqingshi {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 巧变（shenhua 张郃）：阶段开始时弃一张手牌并跳过该阶段；摸牌阶段可
    /// 获至多两名其他角色各一张手牌；出牌阶段可移动场上的一张牌。ABI 559。
    ResolveQiaobian {
        /// Skill owner.
        player: PlayerId,
        /// Triggered phase: 0 judge, 1 draw, 2 play, 3 discard.
        phase: u8,
        /// Choice barrier stage (0 discard, 1 benefits/skip, 2+ benefit flow).
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 将当前正在结算的【杀】的指定目标从保存目标中移除（此杀对该目标无效）。
    /// ABI 560。
    ExcludeSavedShaTarget {
        /// The Sha card currently resolving.
        card: CardId,
        /// Target to exclude from the Sha's saved target list.
        target: PlayerId,
    },
    /// 享乐（shenhua 刘禅）：使用者弃一张基本牌或此杀对持有者无效。ABI 560。
    ResolveXiangle {
        /// Skill owner (the Sha target).
        player: PlayerId,
        /// The Sha user.
        source: PlayerId,
        /// The Sha card.
        card: CardId,
        /// Choice barrier stage: 0 open discard, 1 commit/exclude.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 放权（shenhua 刘禅）：跳过本回合出牌阶段并标记结束阶段可授额外回合。
    /// ABI 560。
    ResolveFangquanSkip {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 放权（shenhua 刘禅）：结束阶段弃一张手牌并令一名其他角色进行一个额外回合。
    /// ABI 560。
    ResolveFangquanGrant {
        /// Skill owner.
        player: PlayerId,
        /// Choice barrier stage: 0 open discard, 1 open target, 2 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 若愚（shenhua 刘禅，主公技）：准备阶段体力值为全场最少时觉醒：加上限、
    /// 回复 1 并授予激将。ABI 560。
    ResolveRuoyu {
        /// Skill owner.
        player: PlayerId,
        /// 激将 granted skill definition.
        jijiang_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 罪论（shenhua 诸葛瞻）：结束阶段揭示牌堆顶三张，按条件计数分派或一起
    /// 失去体力。ABI 563。
    ResolveZuilun {
        /// Skill owner.
        player: PlayerId,
        /// Choice barrier stage: 0 reveal/dispatch, 1 pick-to-top, 2 lose-hp target.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 父荫（shenhua 诸葛瞻，锁定技）：其他角色以不小于你手牌数的【杀】指定你为
    /// 目标且本回合首次时，此杀对你无效。ABI 563。
    ResolveFuyin {
        /// Skill owner (the Sha target).
        player: PlayerId,
        /// The Sha user.
        source: PlayerId,
        /// The Sha card.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 回合内禁用指定角色对另一角色使用牌。ABI 571。
    BanUseAgainst {
        /// Banned card user.
        user: PlayerId,
        /// Banned target.
        target: PlayerId,
    },
    /// 拒战（shenhua 严颜，转换技）：阴——成为其他角色【杀】的目标后与其各摸一张
    /// 牌并令其本回合不能对你使用牌；阳——你使用【杀】指定目标后获得其一张牌并
    /// 令你本回合不能对其使用牌。ABI 571。
    ResolveJuzhan {
        /// Skill owner.
        player: PlayerId,
        /// Event source (Sha user / Sha target).
        source: PlayerId,
        /// The Sha card.
        card: CardId,
        /// Declared Sha targets (branch 阳).
        targets: Vec<PlayerId>,
        /// Choice barrier stage: 0 dispatch, 1 gain card.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 奇制（shenhua 王基）：使用牌指定目标后弃置一名非目标角色的一张手牌/装备牌
    /// 并令其摸一张牌。ABI 573。
    ResolveQizhi {
        /// Skill owner.
        player: PlayerId,
        /// Declared card targets (excluded from the discard choice).
        targets: Vec<PlayerId>,
        /// Choice barrier stage: 0 open target, 1 open card, 2 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 进趋（shenhua 王基）：出牌阶段结束时摸两张并将手牌弃置至本回合奇制次数张。
    /// ABI 573。
    ResolveJinqu {
        /// Skill owner.
        player: PlayerId,
        /// 奇制 skill definition whose per-turn count gates the discard.
        qizhi_def: SkillDefId,
        /// Choice barrier stage: 0 open discard, 1 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 荐相（shenhua 蒯良蒯越）：成为其他角色使用牌的目标后令手牌数最少的一名
    /// 角色摸一张牌。ABI 574。
    ResolveJianxiang {
        /// Skill owner.
        player: PlayerId,
        /// Choice barrier stage: 0 open target, 1 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 审时（shenhua 蒯良蒯越，转换技）：阳——出牌阶段限一次交一张牌给手牌最多
    /// 角色并对其造成 1 点伤害，若其因此死亡可令一名角色手牌摸至四张；阴——受伤
    /// 后观看来源手牌交其一张，其回合结束时该牌仍在其手/装备区则你摸至四张。
    /// ABI 574。
    ResolveShenshi {
        /// Skill owner.
        player: PlayerId,
        /// Event source (damage source for 阴; NONE otherwise).
        source: PlayerId,
        /// Choice barrier stage: 0 dispatch, 1 give card, 2 target, 3 death reward.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 明任（shenhua 卢植）：首轮准备阶段摸两张并将一张手牌置于「任」技能牌堆；
    /// 结束阶段可选手牌替换「任」。ABI 576。
    ResolveMingren {
        /// Skill owner.
        player: PlayerId,
        /// Choice barrier stage: 0 place, 1 replace.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 贞良（shenhua 卢植，转换技）：阴——出牌阶段限一次弃一张与「任」同色的牌并
    /// 对攻击范围内角色造成 1 点伤害；阳——回合外使用/打出同色牌后令一名角色摸
    /// 一张牌。ABI 576。
    ResolveZhenliang {
        /// Skill owner.
        player: PlayerId,
        /// Event source (the used/responded card's player for 阳).
        source: PlayerId,
        /// The used/responded card for 阳; NONE otherwise.
        card: CardId,
        /// 明任 skill definition owning the「任」pile.
        mingren_def: SkillDefId,
        /// Choice barrier stage: 0 dispatch, 1 discard, 2 damage target, 3 draw target.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 溃诛（shenhua 孙亮）：弃牌阶段结束后按本回合弃牌数 X 选择摸牌或伤害。ABI 577。
    ResolveKuizhu {
        /// Skill owner.
        player: PlayerId,
        /// Choice barrier stage: 0 dispatch, 1 draw targets, 2 damage targets.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 掣政（shenhua 孙亮）：出牌阶段结束时弃置一名攻击范围不包含你的角色的一张牌。
    /// ABI 577。
    ResolveZhizheng {
        /// Skill owner.
        player: PlayerId,
        /// Choice barrier stage: 0 dispatch, 1 card.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 立军（shenhua 孙亮，主公技）：吴势力角色出牌阶段使用实体【杀】后你获得该牌
    /// 并令其摸一张牌。ABI 577。
    ResolveLijun {
        /// Skill owner (the lord).
        player: PlayerId,
        /// The Wu card user.
        source: PlayerId,
        /// The used Sha card.
        card: CardId,
        /// Choice barrier stage: 0 dispatch.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 良姻（shenhua 周妃）：其他角色失去牌后每回合一次双方各摸一张并可回复；
    /// 获得牌后每回合一次弃一张牌并令一名其他角色弃一张牌。ABI 579。
    ResolveLiangyin {
        /// Skill owner.
        player: PlayerId,
        /// 箜声 skill definition whose pile count gates the recover condition.
        kongsheng_def: SkillDefId,
        /// Choice barrier stage: 0 dispatch, 1 draw target, 2 recover, 3 discard, 4 target.
        stage: u8,
        /// 摸牌分支选定的其他角色（阶段 2 使用）。
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 箜声（shenhua 周妃）：准备阶段将任意张牌置为「箜」；结束阶段非装备箜回手、
    /// 剩余装备箜令一名角色依次使用。ABI 579。
    ResolveKongsheng {
        /// Skill owner.
        player: PlayerId,
        /// Choice barrier stage: 0 place, 2 returns, 3 equip target, 4 use one equip.
        stage: u8,
        /// 装备使用目标（阶段 4 使用）。
        target: PlayerId,
        /// 正在使用的装备（阶段 4；NONE 表示无后续装备）。
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势魏延【忠傲/壮誓/饮战/狂骨】复合状态机。ABI 558。
    ResolvePotweiyan {
        /// Skill owner.
        player: PlayerId,
        /// Event target, or NONE when the stage has no target.
        target: PlayerId,
        /// Bound physical Sha, or NONE.
        card: CardId,
        /// Public action 0..=10; continuation-only stages are engine-owned.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势周瑜【炽沄/焰洄/焚涛/雄姿】复合状态机。ABI 561。
    ResolvePotzhouyu {
        /// Skill owner.
        player: PlayerId,
        /// Event target, or NONE.
        target: PlayerId,
        /// Bound physical card, or NONE.
        card: CardId,
        /// Public action 0..=7; continuation-only stages are engine-owned.
        action: u8,
        /// Engine-bound damage provenance flags.
        #[serde(default)]
        card_flags: u8,
        /// Engine-bound ordered card targets.
        #[serde(default)]
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 压军（shiji 崔琰，ABI 428）：出牌阶段开始，本回合新获手牌时选一张新获牌
    /// 与一名其他角色拼点；赢则可将拼点牌置于牌堆顶，输则添加“戒”标记
    /// （手牌上限减标记数，由纯 DSL 辅助技能表达）。
    ///
    /// Appended for serialized ordinal stability.
    ResolveSpyajun {
        /// Skill owner.
        player: PlayerId,
        /// 戒 hand-limit mark definition.
        less_def: SkillDefId,
        /// Owner's chosen comparison card.
        card: CardId,
        /// Chosen comparison target.
        target: PlayerId,
        /// Target's chosen comparison card.
        target_card: CardId,
        /// Continuation stage: 0 pick card, 1 pick target, 2 target picks
        /// compare card, 3 commit + win/lose, 4 optional place-on-top.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 尊帝（shiji 崔琰，ABI 429）：出牌阶段限一次，弃置一张手牌交给一名其他角色，
    /// 然后判定：黑色其摸三张，红色其移动一张场上牌。
    ///
    /// Appended for serialized ordinal stability.
    ResolveSpzundi {
        /// Skill owner.
        player: PlayerId,
        /// Judge beneficiary.
        target: PlayerId,
        /// Chosen movable field card.
        card: CardId,
        /// Chosen move destination.
        destination: PlayerId,
        /// Continuation stage: 0 judge + pick card, 1 pick destination, 2 commit move.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 佯解（shiji 朱儁，ABI 430）：出牌阶段限一次与一名其他角色拼点，
    /// **点数小者胜**；拼点失败时可选一名其他角色视为对拼点目标使用火【杀】。
    ///
    /// Appended for serialized ordinal stability.
    ResolveXinyangjie {
        /// Skill owner.
        player: PlayerId,
        /// Comparison target.
        target: PlayerId,
        /// Owner's chosen comparison card.
        card: CardId,
        /// Target's chosen comparison card.
        target_card: CardId,
        /// Chosen fire-Sha user.
        source: PlayerId,
        /// Continuation stage: 0 pick owner card, 1 target picks card,
        /// 2 commit smaller-wins + fire-sha source choice, 3 commit fire sha.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 清雨（shiji 糜夫人，ABI 431）：使命技——受伤前可弃 2 张 he 牌防伤
    /// （action 0）；准备阶段满血且手牌 < 轮数时使命成功获【选存】（action 1）；
    /// 濒死时使命失败减 1 点体力上限并选一人获【勇决】（action 2 开选人、
    /// action 3 授技）。
    ///
    /// Appended for serialized ordinal stability.
    ResolveQingyu {
        /// Skill owner.
        player: PlayerId,
        /// Public action 0..=3.
        action: u8,
        /// 选存 definition granted on mission success.
        xuancun_def: SkillDefId,
        /// 勇决 definition granted on mission failure.
        mbyongjue_def: SkillDefId,
        /// Chosen failure-gift target.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 整肃（核心库通用系统，ABI 432）：三个整肃条件之一的状态跟踪与弃牌阶段
    /// 判定——mode 0 垒金（出牌阶段点数严格递增且不少于三张，card_used 逐张
    /// 跟踪）/ mode 1 变阵（出牌阶段同花色且不少于两张）/ mode 2 明志（弃牌
    /// 阶段花色全异且不少于两张）。判定成功时授予者（状态段记录）+ 自己各自
    /// 摸两张或回复一点体力，随后移除临时整肃技能。
    ///
    /// Appended for serialized ordinal stability.
    ResolveZhengsu {
        /// Skill owner and tracker.
        player: PlayerId,
        /// Mode 0 垒金 / 1 变阵 / 2 明志.
        mode: u8,
        /// Stage 0 track (card_used, mode 0 only), 1 judge at discard-phase end.
        stage: u8,
        /// Triggering card (mode 0 stage 0).
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 侯封 / 燕集（shiji 朱儁 / 吕范，ABI 433）：出牌阶段开始选择整肃类型授予
    /// 目标（mode 0 侯封：其他角色，记录授予者；mode 1 燕集：自己）。
    ///
    /// Appended for serialized ordinal stability.
    ResolveHoufeng {
        /// Skill owner.
        player: PlayerId,
        /// Grant target (self for 燕集).
        target: PlayerId,
        /// Mode 0 侯封 / 1 燕集.
        mode: u8,
        /// Stage 0 choose type, 1 commit grant.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 点才（shiji 吕范，ABI 434）：其他角色出牌阶段结束时，若其本回合失去牌数
    /// 不小于其体力值且手牌数小于体力上限，其摸至体力上限。
    ///
    /// Appended for serialized ordinal stability.
    ResolveMbdiancai {
        /// Skill owner.
        player: PlayerId,
        /// The play-phase actor whose End just fired.
        actor: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 战海（shiji 周处，ABI 435）：职责技——出牌阶段限一次摸一张并与一名其他
    /// 角色拼点（装备数 < 4 时拼点牌点数 + (4 - 装备数)，上限 13）；赢则展示
    /// 目标手牌并按类别从牌堆各取一张获得，输且拼点牌点数 < 7 使命失败；装备
    /// 后装备数 > 2 使命成功：回复至满、以【彰名】替换【乡害】。
    ///
    /// Appended for serialized ordinal stability.
    ResolveRechuhai {
        /// Skill owner.
        player: PlayerId,
        /// Comparison target.
        target: PlayerId,
        /// Owner's chosen comparison card.
        card: CardId,
        /// Target's chosen comparison card.
        target_card: CardId,
        /// Continuation stage: 0 draw + pick owner card, 1 target picks card,
        /// 2 commit compare + mission, 3 equip-after mission success check.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 凶竖（yingbian 贾充，ABI 436）：其他角色出牌阶段开始时弃置 X 张牌
    /// （X=凶竖标记数，每轮递增）并展示其一张手牌，预测其是否会使用该牌；
    /// 该出牌阶段结束时预测对则其受 1 点伤害，错则获得该牌（目标区域或弃牌堆）。
    ///
    /// Appended for serialized ordinal stability.
    ResolveXiongshu {
        /// Skill owner.
        player: PlayerId,
        /// Play-phase actor / prediction target.
        target: PlayerId,
        /// Chosen shown card.
        card: CardId,
        /// Prediction 0 会使用 / 1 不会使用.
        action: u8,
        /// Continuation stage 0..=3.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 奸回（yingbian 贾充，ABI 437）：伤害结算后——作为伤害来源且受伤者与上次
    /// 相同则摸一张；受伤且来源与上次相同（连续同来源）则来源强制弃一张手牌
    /// 或装备（blob "JH1" 记录上次来源/受伤者）。
    ///
    /// Appended for serialized ordinal stability.
    ResolveJianhui {
        /// Skill owner.
        player: PlayerId,
        /// Damage source.
        source: PlayerId,
        /// Damage target.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 弃哀（shiji 王粲，ABI 439）：出牌阶段限一次将一张非基本牌交给一名其他角色
    /// ——满血则摸两张，已受伤则目标二选一（令拥有者回复 1 点体力 / 摸两张）。
    ///
    /// Appended for serialized ordinal stability.
    ResolveSpqiai {
        /// Skill owner.
        player: PlayerId,
        /// Gift recipient.
        target: PlayerId,
        /// Selected non-basic card (filled by host from the active selection).
        card: CardId,
        /// Choice stage 0 open, 1 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 擅袭（shiji 王粲，ABI 440）：出牌阶段开始选一名无“檄”标记的角色获得
    /// “檄”标记（全局唯一，同时清除他人标记）。
    ///
    /// Appended for serialized ordinal stability.
    ResolveSpshanxi {
        /// Skill owner.
        player: PlayerId,
        /// Chosen mark recipient.
        target: PlayerId,
        /// Stage 0 choose target, 1 commit mark.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 索命（shiji 王粲，ABI 441）：其他角色回复体力后（其有檄标记且体力 > 0）
    /// 其交给你 2 张牌或失去 1 点体力（可交不足时直接失体）。
    ///
    /// Appended for serialized ordinal stability.
    ResolveSpshanxiSuoming {
        /// Skill owner.
        player: PlayerId,
        /// The recovering actor with the 檄 mark.
        actor: PlayerId,
        /// Stage 0 open give choice, 1 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 通夺（shiji 刘巴，ABI 442）：成为其他角色单目标牌目标时（每回合一次）
    /// 选一名有牌角色强制重铸一张牌。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTongduo {
        /// Skill owner.
        player: PlayerId,
        /// Chosen recast actor.
        target: PlayerId,
        /// Chosen recast card.
        card: CardId,
        /// Stage 0 choose actor, 1 choose card, 2 commit recast.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 怨清（shiji 华歆，ABI 443）：出牌阶段结束时，本阶段用过的每种类别
    /// （基本/锦囊/装备）从弃牌堆权威随机取一张置入仁库（技能牌堆）。
    ///
    /// Appended for serialized ordinal stability.
    ResolveYuanqing {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 疏陈（shiji 华歆，ABI 444）：其他角色濒死时（每轮一次）若仁库（怨清
    /// 技能牌堆）> 3 张则获得全部并令其回复 1。
    ///
    /// Appended for serialized ordinal stability.
    ResolveShuchen {
        /// Skill owner.
        player: PlayerId,
        /// The dying player.
        dying: PlayerId,
        /// 怨清 skill definition whose pile is the 仁库.
        renku_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 断笔（shiji 刘巴，ABI 445）：限定技——全场手牌总数 > 人数×2 时可选任意
    /// 名角色，逐目标强制弃 min(3, floor(手牌/2)) 张；全部弃完后再选一名角色
    /// 权威随机获得至多 3 张仍处弃牌堆的被弃牌。
    ///
    /// Appended for serialized ordinal stability.
    ResolveDuanbi {
        /// Skill owner.
        player: PlayerId,
        /// Chosen discard targets.
        targets: Vec<PlayerId>,
        /// Stage 0 open discard, 1 commit discard, 2 open gain target, 3 commit gain.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 躁厉（shiji 孙翊，ABI 446）：回合开始时有“躁厉”标记（使用/打出牌时 +1
    /// 至多 4）→ 弃至少一张牌（可全弃）并摸（标记数+弃牌数）张，总数 > 2 则
    /// 失去 1 点体力；出牌阶段仅可使用本回合获得的牌（复用
    /// hand_card_gained_this_turn_only modifier）。
    ///
    /// Appended for serialized ordinal stability.
    ResolveZaoli {
        /// Skill owner.
        player: PlayerId,
        /// 躁厉 mark count at activation.
        mark: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 孤影（shiji 向宠，ABI 447）：你失去一张手/装/判区牌后（每回合一次，当前
    /// 回合角色为他人）其须二选一——随机交给你一张其可获得的牌 / 你收回失去
    /// 的牌（若仍在弃牌堆）。
    ///
    /// Appended for serialized ordinal stability.
    ResolveGuying {
        /// Skill owner.
        player: PlayerId,
        /// Current-phase actor making the choice.
        actor: PlayerId,
        /// The lost card (filled by host from the event card).
        card: CardId,
        /// Stage 0 open two-choice, 1 commit branch.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 睦阵（shiji 向宠，ABI 448）：出牌阶段限一次三选一（每回合各一次）——
    /// ①装备换手牌 ②给牌换装备 ③睦阵摸牌。各分支的多段选择与结算由引擎权威
    /// 推进。
    ///
    /// Appended for serialized ordinal stability.
    ResolveMuzhen {
        /// Skill owner.
        player: PlayerId,
        /// Chosen branch 0..=2.
        branch: u8,
        /// Stage: 0 open branch, 1 open cards, 2 open target(s), 3 commit.
        stage: u8,
        /// Cards selected in branch stages.
        cards: Vec<CardId>,
        /// Targets selected in branch stages.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 通用当前伤害修改（ABI 562）：damage_before 窗口内对当前伤害量增/减 delta。
    /// 正值走 increase_bound_damage，负值走 reduce_bound_damage（钳制到 0），零值为
    /// 空操作。解锁 lianhuo / dckanghui / renjia 等非卡牌来源伤害修改技能。
    ///
    /// Appended for serialized ordinal stability.
    ModifyCurrentDamage {
        /// Skill owner requesting the modification.
        player: PlayerId,
        /// Delta to apply to the current damage amount (may be negative).
        delta: i32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势吕壹【隳律/颂谳/势施】复合状态机。ABI 564。
    ///
    /// Appended for serialized ordinal stability.
    ResolvePotlvyi {
        /// Skill owner.
        player: PlayerId,
        /// Event actor/source, or NONE.
        target: PlayerId,
        /// Triggering or tracked physical card, or NONE.
        card: CardId,
        /// Public action 0..=9; higher stages are engine-owned continuations.
        action: u8,
        /// Physical cards carried by a CardsMoved trigger.
        #[serde(default)]
        event_cards: Vec<CardId>,
        /// Ordered targets carried by a CardUsed trigger.
        #[serde(default)]
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势曹爽【典易/奢权/渐专/溺尊】复合状态机。ABI 568。
    ///
    /// Appended for serialized ordinal stability.
    ResolvePotcaoshuang {
        /// Skill owner.
        player: PlayerId,
        /// Event source/target, or NONE.
        target: PlayerId,
        /// Triggering physical card, or NONE.
        card: CardId,
        /// Public action 0..=11; higher stages are engine-owned continuations.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势于吉【道转/符济】复合状态机。ABI 570。
    ///
    /// Appended for serialized ordinal stability.
    ResolvePotyuji {
        /// Skill owner or current effect beneficiary.
        player: PlayerId,
        /// Event actor/target, or NONE.
        target: PlayerId,
        /// Triggering physical card, or NONE.
        card: CardId,
        /// Public action 0..=8.
        action: u8,
        /// Physical source cards carried by the triggering card event.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Ordered active-skill targets.
        #[serde(default)]
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// SP【横江】：累计当前回合角色的手牌上限减值，并在回合结束结算摸牌。ABI 572。
    ///
    /// Appended for serialized ordinal stability.
    ResolveRehengjiang {
        /// Skill owner.
        player: PlayerId,
        /// Turn actor for settlement; action 0 reads the authoritative current actor.
        actor: PlayerId,
        /// Public action: 0 records one activation, 1 settles at turn end.
        action: u8,
        /// Temporary hand-limit modifier skill definition.
        hand_limit_def: SkillDefId,
        /// Temporary turn-end settlement skill definition.
        draw_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Permanently grant the first configured skill definition the player does not own. ABI 575.
    /// Appended for serialized ordinal stability.
    AddFirstMissingSkill {
        /// Skill owner.
        player: PlayerId,
        /// Three ordered candidate skill definitions.
        skill_defs: [SkillDefId; 3],
        /// Originating skill instance.
        skill: SkillId,
    },
    /// XIANDING 糜夫人【存嗣】with deferred【闺秀】recovery. ABI 578.
    /// Appended for serialized ordinal stability.
    ResolveDccunsi {
        /// Skill owner.
        player: PlayerId,
        /// Selected recipient of【勇决】.
        target: PlayerId,
        ///【闺秀】definition checked by the deferred recovery.
        guixiu_def: SkillDefId,
        ///【勇决】definition granted to the target.
        yongjue_def: SkillDefId,
        /// Stage 0 grants/draws; stage 1 performs the deferred recovery.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Bind all targets of the exact currently resolving Sha to the originating skill. ABI 580.
    /// Appended for serialized ordinal stability.
    RecordCurrentShaAllTargets {
        /// Sha user.
        player: PlayerId,
        /// Exact logical Sha carrier.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose either to give an exact number of hand cards to one living target in `scope`, or
    /// discard that many hand cards. ABI 581.
    /// Appended for serialized ordinal stability.
    ChooseGiveOrDiscardHand {
        /// Skill owner choosing and paying cards.
        player: PlayerId,
        /// Exact hand-card payment count.
        payment_count: u32,
        /// Recipient target scope.
        scope: DrawTargetScope,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Let the chooser cancel or discard an exact number of hand/equipment cards, then recover
    /// the specified target after movement triggers finish. ABI 582.
    /// Appended for serialized ordinal stability.
    ChooseDiscardHandEquipThenRecoverTarget {
        /// Skill owner and chooser.
        chooser: PlayerId,
        /// Living wounded recovery target.
        target: PlayerId,
        /// Exact number of cards required.
        count: u32,
        /// Maximum HP recovered after the discard resolves.
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 杨彪「举讹」 target confirmation/selection and its repeated random virtual
    /// card uses. ABI 583.
    /// Appended for serialized ordinal stability.
    ResolveDcjue {
        /// Skill owner and virtual-card user.
        player: PlayerId,
        /// End-phase player at stage 0, then the accepted skill target.
        target: PlayerId,
        /// Engine-owned stage: 0 opens the context-appropriate choice, 1 continues the loop.
        stage: u8,
        /// Virtual-card uses still to attempt after acceptance.
        remaining: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 阮籍「昭文/酒遁」. ABI 584.
    /// Appended for serialized ordinal stability.
    ResolveRuanji {
        /// Skill owner.
        player: PlayerId,
        /// Event source, or NONE.
        source: PlayerId,
        /// Triggering card, or NONE.
        card: CardId,
        /// Public action: 0 昭文 setup, 1 red-card draw, 2 酒 rewrite, 3 酒遁 response.
        action: u8,
        /// Physical cards carried by the triggering card use.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 崔琰毛玠「征辟」. ABI 585.
    /// Appended for serialized ordinal stability.
    ResolveZhengbi {
        /// Skill owner.
        player: PlayerId,
        /// 征辟目标。
        target: PlayerId,
        /// 征辟牌。
        card: CardId,
        /// Public action stage.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 崔琰毛玠「奉迎」. ABI 585.
    /// Appended for serialized ordinal stability.
    ResolveFengying {
        /// Skill owner.
        player: PlayerId,
        /// 奉迎辅助技能定义。
        helper_def: SkillDefId,
        /// Public action stage.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 傅干「巧谏」. ABI 586.
    /// Appended for serialized ordinal stability.
    ResolveQiaojian {
        /// Skill owner.
        player: PlayerId,
        /// 巧谏目标。
        target: PlayerId,
        /// 巧谏辅助技能定义。
        helper_def: SkillDefId,
        /// Public action stage.
        action: u8,
        /// 巧谏携带的牌。
        #[serde(default)]
        cards: Vec<CardId>,
        /// Number of leading cards selected by the skill owner.
        #[serde(default)]
        split: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 傅干「析察」. ABI 586.
    /// Appended for serialized ordinal stability.
    ResolveXicha {
        /// Skill owner.
        player: PlayerId,
        /// 事件来源。
        source: PlayerId,
        /// 触发牌。
        card: CardId,
        /// Public action stage.
        action: u8,
        /// 析察携带的牌。
        #[serde(default)]
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 胡遵「斩涛 / 安境」. ABI 587.
    /// Appended for serialized ordinal stability.
    ResolveHuzun {
        /// Skill owner.
        player: PlayerId,
        /// Damage source for 斩涛; none for 安境.
        source: PlayerId,
        /// Damage card for 斩涛; none for 安境.
        card: CardId,
        /// 0=斩涛, 1=安境开始, 2=安境目标结算.
        action: u8,
        /// Physical source cards for 斩涛.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Selected 安境 targets.
        #[serde(default)]
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 诸葛梦雪「寄春 / 寒英」. ABI 588.
    /// Appended for serialized ordinal stability.
    ResolveZhugemengxue {
        /// Skill owner.
        player: PlayerId,
        /// Chosen target for a resumed stage.
        target: PlayerId,
        /// Bound card for a resumed stage.
        card: CardId,
        /// 0=寄春开始, 1=展示, 2=阶段清理, 3=分支结算, 4=目标弃牌选择,
        /// 10=寒英开始, 11=寒英目标结算, 12=展示后选择目标.
        action: u8,
        /// Active-skill selected cards.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 寸目（shenhua 许攸，主公技）：本回合摸牌改为从牌堆底摸。ABI 589。
    /// Appended for serialized ordinal stability.
    DrawCardsFromBottom {
        /// Player drawing.
        player: PlayerId,
        /// Card count.
        count: u32,
    },
    /// 成略（shenhua 许攸）：翻面后摸 1/2 弃 2/1，本回合弃置花色牌无距离/次数限制。
    /// ABI 589.
    ResolveChenglve {
        /// Skill owner.
        player: PlayerId,
        /// 0=翻面+摸牌阶段, 1=弃牌阶段, 2=授予同花色无限修饰阶段.
        stage: u8,
        /// 弃牌阶段选出的花色（阶段 2 使用，位 1..=4 对应黑桃/红桃/梅花/方块）.
        #[serde(default)]
        suits: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 闭月（界貂蝉）：结束阶段开始时，你摸 min(4, 本回合内受到伤害的不同角色数 + 1)
    /// 张牌。ABI 599。
    ResolveSbbiyue {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 窃王（sxrm 严畯）：其他角色受到伤害后，若其与拥有者距离不大于 1，拥有者摸一张
    /// 牌（手牌视为【无懈可击】的部分以近似表达）。ABI 596。
    ResolveSxrmqiewang {
        /// Skill owner.
        player: PlayerId,
        /// 受伤角色。
        event_player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 潜流（sxrm 伏皇后）：成为【杀】目标后，取牌堆底四张；花色数大于 3 则获得之，
    /// 否则置回牌堆顶（旧 JS 的拖动排序以置顶近似）。ABI 596。
    ResolveSxrmqianliu {
        /// Skill owner.
        player: PlayerId,
        /// 使用者（自身为使用者时跳过）。
        source: PlayerId,
        /// 0=取牌堆底判定, 1=获得或置顶.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 旧不屈（shenhua 旧周泰）：体力降至 0 以下时将牌堆顶牌置入「创」技能牌堆，
    /// 点数均不同则本次体力流失不进入濒死。ABI 591。
    ResolveGzbuqu {
        /// Skill owner.
        player: PlayerId,
        /// 本次体力变化量（负数 = 流失）。
        delta: i32,
        /// 0=置入创牌并判定免死, 1=恢复后移除.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 旧不屈·恢复：回复体力后移除「创」牌（点数重复者优先），点数均不同则救回濒死。
    /// ABI 591.
    ResolveGzbuquRecover {
        /// Skill owner.
        player: PlayerId,
        /// 0=选择移除, 1=移除+救回判定.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 雄乱（shenhua 张绣，限定技）：废除装备栏与判定区，本回合你只能对指定角色使用牌，
    /// 且其不能使用或打出手牌。ABI 590。
    ResolveXiongluan {
        /// Skill owner.
        player: PlayerId,
        /// 限定目标（阶段 0 选择）。
        target: PlayerId,
        /// 0=目标选择, 1=施加限制.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 踞襄（shenhua 张绣，主公技）：其他群势力角色出牌阶段限一次，可将一张装备牌
    /// 置入你的装备区（栏位空闲/禁用时直接装备并恢复栏位），否则你获得之。ABI 590。
    ResolveTwjuxiang {
        /// Skill owner (the lord).
        player: PlayerId,
        /// 触发角色的出牌阶段（其他群势力角色）。
        event_player: PlayerId,
        /// 0=装备牌选择, 1=置入或获得.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 寸目（shenhua 许攸，主公技）触发标记：当前摸牌阶段从牌堆底摸牌。ABI 589。
    /// Appended for serialized ordinal stability.
    SetDrawPhaseFromBottom {
        /// Skill owner.
        player: PlayerId,
    },
    /// 恃才（shenhua 许攸）：使用后可将弃置牌堆中的实体牌以任意顺序置于牌堆顶，然后摸一张。
    /// ABI 589.
    ResolveShicai {
        /// Skill owner.
        player: PlayerId,
        /// 使用牌（用于判定本回合是否同类首用）。
        card: CardId,
        /// 已置于牌堆顶的牌（阶段 1 继续时为空）。
        #[serde(default)]
        placed: Vec<CardId>,
        /// 0=开始, 1=放置完成摸一张.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 柏灵筠「灵慧 / 黠策」. ABI 593.
    /// Appended for serialized ordinal stability.
    ResolveBailingyun {
        /// Skill owner.
        player: PlayerId,
        /// Chosen target for a resumed 黠策 stage.
        target: PlayerId,
        /// Chosen physical card for a resumed stage.
        card: CardId,
        /// 0..=4=灵慧, 10=造成伤害分支, 11=受到伤害分支.
        action: u8,
        /// Revealed or remaining 灵慧 cards.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 威关银屏「绍威」. ABI 594.
    /// Appended for serialized ordinal stability.
    ResolveDcshaowei {
        /// Skill owner.
        player: PlayerId,
        /// 0=处理失牌花色，1=回复后选择恢复装备栏.
        action: u8,
        /// Physical cards carried by the CardsMoved trigger.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 威关银屏「涤仇」. ABI 594.
    /// Appended for serialized ordinal stability.
    ResolveDcdichou {
        /// Skill owner.
        player: PlayerId,
        /// Selected marked target for resumed stages.
        target: PlayerId,
        /// Selected discarded card for resumed stages.
        card: CardId,
        /// 0..=3=回合开始，10..=11=回合结束.
        action: u8,
        /// Temporary attack-range/Sha-limit helper definition.
        helper_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 凌操「独锋」. ABI 595.
    /// Appended for serialized ordinal stability.
    ResolveDcdufeng {
        /// Skill owner.
        player: PlayerId,
        /// 0=choose costs, 1=draw and grant turn modifiers.
        action: u8,
        /// Temporary attack-range/Sha-limit helper definition.
        helper_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Force a player to choose and discard between `min` and `max` discardable hand cards.
    /// ABI 596. Appended for serialized ordinal stability.
    ChooseDiscardHandRange {
        /// Player choosing and discarding cards.
        player: PlayerId,
        /// Inclusive minimum selection count.
        min: u32,
        /// Inclusive maximum selection count.
        max: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 陈泰「矜贤」. ABI 597.
    /// Appended for serialized ordinal stability.
    ResolveDcctjiuxian {
        /// Skill owner.
        player: PlayerId,
        /// Damaged Duel target or activation placeholder.
        event_player: PlayerId,
        /// Selected recovery target for resumed recovery.
        target: PlayerId,
        /// Tracked virtual Duel or selected recast cards' cleanup carrier.
        card: CardId,
        /// 0=choose recast, 1=commit recast, 2=choose recovery, 3=recover,
        /// 5=use Duel, 6=cleanup tracked Duel.
        action: u8,
        /// Selected recast cards.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 卞玥「庇族/无胁」. ABI 598.
    /// Appended for serialized ordinal stability.
    ResolveBianyue {
        /// Skill owner.
        player: PlayerId,
        /// Selected 无胁 target or placeholder.
        target: PlayerId,
        /// 0=庇族, 10=choose 无胁 target, 11=swap hands, 12=move damage cards to bottom.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 朱佩兰「辞虑/痛悼」. ABI 600.
    /// Appended for serialized ordinal stability.
    ResolveZhupeilan {
        /// Skill owner.
        player: PlayerId,
        /// Event source, selected 痛悼 target, or placeholder.
        target: PlayerId,
        /// Triggering trick carrier, or none for 痛悼.
        card: CardId,
        /// Package-only helper carrying exact-card policies; none for 痛悼.
        helper_skill_def: SkillDefId,
        /// 0=辞虑 targeted, 1=prevent tracked damage, 2=tracked card resolved,
        /// 3/4/5=apply nullify/prevent/gain choice, 6=offer choice after draw,
        /// 10=choose 痛悼 target, 11=reset target.
        action: u8,
        /// Exact physical materials captured at card-target time.
        #[serde(default)]
        event_cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 势张绣「附袭/豪义」. ABI 601.
    /// Appended for serialized ordinal stability.
    ResolveDcSbZhangxiu {
        /// Skill owner.
        player: PlayerId,
        /// Other Play-phase player or placeholder.
        target: PlayerId,
        /// 0=附袭, 1=附袭弃牌后的虚拟杀, 10=豪义获取, 11=豪义分配.
        action: u8,
        /// Exact 豪义 cards retained across the gain boundary.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 新杀谋周瑜「融火/英谋」. ABI 602.
    /// Appended for serialized ordinal stability.
    ResolveDcSbZhouyu {
        /// Skill owner.
        player: PlayerId,
        /// Triggering card user, or placeholder for non-card stages.
        source: PlayerId,
        /// Triggering or currently tracked physical card.
        card: CardId,
        /// 0=融火蓄势, 1=融火增伤/额外目标, 10=开局转换,
        /// 11=英谋开始, 20=英谋子牌结算.
        action: u8,
        /// Declared targets captured from the triggering card event.
        #[serde(default)]
        event_targets: Vec<PlayerId>,
        /// Targets damaged by the exact resolved card.
        #[serde(default)]
        damaged_targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 新杀谋鲁肃「明势/盟谋」. ABI 603.
    /// Appended for serialized ordinal stability.
    ResolveDcSbLusu {
        /// Skill owner.
        player: PlayerId,
        /// Triggering transfer source or tracked Sha user.
        source: PlayerId,
        /// Triggering or tracked physical card.
        card: CardId,
        /// 0=明势, 10=开局转换, 11=盟谋转移, 20=伤害回复, 21=杀结算.
        action: u8,
        /// Event or engine-owned selected cards.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Event damage amount.
        #[serde(default)]
        amount: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 张臶「技能A/技能B」. ABI 604.
    /// Appended for serialized ordinal stability.
    ResolveZhangjian {
        /// Skill owner.
        player: PlayerId,
        /// Triggering physical damage card, or NONE for 技能B.
        card: CardId,
        /// 技能B grants this skill definition to the selected target.
        grant_skill_def: SkillDefId,
        /// 0=按牌点数改写/防止当前伤害, 1=选择授技目标, 2=完成授技.
        action: u8,
        /// Selected target for engine-owned continuation stages.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 诸葛若雪「琼英/暖惠」. ABI 605.
    /// Appended for serialized ordinal stability.
    ResolveZhugeruoxue {
        /// Skill owner.
        player: PlayerId,
        /// 暖惠 target, field-card destination, or placeholder.
        target: PlayerId,
        /// Moved field card or current virtual basic card.
        card: CardId,
        /// 0..3=琼英, 10..14=暖惠.
        action: u8,
        /// Remaining 暖惠 uses after the current selection.
        #[serde(default)]
        remaining: u32,
        /// Basic-card name bits already used by 暖惠.
        #[serde(default)]
        used_mask: u32,
        /// Whether 暖惠 used a repeated card name.
        #[serde(default)]
        repeated: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 曹轶「蜜饴/寅君」. ABI 606.
    /// Appended for serialized ordinal stability.
    ResolveCaoyi {
        /// Skill or helper owner.
        player: PlayerId,
        /// Triggering or tracked card.
        card: CardId,
        /// Package-only helper carrying pending 蜜饴 entries.
        helper_skill_def: SkillDefId,
        /// 0=蜜饴开始, 10=结束阶段反向结算, 11=回合末清理,
        /// 20=寅君使用无来源杀, 21=寅君杀结算并计数禁用.
        action: u8,
        /// Targets captured from the triggering CardResolved event.
        #[serde(default)]
        event_targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve XIANDING 马伶俐「硝引」 and its 花火 target rewrite. ABI 607.
    /// Appended for serialized ordinal stability.
    ResolveMalingli {
        /// Skill or package-only helper owner.
        player: PlayerId,
        /// Triggering, selected, or cost card.
        card: CardId,
        /// Package-only helper and skill-pile definition.
        helper_skill_def: SkillDefId,
        /// 0..2=硝引准备阶段；10..14=伤害前；20=花火目标改写.
        action: u8,
        /// Current damage source (host-captured).
        #[serde(default = "no_player")]
        source: PlayerId,
        /// Current damage target (host-captured).
        #[serde(default = "no_player")]
        target: PlayerId,
        /// Current damage nature (host-captured).
        #[serde(default)]
        nature: u8,
        /// Bound damage id for engine-owned continuations.
        #[serde(default)]
        damage_id: u32,
        /// Revealed or selected physical cards.
        #[serde(default)]
        event_cards: Vec<CardId>,
        /// Selected contiguous targets.
        #[serde(default)]
        event_targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 玩点论杀 费诗「率言」. ABI 608.
    /// Appended for serialized ordinal stability.
    ResolveNsshuaiyan {
        /// Skill owner who receives the selected benefit.
        player: PlayerId,
        /// Other player who just recovered and must choose the branch.
        recovered_player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 玩点论杀 韩龙「伺机/刺酋」. ABI 609.
    /// Appended for serialized ordinal stability.
    ResolveWdbHanlong {
        /// Skill owner.
        player: PlayerId,
        /// Current damage target; none for 伺机.
        target: PlayerId,
        /// Current damage card; none for 伺机.
        card: CardId,
        /// Package-only 刺酋濒死 helper definition.
        helper_skill_def: SkillDefId,
        /// 0=伺机；1=刺酋伤害前.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 玩点论杀 刘馥「筑城/夺气」. ABI 610.
    /// Appended for serialized ordinal stability.
    ResolveWdbLiufu {
        /// Skill owner.
        player: PlayerId,
        /// Sha user or current Play-phase player; none for 筑城 storage.
        target: PlayerId,
        /// Current Sha; none for storage/夺气.
        card: CardId,
        /// Stable 筑城 definition owning the public pile.
        zhucheng_skill_def: SkillDefId,
        /// 0=筑城 storage；1=筑城 Sha defense；2=夺气；3=engine-owned phase finish.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 玩点论杀 刘焉「绝道/割据」. ABI 611.
    /// Appended for serialized ordinal stability.
    ResolveWdbLiuyan {
        /// Skill owner.
        player: PlayerId,
        /// Selected discard cost for 绝道; none for 割据.
        card: CardId,
        /// 0=绝道；1=割据.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 玩点论杀 留赞「亢音」. ABI 612.
    /// Appended for serialized ordinal stability.
    ResolveWdbLiuzan {
        /// Skill owner.
        player: PlayerId,
        /// Selected other player or continuation placeholder.
        target: PlayerId,
        /// Stable temporary modifier helper definition.
        helper_skill_def: SkillDefId,
        /// 0=pay HP；1=choose discarded target card；2=basic draw branch；3=nonbasic modifier branch.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 玩点论杀 田豫「诱战」. ABI 613.
    /// Appended for serialized ordinal stability.
    ResolveWdbTianyu {
        /// Skill owner.
        player: PlayerId,
        /// Target of the incoming Sha.
        target: PlayerId,
        /// Incoming Sha card.
        card: CardId,
        /// 0=public trigger; later stages are engine-owned.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-owned continuation for the recoverable 诱敌深入 Sha loop. ABI 613.
    ContinueYoudishenru {
        /// 0=start virtual response; 1=inspect the forced Sha; 2=resume the cancelled incoming Sha.
        action: u8,
        /// Protected player who used 诱敌深入.
        protected: PlayerId,
        /// Original Sha user forced to continue attacking.
        source: PlayerId,
        /// Exact forced Sha carrier for action 1.
        forced_card: CardId,
        /// Encoded incoming-Sha continuation.
        saved: Vec<u32>,
    },
    /// Resolve archived extension 玩点论杀 习珍习宏「负仇」. ABI 614.
    /// Appended for serialized ordinal stability.
    ResolveWdbXizhenxihong {
        /// Skill owner.
        player: PlayerId,
        /// Incoming Sha source for action 0; unused for action 1.
        target: PlayerId,
        /// Incoming Sha for action 0; unused for action 1.
        card: CardId,
        /// 0=become Sha target; 1=turn-ended settlement.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-owned continuation for sequential 负仇 settlement. ABI 614.
    ContinueWdbFuchou {
        /// 0=advance to next living source; 1=offer Sha after drawing.
        action: u8,
        /// Skill owner.
        player: PlayerId,
        /// Current source for action 1.
        target: PlayerId,
        /// Remaining recorded sources in original insertion order.
        remaining: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 玩点论杀 杨仪「筹度/立断」. ABI 615.
    /// Appended for serialized ordinal stability.
    ResolveWdbYangyi {
        /// Skill owner for public actions; card user for engine-owned actions.
        player: PlayerId,
        /// Chosen card user or gained-card recipient.
        target: PlayerId,
        /// Selected cost, gained equipment, or resolving 调兵遣将 carrier.
        card: CardId,
        /// 0=筹度; 1=立断; 2=engine-owned 调兵遣将 entry.
        action: u8,
        /// Originating skill instance; none for the physical card effect.
        skill: SkillId,
    },
    /// Engine-owned recoverable 调兵遣将 settlement. ABI 615.
    ContinueDiaobingqianjiang {
        /// 0=reveal after user draw; 1=advance target swaps.
        action: u8,
        /// Card user, who chooses the final top-deck order.
        actor: PlayerId,
        /// Physical or materialized card carrier.
        source_card: CardId,
        /// Targets still waiting to resolve in seat order.
        targets: Vec<PlayerId>,
        /// Currently revealed cards remaining in the processing zone.
        revealed: Vec<CardId>,
    },
    /// Resolve archived extension 玩点论杀 枣祗任峻「粮策/坚壁」and【望梅止渴】. ABI 616.
    ResolveWdbZaozhiRenjun {
        /// Skill owner, or card user for engine-owned card settlement.
        player: PlayerId,
        /// Selected recipient, protected target, or card target.
        target: PlayerId,
        /// Current physical card carrier.
        card: CardId,
        /// Stable triggering-card snapshot, used by 粮策.
        cards: Vec<CardId>,
        /// 0=粮策剩余牌; 1=坚壁; 2=望梅止渴 entry; 3/4=draw after recover/no recover.
        action: u8,
        /// Originating skill instance; none for engine-owned card settlement.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 韩当「险袭」. ABI 617.
    ResolveDddXianxi {
        /// Skill owner and Sha user.
        player: PlayerId,
        /// Current physical Sha carrier.
        card: CardId,
        /// 0=choose extra targets; 1=settle undamaged extra targets.
        action: u8,
        /// Declared targets captured from the event.
        event_targets: Vec<PlayerId>,
        /// Targets damaged by the exact card use.
        damaged_targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 周处「侠横」. ABI 618.
    ResolveDddXiaheng {
        /// Skill owner.
        player: PlayerId,
        /// Character chosen to discard cards.
        discard_target: PlayerId,
        /// Character dealt damage.
        damage_target: PlayerId,
        /// 0=choose discard target; 1=choose damage target; 2=post-damage settlement.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 刘宠「精彀/末焱」. ABI 619.
    ResolveDddLiuchong {
        /// Skill owner.
        player: PlayerId,
        /// Event actor or selected recipient.
        actor: PlayerId,
        /// Triggering physical card, if any.
        card: CardId,
        /// 0=末焱轮首; 1=末焱受伤增伤; 2=精彀武器结算; 3=末焱亮牌后续.
        action: u8,
        /// Revealed physical cards carried by the internal continuation.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 丁奉「短兵」. ABI 621.
    ResolveDddDuanbing {
        /// Skill owner.
        player: PlayerId,
        /// Damage target for the transfer branch, otherwise none.
        target: PlayerId,
        /// Damage card carrier for the transfer branch, otherwise none.
        card: CardId,
        /// 0=start active branch; 1=transfer after card damage; 2/3=internal resumes.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 吴质「灵涌/虚孝」. ABI 622.
    ResolveDddWuzhi {
        /// Skill owner.
        player: PlayerId,
        /// Judgment or selected physical card, otherwise none.
        card: CardId,
        /// Public actions: 0=灵涌，10=虚孝；other values are engine continuations.
        action: u8,
        /// Cards from the triggering discard batch or internal continuation.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 许靖「虚誉/实荐」. ABI 623.
    ResolveDddXujing {
        /// Skill owner.
        player: PlayerId,
        /// Current card user, otherwise none.
        source: PlayerId,
        /// Current physical card, otherwise none.
        card: CardId,
        /// Public actions are 0/1/2 and 10/11/12; 3/13 are engine continuations.
        action: u8,
        /// Current card targets captured from CardUsed.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 曹髦「太思/驱车」. ABI 624.
    ResolveDddCaomao {
        /// Skill owner.
        player: PlayerId,
        /// Public actions: 0=太思，10=驱车；other values are engine continuations.
        action: u8,
        /// Active-skill selected cards or internal continuation cards.
        cards: Vec<CardId>,
        /// Active-skill selected targets or internal continuation targets.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 辛宪英「洞察」. ABI 625.
    ResolveDddXinxianying {
        /// Skill owner.
        player: PlayerId,
        /// Current turn player.
        actor: PlayerId,
        /// Public actions: 0=双方置鉴，1=回合末依次取鉴；other values are continuations.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 向朗「恰化/腹笥/拓籍」. ABI 626.
    ResolveDddXianglang {
        /// Skill owner.
        player: PlayerId,
        /// Current turn player, borrower, or none.
        actor: PlayerId,
        /// Public actions: 0=恰化，1=腹笥初始化，2=拓籍；other values are continuations.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 于禁「整军」. ABI 627.
    ResolveDddYujin {
        /// Skill owner.
        player: PlayerId,
        /// Source-zone owner or HP-changing player.
        actor: PlayerId,
        /// Destination-zone owner for a move event.
        target: PlayerId,
        /// Bit 0/1: moved from/to hand; bit 2/3: moved from/to equipment.
        move_mask: u8,
        /// Public actions: 0=HP change, 1=cards moved; 2 continues equipment handling.
        action: u8,
        /// Equipment candidates retained across the hand-card branch.
        candidates: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 刘晔「察势/齐策」. ABI 628.
    ResolveDddLiuye {
        /// Skill owner.
        player: PlayerId,
        /// Current Play-phase player or card user.
        actor: PlayerId,
        /// Current used card, or none for phase actions.
        card: CardId,
        /// Public actions: 0=察势布置，1=察势命中，2=齐策准备，3=齐策解锁，4=察势过期。
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 鲍三娘「枝连/赍剑」. ABI 629.
    ResolveDddBaosanniang {
        /// Skill owner.
        player: PlayerId,
        /// Gift target or current virtual-card user.
        target: PlayerId,
        /// Gift card or engine-owned materialized virtual card.
        card: CardId,
        /// Public actions: 0=枝连回合末，1=赍剑主动技；other values are continuations.
        action: u8,
        /// Revealed target-hand cards retained across virtual uses.
        cards: Vec<CardId>,
        /// Remaining virtual-card users in authoritative seat order.
        targets: Vec<PlayerId>,
        /// Previously used logical card name, encoded by normalized CardKind tag (0 = none).
        used_kind: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 甄宓「淼形/浮世」. ABI 630.
    ResolveDddZhenji {
        /// Skill owner.
        player: PlayerId,
        /// Optional male exchange target.
        target: PlayerId,
        /// Public actions: 0=淼形起手，1=淼形平衡，2=浮世；other values stay internal.
        action: u8,
        /// Recoverable stage within the selected action.
        stage: u8,
        /// Selected or pending water-pile indices (1 or 2).
        piles: Vec<u8>,
        /// Frozen physical cards selected from a water pile.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 赵昂「奋计」. ABI 631.
    ResolveDddZhaoang {
        /// Skill owner.
        player: PlayerId,
        /// 0=draw phase, 1=discard phase.
        action: u8,
        /// Recoverable stage: 0=choose trick, 1=use trick, 2=finish phase replacement.
        stage: u8,
        /// Engine-owned materialized virtual card.
        card: CardId,
        /// Selected ordinary-trick CardKind tag.
        kind: u8,
        /// Damage baseline for draw, declared target count for discard.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 刘巴「丰政/玉律」. ABI 632.
    ResolveDddLiuba {
        /// Skill owner or global-helper user.
        player: PlayerId,
        /// Current turn player when relevant.
        actor: PlayerId,
        /// Public actions: 0=丰政转化，1=轮首分配，2=安装全局技，3=玉律起始，4=玉律响应。
        action: u8,
        /// Active invocation's selected physical cards or internal frozen cards.
        cards: Vec<CardId>,
        /// Internal target state parallel to `cards` when needed.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 蹇硕「分野/逝潮」. ABI 633.
    ResolveDddJianshuo {
        /// Skill owner.
        player: PlayerId,
        /// Active comparison target or damage source when relevant.
        target: PlayerId,
        /// Public actions: 0=分野，1=逝潮准备阶段，2=逝潮伤害前。
        action: u8,
        /// Internal frozen physical comparison cards.
        cards: Vec<CardId>,
        /// Internal participants/sides parallel to `cards` when needed.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 管宁「游学/承经」. ABI 634.
    ResolveDddGuanning {
        /// Skill owner.
        player: PlayerId,
        /// Current round/turn actor when relevant.
        actor: PlayerId,
        /// Public actions: 0=游学轮首，1=承经回合末，2=承经主动使用。
        action: u8,
        /// Active invocation's selected physical source card or internal carrier.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 曹爽「专摄/危秋」. ABI 635.
    ResolveDddCaoshuang {
        /// Skill owner.
        player: PlayerId,
        /// Current turn actor or recovering player.
        actor: PlayerId,
        /// Current used card, or none outside card-use timing.
        card: CardId,
        /// Public actions: 0=专摄赠牌，1=同名牌使用，2=回合末，3=危秋。
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Cancel the recovery whose `recover_before` trigger queue is running. ABI 635.
    PreventRecovery {
        /// Engine-owned recovery id; content emits zero and the VM host binds it.
        #[serde(default)]
        recovery_id: u32,
    },
    /// Apply recovery after its `recover_before` trigger queue. Engine-owned, ABI 635.
    CommitRecovery {
        /// Engine-owned recovery id, or zero when no trigger matched.
        recovery_id: u32,
        /// Recovery source.
        source: PlayerId,
        /// Recovery target.
        player: PlayerId,
        /// Requested amount.
        amount: u32,
    },
    /// Resolve archived extension 3D精选 薛灵芸「涟洏/暗织」. ABI 636.
    ResolveDddXuelingyun {
        /// Skill owner.
        player: PlayerId,
        /// Current actor.
        actor: PlayerId,
        /// Event card, or none.
        card: CardId,
        /// Event physical materials.
        cards: Vec<CardId>,
        /// Public actions: 0=涟洏，1=清除点数限制，2=记录末牌颜色，3=暗织指定，4=目标失效，5=清理暗织。
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 刘宏「失兴/党锢/覆宗」. ABI 637.
    ResolveDddLiuhong {
        /// Skill owner.
        player: PlayerId,
        /// Current actor, changed player, or dead 群 character.
        actor: PlayerId,
        /// Public actions: 0=回合刷新失兴，1=女性状态刷新，2=首张锦囊，3=党锢，4=覆宗开局，5=覆宗死亡。
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 夏侯玄「朗怀/玄论」. ABI 638.
    ResolveDddXiahouxuan {
        /// Skill owner.
        player: PlayerId,
        /// Public actions: 0=朗怀摸牌替换，1=玄论受伤，2=回合末玄论选择。
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 张闿「劫行/拜泪」. ABI 639.
    ResolveDddZhangkai {
        /// Skill owner.
        player: PlayerId,
        /// Event actor or selected target.
        actor: PlayerId,
        /// Event card, selected physical card, or none.
        card: CardId,
        /// Event physical materials.
        cards: Vec<CardId>,
        /// Recovery amount for the recover-before branch.
        amount: u32,
        /// Public actions: 0=截取回复，1=截取装备，2=回复后重置，3=装备结算后重置，4=拜泪。
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve archived extension 3D精选 梁习「仝御」. ABI 640.
    ResolveDddLiangxi {
        /// Skill owner.
        player: PlayerId,
        /// Event actor or target making the current choice.
        actor: PlayerId,
        /// Exact 五谷丰登 carrier, or none for the remained-card event.
        card: CardId,
        /// Event physical cards.
        cards: Vec<CardId>,
        /// Public actions: 0=choose direction, 1=target accepts or rejects, 2=offer leftovers.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Engine-owned recoverable【五谷丰登】settlement. ABI 616.
    ContinueWugu {
        /// 0=reveal; 1=start target queue; 2=discard leftovers.
        action: u8,
        /// Card user.
        actor: PlayerId,
        /// Physical or materialized card carrier.
        source_card: CardId,
        /// Original targets, kept for reveal count and seat order.
        targets: Vec<PlayerId>,
        /// Targets excluded by skills before their individual resolution.
        excluded: Vec<PlayerId>,
        /// Currently revealed cards remaining in processing.
        revealed: Vec<CardId>,
    },
    /// Guozhan: spend a 珠联璧合 (perfect pair) boon mark to draw two cards, emulating a
    /// 【无中生有】 effect. The mark is consumed from the player's `marks` list.
    /// ABI 641 (ADR 0604).
    GuozhanUseZhulianBihe {
        /// Player spending the mark.
        player: PlayerId,
    },
    /// 欢乐卡牌【手里剑】使用结算：目标弃一张装备牌或受 1 点伤害。
    /// ABI 642 (ADR 0607).
    ResolveShoulijian {
        /// Card user.
        player: PlayerId,
        /// Target of the card.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 取消使用牌：成为其他角色使用牌的目标时，取消该牌对当前角色生效，并获取对
    /// 其使用的实体牌（gwent 杜达元帅1 特殊卡机制）。
    /// ABI 643 (ADR 0608).
    CancelIncomingCardUse {
        /// Card holder being targeted.
        player: PlayerId,
        /// Card user whose use is cancelled.
        source: PlayerId,
        /// The used card being cancelled.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 择一加技能：玩家在两个技能定义间二选一，将所选技能置入目标（gwent 呼朋
    /// hupeng 择一杜达元帅 1/2 置入目标手牌）。
    /// ABI 644 (ADR 0612).
    ChooseOtherGainOneOfTwoSkills {
        /// Chooser (skill owner).
        player: PlayerId,
        /// Recipient of the chosen skill.
        target: PlayerId,
        /// First skill definition option.
        first_def: SkillDefId,
        /// Second skill definition option.
        second_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 叶舞（gwent）：弃置所选手牌（费用），然后按种子 RNG 循环——随机敌人随机弃其
    /// 一张「he」牌，颜色与费用相同则重复；每弃敌方手牌得一张雾妖（wuyao_def），每弃
    /// 敌方装备牌得一张狼（lang_def）。雾妖/狼以技能近似（造卡入口）。
    /// ABI 645 (ADR 0627).
    ResolveGwyewuChain {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// The hand card paid as cost (selected by the active skill).
        cost_cards: Vec<CardId>,
        /// 雾妖 skill definition (gained per enemy hand card discarded).
        wuyao_def: SkillDefId,
        /// 狼 skill definition (gained per enemy equip card discarded).
        lang_def: SkillDefId,
    },
    /// Reset one skill instance's per-turn / per-phase usage and trigger counters, implementing
    /// upstream `player.refreshSkill()`. Used by locked clan skills such as 中流 (clanzhongliu).
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    RefreshSkill {
        /// Skill instance to refresh.
        skill: SkillId,
    },
    /// Attach a persistent content tag (gaintag) to each card. The tag value is a stable id
    /// (typically the owning skill's definition id) so multiple skills' tags never collide.
    /// Implements upstream `player.addGaintag(cards, tag)`.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    TagCard {
        /// Cards to tag.
        cards: Vec<CardId>,
        /// Stable tag id (usually a skill definition id).
        tag: u32,
    },
    /// Remove a persistent content tag from each card.
    /// Implements upstream `player.removeGaintag(tag, cards)`.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    UntagCard {
        /// Cards to untag.
        cards: Vec<CardId>,
        /// Stable tag id to remove.
        tag: u32,
    },
    /// 神君 (clanshenjun): reveal and gaintag every hand card sharing the event
    /// card's effective kind. Implements upstream 同名牌展示 + addGaintag.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ClanshenjunReveal {
        /// Skill owner.
        player: PlayerId,
        /// The card whose kind is matched (the trigger's used card).
        event_card: CardId,
        /// Skill instance whose definition id becomes the tag.
        skill: SkillId,
    },
    /// 神君 (clanshenjun) phase-end use: open the tagged-card + target choice
    /// and play the selected card as itself.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ClanshenjunUse {
        /// Skill owner.
        player: PlayerId,
        /// Skill instance.
        skill: SkillId,
    },
    /// Write `value` under `key` in one skill instance's keyed content storage map.
    /// Implements upstream `player.storage.<skill>[key] = value`.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    StorageSet {
        /// Skill instance whose storage is updated.
        skill: SkillId,
        /// Storage key (content-defined, e.g. a packed suit+category code).
        key: u32,
        /// Storage value.
        value: u32,
    },
    /// Draw `count` cards into the hand of each listed player. Implements upstream
    /// `game.asyncDraw([player, ...event.targets])`. The VM host fills the player list with the
    /// event player + the declared event targets. Used by clanxsyingxiang's cross-player draw.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    DrawAllEventTargets {
        /// Players to draw (event player + declared targets, in seat order).
        players: Vec<PlayerId>,
        /// Count per player.
        count: u32,
    },
    /// The previous round fully ended at the seat-0 turn rollover.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable. Engine-emitted
    /// only (no DSL surface); wakes `round_ended` triggers.
    RoundEnded {
        /// The round that just ended.
        round: u32,
        /// Player whose turn ended last in the old round.
        current: PlayerId,
    },
    /// A new round began at the seat-0 turn rollover.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable. Engine-emitted
    /// only (no DSL surface); wakes `round_started` triggers.
    RoundStarted {
        /// The round that just began.
        round: u32,
        /// First player of the new round.
        current: PlayerId,
    },
    /// 迂志 round-start: the owner chooses one hand card to reveal; the resolver reveals +
    /// gaintags it, draws its name length (tagging the draws), and records round draw counts.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ClanyuzhiReveal {
        /// Skill owner.
        player: PlayerId,
        /// 迂志 skill instance.
        skill: SkillId,
    },
    /// 迂志 round-end: discard every gaintagged hand card, then if this round's 迂志 draws
    /// exceeded the previous round's (both positive) or the round's used-card count, the owner
    /// loses 保族 or takes 1 thunder damage.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ClanyuzhiRoundEnd {
        /// Skill owner.
        player: PlayerId,
        /// 迂志 skill instance.
        skill: SkillId,
    },
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
}
