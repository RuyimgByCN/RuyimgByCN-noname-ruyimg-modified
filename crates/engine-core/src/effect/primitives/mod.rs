//! Primitive effect implementations.
//!
//! `apply_effect` routes requests to functional dispatch modules. Implementations
//! remain grouped by their content package, with smaller skill-specific modules
//! beneath each package. Existing effect matching and handler order are preserved.

use crate::rng::RuleRng;
use crate::state::{
    turn_flag, Card, CardKind, ContinuationKind, EquipSlot, Identity, Phase, Player, Zone,
    ZoneKind, DEFAULT_DRAW_COUNT,
};
use crate::transaction::{JournalEntry, Transaction};
use engine_api::{
    CardDefId, CardId, CardMoveReason, ChoiceId, ChoiceKind, ChoiceOption, ChoiceRequest,
    DomainEvent, MatchOutcome, PlayerId, Suit, TimeoutPolicy, TurnPhaseTag, ZoneId,
};
use rule_api::{
    DrawTargetScope, EffectRequest, QueryRequest, QueryResponse, RelationTag, TargetCardScope,
};

use super::{run_effects, EffectError};

mod guarded_choice;
mod pack_bingshi_1;
mod pack_bingshi_2;
mod pack_bingshi_3;
mod pack_bingshi_4;
mod pack_ddd_1;
mod pack_ddd_2;
mod pack_ddd_3;
mod pack_extra_1;
mod pack_extra_2;
mod pack_huicui;
mod pack_jsrg;
mod pack_mobile;
mod pack_old;
mod pack_refresh;
mod pack_shenhua_1;
mod pack_shenhua_2;
mod pack_shiji;
mod pack_sp_1;
mod pack_sp_2;
mod pack_tw_1;
mod pack_tw_2;
mod pack_tw_3;
mod pack_wandianlunsha;
mod pack_xianding_1;
mod pack_xianding_2;
mod ol_jie_danshou;

pub(crate) use guarded_choice::*;
pub(crate) use pack_bingshi_1::*;
pub(crate) use pack_bingshi_2::*;
pub use pack_bingshi_3::*;
pub(crate) use pack_bingshi_4::*;
pub(crate) use pack_ddd_1::*;
pub(crate) use pack_ddd_2::*;
pub(crate) use pack_ddd_3::*;
pub(crate) use pack_extra_1::*;
pub(crate) use pack_extra_2::*;
pub(crate) use pack_huicui::*;
pub(crate) use pack_jsrg::*;
mod choose_others_add_mark;
pub(crate) use choose_others_add_mark::*;
pub use pack_mobile::*;
pub(crate) use pack_old::*;
pub(crate) use pack_refresh::*;
pub use pack_shenhua_1::*;
pub(crate) use pack_shenhua_2::*;
pub(crate) use pack_shiji::*;
pub(crate) use pack_sp_1::*;
pub(crate) use pack_sp_2::*;
pub use pack_tw_1::*;
pub(crate) use pack_tw_2::*;
pub(crate) use pack_tw_3::*;
pub use pack_wandianlunsha::*;
pub(crate) use pack_xianding_1::*;
pub(crate) use pack_xianding_2::*;
pub(crate) use ol_jie_danshou::*;

/// 卡牌粗类别（0=基本 1=锦囊 2=装备；与 owner_card_category_mask 位一致）。
pub(crate) fn card_category_of(state: &crate::state::GameState, card: CardId) -> Option<u32> {
    let card = state.cards.get(card)?;
    if card.def.is_none() {
        return None;
    }
    let kind = card.kind;
    if kind.is_basic() {
        Some(0)
    } else if kind.is_trick() {
        Some(1)
    } else if kind.is_equip() {
        Some(2)
    } else {
        None
    }
}

