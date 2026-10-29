//! ResolveChoice and continuation resume.
//!
//! `handle_resolve_choice` validates and clears the pending choice, then routes
//! its continuation to a functional dispatch module. Skill-specific handlers live
//! beneath the existing handler groups; guarded continuation order is preserved.

use crate::effect::{open_choice, run_effects, EffectError, RESOLVE_DRLT_WEIDI_CUSTOM_CHOICE};
use crate::state::{CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use crate::transaction::Transaction;
use engine_api::{
    CardId, CardMoveReason, ChoiceId, ChoiceKind, PlayerId, SkillDefId, SkillId, Suit,
};
use rule_api::{DrawTargetScope, EffectRequest};

mod group_post_1;
mod group_post_2;
mod group_post_3;
mod group_post_4;
mod group_post_5;
mod group_post_6;
mod group_post_7;
mod group_post_8;
mod group_post_9;
mod group_pre_1;
mod group_pre_2;
mod group_pre_3;

pub(crate) use group_post_1::*;
pub(crate) use group_post_2::*;
pub(crate) use group_post_3::*;
pub(crate) use group_post_4::*;
pub(crate) use group_post_5::*;
pub(crate) use group_post_6::*;
pub(crate) use group_post_7::*;
pub(crate) use group_post_8::*;
pub(crate) use group_post_9::*;
pub(crate) use group_pre_1::*;
pub(crate) use group_pre_2::*;
pub(crate) use group_pre_3::*;

pub(super) fn handle_resolve_choice(
    tx: &mut Transaction,
    actor: PlayerId,
    choice_id: ChoiceId,
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let pending = tx.state().resolution.pending_choice_id;
    if pending == 0 {
        return Err(EffectError::Illegal("invalid choice: none pending".into()));
    }
    if choice_id.raw() != pending {
        return Err(EffectError::Illegal("invalid choice: id mismatch".into()));
    }

    let observer = PlayerId::new(tx.state().resolution.pending_observer);
    if actor.is_some() && actor != observer {
        return Err(EffectError::Illegal("invalid choice: wrong actor".into()));
    }

    let kind = tx.state().resolution.continuation_kind;
    let payload = tx.state().resolution.continuation_payload.clone();

    let optional_trigger = tx.state().resolution.optional_trigger.clone();
    // Capture VM continuation before clearing (used by VmResume arm).
    let vm_cont = {
        let r = &tx.state().resolution;
        (
            r.vm_cont_frames.clone(),
            r.vm_cont_module.clone(),
            r.vm_cont_pc,
            r.vm_cont_fuel,
            r.vm_cont_ctx,
            r.vm_cont_skill,
            r.vm_cont_trigger_marker,
        )
    };

    {
        let r = &mut tx.state_mut().resolution;
        r.clear_choice();
    }

    match kind {
        ContinuationKind::Custom | ContinuationKind::None => choices_custom::resolve_choice(
            tx,
            actor,
            choice_id,
            observer,
            kind,
            payload,
            selections,
            cancelled,
            vm_cont,
            optional_trigger,
        ),
        ContinuationKind::DyingSave
        | ContinuationKind::DiscardToLimit
        | ContinuationKind::ReorderDrawPile
        | ContinuationKind::ReorderDrawPileFourSplitTwoThenDraw
        | ContinuationKind::DrawReplacement
        | ContinuationKind::OptionalDrawBonus
        | ContinuationKind::PickTargetCard
        | ContinuationKind::HuogongShow
        | ContinuationKind::HuogongDiscard
        | ContinuationKind::JuedouDuel
        | ContinuationKind::JiedaoChoice
        | ContinuationKind::WuxieWindow
        | ContinuationKind::ActiveSkillSelection => choices_dying_save::resolve_choice(
            tx,
            actor,
            observer,
            kind,
            payload,
            selections,
            cancelled,
            vm_cont,
            optional_trigger,
        ),
        ContinuationKind::RespondToSha
        | ContinuationKind::RespondAoeQueue
        | ContinuationKind::RewriteJudgment
        | ContinuationKind::RewriteBeigeJudgment => choices_dying_save_respond_rewrite::resolve_choice(
            tx,
            actor,
            observer,
            kind,
            payload,
            selections,
            cancelled,
            vm_cont,
            optional_trigger,
        ),
        ContinuationKind::ChooseDrawOrRecover
        | ContinuationKind::ChooseDingpanBranch
        | ContinuationKind::ChooseTargetLoseHpDiscardHandShaIfBasic
        | ContinuationKind::ChooseTargetRevealHandDamageIfSha
        | ContinuationKind::ChooseOtherGiveOneHandEquip
        | ContinuationKind::ChooseGiveOrRecastOneHand
        | ContinuationKind::ChooseOtherGiveThenRecover
        | ContinuationKind::ChooseDiscardTargetCard
        | ContinuationKind::ChooseDiscardTargetCardResolveCurrentSha
        | ContinuationKind::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable
        | ContinuationKind::ChooseTargetDrawChoice
        | ContinuationKind::ChooseTargetReclaimEquipmentDrawIfRangeReduced
        | ContinuationKind::ChooseUseVirtualCard
        | ContinuationKind::ChooseDiscardOneThenGainTargetEquipment
        | ContinuationKind::ChooseTargetDraw
        | ContinuationKind::ChooseTargetsDamage
        | ContinuationKind::ChooseTargetDrawThenOwnerIfBelowMaxHp
        | ContinuationKind::ChooseTargetDrawMaxHpCapThenDiscardExcess
        | ContinuationKind::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp
        | ContinuationKind::ChooseGiveOneHandEquipOrRecover
        | ContinuationKind::ChooseGiveTwoDifferentColorsOrRecover
        | ContinuationKind::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp
        | ContinuationKind::ChooseOtherDrawOwnerLostHpTurnOver
        | ContinuationKind::ChooseDiscardOneForShaRefundAndDrawIfRed
        | ContinuationKind::ChooseTargetGainMaxHp
        | ContinuationKind::ChooseOtherAddMark
        | ContinuationKind::ChooseGroup
        | ContinuationKind::ChooseGainOtherSkill
        | ContinuationKind::ChoosePlaceHandXumou
        | ContinuationKind::ChooseOthersAddMark
        | ContinuationKind::ChooseOthersInsertExtraTurn
        | ContinuationKind::ChooseClanNotMaxHandLimitGain
        | ContinuationKind::ChooseAddCurrentShaTarget
        | ContinuationKind::ChooseStoreHandShaInSkillPile
        | ContinuationKind::ChooseSpendSkillPileForDyingJiu
        | ContinuationKind::ChooseCompareTargetHandCard
        | ContinuationKind::ChooseHandCompareTarget
        | ContinuationKind::ChooseHandComparePlayerCard
        | ContinuationKind::ChooseOtherDamage
        | ContinuationKind::ChooseBoundSourceAttackRangeDamage
        | ContinuationKind::ChooseOtherDiscardTargetCard
        | ContinuationKind::ChooseGainDiscardCardByKind
        | ContinuationKind::ChooseGainOwnLostDiscardCardThisTurn
        | ContinuationKind::ChooseMoveFieldCardTo
        | ContinuationKind::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan
        | ContinuationKind::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver
        | ContinuationKind::ChooseGainCardFromEachOtherThenTurnOver
        | ContinuationKind::ChooseHiddenSkillPileHandCards
        | ContinuationKind::ChooseHiddenSkillPileCards
        | ContinuationKind::ChooseChangandajianTarget
        | ContinuationKind::ChooseChangandajianCard
        | ContinuationKind::ChooseTiaojiyanmeiGainTarget
        | ContinuationKind::ChooseOtherYinghun
        | ContinuationKind::ChooseOtherBizheng
        | ContinuationKind::ChooseOtherXiashu
        | ContinuationKind::ChooseDiscardSkillPileCardAddMarkedSkill
        | ContinuationKind::ChooseRepoluTargetsDraw
        | ContinuationKind::ChooseMarkParityControlTarget
        | ContinuationKind::ChooseSelectedTargetDamage
        | ContinuationKind::ChooseDistinctSuitCards
        | ContinuationKind::ChooseInspectedHeartCard
        | ContinuationKind::ChooseInspectedHeartDestination
        | ContinuationKind::ChooseYeyanMode
        | ContinuationKind::ChooseYeyanBigCards
        | ContinuationKind::ChooseYeyanTargets
        | ContinuationKind::ChooseYeyanSingleDamage
        | ContinuationKind::ChooseYeyanDoubleTarget
        | ContinuationKind::ChooseAllLoseHpOrRecover
        | ContinuationKind::ChooseDiscardWeaponOrLoseHp
        | ContinuationKind::ChooseDiscardOneHeOrLoseHp
        | ContinuationKind::ChooseSourceConfirmOwnerLoseHp
        | ContinuationKind::ChooseGainTargetCardAlternateZoneThenTargetDraw
        | ContinuationKind::ChooseQixingHandCards
        | ContinuationKind::ChooseQixingPileCards
        | ContinuationKind::ChooseQixingWeatherCards
        | ContinuationKind::ChooseQixingWeatherTargets
        | ContinuationKind::ChooseLinkOther
        | ContinuationKind::ChooseDrltPoxiCards
        | ContinuationKind::ChooseDrltJieyingTransfer
        | ContinuationKind::ChooseDrltDuoruiDisableSlot
        | ContinuationKind::ChooseDrltDuoruiSkill
        | ContinuationKind::ChooseDrltZhitiEnableSlot
        | ContinuationKind::ChooseJilueJizhiDiscard
        | ContinuationKind::ChooseAddMissingSkill
        | ContinuationKind::ChooseAddSkillBoundCardTarget
        | ContinuationKind::ChooseDiscardMatchingEquipColorAddSkill
        | ContinuationKind::ChooseCurrentCardUseEnhancements
        | ContinuationKind::ChooseAddCurrentCardTarget
        | ContinuationKind::ChooseAddOrRemoveCurrentCardTarget
        | ContinuationKind::ChooseDrawOrDiscardAddSkill
        | ContinuationKind::ChooseDiscardedBlackBasicAsBingliang
        | ContinuationKind::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage
        | ContinuationKind::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded
        | ContinuationKind::ChooseRevealOtherHandCardRememberCategory
        | ContinuationKind::ChooseGainResolvedShaOrShanThenOptionalUseSha
        | ContinuationKind::ChooseOtherTargetsBySkippedPhaseCountDamage
        | ContinuationKind::ChooseOthersAfterUniformHandReveal
        | ContinuationKind::ChooseCardUserOrAddTarget => {
            choices_choose_draw_or_recover_choose::resolve_choice(
                tx,
                actor,
                observer,
                kind,
                payload,
                selections,
                cancelled,
                vm_cont,
                optional_trigger,
            )
        }
        ContinuationKind::VirtualShaTargets
        | ContinuationKind::JudgeDamageOtherOnSuit
        | ContinuationKind::GanglieSourceDiscardOrDamage
        | ContinuationKind::ResolveGanglieThree
        | ContinuationKind::TuxiGainOtherHandsReplaceDraw
        | ContinuationKind::YijiDistributeTopCards
        | ContinuationKind::XiaoguoDiscardBasicThenEquipOrDamage
        | ContinuationKind::ResolveSpXiaoguo
        | ContinuationKind::ResolveTanlu
        | ContinuationKind::TargetDiscardThenDuelIfNonShaAndHpGteOwner
        | ContinuationKind::TongjiDiscardRedirectSha
        | ContinuationKind::LiuliDiscardRedirectSha
        | ContinuationKind::JijieChooseBottomCardRecipient
        | ContinuationKind::LuoshenRepeatBlackJudgment
        | ContinuationKind::JudgeSameColorGainRepeat
        | ContinuationKind::DelegatedGroupResponse
        | ContinuationKind::MoveEquipmentThenDraw
        | ContinuationKind::DiscardEquipmentPairWithArmor
        | ContinuationKind::DiscardTargetEquipmentGainMount
        | ContinuationKind::DiscardTargetEquipmentDrawWeaponRange
        | ContinuationKind::ResolveOlJieDanshou
        | ContinuationKind::GuanxingPick
        | ContinuationKind::RecoverToMaxThenOptionalExchangeSkill
        | ContinuationKind::DiscardTargetCardGainMount
        | ContinuationKind::ResolveDcqiaomeng
        | ContinuationKind::ResolveRetieji
        | ContinuationKind::ResolveNewReluoyi
        | ContinuationKind::ResolveXinyaoming
        | ContinuationKind::ResolveRehuaiyi
        | ContinuationKind::ResolveReanguo
        | ContinuationKind::ResolveBeizhu
        | ContinuationKind::ResolveMbshanjia
        | ContinuationKind::ResolveDaoji
        | ContinuationKind::GiveAllHandThenChooseUseOneOrDamage
        | ContinuationKind::ResolveTwfupan
        | ContinuationKind::ResolveTwruilian
        | ContinuationKind::ResolveTwjianwei
        | ContinuationKind::ResolveTwtanfeng
        | ContinuationKind::ResolveTwzhibian
        | ContinuationKind::ResolveTwyuyan
        | ContinuationKind::ResolveTwguoyi
        | ContinuationKind::ResolveTwchuhai
        | ContinuationKind::ResolveTwyizhu
        | ContinuationKind::ResolveTwxiawei
        | ContinuationKind::ResolveTwbianfuren
        | ContinuationKind::ResolveTwfeiyi
        | ContinuationKind::ResolveTwchenzhen
        | ContinuationKind::ResolveTwqingxi
        | ContinuationKind::ResolveTwzaoli
        | ContinuationKind::ResolveTwzhengjian
        | ContinuationKind::ResolveTwtianyu
        | ContinuationKind::ResolveTwwujing
        | ContinuationKind::ResolveTwshanghe
        | ContinuationKind::ResolveTwwangchang
        | ContinuationKind::ResolveTwcaozhao
        | ContinuationKind::ResolveTwguohuai
        | ContinuationKind::ClanshenjunUseCard
        | ContinuationKind::ClanyuzhiRevealChoice
        | ContinuationKind::ClanyuzhiEndChoice
        | ContinuationKind::QieyiPutTopChoice
        | ContinuationKind::JianzhiCountChoice
        | ContinuationKind::JianzhiAssignCard
        | ContinuationKind::ResolveJiaohua
        | ContinuationKind::ResolveLuanqun
        | ContinuationKind::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp
        | ContinuationKind::FanjianChooseSuitOrHand
        | ContinuationKind::ResolveMbzhuguo
        | ContinuationKind::ResolveMbanxian
        | ContinuationKind::ResolveMbjinzu
        | ContinuationKind::ForceOthersUseShaOnNearestOrLoseHp
        | ContinuationKind::ResolveOllinjie
        | ContinuationKind::GainTargetHandCard
        | ContinuationKind::GainFromSkillPile
        | ContinuationKind::StartDebateTargets
        | ContinuationKind::PinDebateOpinions
        | ContinuationKind::DistributeShadowCards
        | ContinuationKind::RegisterTurnEndDebate
        | ContinuationKind::LoseGainedSkill
        | ContinuationKind::ResolvePotheyun
        | ContinuationKind::CityConsume
        | ContinuationKind::TwjieqiuSelect
        | ContinuationKind::DiscardBasicDrawEqual
        | ContinuationKind::PlaceHandCardToExpansion
        | ContinuationKind::BetweenOpponent
        | ContinuationKind::BetweenFirstCard
        | ContinuationKind::BetweenSecondCard
        | ContinuationKind::DrawThenOptionallyDistributeHandCards
        | ContinuationKind::GiveSelectedThenContinueDistinctRecoverAtTwo
        | ContinuationKind::DiscardTiaojiyanmeiCard
        | ContinuationKind::ResolveGuanwei
        | ContinuationKind::ResolveLanjiang
        | ContinuationKind::ResolveRebingyi
        | ContinuationKind::ResolveOldimeng
        | ContinuationKind::ResolveRejieyin
        | ContinuationKind::ResolveReanxu
        | ContinuationKind::ResolveOldAnxu
        | ContinuationKind::ResolveOldQinqing
        | ContinuationKind::ResolveOldMingjian
        | ContinuationKind::ResolveOldXuanfeng
        | ContinuationKind::ResolveJianrou
        | ContinuationKind::ResolveOldHuisheng
        | ContinuationKind::ResolveOldJiefan
        | ContinuationKind::ResolveChegu
        | ContinuationKind::ResolveJunkguixin
        | ContinuationKind::ResolveGuhuo
        | ContinuationKind::ResolveFaceDownViewAs
        | ContinuationKind::ResolveSwapLowerHandViewAs
        | ContinuationKind::ResolveYaoChenwei
        | ContinuationKind::ResolveYoudi
        | ContinuationKind::ResolveKuangfu
        | ContinuationKind::ResolveShanjia
        | ContinuationKind::ResolveNewYajiao
        | ContinuationKind::GainOneHandFromSelectedTargets
        | ContinuationKind::ResolveZhuiyi
        | ContinuationKind::ResolveXinfuGuolun
        | ContinuationKind::ResolveDrltJueyan
        | ContinuationKind::ResolveRejizhi
        | ContinuationKind::ResolveJiexun
        | ContinuationKind::ResolveWengua
        | ContinuationKind::DiscardHandForMassDamage
        | ContinuationKind::DiscardMarkParityControlTargetCard
        | ContinuationKind::ConfirmMarkParityMassDamage
        | ContinuationKind::GiveExactRecordPhaseTarget => {
            choices_choose_draw_or_recover::resolve_choice(
                tx,
                actor,
                observer,
                kind,
                payload,
                selections,
                cancelled,
                vm_cont,
                optional_trigger,
            )
        }
        ContinuationKind::ChooseGiveCardOrAddTarget
        | ContinuationKind::ChooseDiscardEquipmentReduceCurrentDamage
        | ContinuationKind::ChooseDiscardDamageEndpointCardIncreaseIfSuit
        | ContinuationKind::ChooseEndExtraDrawOrPlay
        | ContinuationKind::ChooseMoveTargetEquipmentOrDraw
        | ContinuationKind::ChooseRecoverOwnerOrDamageByGivenEquipment
        | ContinuationKind::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage
        | ContinuationKind::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth
        | ContinuationKind::ChooseDrawOrRecastAddSkill
        | ContinuationKind::ChooseDrawOrDiscardTargetCardRecordSha
        | ContinuationKind::ChooseDiscardTargetCardsRecoverSameCategory
        | ContinuationKind::ChooseScskuijiDiscard
        | ContinuationKind::ChoosePskuijiDiscard
        | ContinuationKind::ChooseSpjungongDiscardOrLose
        | ContinuationKind::ChooseSpxizhanDiscardOrLose
        | ContinuationKind::ChooseSpsongshuCards
        | ContinuationKind::ChooseCompareTargetHandCardSecondFailureSeatDamage
        | ContinuationKind::ChooseDrawOrDiscardTargetCard
        | ContinuationKind::ChooseSeatPathDamage
        | ContinuationKind::ChooseGainShaForOpeningExtraPlay
        | ContinuationKind::ChooseHandCountDifferentTargetDrawOrRandomDiscard
        | ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw
        | ContinuationKind::ChooseGainOneFromDamageTargetAndNeighbors
        | ContinuationKind::ChoosePreventDamageOrLoseMaxHpIncreaseDamage
        | ContinuationKind::ChooseOneFromSkillPileDiscard
        | ContinuationKind::ChooseTargetHandCardToPile
        | ContinuationKind::ChooseHandCardsToSkillPile
        | ContinuationKind::ChooseGainTopCardsBySuit
        | ContinuationKind::ChooseOtherDrawByHandCount
        | ContinuationKind::ChooseOtherDrawOrRecover
        | ContinuationKind::ChooseDiscardHandEquipThenRemoveSelfSkill
        | ContinuationKind::ChooseTakeLargerHandUntilMax
        | ContinuationKind::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark => {
            choices_choose_give_card_or_add_target_choose::resolve_choice(
                tx,
                actor,
                observer,
                kind,
                payload,
                selections,
                cancelled,
                vm_cont,
                optional_trigger,
            )
        }
        ContinuationKind::RevealHandChooseDiscardColor
        | ContinuationKind::LoseHpGainDiscardCardForOpeningExtraPlay
        | ContinuationKind::SourceFreeViewAsEquipCost
        | ContinuationKind::SourceFreeViewAsPlay
        | ContinuationKind::SourceFreeViewAsResponse
        | ContinuationKind::MultiCardViewAsResponse
        | ContinuationKind::PlaceHandCardsToDeckTop
        | ContinuationKind::ResolveDcbingjiStage
        | ContinuationKind::ResolvePxqixianStage
        | ContinuationKind::ResolveJsrgdaorenStage
        | ContinuationKind::ResolveQianlongStage
        | ContinuationKind::ResolveJsrgzuozhanStage
        | ContinuationKind::ResolveJsrgzuozhanGainStage
        | ContinuationKind::ResolveTwdaiguiStage
        | ContinuationKind::ResolveTwmituStage
        | ContinuationKind::ResolveQiantunStage
        | ContinuationKind::ResolveYangmingStage
        | ContinuationKind::ResolveCanyunStage
        | ContinuationKind::ResolveBiaozhao3Stage
        | ContinuationKind::ResolveDcmoukuiStage
        | ContinuationKind::ResolveZhongyanStage
        | ContinuationKind::ResolveChumeiStage
        | ContinuationKind::ResolveShuaiyanStage
        | ContinuationKind::ResolveMbbeimingStage
        | ContinuationKind::ResolveGaoyuanStage
        | ContinuationKind::ResolveDuoduanStage
        | ContinuationKind::ResolveDcanxuStage
        | ContinuationKind::ResolveChulaoStage
        | ContinuationKind::ResolveMbzhenannStage
        | ContinuationKind::ResolveMeiyongStage
        | ContinuationKind::ResolveDcqiaolveStage
        | ContinuationKind::ResolveXinfuQiaiStage
        | ContinuationKind::ResolveOlzhijieStage
        | ContinuationKind::ResolveOlqiaozhiStage
        | ContinuationKind::ResolveZhongjieStage
        | ContinuationKind::ResolveNhlundaoStage
        | ContinuationKind::ResolveDingceStage
        | ContinuationKind::ResolveQiangyongStage
        | ContinuationKind::ResolveZhengjianStage
        | ContinuationKind::ResolveReliangyinStage
        | ContinuationKind::StoreSelectedInSkillPile
        | ContinuationKind::RearrangePileToDeckTop
        | ContinuationKind::VmResume
        | ContinuationKind::DiscardOneHandOrEquip
        | ContinuationKind::AdjustHandTo
        | ContinuationKind::DiscardHandAddSkillByColor
        | ContinuationKind::RevealTopGainNonBasicOrSwapBasic
        | ContinuationKind::DrawThenChooseGiveOrDiscardHand
        | ContinuationKind::GiveOneHandOrEquip
        | ContinuationKind::UseGivenEquipment
        | ContinuationKind::RevealDiscardSuitGainOrUse
        | ContinuationKind::DiscardOrFireDamageSkipPhases
        | ContinuationKind::ResumePlayedSha
        | ContinuationKind::ResumePlayedShaQueue
        | ContinuationKind::IndependentTrickQueue
        | ContinuationKind::FinishPlayedSha
        | ContinuationKind::ContinueEnteredDrawPhase
        | ContinuationKind::FinishDelayedJudgment
        | ContinuationKind::OptionalTrigger => {
            choices_choose_give_card_or_add_target::resolve_choice(
                tx,
                actor,
                observer,
                kind,
                payload,
                selections,
                cancelled,
                vm_cont,
                optional_trigger,
            )
        }
    }?;
    if tx.state().has_pending_choice() {
        return Err(EffectError::ChoiceRequested);
    }
    crate::effect::resume_deferred_resolution(tx)?;
    resume_interrupted_continuation(tx)
}

mod choices_choose_draw_or_recover;
mod choices_choose_draw_or_recover_choose;
mod choices_choose_give_card_or_add_target;
mod choices_choose_give_card_or_add_target_choose;
mod choices_custom;
mod choices_custom_potchendao;
mod choices_dying_save;
mod choices_dying_save_respond_rewrite;
