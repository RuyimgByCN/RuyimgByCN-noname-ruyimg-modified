//! 效果原语分派（apply_effect）：具名技能结算器（Resolve*）。

use super::*;

pub(super) fn apply_effect(
    tx: &mut Transaction,
    req: EffectRequest,
) -> Result<Vec<DomainEvent>, EffectError> {
    match req {
        EffectRequest::ResolveChenglve {
            player,
            stage,
            suits,
            skill,
        } => resolve_chenglve(tx, player, stage, suits, skill),

        EffectRequest::ResolveShicai {
            player,
            card,
            placed,
            stage,
            skill,
        } => resolve_shicai(tx, player, card, placed, stage, skill),

        EffectRequest::ResolveXiongluan {
            player,
            target,
            stage,
            skill,
        } => resolve_xiongluan(tx, player, target, stage, skill),

        EffectRequest::ResolveGzbuqu {
            player,
            delta,
            stage,
            skill,
        } => resolve_gzbuqu(tx, player, delta, stage, skill),

        EffectRequest::ResolveGzbuquRecover {
            player,
            stage,
            skill,
        } => resolve_gzbuqu_recover(tx, player, stage, skill),

        EffectRequest::ResolveSbbiyue { player, skill } => resolve_sbbiyue(tx, player, skill),

        EffectRequest::ResolveSxrmqiewang {
            player,
            event_player,
            skill,
        } => resolve_sxrmqiewang(tx, player, event_player, skill),

        EffectRequest::ResolveSxrmqianliu {
            player,
            source,
            stage,
            skill,
        } => resolve_sxrmqianliu(tx, player, source, stage, skill),

        EffectRequest::ResolveTwjuxiang {
            player,
            event_player,
            stage,
            skill,
        } => resolve_twjuxiang(tx, player, event_player, stage, skill),

        EffectRequest::ResolveMbzhuguo {
            player,
            target,
            stage,
            skill,
        } => resolve_mbzhuguo(tx, player, target, stage, skill),

        EffectRequest::ResolveMbanxian {
            player,
            cards,
            card,
            stage,
            skill,
        } => resolve_mbanxian(tx, player, cards, card, stage, skill),

        EffectRequest::ResolveMbjinzu {
            player,
            target,
            cards,
            card,
            helper_skill_def,
            stage,
            skill,
        } => resolve_mbjinzu(
            tx,
            player,
            target,
            cards,
            card,
            helper_skill_def,
            stage,
            skill,
        ),

        EffectRequest::ResolveDingpan {
            player,
            target,
            stage,
            skill,
        } => resolve_dingpan(tx, player, target, stage, skill),

        EffectRequest::ResolveTanlu {
            player,
            target,
            amount,
            stage,
            skill,
        } => resolve_tanlu(tx, player, target, amount, stage, skill),

        EffectRequest::ResolveHiddenSkillPile {
            player,
            pile_skill_def,
            action,
            card,
            stage,
            hand_cards,
            pile_cards,
            event_cards,
            skill,
        } => resolve_hidden_skill_pile(
            tx,
            player,
            pile_skill_def,
            action,
            card,
            stage,
            hand_cards,
            pile_cards,
            event_cards,
            skill,
        ),

        EffectRequest::ResolveDiliYuheng {
            player,
            action,
            card,
            event_cards,
            skill,
        } => resolve_dili_yuheng(tx, player, action, card, event_cards, skill),

        EffectRequest::ResolveMassDamageDiscardTurnOver {
            player,
            mark,
            mark_count,
            damage_amount,
            hand_count,
            stage,
            index,
            targets,
            skill,
        } => resolve_mass_damage_discard_turn_over(
            tx,
            player,
            mark,
            mark_count,
            damage_amount,
            hand_count,
            stage,
            index,
            targets,
            skill,
            false,
            true,
        ),

        EffectRequest::ResolveMassDamageDiscardHandEquip {
            player,
            mark,
            mark_count,
            damage_amount,
            discard_count,
            stage,
            index,
            targets,
            skill,
        } => resolve_mass_damage_discard_turn_over(
            tx,
            player,
            mark,
            mark_count,
            damage_amount,
            discard_count,
            stage,
            index,
            targets,
            skill,
            true,
            false,
        ),

        EffectRequest::ResolveMarkParityControl {
            player,
            mark,
            mass_threshold,
            damage_amount,
            stage,
            index,
            target,
            targets,
            skill,
        } => resolve_mark_parity_control(
            tx,
            player,
            mark,
            mass_threshold,
            damage_amount,
            stage,
            index,
            target,
            targets,
            skill,
        ),

        EffectRequest::ResolveGuanchaoRank {
            player,
            card,
            skill,
        } => resolve_guanchao_rank(tx, player, card, skill),

        EffectRequest::ResolveGuanwei {
            player,
            target,
            stage,
            card,
            skill,
        } => resolve_guanwei(tx, player, target, stage, card, skill),

        EffectRequest::ResolveLanjiang {
            player,
            stage,
            index,
            target,
            targets,
            skill,
        } => resolve_lanjiang(tx, player, stage, index, target, targets, skill),

        EffectRequest::ResolveRebingyi { player, skill } => resolve_rebingyi(tx, player, skill),

        EffectRequest::ResolveRebotu { player, skill } => resolve_rebotu(tx, player, skill),

        EffectRequest::ResolveOldimeng {
            player,
            stage,
            first,
            second,
            skill,
        } => resolve_oldimeng(tx, player, stage, first, second, skill),

        EffectRequest::ResolveRejieyin {
            player,
            target,
            stage,
            card,
            equip,
            skill,
        } => resolve_rejieyin(tx, player, target, stage, card, equip, skill),

        EffectRequest::ResolveReanxu {
            player,
            stage,
            first,
            second,
            card,
            from_hand,
            skill,
        } => resolve_reanxu(tx, player, stage, first, second, card, from_hand, skill),

        EffectRequest::ResolveDrltJueyan {
            player,
            stage,
            slot,
            first,
            sha_skill_def,
            weapon_skill_def,
            mount_skill_def,
            armor_skill_def,
            treasure_skill_def,
            skill,
        } => resolve_drlt_jueyan(
            tx,
            player,
            stage,
            slot,
            first,
            sha_skill_def,
            weapon_skill_def,
            mount_skill_def,
            armor_skill_def,
            treasure_skill_def,
            skill,
        ),

        EffectRequest::ResolveRejizhi {
            player,
            stage,
            card,
            skill,
        } => resolve_rejizhi(tx, player, stage, card, skill),

        EffectRequest::ResolveJiexun {
            player,
            stage,
            target,
            draw_count,
            discard_count,
            funan_helper_skill_def,
            skill,
        } => resolve_jiexun(
            tx,
            player,
            stage,
            target,
            draw_count,
            discard_count,
            funan_helper_skill_def,
            skill,
        ),

        EffectRequest::ResolveWenguaSetup {
            player,
            helper_skill_def,
            skill,
        } => resolve_wengua_setup(tx, player, helper_skill_def, skill),

        EffectRequest::ResolveWengua {
            player,
            stage,
            target,
            card,
            branch,
            main_skill_def,
            marker_skill_def,
            skill,
        } => resolve_wengua(
            tx,
            player,
            stage,
            target,
            card,
            branch,
            main_skill_def,
            marker_skill_def,
            skill,
        ),

        EffectRequest::ResolveXinanguo {
            player,
            target,
            stage,
            index,
            remaining,
            skill,
        } => resolve_xinanguo(tx, player, target, stage, index, remaining, skill),

        EffectRequest::ResolveReanguo {
            player,
            target,
            stage,
            index,
            remaining,
            skill,
        } => resolve_reanguo(tx, player, target, stage, index, remaining, skill),

        EffectRequest::ResolveBeizhu {
            player,
            target,
            stage,
            cards,
            skill,
        } => resolve_beizhu(tx, player, target, stage, cards, skill),

        EffectRequest::ResolveMbshanjia {
            player,
            lost_equipment_count,
            distance_skill_def,
            stage,
            reward_mask,
            skill,
        } => resolve_mbshanjia(
            tx,
            player,
            lost_equipment_count,
            distance_skill_def,
            stage,
            reward_mask,
            skill,
        ),

        EffectRequest::ResolveDaoji {
            player,
            target,
            card,
            stage,
            tw,
            skill,
        } => resolve_daoji(tx, player, target, card, stage, tw, skill),

        EffectRequest::ResolveTwfupan {
            player,
            damage_amount,
            card,
            target,
            stage,
            skill,
        } => resolve_twfupan(tx, player, damage_amount, card, target, stage, skill),

        EffectRequest::ResolveTwjiaohua {
            player,
            target,
            event_cards,
            skill,
        } => resolve_twjiaohua(tx, player, target, &event_cards, skill),

        EffectRequest::ResolveTwkujian {
            player,
            target,
            cards,
            action,
            skill,
        } => resolve_twkujian(tx, player, target, &cards, action, skill),

        EffectRequest::ResolveTwruilian {
            player,
            target,
            event_cards,
            action,
            category,
            skill,
        } => resolve_twruilian(tx, player, target, &event_cards, action, category, skill),

        EffectRequest::ResolveTwjianwei {
            player,
            actor,
            target,
            card,
            winner,
            stage,
            skill,
        } => resolve_twjianwei(tx, player, actor, target, card, winner, stage, skill),

        EffectRequest::ResolveTwtanfeng {
            player,
            target,
            card,
            stage,
            phase,
            skill,
        } => resolve_twtanfeng(tx, player, target, card, stage, phase, skill),

        EffectRequest::ResolveTwzhibian {
            player,
            target,
            winner,
            stage,
            skill,
        } => resolve_twzhibian(tx, player, target, winner, stage, skill),

        EffectRequest::ResolveTwyuyan {
            player,
            source,
            card,
            skill,
        } => resolve_twyuyan(tx, player, source, card, skill),

        EffectRequest::ResolveTwguoyi {
            player,
            target,
            card,
            hand_lock_skill_def,
            skill,
        } => resolve_twguoyi(tx, player, target, card, hand_lock_skill_def, skill),

        EffectRequest::ResolveTwchuhai {
            player,
            action,
            stage,
            index,
            event_cards,
            skill,
        } => resolve_twchuhai(tx, player, action, stage, index, event_cards, skill),

        EffectRequest::ResolveTwyizhu {
            player,
            source,
            card,
            action,
            stage,
            event_cards,
            skill,
        } => resolve_twyizhu(tx, player, source, card, action, stage, event_cards, skill),

        EffectRequest::ResolveTwluanchou {
            player,
            helper_skill_def,
            action,
            targets,
            source,
            target,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            skill,
        } => resolve_twluanchou(
            tx,
            player,
            helper_skill_def,
            action,
            targets,
            source,
            target,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            skill,
        ),

        EffectRequest::ResolveTianxiang {
            player,
            target,
            discard_card,
            source,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            draw_skill_def,
            skill,
        } => resolve_tianxiang(
            tx,
            player,
            target,
            discard_card,
            source,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            draw_skill_def,
            skill,
        ),

        EffectRequest::ResolveRetianxiang {
            player,
            target,
            discard_card,
            branch,
            source,
            skill,
        } => resolve_retianxiang(tx, player, target, discard_card, branch, source, skill),

        EffectRequest::ResolveLianzhou {
            player,
            targets,
            skill,
        } => resolve_lianzhou(tx, player, targets, skill),

        EffectRequest::ResolveStddechong {
            player,
            target,
            cards,
            skill,
        } => resolve_stddechong(tx, player, target, cards, skill),

        EffectRequest::ResolveJieliangGain {
            player,
            mover,
            cards,
            skill,
        } => resolve_jieliang_gain(tx, player, mover, cards, skill),

        EffectRequest::ResolveStdjujian {
            player,
            target,
            cards,
            skill,
        } => resolve_stdjujian(tx, player, target, cards, skill),

        EffectRequest::ResolveLulve {
            player,
            target,
            branch,
            skill,
        } => resolve_lulve(tx, player, target, branch, skill),

        EffectRequest::ResolveDctingxian {
            player,
            card,
            targets,
            skill,
        } => resolve_dctingxian(tx, player, card, targets, skill),

        EffectRequest::ResolveHfjieying {
            player,
            target,
            grant_def,
            skill,
        } => resolve_hfjieying(tx, player, target, grant_def, skill),

        EffectRequest::ResolveDcnuchen {
            player,
            target,
            skill,
        } => resolve_dcnuchen(tx, player, target, skill),

        EffectRequest::ResolveDcdanji { player, skill } => resolve_dcdanji(tx, player, skill),

        EffectRequest::ResolveDczhiheng { player, skill } => resolve_dczhiheng(tx, player, skill),

        EffectRequest::ResolveStdgaobian {
            player,
            chooser,
            skill,
        } => resolve_stdgaobian(tx, player, chooser, skill),

        EffectRequest::ResolveSpjiedao { player, skill } => resolve_spjiedao(tx, player, skill),

        EffectRequest::ResolveSibian { player, skill } => resolve_sibian(tx, player, skill),

        EffectRequest::ResolveSpfengyin {
            player,
            target,
            skill,
        } => resolve_spfengyin(tx, player, target, skill),

        EffectRequest::ResolveXinfuLianpian {
            player,
            targets,
            skill,
        } => resolve_xinfu_lianpian(tx, player, targets, skill),

        EffectRequest::ResolveGongjian {
            player,
            targets,
            skill,
        } => resolve_gongjian(tx, player, targets, skill),

        EffectRequest::ResolveXinfuQinguo {
            player,
            card,
            skill,
        } => resolve_xinfu_qinguo(tx, player, card, skill),

        EffectRequest::ResolveRefuhai { player, skill } => resolve_refuhai(tx, player, skill),

        EffectRequest::ResolveGzjinfa {
            player,
            target,
            cost_card,
            given_card,
            stage,
            skill,
        } => resolve_gzjinfa(tx, player, target, cost_card, given_card, stage, skill),

        EffectRequest::ResolveWnQifeng {
            player,
            target,
            skill,
        } => resolve_wn_qifeng(tx, player, target, skill),

        EffectRequest::ResolveDcjue {
            player,
            target,
            stage,
            remaining,
            skill,
        } => resolve_dcjue(tx, player, target, stage, remaining, skill),

        EffectRequest::ResolveRuanji {
            player,
            source,
            card,
            action,
            cards,
            skill,
        } => resolve_ruanji(tx, player, source, card, action, cards, skill),

        EffectRequest::ResolveZhengbi {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_zhengbi(tx, player, target, card, action, skill),

        EffectRequest::ResolveFengying {
            player,
            helper_def,
            action,
            skill,
        } => resolve_fengying(tx, player, helper_def, action, skill),

        EffectRequest::ResolveQiaojian {
            player,
            target,
            helper_def,
            action,
            cards,
            split,
            skill,
        } => resolve_qiaojian(tx, player, target, helper_def, action, cards, split, skill),

        EffectRequest::ResolveXicha {
            player,
            source,
            card,
            action,
            cards,
            skill,
        } => resolve_xicha(tx, player, source, card, action, cards, skill),

        EffectRequest::ResolveHuzun {
            player,
            source,
            card,
            action,
            cards,
            targets,
            skill,
        } => resolve_huzun(tx, player, source, card, action, cards, targets, skill),

        EffectRequest::ResolveZhugemengxue {
            player,
            target,
            card,
            action,
            cards,
            skill,
        } => resolve_zhugemengxue(tx, player, target, card, action, cards, skill),

        EffectRequest::ResolveBailingyun {
            player,
            target,
            card,
            action,
            cards,
            skill,
        } => resolve_bailingyun(tx, player, target, card, action, cards, skill),

        EffectRequest::ResolveDcshaowei {
            player,
            action,
            cards,
            skill,
        } => resolve_dcshaowei(tx, player, action, cards, skill),

        EffectRequest::ResolveDcdichou {
            player,
            target,
            card,
            action,
            helper_skill_def,
            skill,
        } => resolve_dcdichou(tx, player, target, card, action, helper_skill_def, skill),

        EffectRequest::ResolveDcdufeng {
            player,
            action,
            helper_skill_def,
            skill,
        } => resolve_dcdufeng(tx, player, action, helper_skill_def, skill),

        EffectRequest::ResolveDcctjiuxian {
            player,
            event_player,
            target,
            card,
            action,
            cards,
            skill,
        } => resolve_dcctjiuxian(tx, player, event_player, target, card, action, cards, skill),

        EffectRequest::ResolveBianyue {
            player,
            target,
            action,
            skill,
        } => resolve_bianyue(tx, player, target, action, skill),

        EffectRequest::ResolveZhupeilan {
            player,
            target,
            card,
            helper_skill_def,
            action,
            event_cards,
            skill,
        } => resolve_zhupeilan(
            tx,
            player,
            target,
            card,
            helper_skill_def,
            action,
            event_cards,
            skill,
        ),

        EffectRequest::ResolveDcSbZhangxiu {
            player,
            target,
            action,
            cards,
            skill,
        } => resolve_dc_sb_zhangxiu(tx, player, target, action, cards, skill),

        EffectRequest::ResolveDcSbZhouyu {
            player,
            source,
            card,
            action,
            event_targets,
            damaged_targets,
            skill,
        } => resolve_dc_sb_zhouyu(
            tx,
            player,
            source,
            card,
            action,
            event_targets,
            damaged_targets,
            skill,
        ),

        EffectRequest::ResolveDcSbLusu {
            player,
            source,
            card,
            action,
            cards,
            amount,
            skill,
        } => resolve_dc_sb_lusu(tx, player, source, card, action, cards, amount, skill),
        _ => unreachable!("effect routed to a different primitive group"),
    }
}
