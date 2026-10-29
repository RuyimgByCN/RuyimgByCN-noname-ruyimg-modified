//! 效果原语分派（apply_effect）：具名技能结算器（Resolve*）。

use super::*;

pub(super) fn apply_effect(
    tx: &mut Transaction,
    req: EffectRequest,
) -> Result<Vec<DomainEvent>, EffectError> {
    match req {
        EffectRequest::ResolveTwxiawei {
            player,
            wangxing_skill_def,
            action,
            stage,
            skill,
        } => resolve_twxiawei(tx, player, wangxing_skill_def, action, stage, skill),

        EffectRequest::ResolveTwbianfuren {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            cards,
            skill,
        } => resolve_twbianfuren(
            tx,
            player,
            target,
            helper_skill_def,
            action,
            stage,
            cards,
            skill,
        ),

        EffectRequest::ResolveTwfeiyi {
            player,
            action,
            stage,
            card,
            event_cards,
            skill,
        } => resolve_twfeiyi(tx, player, action, stage, card, event_cards, skill),

        EffectRequest::ResolveTwchenzhen {
            player,
            target,
            helper_skill_def,
            record_skill_def,
            action,
            stage,
            card_kind,
            card,
            branch,
            skill,
        } => resolve_twchenzhen(
            tx,
            player,
            target,
            helper_skill_def,
            record_skill_def,
            action,
            stage,
            card_kind,
            card,
            branch,
            skill,
        ),

        EffectRequest::ResolveTwqianju {
            player,
            target,
            skill,
        } => resolve_twqianju(tx, player, target, skill),

        EffectRequest::ResolveTwqingxi {
            player,
            target,
            card,
            stage,
            count,
            cards,
            skill,
        } => resolve_twqingxi(tx, player, target, card, stage, count, cards, skill),

        EffectRequest::ResolveTwzaoli {
            player,
            stage,
            cards,
            slots,
            count,
            skill,
        } => resolve_twzaoli(tx, player, stage, cards, slots, count, skill),

        EffectRequest::ResolveTwzhengjian {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            skill,
        } => resolve_twzhengjian(
            tx,
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            skill,
        ),

        EffectRequest::ResolveTwtianyu {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            skill,
        } => resolve_twtianyu(
            tx,
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            skill,
        ),

        EffectRequest::ResolveTwwujing {
            player,
            target_count,
            action,
            event_cards,
            skill,
        } => resolve_twwujing(tx, player, target_count, action, event_cards, skill),

        EffectRequest::ResolveTwshanghe {
            player,
            stage,
            remaining,
            got_jiu,
            skill,
        } => resolve_twshanghe(tx, player, stage, remaining, got_jiu, skill),

        EffectRequest::ResolveTwwangchang {
            player,
            target,
            card,
            selected_card,
            action,
            stage,
            targets,
            skill,
        } => resolve_twwangchang(
            tx,
            player,
            target,
            card,
            selected_card,
            action,
            stage,
            targets,
            skill,
        ),

        EffectRequest::ResolveTwcaozhao {
            player,
            target,
            source,
            card,
            selected_skill_def,
            twfuzuan_skill_def,
            twfeifu_skill_def,
            action,
            stage,
            skill,
        } => resolve_twcaozhao(
            tx,
            player,
            target,
            source,
            card,
            selected_skill_def,
            twfuzuan_skill_def,
            twfeifu_skill_def,
            action,
            stage,
            skill,
        ),

        EffectRequest::ResolveTwguohuai {
            player,
            target,
            source,
            card,
            mark_def,
            helper_skill_def,
            action,
            stage,
            skill,
        } => resolve_twguohuai(
            tx,
            player,
            target,
            source,
            card,
            mark_def,
            helper_skill_def,
            action,
            stage,
            skill,
        ),

        EffectRequest::ResolveTwchengpu {
            player,
            target,
            source,
            card,
            chunlao_skill_def,
            action,
            stage,
            selected_card,
            skill,
        } => resolve_twchengpu(
            tx,
            player,
            target,
            source,
            card,
            chunlao_skill_def,
            action,
            stage,
            selected_card,
            skill,
        ),

        EffectRequest::ResolveTwcaoang {
            player,
            target,
            source,
            card,
            helper_skill_def,
            action,
            stage,
            skill,
        } => resolve_twcaoang(
            tx,
            player,
            target,
            source,
            card,
            helper_skill_def,
            action,
            stage,
            skill,
        ),

        EffectRequest::ResolveTwniujin {
            player,
            limited_skill_def,
            action,
            stage,
            skill,
        } => resolve_twniujin(tx, player, limited_skill_def, action, stage, skill),

        EffectRequest::ResolveTwmoukui {
            player,
            target,
            card,
            stage,
            skill,
        } => resolve_twmoukui(tx, player, target, card, stage, skill),

        EffectRequest::ResolveTwfuman {
            player,
            target,
            cards,
            action,
            stage,
            skill,
        } => resolve_twfuman(tx, player, target, cards, action, stage, skill),

        EffectRequest::ResolveCuijin {
            player,
            source,
            card,
            action,
            skill,
        } => resolve_cuijin(tx, player, source, card, action, skill),

        EffectRequest::ResolveTwqingkou {
            player,
            winner,
            card,
            action,
            skill,
        } => resolve_twqingkou(tx, player, winner, card, action, skill),

        EffectRequest::ResolveTwchenxun {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_twchenxun(tx, player, target, card, action, skill),

        EffectRequest::ResolveTwfenxian {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_twfenxian(tx, player, target, card, action, skill),

        EffectRequest::ResolveSpsongshu {
            player,
            phase_player,
            renku_skill_def,
            block_skill_def,
        } => resolve_spsongshu(tx, player, phase_player, renku_skill_def, block_skill_def),

        EffectRequest::ResolveSpcunsi { player, target } => resolve_spcunsi(tx, player, target),

        EffectRequest::ResolveYuli {
            player,
            mode,
            source,
            target,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            skill,
        } => resolve_yuli(
            tx, player, mode, source, target, amount, nature, card, card_kind, card_flags, skill,
        ),

        EffectRequest::ResolveTingwei {
            player,
            mode,
            card,
            targets,
            target,
            skill,
        } => resolve_tingwei(tx, player, mode, card, targets, target, skill),

        EffectRequest::ResolveXinmingshi {
            player,
            source,
            skill,
        } => resolve_xinmingshi(tx, player, source, skill),

        EffectRequest::ResolveXunyi {
            player,
            mode,
            source,
            target,
            skill,
        } => resolve_xunyi(tx, player, mode, source, target, skill),

        EffectRequest::ResolveBoming {
            player,
            target,
            card,
            skill,
        } => resolve_boming(tx, player, target, card, skill),

        EffectRequest::ResolveEjian {
            player,
            target,
            card,
            skill,
        } => resolve_ejian(tx, player, target, card, skill),

        EffectRequest::ResolveYiyong {
            player,
            card,
            source,
            skill,
        } => resolve_yiyong(tx, player, card, source, skill),

        EffectRequest::ResolveShanxie { player, skill } => resolve_shanxie(tx, player, skill),

        EffectRequest::ResolveSheyi {
            player,
            target,
            skill,
        } => resolve_sheyi(tx, player, target, skill),

        EffectRequest::ResolveTianyin { player, skill } => resolve_tianyin(tx, player, skill),

        EffectRequest::ResolveJiaohua {
            player,
            category,
            target,
            skill,
        } => resolve_jiaohua(tx, player, category, target, skill),

        EffectRequest::ResolveLuanqun {
            player,
            effect_skill_def,
            direct_hit_skill_def,
            stage,
            participants,
            cards,
            skill,
        } => resolve_luanqun(
            tx,
            player,
            effect_skill_def,
            direct_hit_skill_def,
            stage,
            participants,
            cards,
            skill,
        ),

        EffectRequest::ResolveGanglieThree {
            player,
            target,
            skill,
        } => resolve_ganglie_three(tx, player, target, skill),

        EffectRequest::ResolveZhongyi {
            player,
            action,
            helper_skill_def,
            card,
            skill,
        } => resolve_zhongyi(tx, player, action, helper_skill_def, card, skill),

        EffectRequest::ResolveOldAnxu {
            player,
            stage,
            giver,
            receiver,
            card,
            skill,
        } => resolve_oldanxu(tx, player, stage, giver, receiver, card, skill),

        EffectRequest::ResolveZhuiyi {
            player,
            killer,
            skill,
        } => resolve_zhuiyi(tx, player, killer, skill),

        EffectRequest::ResolveDecadezhenjun {
            player,
            target,
            stage,
            count,
            skill,
        } => resolve_decadezhenjun(tx, player, target, stage, count, skill),

        EffectRequest::ResolveSpXiaoguo {
            player,
            target,
            stage,
            skill,
        } => resolve_sp_xiaoguo(tx, player, target, stage, skill),

        EffectRequest::ResolveXinfuGuolun {
            player,
            target,
            player_card,
            target_card,
            skill,
        } => resolve_xinfu_guolun(tx, player, target, player_card, target_card, skill),

        EffectRequest::ResolveOldQinqing {
            player,
            target,
            lord,
            stage,
            skill,
        } => resolve_oldqinqing(tx, player, target, lord, stage, skill),

        EffectRequest::ResolveOldMingjian {
            player,
            target,
            stage,
            skill,
        } => resolve_oldmingjian(tx, player, target, stage, skill),

        EffectRequest::ResolveOldXuanfeng {
            player,
            target,
            card,
            stage,
            branch,
            skill,
        } => resolve_oldxuanfeng(tx, player, target, card, stage, branch, skill),

        EffectRequest::ResolveJianrou {
            player,
            damage_id,
            skill,
        } => resolve_jianrou(tx, player, damage_id, skill),

        EffectRequest::ResolveOldHuisheng {
            player,
            source,
            damage_id,
            stage,
            cards,
            skill,
        } => resolve_oldhuisheng(tx, player, source, damage_id, stage, cards, skill),

        EffectRequest::ResolveOldJiefan {
            player,
            dying,
            current,
            card,
            damage_id,
            stage,
            skill,
        } => resolve_oldjiefan(tx, player, dying, current, card, damage_id, stage, skill),

        EffectRequest::ResolveChegu {
            player,
            card,
            targets,
            cards,
            branch,
            index,
            stage,
            skill,
        } => resolve_chegu(
            tx, player, card, targets, cards, branch, index, stage, skill,
        ),

        EffectRequest::ResolveJunkguixin {
            player,
            target,
            stage,
            group,
            skill,
        } => resolve_junkguixin(tx, player, target, stage, group, skill),

        EffectRequest::ResolveYaoChenwei {
            player,
            target,
            card,
            mode,
            stage,
            skill,
        } => resolve_yao_chenwei(tx, player, target, card, mode, stage, skill),

        EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage {
            player,
            damage_amount,
            nature,
            stage,
            index,
            targets,
            skill,
        } => resolve_selected_equipment_discard_then_choose_damage(
            tx,
            player,
            damage_amount,
            nature,
            stage,
            index,
            targets,
            skill,
        ),

        EffectRequest::ResolveYeyan {
            player,
            mode,
            stage,
            index,
            targets,
            data,
            skill,
        } => resolve_yeyan(tx, player, mode, stage, index, targets, data, skill),

        EffectRequest::ResolveQixing {
            player,
            initialize,
            stage,
            hand_cards,
            pile_cards,
            skill,
        } => resolve_qixing(tx, player, initialize, stage, hand_cards, pile_cards, skill),

        EffectRequest::ResolveQixingWeather {
            player,
            qixing_skill_def,
            helper_skill_def,
            max_targets,
            stage,
            cards,
            targets,
            skill,
        } => resolve_qixing_weather(
            tx,
            player,
            qixing_skill_def,
            helper_skill_def,
            max_targets,
            stage,
            cards,
            targets,
            skill,
        ),

        EffectRequest::ResolveNzryLongnu {
            player,
            fire_skill_def,
            thunder_skill_def,
            stage,
            skill,
        } => resolve_nzry_longnu(tx, player, fire_skill_def, thunder_skill_def, stage, skill),

        EffectRequest::ResolveDrltPoxi {
            player,
            target,
            hand_limit_skill_def,
            stage,
            own_count,
            skill,
        } => resolve_drlt_poxi(
            tx,
            player,
            target,
            hand_limit_skill_def,
            stage,
            own_count,
            skill,
        ),

        EffectRequest::ResolveDrltJieying {
            player,
            event_player,
            mark,
            mode,
            skill,
        } => resolve_drlt_jieying(tx, player, event_player, mark, mode, skill),

        EffectRequest::ResolveDrltDuorui {
            player,
            target,
            helper_skill_def,
            stage,
            selected_skill_def,
            skill,
        } => resolve_drlt_duorui(
            tx,
            player,
            target,
            helper_skill_def,
            stage,
            selected_skill_def,
            skill,
        ),

        EffectRequest::ResolveDrltZhiti {
            player,
            event_player,
            event_source,
            event_amount,
            mode,
            skill,
        } => resolve_drlt_zhiti(
            tx,
            player,
            event_player,
            event_source,
            event_amount,
            mode,
            skill,
        ),

        EffectRequest::ResolveShenSimayi {
            player,
            event_player,
            event_amount,
            mark,
            mode,
            cards,
            skill,
            ..
        } => resolve_shen_simayi(
            tx,
            player,
            event_player,
            event_amount,
            mark,
            mode,
            cards,
            skill,
        ),

        EffectRequest::ResolveBeige {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            skill,
        } => resolve_beige(
            tx,
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            skill,
        ),

        EffectRequest::ResolveBeigeConfigured {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            heart_recover_amount,
            diamond_draw_count,
            skill,
        } => resolve_beige_configured(
            tx,
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            heart_recover_amount,
            diamond_draw_count,
            skill,
        ),

        EffectRequest::ResolveDccunsi {
            player,
            target,
            guixiu_def,
            yongjue_def,
            stage,
            skill,
        } => resolve_dccunsi(tx, player, target, guixiu_def, yongjue_def, stage, skill),

        EffectRequest::ResolveOllinjie {
            player,
            source,
            skill,
        } => resolve_ollinjie(tx, player, source, skill),

        EffectRequest::ResolveDcbingji {
            player,
            skill,
            stage,
            kind,
        } => resolve_dcbingji(tx, player, skill, stage, kind),

        EffectRequest::ResolvePxqixian { player, skill, .. } => resolve_pxqixian(tx, player, skill),

        EffectRequest::ResolveQuanjia {
            player,
            skill,
            source,
            target,
        } => resolve_quanjia(tx, player, skill, source, target),

        EffectRequest::ResolveDuwang { player, skill } => resolve_duwang(tx, player, skill),

        EffectRequest::ResolveJsrgdaoren { player, skill } => resolve_jsrgdaoren(tx, player, skill),

        EffectRequest::ResolveQianlong { player, skill } => resolve_qianlong(tx, player, skill),

        EffectRequest::ResolveJsrgzuozhan { player, skill } => {
            resolve_jsrgzuozhan(tx, player, skill)
        }

        EffectRequest::ResolveJsrgzuozhanGain {
            player,
            skill,
            event_player,
        } => resolve_jsrgzuozhan_gain(tx, player, skill, event_player),

        EffectRequest::ResolveTwmouzhi {
            player,
            skill,
            mode,
            event_card,
        } => resolve_twmouzhi(tx, player, skill, mode, event_card),

        EffectRequest::ResolveTwdaigui {
            player,
            skill,
            stage,
            count,
        } => resolve_twdaigui(tx, player, skill, stage, count),

        EffectRequest::ResolveTwmitu {
            player,
            skill,
            stage,
            count,
        } => resolve_twmitu(tx, player, skill, stage, count),

        EffectRequest::ResolveQiantun { player, skill } => resolve_qiantun(tx, player, skill, 0),

        EffectRequest::ResolveYangming { player, skill } => resolve_yangming(tx, player, skill, 0),

        EffectRequest::ResolveCanyun { player, skill } => resolve_canyun(tx, player, skill, 0),

        EffectRequest::ResolveChuanyun {
            player,
            skill,
            event_player,
            event_card,
        } => resolve_chuanyun(tx, player, skill, event_player, event_card),

        EffectRequest::ResolveQinmian {
            player,
            skill,
            mode,
            amount,
        } => resolve_qinmian(tx, player, skill, mode, amount),

        EffectRequest::ResolveBiaozhao2 {
            player,
            skill,
            event_player,
        } => resolve_biaozhao2(tx, player, skill, event_player),

        EffectRequest::ResolveChumei {
            player,
            skill,
            stage,
        } => resolve_chumei(tx, player, skill, stage),

        EffectRequest::ResolveShuaiyan {
            player,
            skill,
            stage,
        } => resolve_shuaiyan(tx, player, skill, stage),

        EffectRequest::ResolveMbbeiming {
            player,
            skill,
            stage,
        } => resolve_mbbeiming(tx, player, skill, stage),

        EffectRequest::ResolveGaoyuan {
            player,
            source,
            card,
        } => resolve_gaoyuan(tx, player, source, card),

        EffectRequest::ResolveDuoduan {
            player,
            source,
            card,
            skill,
        } => resolve_duoduan(tx, player, source, card, skill),

        EffectRequest::ResolveDcanxu {
            player,
            targets,
            skill,
        } => resolve_dcanxu(tx, player, &targets),

        EffectRequest::ResolveChulao { player, targets } => resolve_chulao(tx, player, &targets),

        EffectRequest::ResolveMeiyong { player } => resolve_meiyong(tx, player),

        EffectRequest::ResolveTwxiongjun { player } => resolve_twxiongjun(tx, player),

        EffectRequest::ResolveDcqiaolve { player, target } => resolve_dcqiaolve(tx, player, target),

        EffectRequest::ResolveDingce { player, source } => resolve_dingce(tx, player, source),

        EffectRequest::ResolveYixiang { player } => resolve_yixiang(tx, player),

        EffectRequest::ResolveXinfuQiai { player } => resolve_xinfu_qiai(tx, player),

        EffectRequest::ResolveOlwenren {
            player,
            targets,
            skill: _,
        } => resolve_olwenren(tx, player, &targets),

        EffectRequest::ResolveSpolaige { player } => resolve_spolaige(tx, player),

        EffectRequest::ResolveOlzhijie { player } => resolve_olzhijie(tx, player),

        EffectRequest::ResolveOlsuyi { player, target } => resolve_olsuyi(tx, player, target),

        EffectRequest::ResolveOlqiaozhi { player, stage } => resolve_olqiaozhi(tx, player, stage),

        EffectRequest::ResolveDczhee { player, mode } => resolve_dczhee(tx, player, mode),

        EffectRequest::ResolveDchaoxian { player } => resolve_dchaoxian(tx, player),

        EffectRequest::ResolveZhongjie { player } => resolve_zhongjie(tx, player),

        EffectRequest::ResolveStarcuxiaWeiming {
            player,
            source,
            variant,
        } => resolve_starcuxia_weiming(tx, player, source, variant),

        EffectRequest::ResolveSanshou { player } => resolve_sanshou(tx, player),

        EffectRequest::ResolveNhlundao { player, source } => resolve_nhlundao(tx, player, source),

        EffectRequest::ResolveXiangshu { player, targets } => {
            resolve_xiangshu(tx, player, &targets)
        }

        EffectRequest::ResolveQiangyong {
            player,
            target,
            skill,
        } => resolve_qiangyong(tx, player, target, skill),

        EffectRequest::ResolveMbzhenann { player } => resolve_mbzhenann(tx, player),

        EffectRequest::ResolveFriendbiwei {
            player,
            target,
            card,
            skill,
        } => resolve_friendbiwei(tx, player, target, card, skill),

        EffectRequest::ResolveZhengjian {
            player,
            skill,
            stage,
            event_player,
        } => resolve_zhengjian(tx, player, skill, stage, event_player),

        EffectRequest::ResolveReliangyin {
            player,
            skill,
            mode,
        } => resolve_reliangyin(tx, player, skill, mode),

        EffectRequest::ResolveZhongyan {
            player,
            skill,
            stage,
        } => resolve_zhongyan(tx, player, skill, stage),

        EffectRequest::ResolveDcmoukui {
            player,
            skill,
            stage,
            target,
        } => resolve_dcmoukui(tx, player, skill, stage, target),

        EffectRequest::ResolveDcniji {
            player,
            skill,
            mode,
            event_card,
        } => resolve_dcniji(tx, player, skill, mode, event_card),

        EffectRequest::ResolveHengwu {
            player,
            skill,
            event_card,
        } => resolve_hengwu(tx, player, skill, event_card),

        EffectRequest::ResolveBiaozhao3 { player, skill } => {
            resolve_biaozhao3(tx, player, skill, 0)
        }

        EffectRequest::ResolveJsrgchaozheng { player, skill } => {
            resolve_jsrgchaozheng(tx, player, skill)
        }

        _ => unreachable!("effect routed to a different primitive group"),
    }
}
