//! Turn flow effect definitions.

macro_rules! variants {
    (SetTurnState [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (TurnOver [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Toggle a player's character card between face-up and face-down.
    ///
    /// A face-down player skips their next whole turn and returns face-up.
    TurnOver {
        /// Player whose character card is toggled.
        player: PlayerId,
    },
        ] }
    };
    (QueueOpeningExtraPlayPhase [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ClearPhaseGiftTarget [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Clear this skill instance's delayed Play-phase gift target.
    /// Appended for serialized ordinal stability.
    ClearPhaseGiftTarget {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (SkipCurrentPhase [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Skip the owner's current non-idle phase and enter the next phase.
    /// Appended for serialized ordinal stability.
    SkipCurrentPhase {
        /// Current phase owner.
        player: PlayerId,
    },
        ] }
    };
    (BanActiveUntilTurnEnd [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 把技能实例的 `active_turn_banned` 置位，禁用该主动技直到当前回合结束。
    BanActiveUntilTurnEnd {
        /// 技能拥有者。
        player: PlayerId,
        /// 被禁用的主动技实例。
        skill: SkillId,
    },
        ] }
    };
    (SkipNextPlayPhase [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Mark the current living player's upcoming Play phase to be skipped.
    ///
    /// The request is valid only before Play in that player's current turn. Appended for
    /// serialized stability.
    SkipNextPlayPhase {
        /// Current turn player.
        player: PlayerId,
    },
        ] }
    };
    (RecordSelectedPlayPhaseCategory [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Record the selected card's effective category for the current play phase.
    ///
    /// Appended for serialized stability.
    RecordSelectedPlayPhaseCategory {
        /// Selected physical card, captured from the authoritative active invocation.
        card: CardId,
        /// Originating active skill instance.
        skill: SkillId,
    },
        ] }
    };
    (GiveSelectedRecoverAfterPhaseTotalTwo [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (SkipNextJudgePhase [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Mark a living player's next judgment phase to be skipped.
    ///
    /// Appended for serialized ordinal stability.
    SkipNextJudgePhase {
        /// Player whose next judgment phase is skipped.
        player: PlayerId,
    },
        ] }
    };
    (FinishCurrentPhaseAbortResolution [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (SkipJudgeAndDrawPhases [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (SkipNextDiscardPhase [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 跳过本回合目标角色的弃牌阶段（SKIP_DISCARD 标记；出牌阶段后由回合机消费）。
    /// ABI 471。
    SkipNextDiscardPhase {
        /// Target whose discard phase is skipped.
        player: PlayerId,
    },
        ] }
    };
    (SetDrawPhaseFromBottom [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 寸目（shenhua 许攸，主公技）触发标记：当前摸牌阶段从牌堆底摸牌。ABI 589。
    /// Appended for serialized ordinal stability.
    SetDrawPhaseFromBottom {
        /// Skill owner.
        player: PlayerId,
    },
        ] }
    };
    (RegisterTurnEndDebate [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 登记回合末议事（ABI 663，jsrg yaoyan）：逐角色选择后登记为发起者的
    /// 回合末议事参与者。
    RegisterTurnEndDebate {
        /// 发起者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (QueueExtraTurn [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (QueueExtraPhases [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
}
pub(super) use variants;
