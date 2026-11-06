//! PA12 批⑬：选牌弃置/转化/移动交互原语（tags 115..=120）。
use super::batch13_targets::{other_alive_options, seats_from};
use super::*;

pub(super) fn scoped_cards(
    core: &GameStateCore,
    target: u8,
    scope: rule_api::TargetCardScope,
) -> Vec<ChoiceOption> {
    let p = core.player(target);
    let mut out = Vec::new();
    if scope.includes_hand() {
        out.extend(p.hand.iter().copied().map(ChoiceOption::Card));
    }
    if scope.includes_equip() {
        out.extend(p.equips.iter().copied().map(ChoiceOption::Card));
    }
    if scope.includes_judge() {
        out.extend(p.judge.iter().copied().map(ChoiceOption::Card));
    }
    out
}

fn basic_hand_options(core: &GameStateCore, catalog: &Catalog, owner: u8) -> Vec<ChoiceOption> {
    core.player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            (c as usize) < catalog.cards.len() && catalog.cards[c as usize].kind.is_basic_kind()
        })
        .map(ChoiceOption::Card)
        .collect()
}

fn virtual_sha_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: u8,
    ignore_distance: bool,
) -> Vec<ChoiceOption> {
    let atk = core.attack_range(catalog, owner);
    seats_from(core, owner)
        .into_iter()
        .filter(|&p| {
            p != owner
                && core.player(p).alive
                && (ignore_distance || core.computed_distance(catalog, owner, p) <= atk)
        })
        .map(ChoiceOption::Player)
        .collect()
}

fn other_discard_targets(
    core: &GameStateCore,
    owner: u8,
    scope: rule_api::TargetCardScope,
) -> Vec<ChoiceOption> {
    seats_from(core, owner)
        .into_iter()
        .filter(|&p| p != owner && core.player(p).alive && !scoped_cards(core, p, scope).is_empty())
        .map(ChoiceOption::Player)
        .collect()
}

pub(super) fn active_ask(
    effect: &EffectRequest,
    state: &[u32],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match effect {
        EffectRequest::ChooseDiscardOneHandOrEquip { player } if state == [1] => {
            let owner = seat(*player, core)?;
            let options = cards(core, owner);
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                "rvm_discard_pick",
                ChoiceKind::SelectCards,
                options,
                1,
                1,
                false,
            ))
        }
        EffectRequest::DiscardBasicDrawEqual { player, .. } if state == [1] => {
            let owner = seat(*player, core)?;
            let options = basic_hand_options(core, catalog, owner);
            let max = options.len();
            if max == 0 {
                return None;
            }
            Some(ask(
                owner,
                "rvm_discard_pick",
                ChoiceKind::SelectCards,
                options,
                1,
                max,
                true,
            ))
        }
        EffectRequest::ChooseUseVirtualSha {
            player,
            ignore_distance,
        } if state == [1] => {
            let owner = seat(*player, core)?;
            let options = virtual_sha_targets(core, catalog, owner, *ignore_distance);
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectPlayers,
                options,
                1,
                1,
                true,
            ))
        }
        EffectRequest::ChooseOtherDiscardTargetCard { player, scope, .. } => {
            let owner = seat(*player, core)?;
            match *state {
                [1] => {
                    let options = other_discard_targets(core, owner, *scope);
                    if options.is_empty() {
                        return None;
                    }
                    Some(ask(
                        owner,
                        "rvm_choice",
                        ChoiceKind::SelectPlayers,
                        options,
                        1,
                        1,
                        false,
                    ))
                }
                [2, target] if (target as usize) < core.players.len() => {
                    let options = scoped_cards(core, target as u8, *scope);
                    if options.is_empty() {
                        return None;
                    }
                    Some(ask(
                        owner,
                        "rvm_discard_pick",
                        ChoiceKind::SelectCards,
                        options,
                        1,
                        1,
                        false,
                    ))
                }
                _ => None,
            }
        }
        EffectRequest::ChooseDiscardTargetCard {
            player,
            target,
            scope,
        } if state == [1] => {
            let owner = seat(*player, core)?;
            let t = seat(*target, core)?;
            let options = scoped_cards(core, t, *scope);
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                "rvm_discard_pick",
                ChoiceKind::SelectCards,
                options,
                1,
                1,
                false,
            ))
        }
        EffectRequest::ChooseOtherGiveOneHandEquip { player, .. } => {
            let owner = seat(*player, core)?;
            match *state {
                [1] => {
                    let options = cards(core, owner);
                    if options.is_empty() {
                        return None;
                    }
                    Some(ask(
                        owner,
                        "rvm_choice",
                        ChoiceKind::SelectCards,
                        options,
                        1,
                        1,
                        false,
                    ))
                }
                [2, card]
                    if (card as usize) < catalog.cards.len()
                        && (core.player(owner).hand.contains(&card)
                            || core.player(owner).equips.contains(&card)) =>
                {
                    let options = other_alive_options(core, owner);
                    if options.is_empty() {
                        return None;
                    }
                    Some(ask(
                        owner,
                        "rvm_choice",
                        ChoiceKind::SelectPlayers,
                        options,
                        1,
                        1,
                        false,
                    ))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

pub(super) fn step(
    ctx: &mut Ctx<'_>,
    effect: &EffectRequest,
    state: &mut Vec<u32>,
) -> Option<Flow> {
    match effect {
        EffectRequest::ChooseDiscardOneHandOrEquip { player } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_discard_one_he(ctx, owner, state))
        }
        EffectRequest::DiscardBasicDrawEqual { player, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_discard_basic_draw_equal(ctx, owner, state))
        }
        EffectRequest::ChooseUseVirtualSha {
            player,
            ignore_distance,
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_use_virtual_sha(ctx, owner, *ignore_distance, state))
        }
        EffectRequest::ChooseOtherDiscardTargetCard { player, scope, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_other_discard_target_card(ctx, owner, *scope, state))
        }
        EffectRequest::ChooseDiscardTargetCard {
            player,
            target,
            scope,
        } => {
            let owner = seat(*player, ctx.core)?;
            let t = seat(*target, ctx.core)?;
            Some(step_discard_target_card(ctx, owner, t, *scope, state))
        }
        EffectRequest::ChooseOtherGiveOneHandEquip { player, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_other_give_one_he(ctx, owner, state))
        }
        _ => None,
    }
}

