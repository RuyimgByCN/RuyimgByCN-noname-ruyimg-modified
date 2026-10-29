//! 效果原语分派（apply_effect）：具名技能结算器（Resolve*）。

use super::*;

pub(super) fn apply_effect(
    tx: &mut Transaction,
    req: EffectRequest,
) -> Result<Vec<DomainEvent>, EffectError> {
    match req {
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

        EffectRequest::ResolveWdbXizhenxihong {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_wdb_xizhenxihong(tx, player, target, card, action, skill),

        EffectRequest::ResolveWdbYangyi {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_wdb_yangyi(tx, player, target, card, action, skill),

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

        EffectRequest::ResolveGwyewuChain {
            player,
            skill,
            cost_cards,
            wuyao_def,
            lang_def,
        } => resolve_gwyewu_chain(tx, player, skill, cost_cards, wuyao_def, lang_def),

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

        EffectRequest::ResolveMbzhuji {
            player,
            choice,
            stage,
            skill,
        } => resolve_mbzhuji(tx, player, choice, stage, skill),

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

        EffectRequest::ResolveDcmoyu { player, skill } => resolve_dcmoyu(tx, player, skill),

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

        EffectRequest::ResolveTwqiushou { player, skill } => resolve_twqiushou(tx, player, skill),

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

        EffectRequest::ResolveFangquanSkip { player, skill } => {
            resolve_fangquan_skip(tx, player, skill)
        }

        EffectRequest::ResolveFangquanGrant {
            player,
            stage,
            skill,
        } => resolve_fangquan_grant(tx, player, stage, skill),

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

        EffectRequest::ResolveMingren {
            player,
            stage,
            skill,
        } => resolve_mingren(tx, player, stage, skill),

        EffectRequest::ResolveKuizhu {
            player,
            stage,
            skill,
        } => resolve_kuizhu(tx, player, stage, skill),

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

        EffectRequest::ResolveMuzhen { .. } => Err(EffectError::ContentFault(
            "resolve_muzhen pending implementation".into(),
        )),

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

        EffectRequest::ResolveDuanbi {
            player,
            targets,
            stage,
            skill,
        } => resolve_duanbi(tx, player, &targets, stage, skill),

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
