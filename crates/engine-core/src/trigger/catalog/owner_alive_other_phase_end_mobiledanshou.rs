//! Ordered TriggerFilter variants, starting with `OwnerAliveOtherPhaseEndMobiledanshou`.

macro_rules! variants {
    ([$($tail:ident),*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail),*] [
            $($variants)*
    /// Another player's End phase begins and the owner can resolve Mobile Danshou from the
    /// exact current-turn count of cards that player used targeting the owner. ABI 500.
    OwnerAliveOtherPhaseEndMobiledanshou,
    /// 批 220：其他角色结束阶段，且其本回合未对拥有者造成过伤害（伤害对计数）。
    OwnerAliveOtherPhaseEndNotDamagedOwnerThisTurn,
    /// 批 220：其他角色结束阶段，其本回合对拥有者用牌且拥有者本回合未受伤。
    OwnerAliveOtherPhaseEndUsedCardOnOwnerUndamagedThisTurn,
    /// Another player fully resolved a damage card targeting the living owner, and this owner
    /// received damage from that exact use. ABI 503.
    OwnerAliveOtherSourceDamageCardResolvedTargetedDamaged,
    /// Same boundary, but this owner received no damage from the exact use. ABI 503.
    OwnerAliveOtherSourceDamageCardResolvedTargetedNotDamaged,
    /// Another living player is dying and at least one Chen She role has a discardable card.
    /// ABI 503.
    OwnerAliveOtherDyingMbchensheAvailable,
    /// Owner's End phase begins with at least one discardable hand/equipment card; once per round.
    /// ABI 504.
    OwnerIsEventPlayerDiscardableHandEquipOncePerRound,
    /// First discard event outside the Discard phase this turn; 任行 has a legal branch. ABI 504.
    OwnerAliveFirstNonDiscardPhaseDiscardMbrenxingAvailable,
    /// Owner used a hand damage card, this is its first target, and can discard hand cards. ABI 505.
    OwnerSourceFirstTargetHandDamageCardDiscardable,
    /// Owner lost a physical Sha from hand outside use/response, once per turn. ABI 505.
    OwnerLostHandOwnerlessShaNotUseOncePerTurn,
    /// Any phase ended while the owner's originating skill pile is nonempty. ABI 505.
    OwnerAlivePhaseAnyEndOriginatingSkillPileNonempty,
    /// Owner dealt damage with the Sha currently tracked by 断鞅; target has HEJ cards. ABI 505.
    OwnerSourceDamageTrackedMbduanyangTargetRecastable,
    /// 润微's armed loss threshold observes a qualifying owner card-loss batch. ABI 506.
    OwnerMbrunweiLossThreshold,
    /// Owner's Play phase ended with tracked 润微 cards still in hand. ABI 506.
    OwnerPhasePlayEndMbrunweiTrackedHand,
    /// Damage is about to hit a role within final distance one of the owner, once per turn. ABI 506.
    OwnerAliveDamageBeforeTargetDistanceLeOneOncePerTurn,
    /// Owner dealt damage while 凶图's next-damage reward is armed. ABI 507.
    OwnerSourceDamageMbxiongtuArmed,
    /// Owner's Play phase ended while 凶图 has transient state. ABI 507.
    OwnerPhasePlayEndMbxiongtuState,
    /// Owner's turn End phase with a living target remembered by 刚鲠. ABI 508.
    OwnerPhaseEndMbganggengTargetAlive,
    /// Owner entered dying or became handless by losing hand cards; at most twice per turn. ABI 508.
    OwnerFromHandEmptyOrDyingMbsijianTwicePerTurn,
    /// Owner's Prepare phase or post-damage window can move a field card; at most twice per turn. ABI 509.
    OwnerIsTargetCanMoveFieldCardMbyuanmoTwicePerTurn,
    /// Owner used a single-target Sha and this is its first target; at most twice per turn. ABI 510.
    OwnerSourceFirstSingleTargetShaMbfeijingTwicePerTurn,
    /// Damage from the bound Sha is about to hit a 飞径-added target. ABI 510.
    OwnerSourceDamageBeforeMbfeijingExtra,
    /// Owner's originally single-target Sha finished resolving. ABI 510.
    OwnerSourceShaResolvedOriginalSingleMbxiaoge,
    /// A living owner observes another player's Discard phase ending after that player
    /// discarded at least one card into the discard pile during the phase. ABI 513 (固政).
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherDiscardedInDiscardPhase,
    /// A living damage target ties for minimum hand count; once per turn. ABI 514.
    OwnerAliveDamageTargetMinHandMbxuyeOncePerTurn,
    /// A tracked 狂想 role lost its final received exchange card from hand. ABI 514.
    OwnerMbkuangxiangReceivedCardsExhausted,
    /// The owner's next Play phase begins while 狂想 tracking is active. ABI 514.
    OwnerPhasePlayMbkuangxiangActive,
    /// The owner gained cards outside the recursive 督作 movement. ABI 515.
    OwnerGainedCardsPotduzuo,
    /// 批 226：Potduzuo 增益判定的每回合一次变体（防自触发递归）。
    OwnerGainedCardsPotduzuoOncePerTurn,
    /// Sha damage is pending and owner or target has hand count above HP. ABI 515.
    OwnerShaDamageBeforePotbihanAvailable,
    /// Living owner is targeted by a card while at full HP. ABI 516.
    OwnerAliveCardTargetedHealthy,
    /// Current turn player finished a card and has this skill's expansion pile. ABI 517.
    OwnerAliveCardResolvedSourceCurrentPotsifengPile,
    /// A phase-ending player has this skill's expansion pile. ABI 517.
    OwnerAlivePhaseEndPlayerPotsifengPile,
    /// A living owner sees an End phase after at least one living player lost cards this turn;
    /// once per round. ABI 518.
    OwnerAlivePotdingpinAvailableOncePerRound,
    /// Owner finished resolving the exact physical card armed in this skill's phase policy. ABI 519.
    OwnerSourceCardResolvedPhasePolicy,
    /// Owner's Play phase ended with hand/equipment cards and unused 灌沙. ABI 520.
    OwnerPhasePlayEndPotguanshaAvailableOncePerGame,
    /// Owner discarded at least one physical Sha into the shared discard pile. ABI 522.
    ///
    /// Appended for serialized ordinal stability.
    OwnerDiscardedShaToDiscard,
    /// A living owner is the lord of a Wu-faction player's Play phase where both can start a
    /// hand comparison. ABI 523 (制霸).
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherWuPlayPhaseCanCompareOwner,
    /// The living owner is among the targets of a trick that has more than one target and
    /// holds at least one hand/equipment card. ABI 524 (从俭).
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsTargetMultiTargetTrickHasHandEquip,
    /// Another player begins their Play phase, misses at least one 整肃 type, and is
    /// within the owner's attack range. ABI 432 (侯封).
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherPlayPhaseZhengsuMissingInRange,
    /// The owner begins their own Play phase missing at least one 整肃 type. ABI 432 (燕集).
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsTargetMissingZhengsu,
    /// Owner used a club card. ABI 435 (彰名).
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceClub,
    /// Another player begins their Play phase holding a hand card, and the owner
    /// can discard at least as many hand/equip cards as their 凶竖 mark. ABI 436.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherPlayPhaseDiscardableGteMark,
    /// Another player uses a trick whose first target is the owner, and that
    /// player's hand count is tied for the highest among living players. ABI 438.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherTrickFirstTargetSourceMaxHand,
    /// Another player recovered HP while holding the 檄 mark and is still alive
    /// with positive HP. ABI 441 (索命).
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherMarkedXi,
    /// The owner became the single target of another player's card, once per
    /// turn. ABI 442 (通夺).
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsTargetSingleTargetOtherSourceOncePerTurn,
    /// The owner lost exactly one hand/equip/judge card while another player is
    /// the current-phase actor, once per turn. ABI 447 (孤影).
    ///
    /// Appended for serialized ordinal stability.
    OwnerLostOneHandEquipJudgeOtherCurrentPhaseOnce,
    /// Another player's turn ended after a recorded 伏暗 suit entered discard. ABI 525.
    OwnerAliveOtherTurnDiscardedRecordedPotfuanSuit,
    /// The owner's first 伏暗 virtual Sha resolved after dealing damage. ABI 525.
    OwnerSourceShaResolvedPotfuanDealtDamage,
    /// The living owner ends their Play phase after 飞军 first chose a new target this turn.
    /// ABI 526 (兵略).
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsTargetFeijunNewTarget,
    /// The living owner enters their Discard phase owing more cards than their hand limit
    /// while another Qun-faction player is alive. ABI 528 (伪帝).
    ///
    /// Appended for serialized ordinal stability.
    OwnerIsTargetDiscardPhaseOweLimitOtherQun,
    /// The living owner caused Sha damage to a living target and both hold at least one hand
    /// card. ABI 530 (烈刃).
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceShaDamageTargetAliveBothHand,
    /// A living owner observes another player using Nanman whose physical cards are still in
    /// the discard pile. ABI 530 (巨象).
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherNanmanCardsDiscard,
    /// The living owner's bound skill target is the current event's second comparison
    /// participant. ABI 530 (烈刃胜负辅助).
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveSkillBoundTargetIsEventSource,
    /// The owner lost cards out of their own hand/equip/judge zones while it is not the
    /// owner's turn. ABI 533 (屯田).
    ///
    /// Appended for serialized ordinal stability.
    OwnerLostCardsOutsideTurn,
    /// The owner's prepare phase begins with at least three cards in the named skill's
    /// public pile (田 ≥ 3). ABI 534 (凿险).
    ///
    /// Appended for serialized ordinal stability.
    OwnerPhaseBeginSkillPileGteThree(u32),
    /// The owner's prepare phase begins with no hand cards. ABI 538 (志继).
    ///
    /// Appended for serialized ordinal stability.
    OwnerPhaseBeginHandEmpty,
    /// A Nanman card the owner observes resolves with at least one dead target or more
    /// than three damaged targets. ABI 542 (求首).
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveNanmanResolvedTargetDeadOrDamagedGtThree,
    /// The owner's damage-tagged basic/trick names a first target while some target has
    /// hand count at least the owner's and a discardable hand/equipment card. ABI 545
    /// (征荣).
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceFirstDamageCardTargetHandGteOwner,
    /// The owner just responded with a 闪 (shan). ABI 549 (雷击).
    ///
    /// Appended for serialized ordinal stability.
    OwnerRespondedShan,
    /// Another living Qun player's play phase begins while the owner is alive and that
    /// player holds a 闪/闪电. ABI 551 (黄天, once per round).
    ///
    /// Appended for serialized ordinal stability.
    OwnerAliveOtherQunPlayPhaseCanGiveOwner,
    /// Another living player's Play phase begins with a nonempty hand before that actor has
    /// invoked any 诫节 this phase. ABI 554.
    ///
    /// Appended for serialized ordinal stability.
    OwnerAlivePlayPhaseActorHandNonemptyPotjiejieUnused,
    /// The owner is the source of pending Sha damage and at least one 饮战 comparison holds.
    /// ABI 558.
    OwnerSourceShaDamageBeforePotyinzhanCondition,
    /// The owner dealt positive damage to a target at current distance at most one. ABI 558.
    OwnerSourceDamageWithinOne,
    /// Owner gained cards; 势周瑜【炽沄】handles first-gain-per-phase state internally. ABI 561.
    OwnerGainedCardsPotzhouyu,
    /// Owner used a card and this is its first target event; at least one target has hand cards.
    /// ABI 561.
    OwnerSourceFirstTargetHasHandPotyanhui,
    /// A living owner observes fire damage to another currently linked role. ABI 561.
    OwnerAliveFireDamagePotfentao,
    /// 势周瑜【焚涛】has a pending post-damage relink for this event target. ABI 561.
    OwnerAlivePotfentaoPendingDamage,
    /// The owner lost at least one non-damage card from a personal zone, excluding an equipment
    /// card consumed by using that equipment, while 势屯田's charge is below its stored maximum.
    /// ABI 543.
    ///
    /// Appended for serialized ordinal stability.
    OwnerLostNonDamageCardPottuntianChargeNotFull,
    /// A Sha created by this exact 策断 skill instance resolved after dealing damage. ABI 544.
    ///
    /// Appended for serialized ordinal stability.
    OwnerSourceShaResolvedTrackedPotceduanDealtDamage,
    /// Living owner saw one or more Sha enter the discard pile while 战烈 can still gain 烈.
    OwnerAliveShaEnteredDiscardPotzhanlieAvailable,
    /// Owner's tracked 战烈 Sha resolved and requested the post-use draw.
    OwnerSourceShaResolvedTrackedPotzhanlieDraw,
    /// Another living player's Play phase begins and 隳律 still has an unused round color.
    /// ABI 564.
    OwnerAliveOtherPlayPhasePothuilvAvailable,
    /// Owner is the event player and can legally use a source-free Wuzhong. ABI 564.
    OwnerIsTargetCanUseWuzhong,
    /// A non-empty discard batch has a living personal-zone owner. ABI 564.
    OwnerAliveDiscardBatch,
    /// Another player responded to a card used by the living owner. ABI 564.
    OwnerAliveOtherRespondedToOwner,
    /// Owner dealt damage at an unassigned 渐专 timing, or all timings await awakening. ABI 568.
    OwnerSourceDamagePotjianzhuanAvailable,
    /// Owner took damage at an unassigned 渐专 timing, or all timings await awakening. ABI 568.
    OwnerTargetDamagePotjianzhuanAvailable,
    /// Someone entered dying at an unassigned 渐专 timing, or all timings await awakening. ABI 568.
    OwnerAliveDyingPotjianzhuanAvailable,
    /// This 典易/奢权 instance owns the damage-source timing and has not fired it this round.
    OwnerSourceDamagePotcaoshuangAssignedUntriggered,
    /// This 典易/奢权 instance owns the damage-target timing and has not fired it this round.
    OwnerTargetDamagePotcaoshuangAssignedUntriggered,
    /// This 典易/奢权 instance owns the dying timing and has not fired it this round.
    OwnerAliveDyingPotcaoshuangAssignedUntriggered,
    /// Owner just used/responded with the exact card from 溺尊's playable pile. ABI 568.
    OwnerUsedPotnizunPileCard,
    /// Owner is about to take damage after using at least one 溺尊 pile card this round.
    OwnerTargetDamageBeforePotnizunUsed,
    /// A turn ended after owner used a card and tagged cards remain in discard.
    OwnerAliveTurnEndedUsedCardTaggedDiscard,
    /// Living owner has a pending 横江 settlement for this ended turn.
    /// Appended for serialized ordinal stability (ABI 572).
    OwnerAliveTurnEndedRehengjiangPending,
    /// Owner used a logical Sha whose effective suit is heart. ABI 580.
    /// Appended for serialized ordinal stability.
    OwnerSourceHeartSha,
    /// Owner used a red physical card tracked by the active 昭文 instance. ABI 584.
    /// Appended for serialized ordinal stability.
    OwnerSourceActiveDczhaowenTrackedRed,
    /// Owner is a target of Jiu, whose ordinary effect 酒遁 rewrites. ABI 584.
    /// Appended for serialized ordinal stability.
    OwnerIsTargetEventCardJiu,
    /// Another player targeted the owner with a black card and 酒遁 has a legal branch. ABI 584.
    /// Appended for serialized ordinal stability.
    OwnerIsTargetOtherSourceBlackDcjiudunAvailable,
    /// Owner's Play phase ended with a pending 征辟 target. ABI 585.
    /// Appended for serialized ordinal stability.
    OwnerAlivePhasePlayEndZhengbiPending,
    /// Owner took damage from a source with hand cards; repeat once per damage point. ABI 586.
    OwnerIsTargetSourceHandNonemptyPerDamagePoint,
    /// The active 巧谏 helper sees its owner's first still-unconsumed Sha/trick category. ABI 586.
    OwnerSourceQiaojianExtraAvailable,
    /// A player watched by 析察 used their next card. ABI 586.
    OwnerXichaWatchedSourceCardUsed,
    /// A physical card tagged by 巧谏 left the owner's hand. ABI 586.
    OwnerQiaojianTrackedCardLeftHand,
    /// Owner or a character in the owner's attack range took numeric card damage from another
    /// source. ABI 587.
    OwnerAliveDamageTargetSelfOrInRangeOtherSourceNumericCard,
    /// Owner dealt damage, at least one living character is wounded, and this skill has not fired
    /// this turn. ABI 587.
    OwnerSourceWoundedExistsOncePerTurn,
    /// Living owner and event player share a group, and the owner currently has at least one
    /// discardable hand card. ABI 596. Appended for serialized ordinal stability.
    OwnerAliveEventPlayerSameGroupHandNonempty,
    /// Owner dealt damage to the original target of the exact tracked virtual Duel, and that
    /// target's attack range contains a wounded living character other than the owner. ABI 597.
    /// Appended for serialized ordinal stability.
    OwnerTrackedJuedouOriginalTargetDamageRecoverableInRange,
    /// The living owner is the End-phase player, has hand cards, and at least one other living
    /// character also has hand cards. ABI 598. Appended for serialized ordinal stability.
    OwnerIsTargetPhaseEndHandNonemptyOtherHandNonempty,
    /// The living owner is targeted by an ordinary trick and still has a 辞虑 option. ABI 600.
    OwnerIsTargetOrdinaryTrickCilvAvailable,
    /// The damage source card is tracked by this 辞虑 instance's prevention option. ABI 600.
    CilvTrackedCardDamageBefore,
    /// The resolved card has pending 辞虑 cleanup or material gain. ABI 600.
    CilvTrackedCardResolved,
    /// The living owner is dying and this 痛悼 instance has never fired. ABI 600.
    OwnerDyingZhupeilanOncePerGame,
    /// Another tied-most-hand player's Play phase began and 附袭 has a legal branch. ABI 601.
    OwnerAliveOtherPlayPhaseTiedMaxHandDcsbfuxiAvailable,
    /// The owner's End phase began with a damage card that entered discard this turn without
    /// dealing damage. ABI 601.
    OwnerPhaseEndDcsbhaoyiAvailable,
    /// A living owner observes completed fire damage and arms non-stacking 融火. ABI 602.
    OwnerAliveGlobalFireDamageDcsbronghuo,
    /// The living owner is using a fire Sha or Huogong. ABI 602.
    OwnerSourceFireShaOrHuogongDcsbronghuo,
    /// The living owner fully resolved a card with at least one other living declared target;
    /// 英谋 may fire once per turn and must have a legal current conversion branch. ABI 602.
    OwnerSourceCardResolvedHasOtherTargetDcsbyingmouOncePerTurn,
    /// A card currently tracked by an in-flight 英谋 sequence fully resolved. ABI 602.
    DcsbyingmouTrackedCardResolved,
    /// A hand-to-hand transfer involved the owner and the current 盟谋 branch is unused this
    /// turn. ABI 603.
    OwnerHandTransferOtherDcsbmengmouBranchAvailable,
    /// Damage belongs to the exact Sha currently used by 盟谋's Yang branch. ABI 603.
    DcsbmengmouTrackedShaDamage,
    /// The exact Sha currently used by 盟谋's Yang branch fully resolved. ABI 603.
    DcsbmengmouTrackedShaResolved,
    /// The owner is about to take damage associated with a logical or physical card. ABI 604.
    OwnerIsTargetCardDamageBefore,
    /// The owner fully resolved a single-target Sha or trick using only hand materials and may
    /// use an ignore-distance Sha on the surviving target. ABI 606.
    OwnerSourceSingleTargetShaOrTrickAllMaterialsFromHandCanSha,
    /// The exact source-less Sha currently tracked by 寅君 fully resolved. ABI 606.
    DcyinjunTrackedShaResolved,
    /// The owner's own view-as card targeted a player carrying the requested skill pile. ABI 607.
    OwnerSourceOwnViewAsCardTargetedTargetSkillPile(u32),
    /// A living owner observes another player recover outside the owner's turn. ABI 608.
    OwnerAliveOtherRecoveredOutsideOwnerTurn,
    /// The owner ended their Discard phase after discarding at least one logical Sha. ABI 609.
    OwnerIsTargetDiscardedShaInDiscardPhase,
    /// The owner is dealing Sha damage to a currently healthy target. ABI 609.
    OwnerSourceShaDamageBeforeTargetHealthy,
    /// The owner entered End phase with the originating skill's public pile empty. ABI 610.
    OwnerPhaseEndOriginatingSkillPileEmpty,
    /// The owner is a current Sha target and the named public skill pile is nonempty. ABI 610.
    OwnerIsTargetShaSkillPileNonempty(u32),
    /// Another player's Play phase produced a discard while the named public pile is nonempty.
    /// ABI 610.
    OwnerAliveOtherPlayDiscardSkillPileNonempty(u32),
    /// The owner begins Prepare phase and at least one known other group has no member whose
    /// attack range contains the owner. ABI 611.
    OwnerPrepareGroupOutsideAttackRangeExists,
    /// A living owner observes a living Sha target at distance at most one and can discard an
    /// equipment card from hand or equipment. ABI 613.
    OwnerAliveShaTargetDistanceLeOneHasEquipCard,
    /// Owner is the current Sha target, its source is alive, and owner has a hand/equipment card. ABI 614.
    OwnerIsEventPlayerShaSourceAliveHandEquipNonempty,
    /// Living owner has at least one recorded living 负仇 source in this skill instance. ABI 614.
    OwnerAliveWdbFuchouPendingLiving,
    /// Another living player gained exactly one equipment card into hand outside that player's
    /// turn, and that equipment remains immediately usable. ABI 615.
    OwnerAliveOtherGainedSingleEquipToHandOutsideTurnUsable,
    /// 五谷丰登剩余牌刚以专用原因进入弃牌堆且仍可获得。ABI 616.
    OwnerAliveWuguRemainedInDiscard,
    /// Owner is one of at least two targets of a non-multitarget ordinary trick. ABI 616.
    OwnerIsTargetNonMultitargetTrickMultipleTargets,
    /// Owner used a Sha in their Play phase and at least one legal non-target remains when only
    /// ordinary distance is ignored. ABI 617.
    OwnerSourcePlayPhaseShaHasAdditionalTargetIgnoringDistance,
    /// The owner's exact Sha previously extended by 险袭 has fully resolved. ABI 617.
    OwnerSourceCardResolvedTrackedDddXianxi,
    /// Owner dealt card-caused damage to a living target that can receive at least one of the
    /// owner's Bingliang judgment cards. ABI 621.
    OwnerSourceCardDamageTargetCanReceiveOwnerBingliang,
    /// A living owner observes any discard batch that leaves an effective black basic card in
    /// the discard pile. ABI 622.
    OwnerAliveDiscardedBlackBasicToDiscard,
    /// Owner's previous 虚誉 activation is waiting for this owner's next resolved card. ABI 623.
    OwnerSourceCardResolvedDddXuyuPending,
    /// A 虚誉 debt is waiting to expire at the current turn boundary. ABI 623.
    DddXuyuPending,
    /// Another player completed their second Play-phase card while owner can give a card. ABI 623.
    OwnerAliveOtherSourceSecondPlayCardResolvedHandEquipNonempty,
    /// The player watched by 实荐 used their next card with at least one target. ABI 623.
    DddShijianWatchedSourceCardUsedTargeted,
    /// A 实荐 watch is waiting to expire at the current turn boundary. ABI 623.
    DddShijianPending,
    /// A living owner changed HP this turn and a turn-entered discard card remains. ABI 624.
    OwnerAliveHpChangedThisTurnEnteredDiscardAvailable,
    /// Another living player's turn begins while both them and owner have hand cards. ABI 625.
    OwnerAliveOtherPhaseBeginBothHandNonempty,
    /// Owner has at least one card in the 洞察 skill pile at PhaseEnd. ABI 625.
    OwnerAliveDddDongchaPileNonempty,
    /// Another living player's turn begins and owner has at least that player's HP worth of
    /// face-down hand cards. ABI 626.
    OwnerAliveOtherPhaseBeginUnshownHandGteHp,
    /// Owner is in their Play phase, HP category is unused, and the changed player's HP now
    /// equals owner's HP. ABI 627.
    OwnerPlayDddZhengjunHpAvailable,
    /// Owner is in their Play phase and a changed hand/equipment count now equals owner's
    /// corresponding count in an unused category. ABI 627.
    OwnerPlayDddZhengjunCardsMovedAvailable,
    /// Another living player's Play phase begins while owner can discard a hand/equipment card.
    /// ABI 628.
    OwnerAliveOtherPhasePlayDddChashiAvailable,
    /// The watched Play-phase player used a card matching 察势's discarded suit or category.
    /// ABI 628.
    DddChashiMatchingCard,
    /// The living owner's 齐策 persistent lock is currently clear. ABI 628.
    OwnerAliveDddQiceReady,
    /// The Play phase watched by 察势 is ending, so its pending comparison expires. ABI 628.
    DddChashiPendingPhasePlayEnd,
    /// At global phase end, exactly two living players used/responded this turn: owner and one
    /// other male or double-sex player. ABI 629.
    OwnerAliveDddZhilianAvailable,
    /// Owner just finished using or responding with a basic card, still has hand/water cards,
    /// and 浮世 has not fired this turn. ABI 630.
    OwnerAliveSourceBasicDddFushiAvailable,
    /// Owner is entering Draw/Discard and has an unused legal proactive ordinary trick. ABI 631.
    OwnerAliveDddFenjiAvailable,
    /// First observed phase of a round with at least one prior-round 丰政 user. ABI 632.
    OwnerAliveDddFengzhengRoundPending,
    /// A discard-entry batch contains a physical rank matching the owner's 玉律. ABI 632.
    OwnerAliveDddYulvDiscardMatches,
    /// First observed phase of each round for 管宁「游学」. ABI 634.
    OwnerAliveDddYouxueRoundPending,
    /// A marked player's turn ended with an eligible physical use still discarded. ABI 634.
    OwnerAliveDddChengjingAvailable,
    /// Another player's Play phase begins while 曹爽 can give a hand card. ABI 635.
    OwnerAliveDddZhuansheGift,
    /// Watched turn actor used a recorded logical card name. ABI 635.
    OwnerAliveDddZhuansheCardUsed,
    /// Watched actor's turn ends. ABI 635.
    OwnerAliveDddZhuanshePhaseEnd,
    /// 曹爽 has no hand before any recovery. ABI 635.
    OwnerAliveDddWeiqiu,
    /// Owner resolved a red card with recoverable physical materials. ABI 636.
    OwnerAliveDddLianerAvailable,
    /// A turn ended while 涟洏's rank ceiling is active. ABI 636.
    OwnerAliveDddLianerCeiling,
    /// Last globally used/responded card was black. ABI 636.
    OwnerAliveDddAnzhiAvailable,
    /// Record the latest global used/responded card color. ABI 636.
    OwnerAliveDddAnzhiTracking,
    /// Armed target's first black targeting this turn. ABI 636.
    OwnerAliveDddAnzhiTargetBlack,
    /// Armed 暗织 turn ended. ABI 636.
    OwnerAliveDddAnzhiTurnEnd,
    /// A new turn restores 失兴's “no trick used” branch and its derived skills changed. ABI 637.
    OwnerAliveDddShixingTurnRefresh,
    /// An HP transition changed whether an undamaged female exists. ABI 637.
    OwnerAliveDddShixingFemaleChanged,
    /// The first trick of the current turn removes 失兴's 酒诗 branch. ABI 637.
    OwnerAliveDddShixingFirstTrick,
    /// 刘宏 begins End with fewer linked players than living 群 characters. ABI 637.
    OwnerAliveDddDangguAvailable,
    /// A 群 character died while the living owner has active 覆宗. ABI 637.
    OwnerAliveDddFuzongDeath,
    /// The damaged skill owner is alive and currently face-down. ABI 637.
    OwnerIsTargetFaceDown,
    /// A turn ended while 夏侯玄 has a pending 玄论 settlement. ABI 638.
    OwnerAliveDddXuanlunPending,
    /// Another character is about to recover while unused 劫行 can replace it. ABI 639.
    OwnerAliveDddJiexingRecoverOncePerGame,
    /// Another character used equipment whose physical card can enter the owner's slot. ABI 639.
    OwnerAliveDddJiexingEquipOncePerGame,
    /// The owner recovered after 劫行 was consumed. ABI 639.
    OwnerAliveDddJiexingRecoveryReset,
    /// The owner finished using equipment after 劫行 was consumed. ABI 639.
    OwnerAliveDddJiexingEquipReset,
    /// The owner's Prepare phase has at least one legal 拜泪 target. ABI 639.
    OwnerAliveDddBaileiAvailable,
    /// The owner used 五谷丰登 through 仝御. ABI 640.
    OwnerSourceOwnViewAsDddTongyuUsed,
    /// One target was named by the owner's 仝御 五谷丰登. ABI 640.
    OwnerSourceOwnViewAsDddTongyuTargeted,
    /// 仝御 left revealed cards in discard and at least one rejecting target survives. ABI 640.
    OwnerAliveDddTongyuRemained,
    /// Card-name-scoped family (Phase 3): the owner used a card whose logical kind/suit matches
    /// the selector. Fires on `card_used` / `card_responded`. Unlocks card_name_scoped placeholders.
    OwnerSourceCardName(crate::query::CardSelector),
    /// Card-name-scoped family (Phase 3): the owner is the target of a card whose logical kind/suit
    /// matches the selector. Fires on `card_targeted`.
    OwnerIsTargetCardName(crate::query::CardSelector),
    /// Card-name-scoped family (Phase 3): the owner took damage whose originating card matches the
    /// selector. Fires on `damage`.
    OwnerIsTargetCardDamage(crate::query::CardSelector),
    /// Skill-provenance family (Phase 3): the event's card originated from a view-as skill with the
    /// given definition id (analogue of upstream `event.skill == "X"`). Parameter is the skill def id.
    EventViewAsSkill(u32),
    /// Skill-provenance name family (Phase 3): the event's card originated from a view-as skill whose
    /// content-metadata name matches (analogue of upstream `event.skill == "X_backup"` for dynamic
    /// helpers without a static def id). Parameter is the skill name.
    EventViewAsSkillName(String),
    /// The owner is the damage target and the damage is iron-chain-propagated
    /// (`damage_card_flag::CHAIN_PROPAGATED`). Implements 红棉百花袍 (hongmianbaihuapao_skill):
    /// `owner_is_target_chain_damage_linked`.
    OwnerIsTargetChainDamageLinked,
    /// The owner is the damage target, is wounded, and the damage comes from a Sha card.
    /// Implements 得势 (sp2/dcdeshi): `owner_is_target_wounded_sha`.
    OwnerIsTargetWoundedSha,
    /// The owner is at full health and the draw phase is not fixed (`!draw_phase_is_fixed()`).
    /// Implements 螭纹玉佩 (sp/chiwenyupei) draw branch: `owner_is_target_healthy_draw_not_fixed`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsTargetHealthyDrawNotFixed,
    /// The owner used a Sha or a black ordinary trick and has the configured stable mark.
    /// Implements 乱战·增 (sp/reluanzhan_add): `owner_source_sha_or_black_trick_mark_positive_<mark>`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceShaOrBlackTrickMarkPositive(u32),
    /// The living owner is targeted by another player's black card.
    /// Implements 协穆·指定 (sp/xiemu2): `owner_alive_targeted_by_other_black_card`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveTargetedByOtherBlackCard,
    /// The living owner observes another player being killed and is not the killer.
    /// Implements 起乱 draw-on-death (sp/qiluan): `owner_alive_other_source_death`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherSourceDeath,
    /// The owner is the event player during their own play phase and has no Sha in hand.
    /// Implements 摧怀/咄咄 (sp/cihuai, sp/oldcihuai): `owner_is_target_play_phase_hand_no_sha`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsTargetPlayPhaseHandNoSha,
    /// The living owner observes another player's end phase and that event player is not
    /// in the owner's attack range.
    /// Implements 礼下·OL (sp/ollixia): `owner_alive_other_phase_end_player_outside_owner_range`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherPhaseEndPlayerOutsideOwnerRange,
    /// The living owner used a Sha and the event player (a living target) has a
    /// discardable equipment card.
    /// Implements 风石 (sp/zfengshi): `owner_source_sha_target_alive_has_discardable_equip`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceShaTargetAliveHasDiscardableEquip,
    /// The living owner used a card and is among its targets.
    /// Implements 成都 (sp/pxchengdu): `owner_source_card_used_targets_self`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceCardUsedTargetsSelf,
    /// The living owner (current phase) targets a healthy other player with a card.
    /// Implements 廷机·OL (sp/oltingji): `owner_source_current_phase_card_targeted_other_healthy`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceCurrentPhaseCardTargetedOtherHealthy,
    /// The living owner just gained the skill with the given def id.
    /// Implements 州郡 skills (sp/px*): `owner_skill_added_self_<def>`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSkillAddedSelf(u32),
    /// The living owner used a card whose effective suit is the first of that suit this turn.
    /// Implements 文喜 (sp/pxwenxi): `owner_source_first_card_suit_this_turn`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceFirstCardSuitThisTurn,
    /// The living owner lost hand cards and their hand is now empty.
    /// Implements 平原 (sp/pxpingyuan): `owner_lost_hand_now_empty`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerLostHandNowEmpty,
    /// The living owner lost a physical spade 2-9 card from their hand.
    /// Implements 巨鹿 (sp/pxjulu): `owner_lost_spade_two_to_nine`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerLostSpadeTwoToNine,
    /// The living owner is either participant in a completed hand comparison.
    /// Implements 庐江 (sp/pxlujiang win/lose sub-skills):
    /// `owner_hand_compare_finished_participant`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerHandCompareFinishedParticipant,
    /// Owner is alive, the event player is alive, and the event player is either the owner
    /// or a member of `clan`. Implements 保族 (clan/clanbaozu):
    /// `owner_alive_event_player_self_or_clan_<CLAN>`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventPlayerSelfOrClan(String),
    /// The skill owner is the player whose card a `CardResponded` event answered
    /// (`ctx.respond_to == owner`). Implements 矢仄 (clan/clanshize):
    /// `owner_is_respond_to`. Note `ctx.source` for respond events is the RESPONDER.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsRespondTo,
    /// Owner took Sha-card damage (`owner_is_player && card_is_sha` on Damage).
    /// Mirror of `OwnerSourceSha` for the taken side. Implements 极寒冻动·贰
    /// (shahai card/swd); unlocks the「受击杀伤害」family
    /// (xingtigaizao_equip2 / gudonggeng / jinjilengdong_equip2).
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerTargetSha,
    /// The living owner is targeted by another player's red card. Mirror of
    /// `OwnerAliveTargetedByOtherBlackCard` (协穆·指定). Implements the
    /// per-color targeted family (郊夏 / 芳华, shahai):
    /// `owner_alive_targeted_by_other_red_card`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveTargetedByOtherRedCard,
    /// The owner is among the participants of a debate that just opened
    /// (DebateStarted window, ADR 0644 P3). Implements 凶暴 (jsrgxiongbao):
    /// `owner_debate_participant`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerDebateParticipant,
    /// Cards moved into the discard pile and at least one is a shadow card
    /// (jsrg funi③). `owner_cards_moved_contain_ying`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerCardsMovedContainYing,
    /// The skill owner has the Lord identity (identity mode).
    /// Implements 出师 (jsrgchushi) gate: `owner_is_lord`.
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerIsLord,
    /// A living owner observes a logical Sha targeting a player other than
    /// themselves (daimou 殆谋「他人」分支). `owner_alive_other_targeted_by_sha`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherTargetedBySha,
    /// A living owner is targeted by another player's Sha and holds at least
    /// one tagged xumou (scheme) card in their judge zone (daimou「我」分支).
    /// `owner_alive_targeted_by_other_sha_judge_xumou`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveTargetedByOtherShaJudgeXumou,
    /// The event player (turn owner at phase end) holds the Lord identity
    /// (jsrgzhenglve①「一号位的回合结束」identity-mode rendering).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventPlayerLord,
    /// Damage whose source is the living owner and whose target carries a
    /// positive count of the given mark, once per turn (jsrgzhenglve③).
    /// Filter string:
    /// `owner_source_damage_target_mark_positive_<def>_once`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceDamageTargetMarkPositiveOnce(u32),
    /// Own prepare phase while more than two living players carry the given
    /// mark (jsrghuilie awakening gate). Filter string:
    /// `owner_alive_marked_alive_above_two_<def>`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveMarkedAliveAboveTwo(u32),
    /// Any living player carries a positive count of the given mark
    /// (jsrgpingrong gate). Filter string:
    /// `owner_alive_someone_mark_positive_<def>`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveSomeoneMarkPositive(u32),
    /// A living owner holds a non-empty City pile for the given skill def and
    /// some other player lost cards this turn (jsrgshacheng②).
    /// Filter string: `owner_alive_sha_resolved_city_<def>`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveShaResolvedCity(u32),
    /// Phase end of the Lord, who dealt no damage this turn (jsrgzhenglve①
    /// 「至多两名」分支). `owner_alive_event_player_lord_no_damage`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventPlayerLordNoDamage,
    /// Phase end of the Lord, who dealt damage this turn (jsrgzhenglve① single
    /// grant branch). `owner_alive_event_player_lord_damaged`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventPlayerLordDamaged,
    /// Own turn ending with all four suits used this turn (refresh botu 博图).
    /// `owner_alive_turn_used_all_suits`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveTurnUsedAllSuits,
    /// Own turn ending while the turn player carries the given mark and dealt
    /// no damage this turn (xianding guili 归离绑定回收).
    /// `owner_alive_event_target_mark_positive_<mark>_no_damage`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventTargetMarkPositiveNoDamage(u32),
    /// A card-move of at least three cards into the discard pile
    /// (diy nsjihui 集计). `cards_moved_discard_at_least_three`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    CardsMovedDiscardAtLeastThree,
    /// Own turn ending while the turn player carries the given mark
    /// (tw twjieqiu end 子技判定). `owner_alive_event_target_mark_positive_<mark>`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveEventTargetMarkPositive(u32),
    /// A living owner observes a DamageBefore event whose victim holds a
    /// positive count of the given mark (kuanshi 宽释 protection arm, batch 244).
    /// Filter string: `owner_alive_damage_before_event_target_mark_positive_<mark>`.
    OwnerAliveDamageBeforeEventTargetMarkPositive(u32),
    /// A living owner observes a PhaseBegin event whose turn player holds a
    /// positive count of the given mark (kuanshi 宽释 expiry arm, batch 244).
    /// Filter string: `owner_alive_phase_begin_event_target_mark_positive_<mark>`.
    OwnerAlivePhaseBeginEventTargetMarkPositive(u32),
    /// A living owner observes a cards-moved-to-discard event where at least
    /// one discarded card matches the suit and rank of the first card in the
    /// owner's own skill pile for the given pile def (biaozhao2 表召·应,
    /// batch 259). Filter string:
    /// `owner_alive_discard_matches_own_pile_card_<pile_def>`.
    OwnerAliveDiscardMatchesOwnPileCard(u32),
    /// A cards-moved event moved cards INTO an expansion (skill-pile) zone
    /// (reliangyin 良姻① draw arm, batch 268). Filter string:
    /// `cards_moved_into_expansion`.
    CardsMovedIntoExpansion,
    /// A cards-moved event moved cards OUT OF an expansion (skill-pile) zone
    /// (reliangyin 良姻② discard arm, batch 268). Filter string:
    /// `cards_moved_out_of_expansion`.
    CardsMovedOutOfExpansion,
    /// Owner uses a card while holding no card with the given gaintag
    /// (natuxing 兴「手中无锁定兴」门). `owner_source_hand_gaintag_absent_<tag>`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceHandGaintagAbsent(u32),
    /// Own play phase ends with at least one card used this phase
    /// (offline zylianji 连计). `owner_alive_play_end_used_categories_positive`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAlivePlayEndUsedCategoriesPositive,
    /// The living owner gains cards during their own play phase while wounded
    /// (collab haoxue 好学回复臂). `owner_gained_cards_during_play_phase_wounded`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerGainedCardsDuringPlayPhaseWounded,
    /// The living owner discards hand/equip cards to the discard pile outside their own
    /// play phase (collab haoxue 好学流失臂). `owner_discarded_outside_play_phase`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerDiscardedOutsidePlayPhase,
    /// A multi-target (target count > 1) sha or ordinary trick is used with the living
    /// owner among its declared targets (sp danlao 啖酪).
    /// `owner_in_multitarget_sha_or_trick_targets`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerInMultitargetShaOrTrickTargets,
    /// The living owner becomes the target of a Nanman invasion (sp xinfu_zhennan 镇南).
    /// `owner_targeted_by_nanman`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerTargetedByNanman,
    /// Damage-before: the living owner is the damage target, the source is alive, and
    /// the source's HP and hand count both differ from the owner's (sp2 yinyi 隐翼;
    /// upstream linked-nature exclusion not modelled).
    /// `owner_targeted_source_hp_hand_differ_once_per_turn`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerTargetedSourceHpHandDifferOncePerTurn,
    /// Card-resolved: the living owner used an ordinary trick which has finished
    /// resolving during their own play phase, once per turn (sp2 xinfu_yisuan 翼算;
    /// gaining the discarded cards is the effect's business, not the filter's).
    /// `owner_source_trick_resolved_own_play_phase_once_per_turn`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSourceTrickResolvedOwnPlayPhaseOncePerTurn,
    /// A living owner observes another living player enter dying while the owner holds an
    /// equip-kind card in hand or equipment zone (onlyOL olrenxin 仁心; the upstream
    /// equip-card discard cost is approximated by an optional confirm).
    /// `owner_alive_other_dying_has_equip_card`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherDyingHasEquipCard,
    /// A living owner observes damage dealt by a Nanman invasion card, whoever the
    /// target (xianding mansi 蛮肆 main arm).
    /// `owner_alive_damage_card_nanman`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveDamageCardNanman,
    /// A black card enters or leaves the living owner's equipment or judgment zone
    /// (sp2 liangjue 狼灭). Both directions are scoped to the owner's own zones.
    /// `owner_black_card_moved_equip_judge`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerBlackCardMovedEquipJudge,
    /// A card of the declared suit moves into the discard pile from the living owner's
    /// zones while the owner lacks the declared falu mark (sp xinfu_falu 法箓 suit
    /// collection; suit encoded in the filter token, mark def carried by the skill's
    /// DSL add_mark).
    /// `owner_discard_suit_X_no_own_mark_<mark>`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerDiscardSuitNoOwnFaluMark(u8, u32),
    /// The owner (event player) just completed a turn-over in either direction
    /// (refresh shebian 涉编 upstream `player: "turnOverEnd"`).
    /// `owner_turned_over`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerTurnedOver,
    /// Another living wei-group player recovers HP during their own turn's phases
    /// (xianding zhushi 助势 主公技, upstream `global: "recoverEnd"` with
    /// `event.player.group == "wei" && event.player == currentPhase`).
    /// `owner_alive_other_wei_recovery_own_phase`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherWeiRecoveryOwnPhaseOncePerTurn,
    /// Any living player uses 【决斗】 or a red 【杀】 (offline ylyg_jiang 激:
    /// global useCardToPlayered with first target; owner involvement not required).
    /// `owner_alive_global_juedou_or_red_sha_used`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveGlobalJuedouOrRedShaUsed,
    /// Another living player (user != owner) uses 【决斗】 or a red 【杀】.
    /// `owner_alive_other_global_juedou_or_red_sha_used`.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerAliveOtherGlobalJuedouOrRedShaUsed,
    /// The owner's named skill pile (武将牌上, keyed by a skill def literal —
    /// possibly a sibling skill) holds at least one card.
    /// `owner_skill_pile_nonempty_<def>` (expansion S1-B family gate).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    OwnerSkillPileNonempty(u32),
        ] }
    };
}
pub(super) use variants;
