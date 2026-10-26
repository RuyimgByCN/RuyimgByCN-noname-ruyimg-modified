//! Remastered generals effect definitions.

macro_rules! variants {
    (ResolveRebingyi [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveRejizhi [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveXinanguo [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveOldAnxu [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveOldQinqing [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveOldHuisheng [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveJunkguixin [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveNewYajiao [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveXinfuGuolun [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDecadezhenjun [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveNewReluoyi [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveRetianxiang [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveXinfuLianpian [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveRefuhai [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Resolve mobile 卫温诸葛直「浮海」's seat-ordered private binary choices. ABI 493.
    ResolveRefuhai {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveXinfuQinguo [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveXinxingluan [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveRekuanshi [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 宽释（huicui 阚泽）：结束阶段选择一名其他角色标记；其后该角色于本回合内第
    /// 二次受到伤害后回复 1 点体力（每回合一次）。ABI 480。
    ResolveRekuanshi {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveXinkuangfu [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveRekuanshiResolve [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveXinmingshi [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveReleiji [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveXinyangjie [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveRechuhai [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveRehengjiang [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveXinfuQiai [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 专属状态机 (批 291): xinfu_qiai 求援 — once per game, while dying,
    /// every other player in seat order hands the owner one hand/equip card.
    ResolveXinfuQiai {
        /// Dying skill owner receiving the cards.
        player: PlayerId,
    },
        ] }
    };
    (ResolveReliangyin [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
}
pub(super) use variants;
