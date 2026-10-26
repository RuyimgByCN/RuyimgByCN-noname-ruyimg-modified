//! Bridge Rule VM host calls into EffectRequest queues for the engine.

use crate::query::{query, QueryContext};
use crate::state::GameState;
use engine_api::{CardId, PlayerId};
use rule_api::{CapabilitySet, EffectRequest, HostTrap, QueryRequest, QueryResponse};
use rule_vm::{HostContext, VmHost};

/// Host that records effects and answers queries against live `GameState`.
pub struct EngineVmHost<'a> {
    /// Borrowed state (read-only queries).
    pub state: &'a GameState,
    /// Capability set.
    pub caps: CapabilitySet,
    /// Skill / event context.
    pub ctx: HostContext,
    /// Effects collected during execution.
    pub effects: Vec<EffectRequest>,
    /// Active invocation card inputs.
    pub selected_cards: Vec<u32>,
    /// Active invocation player targets.
    pub selected_targets: Vec<u32>,
    /// Physical cards captured from the triggering domain event.
    pub event_cards: Vec<u32>,
    /// Ordered player targets captured from the triggering domain event.
    pub event_targets: Vec<u32>,
    /// Damage nature for the current trigger.
    pub event_nature: u8,
    /// Logical damage-card kind for the current trigger.
    pub event_card_kind: u8,
    /// Damage-card provenance flags for the current trigger.
    pub event_card_flags: u8,
    /// Destination owner for the current CardsMoved trigger.
    pub event_move_target: PlayerId,
    /// From/to hand/equipment bits for the current CardsMoved trigger.
    pub event_move_mask: u8,
}

impl<'a> EngineVmHost<'a> {
    /// Create host for a skill firing on an event.
    pub fn new(
        state: &'a GameState,
        skill_id: u32,
        self_id: u32,
        event_player: u32,
        event_source: u32,
        event_card: u32,
        event_amount: u32,
    ) -> Self {
        Self {
            state,
            caps: CapabilitySet::standard_skill(),
            ctx: HostContext {
                skill_id,
                self_id,
                event_player,
                event_source,
                event_card,
                event_amount,
            },
            effects: Vec::new(),
            selected_cards: Vec::new(),
            selected_targets: Vec::new(),
            event_cards: Vec::new(),
            event_targets: Vec::new(),
            event_nature: 0,
            event_card_kind: 0,
            event_card_flags: 0,
            event_move_target: PlayerId::NONE,
            event_move_mask: 0,
        }
    }
}

