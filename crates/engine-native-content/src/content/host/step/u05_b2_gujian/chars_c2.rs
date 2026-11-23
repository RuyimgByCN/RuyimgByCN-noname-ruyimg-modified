use super::chars_c::*;
use super::helpers::*;
use super::super::*;
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::event::EventData;
use crate::ids::PlayerId;
use crate::machine::{damage_data, Ctx, Flow};
use crate::state::SkillInstance;

pub(super) fn step_duhun(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            let cands = duhun_candidates(ctx.core, machine.owner);
            if cands.is_empty() {
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::TargetChoice,
                machine.owner,
                ASK_DUHUN_TARGET,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gj_duhun_target",
                    options: cands.into_iter().map(ChoiceOption::Player).collect(),
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            )
        }
        HostStage::TargetChoice => {
            let ChoiceOption::Player(t) = answer_first_or_pass(ctx) else {
                return Flow::Done;
            };
            machine.target = Some(t);
            let opts: Vec<ChoiceOption> = ctx
                .core
                .player(machine.owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            if opts.is_empty() || ctx.core.player(t).hand.is_empty() {
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::ComparePlayerPick,
                machine.owner,
                ASK_DUHUN_COMPARE_OWN,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: GJ_BINGPO_SHOW_OWN,
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            )
        }
        HostStage::ComparePlayerPick => {
            let ChoiceOption::Card(c_own) = answer_first_or_pass(ctx) else {
                return Flow::Done;
            };
            let Some(t) = machine.target else {
                return Flow::Done;
            };
            machine.cards = vec![c_own];
            let opts: Vec<ChoiceOption> = ctx
                .core
                .player(t)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            if opts.is_empty() {
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::CompareTargetPick,
                t,
                ASK_DUHUN_COMPARE_TARGET,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: GJ_BINGPO_SHOW_TARGET,
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            )
        }
        HostStage::CompareTargetPick => {
            let ChoiceOption::Card(c_t) = answer_first_or_pass(ctx) else {
                return Flow::Done;
            };
            let Some(c_own) = machine.cards.first().copied() else {
                return Flow::Done;
            };
            let Some(t) = machine.target else {
                return Flow::Done;
            };
            let r_own = ctx.catalog.cards[c_own as usize].rank;
            let r_t = ctx.catalog.cards[c_t as usize].rank;
            discard_card_from_he(ctx, machine.owner, c_own);
            discard_card_from_he(ctx, t, c_t);
            if r_own > r_t {
                let diff = ctx.core.player(t).hp - ctx.core.player(machine.owner).hp;
                ctx.lose_max_hp(machine.owner, 1);
                let new_own_hp = (ctx.core.player(machine.owner).hp + diff)
                    .min(ctx.core.player(machine.owner).max_hp as i32);
                ctx.core.player_mut(machine.owner).hp = new_own_hp;
                if new_own_hp > 0 {
                    ctx.core.player_mut(machine.owner).dying = false;
                }
                ctx.core.player_mut(t).hp =
                    (ctx.core.player(t).hp - diff).min(ctx.core.player(t).max_hp as i32);
                if ctx.core.player(t).hp <= 0 {
                    ctx.emit(EventData::Dying {
                        player: t,
                        source: Some(machine.owner),
                    });
                }
            } else {
                die_player(ctx, machine.owner, None);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_xuanning1(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            let opts: Vec<ChoiceOption> = ctx
                .core
                .player(machine.owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| ctx.catalog.card_kind(c).is_basic_kind())
                .map(ChoiceOption::Card)
                .collect();
            if opts.is_empty() {
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::SelectCard,
                machine.owner,
                ASK_XUANNING1_CARD,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_xuanning1_card",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            )
        }
        HostStage::SelectCard => {
            let ChoiceOption::Card(cid) = answer_first_or_pass(ctx) else {
                return Flow::Done;
            };
            if discard_card_from_he(ctx, machine.owner, cid) {
                set_flag(ctx.core, machine.owner, "gj_xuanning1_used_turn", 1);
                set_flag(ctx.core, machine.owner, "gj_xuanning", 3);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_liuguang(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    loop {
        match machine.stage {
            HostStage::Act => {
                let opts = push_non_self_alive(ctx.core, machine.owner);
                if opts.is_empty() || flag(ctx.core, machine.owner, "gj_xuanning") == 0 {
                    return Flow::Done;
                }
                let max = opts.len().min(3);
                return wait_ask(
                    machine,
                    ctx,
                    HostStage::TargetChoice,
                    machine.owner,
                    ASK_LIUGUANG_TARGETS,
                    ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "gj_liuguang_targets",
                        options: opts,
                        min: 1,
                        max,
                        allow_cancel: true,
                    },
                );
            }
            HostStage::TargetChoice => {
                let mut picked: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    })
                    .collect();
                if picked.is_empty() {
                    return Flow::Done;
                }
                let cnt = flag(ctx.core, machine.owner, "gj_xuanning");
                set_flag(ctx.core, machine.owner, "gj_xuanning", cnt.saturating_sub(1));
                let order = ctx.core.seat_order_from(machine.owner);
                picked.sort_by_key(|p| order.iter().position(|x| x == p).unwrap_or(99));
                machine.targets = picked;
                machine.stage = HostStage::DrawSettle;
                continue;
            }
            HostStage::DrawSettle => {
                let Some(t) = machine.targets.first().copied() else {
                    return Flow::Done;
                };
                machine.target = Some(t);
                if !ctx.core.player(t).alive {
                    machine.targets.remove(0);
                    continue;
                }
                let he: Vec<ChoiceOption> = hand_and_equips(ctx.core, t)
                    .into_iter()
                    .map(ChoiceOption::Card)
                    .collect();
                if he.is_empty() {
                    machine.targets.remove(0);
                    ctx.emit(EventData::Damage(damage_data(Some(machine.owner), t, 1)));
                    return Flow::Yield;
                }
                return wait_ask(
                    machine,
                    ctx,
                    HostStage::SelectCard,
                    t,
                    ASK_LIUGUANG_RESPOND,
                    ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "gj_liuguang_respond",
                        options: he,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                );
            }
            HostStage::SelectCard => {
                machine.targets.remove(0);
                machine.stage = HostStage::DrawSettle;
                let Some(t) = machine.target else {
                    continue;
                };
                if let ChoiceOption::Card(cid) = answer_first_or_pass(ctx) {
                    discard_card_from_he(ctx, t, cid);
                    continue;
                }
                if ctx.core.player(t).alive {
                    ctx.emit(EventData::Damage(damage_data(Some(machine.owner), t, 1)));
                    return Flow::Yield;
                }
                continue;
            }
            _ => return Flow::Done,
        }
    }
}

pub(super) fn step_yangming(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            let opts: Vec<ChoiceOption> = ctx
                .core
                .player(machine.owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| is_red(ctx.catalog, c))
                .map(ChoiceOption::Card)
                .collect();
            if opts.is_empty() {
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::SelectCard,
                machine.owner,
                ASK_YANGMING_CARD,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_yangming_card",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            )
        }
        HostStage::SelectCard => {
            let ChoiceOption::Card(cid) = answer_first_or_pass(ctx) else {
                return Flow::Done;
            };
            if discard_card_from_he(ctx, machine.owner, cid) {
                set_flag(ctx.core, machine.owner, "gj_yangming2", 2);
                if let Some(def) = ctx.catalog.skill_by_name("yangming2") {
                    ctx.insert_skill(
                        (machine.owner, def),
                        SkillInstance {
                            owner: machine.owner,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_yangming2(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            let rem = flag(ctx.core, machine.owner, "gj_yangming2");
            let next = rem.saturating_sub(1);
            set_flag(ctx.core, machine.owner, "gj_yangming2", next);
            if next > 0 {
                return Flow::Done;
            }
            if let Some(def) = ctx.catalog.skill_by_name("yangming2") {
                ctx.core.skills.remove(&(machine.owner, def));
            }
            let cands = yangming2_targets(ctx.core, ctx.catalog, machine.owner);
            if cands.is_empty() {
                return Flow::Done;
            }
            let max = cands.len();
            wait_ask(
                machine,
                ctx,
                HostStage::TargetChoice,
                machine.owner,
                ASK_YANGMING2_TARGETS,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gj_yangming2_targets",
                    options: cands.into_iter().map(ChoiceOption::Player).collect(),
                    min: 1,
                    max,
                    allow_cancel: true,
                },
            )
        }
        HostStage::TargetChoice => {
            for opt in ctx.answer().to_vec() {
                if let ChoiceOption::Player(t) = opt {
                    if ctx.core.player(t).alive {
                        ctx.recover(t, 1, None);
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
