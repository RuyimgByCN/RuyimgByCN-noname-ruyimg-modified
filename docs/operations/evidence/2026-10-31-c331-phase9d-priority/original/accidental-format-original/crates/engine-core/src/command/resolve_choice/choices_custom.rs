//! Choice continuations for `Custom` and related requests.

use super::*;

pub(super) fn resolve_choice(
    tx: &mut Transaction,
    _actor: PlayerId,
    choice_id: ChoiceId,
    observer: PlayerId,
    kind: ContinuationKind,
    payload: Vec<u32>,
    selections: &[u32],
    cancelled: bool,
    _vm_cont: (Vec<u8>, String, u32, u32, [u32; 4], u32, [u32; 3]),
    _optional_trigger: Option<crate::trigger::PendingTrigger>,
) -> Result<(), EffectError> {
    match kind {
        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::TWCHENGPU_CUSTOM_CHOICE) =>
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
                == Some(
                    &crate::effect::CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_ADD_SKILL_CUSTOM_CHOICE,
                ) =>
        {
            resolve_choose_target_draw_max_hp_cap_then_add_skill(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_TIANXIANG_CUSTOM_CHOICE) =>
        {
            resolve_resolve_tianxiang(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_RETIANXIANG_CUSTOM_CHOICE) =>
        {
            resolve_resolve_retianxiang(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_LIANZHOU_CUSTOM_CHOICE) =>
        {
            resolve_resolve_lianzhou(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_STDDECHONG_CUSTOM_CHOICE) =>
        {
            resolve_resolve_stddechong(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::DISCARD_HAND_EQUIP_COUNT_CUSTOM_CHOICE) =>
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
            if payload.first() == Some(&crate::effect::RESOLVE_DC_SB_ZHANGXIU_CHOICE) =>
        {
            crate::effect::resolve_dc_sb_zhangxiu_choice(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DC_SB_ZHOUYU_CHOICE) =>
        {
            crate::effect::resolve_dc_sb_zhouyu_choice(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DC_SB_LUSU_CHOICE) =>
        {
            crate::effect::resolve_dc_sb_lusu_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_ZHANGJIAN_CHOICE) =>
        {
            crate::effect::resolve_zhangjian_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_ZHUGERUOXUE_CHOICE) =>
        {
            crate::effect::resolve_zhugeruoxue_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_CAOYI_CHOICE) =>
        {
            crate::effect::resolve_caoyi_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_MALINGLI_CHOICE) =>
        {
            crate::effect::resolve_malingli_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_NSSHUAIYAN_CHOICE) =>
        {
            crate::effect::resolve_nsshuaiyan_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_WDB_LIUFU_CHOICE) =>
        {
            crate::effect::resolve_wdb_liufu_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_WDB_LIUZAN_CHOICE) =>
        {
            crate::effect::resolve_wdb_liuzan_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_WDB_TIANYU_CHOICE) =>
        {
            crate::effect::resolve_wdb_tianyu_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_WDB_FUCHOU_CHOICE) =>
        {
            crate::effect::resolve_wdb_fuchou_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_WDB_YANGYI_CHOICE) =>
        {
            crate::effect::resolve_wdb_yangyi_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_WDB_ZAOZHIRENJUN_CHOICE) =>
        {
            crate::effect::resolve_wdb_zaozhirenjun_choice(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DDD_XIANXI_CHOICE) =>
        {
            crate::effect::resolve_ddd_xianxi_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DDD_XIAHENG_CHOICE) =>
        {
            crate::effect::resolve_ddd_xiaheng_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DDD_LIUCHONG_CHOICE) =>
        {
            crate::effect::resolve_ddd_liuchong_choice(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DDD_DUANBING_CHOICE) =>
        {
            crate::effect::resolve_ddd_duanbing_choice(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DDD_WUZHI_CHOICE) =>
        {
            crate::effect::resolve_ddd_wuzhi_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DDD_XUJING_CHOICE) =>
        {
            crate::effect::resolve_ddd_xujing_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DDD_CAOMAO_CHOICE) =>
        {
            crate::effect::resolve_ddd_caomao_choice(tx, observer, &payload, selections, cancelled)
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
            crate::effect::resolve_ddd_yujin_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DDD_LIUYE_CHOICE) =>
        {
            crate::effect::resolve_ddd_liuye_choice(tx, observer, &payload, selections, cancelled)
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
            crate::effect::resolve_ddd_zhenji_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DDD_ZHAOANG_CHOICE) =>
        {
            crate::effect::resolve_ddd_zhaoang_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DDD_LIUBA_CHOICE) =>
        {
            crate::effect::resolve_ddd_liuba_choice(tx, observer, &payload, selections, cancelled)
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
            crate::effect::resolve_ddd_liuhong_choice(tx, observer, &payload, selections, cancelled)
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
            crate::effect::resolve_ddd_liangxi_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&super::super::play::YOUDISHENRU_FORCE_CHOICE) =>
        {
            super::super::play::resolve_youdishenru_force_choice(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_JIELIANG_GAIN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_jieliang_gain(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_STDJUJIAN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_stdjujian(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_LULVE_CUSTOM_CHOICE) =>
        {
            resolve_resolve_lulve(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DCTINGXIAN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_dctingxian(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_HFJIEYING_CUSTOM_CHOICE) =>
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
            if payload.first() == Some(&crate::effect::RESOLVE_DCNUCHEN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_dcnuchen(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DCZHIHENG_CUSTOM_CHOICE) =>
        {
            resolve_resolve_dczhiheng(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_STDGAOBIAN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_stdgaobian(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_SPJIEDAO_CUSTOM_CHOICE) =>
        {
            resolve_resolve_spjiedao(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_SIBIAN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_sibian(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first()
                == Some(&crate::effect::REVEAL_CATEGORY_DISTRIBUTION_CUSTOM_CHOICE) =>
        {
            resolve_reveal_category_distribution(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first()
                == Some(&crate::effect::RANDOM_GAIN_REGION_COMPARE_COLOR_CUSTOM_CHOICE) =>
        {
            resolve_random_gain_region_compare_color(tx, observer, &payload, selections, cancelled)
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
            if payload.first() == Some(&crate::effect::RESOLVE_JSRGCHIYING_CUSTOM_CHOICE) =>
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
            if payload.first() == Some(&super::super::play::MANTIANGUOHAI_CUSTOM_CHOICE) =>
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
            crate::effect::resolve_potzhouyu_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::POTLVYI_CUSTOM_CHOICE) =>
        {
            crate::effect::resolve_potlvyi_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::POTCAOSHUANG_CUSTOM_CHOICE) =>
        {
            crate::effect::resolve_potcaoshuang_choice(
                tx, observer, &payload, selections, cancelled,
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
            crate::effect::resolve_pottaishici_choice(tx, observer, &payload, selections, cancelled)
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

        _ => super::choices_custom_potchendao::resolve_choice(
            tx,
            _actor,
            choice_id,
            observer,
            kind,
            payload,
            selections,
            cancelled,
            _vm_cont,
            _optional_trigger,
        ),
    }
}
