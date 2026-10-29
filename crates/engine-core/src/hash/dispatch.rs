//! 效果请求与挂起触发的规范哈希分派（按变体路由到各 effects_* 分组）。

use super::*;

pub(super) fn hash_pending_trigger(h: &mut blake3::Hasher, task: &crate::trigger::PendingTrigger) {
    put_u32(h, task.active.skill_id.raw());
    put_u32(h, task.active.skill_def.raw());
    put_u32(h, task.active.owner.raw());
    put_u32(h, task.active.trigger_def.raw());
    put_u32(h, task.active.event.tag());
    put_u32(h, task.active.timing as u32);
    put_i32(h, task.active.priority);
    put_bool(h, task.active.optional);
    put_bool(h, task.active.enabled);
    put_u32(h, task.context.kind.tag());
    put_u32(h, task.context.player.raw());
    put_u32(h, task.context.source.raw());
    put_u32(h, task.context.respond_to.raw());
    put_u32(h, task.context.amount);
    put_u8(h, task.context.nature);
    put_u32(h, task.context.turn_damage_sources);
    put_u32(h, task.context.turn_damaged_players);
    put_u8(h, card_move_reason_tag(task.context.move_reason));
    put_bool(h, task.context.moved_has_basic);
    put_u8(h, task.context.from_zone.map(zone_kind_tag).unwrap_or(0));
    put_u32(h, task.context.to_zone.map(|zone| zone.raw()).unwrap_or(0));
    put_u32(h, task.context.card.map(|card| card.raw()).unwrap_or(0));
    put_u8(h, task.context.card_kind.unwrap_or(0));
    put_u8(h, task.context.card_flags);
    put_u32(h, task.context.target_count);
    put_ids(h, task.context.targets.iter().map(|player| player.raw()));
    put_ids(h, task.context.cards.iter().map(|card| card.raw()));
    put_u32(h, task.view_as_skill);
    put_u32(h, task.view_as_card);
    put_bool(h, task.history.is_some());
    if let Some(history) = &task.history {
        put_u32(h, history.previous_card_meta);
        put_u8(h, history.previous_card_kind);
        put_u32(h, history.previous_card_def);
        put_u32(h, history.previous_play_card_meta);
        put_u32(h, history.previous_game_card_meta);
        put_u32(h, history.owner_sha_used);
        put_u32(h, history.owner_juedou_used);
        put_i32(h, history.source_card_used);
        put_i32(h, history.owner_card_action_count);
        put_i32(h, history.owner_play_phase_card_used);
        put_u32(h, history.event_player_play_phase_card_used);
        put_u32(h, history.event_player_play_phase_suit_count);
        put_u32(h, history.view_as_skill_def);
        put_u32(h, history.view_as_skill_owner);
        put_bool(h, history.event_player_had_other_red_card_target);
    }
}

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::DrawCards { .. }
        | EffectRequest::ChooseDiscardOneHandOrEquip { .. }
        | EffectRequest::ChooseDiscardOrFireDamageSkipPhases { .. }
        | EffectRequest::DrawCardsFromSkill { .. }
        | EffectRequest::DrawEachBySeat { .. }
        | EffectRequest::DrawThenGiveOne { .. }
        | EffectRequest::DrawThenTakeOne { .. }
        | EffectRequest::ChooseDiscardHandAddSkillByColor { .. }
        | EffectRequest::ChooseDrawOrRecover { .. }
        | EffectRequest::ChooseUseVirtualSha { .. }
        | EffectRequest::ChooseUseVirtualShaWithBonus { .. }
        | EffectRequest::ChooseUseVirtualShaOrDraw { .. }
        | EffectRequest::ChooseDiscardTargetCard { .. }
        | EffectRequest::ChooseGainTopCardsBySuit { .. }
        | EffectRequest::ChooseOtherDrawByHandCount { .. }
        | EffectRequest::ChooseDiscardOneThenGainTargetEquipment { .. }
        | EffectRequest::ChooseTargetDraw { .. }
        | EffectRequest::DrawByPlayPhaseCardCategoryCount { .. }
        | EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed { .. }
        | EffectRequest::ChooseAddCurrentShaTarget { .. }
        | EffectRequest::ChooseStoreHandShaInSkillPile { .. }
        | EffectRequest::ChooseSpendSkillPileForDyingJiu { .. }
        | EffectRequest::ChooseOtherGiveThenRecover { .. }
        | EffectRequest::ChooseHandCompareBetween { .. }
        | EffectRequest::ChooseHandCompareGrantTargetOnWin { .. }
        | EffectRequest::ChooseCompareSelectedHandCard { .. }
        | EffectRequest::ChooseAddSkillBoundCardTarget { .. }
        | EffectRequest::ChooseDiscardMatchingEquipColorAddSkill { .. }
        | EffectRequest::ChooseCurrentCardUseEnhancements { .. }
        | EffectRequest::ChooseDrawOrDiscardAddSkill { .. }
        | EffectRequest::ChooseDiscardedBlackBasicAsBingliang { .. }
        | EffectRequest::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage { .. }
        | EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded { .. }
        | EffectRequest::ChooseRevealOtherHandCardRememberCategory { .. }
        | EffectRequest::DrawThenGiveExactRecordPhaseTarget { .. }
        | EffectRequest::ChooseGainResolvedShaOrShanThenOptionalUseSha { .. }
        | EffectRequest::ChooseUseShaAfterResolvedGift { .. }
        | EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage { .. }
        | EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw { .. }
        | EffectRequest::ChooseCardUserOrAddTarget { .. }
        | EffectRequest::ChooseDiscardEquipmentReduceCurrentDamage { .. }
        | EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage { .. }
        | EffectRequest::ChooseMoveTargetEquipmentOrDraw { .. }
        | EffectRequest::ChooseRecoverOwnerOrDamageByGivenEquipment { .. }
        | EffectRequest::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage { .. }
        | EffectRequest::ChooseHandCountDifferentTargetDrawOrRandomDiscard { .. }
        | EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw { .. }
        | EffectRequest::ChooseSelfDiscardOrTargetDrawAfterTargetDiscard { .. }
        | EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors { .. }
        | EffectRequest::ChoosePreventDamageOrLoseMaxHpIncreaseDamage { .. }
        | EffectRequest::ChooseAddOrRemoveCurrentCardTarget { .. }
        | EffectRequest::ChooseHandCompare { .. }
        | EffectRequest::ChooseOtherDamage { .. }
        | EffectRequest::ChooseOtherDiscardTargetCard { .. } => {
            effects_draw_cards_choose_draw::hash_effect(h, effect)
        }
        EffectRequest::ModifyCurrentDamage { .. }
        | EffectRequest::MoveCards { .. }
        | EffectRequest::MoveCardsWithReason { .. }
        | EffectRequest::UseCard { .. }
        | EffectRequest::DealDamage { .. }
        | EffectRequest::RecoverHp { .. }
        | EffectRequest::ChangeMaxHp { .. }
        | EffectRequest::AddSkill { .. }
        | EffectRequest::RemoveSkill { .. }
        | EffectRequest::AddMark { .. }
        | EffectRequest::RemoveMark { .. }
        | EffectRequest::RevealCards { .. }
        | EffectRequest::RequestChoice { .. }
        | EffectRequest::EnterDying { .. }
        | EffectRequest::KillPlayer { .. }
        | EffectRequest::SetTurnState { .. }
        | EffectRequest::EmitDebateResolved { .. }
        | EffectRequest::EmitRuleEvent { .. }
        | EffectRequest::DiscardFromHand { .. }
        | EffectRequest::DiscardFromHandEquip { .. }
        | EffectRequest::TakeFromHand { .. }
        | EffectRequest::DiscardCards { .. }
        | EffectRequest::CardResponded { .. }
        | EffectRequest::RemoveCardsByKind { .. }
        | EffectRequest::DealShaDamage { .. }
        | EffectRequest::JudgeGainEquipmentByColor { .. }
        | EffectRequest::RevealDiscardSuitGainOrUse { .. }
        | EffectRequest::DiscardAllHandAndEquip { .. }
        | EffectRequest::RequestDyingSave { .. }
        | EffectRequest::TakeOneCard { .. }
        | EffectRequest::TurnOver { .. }
        | EffectRequest::RemoveSkillByDef { .. }
        | EffectRequest::RevealTopGainNonBasicOrSwapBasic { .. }
        | EffectRequest::PreventDamage { .. }
        | EffectRequest::CommitDamage { .. }
        | EffectRequest::JudgeCancelCurrentShaTargetOnRed { .. }
        | EffectRequest::DealCardDamage { .. }
        | EffectRequest::AdjustHandTo { .. }
        | EffectRequest::GainCardByKind { .. }
        | EffectRequest::JudgeRecoverOnSuit { .. }
        | EffectRequest::GiveCards { .. }
        | EffectRequest::UseVirtualCardOn { .. }
        | EffectRequest::UseFieldCard { .. }
        | EffectRequest::EmitDuelResolved { .. }
        | EffectRequest::JudgeDamageOtherOnSuit { .. }
        | EffectRequest::RecastCards { .. }
        | EffectRequest::DisableEquipSlots { .. }
        | EffectRequest::MoveEquipmentThenDraw { .. }
        | EffectRequest::DiscardEquipmentPairWithArmor { .. }
        | EffectRequest::DiscardTargetEquipmentGainMount { .. }
        | EffectRequest::DiscardTargetCardGainMount { .. }
        | EffectRequest::DiscardTargetCards { .. }
        | EffectRequest::MakeCurrentShaTargetUnrespondable { .. }
        | EffectRequest::ShaDodged { .. }
        | EffectRequest::JudgePreventDamageLoseMaxHpOnNonHeart { .. }
        | EffectRequest::RefundCurrentShaUseAndDrawIfRed { .. }
        | EffectRequest::QueueOpeningExtraPlayPhase { .. }
        | EffectRequest::BeginTurn { .. }
        | EffectRequest::FinishTurnBegin { .. }
        | EffectRequest::AdvanceOpeningExtraPlay { .. }
        | EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage { .. }
        | EffectRequest::CommitAddCurrentShaTarget { .. }
        | EffectRequest::CommitStoreHandShaInSkillPile { .. }
        | EffectRequest::CommitSpendSkillPileForDyingJiu { .. }
        | EffectRequest::GuanxingTop { .. }
        | EffectRequest::GuanxingTopByFaluMarks { .. }
        | EffectRequest::GiveSelectedThenRecoverSelf { .. }
        | EffectRequest::AddBoundSkill { .. }
        | EffectRequest::CommitAddSkillBoundCardTarget { .. }
        | EffectRequest::AddCardColorLockSkill { .. }
        | EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { .. }
        | EffectRequest::AddCardUsePolicySkill { .. }
        | EffectRequest::UseVirtualShaDrawIfNoDamage { .. }
        | EffectRequest::DiscardSelectedRecordUniqueCategoryTarget { .. }
        | EffectRequest::FinishLinkIfTargetWounded { .. }
        | EffectRequest::ClearPhaseGiftTarget { .. }
        | EffectRequest::EmitShaResolved { .. }
        | EffectRequest::SkipCurrentPhase { .. }
        | EffectRequest::ResolveOlJieDanshou { .. }
        | EffectRequest::RevealUniformHandDrawSelfChooseOthers { .. }
        | EffectRequest::CommitTransferSavedCardUser { .. }
        | EffectRequest::GiveAllEquipmentChooseRecoverOrDamage { .. }
        | EffectRequest::IncreaseCurrentDrawCount { .. }
        | EffectRequest::ContinueChooseGainOneFromDamageTargetAndNeighbors { .. }
        | EffectRequest::ContinueIncreaseBoundDamage { .. }
        | EffectRequest::ContinueChooseCompareTargetHandCard { .. }
        | EffectRequest::RandomizeCurrentHandCompareOpponent { .. }
        | EffectRequest::EmitHandCompareFinished { .. }
        | EffectRequest::GainHighestRankShaFromEventCards { .. }
        | EffectRequest::GainHandCompareResultCard { .. }
        | EffectRequest::GainDiscardCardByKind { .. }
        | EffectRequest::RevealUniformHandChooseDamage { .. } => {
            effects_draw_cards::hash_effect(h, effect)
        }
        EffectRequest::ChooseGainDiscardCardByKind { .. }
        | EffectRequest::DrawForLifetimeCardActionDivisors { .. }
        | EffectRequest::ChooseAddMissingSkill { .. }
        | EffectRequest::AddFirstMissingSkill { .. }
        | EffectRequest::ChooseTargetsDraw { .. }
        | EffectRequest::GainEventCards { .. }
        | EffectRequest::ChooseTargetDrawToMaxHpCap { .. }
        | EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { .. }
        | EffectRequest::EmitJudgmentResolved { .. }
        | EffectRequest::JudgeRecoverOtherOnSuit { .. }
        | EffectRequest::ForceOthersUseShaOnNearestOrLoseHp { .. }
        | EffectRequest::ContinueDyingSave { .. }
        | EffectRequest::RestoreCharacterState { .. }
        | EffectRequest::DiscardAllHandEquipJudge { .. }
        | EffectRequest::ChooseDiscardTargetCardResolveCurrentSha { .. }
        | EffectRequest::UseVirtualCardOnSelectedTargets { .. }
        | EffectRequest::DealDamageBanActiveIfDying { .. }
        | EffectRequest::SpjungongDiscardOrLoseThenSha { .. }
        | EffectRequest::BanActiveUntilTurnEnd { .. }
        | EffectRequest::SpwanweiActive { .. }
        | EffectRequest::SpwanweiDying { .. }
        | EffectRequest::SuppressNonLockedSkillsUntilTurnEnd { .. }
        | EffectRequest::ChooseRecoverTarget { .. }
        | EffectRequest::GiveAllHandThenChooseUseOneOrDamage { .. }
        | EffectRequest::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable { .. }
        | EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants { .. }
        | EffectRequest::ExcludeSavedShaTarget { .. }
        | EffectRequest::BanUseAgainst { .. }
        | EffectRequest::RecordCurrentShaAllTargets { .. }
        | EffectRequest::ChooseGiveOrDiscardHand { .. } => {
            effects_choose_gain_discard_card_by_kind::hash_effect(h, effect)
        }
        EffectRequest::ResolveSpxizhan { .. }
        | EffectRequest::ResolveSpsongshu { .. }
        | EffectRequest::ResolveSpcunsi { .. }
        | EffectRequest::ResolveXinmingshi { .. }
        | EffectRequest::ResolveBoming { .. }
        | EffectRequest::ResolveEjian { .. }
        | EffectRequest::ResolveDbzhuifeng { .. }
        | EffectRequest::ResolveDbquedi { .. }
        | EffectRequest::ResolveSpyinju { .. }
        | EffectRequest::ResolveSpchijie { .. }
        | EffectRequest::ResolveSpshangyi { .. }
        | EffectRequest::ResolveSpjianyi { .. }
        | EffectRequest::ResolveFyjianyu { .. }
        | EffectRequest::ResolveLiubing { .. }
        | EffectRequest::ResolveHeji { .. }
        | EffectRequest::ResolveSpyajun { .. }
        | EffectRequest::ResolveSpzundi { .. }
        | EffectRequest::ResolveXinyangjie { .. }
        | EffectRequest::ResolveQingyu { .. }
        | EffectRequest::ResolveHoufeng { .. }
        | EffectRequest::ResolveMbdiancai { .. }
        | EffectRequest::ResolveJianhui { .. }
        | EffectRequest::ResolveSpqiai { .. }
        | EffectRequest::ResolveSpshanxi { .. }
        | EffectRequest::ResolveSpshanxiSuoming { .. }
        | EffectRequest::ResolveDuanbi { .. }
        | EffectRequest::ResolveGuying { .. }
        | EffectRequest::ResolveMbzhonghui { .. }
        | EffectRequest::ResolvePotjiejie { .. }
        | EffectRequest::ResolvePotqingshi { .. }
        | EffectRequest::ResolvePotweiyan { .. }
        | EffectRequest::ResolvePotzhouyu { .. }
        | EffectRequest::ResolvePotlvyi { .. }
        | EffectRequest::ResolvePotcaoshuang { .. }
        | EffectRequest::ResolvePotyuji { .. }
        | EffectRequest::ResolveTwqiushou { .. }
        | EffectRequest::ResolveHongju { .. }
        | EffectRequest::ResolveQingjian { .. }
        | EffectRequest::ResolveHuangtian { .. }
        | EffectRequest::ResolveHaoshi { .. }
        | EffectRequest::ResolveDimeng { .. }
        | EffectRequest::ResolveQiaobian { .. }
        | EffectRequest::ResolveFangquanSkip { .. }
        | EffectRequest::ResolveFangquanGrant { .. }
        | EffectRequest::ResolveFuyin { .. }
        | EffectRequest::ResolveJuzhan { .. }
        | EffectRequest::ResolveQizhi { .. }
        | EffectRequest::ResolveJinqu { .. }
        | EffectRequest::ResolveJianxiang { .. }
        | EffectRequest::ResolveMingren { .. }
        | EffectRequest::ResolveKuizhu { .. }
        | EffectRequest::ResolveLijun { .. }
        | EffectRequest::ResolveLiangyin { .. }
        | EffectRequest::ResolveKongsheng { .. }
        | EffectRequest::ResolveDccunsi { .. } => {
            effects_choose_gain_discard_card_by_kind_resolvers::hash_effect(h, effect)
        }
        EffectRequest::ResolveYuli { .. }
        | EffectRequest::ResolveTingwei { .. }
        | EffectRequest::ResolveXunyi { .. }
        | EffectRequest::ResolveYiyong { .. }
        | EffectRequest::ResolveShanxie { .. }
        | EffectRequest::ResolveSheyi { .. }
        | EffectRequest::ResolveTianyin { .. }
        | EffectRequest::ResolveZhengsu { .. }
        | EffectRequest::ResolveRechuhai { .. }
        | EffectRequest::ResolveXiongshu { .. }
        | EffectRequest::ResolveTongduo { .. }
        | EffectRequest::ResolveYuanqing { .. }
        | EffectRequest::ResolveShuchen { .. }
        | EffectRequest::ResolveZaoli { .. }
        | EffectRequest::ResolveRehengjiang { .. }
        | EffectRequest::ResolveTuntian { .. }
        | EffectRequest::ResolveZaoxian { .. }
        | EffectRequest::ResolveTiaoxin { .. }
        | EffectRequest::ResolveZhiji { .. }
        | EffectRequest::ResolveZaiqi { .. }
        | EffectRequest::ResolveZhengrong { .. }
        | EffectRequest::ResolveReleiji { .. }
        | EffectRequest::ResolveXiangle { .. }
        | EffectRequest::ResolveRuoyu { .. }
        | EffectRequest::ResolveZuilun { .. }
        | EffectRequest::ResolveShenshi { .. }
        | EffectRequest::ResolveZhenliang { .. }
        | EffectRequest::ResolveZhizheng { .. } => {
            effects_choose_gain_discard_card_by_kind_resolvers_r_z::hash_effect(h, effect)
        }
        EffectRequest::ChooseDiscardHandEquipThenRecoverTarget { .. }
        | EffectRequest::DrawCardsFromBottom { .. }
        | EffectRequest::SetDrawPhaseFromBottom { .. }
        | EffectRequest::ChooseDiscardHandRange { .. }
        | EffectRequest::ContinueYoudishenru { .. }
        | EffectRequest::ContinueWdbFuchou { .. }
        | EffectRequest::ContinueDiaobingqianjiang { .. }
        | EffectRequest::PreventRecovery { .. }
        | EffectRequest::CommitRecovery { .. }
        | EffectRequest::GuozhanUseZhulianBihe { .. }
        | EffectRequest::CancelIncomingCardUse { .. }
        | EffectRequest::ChooseOtherGainOneOfTwoSkills { .. }
        | EffectRequest::ContinueWugu { .. }
        | EffectRequest::EquipWeaponFromDrawPile { .. }
        | EffectRequest::ReplayVirtualCardOnTargets { .. } => {
            effects_choose_discard_hand_equip_then_recover_target::hash_effect(h, effect)
        }
        EffectRequest::ResolveDcjue { .. }
        | EffectRequest::ResolveRuanji { .. }
        | EffectRequest::ResolveZhengbi { .. }
        | EffectRequest::ResolveFengying { .. }
        | EffectRequest::ResolveQiaojian { .. }
        | EffectRequest::ResolveHuzun { .. }
        | EffectRequest::ResolveZhugemengxue { .. }
        | EffectRequest::ResolveChenglve { .. }
        | EffectRequest::ResolveShicai { .. }
        | EffectRequest::ResolveBailingyun { .. }
        | EffectRequest::ResolveDcshaowei { .. }
        | EffectRequest::ResolveDcdichou { .. }
        | EffectRequest::ResolveDcdufeng { .. }
        | EffectRequest::ResolveDcctjiuxian { .. }
        | EffectRequest::ResolveBianyue { .. }
        | EffectRequest::ResolveZhupeilan { .. }
        | EffectRequest::ResolveDcSbZhangxiu { .. }
        | EffectRequest::ResolveDcSbZhouyu { .. }
        | EffectRequest::ResolveDcSbLusu { .. }
        | EffectRequest::ResolveZhangjian { .. }
        | EffectRequest::ResolveZhugeruoxue { .. }
        | EffectRequest::ResolveCaoyi { .. }
        | EffectRequest::ResolveMalingli { .. }
        | EffectRequest::ResolveNsshuaiyan { .. }
        | EffectRequest::ResolveShoulijian { .. }
        | EffectRequest::ResolveGwyewuChain { .. }
        | EffectRequest::ResolveXiongluan { .. }
        | EffectRequest::ResolveGzbuqu { .. }
        | EffectRequest::ResolveGzbuquRecover { .. }
        | EffectRequest::ResolveSbbiyue { .. }
        | EffectRequest::ResolveSxrmqiewang { .. }
        | EffectRequest::ResolveSxrmqianliu { .. }
        | EffectRequest::ResolveTwjuxiang { .. }
        | EffectRequest::ResolveXicha { .. }
        | EffectRequest::ResolvePottuntian { .. }
        | EffectRequest::ResolvePotjixi { .. }
        | EffectRequest::ResolvePotceduan { .. }
        | EffectRequest::ResolvePottaishici { .. }
        | EffectRequest::ResolveTwfupan { .. }
        | EffectRequest::ResolveTwjiaohua { .. }
        | EffectRequest::ResolveTwkujian { .. }
        | EffectRequest::ResolveTwruilian { .. }
        | EffectRequest::ResolveTwjianwei { .. }
        | EffectRequest::ResolveTwtanfeng { .. }
        | EffectRequest::ResolveTwzhibian { .. }
        | EffectRequest::ResolveTwyuyan { .. }
        | EffectRequest::ResolveTwguoyi { .. }
        | EffectRequest::ResolveTwchuhai { .. }
        | EffectRequest::ResolveTwyizhu { .. } => {
            effects_choose_discard_hand_equip_then_recover_target_resolvers::hash_effect(h, effect)
        }
        EffectRequest::ResolveWdbHanlong { .. }
        | EffectRequest::ResolveWdbLiufu { .. }
        | EffectRequest::ResolveWdbLiuyan { .. }
        | EffectRequest::ResolveWdbLiuzan { .. }
        | EffectRequest::ResolveWdbTianyu { .. }
        | EffectRequest::ResolveWdbXizhenxihong { .. }
        | EffectRequest::ResolveWdbYangyi { .. }
        | EffectRequest::ResolveWdbZaozhiRenjun { .. }
        | EffectRequest::ResolveDddXianxi { .. }
        | EffectRequest::ResolveDddXiaheng { .. }
        | EffectRequest::ResolveDddLiuchong { .. }
        | EffectRequest::ResolveDddDuanbing { .. }
        | EffectRequest::ResolveDddWuzhi { .. }
        | EffectRequest::ResolveDddXujing { .. }
        | EffectRequest::ResolveDddCaomao { .. }
        | EffectRequest::ResolveDddXinxianying { .. }
        | EffectRequest::ResolveDddXianglang { .. }
        | EffectRequest::ResolveDddYujin { .. }
        | EffectRequest::ResolveDddLiuye { .. }
        | EffectRequest::ResolveDddBaosanniang { .. }
        | EffectRequest::ResolveDddZhenji { .. }
        | EffectRequest::ResolveDddZhaoang { .. }
        | EffectRequest::ResolveDddLiuba { .. }
        | EffectRequest::ResolveDddJianshuo { .. }
        | EffectRequest::ResolveDddGuanning { .. }
        | EffectRequest::ResolveDddCaoshuang { .. }
        | EffectRequest::ResolveDddXuelingyun { .. }
        | EffectRequest::ResolveDddLiuhong { .. }
        | EffectRequest::ResolveDddXiahouxuan { .. }
        | EffectRequest::ResolveDddZhangkai { .. }
        | EffectRequest::ResolveDddLiangxi { .. } => {
            effects_choose_discard_hand_equip_then_recover_target_resolvers_ddd_wdb::hash_effect(h, effect)
        }
        EffectRequest::ResolveTwluanchou { .. }
        | EffectRequest::ResolveTwxiawei { .. }
        | EffectRequest::ResolveTwbianfuren { .. }
        | EffectRequest::ResolveTwfeiyi { .. }
        | EffectRequest::ResolveTwchenzhen { .. }
        | EffectRequest::ResolveTwqianju { .. }
        | EffectRequest::ResolveTwqingxi { .. }
        | EffectRequest::ResolveTwzaoli { .. }
        | EffectRequest::ResolveTwzhengjian { .. }
        | EffectRequest::ResolveTwtianyu { .. }
        | EffectRequest::ResolveTwwujing { .. }
        | EffectRequest::ResolveTwshanghe { .. }
        | EffectRequest::ResolveTwwangchang { .. }
        | EffectRequest::ResolveTwcaozhao { .. }
        | EffectRequest::ResolveTwguohuai { .. }
        | EffectRequest::ResolveTwchengpu { .. }
        | EffectRequest::ResolveTwcaoang { .. }
        | EffectRequest::ResolveTwniujin { .. }
        | EffectRequest::ResolveMbqingdao { .. }
        | EffectRequest::ResolveMbxiugeng { .. }
        | EffectRequest::ResolveMbchenshe { .. }
        | EffectRequest::ResolveMbquanchong { .. }
        | EffectRequest::ResolveMbrenxing { .. }
        | EffectRequest::ResolveMbchizhang { .. }
        | EffectRequest::ResolveMbduanyang { .. }
        | EffectRequest::ResolveMbrunwei { .. }
        | EffectRequest::ResolveMbshuanghuai { .. }
        | EffectRequest::ResolveMbxiongtu { .. }
        | EffectRequest::ResolveMbganggeng { .. }
        | EffectRequest::ResolveMbsijian { .. }
        | EffectRequest::ResolveMbjianji { .. }
        | EffectRequest::ResolveMbyuanmo { .. }
        | EffectRequest::ResolveMbfeijing { .. }
        | EffectRequest::ResolveMbxiaoge { .. }
        | EffectRequest::ResolveMbxuye { .. }
        | EffectRequest::ResolveMbkuangxiang { .. }
        | EffectRequest::ResolveMbzhuji { .. } => {
            effects_resolve_twluanchou_resolvers_tw_mb::hash_effect(h, effect)
        }
        EffectRequest::ResolveTianxiang { .. }
        | EffectRequest::ResolveRetianxiang { .. }
        | EffectRequest::ResolveLianzhou { .. }
        | EffectRequest::ResolveStddechong { .. }
        | EffectRequest::ResolveJieliangGain { .. }
        | EffectRequest::ResolveStdjujian { .. }
        | EffectRequest::ResolveLulve { .. }
        | EffectRequest::ResolveDctingxian { .. }
        | EffectRequest::ResolveHfjieying { .. }
        | EffectRequest::ResolveDcnuchen { .. }
        | EffectRequest::ResolveDcdanji { .. }
        | EffectRequest::ResolveDczhiheng { .. }
        | EffectRequest::ResolveStdgaobian { .. }
        | EffectRequest::ResolveSpjiedao { .. }
        | EffectRequest::ResolveSibian { .. }
        | EffectRequest::ResolveSpfengyin { .. }
        | EffectRequest::ResolveXinfuLianpian { .. }
        | EffectRequest::ResolveGongjian { .. }
        | EffectRequest::ResolveRefuhai { .. }
        | EffectRequest::ResolveGzjinfa { .. }
        | EffectRequest::ResolveWnQifeng { .. }
        | EffectRequest::ResolveXinkuangfu { .. }
        | EffectRequest::ResolveDcxiangshu { .. }
        | EffectRequest::ResolveDcxiangshuLottery { .. }
        | EffectRequest::ResolveDcyingtu { .. }
        | EffectRequest::ResolveMobiledanshou { .. }
        | EffectRequest::ResolveJsrgchiying { .. }
        | EffectRequest::ResolveShuangxiong { .. }
        | EffectRequest::ResolveZhijian { .. }
        | EffectRequest::ResolveGuzheng { .. }
        | EffectRequest::ResolveZhiba { .. }
        | EffectRequest::ResolveZhibaSettle { .. }
        | EffectRequest::ResolveDrltCongjian { .. }
        | EffectRequest::ResolveNzryFeijun { .. }
        | EffectRequest::ResolveDrltYongsi { .. }
        | EffectRequest::ResolveDrltWeidi { .. }
        | EffectRequest::ResolveLieren { .. }
        | EffectRequest::ResolvePotchengpu { .. }
        | EffectRequest::ResolvePotjugu { .. }
        | EffectRequest::ResolvePotsifeng { .. }
        | EffectRequest::ResolvePotchenqun { .. }
        | EffectRequest::ResolvePotwanglie { .. }
        | EffectRequest::ResolvePothongyi { .. }
        | EffectRequest::ResolvePotguansha { .. }
        | EffectRequest::ResolvePotjiyu { .. }
        | EffectRequest::ResolvePotlibing { .. }
        | EffectRequest::ResolvePotpoxi { .. }
        | EffectRequest::ResolvePotfuan { .. }
        | EffectRequest::ResolvePotyinxian { .. }
        | EffectRequest::ResolvePothuanjie { .. }
        | EffectRequest::ResolvePotnigu { .. } => {
            effects_resolve_twluanchou_resolvers::hash_effect(h, effect)
        }
        EffectRequest::DiscardHandEquipCount { .. }
        | EffectRequest::SkipNextDiscardPhase { .. }
        | EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss { .. }
        | EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp { .. }
        | EffectRequest::ChooseGainTargetHandCeilHalfDifference { .. }
        | EffectRequest::GiveSelectedThenTargetMayUseDrawOwner { .. }
        | EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne { .. }
        | EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip { .. }
        | EffectRequest::EmitCardResolved { .. } => {
            effects_resolve_twluanchou::hash_effect(h, effect)
        }
        EffectRequest::ResolvePotlulian { .. }
        | EffectRequest::ResolvePotdimeng { .. }
        | EffectRequest::ResolvePothaoshi { .. }
        | EffectRequest::ResolveSpmiaolve { .. }
        | EffectRequest::ResolveSpyingjia { .. }
        | EffectRequest::ResolveMantianguohai { .. }
        | EffectRequest::ResolveXinfuQinguo { .. }
        | EffectRequest::ResolveDcmanzhi { .. }
        | EffectRequest::ResolveXinxingluan { .. }
        | EffectRequest::ResolveDckuizhen { .. }
        | EffectRequest::ResolveDckuizhenResolve { .. }
        | EffectRequest::ResolveNsdingzhou { .. }
        | EffectRequest::ResolveSxrmlanjiao { .. }
        | EffectRequest::ResolveRekuanshi { .. }
        | EffectRequest::ResolveRekuanshiResolve { .. }
        | EffectRequest::ResolveDcmoyu { .. }
        | EffectRequest::ResolveXizhen { .. }
        | EffectRequest::ResolveXuxie { .. }
        | EffectRequest::ResolveDcchiying { .. }
        | EffectRequest::ResolveXuxieAdd { .. }
        | EffectRequest::ResolveTwmoukui { .. }
        | EffectRequest::ResolveTwfuman { .. }
        | EffectRequest::ResolveCuijin { .. }
        | EffectRequest::ResolveTwqingkou { .. }
        | EffectRequest::ResolveTwchenxun { .. }
        | EffectRequest::ResolveTwfenxian { .. }
        | EffectRequest::ResolveQiluan { .. }
        | EffectRequest::ResolveOllinjie { .. }
        | EffectRequest::ResolveQianlong { .. }
        | EffectRequest::ResolveJsrgzuozhan { .. }
        | EffectRequest::ResolveJsrgzuozhanGain { .. }
        | EffectRequest::ResolveTwmouzhi { .. }
        | EffectRequest::ResolveTwdaigui { .. }
        | EffectRequest::ResolveTwmitu { .. }
        | EffectRequest::ResolveQiantun { .. }
        | EffectRequest::ResolveYangming { .. }
        | EffectRequest::ResolveCanyun { .. }
        | EffectRequest::ResolveHengwu { .. }
        | EffectRequest::ResolveBiaozhao3 { .. }
        | EffectRequest::ResolveChuanyun { .. }
        | EffectRequest::ResolveBiaozhao2 { .. }
        | EffectRequest::ResolveDcniji { .. }
        | EffectRequest::ResolveDcmoukui { .. }
        | EffectRequest::ResolveZhongyan { .. }
        | EffectRequest::ResolveChumei { .. }
        | EffectRequest::ResolveShuaiyan { .. }
        | EffectRequest::ResolveTwxiongjun { .. }
        | EffectRequest::ResolveYixiang { .. }
        | EffectRequest::ResolveXinfuQiai { .. }
        | EffectRequest::ResolveSpolaige { .. }
        | EffectRequest::ResolveOlzhijie { .. }
        | EffectRequest::ResolveOlsuyi { .. }
        | EffectRequest::ResolveOlqiaozhi { .. }
        | EffectRequest::ResolveDczhee { .. }
        | EffectRequest::ResolveDchaoxian { .. }
        | EffectRequest::ResolveZhongjie { .. }
        | EffectRequest::ResolveStarcuxiaWeiming { .. }
        | EffectRequest::ResolveSanshou { .. }
        | EffectRequest::ResolveNhlundao { .. }
        | EffectRequest::ResolveXiangshu { .. }
        | EffectRequest::ResolveOlwenren { .. }
        | EffectRequest::ResolveDcqiaolve { .. }
        | EffectRequest::ResolveDingce { .. }
        | EffectRequest::ResolveQiangyong { .. }
        | EffectRequest::ResolveMeiyong { .. }
        | EffectRequest::ResolveMbzhenann { .. }
        | EffectRequest::ResolveFriendbiwei { .. }
        | EffectRequest::ResolveChulao { .. }
        | EffectRequest::ResolveDcanxu { .. }
        | EffectRequest::ResolveDuoduan { .. }
        | EffectRequest::ResolveGaoyuan { .. }
        | EffectRequest::ResolveMbbeiming { .. }
        | EffectRequest::ResolveZhengjian { .. }
        | EffectRequest::ResolveReliangyin { .. }
        | EffectRequest::ResolveQinmian { .. }
        | EffectRequest::ResolveJsrgdaoren { .. }
        | EffectRequest::ResolveDuwang { .. }
        | EffectRequest::ResolveQuanjia { .. }
        | EffectRequest::ResolvePxqixian { .. }
        | EffectRequest::ResolveDcbingji { .. }
        | EffectRequest::ResolveTurnEndDebates { .. }
        | EffectRequest::ResolvePotheyun { .. }
        | EffectRequest::ResolveTwjieqiu { .. }
        | EffectRequest::ResolveJsrgchaozheng { .. }
        | EffectRequest::ResolveJsrgsheju { .. } => {
            effects_resolve_potlulian_resolvers::hash_effect(h, effect)
        }
        EffectRequest::SetSkillContentFlag { .. }
        | EffectRequest::CommitDamageBanActiveIfDying { .. }
        | EffectRequest::ChooseTargetReclaimEquipmentDrawIfRangeReduced { .. }
        | EffectRequest::CommitTargetReclaimEquipmentDrawIfRangeReduced { .. }
        | EffectRequest::FinishTargetReclaimEquipmentDrawIfRangeReduced { .. }
        | EffectRequest::ChooseUseVirtualCard { .. }
        | EffectRequest::ChooseUseVirtualNanman { .. }
        | EffectRequest::DiscardAllOwnJudge { .. }
        | EffectRequest::StoreEventCardInSkillPile { .. }
        | EffectRequest::ChooseTargetGainMaxHp { .. }
        | EffectRequest::GainTargetHandCard { .. }
        | EffectRequest::ChooseClanNotMaxHandLimitGain { .. }
        | EffectRequest::GainWholeSkillPile { .. }
        | EffectRequest::ChooseOneFromSkillPileDiscard { .. }
        | EffectRequest::PlaceHandCardsToDeckTop { .. }
        | EffectRequest::RearrangePileToDeckTop { .. }
        | EffectRequest::CommitGaoyuanRedirect { .. }
        | EffectRequest::ChooseHandCardsToSkillPile { .. }
        | EffectRequest::ChooseTargetHandCardToPile { .. }
        | EffectRequest::PlaceTopDeckToExpansion { .. }
        | EffectRequest::DiscardSkillPile { .. }
        | EffectRequest::GainFromSkillPile { .. }
        | EffectRequest::DiscardBasicDrawEqual { .. }
        | EffectRequest::StartDebateAllOthers { .. }
        | EffectRequest::StartDebateTargets { .. }
        | EffectRequest::PinDebateOpinions { .. }
        | EffectRequest::AlignDebateOpinions { .. }
        | EffectRequest::GainShadowCards { .. }
        | EffectRequest::ChangeGroup { .. }
        | EffectRequest::DistributeShadowCards { .. }
        | EffectRequest::RegisterTurnEndDebate { .. }
        | EffectRequest::ChooseGroup { .. }
        | EffectRequest::ChooseGainOtherSkill { .. }
        | EffectRequest::LoseGainedSkill { .. }
        | EffectRequest::PlaceXumou { .. }
        | EffectRequest::PlaceDeckTopXumou { .. }
        | EffectRequest::ChoosePlaceHandXumou { .. }
        | EffectRequest::DiscardOneOwnXumou { .. }
        | EffectRequest::PlaceDeckTopExpansion { .. }
        | EffectRequest::CityConsume { .. }
        | EffectRequest::QueueExtraTurn { .. }
        | EffectRequest::ChooseOthersInsertExtraTurn { .. }
        | EffectRequest::QueueExtraPhases { .. }
        | EffectRequest::SkipRemainingPhases { .. }
        | EffectRequest::ChooseOthersAddMark { .. }
        | EffectRequest::ChooseOtherAddMark { .. }
        | EffectRequest::PlaceHandCardToExpansion { .. }
        | EffectRequest::ExtraResolveCurrentTrick { .. }
        | EffectRequest::FinishPhaseEndVirtualCard
        | EffectRequest::IncreaseCurrentShaBaseDamage { .. }
        | EffectRequest::SkipNextPlayPhase { .. }
        | EffectRequest::ChooseOtherDrawOrRecover { .. }
        | EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill { .. }
        | EffectRequest::DrawSelectedTargets { .. }
        | EffectRequest::RecoverIfMinCurrentHp { .. }
        | EffectRequest::IncreaseCurrentCardBaseDamage { .. }
        | EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying { .. }
        | EffectRequest::CommitLoseHpThenDamageOtherLoseMaxHpIfDying { .. } => {
            effects_resolve_potlulian::hash_effect(h, effect)
        }
        EffectRequest::RecoverSelectedTargets { .. }
        | EffectRequest::RecordSelectedPlayPhaseCategory { .. }
        | EffectRequest::TakeTargetHandThenRepeatUntilMax { .. }
        | EffectRequest::ContinueTakeLargerHandUntilMax { .. }
        | EffectRequest::DrawThenChooseGiveOrDiscardHand { .. }
        | EffectRequest::GainTargetEquipment { .. }
        | EffectRequest::DiscardAllTargetEquipment { .. }
        | EffectRequest::IncreaseCurrentShaTargetDamage { .. }
        | EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp { .. }
        | EffectRequest::GainEquipmentFromDrawPile { .. }
        | EffectRequest::GiveAllHandRecoverToOne { .. }
        | EffectRequest::ReplaceSkillWithBoundAfterResolutionIfTargetAlive { .. }
        | EffectRequest::DiscardTargetEquipmentDrawWeaponRange { .. }
        | EffectRequest::RecoverToMaxThenOptionalExchangeSkill { .. }
        | EffectRequest::RevealHandDiscardKind { .. }
        | EffectRequest::FinishOpeningExtraPlayShaGain { .. }
        | EffectRequest::RevealTargetHandGainShanArmorOrDrawTwo { .. }
        | EffectRequest::GiveAllHand { .. }
        | EffectRequest::GainCardIfDiscard { .. }
        | EffectRequest::RevealHandToPlayer { .. }
        | EffectRequest::RevealHandChooseDiscardColor { .. }
        | EffectRequest::ContinueChooseCompareTargetHandCardSecondFailureSeatDamage { .. }
        | EffectRequest::RecordCompareFailureSeatDamage { .. }
        | EffectRequest::DealDamageEachInOrder { .. }
        | EffectRequest::RemoveAllSkills { .. }
        | EffectRequest::EnableEquipSlot { .. }
        | EffectRequest::TargetChooseSuitGainOwnerHandDamageIfMismatch { .. }
        | EffectRequest::JudgeSourceDiscardTwoHandOrDamage { .. }
        | EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo { .. }
        | EffectRequest::DistributeTopTwoCards { .. }
        | EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed { .. }
        | EffectRequest::JudgeBlackGainRepeat { .. }
        | EffectRequest::UseVirtualJuedouSecondOnFirst { .. }
        | EffectRequest::JudgeSetSkillMarkToRank { .. }
        | EffectRequest::DisableSkillByDefUntilTurnEnd { .. }
        | EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner { .. }
        | EffectRequest::RevealTargetHandShaOrDiscardBind { .. }
        | EffectRequest::DrawRememberedHandCards { .. }
        | EffectRequest::DiscardRememberedHandCards { .. }
        | EffectRequest::GainDrawCardsForLifetimeCardActionDivisors { .. }
        | EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay { .. }
        | EffectRequest::DrawThenOptionallyDistributeHandCards { .. }
        | EffectRequest::GiveSelectedThenContinueDistinctRecoverAtTwo { .. }
        | EffectRequest::AddSkillUntilTurnEnd { .. }
        | EffectRequest::RecordActiveTarget { .. }
        | EffectRequest::ReplaceDrawWithDistinctSuitSelection { .. }
        | EffectRequest::InspectTargetHandChooseHeartDiscardOrTop { .. }
        | EffectRequest::AdvanceTurn
        | EffectRequest::JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice { .. }
        | EffectRequest::DamageThenDiscardAllJudge { .. }
        | EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors { .. }
        | EffectRequest::SkipNextJudgePhase { .. }
        | EffectRequest::SkipJudgeAndDrawPhases { .. }
        | EffectRequest::TakeOneHandCard { .. }
        | EffectRequest::GainAllSkillPileCards { .. }
        | EffectRequest::GainOtherSkillPileCards { .. }
        | EffectRequest::LoseHpDoubleCurrentDamageThenReplace { .. }
        | EffectRequest::SetNonLockedSkillSuppression { .. } => {
            effects_recover_selected_targets::hash_effect(h, effect)
        }
        EffectRequest::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark { .. }
        | EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp { .. }
        | EffectRequest::ChooseBoundSourceAttackRangeDamage { .. }
        | EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess { .. }
        | EffectRequest::ChooseTargetDrawMaxHpCapThenAddSkill { .. }
        | EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp { .. }
        | EffectRequest::ChooseGiveOneHandEquipOrRecover { .. }
        | EffectRequest::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp { .. }
        | EffectRequest::ChooseDiscardHandEquipThenRecover { .. }
        | EffectRequest::ChooseTargetsDamage { .. }
        | EffectRequest::ChooseTargetRevealHandDamageIfSha { .. }
        | EffectRequest::ChooseOtherGiveOneHandEquip { .. }
        | EffectRequest::ChooseGiveOrRecastOneHand { .. }
        | EffectRequest::ChooseRevealHandCardBindCategoryTarget { .. }
        | EffectRequest::ChooseMoveTargetEquipmentAndOrDraw { .. }
        | EffectRequest::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth { .. }
        | EffectRequest::ChooseDrawOrRecastAddSkill { .. }
        | EffectRequest::ChooseGainShaForOpeningExtraPlay { .. }
        | EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha { .. }
        | EffectRequest::ChooseOptionalDiscardTargetCard { .. }
        | EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory { .. }
        | EffectRequest::ChooseScskuijiDiscard { .. }
        | EffectRequest::ChoosePskuijiDiscard { .. }
        | EffectRequest::ChooseCompareSelectedHandCardSecondFailureSeatDamage { .. }
        | EffectRequest::ChooseDrawOrDiscardTargetCard { .. }
        | EffectRequest::ChooseSeatPathDamage { .. }
        | EffectRequest::ChooseOtherRecover { .. }
        | EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw { .. }
        | EffectRequest::ChooseDiscardBasicThenTargetDiscardEquipOrDamage { .. }
        | EffectRequest::ChooseCurrentShaTargetDiscardRedirectToOwner { .. }
        | EffectRequest::ChooseTargetGainBottomCard { .. }
        | EffectRequest::ChooseDiscardRedirectCurrentSha { .. }
        | EffectRequest::ChooseDiscardOneFromEachLinked { .. }
        | EffectRequest::ChooseTargetLoseHpDiscardHandShaIfBasic { .. }
        | EffectRequest::ChooseOptionalTargetsDraw { .. }
        | EffectRequest::ChooseGainOwnLostDiscardCardThisTurn { .. }
        | EffectRequest::ChooseMoveFieldCardTo { .. }
        | EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan { .. }
        | EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver { .. }
        | EffectRequest::ChooseGainCardFromEachOtherThenTurnOver { .. }
        | EffectRequest::ChooseAllLoseHpOrRecover { .. }
        | EffectRequest::ChooseLinkOther { .. }
        | EffectRequest::ChooseDiscardWeaponOrLoseHp { .. }
        | EffectRequest::ChooseDiscardOneHeOrLoseHp { .. }
        | EffectRequest::ChooseSourceConfirmOwnerLoseHp { .. }
        | EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw { .. }
        | EffectRequest::ChooseDiscardDamageEndpointCardIncreaseIfSuit { .. }
        | EffectRequest::ChooseOtherHandStoreSkillPile { .. }
        | EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits { .. } => {
            effects_recover_selected_targets_choose::hash_effect(h, effect)
        }
        EffectRequest::ResolveMassDamageDiscardTurnOver { .. }
        | EffectRequest::ResolveMassDamageDiscardHandEquip { .. }
        | EffectRequest::ResolveMarkParityControl { .. }
        | EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage { .. }
        | EffectRequest::ResolveYeyan { .. }
        | EffectRequest::ResolveQixing { .. }
        | EffectRequest::ResolveQixingWeather { .. }
        | EffectRequest::ResolveNzryLongnu { .. }
        | EffectRequest::ResolveDrltPoxi { .. }
        | EffectRequest::ResolveDrltJieying { .. }
        | EffectRequest::ResolveDrltDuorui { .. }
        | EffectRequest::ResolveDrltZhiti { .. }
        | EffectRequest::ResolveShenSimayi { .. }
        | EffectRequest::ResolveBeige { .. }
        | EffectRequest::ResolveHiddenSkillPile { .. } => {
            effects_recover_selected_targets_resolvers::hash_effect(h, effect)
        }
        EffectRequest::ResolveDiliYuheng { .. }
        | EffectRequest::ResolveGuanchaoRank { .. }
        | EffectRequest::ResolveGuanwei { .. }
        | EffectRequest::ResolveLanjiang { .. }
        | EffectRequest::ResolveRebingyi { .. }
        | EffectRequest::ResolveRebotu { .. }
        | EffectRequest::ResolveOldimeng { .. }
        | EffectRequest::ResolveRejieyin { .. }
        | EffectRequest::ResolveReanxu { .. }
        | EffectRequest::ResolveDrltJueyan { .. }
        | EffectRequest::ResolveRejizhi { .. }
        | EffectRequest::ResolveJiexun { .. }
        | EffectRequest::ResolveWenguaSetup { .. }
        | EffectRequest::ResolveWengua { .. }
        | EffectRequest::ResolveXinanguo { .. }
        | EffectRequest::ResolveGanglieThree { .. }
        | EffectRequest::ResolveZhongyi { .. }
        | EffectRequest::ResolveOldAnxu { .. }
        | EffectRequest::ResolveOldQinqing { .. }
        | EffectRequest::ResolveOldMingjian { .. }
        | EffectRequest::ResolveOldXuanfeng { .. }
        | EffectRequest::ResolveJianrou { .. }
        | EffectRequest::ResolveOldHuisheng { .. }
        | EffectRequest::ResolveOldJiefan { .. }
        | EffectRequest::ResolveChegu { .. }
        | EffectRequest::ResolveJunkguixin { .. }
        | EffectRequest::ResolveYaoChenwei { .. }
        | EffectRequest::ResolveYoudi { .. }
        | EffectRequest::ResolveKuangfu { .. }
        | EffectRequest::ResolveShanjia { .. }
        | EffectRequest::ResolveNewYajiao { .. }
        | EffectRequest::ResolveDingpin { .. }
        | EffectRequest::ResolveXinfuGuolun { .. }
        | EffectRequest::ResolveBeigeConfigured { .. }
        | EffectRequest::ResolveZhuiyi { .. }
        | EffectRequest::ResolveDecadezhenjun { .. }
        | EffectRequest::ResolveSpXiaoguo { .. }
        | EffectRequest::ResolveDingpan { .. }
        | EffectRequest::ResolveTanlu { .. }
        | EffectRequest::ResolveDcqiaomeng { .. }
        | EffectRequest::ResolveNewReluoyi { .. }
        | EffectRequest::ResolveRemoshi { .. }
        | EffectRequest::ResolveXinyaoming { .. }
        | EffectRequest::ResolveRehuaiyi { .. }
        | EffectRequest::ResolveReanguo { .. }
        | EffectRequest::ResolveBeizhu { .. }
        | EffectRequest::ResolveMbshanjia { .. }
        | EffectRequest::ResolveDaoji { .. }
        | EffectRequest::ResolveJiaohua { .. }
        | EffectRequest::ResolveLuanqun { .. }
        | EffectRequest::ResolveMbzhuguo { .. }
        | EffectRequest::ResolveMbanxian { .. }
        | EffectRequest::ResolveMbjinzu { .. }
        | EffectRequest::ResolveMuzhen { .. } => {
            effects_resolve_dili_yuheng_resolvers::hash_effect(h, effect)
        }
        EffectRequest::FinalizeTiaojiyanmei { .. }
        | EffectRequest::BeginTiaojiyanmeiDiscard { .. }
        | EffectRequest::DiscardSelectedDrawAllHandBonus { .. }
        | EffectRequest::ChooseOtherYinghun { .. }
        | EffectRequest::ChooseOtherBizheng { .. }
        | EffectRequest::ChooseOtherXiashu { .. }
        | EffectRequest::DiscardEquipmentSlot { .. }
        | EffectRequest::StoreSelectedInSkillPile { .. }
        | EffectRequest::EndMatchForCampIfSkillPileAtLeast { .. }
        | EffectRequest::ChooseAddCurrentShaTargetWithinDistance { .. }
        | EffectRequest::RefundCurrentShaUse { .. }
        | EffectRequest::ResetLinkedSelectedTargets { .. }
        | EffectRequest::DamageThenDrawSelectedTargets { .. }
        | EffectRequest::FinishCurrentPhaseAbortResolution { .. }
        | EffectRequest::GainTopCardToSkillPile { .. }
        | EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill { .. }
        | EffectRequest::GainOneHandFromSelectedTargets { .. }
        | EffectRequest::ChooseRepoluTargetsDraw { .. }
        | EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank { .. }
        | EffectRequest::ChooseGainCardFromEachOtherReplaceDraw { .. }
        | EffectRequest::GiveSelectedThenRecoverTarget { .. }
        | EffectRequest::JudgeSameColorGainRepeat { .. }
        | EffectRequest::ChooseCurrentTrickTargetsToNullify { .. }
        | EffectRequest::JudgeBlackGainRepeatProtectHandLimit { .. }
        | EffectRequest::ChooseGainAllResolvedShaShanThenColorRewards { .. }
        | EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw { .. }
        | EffectRequest::ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus { .. }
        | EffectRequest::ChooseTargetDiscardDistanceHandOrIncreaseDamage { .. }
        | EffectRequest::ChooseDrawViewTwoGainOrMoveEquipment { .. }
        | EffectRequest::ChooseGiveTwoDifferentColorsOrRecover { .. }
        | EffectRequest::TakeOneScopedCard { .. }
        | EffectRequest::RevealHand { .. }
        | EffectRequest::DrawLinkedPlayers { .. }
        | EffectRequest::RevealTopGainIfColor { .. }
        | EffectRequest::RefreshSkill { .. }
        | EffectRequest::TagCard { .. }
        | EffectRequest::UntagCard { .. }
        | EffectRequest::ClanshenjunReveal { .. }
        | EffectRequest::ClanshenjunUse { .. }
        | EffectRequest::StorageSet { .. }
        | EffectRequest::DrawAllEventTargets { .. }
        | EffectRequest::RoundEnded { .. }
        | EffectRequest::RoundStarted { .. }
        | EffectRequest::ClanyuzhiReveal { .. }
        | EffectRequest::ClanyuzhiRoundEnd { .. }
        | EffectRequest::QieyiPhaseStart { .. }
        | EffectRequest::QieyiUseAfter { .. }
        | EffectRequest::JianzhiPhaseEnd { .. } => {
            effects_resolve_dili_yuheng::hash_effect(h, effect)
        }
    }
}
