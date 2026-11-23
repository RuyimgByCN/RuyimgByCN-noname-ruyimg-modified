use super::helpers::*;
use super::super::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{damage_data, Ctx, Flow};
use crate::state::GameStateCore;

pub(super) const ASK_CHIZHEN_DISCARD: u32 = 301;
pub(super) const ASK_CHIZHEN_DUEL_TARGET: u32 = 302;
pub(super) const ASK_DANGPING_CARD: u32 = 303;
pub(super) const ASK_DANGPING_TARGET: u32 = 304;
pub(super) const ASK_YUNYIN_CARD: u32 = 305;
pub(super) const ASK_YUNYIN_VCARD: u32 = 306;
pub(super) const ASK_YUNYIN_TARGET: u32 = 307;
pub(super) const ASK_DUHUN_TARGET: u32 = 308;
pub(super) const ASK_DUHUN_COMPARE_OWN: u32 = 309;
pub(super) const ASK_DUHUN_COMPARE_TARGET: u32 = 310;
pub(super) const ASK_XUANNING1_CARD: u32 = 311;
pub(super) const ASK_LIUGUANG_TARGETS: u32 = 312;
pub(super) const ASK_LIUGUANG_RESPOND: u32 = 313;
pub(super) const ASK_YANGMING_CARD: u32 = 314;
pub(super) const ASK_YANGMING2_TARGETS: u32 = 315;

pub(super) const YUNYIN_VCARDS: [CardKind; 8] = [
    CardKind::Sha,
    CardKind::Tao,
    CardKind::Jiu,
    CardKind::Wuzhong,
    CardKind::Taoyuan,
    CardKind::SavageAttack,
    CardKind::ArcheryAttack,
    CardKind::Duel,
];

