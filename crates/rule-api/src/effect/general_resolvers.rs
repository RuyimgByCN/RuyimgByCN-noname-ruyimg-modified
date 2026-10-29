//! General resolvers effect definitions (single-shot; multi-stage resolvers live in `staged_resolvers`).

macro_rules! variants {
    (ResolveJianrou [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Discard two cards to prevent the bound damage for 剑柔.
    ResolveJianrou {
        /// Damaged skill owner.
        player: PlayerId,
        /// Engine-owned pre-resolution damage id.
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveChegu [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveYaoChenwei [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDingpin [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveBeigeConfigured [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDingpan [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveBeizhu [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveLianzhou [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveJieliangGain [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveLulve [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveSibian [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveGongjian [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveWnQifeng [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Resolve offline 杨秋「齐锋」 using the owner's current-turn lost-card count. ABI 496.
    ResolveWnQifeng {
        /// Skill owner and virtual Sha user.
        player: PlayerId,
        /// Player whose turn is ending.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveXizhen [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveXuxieAdd [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 虚泄结算：出牌阶段结束时若存在体力上限大于你的角色，恢复 1 点体力上限。
    /// ABI 483（第二阶段）。
    ResolveXuxieAdd {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveCuijin [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveYuli [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveMantianguohai [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Internal continuation for 【瞒天过海】 after a physical gain/give move.
    ResolveMantianguohai {
        /// Card user.
        player: PlayerId,
        /// Zero advances the per-target trick queue; one advances the give-back queue.
        stage: u8,
        /// Remaining independent-trick queue payload or remaining give targets.
        payload: Vec<u32>,
    },
        ] }
    };
    (ResolveZuilun [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveXiongshu [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveChenglve [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveXiongluan [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveShicai [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveBianyue [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveShoulijian [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveQiluan [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveQianlong [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 专属状态机 (批 241): qianlong 潜龙 — after damage, view the top three
    /// deck cards and gain up to lost-HP count, the rest go to deck bottom.
    ResolveQianlong {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveYixiang [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 专属状态机 (批 288): yixiang 义襄 — randomly gain a basic card from
    /// the deck that is not already in the owner\'s hand.
    ResolveYixiang {
        /// Skill owner.
        player: PlayerId,
    },
        ] }
    };
    (ResolveChulao [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 专属状态机 (批 274): chulao 黜老 — discard one card, then for each
    /// selected target of a distinct group the owner picks one of their cards
    /// to discard; spade discards draw (owner from cost, target from pick).
    ResolveChulao {
        /// Skill owner.
        player: PlayerId,
        /// Selected targets (distinct groups, at least one).
        targets: Vec<PlayerId>,
    },
        ] }
    };
    (ResolveZhengjian [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
}
pub(super) use variants;