impl VmHost for EngineVmHost<'_> {
    fn capabilities(&self) -> &CapabilitySet {
        &self.caps
    }

    fn context(&self) -> HostContext {
        self.ctx
    }

    fn selected_cards(&self) -> &[u32] {
        &self.selected_cards
    }

    fn selected_targets(&self) -> &[u32] {
        &self.selected_targets
    }

    fn event_cards(&self) -> &[u32] {
        &self.event_cards
    }

    fn event_targets(&self) -> &[u32] {
        &self.event_targets
    }

    fn emit_effect(&mut self, req: EffectRequest) -> Result<(), HostTrap> {
        if !self.caps.emit_effects {
            return Err(HostTrap::CapabilityDenied {
                name: "emit_effects".into(),
            });
        }
        self.effects.push(match req {
EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed { .. }
        | EffectRequest::TakeOneHandCard { .. }
        | EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits { .. }
        | EffectRequest::LoseHpDoubleCurrentDamageThenReplace { .. }
        | EffectRequest::PreventDamage { .. }
        | EffectRequest::PreventRecovery { .. }
        | EffectRequest::JudgePreventDamageLoseMaxHpOnNonHeart { .. }
        | EffectRequest::ChooseDiscardEquipmentReduceCurrentDamage { .. }
        | EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage { .. }
        | EffectRequest::ChooseDiscardDamageEndpointCardIncreaseIfSuit { .. }
        | EffectRequest::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage { .. }
        | EffectRequest::ChooseTargetDiscardDistanceHandOrIncreaseDamage { .. }
        | EffectRequest::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth { .. }
        | EffectRequest::ChooseDrawOrRecastAddSkill { .. }
        | EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha { .. }
        | EffectRequest::ChooseGainShaForOpeningExtraPlay { .. }
        | EffectRequest::FinishOpeningExtraPlayShaGain { .. }
        | EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay { .. }
        | EffectRequest::ChooseGainOwnLostDiscardCardThisTurn { .. }
        | EffectRequest::ChooseMoveFieldCardTo { .. }
        | EffectRequest::DrawThenOptionallyDistributeHandCards { .. }
        | EffectRequest::GiveSelectedThenContinueDistinctRecoverAtTwo { .. }
        | EffectRequest::GiveSelectedThenRecoverTarget { .. }
        | EffectRequest::GiveSelectedThenRecoverSelf { .. }
        | EffectRequest::ResolveDingpan { .. }
        | EffectRequest::ResolveTanlu { .. }
        | EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan { .. }
        | EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver { .. }
        | EffectRequest::ChooseGainCardFromEachOtherThenTurnOver { .. }
        | EffectRequest::ResolveHiddenSkillPile { .. }
        | EffectRequest::SetNonLockedSkillSuppression { .. }
        | EffectRequest::ResolveDiliYuheng { .. }
        | EffectRequest::DiscardSelectedDrawAllHandBonus { .. }
        | EffectRequest::ChooseOtherYinghun { .. }
        | EffectRequest::ChooseOtherBizheng { .. }
        | EffectRequest::ChooseOtherXiashu { .. }
        | EffectRequest::ResolveGuanchaoRank { .. }
        | EffectRequest::ResolveGuanwei { .. }
        | EffectRequest::ResolveLanjiang { .. }
        | EffectRequest::ResolveRebingyi { .. }
        | EffectRequest::ResolveRebotu { .. }
        | EffectRequest::ResolveOldimeng { .. }
        | EffectRequest::ResolveRejieyin { .. }
        | EffectRequest::ResolveReanxu { .. }
        | EffectRequest::ResolveOldAnxu { .. }
        | EffectRequest::StoreSelectedInSkillPile { .. }
        | EffectRequest::ResolveDcbingji { .. }
        | EffectRequest::ResolveQuanjia { .. }
        | EffectRequest::ResolveDuwang { .. }
        | EffectRequest::ResolveOldQinqing { .. }
        | EffectRequest::ResolveOldMingjian { .. }
        | EffectRequest::ResolveOldXuanfeng { .. }
        | EffectRequest::ResolveJianrou { .. }
        | EffectRequest::ResolveOldHuisheng { .. }
        | EffectRequest::ResolveOldJiefan { .. }
        | EffectRequest::ResolveChegu { .. }
        | EffectRequest::ResolveJunkguixin { .. }
        | EffectRequest::ResolveYaoChenwei { .. }
        | EffectRequest::ResolveDrltJueyan { .. }
        | EffectRequest::ResolveRejizhi { .. }
        | EffectRequest::ResolveJiexun { .. }
        | EffectRequest::ResolveWenguaSetup { .. }
        | EffectRequest::ResolveWengua { .. }
        | EffectRequest::ResolveXinanguo { .. }
        | EffectRequest::ResolveGanglieThree { .. }
        | EffectRequest::ResolveZhongyi { .. }
        | EffectRequest::ResolveMassDamageDiscardTurnOver { .. }
        | EffectRequest::ResolveMassDamageDiscardHandEquip { .. }
        | EffectRequest::RecordActiveTarget { .. }
        | EffectRequest::ChooseDiscardWeaponOrLoseHp { .. }
        | EffectRequest::ChooseDiscardOneHeOrLoseHp { .. }
        | EffectRequest::JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice { .. }
        | EffectRequest::ChooseSourceConfirmOwnerLoseHp { .. }
        | EffectRequest::ResolveMarkParityControl { .. }
        | EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage { .. }
        | EffectRequest::ReplaceDrawWithDistinctSuitSelection { .. }
        | EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors { .. }
        | EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw { .. }
        | EffectRequest::InspectTargetHandChooseHeartDiscardOrTop { .. }
        | EffectRequest::ResolveYeyan { .. }
        | EffectRequest::ChooseAllLoseHpOrRecover { .. }
        | EffectRequest::ResolveQixing { .. }
        | EffectRequest::ResolveQixingWeather { .. }
        | EffectRequest::ResolveNzryLongnu { .. }
        | EffectRequest::ChooseLinkOther { .. }
        | EffectRequest::ResolveDrltPoxi { .. }
        | EffectRequest::ResolveDrltJieying { .. }
        | EffectRequest::ResolveDrltDuorui { .. }
        | EffectRequest::ResolveDrltZhiti { .. }
        | EffectRequest::ResolveShenSimayi { .. }
        | EffectRequest::ResolveBeige { .. }
        | EffectRequest::ResolveBeigeConfigured { .. }
        | EffectRequest::RevealTargetHandGainShanArmorOrDrawTwo { .. }
        | EffectRequest::DrawCards { .. }
        | EffectRequest::DrawEachBySeat { .. }
        | EffectRequest::DrawThenGiveOne { .. }
        | EffectRequest::DrawThenTakeOne { .. }
        | EffectRequest::MoveEquipmentThenDraw { .. }
        | EffectRequest::ChooseUseVirtualShaOrDraw { .. }
        | EffectRequest::DiscardEquipmentPairWithArmor { .. }
        | EffectRequest::ChooseOtherDrawByHandCount { .. }
        | EffectRequest::ChooseOtherDrawOrRecover { .. }
        | EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill { .. }
        | EffectRequest::ChooseDiscardHandEquipThenRecover { .. }
        | EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory { .. }
        | EffectRequest::ChooseCompareSelectedHandCardSecondFailureSeatDamage { .. }
        | EffectRequest::DrawSelectedTargets { .. }
        | EffectRequest::ResolveOlwenren { .. }
        | EffectRequest::RecoverSelectedTargets { .. }
        | EffectRequest::RecordSelectedPlayPhaseCategory { .. }
        | EffectRequest::ChooseTargetDraw { .. } => bindings_judge_make_current_sha_target_unrespondable_on_red::bind_effect(self, req)?,
EffectRequest::ChooseTargetsDraw { .. }
        | EffectRequest::ChooseTargetsDamage { .. }
        | EffectRequest::ChooseOptionalTargetsDraw { .. }
        | EffectRequest::RevealTargetHandShaOrDiscardBind { .. }
        | EffectRequest::DrawRememberedHandCards { .. }
        | EffectRequest::DiscardRememberedHandCards { .. }
        | EffectRequest::DiscardTargetEquipmentDrawWeaponRange { .. }
        | EffectRequest::ChooseTargetRevealHandDamageIfSha { .. }
        | EffectRequest::ChooseOtherGiveOneHandEquip { .. }
        | EffectRequest::ChooseGiveOrRecastOneHand { .. }
        | EffectRequest::ChooseRevealHandCardBindCategoryTarget { .. }
        | EffectRequest::RecoverToMaxThenOptionalExchangeSkill { .. }
        | EffectRequest::ChooseTargetDrawToMaxHpCap { .. }
        | EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { .. }
        | EffectRequest::DrawByPlayPhaseCardCategoryCount { .. }
        | EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed { .. }
        | EffectRequest::ResolveOllinjie { .. }
        | EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank { .. }
        | EffectRequest::QueueOpeningExtraPlayPhase { .. }
        | EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage { .. }
        | EffectRequest::IncreaseCurrentShaBaseDamage { .. }
        | EffectRequest::IncreaseCurrentCardBaseDamage { .. }
        | EffectRequest::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark { .. }
        | EffectRequest::DrawThenChooseGiveOrDiscardHand { .. }
        | EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp { .. }
        | EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess { .. }
        | EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp { .. }
        | EffectRequest::TargetChooseSuitGainOwnerHandDamageIfMismatch { .. }
        | EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo { .. }
        | EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw { .. }
        | EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw { .. }
        | EffectRequest::ResolveDcqiaomeng { .. }
        | EffectRequest::ResolveNewReluoyi { .. }
        | EffectRequest::ResolveRemoshi { .. }
        | EffectRequest::ResolveXinyaoming { .. }
        | EffectRequest::ResolveRehuaiyi { .. }
        | EffectRequest::ResolveReanguo { .. }
        | EffectRequest::ResolveBeizhu { .. }
        | EffectRequest::ResolveMbshanjia { .. }
        | EffectRequest::ResolveDaoji { .. }
        | EffectRequest::GiveAllHandThenChooseUseOneOrDamage { .. }
        | EffectRequest::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable { .. }
        | EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants { .. }
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
        | EffectRequest::ResolveTwyizhu { .. }
        | EffectRequest::ResolveTwluanchou { .. }
        | EffectRequest::ResolveYuli { .. }
        | EffectRequest::ResolveTingwei { .. }
        | EffectRequest::ResolveXinmingshi { .. }
        | EffectRequest::ResolveXunyi { .. }
        | EffectRequest::ResolveBoming { .. }
        | EffectRequest::ResolveSpqiai { .. }
        | EffectRequest::ResolveEjian { .. }
        | EffectRequest::ResolveYiyong { .. }
        | EffectRequest::ResolveShanxie { .. }
        | EffectRequest::ResolveTuntian { .. }
        | EffectRequest::ResolveZaoxian { .. }
        | EffectRequest::ResolveTianxiang { .. }
        | EffectRequest::ResolveRetianxiang { .. }
        | EffectRequest::ChooseOtherAddMark { .. }
        | EffectRequest::ResolveStddechong { .. }
        | EffectRequest::ResolveStdjujian { .. }
        | EffectRequest::ResolveDctingxian { .. }
        | EffectRequest::ResolveXinfuQinguo { .. }
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
        | EffectRequest::ResolveXuxieAdd { .. }
        | EffectRequest::ResolveDcchiying { .. }
        | EffectRequest::ResolveDcmanzhi { .. }
        | EffectRequest::ResolveXinkuangfu { .. }
        | EffectRequest::ResolveDcxiangshu { .. }
        | EffectRequest::ResolveDcxiangshuLottery { .. }
        | EffectRequest::ResolveGongjian { .. }
        | EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss { .. }
        | EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp { .. }
        | EffectRequest::ResolveRefuhai { .. }
        | EffectRequest::ChooseGainTargetHandCeilHalfDifference { .. }
        | EffectRequest::ResolveGzjinfa { .. }
        | EffectRequest::ResolveWnQifeng { .. }
        | EffectRequest::GiveSelectedThenTargetMayUseDrawOwner { .. }
        | EffectRequest::ResolveDcyingtu { .. }
        | EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne { .. }
        | EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip { .. }
        | EffectRequest::ResolveMobiledanshou { .. }
        | EffectRequest::ResolveJsrgchiying { .. }
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
        | EffectRequest::ResolveMbkuangxiang { .. } => bindings_choose_targets_draw::bind_effect(self, req)?,
EffectRequest::ResolvePotchengpu { .. }
        | EffectRequest::ResolvePotjugu { .. }
        | EffectRequest::ResolvePotsifeng { .. }
        | EffectRequest::ResolvePotchenqun { .. }
        | EffectRequest::ResolvePotwanglie { .. }
        | EffectRequest::ResolvePothongyi { .. }
        | EffectRequest::ResolvePotguansha { .. }
        | EffectRequest::ResolvePotjiyu { .. }
        | EffectRequest::ResolveMbzhuji { .. }
        | EffectRequest::ResolvePotlibing { .. }
        | EffectRequest::ResolvePotpoxi { .. }
        | EffectRequest::ResolvePotfuan { .. }
        | EffectRequest::ResolvePotyinxian { .. }
        | EffectRequest::ResolvePothuanjie { .. }
        | EffectRequest::ResolvePotnigu { .. }
        | EffectRequest::ResolvePotlulian { .. }
        | EffectRequest::ResolvePotdimeng { .. }
        | EffectRequest::ResolvePothaoshi { .. }
        | EffectRequest::ResolveSpmiaolve { .. }
        | EffectRequest::ResolveSpyingjia { .. }
        | EffectRequest::ResolveTiaoxin { .. }
        | EffectRequest::ResolveZhiji { .. }
        | EffectRequest::ResolveZaiqi { .. }
        | EffectRequest::ResolveTwqiushou { .. }
        | EffectRequest::ResolvePottuntian { .. }
        | EffectRequest::ResolvePotjixi { .. }
        | EffectRequest::ResolvePotceduan { .. }
        | EffectRequest::ResolvePottaishici { .. }
        | EffectRequest::ResolveMbzhonghui { .. }
        | EffectRequest::ResolvePotjiejie { .. }
        | EffectRequest::ResolvePotqingshi { .. }
        | EffectRequest::ResolvePotweiyan { .. }
        | EffectRequest::ResolvePotzhouyu { .. }
        | EffectRequest::ResolvePotlvyi { .. }
        | EffectRequest::ResolvePotcaoshuang { .. }
        | EffectRequest::ResolvePotyuji { .. }
        | EffectRequest::ResolveHaoshi { .. }
        | EffectRequest::ResolveDimeng { .. }
        | EffectRequest::ResolveQiaobian { .. }
        | EffectRequest::ResolveXiangle { .. }
        | EffectRequest::ResolveFangquanSkip { .. }
        | EffectRequest::ResolveFangquanGrant { .. }
        | EffectRequest::ResolveRuoyu { .. }
        | EffectRequest::ResolveZuilun { .. }
        | EffectRequest::ResolveFuyin { .. }
        | EffectRequest::ResolveJuzhan { .. }
        | EffectRequest::ResolveQizhi { .. }
        | EffectRequest::ResolveJinqu { .. }
        | EffectRequest::ResolveJianxiang { .. }
        | EffectRequest::ResolveShenshi { .. }
        | EffectRequest::ResolveMingren { .. }
        | EffectRequest::ResolveZhenliang { .. }
        | EffectRequest::ResolveKuizhu { .. }
        | EffectRequest::ResolveZhizheng { .. }
        | EffectRequest::ResolveLijun { .. }
        | EffectRequest::ResolveLiangyin { .. }
        | EffectRequest::ResolveKongsheng { .. }
        | EffectRequest::ResolveReleiji { .. }
        | EffectRequest::ResolveHuangtian { .. }
        | EffectRequest::ResolveZhengrong { .. }
        | EffectRequest::ResolveHongju { .. }
        | EffectRequest::ResolveQingjian { .. }
        | EffectRequest::ResolveXinfuLianpian { .. }
        | EffectRequest::ResolveJieliangGain { .. }
        | EffectRequest::ResolveShuangxiong { .. }
        | EffectRequest::ResolveZhijian { .. }
        | EffectRequest::ResolveGuzheng { .. }
        | EffectRequest::ResolveZhiba { .. }
        | EffectRequest::ResolveDrltCongjian { .. }
        | EffectRequest::ResolveNzryFeijun { .. }
        | EffectRequest::ResolveDrltYongsi { .. }
        | EffectRequest::ResolveDrltWeidi { .. }
        | EffectRequest::ResolveLieren { .. }
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
        | EffectRequest::ResolveRehengjiang { .. }
        | EffectRequest::ResolveTwguohuai { .. }
        | EffectRequest::ResolveTwchengpu { .. }
        | EffectRequest::ResolveTwcaoang { .. }
        | EffectRequest::ResolveTwniujin { .. }
        | EffectRequest::ResolveTwmoukui { .. }
        | EffectRequest::ResolveTwfuman { .. }
        | EffectRequest::ResolveCuijin { .. }
        | EffectRequest::ResolveTwqingkou { .. }
        | EffectRequest::ResolveTwchenxun { .. }
        | EffectRequest::ResolveTwfenxian { .. }
        | EffectRequest::SetSkillContentFlag { .. }
        | EffectRequest::ResolveJiaohua { .. }
        | EffectRequest::ResolveLuanqun { .. }
        | EffectRequest::DistributeTopTwoCards { .. }
        | EffectRequest::ChooseDiscardBasicThenTargetDiscardEquipOrDamage { .. }
        | EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner { .. }
        | EffectRequest::ChooseTargetLoseHpDiscardHandShaIfBasic { .. }
        | EffectRequest::ChooseCurrentShaTargetDiscardRedirectToOwner { .. }
        | EffectRequest::ChooseDiscardRedirectCurrentSha { .. }
        | EffectRequest::ChooseTargetGainBottomCard { .. }
        | EffectRequest::JudgeBlackGainRepeat { .. }
        | EffectRequest::JudgeBlackGainRepeatProtectHandLimit { .. }
        | EffectRequest::JudgeSameColorGainRepeat { .. } => bindings_resolve_potchengpu::bind_effect(self, req)?,
EffectRequest::ChooseCurrentTrickTargetsToNullify { .. }
        | EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp { .. }
        | EffectRequest::ChooseGiveOneHandEquipOrRecover { .. }
        | EffectRequest::ChooseGiveTwoDifferentColorsOrRecover { .. }
        | EffectRequest::ResolveMbzhuguo { .. }
        | EffectRequest::ResolveMbanxian { .. }
        | EffectRequest::ResolveMbjinzu { .. }
        | EffectRequest::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp { .. }
        | EffectRequest::ChooseBoundSourceAttackRangeDamage { .. }
        | EffectRequest::ChooseAddCurrentShaTarget { .. }
        | EffectRequest::ChooseAddCurrentShaTargetWithinDistance { .. }
        | EffectRequest::RefundCurrentShaUse { .. }
        | EffectRequest::DamageThenDrawSelectedTargets { .. }
        | EffectRequest::ResolveYoudi { .. }
        | EffectRequest::ResolveKuangfu { .. }
        | EffectRequest::ResolveShanjia { .. }
        | EffectRequest::ResolveNewYajiao { .. }
        | EffectRequest::ResolveDingpin { .. }
        | EffectRequest::GainTopCardToSkillPile { .. }
        | EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill { .. }
        | EffectRequest::GainOneHandFromSelectedTargets { .. }
        | EffectRequest::ChooseRepoluTargetsDraw { .. }
        | EffectRequest::ResolveZhuiyi { .. }
        | EffectRequest::ResolveDecadezhenjun { .. }
        | EffectRequest::ResolveSpXiaoguo { .. }
        | EffectRequest::ChooseGainCardFromEachOtherReplaceDraw { .. }
        | EffectRequest::ResolveXinfuGuolun { .. }
        | EffectRequest::ChooseStoreHandShaInSkillPile { .. }
        | EffectRequest::ChooseSpendSkillPileForDyingJiu { .. }
        | EffectRequest::ChooseCompareSelectedHandCard { .. }
        | EffectRequest::RandomizeCurrentHandCompareOpponent { .. }
        | EffectRequest::GainHighestRankShaFromEventCards { .. }
        | EffectRequest::GainHandCompareResultCard { .. }
        | EffectRequest::ChooseAddOrRemoveCurrentCardTarget { .. }
        | EffectRequest::ChooseHandCompare { .. }
        | EffectRequest::ChooseHandCompareGrantTargetOnWin { .. }
        | EffectRequest::ChooseHandCompareBetween { .. }
        | EffectRequest::GuanxingTop { .. }
        | EffectRequest::ChooseOtherGiveThenRecover { .. }
        | EffectRequest::GuanxingTopByFaluMarks { .. }
        | EffectRequest::ChooseOtherDamage { .. }
        | EffectRequest::RevealUniformHandChooseDamage { .. }
        | EffectRequest::ChooseGainDiscardCardByKind { .. }
        | EffectRequest::ChooseAddMissingSkill { .. }
        | EffectRequest::AddFirstMissingSkill { .. }
        | EffectRequest::ResolveDccunsi { .. }
        | EffectRequest::RecordCurrentShaAllTargets { .. }
        | EffectRequest::ChooseGiveOrDiscardHand { .. }
        | EffectRequest::ChooseDiscardHandEquipThenRecoverTarget { .. }
        | EffectRequest::ResolveDcjue { .. }
        | EffectRequest::ResolveRuanji { .. }
        | EffectRequest::ResolveZhengbi { .. }
        | EffectRequest::ResolveFengying { .. }
        | EffectRequest::ResolveQiaojian { .. }
        | EffectRequest::ResolveXicha { .. }
        | EffectRequest::ResolveHuzun { .. }
        | EffectRequest::ResolveChenglve { .. }
        | EffectRequest::ResolveShicai { .. }
        | EffectRequest::ResolveXiongluan { .. }
        | EffectRequest::ResolveGzbuqu { .. }
        | EffectRequest::ResolveSbbiyue { .. }
        | EffectRequest::ResolveSxrmqiewang { .. }
        | EffectRequest::ResolveSxrmqianliu { .. }
        | EffectRequest::ChooseOtherHandStoreSkillPile { .. }
        | EffectRequest::GainAllSkillPileCards { .. }
        | EffectRequest::GainOtherSkillPileCards { .. }
        | EffectRequest::ResolveGzbuquRecover { .. }
        | EffectRequest::ResolveTwjuxiang { .. }
        | EffectRequest::ResolveZhugemengxue { .. }
        | EffectRequest::ResolveBailingyun { .. }
        | EffectRequest::ResolveDcshaowei { .. }
        | EffectRequest::ResolveDcdichou { .. }
        | EffectRequest::ResolveDcdufeng { .. }
        | EffectRequest::ChooseDiscardHandRange { .. }
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
        | EffectRequest::ResolveWdbHanlong { .. }
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
        | EffectRequest::ResolveDddLiuba { .. } => bindings_choose_current_trick_targets_to_nullify::bind_effect(self, req)?,
EffectRequest::ResolveDddJianshuo { .. }
        | EffectRequest::ResolveDddGuanning { .. }
        | EffectRequest::ResolveDddCaoshuang { .. }
        | EffectRequest::ResolveDddXuelingyun { .. }
        | EffectRequest::ResolveDddLiuhong { .. }
        | EffectRequest::ResolveDddXiahouxuan { .. }
        | EffectRequest::ResolveDddZhangkai { .. }
        | EffectRequest::ResolveDddLiangxi { .. }
        | EffectRequest::ChooseOtherDiscardTargetCard { .. }
        | EffectRequest::ChooseAddSkillBoundCardTarget { .. }
        | EffectRequest::ChooseDiscardMatchingEquipColorAddSkill { .. }
        | EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { .. }
        | EffectRequest::ChooseCurrentCardUseEnhancements { .. }
        | EffectRequest::ChooseDrawOrDiscardAddSkill { .. }
        | EffectRequest::ChooseDiscardedBlackBasicAsBingliang { .. }
        | EffectRequest::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage { .. }
        | EffectRequest::DiscardSelectedRecordUniqueCategoryTarget { .. }
        | EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded { .. }
        | EffectRequest::ChooseRevealOtherHandCardRememberCategory { .. }
        | EffectRequest::DrawThenGiveExactRecordPhaseTarget { .. }
        | EffectRequest::ClearPhaseGiftTarget { .. }
        | EffectRequest::ChooseGainResolvedShaOrShanThenOptionalUseSha { .. }
        | EffectRequest::ChooseGainAllResolvedShaShanThenColorRewards { .. }
        | EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage { .. }
        | EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw { .. }
        | EffectRequest::RevealUniformHandDrawSelfChooseOthers { .. }
        | EffectRequest::ChooseCardUserOrAddTarget { .. }
        | EffectRequest::ChooseMoveTargetEquipmentOrDraw { .. }
        | EffectRequest::ChooseMoveTargetEquipmentAndOrDraw { .. }
        | EffectRequest::ChooseDrawViewTwoGainOrMoveEquipment { .. }
        | EffectRequest::GiveAllEquipmentChooseRecoverOrDamage { .. }
        | EffectRequest::ChooseHandCountDifferentTargetDrawOrRandomDiscard { .. }
        | EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw { .. }
        | EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors { .. }
        | EffectRequest::ChoosePreventDamageOrLoseMaxHpIncreaseDamage { .. }
        | EffectRequest::ForceOthersUseShaOnNearestOrLoseHp { .. } => bindings_resolve_ddd_jianshuo::bind_effect(self, req)?,
req => req,
});
        Ok(())
    }

    fn run_query(&mut self, req: QueryRequest) -> Result<QueryResponse, HostTrap> {
        if !self.caps.run_queries {
            return Err(HostTrap::CapabilityDenied {
                name: "run_queries".into(),
            });
        }
        let mut ctx = QueryContext::new();
        Ok(query(self.state, &req, &mut ctx))
    }
}

