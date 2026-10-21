//! Ordered EffectRequest variants, starting with `ResolveHoufeng`.

macro_rules! variants {
    ([$($tail:ident),*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail),*] [
            $($variants)*
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
        ] }
    };
}
pub(super) use variants;
