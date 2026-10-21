//! Ordered StmtIr variants, starting with `EffectResolveMbzhuji`.

macro_rules! variants {
    ([$($tail:ident),*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail),*] [
            $($variants)*
    /// 兵势孙韶「筑墼」复合。ABI 521.
    EffectResolveMbzhuji {
        /// Skill owner expression.
        player: ExprIr,
        /// Resolver stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_potlibing(player, card, stage). ABI 522.
    EffectResolvePotlibing {
        /// Skill owner.
        player: ExprIr,
        /// Used or resolved card.
        card: ExprIr,
        /// Zero handles resolution; one handles use.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_potpoxi(player). ABI 522.
    EffectResolvePotpoxi {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_potfuan(player, card, stage, range_helper_def, yinxian_def). ABI 525.
    EffectResolvePotfuan {
        /// Skill owner.
        player: ExprIr,
        /// Movement event card.
        card: ExprIr,
        /// Resolution stage.
        stage: u8,
        /// Attack-range helper skill definition.
        range_helper_def: u32,
        /// 引弦 helper skill definition.
        yinxian_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_potyinxian(player, card, stage). ABI 525.
    EffectResolvePotyinxian {
        /// Skill owner.
        player: ExprIr,
        /// Movement event card.
        card: ExprIr,
        /// Resolution stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 势桓阶「共谋」「正朔」复合。ABI 529.
    EffectResolvePothuanjie {
        /// Skill owner expression.
        player: ExprIr,
        /// Exchange target expression; ignored while opening the choice.
        target: ExprIr,
        /// Resolver stage.
        stage: u8,
        /// 奇策 skill definition.
        qice_def: u32,
        /// 看破 skill definition.
        kanpo_def: u32,
        /// Source span.
        span: Span,
    },
    /// 势孙綝「逆固」复合。ABI 532.
    EffectResolvePotnigu {
        /// Skill owner expression.
        player: ExprIr,
        /// Resolver stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 势孙綝「戮连」复合。ABI 532.
    EffectResolvePotlulian {
        /// Skill owner expression.
        player: ExprIr,
        /// Triggering physical card.
        card: ExprIr,
        /// Resolver stage.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// 势鲁肃「缔盟」复合。ABI 535.
    EffectResolvePotdimeng {
        /// Skill owner expression.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 势鲁肃「好施」复合。ABI 536.
    EffectResolvePothaoshi {
        /// Skill owner expression.
        player: ExprIr,
        /// Zero grants at End phase; one clears at the next turn start.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 势董昭「妙略」复合。ABI 539.
    EffectResolveSpmiaolve {
        /// Skill owner expression.
        player: ExprIr,
        /// Zero initializes; one handles damage.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 势董昭「迎驾」复合。ABI 539.
    EffectResolveSpyingjia {
        /// Skill owner expression.
        player: ExprIr,
        /// Zero resets, one records, two settles.
        action: u8,
        /// Extra-turn draw helper definition.
        helper_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spyajun(player, less_def, stage)
    ///
    /// ABI 428: 压军 — at Play-phase begin with a card gained this turn, compare
    /// a gained card against another player; on win optionally place the
    /// comparison card on top of the draw pile, on loss add a hand-limit mark.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpyajun {
        /// Skill owner.
        player: ExprIr,
        /// 戒 hand-limit mark definition.
        less_def: u32,
        /// Continuation stage.
        stage: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spzundi(player, target)
    ///
    /// ABI 429: 尊帝 — judge after discarding a hand card to another player;
    /// black: that player draws 3; red: that player moves a field card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpzundi {
        /// Skill owner.
        player: ExprIr,
        /// Judge beneficiary.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xinyangjie(player, target)
    ///
    /// ABI 430: 佯解 — compare against another player with the SMALLER rank
    /// winning; on failure choose another player to use a virtual fire Sha on
    /// the comparison target.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveXinyangjie {
        /// Skill owner.
        player: ExprIr,
        /// Comparison target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_qingyu(player, action, xuancun_def, mbyongjue_def)
    ///
    /// ABI 431: 清雨 — mission skill: defend by discarding 2 he cards before
    /// damage (action 0); on Prepare phase healthy with hand below the round
    /// number the mission succeeds and grants 选存 (action 1); on dying the
    /// mission fails, lowering max HP and granting 勇决 to a chosen player
    /// (actions 2/3).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveQingyu {
        /// Skill owner.
        player: ExprIr,
        /// Public action 0..=3.
        action: u8,
        /// 选存 definition.
        xuancun_def: u32,
        /// 勇决 definition.
        mbyongjue_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_zhengsu(player, mode, stage)
    ///
    /// ABI 432: 整肃 system — mode 0 垒金 rank-ascending tracking, mode 1
    /// 变阵 same-suit, mode 2 明志 distinct-suit discards; judged at
    /// discard-phase end with a draw-or-recover reward for grantees + self.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveZhengsu {
        /// Skill owner.
        player: ExprIr,
        /// Mode 0 垒金 / 1 变阵 / 2 明志.
        mode: u8,
        /// Stage 0 track, 1 judge.
        stage: u8,
        /// Triggering card (mode 0 stage 0).
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_houfeng(player, target, mode, stage)
    ///
    /// ABI 433: 侯封 / 燕集 — choose a 整肃 type and grant it to the target
    /// (mode 0 侯封 with grantor record; mode 1 燕集 for self).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveHoufeng {
        /// Skill owner.
        player: ExprIr,
        /// Grant target.
        target: ExprIr,
        /// Mode 0 侯封 / 1 燕集.
        mode: u8,
        /// Stage 0 choose type, 1 commit grant.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_mbdiancai(player, actor)
    ///
    /// ABI 434: 点才 — at another player's Play-phase end, if that actor lost at
    /// least as many cards this turn as their HP and holds fewer cards than their
    /// max HP, they draw up to their max HP.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveMbdiancai {
        /// Skill owner.
        player: ExprIr,
        /// The play-phase actor whose End just fired.
        actor: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_rechuhai(player, target, card, target_card, stage)
    ///
    /// ABI 435: 战海 — duty skill: draw one and compare with rank bonus from
    /// missing equipment; on win reveal the target's hand and gain one pile card
    /// per category; on loss with rank below 7 the mission fails; with more than
    /// two equipment the mission succeeds and swaps 乡害 for 彰名.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveRechuhai {
        /// Skill owner.
        player: ExprIr,
        /// Comparison target.
        target: ExprIr,
        /// Owner's chosen comparison card.
        card: ExprIr,
        /// Target's chosen comparison card.
        target_card: ExprIr,
        /// Continuation stage 0..=3.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_xiongshu(player, target, card, action, stage)
    ///
    /// ABI 436: 凶竖 — discard X cards (X = skill mark) at another player's
    /// Play-phase start, reveal one of their hand cards, predict whether they
    /// will use it; at that phase's end a correct prediction deals 1 damage,
    /// a wrong one lets the owner gain the card.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveXiongshu {
        /// Skill owner.
        player: ExprIr,
        /// Play-phase actor / prediction target.
        target: ExprIr,
        /// Chosen shown card.
        card: ExprIr,
        /// Prediction 0 会使用 / 1 不会使用.
        action: u8,
        /// Continuation stage 0..=3.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_jianhui(player, source, target)
    ///
    /// ABI 437: 奸回 — after damage, as source drawing one when the damaged
    /// player matches the last one; as target forcing the source to discard a
    /// hand/equip card when the source matches the last one.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveJianhui {
        /// Skill owner.
        player: ExprIr,
        /// Damage source.
        source: ExprIr,
        /// Damage target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spqiai(player, target)
    ///
    /// ABI 439: 弃哀 — give a selected non-basic card to another player; at
    /// full HP draw two, else the recipient chooses between the owner
    /// recovering 1 or drawing two.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpqiai {
        /// Skill owner.
        player: ExprIr,
        /// Gift recipient.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spshanxi(player, target, stage)
    ///
    /// ABI 440: 擅袭 — choose a player without the 檄 mark and give it to them
    /// (globally unique; other players' marks are cleared).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpshanxi {
        /// Skill owner.
        player: ExprIr,
        /// Chosen mark recipient.
        target: ExprIr,
        /// Stage 0 choose target, 1 commit mark.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_spshanxi_suoming(player, actor, stage)
    ///
    /// ABI 441: 索命 — a recovering actor with the 檄 mark gives the owner two
    /// cards or loses 1 HP (loses HP directly when they cannot give).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveSpshanxiSuoming {
        /// Skill owner.
        player: ExprIr,
        /// The recovering actor with the 檄 mark.
        actor: ExprIr,
        /// Stage 0 open give choice, 1 commit.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_tongduo(player, target, card, stage)
    ///
    /// ABI 442: 通夺 — when the owner becomes the single target of another
    /// player's card (once per turn), choose a player with cards to recast one.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveTongduo {
        /// Skill owner.
        player: ExprIr,
        /// Chosen recast actor.
        target: ExprIr,
        /// Chosen recast card.
        card: ExprIr,
        /// Stage 0 choose actor, 1 choose card, 2 commit recast.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_yuanqing(player)
    ///
    /// ABI 443: 怨清 — at Play-phase end, for each category used this phase,
    /// randomly take one matching card from the discard pile into the 仁库.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveYuanqing {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_shuchen(player, dying, renku_def)
    ///
    /// ABI 444: 疏陈 — when another player enters dying (once per round) and the
    /// 仁库 holds more than three cards, gain them all and recover that player.
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveShuchen {
        /// Skill owner.
        player: ExprIr,
        /// The dying player.
        dying: ExprIr,
        /// 怨清 definition whose pile is the 仁库.
        renku_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_duanbi(player, stage)
    ///
    /// ABI 445: 断笔 — limited skill: when the total hand count exceeds twice the
    /// living player count, choose any number of players; each discards
    /// min(3, floor(hand/2)); afterwards one player gains up to three of those
    /// discarded cards still in the discard pile (authoritative random).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveDuanbi {
        /// Skill owner.
        player: ExprIr,
        /// Stage 0 open discard, 2 open gain target, 3 commit gain.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_zaoli(player, mark)
    ///
    /// ABI 446: 躁厉 — at turn start with 躁厉 marks, discard at least one card
    /// (up to all) and draw mark+discarded; losing 1 HP when the total exceeds
    /// two. Play phase only allows hand cards gained this turn (modifier).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveZaoli {
        /// Skill owner.
        player: ExprIr,
        /// 躁厉 mark count at activation.
        mark: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_guying(player, actor, card, stage)
    ///
    /// ABI 447: 孤影 — after the owner loses one hand/equip/judge card (once per
    /// turn, another player's phase), that actor chooses between randomly giving
    /// the owner one gainable card, or the owner taking back the lost card (if
    /// still in the discard pile).
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectResolveGuying {
        /// Skill owner.
        player: ExprIr,
        /// Current-phase actor making the choice.
        actor: ExprIr,
        /// The lost card.
        card: ExprIr,
        /// Stage 0 open two-choice, 1 commit branch.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// effect modify_current_damage(player, delta)
    ///
    /// Appended to keep existing serialized enum ordinals stable.
    EffectModifyCurrentDamage {
        /// Skill owner requesting the modification.
        player: ExprIr,
        /// Delta applied to the current damage amount (may be negative).
        delta: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_potyuji(player, target, card, action)
    ///
    /// Appended for serialized ordinal stability (ABI 570).
    EffectResolvePotyuji {
        /// Skill owner or beneficiary.
        player: ExprIr,
        /// Event actor/target.
        target: ExprIr,
        /// Triggering physical card.
        card: ExprIr,
        /// Public action 0..=8.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_rehengjiang(player, actor, action, hand_limit_def, draw_def)
    ///
    /// Appended for serialized ordinal stability (ABI 572).
    EffectResolveRehengjiang {
        /// Skill owner.
        player: ExprIr,
        /// Turn actor for settlement.
        actor: ExprIr,
        /// Public action 0..=1.
        action: u8,
        /// Temporary hand-limit modifier definition.
        hand_limit_def: u32,
        /// Temporary draw-settlement definition.
        draw_def: u32,
        /// Source span.
        span: Span,
    },
    /// Permanently grant the first configured skill definition the player does not own.
    /// Appended for serialized ordinal stability (ABI 575).
    EffectAddFirstMissingSkill {
        /// Skill owner.
        player: ExprIr,
        /// Three ordered candidate skill definitions.
        skill_defs: [u32; 3],
        /// Source span.
        span: Span,
    },
    /// Resolve XIANDING 糜夫人【存嗣】and the deferred【闺秀】recovery. ABI 578.
    /// Appended for serialized ordinal stability.
    EffectResolveDccunsi {
        /// Skill owner.
        player: ExprIr,
        /// Selected recipient of【勇决】.
        target: ExprIr,
        ///【闺秀】definition checked by the deferred recovery.
        guixiu_def: u32,
        ///【勇决】definition granted to the target.
        yongjue_def: u32,
        /// Public stage 0; stage 1 is engine-owned deferred settlement.
        stage: u8,
        /// Source span.
        span: Span,
    },
    /// Bind every target of the exact currently resolving Sha to this skill. ABI 580.
    /// Appended for serialized ordinal stability.
    EffectRecordCurrentShaAllTargets {
        /// Sha user.
        player: ExprIr,
        /// Exact logical Sha carrier.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_give_or_discard_hand(player, payment_count, scope)
    ///
    /// Force the player to give exactly `payment_count` hand cards to one living target in
    /// `scope`, or discard that many hand cards. ABI 581.
    /// Appended for serialized ordinal stability.
    EffectChooseGiveOrDiscardHand {
        /// Skill owner choosing and paying cards.
        player: ExprIr,
        /// Exact hand-card payment count.
        payment_count: ExprIr,
        /// Recipient target scope.
        scope: DrawTargetScopeIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_hand_equip_then_recover_target(chooser, target, count, amount)
    ///
    /// Let `chooser` cancel or discard exactly `count` current hand/equipment cards, then
    /// recover `target` by `amount`. ABI 582. Appended for serialized ordinal stability.
    EffectChooseDiscardHandEquipThenRecoverTarget {
        /// Skill owner and chooser.
        chooser: ExprIr,
        /// Living wounded recovery target.
        target: ExprIr,
        /// Exact discard count.
        count: ExprIr,
        /// Maximum HP recovered after the discard resolves.
        amount: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect choose_discard_hand_range(player, min, max)
    ///
    /// Force a choice of discardable hand cards within the inclusive range. ABI 596.
    /// Appended for serialized ordinal stability.
    EffectChooseDiscardHandRange {
        /// Player choosing and discarding cards.
        player: ExprIr,
        /// Inclusive minimum selection count.
        min: u32,
        /// Inclusive maximum selection count.
        max: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dcctjiuxian(player, event_player, action)
    ///
    /// Resolve XIANDING 陈泰「矜贤」. ABI 597. Appended for serialized ordinal stability.
    EffectResolveDcctjiuxian {
        /// Skill owner.
        player: ExprIr,
        /// Damaged Duel target for the recovery branch; `self` for activation.
        event_player: ExprIr,
        /// 0=start active branch, 2=start damage recovery branch.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_bianyue(player, target, action)
    ///
    /// Resolve XIANDING 卞玥「庇族/无胁」. ABI 598. Appended for serialized ordinal stability.
    EffectResolveBianyue {
        /// Skill owner.
        player: ExprIr,
        /// Event player placeholder; the target is selected by the effect when needed.
        target: ExprIr,
        /// 0=庇族 active branch, 10=无胁 End-phase branch.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_zhupeilan(player, target, card, helper_def, action)
    ///
    /// Resolve XIANDING 朱佩兰「辞虑/痛悼」. ABI 600. Appended for serialized ordinal stability.
    EffectResolveZhupeilan {
        /// Skill owner.
        player: ExprIr,
        /// Event source or target placeholder.
        target: ExprIr,
        /// Triggering trick carrier, or none for 痛悼.
        card: ExprIr,
        /// Package-only helper carrying exact-card policies; zero for 痛悼.
        helper_skill_def: u32,
        /// 0=辞虑 targeted, 1=prevent tracked damage, 2=tracked card resolved,
        /// 10=痛悼 dying branch.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dc_sb_zhangxiu(player, target, action)
    ///
    /// Resolve XIANDING 势张绣「附袭/豪义」. ABI 601. Appended for ordinal stability.
    EffectResolveDcSbZhangxiu {
        /// Skill owner.
        player: ExprIr,
        /// Other Play-phase player, or placeholder for 豪义.
        target: ExprIr,
        /// 0=附袭, 10=豪义.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dc_sb_zhouyu(player, source, card, action)
    ///
    /// Resolve XIANDING 新杀谋周瑜「融火/英谋」. ABI 602. Appended for ordinal stability.
    EffectResolveDcSbZhouyu {
        /// Skill owner.
        player: ExprIr,
        /// Triggering card user, or placeholder.
        source: ExprIr,
        /// Triggering or tracked card, or placeholder.
        card: ExprIr,
        /// 0/1=融火, 10/11/20=英谋.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dc_sb_lusu(player, source, card, action)
    ///
    /// Resolve XIANDING 新杀谋鲁肃「明势/盟谋」. ABI 603. Appended for ordinal stability.
    EffectResolveDcSbLusu {
        /// Skill owner.
        player: ExprIr,
        /// Triggering transfer source or tracked Sha user.
        source: ExprIr,
        /// Triggering or tracked card.
        card: ExprIr,
        /// 0=明势, 10/11/20/21=盟谋.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_zhangjian(player, card, grant_skill_def, action)
    ///
    /// Resolve XIANDING 张臶「技能A/技能B」. ABI 604. Appended for ordinal stability.
    EffectResolveZhangjian {
        /// Skill owner.
        player: ExprIr,
        /// Triggering damage card, or placeholder.
        card: ExprIr,
        /// Granted skill definition for 技能B.
        grant_skill_def: u32,
        /// 0=技能A, 1=技能B.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_zhugeruoxue(player, target, card, action)
    ///
    /// Resolve XIANDING 诸葛若雪「琼英/暖惠」. ABI 605. Appended for ordinal stability.
    EffectResolveZhugeruoxue {
        /// Skill owner.
        player: ExprIr,
        /// Target or placeholder.
        target: ExprIr,
        /// Field card or placeholder.
        card: ExprIr,
        /// 0=琼英, 10=暖惠.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_caoyi(player, card, helper_skill_def, action)
    ///
    /// Resolve XIANDING 曹轶「蜜饴/寅君」. ABI 606. Appended for ordinal stability.
    EffectResolveCaoyi {
        /// Skill or helper owner.
        player: ExprIr,
        /// Triggering card or placeholder.
        card: ExprIr,
        /// Package-only 蜜饴 helper definition.
        helper_skill_def: u32,
        /// 0, 10, 11, 20, or 21.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_malingli(player, card, helper_skill_def, action)
    ///
    /// Resolve XIANDING 马伶俐「硝引/花火」. ABI 607. Appended for ordinal stability.
    EffectResolveMalingli {
        /// Skill or helper owner.
        player: ExprIr,
        /// Triggering card or placeholder.
        card: ExprIr,
        /// Package-only helper definition.
        helper_skill_def: u32,
        /// Public action 0, 10, or 20.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_nsshuaiyan(player, recovered_player)
    ///
    /// Resolve archived extension 玩点论杀 费诗「率言」. ABI 608.
    EffectResolveNsshuaiyan {
        /// Skill owner.
        player: ExprIr,
        /// Other player who recovered and chooses the branch.
        recovered_player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wdb_hanlong(player, target, card, helper_skill_def, action)
    ///
    /// Resolve archived extension 玩点论杀 韩龙「伺机/刺酋」. ABI 609.
    EffectResolveWdbHanlong {
        /// Skill owner.
        player: ExprIr,
        /// Current damage target or placeholder.
        target: ExprIr,
        /// Current damage card or placeholder.
        card: ExprIr,
        /// Package-only 刺酋濒死 helper definition.
        helper_skill_def: u32,
        /// Public action 0 or 1.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wdb_liufu(player, target, card, zhucheng_skill_def, action)
    ///
    /// Resolve archived extension 玩点论杀 刘馥「筑城/夺气」. ABI 610.
    EffectResolveWdbLiufu {
        /// Skill owner.
        player: ExprIr,
        /// Sha user/current Play-phase player or placeholder.
        target: ExprIr,
        /// Current Sha or placeholder.
        card: ExprIr,
        /// Stable 筑城 definition owning the public pile.
        zhucheng_skill_def: u32,
        /// Public action 0, 1, or 2.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wdb_liuyan(player, card, action)
    ///
    /// Resolve archived extension 玩点论杀 刘焉「绝道/割据」. ABI 611.
    EffectResolveWdbLiuyan {
        /// Skill owner.
        player: ExprIr,
        /// Selected active cost or placeholder.
        card: ExprIr,
        /// Public action 0 or 1.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wdb_liuzan(player, target, helper_skill_def, action)
    ///
    /// Resolve archived extension 玩点论杀 留赞「亢音」. ABI 612.
    EffectResolveWdbLiuzan {
        /// Skill owner.
        player: ExprIr,
        /// Selected other player or continuation placeholder.
        target: ExprIr,
        /// Stable definition of the temporary modifier helper.
        helper_skill_def: u32,
        /// Public action 0; engine-owned continuations 1..=3.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wdb_tianyu(player, target, card, action)
    ///
    /// Resolve archived extension 玩点论杀 田豫「诱战」. ABI 613.
    EffectResolveWdbTianyu {
        /// Skill owner.
        player: ExprIr,
        /// Target of the incoming Sha.
        target: ExprIr,
        /// Incoming Sha card.
        card: ExprIr,
        /// Public action 0; engine-owned continuations use later stages.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wdb_xizhenxihong(player, target, card, action)
    ///
    /// Resolve archived extension 玩点论杀 习珍习宏「负仇」. ABI 614.
    EffectResolveWdbXizhenxihong {
        /// Skill owner.
        player: ExprIr,
        /// Incoming Sha source for action 0; unused for action 1.
        target: ExprIr,
        /// Incoming Sha for action 0; unused for action 1.
        card: ExprIr,
        /// 0=become Sha target; 1=turn-ended settlement.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wdb_yangyi(player, target, card, action)
    ///
    /// Resolve archived extension 玩点论杀 杨仪「筹度/立断」. ABI 615.
    EffectResolveWdbYangyi {
        /// Skill owner.
        player: ExprIr,
        /// Chosen 调兵遣将 user for 筹度; gained-card recipient for 立断.
        target: ExprIr,
        /// Selected 筹度 cost or the single equipment gained for 立断.
        card: ExprIr,
        /// 0=筹度; 1=立断.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_wdb_zaozhirenjun(player, target, card, action)
    ///
    /// Resolve archived extension 玩点论杀 枣祗任峻. ABI 616.
    EffectResolveWdbZaozhiRenjun {
        /// Skill owner.
        player: ExprIr,
        /// Selected target, if any.
        target: ExprIr,
        /// Current card carrier, if any.
        card: ExprIr,
        /// Public action 0=粮策; 1=坚壁.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dddxianxi(player, card, action)
    ///
    /// Resolve archived extension 3D精选 韩当「险袭」. ABI 617.
    EffectResolveDddXianxi {
        /// Skill owner and Sha user.
        player: ExprIr,
        /// Current physical Sha carrier.
        card: ExprIr,
        /// 0=choose extra targets; 1=settle undamaged extra targets.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_dddxiaheng(player, action)
    ///
    /// Resolve archived extension 3D精选 周处「侠横」. ABI 618.
    EffectResolveDddXiaheng {
        /// Skill owner.
        player: ExprIr,
        /// Public action 0 starts the forced settlement.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_ddd_liuchong(player, actor, card, action)
    ///
    /// Resolve archived extension 3D精选 刘宠「精彀/末焱」. ABI 619.
    EffectResolveDddLiuchong {
        /// Skill owner.
        player: ExprIr,
        /// Event actor.
        actor: ExprIr,
        /// Triggering physical card, if any.
        card: ExprIr,
        /// 0=末焱轮首; 1=末焱受伤增伤; 2=精彀武器结算.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_ddd_duanbing(player, target, card, action)
    ///
    /// Resolve archived extension 3D精选 丁奉「短兵」. ABI 621.
    EffectResolveDddDuanbing {
        /// Skill owner.
        player: ExprIr,
        /// Damage target for the transfer branch, otherwise none.
        target: ExprIr,
        /// Damage card carrier for the transfer branch, otherwise none.
        card: ExprIr,
        /// 0=start active branch; 1=transfer Bingliang after card damage.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_ddd_wuzhi(player, card, action)
    ///
    /// Resolve archived extension 3D精选 吴质「灵涌/虚孝」. ABI 622.
    EffectResolveDddWuzhi {
        /// Skill owner.
        player: ExprIr,
        /// Judgment or selected physical card, otherwise none.
        card: ExprIr,
        /// 0=灵涌判定；10=虚孝弃置牌处理。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_ddd_xujing(player, source, card, action)
    ///
    /// Resolve archived extension 3D精选 许靖「虚誉/实荐」. ABI 623.
    EffectResolveDddXujing {
        /// Skill owner.
        player: ExprIr,
        /// Current card user, otherwise none.
        source: ExprIr,
        /// Current physical card, otherwise none.
        card: ExprIr,
        /// 0/1/2=虚誉发动/债务/清理；10/11/12=实荐赠牌/后续/清理。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_ddd_caomao(player, action)
    ///
    /// Resolve archived extension 3D精选 曹髦「太思/驱车」. ABI 624.
    EffectResolveDddCaomao {
        /// Skill owner.
        player: ExprIr,
        /// 0=太思；10=驱车主动入口。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_ddd_xinxianying(player, actor, action)
    ///
    /// Resolve archived extension 3D精选 辛宪英「洞察」. ABI 625.
    EffectResolveDddXinxianying {
        /// Skill owner.
        player: ExprIr,
        /// Current turn player.
        actor: ExprIr,
        /// 0=双方置鉴；1=回合末依次取鉴。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// effect resolve_ddd_xianglang(player, actor, action)
    ///
    /// Resolve archived extension 3D精选 向朗「恰化/腹笥/拓籍」. ABI 626.
    EffectResolveDddXianglang {
        /// Skill owner.
        player: ExprIr,
        /// Current turn player, borrower, or none.
        actor: ExprIr,
        /// 0=恰化；1=腹笥全局初始化；2=拓籍借牌后结算。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 于禁「整军」复合（ABI 627）。
    EffectResolveDddYujin {
        /// Skill owner.
        player: ExprIr,
        /// Player whose HP/cards changed.
        actor: ExprIr,
        /// Public action: 0=HP change, 1=cards moved.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 刘晔「察势/齐策」复合（ABI 628）。
    EffectResolveDddLiuye {
        /// Skill owner.
        player: ExprIr,
        /// Current Play-phase player or card user.
        actor: ExprIr,
        /// Current used card, or none for phase actions.
        card: ExprIr,
        /// 0=察势布置，1=察势命中，2=齐策准备，3=齐策解锁，4=察势过期。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 鲍三娘「枝连/赍剑」复合（ABI 629）。
    EffectResolveDddBaosanniang {
        /// Skill owner.
        player: ExprIr,
        /// 0=枝连回合末，1=赍剑主动技。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 甄宓「淼形/浮世」复合（ABI 630）。
    EffectResolveDddZhenji {
        /// Skill owner.
        player: ExprIr,
        /// 0=淼形起手拆组，1=淼形摸牌阶段后平衡，2=浮世整手交换。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 赵昂「奋计」（ABI 631）。
    EffectResolveDddZhaoang {
        /// Skill owner.
        player: ExprIr,
        /// 0=摸牌阶段，1=弃牌阶段。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 刘巴「丰政/玉律」（ABI 632）。
    EffectResolveDddLiuba {
        /// Skill owner or global-helper user.
        player: ExprIr,
        /// Current turn player when relevant.
        actor: ExprIr,
        /// 0=丰政转化，1=轮首分配，2=安装全局技，3=玉律起始，4=玉律响应。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 蹇硕「分野/逝潮」（ABI 633）。
    EffectResolveDddJianshuo {
        /// Skill owner.
        player: ExprIr,
        /// Active comparison target or damage source when relevant.
        target: ExprIr,
        /// 0=分野，1=逝潮准备阶段，2=逝潮伤害前。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 管宁「游学/承经」（ABI 634）。
    EffectResolveDddGuanning {
        /// Skill owner.
        player: ExprIr,
        /// Current round/turn actor when relevant.
        actor: ExprIr,
        /// 0=游学轮首，1=承经回合末，2=承经主动使用。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 曹爽「专摄/危秋」（ABI 635）。
    EffectResolveDddCaoshuang {
        /// Skill owner.
        player: ExprIr,
        /// Current actor or recovering player.
        actor: ExprIr,
        /// Current used card, or none.
        card: ExprIr,
        /// 0=赠牌，1=同名牌使用，2=回合末，3=危秋。
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 薛灵芸「涟洏/暗织」（ABI 636）。
    EffectResolveDddXuelingyun {
        /// Skill owner.
        player: ExprIr,
        /// Current actor.
        actor: ExprIr,
        /// Event card.
        card: ExprIr,
        /// Public action tag.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 刘宏「失兴/党锢/覆宗」（ABI 637）。
    EffectResolveDddLiuhong {
        /// Skill owner.
        player: ExprIr,
        /// Current turn actor, changed player, or dead 群 character.
        actor: ExprIr,
        /// Public action tag.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 夏侯玄「朗怀/玄论」（ABI 638）。
    EffectResolveDddXiahouxuan {
        /// Skill owner.
        player: ExprIr,
        /// Public action tag.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 张闿「劫行/拜泪」（ABI 639）。
    EffectResolveDddZhangkai {
        /// Skill owner.
        player: ExprIr,
        /// Current actor or event player.
        actor: ExprIr,
        /// Event card.
        card: ExprIr,
        /// Public action tag.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// 3D精选 梁习「仝御」（ABI 640）。
    EffectResolveDddLiangxi {
        /// Skill owner.
        player: ExprIr,
        /// Current actor or event player.
        actor: ExprIr,
        /// Event card.
        card: ExprIr,
        /// Public action tag.
        action: u8,
        /// Source span.
        span: Span,
    },
    /// Guozhan: spend a 珠联璧合 mark to draw two cards (ABI 641, ADR 0604).
    EffectGuozhanUseZhulianBihe {
        /// Skill owner.
        player: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 欢乐卡牌【手里剑】使用结算（ABI 642, ADR 0607）。
    EffectResolveShoulijian {
        /// Card user.
        player: ExprIr,
        /// Card target.
        target: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 取消使用牌（ABI 643, ADR 0608）：成为其他角色使用牌的目标时，取消该牌对当前
    /// 角色生效并获取对其使用的实体牌（gwent 杜达元帅1 特殊卡）。
    EffectCancelIncomingCardUse {
        /// Card holder being targeted.
        player: ExprIr,
        /// Card user whose use is cancelled.
        source: ExprIr,
        /// The used card being cancelled.
        card: ExprIr,
        /// Source span.
        span: Span,
    },
    /// 择一加技能（ABI 644, ADR 0612）：玩家在两个技能定义间二选一，将所选技能置入
    /// 目标（gwent 呼朋 hupeng 择一杜达元帅1/2）。
    EffectChooseOtherGainOneOfTwoSkills {
        /// Chooser (skill owner).
        player: ExprIr,
        /// Recipient of the chosen skill.
        target: ExprIr,
        /// First skill definition option.
        first_def: u32,
        /// Second skill definition option.
        second_def: u32,
        /// Source span.
        span: Span,
    },
    /// 叶舞（gwent）随机弃牌链：宿主原语内以种子 RNG 循环，并造卡（雾妖/狼 以技能
    /// 近似）。ABI 645 (ADR 0627).
    EffectGwyewuRandomDiscardChain {
        /// Skill owner (self).
        player: ExprIr,
        /// 雾妖 skill definition (gained per enemy hand card discarded).
        wuyao_def: u32,
        /// 狼 skill definition (gained per enemy equip card discarded).
        lang_def: u32,
        /// Source span.
        span: Span,
    },
    /// effect resolve_ollinjie(player, source) — 临界 (sp/ollinjie): the damage source is
    /// forced to discard one hand card; if the discarded card is effectively a Sha, the
    /// owner draws one. ABI 608.
    EffectResolveOllinjie {
        /// Skill owner who draws if the discarded card is a Sha.
        player: ExprIr,
        /// Damage source forced to discard one hand card.
        source: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect discard_hand_expr(player, count) — forced hand discard with an
    /// expression-driven count (e.g. `7 - mark_count(self, def)`). Compiles to the
    /// same HostEffect op as literal-count `discard_hand`; no ABI bump required.
    EffectDiscardHandExpr {
        /// Discarding player.
        player: ExprIr,
        /// Number of cards (clamped to the available hand).
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect discard_hand_equip_expr(player, count) — forced hand-or-equip discard
    /// with an expression-driven count. Same HostEffect op as `discard_hand_equip`.
    EffectDiscardHandEquipExpr {
        /// Discarding player.
        player: ExprIr,
        /// Number of cards (clamped to the available hand+equip).
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect choose_target_gain_max_hp(player, self_skill) — 寿春 (sp/pxshouchun):
    /// the owner selects any living player (self included); that player gains 1 max hp.
    /// ABI 646.
    EffectChooseTargetGainMaxHp {
        /// Choosing player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect gain_target_hand_card(player, self_skill) — 肃州 (sp/pxsizhou):
    /// the owner selects another living player, then gains one of that player's
    /// hand cards. ABI 647.
    EffectGainTargetHandCard {
        /// Gaining player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect choose_clan_not_max_hand_limit_gain(player, self_skill) — 穆荫
    /// (clan/clanmuyin): select a same-clan-or-self player whose hand limit is
    /// not the field max; that player gains +1 hand limit (persistent).
    /// ABI 648.
    EffectChooseClanNotMaxHandLimitGain {
        /// Choosing player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect extra_resolve_current_trick(player, card, self_skill) — 额外结算
    /// family (clanfenshi): queue one extra effect pass for the suspended
    /// trick. ABI 649.
    EffectExtraResolveCurrentTrick {
        /// Owning player.
        player: ExprIr,
        /// Suspended trick source card.
        card: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect place_hand_card_to_expansion(player, self_skill) — 置入
    /// (expansion P2, ADR 0641): place one selected hand card onto the skill
    /// pile. ABI 650.
    EffectPlaceHandCardToExpansion {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect gain_from_skill_pile(player, self_skill) — 取用
    /// (expansion P1, ADR 0641): select one card from own skill pile
    /// into hand. ABI 651.
    EffectGainFromSkillPile {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect discard_skill_pile(player, self_skill) — 弃置自身技能堆全部牌
    /// (expansion S1-B, ADR 0641 §8.2)。ABI 684。
    EffectDiscardSkillPile {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Skill definition owning the pile (u32 literal).
        pile_def: u32,
        /// Span.
        span: Span,
    },
    /// effect place_top_deck_to_expansion(player, self_skill, pile_def, count)
    /// — 置入 (expansion P2-var, ADR 0641 §8.7)。ABI 685。
    EffectPlaceTopDeckToExpansion {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Skill definition owning the pile (u32 literal).
        pile_def: u32,
        /// Number of cards.
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect choose_one_from_skill_pile_discard(player, self_skill, pile_def)
    /// — 选堆内单卡弃置 (expansion S1-B, ADR 0641 §8.7)。ABI 686。
    EffectChooseOneFromSkillPileDiscard {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Skill definition owning the pile (u32 literal).
        pile_def: u32,
        /// Span.
        span: Span,
    },
    /// effect choose_target_hand_card_to_pile(player, self_skill, pile_def)
    /// — 置入他人手牌 (expansion A1, ADR 0641 §8.7)。ABI 688。
    EffectChooseTargetHandCardToPile {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Skill definition owning the pile (u32 literal).
        pile_def: u32,
        /// Span.
        span: Span,
    },
    /// effect rearrange_pile_to_deck_top(player, self_skill, pile_def, count)
    /// — 堆重排 (chooseToMove family, ADR 0641 §8.8)。ABI 690。
    EffectRearrangePileToDeckTop {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Skill definition owning the pile (u32 literal).
        pile_def: u32,
        /// Number of cards to rearrange.
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_qianlong(player, self_skill) — 专属状态机 (批 241)。
    EffectResolveQianlong {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_jsrgzuozhan(player, self_skill) — 专属状态机 (批 242)。
    EffectResolveJsrgzuozhan {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_twmouzhi(player, self_skill, mode, event_card) — 谋制
    /// 专属状态机 (批 245)。
    EffectResolveTwmouzhi {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// 0 = damage_before check/prevent, 1 = damage record.
        mode: ExprIr,
        /// Damage event card (0 if none).
        event_card: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_twdaigui(player, self_skill, stage, count) — 待归多段
    /// 状态机 (批 247, slice 1)。
    EffectResolveTwdaigui {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Machine stage (0 = entry/target selection, 2 = reveal bottom cards).
        stage: ExprIr,
        /// Number of revealed cards (target count bound at stage 2).
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_twmitu(player, self_skill, stage, count) — 迷途多段
    /// 拼点链状态机 (批 249, slice 1)。
    EffectResolveTwmitu {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Machine stage (0 = entry/target selection).
        stage: ExprIr,
        /// Reserved argument (upstream symmetry).
        count: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_qiantun(player, self_skill) — 谦吞拼点族状态机 (批 252)。
    EffectResolveQiantun {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_yangming(player, self_skill) — 养名循环拼点状态机
    /// (批 253)。
    EffectResolveYangming {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_canyun(player, self_skill) — 残殒装备比较状态机
    /// (批 255)。
    EffectResolveCanyun {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_hengwu(player, self_skill, event_card) — 横骛花色计数
    /// 状态机 (批 256)。
    EffectResolveHengwu {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Used/responded card.
        event_card: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_biaozhao3(player, self_skill) — 表召·归状态机 (批 257)。
    EffectResolveBiaozhao3 {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_biaozhao2(player, self_skill, event_player) — 表召·应
    /// 状态机 (批 259)。
    EffectResolveBiaozhao2 {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// The discarder.
        event_player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_dcniji(player, self_skill, mode, event_card) — 逆击
    /// 状态机 (批 260)。
    EffectResolveDcniji {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// 0 = targeted (draw+tag), 1 = phase end (discard tagged).
        mode: ExprIr,
        /// The targeting card (mode 0).
        event_card: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_dcmoukui(player, self_skill, stage, target) — 谋溃
    /// 状态机 (批 261)。
    EffectResolveDcmoukui {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Machine stage (0 = entry choice, 3 = discard pick).
        stage: ExprIr,
        /// The sha target.
        target: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_zhongyan(player, self_skill, stage) — 忠言状态机
    /// (批 262)。
    EffectResolveZhongyan {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Machine stage (0 = entry/target selection).
        stage: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_chumei(player, self_skill, stage) — 狐魅状态机
    /// (批 263)。
    EffectResolveChumei {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Machine stage (0 = entry/target selection, 2 = option choice).
        stage: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_mbbeiming(player, self_skill, stage) — 孛明状态机
    /// (批 269)。
    EffectResolveMbbeiming {
        /// Owning player.
        player: ExprIr,
        /// Skill instance.
        skill: ExprIr,
        /// Machine stage (0 = entry/target selection).
        stage: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect reset_linked_selected_targets(source) — 断索主动原语
    /// (批 270)：重置所选全部连环角色，再各造成 1 点火焰伤害。
    EffectResetLinkedSelectedTargets {
        /// Damage source (skill owner).
        source: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_duoduan(player, source, card) — 度断状态机
    /// (批 272)：重铸一张牌后选一项——使用者摸 2 且此杀对 owner 无效，
    /// 或使用者弃一张牌且 owner 不可响应此杀。
    EffectResolveDuoduan {
        /// Sha target holding the skill.
        player: ExprIr,
        /// Sha user.
        source: ExprIr,
        /// Physical Sha card.
        card: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_qiangyong(player, target) — 羌勇状态机 (批 278)：
    /// 【杀】造成伤害时弃置受伤角色 X 张牌（X=本回合用杀数），
    /// 其无手牌则此伤害 +1。
    EffectResolveQiangyong {
        /// Damage source (skill owner).
        player: ExprIr,
        /// Wounded player.
        target: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_dcqiaolve(player) — 巧略状态机 (批 286)：先弃自己
    /// 一张牌，再取事件目标一张随机手牌（弃牌代价补全面）。
    EffectResolveDcqiaolve {
        /// Skill owner.
        player: ExprIr,
        /// Event target.
        target: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_twxiongjun(player) — 雄军状态机 (批 287)：owner 造成
    /// 伤害后，每名拥有〖雄军〗的角色各摸一张牌。
    EffectResolveTwxiongjun {
        /// Damage source (skill owner).
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_yixiang(player) — 义襄状态机 (批 288)：每名角色回合
    /// 限一次，成为使用牌目标时随机获得牌堆中一张自己没有的基本牌。
    EffectResolveYixiang {
        /// Skill owner.
        player: ExprIr,
        /// Span.
        span: Span,
    },
    /// effect resolve_xinfu_qiai(player) — 求援状态机 (批 291)：限定技，濒死
    /// 时其他角色依座次各交给 owner 一张牌（手+装备，强制，跳过无牌者）。
    EffectResolveXinfuQiai {
        /// Dying skill owner receiving the cards.
        player: ExprIr,
        /// Span.
        span: Span,
    },
        ] }
    };
}
pub(super) use variants;
