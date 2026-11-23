use super::helpers::*;
use super::super::*;
use crate::catalog::{Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{damage_data_with, Ctx, Flow, LogEvent};
use crate::state::{GameStateCore, SkillInstance};

const ASK_XUELU_CARD: u32 = 101;
const ASK_XUELU_TARGET: u32 = 102;
const ASK_QINGLAN_CARD: u32 = 103;
const ASK_XLQIANHUAN_CONFIRM: u32 = 104;
const ASK_MEIHU_CONFIRM: u32 = 105;
const ASK_MEIHU_GIVE: u32 = 106;
const ASK_XIDIE_DISCARD: u32 = 107;
const ASK_FANYIN_TARGET: u32 = 108;
const ASK_FUMO_CARD1: u32 = 109;
const ASK_FUMO_CARD2: u32 = 110;
const ASK_YUEHUA_CONFIRM: u32 = 111;

pub(in crate::content::host) fn trigger_meta_char_a(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events: Vec::new(),
    };
    match name {
        "xuelu" => Some(meta(EventKind::TurnPhaseEnd, Timing::Normal, 0)),
        "fanshi" => Some(meta(EventKind::PhaseEnd, Timing::Normal, 0)),
        "qinglan" => Some(meta(EventKind::DamageBefore, Timing::Normal, -5)),
        "yuehua" => Some(TriggerMeta {
            event: EventKind::CardUsed,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: false,
            extra_events: vec![EventKind::CardsDiscarded, EventKind::CardResponded],
        }),
        "xlqianhuan" => Some(meta(EventKind::TurnPhaseAfter, Timing::Normal, 0)),
        "meihu" => Some(meta(EventKind::Damage, Timing::Normal, 0)),
        "xidie" => Some(TriggerMeta {
            event: EventKind::TurnBegin,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: false,
            extra_events: vec![EventKind::TurnPhaseEnd],
        }),
        "xidie2" => Some(meta(EventKind::TurnPhaseEnd, Timing::Normal, 0)),
        "fanyin" => Some(meta(EventKind::TurnPhaseEnd, Timing::Normal, 0)),
        "mingkong" => Some(meta(EventKind::DamageBegin, Timing::Normal, 0)),
        "mingkong2" => Some(TriggerMeta {
            event: EventKind::DamageEnd,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: false,
            extra_events: vec![EventKind::DamageZero],
        }),
        "fumo" => Some(meta(EventKind::Damage, Timing::Normal, 0)),
        "shahun" => Some(meta(EventKind::Dying, Timing::Normal, 0)),
        "shahun2" => Some(meta(EventKind::TurnPhaseAfter, Timing::Normal, 0)),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_char_a(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn filter_char_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "xuelu" => {
            matches!(event, EventData::TurnEndWindow { player, after: false } if *player == owner)
                && ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
                && hand_and_equips(ctx.state, owner)
                    .into_iter()
                    .any(|c| is_red(ctx.catalog, c))
                && ctx.state.alive_players().any(|p| p != owner)
        }
        "fanshi" => {
            matches!(
                event,
                EventData::Phase {
                    player,
                    phase: Phase::Discard,
                    begin: false,
                } if *player == owner
            ) && ctx.state.player(owner).turn_damage > 0
        }
        "qinglan" => match event {
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && matches!(d.nature, DamageNature::Fire | DamageNature::Thunder)
                    && !hand_and_equips(ctx.state, owner).is_empty()
            }
            _ => false,
        },
        "yuehua" => {
            if ctx.state.turn.current == owner {
                return false;
            }
            match event {
                EventData::CardUsed(u) if u.player == owner && !u.cancelled => {
                    u.card.is_some_and(|c| is_red(ctx.catalog, c))
                }
                EventData::CardResponded { player, card } if *player == owner => {
                    is_red(ctx.catalog, *card)
                }
                EventData::CardsDiscarded { player, cards } if *player == owner => cards
                    .iter()
                    .any(|&c| is_red(ctx.catalog, c) && !ctx.state.player(owner).judge.contains(&c)),
                _ => false,
            }
        }
        "xlqianhuan" => {
            matches!(event, EventData::TurnEndWindow { player, after: true } if *player == owner)
                && ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
        }
        "meihu" => match event {
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.source.is_some_and(|s| {
                        s != owner
                            && ctx.state.player(s).alive
                            && !ctx.state.player(s).hand.is_empty()
                    })
            }
            _ => false,
        },
        "xidie" => {
            (matches!(event, EventData::TurnBegin { player } if *player == owner)
                && ctx.state.player(owner).hand.len() as i32 > ctx.state.player(owner).hp)
                || (matches!(event, EventData::TurnEndWindow { player, after: false } if *player == owner)
                    && flag(ctx.state, owner, "gj_xidie_count") > 0)
        }
        "xidie2" => {
            matches!(event, EventData::TurnEndWindow { player, after: false } if *player == owner)
                && flag(ctx.state, owner, "gj_xidie_count") > 0
        }
        "fanyin" => {
            matches!(event, EventData::TurnEndWindow { player, after: false } if *player == owner)
                && ctx.state.alive_players().any(|p| fanyin_eligible(ctx.state, p))
        }
        "mingkong" => match event {
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount >= 1
                    && ctx.state.player(owner).hand.is_empty()
            }
            _ => false,
        },
        "mingkong2" => match event {
            EventData::Damage(d) => {
                d.source == Some(owner)
                    && !d.prevented
                    && flag(ctx.state, owner, "gj_mingkong2_pending") > 0
            }
            _ => false,
        },
        "fumo" => match event {
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.source.is_some_and(|s| ctx.state.player(s).alive)
                    && (ctx
                        .state
                        .player(owner)
                        .hand
                        .iter()
                        .filter(|&&c| is_red(ctx.catalog, c))
                        .count()
                        >= 2
                        || ctx
                            .state
                            .player(owner)
                            .hand
                            .iter()
                            .filter(|&&c| is_black(ctx.catalog, c))
                            .count()
                            >= 2)
            }
            _ => false,
        },
        "shahun2" => {
            matches!(event, EventData::TurnEndWindow { player, after: true } if *player == owner)
                && flag(ctx.state, owner, "gj_shahun_timer") > 0
        }
        _ => false,
    }
}

