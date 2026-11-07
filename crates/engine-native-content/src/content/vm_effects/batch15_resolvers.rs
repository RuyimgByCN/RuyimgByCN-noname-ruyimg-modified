//! PA12 批⑮：25 种具名复合结算器（tags 154..=178）的交互选牌/选目标/选分支状态机。
//!
//! 当入队请求已携带确定参数（或当前局面仅存唯一/零可选项）时返回 `None`，
//! 交由内核 `rvm_ops::commit_batch15` 直接权威落账；当存在需玩家决策的多选一
//! 窗口时挂起 `Flow::WaitChoice`，并在应答到达后将所选参数注入请求落账。

use super::batch13_targets::{other_alive_options, seats_from};
use super::*;

fn hand_options(core: &GameStateCore, owner: u8) -> Vec<ChoiceOption> {
    core.player(owner)
        .hand
        .iter()
        .copied()
        .map(ChoiceOption::Card)
        .collect()
}

fn first_other_rule_pid(core: &GameStateCore, owner: u8) -> rule_api::PlayerId {
    seats_from(core, owner)
        .into_iter()
        .find(|&p| p != owner && core.player(p).alive)
        .map(|p| rule_api::PlayerId::new(p as u32 + 1))
        .unwrap_or(rule_api::PlayerId::NONE)
}

