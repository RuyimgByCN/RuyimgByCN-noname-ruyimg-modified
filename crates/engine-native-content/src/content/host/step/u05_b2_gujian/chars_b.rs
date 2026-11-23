use super::helpers::*;
use super::super::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::event::{DamageNature, DetachedKind, EventData, EventKind, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

const ASK_ZUIJI_CARD: u32 = 201;
const ASK_MANWU_CONFIRM: u32 = 202;
const ASK_XFANGHUA_CONFIRM: u32 = 203;
const ASK_ZUIZHAN_CONFIRM: u32 = 204;
const ASK_MEIYING_DISCARD: u32 = 205;
const ASK_YANJIA_CARD1: u32 = 206;
const ASK_YANJIA_CARD2: u32 = 207;
const ASK_XIUHUA_GAIN: u32 = 208;
const ASK_LIUYING_TARGET: u32 = 209;

pub(in crate::content::host) fn trigger_meta_char_b(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events: Vec::new(),
    };
    match name {
        "zuiji" | "yanjia" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 0)),
        "manwu" => Some(meta(EventKind::TurnPhaseEnd, Timing::Normal, 0)),
        "xfanghua" => Some(meta(EventKind::CardUsed, Timing::Normal, -1)),
        "jianwu" => Some(meta(EventKind::ShaBegin, Timing::Normal, 0)),
        "zuizhan" => Some(meta(EventKind::CardUsed, Timing::Normal, 0)),
        "meiying" => Some(meta(EventKind::TurnPhaseEnd, Timing::Normal, 0)),
        "xiuhua" => Some(TriggerMeta {
            event: EventKind::CardsDiscarded,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: false,
            extra_events: vec![EventKind::Equipped],
        }),
        "liuying" => Some(TriggerMeta {
            event: EventKind::CardUsed,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: false,
            extra_events: vec![EventKind::TurnPhaseAfter],
        }),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_char_b(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn activate_char_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    match skill.name.as_str() {
        "zuiji" => {
            if ctx.state.turn.current != owner
                || hand_and_equips(ctx.state, owner).is_empty()
                || ctx.state.turn.jiu_used
            {
                return None;
            }
            Some(Box::new(HostMachine {
                def_key: skill.name.clone(),
                behavior: HostBehavior::U05GujianCharBatchB,
                owner,
                target: None,
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }))
        }
        "yanjia" => {
            let eqs = yanjia_eligible_equips(ctx.state, ctx.catalog, owner);
            let has_distinct = eqs.iter().any(|&a| {
                eqs.iter()
                    .any(|&b| ctx.catalog.cards[a as usize].name != ctx.catalog.cards[b as usize].name)
            });
            if ctx.state.turn.current != owner || !has_distinct {
                return None;
            }
            Some(Box::new(HostMachine {
                def_key: skill.name.clone(),
                behavior: HostBehavior::U05GujianCharBatchB,
                owner,
                target: None,
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }))
        }
        _ => None,
    }
}

pub(in crate::content::host) fn filter_char_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "manwu" => match event {
            EventData::TurnEndWindow { player, after: false } if ctx.state.player(*player).alive => {
                let h = ctx.state.player(*player).hand.len();
                ctx.state.alive_players().all(|q| ctx.state.player(q).hand.len() >= h)
            }
            _ => false,
        },
        "xfanghua" => match event {
            EventData::CardUsed(u) if !u.cancelled && u.targets.contains(&owner) => {
                ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
                    && u.suit_in(ctx.state, ctx.catalog).is_some_and(|s| {
                        matches!(s, crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond)
                    })
            }
            _ => false,
        },
        "zuizhan" => match event {
            EventData::CardUsed(u) if !u.cancelled && u.player == owner => {
                let kind = u.effective_kind_in(ctx.state, ctx.catalog);
                kind.is_sha_like()
                    && ctx.state.alive_players().any(|p| {
                        p != owner
                            && !u.targets.contains(&p)
                            && can_target_card(ctx.state, ctx.catalog, owner, p, CardKind::Sha)
                    })
            }
            _ => false,
        },
        "meiying" => match event {
            EventData::TurnEndWindow { player, after: false } if *player != owner && ctx.state.player(*player).alive => {
                flag(ctx.state, *player, "gj_turn_targeted_other") == 0
                    && hand_and_equips(ctx.state, owner)
                        .into_iter()
                        .any(|c| is_red(ctx.catalog, c))
            }
            _ => false,
        },
        "xiuhua" => match event {
            EventData::CardsDiscarded { player, cards } if *player != owner => cards.iter().any(|&c| {
                ctx.catalog.card_kind(c).equip_slot().is_some() && ctx.state.discard.contains(&c)
            }),
            _ => false,
        },
        "liuying" => match event {
            EventData::CardUsed(u) if !u.cancelled && u.player == owner => {
                u.effective_kind_in(ctx.state, ctx.catalog).is_sha_like()
                    && !liuying_candidates(ctx.state, ctx.catalog, owner).is_empty()
            }
            EventData::TurnEndWindow { player, after: true } if *player == owner => {
                ctx.state
                    .players
                    .iter()
                    .enumerate()
                    .any(|(i, _)| flag(ctx.state, owner, &format!("gj_liuying_hit:{i}")) > 0)
            }
            _ => false,
        },
        _ => false,
    }
}

