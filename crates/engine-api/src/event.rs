//! Domain events produced by committed resolution segments.

use crate::ids::{CardId, PlayerId, SkillId, ZoneId};
use serde::{Deserialize, Serialize};

fn no_player() -> PlayerId {
    PlayerId::NONE
}

/// Rule-level reason for a card-zone movement.
///
/// The destination zone alone is not sufficient: a card may enter the discard pile because it
/// was discarded, used, responded with, judged, or cleaned up by another rule. Content triggers
/// must use this reason instead of guessing from zones.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardMoveReason {
    /// Legacy or generic movement whose semantic cause was not specified.
    #[default]
    Unspecified,
    /// A rule-defined discard paid or imposed by a player.
    Discard,
    /// A player voluntarily transferred cards from their personal zone to another player's hand.
    ///
    /// Appended so existing serialized enum ordinals remain stable.
    Give,
    /// Cards voluntarily placed into the discard pile to draw replacements.
    ///
    /// Appended so existing serialized enum ordinals remain stable.
    Recast,
    /// Physical cards consumed by a use or response, including extra view-as sources.
    ///
    /// Appended so existing serialized enum ordinals remain stable.
    UseOrRespond,
    /// Physical cards consumed by a card use. Appended for ordinal stability.
    Use,
    /// Physical cards consumed by a response or save. Appended for ordinal stability.
    Respond,
    /// A physical fire Sha transferred by 势程普「督作」.
    ///
    /// Appended for serialized ordinal stability.
    Potduzuo,
    /// Cards left over after all effective targets of 【五谷丰登】 have chosen.
    WuguRemained,
}

impl CardMoveReason {
    /// Whether this movement consumed a card through either legacy-combined or precise action.
    pub const fn is_use_or_respond(self) -> bool {
        matches!(self, Self::UseOrRespond | Self::Use | Self::Respond)
    }
}