/// Execute a verified module and return collected effects.
///
/// If the module pauses on `choose{}` (HOST_CHOICE), the effects emitted before
/// the pause are returned but the pause is dropped — callers that support DSL
/// choice must use [`run_module`] instead.
pub fn run_module_for_effects(
    state: &GameState,
    module: &rule_vm::VerifiedModule,
    skill_id: u32,
    self_id: u32,
    event_player: u32,
    event_source: u32,
    event_card: u32,
    event_amount: u32,
    fuel: u32,
) -> Result<Vec<EffectRequest>, rule_vm::VmError> {
    Ok(run_module(
        state,
        module,
        skill_id,
        [self_id, event_player, event_source, event_card],
        event_amount,
        fuel,
    )?
    .effects)
}

/// Outcome of running a VM module: emitted effects plus an optional pause.
pub struct VmRunOutcome {
    /// Effects emitted before completion or pause.
    pub effects: Vec<EffectRequest>,
    /// Set when the module paused on `choose{}`.
    pub paused: Option<VmPause>,
}

/// A paused VM run awaiting a choice resolution.
pub struct VmPause {
    /// Serialized continuation frame pack.
    pub frames: Vec<u8>,
    /// Resume program counter.
    pub pc: u32,
    /// Fuel remaining for the resumed segment.
    pub fuel_remaining: u32,
    /// Option tokens offered.
    pub options: Vec<u32>,
}