pub(super) fn fanyin_eligible(core: &GameStateCore, target: PlayerId) -> bool {
    let p = core.player(target);
    if !p.alive {
        return false;
    }
    if p.linked || p.face_down || !p.judge.is_empty() {
        return true;
    }
    let is_min_hp = core.alive_players().all(|q| core.player(q).hp >= p.hp);
    is_min_hp && p.hp < p.max_hp as i32
}

pub(in crate::content::host) fn machine_char_a(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let (target, card) = match event {
        EventData::Damage(d) => (d.source.or(Some(d.target)), Some(d.target as u32)),
        _ => (None, None),
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::U05GujianCharBatchA,
        owner,
        target,
        card,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(super) fn step_char_a(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.def_key.as_str() {
        "xuelu" => step_xuelu(machine, ctx),
        "fanshi" => match machine.stage {
            HostStage::Act => {
                ctx.emit(EventData::LoseHp {
                    player: machine.owner,
                    amount: 1,
                });
                machine.stage = HostStage::DrawSettle;
                Flow::Yield
            }
            HostStage::DrawSettle => {
                if ctx.core.player(machine.owner).alive {
                    ctx.start_draw(machine.owner, 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "qinglan" => step_qinglan(machine, ctx),
        "yuehua" => step_yuehua(machine, ctx),
        "xlqianhuan" => step_xlqianhuan(machine, ctx),
        "meihu" => step_meihu(machine, ctx),
        "xidie" => {
            if ctx.core.turn.phase == Phase::End && matches!(machine.stage, HostStage::Act) {
                let n = flag(ctx.core, machine.owner, "gj_xidie_count");
                if n > 0 {
                    set_flag(ctx.core, machine.owner, "gj_xidie_count", 0);
                    ctx.start_draw(machine.owner, n);
                    return Flow::Done;
                }
            }
            step_xidie(machine, ctx)
        }
        "xidie2" => {
            let n = flag(ctx.core, machine.owner, "gj_xidie_count");
            set_flag(ctx.core, machine.owner, "gj_xidie_count", 0);
            if n > 0 {
                ctx.start_draw(machine.owner, n);
            }
            Flow::Done
        }
        "fanyin" => step_fanyin(machine, ctx),
        "mingkong" => {
            let hand_empty = ctx.core.player(machine.owner).hand.is_empty();
            let source = {
                let d = ctx.current_damage_mut();
                if d.amount <= 0 || !hand_empty {
                    return Flow::Done;
                }
                d.amount = d.amount.saturating_sub(1);
                d.source
            };
            if let Some(src) = source {
                if ctx.core.player(src).alive {
                    if let Some(def) = ctx.catalog.skill_by_name("mingkong2") {
                        set_flag(ctx.core, src, "gj_mingkong2_pending", 1);
                        ctx.insert_skill(
                            (src, def),
                            SkillInstance {
                                owner: src,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: true,
                                rounds: 0,
                            },
                        );
                    }
                }
            }
            Flow::Done
        }
        "mingkong2" => {
            if flag(ctx.core, machine.owner, "gj_mingkong2_pending") > 0 {
                set_flag(ctx.core, machine.owner, "gj_mingkong2_pending", 0);
                ctx.start_draw(machine.owner, 1);
            }
            if let Some(def) = ctx.catalog.skill_by_name("mingkong2") {
                ctx.core.skills.remove(&(machine.owner, def));
            }
            Flow::Done
        }
        "fumo" => step_fumo(machine, ctx),
        "shahun" => super::chars_e2::step_shahun(machine, ctx),
        "shahun2" => {
            let t = flag(ctx.core, machine.owner, "gj_shahun_timer");
            if t > 1 {
                set_flag(ctx.core, machine.owner, "gj_shahun_timer", t - 1);
            } else if t == 1 {
                set_flag(ctx.core, machine.owner, "gj_shahun_timer", 0);
                die_player(ctx, machine.owner, None);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_xuelu(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            let opts: Vec<ChoiceOption> = hand_and_equips(ctx.core, machine.owner)
                .into_iter()
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
                ASK_XUELU_CARD,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_xuelu_card",
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
            let targets = push_non_self_alive(ctx.core, machine.owner);
            if targets.is_empty() {
                return Flow::Done;
            }
            machine.cards = vec![cid];
            wait_ask(
                machine,
                ctx,
                HostStage::TargetChoice,
                machine.owner,
                ASK_XUELU_TARGET,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gj_xuelu_target",
                    options: targets,
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
            let Some(cid) = machine.cards.first().copied() else {
                return Flow::Done;
            };
            let lost = (ctx.core.player(machine.owner).max_hp as i32
                - ctx.core.player(machine.owner).hp)
                .max(0);
            let dmg = ((lost + 1) / 2).min(2);
            discard_card_from_he(ctx, machine.owner, cid);
            if dmg > 0 && ctx.core.player(t).alive {
                ctx.emit(EventData::Damage(damage_data_with(
                    Some(machine.owner),
                    t,
                    dmg,
                    DamageNature::Fire,
                    None,
                )));
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_qinglan(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            let opts: Vec<ChoiceOption> = hand_and_equips(ctx.core, machine.owner)
                .into_iter()
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
                ASK_QINGLAN_CARD,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_qinglan_card",
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
            discard_card_from_he(ctx, machine.owner, cid);
            let src = {
                let d = ctx.current_damage_mut();
                d.prevented = true;
                d.amount = 0;
                d.source
            };
            if let Some(s) = src {
                if ctx.core.player(s).alive {
                    ctx.recover(s, 1, None);
                    ctx.start_draw(s, 1);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_yuehua(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => wait_ask(
            machine,
            ctx,
            HostStage::SelectCard,
            machine.owner,
            ASK_YUEHUA_CONFIRM,
            ChoiceSpec {
                kind: ChoiceKind::Confirm,
                prompt: "gj_yuehua_confirm",
                options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        ),
        HostStage::SelectCard => {
            if matches!(answer_first_or_pass(ctx), ChoiceOption::Confirm) {
                ctx.start_draw(machine.owner, 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_xlqianhuan(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => wait_ask(
            machine,
            ctx,
            HostStage::SelectCard,
            machine.owner,
            ASK_XLQIANHUAN_CONFIRM,
            ChoiceSpec {
                kind: ChoiceKind::Confirm,
                prompt: "gj_xlqianhuan_confirm",
                options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        ),
        HostStage::SelectCard => {
            if matches!(answer_first_or_pass(ctx), ChoiceOption::Confirm) {
                ctx.recover(machine.owner, 1, None);
                ctx.turn_over(machine.owner);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_meihu(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(src) = machine.target else {
        return Flow::Done;
    };
    match machine.stage {
        HostStage::Act => {
            if !ctx.core.player(src).alive || ctx.core.player(src).hand.is_empty() {
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::SelectCard,
                machine.owner,
                ASK_MEIHU_CONFIRM,
                ChoiceSpec {
                    kind: ChoiceKind::Confirm,
                    prompt: "gj_meihu_confirm",
                    options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            )
        }
        HostStage::SelectCard => {
            if !matches!(answer_first_or_pass(ctx), ChoiceOption::Confirm) {
                return Flow::Done;
            }
            let opts: Vec<ChoiceOption> = ctx
                .core
                .player(src)
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
                HostStage::TargetChoice,
                src,
                ASK_MEIHU_GIVE,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_meihu_give",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            )
        }
        HostStage::TargetChoice => {
            if let ChoiceOption::Card(cid) = answer_first_or_pass(ctx) {
                give_card(ctx, src, machine.owner, cid);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_xidie(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            let excess =
                (ctx.core.player(machine.owner).hand.len() as i32 - ctx.core.player(machine.owner).hp)
                    .max(0) as usize;
            let max = excess.min(3);
            if max == 0 {
                return Flow::Done;
            }
            let opts: Vec<ChoiceOption> = ctx
                .core
                .player(machine.owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            wait_ask(
                machine,
                ctx,
                HostStage::SelectCard,
                machine.owner,
                ASK_XIDIE_DISCARD,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_xidie_discard",
                    options: opts,
                    min: 1,
                    max,
                    allow_cancel: true,
                },
            )
        }
        HostStage::SelectCard => {
            let picked: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if !picked.is_empty() {
                discard_cards_from_he(ctx, machine.owner, &picked);
                set_flag(ctx.core, machine.owner, "gj_xidie_count", picked.len() as u32);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_fanyin(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            let opts: Vec<ChoiceOption> = ctx
                .core
                .alive_players()
                .filter(|&p| fanyin_eligible(ctx.core, p))
                .map(ChoiceOption::Player)
                .collect();
            if opts.is_empty() {
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::TargetChoice,
                machine.owner,
                ASK_FANYIN_TARGET,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gj_fanyin_target",
                    options: opts,
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
            if !ctx.core.player(t).alive {
                return Flow::Done;
            }
            if ctx.core.player(t).linked {
                ctx.core.player_mut(t).linked = false;
            }
            if ctx.core.player(t).face_down {
                ctx.turn_over(t);
            }
            let judges: Vec<CardId> = ctx.core.player_mut(t).judge.drain(..).collect();
            for &jc in &judges {
                ctx.to_discard(jc);
            }
            if !judges.is_empty() {
                ctx.log(LogEvent::CardDiscarded {
                    player: t,
                    cards: judges,
                });
            }
            let hp = ctx.core.player(t).hp;
            if ctx.core.alive_players().all(|q| ctx.core.player(q).hp >= hp) {
                ctx.recover(t, 1, None);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_fumo(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(src) = machine.target else {
        return Flow::Done;
    };
    match machine.stage {
        HostStage::Act => {
            let hand = &ctx.core.player(machine.owner).hand;
            let red_n = hand.iter().filter(|&&c| is_red(ctx.catalog, c)).count();
            let blk_n = hand.iter().filter(|&&c| is_black(ctx.catalog, c)).count();
            let opts: Vec<ChoiceOption> = hand
                .iter()
                .copied()
                .filter(|&c| (is_red(ctx.catalog, c) && red_n >= 2) || (is_black(ctx.catalog, c) && blk_n >= 2))
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
                ASK_FUMO_CARD1,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_fumo_card1",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            )
        }
        HostStage::SelectCard => {
            let ChoiceOption::Card(c1) = answer_first_or_pass(ctx) else {
                return Flow::Done;
            };
            let c1_red = is_red(ctx.catalog, c1);
            let opts: Vec<ChoiceOption> = ctx
                .core
                .player(machine.owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| c != c1 && is_red(ctx.catalog, c) == c1_red)
                .map(ChoiceOption::Card)
                .collect();
            if opts.is_empty() {
                return Flow::Done;
            }
            machine.cards = vec![c1];
            wait_ask(
                machine,
                ctx,
                HostStage::TargetChoice,
                machine.owner,
                ASK_FUMO_CARD2,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_fumo_card2",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            )
        }
        HostStage::TargetChoice => {
            let ChoiceOption::Card(c2) = answer_first_or_pass(ctx) else {
                return Flow::Done;
            };
            let Some(c1) = machine.cards.first().copied() else {
                return Flow::Done;
            };
            discard_cards_from_he(ctx, machine.owner, &[c1, c2]);
            if ctx.core.player(src).alive {
                ctx.emit(EventData::Damage(damage_data_with(
                    Some(machine.owner),
                    src,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn ask_char_a(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<crate::snap::ExpectedAsk> {
    let ask_id = current_ask_id(core, owner, HostBehavior::U05GujianCharBatchA);
    let obs = current_ask_obs(core, owner, HostBehavior::U05GujianCharBatchA);
    match (stage, ask_id) {
        (HostStage::SelectCard, ASK_XUELU_CARD) => build_ask(
            obs,
            "gj_xuelu_card",
            ChoiceKind::SelectCards,
            hand_and_equips(core, owner)
                .into_iter()
                .filter(|&c| is_red(catalog, c))
                .map(ChoiceOption::Card)
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::TargetChoice, ASK_XUELU_TARGET) => build_ask(
            obs,
            "gj_xuelu_target",
            ChoiceKind::SelectPlayers,
            push_non_self_alive(core, owner),
            1,
            1,
            true,
        ),
        (HostStage::SelectCard, ASK_QINGLAN_CARD) => build_ask(
            obs,
            "gj_qinglan_card",
            ChoiceKind::SelectCards,
            hand_and_equips(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::SelectCard, ASK_YUEHUA_CONFIRM) => build_ask(
            obs,
            "gj_yuehua_confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            false,
        ),
        (HostStage::SelectCard, ASK_XLQIANHUAN_CONFIRM) => build_ask(
            obs,
            "gj_xlqianhuan_confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            false,
        ),
        (HostStage::SelectCard, ASK_MEIHU_CONFIRM) => build_ask(
            obs,
            "gj_meihu_confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            false,
        ),
        (HostStage::TargetChoice, ASK_MEIHU_GIVE) => build_ask(
            target.unwrap_or(obs),
            "gj_meihu_give",
            ChoiceKind::SelectCards,
            core.player(target.unwrap_or(obs))
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect(),
            1,
            1,
            false,
        ),
        (HostStage::SelectCard, ASK_XIDIE_DISCARD) => {
            let max = (core.player(owner).hand.len() as i32 - core.player(owner).hp)
                .max(0)
                .min(3) as usize;
            build_ask(
                obs,
                "gj_xidie_discard",
                ChoiceKind::SelectCards,
                core.player(owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect(),
                1,
                max.max(1),
                true,
            )
        }
        (HostStage::TargetChoice, ASK_FANYIN_TARGET) => build_ask(
            obs,
            "gj_fanyin_target",
            ChoiceKind::SelectPlayers,
            core.alive_players()
                .filter(|&p| fanyin_eligible(core, p))
                .map(ChoiceOption::Player)
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::SelectCard, ASK_FUMO_CARD1) => {
            let hand = &core.player(owner).hand;
            let red_n = hand.iter().filter(|&&c| is_red(catalog, c)).count();
            let blk_n = hand.iter().filter(|&&c| is_black(catalog, c)).count();
            build_ask(
                obs,
                "gj_fumo_card1",
                ChoiceKind::SelectCards,
                hand.iter()
                    .copied()
                    .filter(|&c| {
                        (is_red(catalog, c) && red_n >= 2) || (is_black(catalog, c) && blk_n >= 2)
                    })
                    .map(ChoiceOption::Card)
                    .collect(),
                1,
                1,
                true,
            )
        }
        (HostStage::TargetChoice, ASK_FUMO_CARD2) => {
            let c1 = cards.first().copied()?;
            let c1_red = is_red(catalog, c1);
            build_ask(
                obs,
                "gj_fumo_card2",
                ChoiceKind::SelectCards,
                core.player(owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|&c| c != c1 && is_red(catalog, c) == c1_red)
                    .map(ChoiceOption::Card)
                    .collect(),
                1,
                1,
                true,
            )
        }
        _ => None,
    }
}
