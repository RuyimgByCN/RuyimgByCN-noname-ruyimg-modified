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
        | EffectRequest::ResolveDddCaoshuang { .. } => Some(res),
        _ => None,
    }
}
