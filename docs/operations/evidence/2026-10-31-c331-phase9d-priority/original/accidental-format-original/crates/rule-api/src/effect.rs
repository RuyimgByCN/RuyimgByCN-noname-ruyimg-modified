//! Typed primitive effect requests.
//!
//! Skills never mutate stores directly; they emit these requests to the scheduler.

use engine_api::{CardId, MarkDefId, PlayerId, SkillDefId, SkillId, Suit, ZoneId};
use serde::{Deserialize, Serialize};

fn no_skill() -> SkillId {
    SkillId::NONE
}

fn no_skill_def() -> SkillDefId {
    SkillDefId::NONE
}

/// Canonical target-card zones available to a typed choose/discard effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetCardScope {
    /// Equipment only.
    Equip,
    /// Hand and equipment.
    HandEquip,
    /// Hand, equipment, and judgment zones (legacy `hej`).
    HandEquipJudge,
    /// Judgment zone only.
    Judge,
    /// Equipment and judgment zones only (legacy `ej`, no hand).
    ///
    /// Appended for serialized ordinal and Rule VM payload-tag stability.
    EquipJudge,
    /// Hand only.
    ///
    /// Appended for serialized ordinal and Rule VM payload-tag stability.
    Hand,
}

impl TargetCardScope {
    /// Stable Rule VM payload tag.
    pub const fn tag(self) -> u8 {
        match self {
            Self::Equip => 0,
            Self::HandEquip => 1,
            Self::HandEquipJudge => 2,
            Self::Judge => 3,
            Self::EquipJudge => 5,
            Self::Hand => 4,
        }
    }

    /// Decode a stable Rule VM payload tag.
    pub const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::Equip),
            1 => Some(Self::HandEquip),
            2 => Some(Self::HandEquipJudge),
            3 => Some(Self::Judge),
            5 => Some(Self::EquipJudge),
            4 => Some(Self::Hand),
            _ => None,
        }
    }

    /// Whether the scope contains the hand zone.
    pub const fn includes_hand(self) -> bool {
        matches!(self, Self::HandEquip | Self::HandEquipJudge | Self::Hand)
    }

    /// Whether the scope contains the equipment zone.
    pub const fn includes_equip(self) -> bool {
        matches!(
            self,
            Self::Equip | Self::HandEquip | Self::HandEquipJudge | Self::EquipJudge
        )
    }

    /// Whether the scope contains the judgment zone.
    pub const fn includes_judge(self) -> bool {
        matches!(self, Self::HandEquipJudge | Self::Judge | Self::EquipJudge)
    }
}

/// Canonical living-player scopes available to typed target choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawTargetScope {
    /// Any other living player.
    Other,
    /// Any other living male or double-sex player.
    MaleOther,
    /// Any other living female or double-sex player.
    FemaleOther,
    /// Any living player, including the chooser.
    Any,
    /// Any living player tied for the fewest hand cards.
    ///
    /// Appended for stable serialized ordinals and Rule VM payload tags.
    MinHand,
    /// Any living player whose current HP equals the chooser's current HP.
    ///
    /// Appended for stable serialized ordinals and Rule VM payload tags.
    SameHp,
    /// Any living player with no hand cards.
    ///
    /// Appended for stable serialized ordinals and Rule VM payload tags.
    HandEmpty,
    /// Any living wounded player, including the chooser.
    ///
    /// Appended for stable serialized ordinals and Rule VM payload tags.
    Wounded,
    /// Any other living player whose hand count does not exceed the chooser's.
    ///
    /// Appended for stable serialized ordinals and Rule VM payload tags.
    HandLteChooser,
}

impl DrawTargetScope {
    /// Stable Rule VM payload tag.
    pub const fn tag(self) -> u8 {
        match self {
            Self::Other => 0,
            Self::MaleOther => 1,
            Self::FemaleOther => 2,
            Self::Any => 3,
            Self::MinHand => 4,
            Self::SameHp => 5,
            Self::HandEmpty => 6,
            Self::Wounded => 7,
            Self::HandLteChooser => 8,
        }
    }

