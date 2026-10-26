//! Online generals effect definitions.

macro_rules! variants {
    (ResolveDcqiaomeng [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveStddechong [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveStdjujian [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDctingxian [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDcnuchen [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDcmanzhi [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 蛮智（huicui 孟优）：准备/结束阶段二选一：①令一名其他角色交给你至多两张牌，
    /// 其视为使用一张无距离限制的【杀】；②你获得一名其他角色区域内的至多两张牌，
    /// 交给其等量牌后摸一张。每分支每回合一次（技能 mark 记录）。ABI 475。
    ResolveDcmanzhi {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveDckuizhen [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDcmoyu [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 魔狱（huicui 裴元绍）：出牌阶段限一次（每目标每回合一次），获得目标 1（或 2）
    /// 张手牌/装备/判定牌，其视为对你使用一张无距离限制的【杀】；若你因此受伤，
    /// 本回合禁用本技，否则下次多拿一张。ABI 481。
    ResolveDcmoyu {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveDcchiying [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDcxiangshu [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDcyingtu [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDccunsi [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDcjue [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDcshaowei [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDcctjiuxian [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDcSbZhangxiu [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveOllinjie [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDcniji [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveOlzhijie [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveStarcuxiaWeiming [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveFriendbiwei [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDcanxu [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDcbingji [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
}
pub(super) use variants;