fn classify_interactive(
    effect: &EffectRequest,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(u8, ChoiceKind, Vec<ChoiceOption>, u8, u8, bool)> {
    match effect {
        EffectRequest::ResolvePotweiyan {
            player,
            card,
            action: 2 | 12,
            ..
        } if !card.is_some() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        EffectRequest::ResolvePotlvyi {
            player,
            target,
            action: 0,
            ..
        } if !target.is_some() => {
            let owner = seat(*player, core)?;
            let opts: Vec<ChoiceOption> = seats_from(core, owner)
                .into_iter()
                .filter(|&p| core.player(p).alive && !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolvePotyuji {
            player,
            action: 0,
            cards,
            ..
        } if cards.is_empty() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            let has_other = !other_alive_options(core, owner).is_empty();
            (!opts.is_empty() && has_other).then_some((
                owner,
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        EffectRequest::ResolvePotzhouyu {
            player,
            action: 1,
            targets,
            ..
        } if targets.is_empty() => {
            let owner = seat(*player, core)?;
            let opts: Vec<ChoiceOption> = seats_from(core, owner)
                .into_iter()
                .filter(|&p| p != owner && core.player(p).alive && !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolveMbzhonghui {
            player,
            action: 0,
            mbsizi_def,
            ..
        } => {
            let owner = seat(*player, core)?;
            let charge = core
                .vm_aux
                .marks
                .get(&(owner, mbsizi_def.raw()))
                .copied()
                .unwrap_or(0)
                .clamp(0, 4) as u8;
            if charge <= 1 {
                return None;
            }
            let opts = (1..=charge).map(ChoiceOption::Tag).collect();
            Some((owner, ChoiceKind::PickTag, opts, 1, 1, false))
        }
        EffectRequest::ResolveDddLiuhong {
            player,
            actor,
            action: 3,
            ..
        } if !actor.is_some() => {
            let owner = seat(*player, core)?;
            let opts: Vec<ChoiceOption> = seats_from(core, owner)
                .into_iter()
                .filter(|&p| core.player(p).alive && !core.player(p).linked)
                .map(ChoiceOption::Player)
                .collect();
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolveDddXuelingyun {
            player,
            cards,
            action: 0,
            ..
        } if cards.is_empty() => {
            let owner = seat(*player, core)?;
            let opts: Vec<ChoiceOption> = core
                .discard
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            let max = (opts.len() as u8).min(2);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, max, true))
        }
        EffectRequest::ResolvePottaishici {
            player,
            target,
            action: 2,
            ..
        } if !target.is_some() => {
            let owner = seat(*player, core)?;
            let opts = other_alive_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolveDddXujing {
            player,
            source,
            action: 10,
            ..
        } if !source.is_some() => {
            let owner = seat(*player, core)?;
            if core.player(owner).hand.is_empty() {
                return None;
            }
            let opts = other_alive_options(core, owner);
            (!opts.is_empty()).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolveDddLiuba {
            player,
            cards,
            action: 0,
            ..
        } if cards.is_empty() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            let max = (opts.len() as u8).min(2);
            (!opts.is_empty()).then_some((owner, ChoiceKind::SelectCards, opts, 1, max, true))
        }
        EffectRequest::ResolveDddLiuye {
            player,
            actor,
            action: 0,
            ..
        } if !actor.is_some() => {
            let owner = seat(*player, core)?;
            if core.player(owner).hand.is_empty() {
                return None;
            }
            let opts = other_alive_options(core, owner);
            (!opts.is_empty()).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolveDddZhangkai {
            player,
            actor,
            action: 4,
            ..
        } if !actor.is_some() => {
            let owner = seat(*player, core)?;
            let opts = other_alive_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolveHiddenSkillPile {
            player,
            pile_skill_def,
            action: 1 | 3,
            pile_cards,
            ..
        } if pile_cards.is_empty() => {
            let owner = seat(*player, core)?;
            let key_name = crate::rvm_ops::resolve_skill_def(catalog, pile_skill_def.raw())
                .map(|d| catalog.skills[d as usize].name.to_string())
                .unwrap_or_else(|| format!("pile:{}", pile_skill_def.raw()));
            let mut opts: Vec<ChoiceOption> = core
                .vm_aux
                .piles
                .get(&(owner, key_name))
                .into_iter()
                .flatten()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            if opts.is_empty() {
                for ((p, _), cards) in &core.vm_aux.piles {
                    if *p == owner && !cards.is_empty() {
                        opts = cards.iter().copied().map(ChoiceOption::Card).collect();
                        break;
                    }
                }
            }
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        EffectRequest::ResolveTwfuman {
            player,
            cards,
            action: 0,
            ..
        } if cards.is_empty() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        EffectRequest::ResolveTwkujian {
            player,
            cards,
            action: 0,
            ..
        } if cards.is_empty() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            let max = (opts.len() as u8).min(3);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, max, true))
        }
        EffectRequest::ResolveDcSbLusu {
            player,
            cards,
            action: 1 | 2,
            ..
        } if cards.is_empty() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        EffectRequest::ResolveMbxiongtu {
            player,
            cards,
            stage: 0,
            ..
        } if cards.is_empty() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            let max = (opts.len() as u8).min(2);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, max, true))
        }
        EffectRequest::ResolveSpyingjia {
            player,
            card,
            action: 2,
            ..
        } if !card.is_some() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        EffectRequest::ResolveDddCaoshuang {
            player,
            card,
            action: 0,
            ..
        } if !card.is_some() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        _ => None,
    }
}

pub(super) fn active_ask(
    effect: &EffectRequest,
    state: &[u32],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if state != [1] {
        return None;
    }
    let (observer, kind, options, min, max, allow_cancel) =
        classify_interactive(effect, core, catalog)?;
    Some(ask(
        observer,
        "rvm_choice",
        kind,
        options,
        usize::from(min),
        usize::from(max),
        allow_cancel,
    ))
}

fn batch15_owner(effect: &EffectRequest, core: &GameStateCore) -> Option<u8> {
    match effect {
        EffectRequest::ResolvePotweiyan { player, .. }
        | EffectRequest::ResolvePotcaoshuang { player, .. }
        | EffectRequest::ResolvePotlvyi { player, .. }
        | EffectRequest::ResolvePotyuji { player, .. }
        | EffectRequest::ResolvePotzhouyu { player, .. }
        | EffectRequest::ResolveMbzhonghui { player, .. }
        | EffectRequest::ResolveDddLiuhong { player, .. }
        | EffectRequest::ResolveTwguohuai { player, .. }
        | EffectRequest::ResolveDddXuelingyun { player, .. }
        | EffectRequest::ResolvePottaishici { player, .. }
        | EffectRequest::ResolveDddXujing { player, .. }
        | EffectRequest::ResolveDddLiuba { player, .. }
        | EffectRequest::ResolveDddLiuye { player, .. }
        | EffectRequest::ResolveDddZhangkai { player, .. }
        | EffectRequest::ResolveHiddenSkillPile { player, .. }
        | EffectRequest::ResolveTwfuman { player, .. }
        | EffectRequest::ResolveTwkujian { player, .. }
        | EffectRequest::ResolveTwxiawei { player, .. }
        | EffectRequest::ResolveCaoyi { player, .. }
        | EffectRequest::ResolveDcSbLusu { player, .. }
        | EffectRequest::ResolveDcSbZhouyu { player, .. }
        | EffectRequest::ResolveMbxiongtu { player, .. }
        | EffectRequest::ResolvePothongyi { player, .. }
        | EffectRequest::ResolveSpyingjia { player, .. }
        | EffectRequest::ResolveDddCaoshuang { player, .. } => seat(*player, core),
        _ => None,
    }
}

pub(super) fn step(
    ctx: &mut Ctx<'_>,
    skill: &str,
    effect: &EffectRequest,
    state: &mut Vec<u32>,
) -> Option<Flow> {
    let owner = batch15_owner(effect, ctx.core)?;
    if state.is_empty() {
        let (observer, kind, options, min, max, allow_cancel) =
            classify_interactive(effect, ctx.core, ctx.catalog)?;
        state.push(1);
        return Some(Flow::WaitChoice {
            observer,
            spec: ChoiceSpec {
                kind,
                prompt: "rvm_choice",
                options,
                min: usize::from(min),
                max: usize::from(max),
                allow_cancel,
            },
        });
    }
    let ans = ctx.answer();
    if ans.is_empty() {
        if let EffectRequest::ResolvePotweiyan {
            player,
            target,
            card,
            action: 2 | 12,
            skill: sk,
        } = effect
        {
            crate::rvm_ops::commit_effect(
                ctx,
                skill,
                owner,
                EffectRequest::ResolvePotweiyan {
                    player: *player,
                    target: *target,
                    card: *card,
                    action: 14,
                    skill: *sk,
                },
            );
        }
        return Some(Flow::Done);
    }
    let picked_cards: Vec<rule_api::CardId> = ans
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(rule_api::CardId::new(*c)),
            _ => None,
        })
        .collect();
    let picked_players: Vec<rule_api::PlayerId> = ans
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Player(p) => Some(rule_api::PlayerId::new(*p as u32 + 1)),
            _ => None,
        })
        .collect();
    let picked_tag: Option<u8> = ans.iter().find_map(|o| match o {
        ChoiceOption::Tag(t) => Some(*t),
        _ => None,
    });
    let mut resolved = effect.clone();
    match &mut resolved {
        EffectRequest::ResolvePotweiyan { card, .. }
        | EffectRequest::ResolveSpyingjia { card, .. }
        | EffectRequest::ResolveDddCaoshuang { card, .. } => {
            if let Some(&c) = picked_cards.first() {
                *card = c;
            }
        }
        EffectRequest::ResolvePotlvyi { target, .. }
        | EffectRequest::ResolvePottaishici { target, .. } => {
            if let Some(&p) = picked_players.first() {
                *target = p;
            }
        }
        EffectRequest::ResolvePotyuji {
            cards,
            targets,
            ..
        } => {
            if !picked_cards.is_empty() {
                *cards = picked_cards;
                if targets.is_empty() {
                    let dst = first_other_rule_pid(ctx.core, owner);
                    if dst.is_some() {
                        targets.push(dst);
                    }
                }
            }
        }
        EffectRequest::ResolvePotzhouyu { targets, .. } => {
            if !picked_players.is_empty() {
                *targets = picked_players;
            }
        }
        EffectRequest::ResolveMbzhonghui { action, amount, .. } => {
            if let Some(t) = picked_tag {
                *action = 8;
                *amount = u32::from(t.max(1));
            }
        }
        EffectRequest::ResolveDddLiuhong { actor, .. }
        | EffectRequest::ResolveDddLiuye { actor, .. }
        | EffectRequest::ResolveDddZhangkai { actor, .. } => {
            if let Some(&p) = picked_players.first() {
                *actor = p;
            }
        }
        EffectRequest::ResolveDddXuelingyun { cards, .. }
        | EffectRequest::ResolveDddLiuba { cards, .. }
        | EffectRequest::ResolveDcSbLusu { cards, .. } => {
            if !picked_cards.is_empty() {
                *cards = picked_cards;
            }
        }
        EffectRequest::ResolveDddXujing { source, .. } => {
            if let Some(&p) = picked_players.first() {
                *source = p;
            }
        }
        EffectRequest::ResolveHiddenSkillPile { pile_cards, .. } => {
            if !picked_cards.is_empty() {
                *pile_cards = picked_cards;
            }
        }
        EffectRequest::ResolveTwfuman {
            player,
            target,
            cards,
            ..
        }
        | EffectRequest::ResolveTwkujian {
            player,
            target,
            cards,
            ..
        }
        | EffectRequest::ResolveMbxiongtu {
            player,
            target,
            cards,
            ..
        } => {
            if !picked_cards.is_empty() {
                *cards = picked_cards;
                if !target.is_some() || *target == *player {
                    *target = first_other_rule_pid(ctx.core, owner);
                }
            }
        }
        _ => {}
    }
    crate::rvm_ops::commit_effect(ctx, skill, owner, resolved);
    Some(Flow::Done)
}

pub(super) fn validate(effect: &EffectRequest, state: &[u32], core: &GameStateCore) -> bool {
    state == [1] && batch15_owner(effect, core).is_some()
}
