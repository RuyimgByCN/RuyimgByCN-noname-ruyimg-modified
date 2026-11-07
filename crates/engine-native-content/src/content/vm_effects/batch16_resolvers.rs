//! PA12 批⑯：30 种具名复合结算器（tags 179..=208）的交互选牌/选目标/选分支状态机。
//!
//! 当入队请求已携带确定参数（或当前局面仅存唯一/零可选项）时返回 `None`，
//! 交由内核 `rvm_ops::commit_batch16` 直接权威落账；当存在需玩家决策的多选一
//! 窗口时挂起 `Flow::WaitChoice`，并在应答到达后将所选参数注入请求落账。

use super::batch13_targets::other_alive_options;
use super::*;

fn hand_options(core: &GameStateCore, owner: u8) -> Vec<ChoiceOption> {
    core.player(owner)
        .hand
        .iter()
        .copied()
        .map(ChoiceOption::Card)
        .collect()
}

fn hand_equip_options(core: &GameStateCore, owner: u8) -> Vec<ChoiceOption> {
    core.player(owner)
        .hand
        .iter()
        .chain(core.player(owner).equips.iter())
        .copied()
        .map(ChoiceOption::Card)
        .collect()
}

fn classify_interactive(
    effect: &EffectRequest,
    core: &GameStateCore,
    _catalog: &Catalog,
) -> Option<(u8, ChoiceKind, Vec<ChoiceOption>, u8, u8, bool)> {
    match effect {
        EffectRequest::ResolveStddechong {
            player,
            cards,
            ..
        } if cards.is_empty() => {
            let owner = seat(*player, core)?;
            let opts = hand_equip_options(core, owner);
            let max = (opts.len() as u8).min(3);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, max, true))
        }
        EffectRequest::ResolveDddYujin {
            player,
            target,
            action: 0,
            ..
        } if !target.is_some() => {
            let owner = seat(*player, core)?;
            let opts = alive_options(core);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolveDddZhenji {
            player,
            action: 2,
            cards,
            ..
        } if cards.is_empty() => {
            let owner = seat(*player, core)?;
            let mut opts: Vec<ChoiceOption> = core
                .vm_aux
                .piles
                .get(&(owner, "dddmiaoxing_1".into()))
                .into_iter()
                .chain(core.vm_aux.piles.get(&(owner, "dddmiaoxing_2".into())))
                .flatten()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            if opts.is_empty() {
                opts = hand_options(core, owner);
            }
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        EffectRequest::ResolveZhengjian {
            player,
            stage: 0,
            ..
        } => {
            let owner = seat(*player, core)?;
            let opts = alive_options(core);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolveQiaobian {
            player,
            stage: 0,
            ..
        } => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        EffectRequest::ResolveTwfeiyi {
            player,
            action: 1,
            stage: 0,
            card,
            ..
        } if !card.is_some() => {
            let owner = seat(*player, core)?;
            let opts = vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)];
            Some((owner, ChoiceKind::PickTag, opts, 1, 1, false))
        }
        EffectRequest::ResolveZhengsu {
            player,
            stage: 1,
            ..
        } => {
            let owner = seat(*player, core)?;
            let p = core.player(owner);
            if p.hp >= p.max_hp as i32 {
                return None;
            }
            let opts = vec![ChoiceOption::Tag(1), ChoiceOption::Tag(2)];
            Some((owner, ChoiceKind::PickTag, opts, 1, 1, false))
        }
        EffectRequest::ResolveTwcaoang {
            player,
            card,
            action: 1,
            ..
        } if !card.is_some() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        EffectRequest::ResolveTwcaozhao {
            player,
            target,
            action: 0,
            ..
        } if !target.is_some() => {
            let owner = seat(*player, core)?;
            let opts = alive_options(core);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolveTwchengpu {
            player,
            action: 1,
            selected_card,
            ..
        } if !selected_card.is_some() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        EffectRequest::ResolveTwchenxun {
            player,
            target,
            action: 0,
            ..
        } if !target.is_some() || *target == *player => {
            let owner = seat(*player, core)?;
            let opts = other_alive_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolveTwchenzhen {
            player,
            action: 0,
            card_kind: 0,
            ..
        } => {
            let owner = seat(*player, core)?;
            let opts = vec![ChoiceOption::Tag(1), ChoiceOption::Tag(2), ChoiceOption::Tag(3)];
            Some((owner, ChoiceKind::PickTag, opts, 1, 1, false))
        }
        EffectRequest::ResolveTwyizhu {
            player,
            card,
            action: 0,
            ..
        } if !card.is_some() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, false))
        }
        EffectRequest::ResolveRuanji {
            player,
            action: 3,
            cards,
            ..
        } if cards.is_empty() => {
            let owner = seat(*player, core)?;
            let drunk = core
                .vm_aux
                .flags
                .get(&(owner, "dcjiudun_drunk".into()))
                .copied()
                .unwrap_or(0);
            if drunk == 0 {
                return None;
            }
            let opts = hand_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        EffectRequest::ResolveQiaojian {
            player,
            action: 0,
            cards,
            ..
        } if cards.is_empty() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            let max = (opts.len() as u8).min(2);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, max, true))
        }
        EffectRequest::ResolveZhupeilan {
            player,
            target,
            action: 10,
            ..
        } if !target.is_some() || *target == *player => {
            let owner = seat(*player, core)?;
            let opts = other_alive_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolveMbduanyang {
            player,
            mode: 0,
            cards,
            ..
        } if cards.is_empty() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        EffectRequest::ResolveMbrunwei {
            player,
            target,
            stage: 0,
            ..
        } if !target.is_some() || *target == *player => {
            let owner = seat(*player, core)?;
            let opts = alive_options(core);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolveMbkuangxiang {
            player,
            target,
            stage: 0,
            ..
        } if !target.is_some() || *target == *player => {
            let owner = seat(*player, core)?;
            let opts = other_alive_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolvePotsifeng {
            player,
            card,
            stage: 0,
            ..
        } if !card.is_some() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        EffectRequest::ResolvePotwanglie {
            player,
            card,
            stage: 0,
            ..
        } if !card.is_some() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        EffectRequest::ResolvePotchengpu {
            player,
            target,
            stage: 0,
            ..
        } if !target.is_some() || *target == *player => {
            let owner = seat(*player, core)?;
            let opts = other_alive_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolvePottuntian {
            player,
            action: 2,
            targets,
            ..
        } if targets.is_empty() => {
            let owner = seat(*player, core)?;
            let opts = alive_options(core);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolvePotjixi {
            player,
            action: 2,
            targets,
            ..
        } if targets.is_empty() => {
            let owner = seat(*player, core)?;
            let opts = other_alive_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        EffectRequest::ResolvePotjiyu {
            player,
            card,
            stage: 0,
            ..
        } if !card.is_some() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, false))
        }
        EffectRequest::ResolveDddGuanning {
            player,
            action: 2,
            card,
            ..
        } if !card.is_some() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        EffectRequest::ResolveDddJianshuo {
            player,
            action: 0,
            cards,
            ..
        } if cards.is_empty() => {
            let owner = seat(*player, core)?;
            let opts = hand_options(core, owner);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, 1, false))
        }
        EffectRequest::ResolveShenSimayi {
            player,
            mode: 3,
            cards,
            ..
        } if cards.is_empty() => {
            let owner = seat(*player, core)?;
            let opts = hand_equip_options(core, owner);
            let max = (opts.len() as u8).min(4);
            (opts.len() > 1).then_some((owner, ChoiceKind::SelectCards, opts, 1, max, true))
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

fn batch16_owner(effect: &EffectRequest, core: &GameStateCore) -> Option<u8> {
    match effect {
        EffectRequest::ResolveStddechong { player, .. }
        | EffectRequest::ResolveDddYujin { player, .. }
        | EffectRequest::ResolveDddZhenji { player, .. }
        | EffectRequest::ResolveZhengjian { player, .. }
        | EffectRequest::ResolveQiaobian { player, .. }
        | EffectRequest::ResolveTwfeiyi { player, .. }
        | EffectRequest::ResolveZhengsu { player, .. }
        | EffectRequest::ResolveTwcaoang { player, .. }
        | EffectRequest::ResolveTwcaozhao { player, .. }
        | EffectRequest::ResolveTwchengpu { player, .. }
        | EffectRequest::ResolveTwchenxun { player, .. }
        | EffectRequest::ResolveTwchenzhen { player, .. }
        | EffectRequest::ResolveTwyizhu { player, .. }
        | EffectRequest::ResolveRuanji { player, .. }
        | EffectRequest::ResolveQiaojian { player, .. }
        | EffectRequest::ResolveZhupeilan { player, .. }
        | EffectRequest::ResolveMbduanyang { player, .. }
        | EffectRequest::ResolveMbrunwei { player, .. }
        | EffectRequest::ResolveMbkuangxiang { player, .. }
        | EffectRequest::ResolvePotsifeng { player, .. }
        | EffectRequest::ResolvePotwanglie { player, .. }
        | EffectRequest::ResolvePotchengpu { player, .. }
        | EffectRequest::ResolvePottuntian { player, .. }
        | EffectRequest::ResolvePotjixi { player, .. }
        | EffectRequest::ResolvePotjiyu { player, .. }
        | EffectRequest::ResolvePotnigu { player, .. }
        | EffectRequest::ResolvePotfuan { player, .. }
        | EffectRequest::ResolveDddGuanning { player, .. }
        | EffectRequest::ResolveDddJianshuo { player, .. }
        | EffectRequest::ResolveShenSimayi { player, .. } => seat(*player, core),
        _ => None,
    }
}

pub(super) fn step(
    ctx: &mut Ctx<'_>,
    skill: &str,
    effect: &EffectRequest,
    state: &mut Vec<u32>,
) -> Option<Flow> {
    let owner = batch16_owner(effect, ctx.core)?;
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
        EffectRequest::ResolveStddechong { cards, .. }
        | EffectRequest::ResolveDddZhenji { cards, .. }
        | EffectRequest::ResolveRuanji { cards, .. }
        | EffectRequest::ResolveQiaojian { cards, .. }
        | EffectRequest::ResolveMbduanyang { cards, .. }
        | EffectRequest::ResolveDddJianshuo { cards, .. }
        | EffectRequest::ResolveShenSimayi { cards, .. } => {
            if !picked_cards.is_empty() {
                *cards = picked_cards;
            }
        }
        EffectRequest::ResolveDddYujin { target, .. }
        | EffectRequest::ResolveTwcaozhao { target, .. }
        | EffectRequest::ResolveTwchenxun { target, .. }
        | EffectRequest::ResolveZhupeilan { target, .. }
        | EffectRequest::ResolveMbrunwei { target, .. }
        | EffectRequest::ResolveMbkuangxiang { target, .. }
        | EffectRequest::ResolvePotchengpu { target, .. } => {
            if let Some(&p) = picked_players.first() {
                *target = p;
            }
        }
        EffectRequest::ResolveZhengjian { event_player, .. } => {
            if let Some(&p) = picked_players.first() {
                *event_player = p;
            }
        }
        EffectRequest::ResolveQiaobian { stage, .. } => {
            if let Some(&c) = picked_cards.first() {
                let raw = c.raw();
                if ctx.core.player(owner).hand.contains(&raw) {
                    ctx.discard_from_hand(owner, raw);
                    *stage = 1;
                }
            }
        }
        EffectRequest::ResolveTwfeiyi { stage, .. } => {
            if let Some(t) = picked_tag {
                *stage = t;
            }
        }
        EffectRequest::ResolveZhengsu { mode, .. } => {
            if let Some(t) = picked_tag {
                *mode = t;
            }
        }
        EffectRequest::ResolveTwcaoang { card, .. }
        | EffectRequest::ResolveTwyizhu { card, .. }
        | EffectRequest::ResolvePotsifeng { card, .. }
        | EffectRequest::ResolvePotwanglie { card, .. }
        | EffectRequest::ResolvePotjiyu { card, .. }
        | EffectRequest::ResolveDddGuanning { card, .. } => {
            if let Some(&c) = picked_cards.first() {
                *card = c;
            }
        }
        EffectRequest::ResolveTwchengpu { selected_card, .. } => {
            if let Some(&c) = picked_cards.first() {
                *selected_card = c;
            }
        }
        EffectRequest::ResolveTwchenzhen { card_kind, .. } => {
            if let Some(t) = picked_tag {
                *card_kind = t;
            }
        }
        EffectRequest::ResolvePottuntian { targets, .. }
        | EffectRequest::ResolvePotjixi { targets, .. } => {
            if !picked_players.is_empty() {
                *targets = picked_players;
            }
        }
        _ => {}
    }
    crate::rvm_ops::commit_effect(ctx, skill, owner, resolved);
    Some(Flow::Done)
}

pub(super) fn validate(effect: &EffectRequest, state: &[u32], core: &GameStateCore) -> bool {
    state == [1] && batch16_owner(effect, core).is_some()
}
