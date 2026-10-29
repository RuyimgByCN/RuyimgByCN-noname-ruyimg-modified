//! Choice continuations for `Custom` requests (continued from `choices_custom`; same order).

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
