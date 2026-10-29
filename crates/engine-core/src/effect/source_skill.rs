//! 效果请求的来源技能：嵌套卡牌管线据此保留技能出处，抑制同技能递归。

use super::*;

/// 本请求的效果来源技能编号；不带技能出处的请求沿用调用方传入的来源（`inherited`）。
pub(super) fn effect_source_skill_of(req: &EffectRequest, inherited: u32) -> u32 {
    match req {
        EffectRequest::DrawCardsFromSkill { skill, .. }
        | EffectRequest::ResolveQixing { skill, .. }
        | EffectRequest::ResolveQixingWeather { skill, .. }
        | EffectRequest::ResolveNzryLongnu { skill, .. }
        | EffectRequest::ChooseLinkOther { skill, .. }
        | EffectRequest::ResolveDrltPoxi { skill, .. }
        | EffectRequest::ResolveDrltJieying { skill, .. }
        | EffectRequest::ResolveDrltDuorui { skill, .. }
        | EffectRequest::ResolveDrltZhiti { skill, .. }
        | EffectRequest::ResolveShenSimayi { skill, .. }
        | EffectRequest::ResolveBeige { skill, .. }
        | EffectRequest::ResolveBeigeConfigured { skill, .. } => skill.raw(),
        EffectRequest::DrawEachBySeat { skill, .. }
        | EffectRequest::DrawSelectedTargets { skill, .. }
        | EffectRequest::ResolveOlwenren { skill, .. }
        | EffectRequest::RecoverSelectedTargets { skill, .. }
        | EffectRequest::RecordSelectedPlayPhaseCategory { skill, .. }
        | EffectRequest::DrawThenGiveOne { skill, .. }
        | EffectRequest::DrawThenTakeOne { skill, .. }
        | EffectRequest::MoveEquipmentThenDraw { skill, .. }
        | EffectRequest::ChooseUseVirtualShaOrDraw { skill, .. }
        | EffectRequest::DiscardEquipmentPairWithArmor { skill, .. }
        | EffectRequest::ChooseOtherDrawByHandCount { skill, .. }
        | EffectRequest::ChooseOtherDrawOrRecover { skill, .. }
        | EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill { skill, .. }
        | EffectRequest::ChooseDiscardHandEquipThenRecover { skill, .. }
        | EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory { skill, .. }
        | EffectRequest::ChooseScskuijiDiscard { skill, .. }
        | EffectRequest::ChoosePskuijiDiscard { skill, .. }
        | EffectRequest::ChooseTargetDraw { skill, .. }
        | EffectRequest::ChooseTargetsDraw { skill, .. }
        | EffectRequest::ChooseTargetsDamage { skill, .. }
        | EffectRequest::ChooseOptionalTargetsDraw { skill, .. }
        | EffectRequest::RevealTargetHandShaOrDiscardBind { skill, .. }
        | EffectRequest::DrawRememberedHandCards { skill, .. }
        | EffectRequest::DiscardRememberedHandCards { skill, .. }
        | EffectRequest::DiscardTargetEquipmentDrawWeaponRange { skill, .. }
        | EffectRequest::ChooseTargetRevealHandDamageIfSha { skill, .. }
        | EffectRequest::ChooseOtherGiveOneHandEquip { skill, .. }
        | EffectRequest::ChooseGiveOrRecastOneHand { skill, .. }
        | EffectRequest::ChooseRevealHandCardBindCategoryTarget { skill, .. }
        | EffectRequest::ChooseTargetDrawToMaxHpCap { skill, .. }
        | EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp { skill, .. }
        | EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess { skill, .. }
        | EffectRequest::ChooseTargetDrawMaxHpCapThenAddSkill { skill, .. }
        | EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
            skill, ..
        }
        | EffectRequest::TargetChooseSuitGainOwnerHandDamageIfMismatch { skill, .. }
        | EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo { skill, .. }
        | EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw { skill, .. }
        | EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw { skill, .. }
        | EffectRequest::ResolveDcqiaomeng { skill, .. }
        | EffectRequest::ResolveNewReluoyi { skill, .. }
        | EffectRequest::ResolveRemoshi { skill, .. }
        | EffectRequest::ResolveXinyaoming { skill, .. }
        | EffectRequest::ResolveRehuaiyi { skill, .. }
        | EffectRequest::ResolveReanguo { skill, .. }
        | EffectRequest::ResolveBeizhu { skill, .. }
        | EffectRequest::ResolveMbshanjia { skill, .. }
        | EffectRequest::ResolveDaoji { skill, .. }
        | EffectRequest::GiveAllHandThenChooseUseOneOrDamage { skill, .. }
        | EffectRequest::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
            skill,
            ..
        }
        | EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants { skill, .. }
        | EffectRequest::ResolveTwfupan { skill, .. }
        | EffectRequest::ResolveTwtanfeng { skill, .. }
        | EffectRequest::ResolveTwzhibian { skill, .. }
        | EffectRequest::ResolveTwyuyan { skill, .. }
        | EffectRequest::ResolveTwguoyi { skill, .. }
        | EffectRequest::ReplayVirtualCardOnTargets { skill, .. }
        | EffectRequest::ResolveTwchuhai { skill, .. }
        | EffectRequest::ResolveTwyizhu { skill, .. }
        | EffectRequest::ResolveTwluanchou { skill, .. }
        | EffectRequest::ResolveTianxiang { skill, .. }
        | EffectRequest::ResolveRetianxiang { skill, .. }
        | EffectRequest::ResolveLianzhou { skill, .. }
        | EffectRequest::ResolveStddechong { skill, .. }
        | EffectRequest::ChooseOtherAddMark { skill, .. }
        | EffectRequest::ResolveJsrgchaozheng { skill, .. }
        | EffectRequest::StartDebateAllOthers { skill, .. }
        | EffectRequest::DiscardBasicDrawEqual { skill, .. }
        | EffectRequest::StartDebateTargets { skill, .. }
        | EffectRequest::ResolveJsrgsheju { skill, .. }
        | EffectRequest::DiscardHandEquipCount { skill, .. }
        | EffectRequest::ResolveJieliangGain { skill, .. }
        | EffectRequest::ResolveStdjujian { skill, .. }
        | EffectRequest::ResolveLulve { skill, .. }
        | EffectRequest::ResolveDctingxian { skill, .. }
        | EffectRequest::ResolveHfjieying { skill, .. }
        | EffectRequest::ResolveDcnuchen { skill, .. }
        | EffectRequest::ResolveDcdanji { skill, .. }
        | EffectRequest::ResolveDczhiheng { skill, .. }
        | EffectRequest::ResolveStdgaobian { skill, .. }
        | EffectRequest::ResolveSpjiedao { skill, .. }
        | EffectRequest::ResolveSibian { skill, .. }
        | EffectRequest::ResolveSpfengyin { skill, .. }
        | EffectRequest::ResolveXinfuLianpian { skill, .. }
        | EffectRequest::ResolveGongjian { skill, .. }
        | EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss { skill, .. }
        | EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp { skill, .. }
        | EffectRequest::ResolveRefuhai { skill, .. }
        | EffectRequest::ChooseGainTargetHandCeilHalfDifference { skill, .. }
        | EffectRequest::ResolveGzjinfa { skill, .. }
        | EffectRequest::ResolveWnQifeng { skill, .. }
        | EffectRequest::ResolveDcjue { skill, .. }
        | EffectRequest::ResolveRuanji { skill, .. }
        | EffectRequest::ResolveZhengbi { skill, .. }
        | EffectRequest::ResolveFengying { skill, .. }
        | EffectRequest::ResolveQiaojian { skill, .. }
        | EffectRequest::ResolveXicha { skill, .. }
        | EffectRequest::ResolveHuzun { skill, .. }
        | EffectRequest::ResolveZhugemengxue { skill, .. }
        | EffectRequest::ResolveBailingyun { skill, .. }
        | EffectRequest::ResolveDcshaowei { skill, .. }
        | EffectRequest::ResolveDcdichou { skill, .. }
        | EffectRequest::ResolveDcdufeng { skill, .. }
        | EffectRequest::ChooseDiscardHandRange { skill, .. }
        | EffectRequest::ResolveDcctjiuxian { skill, .. }
        | EffectRequest::ResolveBianyue { skill, .. }
        | EffectRequest::ResolveZhupeilan { skill, .. }
        | EffectRequest::ResolveDcSbZhangxiu { skill, .. }
        | EffectRequest::ResolveDcSbZhouyu { skill, .. }
        | EffectRequest::ResolveDcSbLusu { skill, .. }
        | EffectRequest::ResolveZhangjian { skill, .. }
        | EffectRequest::ResolveZhugeruoxue { skill, .. }
        | EffectRequest::ResolveCaoyi { skill, .. }
        | EffectRequest::ResolveMalingli { skill, .. }
        | EffectRequest::ResolveNsshuaiyan { skill, .. }
        | EffectRequest::ResolveWdbHanlong { skill, .. }
        | EffectRequest::ResolveWdbLiufu { skill, .. }
        | EffectRequest::ResolveWdbLiuyan { skill, .. }
        | EffectRequest::ResolveWdbLiuzan { skill, .. }
        | EffectRequest::ResolveWdbTianyu { skill, .. }
        | EffectRequest::ResolveWdbXizhenxihong { skill, .. }
        | EffectRequest::ResolveWdbYangyi { skill, .. }
        | EffectRequest::ResolveWdbZaozhiRenjun { skill, .. }
        | EffectRequest::ResolveDddXianxi { skill, .. }
        | EffectRequest::ResolveDddXiaheng { skill, .. }
        | EffectRequest::ResolveDddLiuchong { skill, .. }
        | EffectRequest::ResolveDddDuanbing { skill, .. }
        | EffectRequest::ResolveDddWuzhi { skill, .. }
        | EffectRequest::ResolveDddXujing { skill, .. }
        | EffectRequest::ResolveDddCaomao { skill, .. }
        | EffectRequest::ResolveDddXinxianying { skill, .. }
        | EffectRequest::ResolveDddXianglang { skill, .. }
        | EffectRequest::ResolveDddYujin { skill, .. }
        | EffectRequest::ResolveDddLiuye { skill, .. }
        | EffectRequest::ResolveDddBaosanniang { skill, .. }
        | EffectRequest::ResolveDddZhenji { skill, .. }
        | EffectRequest::ResolveDddZhaoang { skill, .. }
        | EffectRequest::ResolveDddLiuba { skill, .. }
        | EffectRequest::ResolveDddJianshuo { skill, .. }
        | EffectRequest::ResolveDddGuanning { skill, .. }
        | EffectRequest::ResolveDddCaoshuang { skill, .. }
        | EffectRequest::ResolveDddXuelingyun { skill, .. }
        | EffectRequest::ResolveDddLiuhong { skill, .. }
        | EffectRequest::ResolveDddXiahouxuan { skill, .. }
        | EffectRequest::ResolveDddZhangkai { skill, .. }
        | EffectRequest::ResolveDddLiangxi { skill, .. }
        | EffectRequest::ResolveChenglve { skill, .. }
        | EffectRequest::ResolveShicai { skill, .. }
        | EffectRequest::ResolveXiongluan { skill, .. }
        | EffectRequest::ResolveTwjuxiang { skill, .. }
        | EffectRequest::ResolveGzbuqu { skill, .. }
        | EffectRequest::ResolveGzbuquRecover { skill, .. }
        | EffectRequest::ResolveSbbiyue { skill, .. }
        | EffectRequest::ResolveSxrmqiewang { skill, .. }
        | EffectRequest::ResolveSxrmqianliu { skill, .. }
        | EffectRequest::GiveSelectedThenTargetMayUseDrawOwner { skill, .. }
        | EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne { skill, .. }
        | EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
            skill,
            ..
        }
        | EffectRequest::ResolveMobiledanshou { skill, .. }
        | EffectRequest::ResolveJsrgchiying { skill, .. }
        | EffectRequest::ResolveMbqingdao { skill, .. }
        | EffectRequest::ResolveMbxiugeng { skill, .. }
        | EffectRequest::ResolveMbchenshe { skill, .. }
        | EffectRequest::ResolveMbquanchong { skill, .. }
        | EffectRequest::ResolveMbrenxing { skill, .. }
        | EffectRequest::ResolveMbchizhang { skill, .. }
        | EffectRequest::ResolveMbduanyang { skill, .. }
        | EffectRequest::ResolveMbrunwei { skill, .. }
        | EffectRequest::ResolveMbshuanghuai { skill, .. }
        | EffectRequest::ResolveMbxiongtu { skill, .. }
        | EffectRequest::ResolveMbganggeng { skill, .. }
        | EffectRequest::ResolveMbsijian { skill, .. }
        | EffectRequest::ResolveMbjianji { skill, .. }
        | EffectRequest::ResolveMbyuanmo { skill, .. }
        | EffectRequest::ResolveMbfeijing { skill, .. }
        | EffectRequest::ResolveMbxiaoge { skill, .. }
        | EffectRequest::ResolveMbxuye { skill, .. }
        | EffectRequest::ResolveMbkuangxiang { skill, .. }
        | EffectRequest::ResolvePotchengpu { skill, .. }
        | EffectRequest::ResolvePotjugu { skill, .. }
        | EffectRequest::ResolvePotsifeng { skill, .. }
        | EffectRequest::ResolvePotchenqun { skill, .. }
        | EffectRequest::ResolvePotwanglie { skill, .. }
        | EffectRequest::ResolvePothongyi { skill, .. }
        | EffectRequest::ResolvePotguansha { skill, .. }
        | EffectRequest::ResolvePotjiyu { skill, .. }
        | EffectRequest::ResolveMbzhuji { skill, .. }
        | EffectRequest::ResolvePotlibing { skill, .. }
        | EffectRequest::ResolvePotpoxi { skill, .. }
        | EffectRequest::ResolvePotfuan { skill, .. }
        | EffectRequest::ResolvePotyinxian { skill, .. }
        | EffectRequest::ResolvePothuanjie { skill, .. }
        | EffectRequest::ResolvePotdimeng { skill, .. }
        | EffectRequest::ResolvePothaoshi { skill, .. }
        | EffectRequest::ResolveSpmiaolve { skill, .. }
        | EffectRequest::ResolveSpyingjia { skill, .. }
        | EffectRequest::ResolveDcyingtu { skill, .. }
        | EffectRequest::ResolveXinfuQinguo { skill, .. }
        | EffectRequest::ResolveDcmanzhi { skill, .. }
        | EffectRequest::ResolveXinxingluan { skill, .. }
        | EffectRequest::ResolveDckuizhen { skill, .. }
        | EffectRequest::ResolveDckuizhenResolve { skill, .. }
        | EffectRequest::ResolveNsdingzhou { skill, .. }
        | EffectRequest::ResolveSxrmlanjiao { skill, .. }
        | EffectRequest::ResolveRekuanshi { skill, .. }
        | EffectRequest::ResolveRekuanshiResolve { skill, .. }
        | EffectRequest::ResolveDcmoyu { skill, .. }
        | EffectRequest::ResolveXizhen { skill, .. }
        | EffectRequest::ResolveXuxie { skill, .. }
        | EffectRequest::ResolveDcchiying { skill, .. }
        | EffectRequest::ResolveXinkuangfu { skill, .. }
        | EffectRequest::ResolveDcxiangshu { skill, .. }
        | EffectRequest::ResolveDcxiangshuLottery { skill, .. }
        | EffectRequest::ResolveShuangxiong { skill, .. }
        | EffectRequest::ResolveZhijian { skill, .. }
        | EffectRequest::ResolveGuzheng { skill, .. }
        | EffectRequest::ResolveZhiba { skill, .. }
        | EffectRequest::ResolveZhibaSettle { skill, .. }
        | EffectRequest::ResolveDrltCongjian { skill, .. }
        | EffectRequest::ResolveNzryFeijun { skill, .. }
        | EffectRequest::ResolveDrltYongsi { skill, .. }
        | EffectRequest::ResolveDrltWeidi { skill, .. }
        | EffectRequest::ResolveLieren { skill, .. }
        | EffectRequest::ResolveTuntian { skill, .. }
        | EffectRequest::ResolveZaoxian { skill, .. }
        | EffectRequest::ResolveTiaoxin { skill, .. }
        | EffectRequest::ResolveZhiji { skill, .. }
        | EffectRequest::ResolveZaiqi { skill, .. }
        | EffectRequest::ResolveTwqiushou { skill, .. }
        | EffectRequest::ResolveZhengrong { skill, .. }
        | EffectRequest::ResolveHongju { skill, .. }
        | EffectRequest::ResolveQingjian { skill, .. }
        | EffectRequest::ResolveReleiji { skill, .. }
        | EffectRequest::ResolveHuangtian { skill, .. }
        | EffectRequest::ResolveHaoshi { skill, .. }
        | EffectRequest::ResolveDimeng { skill, .. }
        | EffectRequest::ResolveQiaobian { skill, .. }
        | EffectRequest::ResolveXiangle { skill, .. }
        | EffectRequest::ResolveFangquanSkip { skill, .. }
        | EffectRequest::ResolveFangquanGrant { skill, .. }
        | EffectRequest::ResolveRuoyu { skill, .. }
        | EffectRequest::ResolveZuilun { skill, .. }
        | EffectRequest::ResolveFuyin { skill, .. }
        | EffectRequest::ResolveJuzhan { skill, .. }
        | EffectRequest::ResolveQizhi { skill, .. }
        | EffectRequest::ResolveJinqu { skill, .. }
        | EffectRequest::ResolveJianxiang { skill, .. }
        | EffectRequest::ResolveShenshi { skill, .. }
        | EffectRequest::ResolveMingren { skill, .. }
        | EffectRequest::ResolveZhenliang { skill, .. }
        | EffectRequest::ResolveKuizhu { skill, .. }
        | EffectRequest::ResolveZhizheng { skill, .. }
        | EffectRequest::ResolveLijun { skill, .. }
        | EffectRequest::ResolveLiangyin { skill, .. }
        | EffectRequest::ResolveKongsheng { skill, .. }
        | EffectRequest::ResolvePottuntian { skill, .. }
        | EffectRequest::ResolvePotjixi { skill, .. }
        | EffectRequest::ResolvePotceduan { skill, .. }
        | EffectRequest::ResolvePottaishici { skill, .. }
        | EffectRequest::ResolveMbzhonghui { skill, .. }
        | EffectRequest::ResolvePotjiejie { skill, .. }
        | EffectRequest::ResolvePotqingshi { skill, .. }
        | EffectRequest::ResolvePotweiyan { skill, .. }
        | EffectRequest::ResolvePotzhouyu { skill, .. }
        | EffectRequest::ResolvePotlvyi { skill, .. }
        | EffectRequest::ResolvePotcaoshuang { skill, .. }
        | EffectRequest::ResolvePotyuji { skill, .. }
        | EffectRequest::ResolveRehengjiang { skill, .. }
        | EffectRequest::ResolveXuxieAdd { skill, .. }
        | EffectRequest::ResolveTwxiawei { skill, .. }
        | EffectRequest::ResolveTwbianfuren { skill, .. }
        | EffectRequest::ResolveTwfeiyi { skill, .. }
        | EffectRequest::ResolveTwchenzhen { skill, .. }
        | EffectRequest::ResolveTwqianju { skill, .. }
        | EffectRequest::ResolveTwqingxi { skill, .. }
        | EffectRequest::ResolveTwzaoli { skill, .. }
        | EffectRequest::ResolveTwzhengjian { skill, .. }
        | EffectRequest::ResolveTwtianyu { skill, .. }
        | EffectRequest::ResolveTwwujing { skill, .. }
        | EffectRequest::ResolveTwshanghe { skill, .. }
        | EffectRequest::ResolveTwwangchang { skill, .. }
        | EffectRequest::ResolveTwcaozhao { skill, .. }
        | EffectRequest::ResolveTwguohuai { skill, .. }
        | EffectRequest::ResolveTwchengpu { skill, .. }
        | EffectRequest::ResolveTwcaoang { skill, .. }
        | EffectRequest::ResolveTwniujin { skill, .. }
        | EffectRequest::ResolveTwmoukui { skill, .. }
        | EffectRequest::ResolveTwfuman { skill, .. }
        | EffectRequest::ResolveCuijin { skill, .. }
        | EffectRequest::ResolveTwjiaohua { skill, .. }
        | EffectRequest::ResolveJiaohua { skill, .. }
        | EffectRequest::ResolveLuanqun { skill, .. }
        | EffectRequest::ResolveMbzhuguo { skill, .. }
        | EffectRequest::ResolveMbanxian { skill, .. }
        | EffectRequest::ResolveMbjinzu { skill, .. }
        | EffectRequest::DistributeTopTwoCards { skill, .. }
        | EffectRequest::ChooseDiscardBasicThenTargetDiscardEquipOrDamage { skill, .. }
        | EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner { skill, .. }
        | EffectRequest::ChooseTargetLoseHpDiscardHandShaIfBasic { skill, .. }
        | EffectRequest::ChooseCurrentShaTargetDiscardRedirectToOwner { skill, .. }
        | EffectRequest::ChooseDiscardRedirectCurrentSha { skill, .. }
        | EffectRequest::ChooseTargetGainBottomCard { skill, .. }
        | EffectRequest::JudgeBlackGainRepeat { skill, .. }
        | EffectRequest::JudgeSameColorGainRepeat { skill, .. }
        | EffectRequest::JudgeSetSkillMarkToRank { skill, .. }
        | EffectRequest::ChooseDiscardOneFromEachLinked { skill, .. }
        | EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp { skill, .. }
        | EffectRequest::ChooseGiveOneHandEquipOrRecover { skill, .. }
        | EffectRequest::ChooseGiveTwoDifferentColorsOrRecover { skill, .. }
        | EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { skill, .. }
        | EffectRequest::DrawByPlayPhaseCardCategoryCount { skill, .. }
        | EffectRequest::RefundCurrentShaUseAndDrawIfRed { skill, .. }
        | EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed { skill, .. }
        | EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
            skill, ..
        }
        | EffectRequest::ResolveOllinjie { skill, .. }
        | EffectRequest::QueueOpeningExtraPlayPhase { skill, .. }
        | EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits { skill, .. }
        | EffectRequest::ChooseTargetGainMaxHp { skill, .. }
        | EffectRequest::GainTargetHandCard { skill, .. }
        | EffectRequest::ResolveGuanwei { skill, .. }
        | EffectRequest::ResolveLanjiang { skill, .. }
        | EffectRequest::ResolveRebingyi { skill, .. }
        | EffectRequest::ResolveRebotu { skill, .. }
        | EffectRequest::ResolveOldimeng { skill, .. }
        | EffectRequest::ResolveRejieyin { skill, .. }
        | EffectRequest::ResolveReanxu { skill, .. }
        | EffectRequest::ResolveOldAnxu { skill, .. }
        | EffectRequest::StoreSelectedInSkillPile { skill, .. }
        | EffectRequest::GainTopCardToSkillPile { skill, .. }
        | EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill { skill, .. }
        | EffectRequest::GainOneHandFromSelectedTargets { skill, .. }
        | EffectRequest::ChooseRepoluTargetsDraw { skill, .. }
        | EffectRequest::ResolveZhuiyi { skill, .. }
        | EffectRequest::ResolveXinfuGuolun { skill, .. }
        | EffectRequest::ResolveOldQinqing { skill, .. }
        | EffectRequest::ResolveDrltJueyan { skill, .. }
        | EffectRequest::ResolveRejizhi { skill, .. }
        | EffectRequest::ResolveJiexun { skill, .. }
        | EffectRequest::ResolveWenguaSetup { skill, .. }
        | EffectRequest::ResolveWengua { skill, .. }
        | EffectRequest::ResolveXinanguo { skill, .. }
        | EffectRequest::LoseHpDoubleCurrentDamageThenReplace { skill, .. }
        | EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage { skill, .. }
        | EffectRequest::IncreaseCurrentShaBaseDamage { skill, .. }
        | EffectRequest::IncreaseCurrentCardBaseDamage { skill, .. }
        | EffectRequest::ChooseAddCurrentShaTarget { skill, .. }
        | EffectRequest::ChooseAddCurrentShaTargetWithinDistance { skill, .. }
        | EffectRequest::RefundCurrentShaUse { skill, .. }
        | EffectRequest::DamageThenDrawSelectedTargets { skill, .. }
        | EffectRequest::ResolveYoudi { skill, .. }
        | EffectRequest::ResolveKuangfu { skill, .. }
        | EffectRequest::ResolveShanjia { skill, .. }
        | EffectRequest::ResolveNewYajiao { skill, .. }
        | EffectRequest::ResolveDingpin { skill, .. }
        | EffectRequest::CommitAddCurrentShaTarget { skill, .. }
        | EffectRequest::ChooseStoreHandShaInSkillPile { skill, .. }
        | EffectRequest::ChooseSpendSkillPileForDyingJiu { skill, .. }
        | EffectRequest::CommitStoreHandShaInSkillPile { skill, .. }
        | EffectRequest::CommitSpendSkillPileForDyingJiu { skill, .. }
        | EffectRequest::ChooseCompareSelectedHandCard { skill, .. }
        | EffectRequest::ContinueChooseCompareTargetHandCard { skill, .. }
        | EffectRequest::ChooseCompareSelectedHandCardSecondFailureSeatDamage {
            skill, ..
        }
        | EffectRequest::ContinueChooseCompareTargetHandCardSecondFailureSeatDamage {
            skill, ..
        }
        | EffectRequest::ChooseDrawOrDiscardTargetCard { skill, .. }
        | EffectRequest::RecordCompareFailureSeatDamage { skill, .. }
        | EffectRequest::ChooseSeatPathDamage { skill, .. }
        | EffectRequest::DealDamageEachInOrder { skill, .. }
        | EffectRequest::RandomizeCurrentHandCompareOpponent { skill, .. }
        | EffectRequest::GainHighestRankShaFromEventCards { skill, .. }
        | EffectRequest::GainHandCompareResultCard { skill, .. }
        | EffectRequest::ChooseAddOrRemoveCurrentCardTarget { skill, .. }
        | EffectRequest::ChooseHandCompare { skill, .. }
        | EffectRequest::ChooseHandCompareGrantTargetOnWin { skill, .. }
        | EffectRequest::ChooseHandCompareBetween { skill, .. }
        | EffectRequest::GuanxingTop { skill, .. }
        | EffectRequest::ChooseOtherGiveThenRecover { skill, .. }
        | EffectRequest::GuanxingTopByFaluMarks { skill, .. }
        | EffectRequest::ChooseOtherDamage { skill, .. }
        | EffectRequest::RevealUniformHandChooseDamage { skill, .. }
        | EffectRequest::ChooseGainDiscardCardByKind { skill, .. }
        | EffectRequest::ChooseAddMissingSkill { skill, .. }
        | EffectRequest::AddFirstMissingSkill { skill, .. }
        | EffectRequest::ResolveDccunsi { skill, .. }
        | EffectRequest::RecordCurrentShaAllTargets { skill, .. }
        | EffectRequest::ChooseGiveOrDiscardHand { skill, .. }
        | EffectRequest::ChooseDiscardHandEquipThenRecoverTarget { skill, .. }
        | EffectRequest::ChooseOtherDiscardTargetCard { skill, .. }
        | EffectRequest::ChooseAddSkillBoundCardTarget { skill, .. }
        | EffectRequest::CommitAddSkillBoundCardTarget { skill, .. }
        | EffectRequest::ChooseDiscardMatchingEquipColorAddSkill { skill, .. }
        | EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { skill, .. }
        | EffectRequest::ChooseCurrentCardUseEnhancements { skill, .. }
        | EffectRequest::ChooseDrawOrDiscardAddSkill { skill, .. }
        | EffectRequest::ChooseDiscardedBlackBasicAsBingliang { skill, .. }
        | EffectRequest::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage { skill, .. }
        | EffectRequest::UseVirtualShaDrawIfNoDamage { skill, .. }
        | EffectRequest::DiscardSelectedRecordUniqueCategoryTarget { skill, .. }
        | EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
            skill, ..
        }
        | EffectRequest::FinishLinkIfTargetWounded { skill, .. }
        | EffectRequest::ChooseRevealOtherHandCardRememberCategory { skill, .. }
        | EffectRequest::DrawThenGiveExactRecordPhaseTarget { skill, .. }
        | EffectRequest::ClearPhaseGiftTarget { skill, .. }
        | EffectRequest::ChooseGainResolvedShaOrShanThenOptionalUseSha { skill, .. }
        | EffectRequest::ChooseGainAllResolvedShaShanThenColorRewards { skill, .. }
        | EffectRequest::ChooseUseShaAfterResolvedGift { skill, .. } => skill.raw(),
        EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors { skill, .. } => {
            skill.raw()
        }
        EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw { skill, .. } => {
            skill.raw()
        }
        EffectRequest::DealDamageBanActiveIfDying { skill, .. }
        | EffectRequest::CommitDamageBanActiveIfDying { skill, .. }
        | EffectRequest::ChooseTargetReclaimEquipmentDrawIfRangeReduced { skill, .. }
        | EffectRequest::CommitTargetReclaimEquipmentDrawIfRangeReduced { skill, .. }
        | EffectRequest::FinishTargetReclaimEquipmentDrawIfRangeReduced { skill, .. } => {
            skill.raw()
        }
        EffectRequest::SpjungongDiscardOrLoseThenSha { skill, .. }
        | EffectRequest::BanActiveUntilTurnEnd { skill, .. } => skill.raw(),
        EffectRequest::ResolveTwfenxian { skill, .. }
        | EffectRequest::SetSkillContentFlag { skill, .. } => skill.raw(),
        EffectRequest::ChooseUseVirtualCard { skill, .. }
        | EffectRequest::ChooseUseVirtualNanman { skill, .. }
        | EffectRequest::ResolveQiluan { skill, .. }
        | EffectRequest::DiscardAllOwnJudge { skill, .. }
        | EffectRequest::StoreEventCardInSkillPile { skill, .. }
        | EffectRequest::ResolveOllinjie { skill, .. }
        | EffectRequest::ChooseTargetGainMaxHp { skill, .. }
        | EffectRequest::GainTargetHandCard { skill, .. } => skill.raw(),
        EffectRequest::RevealUniformHandDrawSelfChooseOthers { skill, .. }
        | EffectRequest::ChooseCardUserOrAddTarget { skill, .. }
        | EffectRequest::CommitTransferSavedCardUser { skill, .. }
        | EffectRequest::ChooseDiscardEquipmentReduceCurrentDamage { skill, .. }
        | EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage { skill, .. }
        | EffectRequest::ChooseDiscardDamageEndpointCardIncreaseIfSuit { skill, .. }
        | EffectRequest::ChooseMoveTargetEquipmentOrDraw { skill, .. }
        | EffectRequest::ChooseMoveTargetEquipmentAndOrDraw { skill, .. }
        | EffectRequest::ChooseDrawViewTwoGainOrMoveEquipment { skill, .. }
        | EffectRequest::GiveAllEquipmentChooseRecoverOrDamage { skill, .. }
        | EffectRequest::ChooseRecoverOwnerOrDamageByGivenEquipment { skill, .. } => {
            skill.raw()
        }
        EffectRequest::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage { skill, .. }
        | EffectRequest::ChooseTargetDiscardDistanceHandOrIncreaseDamage {
            skill, ..
        } => skill.raw(),
        EffectRequest::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
            skill,
            ..
        }
        | EffectRequest::ChooseDrawOrRecastAddSkill { skill, .. }
        | EffectRequest::ChooseGainShaForOpeningExtraPlay { skill, .. }
        | EffectRequest::FinishOpeningExtraPlayShaGain { skill, .. }
        | EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay { skill, .. }
        | EffectRequest::ChooseGainOwnLostDiscardCardThisTurn { skill, .. }
        | EffectRequest::ChooseMoveFieldCardTo { skill, .. }
        | EffectRequest::DrawThenOptionallyDistributeHandCards { skill, .. }
        | EffectRequest::GiveSelectedThenContinueDistinctRecoverAtTwo { skill, .. }
        | EffectRequest::GiveSelectedThenRecoverTarget { skill, .. }
        | EffectRequest::GiveSelectedThenRecoverSelf { skill, .. }
        | EffectRequest::GiveSelectedThenRecoverSelf { skill, .. }
        | EffectRequest::ResolveDingpan { skill, .. }
        | EffectRequest::ResolveTanlu { skill, .. }
        | EffectRequest::RevealTargetHandGainShanArmorOrDrawTwo { skill, .. } => skill.raw(),
        EffectRequest::ChooseHandCountDifferentTargetDrawOrRandomDiscard { skill, .. } => {
            skill.raw()
        }
        EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw {
            skill, ..
        }
        | EffectRequest::ChooseSelfDiscardOrTargetDrawAfterTargetDiscard { skill, .. }
        | EffectRequest::ResolveDecadezhenjun { skill, .. }
        | EffectRequest::ResolveSpXiaoguo { skill, .. }
        | EffectRequest::ChooseGainCardFromEachOtherReplaceDraw { skill, .. } => {
            skill.raw()
        }
        EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors { skill, .. }
        | EffectRequest::ContinueChooseGainOneFromDamageTargetAndNeighbors { skill, .. } => {
            skill.raw()
        }
        EffectRequest::ChoosePreventDamageOrLoseMaxHpIncreaseDamage { skill, .. }
        | EffectRequest::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark { skill, .. }
        | EffectRequest::DrawThenChooseGiveOrDiscardHand { skill, .. }
        | EffectRequest::ContinueIncreaseBoundDamage { skill, .. }
        | EffectRequest::ForceOthersUseShaOnNearestOrLoseHp { skill, .. }
        | EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying { skill, .. }
        | EffectRequest::CommitLoseHpThenDamageOtherLoseMaxHpIfDying { skill, .. } => {
            skill.raw()
        }
        EffectRequest::ReplaceSkillWithBoundAfterResolutionIfTargetAlive { skill, .. }
        | EffectRequest::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp { skill, .. } => {
            skill.raw()
        }
        _ => inherited,
    }
}