pub(in crate::content::host) fn trigger_meta_char_c(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events: Vec::new(),
    };
    match name {
        "shishui" => Some(meta(EventKind::CardUsed, Timing::Normal, 0)),
        "chizhen" => Some(TriggerMeta {
            optional: true,
            ..meta(EventKind::PhaseBegin, Timing::Normal, 0)
        }),
        "shahai-gujian__ref_dangping" => Some(meta(EventKind::DamageSource, Timing::Normal, 0)),
        "yunyin" => Some(TriggerMeta {
            event: EventKind::TurnPhaseEnd,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: false,
            extra_events: vec![EventKind::TurnPhaseAfter],
        }),
        "duhun" => Some(meta(EventKind::Dying, Timing::Normal, 0)),
        "shahai-gujian__ref_xuanning" | "xuanning2" => Some(meta(EventKind::Damage, Timing::Normal, 0)),
        "xuanning1" | "yangming" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 0)),
        "liuguang" => Some(meta(EventKind::TurnBegin, Timing::Normal, 0)),
        "yangming2" => Some(meta(EventKind::PhaseEnd, Timing::Normal, 0)),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_char_c(name: &str) -> SkillLimits {
    let clean = name.rsplit_once("__ref_").map(|(_, s)| s).unwrap_or(name);
    match clean {
        "xuanning" | "xuanning1" | "yangming" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn activate_char_c(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let clean = skill.name.rsplit_once("__ref_").map(|(_, s)| s).unwrap_or(&skill.name);
    match clean {
        "xuanning" | "xuanning1" => {
            let has_basic = ctx
                .state
                .player(owner)
                .hand
                .iter()
                .any(|&c| ctx.catalog.card_kind(c).is_basic_kind());
            if !has_basic
                || flag(ctx.state, owner, "gj_xuanning") >= 3
                || flag(ctx.state, owner, "gj_xuanning1_used_turn") > 0
            {
                return None;
            }
            Some(Box::new(HostMachine {
                def_key: "xuanning1".to_string(),
                behavior: HostBehavior::U05GujianCharBatchC,
                owner,
                target: None,
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }))
        }
        "yangming" => {
            if flag(ctx.state, owner, "gj_yangming2") > 0
                || !ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&c| is_red(ctx.catalog, c))
            {
                return None;
            }
            Some(Box::new(HostMachine {
                def_key: skill.name.clone(),
                behavior: HostBehavior::U05GujianCharBatchC,
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

pub(in crate::content::host) fn filter_char_c(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    let clean = skill.name.rsplit_once("__ref_").map(|(_, s)| s).unwrap_or(&skill.name);
    match clean {
        "shishui" => match event {
            EventData::CardUsed(u) if !u.cancelled && u.player == owner && !u.targets.is_empty() => {
                u.card.is_some_and(|c| is_red(ctx.catalog, c))
            }
            _ => false,
        },
        "chizhen" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } if *player == owner
        ),
        "dangping" => match event {
            EventData::Damage(d) => {
                d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && flag(ctx.state, owner, "gj_dangping_used_turn") == 0
                    && !hand_and_equips(ctx.state, owner).is_empty()
                    && !dangping_targets(ctx.state, ctx.catalog, owner, d.target).is_empty()
            }
            _ => false,
        },
        "yunyin" => match event {
            EventData::TurnEndWindow { player, after: false } if *player == owner => {
                !yunyin_discardable_cards(ctx.state, ctx.catalog, owner).is_empty()
            }
            EventData::TurnEndWindow { player, after: true } if *player == owner => {
                flag(ctx.state, owner, "gj_yunyin_suit_mask") > 0
            }
            _ => false,
        },
        "xuanning" | "xuanning2" => match event {
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && flag(ctx.state, owner, "gj_xuanning") > 0
                    && d.source
                        .is_some_and(|s| ctx.state.player(s).alive && !hand_and_equips(ctx.state, s).is_empty())
            }
            _ => false,
        },
        "liuguang" => {
            matches!(event, EventData::TurnBegin { player } if *player == owner)
                && flag(ctx.state, owner, "gj_xuanning") > 0
                && ctx.state.alive_players().any(|p| p != owner)
        }
        "yangming2" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: false,
            } if *player == owner
        ),
        _ => false,
    }
}

pub(super) fn dangping_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    injured: PlayerId,
) -> Vec<PlayerId> {
    core.alive_players()
        .filter(|&p| {
            p != owner && p != injured && core.computed_distance(catalog, injured, p) <= 1
        })
        .collect()
}

pub(super) fn yunyin_discardable_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let mask = flag(core, owner, "gj_yunyin_suit_mask");
    core.player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            let bit = 1u32 << (catalog.cards[c as usize].suit as u32);
            (mask & bit) == 0
        })
        .collect()
}

pub(super) fn yunyin_vcard_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    kind: CardKind,
) -> Vec<PlayerId> {
    match kind {
        CardKind::Sha => core
            .alive_players()
            .filter(|&p| p != owner && can_target_card(core, catalog, owner, p, CardKind::Sha))
            .collect(),
        CardKind::Tao => {
            if core.player(owner).hp < core.player(owner).max_hp as i32 {
                vec![owner]
            } else {
                Vec::new()
            }
        }
        CardKind::Jiu => {
            if !core.turn.jiu_used {
                vec![owner]
            } else {
                Vec::new()
            }
        }
        CardKind::Wuzhong => vec![owner],
        CardKind::Taoyuan => core.alive_players().collect(),
        CardKind::SavageAttack | CardKind::ArcheryAttack => {
            core.alive_players().filter(|&p| p != owner).collect()
        }
        CardKind::Duel => core
            .alive_players()
            .filter(|&p| p != owner && can_target_card(core, catalog, owner, p, CardKind::Duel))
            .collect(),
        _ => Vec::new(),
    }
}

pub(super) fn duhun_candidates(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    let max_hp = core.player(owner).max_hp as i32;
    if max_hp <= 1 || core.player(owner).hand.is_empty() {
        return Vec::new();
    }
    core.alive_players()
        .filter(|&p| {
            let pp = core.player(p);
            p != owner && !pp.hand.is_empty() && pp.hp > 0 && pp.hp <= max_hp
        })
        .collect()
}

