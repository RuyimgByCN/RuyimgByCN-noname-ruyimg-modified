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
            if payload.first() == Some(&crate::effect::RESOLVE_SPFENGYIN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_spfengyin(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_XINFU_LIANPIAN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_xinfu_lianpian(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_GONGJIAN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_gongjian(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_XINFU_QINGUO_CUSTOM_CHOICE) =>
        {
            resolve_resolve_xinfu_qinguo(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DCMANZHI_CUSTOM_CHOICE) =>
        {
            resolve_resolve_dcmanzhi(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_XINXINGLUAN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_xinxingluan(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DCKUIZHEN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_dckuizhen(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_NSDINGZHOU_CUSTOM_CHOICE) =>
        {
            resolve_resolve_nsdingzhou(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_SXRMLANJIAO_CUSTOM_CHOICE) =>
        {
            resolve_resolve_sxrmlanjiao(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_REKUANSHI_CUSTOM_CHOICE) =>
        {
            resolve_resolve_rekuanshi(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DCMOYU_CUSTOM_CHOICE) =>
        {
            resolve_resolve_dcmoyu(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_XIZHEN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_xizhen(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_XUXIE_CUSTOM_CHOICE) =>
        {
            resolve_resolve_xuxie(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DCCHIYING_CUSTOM_CHOICE) =>
        {
            resolve_resolve_dcchiying(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_XINKUANGFU_CUSTOM_CHOICE) =>
        {
            resolve_resolve_xinkuangfu(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DCXIANG_SHU_CUSTOM_CHOICE) =>
        {
            resolve_resolve_dcxiangshu(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_ZHIJIAN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_zhijian(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_GUZHENG_CUSTOM_CHOICE) =>
        {
            resolve_resolve_guzheng(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_ZHIBA_CUSTOM_CHOICE) =>
        {
            resolve_resolve_zhiba(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_TIAOXIN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_tiaoxin(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_ZHENGRONG_CUSTOM_CHOICE) =>
        {
            resolve_resolve_zhengrong(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_HONGJU_CUSTOM_CHOICE) =>
        {
            resolve_resolve_hongju(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_QINGJIAN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_qingjian(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_RELEIJI_CUSTOM_CHOICE) =>
        {
            resolve_resolve_releiji(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_HUANGTIAN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_huangtian(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_HAOSHI_CUSTOM_CHOICE) =>
        {
            resolve_resolve_haoshi(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DIMENG_CUSTOM_CHOICE) =>
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

        ContinuationKind::Custom if payload.first() == Some(&crate::effect::RESOLVE_QIZHI_CARD) => {
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
            if payload.first() == Some(&crate::effect::RESOLVE_DRLT_CONGJIAN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_drlt_congjian(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_NZRY_FEIJUN_CUSTOM_CHOICE) =>
        {
            resolve_resolve_nzry_feijun(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DRLT_WEIDI_CUSTOM_CHOICE) =>
        {
            resolve_resolve_drlt_weidi(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DCYINGTU_CUSTOM_CHOICE) =>
        {
            resolve_resolve_dcyingtu(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_TINGWEI_SELECT) =>
        {
            resolve_tingwei_select(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_TINGWEI_OPTION) =>
        {
            resolve_tingwei_option(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_XINMINGSHI_CUSTOM_CHOICE) =>
        {
            resolve_xinmingshi_discard(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_XUNYI_SELECT) =>
        {
            resolve_xunyi_select(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_EJIAN_CHOICE) =>
        {
            resolve_ejian_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_SHEYI_CHOICE) =>
        {
            resolve_sheyi_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DCJUE_CHOICE) =>
        {
            resolve_dcjue_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_RUANJI_CHOICE) =>
        {
            resolve_ruanji_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_ZHENGBI_CHOICE) =>
        {
            resolve_zhengbi_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_QIAOJIAN_CHOICE) =>
        {
            resolve_qiaojian_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_XICHA_CHOICE) =>
        {
            resolve_xicha_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_HUZUN_CHOICE) =>
        {
            resolve_huzun_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_ZHUGEMENGXUE_CHOICE) =>
        {
            resolve_zhugemengxue_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_BAILINGYUN_CHOICE) =>
        {
            resolve_bailingyun_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DCDICHOU_CHOICE) =>
        {
            resolve_dcdichou_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DCDUFENG_CHOICE) =>
        {
            resolve_dcdufeng_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_CHENGLVE_DISCARD) =>
        {
            resolve_chenglve_discard(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_SHICAI_PLACE) =>
        {
            resolve_shicai_place(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_TWJUXIANG_CARD) =>
        {
            resolve_twjuxiang_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_GZBUQU_RECOVER_CARD) =>
        {
            resolve_gzbuqu_recover_card(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DBZHUIFENG_TARGET) =>
        {
            resolve_dbzhuifeng_target(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DBQUEDI_CHOICE) =>
        {
            resolve_dbquedi_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_SPYINJU_CHOICE) =>
        {
            resolve_spyinju_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_SPSHANGYI_CHOICE) =>
        {
            resolve_spshangyi_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_HEJI_CHOICE) =>
        {
            resolve_heji_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_SPYAJUN_CUSTOM_CHOICE) =>
        {
            resolve_spyajun_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_SPZUNDI_CUSTOM_CHOICE) =>
        {
            resolve_spzundi_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_XINYANGJIE_CUSTOM_CHOICE) =>
        {
            resolve_xinyangjie_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_QINGYU_CUSTOM_CHOICE) =>
        {
            resolve_qingyu_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_QINGYU_FAIL_TARGET) =>
        {
            resolve_qingyu_fail_target(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_HOUFENG_CHOICE) =>
        {
            resolve_houfeng_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_RECHUHAI_CHOICE) =>
        {
            resolve_rechuhai_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_XIONGSHU_CHOICE) =>
        {
            resolve_xiongshu_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_SPQIAI_CHOICE) =>
        {
            resolve_spqiai_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_SPSHANXI_CHOICE) =>
        {
            resolve_spshanxi_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_SUOMING_CHOICE) =>
        {
            resolve_suoming_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_TONGDUO_CHOICE) =>
        {
            resolve_tongduo_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_DUANBI_CHOICE) =>
        {
            resolve_duanbi_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_ZAOLI_CHOICE) =>
        {
            resolve_zaoli_choice(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::Custom
            if payload.first() == Some(&crate::effect::RESOLVE_GUYING_CHOICE) =>
        {
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

        ContinuationKind::Custom if payload.first() == Some(&crate::mode::LOUREN_XIN_CHOICE) => {
            // ADR 0598: guozhan 拉拢人心 (ambition-lord alliance) choice resolution.
            crate::mode::resolve_lourenxin_choice(tx.state_mut(), observer, selections, cancelled);
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
        _ => unreachable!("choice routed to a different continuation group"),
    }
}
