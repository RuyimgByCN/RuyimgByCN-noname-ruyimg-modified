//! 平寇: settle one complete damage frame at a time, and draw once per actual damaged player.
use super::*;

pub(super) fn active_ask(core: &GameStateCore, owner: u8, state: &[u32]) -> Option<ExpectedAsk> {
    if state != [1, 0, 0, 0] {
        return None;
    }
    let options: Vec<_> = core
        .alive_players()
        .filter(|p| *p != owner)
        .map(ChoiceOption::Player)
        .collect();
    let skipped = core
        .vm_aux
        .flags
        .get(&(owner, "vm_skipped_phase_count".to_string()))
        .copied()
        .unwrap_or(0) as usize;
    let max = skipped.min(options.len());
    if max == 0 {
        return None;
    }
    Some(ask(
        owner,
        "rvm_pingkou_targets",
        ChoiceKind::SelectPlayers,
        options,
        1,
        max,
        true,
    ))
}

pub(super) fn step(ctx: &mut Ctx<'_>, owner: u8, amount: u32, state: &mut Vec<u32>) -> Flow {
    let Ok(amount)=i32::try_from(amount) else {
        ctx.res.fault=Some(crate::machine::VmFault{skill:"pingkou".into(),owner,event:"Damage".into(),kind:"DamageAmountOverflow".into(),detail:"source amount exceeds i32".into()});return Flow::Done;
    };
    if state.is_empty() {
        *state = vec![1, 0, 0, 0];
        return active_ask(ctx.core, owner, state)
            .map(wait)
            .unwrap_or(Flow::Done);
    }
    if state.len() < 4 {
        return Flow::Done;
    }
    match state[0] {
        1 => {
            let mut targets: Vec<_> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .collect();
            if targets.is_empty() {
                return Flow::Done;
            }
            targets.sort_by_key(|p| ctx.core.seat_steps_from(owner, *p));
            state.extend(targets.into_iter().map(u32::from));
            state[0] = 2;
            Flow::Yield
        }
        2 => {
            if !ctx.core.player(owner).alive {
                return Flow::Done;
            }
            while (state[1] as usize) < state.len() - 4 {
                let target = state[4 + state[1] as usize] as u8;
                state[1] += 1;
                if !ctx.core.player(target).alive {
                    continue;
                }
                state[2] = ctx.damage_with_receipt(
                    owner,
                    crate::machine::damage_data(Some(owner), target, amount),
                );
                state[0] = 3;
                return Flow::Yield;
            }
            Flow::Done
        }
        3 => {
            let damaged = ctx
                .core
                .vm_aux
                .flags
                .remove(&(owner, format!("vm_damage_receipt:{}", state[2])))
                .unwrap_or(0);
            state[0] = 2;
            if damaged > 0 && damaged <= ctx.core.players.len() as u32 {
                let bit = 1 << (damaged - 1);
                if state[3] & bit == 0 {
                    state[3] |= bit;
                    if ctx.core.player(owner).alive {
                        ctx.draw_cards(owner, 1);
                    }
                }
            }
            Flow::Yield
        }
        _ => Flow::Done,
    }
}
