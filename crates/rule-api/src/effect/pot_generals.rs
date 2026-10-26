//! Pot generals effect definitions.

macro_rules! variants {
    (ResolvePotchengpu [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolvePotlibing [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolvePothuanjie [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolvePotnigu [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolvePottuntian [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolvePotjiejie [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolvePotweiyan [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolvePotlvyi [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolvePotheyun [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// §7 技能夺取设施 S2（ABI 677）：选共享技能的角色，失去共同技能，
    /// 目标摸 2（potheyun 合韵）。
    ResolvePotheyun {
        /// 发动者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
}
pub(super) use variants;
