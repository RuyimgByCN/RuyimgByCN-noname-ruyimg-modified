//! ResolveChoice and continuation resume.
//!
//! Split out of a single 41k-line file (ADR 0594). This module keeps
//! `handle_resolve_choice` — the dispatch from `ContinuationKind` to the handler that
//! resumes it — while the handlers live in the `group_*` submodules. The split is
//! positional, so no item moved relative to its neighbours.


use crate::effect::{open_choice, run_effects, EffectError, RESOLVE_DRLT_WEIDI_CUSTOM_CHOICE};
use crate::state::{CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use crate::transaction::Transaction;
use engine_api::{
    CardId, CardMoveReason, ChoiceId, ChoiceKind, PlayerId, SkillDefId, SkillId, Suit,
};
use rule_api::{DrawTargetScope, EffectRequest};


mod group_pre_1;
mod group_pre_2;
mod group_pre_3;
mod group_post_1;
mod group_post_2;
mod group_post_3;
mod group_post_4;
mod group_post_5;
mod group_post_6;
mod group_post_7;
mod group_post_8;
mod group_post_9;

pub(crate) use group_pre_1::*;
pub(crate) use group_pre_2::*;
pub(crate) use group_pre_3::*;
pub(crate) use group_post_1::*;
pub(crate) use group_post_2::*;
pub(crate) use group_post_3::*;
pub(crate) use group_post_4::*;
pub(crate) use group_post_5::*;
pub(crate) use group_post_6::*;
pub(crate) use group_post_7::*;
pub(crate) use group_post_8::*;
pub(crate) use group_post_9::*;

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

    let result = (|| -> Result<(), EffectError> {
        match kind {
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::TWCHENGPU_CUSTOM_CHOICE) =>
            {
                resolve_twchengpu_custom(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS_CUSTOM_CHOICE) =>
            {
                resolve_choose_other_gain_one_of_two_skills(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::SHUIYANQIJUNX_CUSTOM_CHOICE) =>
            {
                resolve_shuiyanqijunx_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::KEFUZHONGYUAN_CUSTOM_CHOICE) =>
            {
                resolve_kefuzhongyuan_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::GUGUOANBANG_CUSTOM_CHOICE) =>
            {
                resolve_guguoanbang_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::LIANJUNSHENGYAN_CUSTOM_CHOICE) =>
            {
                resolve_lianjunshengyan_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::WENHELUANWU_CUSTOM_CHOICE) =>
            {
                resolve_wenheluanwu_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::HAOLINGTIANXIA_CUSTOM_CHOICE) =>
            {
                resolve_haolingtianxia_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::CHILING_CUSTOM_CHOICE) =>
            {
                resolve_chiling_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::TWCAOANG_CUSTOM_CHOICE) =>
            {
                resolve_twcaoang_custom(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::TWNIUJIN_CUSTOM_CHOICE) =>
            {
                resolve_twniujin_custom(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::TWMOUKUI_CUSTOM_CHOICE) =>
            {
                resolve_twmoukui_custom(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_ADD_SKILL_CUSTOM_CHOICE) =>
            {
                resolve_choose_target_draw_max_hp_cap_then_add_skill(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_TIANXIANG_CUSTOM_CHOICE) =>
            {
                resolve_resolve_tianxiang(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_RETIANXIANG_CUSTOM_CHOICE) =>
            {
                resolve_resolve_retianxiang(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_LIANZHOU_CUSTOM_CHOICE) =>
            {
                resolve_resolve_lianzhou(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_STDDECHONG_CUSTOM_CHOICE) =>
            {
                resolve_resolve_stddechong(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::DISCARD_HAND_EQUIP_COUNT_CUSTOM_CHOICE) =>
            {
                resolve_discard_hand_equip_count(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::CHOOSE_DISCARD_HAND_RANGE_CHOICE) =>
            {
                resolve_choose_discard_hand_range(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DCCTJIUXIAN_CHOICE) =>
            {
                resolve_dcctjiuxian_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_BIANYUE_CHOICE) =>
            {
                resolve_bianyue_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_ZHUPEILAN_CHOICE) =>
            {
                resolve_zhupeilan_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_DC_SB_ZHANGXIU_CHOICE) =>
            {
                crate::effect::resolve_dc_sb_zhangxiu_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_DC_SB_ZHOUYU_CHOICE) =>
            {
                crate::effect::resolve_dc_sb_zhouyu_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DC_SB_LUSU_CHOICE) =>
            {
                crate::effect::resolve_dc_sb_lusu_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_ZHANGJIAN_CHOICE) =>
            {
                crate::effect::resolve_zhangjian_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_ZHUGERUOXUE_CHOICE) =>
            {
                crate::effect::resolve_zhugeruoxue_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_CAOYI_CHOICE) =>
            {
                crate::effect::resolve_caoyi_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_MALINGLI_CHOICE) =>
            {
                crate::effect::resolve_malingli_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_NSSHUAIYAN_CHOICE) =>
            {
                crate::effect::resolve_nsshuaiyan_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_WDB_LIUFU_CHOICE) =>
            {
                crate::effect::resolve_wdb_liufu_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_WDB_LIUZAN_CHOICE) =>
            {
                crate::effect::resolve_wdb_liuzan_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_WDB_TIANYU_CHOICE) =>
            {
                crate::effect::resolve_wdb_tianyu_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_WDB_FUCHOU_CHOICE) =>
            {
                crate::effect::resolve_wdb_fuchou_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_WDB_YANGYI_CHOICE) =>
            {
                crate::effect::resolve_wdb_yangyi_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_WDB_ZAOZHIRENJUN_CHOICE) =>
            {
                crate::effect::resolve_wdb_zaozhirenjun_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_XIANXI_CHOICE) =>
            {
                crate::effect::resolve_ddd_xianxi_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_XIAHENG_CHOICE) =>
            {
                crate::effect::resolve_ddd_xiaheng_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_LIUCHONG_CHOICE) =>
            {
                crate::effect::resolve_ddd_liuchong_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_DUANBING_CHOICE) =>
            {
                crate::effect::resolve_ddd_duanbing_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_WUZHI_CHOICE) =>
            {
                crate::effect::resolve_ddd_wuzhi_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_XUJING_CHOICE) =>
            {
                crate::effect::resolve_ddd_xujing_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_CAOMAO_CHOICE) =>
            {
                crate::effect::resolve_ddd_caomao_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_XINXIANYING_CHOICE) =>
            {
                crate::effect::resolve_ddd_xinxianying_choice(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_XIANGLANG_CHOICE) =>
            {
                crate::effect::resolve_ddd_xianglang_choice(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_YUJIN_CHOICE) =>
            {
                crate::effect::resolve_ddd_yujin_choice(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_LIUYE_CHOICE) =>
            {
                crate::effect::resolve_ddd_liuye_choice(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_BAOSANNIANG_CHOICE) =>
            {
                crate::effect::resolve_ddd_baosanniang_choice(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_ZHENJI_CHOICE) =>
            {
                crate::effect::resolve_ddd_zhenji_choice(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_ZHAOANG_CHOICE) =>
            {
                crate::effect::resolve_ddd_zhaoang_choice(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_LIUBA_CHOICE) =>
            {
                crate::effect::resolve_ddd_liuba_choice(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_JIANSHUO_CHOICE) =>
            {
                crate::effect::resolve_ddd_jianshuo_choice(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_GUANNING_CHOICE) =>
            {
                crate::effect::resolve_ddd_guanning_choice(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_CAOSHUANG_CHOICE) =>
            {
                crate::effect::resolve_ddd_caoshuang_choice(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_XUELINGYUN_CHOICE) =>
            {
                crate::effect::resolve_ddd_xuelingyun_choice(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_LIUHONG_CHOICE) =>
            {
                crate::effect::resolve_ddd_liuhong_choice(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_XIAHOUXUAN_CHOICE) =>
            {
                crate::effect::resolve_ddd_xiahouxuan_choice(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_ZHANGKAI_CHOICE) =>
            {
                crate::effect::resolve_ddd_zhangkai_choice(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DDD_LIANGXI_CHOICE) =>
            {
                crate::effect::resolve_ddd_liangxi_choice(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&super::play::YOUDISHENRU_FORCE_CHOICE) =>
            {
                super::play::resolve_youdishenru_force_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_JIELIANG_GAIN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_jieliang_gain(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_STDJUJIAN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_stdjujian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_LULVE_CUSTOM_CHOICE) =>
            {
                resolve_resolve_lulve(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_DCTINGXIAN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_dctingxian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_HFJIEYING_CUSTOM_CHOICE) =>
            {
                resolve_resolve_hfjieying(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTHUANJIE_CUSTOM_CHOICE) =>
            {
                resolve_pothuanjie_target(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTNIGU_CUSTOM_CHOICE) =>
            {
                resolve_potnigu_gift(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTLULIAN_CUSTOM_CHOICE) =>
            {
                resolve_potlulian_target(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTDIMENG_CUSTOM_CHOICE) =>
            {
                resolve_potdimeng_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTHAOSHI_CUSTOM_CHOICE) =>
            {
                resolve_pothaoshi_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_DCNUCHEN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_dcnuchen(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_DCZHIHENG_CUSTOM_CHOICE) =>
            {
                resolve_resolve_dczhiheng(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_STDGAOBIAN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_stdgaobian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_SPJIEDAO_CUSTOM_CHOICE) =>
            {
                resolve_resolve_spjiedao(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_SIBIAN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_sibian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::REVEAL_CATEGORY_DISTRIBUTION_CUSTOM_CHOICE) =>
            {
                resolve_reveal_category_distribution(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RANDOM_GAIN_REGION_COMPARE_COLOR_CUSTOM_CHOICE) =>
            {
                resolve_random_gain_region_compare_color(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::JIEBING_CUSTOM_CHOICE) =>
            {
                resolve_jiebing(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::MOBILEDANSHOU_CUSTOM_CHOICE) =>
            {
                resolve_mobiledanshou(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_JSRGCHIYING_CUSTOM_CHOICE) =>
            {
                resolve_jsrgchiying(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::MBQINGDAO_CUSTOM_CHOICE) =>
            {
                resolve_mbqingdao(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::MBXIUGENG_CUSTOM_CHOICE) =>
            {
                resolve_mbxiugeng(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::MBCHENSHE_CUSTOM_CHOICE) =>
            {
                resolve_mbchenshe(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::MBRENXING_CUSTOM_CHOICE) =>
            {
                resolve_mbrenxing(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::MBCHIZHANG_CUSTOM_CHOICE) =>
            {
                resolve_mbchizhang(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::MBDUANYANG_CUSTOM_CHOICE) =>
            {
                resolve_mbduanyang(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::MBRUNWEI_CUSTOM_CHOICE) =>
            {
                resolve_mbrunwei(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::MBSHUANGHUAI_CUSTOM_CHOICE) =>
            {
                resolve_mbshuanghuai(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::MBXIONGTU_CUSTOM_CHOICE) =>
            {
                resolve_mbxiongtu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::MBSIJIAN_CUSTOM_CHOICE) =>
            {
                resolve_mbsijian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::MBJIANJI_CUSTOM_CHOICE) =>
            {
                resolve_mbjianji(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::MBYUANMO_CUSTOM_CHOICE) =>
            {
                resolve_mbyuanmo(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::MBFEIJING_CUSTOM_CHOICE) =>
            {
                resolve_mbfeijing(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::MBXUYE_CUSTOM_CHOICE) =>
            {
                resolve_mbxuye(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTCHENGPU_CUSTOM_CHOICE) =>
            {
                resolve_potchengpu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&super::play::MANTIANGUOHAI_CUSTOM_CHOICE) =>
            {
                resolve_mantianguohai_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::SPMIAOLVE_CUSTOM_CHOICE) =>
            {
                resolve_spmiaolve_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::SPYINGJIA_CUSTOM_CHOICE) =>
            {
                resolve_spyingjia_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTTUNTIAN_CUSTOM_CHOICE) =>
            {
                resolve_pottuntian_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::MBZHONGHUI_CUSTOM_CHOICE) =>
            {
                resolve_mbzhonghui_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTJIEJIE_CUSTOM_CHOICE) =>
            {
                resolve_potjiejie_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTQINGSHI_CUSTOM_CHOICE) =>
            {
                resolve_potqingshi_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTWEIYAN_CUSTOM_CHOICE) =>
            {
                resolve_potweiyan_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTZHOUYU_CUSTOM_CHOICE) =>
            {
                crate::effect::resolve_potzhouyu_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTLVYI_CUSTOM_CHOICE) =>
            {
                crate::effect::resolve_potlvyi_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTCAOSHUANG_CUSTOM_CHOICE) =>
            {
                crate::effect::resolve_potcaoshuang_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTJIXI_CUSTOM_CHOICE) =>
            {
                resolve_potjixi_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTCEDUAN_CUSTOM_CHOICE) =>
            {
                resolve_potceduan_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTTAISHICI_CUSTOM_CHOICE) =>
            {
                crate::effect::resolve_pottaishici_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTJUGU_CUSTOM_CHOICE) =>
            {
                resolve_potjugu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTSIFENG_CUSTOM_CHOICE) =>
            {
                resolve_potsifeng(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTCHENQUN_CUSTOM_CHOICE) =>
            {
                resolve_potchenqun(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTCHENDAO_CUSTOM_CHOICE) =>
            {
                resolve_potchendao(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::MBZHUJI_CUSTOM_CHOICE) =>
            {
                resolve_mbzhuji(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTPOXI_CUSTOM_CHOICE) =>
            {
                resolve_potpoxi(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::POTFUAN_CUSTOM_CHOICE) =>
            {
                resolve_potfuan(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::REFUHAI_CUSTOM_CHOICE) =>
            {
                resolve_refuhai(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::YANGWU_GAIN_HAND_CUSTOM_CHOICE) =>
            {
                resolve_yangwu_gain_hand(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::GZJINFA_CUSTOM_CHOICE) =>
            {
                resolve_gzjinfa(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_SPFENGYIN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_spfengyin(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_XINFU_LIANPIAN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_xinfu_lianpian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_GONGJIAN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_gongjian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_XINFU_QINGUO_CUSTOM_CHOICE) =>
            {
                resolve_resolve_xinfu_qinguo(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_DCMANZHI_CUSTOM_CHOICE) =>
            {
                resolve_resolve_dcmanzhi(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_XINXINGLUAN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_xinxingluan(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_DCKUIZHEN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_dckuizhen(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_NSDINGZHOU_CUSTOM_CHOICE) =>
            {
                resolve_resolve_nsdingzhou(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_SXRMLANJIAO_CUSTOM_CHOICE) =>
            {
                resolve_resolve_sxrmlanjiao(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_REKUANSHI_CUSTOM_CHOICE) =>
            {
                resolve_resolve_rekuanshi(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_DCMOYU_CUSTOM_CHOICE) =>
            {
                resolve_resolve_dcmoyu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_XIZHEN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_xizhen(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_XUXIE_CUSTOM_CHOICE) =>
            {
                resolve_resolve_xuxie(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_DCCHIYING_CUSTOM_CHOICE) =>
            {
                resolve_resolve_dcchiying(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_XINKUANGFU_CUSTOM_CHOICE) =>
            {
                resolve_resolve_xinkuangfu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_DCXIANG_SHU_CUSTOM_CHOICE) =>
            {
                resolve_resolve_dcxiangshu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_ZHIJIAN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_zhijian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_GUZHENG_CUSTOM_CHOICE) =>
            {
                resolve_resolve_guzheng(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_ZHIBA_CUSTOM_CHOICE) =>
            {
                resolve_resolve_zhiba(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_TIAOXIN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_tiaoxin(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_ZHENGRONG_CUSTOM_CHOICE) =>
            {
                resolve_resolve_zhengrong(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_HONGJU_CUSTOM_CHOICE) =>
            {
                resolve_resolve_hongju(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_QINGJIAN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_qingjian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_RELEIJI_CUSTOM_CHOICE) =>
            {
                resolve_resolve_releiji(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_HUANGTIAN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_huangtian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_HAOSHI_CUSTOM_CHOICE) =>
            {
                resolve_resolve_haoshi(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_DIMENG_CUSTOM_CHOICE) =>
            {
                resolve_resolve_dimeng(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_QIAOBIAN_DISCARD) =>
            {
                resolve_qiaobian_discard(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_QIAOBIAN_DRAW_TARGETS) =>
            {
                resolve_qiaobian_draw_targets(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_QIAOBIAN_DRAW_PICK) =>
            {
                resolve_qiaobian_draw_pick(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_QIAOBIAN_PLAY_CARD) =>
            {
                resolve_qiaobian_play_card(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_QIAOBIAN_PLAY_TARGET) =>
            {
                resolve_qiaobian_play_target(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_XIANGLE_CUSTOM_CHOICE) =>
            {
                resolve_resolve_xiangle(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_FANGQUAN_DISCARD) =>
            {
                resolve_fangquan_discard(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_FANGQUAN_TARGET) =>
            {
                resolve_fangquan_target(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_ZUILUN_PICK) =>
            {
                resolve_zuilun_pick(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_ZUILUN_TARGET) =>
            {
                resolve_zuilun_target(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_JUZHAN_GAIN) =>
            {
                resolve_juzhan_gain(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_QIZHI_TARGET) =>
            {
                resolve_qizhi_target(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_QIZHI_CARD) =>
            {
                resolve_qizhi_card(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_JINQU_DISCARD) =>
            {
                resolve_jinqu_discard(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_JIANXIANG_TARGET) =>
            {
                resolve_jianxiang_target(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_SHENSHI_GIVE) =>
            {
                resolve_shenshi_give(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_SHENSHI_TARGET) =>
            {
                resolve_shenshi_target(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_SHENSHI_REWARD) =>
            {
                resolve_shenshi_reward(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_MINGREN_PLACE) =>
            {
                resolve_mingren_place(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_MINGREN_REPLACE) =>
            {
                resolve_mingren_place(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_ZHENLIANG_DISCARD) =>
            {
                resolve_zhenliang_discard(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_ZHENLIANG_DAMAGE) =>
            {
                resolve_zhenliang_damage(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_ZHENLIANG_DRAW) =>
            {
                resolve_zhenliang_draw(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_KUIZHU_DRAW)
                    && *payload.get(1).unwrap_or(&0) == 1 =>
            {
                resolve_kuizhu_dispatch(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_KUIZHU_DRAW) =>
            {
                resolve_kuizhu_draw(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_KUIZHU_DAMAGE) =>
            {
                resolve_kuizhu_damage(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_ZHIZHENG_CARD)
                    && *payload.get(1).unwrap_or(&0) == 1 =>
            {
                resolve_zhizheng_card(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_LIANGYIN_DRAW) =>
            {
                resolve_liangyin_draw(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_LIANGYIN_RECOVER) =>
            {
                resolve_liangyin_recover(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_LIANGYIN_DISCARD) =>
            {
                resolve_liangyin_discard(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_LIANGYIN_TARGET) =>
            {
                resolve_liangyin_target(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_LIANGYIN_COMMIT) =>
            {
                resolve_liangyin_commit(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_KONGSHENG_PLACE) =>
            {
                resolve_kongsheng_place(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_KONGSHENG_TARGET) =>
            {
                resolve_kongsheng_target(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_ZHIZHENG_CARD) =>
            {
                resolve_zhizheng_commit(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_DRLT_CONGJIAN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_drlt_congjian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_NZRY_FEIJUN_CUSTOM_CHOICE) =>
            {
                resolve_resolve_nzry_feijun(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_DRLT_WEIDI_CUSTOM_CHOICE) =>
            {
                resolve_resolve_drlt_weidi(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_DCYINGTU_CUSTOM_CHOICE) =>
            {
                resolve_resolve_dcyingtu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_TINGWEI_SELECT) => {
                resolve_tingwei_select(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_TINGWEI_OPTION) => {
                resolve_tingwei_option(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first()
                    == Some(&crate::effect::RESOLVE_XINMINGSHI_CUSTOM_CHOICE) =>
            {
                resolve_xinmingshi_discard(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_XUNYI_SELECT) => {
                resolve_xunyi_select(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_EJIAN_CHOICE) => {
                resolve_ejian_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_SHEYI_CHOICE) => {
                resolve_sheyi_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DCJUE_CHOICE) => {
                resolve_dcjue_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_RUANJI_CHOICE) => {
                resolve_ruanji_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_ZHENGBI_CHOICE) => {
                resolve_zhengbi_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_QIAOJIAN_CHOICE) => {
                resolve_qiaojian_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_XICHA_CHOICE) => {
                resolve_xicha_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_HUZUN_CHOICE) => {
                resolve_huzun_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_ZHUGEMENGXUE_CHOICE) => {
                resolve_zhugemengxue_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_BAILINGYUN_CHOICE) => {
                resolve_bailingyun_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DCDICHOU_CHOICE) => {
                resolve_dcdichou_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DCDUFENG_CHOICE) => {
                resolve_dcdufeng_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_CHENGLVE_DISCARD) => {
                resolve_chenglve_discard(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_SHICAI_PLACE) => {
                resolve_shicai_place(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_TWJUXIANG_CARD) => {
                resolve_twjuxiang_card(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_GZBUQU_RECOVER_CARD) => {
                resolve_gzbuqu_recover_card(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DBZHUIFENG_TARGET) => {
                resolve_dbzhuifeng_target(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DBQUEDI_CHOICE) => {
                resolve_dbquedi_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_SPYINJU_CHOICE) => {
                resolve_spyinju_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_SPSHANGYI_CHOICE) => {
                resolve_spshangyi_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_HEJI_CHOICE) => {
                resolve_heji_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_SPYAJUN_CUSTOM_CHOICE) => {
                resolve_spyajun_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_SPZUNDI_CUSTOM_CHOICE) => {
                resolve_spzundi_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_XINYANGJIE_CUSTOM_CHOICE) => {
                resolve_xinyangjie_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_QINGYU_CUSTOM_CHOICE) => {
                resolve_qingyu_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_QINGYU_FAIL_TARGET) => {
                resolve_qingyu_fail_target(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_HOUFENG_CHOICE) => {
                resolve_houfeng_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_RECHUHAI_CHOICE) => {
                resolve_rechuhai_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_XIONGSHU_CHOICE) => {
                resolve_xiongshu_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_SPQIAI_CHOICE) => {
                resolve_spqiai_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_SPSHANXI_CHOICE) => {
                resolve_spshanxi_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_SUOMING_CHOICE) => {
                resolve_suoming_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_TONGDUO_CHOICE) => {
                resolve_tongduo_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_DUANBI_CHOICE) => {
                resolve_duanbi_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_ZAOLI_CHOICE) => {
                resolve_zaoli_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::RESOLVE_GUYING_CHOICE) => {
                resolve_guying_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::YUANLVE_CUSTOM_CHOICE) =>
            {
                resolve_yuanlve_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::TWCHENXUN_CUSTOM_CHOICE) =>
            {
                resolve_twchenxun_target(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::effect::TWFENXIAN_CUSTOM_CHOICE) =>
            {
                resolve_twfenxian_choice(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::Custom
                if payload.first() == Some(&crate::mode::LOUREN_XIN_CHOICE) =>
            {
                // ADR 0598: guozhan 拉拢人心 (ambition-lord alliance) choice resolution.
                crate::mode::resolve_lourenxin_choice(
                    tx.state_mut(),
                    observer,
                    selections,
                    cancelled,
                );
                Ok(())
            }
            ContinuationKind::None | ContinuationKind::Custom => {
                tx.push_event(engine_api::DomainEvent::RuleNote {
                    code: 2,
                    payload: {
                        let mut v = vec![choice_id.raw(), u32::from(cancelled)];
                        v.extend_from_slice(selections);
                        v
                    },
                });
                Ok(())
            }
            ContinuationKind::DyingSave => {
                let dying = PlayerId::new(*payload.first().unwrap_or(&0));
                let killer = PlayerId::new(*payload.get(1).unwrap_or(&0));
                let responder = PlayerId::new(*payload.get(2).unwrap_or(&0));
                let len = *payload.get(3).unwrap_or(&u32::MAX) as usize;
                let remaining = payload
                    .get(4..)
                    .filter(|rest| rest.len() == len)
                    .ok_or_else(|| EffectError::EngineFault("truncated dying-save queue".into()))?
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect::<Vec<_>>();
                if observer != responder {
                    return Err(EffectError::Illegal(
                        "dying save must be answered by the current responder".into(),
                    ));
                }
                let pick = if cancelled {
                    0
                } else {
                    selections.first().copied().unwrap_or(0)
                };
                if pick == 0 {
                    run_effects(
                        tx,
                        vec![EffectRequest::ContinueDyingSave {
                            player: dying,
                            source: killer,
                            responders: remaining,
                        }],
                    )?;
                } else if pick == 1 {
                    let (save_card, save_kind) =
                        crate::query::dying_save_card_and_kind_for(tx.state(), responder, dying)
                            .ok_or_else(|| EffectError::Illegal("card save disabled".into()))?;
                    let from = tx
                        .state()
                        .cards
                        .get(save_card)
                        .map(|card| card.zone)
                        .ok_or_else(|| EffectError::Illegal("missing save card".into()))?;
                    let discard = tx
                        .state()
                        .zones
                        .find(ZoneKind::DiscardPile, PlayerId::NONE)
                        .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
                    let mut responders = vec![responder];
                    responders.extend(remaining);
                    let mut effects = vec![EffectRequest::MoveCardsWithReason {
                        cards: vec![save_card],
                        to: discard,
                        from: Some(from),
                        reason: CardMoveReason::Respond,
                    }];
                    append_pothaoshi_refills(
                        tx.state(),
                        responder,
                        &[save_card],
                        &mut effects,
                    );
                    effects.push(EffectRequest::CardResponded {
                        player: responder,
                        respond_to: dying,
                        card: save_card,
                        response_kind: save_kind.tag(),
                    });
                    effects.extend([
                            EffectRequest::RecoverHp {
                                source: responder,
                                player: dying,
                                amount: 1,
                            },
                            EffectRequest::ContinueDyingSave {
                                player: dying,
                                source: killer,
                                responders,
                            },
                    ]);
                    run_effects(tx, effects)?;
                } else if let Some((skill, kind)) = [CardKind::Tao, CardKind::Jiu]
                    .into_iter()
                    .filter(|kind| *kind != CardKind::Jiu || responder == dying)
                    .find_map(|kind| {
                        super::skill::source_free_response_skill(tx.state(), responder, kind, pick)
                            .map(|skill| (skill, kind))
                    })
                {
                    return super::skill::begin_source_free_response(
                        tx, responder, skill, kind, 5, &payload,
                    );
                } else if let Some((skill, kind)) = [CardKind::Tao, CardKind::Jiu]
                    .into_iter()
                    .filter(|kind| *kind != CardKind::Jiu || responder == dying)
                    .find_map(|kind| {
                        super::skill::multi_card_response_skill(tx.state(), responder, kind, pick)
                            .map(|skill| (skill, kind))
                    })
                {
                    return super::skill::begin_multi_card_response(
                        tx, responder, skill, kind, 5, &payload,
                    );
                } else {
                    return Err(EffectError::Illegal("invalid dying-save option".into()));
                }
                resume_interrupted_continuation(tx)
            }
            ContinuationKind::DiscardToLimit => {
                let player = PlayerId::new(*payload.first().unwrap_or(&0));
                let need = *payload.get(1).unwrap_or(&0) as usize;
                if cancelled {
                    return Err(EffectError::Illegal("discard cannot cancel".into()));
                }
                if selections.len() != need {
                    return Err(EffectError::Illegal(format!(
                        "must discard exactly {need} cards"
                    )));
                }
                let legal: std::collections::BTreeSet<u32> =
                    crate::query::discard_to_limit_tokens(tx.state(), player)
                        .into_iter()
                        .map(|card| card.raw())
                        .collect();
                let mut cards = Vec::new();
                for token in selections {
                    let cid = CardId::new(*token);
                    let c = tx
                        .state()
                        .cards
                        .get(cid)
                        .ok_or_else(|| EffectError::Illegal("bad discard card".into()))?;
                    if c.owner != player {
                        return Err(EffectError::Illegal(
                            "discard card not owned by player".into(),
                        ));
                    }
                    if !legal.contains(token) {
                        return Err(EffectError::Illegal(
                            "card cannot be discarded for hand limit".into(),
                        ));
                    }
                    cards.push(cid);
                }
                let mut seen = std::collections::BTreeSet::new();
                for c in &cards {
                    if !seen.insert(c.raw()) {
                        return Err(EffectError::Illegal("duplicate discard".into()));
                    }
                }
                run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }])?;
                use crate::state::Phase;
                tx.state_mut().turn.phase = Phase::End;
                let current = tx.state().turn.current;
                tx.push_event(engine_api::DomainEvent::TurnStateChanged {
                    current,
                    phase: Phase::End.to_tag(),
                });
                Ok(())
            }
            ContinuationKind::ReorderDrawPile => {
                // payload: [player, take, c0..] where c0.. are the peeked top cards
                // listed bottom→top (draw order last). Selected tokens sink to the
                // bottom (in selection order); the rest keep their original order
                // on top. Cancel = keep all on top unchanged.
                let take = *payload.get(1).unwrap_or(&0) as usize;
                let peeked: Vec<u32> = payload.iter().skip(2).take(take).copied().collect();
                let selected: std::collections::BTreeSet<u32> = if cancelled {
                    Default::default()
                } else {
                    selections.iter().copied().collect()
                };
                // Validate selections belong to the peeked set.
                for s in &selected {
                    if !peeked.contains(s) {
                        return Err(EffectError::Illegal("scry: card not among peeked".into()));
                    }
                }
                let draw = tx
                    .state()
                    .zones
                    .find(ZoneKind::DrawPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
                let z = tx
                    .state()
                    .zones
                    .get(draw)
                    .ok_or_else(|| EffectError::EngineFault("draw zone missing".into()))?;
                let total = z.cards.len();
                if total < take {
                    return Err(EffectError::EngineFault("scry: pile shrank".into()));
                }
                // `peeked` is bottom→top; iterate top→bottom for stable "keep on top".
                let mut keep_top: Vec<u32> = Vec::new(); // will be applied top-most first
                let mut to_bottom: Vec<u32> = Vec::new();
                for &cid in peeked.iter().rev() {
                    if selected.contains(&cid) {
                        to_bottom.push(cid);
                    } else {
                        keep_top.push(cid);
                    }
                }
                // Rebuild (stored bottom→top): [ selected (new bottom) ][ untouched
                // middle ][ unselected (top, original order preserved) ]. Both
                // keep_top and to_bottom are currently top→bottom, so reverse each to
                // store bottom→top; the original top-most card ends up last (drawn
                // first). Selected-card relative order follows the peek, not raw
                // click order — a deterministic simplification of full free reorder.
                let z = tx.state_mut().zones.get_mut(draw).unwrap();
                let middle: Vec<engine_api::CardId> = z.cards[..total - take].to_vec();
                let mut rebuilt: Vec<engine_api::CardId> = Vec::with_capacity(total);
                for cid in to_bottom.iter().rev() {
                    rebuilt.push(engine_api::CardId::new(*cid));
                }
                rebuilt.extend(middle);
                for cid in keep_top.iter().rev() {
                    rebuilt.push(engine_api::CardId::new(*cid));
                }
                z.cards = rebuilt;
                use crate::state::Phase;
                tx.state_mut().turn.phase = Phase::Judge;
                let current = tx.state().turn.current;
                tx.push_event(engine_api::DomainEvent::TurnStateChanged {
                    current,
                    phase: Phase::Judge.to_tag(),
                });
                Ok(())
            }
            ContinuationKind::ReorderDrawPileFourSplitTwoThenDraw => {
                let player = PlayerId::new(*payload.first().unwrap_or(&0));
                let take = *payload.get(1).unwrap_or(&0) as usize;
                let peeked: Vec<u32> = payload.iter().skip(2).take(take).copied().collect();
                if !cancelled {
                    if take < 2 || selections.len() != take {
                        return Err(EffectError::Illegal(
                            "draw-phase scry: must order every offered card".into(),
                        ));
                    }
                    let selected: std::collections::BTreeSet<u32> =
                        selections.iter().copied().collect();
                    let offered: std::collections::BTreeSet<u32> = peeked.iter().copied().collect();
                    if selected.len() != take || selected != offered {
                        return Err(EffectError::Illegal(
                            "draw-phase scry: selection must match offered cards".into(),
                        ));
                    }
                    let draw = tx
                        .state()
                        .zones
                        .find(ZoneKind::DrawPile, PlayerId::NONE)
                        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
                    let zone = tx
                        .state()
                        .zones
                        .get(draw)
                        .ok_or_else(|| EffectError::EngineFault("draw zone missing".into()))?;
                    let total = zone.cards.len();
                    if total < take
                        || zone.cards[total - take..]
                            .iter()
                            .map(|card| card.raw())
                            .ne(peeked.iter().copied())
                    {
                        return Err(EffectError::EngineFault(
                            "draw-phase scry: draw pile changed while choice was open".into(),
                        ));
                    }
                    let top_count = take - 2;
                    let middle = zone.cards[..total - take].to_vec();
                    let mut rebuilt = Vec::with_capacity(total);
                    rebuilt.extend(selections[top_count..].iter().copied().map(CardId::new));
                    rebuilt.extend(middle);
                    rebuilt.extend(
                        selections[..top_count]
                            .iter()
                            .rev()
                            .copied()
                            .map(CardId::new),
                    );
                    tx.state_mut().zones.get_mut(draw).unwrap().cards = rebuilt;
                }
                super::turn::continue_draw_phase(tx, player)
            }
            ContinuationKind::DrawReplacement => {
                use crate::state::turn_flag;
                let player = PlayerId::new(*payload.first().unwrap_or(&0));
                let draw_n = *payload.get(1).unwrap_or(&2);
                let forgo = !cancelled && selections.contains(&1);
                if forgo {
                    tx.state_mut().turn.set_flag(turn_flag::DRAW_REPLACED, 1);
                }
                let draw_n = draw_n.saturating_sub(u32::from(forgo));
                if super::turn::open_optional_draw_bonus(tx, player, draw_n, true)? {
                    return Err(EffectError::ChoiceRequested);
                } else {
                    run_effects(
                        tx,
                        vec![EffectRequest::DrawCards {
                            player,
                            count: draw_n,
                        }],
                    )?;
                }
                super::turn::finish_replaced_draw_phase(tx);
                Ok(())
            }
            ContinuationKind::OptionalDrawBonus => {
                let player = PlayerId::new(*payload.first().unwrap_or(&0));
                let mut draw_n = *payload.get(1).unwrap_or(&2);
                let advance_after = *payload.get(2).unwrap_or(&0) != 0;
                let bonuses = payload.get(3..).unwrap_or_default();
                if !cancelled && selections.first().copied() == Some(1) {
                    draw_n = draw_n.saturating_add(*bonuses.first().unwrap_or(&0));
                }
                if bonuses.len() > 1 {
                    crate::effect::open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        vec![0, 1],
                        engine_api::ChoiceKind::Confirm,
                        ContinuationKind::OptionalDrawBonus,
                        [player.raw(), draw_n, u32::from(advance_after)]
                            .into_iter()
                            .chain(bonuses[1..].iter().copied())
                            .collect(),
                    )?;
                    return Err(EffectError::ChoiceRequested);
                }
                run_effects(
                    tx,
                    vec![EffectRequest::DrawCards {
                        player,
                        count: draw_n,
                    }],
                )?;
                if advance_after {
                    super::turn::finish_replaced_draw_phase(tx);
                }
                Ok(())
            }
            ContinuationKind::PickTargetCard => {
                // mode 0=过河 discard (hand/equip/judge), 1=take (hand/equip),
                // 2=take (equip), 3=take (hand), 10..14=typed take scope.
                let target = PlayerId::new(*payload.first().unwrap_or(&0));
                let actor = PlayerId::new(*payload.get(1).unwrap_or(&0));
                let mode = *payload.get(2).unwrap_or(&0);
                if cancelled {
                    return Err(EffectError::Illegal("must pick a card".into()));
                }
                if selections.len() != 1 {
                    return Err(EffectError::Illegal("pick exactly one card".into()));
                }
                let cid = CardId::new(selections[0]);
                let c = tx
                    .state()
                    .cards
                    .get(cid)
                    .ok_or_else(|| EffectError::Illegal("bad card".into()))?
                    .clone();
                if c.owner != target {
                    return Err(EffectError::Illegal("card not owned by target".into()));
                }
                let from_zone = c.zone;
                // Validate zone allowed
                let in_hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, target)
                    .map(|z| z == from_zone)
                    .unwrap_or(false);
                let in_equip = tx
                    .state()
                    .zones
                    .find(ZoneKind::Equip, target)
                    .map(|z| z == from_zone)
                    .unwrap_or(false);
                let in_judge = tx
                    .state()
                    .zones
                    .find(ZoneKind::Judge, target)
                    .map(|z| z == from_zone)
                    .unwrap_or(false);
                let take_scope = match mode {
                    1 => Some(rule_api::TargetCardScope::HandEquip),
                    2 => Some(rule_api::TargetCardScope::Equip),
                    3 => Some(rule_api::TargetCardScope::Hand),
                    10..=14 => {
                        rule_api::TargetCardScope::from_tag((mode - 10) as u8)
                    }
                    _ => None,
                };
                let result = if let Some(scope) = take_scope {
                    if !crate::query::gainable_target_cards(tx.state(), actor, target, scope)
                        .contains(&cid)
                    {
                        return Err(EffectError::Illegal("cannot take this card".into()));
                    }
                    let to_hand = tx
                        .state()
                        .zones
                        .find(ZoneKind::Hand, actor)
                        .ok_or_else(|| EffectError::Illegal("no actor hand".into()))?;
                    super::play::run_effects_then_finish_independent_trick_target(
                        tx,
                        vec![EffectRequest::MoveCards {
                            cards: vec![cid],
                            to: to_hand,
                            from: Some(from_zone),
                        }],
                        payload.get(3..).unwrap_or_default(),
                    )
                } else if mode == 20 {
                    // store to actor's skill pile (ABI 567): hand only
                    if !in_hand {
                        return Err(EffectError::Illegal(
                            "skill-pile storage requires a hand card".into(),
                        ));
                    }
                    let skill_id = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
                    let skill_def = crate::effect::skill_pile_def(tx, actor, skill_id)?;
                    let pile = crate::effect::ensure_skill_pile_zone(tx, actor, skill_def)?;
                    super::play::run_effects_then_finish_independent_trick_target(
                        tx,
                        vec![EffectRequest::MoveCards {
                            cards: vec![cid],
                            to: pile,
                            from: Some(from_zone),
                        }],
                        payload.get(4..).unwrap_or_default(),
                    )
                } else {
                    // discard: hand / equip / judge
                    if !in_hand && !in_equip && !in_judge {
                        return Err(EffectError::Illegal("cannot discard this card".into()));
                    }
                    if crate::query::card_discard_protected_from(tx.state(), actor, target, cid) {
                        return Err(EffectError::Illegal(
                            "card is protected from discard".into(),
                        ));
                    }
                    if in_judge {
                        let discard = tx
                            .state()
                            .zones
                            .find(ZoneKind::DiscardPile, PlayerId::NONE)
                            .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
                        super::play::run_effects_then_finish_independent_trick_target(
                            tx,
                            vec![EffectRequest::MoveCards {
                                cards: vec![cid],
                                to: discard,
                                from: Some(from_zone),
                            }],
                            payload.get(3..).unwrap_or_default(),
                        )
                    } else {
                        super::play::run_effects_then_finish_independent_trick_target(
                            tx,
                            vec![EffectRequest::DiscardCards {
                                player: target,
                                cards: vec![cid],
                            }],
                            payload.get(3..).unwrap_or_default(),
                        )
                    }
                };
                result
            }
            ContinuationKind::HuogongShow => {
                // payload: [actor, target, optional queued-trick tail...]
                let actor = PlayerId::new(*payload.first().unwrap_or(&0));
                let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
                if cancelled || selections.len() != 1 {
                    return Err(EffectError::Illegal("must show one card".into()));
                }
                let shown = CardId::new(selections[0]);
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, target)
                    .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
                let c = tx
                    .state()
                    .cards
                    .get(shown)
                    .ok_or_else(|| EffectError::Illegal("bad card".into()))?
                    .clone();
                if c.zone != hand || c.owner != target {
                    return Err(EffectError::Illegal("shown card not in hand".into()));
                }
                let suit = crate::query::effective_card_suit(tx.state(), target, &c);
                // Actor discards a same-suit card or cancels
                let actor_hand = tx.state().hand_cards(actor);
                let tokens: Vec<u32> = actor_hand
                    .iter()
                    .filter(|cid| {
                        tx.state()
                            .cards
                            .get(**cid)
                            .map(|x| {
                                crate::query::effective_card_suit(tx.state(), actor, x) == suit
                            })
                            .unwrap_or(false)
                    })
                    .map(|c| c.raw())
                    .collect();
                run_effects(
                    tx,
                    vec![EffectRequest::EmitRuleEvent {
                        code: 210, // huogong show
                        payload: vec![
                            actor.raw(),
                            target.raw(),
                            shown.raw(),
                            super::judge::suit_tag(suit),
                        ],
                    }],
                )?;
                if tokens.is_empty() {
                    // Cannot discard matching suit — fire fails
                    return super::play::finish_independent_trick_target(
                        tx,
                        payload.get(2..).unwrap_or_default(),
                    );
                }
                let mut next_payload =
                    vec![actor.raw(), target.raw(), super::judge::suit_tag(suit)];
                next_payload.extend_from_slice(payload.get(2..).unwrap_or_default());
                crate::effect::open_choice(
                    tx,
                    actor,
                    0,
                    1,
                    true, // cancel = no damage
                    tokens,
                    engine_api::ChoiceKind::SelectCards,
                    crate::state::ContinuationKind::HuogongDiscard,
                    next_payload,
                )?;
                Err(EffectError::ChoiceRequested)
            }
            ContinuationKind::HuogongDiscard => {
                // payload: [actor, target, suit_tag, optional queued-trick tail...]
                let actor = PlayerId::new(*payload.first().unwrap_or(&0));
                let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
                let suit = super::judge::suit_from_tag(*payload.get(2).unwrap_or(&1));
                if cancelled || selections.is_empty() {
                    return super::play::finish_independent_trick_target(
                        tx,
                        payload.get(3..).unwrap_or_default(),
                    );
                }
                if selections.len() != 1 {
                    return Err(EffectError::Illegal("discard one card".into()));
                }
                let cid = CardId::new(selections[0]);
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, actor)
                    .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
                let c = tx
                    .state()
                    .cards
                    .get(cid)
                    .ok_or_else(|| EffectError::Illegal("bad card".into()))?
                    .clone();
                if c.zone != hand
                    || c.owner != actor
                    || crate::query::effective_card_suit(tx.state(), actor, &c) != suit
                {
                    return Err(EffectError::Illegal("must discard same suit".into()));
                }
                super::play::run_effects_then_finish_independent_trick_target(
                    tx,
                    vec![
                        EffectRequest::DiscardCards {
                            player: actor,
                            cards: vec![cid],
                        },
                        {
                            let (card, bonus) =
                                super::play::saved_independent_trick_damage_card_and_bonus(
                                    payload.get(3..).unwrap_or_default(),
                                );
                            if card.is_some() {
                                EffectRequest::DealCardDamage {
                                    source: actor,
                                    target,
                                    amount: 1u32.saturating_add(bonus),
                                    is_lose_hp: false,
                                    nature: rule_api::damage_nature::FIRE,
                                    card,
                                    card_kind: CardKind::TrickHuogong.tag(),
                                    card_flags: 0,
                                }
                            } else {
                                EffectRequest::DealDamage {
                                    source: actor,
                                    target,
                                    amount: 1,
                                    is_lose_hp: false,
                                    nature: rule_api::damage_nature::FIRE,
                                }
                            }
                        },
                        EffectRequest::EmitRuleEvent {
                            code: 211, // huogong hit
                            payload: vec![actor.raw(), target.raw(), cid.raw()],
                        },
                    ],
                    payload.get(3..).unwrap_or_default(),
                )
            }
            ContinuationKind::RespondToSha => {
                // payload: [source, target, amount, need, have, nature, respondable, card]
                let source = PlayerId::new(*payload.first().unwrap_or(&0));
                let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
                let amount = *payload.get(2).unwrap_or(&1);
                let need = *payload.get(3).unwrap_or(&1);
                let have = *payload.get(4).unwrap_or(&0);
                let damage_card_context = *payload.get(5).unwrap_or(&0);
                let (nature, card_flags) =
                    super::play::unpack_damage_card_context(damage_card_context);
                let respondable = *payload.get(6).unwrap_or(&1) != 0;
                let card = CardId::new(*payload.get(7).unwrap_or(&0));
                if cancelled || selections.is_empty() {
                    return super::play::deal_single_sha_damage(
                        tx, source, target, amount, nature, card_flags, card,
                    );
                }
                if selections.len() != 1 {
                    return Err(EffectError::Illegal("play at most one 闪".into()));
                }
                let token = selections[0];
                if have == 0
                    && super::play::response_youdishenru_tokens(tx.state(), target)
                        .contains(&token)
                {
                    return super::play::use_youdishenru_for_saved_sha(
                        tx,
                        target,
                        source,
                        card,
                        CardId::new(token),
                        ContinuationKind::RespondToSha,
                        &payload,
                    );
                }
                // 八卦：判定红心当闪
                if token == crate::content::BAGUA_RESPONSE_TOKEN {
                    let (ok, judgment) = super::play::try_bagua_as_shan(tx, target)?;
                    if ok {
                        let next_have = have + 1;
                        let resume_payload = if next_have < need {
                            vec![
                                source.raw(),
                                target.raw(),
                                amount,
                                need,
                                next_have,
                                damage_card_context,
                                u32::from(respondable),
                                card.raw(),
                            ]
                        } else {
                            Vec::new()
                        };
                        if next_have >= need {
                            super::play::save_played_sha_finish(
                                tx,
                                source,
                                card,
                                damage_card_context,
                            )?;
                        }
                        let mut effects = vec![judgment.clone()];
                        effects.extend(sha_response_effects(
                            target,
                            source,
                            CardId::NONE,
                            card,
                            next_have,
                            need,
                        ));
                        run_response_effects(
                            tx,
                            effects,
                            ContinuationKind::RespondToSha,
                            resume_payload,
                        )?;
                        if next_have < need {
                            return super::play::open_sha_response_step(
                                tx,
                                source,
                                target,
                                amount,
                                need,
                                next_have,
                                nature,
                                card_flags,
                                respondable,
                                card,
                            );
                        }
                        return super::play::finish_saved_played_sha(tx, source, card);
                    }
                    run_response_effects(
                        tx,
                        vec![judgment],
                        ContinuationKind::RespondToSha,
                        payload.clone(),
                    )?;
                    // fail → ask again (may play real 闪 or retry 八卦)
                    return super::play::open_sha_response_step(
                        tx,
                        source,
                        target,
                        amount,
                        need,
                        have,
                        nature,
                        card_flags,
                        respondable,
                        card,
                    );
                }
                if let Some(skill) = super::skill::source_free_response_skill(
                    tx.state(),
                    target,
                    CardKind::Shan,
                    token,
                ) {
                    return super::skill::begin_source_free_response(
                        tx,
                        target,
                        skill,
                        CardKind::Shan,
                        1,
                        &payload,
                    );
                }
                if let Some(skill) = super::skill::multi_card_response_skill(
                    tx.state(),
                    target,
                    CardKind::Shan,
                    token,
                ) {
                    return super::skill::begin_multi_card_response(
                        tx,
                        target,
                        skill,
                        CardKind::Shan,
                        1,
                        &payload,
                    );
                }
                let cid = CardId::new(token);
                let legal = super::play::response_shan_tokens(tx.state(), target);
                if !legal.contains(&cid.raw()) {
                    return Err(EffectError::Illegal("not a legal 闪 response".into()));
                }
                let from = response_card_zone(tx, target, cid)?;
                tx.state_mut()
                    .resolution
                    .register_sha_response_card(source, card, cid);
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
                let next_have = have + 1;
                let resume_payload = if next_have < need {
                    vec![
                        source.raw(),
                        target.raw(),
                        amount,
                        need,
                        next_have,
                        damage_card_context,
                        u32::from(respondable),
                        card.raw(),
                    ]
                } else {
                    Vec::new()
                };
                if next_have >= need {
                    super::play::save_played_sha_finish(tx, source, card, damage_card_context)?;
                }
                let mut response_effects = vec![EffectRequest::MoveCards {
                    cards: vec![cid],
                    to: discard,
                    from: Some(from),
                }];
                response_effects.extend(sha_response_effects(
                    target, source, cid, card, next_have, need,
                ));
                response_effects.push(EffectRequest::EmitRuleEvent {
                    code: 121,
                    payload: vec![source.raw(), target.raw(), cid.raw(), next_have, need],
                });
                run_physical_response_effects(
                    tx,
                    response_effects,
                    ContinuationKind::RespondToSha,
                    resume_payload,
                    target,
                    cid,
                    CardKind::Shan,
                )?;
                if next_have < need {
                    return super::play::open_sha_response_step(
                        tx,
                        source,
                        target,
                        amount,
                        need,
                        next_have,
                        nature,
                        card_flags,
                        respondable,
                        card,
                    );
                }
                super::play::finish_saved_played_sha(tx, source, card)
            }
            ContinuationKind::RespondAoeQueue => {
                let kind = *payload.first().unwrap_or(&0);
                if kind == 3 {
                    return resolve_multi_sha_queue_choice(tx, &payload, selections, cancelled);
                }
                let source = PlayerId::new(*payload.get(1).unwrap_or(&0));
                let amount = *payload.get(2).unwrap_or(&1);
                let idx = *payload.get(3).unwrap_or(&0) as usize;
                let target = PlayerId::new(*payload.get(5 + idx).unwrap_or(&0));
                if !cancelled && !selections.is_empty() {
                    if selections.len() != 1 {
                        return Err(EffectError::Illegal("play one response card".into()));
                    }
                    let token = selections[0];
                    // 万箭：八卦当闪
                    if kind == 2 && token == crate::content::BAGUA_RESPONSE_TOKEN {
                        let (ok, judgment) = super::play::try_bagua_as_shan(tx, target)?;
                        if ok {
                            let mut next = payload.clone();
                            next[3] = (idx + 1) as u32;
                            run_response_effects(
                                tx,
                                vec![
                                    judgment.clone(),
                                    EffectRequest::CardResponded {
                                        player: target,
                                        respond_to: source,
                                        card: CardId::NONE,
                                        response_kind: CardKind::Shan.tag(),
                                    },
                                ],
                                ContinuationKind::RespondAoeQueue,
                                next,
                            )?;
                        }
                        if !ok {
                            run_response_effects(
                                tx,
                                vec![judgment],
                                ContinuationKind::RespondAoeQueue,
                                payload.clone(),
                            )?;
                            // fail: re-open same target
                            let tokens = super::play::response_shan_options(tx.state(), target);
                            if tokens.is_empty() {
                                run_effects(
                                    tx,
                                    vec![EffectRequest::DealDamage {
                                        source,
                                        target,
                                        amount,
                                        is_lose_hp: false,
                                        nature: 0,
                                    }],
                                )?;
                            } else {
                                crate::effect::open_choice(
                                    tx,
                                    target,
                                    0,
                                    1,
                                    true,
                                    tokens,
                                    engine_api::ChoiceKind::SelectCards,
                                    ContinuationKind::RespondAoeQueue,
                                    payload.clone(),
                                )?;
                                return Err(EffectError::ChoiceRequested);
                            }
                        }
                        // success or damage applied after fail-empty → advance
                    } else {
                        let response_kind = if kind == 1 {
                            CardKind::Sha
                        } else {
                            CardKind::Shan
                        };
                        if let Some(skill) = super::skill::source_free_response_skill(
                            tx.state(),
                            target,
                            response_kind,
                            token,
                        ) {
                            return super::skill::begin_source_free_response(
                                tx,
                                target,
                                skill,
                                response_kind,
                                2,
                                &payload,
                            );
                        }
                        if let Some(skill) = super::skill::multi_card_response_skill(
                            tx.state(),
                            target,
                            response_kind,
                            token,
                        ) {
                            return super::skill::begin_multi_card_response(
                                tx,
                                target,
                                skill,
                                response_kind,
                                2,
                                &payload,
                            );
                        }
                        let cid = CardId::new(token);
                        let legal = match kind {
                            1 => super::play::response_sha_tokens(tx.state(), target),
                            2 => super::play::response_shan_tokens(tx.state(), target),
                            _ => vec![],
                        };
                        if !legal.contains(&cid.raw()) {
                            return Err(EffectError::Illegal("illegal response card".into()));
                        }
                        let from = response_card_zone(tx, target, cid)?;
                        let discard = tx
                            .state()
                            .zones
                            .find(ZoneKind::DiscardPile, PlayerId::NONE)
                            .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
                        let response_kind = if kind == 1 {
                            CardKind::Sha
                        } else {
                            CardKind::Shan
                        };
                        let mut next = payload.clone();
                        next[3] = (idx + 1) as u32;
                        run_physical_response_effects(
                            tx,
                            vec![
                                EffectRequest::MoveCards {
                                    cards: vec![cid],
                                    to: discard,
                                    from: Some(from),
                                },
                                EffectRequest::CardResponded {
                                    player: target,
                                    respond_to: source,
                                    card: cid,
                                    response_kind: response_kind.tag(),
                                },
                                EffectRequest::EmitRuleEvent {
                                    code: 123,
                                    payload: vec![kind, source.raw(), target.raw(), cid.raw()],
                                },
                            ],
                            ContinuationKind::RespondAoeQueue,
                            next,
                            target,
                            cid,
                            response_kind,
                        )?;
                    }
                } else {
                    run_effects(
                        tx,
                        vec![EffectRequest::DealDamage {
                            source,
                            target,
                            amount,
                            is_lose_hp: false,
                            nature: 0,
                        }],
                    )?;
                }
                if tx.state().resolution.continuation_kind == ContinuationKind::DyingSave {
                    if tx.state().resolution.resume_kind == ContinuationKind::RespondAoeQueue
                        && tx.state().resolution.resume_payload.len() > 3
                    {
                        tx.state_mut().resolution.resume_payload[3] = (idx + 1) as u32;
                    }
                    return Ok(());
                }
                let mut next = payload.clone();
                next[3] = (idx + 1) as u32;
                tx.state_mut().resolution.continuation_kind = ContinuationKind::RespondAoeQueue;
                tx.state_mut().resolution.continuation_payload = next;
                super::play::advance_aoe_queue(tx)
            }
            ContinuationKind::JuedouDuel => {
                let source = PlayerId::new(*payload.first().unwrap_or(&0));
                let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
                let next = PlayerId::new(*payload.get(2).unwrap_or(&0));
                let exchanges = *payload.get(3).unwrap_or(&0);
                let need = *payload.get(4).unwrap_or(&1);
                let have = *payload.get(5).unwrap_or(&0);
                let tail = payload.get(6..).unwrap_or_default();
                if cancelled || selections.is_empty() {
                    let dmg_source = if next == target { source } else { target };
                    let amount = crate::query::sha_damage_for(tx.state(), dmg_source, 1);
                    return super::play::finish_juedou_with_tail(
                        tx, source, target, next, amount, tail,
                    );
                }
                if selections.len() != 1 {
                    return Err(EffectError::Illegal("play one 杀".into()));
                }
                if let Some(skill) = super::skill::source_free_response_skill(
                    tx.state(),
                    next,
                    CardKind::Sha,
                    selections[0],
                ) {
                    return super::skill::begin_source_free_response(
                        tx,
                        next,
                        skill,
                        CardKind::Sha,
                        3,
                        &payload,
                    );
                }
                if let Some(skill) = super::skill::multi_card_response_skill(
                    tx.state(),
                    next,
                    CardKind::Sha,
                    selections[0],
                ) {
                    return super::skill::begin_multi_card_response(
                        tx,
                        next,
                        skill,
                        CardKind::Sha,
                        3,
                        &payload,
                    );
                }
                let cid = CardId::new(selections[0]);
                let legal = super::play::response_sha_tokens(tx.state(), next);
                if !legal.contains(&cid.raw()) {
                    return Err(EffectError::Illegal("not a legal 杀".into()));
                }
                let from = response_card_zone(tx, next, cid)?;
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
                let resume = super::play::juedou_payload_after_response(
                    source, target, next, exchanges, need, have, tail,
                );
                run_physical_response_effects(
                    tx,
                    vec![
                        EffectRequest::MoveCardsWithReason {
                            cards: vec![cid],
                            to: discard,
                            from: Some(from),
                            reason: CardMoveReason::Respond,
                        },
                        EffectRequest::CardResponded {
                            player: next,
                            respond_to: source,
                            card: cid,
                            response_kind: CardKind::Sha.tag(),
                        },
                        EffectRequest::EmitRuleEvent {
                            code: 124,
                            payload: vec![source.raw(), target.raw(), next.raw(), cid.raw()],
                        },
                    ],
                    ContinuationKind::JuedouDuel,
                    resume.clone(),
                    next,
                    cid,
                    CardKind::Sha,
                )?;
                super::play::continue_juedou_payload(tx, &resume)
            }
            ContinuationKind::RewriteJudgment => {
                // payload: [owner, delayed, result_card, suit, rank, kind_tag, rewriter_len,
                //           rewriters..., rest_len, delayed_rest...]
                let owner = PlayerId::new(*payload.first().unwrap_or(&0));
                let delayed = CardId::new(*payload.get(1).unwrap_or(&0));
                let mut result_card = CardId::new(*payload.get(2).unwrap_or(&0));
                let mut suit = super::judge::suit_from_tag(*payload.get(3).unwrap_or(&1));
                let mut rank = *payload.get(4).unwrap_or(&1) as u8;
                let kind = super::judge::kind_from_tag(*payload.get(5).unwrap_or(&0));
                let rewriter_len = *payload.get(6).unwrap_or(&0) as usize;
                let mut rewriters = Vec::new();
                for index in 0..rewriter_len {
                    if let Some(player) = payload.get(7 + index) {
                        rewriters.push(PlayerId::new(*player));
                    }
                }
                let Some(rewriter) = rewriters.first().copied() else {
                    return Err(EffectError::Illegal("missing judgment rewriter".into()));
                };
                let rest_len_index = 7 + rewriter_len;
                let rest_len = *payload.get(rest_len_index).unwrap_or(&0) as usize;
                let mut rest = Vec::new();
                for index in 0..rest_len {
                    if let Some(id) = payload.get(rest_len_index + 1 + index) {
                        rest.push(CardId::new(*id));
                    }
                }
                if !cancelled && !selections.is_empty() {
                    if selections.len() != 1 {
                        return Err(EffectError::Illegal("pick one card to rewrite".into()));
                    }
                    let cid = CardId::new(selections[0]);
                    if !super::judge::judgment_rewrite_cards(tx, rewriter).contains(&cid) {
                        return Err(EffectError::Illegal(
                            "card is not legal for judgment rewrite".into(),
                        ));
                    }
                    let c = tx
                        .state()
                        .cards
                        .get(cid)
                        .ok_or_else(|| EffectError::Illegal("bad card".into()))?
                        .clone();
                    let from = response_card_zone(tx, rewriter, cid)?;
                    result_card = cid;
                    suit = crate::query::effective_card_suit(tx.state(), owner, &c);
                    rank = c.rank;
                    let discard = tx
                        .state()
                        .zones
                        .find(ZoneKind::DiscardPile, PlayerId::NONE)
                        .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
                    let mut effects = Vec::new();
                    if let Some(mark) = crate::query::judgment_rewrite_mark_cost_for_card(
                        tx.state(),
                        rewriter,
                        &c,
                    )
                    {
                        effects.push(EffectRequest::RemoveMark {
                            player: rewriter,
                            mark: engine_api::MarkDefId::new(mark),
                            amount: 1,
                        });
                    }
                    effects.extend([
                        EffectRequest::MoveCards {
                            cards: vec![cid],
                            to: discard,
                            from: Some(from),
                        },
                        EffectRequest::EmitRuleEvent {
                            code: 201, // guicai rewrite
                            payload: vec![
                                rewriter.raw(),
                                owner.raw(),
                                delayed.raw(),
                                super::judge::suit_tag(suit),
                                rank as u32,
                            ],
                        },
                    ]);
                    run_effects(
                        tx,
                        effects,
                    )?;
                }
                super::judge::continue_judgment_rewriters(
                    tx,
                    owner,
                    delayed,
                    kind,
                    result_card,
                    suit,
                    rank,
                    &rewriters[1..],
                    &rest,
                )
            }
            ContinuationKind::RewriteBeigeJudgment => {
                if payload.len() < 8 {
                    return Err(EffectError::EngineFault(
                        "truncated Beige judgment rewrite".into(),
                    ));
                }
                let player = PlayerId::new(payload[0]);
                let target = PlayerId::new(payload[1]);
                let source = PlayerId::new(payload[2]);
                let mut result_card = CardId::new(payload[3]);
                let mut suit = super::judge::suit_from_tag(payload[4]);
                let mut rank = payload[5] as u8;
                let skill = engine_api::SkillId::new(payload[6]);
                let rewriter_len = payload[7] as usize;
                let rewriter_end = 8usize.saturating_add(rewriter_len);
                if rewriter_len == 0 || payload.len() < rewriter_end {
                    return Err(EffectError::EngineFault(
                        "truncated Beige judgment rewriter queue".into(),
                    ));
                }
                let configured_counts = match payload.len().saturating_sub(rewriter_end) {
                    0 => None,
                    3 if payload[rewriter_end] == 1 => {
                        Some((payload[rewriter_end + 1], payload[rewriter_end + 2]))
                    }
                    _ => {
                        return Err(EffectError::EngineFault(
                            "invalid Beige judgment policy suffix".into(),
                        ));
                    }
                };
                let rewriters: Vec<_> = payload[8..rewriter_end]
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect();
                let rewriter = rewriters[0];
                if observer != rewriter {
                    return Err(EffectError::Illegal(
                        "only the current player may rewrite Beige judgment".into(),
                    ));
                }
                if cancelled || selections.is_empty() {
                    return match configured_counts {
                        Some((heart_recover_amount, diamond_draw_count)) => {
                            crate::effect::continue_beige_judgment_rewriters_configured(
                                tx,
                                player,
                                target,
                                source,
                                result_card,
                                suit,
                                rank,
                                &rewriters[1..],
                                heart_recover_amount,
                                diamond_draw_count,
                                skill,
                            )
                        }
                        None => crate::effect::continue_beige_judgment_rewriters(
                            tx,
                            player,
                            target,
                            source,
                            result_card,
                            suit,
                            rank,
                            &rewriters[1..],
                            skill,
                        ),
                    }
                    .map(|_| ());
                }
                if selections.len() != 1 {
                    return Err(EffectError::Illegal(
                        "pick one card to rewrite Beige judgment".into(),
                    ));
                }
                let cid = CardId::new(selections[0]);
                if !super::judge::judgment_rewrite_cards(tx, rewriter).contains(&cid) {
                    return Err(EffectError::Illegal(
                        "card is not legal for Beige judgment rewrite".into(),
                    ));
                }
                let card = tx
                    .state()
                    .cards
                    .get(cid)
                    .ok_or_else(|| EffectError::Illegal("bad rewrite card".into()))?
                    .clone();
                let from = response_card_zone(tx, rewriter, cid)?;
                result_card = cid;
                suit = crate::query::effective_card_suit(tx.state(), target, &card);
                rank = card.rank;
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
                tx.state_mut()
                    .resolution
                    .deferred_tail_effects
                    .push(match configured_counts {
                        Some((heart_recover_amount, diamond_draw_count)) => {
                            EffectRequest::ResolveBeigeConfigured {
                                player,
                                target,
                                source,
                                stage: 4,
                                result_card,
                                suit,
                                rank,
                                rewriters: rewriters[1..].to_vec(),
                                heart_recover_amount,
                                diamond_draw_count,
                                skill,
                            }
                        }
                        None => EffectRequest::ResolveBeige {
                            player,
                            target,
                            source,
                            stage: 4,
                            result_card,
                            suit,
                            rank,
                            rewriters: rewriters[1..].to_vec(),
                            skill,
                        },
                    });
                let mut effects = Vec::new();
                if let Some(mark) = crate::query::judgment_rewrite_mark_cost_for_card(
                    tx.state(),
                    rewriter,
                    &card,
                ) {
                    effects.push(EffectRequest::RemoveMark {
                        player: rewriter,
                        mark: engine_api::MarkDefId::new(mark),
                        amount: 1,
                    });
                }
                effects.extend([
                    EffectRequest::MoveCards {
                        cards: vec![cid],
                        to: discard,
                        from: Some(from),
                    },
                    EffectRequest::EmitRuleEvent {
                        code: 201,
                        payload: vec![
                            rewriter.raw(),
                            target.raw(),
                            CardId::NONE.raw(),
                            super::judge::suit_tag(suit),
                            u32::from(rank),
                        ],
                    },
                ]);
                run_effects(tx, effects).map(|_| ())
            }
            ContinuationKind::JiedaoChoice => {
                // payload: [actor, holder, victim]; selection 0=give weapon, 1=sha
                let actor = PlayerId::new(*payload.first().unwrap_or(&0));
                let holder = PlayerId::new(*payload.get(1).unwrap_or(&0));
                let victim = PlayerId::new(*payload.get(2).unwrap_or(&0));
                if cancelled || selections.is_empty() {
                    return Err(EffectError::Illegal("jiedao must choose".into()));
                }
                let pick = selections[0];
                if pick == 1 {
                    // Holder uses 杀 on victim (with 闪 response)
                    return super::play::begin_sha_response(
                        tx, holder, victim, 1, 0, 0, true, None,
                    );
                }
                jiedao_give_weapon(tx, actor, holder)
            }
            ContinuationKind::WuxieWindow => {
                // payload: [nullified, ask_idx, ask_len, p0.., effect_tag, actor, respond_to,
                // response_kind, response_suit, ...]
                let ask_idx = *payload.get(1).unwrap_or(&0) as usize;
                let ask_len = *payload.get(2).unwrap_or(&0) as usize;
                let observer = if ask_len > 0 && 3 + ask_idx < payload.len() {
                    PlayerId::new(payload[3 + ask_idx])
                } else {
                    PlayerId::NONE
                };
                if cancelled || selections.is_empty() {
                    // Pass → next asker
                    let mut p = payload.clone();
                    if p.len() > 1 {
                        p[1] = (ask_idx + 1) as u32;
                    }
                    tx.state_mut().resolution.continuation_kind = ContinuationKind::WuxieWindow;
                    tx.state_mut().resolution.continuation_payload = p;
                    return super::play::advance_wuxie_window(tx);
                }
                if selections.len() != 1 {
                    return Err(EffectError::Illegal("play one 无懈".into()));
                }
                if let Some(skill) = super::skill::source_free_response_skill(
                    tx.state(),
                    observer,
                    CardKind::TrickWuxie,
                    selections[0],
                ) {
                    return super::skill::begin_source_free_response(
                        tx,
                        observer,
                        skill,
                        CardKind::TrickWuxie,
                        4,
                        &payload,
                    );
                }
                if let Some(skill) = super::skill::multi_card_response_skill(
                    tx.state(),
                    observer,
                    CardKind::TrickWuxie,
                    selections[0],
                ) {
                    return super::skill::begin_multi_card_response(
                        tx,
                        observer,
                        skill,
                        CardKind::TrickWuxie,
                        4,
                        &payload,
                    );
                }
                let cid = CardId::new(selections[0]);
                let legal = super::play::wuxie_tokens_for_payload(tx.state(), observer, &payload);
                if !legal.contains(&cid.raw()) {
                    return Err(EffectError::Illegal("not a 无懈 in hand".into()));
                }
                let response_suit = tx
                    .state()
                    .cards
                    .get(cid)
                    .map(|card| crate::query::effective_card_suit(tx.state(), observer, card))
                    .ok_or_else(|| EffectError::Illegal("bad 无懈 card".into()))?;
                let from = response_card_zone(tx, observer, cid)?;
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
                // Precompute the next wuxie window so an optional response trigger can
                // pause here without losing the parity/seat continuation.
                let nullified = payload.first().copied().unwrap_or(0) ^ 1;
                let effect_off = 3 + ask_len;
                let effect_tag = *payload.get(effect_off).unwrap_or(&0);
                let actor = PlayerId::new(*payload.get(effect_off + 1).unwrap_or(&0));
                let respond_to = PlayerId::new(*payload.get(effect_off + 2).unwrap_or(&0));
                let effect_args: Vec<u32> = if payload.len() > effect_off + 5 {
                    payload[effect_off + 5..].to_vec()
                } else {
                    vec![]
                };
                let start = {
                    let order = tx.state().players.alive_by_seat();
                    let pos = order.iter().position(|p| *p == observer).unwrap_or(0);
                    if order.is_empty() {
                        observer
                    } else {
                        order[(pos + 1) % order.len()]
                    }
                };
                let mut resume_payload = vec![
                    start.raw(),
                    nullified,
                    effect_tag,
                    actor.raw(),
                    observer.raw(),
                    CardKind::TrickWuxie.tag() as u32,
                    super::judge::suit_tag(response_suit),
                ];
                resume_payload.extend_from_slice(&effect_args);
                let mut effects = vec![EffectRequest::MoveCardsWithReason {
                    cards: vec![cid],
                    to: discard,
                    from: Some(from),
                    reason: CardMoveReason::Respond,
                }];
                append_pothaoshi_refills(tx.state(), observer, &[cid], &mut effects);
                effects.push(EffectRequest::CardResponded {
                    player: observer,
                    respond_to,
                    card: cid,
                    response_kind: CardKind::TrickWuxie.tag(),
                });
                effects.push(EffectRequest::EmitRuleEvent {
                    code: 221, // 无懈 used
                    payload: vec![observer.raw(), cid.raw()],
                });
                run_response_effects(
                    tx,
                    effects,
                    ContinuationKind::WuxieWindow,
                    resume_payload,
                )?;
                // Toggle nullify parity and re-open ask from next seat after 无懈 user
                super::play::begin_wuxie_window_with_parity(
                    tx,
                    start,
                    nullified,
                    effect_tag,
                    actor,
                    observer,
                    CardKind::TrickWuxie,
                    response_suit,
                    &effect_args,
                )
            }
            ContinuationKind::ActiveSkillSelection => {
                if cancelled {
                    return Ok(());
                }
                let skill = engine_api::SkillId::new(*payload.first().unwrap_or(&0));
                let target_count = *payload.get(1).unwrap_or(&0) as usize;
                let targets: Vec<PlayerId> = payload
                    .iter()
                    .skip(2)
                    .take(target_count)
                    .copied()
                    .map(PlayerId::new)
                    .collect();
                let cards: Vec<CardId> = selections.iter().copied().map(CardId::new).collect();
                super::skill::handle_activate_skill(tx, observer, skill, &targets, &cards)
            }
            ContinuationKind::ChooseDrawOrRecover => {
                let player = PlayerId::new(*payload.first().unwrap_or(&0));
                let draw_count = *payload.get(1).unwrap_or(&1);
                let recover_count = *payload.get(2).unwrap_or(&1);
                let allow_cancel = *payload.get(3).unwrap_or(&0) != 0;
                let recover_target =
                    PlayerId::new(payload.get(4).copied().unwrap_or(player.raw()));
                if cancelled {
                    return if allow_cancel {
                        Ok(())
                    } else {
                        Err(EffectError::Illegal(
                            "draw-or-recover choice cannot cancel".into(),
                        ))
                    };
                }
                let effect = match selections {
                    [1] => EffectRequest::DrawCards {
                        player,
                        count: draw_count,
                    },
                    [2] => EffectRequest::RecoverHp {
                        source: recover_target,
                        player: recover_target,
                        amount: recover_count,
                    },
                    _ => {
                        return Err(EffectError::Illegal(
                            "choose draw or recover exactly once".into(),
                        ));
                    }
                };
                run_effects(tx, vec![effect]).map(|_| ())
            }
            ContinuationKind::VirtualShaTargets => {
                            let player = PlayerId::new(*payload.first().unwrap_or(&0));
                let card = CardId::new(*payload.get(1).unwrap_or(&0));
                let ignore_distance = *payload.get(2).unwrap_or(&0) != 0;
                let ignore_use_count = *payload.get(3).unwrap_or(&0) != 0;
                let draw_on_no_use = *payload.get(4).unwrap_or(&0) != 0;
                let skill = engine_api::SkillId::new(*payload.get(5).unwrap_or(&0));
                let lose_hp_before_use = *payload.get(6).unwrap_or(&0);
                let damage_bonus = *payload.get(7).unwrap_or(&0);
                if cancelled {
                    let processing = tx
                        .state()
                        .zones
                        .find(crate::state::ZoneKind::Processing, PlayerId::NONE)
                        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
                    let removed = tx
                        .state()
                        .zones
                        .find(crate::state::ZoneKind::Removed, PlayerId::NONE)
                        .ok_or_else(|| EffectError::Illegal("no removed zone".into()))?;
                    crate::effect::move_one_card(
                        tx,
                        card,
                        processing,
                        removed,
                        PlayerId::NONE,
                        true,
                    )?;
                    return if draw_on_no_use {
                        run_effects(
                            tx,
                            vec![EffectRequest::DrawCardsFromSkill {
                                player,
                                count: 1,
                                skill,
                            }],
                        )
                        .map(|_| ())
                    } else {
                        Ok(())
                    };
                }
                let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
                if lose_hp_before_use > 0 {
                    tx.state_mut().resolution.deferred_tail_effects.push(
                        EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants {
                            player,
                            card,
                            targets,
                            stage: 1,
                            damage_bonus,
                            skill,
                        },
                    );
                    return run_effects(
                        tx,
                        vec![EffectRequest::DealDamage {
                            source: PlayerId::NONE,
                            target: player,
                            amount: lose_hp_before_use,
                            is_lose_hp: true,
                            nature: rule_api::damage_nature::NORMAL,
                        }],
                    )
                    .map(|_| ());
                }
                super::play::handle_skill_card_with_policies(
                    tx,
                    player,
                    card,
                    &targets,
                    ignore_distance,
                    ignore_use_count,
                )
            }
            ContinuationKind::JudgeDamageOtherOnSuit => {
                if cancelled {
                    return Err(EffectError::Illegal(
                        "judgment damage target choice cannot cancel".into(),
                    ));
                }
                let player = PlayerId::new(*payload.first().unwrap_or(&0));
                let amount = *payload.get(1).unwrap_or(&0);
                let nature = *payload.get(2).unwrap_or(&0) as u8;
                let [target] = selections else {
                    return Err(EffectError::Illegal(
                        "choose exactly one judgment damage target".into(),
                    ));
                };
                run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: player,
                        target: PlayerId::new(*target),
                        amount,
                        is_lose_hp: false,
                        nature,
                    }],
                )
                .map(|_| ())
            }
            ContinuationKind::GanglieSourceDiscardOrDamage => {
                let [player, source, rest @ ..] = payload.as_slice() else {
                    return Err(EffectError::EngineFault(
                        "bad Ganglie discard continuation".into(),
                    ));
                };
                let player = PlayerId::new(*player);
                let source = PlayerId::new(*source);
                let hand_only = rest.first().copied().unwrap_or(1) != 0;
                if observer != source
                    || !tx
                        .state()
                        .players
                        .get(source)
                        .is_some_and(|participant| participant.alive)
                {
                    return Err(EffectError::Illegal(
                        "Ganglie discard context changed".into(),
                    ));
                }
                if cancelled {
                    if !selections.is_empty() {
                        return Err(EffectError::Illegal(
                            "cancelled Ganglie discard contains cards".into(),
                        ));
                    }
                    return run_effects(
                        tx,
                        vec![EffectRequest::DealDamage {
                            source: player,
                            target: source,
                            amount: 1,
                            is_lose_hp: false,
                            nature: rule_api::damage_nature::NORMAL,
                        }],
                    )
                    .map(|_| ());
                }
                let available = crate::effect::ganglie_discardable_cards(tx, source, hand_only);
                let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
                if cards.len() != 2
                    || cards[0] == cards[1]
                    || cards.iter().any(|card| !available.contains(card))
                {
                    return Err(EffectError::Illegal(
                        "Ganglie requires exactly two discardable cards".into(),
                    ));
                }
                run_effects(
                    tx,
                    vec![EffectRequest::DiscardCards {
                        player: source,
                        cards,
                    }],
                )
                .map(|_| ())
            }
            ContinuationKind::ResolveGanglieThree => {
                let [player, skill] = payload.as_slice() else {
                    return Err(EffectError::EngineFault(
                        "bad Ganglie Three target continuation".into(),
                    ));
                };
                if cancelled || selections.len() != 1 {
                    return Err(EffectError::Illegal(
                        "Ganglie Three requires one enemy target".into(),
                    ));
                }
                run_effects(
                    tx,
                    vec![EffectRequest::ResolveGanglieThree {
                        player: PlayerId::new(*player),
                        target: PlayerId::new(selections[0]),
                        skill: engine_api::SkillId::new(*skill),
                    }],
                )
                .map(|_| ())
            }
            ContinuationKind::TuxiGainOtherHandsReplaceDraw => {
                resolve_tuxi_gain_other_hands_replace_draw(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::YijiDistributeTopCards => {
                resolve_yiji_distribute_top_cards(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::XiaoguoDiscardBasicThenEquipOrDamage => {
                resolve_xiaoguo_discard_basic_then_equip_or_damage(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ResolveSpXiaoguo => {
                resolve_sp_xiaoguo(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseDingpanBranch => {
                resolve_choose_dingpan_branch(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTanlu => {
                resolve_tanlu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::TargetDiscardThenDuelIfNonShaAndHpGteOwner => {
                resolve_target_discard_then_duel(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseTargetLoseHpDiscardHandShaIfBasic => {
                resolve_choose_target_lose_hp_discard_hand_sha_if_basic(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::TongjiDiscardRedirectSha => {
                resolve_tongji_discard_redirect_sha(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::LiuliDiscardRedirectSha => {
                resolve_liuli_discard_redirect_sha(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::JijieChooseBottomCardRecipient => {
                resolve_jijie_choose_bottom_card_recipient(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::LuoshenRepeatBlackJudgment => {
                resolve_luoshen_repeat_black_judgment(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::JudgeSameColorGainRepeat => {
                resolve_judge_same_color_gain_repeat(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::DelegatedGroupResponse => {
                resolve_delegated_group_response(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::MoveEquipmentThenDraw => {
                resolve_move_equipment_then_draw(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::DiscardEquipmentPairWithArmor => {
                resolve_discard_equipment_pair_with_armor(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::DiscardTargetEquipmentGainMount => {
                resolve_discard_target_equipment_gain_mount(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::DiscardTargetEquipmentDrawWeaponRange => {
                resolve_discard_target_equipment_draw_weapon_range(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseTargetRevealHandDamageIfSha => {
                resolve_choose_target_reveal_hand_damage_if_sha(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseOtherGiveOneHandEquip => {
                resolve_choose_other_give_one_hand_equip(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseOtherGiveThenRecover => {
                resolve_choose_other_give_then_recover(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::GuanxingPick => {
                resolve_guanxing_pick(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::RecoverToMaxThenOptionalExchangeSkill => {
                resolve_recover_to_max_then_optional_exchange_skill(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::DiscardTargetCardGainMount => resolve_discard_target_card_gain_mount(
                tx, observer, &payload, selections, cancelled,
            ),
            ContinuationKind::ChooseDiscardTargetCard => {
                resolve_choose_discard_target_card(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseDiscardTargetCardResolveCurrentSha => {
                resolve_choose_discard_target_card_current_sha(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ResolveDcqiaomeng => {
                resolve_dcqiaomeng(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveRetieji => {
                resolve_retieji(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveNewReluoyi => {
                resolve_new_reluoyi(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveXinyaoming => {
                resolve_xinyaoming(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveRehuaiyi => {
                resolve_rehuaiyi(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveReanguo => {
                resolve_reanguo(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveBeizhu => {
                resolve_beizhu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveMbshanjia => {
                resolve_mbshanjia(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveDaoji => {
                resolve_daoji(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::GiveAllHandThenChooseUseOneOrDamage => {
                resolve_give_all_hand_then_choose_use_one_or_damage(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable => {
                resolve_choose_category_then_target_give_matching_or_sha_unrespondable(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ResolveTwfupan => {
                resolve_twfupan(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwruilian => {
                resolve_twruilian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwjianwei => {
                resolve_twjianwei(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwtanfeng => {
                resolve_twtanfeng(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwzhibian => {
                resolve_twzhibian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwyuyan => {
                resolve_twyuyan(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwguoyi => {
                resolve_twguoyi(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwchuhai => {
                resolve_twchuhai(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwyizhu => {
                resolve_twyizhu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwxiawei => {
                resolve_twxiawei(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwbianfuren => {
                resolve_twbianfuren(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwfeiyi => {
                resolve_twfeiyi(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwchenzhen => {
                resolve_twchenzhen(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwqingxi => {
                resolve_twqingxi(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwzaoli => {
                resolve_twzaoli(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwzhengjian => {
                resolve_twzhengjian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwtianyu => {
                resolve_twtianyu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwwujing => {
                resolve_twwujing(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwshanghe => {
                resolve_twshanghe(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwwangchang => {
                resolve_twwangchang(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwcaozhao => {
                resolve_twcaozhao(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwguohuai => {
                resolve_twguohuai(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ClanshenjunUseCard => {
                crate::effect::resolve_clanshenjun_use_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ClanyuzhiRevealChoice => {
                crate::effect::resolve_clanyuzhi_reveal_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ClanyuzhiEndChoice => {
                crate::effect::resolve_clanyuzhi_end_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::QieyiPutTopChoice => {
                crate::effect::resolve_qieyi_put_top_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::JianzhiCountChoice => {
                crate::effect::resolve_jianzhi_count_choice(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::JianzhiAssignCard => {
                crate::effect::resolve_jianzhi_assign_card(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseTargetDrawChoice => {
                // Delegate to the concurrent process's resolve_choose_target_draw
                crate::command::resolve_choice::group_post_3::resolve_choose_target_draw(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ResolveJiaohua => {
                resolve_jiaohua(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveLuanqun => {
                resolve_luanqun(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseTargetReclaimEquipmentDrawIfRangeReduced => {
                resolve_choose_target_reclaim_equipment_draw_if_range_reduced(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseUseVirtualCard => {
                resolve_choose_use_virtual_card(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseDiscardOneThenGainTargetEquipment => {
                resolve_choose_discard_one_then_gain_target_equipment(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseTargetDraw => {
                resolve_choose_target_draw(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseTargetsDamage => {
                resolve_choose_targets_damage(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseTargetDrawThenOwnerIfBelowMaxHp => {
                resolve_choose_target_draw_then_owner_if_below_max_hp(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseTargetDrawMaxHpCapThenDiscardExcess => {
                resolve_choose_target_draw_max_hp_cap_then_discard_excess(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp => {
                resolve_give_selected_then_target_choose_discard_same_suit_or_lose_hp(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::FanjianChooseSuitOrHand => resolve_fanjian_choose_suit_or_hand(
                tx, observer, &payload, selections, cancelled,
            ),
            ContinuationKind::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp => {
                resolve_choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseGiveOneHandEquipOrRecover => {
                resolve_choose_give_one_hand_equip_or_recover(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseGiveTwoDifferentColorsOrRecover => {
                resolve_choose_give_two_different_colors_or_recover(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ResolveMbzhuguo => {
                resolve_mbzhuguo(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveMbanxian => {
                resolve_mbanxian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveMbjinzu => {
                resolve_mbjinzu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp => {
                resolve_choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseOtherDrawOwnerLostHpTurnOver => {
                resolve_choose_other_draw_owner_lost_hp_turn_over(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ForceOthersUseShaOnNearestOrLoseHp => {
                resolve_force_others_use_sha_on_nearest_or_lose_hp(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseDiscardOneForShaRefundAndDrawIfRed => {
                resolve_choose_discard_one_for_sha_refund_and_draw_if_red(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ResolveOllinjie => {
                resolve_ollinjie(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseTargetGainMaxHp => {
                resolve_choose_target_gain_max_hp(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::GainTargetHandCard => {
                resolve_gain_target_hand_card(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::GainFromSkillPile => {
                resolve_gain_from_skill_pile(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseOtherAddMark => {
                resolve_choose_other_add_mark(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::StartDebateTargets => {
                resolve_start_debate_targets(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::PinDebateOpinions => {
                crate::effect::resolve_pin_debate_opinions(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::DistributeShadowCards => {
                crate::effect::resolve_distribute_shadow_cards(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::RegisterTurnEndDebate => {
                crate::effect::resolve_register_turn_end_debate(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseGroup => {
                crate::effect::resolve_choose_group(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseGainOtherSkill => {
                crate::effect::resolve_choose_gain_other_skill(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::LoseGainedSkill => {
                crate::effect::resolve_lose_gained_skill(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ResolvePotheyun => {
                crate::effect::resolve_potheyun_pick(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChoosePlaceHandXumou => {
                crate::effect::resolve_choose_place_hand_xumou(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::CityConsume => {
                crate::effect::resolve_city_consume(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseOthersAddMark => {
                crate::effect::resolve_choose_others_add_mark(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseOthersInsertExtraTurn => {
                crate::effect::resolve_choose_others_insert_extra_turn(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::TwjieqiuSelect => {
                crate::effect::resolve_twjieqiu_select(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::DiscardBasicDrawEqual => {
                resolve_discard_basic_draw_equal(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::PlaceHandCardToExpansion => {
                resolve_place_hand_card_to_expansion(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseClanNotMaxHandLimitGain => {
                resolve_choose_clan_not_max_hand_limit_gain(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseAddCurrentShaTarget => {
                resolve_choose_add_current_sha_target(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseStoreHandShaInSkillPile => {
                resolve_choose_store_hand_sha_in_skill_pile(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseSpendSkillPileForDyingJiu => {
                resolve_choose_spend_skill_pile_for_dying_jiu(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseCompareTargetHandCard => {
                resolve_choose_compare_target_hand_card(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseHandCompareTarget => {
                resolve_choose_hand_compare_target(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::BetweenOpponent => {
                resolve_between_opponent(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::BetweenFirstCard => {
                resolve_between_first_card(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::BetweenSecondCard => {
                resolve_between_second_card(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseHandComparePlayerCard => {
                resolve_choose_hand_compare_player_card(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseOtherDamage => {
                resolve_choose_other_damage(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseBoundSourceAttackRangeDamage => {
                resolve_choose_bound_source_attack_range_damage(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseOtherDiscardTargetCard => {
                resolve_choose_other_discard_target_card(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseGainDiscardCardByKind => {
                resolve_choose_gain_discard_card_by_kind(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseGainOwnLostDiscardCardThisTurn => {
                resolve_choose_gain_own_lost_discard_card_this_turn(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseMoveFieldCardTo => {
                resolve_choose_move_field_card_to(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::DrawThenOptionallyDistributeHandCards => {
                resolve_draw_then_optionally_distribute_hand_cards(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::GiveSelectedThenContinueDistinctRecoverAtTwo => {
                resolve_give_selected_then_continue_distinct_recover_at_two(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan => {
                resolve_choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver => {
                resolve_choose_zone_priority_gain_random_card_from_each_other_then_turn_over(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseGainCardFromEachOtherThenTurnOver => {
                resolve_choose_gain_card_from_each_other_then_turn_over(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseHiddenSkillPileHandCards => {
                resolve_choose_hidden_skill_pile_hand_cards(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseHiddenSkillPileCards => {
                resolve_choose_hidden_skill_pile_cards(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseChangandajianTarget => {
                resolve_choose_changandajian_target(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseChangandajianCard => {
                resolve_choose_changandajian_card(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::DiscardTiaojiyanmeiCard => {
                resolve_discard_tiaojiyanmei_card(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseTiaojiyanmeiGainTarget => {
                resolve_choose_tiaojiyanmei_gain_target(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseOtherYinghun => {
                resolve_choose_other_yinghun(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseOtherBizheng => {
                resolve_choose_other_bizheng(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseOtherXiashu => {
                resolve_choose_other_xiashu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveGuanwei => {
                resolve_guanwei(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveLanjiang => {
                resolve_lanjiang(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveRebingyi => {
                resolve_rebingyi(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveOldimeng => {
                resolve_oldimeng(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveRejieyin => {
                resolve_rejieyin(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveReanxu => {
                resolve_reanxu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveOldAnxu => {
                resolve_oldanxu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveOldQinqing => {
                resolve_oldqinqing(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveOldMingjian => {
                resolve_oldmingjian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveOldXuanfeng => {
                resolve_oldxuanfeng(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveJianrou => {
                resolve_jianrou(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveOldHuisheng => {
                resolve_oldhuisheng(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveOldJiefan => {
                resolve_oldjiefan(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveChegu => {
                resolve_chegu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveJunkguixin => {
                resolve_junkguixin(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveGuhuo => {
                resolve_guhuo(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveFaceDownViewAs => {
                resolve_face_down_view_as(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveSwapLowerHandViewAs => {
                resolve_swap_lower_hand_view_as(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveYaoChenwei => {
                resolve_yao_chenwei(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveYoudi => {
                resolve_youdi(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveKuangfu => {
                resolve_kuangfu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveShanjia => {
                resolve_shanjia(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveNewYajiao => {
                resolve_new_yajiao(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseDiscardSkillPileCardAddMarkedSkill => {
                let [player, target, helper_skill_def, skill] = payload.as_slice() else {
                    return Err(EffectError::EngineFault(
                        "bad skill-pile discard continuation".into(),
                    ));
                };
                let [selected] = selections else {
                    return Err(EffectError::Illegal(
                        "skill-pile discard requires one card".into(),
                    ));
                };
                if cancelled || observer != PlayerId::new(*player) {
                    return Err(EffectError::Illegal(
                        "invalid skill-pile discard choice".into(),
                    ));
                }
                run_effects(
                    tx,
                    vec![EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
                        player: PlayerId::new(*player),
                        target: PlayerId::new(*target),
                        helper_skill_def: engine_api::SkillDefId::new(*helper_skill_def),
                        selected: CardId::new(*selected),
                        skill: engine_api::SkillId::new(*skill),
                    }],
                )
                .map(|_| ())
            }
            ContinuationKind::GainOneHandFromSelectedTargets => {
                resolve_gain_one_hand_from_selected_targets(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseRepoluTargetsDraw => {
                resolve_choose_repolu_targets_draw(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ResolveZhuiyi => {
                resolve_zhuiyi(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveXinfuGuolun => {
                resolve_xinfu_guolun(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveDrltJueyan => {
                resolve_drlt_jueyan(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveRejizhi => {
                resolve_rejizhi(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveJiexun => {
                resolve_jiexun(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveWengua => {
                resolve_wengua(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::DiscardHandForMassDamage => {
                if cancelled || payload.len() < 3 {
                    return Err(EffectError::Illegal(
                        "mass hand discard cannot be cancelled".into(),
                    ));
                }
                let player = PlayerId::new(payload[0]);
                let required = payload[1] as usize;
                let option_count = payload[2] as usize;
                let (hand_equip, option_start) = match payload.len() {
                    len if len == 3 + option_count => (false, 3),
                    len if len == 4 + option_count && payload[3] == 1 => (true, 4),
                    _ => {
                        return Err(EffectError::Illegal(
                            "invalid mass discard payload".into(),
                        ))
                    }
                };
                if player != observer
                    || required == 0
                    || selections.len() != required
                {
                    return Err(EffectError::Illegal(
                        "invalid mass hand discard selection".into(),
                    ));
                }
                let original: std::collections::BTreeSet<_> =
                    payload[option_start..].iter().copied().collect();
                let hand = tx.state().zones.find(ZoneKind::Hand, player);
                let current: std::collections::BTreeSet<_> =
                    crate::query::discardable_hand_equip_cards(tx.state(), player)
                        .into_iter()
                        .filter(|card| {
                            hand_equip
                                || hand.is_some_and(|zone| {
                                    tx.state()
                                        .cards
                                        .get(*card)
                                        .is_some_and(|card| card.zone == zone)
                                })
                        })
                        .map(CardId::raw)
                        .collect();
                let mut selected = std::collections::BTreeSet::new();
                if !selections.iter().all(|card| {
                    selected.insert(*card) && original.contains(card) && current.contains(card)
                }) {
                    return Err(EffectError::Illegal(
                        "mass hand discard card is duplicated or no longer legal".into(),
                    ));
                }
                run_effects(
                    tx,
                    vec![EffectRequest::DiscardCards {
                        player,
                        cards: selections.iter().copied().map(CardId::new).collect(),
                    }],
                )
                .map(|_| ())
            }
            ContinuationKind::ChooseMarkParityControlTarget => {
                resolve_choose_mark_parity_control_target(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::DiscardMarkParityControlTargetCard => {
                resolve_discard_mark_parity_control_target_card(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ConfirmMarkParityMassDamage => {
                resolve_confirm_mark_parity_mass_damage(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseSelectedTargetDamage => {
                resolve_choose_selected_target_damage(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseDistinctSuitCards => resolve_choose_distinct_suit_cards(
                tx,
                observer,
                &payload,
                selections,
                cancelled,
            ),
            ContinuationKind::ChooseInspectedHeartCard => {
                resolve_choose_inspected_heart_card(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseInspectedHeartDestination => {
                resolve_choose_inspected_heart_destination(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseYeyanMode => {
                resolve_choose_yeyan_mode(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseYeyanBigCards => {
                resolve_choose_yeyan_big_cards(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseYeyanTargets => {
                resolve_choose_yeyan_targets(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseYeyanSingleDamage => {
                resolve_choose_yeyan_single_damage(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseYeyanDoubleTarget => {
                resolve_choose_yeyan_double_target(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseAllLoseHpOrRecover => {
                resolve_choose_all_lose_hp_or_recover(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseDiscardWeaponOrLoseHp => {
                resolve_choose_discard_weapon_or_lose_hp(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseDiscardOneHeOrLoseHp => {
                resolve_choose_discard_one_he_or_lose_hp(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseSourceConfirmOwnerLoseHp => {
                resolve_choose_source_confirm_owner_lose_hp(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseGainTargetCardAlternateZoneThenTargetDraw => {
                resolve_choose_gain_target_card_alternate_zone_then_target_draw(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseQixingHandCards => {
                resolve_choose_qixing_hand_cards(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseQixingPileCards => {
                resolve_choose_qixing_pile_cards(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseQixingWeatherCards => {
                resolve_choose_qixing_weather_cards(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseQixingWeatherTargets => {
                resolve_choose_qixing_weather_targets(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseLinkOther => {
                resolve_choose_link_other(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseDrltPoxiCards => {
                resolve_choose_drlt_poxi_cards(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseDrltJieyingTransfer => {
                resolve_choose_drlt_jieying_transfer(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseDrltDuoruiDisableSlot => {
                resolve_choose_drlt_duorui_disable_slot(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseDrltDuoruiSkill => {
                resolve_choose_drlt_duorui_skill(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseDrltZhitiEnableSlot => {
                resolve_choose_drlt_zhiti_enable_slot(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseJilueJizhiDiscard => {
                resolve_choose_jilue_jizhi_discard(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseAddMissingSkill => {
                resolve_choose_add_missing_skill(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseAddSkillBoundCardTarget => {
                resolve_choose_add_skill_bound_card_target(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseDiscardMatchingEquipColorAddSkill => {
                resolve_choose_discard_matching_equip_color_add_skill(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseCurrentCardUseEnhancements => {
                resolve_choose_current_card_use_enhancements(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseAddCurrentCardTarget => resolve_choose_add_current_card_target(
                tx, observer, &payload, selections, cancelled,
            ),
            ContinuationKind::ChooseAddOrRemoveCurrentCardTarget => {
                resolve_choose_add_or_remove_current_card_target(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseDrawOrDiscardAddSkill => {
                resolve_choose_draw_or_discard_add_skill(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseDiscardedBlackBasicAsBingliang => {
                resolve_choose_discarded_black_basic_as_bingliang(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage => {
                resolve_choose_discard_judge_use_virtual_sha_draw_if_no_damage(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded => {
                resolve_choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseRevealOtherHandCardRememberCategory => {
                resolve_choose_reveal_other_hand_card_remember_category(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::GiveExactRecordPhaseTarget => resolve_give_exact_record_phase_target(
                tx, observer, &payload, selections, cancelled,
            ),
            ContinuationKind::ChooseGainResolvedShaOrShanThenOptionalUseSha => {
                resolve_choose_gain_resolved_sha_or_shan_then_optional_use_sha(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseOtherTargetsBySkippedPhaseCountDamage => {
                resolve_choose_other_targets_by_skipped_phase_count_damage(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseOthersAfterUniformHandReveal => {
                resolve_choose_others_after_uniform_hand_reveal(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseCardUserOrAddTarget => resolve_choose_card_user_or_add_target(
                tx, observer, &payload, selections, cancelled,
            ),
            ContinuationKind::ChooseGiveCardOrAddTarget => resolve_choose_give_card_or_add_target(
                tx, observer, &payload, selections, cancelled,
            ),
            ContinuationKind::ChooseDiscardEquipmentReduceCurrentDamage => {
                resolve_choose_discard_equipment_reduce_current_damage(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseDiscardDamageEndpointCardIncreaseIfSuit => {
                resolve_choose_discard_damage_endpoint_card_increase_if_suit(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseEndExtraDrawOrPlay => {
                resolve_choose_end_extra_draw_or_play(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseMoveTargetEquipmentOrDraw => {
                resolve_choose_move_target_equipment_or_draw(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseRecoverOwnerOrDamageByGivenEquipment => {
                resolve_choose_recover_owner_or_damage_by_given_equipment(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage => {
                resolve_choose_target_discard_weapon_range_hand_or_increase_damage(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth => {
                resolve_choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseDrawOrRecastAddSkill => {
                resolve_choose_draw_or_recast_add_skill(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseDrawOrDiscardTargetCardRecordSha => {
                resolve_choose_draw_or_discard_target_card_record_sha(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::RevealHandChooseDiscardColor => {
                resolve_reveal_hand_choose_discard_color(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseDiscardTargetCardsRecoverSameCategory => {
                resolve_choose_discard_target_cards_recover_same_category(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseScskuijiDiscard => {
                resolve_choose_scskuiji_discard(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChoosePskuijiDiscard => {
                resolve_choose_pskuiji_discard(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseSpjungongDiscardOrLose => {
                resolve_choose_spjungong_discard_or_lose(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseSpxizhanDiscardOrLose => {
                resolve_choose_spxizhan_discard_or_lose(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseSpsongshuCards => {
                resolve_choose_spsongshu_cards(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseCompareTargetHandCardSecondFailureSeatDamage => {
                resolve_choose_compare_target_hand_card_second_failure_seat_damage(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseDrawOrDiscardTargetCard => {
                resolve_choose_draw_or_discard_target_card(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseSeatPathDamage => {
                resolve_choose_seat_path_damage(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseGainShaForOpeningExtraPlay => {
                resolve_choose_gain_sha_for_opening_extra_play(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::LoseHpGainDiscardCardForOpeningExtraPlay => {
                resolve_opening_extra_play_hp_loss_gain(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseHandCountDifferentTargetDrawOrRandomDiscard => {
                resolve_choose_hand_count_different_target_draw_or_random_discard(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw => {
                resolve_choose_target_discard_excess_then_self_discard_or_target_draw(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseGainOneFromDamageTargetAndNeighbors => {
                resolve_choose_gain_one_from_damage_target_and_neighbors(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChoosePreventDamageOrLoseMaxHpIncreaseDamage => {
                resolve_choose_prevent_damage_or_lose_max_hp_increase_damage(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::SourceFreeViewAsEquipCost => {
                if cancelled || selections.len() != 1 {
                    return Err(EffectError::Illegal(
                        "choose exactly one enabled equipment slot".into(),
                    ));
                }
                let skill = engine_api::SkillId::new(*payload.first().unwrap_or(&0));
                let kind = payload
                    .get(1)
                    .and_then(|tag| CardKind::from_tag(*tag as u8))
                    .ok_or_else(|| EffectError::Illegal("bad response card kind".into()))?;
                let action = *payload.get(2).unwrap_or(&0);
                super::skill::complete_source_free_response_cost(
                    tx,
                    observer,
                    skill,
                    kind,
                    selections[0] as u8,
                    action,
                    payload.get(3..).unwrap_or_default(),
                )
            }
            ContinuationKind::SourceFreeViewAsPlay | ContinuationKind::SourceFreeViewAsResponse => {
                Err(EffectError::EngineFault(
                    "saved source-free continuation opened as a direct choice".into(),
                ))
            }
            ContinuationKind::MultiCardViewAsResponse => {
                resolve_multi_card_view_as_response(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseOneFromSkillPileDiscard => {
                resolve_choose_one_from_skill_pile_discard(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ChooseTargetHandCardToPile => {
                resolve_choose_target_hand_card_to_pile(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::PlaceHandCardsToDeckTop => {
                resolve_place_hand_cards_to_deck_top(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::ResolveDcbingjiStage => {
                resolve_resolve_dcbingji(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolvePxqixianStage => {
                resolve_resolve_pxqixian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveJsrgdaorenStage => {
                resolve_resolve_jsrgdaoren(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveQianlongStage => {
                resolve_resolve_qianlong(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveJsrgzuozhanStage => {
                resolve_resolve_jsrgzuozhan(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveJsrgzuozhanGainStage => {
                resolve_resolve_jsrgzuozhan_gain(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwdaiguiStage if !payload.is_empty() && payload[3] == 3 => {
                resolve_resolve_twdaigui_round(tx, observer, &payload, selections)
            }
            ContinuationKind::ResolveTwdaiguiStage => {
                resolve_resolve_twdaigui(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveTwmituStage if !payload.is_empty() && payload[3] >= 3 => {
                resolve_resolve_twmitu_round(tx, observer, &payload, selections)
            }
            ContinuationKind::ResolveTwmituStage => {
                resolve_resolve_twmitu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveQiantunStage => {
                resolve_resolve_qiantun(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveYangmingStage if !payload.is_empty() && payload[3] >= 2 => {
                resolve_resolve_yangming_round(tx, observer, &payload, selections)
            }
            ContinuationKind::ResolveYangmingStage => {
                resolve_resolve_yangming(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveCanyunStage => {
                resolve_resolve_canyun(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveBiaozhao3Stage => {
                resolve_resolve_biaozhao3(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveDcmoukuiStage => {
                resolve_resolve_dcmoukui(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveZhongyanStage => {
                resolve_resolve_zhongyan(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveChumeiStage => {
                resolve_resolve_chumei(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveShuaiyanStage => {
                resolve_resolve_shuaiyan(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveMbbeimingStage => {
                resolve_resolve_mbbeiming(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveGaoyuanStage => {
                resolve_resolve_gaoyuan(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveDuoduanStage => {
                resolve_resolve_duoduan(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveDcanxuStage => {
                resolve_resolve_dcanxu(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveChulaoStage => {
                resolve_resolve_chulao(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveMbzhenannStage => {
                resolve_resolve_mbzhenann(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveMeiyongStage => {
                resolve_resolve_meiyong(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveDcqiaolveStage => {
                resolve_resolve_dcqiaolve(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveXinfuQiaiStage => {
                resolve_resolve_xinfu_qiai(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveOlzhijieStage => {
                resolve_resolve_olzhijie(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveOlqiaozhiStage => {
                resolve_resolve_olqiaozhi(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveZhongjieStage => {
                resolve_resolve_zhongjie(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveNhlundaoStage => {
                resolve_resolve_nhlundao(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveDingceStage => {
                resolve_resolve_dingce(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveQiangyongStage => {
                resolve_resolve_qiangyong(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveZhengjianStage => {
                resolve_resolve_zhengjian(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ResolveReliangyinStage => {
                resolve_resolve_liangyin(tx, observer, &payload, selections, cancelled)
            }
            ContinuationKind::ChooseHandCardsToSkillPile => {
                resolve_choose_hand_cards_to_skill_pile(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::StoreSelectedInSkillPile => {
                resolve_store_selected_in_skill_pile(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::RearrangePileToDeckTop => {
                resolve_rearrange_pile_to_deck_top(
                    tx,
                    observer,
                    &payload,
                    selections,
                    cancelled,
                )
            }
            ContinuationKind::VmResume => {
                let (frames, module_name, pc, fuel, ctx, skill_id, trigger_marker) = vm_cont;
                if cancelled {
                    // Cancelling a DSL choose just ends the skill with no further effects.
                    if trigger_marker != [0; 3] {
                        tx.state_mut()
                            .resolution
                            .deferred_effects
                            .push(vm_trigger_marker(trigger_marker));
                    }
                    return Ok(());
                }
                // The chosen token is the single selection (choose is single-select).
                let chosen = *selections.first().unwrap_or(&0);
                let module = tx
                    .state()
                    .content
                    .module(&module_name)
                    .cloned()
                    .ok_or_else(|| {
                        EffectError::ContentFault(format!("missing module {module_name}"))
                    })?;
                let outcome = crate::vm_host::resume_module(
                    tx.state(),
                    &module,
                    skill_id,
                    ctx,
                    frames,
                    pc,
                    fuel,
                    chosen,
                )
                .map_err(|e| EffectError::ContentFault(format!("vm resume: {e}")))?;
                if trigger_marker == [0; 3] {
                    run_effects(tx, outcome.effects)?;
                } else {
                    tx.state_mut()
                        .resolution
                        .deferred_effects
                        .extend(outcome.effects);
                }
                if let Some(pause) = outcome.paused {
                    open_vm_choice(
                        tx,
                        PlayerId::new(ctx[0]),
                        &module_name,
                        skill_id,
                        ctx,
                        trigger_marker,
                        pause,
                    )?;
                    return Err(EffectError::ChoiceRequested);
                }
                if trigger_marker != [0; 3] {
                    tx.state_mut()
                        .resolution
                        .deferred_effects
                        .push(vm_trigger_marker(trigger_marker));
                }
                Ok(())
            }
            ContinuationKind::DiscardOneHandOrEquip => {
                let player = PlayerId::new(*payload.first().unwrap_or(&0));
                if cancelled || selections.len() != 1 {
                    return Err(EffectError::Illegal(
                        "must discard exactly one hand or equipment card".into(),
                    ));
                }
                let card = CardId::new(selections[0]);
                if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card) {
                    return Err(EffectError::Illegal(
                        "discard card not in player hand or equipment".into(),
                    ));
                }
                run_effects(
                    tx,
                    vec![EffectRequest::DiscardCards {
                        player,
                        cards: vec![card],
                    }],
                )
                .map(|_| ())
            }
            ContinuationKind::AdjustHandTo => {
                let player = PlayerId::new(*payload.first().unwrap_or(&0));
                let need = *payload.get(1).unwrap_or(&0) as usize;
                if cancelled || selections.len() != need {
                    return Err(EffectError::Illegal(format!(
                        "must discard exactly {need} hand cards"
                    )));
                }
                let legal: std::collections::BTreeSet<u32> = tx
                    .state()
                    .hand_cards(player)
                    .into_iter()
                    .map(CardId::raw)
                    .collect();
                let selected: std::collections::BTreeSet<u32> =
                    selections.iter().copied().collect();
                if selected.len() != need || !selected.is_subset(&legal) {
                    return Err(EffectError::Illegal(
                        "adjust-hand selection must contain distinct current hand cards".into(),
                    ));
                }
                run_effects(
                    tx,
                    vec![EffectRequest::DiscardCards {
                        player,
                        cards: selections.iter().copied().map(CardId::new).collect(),
                    }],
                )
                .map(|_| ())
            }
            ContinuationKind::DiscardHandAddSkillByColor => {
                if cancelled {
                    return Ok(());
                }
                if selections.len() != 1 || payload.len() != 3 {
                    return Err(EffectError::Illegal(
                        "discard exactly one hand card to gain a color skill".into(),
                    ));
                }
                let player = PlayerId::new(payload[0]);
                let card = CardId::new(selections[0]);
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, player)
                    .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
                let card_state = tx
                    .state()
                    .cards
                    .get(card)
                    .ok_or_else(|| EffectError::Illegal("bad discard card".into()))?;
                if card_state.zone != hand
                    || !crate::query::discardable_hand_equip_cards(tx.state(), player)
                        .contains(&card)
                {
                    return Err(EffectError::Illegal(
                        "discard card not in player hand".into(),
                    ));
                }
                let skill_def =
                    match crate::query::effective_card_suit(tx.state(), player, card_state) {
                        engine_api::Suit::Heart | engine_api::Suit::Diamond => payload[1],
                        engine_api::Suit::Spade | engine_api::Suit::Club => payload[2],
                        engine_api::Suit::None => {
                            return Err(EffectError::Illegal(
                                "suitless card cannot select a color skill".into(),
                            ));
                        }
                    };
                run_effects(
                    tx,
                    vec![
                        EffectRequest::DiscardCards {
                            player,
                            cards: vec![card],
                        },
                        EffectRequest::AddSkill {
                            player,
                            skill_def: engine_api::SkillDefId::new(skill_def),
                        },
                    ],
                )
                .map(|_| ())
            }
            ContinuationKind::RevealTopGainNonBasicOrSwapBasic => {
                if payload.len() != 2 {
                    return Err(EffectError::EngineFault(
                        "bad top-card swap continuation".into(),
                    ));
                }
                let player = PlayerId::new(payload[0]);
                let revealed = CardId::new(payload[1]);
                let processing = tx
                    .state()
                    .zones
                    .find(ZoneKind::Processing, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
                if !tx
                    .state()
                    .cards
                    .get(revealed)
                    .is_some_and(|card| card.zone == processing && card.owner.is_none())
                {
                    return Err(EffectError::Illegal(
                        "revealed card left processing zone".into(),
                    ));
                }
                if cancelled {
                    if !selections.is_empty() {
                        return Err(EffectError::Illegal(
                            "cancelled top-card swap has selections".into(),
                        ));
                    }
                    let discard = tx
                        .state()
                        .zones
                        .find(ZoneKind::DiscardPile, PlayerId::NONE)
                        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
                    return run_effects(
                        tx,
                        vec![EffectRequest::MoveCards {
                            cards: vec![revealed],
                            to: discard,
                            from: Some(processing),
                        }],
                    )
                    .map(|_| ());
                }
                if selections.len() != 1 {
                    return Err(EffectError::Illegal(
                        "select one hand card for top-card swap".into(),
                    ));
                }
                let replacement = CardId::new(selections[0]);
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, player)
                    .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
                if !tx
                    .state()
                    .cards
                    .get(replacement)
                    .is_some_and(|card| card.zone == hand && card.owner == player)
                {
                    return Err(EffectError::Illegal(
                        "replacement card not in player hand".into(),
                    ));
                }
                let draw = tx
                    .state()
                    .zones
                    .find(ZoneKind::DrawPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
                run_effects(
                    tx,
                    vec![
                        EffectRequest::MoveCards {
                            cards: vec![replacement],
                            to: draw,
                            from: Some(hand),
                        },
                        EffectRequest::MoveCards {
                            cards: vec![revealed],
                            to: hand,
                            from: Some(processing),
                        },
                    ],
                )
                .map(|_| ())
            }
            ContinuationKind::ChooseGainTopCardsBySuit => {
                if payload.len() < 3 {
                    return Err(EffectError::EngineFault(
                        "bad top-card suit-choice continuation".into(),
                    ));
                }
                let player = PlayerId::new(payload[0]);
                let suit = crate::command::judge::suit_from_tag(payload[1]);
                let take = payload[2] as usize;
                if take == 0 || payload.len() != 3 + take || suit == engine_api::Suit::None {
                    return Err(EffectError::EngineFault(
                        "truncated top-card suit-choice continuation".into(),
                    ));
                }
                let draw = tx
                    .state()
                    .zones
                    .find(ZoneKind::DrawPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
                let zone = tx
                    .state()
                    .zones
                    .get(draw)
                    .ok_or_else(|| EffectError::EngineFault("draw zone missing".into()))?;
                if zone.cards.len() < take
                    || zone.cards[zone.cards.len() - take..]
                        .iter()
                        .map(|card| card.raw())
                        .ne(payload[3..].iter().copied())
                {
                    return Err(EffectError::Illegal(
                        "viewed cards no longer match the draw-pile top".into(),
                    ));
                }
                if cancelled {
                    if !selections.is_empty() {
                        return Err(EffectError::Illegal(
                            "cancelled top-card suit choice has selections".into(),
                        ));
                    }
                    return Ok(());
                }
                if selections.is_empty() || selections.len() > take {
                    return Err(EffectError::Illegal(
                        "select a nonempty subset of viewed cards".into(),
                    ));
                }
                let selected: std::collections::BTreeSet<u32> =
                    selections.iter().copied().collect();
                if selected.len() != selections.len()
                    || !selected.iter().all(|raw| payload[3..].contains(raw))
                    || !selected.iter().all(|raw| {
                        tx.state().cards.get(CardId::new(*raw)).is_some_and(|card| {
                            crate::query::effective_card_suit(tx.state(), player, card) == suit
                        })
                    })
                {
                    return Err(EffectError::Illegal(
                        "selected cards must be distinct viewed cards of the required suit".into(),
                    ));
                }
                if !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|player| player.alive)
                {
                    return Err(EffectError::Illegal(
                        "top-card suit choice player is not alive".into(),
                    ));
                }
                let cards: Vec<CardId> = selections.iter().copied().map(CardId::new).collect();
                tx.push_event(engine_api::DomainEvent::CardsRevealed {
                    cards: cards.clone(),
                    to: Vec::new(),
                });
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, player)
                    .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
                run_effects(
                    tx,
                    vec![EffectRequest::MoveCards {
                        cards,
                        to: hand,
                        from: Some(draw),
                    }],
                )
                .map(|_| ())
            }
            ContinuationKind::ChooseOtherDrawByHandCount => {
                if payload.len() != 5 {
                    return Err(EffectError::EngineFault(
                        "bad other-player draw continuation".into(),
                    ));
                }
                let player = PlayerId::new(payload[0]);
                let normal_count = payload[1];
                let empty_count = payload[2];
                let times = payload[3];
                let skill = engine_api::SkillId::new(payload[4]);
                if player != observer || times == 0 {
                    return Err(EffectError::EngineFault(
                        "invalid other-player draw continuation owner or count".into(),
                    ));
                }
                let mut effects = Vec::with_capacity(2);
                if cancelled {
                    if !selections.is_empty() {
                        return Err(EffectError::Illegal(
                            "cancelled other-player draw choice has selections".into(),
                        ));
                    }
                } else {
                    if selections.len() != 1 {
                        return Err(EffectError::Illegal(
                            "select exactly one other player to draw".into(),
                        ));
                    }
                    let target = PlayerId::new(selections[0]);
                    if target == player
                        || !tx
                            .state()
                            .players
                            .get(target)
                            .is_some_and(|participant| participant.alive)
                    {
                        return Err(EffectError::Illegal(
                            "draw target must be another living player".into(),
                        ));
                    }
                    let count = if tx.state().hand_count(target) == 0 {
                        empty_count
                    } else {
                        normal_count
                    };
                    if count > 0 {
                        effects.push(EffectRequest::DrawCardsFromSkill {
                            player: target,
                            count,
                            skill,
                        });
                    }
                }
                if times > 1 {
                    effects.push(EffectRequest::ChooseOtherDrawByHandCount {
                        player,
                        normal_count,
                        empty_count,
                        times: times - 1,
                        skill,
                    });
                }
                run_effects(tx, effects).map(|_| ())
            }
            ContinuationKind::ChooseOtherDrawOrRecover => {
                let stage = *payload.first().unwrap_or(&u32::MAX);
                match stage {
                    0 => {
                        if payload.len() != 6 {
                            return Err(EffectError::EngineFault(
                                "bad other-player draw-or-recover target continuation".into(),
                            ));
                        }
                        let player = PlayerId::new(payload[1]);
                        let draw_count = payload[2];
                        let recover_count = payload[3];
                        let times = payload[4];
                        let skill = engine_api::SkillId::new(payload[5]);
                        if player != observer || times == 0 {
                            return Err(EffectError::EngineFault(
                                "invalid other-player draw-or-recover owner or count".into(),
                            ));
                        }
                        if cancelled {
                            if !selections.is_empty() {
                                return Err(EffectError::Illegal(
                                    "cancelled draw-or-recover target choice has selections".into(),
                                ));
                            }
                            return if times > 1 {
                                run_effects(
                                    tx,
                                    vec![EffectRequest::ChooseOtherDrawOrRecover {
                                        player,
                                        draw_count,
                                        recover_count,
                                        times: times - 1,
                                        skill,
                                    }],
                                )
                                .map(|_| ())
                            } else {
                                Ok(())
                            };
                        }
                        if selections.len() != 1 {
                            return Err(EffectError::Illegal(
                                "select exactly one other player to draw or recover".into(),
                            ));
                        }
                        let target = PlayerId::new(selections[0]);
                        let participant = tx.state().players.get(target).ok_or_else(|| {
                            EffectError::Illegal("bad draw-or-recover target".into())
                        })?;
                        if target == player || !participant.alive {
                            return Err(EffectError::Illegal(
                                "draw-or-recover target must be another living player".into(),
                            ));
                        }
                        let mut effects = Vec::with_capacity(2);
                        if participant.hp >= participant.max_hp as i32 || recover_count == 0 {
                            if draw_count > 0 {
                                effects.push(EffectRequest::DrawCardsFromSkill {
                                    player: target,
                                    count: draw_count,
                                    skill,
                                });
                            }
                        } else if draw_count == 0 {
                            effects.push(EffectRequest::RecoverHp {
                                source: target,
                                player: target,
                                amount: recover_count,
                            });
                        } else {
                            return crate::effect::open_choice(
                                tx,
                                target,
                                1,
                                1,
                                false,
                                vec![1, 2],
                                engine_api::ChoiceKind::Custom,
                                ContinuationKind::ChooseOtherDrawOrRecover,
                                vec![
                                    1,
                                    player.raw(),
                                    target.raw(),
                                    draw_count,
                                    recover_count,
                                    times,
                                    skill.raw(),
                                ],
                            )
                            .map(|_| ());
                        }
                        if times > 1 {
                            effects.push(EffectRequest::ChooseOtherDrawOrRecover {
                                player,
                                draw_count,
                                recover_count,
                                times: times - 1,
                                skill,
                            });
                        }
                        run_effects(tx, effects).map(|_| ())
                    }
                    1 => {
                        if payload.len() != 7 {
                            return Err(EffectError::EngineFault(
                                "bad other-player draw-or-recover branch continuation".into(),
                            ));
                        }
                        let player = PlayerId::new(payload[1]);
                        let target = PlayerId::new(payload[2]);
                        let draw_count = payload[3];
                        let recover_count = payload[4];
                        let times = payload[5];
                        let skill = engine_api::SkillId::new(payload[6]);
                        if observer != target || times == 0 || cancelled {
                            return Err(EffectError::Illegal(
                                "draw-or-recover branch cannot cancel or change actor".into(),
                            ));
                        }
                        if !tx
                            .state()
                            .players
                            .get(target)
                            .is_some_and(|participant| participant.alive)
                        {
                            return Err(EffectError::Illegal(
                                "draw-or-recover target is no longer living".into(),
                            ));
                        }
                        let mut effects = Vec::with_capacity(2);
                        match selections {
                            [1] => effects.push(EffectRequest::DrawCardsFromSkill {
                                player: target,
                                count: draw_count,
                                skill,
                            }),
                            [2] => effects.push(EffectRequest::RecoverHp {
                                source: target,
                                player: target,
                                amount: recover_count,
                            }),
                            _ => {
                                return Err(EffectError::Illegal(
                                    "choose draw or recover exactly once".into(),
                                ));
                            }
                        }
                        if times > 1 {
                            effects.push(EffectRequest::ChooseOtherDrawOrRecover {
                                player,
                                draw_count,
                                recover_count,
                                times: times - 1,
                                skill,
                            });
                        }
                        run_effects(tx, effects).map(|_| ())
                    }
                    2 => {
                        if payload.len() != 3 {
                            return Err(EffectError::EngineFault(
                                "bad forced other-player recovery continuation".into(),
                            ));
                        }
                        let player = PlayerId::new(payload[1]);
                        let amount = payload[2];
                        if observer != player || cancelled {
                            return Err(EffectError::Illegal(
                                "forced other-player recovery cannot cancel or change actor"
                                    .into(),
                            ));
                        }
                        if selections.len() != 1 {
                            return Err(EffectError::Illegal(
                                "select exactly one other player to recover".into(),
                            ));
                        }
                        let target = PlayerId::new(selections[0]);
                        if target == player
                            || !tx
                                .state()
                                .players
                                .get(target)
                                .is_some_and(|participant| participant.alive)
                        {
                            return Err(EffectError::Illegal(
                                "recovery target must be another living player".into(),
                            ));
                        }
                        run_effects(
                            tx,
                            vec![EffectRequest::RecoverHp {
                                source: target,
                                player: target,
                                amount,
                            }],
                        )
                        .map(|_| ())
                    }
                    _ => Err(EffectError::EngineFault(
                        "unknown other-player draw-or-recover continuation stage".into(),
                    )),
                }
            }
            ContinuationKind::ChooseDiscardHandEquipThenRemoveSelfSkill => {
                if payload.len() < 4 {
                    return Err(EffectError::Illegal(
                        "invalid dynamic hand/equipment discard payload".into(),
                    ));
                }
                let player = PlayerId::new(payload[0]);
                let skill = engine_api::SkillId::new(payload[1]);
                let required = payload[2] as usize;
                let option_count = payload[3] as usize;
                let recover_target = (payload.len() == 6 + option_count)
                    .then(|| (PlayerId::new(payload[4 + option_count]), payload[5 + option_count]));
                if cancelled {
                    return if recover_target.is_some() {
                        Ok(())
                    } else {
                        Err(EffectError::Illegal(
                            "dynamic hand/equipment discard cannot be cancelled".into(),
                        ))
                    };
                }
                if player != observer
                    || required == 0
                    || (payload.len() != 4 + option_count && recover_target.is_none())
                    || selections.len() != required
                {
                    return Err(EffectError::Illegal(
                        "invalid dynamic hand/equipment discard selection".into(),
                    ));
                }
                let originating_skill_enabled = tx.state().skills.get(skill).is_some_and(|entry| {
                    entry.owner == player && tx.state().skill_effectively_enabled(entry)
                });
                if !originating_skill_enabled {
                    return Err(EffectError::Illegal(
                        "dynamic discard source skill is no longer enabled".into(),
                    ));
                }
                let original: std::collections::BTreeSet<_> =
                    payload[4..4 + option_count].iter().copied().collect();
                let current: std::collections::BTreeSet<_> =
                    crate::query::discardable_hand_equip_cards(tx.state(), player)
                        .into_iter()
                        .map(CardId::raw)
                        .collect();
                let mut selected = std::collections::BTreeSet::new();
                if !selections.iter().all(|card| {
                    selected.insert(*card) && original.contains(card) && current.contains(card)
                }) {
                    return Err(EffectError::Illegal(
                        "dynamic discard card is duplicated or no longer legal".into(),
                    ));
                }
                if let Some((target, amount)) = recover_target {
                    if amount == 0
                        || !tx.state().players.get(target).is_some_and(|participant| {
                            participant.alive && participant.hp < participant.max_hp as i32
                        })
                    {
                        return Err(EffectError::Illegal(
                            "dynamic discard recovery target is no longer legal".into(),
                        ));
                    }
                    tx.state_mut()
                        .resolution
                        .deferred_tail_effects
                        .push(EffectRequest::RecoverHp {
                            source: player,
                            player: target,
                            amount,
                        });
                }
                run_effects(
                    tx,
                    vec![EffectRequest::DiscardCards {
                        player,
                        cards: selections.iter().copied().map(CardId::new).collect(),
                    }],
                )
                .map(|_| ())
            }
            ContinuationKind::ChooseTakeLargerHandUntilMax => {
                resolve_choose_take_larger_hand_until_max(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark => {
                resolve_choose_draw_or_increase_current_card_damage_or_both_mark(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::DrawThenChooseGiveOrDiscardHand => {
                resolve_draw_then_choose_give_or_discard_hand(
                    tx, observer, &payload, selections, cancelled,
                )
            }
            ContinuationKind::GiveOneHandOrEquip => {
                if cancelled || selections.len() != 1 || payload.len() < 6 {
                    return Err(EffectError::Illegal(
                        "must give exactly one hand or equipment card".into(),
                    ));
                }
                let from_player = PlayerId::new(payload[0]);
                let to_player = PlayerId::new(payload[1]);
                let use_given_equipment = payload[3] != 0;
                let skill = engine_api::SkillId::new(payload[4]);
                let option_count = payload[5] as usize;
                if payload.len() != 6 + option_count {
                    return Err(EffectError::EngineFault(
                        "truncated give-card continuation".into(),
                    ));
                }
                let card = CardId::new(selections[0]);
                if !payload[6..].contains(&card.raw()) {
                    return Err(EffectError::Illegal(
                        "card was not offered for giving".into(),
                    ));
                }
                let from = response_card_zone(tx, from_player, card)?;
                if !tx
                    .state()
                    .players
                    .get(to_player)
                    .is_some_and(|player| player.alive)
                {
                    return Ok(());
                }
                let to = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, to_player)
                    .ok_or_else(|| EffectError::Illegal("no recipient hand".into()))?;
                if use_given_equipment {
                    tx.state_mut()
                        .resolution
                        .deferred_tail_effects
                        .push(EffectRequest::DrawThenGiveOne {
                            player: from_player,
                            target: to_player,
                            count: 0,
                            drawn_only: false,
                            use_given_equipment: true,
                            skill,
                            drawn: vec![card],
                            ready_to_give: true,
                        });
                }
                run_effects(
                    tx,
                    vec![EffectRequest::GiveCards {
                        cards: vec![card],
                        to,
                        from,
                    }],
                )
                .map(|_| ())
            }
            ContinuationKind::UseGivenEquipment => {
                let [recipient_raw, card_raw] = payload.as_slice() else {
                    return Err(EffectError::EngineFault(
                        "bad use-given-equipment continuation".into(),
                    ));
                };
                if observer != PlayerId::new(*recipient_raw) {
                    return Err(EffectError::Illegal(
                        "given-equipment observer mismatch".into(),
                    ));
                }
                if cancelled || selections.first().copied() == Some(0) {
                    return if selections.is_empty() || selections == [0] {
                        Ok(())
                    } else {
                        Err(EffectError::Illegal(
                            "invalid given-equipment cancellation".into(),
                        ))
                    };
                }
                if selections != [1] {
                    return Err(EffectError::Illegal(
                        "invalid given-equipment confirmation".into(),
                    ));
                }
                super::play::use_equipment_from_hand(
                    tx,
                    observer,
                    CardId::new(*card_raw),
                )
            }
            ContinuationKind::RevealDiscardSuitGainOrUse => {
                crate::command::reveal_options::resolve_choice(tx, &payload, selections, cancelled)
            }
            ContinuationKind::DiscardOrFireDamageSkipPhases => {
                crate::command::discard_damage_options::resolve_choice(
                    tx, &payload, selections, cancelled,
                )
            }
            ContinuationKind::ResumePlayedSha
            | ContinuationKind::ResumePlayedShaQueue
            | ContinuationKind::IndependentTrickQueue
            | ContinuationKind::FinishPlayedSha
            | ContinuationKind::ContinueEnteredDrawPhase
            | ContinuationKind::FinishDelayedJudgment => Err(EffectError::EngineFault(
                "saved continuation cannot own a choice".into(),
            )),
            ContinuationKind::OptionalTrigger => {
                let task = optional_trigger.ok_or_else(|| {
                    EffectError::EngineFault("missing optional trigger continuation".into())
                })?;
                if !cancelled && selections.first().copied() == Some(1) {
                    tx.state_mut().resolution.deferred_triggers.insert(0, task);
                }
                Ok(())
            }
        }
    })();
    result?;
    if tx.state().has_pending_choice() {
        return Err(EffectError::ChoiceRequested);
    }
    crate::effect::resume_deferred_resolution(tx)?;
    resume_interrupted_continuation(tx)
}
