//! 冯河: give a physical hand card or recast a recastable card; reward only an actual give.
use super::*;

fn owns(core: &GameStateCore, catalog: &Catalog, owner: u8, source_def: u32) -> bool {
    source_def != 0
        && core.skills.keys().any(|(seat, def)| {
            *seat == owner
                && catalog.skills[*def as usize].behavior.package_skill_id() == Some(source_def)
        })
}

pub(super) fn active_ask(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: u8,
    state: &[u32],
) -> Option<ExpectedAsk> {
    let [stage, card, _source] = *state else {
        return None;
    };
    Some(match stage {
        1 => ask(
            owner,
            "rvm_fenghe_card",
            ChoiceKind::SelectCards,
            core.player(owner)
                .hand
                .iter()
                .map(|c| ChoiceOption::Card(*c))
                .collect(),
            1,
            1,
            false,
        ),
        2 if (card as usize) < catalog.cards.len() && core.player(owner).hand.contains(&card) => {
            let mut targets: Vec<_> = core
                .alive_players()
                .filter(|p| *p != owner)
                .map(ChoiceOption::Player)
                .collect();
            if catalog.card_kind(card) == crate::catalog::CardKind::Chain {
                targets.push(ChoiceOption::Pass);
            }
            ask(
                owner,
                "rvm_fenghe_destination",
                ChoiceKind::SelectPlayers,
                targets,
                1,
                1,
                false,
            )
        }
        _ => return None,
    })
}

pub(super) fn step(
    ctx: &mut Ctx<'_>,
    owner: u8,
    required: u32,
    grant_skill: u32,
    grant_mark: u32,
    state: &mut Vec<u32>,
) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if state.is_empty() {
        if ctx.core.player(owner).hand.is_empty() {
            return Flow::Done;
        }
        let source = ctx
            .current_damage()
            .ok()
            .and_then(|d| d.source)
            .map(|s| s as u32 + 1)
            .unwrap_or(0);
        *state = vec![1, 0, source];
        return wait(active_ask(ctx.core, ctx.catalog, owner, state).expect("card ask"));
    }
    if state.len() != 3 {
        return Flow::Done;
    }
    match state[0] {
        1 => {
            let Some(ChoiceOption::Card(card)) = ctx.answer().first() else {
                return Flow::Done;
            };
            state[1] = *card;
            state[0] = 2;
            wait(active_ask(ctx.core, ctx.catalog, owner, state).expect("destination ask"))
        }
        2 => {
            let card = state[1];
            if !ctx.core.player(owner).hand.contains(&card) {
                return Flow::Done;
            }
            match ctx.answer().first() {
                Some(ChoiceOption::Player(target)) => {
                    let target = *target;
                    ctx.give_hand_card(owner, target, card);
                    state[0] = 3;
                }
                Some(ChoiceOption::Pass)
                    if ctx.catalog.card_kind(card) == crate::catalog::CardKind::Chain =>
                {
                    discard(ctx, owner, &[card]);
                    state[0] = 4;
                }
                _ => return Flow::Done,
            }
            Flow::Yield
        }
        3 => {
            let Some(source) = state[2].checked_sub(1).map(|s| s as u8) else {
                return Flow::Done;
            };
            if ctx.core.player(source).alive
                && !ctx.core.player(source).dying
                && owns(ctx.core, ctx.catalog, owner, required)
            {
                if grant_mark != 0 {
                    *ctx.core
                        .vm_aux
                        .marks
                        .entry((source, grant_mark))
                        .or_default() += 1;
                }
                if grant_skill != 0 {
                    for (def, entry) in ctx.catalog.skills.iter().enumerate() {
                        if entry.behavior.package_skill_id() == Some(grant_skill) {
                            ctx.ensure_skill((source, def as u16),
                                crate::state::SkillInstance {
                                    owner: source,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: false,
                                    rounds: 0,
                                },
                            );
                        }
                    }
                }
            }
            Flow::Done
        }
        4 => {
            ctx.draw_cards(owner, 1);
            state[0] = 5;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}
