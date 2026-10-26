//! Sp generals effect definitions.

macro_rules! variants {
    (SpwanweiActive [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveSpXiaoguo [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveSpxizhan [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveSpsongshu [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveHfjieying [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveSpjiedao [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveSpfengyin [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveGzjinfa [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveNsdingzhou [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveSpcunsi [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 存嗣：拥有者翻面，从牌堆/弃牌堆取一张【杀】交给目标。
    ///
    /// Appended for serialized ordinal stability.
    ResolveSpcunsi {
        /// Skill owner.
        player: PlayerId,
        /// Recipient of the Sha.
        target: PlayerId,
    },
        ] }
    };
    (ResolveDbzhuifeng [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveSpmiaolve [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveSpyajun [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveSpqiai [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveSxrmqiewang [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveNsshuaiyan [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveSpolaige [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 专属状态机 (批 293): spolaige 觉醒 — after the match's second dying
    /// window, exchange 西向 for 朔北 and draw/recover up to max HP.
    ResolveSpolaige {
        /// Awakening skill owner.
        player: PlayerId,
    },
        ] }
    };
}
pub(super) use variants;