    /// Decode a stable Rule VM payload tag.
    pub const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::Other),
            1 => Some(Self::MaleOther),
            2 => Some(Self::FemaleOther),
            3 => Some(Self::Any),
            4 => Some(Self::MinHand),
            5 => Some(Self::SameHp),
            6 => Some(Self::HandEmpty),
            7 => Some(Self::Wounded),
            8 => Some(Self::HandLteChooser),
            _ => None,
        }
    }
}

fn no_player() -> PlayerId {
    PlayerId::NONE
}

fn no_card() -> CardId {
    CardId::NONE
}

fn no_suit() -> Suit {
    Suit::None
}

/// Damage nature tags for [`EffectRequest::DealDamage::nature`].
pub mod damage_nature {
    /// Ordinary damage.
    pub const NORMAL: u8 = 0;
    /// Fire damage (e.g. 火攻); 藤甲 +1.
    pub const FIRE: u8 = 1;
    /// Thunder damage (e.g. 闪电).
    pub const THUNDER: u8 = 2;
    /// Divine damage (神属性); elemental but does not propagate through iron chains.
    pub const KAMI: u8 = 3;
}

/// Stable selector groups for match-lifetime card-action rewards.
pub mod lifetime_card_reward_group {
    /// Any physical Sha nature or Shan.
    pub const SHA_OR_SHAN: u8 = 1;
    /// Tao or Jiu.
    pub const TAO_OR_JIU: u8 = 2;
    /// Duel or Wuzhong.
    pub const JUE_DOU_OR_WU_ZHONG: u8 = 3;
}

/// Provenance flags carried by card-linked damage.
pub mod damage_card_flag {
    /// The resolved Sha consumed a 酒 mark for its damage bonus.
    pub const JIU_BOOSTED: u8 = 1 << 0;
    /// This Sha was converted from ordinary to fire and must cost its user 1 HP after the whole
    /// use if any linked damage from it was committed.
    pub const CONVERTED_FIRE_LOSE_HP_AFTER_DAMAGE: u8 = 1 << 1;
    /// At least one damage segment belonging to the current card use was committed.
    pub const CARD_USE_DEALT_DAMAGE: u8 = 1 << 2;
    /// Draw for the originating skill after this exact Sha use if it commits no damage.
    pub const DRAW_IF_NO_DAMAGE: u8 = 1 << 3;
    /// This segment is elemental chain propagation rather than damage whose immediate parent is
    /// the originating card use.
    pub const CHAIN_PROPAGATED: u8 = 1 << 4;
    /// Damage belongs to the exact Sha used for old 解烦.
    pub const OLDJIEFAN: u8 = 1 << 5;
    /// The target was linked immediately before this damage segment committed.
    pub const TARGET_WAS_LINKED: u8 = 1 << 6;
    /// Damage belongs to a Sha forced by 备诛.
    pub const BEIZHU: u8 = 1 << 7;
}

// Functional modules feed one flat enum; this registry preserves Serde variant ordinals.
// Append new registry entries; keep each definition in its owning functional module.
mod card_movement;
mod choices;
mod clan_generals;
mod classic_generals;
mod damage_and_recovery;
mod draw_and_gain;
mod events;
mod extension_generals;
mod general_resolvers;
mod jsrg_generals;
mod judgment;
mod life_cycle;
mod mobile_generals;
mod online_generals;
mod pot_generals;
mod remastered_generals;
mod sha_resolution;
mod skills_and_marks;
mod sp_generals;
mod staged_resolvers;
mod taiwan_generals;
mod targeting;
mod turn_flow;

