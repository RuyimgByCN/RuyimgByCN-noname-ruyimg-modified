//! Extension generals effect definitions.

macro_rules! variants {
    (ResolveWdbHanlong [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveWdbXizhenxihong [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveWdbZaozhiRenjun [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveDddXuelingyun [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
    (ResolveGwyewuChain [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
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
        ] }
    };
}
pub(super) use variants;