/// Rule-relevant domain event (presentation maps these separately).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainEvent {
    /// Match started.
    GameStarted {
        /// Seat count.
        player_count: u8,
    },
    /// Cards drawn.
    CardsDrawn {
        /// Drawing player.
        player: PlayerId,
        /// Cards drawn (order preserved).
        cards: Vec<CardId>,
    },
    /// Cards moved between zones.
    CardsMoved {
        /// Cards in move order.
        cards: Vec<CardId>,
        /// Source zone.
        from: ZoneId,
        /// Destination zone.
        to: ZoneId,
        /// Semantic cause of the movement; never infer this from `to` alone.
        #[serde(default)]
        reason: CardMoveReason,
    },
    /// Card used (played as use).
    CardUsed {
        /// User.
        player: PlayerId,
        /// Card.
        card: CardId,
        /// Declared targets in seat-independent command order.
        #[serde(default)]
        targets: Vec<PlayerId>,
    },
    /// One complete card use finished after every target, response, damage, dying, and nested
    /// choice belonging to it settled.
    ///
    /// Appended so existing serialized event ordinals remain stable. ABI 503.
    CardResolved {
        /// Card user.
        source: PlayerId,
        /// Physical card or engine materialization carrying the logical use.
        card: CardId,
        /// Effective card-kind tag captured when the use began.
        kind: u8,
        /// Complete declared target list.
        targets: Vec<PlayerId>,
        /// Targets that actually received damage from this exact card use.
        damaged_targets: Vec<PlayerId>,
        /// Physical materials consumed by the logical use; virtual carriers are omitted.
        #[serde(default)]
        source_cards: Vec<CardId>,
        /// Whether every physical material originated in the user's hand.
        #[serde(default)]
        all_sources_were_hand: bool,
    },
    /// Damage dealt.
    DamageDealt {
        /// Source (may be none for system).
        source: PlayerId,
        /// Target.
        target: PlayerId,
        /// Amount (integer HP).
        amount: u32,
        /// Whether this was lose-HP style.
        is_lose_hp: bool,
        /// Damage nature: 0=normal, 1=fire, 2=thunder.
        #[serde(default)]
        nature: u8,
    },
    /// HP recovered.
    HpRecovered {
        /// Recovery source (`NONE` for unattributed recovery).
        #[serde(default = "no_player")]
        source: PlayerId,
        /// Target.
        player: PlayerId,
        /// Amount.
        amount: u32,
    },
    /// Max HP changed.
    MaxHpChanged {
        /// Player.
        player: PlayerId,
        /// New max HP.
        new_max: u32,
    },
    /// Player entered dying state.
    EnteredDying {
        /// Player.
        player: PlayerId,
        /// Damage source that caused dying, or none for non-damage HP loss/manual entry.
        #[serde(default = "none_player_id")]
        source: PlayerId,
    },
    /// Player left dying state after being saved.
    LeftDying {
        /// Player.
        player: PlayerId,
    },
    /// Player killed / eliminated.
    PlayerKilled {
        /// Victim.
        player: PlayerId,
        /// Killer if any.
        killer: PlayerId,
        /// Hand and equipment cards owned immediately before death cleanup.
        #[serde(default)]
        hand_equip_cards: Vec<CardId>,
    },
    /// Turn / phase transition.
    TurnStateChanged {
        /// Current player.
        current: PlayerId,
        /// Phase tag.
        phase: TurnPhaseTag,
    },
    /// A phase ended before the following `TurnStateChanged` event.
    PhaseEnded {
        /// Player whose phase ended.
        current: PlayerId,
        /// Phase that ended.
        phase: TurnPhaseTag,
        /// Turn-scoped bitset of players who dealt damage before this phase ended.
        #[serde(default)]
        damage_sources: u32,
        /// Turn-scoped bitset of players who received damage before this phase ended.
        #[serde(default)]
        damaged_players: u32,
        /// Whether the ending turn player used any card targeting another player this turn.
        #[serde(default)]
        card_targeted_other: bool,
        /// Whether the ending turn player dealt damage to another player this turn.
        #[serde(default)]
        current_dealt_damage_other: bool,
        /// Phase-scoped bitset of players named as targets by a used card.
        #[serde(default)]
        phase_targeted_players: u32,
        /// Phase-scoped bitset of players whose actual HP value changed.
        #[serde(default)]
        phase_hp_changed_players: u32,
    },
    /// Skill added.
    SkillAdded {
        /// Owner.
        player: PlayerId,
        /// Skill instance.
        skill: SkillId,
    },
    /// Skill removed.
    SkillRemoved {
        /// Owner.
        player: PlayerId,
        /// Skill instance.
        skill: SkillId,
    },
    /// Mark added / stack changed.
    MarkChanged {
        /// Player.
        player: PlayerId,
        /// Mark definition raw id.
        mark_def: u32,
        /// New count.
        count: i32,
    },
    /// Cards revealed to some observers (full visibility handled in projection).
    CardsRevealed {
        /// Cards.
        cards: Vec<CardId>,
        /// To whom (empty = public).
        to: Vec<PlayerId>,
    },
    /// Choice published.
    ChoiceRequested {
        /// Choice id raw.
        choice_id: u32,
        /// Observer.
        observer: PlayerId,
    },
    /// Match outcome finalized.
    MatchEnded {
        /// Outcome tag.
        outcome: crate::outcome::MatchOutcome,
    },
    /// Generic content / system rule event.
    RuleNote {
        /// Stable event code.
        code: u32,
        /// Payload integers (ordered).
        payload: Vec<u32>,
    },
    /// Card played as a response rather than used on targets.
    CardResponded {
        /// Responding player.
        player: PlayerId,
        /// Player whose card is being answered; none when no response relation exists.
        #[serde(default = "no_player")]
        respond_to: PlayerId,
        /// Physical card; none for a virtual response such as 八卦.
        card: CardId,
        /// Logical engine card-kind tag after view-as conversion.
        response_kind: u8,
    },
    /// A phase was skipped before entering it.
    PhaseSkipped {
        /// Player whose phase was skipped.
        current: PlayerId,
        /// Phase that was skipped.
        phase: TurnPhaseTag,
    },
    /// A player's character card changed between face-up and face-down.
    PlayerTurnedOver {
        /// Affected player.
        player: PlayerId,
        /// New state: true means face-down.
        turned_over: bool,
    },
    /// A Duel finished because one participant failed to respond with Sha.
    ///
    /// Appended so existing serialized event ordinals remain stable.
    DuelResolved {
        /// Physical or materialized virtual card that carried the Duel.
        card: CardId,
        /// Card-resolution flags; currently records whether Duel damage actually committed.
        card_flags: u8,
        /// Player who used the Duel.
        initiator: PlayerId,
        /// Original Duel target.
        target: PlayerId,
        /// Participant who dealt the Duel damage.
        winner: PlayerId,
        /// Participant who failed to respond and received the Duel damage.
        loser: PlayerId,
    },
    /// One equipment slot became disabled.
    ///
    /// Appended so existing serialized event ordinals remain stable.
    EquipSlotDisabled {
        /// Affected player.
        player: PlayerId,
        /// Stable slot tag (`1` weapon through `5` treasure).
        slot: u8,
    },
    /// A player's character card changed between linked and unlinked.
    ///
    /// Appended so existing serialized event ordinals remain stable.
    PlayerLinkedChanged {
        /// Affected player.
        player: PlayerId,
        /// New state: true means linked.
        linked: bool,
    },
    /// A logical Sha was fully dodged after the target supplied every required Shan.
    ///
    /// Appended so existing serialized event ordinals remain stable.
    ShaDodged {
        /// Sha user.
        source: PlayerId,
        /// Target who dodged the Sha.
        target: PlayerId,
        /// Current logical Sha carrier (which may be a view-as source card), or `NONE` for a
        /// source-free virtual Sha.
        card: CardId,
    },
    /// A new player's whole turn has begun, before any phase in that turn starts.
    ///
    /// Appended so existing serialized event ordinals remain stable.
    TurnBegan {
        /// Player whose turn is opening.
        player: PlayerId,
    },
    /// One legal target was appended while a card use was still resolving.
    ///
    /// Appended so existing serialized event ordinals remain stable. This wakes only
    /// per-target trigger contexts; it does not emit a second card-use event.
    CardTargetAdded {
        /// Card user.
        player: PlayerId,
        /// Physical card currently resolving.
        card: CardId,
        /// Newly appended target.
        target: PlayerId,
        /// Complete target list after the append.
        targets: Vec<PlayerId>,
    },
    /// One logical Sha use has fully resolved, including every target and nested Shan response.
    ///
    /// Appended so existing serialized event ordinals remain stable. `response_cards` contains
    /// only physical cards that were played as Shan for this exact Sha use.
    ShaResolved {
        /// Sha user.
        source: PlayerId,
        /// Physical card carrying the logical Sha, or an engine materialization for a virtual Sha.
        card: CardId,
        /// Complete declared target list for this Sha use.
        targets: Vec<PlayerId>,
        /// Physical source cards played as Shan while resolving this Sha.
        response_cards: Vec<CardId>,
        /// Stable card-use flags, including whether this Sha dealt damage.
        #[serde(default)]
        card_flags: u8,
    },
    /// The suspended card use changed user after a skill-mediated hand-card gift.
    ///
    /// Appended so existing serialized event ordinals remain stable.
    CardUserChanged {
        /// Previous card user.
        previous: PlayerId,
        /// New card user.
        player: PlayerId,
        /// Physical card whose use remains suspended.
        card: CardId,
    },
    /// A two-player hand comparison has started after the first participant selected a card but
    /// before the second participant's card is finalized.
    ///
    /// Appended so existing serialized event ordinals remain stable.
    HandCompareStarted {
        /// First comparison participant.
        player: PlayerId,
        /// Second comparison participant.
        target: PlayerId,
        /// Initially selected card of the first participant.
        player_card: CardId,
    },
    /// A two-player hand comparison has completed and both physical cards have left hand.
    ///
    /// Appended so existing serialized event ordinals remain stable.
    /// A debate settled (ADR 0644 P2). Result: 0 none, 1 red, 2 black.
    DebateResolved {
        /// Initiator.
        initiator: PlayerId,
        /// Red-opinion players.
        #[serde(default)]
        red: Vec<PlayerId>,
        /// Black-opinion players.
        #[serde(default)]
        black: Vec<PlayerId>,
        /// Result code: 0 none, 1 red, 2 black.
        result: u8,
    },
    /// A debate opened, before opinions are shown (ADR 0644 P3 debateShowOpinion
    /// window). Skills may pin opinions in response.
    DebateStarted {
        /// Initiator.
        initiator: PlayerId,
        /// Participants whose opinions will be shown.
        #[serde(default)]
        participants: Vec<PlayerId>,
    },
    /// Both participants finished a hand compare.
    HandCompareFinished {
        /// First comparison participant.
        player: PlayerId,
        /// Second comparison participant.
        target: PlayerId,
        /// Final card used by the first participant.
        player_card: CardId,
        /// Final card used by the second participant.
        target_card: CardId,
        /// Strict winner, or `NONE` for a tie.
        winner: PlayerId,
    },
    /// One judgment completed after every rewrite opportunity settled.
    ///
    /// Appended so existing serialized event ordinals remain stable. `cause_card` is the
    /// delayed trick or equipment that initiated the judgment, or `NONE` for a skill judgment.
    JudgmentResolved {
        /// Player whose judgment completed.
        player: PlayerId,
        /// Physical card carrying the final judgment result, including a rewrite card.
        result_card: CardId,
        /// Final effective suit for the judged player.
        suit: crate::view::Suit,
        /// Final rank.
        rank: u8,
        /// Delayed trick/equipment that caused the judgment, or `NONE`.
        cause_card: CardId,
    },
    /// One disabled copy of an equipment slot was restored.
    ///
    /// Appended so existing serialized event ordinals remain stable.
    EquipSlotEnabled {
        /// Player causing the restoration (`NONE` when unattributed).
        source: PlayerId,
        /// Affected player.
        player: PlayerId,
        /// Stable slot tag (`1` weapon through `5` treasure).
        slot: u8,
    },
    /// One player gave physical cards to another player's hand.
    ///
    /// Appended so existing serialized event ordinals remain stable.
    CardsGiven {
        /// Attributed giver.
        giver: PlayerId,
        /// Recipient.
        recipient: PlayerId,
        /// Physical cards transferred.
        cards: Vec<CardId>,
    },
    /// A positive damage proposal was cancelled or reduced to zero before changing HP.
    ///
    /// Appended so existing serialized event ordinals remain stable.
    DamagePrevented {
        /// Original damage source, or `NONE` for unattributed damage.
        source: PlayerId,
        /// Intended damage target.
        target: PlayerId,
        /// Current turn player when prevention completed, or `NONE` between turns.
        current: PlayerId,
    },
    /// The previous round has fully ended: every player's turn of round N completed and the
    /// turn order is about to roll into round N+1.
    ///
    /// Appended so existing serialized event ordinals remain stable. Emitted at the round
    /// rollover before `RoundStarted`; the player whose turn just ended is in `current`.
    RoundEnded {
        /// The round that just ended.
        round: u32,
        /// Player whose turn ended last in the old round.
        current: PlayerId,
    },
    /// A new round has begun: the turn order rolled from round N to N+1 and the first player's
    /// turn is about to start.
    ///
    /// Appended so existing serialized event ordinals remain stable. Emitted at the round
    /// rollover after `RoundEnded`.
    RoundStarted {
        /// The round that just began.
        round: u32,
        /// First player of the new round.
        current: PlayerId,
    },
}

fn none_player_id() -> PlayerId {
    PlayerId::NONE
}

/// Coarse phase tags exposed on the public API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnPhaseTag {
    /// Round preparation.
    Prepare,
    /// Judge phase.
    Judge,
    /// Draw phase.
    Draw,
    /// Play phase.
    Play,
    /// Discard phase.
    Discard,
    /// End phase.
    End,
    /// Between turns / not in turn.
    Idle,
}
