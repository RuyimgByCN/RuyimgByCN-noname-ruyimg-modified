//! OL界胆守: choose an unused option, settle it completely, then discard its ordinal.
use super::*;

fn used(core: &GameStateCore, owner: u8, skill: &str, option: u32) -> bool {
    core.vm_aux
        .flags
        .contains_key(&(owner, crate::rvm_ops::storage_key(skill, option)))
}

fn targets(core: &GameStateCore, owner: u8, option: u32) -> Vec<ChoiceOption> {
    core.alive_players()
        .filter(|target| match option {
            1 => !cards(core, *target).is_empty(),
            2 => *target != owner && !cards(core, *target).is_empty(),
            _ => true,
        })
        .map(ChoiceOption::Player)
        .collect()
}

pub(super) fn options(core: &GameStateCore, owner: u8, skill: &str) -> Vec<ChoiceOption> {
    if cards(core, owner).is_empty() {
        return Vec::new();
    }
    (1..=4)
        .filter(|option| {
            !used(core, owner, skill, *option) && !targets(core, owner, *option).is_empty()
        })
        .map(|option| ChoiceOption::Tag(option as u8))
        .collect()
}

pub(super) fn active_ask(
    core: &GameStateCore,
    owner: u8,
    skill: &str,
    state: &[u32],
) -> Option<ExpectedAsk> {
    let [stage, option, target] = *state else {
        return None;
    };
    Some(match stage {
        1 => ask(
            owner,
            "rvm_danshou_option",
            ChoiceKind::PickTag,
            options(core, owner, skill),
            1,
            1,
            true,
        ),
        2 => ask(
            owner,
            "rvm_danshou_target",
            ChoiceKind::SelectPlayers,
            targets(core, owner, option),
            1,
            1,
            false,
        ),
        3 if (target as usize) < core.players.len() => ask(
            owner,
            "rvm_danshou_card",
            ChoiceKind::SelectCards,
            cards(core, target as u8),
            1,
            1,
            false,
        ),
        5 => {
            let cards = cards(core, owner);
            let count = (option as usize).min(cards.len());
            if count == 0 {
                return None;
            }
            ask(
                owner,
                "rvm_danshou_discard",
                ChoiceKind::SelectCards,
                cards,
                count,
                count,
                false,
            )
        }
        _ => return None,
    })
}

pub(super) fn step(ctx: &mut Ctx<'_>, skill: &str, owner: u8, state: &mut Vec<u32>) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if state.is_empty() {
        if options(ctx.core, owner, skill).is_empty() {
            return Flow::Done;
        }
        *state = vec![1, 0, 0];
        return wait(active_ask(ctx.core, owner, skill, state).expect("option ask"));
    }
    if state.len() != 3 {
        return Flow::Done;
    }
    match state[0] {
        1 => {
            let Some(ChoiceOption::Tag(option)) = ctx.answer().first() else {
                return Flow::Done;
            };
            state[1] = *option as u32;
            state[0] = 2;
            wait(active_ask(ctx.core, owner, skill, state).expect("target ask"))
        }
        2 => {
            let Some(ChoiceOption::Player(target)) = ctx.answer().first() else {
                return Flow::Done;
            };
            let target = *target;
            let option = state[1];
            state[2] = target as u32;
            ctx.core
                .vm_aux
                .flags
                .insert((owner, crate::rvm_ops::storage_key(skill, option)), 1);
            ctx.core
                .vm_aux
                .storage
                .insert(format!("{skill}#{option}"), 1);
            if option <= 2 {
                state[0] = 3;
                return wait(active_ask(ctx.core, owner, skill, state).expect("entity ask"));
            }
            state[0] = 4;
            if option == 3 {
                ctx.emit(crate::event::EventData::Damage(
                    crate::machine::damage_data(Some(owner), target, 1),
                ));
            } else {
                state[0] = 7;
                ctx.draw_cards(owner, 2);
            }
            Flow::Yield
        }
        3 => {
            let Some(ChoiceOption::Card(card)) = ctx.answer().first() else {
                return Flow::Done;
            };
            let card = *card;
            let target = state[2] as u8;
            if state[1] == 1 {
                discard(ctx, target, &[card]);
            } else if ctx.core.player(target).hand.contains(&card) {
                ctx.give_hand_card(target, owner, card);
            } else {
                ctx.take_owned_card(target, card);
                ctx.core.player_mut(owner).hand.push(card);
                ctx.emit(crate::event::EventData::CardsMoved {
                    cards: vec![card],
                    from: target,
                    to: owner,
                });
            }
            state[0] = 4;
            Flow::Yield
        }
        7 => {
            state[0] = 4;
            let target = state[2] as u8;
            if ctx.core.player(target).alive {
                ctx.draw_cards(target, 2);
                Flow::Yield
            } else {
                self::step(ctx, skill, owner, state)
            }
        }
        4 => {
            state[0] = 5;
            active_ask(ctx.core, owner, skill, state)
                .map(wait)
                .unwrap_or(Flow::Done)
        }
        5 => {
            let picked: Vec<_> = ctx
                .answer()
                .iter()
                .filter_map(|c| match c {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            discard(ctx, owner, &picked);
            state[0] = 6;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}