/// Apply one primitive effect to the transaction working state.
pub fn apply_effect(
    tx: &mut Transaction,
    req: EffectRequest,
) -> Result<Vec<DomainEvent>, EffectError> {
    match req {
        EffectRequest::ResolveOlJieDanshou { player, skill } => {
            crate::effect::primitives::ol_jie_danshou::resolve_ol_jie_danshou(
                tx, player, skill,
            )
        }
        EffectRequest::ResolveTiqi { .. }
        | EffectRequest::ResolveBaoshu { .. }
        | EffectRequest::ResolveDcbihuo { .. } => {
            Err(EffectError::Illegal("R05 compounds are native-only".into()))
        }
        EffectRequest::DrawCards { .. }
        | EffectRequest::DrawCardsFromSkill { .. }
        | EffectRequest::DrawCardsFromBottom { .. }
        | EffectRequest::SetDrawPhaseFromBottom { .. }
        | EffectRequest::DrawRememberedHandCards { .. }
        | EffectRequest::DiscardRememberedHandCards { .. }
        | EffectRequest::DrawEachBySeat { .. }
        | EffectRequest::DrawThenGiveOne { .. }
        | EffectRequest::DrawThenTakeOne { .. }
        | EffectRequest::TakeOneCard { .. }
        | EffectRequest::TakeOneHandCard { .. }
        | EffectRequest::ChooseOtherHandStoreSkillPile { .. }
        | EffectRequest::GainAllSkillPileCards { .. }
        | EffectRequest::GainOtherSkillPileCards { .. }
        | EffectRequest::TakeOneScopedCard { .. }
        | EffectRequest::TakeTargetHandThenRepeatUntilMax { .. }
        | EffectRequest::ContinueTakeLargerHandUntilMax { .. }
        | EffectRequest::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark { .. }
        | EffectRequest::DrawThenChooseGiveOrDiscardHand { .. }
        | EffectRequest::GainTargetEquipment { .. }
        | EffectRequest::DiscardAllTargetEquipment { .. }
        | EffectRequest::IncreaseCurrentShaTargetDamage { .. }
        | EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp { .. }
        | EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess { .. }
        | EffectRequest::ChooseTargetDrawMaxHpCapThenAddSkill { .. }
        | EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp { .. }
        | EffectRequest::TargetChooseSuitGainOwnerHandDamageIfMismatch { .. }
        | EffectRequest::GainEquipmentFromDrawPile { .. }
        | EffectRequest::GiveAllHandRecoverToOne { .. }
        | EffectRequest::GiveAllHand { .. }
        | EffectRequest::GainCardIfDiscard { .. }
        | EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha { .. }
        | EffectRequest::RevealHandToPlayer { .. }
        | EffectRequest::ChooseOptionalDiscardTargetCard { .. }
        | EffectRequest::RevealHandChooseDiscardColor { .. }
        | EffectRequest::RevealHand { .. }
        | EffectRequest::DrawLinkedPlayers { .. }
        | EffectRequest::RevealTopGainIfColor { .. }
        | EffectRequest::SuppressNonLockedSkillsUntilTurnEnd { .. }
        | EffectRequest::ChooseRecoverTarget { .. }
        | EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory { .. }
        | EffectRequest::ChooseScskuijiDiscard { .. }
        | EffectRequest::ChoosePskuijiDiscard { .. }
        | EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp { .. }
        | EffectRequest::ChooseGiveOneHandEquipOrRecover { .. }
        | EffectRequest::ChooseGiveTwoDifferentColorsOrRecover { .. }
        | EffectRequest::ReplaceSkillWithBoundAfterResolutionIfTargetAlive { .. }
        | EffectRequest::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp { .. }
        | EffectRequest::GainEventCards { .. }
        | EffectRequest::TurnOver { .. }
        | EffectRequest::RestoreCharacterState { .. }
        | EffectRequest::DiscardAllHandEquipJudge { .. }
        | EffectRequest::DamageThenDiscardAllJudge { .. }
        | EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors { .. }
        | EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw { .. }
        | EffectRequest::SkipNextJudgePhase { .. }
        | EffectRequest::GainCardByKind { .. }
        | EffectRequest::GainDiscardCardByKind { .. }
        | EffectRequest::RevealUniformHandChooseDamage { .. }
        | EffectRequest::ChooseGainDiscardCardByKind { .. }
        | EffectRequest::ChooseGainOwnLostDiscardCardThisTurn { .. }
        | EffectRequest::ChooseMoveFieldCardTo { .. }
        | EffectRequest::DrawThenOptionallyDistributeHandCards { .. }
        | EffectRequest::GiveSelectedThenContinueDistinctRecoverAtTwo { .. }
        | EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan { .. }
        | EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver { .. }
        | EffectRequest::ChooseGainCardFromEachOtherThenTurnOver { .. }
        | EffectRequest::ChooseGainCardFromEachOtherReplaceDraw { .. }
        | EffectRequest::GiveSelectedThenRecoverTarget { .. }
        | EffectRequest::SetNonLockedSkillSuppression { .. }
        | EffectRequest::FinalizeTiaojiyanmei { .. }
        | EffectRequest::BeginTiaojiyanmeiDiscard { .. }
        | EffectRequest::DiscardSelectedDrawAllHandBonus { .. }
        | EffectRequest::ChooseOtherYinghun { .. }
        | EffectRequest::AddSkillUntilTurnEnd { .. }
        | EffectRequest::RecordActiveTarget { .. }
        | EffectRequest::ChooseDiscardWeaponOrLoseHp { .. }
        | EffectRequest::ChooseDiscardOneHeOrLoseHp { .. }
        | EffectRequest::JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice { .. }
        | EffectRequest::ChooseSourceConfirmOwnerLoseHp { .. }
        | EffectRequest::ChooseOtherBizheng { .. }
        | EffectRequest::ChooseOtherXiashu { .. }
        | EffectRequest::GiveAllHandThenChooseUseOneOrDamage { .. }
        | EffectRequest::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable { .. }
        | EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants { .. }
        | EffectRequest::EquipWeaponFromDrawPile { .. }
        | EffectRequest::ReplayVirtualCardOnTargets { .. }
        | EffectRequest::DiscardHandEquipCount { .. }
        | EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss { .. }
        | EffectRequest::SkipNextDiscardPhase { .. }
        | EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp { .. }
        | EffectRequest::ChooseGainTargetHandCeilHalfDifference { .. }
        | EffectRequest::ChooseDiscardHandRange { .. } => dispatch_draw_cards::apply_effect(tx, req),
        EffectRequest::ResolveChenglve { .. }
        | EffectRequest::ResolveShicai { .. }
        | EffectRequest::ResolveXiongluan { .. }
        | EffectRequest::ResolveGzbuqu { .. }
        | EffectRequest::ResolveGzbuquRecover { .. }
        | EffectRequest::ResolveSbbiyue { .. }
        | EffectRequest::ResolveSxrmqiewang { .. }
        | EffectRequest::ResolveSxrmqianliu { .. }
        | EffectRequest::ResolveTwjuxiang { .. }
        | EffectRequest::ResolveMbzhuguo { .. }
        | EffectRequest::ResolveMbanxian { .. }
        | EffectRequest::ResolveMbjinzu { .. }
        | EffectRequest::ResolveDingpan { .. }
        | EffectRequest::ResolveTanlu { .. }
        | EffectRequest::ResolveHiddenSkillPile { .. }
        | EffectRequest::ResolveDiliYuheng { .. }
        | EffectRequest::ResolveMassDamageDiscardTurnOver { .. }
        | EffectRequest::ResolveMassDamageDiscardHandEquip { .. }
        | EffectRequest::ResolveMarkParityControl { .. }
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
        | EffectRequest::ResolveReanguo { .. }
        | EffectRequest::ResolveBeizhu { .. }
        | EffectRequest::ResolveMbshanjia { .. }
        | EffectRequest::ResolveDaoji { .. }
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
        | EffectRequest::ResolveTianxiang { .. }
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
        | EffectRequest::ResolveXinfuQinguo { .. }
        | EffectRequest::ResolveRefuhai { .. }
        | EffectRequest::ResolveGzjinfa { .. }
        | EffectRequest::ResolveWnQifeng { .. }
        | EffectRequest::ResolveDcjue { .. }
        | EffectRequest::ResolveRuanji { .. }
        | EffectRequest::ResolveZhengbi { .. }
        | EffectRequest::ResolveFengying { .. }
        | EffectRequest::ResolveQiaojian { .. }
        | EffectRequest::ResolveXicha { .. }
        | EffectRequest::ResolveHuzun { .. }
        | EffectRequest::ResolveZhugemengxue { .. }
        | EffectRequest::ResolveBailingyun { .. }
        | EffectRequest::ResolveDcshaowei { .. }
        | EffectRequest::ResolveDcdichou { .. }
        | EffectRequest::ResolveDcdufeng { .. }
        | EffectRequest::ResolveDcctjiuxian { .. }
        | EffectRequest::ResolveBianyue { .. }
        | EffectRequest::ResolveZhupeilan { .. }
        | EffectRequest::ResolveDcSbZhangxiu { .. }
        | EffectRequest::ResolveDcSbZhouyu { .. }
        | EffectRequest::ResolveDcSbLusu { .. } => dispatch_draw_cards_resolvers::apply_effect(tx, req),
        EffectRequest::ResolveZhangjian { .. }
        | EffectRequest::ResolveZhugeruoxue { .. }
        | EffectRequest::ResolveShoulijian { .. }
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
        | EffectRequest::ResolvePotnigu { .. }
        | EffectRequest::ResolvePotlulian { .. }
        | EffectRequest::ResolvePotdimeng { .. }
        | EffectRequest::ResolvePothaoshi { .. }
        | EffectRequest::ResolveSxrmlanjiao { .. }
        | EffectRequest::ResolveRekuanshi { .. }
        | EffectRequest::ResolveRekuanshiResolve { .. }
        | EffectRequest::ResolveXizhen { .. }
        | EffectRequest::ResolveXuxie { .. }
        | EffectRequest::ResolveXuxieAdd { .. }
        | EffectRequest::ResolveShuangxiong { .. }
        | EffectRequest::ResolveZhijian { .. }
        | EffectRequest::ResolveZhiba { .. }
        | EffectRequest::ResolveZhibaSettle { .. }
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
        | EffectRequest::ResolveZhizheng { .. }
        | EffectRequest::ResolvePottuntian { .. }
        | EffectRequest::ResolvePotjixi { .. }
        | EffectRequest::ResolvePotceduan { .. }
        | EffectRequest::ResolvePottaishici { .. }
        | EffectRequest::ResolvePotjiejie { .. }
        | EffectRequest::ResolvePotqingshi { .. }
        | EffectRequest::ResolvePotcaoshuang { .. }
        | EffectRequest::ResolvePotyuji { .. }
        | EffectRequest::ResolveRehengjiang { .. }
        | EffectRequest::ResolvePotlvyi { .. }
        | EffectRequest::ResolvePotweiyan { .. }
        | EffectRequest::ResolvePotzhouyu { .. }
        | EffectRequest::ResolveZhengsu { .. }
        | EffectRequest::ResolveRechuhai { .. }
        | EffectRequest::ResolveXiongshu { .. }
        | EffectRequest::ResolveTongduo { .. }
        | EffectRequest::ResolveYuanqing { .. }
        | EffectRequest::ResolveShuchen { .. }
        | EffectRequest::ResolveZaoli { .. } => {
            dispatch_resolve_zhangjian_resolvers_pot_r_z::apply_effect(tx, req)
        }
        EffectRequest::ResolveMobiledanshou { .. }
        | EffectRequest::ResolveJsrgchiying { .. }
        | EffectRequest::ResolveMbqingdao { .. }
        | EffectRequest::ResolveMbxiugeng { .. }
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
        | EffectRequest::ResolveDddLiuba { .. }
        | EffectRequest::ResolveDddJianshuo { .. }
        | EffectRequest::ResolveDddGuanning { .. }
        | EffectRequest::ResolveDddCaoshuang { .. }
        | EffectRequest::ResolveDddXuelingyun { .. }
        | EffectRequest::ResolveDddLiuhong { .. }
        | EffectRequest::ResolveDddXiahouxuan { .. }
        | EffectRequest::ResolveDddZhangkai { .. }
        | EffectRequest::ResolveDddLiangxi { .. }
        | EffectRequest::ResolveGwyewuChain { .. }
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
        | EffectRequest::ResolveMbzhuji { .. }
        | EffectRequest::ResolveSpmiaolve { .. }
        | EffectRequest::ResolveSpyingjia { .. }
        | EffectRequest::ResolveMantianguohai { .. }
        | EffectRequest::ResolveDcmanzhi { .. }
        | EffectRequest::ResolveXinxingluan { .. }
        | EffectRequest::ResolveDckuizhen { .. }
        | EffectRequest::ResolveDckuizhenResolve { .. }
        | EffectRequest::ResolveNsdingzhou { .. }
        | EffectRequest::ResolveDcmoyu { .. }
        | EffectRequest::ResolveDcchiying { .. }
        | EffectRequest::ResolveXinkuangfu { .. }
        | EffectRequest::ResolveDcxiangshu { .. }
        | EffectRequest::ResolveGuzheng { .. }
        | EffectRequest::ResolveDrltCongjian { .. }
        | EffectRequest::ResolveNzryFeijun { .. }
        | EffectRequest::ResolveDrltYongsi { .. }
        | EffectRequest::ResolveDrltWeidi { .. }
        | EffectRequest::ResolveLieren { .. }
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
        | EffectRequest::BanUseAgainst { .. }
        | EffectRequest::ResolveJuzhan { .. }
        | EffectRequest::ResolveQizhi { .. }
        | EffectRequest::ResolveJinqu { .. }
        | EffectRequest::ResolveJianxiang { .. }
        | EffectRequest::ResolveMingren { .. }
        | EffectRequest::ResolveKuizhu { .. }
        | EffectRequest::ResolveLijun { .. }
        | EffectRequest::ResolveLiangyin { .. }
        | EffectRequest::ResolveKongsheng { .. }
        | EffectRequest::ResolveMbzhonghui { .. }
        | EffectRequest::ResolveMuzhen { .. }
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
        | EffectRequest::ResolveDcxiangshuLottery { .. }
        | EffectRequest::ResolveDcyingtu { .. } => {
            dispatch_resolve_zhangjian_resolvers::apply_effect(tx, req)
        }
        EffectRequest::GiveSelectedThenTargetMayUseDrawOwner { .. }
        | EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne { .. }
        | EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip { .. }
        | EffectRequest::ContinueYoudishenru { .. }
        | EffectRequest::ContinueWdbFuchou { .. }
        | EffectRequest::ContinueDiaobingqianjiang { .. }
        | EffectRequest::GuozhanUseZhulianBihe { .. }
        | EffectRequest::CancelIncomingCardUse { .. }
        | EffectRequest::ChooseOtherGainOneOfTwoSkills { .. }
        | EffectRequest::ContinueWugu { .. }
        | EffectRequest::ExcludeSavedShaTarget { .. } => {
            dispatch_resolve_zhangjian::apply_effect(tx, req)
        }
        EffectRequest::ResolveTwxiawei { .. }
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
        | EffectRequest::ResolveTwmoukui { .. }
        | EffectRequest::ResolveTwfuman { .. }
        | EffectRequest::ResolveCuijin { .. }
        | EffectRequest::ResolveTwqingkou { .. }
        | EffectRequest::ResolveTwchenxun { .. }
        | EffectRequest::ResolveTwfenxian { .. }
        | EffectRequest::ResolveSpsongshu { .. }
        | EffectRequest::ResolveSpcunsi { .. }
        | EffectRequest::ResolveYuli { .. }
        | EffectRequest::ResolveTingwei { .. }
        | EffectRequest::ResolveXinmingshi { .. }
        | EffectRequest::ResolveXunyi { .. }
        | EffectRequest::ResolveBoming { .. }
        | EffectRequest::ResolveEjian { .. }
        | EffectRequest::ResolveYiyong { .. }
        | EffectRequest::ResolveShanxie { .. }
        | EffectRequest::ResolveSheyi { .. }
        | EffectRequest::ResolveTianyin { .. }
        | EffectRequest::ResolveJiaohua { .. }
        | EffectRequest::ResolveLuanqun { .. }
        | EffectRequest::ResolveGanglieThree { .. }
        | EffectRequest::ResolveZhongyi { .. }
        | EffectRequest::ResolveOldAnxu { .. }
        | EffectRequest::ResolveZhuiyi { .. }
        | EffectRequest::ResolveDecadezhenjun { .. }
        | EffectRequest::ResolveSpXiaoguo { .. }
        | EffectRequest::ResolveXinfuGuolun { .. }
        | EffectRequest::ResolveOldQinqing { .. }
        | EffectRequest::ResolveOldMingjian { .. }
        | EffectRequest::ResolveOldXuanfeng { .. }
        | EffectRequest::ResolveJianrou { .. }
        | EffectRequest::ResolveOldHuisheng { .. }
        | EffectRequest::ResolveOldJiefan { .. }
        | EffectRequest::ResolveChegu { .. }
        | EffectRequest::ResolveJunkguixin { .. }
        | EffectRequest::ResolveYaoChenwei { .. }
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
        | EffectRequest::ResolveBeigeConfigured { .. }
        | EffectRequest::ResolveDccunsi { .. }
        | EffectRequest::ResolveOllinjie { .. }
        | EffectRequest::ResolveDcbingji { .. }
        | EffectRequest::ResolvePxqixian { .. }
        | EffectRequest::ResolveQuanjia { .. }
        | EffectRequest::ResolveDuwang { .. }
        | EffectRequest::ResolveJsrgdaoren { .. }
        | EffectRequest::ResolveQianlong { .. }
        | EffectRequest::ResolveJsrgzuozhan { .. }
        | EffectRequest::ResolveJsrgzuozhanGain { .. }
        | EffectRequest::ResolveTwmouzhi { .. }
        | EffectRequest::ResolveTwdaigui { .. }
        | EffectRequest::ResolveTwmitu { .. }
        | EffectRequest::ResolveQiantun { .. }
        | EffectRequest::ResolveYangming { .. }
        | EffectRequest::ResolveCanyun { .. }
        | EffectRequest::ResolveChuanyun { .. }
        | EffectRequest::ResolveQinmian { .. }
        | EffectRequest::ResolveBiaozhao2 { .. }
        | EffectRequest::ResolveChumei { .. }
        | EffectRequest::ResolveShuaiyan { .. }
        | EffectRequest::ResolveMbbeiming { .. }
        | EffectRequest::ResolveGaoyuan { .. }
        | EffectRequest::ResolveDuoduan { .. }
        | EffectRequest::ResolveDcanxu { .. }
        | EffectRequest::ResolveChulao { .. }
        | EffectRequest::ResolveMeiyong { .. }
        | EffectRequest::ResolveTwxiongjun { .. }
        | EffectRequest::ResolveDcqiaolve { .. }
        | EffectRequest::ResolveDingce { .. }
        | EffectRequest::ResolveYixiang { .. }
        | EffectRequest::ResolveXinfuQiai { .. }
        | EffectRequest::ResolveOlwenren { .. }
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
        | EffectRequest::ResolveQiangyong { .. }
        | EffectRequest::ResolveMbzhenann { .. }
        | EffectRequest::ResolveFriendbiwei { .. }
        | EffectRequest::ResolveZhengjian { .. }
        | EffectRequest::ResolveReliangyin { .. }
        | EffectRequest::ResolveZhongyan { .. }
        | EffectRequest::ResolveDcmoukui { .. }
        | EffectRequest::ResolveDcniji { .. }
        | EffectRequest::ResolveHengwu { .. }
        | EffectRequest::ResolveBiaozhao3 { .. }
        | EffectRequest::ResolveJsrgchaozheng { .. } => {
            dispatch_resolve_twxiawei_resolvers::apply_effect(tx, req)
        }
        EffectRequest::SetSkillContentFlag { .. }
        | EffectRequest::DiscardEquipmentSlot { .. }
        | EffectRequest::StoreSelectedInSkillPile { .. }
        | EffectRequest::GainTopCardToSkillPile { .. }
        | EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill { .. }
        | EffectRequest::GainOneHandFromSelectedTargets { .. }
        | EffectRequest::ChooseRepoluTargetsDraw { .. }
        | EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank { .. }
        | EffectRequest::EndMatchForCampIfSkillPileAtLeast { .. }
        | EffectRequest::ReplaceDrawWithDistinctSuitSelection { .. }
        | EffectRequest::InspectTargetHandChooseHeartDiscardOrTop { .. }
        | EffectRequest::ChooseAllLoseHpOrRecover { .. }
        | EffectRequest::ChooseLinkOther { .. }
        | EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay { .. }
        | EffectRequest::DrawForLifetimeCardActionDivisors { .. }
        | EffectRequest::GainDrawCardsForLifetimeCardActionDivisors { .. }
        | EffectRequest::ChooseAddMissingSkill { .. }
        | EffectRequest::AddFirstMissingSkill { .. }
        | EffectRequest::RecordCurrentShaAllTargets { .. }
        | EffectRequest::ChooseGiveOrDiscardHand { .. }
        | EffectRequest::ChooseDiscardHandEquipThenRecoverTarget { .. }
        | EffectRequest::JudgeRecoverOnSuit { .. }
        | EffectRequest::JudgeRecoverOtherOnSuit { .. }
        | EffectRequest::ForceOthersUseShaOnNearestOrLoseHp { .. }
        | EffectRequest::JudgeDamageOtherOnSuit { .. }
        | EffectRequest::JudgeSourceDiscardTwoHandOrDamage { .. }
        | EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo { .. }
        | EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw { .. }
        | EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw { .. }
        | EffectRequest::DistributeTopTwoCards { .. }
        | EffectRequest::RecastCards { .. }
        | EffectRequest::DisableEquipSlots { .. }
        | EffectRequest::MoveEquipmentThenDraw { .. }
        | EffectRequest::DiscardEquipmentPairWithArmor { .. }
        | EffectRequest::DiscardTargetEquipmentGainMount { .. }
        | EffectRequest::DiscardTargetEquipmentDrawWeaponRange { .. }
        | EffectRequest::ChooseTargetRevealHandDamageIfSha { .. }
        | EffectRequest::ChooseOtherGiveOneHandEquip { .. }
        | EffectRequest::ChooseGiveOrRecastOneHand { .. }
        | EffectRequest::ChooseRevealHandCardBindCategoryTarget { .. }
        | EffectRequest::RecoverToMaxThenOptionalExchangeSkill { .. }
        | EffectRequest::DiscardTargetCardGainMount { .. }
        | EffectRequest::ChooseDiscardTargetCard { .. }
        | EffectRequest::JudgePreventDamageLoseMaxHpOnNonHeart { .. }
        | EffectRequest::ChooseGainTopCardsBySuit { .. }
        | EffectRequest::ChooseOtherDrawByHandCount { .. }
        | EffectRequest::ChooseDiscardOneThenGainTargetEquipment { .. }
        | EffectRequest::ChooseTargetDraw { .. }
        | EffectRequest::ChooseTargetsDraw { .. }
        | EffectRequest::ChooseTargetsDamage { .. }
        | EffectRequest::ChooseOptionalTargetsDraw { .. }
        | EffectRequest::RevealTargetHandShaOrDiscardBind { .. }
        | EffectRequest::ChooseTargetDrawToMaxHpCap { .. }
        | EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { .. }
        | EffectRequest::DrawByPlayPhaseCardCategoryCount { .. }
        | EffectRequest::RefundCurrentShaUseAndDrawIfRed { .. }
        | EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed { .. }
        | EffectRequest::ChooseTargetGainMaxHp { .. }
        | EffectRequest::GainTargetHandCard { .. }
        | EffectRequest::ChooseClanNotMaxHandLimitGain { .. }
        | EffectRequest::ExtraResolveCurrentTrick { .. }
        | EffectRequest::PlaceHandCardToExpansion { .. }
        | EffectRequest::GainFromSkillPile { .. }
        | EffectRequest::DiscardSkillPile { .. }
        | EffectRequest::PlaceTopDeckToExpansion { .. }
        | EffectRequest::ChooseOneFromSkillPileDiscard { .. }
        | EffectRequest::GainWholeSkillPile { .. }
        | EffectRequest::RearrangePileToDeckTop { .. }
        | EffectRequest::ChooseHandCardsToSkillPile { .. }
        | EffectRequest::ResetLinkedSelectedTargets { .. }
        | EffectRequest::CommitGaoyuanRedirect { .. }
        | EffectRequest::PlaceHandCardsToDeckTop { .. }
        | EffectRequest::ChooseTargetHandCardToPile { .. }
        | EffectRequest::ChooseOtherAddMark { .. }
        | EffectRequest::StartDebateAllOthers { .. }
        | EffectRequest::DiscardBasicDrawEqual { .. }
        | EffectRequest::StartDebateTargets { .. } => {
            dispatch_resolve_twxiawei::apply_effect(tx, req)
        }
        EffectRequest::PinDebateOpinions { .. }
        | EffectRequest::AlignDebateOpinions { .. }
        | EffectRequest::GainShadowCards { .. }
        | EffectRequest::ChangeGroup { .. }
        | EffectRequest::DistributeShadowCards { .. }
        | EffectRequest::RegisterTurnEndDebate { .. }
        | EffectRequest::LoseGainedSkill { .. }
        | EffectRequest::PlaceXumou { .. }
        | EffectRequest::PlaceDeckTopXumou { .. }
        | EffectRequest::DiscardOneOwnXumou { .. }
        | EffectRequest::PlaceDeckTopExpansion { .. }
        | EffectRequest::CityConsume { .. }
        | EffectRequest::QueueExtraTurn { .. }
        | EffectRequest::QueueExtraPhases { .. }
        | EffectRequest::SkipRemainingPhases { .. }
        | EffectRequest::QueueOpeningExtraPlayPhase { .. }
        | EffectRequest::LoseHpDoubleCurrentDamageThenReplace { .. }
        | EffectRequest::BeginTurn { .. }
        | EffectRequest::FinishTurnBegin { .. }
        | EffectRequest::AdvanceOpeningExtraPlay { .. }
        | EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage { .. }
        | EffectRequest::IncreaseCurrentShaBaseDamage { .. }
        | EffectRequest::IncreaseCurrentCardBaseDamage { .. }
        | EffectRequest::RefundCurrentShaUse { .. }
        | EffectRequest::GuanxingTop { .. }
        | EffectRequest::GuanxingTopByFaluMarks { .. }
        | EffectRequest::GiveSelectedThenRecoverSelf { .. }
        | EffectRequest::ContinueChooseCompareTargetHandCard { .. }
        | EffectRequest::ContinueChooseCompareTargetHandCardSecondFailureSeatDamage { .. }
        | EffectRequest::RecordCompareFailureSeatDamage { .. }
        | EffectRequest::DealDamageEachInOrder { .. }
        | EffectRequest::RandomizeCurrentHandCompareOpponent { .. }
        | EffectRequest::EmitHandCompareFinished { .. }
        | EffectRequest::GainHighestRankShaFromEventCards { .. }
        | EffectRequest::GainHandCompareResultCard { .. }
        | EffectRequest::AddCardColorLockSkill { .. }
        | EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { .. }
        | EffectRequest::UseVirtualShaDrawIfNoDamage { .. }
        | EffectRequest::DiscardSelectedRecordUniqueCategoryTarget { .. }
        | EffectRequest::FinishLinkIfTargetWounded { .. }
        | EffectRequest::DrawThenGiveExactRecordPhaseTarget { .. }
        | EffectRequest::ClearPhaseGiftTarget { .. }
        | EffectRequest::EmitShaResolved { .. }
        | EffectRequest::EmitCardResolved { .. }
        | EffectRequest::SkipCurrentPhase { .. }
        | EffectRequest::RevealUniformHandDrawSelfChooseOthers { .. }
        | EffectRequest::ModifyCurrentDamage { .. }
        | EffectRequest::GiveAllEquipmentChooseRecoverOrDamage { .. }
        | EffectRequest::FinishOpeningExtraPlayShaGain { .. }
        | EffectRequest::RevealTargetHandGainShanArmorOrDrawTwo { .. }
        | EffectRequest::IncreaseCurrentDrawCount { .. }
        | EffectRequest::SkipJudgeAndDrawPhases { .. }
        | EffectRequest::SkipNextPlayPhase { .. }
        | EffectRequest::EnableEquipSlot { .. }
        | EffectRequest::DrawSelectedTargets { .. }
        | EffectRequest::DamageThenDrawSelectedTargets { .. }
        | EffectRequest::FinishCurrentPhaseAbortResolution { .. }
        | EffectRequest::RecoverSelectedTargets { .. }
        | EffectRequest::RecordSelectedPlayPhaseCategory { .. }
        | EffectRequest::RecoverIfMinCurrentHp { .. }
        | EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors { .. }
        | EffectRequest::ContinueChooseGainOneFromDamageTargetAndNeighbors { .. }
        | EffectRequest::ContinueIncreaseBoundDamage { .. }
        | EffectRequest::AddCardUsePolicySkill { .. }
        | EffectRequest::DiscardTargetCards { .. }
        | EffectRequest::RemoveSkillByDef { .. }
        | EffectRequest::RevealTopGainNonBasicOrSwapBasic { .. }
        | EffectRequest::PreventDamage { .. }
        | EffectRequest::MoveCards { .. }
        | EffectRequest::MoveCardsWithReason { .. }
        | EffectRequest::GiveCards { .. }
        | EffectRequest::UseCard { .. }
        | EffectRequest::UseFieldCard { .. }
        | EffectRequest::DealDamage { .. }
        | EffectRequest::DealDamageBanActiveIfDying { .. }
        | EffectRequest::SpjungongDiscardOrLoseThenSha { .. }
        | EffectRequest::BanActiveUntilTurnEnd { .. }
        | EffectRequest::SpwanweiActive { .. }
        | EffectRequest::SpwanweiDying { .. }
        | EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying { .. }
        | EffectRequest::DealShaDamage { .. }
        | EffectRequest::DealCardDamage { .. }
        | EffectRequest::RecoverHp { .. }
        | EffectRequest::PreventRecovery { .. }
        | EffectRequest::ChangeMaxHp { .. }
        | EffectRequest::AddSkill { .. }
        | EffectRequest::AddBoundSkill { .. }
        | EffectRequest::RemoveSkill { .. }
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
        | EffectRequest::JianzhiPhaseEnd { .. }
        | EffectRequest::AddMark { .. }
        | EffectRequest::RemoveMark { .. }
        | EffectRequest::RevealCards { .. }
        | EffectRequest::RevealHandDiscardKind { .. }
        | EffectRequest::RequestChoice { .. }
        | EffectRequest::RequestDyingSave { .. }
        | EffectRequest::ContinueDyingSave { .. }
        | EffectRequest::EnterDying { .. }
        | EffectRequest::KillPlayer { .. }
        | EffectRequest::SetTurnState { .. }
        | EffectRequest::AdvanceTurn
        | EffectRequest::EmitRuleEvent { .. }
        | EffectRequest::EmitDebateResolved { .. }
        | EffectRequest::DiscardFromHand { .. }
        | EffectRequest::DiscardFromHandEquip { .. }
        | EffectRequest::AdjustHandTo { .. }
        | EffectRequest::DiscardCards { .. }
        | EffectRequest::DiscardAllHandAndEquip { .. }
        | EffectRequest::RemoveAllSkills { .. }
        | EffectRequest::JudgeGainEquipmentByColor { .. }
        | EffectRequest::JudgeCancelCurrentShaTargetOnRed { .. }
        | EffectRequest::MakeCurrentShaTargetUnrespondable { .. }
        | EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed { .. }
        | EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner { .. } => {
            dispatch_pin_debate_opinions::apply_effect(tx, req)
        }
        EffectRequest::ChooseGroup { .. }
        | EffectRequest::ChooseGainOtherSkill { .. }
        | EffectRequest::ChoosePlaceHandXumou { .. }
        | EffectRequest::ChooseOthersInsertExtraTurn { .. }
        | EffectRequest::ChooseOthersAddMark { .. }
        | EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits { .. }
        | EffectRequest::ChooseAddCurrentShaTarget { .. }
        | EffectRequest::ChooseAddCurrentShaTargetWithinDistance { .. }
        | EffectRequest::CommitAddCurrentShaTarget { .. }
        | EffectRequest::ChooseStoreHandShaInSkillPile { .. }
        | EffectRequest::ChooseSpendSkillPileForDyingJiu { .. }
        | EffectRequest::CommitStoreHandShaInSkillPile { .. }
        | EffectRequest::CommitSpendSkillPileForDyingJiu { .. }
        | EffectRequest::ChooseCompareSelectedHandCard { .. }
        | EffectRequest::ChooseCompareSelectedHandCardSecondFailureSeatDamage { .. }
        | EffectRequest::ChooseHandCompare { .. }
        | EffectRequest::ChooseHandCompareGrantTargetOnWin { .. }
        | EffectRequest::ChooseHandCompareBetween { .. }
        | EffectRequest::ChooseOtherGiveThenRecover { .. }
        | EffectRequest::ChooseOtherDamage { .. }
        | EffectRequest::ChooseBoundSourceAttackRangeDamage { .. }
        | EffectRequest::ChooseOtherDiscardTargetCard { .. }
        | EffectRequest::ChooseDrawOrDiscardTargetCard { .. }
        | EffectRequest::ChooseSeatPathDamage { .. }
        | EffectRequest::ChooseAddOrRemoveCurrentCardTarget { .. }
        | EffectRequest::ChooseAddSkillBoundCardTarget { .. }
        | EffectRequest::CommitAddSkillBoundCardTarget { .. }
        | EffectRequest::ChooseDiscardMatchingEquipColorAddSkill { .. }
        | EffectRequest::ChooseCurrentCardUseEnhancements { .. }
        | EffectRequest::ChooseDrawOrDiscardAddSkill { .. }
        | EffectRequest::ChooseDiscardedBlackBasicAsBingliang { .. }
        | EffectRequest::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage { .. }
        | EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded { .. }
        | EffectRequest::ChooseRevealOtherHandCardRememberCategory { .. }
        | EffectRequest::ChooseGainResolvedShaOrShanThenOptionalUseSha { .. }
        | EffectRequest::ChooseGainAllResolvedShaShanThenColorRewards { .. }
        | EffectRequest::ChooseUseShaAfterResolvedGift { .. }
        | EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage { .. }
        | EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw { .. }
        | EffectRequest::ChooseCardUserOrAddTarget { .. }
        | EffectRequest::CommitTransferSavedCardUser { .. }
        | EffectRequest::ChooseDiscardEquipmentReduceCurrentDamage { .. }
        | EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage { .. }
        | EffectRequest::ChooseDiscardDamageEndpointCardIncreaseIfSuit { .. }
        | EffectRequest::ChooseMoveTargetEquipmentOrDraw { .. }
        | EffectRequest::ChooseMoveTargetEquipmentAndOrDraw { .. }
        | EffectRequest::ChooseDrawViewTwoGainOrMoveEquipment { .. }
        | EffectRequest::ChooseRecoverOwnerOrDamageByGivenEquipment { .. }
        | EffectRequest::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage { .. }
        | EffectRequest::ChooseTargetDiscardDistanceHandOrIncreaseDamage { .. }
        | EffectRequest::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth { .. }
        | EffectRequest::ChooseDrawOrRecastAddSkill { .. }
        | EffectRequest::ChooseGainShaForOpeningExtraPlay { .. }
        | EffectRequest::ChooseHandCountDifferentTargetDrawOrRandomDiscard { .. }
        | EffectRequest::ChooseOtherDrawOrRecover { .. }
        | EffectRequest::ChooseOtherRecover { .. }
        | EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill { .. }
        | EffectRequest::ChooseDiscardHandEquipThenRecover { .. }
        | EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw { .. }
        | EffectRequest::ChooseSelfDiscardOrTargetDrawAfterTargetDiscard { .. }
        | EffectRequest::ChoosePreventDamageOrLoseMaxHpIncreaseDamage { .. }
        | EffectRequest::ChooseDiscardHandAddSkillByColor { .. }
        | EffectRequest::CommitDamage { .. }
        | EffectRequest::CommitDamageBanActiveIfDying { .. }
        | EffectRequest::CommitLoseHpThenDamageOtherLoseMaxHpIfDying { .. }
        | EffectRequest::CommitRecovery { .. }
        | EffectRequest::ChooseDiscardOneHandOrEquip { .. }
        | EffectRequest::ChooseDiscardBasicThenTargetDiscardEquipOrDamage { .. }
        | EffectRequest::ChooseTargetLoseHpDiscardHandShaIfBasic { .. } => {
            dispatch_pin_debate_opinions_choose_commit::apply_effect(tx, req)
        }
        EffectRequest::ResolveTurnEndDebates { .. }
        | EffectRequest::ResolvePotheyun { .. }
        | EffectRequest::ResolveTwjieqiu { .. }
        | EffectRequest::ResolveJsrgsheju { .. }
        | EffectRequest::ResolveYoudi { .. }
        | EffectRequest::ResolveKuangfu { .. }
        | EffectRequest::ResolveShanjia { .. }
        | EffectRequest::ResolveNewYajiao { .. }
        | EffectRequest::ResolveDingpin { .. }
        | EffectRequest::ResolveSpxizhan { .. } => {
            dispatch_pin_debate_opinions_resolvers::apply_effect(tx, req)
        }
        EffectRequest::ChooseCurrentShaTargetDiscardRedirectToOwner { .. }
        | EffectRequest::ChooseDiscardRedirectCurrentSha { .. }
        | EffectRequest::ChooseTargetGainBottomCard { .. }
        | EffectRequest::JudgeBlackGainRepeat { .. }
        | EffectRequest::JudgeBlackGainRepeatProtectHandLimit { .. }
        | EffectRequest::JudgeSameColorGainRepeat { .. }
        | EffectRequest::ChooseCurrentTrickTargetsToNullify { .. }
        | EffectRequest::UseVirtualJuedouSecondOnFirst { .. }
        | EffectRequest::JudgeSetSkillMarkToRank { .. }
        | EffectRequest::ChooseDiscardOneFromEachLinked { .. }
        | EffectRequest::DisableSkillByDefUntilTurnEnd { .. }
        | EffectRequest::ChooseDiscardTargetCardResolveCurrentSha { .. }
        | EffectRequest::ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus { .. }
        | EffectRequest::ResolveDcqiaomeng { .. }
        | EffectRequest::ResolveNewReluoyi { .. }
        | EffectRequest::ResolveRemoshi { .. }
        | EffectRequest::ResolveXinyaoming { .. }
        | EffectRequest::ResolveRehuaiyi { .. }
        | EffectRequest::ChooseDrawOrRecover { .. }
        | EffectRequest::ChooseUseVirtualSha { .. }
        | EffectRequest::ChooseUseVirtualShaWithBonus { .. }
        | EffectRequest::ChooseUseVirtualShaOrDraw { .. }
        | EffectRequest::UseVirtualCardOn { .. }
        | EffectRequest::UseVirtualCardOnSelectedTargets { .. }
        | EffectRequest::ChooseTargetReclaimEquipmentDrawIfRangeReduced { .. }
        | EffectRequest::CommitTargetReclaimEquipmentDrawIfRangeReduced { .. }
        | EffectRequest::FinishTargetReclaimEquipmentDrawIfRangeReduced { .. }
        | EffectRequest::ChooseUseVirtualCard { .. }
        | EffectRequest::ChooseUseVirtualNanman { .. }
        | EffectRequest::ResolveQiluan { .. }
        | EffectRequest::DiscardAllOwnJudge { .. }
        | EffectRequest::StoreEventCardInSkillPile { .. }
        | EffectRequest::FinishPhaseEndVirtualCard
        | EffectRequest::EmitDuelResolved { .. }
        | EffectRequest::EmitJudgmentResolved { .. }
        | EffectRequest::RevealDiscardSuitGainOrUse { .. }
        | EffectRequest::ChooseDiscardOrFireDamageSkipPhases { .. }
        | EffectRequest::TakeFromHand { .. }
        | EffectRequest::CardResponded { .. }
        | EffectRequest::ShaDodged { .. }
        | EffectRequest::RemoveCardsByKind { .. } => {
            dispatch_choose_current_sha_target_discard_redirect_to_owner::apply_effect(tx, req)
        }
    }
}

mod dispatch_choose_current_sha_target_discard_redirect_to_owner;
mod dispatch_draw_cards;
mod dispatch_draw_cards_resolvers;
mod dispatch_pin_debate_opinions;
mod dispatch_pin_debate_opinions_choose_commit;
mod dispatch_pin_debate_opinions_resolvers;
mod dispatch_resolve_twxiawei;
mod dispatch_resolve_twxiawei_resolvers;
mod dispatch_resolve_zhangjian;
mod dispatch_resolve_zhangjian_resolvers;
mod dispatch_resolve_zhangjian_resolvers_pot_r_z;
