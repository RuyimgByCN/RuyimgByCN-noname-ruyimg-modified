//! PA12 批⑮：25 种具名复合结算器（tags 154..=178）的入队参数校验。

use super::*;

pub(super) fn accept_batch15_effect(
    req: &EffectRequest,
    seats: u8,
) -> Option<Result<(), HostTrap>> {
    let ok_player = |p: rule_api::PlayerId, what: &str| -> Result<PlayerId, HostTrap> {
        seat_of(p, seats).ok_or_else(|| reject(what))
    };
    let opt_player = |p: rule_api::PlayerId, what: &str| -> Result<(), HostTrap> {
        if p.is_some() {
            ok_player(p, what)?;
        }
        Ok(())
    };
    let ok_players = |ps: &[rule_api::PlayerId], what: &str| -> Result<(), HostTrap> {
        if ps.len() > u8::MAX as usize {
            return Err(reject(what));
        }
        for &p in ps {
            ok_player(p, what)?;
        }
        Ok(())
    };
    let ok_cards = |cs: &[rule_api::CardId], what: &str| -> Result<(), HostTrap> {
        if cs.len() > u8::MAX as usize || cs.iter().any(|c| !c.is_some()) {
            return Err(reject(what));
        }
        Ok(())
    };
    let res = (|| -> Result<(), HostTrap> {
        match req {
            EffectRequest::ResolvePotweiyan {
                player,
                target,
                action,
                ..
            } => {
                ok_player(*player, "ResolvePotweiyan.player")?;
                opt_player(*target, "ResolvePotweiyan.target")?;
                if *action > 15 {
                    return Err(reject("ResolvePotweiyan.action"));
                }
            }
            EffectRequest::ResolvePotcaoshuang {
                player,
                target,
                action,
                ..
            } => {
                ok_player(*player, "ResolvePotcaoshuang.player")?;
                opt_player(*target, "ResolvePotcaoshuang.target")?;
                if *action > 12 {
                    return Err(reject("ResolvePotcaoshuang.action"));
                }
            }
            EffectRequest::ResolvePotlvyi {
                player,
                target,
                action,
                event_cards,
                targets,
                ..
            } => {
                ok_player(*player, "ResolvePotlvyi.player")?;
                opt_player(*target, "ResolvePotlvyi.target")?;
                ok_cards(event_cards, "ResolvePotlvyi.event_cards")?;
                ok_players(targets, "ResolvePotlvyi.targets")?;
                if *action > 11 {
                    return Err(reject("ResolvePotlvyi.action"));
                }
            }
            EffectRequest::ResolvePotyuji {
                player,
                target,
                action,
                cards,
                targets,
                ..
            } => {
                ok_player(*player, "ResolvePotyuji.player")?;
                opt_player(*target, "ResolvePotyuji.target")?;
                ok_cards(cards, "ResolvePotyuji.cards")?;
                ok_players(targets, "ResolvePotyuji.targets")?;
                if *action > 8 {
                    return Err(reject("ResolvePotyuji.action"));
                }
            }
            EffectRequest::ResolvePotzhouyu {
                player,
                target,
                action,
                targets,
                ..
            } => {
                ok_player(*player, "ResolvePotzhouyu.player")?;
                opt_player(*target, "ResolvePotzhouyu.target")?;
                ok_players(targets, "ResolvePotzhouyu.targets")?;
                if *action > 10 {
                    return Err(reject("ResolvePotzhouyu.action"));
                }
            }
            EffectRequest::ResolveMbzhonghui {
                player,
                target,
                action,
                ..
            } => {
                ok_player(*player, "ResolveMbzhonghui.player")?;
                opt_player(*target, "ResolveMbzhonghui.target")?;
                if *action > 8 {
                    return Err(reject("ResolveMbzhonghui.action"));
                }
            }
            EffectRequest::ResolveDddLiuhong {
                player,
                actor,
                action,
                ..
            } => {
                ok_player(*player, "ResolveDddLiuhong.player")?;
                opt_player(*actor, "ResolveDddLiuhong.actor")?;
                if *action > 5 {
                    return Err(reject("ResolveDddLiuhong.action"));
                }
            }
            EffectRequest::ResolveTwguohuai {
                player,
                target,
                source,
                action,
                stage,
                ..
            } => {
                ok_player(*player, "ResolveTwguohuai.player")?;
                opt_player(*target, "ResolveTwguohuai.target")?;
                opt_player(*source, "ResolveTwguohuai.source")?;
                if *action > 4 || *stage > 2 {
                    return Err(reject("ResolveTwguohuai.action"));
                }
            }
            EffectRequest::ResolveDddXuelingyun {
                player,
                actor,
                cards,
                action,
                ..
            } => {
                ok_player(*player, "ResolveDddXuelingyun.player")?;
                opt_player(*actor, "ResolveDddXuelingyun.actor")?;
                ok_cards(cards, "ResolveDddXuelingyun.cards")?;
                if *action > 5 {
                    return Err(reject("ResolveDddXuelingyun.action"));
                }
            }
            EffectRequest::ResolvePottaishici {
                player,
                target,
                action,
                cards,
                ..
            } => {
                ok_player(*player, "ResolvePottaishici.player")?;
                opt_player(*target, "ResolvePottaishici.target")?;
                ok_cards(cards, "ResolvePottaishici.cards")?;
                if *action > 7 {
                    return Err(reject("ResolvePottaishici.action"));
                }
            }
            EffectRequest::ResolveDddXujing {
                player,
                source,
                action,
                targets,
                ..
            } => {
                ok_player(*player, "ResolveDddXujing.player")?;
                opt_player(*source, "ResolveDddXujing.source")?;
                ok_players(targets, "ResolveDddXujing.targets")?;
                if !matches!(*action, 0..=3 | 10..=13) {
                    return Err(reject("ResolveDddXujing.action"));
                }
            }
            EffectRequest::ResolveDddLiuba {
                player,
                actor,
                action,
                cards,
                targets,
                ..
            } => {
                ok_player(*player, "ResolveDddLiuba.player")?;
                opt_player(*actor, "ResolveDddLiuba.actor")?;
                ok_cards(cards, "ResolveDddLiuba.cards")?;
                ok_players(targets, "ResolveDddLiuba.targets")?;
                if !matches!(*action, 0..=4 | 10..=15) {
                    return Err(reject("ResolveDddLiuba.action"));
                }
            }
            EffectRequest::ResolveDddLiuye {
                player,
                actor,
                action,
                ..
            } => {
                ok_player(*player, "ResolveDddLiuye.player")?;
                opt_player(*actor, "ResolveDddLiuye.actor")?;
                if *action > 4 {
                    return Err(reject("ResolveDddLiuye.action"));
                }
            }
            EffectRequest::ResolveDddZhangkai {
                player,
                actor,
                cards,
                action,
                ..
            } => {
                ok_player(*player, "ResolveDddZhangkai.player")?;
                opt_player(*actor, "ResolveDddZhangkai.actor")?;
                ok_cards(cards, "ResolveDddZhangkai.cards")?;
                if *action > 5 {
                    return Err(reject("ResolveDddZhangkai.action"));
                }
            }
            EffectRequest::ResolveHiddenSkillPile {
                player,
                pile_skill_def,
                action,
                stage,
                hand_cards,
                pile_cards,
                event_cards,
                ..
            } => {
                ok_player(*player, "ResolveHiddenSkillPile.player")?;
                if !pile_skill_def.is_some() || *action > 4 || *stage > 1 {
                    return Err(reject("ResolveHiddenSkillPile.args"));
                }
                ok_cards(hand_cards, "ResolveHiddenSkillPile.hand_cards")?;
                ok_cards(pile_cards, "ResolveHiddenSkillPile.pile_cards")?;
                ok_cards(event_cards, "ResolveHiddenSkillPile.event_cards")?;
            }
            EffectRequest::ResolveTwfuman {
                player,
                target,
                cards,
                action,
                stage,
                ..
            } => {
                ok_player(*player, "ResolveTwfuman.player")?;
                opt_player(*target, "ResolveTwfuman.target")?;
                ok_cards(cards, "ResolveTwfuman.cards")?;
                if *action > 2 || *stage > 1 {
                    return Err(reject("ResolveTwfuman.action"));
                }
            }
            EffectRequest::ResolveTwkujian {
                player,
                target,
                cards,
                action,
                ..
            } => {
                ok_player(*player, "ResolveTwkujian.player")?;
                opt_player(*target, "ResolveTwkujian.target")?;
                ok_cards(cards, "ResolveTwkujian.cards")?;
                if *action > 2 {
                    return Err(reject("ResolveTwkujian.action"));
                }
            }
            EffectRequest::ResolveTwxiawei {
                player,
                action,
                stage,
                ..
            } => {
                ok_player(*player, "ResolveTwxiawei.player")?;
                if *action > 4 || *stage > 4 {
                    return Err(reject("ResolveTwxiawei.action"));
                }
            }
            EffectRequest::ResolveCaoyi {
                player,
                action,
                event_targets,
                ..
            } => {
                ok_player(*player, "ResolveCaoyi.player")?;
                ok_players(event_targets, "ResolveCaoyi.event_targets")?;
                if !matches!(*action, 0 | 1 | 10 | 11 | 20 | 21) {
                    return Err(reject("ResolveCaoyi.action"));
                }
            }
            EffectRequest::ResolveDcSbLusu {
                player,
                source,
                action,
                cards,
                ..
            } => {
                ok_player(*player, "ResolveDcSbLusu.player")?;
                opt_player(*source, "ResolveDcSbLusu.source")?;
                ok_cards(cards, "ResolveDcSbLusu.cards")?;
                if !matches!(*action, 0..=2 | 10 | 11 | 20..=23) {
                    return Err(reject("ResolveDcSbLusu.action"));
                }
            }
            EffectRequest::ResolveDcSbZhouyu {
                player,
                source,
                action,
                event_targets,
                damaged_targets,
                ..
            } => {
                ok_player(*player, "ResolveDcSbZhouyu.player")?;
                opt_player(*source, "ResolveDcSbZhouyu.source")?;
                ok_players(event_targets, "ResolveDcSbZhouyu.event_targets")?;
                ok_players(damaged_targets, "ResolveDcSbZhouyu.damaged_targets")?;
                if !matches!(*action, 0 | 1 | 10..=15 | 20..=23) {
                    return Err(reject("ResolveDcSbZhouyu.action"));
                }
            }
            EffectRequest::ResolveMbxiongtu {
                player,
                target,
                stage,
                cards,
                ..
            } => {
                ok_player(*player, "ResolveMbxiongtu.player")?;
                opt_player(*target, "ResolveMbxiongtu.target")?;
                ok_cards(cards, "ResolveMbxiongtu.cards")?;
                if *stage > 7 {
                    return Err(reject("ResolveMbxiongtu.stage"));
                }
            }
            EffectRequest::ResolvePothongyi { player, stage, .. } => {
                ok_player(*player, "ResolvePothongyi.player")?;
                if *stage > 3 {
                    return Err(reject("ResolvePothongyi.stage"));
                }
            }
            EffectRequest::ResolveSpyingjia {
                player,
                action,
                target,
                ..
            } => {
                ok_player(*player, "ResolveSpyingjia.player")?;
                opt_player(*target, "ResolveSpyingjia.target")?;
                if *action > 4 {
                    return Err(reject("ResolveSpyingjia.action"));
                }
            }
            EffectRequest::ResolveDddCaoshuang {
                player,
                actor,
                action,
                ..
            } => {
                ok_player(*player, "ResolveDddCaoshuang.player")?;
                opt_player(*actor, "ResolveDddCaoshuang.actor")?;
                if *action > 3 {
                    return Err(reject("ResolveDddCaoshuang.action"));
                }
            }
            EffectRequest::ResolveStddechong {
                player,
                target,
                cards,
                ..
            } => {
                ok_player(*player, "ResolveStddechong.player")?;
                opt_player(*target, "ResolveStddechong.target")?;
                ok_cards(cards, "ResolveStddechong.cards")?;
            }
            EffectRequest::ResolveDddYujin {
                player,
                actor,
                target,
                action,
                candidates,
                ..
            } => {
                ok_player(*player, "ResolveDddYujin.player")?;
                opt_player(*actor, "ResolveDddYujin.actor")?;
                opt_player(*target, "ResolveDddYujin.target")?;
                ok_players(candidates, "ResolveDddYujin.candidates")?;
                if *action > 10 {
                    return Err(reject("ResolveDddYujin.action"));
                }
            }
            EffectRequest::ResolveDddZhenji {
                player,
                target,
                action,
                stage,
                piles,
                cards,
                ..
            } => {
                ok_player(*player, "ResolveDddZhenji.player")?;
                opt_player(*target, "ResolveDddZhenji.target")?;
                ok_cards(cards, "ResolveDddZhenji.cards")?;
                if *action > 10 || *stage > 10 || piles.len() > u8::MAX as usize {
                    return Err(reject("ResolveDddZhenji.args"));
                }
            }
            EffectRequest::ResolveZhengjian {
                player,
                stage,
                event_player,
                ..
            } => {
                ok_player(*player, "ResolveZhengjian.player")?;
                opt_player(*event_player, "ResolveZhengjian.event_player")?;
                if *stage > 3 {
                    return Err(reject("ResolveZhengjian.stage"));
                }
            }
            EffectRequest::ResolveQiaobian {
                player,
                phase,
                stage,
                ..
            } => {
                ok_player(*player, "ResolveQiaobian.player")?;
                if *phase > 3 || *stage > 5 {
                    return Err(reject("ResolveQiaobian.args"));
                }
            }
            EffectRequest::ResolveTwfeiyi {
                player,
                action,
                stage,
                event_cards,
                ..
            } => {
                ok_player(*player, "ResolveTwfeiyi.player")?;
                ok_cards(event_cards, "ResolveTwfeiyi.event_cards")?;
                if *action > 2 || *stage > 3 {
                    return Err(reject("ResolveTwfeiyi.args"));
                }
            }
            EffectRequest::ResolveZhengsu {
                player,
                mode,
                stage,
                ..
            } => {
                ok_player(*player, "ResolveZhengsu.player")?;
                if *mode > 2 || *stage > 2 {
                    return Err(reject("ResolveZhengsu.args"));
                }
            }
            EffectRequest::ResolveTwcaoang {
                player,
                target,
                source,
                action,
                stage,
                ..
            } => {
                ok_player(*player, "ResolveTwcaoang.player")?;
                opt_player(*target, "ResolveTwcaoang.target")?;
                opt_player(*source, "ResolveTwcaoang.source")?;
                if *action > 2 || *stage > 3 {
                    return Err(reject("ResolveTwcaoang.args"));
                }
            }
            EffectRequest::ResolveTwcaozhao {
                player,
                target,
                source,
                action,
                stage,
                ..
            } => {
                ok_player(*player, "ResolveTwcaozhao.player")?;
                opt_player(*target, "ResolveTwcaozhao.target")?;
                opt_player(*source, "ResolveTwcaozhao.source")?;
                if *action > 2 || *stage > 3 {
                    return Err(reject("ResolveTwcaozhao.args"));
                }
            }
            EffectRequest::ResolveTwchengpu {
                player,
                target,
                source,
                action,
                stage,
                ..
            } => {
                ok_player(*player, "ResolveTwchengpu.player")?;
                opt_player(*target, "ResolveTwchengpu.target")?;
                opt_player(*source, "ResolveTwchengpu.source")?;
                if *action > 3 || *stage > 3 {
                    return Err(reject("ResolveTwchengpu.args"));
                }
            }
            EffectRequest::ResolveTwchenxun {
                player,
                target,
                action,
                ..
            } => {
                ok_player(*player, "ResolveTwchenxun.player")?;
                opt_player(*target, "ResolveTwchenxun.target")?;
                if *action > 3 {
                    return Err(reject("ResolveTwchenxun.action"));
                }
            }
            EffectRequest::ResolveTwchenzhen {
                player,
                target,
                action,
                stage,
                ..
            } => {
                ok_player(*player, "ResolveTwchenzhen.player")?;
                opt_player(*target, "ResolveTwchenzhen.target")?;
                if *action > 3 || *stage > 3 {
                    return Err(reject("ResolveTwchenzhen.args"));
                }
            }
            EffectRequest::ResolveTwyizhu {
                player,
                source,
                action,
                stage,
                event_cards,
                ..
            } => {
                ok_player(*player, "ResolveTwyizhu.player")?;
                opt_player(*source, "ResolveTwyizhu.source")?;
                ok_cards(event_cards, "ResolveTwyizhu.event_cards")?;
                if *action > 2 || *stage > 3 {
                    return Err(reject("ResolveTwyizhu.args"));
                }
            }
            EffectRequest::ResolveRuanji {
                player,
                source,
                action,
                cards,
                ..
            } => {
                ok_player(*player, "ResolveRuanji.player")?;
                opt_player(*source, "ResolveRuanji.source")?;
                ok_cards(cards, "ResolveRuanji.cards")?;
                if !matches!(*action, 0..=4 | 10..=15) {
                    return Err(reject("ResolveRuanji.action"));
                }
            }
            EffectRequest::ResolveQiaojian {
                player,
                target,
                action,
                cards,
                ..
            } => {
                ok_player(*player, "ResolveQiaojian.player")?;
                opt_player(*target, "ResolveQiaojian.target")?;
                ok_cards(cards, "ResolveQiaojian.cards")?;
                if *action > 12 {
                    return Err(reject("ResolveQiaojian.action"));
                }
            }
            EffectRequest::ResolveZhupeilan {
                player,
                target,
                action,
                event_cards,
                ..
            } => {
                ok_player(*player, "ResolveZhupeilan.player")?;
                opt_player(*target, "ResolveZhupeilan.target")?;
                ok_cards(event_cards, "ResolveZhupeilan.event_cards")?;
                if !matches!(*action, 0..=3 | 10..=12) {
                    return Err(reject("ResolveZhupeilan.action"));
                }
            }
            EffectRequest::ResolveMbduanyang {
                player,
                target,
                mode,
                stage,
                cards,
                ..
            } => {
                ok_player(*player, "ResolveMbduanyang.player")?;
                opt_player(*target, "ResolveMbduanyang.target")?;
                ok_cards(cards, "ResolveMbduanyang.cards")?;
                if *mode > 2 || *stage > 3 {
                    return Err(reject("ResolveMbduanyang.args"));
                }
            }
            EffectRequest::ResolveMbrunwei {
                player,
                target,
                stage,
                cards,
                ..
            } => {
                ok_player(*player, "ResolveMbrunwei.player")?;
                opt_player(*target, "ResolveMbrunwei.target")?;
                ok_cards(cards, "ResolveMbrunwei.cards")?;
                if *stage > 5 {
                    return Err(reject("ResolveMbrunwei.stage"));
                }
            }
            EffectRequest::ResolveMbkuangxiang {
                player,
                target,
                stage,
                ..
            } => {
                ok_player(*player, "ResolveMbkuangxiang.player")?;
                opt_player(*target, "ResolveMbkuangxiang.target")?;
                if *stage > 3 {
                    return Err(reject("ResolveMbkuangxiang.stage"));
                }
            }
            EffectRequest::ResolvePotsifeng {
                player,
                target,
                stage,
                ..
            } => {
                ok_player(*player, "ResolvePotsifeng.player")?;
                opt_player(*target, "ResolvePotsifeng.target")?;
                if *stage > 5 {
                    return Err(reject("ResolvePotsifeng.stage"));
                }
            }
            EffectRequest::ResolvePotwanglie { player, stage, .. } => {
                ok_player(*player, "ResolvePotwanglie.player")?;
                if *stage > 3 {
                    return Err(reject("ResolvePotwanglie.stage"));
                }
            }
            EffectRequest::ResolvePotchengpu {
                player,
                target,
                stage,
                ..
            } => {
                ok_player(*player, "ResolvePotchengpu.player")?;
                opt_player(*target, "ResolvePotchengpu.target")?;
                if *stage > 4 {
                    return Err(reject("ResolvePotchengpu.stage"));
                }
            }
            EffectRequest::ResolvePottuntian {
                player,
                action,
                targets,
                ..
            } => {
                ok_player(*player, "ResolvePottuntian.player")?;
                ok_players(targets, "ResolvePottuntian.targets")?;
                if *action > 3 {
                    return Err(reject("ResolvePottuntian.action"));
                }
            }
            EffectRequest::ResolvePotjixi {
                player,
                target,
                action,
                targets,
                ..
            } => {
                ok_player(*player, "ResolvePotjixi.player")?;
                opt_player(*target, "ResolvePotjixi.target")?;
                ok_players(targets, "ResolvePotjixi.targets")?;
                if *action > 4 {
                    return Err(reject("ResolvePotjixi.action"));
                }
            }
            EffectRequest::ResolvePotjiyu { player, stage, .. } => {
                ok_player(*player, "ResolvePotjiyu.player")?;
                if *stage > 2 {
                    return Err(reject("ResolvePotjiyu.stage"));
                }
            }
            EffectRequest::ResolvePotnigu {
                player,
                target,
                targets,
                stage,
                ..
            } => {
                ok_player(*player, "ResolvePotnigu.player")?;
                opt_player(*target, "ResolvePotnigu.target")?;
                ok_players(targets, "ResolvePotnigu.targets")?;
                if *stage > 4 {
                    return Err(reject("ResolvePotnigu.stage"));
                }
            }
            EffectRequest::ResolvePotfuan { player, stage, .. } => {
                ok_player(*player, "ResolvePotfuan.player")?;
                if *stage > 3 {
                    return Err(reject("ResolvePotfuan.stage"));
                }
            }
            EffectRequest::ResolveDddGuanning {
                player,
                actor,
                action,
                ..
            } => {
                ok_player(*player, "ResolveDddGuanning.player")?;
                opt_player(*actor, "ResolveDddGuanning.actor")?;
                if *action > 3 {
                    return Err(reject("ResolveDddGuanning.action"));
                }
            }
            EffectRequest::ResolveDddJianshuo {
                player,
                target,
                action,
                cards,
                targets,
                ..
            } => {
                ok_player(*player, "ResolveDddJianshuo.player")?;
                opt_player(*target, "ResolveDddJianshuo.target")?;
                ok_cards(cards, "ResolveDddJianshuo.cards")?;
                ok_players(targets, "ResolveDddJianshuo.targets")?;
                if *action > 4 {
                    return Err(reject("ResolveDddJianshuo.action"));
                }
            }
            EffectRequest::ResolveShenSimayi {
                player,
                event_player,
                mark,
                mode,
                cards,
                targets,
                ..
            } => {
                ok_player(*player, "ResolveShenSimayi.player")?;
                opt_player(*event_player, "ResolveShenSimayi.event_player")?;
                ok_cards(cards, "ResolveShenSimayi.cards")?;
                ok_players(targets, "ResolveShenSimayi.targets")?;
                if !mark.is_some() || *mode > 4 {
                    return Err(reject("ResolveShenSimayi.args"));
                }
            }
            _ => return Ok(()),
        }
        Ok(())
    })();
    match req {
        EffectRequest::ResolvePotweiyan { .. }
        | EffectRequest::ResolvePotcaoshuang { .. }
        | EffectRequest::ResolvePotlvyi { .. }
        | EffectRequest::ResolvePotyuji { .. }
        | EffectRequest::ResolvePotzhouyu { .. }
        | EffectRequest::ResolveMbzhonghui { .. }
        | EffectRequest::ResolveDddLiuhong { .. }
        | EffectRequest::ResolveTwguohuai { .. }
        | EffectRequest::ResolveDddXuelingyun { .. }
        | EffectRequest::ResolvePottaishici { .. }
        | EffectRequest::ResolveDddXujing { .. }
        | EffectRequest::ResolveDddLiuba { .. }
        | EffectRequest::ResolveDddLiuye { .. }
        | EffectRequest::ResolveDddZhangkai { .. }
        | EffectRequest::ResolveHiddenSkillPile { .. }
        | EffectRequest::ResolveTwfuman { .. }
        | EffectRequest::ResolveTwkujian { .. }
        | EffectRequest::ResolveTwxiawei { .. }
        | EffectRequest::ResolveCaoyi { .. }
        | EffectRequest::ResolveDcSbLusu { .. }
        | EffectRequest::ResolveDcSbZhouyu { .. }
        | EffectRequest::ResolveMbxiongtu { .. }
        | EffectRequest::ResolvePothongyi { .. }
        | EffectRequest::ResolveSpyingjia { .. }
        | EffectRequest::ResolveDddCaoshuang { .. }
        | EffectRequest::ResolveStddechong { .. }
        | EffectRequest::ResolveDddYujin { .. }
        | EffectRequest::ResolveDddZhenji { .. }
        | EffectRequest::ResolveZhengjian { .. }
        | EffectRequest::ResolveQiaobian { .. }
        | EffectRequest::ResolveTwfeiyi { .. }
        | EffectRequest::ResolveZhengsu { .. }
        | EffectRequest::ResolveTwcaoang { .. }
        | EffectRequest::ResolveTwcaozhao { .. }
        | EffectRequest::ResolveTwchengpu { .. }
        | EffectRequest::ResolveTwchenxun { .. }
        | EffectRequest::ResolveTwchenzhen { .. }
        | EffectRequest::ResolveTwyizhu { .. }
        | EffectRequest::ResolveRuanji { .. }
        | EffectRequest::ResolveQiaojian { .. }
        | EffectRequest::ResolveZhupeilan { .. }
        | EffectRequest::ResolveMbduanyang { .. }
        | EffectRequest::ResolveMbrunwei { .. }
        | EffectRequest::ResolveMbkuangxiang { .. }
        | EffectRequest::ResolvePotsifeng { .. }
        | EffectRequest::ResolvePotwanglie { .. }
        | EffectRequest::ResolvePotchengpu { .. }
        | EffectRequest::ResolvePottuntian { .. }
        | EffectRequest::ResolvePotjixi { .. }
        | EffectRequest::ResolvePotjiyu { .. }
        | EffectRequest::ResolvePotnigu { .. }
        | EffectRequest::ResolvePotfuan { .. }
        | EffectRequest::ResolveDddGuanning { .. }
        | EffectRequest::ResolveDddJianshuo { .. }
        | EffectRequest::ResolveShenSimayi { .. } => Some(res),
        _ => batch17::accept_batch17_effect(req, seats),
    }
}

mod batch17;
pub(crate) use batch17::backfill_batch17_skill;
