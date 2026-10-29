//! 效果原语分派（apply_effect）：pot、r_z 族结算器。

use super::*;

pub(super) fn apply_effect(
    tx: &mut Transaction,
    req: EffectRequest,
) -> Result<Vec<DomainEvent>, EffectError> {
    match req {
        EffectRequest::ResolveZhangjian {
            player,
            card,
            grant_skill_def,
            action,
            target,
            skill,
        } => resolve_zhangjian(tx, player, card, grant_skill_def, action, target, skill),

        EffectRequest::ResolveZhugeruoxue {
            player,
            target,
            card,
            action,
            remaining,
            used_mask,
            repeated,
            skill,
        } => resolve_zhugeruoxue(
            tx, player, target, card, action, remaining, used_mask, repeated, skill,
        ),

        EffectRequest::ResolveShoulijian {
            player,
            target,
            skill,
        } => resolve_shoulijian(tx, player, target, skill),

        EffectRequest::ResolvePotchengpu {
            player,
            target,
            stage,
            damage_id,
            skill,
        } => resolve_potchengpu(tx, player, target, stage, damage_id, skill),

        EffectRequest::ResolvePotjugu { player, skill } => resolve_potjugu(tx, player, skill),

        EffectRequest::ResolvePotsifeng {
            player,
            target,
            card,
            stage,
            skill,
        } => resolve_potsifeng(tx, player, target, card, stage, skill),

        EffectRequest::ResolvePotchenqun {
            player,
            target,
            stage,
            skill,
        } => resolve_potchenqun(tx, player, target, stage, skill),

        EffectRequest::ResolvePotwanglie {
            player,
            card,
            helper_skill_def,
            stage,
            skill,
        } => resolve_potwanglie(tx, player, card, helper_skill_def, stage, skill),

        EffectRequest::ResolvePothongyi {
            player,
            stage,
            skill,
        } => resolve_pothongyi(tx, player, stage, skill),

        EffectRequest::ResolvePotguansha {
            player,
            helper_skill_def,
            skill,
        } => resolve_potguansha(tx, player, helper_skill_def, skill),

        EffectRequest::ResolvePotjiyu {
            player,
            card,
            stage,
            skill,
        } => resolve_potjiyu(tx, player, card, stage, skill),

        EffectRequest::ResolvePotlibing {
            player,
            card,
            stage,
            skill,
        } => resolve_potlibing(tx, player, card, stage, skill),

        EffectRequest::ResolvePotpoxi {
            player,
            cards,
            skill,
        } => resolve_potpoxi(tx, player, cards, skill),

        EffectRequest::ResolvePotfuan {
            player,
            card,
            stage,
            range_helper_def,
            yinxian_def,
            skill,
        } => resolve_potfuan(
            tx,
            player,
            card,
            stage,
            range_helper_def,
            yinxian_def,
            skill,
        ),

        EffectRequest::ResolvePotyinxian {
            player,
            card,
            stage,
            skill,
        } => resolve_potyinxian(tx, player, card, stage, skill),

        EffectRequest::ResolvePothuanjie {
            player,
            target,
            stage,
            qice_def,
            kanpo_def,
            skill,
        } => resolve_pothuanjie(tx, player, target, stage, qice_def, kanpo_def, skill),

        EffectRequest::ResolvePotnigu {
            player,
            target,
            card,
            targets,
            index,
            declined,
            stage,
            skill,
        } => resolve_potnigu(
            tx, player, target, card, targets, index, declined, stage, skill,
        ),

        EffectRequest::ResolvePotlulian {
            player,
            card,
            target,
            stage,
            skill,
        } => resolve_potlulian(tx, player, card, target, stage, skill),

        EffectRequest::ResolvePotdimeng {
            player,
            targets,
            skill,
        } => resolve_potdimeng(tx, player, targets, skill),

        EffectRequest::ResolvePothaoshi {
            player,
            target,
            action,
            skill,
        } => resolve_pothaoshi(tx, player, target, action, skill),

        EffectRequest::ResolveSxrmlanjiao {
            player,
            target,
            stage,
            view,
            skill,
        } => resolve_sxrmlanjiao(tx, player, target, stage, view, skill),

        EffectRequest::ResolveRekuanshi { player, skill } => resolve_rekuanshi(tx, player, skill),

        EffectRequest::ResolveRekuanshiResolve {
            player,
            damaged,
            skill,
        } => resolve_rekuanshi_resolve(tx, player, damaged, skill),

        EffectRequest::ResolveXizhen { player, skill } => resolve_xizhen(tx, player, skill),

        EffectRequest::ResolveXuxie { player, skill } => resolve_xuxie(tx, player, skill),

        EffectRequest::ResolveXuxieAdd { player, skill } => resolve_xuxie_add(tx, player, skill),

        EffectRequest::ResolveShuangxiong { player, skill } => {
            resolve_shuangxiong(tx, player, skill)
        }

        EffectRequest::ResolveZhijian {
            player,
            card,
            stage,
            target,
            skill,
        } => resolve_zhijian(tx, player, card, stage, target, skill),

        EffectRequest::ResolveZhiba {
            player,
            actor,
            skill,
        } => resolve_zhiba(tx, player, actor, skill),

        EffectRequest::ResolveZhibaSettle {
            player,
            actor,
            winner,
            player_card,
            target_card,
            stage,
            skill,
        } => resolve_zhiba_settle(
            tx,
            player,
            actor,
            winner,
            player_card,
            target_card,
            stage,
            skill,
        ),

        EffectRequest::ResolveTuntian { player, skill } => resolve_tuntian(tx, player, skill),

        EffectRequest::ResolveZaoxian {
            player,
            jixi_def,
            tuntian_def,
            skill,
        } => resolve_zaoxian(tx, player, jixi_def, tuntian_def, skill),

        EffectRequest::ResolveTiaoxin {
            player,
            target,
            stage,
            skill,
        } => resolve_tiaoxin(tx, player, target, stage, skill),

        EffectRequest::ResolveZhiji {
            player,
            guanxing_def,
            skill,
        } => resolve_zhiji(tx, player, guanxing_def, skill),

        EffectRequest::ResolveZaiqi {
            player,
            extra,
            skill,
        } => resolve_zaiqi(tx, player, extra, skill),

        EffectRequest::ResolveZhengrong {
            player,
            candidates,
            target,
            stage,
            skill,
        } => resolve_zhengrong(tx, player, &candidates, target, stage, skill),

        EffectRequest::ResolveReleiji {
            player,
            target,
            stage,
            skill,
        } => resolve_releiji(tx, player, target, stage, skill),

        EffectRequest::ResolveXiangle {
            player,
            source,
            card,
            stage,
            skill,
        } => resolve_xiangle(tx, player, source, card, stage, skill),

        EffectRequest::ResolveRuoyu {
            player,
            jijiang_def,
            skill,
        } => resolve_ruoyu(tx, player, jijiang_def, skill),

        EffectRequest::ResolveZuilun {
            player,
            stage,
            skill,
        } => resolve_zuilun(tx, player, stage, skill),

        EffectRequest::ResolveShenshi {
            player,
            source,
            stage,
            skill,
        } => resolve_shenshi(tx, player, source, stage, skill),

        EffectRequest::ResolveZhenliang {
            player,
            source,
            card,
            mingren_def,
            stage,
            skill,
        } => resolve_zhenliang(tx, player, source, card, mingren_def, stage, skill),

        EffectRequest::ResolveZhizheng {
            player,
            stage,
            skill,
        } => resolve_zhizheng(tx, player, stage, skill),

        EffectRequest::ResolvePottuntian {
            player,
            action,
            amount,
            targets,
            zaoxian_def,
            skill,
            ..
        } => resolve_pottuntian(tx, player, action, amount, targets, zaoxian_def, skill),

        EffectRequest::ResolvePotjixi {
            player,
            target,
            action,
            targets,
            skill,
        } => resolve_potjixi(tx, player, target, action, targets, skill),

        EffectRequest::ResolvePotceduan {
            player,
            target,
            action,
            skill,
        } => resolve_potceduan(tx, player, target, action, skill),

        EffectRequest::ResolvePottaishici {
            player,
            target,
            action,
            cards,
            hanzhan_def,
            zhanlie_def,
            skill,
        } => resolve_pottaishici(
            tx,
            player,
            target,
            action,
            cards,
            hanzhan_def,
            zhanlie_def,
            skill,
        ),

        EffectRequest::ResolvePotjiejie {
            player,
            actor,
            used_def,
            suit_defs,
            qingshi_def,
            skill,
        } => resolve_potjiejie(tx, player, actor, used_def, suit_defs, qingshi_def, skill),

        EffectRequest::ResolvePotqingshi { player, skill } => resolve_potqingshi(tx, player, skill),

        EffectRequest::ResolvePotcaoshuang {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_potcaoshuang(tx, player, target, card, action, skill),

        EffectRequest::ResolvePotyuji {
            player,
            target,
            card,
            action,
            cards,
            targets,
            skill,
        } => resolve_potyuji(tx, player, target, card, action, &cards, &targets, skill),

        EffectRequest::ResolveRehengjiang {
            player,
            actor,
            action,
            hand_limit_def,
            draw_def,
            skill,
        } => resolve_rehengjiang(tx, player, actor, action, hand_limit_def, draw_def, skill),

        EffectRequest::ResolvePotlvyi {
            player,
            target,
            card,
            action,
            event_cards,
            targets,
            skill,
        } => resolve_potlvyi(
            tx,
            player,
            target,
            card,
            action,
            &event_cards,
            &targets,
            skill,
        ),

        EffectRequest::ResolvePotweiyan {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_potweiyan(tx, player, target, card, action, skill),

        EffectRequest::ResolvePotzhouyu {
            player,
            target,
            card,
            action,
            card_flags,
            targets,
            skill,
        } => resolve_potzhouyu(
            tx, player, target, card, action, card_flags, &targets, skill,
        ),

        EffectRequest::ResolveZhengsu {
            player,
            mode,
            stage,
            card,
            skill,
        } => resolve_zhengsu(tx, player, mode, stage, card, skill),

        EffectRequest::ResolveRechuhai {
            player,
            target,
            card,
            target_card,
            stage,
            skill,
        } => resolve_rechuhai(tx, player, target, card, target_card, stage, skill),

        EffectRequest::ResolveXiongshu {
            player,
            target,
            card,
            action,
            stage,
            skill,
        } => resolve_xiongshu(tx, player, target, card, action, stage, skill),

        EffectRequest::ResolveTongduo {
            player,
            target,
            card,
            stage,
            skill,
        } => resolve_tongduo(tx, player, target, card, stage, skill),

        EffectRequest::ResolveYuanqing { player, skill } => resolve_yuanqing(tx, player, skill),

        EffectRequest::ResolveShuchen {
            player,
            dying,
            renku_def,
            skill,
        } => resolve_shuchen(tx, player, dying, renku_def, skill),

        EffectRequest::ResolveZaoli {
            player,
            mark,
            skill,
        } => resolve_zaoli(tx, player, mark, skill),

        _ => unreachable!("effect routed to a different primitive group"),
    }
}