pub(in crate::content::host) fn machine_char_b(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let (target, cards) = match event {
        EventData::TurnEndWindow { player, .. } => (Some(*player), Vec::new()),
        EventData::CardsDiscarded { player, cards } => (Some(*player), cards.clone()),
        EventData::CardUsed(u) => (Some(u.player), u.card.into_iter().collect()),
        _ => (None, Vec::new()),
    };
    let stage = if matches!(event, EventData::TurnEndWindow { after: true, .. }) {
        HostStage::ShenmiguoRestage
    } else {
        HostStage::Act
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::U05GujianCharBatchB,
        owner,
        target,
        card: None,
        cards,
        targets: Vec::new(),
        stage,
    }
}

pub(super) fn yanjia_eligible_equips(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Vec<CardId> {
    hand_and_equips(core, owner)
        .into_iter()
        .filter(|&c| {
            let def = &catalog.cards[c as usize];
            def.kind.equip_slot().is_some()
                && !def.off_deck
                && !matches!(
                    def.kind,
                    CardKind::Gjyuheng
                        | CardKind::Donghuangzhong
                        | CardKind::Xuanyuanjian
                        | CardKind::Lianyaohu
                        | CardKind::Haotianta
                        | CardKind::Shennongding
                        | CardKind::Kongdongyin
                        | CardKind::Kunlunjingc
                        | CardKind::Nvwashi
                )
                && flag(core, 0, &format!("gj_yanjia_merged:{c}")) == 0
        })
        .collect()
}

pub(super) fn liuying_candidates(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Vec<PlayerId> {
    core.alive_players()
        .filter(|&p| {
            p != owner
                && flag(core, owner, &format!("gj_liuying_hit:{p}")) == 0
                && can_target_card(core, catalog, owner, p, CardKind::Sha)
        })
        .collect()
}

pub(super) fn step_char_b(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.def_key.as_str() {
        "zuiji" => step_zuiji(machine, ctx),
        "manwu" => step_manwu(machine, ctx),
        "xfanghua" => step_xfanghua(machine, ctx),
        "zuizhan" => step_zuizhan(machine, ctx),
        "meiying" => step_meiying(machine, ctx),
        "yanjia" => step_yanjia(machine, ctx),
        "xiuhua" => step_xiuhua(machine, ctx),
        "liuying" => step_liuying(machine, ctx),
        _ => Flow::Done,
    }
}

fn step_zuiji(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            let opts: Vec<ChoiceOption> = hand_and_equips(ctx.core, machine.owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            if opts.is_empty() || ctx.core.turn.jiu_used {
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::SelectCard,
                machine.owner,
                ASK_ZUIJI_CARD,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_zuiji_card",
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
            let has_card = ctx.core.player(machine.owner).hand.contains(&cid)
                || ctx.core.player(machine.owner).equips.contains(&cid);
            if has_card && !ctx.core.turn.jiu_used {
                let use_ev = crate::machine::card_use_with_nature(
                    cid,
                    machine.owner,
                    vec![machine.owner],
                    DamageNature::Normal,
                    Some(CardKind::Jiu),
                );
                ctx.emit(EventData::CardUsed(use_ev));
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_manwu(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = machine.target else {
        return Flow::Done;
    };
    match machine.stage {
        HostStage::Act => {
            if !ctx.core.player(t).alive {
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::SelectCard,
                machine.owner,
                ASK_MANWU_CONFIRM,
                ChoiceSpec {
                    kind: ChoiceKind::Confirm,
                    prompt: "gj_manwu_confirm",
                    options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            )
        }
        HostStage::SelectCard => {
            if matches!(answer_first_or_pass(ctx), ChoiceOption::Confirm) && ctx.core.player(t).alive {
                ctx.start_draw(t, 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_xfanghua(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            if ctx.core.player(machine.owner).hp >= ctx.core.player(machine.owner).max_hp as i32 {
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::SelectCard,
                machine.owner,
                ASK_XFANGHUA_CONFIRM,
                ChoiceSpec {
                    kind: ChoiceKind::Confirm,
                    prompt: "gj_xfanghua_confirm",
                    options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            )
        }
        HostStage::SelectCard => {
            if matches!(answer_first_or_pass(ctx), ChoiceOption::Confirm) {
                ctx.recover(machine.owner, 1, None);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_zuizhan(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => wait_ask(
            machine,
            ctx,
            HostStage::SelectCard,
            machine.owner,
            ASK_ZUIZHAN_CONFIRM,
            ChoiceSpec {
                kind: ChoiceKind::Confirm,
                prompt: "gj_zuizhan_confirm",
                options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        ),
        HostStage::SelectCard => {
            if !matches!(answer_first_or_pass(ctx), ChoiceOption::Confirm) {
                return Flow::Done;
            }
            let existing = ctx
                .current_card_use()
                .map(|u| u.targets.clone())
                .unwrap_or_default();
            let cands: Vec<PlayerId> = ctx
                .core
                .alive_players()
                .filter(|&p| {
                    p != machine.owner
                        && !existing.contains(&p)
                        && can_target_card(ctx.core, ctx.catalog, machine.owner, p, CardKind::Sha)
                })
                .collect();
            if !cands.is_empty() {
                let idx = ctx.core.rng.gen_range(cands.len() as u32) as usize;
                let extra = cands[idx];
                ctx.current_card_use_mut().targets.push(extra);
                ctx.start_draw(machine.owner, 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_meiying(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = machine.target else {
        return Flow::Done;
    };
    match machine.stage {
        HostStage::Act => {
            if !ctx.core.player(t).alive {
                return Flow::Done;
            }
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
                ASK_MEIYING_DISCARD,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_meiying_discard",
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
            if discard_card_from_he(ctx, machine.owner, cid) && ctx.core.player(t).alive {
                spawn_virtual_sha(ctx, machine.owner, vec![t], DamageNature::Normal);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_yanjia(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            let opts: Vec<ChoiceOption> = yanjia_eligible_equips(ctx.core, ctx.catalog, machine.owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            if opts.len() < 2 {
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::SelectCard,
                machine.owner,
                ASK_YANJIA_CARD1,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_yanjia_card1",
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
            let n1 = ctx.catalog.cards[c1 as usize].name;
            let opts: Vec<ChoiceOption> = yanjia_eligible_equips(ctx.core, ctx.catalog, machine.owner)
                .into_iter()
                .filter(|&c| c != c1 && ctx.catalog.cards[c as usize].name != n1)
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
                ASK_YANJIA_CARD2,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_yanjia_card2",
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
            for c in [c1, c2] {
                if ctx.core.player(machine.owner).equips.contains(&c) {
                    ctx.core.player_mut(machine.owner).equips.retain(|x| *x != c);
                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, machine.owner, c);
                }
                ctx.core.player_mut(machine.owner).hand.retain(|x| *x != c);
            }
            ctx.core.detached_cards.push((c2, DetachedKind::Unclaimed));
            set_flag(ctx.core, 0, &format!("gj_yanjia_merged:{c1}"), c2 + 1);
            ctx.core.player_mut(machine.owner).hand.push(c1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_xiuhua(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let equips: Vec<CardId> = machine
        .cards
        .iter()
        .copied()
        .filter(|&c| ctx.catalog.card_kind(c).equip_slot().is_some() && ctx.core.discard.contains(&c))
        .collect();
    if equips.is_empty() {
        return Flow::Done;
    }
    match machine.stage {
        HostStage::Act => wait_ask(
            machine,
            ctx,
            HostStage::SelectCard,
            machine.owner,
            ASK_XIUHUA_GAIN,
            ChoiceSpec {
                kind: ChoiceKind::Confirm,
                prompt: "gj_xiuhua_gain",
                options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        ),
        HostStage::SelectCard => {
            if matches!(answer_first_or_pass(ctx), ChoiceOption::Confirm) {
                for &c in &equips {
                    ctx.core.discard.retain(|x| *x != c);
                    ctx.core.player_mut(machine.owner).hand.push(c);
                }
                ctx.emit(EventData::CardsDrawn {
                    player: machine.owner,
                    cards: equips,
                });
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_liuying(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if matches!(machine.stage, HostStage::ShenmiguoRestage) {
        // Clear per-turn hit list on owner TurnEndWindow { after: true }
        let seats = ctx.core.players.len();
        for i in 0..seats {
            set_flag(ctx.core, machine.owner, &format!("gj_liuying_hit:{i}"), 0);
        }
        return Flow::Done;
    }
    match machine.stage {
        HostStage::Act => {
            let cands = liuying_candidates(ctx.core, ctx.catalog, machine.owner);
            if cands.is_empty() {
                return Flow::Done;
            }
            let opts = cands.into_iter().map(ChoiceOption::Player).collect();
            wait_ask(
                machine,
                ctx,
                HostStage::TargetChoice,
                machine.owner,
                ASK_LIUYING_TARGET,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gj_liuying_target",
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
            if let Some(top) = ctx.core.deck.pop() {
                let black = is_black(ctx.catalog, top);
                ctx.to_discard(top);
                if black && ctx.core.player(t).alive {
                    spawn_virtual_sha(ctx, machine.owner, vec![t], DamageNature::Normal);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn ask_char_b(
    stage: HostStage,
    owner: PlayerId,
    _target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<crate::snap::ExpectedAsk> {
    let ask_id = current_ask_id(core, owner, HostBehavior::U05GujianCharBatchB);
    let obs = current_ask_obs(core, owner, HostBehavior::U05GujianCharBatchB);
    match (stage, ask_id) {
        (HostStage::SelectCard, ASK_ZUIJI_CARD) => build_ask(
            obs,
            "gj_zuiji_card",
            ChoiceKind::SelectCards,
            hand_and_equips(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::SelectCard, ASK_MANWU_CONFIRM) => build_ask(
            obs,
            "gj_manwu_confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            false,
        ),
        (HostStage::SelectCard, ASK_XFANGHUA_CONFIRM) => build_ask(
            obs,
            "gj_xfanghua_confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            false,
        ),
        (HostStage::SelectCard, ASK_ZUIZHAN_CONFIRM) => build_ask(
            obs,
            "gj_zuizhan_confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            false,
        ),
        (HostStage::SelectCard, ASK_MEIYING_DISCARD) => build_ask(
            obs,
            "gj_meiying_discard",
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
        (HostStage::SelectCard, ASK_YANJIA_CARD1) => build_ask(
            obs,
            "gj_yanjia_card1",
            ChoiceKind::SelectCards,
            yanjia_eligible_equips(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::TargetChoice, ASK_YANJIA_CARD2) => {
            let c1 = cards.first().copied()?;
            let n1 = catalog.cards[c1 as usize].name;
            build_ask(
                obs,
                "gj_yanjia_card2",
                ChoiceKind::SelectCards,
                yanjia_eligible_equips(core, catalog, owner)
                    .into_iter()
                    .filter(|&c| c != c1 && catalog.cards[c as usize].name != n1)
                    .map(ChoiceOption::Card)
                    .collect(),
                1,
                1,
                true,
            )
        }
        (HostStage::SelectCard, ASK_XIUHUA_GAIN) => build_ask(
            obs,
            "gj_xiuhua_gain",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            false,
        ),
        (HostStage::TargetChoice, ASK_LIUYING_TARGET) => build_ask(
            obs,
            "gj_liuying_target",
            ChoiceKind::SelectPlayers,
            liuying_candidates(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect(),
            1,
            1,
            true,
        ),
        _ => None,
    }
}