fn step_discard_one_he(ctx: &mut Ctx<'_>, owner: u8, state: &mut Vec<u32>) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if state.is_empty() {
        let options = cards(ctx.core, owner);
        if options.is_empty() {
            return Flow::Done;
        }
        *state = vec![1];
        return wait(ask(
            owner,
            "rvm_discard_pick",
            ChoiceKind::SelectCards,
            options,
            1,
            1,
            false,
        ));
    }
    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if ctx.core.player(owner).hand.contains(&card) || ctx.core.player(owner).equips.contains(&card)
    {
        discard(ctx, owner, &[card]);
    }
    Flow::Done
}

fn step_discard_basic_draw_equal(ctx: &mut Ctx<'_>, owner: u8, state: &mut Vec<u32>) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if state.is_empty() {
        let options = basic_hand_options(ctx.core, ctx.catalog, owner);
        let max = options.len();
        if max == 0 {
            return Flow::Done;
        }
        *state = vec![1];
        return wait(ask(
            owner,
            "rvm_discard_pick",
            ChoiceKind::SelectCards,
            options,
            1,
            max,
            true,
        ));
    }
    let picked: Vec<u32> = ctx
        .answer()
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c)
                if ctx.core.player(owner).hand.contains(c)
                    && (*c as usize) < ctx.catalog.cards.len()
                    && ctx.catalog.cards[*c as usize].kind.is_basic_kind() =>
            {
                Some(*c)
            }
            _ => None,
        })
        .collect();
    if !picked.is_empty() {
        let k = picked.len() as u32;
        discard(ctx, owner, &picked);
        ctx.start_draw(owner, k);
    }
    Flow::Done
}

fn step_use_virtual_sha(
    ctx: &mut Ctx<'_>,
    owner: u8,
    ignore_distance: bool,
    state: &mut Vec<u32>,
) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if state.is_empty() {
        let options = virtual_sha_targets(ctx.core, ctx.catalog, owner, ignore_distance);
        if options.is_empty() {
            return Flow::Done;
        }
        *state = vec![1];
        return wait(ask(
            owner,
            "rvm_choice",
            ChoiceKind::SelectPlayers,
            options,
            1,
            1,
            true,
        ));
    }
    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if virtual_sha_targets(ctx.core, ctx.catalog, owner, ignore_distance)
        .contains(&ChoiceOption::Player(target))
    {
        ctx.emit(EventData::CardUsed(crate::event::CardUseData {
            card: None,
            player: owner,
            targets: vec![target],
            nature: crate::event::DamageNature::Normal,
            use_as: Some(crate::catalog::CardKind::Sha),
            cancelled: false,
        }));
    }
    Flow::Done
}

