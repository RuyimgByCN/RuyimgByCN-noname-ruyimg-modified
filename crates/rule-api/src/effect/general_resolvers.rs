//! General resolvers effect definitions.

macro_rules! variants {
    (ResolveGuanchaoRank [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveJiexun [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
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
    (ResolveYoudi [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
    (ResolveDaoji [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
    (ResolveXunyi [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveLiubing [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
    (ResolveZhengrong [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
    (ResolveJuzhan [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveQingyu [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
    (ResolveTongduo [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveRuanji [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
    (ResolveZhangjian [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
    (QieyiPhaseStart [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
    (ResolveQiantun [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveZhongyan [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
    (ResolveZhongjie [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDingce [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
    (ResolveDuoduan [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
    (ResolveQinmian [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
}
pub(super) use variants;
