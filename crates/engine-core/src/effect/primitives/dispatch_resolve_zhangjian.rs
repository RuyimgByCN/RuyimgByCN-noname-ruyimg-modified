//! Primitive dispatch for `ResolveZhangjian` and related requests.

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

        EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
            player,
            target,
            card,
            targets,
            stage,
            skill,
        } => give_selected_then_target_may_use_draw_owner(
            tx, player, target, card, targets, stage, skill,
        ),

        EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
            player,
            target,
            damage_id,
            skill,
        } => choose_category_then_target_give_matching_or_damage_plus_one(
            tx, player, target, damage_id, skill,
        ),

        EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
            player,
            source,
            card,
            stage,
            skill,
        } => choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip(
            tx, player, source, card, stage, skill,
        ),

        EffectRequest::ResolveMobiledanshou {
            player,
            target,
            skill,
        } => resolve_mobiledanshou(tx, player, target, skill),

        EffectRequest::ResolveJsrgchiying {
            player,
            target,
            started,
            remaining,
            accum,
            skill,
        } => resolve_jsrgchiying(tx, player, target, started, remaining, accum, skill),

        EffectRequest::ResolveMbqingdao {
            player,
            source,
            card,
            damaged,
            skill,
        } => resolve_mbqingdao(tx, player, source, card, damaged, skill),

        EffectRequest::ResolveMbxiugeng {
            player,
            helper_def,
            hand_limit_def,
            stage,
            targets,
            skill,
        } => resolve_mbxiugeng(
            tx,
            player,
            helper_def,
            hand_limit_def,
            stage,
            targets,
            skill,
        ),

        EffectRequest::ResolveCaoyi {
            player,
            card,
            helper_skill_def,
            action,
            event_targets,
            skill,
        } => resolve_caoyi(
            tx,
            player,
            card,
            helper_skill_def,
            action,
            event_targets,
            skill,
        ),

        EffectRequest::ResolveMalingli {
            player,
            card,
            helper_skill_def,
            action,
            source,
            target,
            nature,
            damage_id,
            event_cards,
            event_targets,
            skill,
        } => resolve_malingli(
            tx,
            player,
            card,
            helper_skill_def,
            action,
            source,
            target,
            nature,
            damage_id,
            event_cards,
            event_targets,
            skill,
        ),

        EffectRequest::ResolveNsshuaiyan {
            player,
            recovered_player,
            skill,
        } => resolve_nsshuaiyan(tx, player, recovered_player, skill),

        EffectRequest::ResolveWdbHanlong {
            player,
            target,
            card,
            helper_skill_def,
            action,
            skill,
        } => resolve_wdb_hanlong(tx, player, target, card, helper_skill_def, action, skill),

        EffectRequest::ResolveWdbLiufu {
            player,
            target,
            card,
            zhucheng_skill_def,
            action,
            skill,
        } => resolve_wdb_liufu(tx, player, target, card, zhucheng_skill_def, action, skill),

        EffectRequest::ResolveWdbLiuyan {
            player,
            card,
            action,
            skill,
        } => resolve_wdb_liuyan(tx, player, card, action, skill),

        EffectRequest::ResolveWdbLiuzan {
            player,
            target,
            helper_skill_def,
            action,
            skill,
        } => resolve_wdb_liuzan(tx, player, target, helper_skill_def, action, skill),

        EffectRequest::ResolveWdbTianyu {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_wdb_tianyu(tx, player, target, card, action, skill),

        EffectRequest::ContinueYoudishenru {
            action,
            protected,
            source,
            forced_card,
            saved,
        } => {
            crate::command::play::continue_youdishenru(
                tx,
                action,
                protected,
                source,
                forced_card,
                &saved,
            )?;
            Ok(Vec::new())
        }

        EffectRequest::ResolveWdbXizhenxihong {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_wdb_xizhenxihong(tx, player, target, card, action, skill),

        EffectRequest::ContinueWdbFuchou {
            action,
            player,
            target,
            remaining,
            skill,
        } => continue_wdb_fuchou(tx, action, player, target, remaining, skill),

        EffectRequest::ResolveWdbYangyi {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_wdb_yangyi(tx, player, target, card, action, skill),

        EffectRequest::ContinueDiaobingqianjiang {
            action,
            actor,
            source_card,
            targets,
            revealed,
        } => continue_diaobingqianjiang(tx, action, actor, source_card, targets, revealed),

        EffectRequest::ResolveWdbZaozhiRenjun {
            player,
            target,
            card,
            cards,
            action,
            skill,
        } => resolve_wdb_zaozhirenjun(tx, player, target, card, cards, action, skill),

        EffectRequest::ResolveDddXianxi {
            player,
            card,
            action,
            event_targets,
            damaged_targets,
            skill,
        } => resolve_ddd_xianxi(
            tx,
            player,
            card,
            action,
            event_targets,
            damaged_targets,
            skill,
        ),

        EffectRequest::ResolveDddXiaheng {
            player,
            discard_target,
            damage_target,
            action,
            skill,
        } => resolve_ddd_xiaheng(tx, player, discard_target, damage_target, action, skill),

        EffectRequest::ResolveDddLiuchong {
            player,
            actor,
            card,
            action,
            cards,
            skill,
        } => resolve_ddd_liuchong(tx, player, actor, card, action, cards, skill),

        EffectRequest::ResolveDddDuanbing {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_ddd_duanbing(tx, player, target, card, action, skill),

        EffectRequest::ResolveDddWuzhi {
            player,
            card,
            action,
            cards,
            skill,
        } => resolve_ddd_wuzhi(tx, player, card, action, cards, skill),

        EffectRequest::ResolveDddXujing {
            player,
            source,
            card,
            action,
            targets,
            skill,
        } => resolve_ddd_xujing(tx, player, source, card, action, targets, skill),

        EffectRequest::ResolveDddCaomao {
            player,
            action,
            cards,
            targets,
            skill,
        } => resolve_ddd_caomao(tx, player, action, cards, targets, skill),

        EffectRequest::ResolveDddXinxianying {
            player,
            actor,
            action,
            skill,
        } => resolve_ddd_xinxianying(tx, player, actor, action, skill),

        EffectRequest::ResolveDddXianglang {
            player,
            actor,
            action,
            skill,
        } => resolve_ddd_xianglang(tx, player, actor, action, skill),

        EffectRequest::ResolveDddYujin {
            player,
            actor,
            target,
            move_mask,
            action,
            candidates,
            skill,
        } => resolve_ddd_yujin(
            tx, player, actor, target, move_mask, action, candidates, skill,
        ),

        EffectRequest::ResolveDddLiuye {
            player,
            actor,
            card,
            action,
            skill,
        } => resolve_ddd_liuye(tx, player, actor, card, action, skill),

        EffectRequest::ResolveDddBaosanniang {
            player,
            target,
            card,
            action,
            cards,
            targets,
            used_kind,
            skill,
        } => resolve_ddd_baosanniang(
            tx, player, target, card, action, cards, targets, used_kind, skill,
        ),

        EffectRequest::ResolveDddZhenji {
            player,
            target,
            action,
            stage,
            piles,
            cards,
            skill,
        } => resolve_ddd_zhenji(tx, player, target, action, stage, piles, cards, skill),

        EffectRequest::ResolveDddZhaoang {
            player,
            action,
            stage,
            card,
            kind,
            count,
            skill,
        } => resolve_ddd_zhaoang(tx, player, action, stage, card, kind, count, skill),

        EffectRequest::ResolveDddLiuba {
            player,
            actor,
            action,
            cards,
            targets,
            skill,
        } => resolve_ddd_liuba(tx, player, actor, action, cards, targets, skill),

        EffectRequest::ResolveDddJianshuo {
            player,
            target,
            action,
            cards,
            targets,
            skill,
        } => resolve_ddd_jianshuo(tx, player, target, action, cards, targets, skill),

        EffectRequest::ResolveDddGuanning {
            player,
            actor,
            action,
            card,
            skill,
        } => resolve_ddd_guanning(tx, player, actor, action, card, skill),

        EffectRequest::ResolveDddCaoshuang {
            player,
            actor,
            card,
            action,
            skill,
        } => resolve_ddd_caoshuang(tx, player, actor, card, action, skill),

        EffectRequest::ResolveDddXuelingyun {
            player,
            actor,
            card,
            cards,
            action,
            skill,
        } => resolve_ddd_xuelingyun(tx, player, actor, card, cards, action, skill),

        EffectRequest::ResolveDddLiuhong {
            player,
            actor,
            action,
            skill,
        } => resolve_ddd_liuhong(tx, player, actor, action, skill),

        EffectRequest::ResolveDddXiahouxuan {
            player,
            action,
            skill,
        } => resolve_ddd_xiahouxuan(tx, player, action, skill),

        EffectRequest::ResolveDddZhangkai {
            player,
            actor,
            card,
            cards,
            amount,
            action,
            skill,
        } => resolve_ddd_zhangkai(tx, player, actor, card, cards, amount, action, skill),

        EffectRequest::ResolveDddLiangxi {
            player,
            actor,
            card,
            cards,
            action,
            skill,
        } => resolve_ddd_liangxi(tx, player, actor, card, cards, action, skill),

        EffectRequest::GuozhanUseZhulianBihe { player } => {
            resolve_guozhan_use_zhulian_bihe(tx, player)
        }

        EffectRequest::ResolveShoulijian {
            player,
            target,
            skill,
        } => resolve_shoulijian(tx, player, target, skill),

        EffectRequest::CancelIncomingCardUse {
            player,
            source,
            card,
            skill,
        } => cancel_incoming_card_use(tx, player, source, card, skill),

        EffectRequest::ChooseOtherGainOneOfTwoSkills {
            player,
            target,
            first_def,
            second_def,
            skill,
        } => choose_other_gain_one_of_two_skills(tx, player, target, first_def, second_def, skill),

        EffectRequest::ResolveGwyewuChain {
            player,
            skill,
            cost_cards,
            wuyao_def,
            lang_def,
        } => resolve_gwyewu_chain(tx, player, skill, cost_cards, wuyao_def, lang_def),

        EffectRequest::ContinueWugu {
            action,
            actor,
            source_card,
            targets,
            excluded,
            revealed,
        } => continue_wugu(tx, action, actor, source_card, targets, excluded, revealed),

        EffectRequest::ResolveMbchenshe {
            player,
            dying,
            source,
            stage,
            roles,
            suits,
            skill,
        } => resolve_mbchenshe(tx, player, dying, source, stage, roles, suits, skill),

        EffectRequest::ResolveMbquanchong {
            player,
            helper_def,
            skill,
        } => resolve_mbquanchong(tx, player, helper_def, skill),

        EffectRequest::ResolveMbrenxing {
            player,
            stage,
            target,
            skill,
        } => resolve_mbrenxing(tx, player, stage, target, skill),

        EffectRequest::ResolveMbchizhang {
            player,
            card,
            stage,
            cards,
            skill,
        } => resolve_mbchizhang(tx, player, card, stage, cards, skill),

        EffectRequest::ResolveMbduanyang {
            player,
            target,
            card,
            mode,
            stage,
            cards,
            skill,
        } => resolve_mbduanyang(tx, player, target, card, mode, stage, cards, skill),

        EffectRequest::ResolveMbrunwei {
            player,
            target,
            stage,
            value,
            cards,
            skill,
        } => resolve_mbrunwei(tx, player, target, stage, value, cards, skill),

        EffectRequest::ResolveMbshuanghuai {
            player,
            target,
            stage,
            choice,
            skill,
        } => resolve_mbshuanghuai(tx, player, target, stage, choice, skill),

        EffectRequest::ResolveMbxiongtu {
            player,
            target,
            stage,
            card,
            cards,
            skill,
        } => resolve_mbxiongtu(tx, player, target, stage, card, cards, skill),

        EffectRequest::ResolveMbganggeng {
            player,
            target,
            stage,
            cards,
            skill,
        } => resolve_mbganggeng(tx, player, target, stage, cards, skill),

        EffectRequest::ResolveMbsijian {
            player,
            target,
            stage,
            choice,
            helper_def,
            skill,
        } => resolve_mbsijian(tx, player, target, stage, choice, helper_def, skill),

        EffectRequest::ResolveMbjianji {
            player,
            source,
            target,
            stage,
            card,
            cards,
            targets,
            skill,
        } => resolve_mbjianji(
            tx, player, source, target, stage, card, cards, targets, skill,
        ),

        EffectRequest::ResolveMbyuanmo {
            player,
            target,
            stage,
            card,
            value,
            skill,
        } => resolve_mbyuanmo(tx, player, target, stage, card, value, skill),

        EffectRequest::ResolveMbfeijing {
            player,
            target,
            card,
            stage,
            targets,
            cards,
            skill,
        } => resolve_mbfeijing(tx, player, target, card, stage, targets, cards, skill),

        EffectRequest::ResolveMbxiaoge {
            player,
            target,
            card,
            stage,
            targets,
            skill,
        } => resolve_mbxiaoge(tx, player, target, card, stage, targets, skill),

        EffectRequest::ResolveMbxuye {
            player,
            target,
            stage,
            was_max,
            skill,
        } => resolve_mbxuye(tx, player, target, stage, was_max, skill),

        EffectRequest::ResolveMbkuangxiang {
            player,
            target,
            stage,
            skill,
        } => resolve_mbkuangxiang(tx, player, target, stage, skill),

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

        EffectRequest::ResolveMbzhuji {
            player,
            choice,
            stage,
            skill,
        } => resolve_mbzhuji(tx, player, choice, stage, skill),

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

        EffectRequest::ResolveSpmiaolve {
            player,
            action,
            skill,
        } => resolve_spmiaolve(tx, player, action, skill),

        EffectRequest::ResolveSpyingjia {
            player,
            action,
            helper_def,
            card,
            card_kind,
            target,
            skill,
        } => resolve_spyingjia(
            tx, player, action, helper_def, card, card_kind, target, skill,
        ),

        EffectRequest::ResolveMantianguohai {
            player,
            stage,
            payload,
        } => resolve_mantianguohai(tx, player, stage, payload),

        EffectRequest::ResolveDcmanzhi { player, skill } => resolve_dcmanzhi(tx, player, skill),

        EffectRequest::ResolveXinxingluan { player, skill } => {
            resolve_xinxingluan(tx, player, skill)
        }

        EffectRequest::ResolveDckuizhen { player, skill } => resolve_dckuizhen(tx, player, skill),

        EffectRequest::ResolveDckuizhenResolve {
            player,
            winner,
            loser,
            skill,
        } => resolve_dckuizhen_resolve(tx, player, winner, loser, skill),

        EffectRequest::ResolveNsdingzhou { player, skill } => resolve_nsdingzhou(tx, player, skill),

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

        EffectRequest::ResolveDcmoyu { player, skill } => resolve_dcmoyu(tx, player, skill),

        EffectRequest::ResolveXizhen { player, skill } => resolve_xizhen(tx, player, skill),

        EffectRequest::ResolveXuxie { player, skill } => resolve_xuxie(tx, player, skill),

        EffectRequest::ResolveXuxieAdd { player, skill } => resolve_xuxie_add(tx, player, skill),

        EffectRequest::ResolveDcchiying {
            player,
            target,
            stage,
            accum,
            skill,
        } => resolve_dcchiying(tx, player, target, stage, accum, skill),

        EffectRequest::ResolveXinkuangfu {
            player,
            target,
            sha_target,
            stage,
            skill,
        } => resolve_xinkuangfu(tx, player, target, sha_target, stage, skill),

        EffectRequest::ResolveDcxiangshu {
            player,
            target,
            skill,
        } => resolve_dcxiangshu(tx, player, target, skill),

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

        EffectRequest::ResolveGuzheng {
            player,
            event_player,
            stage,
            cards,
            return_card,
            take_rest,
            skill,
        } => resolve_guzheng(
            tx,
            player,
            event_player,
            stage,
            cards,
            return_card,
            take_rest,
            skill,
        ),

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

        EffectRequest::ResolveDrltCongjian {
            player,
            targets,
            card,
            target,
            stage,
            skill,
        } => resolve_drlt_congjian(tx, player, targets, card, target, stage, skill),

        EffectRequest::ResolveNzryFeijun {
            player,
            target,
            card,
            branch,
            stage,
            skill,
        } => resolve_nzry_feijun(tx, player, target, card, branch, stage, skill),

        EffectRequest::ResolveDrltYongsi {
            player,
            stage,
            skill,
        } => resolve_drlt_yongsi(tx, player, stage, skill),

        EffectRequest::ResolveDrltWeidi {
            player,
            cards,
            targets,
            stage,
            skill,
        } => resolve_drlt_weidi(tx, player, cards, targets, stage, skill),

        EffectRequest::ResolveLieren {
            player,
            target,
            win_skill_def,
            lose_skill_def,
            skill,
        } => resolve_lieren(tx, player, target, win_skill_def, lose_skill_def, skill),

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

        EffectRequest::ResolveTwqiushou { player, skill } => resolve_twqiushou(tx, player, skill),

        EffectRequest::ResolveZhengrong {
            player,
            candidates,
            target,
            stage,
            skill,
        } => resolve_zhengrong(tx, player, &candidates, target, stage, skill),

        EffectRequest::ResolveHongju {
            player,
            qingjian_def,
            rong_def,
            stage,
            skill,
        } => resolve_hongju(tx, player, qingjian_def, rong_def, stage, skill),

        EffectRequest::ResolveQingjian {
            player,
            rong_def,
            stage,
            skill,
        } => resolve_qingjian(tx, player, rong_def, stage, skill),

        EffectRequest::ResolveReleiji {
            player,
            target,
            stage,
            skill,
        } => resolve_releiji(tx, player, target, stage, skill),

        EffectRequest::ResolveHuangtian {
            player,
            actor,
            stage,
            skill,
        } => resolve_huangtian(tx, player, actor, stage, skill),

        EffectRequest::ResolveHaoshi {
            player,
            stage,
            skill,
        } => resolve_haoshi(tx, player, stage, skill),

        EffectRequest::ResolveDimeng {
            player,
            targets,
            stage,
            skill,
        } => resolve_dimeng(tx, player, &targets, stage, skill),

        EffectRequest::ResolveQiaobian {
            player,
            phase,
            stage,
            skill,
        } => resolve_qiaobian(tx, player, phase, stage, skill),
        // ponytail: 巧变（ABI 559）由 C 段（omp）实现中，stub 保持编译通过。
        EffectRequest::ResolveQiaobian { .. } => Ok(Vec::new()),

        EffectRequest::ExcludeSavedShaTarget { card, target } => {
            exclude_saved_sha_target(tx, card, target)
        }

        EffectRequest::ResolveXiangle {
            player,
            source,
            card,
            stage,
            skill,
        } => resolve_xiangle(tx, player, source, card, stage, skill),

        EffectRequest::ResolveFangquanSkip { player, skill } => {
            resolve_fangquan_skip(tx, player, skill)
        }

        EffectRequest::ResolveFangquanGrant {
            player,
            stage,
            skill,
        } => resolve_fangquan_grant(tx, player, stage, skill),

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

        EffectRequest::ResolveFuyin {
            player,
            source,
            card,
            skill,
        } => resolve_fuyin(tx, player, source, card, skill),

        EffectRequest::BanUseAgainst { user, target } => ban_use_against(tx, user, target),

        EffectRequest::ResolveJuzhan {
            player,
            source,
            card,
            targets,
            stage,
            skill,
        } => resolve_juzhan(tx, player, source, card, &targets, stage, skill),
        // ponytail: 并行会话中间态（ban-use-pairs / 拒战）stub 保持编译通过。
        EffectRequest::BanUseAgainst { .. } | EffectRequest::ResolveJuzhan { .. } => Ok(Vec::new()),

        EffectRequest::ResolveQizhi {
            player,
            targets,
            stage,
            skill,
        } => resolve_qizhi(tx, player, &targets, stage, skill),

        EffectRequest::ResolveJinqu {
            player,
            qizhi_def,
            stage,
            skill,
        } => resolve_jinqu(tx, player, qizhi_def, stage, skill),

        EffectRequest::ResolveJianxiang {
            player,
            stage,
            skill,
        } => resolve_jianxiang(tx, player, stage, skill),

        EffectRequest::ResolveShenshi {
            player,
            source,
            stage,
            skill,
        } => resolve_shenshi(tx, player, source, stage, skill),

        EffectRequest::ResolveMingren {
            player,
            stage,
            skill,
        } => resolve_mingren(tx, player, stage, skill),

        EffectRequest::ResolveZhenliang {
            player,
            source,
            card,
            mingren_def,
            stage,
            skill,
        } => resolve_zhenliang(tx, player, source, card, mingren_def, stage, skill),

        EffectRequest::ResolveKuizhu {
            player,
            stage,
            skill,
        } => resolve_kuizhu(tx, player, stage, skill),

        EffectRequest::ResolveZhizheng {
            player,
            stage,
            skill,
        } => resolve_zhizheng(tx, player, stage, skill),

        EffectRequest::ResolveLijun {
            player,
            source,
            card,
            stage,
            skill,
        } => resolve_lijun(tx, player, source, card, stage, skill),

        EffectRequest::ResolveLiangyin {
            player,
            kongsheng_def,
            stage,
            target,
            skill,
        } => resolve_liangyin(tx, player, kongsheng_def, stage, target, skill),

        EffectRequest::ResolveKongsheng {
            player,
            stage,
            target,
            card,
            skill,
        } => resolve_kongsheng(tx, player, stage, target, card, skill),

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

        EffectRequest::ResolveMbzhonghui {
            player,
            target,
            action,
            amount,
            mbsizi_def,
            mbkechang_def,
            skill,
        } => resolve_mbzhonghui(
            tx,
            player,
            target,
            action,
            amount,
            mbsizi_def,
            mbkechang_def,
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

        EffectRequest::ResolveMuzhen { .. } => Err(EffectError::ContentFault(
            "resolve_muzhen pending implementation".into(),
        )),

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

        EffectRequest::ResolveDbzhuifeng { player, skill } => resolve_dbzhuifeng(tx, player, skill),

        EffectRequest::ResolveDbquedi {
            player,
            target,
            card,
            skill,
        } => resolve_dbquedi(tx, player, target, card, skill),

        EffectRequest::ResolveSpyinju {
            player,
            target,
            skill,
        } => resolve_spyinju(tx, player, target, skill),

        EffectRequest::ResolveSpchijie {
            player,
            source,
            card,
            skill,
        } => resolve_spchijie(tx, player, source, card, skill),

        EffectRequest::ResolveSpshangyi {
            player,
            target,
            skill,
        } => resolve_spshangyi(tx, player, target, skill),

        EffectRequest::ResolveSpjianyi { player, skill } => resolve_spjianyi(tx, player, skill),

        EffectRequest::ResolveFyjianyu {
            player,
            source,
            target,
            targets,
            skill,
        } => resolve_fyjianyu(tx, player, source, target, targets, skill),

        EffectRequest::ResolveLiubing {
            player,
            card,
            skill,
        } => resolve_liubing(tx, player, card, skill),

        EffectRequest::ResolveHeji {
            player,
            target,
            card,
            skill,
        } => resolve_heji(tx, player, target, card, skill),

        EffectRequest::ResolveSpyajun {
            player,
            less_def,
            card,
            target,
            target_card,
            stage,
            skill,
        } => resolve_spyajun(
            tx,
            player,
            less_def,
            card,
            target,
            target_card,
            stage,
            skill,
        ),

        EffectRequest::ResolveSpzundi {
            player,
            target,
            card,
            destination,
            stage,
            skill,
        } => resolve_spzundi(tx, player, target, card, destination, stage, skill),

        EffectRequest::ResolveXinyangjie {
            player,
            target,
            card,
            target_card,
            source,
            stage,
            skill,
        } => resolve_xinyangjie(tx, player, target, card, target_card, source, stage, skill),

        EffectRequest::ResolveQingyu {
            player,
            action,
            xuancun_def,
            mbyongjue_def,
            target,
            skill,
        } => resolve_qingyu(
            tx,
            player,
            action,
            xuancun_def,
            mbyongjue_def,
            target,
            skill,
        ),

        EffectRequest::ResolveZhengsu {
            player,
            mode,
            stage,
            card,
            skill,
        } => resolve_zhengsu(tx, player, mode, stage, card, skill),

        EffectRequest::ResolveHoufeng {
            player,
            target,
            mode,
            stage,
            skill,
        } => resolve_houfeng(tx, player, target, mode, stage, skill),

        EffectRequest::ResolveMbdiancai {
            player,
            actor,
            skill,
        } => resolve_mbdiancai(tx, player, actor, skill),

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

        EffectRequest::ResolveJianhui {
            player,
            source,
            target,
            skill,
        } => resolve_jianhui(tx, player, source, target, skill),

        EffectRequest::ResolveSpqiai {
            player,
            target,
            card,
            stage,
            skill,
        } => resolve_spqiai(tx, player, target, card, stage, skill),

        EffectRequest::ResolveSpshanxi {
            player,
            target,
            stage,
            skill,
        } => resolve_spshanxi(tx, player, target, stage, skill),

        EffectRequest::ResolveSpshanxiSuoming {
            player,
            actor,
            stage,
            skill,
        } => resolve_spshanxi_suoming(tx, player, actor, stage, skill),

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

        EffectRequest::ResolveDuanbi {
            player,
            targets,
            stage,
            skill,
        } => resolve_duanbi(tx, player, &targets, stage, skill),

        EffectRequest::ResolveZaoli {
            player,
            mark,
            skill,
        } => resolve_zaoli(tx, player, mark, skill),

        EffectRequest::ResolveGuying {
            player,
            actor,
            card,
            stage,
            skill,
        } => resolve_guying(tx, player, actor, card, stage, skill),

        EffectRequest::ResolveDcxiangshuLottery {
            player,
            target,
            skill,
        } => resolve_dcxiangshu_lottery(tx, player, target, skill),

        EffectRequest::ResolveDcyingtu {
            player,
            target,
            stage,
            cards,
            skill,
        } => resolve_dcyingtu(tx, player, target, stage, cards, skill),
        _ => unreachable!("effect routed to a different primitive group"),
    }
}