/// Execute a module, surfacing a pause if it hits `choose{}`.
///
/// `ctx` = `[self_id, event_player, event_source, event_card]`.
pub fn run_module(
    state: &GameState,
    module: &rule_vm::VerifiedModule,
    skill_id: u32,
    ctx: [u32; 4],
    event_amount: u32,
    fuel: u32,
) -> Result<VmRunOutcome, rule_vm::VmError> {
    run_module_with_selected_cards(state, module, skill_id, ctx, event_amount, &[], fuel)
}

/// Execute a trigger module while preserving every physical card from the domain event.
pub fn run_module_with_event_cards(
    state: &GameState,
    module: &rule_vm::VerifiedModule,
    skill_id: u32,
    ctx: [u32; 4],
    event_amount: u32,
    event_cards: &[u32],
    event_targets: &[u32],
    event_nature: u8,
    event_card_kind: u8,
    event_card_flags: u8,
    event_move_target: PlayerId,
    event_move_mask: u8,
    fuel: u32,
) -> Result<VmRunOutcome, rule_vm::VmError> {
    let mut vm = rule_vm::RuleVm::new();
    let mut host = EngineVmHost::new(
        state,
        skill_id,
        ctx[0],
        ctx[1],
        ctx[2],
        ctx[3],
        event_amount,
    );
    host.event_cards = event_cards.to_vec();
    host.event_targets = event_targets.to_vec();
    host.event_nature = event_nature;
    host.event_card_kind = event_card_kind;
    host.event_card_flags = event_card_flags;
    host.event_move_target = event_move_target;
    host.event_move_mask = event_move_mask;
    match vm.execute(module, &mut host, fuel)? {
        rule_vm::ExecResult::Halted { .. } => Ok(VmRunOutcome {
            effects: host.effects,
            paused: None,
        }),
        rule_vm::ExecResult::Paused {
            continuation,
            options,
        } => Ok(VmRunOutcome {
            effects: host.effects,
            paused: Some(VmPause {
                frames: continuation.frames,
                pc: continuation.pc,
                fuel_remaining: continuation.fuel_remaining,
                options,
            }),
        }),
    }
}

