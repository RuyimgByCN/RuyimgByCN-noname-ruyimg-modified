//! Ordered EffectRequest variants, starting with `ResolveDcyingtu`.

macro_rules! variants {
    ([$($tail:ident),*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail),*] [
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
    /// Choose another living player other than the damage source, gain one deterministic-random
    /// gainable hand/equipment card, reveal it, and use it when it is an equipment card. ABI 499.
    ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
        /// Damaged skill owner.
        player: PlayerId,
        /// Damage source excluded from the target choice; may be `NONE`.
        source: PlayerId,
        /// Randomly gained equipment awaiting ordinary use.
        card: CardId,
        /// Resolution stage: 0 = choose target; 1 = use gained equipment if still legal.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// 双雄（shenhua 颜良文丑）：摸牌阶段可选改为判定，获得判定牌并记录判定颜色，
    /// 本回合非该颜色手牌可当【决斗】使用。ABI 511。
    ResolveShuangxiong {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 直谏（shenhua 张昭张纮）：出牌阶段限一次，将一张手牌区装备牌交给一名可装备
    /// 该牌的其他角色，然后摸一张。ABI 512。
    ResolveZhijian {
        /// Skill owner and giver.
        player: PlayerId,
        /// Exact selected equipment card from hand.
        card: CardId,
        /// Choice barrier stage: 0 choose target, 1 commit give.
        stage: u8,
        /// Chosen recipient.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 固政（shenhua 张昭张纮）：其他角色弃牌阶段结束后，归还其一张某阶段弃置且仍处
    /// 弃牌堆的牌，并可选获得其余弃牌。ABI 513。
    ResolveGuzheng {
        /// Skill owner.
        player: PlayerId,
        /// The player whose discard phase ended.
        event_player: PlayerId,
        /// Choice barrier stage: 0 choose return card, 1 choose take-rest.
        stage: u8,
        /// Snapshot of the discard-phase lost cards (player id + card id pairs).
        cards: Vec<u32>,
        /// Card chosen to return.
        return_card: CardId,
        /// Whether the owner takes the remaining phase discards.
        take_rest: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// 势程普「督作」「避憾」复合。ABI 515。
    ResolvePotchengpu {
        /// Skill owner.
        player: PlayerId,
        /// Selected recipient, damage target, or hand-discard role.
        target: PlayerId,
        /// Zero/one resolve 督作; two/three resolve 避憾.
        stage: u8,
        /// Bound damage id for 避憾.
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势臧洪「拒孤」：摸二并按本回合发动次数强制弃牌。ABI 516。
    ResolvePotjugu {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势曹真「伺锋」分配、逐牌消耗与阶段末结算。ABI 517。
    ResolvePotsifeng {
        /// Skill owner.
        player: PlayerId,
        /// Expansion holder or event card user.
        target: PlayerId,
        /// Resolved card for color comparison.
        card: CardId,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势陈群「法恩」「定品」复合。ABI 518。
    ResolvePotchenqun {
        /// Skill owner.
        player: PlayerId,
        /// Current card user, or unused for 定品.
        target: PlayerId,
        /// Zero resolves 法恩; one resolves 定品.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势陈到「往烈」选牌、精确牌结算与阶段清理。ABI 519。
    ResolvePotwanglie {
        /// Skill owner.
        player: PlayerId,
        /// Armed or resolved physical card.
        card: CardId,
        /// Phase-local self-target-only helper definition.
        helper_skill_def: SkillDefId,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势陈到「弘毅」初始/伤害加标记及准备、结束阶段配对结算。ABI 519。
    ResolvePothongyi {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势娄圭「灌沙」全牌替换与本回合手牌上限增益。ABI 520。
    ResolvePotguansha {
        /// Skill owner.
        player: PlayerId,
        /// Temporary hand-limit helper definition.
        helper_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势娄圭「急御」异类获牌、耗尽检测与最多两次重置。ABI 520。
    ResolvePotjiyu {
        /// Skill owner.
        player: PlayerId,
        /// Selected hand card for stage zero.
        card: CardId,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// 兵势夏侯霸「砺兵」循环增益、伤害牌加伤与重置。ABI 522。
    ResolvePotlibing {
        /// Skill owner.
        player: PlayerId,
        /// Used or resolved physical card.
        card: CardId,
        /// Zero handles card resolution; one handles card use.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 兵势夏侯霸「破袭」逐张强制使用弃置的实体杀。ABI 522。
    ResolvePotpoxi {
        /// Skill owner and Sha user.
        player: PlayerId,
        /// Current movement event cards, or remaining cards after one forced use.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 兵势张任「伏暗」降攻击范围、记花色与回合结束追击。ABI 525。
    ResolvePotfuan {
        /// Skill owner.
        player: PlayerId,
        /// Movement event card.
        card: CardId,
        /// Resolution stage.
        stage: u8,
        /// Helper skill that lowers the owner's attack range.
        range_helper_def: SkillDefId,
        /// 引弦 helper skill definition granted on suit match.
        yinxian_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 兵势张任「引弦」最远目标杀增伤与回合外用牌加攻击范围。ABI 525。
    ResolvePotyinxian {
        /// Skill owner.
        player: PlayerId,
        /// Movement event card.
        card: CardId,
        /// Resolution stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// 从俭（shenhua 张绣）：成为多目标锦囊牌的目标后，交给其中一名其他目标一张
    /// 手牌/装备牌，然后摸一张（所给为装备牌则摸两张）。ABI 524。
    ResolveDrltCongjian {
        /// Skill owner.
        player: PlayerId,
        /// The trick's targets captured at trigger time.
        targets: Vec<PlayerId>,
        /// Card chosen to give.
        card: CardId,
        /// Chosen recipient.
        target: PlayerId,
        /// Choice barrier stage: 0 choose recipient, 1 choose card, 2 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 飞军（shenhua 王平）：出牌阶段限一次弃一张牌，按可用性选“手牌数大于你者交给
    /// 你一张牌”或“装备数大于你者弃置一张装备牌”分支；目标首次被选时置兵略旗标。
    /// ABI 526。
    ResolveNzryFeijun {
        /// Skill owner.
        player: PlayerId,
        /// Chosen target.
        target: PlayerId,
        /// Card the target chose to give or discard.
        card: CardId,
        /// Branch: 0 = hand-give, 1 = equip-discard.
        branch: u8,
        /// Choice barrier stage: 0 branch, 1 target, 2 target card, 3 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 庸肆（shenhua 袁术）：摸牌阶段摸牌数改为势力数；出牌阶段结束时按本回合是否
    /// 造成伤害：未造成伤害摸至体力值，否则授予既有庸肆弃牌阶段状态。ABI 527。
    ResolveDrltYongsi {
        /// Skill owner.
        player: PlayerId,
        /// Zero = draw-phase replacement, one = play-phase-end settlement.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 伪帝（shenhua 袁术，主公技）：弃牌阶段开始把至多 min(需弃置数, 其他群势力
    /// 角色数) 张手牌分别交给等量其他群势力角色。ABI 528。
    ResolveDrltWeidi {
        /// Skill owner.
        player: PlayerId,
        /// Cards chosen to give.
        cards: Vec<CardId>,
        /// Chosen recipients, paired with cards by index.
        targets: Vec<PlayerId>,
        /// Choice barrier stage: 0 cards, 1 targets, 2 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势桓阶「共谋」「正朔」复合。ABI 529。
    ///
    /// Appended for serialized ordinal stability.
    ResolvePothuanjie {
        /// Skill owner.
        player: PlayerId,
        /// Exchange target; NONE while opening the target choice.
        target: PlayerId,
        /// Zero opens 共谋 target choice, one commits 共谋, two resolves 正朔.
        stage: u8,
        /// 奇策 definition temporarily granted to the owner.
        qice_def: SkillDefId,
        /// 看破 definition temporarily granted to the exchange target.
        kanpo_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 烈刃（shenhua 祝融）：以固定受伤害角色为目标启动通用拼点选牌续体；胜者获得
    /// 绑定辅助技能。ABI 531。
    ResolveLieren {
        /// Skill owner and comparing player.
        player: PlayerId,
        /// The damaged player and compare target.
        target: PlayerId,
        /// Helper granted on a strict win.
        win_skill_def: SkillDefId,
        /// Helper granted on a tie or loss.
        lose_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势孙綝「逆固」：依次收取攻击范围内角色可选的一张牌，并记录拒绝人数；
    /// 后续伤害前消耗一次记录使伤害 +1。ABI 532。
    ResolvePotnigu {
        /// Skill owner.
        player: PlayerId,
        /// Current giver during the staged gift sequence.
        target: PlayerId,
        /// Selected gift card, or NONE for a refusal.
        card: CardId,
        /// Stable giver order captured at activation.
        targets: Vec<PlayerId>,
        /// Current giver index.
        index: u8,
        /// Refusal count accumulated so far.
        declined: u8,
        /// Zero starts, one commits one answer, two advances, three buffs damage, four clears.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势孙綝「戮连」：记录手牌来源的完整用牌，结算后横置目标、摸牌并处理
    /// “成势”火焰伤害。ABI 532。
    ResolvePotlulian {
        /// Skill owner.
        player: PlayerId,
        /// Physical carrier of the logical card use.
        card: CardId,
        /// Forced fire-damage target after the choice barrier.
        target: PlayerId,
        /// Zero records card use, one settles, two commits fire damage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势鲁肃「缔盟」：交换两名角色的全部手牌，并结算拥有者的弃牌/摸牌代价选择。
    /// ABI 535。
    ResolvePotdimeng {
        /// Skill owner.
        player: PlayerId,
        /// Exactly two selected players.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势鲁肃「好施」：结束阶段授权其他角色借用拥有者手牌；拥有者下个回合
    /// 开始时清除授权。内部动作 2 结算借牌耗尽手牌后的至三张补牌。ABI 536。
    ResolvePothaoshi {
        /// Skill owner and physical hand owner.
        player: PlayerId,
        /// Authorized borrower for internal refill settlement; none for content actions.
        target: PlayerId,
        /// Zero opens authorization choice, one clears grants, two settles one empty-hand use.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Internal continuation for 【瞒天过海】 after a physical gain/give move.
    ResolveMantianguohai {
        /// Card user.
        player: PlayerId,
        /// Zero advances the per-target trick queue; one advances the give-back queue.
        stage: u8,
        /// Remaining independent-trick queue payload or remaining give targets.
        payload: Vec<u32>,
    },
    /// 屯田（shenhua 邓艾）：回合外失去牌后判定；判定牌不为红桃时移入本技能
    /// 牌堆（田）。ABI 533。
    ResolveTuntian {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 凿险（shenhua 邓艾）：准备阶段本技能牌堆至少 3 张时觉醒，减 1 点体力
    /// 上限并授予急袭。ABI 534。
    ResolveZaoxian {
        /// Skill owner.
        player: PlayerId,
        /// 急袭 definition granted on awakening.
        jixi_def: SkillDefId,
        /// 屯田 definition whose skill pile stores the 田 cards.
        tuntian_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 挑衅（shenhua 姜维）：出牌阶段限一次，选择一名攻击范围含持有者的其他角色，
    /// 其选择对持有者使用一张杀，或令持有者弃置其一张牌。ABI 537。
    ResolveTiaoxin {
        /// Skill owner.
        player: PlayerId,
        /// The challenged player; NONE while opening the target choice.
        target: PlayerId,
        /// Choice barrier stage: 0 open target, 1 open sha-or-refuse, 2 open sha source,
        /// 3 play the sha, 4 open discard card, 5 commit the discard.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 志继（shenhua 姜维）：觉醒技，准备阶段无手牌时摸两张牌、回复 1 点体力、
    /// 减 1 点体力上限并获得观星。ABI 538。
    ResolveZhiji {
        /// Skill owner.
        player: PlayerId,
        /// 观星 definition granted on awakening.
        guanxing_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 再起（shenhua 孟获）：摸牌阶段替换——已受伤时弃摸牌并翻亮牌堆顶 X 张
    /// （X=已损失体力+extra），红桃每张回复 1 点体力，其余获得。ABI 540。
    ResolveZaiqi {
        /// Skill owner.
        player: PlayerId,
        /// Extra revealed cards beyond the lost HP count.
        extra: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 求首（shenhua 孟获，主公技）：南蛮结算后目标死亡或受创目标数超过 3 时，
    /// 所有存活蜀/群角色各摸一张牌。ABI 542。
    ResolveTwqiushou {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势邓艾「屯田/凿险」：维护蓄力上限与蓄力标记，分发红桃牌并按消耗阈值
    /// 从弃牌堆获得锦囊。ABI 543。
    ResolvePottuntian {
        /// Skill owner.
        player: PlayerId,
        /// 0 = gain charge, 1 = expand full capacity, 2 = choose spend, 3 = commit spend.
        action: u8,
        /// Number of charge points committed by action 3.
        amount: u8,
        /// Selected recipients for the heart-card distribution.
        targets: Vec<PlayerId>,
        /// 势凿险 definition used for the threshold rewards.
        zaoxian_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势邓艾「急袭」：记录本回合用牌目标，回合结束弃当前角色一张牌后，
    /// 对任意名已记录角色使用无距离限制的虚拟【顺手牵羊】。ABI 543。
    ResolvePotjixi {
        /// Skill owner.
        player: PlayerId,
        /// Event/current turn player depending on action.
        target: PlayerId,
        /// 0 = reset, 1 = record target, 2 = start, 3 = choose targets, 4 = use trick.
        action: u8,
        /// Selected targets for action 4.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势陈矫「策断」：收集目标攻击范围内角色展示的手牌，并使用多牌【杀】。ABI 544。
    ResolvePotceduan {
        /// Skill owner.
        player: PlayerId,
        /// Active target.
        target: PlayerId,
        /// 0 = collect reveals, 1 = draw after the tracked Sha dealt damage.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势太史慈「酣战/战烈/振锋」状态机。ABI 548。
    ResolvePottaishici {
        /// Skill owner.
        player: PlayerId,
        /// Selected active target, or the current event player when unused.
        target: PlayerId,
        /// 0 = turn reset, 1 = collect discarded Sha, 2 = launch 战烈,
        /// 3 = 酣战, 4 = 振锋, 5 = tracked 战烈 draw;
        /// 6 is the internal post-draw 酣战 continuation.
        action: u8,
        /// Event cards supplied by the VM host for action 1.
        cards: Vec<CardId>,
        /// 酣战 definition whose X source may be rewritten.
        hanzhan_def: SkillDefId,
        /// 战烈 definition whose X source and marks are authoritative.
        zhanlie_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// 黄天（shenhua 张角，主公技）：其他群势力角色出牌阶段限一次可将一张闪/闪电
    /// 交给持有者。ABI 551。
    ResolveHuangtian {
        /// Skill owner (lord).
        player: PlayerId,
        /// The giving Qun player.
        actor: PlayerId,
        /// Choice barrier stage: 0 open card choice, 1 commit the give.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// 好施（shenhua 鲁肃）：摸牌阶段额外摸两张；摸牌阶段结束时手牌>5 则选手牌数
    /// 最少角色给 floor(手牌/2) 张。ABI 555。
    ResolveHaoshi {
        /// Skill owner.
        player: PlayerId,
        /// Choice barrier stage: 0 draw replacement, 1 open target, 2 open cards, 3 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 缔盟（shenhua 鲁肃）：出牌阶段限一次，选两名其他角色，弃置 X 张牌
    /// （X=两者手牌差）后交换其手牌。ABI 556。
    ResolveDimeng {
        /// Skill owner.
        player: PlayerId,
        /// The two chosen targets; NONE while opening the target choice.
        targets: Vec<PlayerId>,
        /// Choice barrier stage: 0 open targets, 1 open discard, 2 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势辛宪英「诫节」：当前回合角色展示手牌，持有者选择花色并结算对应分支。ABI 554。
    ResolvePotjiejie {
        /// Skill owner.
        player: PlayerId,
        /// Current Play-phase actor.
        actor: PlayerId,
        /// 诫节已用 helper definition.
        used_def: SkillDefId,
        /// Four suit-unlimited helper definitions in spade/heart/club/diamond order.
        suit_defs: [SkillDefId; 4],
        /// 清识 definition used by the unique-maximum bonus.
        qingshi_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势辛宪英「清识」：选择角色，按阵营关系共同摸牌或双方弃牌。ABI 554。
    ResolvePotqingshi {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 巧变（shenhua 张郃）：阶段开始时弃一张手牌并跳过该阶段；摸牌阶段可
    /// 获至多两名其他角色各一张手牌；出牌阶段可移动场上的一张牌。ABI 559。
    ResolveQiaobian {
        /// Skill owner.
        player: PlayerId,
        /// Triggered phase: 0 judge, 1 draw, 2 play, 3 discard.
        phase: u8,
        /// Choice barrier stage (0 discard, 1 benefits/skip, 2+ benefit flow).
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 将当前正在结算的【杀】的指定目标从保存目标中移除（此杀对该目标无效）。
    /// ABI 560。
    ExcludeSavedShaTarget {
        /// The Sha card currently resolving.
        card: CardId,
        /// Target to exclude from the Sha's saved target list.
        target: PlayerId,
    },
    /// 享乐（shenhua 刘禅）：使用者弃一张基本牌或此杀对持有者无效。ABI 560。
    ResolveXiangle {
        /// Skill owner (the Sha target).
        player: PlayerId,
        /// The Sha user.
        source: PlayerId,
        /// The Sha card.
        card: CardId,
        /// Choice barrier stage: 0 open discard, 1 commit/exclude.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 放权（shenhua 刘禅）：跳过本回合出牌阶段并标记结束阶段可授额外回合。
    /// ABI 560。
    ResolveFangquanSkip {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 放权（shenhua 刘禅）：结束阶段弃一张手牌并令一名其他角色进行一个额外回合。
    /// ABI 560。
    ResolveFangquanGrant {
        /// Skill owner.
        player: PlayerId,
        /// Choice barrier stage: 0 open discard, 1 open target, 2 commit.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 若愚（shenhua 刘禅，主公技）：准备阶段体力值为全场最少时觉醒：加上限、
    /// 回复 1 并授予激将。ABI 560。
    ResolveRuoyu {
        /// Skill owner.
        player: PlayerId,
        /// 激将 granted skill definition.
        jijiang_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// 回合内禁用指定角色对另一角色使用牌。ABI 571。
    BanUseAgainst {
        /// Banned card user.
        user: PlayerId,
        /// Banned target.
        target: PlayerId,
    },
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
    /// 势魏延【忠傲/壮誓/饮战/狂骨】复合状态机。ABI 558。
    ResolvePotweiyan {
        /// Skill owner.
        player: PlayerId,
        /// Event target, or NONE when the stage has no target.
        target: PlayerId,
        /// Bound physical Sha, or NONE.
        card: CardId,
        /// Public action 0..=10; continuation-only stages are engine-owned.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 势周瑜【炽沄/焰洄/焚涛/雄姿】复合状态机。ABI 561。
    ResolvePotzhouyu {
        /// Skill owner.
        player: PlayerId,
        /// Event target, or NONE.
        target: PlayerId,
        /// Bound physical card, or NONE.
        card: CardId,
        /// Public action 0..=7; continuation-only stages are engine-owned.
        action: u8,
        /// Engine-bound damage provenance flags.
        #[serde(default)]
        card_flags: u8,
        /// Engine-bound ordered card targets.
        #[serde(default)]
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
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
        ] }
    };
}
pub(super) use variants;
