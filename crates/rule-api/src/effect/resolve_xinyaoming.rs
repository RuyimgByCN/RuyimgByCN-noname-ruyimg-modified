//! Ordered EffectRequest variants, starting with `ResolveXinyaoming`.

macro_rules! variants {
    ([$($tail:ident),*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail),*] [
            $($variants)*
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
    /// Let the bound damage target discard `max(1, 4-distance)` hand cards or increase the
    /// current damage by one.
    ///
    /// Appended for serialized ordinal stability.
    ChooseTargetDiscardDistanceHandOrIncreaseDamage {
        /// Damage source and skill owner.
        player: PlayerId,
        /// Bound damage target and discard chooser.
        target: PlayerId,
        /// Engine-owned damage event id.
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose to draw one card, privately inspect up to two sampled target hand cards and gain
    /// one, or move one compatible target equipment card.
    ///
    /// Appended for serialized ordinal stability.
    ChooseDrawViewTwoGainOrMoveEquipment {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other player whose turn ended.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Let `giver` give exactly two differently colored hand/equipment cards to `target`; if
    /// the giver declines or has no valid pair, recover the target by one HP.
    ///
    /// `owner` identifies the originating skill owner and may differ from the choice observer.
    /// Appended for serialized ordinal stability.
    ChooseGiveTwoDifferentColorsOrRecover {
        /// Living owner of the originating skill.
        owner: PlayerId,
        /// Living player who chooses and gives the cards.
        giver: PlayerId,
        /// Living recipient, normally the current dying player.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// Let one player choose and gain one card from another player's typed zone scope.
    ///
    /// Appended for serialized ordinal stability. Unlike the legacy [`Self::TakeOneCard`],
    /// this request supports the judgment zone and uses the shared gain-protection query.
    TakeOneScopedCard {
        /// Living player who owns the offered cards.
        from: PlayerId,
        /// Living player choosing and gaining the card.
        to: PlayerId,
        /// Canonical zones offered to the chooser.
        scope: TargetCardScope,
    },
    /// Publicly reveal one player's complete current hand.
    ///
    /// Appended for serialized ordinal stability.
    RevealHand {
        /// Player whose hand is revealed.
        player: PlayerId,
    },
    /// Draw `count` cards for every currently linked living player in seat order.
    ///
    /// Appended for serialized ordinal stability.
    DrawLinkedPlayers {
        /// Cards drawn by each linked player.
        count: u32,
    },
    /// Publicly reveal the draw-pile top card, then gain it when its printed color matches.
    /// A nonmatching card enters the discard pile.
    ///
    /// Appended for serialized ordinal stability.
    RevealTopGainIfColor {
        /// Living player revealing and possibly gaining the card.
        player: PlayerId,
        /// Printed-color mask: `1` red or `2` black.
        color_mask: u8,
    },
    /// Suppress one player's non-locked skills until the current turn ends.
    ///
    /// Appended for serialized ordinal stability.
    SuppressNonLockedSkillsUntilTurnEnd {
        /// Living player being suppressed.
        player: PlayerId,
    },
    /// Let one player cancel or recover a specified target.
    ///
    /// Appended for serialized ordinal stability.
    ChooseRecoverTarget {
        /// Living player making the choice.
        chooser: PlayerId,
        /// Living wounded recovery target.
        target: PlayerId,
        /// Maximum HP recovered.
        amount: u32,
    },
    /// Give every hand card to another player, who may use one of those exact cards; return the
    /// other gifted cards before that use, or deal one normal damage when the offer is declined.
    ///
    /// Appended for serialized ordinal stability.
    GiveAllHandThenChooseUseOneOrDamage {
        /// Original hand owner and damage source.
        player: PlayerId,
        /// Gift recipient and optional card user.
        target: PlayerId,
        /// Exact gifted cards captured at stage zero.
        #[serde(default)]
        cards: Vec<CardId>,
        /// Selected legal targets for the gifted card at the final stage.
        #[serde(default)]
        targets: Vec<PlayerId>,
        /// Engine-owned stage: zero gives, one chooses a card, two chooses targets, three chooses
        /// a target pair.
        #[serde(default)]
        stage: u8,
        /// Selected gifted card after stage one.
        #[serde(default = "no_card")]
        card: CardId,
        /// Originating active skill instance.
        skill: SkillId,
    },
    /// Choose one logical card category, then let the current Sha target give the source one
    /// matching hand/equipment card or make that Sha unrespondable for the target.
    ///
    /// Appended for serialized ordinal stability.
    ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
        /// Current Sha user and skill owner.
        player: PlayerId,
        /// Current physical Sha.
        card: CardId,
        /// Current Sha target.
        target: PlayerId,
        /// Originating trigger skill instance.
        skill: SkillId,
    },
    /// Choose targets for an ignore-distance virtual Sha, then lose 1 HP and use it. Its base
    /// damage is increased when the user played multiple distinct basic-card variants this turn.
    ///
    /// Appended for serialized ordinal stability.
    ChooseLoseHpUseVirtualShaByBasicVariants {
        /// Skill owner and virtual Sha user.
        player: PlayerId,
        /// Materialized virtual Sha after target selection begins.
        #[serde(default = "no_card")]
        card: CardId,
        /// Selected legal Sha targets at the post-cost stage.
        #[serde(default)]
        targets: Vec<PlayerId>,
        /// Zero chooses targets; one uses the saved virtual Sha after the HP-loss barrier.
        #[serde(default)]
        stage: u8,
        /// Captured virtual Sha damage bonus.
        #[serde(default)]
        damage_bonus: u32,
        /// Originating trigger skill.
        skill: SkillId,
    },
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
    /// 复叛：按伤害点数摸牌，交出一张牌，并按目标的累计状态结算奖励或伤害。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwfupan {
        /// Skill owner.
        player: PlayerId,
        /// Damage amount captured by the trigger.
        damage_amount: u32,
        /// Selected card between the card and target choice stages.
        #[serde(default = "no_card")]
        card: CardId,
        /// Selected recipient between the gift and confirmation stages.
        #[serde(default = "no_player")]
        target: PlayerId,
        /// Engine-owned stage.
        #[serde(default)]
        stage: u8,
        /// Originating trigger skill instance.
        skill: SkillId,
    },
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
    /// 教化（TW）：摸牌后令摸牌角色获得一张本次未获得类别的牌。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwjiaohua {
        /// Skill owner.
        player: PlayerId,
        /// Player who drew the triggering cards.
        target: PlayerId,
        /// Physical cards captured by the draw event.
        event_cards: Vec<CardId>,
        /// Originating trigger skill instance.
        skill: SkillId,
    },
    /// 苦谏（TW）：赠予并追踪实体牌，或结算其使用/打出及其他失去分支。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwkujian {
        /// Skill owner.
        player: PlayerId,
        /// Gift recipient or player who used/lost tracked cards.
        target: PlayerId,
        /// Selected or event physical cards.
        cards: Vec<CardId>,
        /// Zero gifts; one rewards use/response; two penalizes another loss.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Move cards with an engine-owned semantic reason.
    ///
    /// Appended separately so existing `MoveCards` producers keep their wire shape and ordinals.
    MoveCardsWithReason {
        /// Cards in order.
        cards: Vec<CardId>,
        /// Destination zone.
        to: ZoneId,
        /// Optional explicit source (validated by core).
        from: Option<ZoneId>,
        /// Authoritative movement cause.
        reason: engine_api::CardMoveReason,
    },
    /// 睿敛（TW）：选择本轮关注角色、记录其弃牌，并在其回合结束按类别获得弃牌堆牌。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwruilian {
        /// Skill owner.
        player: PlayerId,
        /// Selected role or triggering event player.
        target: PlayerId,
        /// Physical cards captured by a discard event.
        #[serde(default)]
        event_cards: Vec<CardId>,
        /// Zero prompts for a target; one records a discard; two closes the tracked turn; three gains.
        action: u8,
        /// Selected category for action three: 1 basic, 2 trick, 3 equipment.
        #[serde(default)]
        category: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 济溯（collab 叶诗文）：判定阶段开始时跳过本次判定与摸牌阶段，随后视为使用虚拟【杀】。
    ///
    /// Sets the pending judge skip and the SKIP_DRAW turn flag for the current player.
    /// Appended for serialized ordinal stability.
    SkipJudgeAndDrawPhases {
        /// Current phase player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Randomly equip one physical weapon from the shared draw pile when the slot is empty.
    ///
    /// Appended for serialized ordinal stability.
    EquipWeaponFromDrawPile {
        /// Equipment recipient.
        player: PlayerId,
    },
    /// 剑威（TW）：双向发起拼点并按胜负结算区域牌或武器。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwjianwei {
        /// Skill owner.
        player: PlayerId,
        /// Comparison initiator.
        actor: PlayerId,
        /// Comparison opponent.
        #[serde(default = "no_player")]
        target: PlayerId,
        /// Initiator's selected comparison card.
        #[serde(default = "no_card")]
        card: CardId,
        /// Comparison winner for settlement.
        #[serde(default = "no_player")]
        winner: PlayerId,
        /// Zero starts owner's branch; one starts another player's branch; two starts the
        /// comparison after both participants are fixed; three settles the finished comparison.
        #[serde(default)]
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 探锋（TW）：弃置目标区域牌后令其选择以牌当【杀】，否则受伤并跳过阶段。
    ///
    /// Appended for serialized ordinal stability.
    ResolveTwtanfeng {
        /// Skill owner.
        player: PlayerId,
        /// Selected other player.
        #[serde(default = "no_player")]
        target: PlayerId,
        /// Selected physical card used as Sha.
        #[serde(default = "no_card")]
        card: CardId,
        /// Resolver stage.
        #[serde(default)]
        stage: u8,
        /// Selected future phase tag for the skip branch.
        #[serde(default)]
        phase: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 直辩（TW）：发起拼点并结算移动对应区域牌、回复或背水。
    ResolveTwzhibian {
        /// Skill owner.
        player: PlayerId,
        /// Comparison opponent.
        #[serde(default = "no_player")]
        target: PlayerId,
        /// Comparison winner; NONE is a tie.
        #[serde(default = "no_player")]
        winner: PlayerId,
        /// Resolver stage.
        #[serde(default)]
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 御严（TW）：令当前 Sha 使用者交牌，否则取消拥有者这一目标。
    ResolveTwyuyan {
        /// Skill owner and current Sha target.
        player: PlayerId,
        /// Current Sha source.
        source: PlayerId,
        /// Current physical Sha carrier.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 果毅（TW）：目标选择禁用手牌或弃牌，并在满足条件时登记原牌重放。
    ResolveTwguoyi {
        /// Skill owner and current card user.
        player: PlayerId,
        /// Current card target making the branch choice.
        target: PlayerId,
        /// Current physical or materialized card.
        card: CardId,
        /// Temporary helper skill that disables hand-card use and responses.
        hand_lock_skill_def: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Replay a virtual card on its original targets after the source card has fully settled.
    ReplayVirtualCardOnTargets {
        /// Card user.
        player: PlayerId,
        /// Stable logical card-kind tag.
        kind: u8,
        /// Original targets in declaration order.
        targets: Vec<PlayerId>,
        /// Skill provenance used to prevent recursive replay.
        skill: SkillId,
    },
    /// 除害（TW）：完成使命后的判定区废除/座次收牌，或弃置本批从他人获得的牌。
    ResolveTwchuhai {
        /// Skill owner.
        player: PlayerId,
        /// Zero resolves mission success; one discards one triggering gained card.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Engine-owned seat queue index.
        index: u32,
        /// Physical cards captured by the triggering movement.
        event_cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 遗珠（TW）：埋置实体牌、修改他人的单目标用牌，或清除弃牌堆中的追踪牌。
    ResolveTwyizhu {
        /// Skill owner.
        player: PlayerId,
        /// Current card user for target adjustment.
        source: PlayerId,
        /// Current logical card.
        card: CardId,
        /// Zero plants pearls; one handles their use; two clears discarded pearls.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Physical cards captured by the triggering event.
        event_cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 乱筹/共患（TW）：重置共享技能持有者，或将当前伤害完整转移给技能拥有者。
    ResolveTwluanchou {
        /// Skill owner.
        player: PlayerId,
        /// Shared helper skill definition.
        helper_skill_def: u32,
        /// Zero resets the two selected holders; one redirects current damage.
        action: u8,
        /// Active invocation targets.
        targets: Vec<PlayerId>,
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
    /// 天香（old/shenhua 小乔）：弃一张红桃手牌、选择一名其他角色，将当前伤害
    /// 重定向（取消后按同源/同量/同属性/同牌重新结算）并为其附加摸牌技能。
    ///
    /// VM 首发（`target == NONE`）时由 vm_host 从 damage_before 上下文捕获
    /// source/amount/nature/card；选择结算后经 `ResolveTianxiang` 重发应用。
    /// ABI 456（C 段 omp）。
    ResolveTianxiang {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Selected redirect target, or `NONE` before the target choice.
        target: PlayerId,
        /// Heart hand card discarded as the cost, or `NONE` before the card choice.
        discard_card: CardId,
        /// Current damage source (host-captured).
        source: PlayerId,
        /// Current damage amount (host-captured).
        amount: u32,
        /// Current damage nature (host-captured).
        nature: u8,
        /// Current damage card (host-captured).
        card: CardId,
        /// Current logical card kind (host-captured).
        card_kind: u8,
        /// Current card provenance flags (host-captured).
        card_flags: u8,
        /// Skill granted to the redirect target after the damage settles.
        draw_skill_def: SkillDefId,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// 通用弃牌数量代价（`yangzhong` 殃众等）：选择恰好 `count` 张可弃置手牌/装备
    /// 并弃置。
    ///
    /// `count == 0` 是 VM 首发（打开选牌）；选择结算后重发弃置。ABI 460。
    DiscardHandEquipCount {
        /// Player discarding the cards.
        player: PlayerId,
        /// Number of cards to discard.
        count: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// 跳过本回合目标角色的弃牌阶段（SKIP_DISCARD 标记；出牌阶段后由回合机消费）。
    /// ABI 471。
    SkipNextDiscardPhase {
        /// Target whose discard phase is skipped.
        player: PlayerId,
    },
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
    /// Reveal cards, choose one category, distribute all matching cards up to a per-recipient cap,
    /// then make recipients with at least the owner's share lose 1 HP. ABI 491.
    RevealChooseCategoryDistributeSelfThresholdHpLoss {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Number of draw-pile cards to reveal.
        reveal_count: u8,
        /// Maximum matching cards each recipient may receive.
        recipient_cap: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Choose another player with a region card, gain one authoritative-random card, draw one,
    /// and lose 1 HP when the gained and drawn cards have different colors. ABI 492.
    ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Resolve mobile 卫温诸葛直「浮海」's seat-ordered private binary choices. ABI 493.
    ResolveRefuhai {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Discard the selected active-skill hand card, then visibly gain half (rounded up) of the
    /// target's post-cost hand-count advantage. ABI 494.
    ChooseGainTargetHandCeilHalfDifference {
        /// Skill owner and chooser.
        player: PlayerId,
        /// Other target.
        target: PlayerId,
        /// Exact selected heart hand card paid as the cost.
        card: CardId,
        /// Originating skill instance.
        skill: SkillId,
        /// Whether the discard and its nested triggers have completed.
        ready_to_choose: bool,
    },
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
    /// Resolve offline 杨秋「齐锋」 using the owner's current-turn lost-card count. ABI 496.
    ResolveWnQifeng {
        /// Skill owner and virtual Sha user.
        player: PlayerId,
        /// Player whose turn is ending.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// 蛮智（huicui 孟优）：准备/结束阶段二选一：①令一名其他角色交给你至多两张牌，
    /// 其视为使用一张无距离限制的【杀】；②你获得一名其他角色区域内的至多两张牌，
    /// 交给其等量牌后摸一张。每分支每回合一次（技能 mark 记录）。ABI 475。
    ResolveDcmanzhi {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// 宽释（huicui 阚泽）：结束阶段选择一名其他角色标记；其后该角色于本回合内第
    /// 二次受到伤害后回复 1 点体力（每回合一次）。ABI 480。
    ResolveRekuanshi {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 魔狱（huicui 裴元绍）：出牌阶段限一次（每目标每回合一次），获得目标 1（或 2）
    /// 张手牌/装备/判定牌，其视为对你使用一张无距离限制的【杀】；若你因此受伤，
    /// 本回合禁用本技，否则下次多拿一张。ABI 481。
    ResolveDcmoyu {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// 驰应（江山如故高翔）：目标攻击范围内除技能拥有者外的角色依座次各弃一张牌；
    /// 若其中基本牌数不大于目标体力，目标获得仍在弃牌堆的这些基本牌。ABI 501。
    ResolveJsrgchiying {
        /// Skill owner.
        player: PlayerId,
        /// Active-skill target whose range determines the forced discarders.
        target: PlayerId,
        /// Whether the fixed seat-ordered discarder list has already been captured.
        started: bool,
        /// Remaining forced discarders, captured before the first discard.
        remaining: Vec<PlayerId>,
        /// Cards discarded by this resolution.
        accum: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// 虚泄结算：出牌阶段结束时若存在体力上限大于你的角色，恢复 1 点体力上限。
    /// ABI 483（第二阶段）。
    ResolveXuxieAdd {
        /// Skill owner.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// 狭威/穷技（TW）：维护“威”牌堆、妄行代价与空堆伤害增量。
    ResolveTwxiawei {
        /// Skill owner.
        player: PlayerId,
        /// Temporary 妄行 settlement helper definition.
        wangxing_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 挽危/约俭（TW）：处理最低体力伤害保护、回合末牌堆操作与约俭置顶置底收益。
    ResolveTwbianfuren {
        /// Skill owner.
        player: PlayerId,
        /// Current damage target, or NONE outside 挽危伤害结算.
        target: PlayerId,
        /// 挽危临时技能或约俭手牌上限技能定义.
        helper_skill_def: u32,
        /// Zero handles 挽危 damage; one handles its end-phase effect; two handles 约俭.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected physical cards.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 生息/宽济（TW）：获得调剂盐梅/智囊，或转赠非使用进入弃牌堆的牌。
    ResolveTwfeiyi {
        /// Skill owner.
        player: PlayerId,
        /// Zero prepare Shengxi; one end Shengxi; two Kuanji.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected card.
        card: CardId,
        /// Physical cards carried by the trigger event.
        event_cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 睦约/察异（TW）：声明牌名获牌及察异的延迟分支切换。
    ResolveTwchenzhen {
        /// Skill owner or helper owner.
        player: PlayerId,
        /// Current event player when relevant.
        target: PlayerId,
        /// 睦约免弃技能或察异弃牌技能定义.
        helper_skill_def: u32,
        /// 察异延迟记录技能定义.
        record_skill_def: u32,
        /// Zero Muyue; one Chayi setup; two use-card discard; three delayed switch.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Declared physical card kind tag.
        card_kind: u8,
        /// Selected physical card.
        card: CardId,
        /// Branch/state byte.
        branch: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 前驱乐/倾袭（TW）：伤害后补装备，或结算首杀目标的装备选择。
    ResolveTwqianju {
        /// Skill owner.
        player: PlayerId,
        /// Damaged player.
        target: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 倾袭（TW）：首杀目标选择摸牌直击或双方弃装备并加伤。
    ResolveTwqingxi {
        /// Sha user and skill owner.
        player: PlayerId,
        /// Current Sha target.
        target: PlayerId,
        /// Current physical Sha.
        card: CardId,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Number of target equipment cards discarded by the branch.
        count: u8,
        /// Selected source equipment cards.
        cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 躁厉（TW）：弃牌摸牌并按原装备槽补入装备。
    ResolveTwzaoli {
        /// Skill owner.
        player: PlayerId,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected discard cards.
        cards: Vec<CardId>,
        /// Distinct equipment-slot tags to refill.
        slots: Vec<u8>,
        /// Number of cards discarded by the skill.
        count: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 征建/众斥（TW）：选择惩罚条件、收牌切换并在累计后觉醒。
    ResolveTwzhengjian {
        /// Skill owner.
        player: PlayerId,
        /// Current Play-phase owner.
        target: PlayerId,
        /// 众斥伤害加值技能定义.
        helper_skill_def: u32,
        /// Zero setup; one phase-end settlement.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected physical card.
        card: CardId,
        /// Choice bitfield.
        branch: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 震袭/扬师（TW）：弃手牌、移动场上牌，或按攻击范围成长/获得【杀】。
    ResolveTwtianyu {
        /// Skill owner.
        player: PlayerId,
        /// Current Sha target for 震袭; selected move destination in the final stage.
        target: PlayerId,
        /// 扬师 attack-range mark skill definition.
        helper_skill_def: u32,
        /// Zero for 震袭; one for 扬师.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected field card.
        card: CardId,
        /// 震袭 branch bitfield: discard = 1, move = 2.
        branch: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 锋捍/从击（TW）：按牌目标数选择摸牌，或转赠事件中的红色弃牌。
    ResolveTwwujing {
        /// Skill owner.
        player: PlayerId,
        /// Declared target count for 锋捍.
        target_count: u8,
        /// Zero for 锋捍; one for 从击.
        action: u8,
        /// Cards carried by the triggering movement event.
        event_cards: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 存嗣：拥有者翻面，从牌堆/弃牌堆取一张【杀】交给目标。
    ///
    /// Appended for serialized ordinal stability.
    ResolveSpcunsi {
        /// Skill owner.
        player: PlayerId,
        /// Recipient of the Sha.
        target: PlayerId,
    },
    /// 商和（TW）：濒死时其他角色依次交牌；无人交【酒】则回复至一点体力。
    ResolveTwshanghe {
        /// Dying skill owner.
        player: PlayerId,
        /// Zero initializes seat order; one continues an existing queue.
        stage: u8,
        /// Remaining contributors in authoritative seat order.
        remaining: Vec<PlayerId>,
        /// Whether any contributed card was logically Jiu for the dying owner.
        got_jiu: bool,
        /// Originating limited skill instance.
        skill: SkillId,
    },
    /// 开济/慑叛（TW 王昶）的目标选择、置顶与当前目标排除。
    ResolveTwwangchang {
        /// Skill owner.
        player: PlayerId,
        /// Card user for 慑叛; NONE for 开济.
        target: PlayerId,
        /// Triggering physical/virtual card carrier.
        card: CardId,
        /// Selected target-region card for 慑叛置顶.
        selected_card: CardId,
        /// Zero for 开济; one for 慑叛.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected 开济 draw targets.
        targets: Vec<PlayerId>,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 辅纂/宠齐/非服（TW 曹肇）的转换技切换、全场授予与单目标【杀】赠牌。
    ResolveTwcaozhao {
        /// Skill owner.
        player: PlayerId,
        /// Event target or selected player.
        target: PlayerId,
        /// Event source for 非服.
        source: PlayerId,
        /// Triggering or selected physical card.
        card: CardId,
        /// Selected conversion skill definition.
        selected_skill_def: u32,
        /// 辅纂 definition granted by 宠齐.
        twfuzuan_skill_def: u32,
        /// 非服 definition granted by 宠齐.
        twfeifu_skill_def: u32,
        /// Zero 辅纂; one 宠齐; two 非服.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 精策/御章（TW 郭淮）的摸牌蓄策、受伤分支与跳阶段结算。
    ResolveTwguohuai {
        /// Skill owner.
        player: PlayerId,
        /// Trigger event player.
        target: PlayerId,
        /// Damage source.
        source: PlayerId,
        /// Triggering card carrier.
        card: CardId,
        /// 御章 mark definition.
        mark_def: u32,
        /// Temporary no-card helper skill definition.
        helper_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 疠火/醇醪（TW 程普）的改火代价、置醇、增伤与救援结算。
    ResolveTwchengpu {
        /// Skill owner.
        player: PlayerId,
        /// Event or selected target.
        target: PlayerId,
        /// Card user or damage source.
        source: PlayerId,
        /// Triggering Sha carrier.
        card: CardId,
        /// 醇醪 skill definition and pile key.
        chunlao_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Engine-owned resolver stage.
        stage: u8,
        /// Selected field/hand/equipment card.
        selected_card: CardId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 孝廉（TW 曹昂）的单目标杀替换与“马”牌距离结算。
    ResolveTwcaoang {
        /// Skill owner.
        player: PlayerId,
        /// Original Sha target or damage target.
        target: PlayerId,
        /// Sha user or damage source.
        source: PlayerId,
        /// Triggering Sha carrier.
        card: CardId,
        /// Distance helper skill and pile definition.
        helper_skill_def: u32,
        /// Resolver action.
        action: u8,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 挫锐/裂围（TW 牛金）的限定补牌、判定区废除与击杀奖励。
    ResolveTwniujin {
        /// Skill owner.
        player: PlayerId,
        /// 挫锐 limited skill definition.
        limited_skill_def: SkillDefId,
        /// Zero resolves 挫锐; one resolves 裂围.
        action: u8,
        /// Engine-owned stage.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 谋溃（TW 伏完）的摸牌、弃目标牌与背水结算。
    ResolveTwmoukui {
        /// Sha user and skill owner.
        player: PlayerId,
        /// Current Sha target.
        target: PlayerId,
        /// Exact Sha carrier.
        card: CardId,
        /// Engine-owned stage; zero opens the branch choice.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 抚蛮（TW 马忠）的实体赠牌与技能实例追踪。
    ResolveTwfuman {
        /// Skill owner and card giver.
        player: PlayerId,
        /// Selected other recipient.
        target: PlayerId,
        /// Selected physical hand card.
        cards: Vec<CardId>,
        /// Zero gives selected cards; one clears the event card; two clears moved event cards.
        action: u8,
        /// Engine-owned stage; zero performs the gift.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
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
    /// 清口（TW 冯习）的精确虚拟【决斗】结算。
    ResolveTwqingkou {
        /// Skill owner and Duel initiator.
        player: PlayerId,
        /// Duel damage source on completion; ignored while starting the Duel.
        winner: PlayerId,
        /// Exact materialized virtual Duel card; ignored while starting the Duel.
        card: CardId,
        /// Zero starts and tracks the virtual Duel; one applies its post-resolution reward.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 陈勋（幻刘封）的逐轮虚拟【决斗】链。
    ResolveTwchenxun {
        /// Skill owner and Duel initiator.
        player: PlayerId,
        /// Selected Duel target; none while opening a target choice.
        target: PlayerId,
        /// Exact materialized Duel card at post-resolution boundaries.
        card: CardId,
        /// Zero starts a round; one rewards and continues; two penalizes; three uses the
        /// selected target; four opens the next optional target choice.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 焚险（幻黄盖）的目标分支与实体牌转化结算。
    ResolveTwfenxian {
        /// Skill owner, except action four where this is the Duel user.
        player: PlayerId,
        /// Selected skill target, except action four where this is the Duel target.
        target: PlayerId,
        /// Selected field card for the Duel branch.
        card: CardId,
        /// Zero opens branches; one uses Huogong; two opens field cards; three opens Duel
        /// targets; four uses the selected field card as Duel.
        action: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Set or clear one persistent content flag on the originating skill instance.
    SetSkillContentFlag {
        /// Skill owner.
        player: PlayerId,
        /// Flag bit in 0..64.
        bit: u8,
        /// New flag value.
        value: bool,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Give one selected non-equipment hand card, let the recipient optionally use it, and draw
    /// one for the owner after a successful use. ABI 497 (`yuanlve`).
    GiveSelectedThenTargetMayUseDrawOwner {
        /// Skill owner and giver.
        player: PlayerId,
        /// Other living recipient.
        target: PlayerId,
        /// Exact selected card.
        card: CardId,
        /// Selected targets for the recipient's use.
        targets: Vec<PlayerId>,
        /// Resolution stage: 0 gift, 1 offer use, 2 commit use.
        stage: u8,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// Declare one logical card category; the damage target gives one matching hand/equipment
    /// card or the bound damage is increased by one. ABI 498 (`jytongbei`).
    ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
        /// Damage source and skill owner.
        player: PlayerId,
        /// Damage target.
        target: PlayerId,
        /// Bound pre-resolution damage id.
        damage_id: u32,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
}
pub(super) use variants;