fn step_other_discard_target_card(
    ctx: &mut Ctx<'_>,
    owner: u8,
    scope: rule_api::TargetCardScope,
    state: &mut Vec<u32>,
) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if state.is_empty() {
        let options = other_discard_targets(ctx.core, owner, scope);
        if options.is_empty() {
            return Flow::Done;
        }
        *state = vec![1];
        return wait(ask(
            owner,
            "rvm_choice",
            ChoiceKind::SelectPlayers,
            options,
            1,
            1,
            false,
        ));
    }
    match state.as_slice() {
        [1] => {
            let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !other_discard_targets(ctx.core, owner, scope)
                .contains(&ChoiceOption::Player(target))
            {
                return Flow::Done;
            }
            let options = scoped_cards(ctx.core, target, scope);
            if options.is_empty() {
                return Flow::Done;
            }
            *state = vec![2, target as u32];
            wait(ask(
                owner,
                "rvm_discard_pick",
                ChoiceKind::SelectCards,
                options,
                1,
                1,
                false,
            ))
        }
        [2, target] => {
            let target = *target as u8;
            let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if scoped_cards(ctx.core, target, scope).contains(&ChoiceOption::Card(card)) {
                discard(ctx, target, &[card]);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_discard_target_card(
    ctx: &mut Ctx<'_>,
    owner: u8,
    target: u8,
    scope: rule_api::TargetCardScope,
    state: &mut Vec<u32>,
) -> Flow {
    if !ctx.core.player(owner).alive || !ctx.core.player(target).alive {
        return Flow::Done;
    }
    if state.is_empty() {
        let options = scoped_cards(ctx.core, target, scope);
        if options.is_empty() {
            return Flow::Done;
        }
        *state = vec![1];
        return wait(ask(
            owner,
            "rvm_discard_pick",
            ChoiceKind::SelectCards,
            options,
            1,
            1,
            false,
        ));
    }
    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if scoped_cards(ctx.core, target, scope).contains(&ChoiceOption::Card(card)) {
        discard(ctx, target, &[card]);
    }
    Flow::Done
}

fn step_other_give_one_he(ctx: &mut Ctx<'_>, owner: u8, state: &mut Vec<u32>) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if state.is_empty() {
        let card_opts = cards(ctx.core, owner);
        if card_opts.is_empty() || other_alive_options(ctx.core, owner).is_empty() {
            return Flow::Done;
        }
        *state = vec![1];
        return wait(ask(
            owner,
            "rvm_choice",
            ChoiceKind::SelectCards,
            card_opts,
            1,
            1,
            false,
        ));
    }
    match state.as_slice() {
        [1] => {
            let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let target_opts = other_alive_options(ctx.core, owner);
            if target_opts.is_empty() {
                return Flow::Done;
            }
            *state = vec![2, card];
            wait(ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectPlayers,
                target_opts,
                1,
                1,
                false,
            ))
        }
        [2, card] => {
            let card = *card;
            let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if target == owner || !ctx.core.player(target).alive {
                return Flow::Done;
            }
            if ctx.core.player(owner).hand.contains(&card) {
                ctx.give_hand_card(owner, target, card);
            } else if ctx.core.player(owner).equips.contains(&card) {
                ctx.take_owned_card(owner, card);
                ctx.core.player_mut(target).hand.push(card);
                ctx.out.push(crate::machine::LogEvent::CardsGivenPublic {
                    from: owner,
                    to: target,
                    cards: vec![card],
                });
                ctx.emit(EventData::CardsMoved {
                    cards: vec![card],
                    from: owner,
                    to: target,
                });
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn validate(
    effect: &EffectRequest,
    state: &[u32],
    core: &GameStateCore,
    card_count: usize,
) -> bool {
    let seats = core.players.len();
    match effect {
        EffectRequest::ChooseDiscardOneHandOrEquip { player }
        | EffectRequest::DiscardBasicDrawEqual { player, .. }
        | EffectRequest::ChooseUseVirtualSha { player, .. } => {
            player.raw() > 0 && (player.raw() as usize) <= seats && state == [1]
        }
        EffectRequest::ChooseDiscardTargetCard { player, target, .. } => {
            player.raw() > 0
                && (player.raw() as usize) <= seats
                && target.raw() > 0
                && (target.raw() as usize) <= seats
                && state == [1]
        }
        EffectRequest::ChooseOtherDiscardTargetCard { player, .. } => {
            player.raw() > 0
                && (player.raw() as usize) <= seats
                && (state == [1]
                    || (state.len() == 2
                        && state[0] == 2
                        && (state[1] as usize) < seats
                        && state[1] + 1 != player.raw()))
        }
        EffectRequest::ChooseOtherGiveOneHandEquip { player, .. } => {
            player.raw() > 0
                && (player.raw() as usize) <= seats
                && (state == [1]
                    || (state.len() == 2
                        && state[0] == 2
                        && (state[1] as usize) < card_count
                        && (core.player((player.raw() - 1) as u8).hand.contains(&state[1])
                            || core.player((player.raw() - 1) as u8).equips.contains(&state[1]))))
        }
        _ => false,
    }
}