macro_rules! collect_variants {
    ([( $head:ident, $part:ident ) $(, ($tail:ident, $next:ident))*] [$($variants:tt)*]) => {
        $head::variants! { $part [$(($tail, $next)),*] [$($variants)*] }
    };
    ([] [$($variants:tt)*]) => {
        /// Typed effect the Rule VM / content may request.
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum EffectRequest { $($variants)* }
    };
}
collect_variants! { [
    (draw_and_gain, DrawCards),
    (card_movement, MoveCards),
    (damage_and_recovery, DealDamage),
    (skills_and_marks, AddSkill),
    (card_movement, RevealCards),
    (choices, RequestChoice),
    (life_cycle, EnterDying),
    (turn_flow, SetTurnState),
    (events, EmitDebateResolved),
    (card_movement, DiscardFromHand),
    (events, CardResponded),
    (card_movement, RemoveCardsByKind),
    (damage_and_recovery, DealShaDamage),
    (choices, ChooseDiscardOneHandOrEquip),
    (judgment, JudgeGainEquipmentByColor),
    (card_movement, RevealDiscardSuitGainOrUse),
    (damage_and_recovery, ChooseDiscardOrFireDamageSkipPhases),
    (draw_and_gain, DrawCardsFromSkill),
    (card_movement, DiscardAllHandAndEquip),
    (life_cycle, RequestDyingSave),
    (draw_and_gain, DrawEachBySeat),
    (card_movement, TakeOneCard),
    (turn_flow, TurnOver),
    (skills_and_marks, RemoveSkillByDef),
    (choices, ChooseDiscardHandAddSkillByColor),
    (card_movement, RevealTopGainNonBasicOrSwapBasic),
    (damage_and_recovery, PreventDamage),
    (judgment, JudgeCancelCurrentShaTargetOnRed),
    (choices, ChooseDrawOrRecover),
    (damage_and_recovery, DealCardDamage),
    (choices, ChooseUseVirtualSha),
    (draw_and_gain, AdjustHandTo),
    (card_movement, GainCardByKind),
    (judgment, JudgeRecoverOnSuit),
    (card_movement, GiveCards),
    (events, EmitDuelResolved),
    (judgment, JudgeDamageOtherOnSuit),
    (card_movement, RecastCards),
    (choices, ChooseUseVirtualShaOrDraw),
    (card_movement, DiscardEquipmentPairWithArmor),
    (sha_resolution, MakeCurrentShaTargetUnrespondable),
    (events, ShaDodged),
    (card_movement, ChooseDiscardTargetCard),
    (judgment, JudgePreventDamageLoseMaxHpOnNonHeart),
    (card_movement, ChooseGainTopCardsBySuit),
    (choices, ChooseOtherDrawByHandCount),
    (draw_and_gain, DrawByPlayPhaseCardCategoryCount),
    (sha_resolution, RefundCurrentShaUseAndDrawIfRed),
    (choices, ChooseDiscardOneForShaRefundAndDrawIfRed),
    (turn_flow, QueueOpeningExtraPlayPhase),
    (damage_and_recovery, ConvertCurrentShaToFireLoseHpIfDamage),
    (choices, ChooseAddCurrentShaTarget),
    (sha_resolution, CommitAddCurrentShaTarget),
    (choices, ChooseStoreHandShaInSkillPile),
    (skills_and_marks, CommitStoreHandShaInSkillPile),
    (card_movement, ChooseCompareSelectedHandCard),
    (skills_and_marks, AddBoundSkill),
    (card_movement, ChooseAddSkillBoundCardTarget),
    (choices, ChooseDiscardMatchingEquipColorAddSkill),
    (skills_and_marks, AddCardColorLockSkill),
    (card_movement, UseVirtualShaFromCardColorLockSourceIfNoSha),
    (skills_and_marks, AddCardUsePolicySkill),
    (choices, ChooseDrawOrDiscardAddSkill),
    (damage_and_recovery, ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage),
    (card_movement, DiscardSelectedRecordUniqueCategoryTarget),
    (choices, ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded),
    (card_movement, ChooseRevealOtherHandCardRememberCategory),
    (draw_and_gain, DrawThenGiveExactRecordPhaseTarget),
    (turn_flow, ClearPhaseGiftTarget),
    (events, EmitShaResolved),
    (choices, ChooseGainResolvedShaOrShanThenOptionalUseSha),
    (turn_flow, SkipCurrentPhase),
    (damage_and_recovery, ChooseOtherTargetsBySkippedPhaseCountDamage),
    (card_movement, RevealUniformHandDrawSelfChooseOthers),
    (damage_and_recovery, ChooseDiscardEquipmentReduceCurrentDamage),
    (choices, ChooseMoveTargetEquipmentOrDraw),
    (damage_and_recovery, GiveAllEquipmentChooseRecoverOrDamage),
    (choices, ChooseHandCountDifferentTargetDrawOrRandomDiscard),
    (draw_and_gain, IncreaseCurrentDrawCount),
    (choices, ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw),
    (draw_and_gain, DrawThenChooseGainOneFromDamageTargetAndNeighbors),
    (damage_and_recovery, ContinueChooseGainOneFromDamageTargetAndNeighbors),
    (card_movement, ContinueChooseCompareTargetHandCard),
    (events, EmitHandCompareFinished),
    (card_movement, GainHighestRankShaFromEventCards),
    (choices, ChooseHandCompare),
    (draw_and_gain, GiveSelectedThenRecoverSelf),
    (choices, ChooseHandCompareBetween),
    (judgment, GuanxingTop),
    (choices, ChooseOtherGiveThenRecover),
    (judgment, GuanxingTopByFaluMarks),
    (damage_and_recovery, ChooseOtherDamage),
    (card_movement, ChooseOtherDiscardTargetCard),
    (damage_and_recovery, RevealUniformHandChooseDamage),
    (card_movement, ChooseGainDiscardCardByKind),
    (draw_and_gain, DrawForLifetimeCardActionDivisors),
    (choices, ChooseAddMissingSkill),
    (card_movement, GainEventCards),
    (damage_and_recovery, ChooseTargetDrawToMaxHpCap),
    (choices, ChooseOtherDrawOwnerLostHpTurnOver),
    (judgment, JudgeRecoverOtherOnSuit),
    (sha_resolution, ForceOthersUseShaOnNearestOrLoseHp),
    (life_cycle, ContinueDyingSave),
    (card_movement, DiscardAllHandEquipJudge),
    (damage_and_recovery, DealDamageBanActiveIfDying),
    (sha_resolution, SpjungongDiscardOrLoseThenSha),
    (turn_flow, BanActiveUntilTurnEnd),
    (sp_generals, SpwanweiActive),
    (damage_and_recovery, CommitDamageBanActiveIfDying),
    (choices, ChooseTargetReclaimEquipmentDrawIfRangeReduced),
    (card_movement, ChooseUseVirtualCard),
    (damage_and_recovery, IncreaseCurrentShaBaseDamage),
    (turn_flow, SkipNextPlayPhase),
    (choices, ChooseOtherDrawOrRecover),
    (draw_and_gain, DrawSelectedTargets),
    (damage_and_recovery, RecoverIfMinCurrentHp),
    (turn_flow, RecordSelectedPlayPhaseCategory),
    (card_movement, TakeTargetHandThenRepeatUntilMax),
    (damage_and_recovery, ChooseDrawOrIncreaseCurrentCardDamageOrBothMark),
    (draw_and_gain, DrawThenChooseGiveOrDiscardHand),
    (card_movement, DiscardAllTargetEquipment),
    (damage_and_recovery, IncreaseCurrentShaTargetDamage),
    (draw_and_gain, GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp),
    (choices, ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp),
    (skills_and_marks, ReplaceSkillWithBoundAfterResolutionIfTargetAlive),
    (choices, ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp),
    (damage_and_recovery, ChooseTargetsDamage),
    (card_movement, DiscardTargetEquipmentDrawWeaponRange),
    (damage_and_recovery, ChooseTargetRevealHandDamageIfSha),
    (choices, ChooseOtherGiveOneHandEquip),
    (card_movement, ChooseRevealHandCardBindCategoryTarget),
    (damage_and_recovery, RecoverToMaxThenOptionalExchangeSkill),
    (card_movement, RevealHandDiscardKind),
    (choices, ChooseMoveTargetEquipmentAndOrDraw),
    (damage_and_recovery, ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth),
    (choices, ChooseDrawOrRecastAddSkill),
    (draw_and_gain, FinishOpeningExtraPlayShaGain),
    (card_movement, RevealTargetHandGainShanArmorOrDrawTwo),
    (draw_and_gain, GiveAllHand),
    (card_movement, GainCardIfDiscard),
    (choices, ChooseScskuijiDiscard),
    (damage_and_recovery, ChooseCompareSelectedHandCardSecondFailureSeatDamage),
    (card_movement, ChooseDrawOrDiscardTargetCard),
    (damage_and_recovery, RecordCompareFailureSeatDamage),
    (skills_and_marks, RemoveAllSkills),
    (choices, ChooseOtherRecover),
    (card_movement, EnableEquipSlot),
    (damage_and_recovery, TargetChooseSuitGainOwnerHandDamageIfMismatch),
    (judgment, JudgeSourceDiscardTwoHandOrDamage),
    (turn_flow, GiveSelectedRecoverAfterPhaseTotalTwo),
    (choices, ChooseOtherHandTargetsGainOneReplaceDraw),
    (card_movement, DistributeTopTwoCards),
    (judgment, JudgeMakeCurrentShaTargetUnrespondableOnRed),
    (damage_and_recovery, ChooseDiscardBasicThenTargetDiscardEquipOrDamage),
    (choices, ChooseCurrentShaTargetDiscardRedirectToOwner),
    (card_movement, ChooseTargetGainBottomCard),
    (judgment, JudgeBlackGainRepeat),
    (card_movement, UseVirtualJuedouSecondOnFirst),
    (judgment, JudgeSetSkillMarkToRank),
    (choices, ChooseDiscardOneFromEachLinked),
    (skills_and_marks, DisableSkillByDefUntilTurnEnd),
    (sha_resolution, TargetDiscardThenDuelIfNonShaAndHpGteOwner),
    (choices, ChooseTargetLoseHpDiscardHandShaIfBasic),
    (card_movement, RevealTargetHandShaOrDiscardBind),
    (draw_and_gain, DrawRememberedHandCards),
    (card_movement, DiscardRememberedHandCards),
    (damage_and_recovery, LoseHpGainDiscardCardForOpeningExtraPlay),
    (card_movement, ChooseGainOwnLostDiscardCardThisTurn),
    (draw_and_gain, DrawThenOptionallyDistributeHandCards),
    (choices, ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan),
    (card_movement, ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver),
    (skills_and_marks, AddSkillUntilTurnEnd),
    (classic_generals, ResolveMassDamageDiscardTurnOver),
    (targeting, RecordActiveTarget),
    (classic_generals, ResolveMarkParityControl),
    (draw_and_gain, ReplaceDrawWithDistinctSuitSelection),
    (targeting, InspectTargetHandChooseHeartDiscardOrTop),
    (classic_generals, ResolveYeyan),
    (choices, ChooseAllLoseHpOrRecover),
    (classic_generals, ResolveQixing),
    (choices, ChooseLinkOther),
    (classic_generals, ResolveDrltPoxi),
    (choices, ChooseDiscardWeaponOrLoseHp),
    (damage_and_recovery, DamageThenDiscardAllJudge),
    (skills_and_marks, ReplaceDrawGainTopTwoAddSkillsIfDifferentColors),
    (card_movement, ChooseGainTargetCardAlternateZoneThenTargetDraw),
    (turn_flow, SkipNextJudgePhase),
    (damage_and_recovery, ChooseDiscardDamageEndpointCardIncreaseIfSuit),
    (card_movement, TakeOneHandCard),
    (choices, ChooseOtherHandStoreSkillPile),
    (card_movement, GainAllSkillPileCards),
    (choices, ChooseDiscardOneHeOrLoseHp),
    (judgment, JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice),
    (choices, ChooseSourceConfirmOwnerLoseHp),
    (damage_and_recovery, LoseHpDoubleCurrentDamageThenReplace),
    (classic_generals, ResolveHiddenSkillPile),
    (skills_and_marks, SetNonLockedSkillSuppression),
    (classic_generals, ResolveDiliYuheng),
    (card_movement, FinalizeTiaojiyanmei),
    (choices, ChooseOtherYinghun),
    (staged_resolvers, ResolveGuanchaoRank),
    (remastered_generals, ResolveRebingyi),
    (classic_generals, ResolveDrltJueyan),
    (remastered_generals, ResolveRejizhi),
    (staged_resolvers, ResolveJiexun),
    (remastered_generals, ResolveXinanguo),
    (classic_generals, ResolveGanglieThree),
    (card_movement, DiscardEquipmentSlot),
    (remastered_generals, ResolveOldAnxu),
    (skills_and_marks, StoreSelectedInSkillPile),
    (remastered_generals, ResolveOldQinqing),
    (general_resolvers, ResolveJianrou),
    (remastered_generals, ResolveOldHuisheng),
    (general_resolvers, ResolveChegu),
    (remastered_generals, ResolveJunkguixin),
    (general_resolvers, ResolveYaoChenwei),
    (choices, ChooseAddCurrentShaTargetWithinDistance),
    (sha_resolution, RefundCurrentShaUse),
    (damage_and_recovery, DamageThenDrawSelectedTargets),
    (staged_resolvers, ResolveYoudi),
    (remastered_generals, ResolveNewYajiao),
    (general_resolvers, ResolveDingpin),
    (turn_flow, FinishCurrentPhaseAbortResolution),
    (card_movement, GainTopCardToSkillPile),
    (draw_and_gain, GainOneHandFromSelectedTargets),
    (choices, ChooseRepoluTargetsDraw),
    (remastered_generals, ResolveXinfuGuolun),
    (choices, ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank),
    (general_resolvers, ResolveBeigeConfigured),
    (remastered_generals, ResolveDecadezhenjun),
    (sp_generals, ResolveSpXiaoguo),
    (card_movement, ChooseGainCardFromEachOtherReplaceDraw),
    (draw_and_gain, GiveSelectedThenRecoverTarget),
    (general_resolvers, ResolveDingpan),
    (judgment, JudgeSameColorGainRepeat),
    (choices, ChooseCurrentTrickTargetsToNullify),
    (judgment, JudgeBlackGainRepeatProtectHandLimit),
    (choices, ChooseGainAllResolvedShaShanThenColorRewards),
    (card_movement, ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus),
    (online_generals, ResolveDcqiaomeng),
    (remastered_generals, ResolveNewReluoyi),
    (damage_and_recovery, ChooseTargetDiscardDistanceHandOrIncreaseDamage),
    (choices, ChooseDrawViewTwoGainOrMoveEquipment),
    (general_resolvers, ResolveBeizhu),
    (mobile_generals, ResolveMbshanjia),
    (staged_resolvers, ResolveDaoji),
    (choices, ChooseGiveTwoDifferentColorsOrRecover),
    (mobile_generals, ResolveMbzhuguo),
    (card_movement, TakeOneScopedCard),
    (draw_and_gain, DrawLinkedPlayers),
    (card_movement, RevealTopGainIfColor),
    (skills_and_marks, SuppressNonLockedSkillsUntilTurnEnd),
    (choices, ChooseRecoverTarget),
    (damage_and_recovery, GiveAllHandThenChooseUseOneOrDamage),
    (choices, ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable),
    (sp_generals, ResolveSpxizhan),
    (taiwan_generals, ResolveTwfupan),
    (sp_generals, ResolveSpsongshu),
    (taiwan_generals, ResolveTwjiaohua),
    (card_movement, MoveCardsWithReason),
    (taiwan_generals, ResolveTwruilian),
    (turn_flow, SkipJudgeAndDrawPhases),
    (card_movement, EquipWeaponFromDrawPile),
    (taiwan_generals, ResolveTwjianwei),
    (card_movement, ReplayVirtualCardOnTargets),
    (taiwan_generals, ResolveTwchuhai),
    (classic_generals, ResolveTianxiang),
    (remastered_generals, ResolveRetianxiang),
    (general_resolvers, ResolveLianzhou),
    (online_generals, ResolveStddechong),
    (card_movement, DiscardHandEquipCount),
    (general_resolvers, ResolveJieliangGain),
    (online_generals, ResolveStdjujian),
    (general_resolvers, ResolveLulve),
    (online_generals, ResolveDctingxian),
    (sp_generals, ResolveHfjieying),
    (online_generals, ResolveDcnuchen),
    (sp_generals, ResolveSpjiedao),
    (general_resolvers, ResolveSibian),
    (turn_flow, SkipNextDiscardPhase),
    (sp_generals, ResolveSpfengyin),
    (remastered_generals, ResolveXinfuLianpian),
    (general_resolvers, ResolveGongjian),
    (card_movement, RevealChooseCategoryDistributeSelfThresholdHpLoss),
    (choices, ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp),
    (remastered_generals, ResolveRefuhai),
    (choices, ChooseGainTargetHandCeilHalfDifference),
    (sp_generals, ResolveGzjinfa),
    (general_resolvers, ResolveWnQifeng),
    (remastered_generals, ResolveXinfuQinguo),
    (online_generals, ResolveDcmanzhi),
    (remastered_generals, ResolveXinxingluan),
    (online_generals, ResolveDckuizhen),
    (sp_generals, ResolveNsdingzhou),
    (remastered_generals, ResolveRekuanshi),
    (online_generals, ResolveDcmoyu),
    (general_resolvers, ResolveXizhen),
    (online_generals, ResolveDcchiying),
    (jsrg_generals, ResolveJsrgchiying),
    (remastered_generals, ResolveXinkuangfu),
    (online_generals, ResolveDcxiangshu),
    (general_resolvers, ResolveXuxieAdd),
    (remastered_generals, ResolveRekuanshiResolve),
    (taiwan_generals, ResolveTwxiawei),
    (sp_generals, ResolveSpcunsi),
    (taiwan_generals, ResolveTwshanghe),
    (general_resolvers, ResolveCuijin),
    (taiwan_generals, ResolveTwqingkou),
    (skills_and_marks, SetSkillContentFlag),
    (draw_and_gain, GiveSelectedThenTargetMayUseDrawOwner),
    (damage_and_recovery, ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne),
    (online_generals, ResolveDcyingtu),
    (choices, ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip),
    (mobile_generals, ResolveMobiledanshou),
    (events, EmitCardResolved),
    (mobile_generals, ResolveMbqingdao),
    (classic_generals, ResolveShuangxiong),
    (mobile_generals, ResolveMbxuye),
    (pot_generals, ResolvePotchengpu),
    (mobile_generals, ResolveMbzhuji),
    (pot_generals, ResolvePotlibing),
    (general_resolvers, ResolveYuli),
    (remastered_generals, ResolveXinmingshi),
    (staged_resolvers, ResolveXunyi),
    (sp_generals, ResolveDbzhuifeng),
    (staged_resolvers, ResolveLiubing),
    (classic_generals, ResolveDrltCongjian),
    (pot_generals, ResolvePothuanjie),
    (classic_generals, ResolveLieren),
    (pot_generals, ResolvePotnigu),
    (sp_generals, ResolveSpmiaolve),
    (general_resolvers, ResolveMantianguohai),
    (classic_generals, ResolveTuntian),
    (taiwan_generals, ResolveTwqiushou),
    (pot_generals, ResolvePottuntian),
    (staged_resolvers, ResolveZhengrong),
    (remastered_generals, ResolveReleiji),
    (classic_generals, ResolveHuangtian),
    (mobile_generals, ResolveMbzhonghui),
    (classic_generals, ResolveHaoshi),
    (pot_generals, ResolvePotjiejie),
    (classic_generals, ResolveQiaobian),
    (sha_resolution, ExcludeSavedShaTarget),
    (classic_generals, ResolveXiangle),
    (general_resolvers, ResolveZuilun),
    (targeting, BanUseAgainst),
    (staged_resolvers, ResolveJuzhan),
    (pot_generals, ResolvePotweiyan),
    (sp_generals, ResolveSpyajun),
    (remastered_generals, ResolveXinyangjie),
    (staged_resolvers, ResolveQingyu),
    (mobile_generals, ResolveMbdiancai),
    (remastered_generals, ResolveRechuhai),
    (general_resolvers, ResolveXiongshu),
    (sp_generals, ResolveSpqiai),
    (staged_resolvers, ResolveTongduo),
    (damage_and_recovery, ModifyCurrentDamage),
    (pot_generals, ResolvePotlvyi),
    (remastered_generals, ResolveRehengjiang),
    (skills_and_marks, AddFirstMissingSkill),
    (online_generals, ResolveDccunsi),
    (sha_resolution, RecordCurrentShaAllTargets),
    (choices, ChooseGiveOrDiscardHand),
    (online_generals, ResolveDcjue),
    (staged_resolvers, ResolveRuanji),
    (draw_and_gain, DrawCardsFromBottom),
    (general_resolvers, ResolveChenglve),
    (sp_generals, ResolveSxrmqiewang),
    (general_resolvers, ResolveXiongluan),
    (taiwan_generals, ResolveTwjuxiang),
    (turn_flow, SetDrawPhaseFromBottom),
    (general_resolvers, ResolveShicai),
    (online_generals, ResolveDcshaowei),
    (choices, ChooseDiscardHandRange),
    (online_generals, ResolveDcctjiuxian),
    (general_resolvers, ResolveBianyue),
    (online_generals, ResolveDcSbZhangxiu),
    (staged_resolvers, ResolveZhangjian),
    (sp_generals, ResolveNsshuaiyan),
    (extension_generals, ResolveWdbHanlong),
    (sha_resolution, ContinueYoudishenru),
    (extension_generals, ResolveWdbXizhenxihong),
    (card_movement, ContinueDiaobingqianjiang),
    (extension_generals, ResolveWdbZaozhiRenjun),
    (damage_and_recovery, PreventRecovery),
    (extension_generals, ResolveDddXuelingyun),
    (card_movement, ContinueWugu),
    (draw_and_gain, GuozhanUseZhulianBihe),
    (general_resolvers, ResolveShoulijian),
    (card_movement, CancelIncomingCardUse),
    (choices, ChooseOtherGainOneOfTwoSkills),
    (extension_generals, ResolveGwyewuChain),
    (skills_and_marks, RefreshSkill),
    (clan_generals, ClanshenjunReveal),
    (skills_and_marks, StorageSet),
    (draw_and_gain, DrawAllEventTargets),
    (events, RoundEnded),
    (clan_generals, ClanyuzhiReveal),
    (staged_resolvers, QieyiPhaseStart),
    (choices, ChooseUseVirtualNanman),
    (general_resolvers, ResolveQiluan),
    (card_movement, DiscardAllOwnJudge),
    (online_generals, ResolveOllinjie),
    (damage_and_recovery, ChooseTargetGainMaxHp),
    (card_movement, GainTargetHandCard),
    (draw_and_gain, ChooseClanNotMaxHandLimitGain),
    (card_movement, ExtraResolveCurrentTrick),
    (skills_and_marks, GainFromSkillPile),
    (card_movement, DiscardSkillPile),
    (choices, ChooseOneFromSkillPileDiscard),
    (card_movement, ChooseTargetHandCardToPile),
    (jsrg_generals, ResolveJsrgdaoren),
    (general_resolvers, ResolveQianlong),
    (jsrg_generals, ResolveJsrgzuozhan),
    (taiwan_generals, ResolveTwmouzhi),
    (staged_resolvers, ResolveQiantun),
    (online_generals, ResolveDcniji),
    (staged_resolvers, ResolveZhongyan),
    (taiwan_generals, ResolveTwxiongjun),
    (general_resolvers, ResolveYixiang),
    (remastered_generals, ResolveXinfuQiai),
    (sp_generals, ResolveSpolaige),
    (online_generals, ResolveOlzhijie),
    (staged_resolvers, ResolveZhongjie),
    (online_generals, ResolveStarcuxiaWeiming),
    (staged_resolvers, ResolveDingce),
    (mobile_generals, ResolveMbzhenann),
    (online_generals, ResolveFriendbiwei),
    (general_resolvers, ResolveChulao),
    (online_generals, ResolveDcanxu),
    (staged_resolvers, ResolveDuoduan),
    (targeting, ResetLinkedSelectedTargets),
    (mobile_generals, ResolveMbbeiming),
    (general_resolvers, ResolveZhengjian),
    (remastered_generals, ResolveReliangyin),
    (staged_resolvers, ResolveQinmian),
    (online_generals, ResolveDcbingji),
    (card_movement, ChooseHandCardsToSkillPile),
    (skills_and_marks, GainWholeSkillPile),
    (jsrg_generals, ResolveJsrgchaozheng),
    (card_movement, DiscardBasicDrawEqual),
    (events, StartDebateAllOthers),
    (card_movement, GainShadowCards),
    (skills_and_marks, ChangeGroup),
    (card_movement, DistributeShadowCards),
    (turn_flow, RegisterTurnEndDebate),
    (choices, ChooseGroup),
    (skills_and_marks, LoseGainedSkill),
    (pot_generals, ResolvePotheyun),
    (card_movement, PlaceXumou),
    (events, ResolveTurnEndDebates),
    (choices, ChooseOtherAddMark),
    (card_movement, PlaceDeckTopXumou),
    (choices, ChoosePlaceHandXumou),
    (card_movement, DiscardOneOwnXumou),
    (turn_flow, QueueExtraTurn),
    (choices, ChooseOthersInsertExtraTurn),
    (turn_flow, QueueExtraPhases),
    (taiwan_generals, ResolveTwjieqiu),
    (choices, ChooseOthersAddMark),
    (damage_and_recovery, ChooseOtherTargetsBySkippedPhaseCountDamageDraw),
    (choices, ChooseGiveOrRecastOneHand),
    (choices, ResolveOlJieDanshou)
] [] }
