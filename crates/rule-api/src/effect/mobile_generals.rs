//! Mobile generals effect definitions.

macro_rules! variants {
    (ResolveMbshanjia [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveMbzhuguo [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveMobiledanshou [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveMbqingdao [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveMbxuye [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveMbzhuji [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveMbzhonghui [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveMbdiancai [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveMbzhenann [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 专属状态机 (批 276): mbzhennann 镇南 — when targeted by a trick whose
    /// target count exceeds max(1, the user's hp), discard one card and deal
    /// 1 damage to any character.
    ResolveMbzhenann {
        /// Skill owner (a target of the trick).
        player: PlayerId,
    },
        ] }
    };
    (ResolveMbbeiming [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
}
pub(super) use variants;