pub(super) fn yangming2_targets(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Vec<PlayerId> {
    core.alive_players()
        .filter(|&p| {
            core.player(p).hp < core.player(p).max_hp as i32
                && core.computed_distance(catalog, owner, p) <= 1
        })
        .collect()
}

pub(in crate::content::host) fn machine_char_c(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let target = match event {
        EventData::Damage(d) => Some(d.source.unwrap_or(d.target)),
        EventData::TurnEndWindow { player, after: true } => Some(*player),
        _ => None,
    };
    let card = match event {
        EventData::Damage(d) => Some(d.target as u32),
        _ => None,
    };
    let targets = match event {
        EventData::CardUsed(u) => u.targets.clone(),
        _ => Vec::new(),
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::U05GujianCharBatchC,
        owner,
        target,
        card,
        cards: Vec::new(),
        targets,
        stage: HostStage::Act,
    }
}

pub(super) fn step_char_c(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let clean = machine
        .def_key
        .rsplit_once("__ref_")
        .map(|(_, s)| s)
        .unwrap_or(&machine.def_key)
        .to_string();
    match clean.as_str() {
        "shishui" => {
            while !machine.targets.is_empty() {
                let t = machine.targets.remove(0);
                if ctx.core.player(t).alive {
                    ctx.emit(EventData::LoseHp {
                        player: t,
                        amount: 1,
                    });
                    if !machine.targets.is_empty() {
                        return Flow::Yield;
                    }
                }
            }
            Flow::Done
        }
        "chizhen" => step_chizhen(machine, ctx),
        "dangping" => step_dangping(machine, ctx),
        "yunyin" => step_yunyin(machine, ctx),
        "duhun" => super::chars_c2::step_duhun(machine, ctx),
        "xuanning1" => super::chars_c2::step_xuanning1(machine, ctx),
        "xuanning" | "xuanning2" => {
            let count = flag(ctx.core, machine.owner, "gj_xuanning");
            if count == 0 {
                return Flow::Done;
            }
            if let Some(src) = machine.target {
                let he = hand_and_equips(ctx.core, src);
                if !he.is_empty() {
                    let ci = ctx.core.rng.gen_range(he.len() as u32) as usize;
                    discard_card_from_he(ctx, src, he[ci]);
                }
            }
            set_flag(ctx.core, machine.owner, "gj_xuanning", count - 1);
            Flow::Done
        }
        "liuguang" => super::chars_c2::step_liuguang(machine, ctx),
        "yangming" => super::chars_c2::step_yangming(machine, ctx),
        "yangming2" => super::chars_c2::step_yangming2(machine, ctx),
        _ => Flow::Done,
    }
}

fn step_chizhen(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            let p = ctx.core.player(machine.owner);
            let n = (p.max_hp as i32 - p.hp).max(1) as u32;
            machine.card = Some(n);
            ctx.start_draw(machine.owner, n);
            machine.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let n = machine.card.unwrap_or(1) as usize;
            let opts: Vec<ChoiceOption> = hand_and_equips(ctx.core, machine.owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            if opts.is_empty() {
                return Flow::Done;
            }
            let cnt = n.min(opts.len());
            wait_ask(
                machine,
                ctx,
                HostStage::SelectCard,
                machine.owner,
                ASK_CHIZHEN_DISCARD,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_chizhen_discard",
                    options: opts,
                    min: cnt,
                    max: cnt,
                    allow_cancel: false,
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
            let discarded_sha = picked
                .iter()
                .any(|&c| ctx.catalog.card_kind(c).is_sha_like());
            discard_cards_from_he(ctx, machine.owner, &picked);
            if !discarded_sha {
                return Flow::Done;
            }
            let opts: Vec<ChoiceOption> = ctx
                .core
                .alive_players()
                .filter(|&p| {
                    p != machine.owner && can_target_card(ctx.core, ctx.catalog, machine.owner, p, CardKind::Duel)
                })
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
                ASK_CHIZHEN_DUEL_TARGET,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gj_chizhen_duel_target",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            )
        }
        HostStage::TargetChoice => {
            if let ChoiceOption::Player(t) = answer_first_or_pass(ctx) {
                if ctx.core.player(t).alive {
                    ctx.emit(EventData::CardUsed(card_use_virtual_with_nature(
                        machine.owner,
                        vec![t],
                        DamageNature::Normal,
                        CardKind::Duel,
                    )));
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dangping(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let injured = machine.card.unwrap_or(0) as PlayerId;
    match machine.stage {
        HostStage::Act => {
            let opts: Vec<ChoiceOption> = hand_and_equips(ctx.core, machine.owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            let cands = dangping_targets(ctx.core, ctx.catalog, machine.owner, injured);
            if opts.is_empty() || cands.is_empty() {
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::SelectCard,
                machine.owner,
                ASK_DANGPING_CARD,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_dangping_card",
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
            machine.cards = vec![cid];
            let opts: Vec<ChoiceOption> = dangping_targets(ctx.core, ctx.catalog, machine.owner, injured)
                .into_iter()
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
                ASK_DANGPING_TARGET,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gj_dangping_target",
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
            let Some(cid) = machine.cards.first().copied() else {
                return Flow::Done;
            };
            if discard_card_from_he(ctx, machine.owner, cid) {
                set_flag(ctx.core, machine.owner, "gj_dangping_used_turn", 1);
                if ctx.core.player(t).alive {
                    ctx.emit(EventData::Damage(damage_data(Some(machine.owner), t, 1)));
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_yunyin(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if machine.target.is_some() {
        set_flag(ctx.core, machine.owner, "gj_yunyin_suit_mask", 0);
        return Flow::Done;
    }
    if ctx.core.turn.current != machine.owner {
        set_flag(ctx.core, machine.owner, "gj_yunyin_suit_mask", 0);
        return Flow::Done;
    }
    match machine.stage {
        HostStage::Act => {
            let opts: Vec<ChoiceOption> = yunyin_discardable_cards(ctx.core, ctx.catalog, machine.owner)
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
                ASK_YUNYIN_CARD,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_yunyin_card",
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
            if !discard_card_from_he(ctx, machine.owner, cid) {
                return Flow::Done;
            }
            let opts: Vec<ChoiceOption> = YUNYIN_VCARDS
                .iter()
                .enumerate()
                .filter_map(|(i, &k)| {
                    if yunyin_vcard_targets(ctx.core, ctx.catalog, machine.owner, k).is_empty() {
                        None
                    } else {
                        Some(ChoiceOption::Tag(i as u8))
                    }
                })
                .collect();
            if opts.is_empty() {
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::TargetChoice,
                machine.owner,
                ASK_YUNYIN_VCARD,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "gj_yunyin_vcard",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            )
        }
        HostStage::TargetChoice => {
            let ChoiceOption::Tag(idx) = answer_first_or_pass(ctx) else {
                return Flow::Done;
            };
            let kind = YUNYIN_VCARDS.get(idx as usize).copied().unwrap_or(CardKind::Wuzhong);
            machine.card = Some(idx as u32);
            let targs = yunyin_vcard_targets(ctx.core, ctx.catalog, machine.owner, kind);
            if targs.is_empty() {
                return Flow::Done;
            }
            if matches!(
                kind,
                CardKind::Tao
                    | CardKind::Jiu
                    | CardKind::Wuzhong
                    | CardKind::Taoyuan
                    | CardKind::SavageAttack
                    | CardKind::ArcheryAttack
            ) {
                ctx.emit(EventData::CardUsed(card_use_virtual_with_nature(
                    machine.owner,
                    targs,
                    DamageNature::Normal,
                    kind,
                )));
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::DrawSettle,
                machine.owner,
                ASK_YUNYIN_TARGET,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gj_yunyin_target",
                    options: targs.into_iter().map(ChoiceOption::Player).collect(),
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            )
        }
        HostStage::DrawSettle => {
            let ChoiceOption::Player(t) = answer_first_or_pass(ctx) else {
                return Flow::Done;
            };
            let kind = YUNYIN_VCARDS
                .get(machine.card.unwrap_or(0) as usize)
                .copied()
                .unwrap_or(CardKind::Sha);
            ctx.emit(EventData::CardUsed(card_use_virtual_with_nature(
                machine.owner,
                vec![t],
                DamageNature::Normal,
                kind,
            )));
            Flow::Done
        }
        _ => Flow::Done,
    }
}