/// Execute a module with declaratively selected active-skill card inputs.
pub fn run_module_with_selected_cards(
    state: &GameState,
    module: &rule_vm::VerifiedModule,
    skill_id: u32,
    ctx: [u32; 4],
    event_amount: u32,
    selected_cards: &[u32],
    fuel: u32,
) -> Result<VmRunOutcome, rule_vm::VmError> {
    run_module_with_selected_inputs(
        state,
        module,
        skill_id,
        ctx,
        event_amount,
        selected_cards,
        &[],
        fuel,
    )
}

/// Execute a module with declaratively selected active-skill card and player inputs.
pub fn run_module_with_selected_inputs(
    state: &GameState,
    module: &rule_vm::VerifiedModule,
    skill_id: u32,
    ctx: [u32; 4],
    event_amount: u32,
    selected_cards: &[u32],
    selected_targets: &[u32],
    fuel: u32,
) -> Result<VmRunOutcome, rule_vm::VmError> {
    let mut vm = rule_vm::RuleVm::new();
    let mut host = EngineVmHost::new(
        state,
        skill_id,
        ctx[0],
        ctx[1],
        ctx[2],
        ctx[3],
        event_amount,
    );
    host.selected_cards = selected_cards.to_vec();
    host.selected_targets = selected_targets.to_vec();
    match vm.execute(module, &mut host, fuel)? {
        rule_vm::ExecResult::Halted { .. } => Ok(VmRunOutcome {
            effects: host.effects,
            paused: None,
        }),
        rule_vm::ExecResult::Paused {
            continuation,
            options,
        } => Ok(VmRunOutcome {
            effects: host.effects,
            paused: Some(VmPause {
                frames: continuation.frames,
                pc: continuation.pc,
                fuel_remaining: continuation.fuel_remaining,
                options,
            }),
        }),
    }
}

