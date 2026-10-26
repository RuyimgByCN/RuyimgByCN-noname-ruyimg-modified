//! Events effect definitions.

macro_rules! variants {
    (EmitDebateResolved [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (CardResponded [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (EmitDuelResolved [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ShaDodged [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (EmitShaResolved [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (EmitHandCompareFinished [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (EmitCardResolved [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (RoundEnded [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (StartDebateAllOthers [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveTurnEndDebates [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 回合末议事结算（ABI 663）：对登记名单发起议事并收尾。
    ResolveTurnEndDebates {
        /// 发起者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
}
pub(super) use variants;