/// Resume a paused VM module with the chosen token, returning further effects
/// and any subsequent pause.
pub fn resume_module(
    state: &GameState,
    module: &rule_vm::VerifiedModule,
    skill_id: u32,
    ctx: [u32; 4],
    frames: Vec<u8>,
    pc: u32,
    fuel: u32,
    chosen_token: u32,
) -> Result<VmRunOutcome, rule_vm::VmError> {
    let mut vm = rule_vm::RuleVm::new();
    let mut host = EngineVmHost::new(state, skill_id, ctx[0], ctx[1], ctx[2], ctx[3], 0);
    let cont = rule_vm::Continuation {
        id: 0,
        frames,
        pc,
        fuel_remaining: fuel,
        module_hash: module.logic_hash,
    };
    match vm.resume(module, &mut host, cont, chosen_token)? {
        rule_vm::ExecResult::Halted { .. } => Ok(VmRunOutcome {
            effects: host.effects,
            paused: None,
        }),
        rule_vm::ExecResult::Paused {
            continuation,
            options,
        } => Ok(VmRunOutcome {
            effects: host.effects,
            paused: Some(VmPause {
                frames: continuation.frames,
                pc: continuation.pc,
                fuel_remaining: continuation.fuel_remaining,
                options,
            }),
        }),
    }
}

mod bindings_choose_current_trick_targets_to_nullify;
mod bindings_choose_targets_draw;
mod bindings_judge_make_current_sha_target_unrespondable_on_red;
mod bindings_resolve_ddd_jianshuo;
mod bindings_resolve_potchengpu;
