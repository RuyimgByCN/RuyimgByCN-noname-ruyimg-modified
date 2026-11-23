use super::helpers::*;
use super::super::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::event::{DamageNature, EventData, EventKind, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::{GameStateCore, SkillInstance};

pub(super) const ASK_LINGYAN_CARD: u32 = 501;
pub(super) const ASK_WOXUE_CONFIRM: u32 = 502;
pub(super) const ASK_YANGE_TARGET: u32 = 503;
pub(super) const ASK_LIANJING_TARGETS: u32 = 504;
pub(super) const ASK_CIHONG_TARGET: u32 = 505;
pub(super) const ASK_CIHONG_RED: u32 = 506;
pub(super) const ASK_CIHONG_LOSEHP: u32 = 507;
pub(super) const ASK_CIHONG_TURNOVER: u32 = 508;
pub(super) const ASK_XUANCI_CARD: u32 = 509;
pub(super) const ASK_XUANCI_TARGET: u32 = 510;

pub(in crate::content::host) fn trigger_meta_char_e1(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "lingyan" | "xuanci" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, Vec::new())),
        "xunjian" => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            0,
            vec![EventKind::CardResponded],
        )),
        "humeng" => Some(meta(EventKind::Dying, Timing::Normal, 10, Vec::new())),
        "xianju" => Some(meta(EventKind::TurnPhaseEnd, Timing::Normal, 0, Vec::new())),
        "woxue" => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            0,
            vec![EventKind::CardResponded],
        )),
        "yange" => Some(meta(EventKind::TurnBegin, Timing::Normal, 0, Vec::new())),
        "lianjing" => Some(meta(EventKind::TurnPhaseAfter, Timing::Normal, 0, Vec::new())),
        "cihong" => Some(meta(EventKind::TurnPhaseEnd, Timing::Normal, 0, Vec::new())),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_char_e1(name: &str) -> SkillLimits {
    match name {
        "lingyan" | "xuanci" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn activate_char_e1(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    match skill.name.as_str() {
        "lingyan" => {
            if lingyan_eligible_cards(ctx.state, ctx.catalog, owner).is_empty() {
                return None;
            }
            Some(Box::new(HostMachine {
                def_key: skill.name.clone(),
                behavior: HostBehavior::U05GujianCharBatchE,
                owner,
                target: None,
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }))
        }
        "xuanci" => {
            if hand_and_equips(ctx.state, owner)
                .into_iter()
                .all(|c| ctx.catalog.cards[c as usize].suit != crate::catalog::Suit::Club)
            {
                return None;
            }
            if !ctx.state.alive_players().any(|p| p != owner) {
                return None;
            }
            Some(Box::new(HostMachine {
                def_key: skill.name.clone(),
                behavior: HostBehavior::U05GujianCharBatchE,
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

pub(in crate::content::host) fn filter_char_e1(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "xunjian" => {
            let n_ly = ctx.state.player(owner).off_court.len();
            if n_ly == 0 {
                return false;
            }
            let card_id = match event {
                EventData::CardUsed(u) if u.player == owner && !u.cancelled => u.card,
                EventData::CardResponded { player, card } if *player == owner => Some(*card),
                _ => None,
            };
            card_id.is_some_and(|cid| {
                let cname = ctx.catalog.cards[cid as usize].name;
                ctx.state
                    .deck
                    .iter()
                    .any(|&d| ctx.catalog.cards[d as usize].name == cname)
            })
        }
        "humeng" => {
            matches!(event, EventData::Dying { player, .. } if *player == owner)
                && flag(ctx.state, owner, "gj_humeng_done") == 0
        }
        "xianju" => false,
        "woxue" => {
            if ctx.state.turn.current == owner {
                return false;
            }
            let matched = match event {
                EventData::CardUsed(u) if u.player == owner && !u.cancelled => true,
                EventData::CardResponded { player, .. } if *player == owner => true,
                _ => false,
            };
            let cur = ctx.state.turn.current;
            matched
                && ctx.state.player(cur).alive
                && !has_skill(ctx.state, ctx.catalog, cur, GW_CIGU_STATE_KEY)
                && !has_skill(ctx.state, ctx.catalog, cur, "gw_ciguhanshuang")
        }
        "yange" => {
            matches!(event, EventData::TurnBegin { player } if *player == owner)
                && ctx.state.alive_players().any(|p| p != owner)
                && if flag(ctx.state, owner, "gj_yange_inited") == 0 {
                    ctx.state
                        .alive_players()
                        .any(|p| p != owner && !ctx.state.is_friend_of(ctx.catalog, owner, p))
                } else {
                    flag(ctx.state, owner, "gj_yange_marks") > 0
                }
        }
        "lianjing" => {
            matches!(event, EventData::TurnEndWindow { player, after: true } if *player == owner)
                && flag(ctx.state, 0, "gj_lianjing_len") == 0
                && ctx.state.turn.round >= flag(ctx.state, owner, "gj_lianjing_ready_round")
                && ctx.state.alive_players().any(|p| p != owner)
        }
        "cihong" => {
            matches!(event, EventData::TurnEndWindow { player, after: false } if *player == owner)
                && ctx.state.turn.round >= flag(ctx.state, owner, "gj_cihong_ready_round")
                && ctx.state.alive_players().any(|p| p != owner)
        }
        _ => false,
    }
}

pub(super) fn lingyan_eligible_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    let used_ranks: Vec<u8> = core
        .player(owner)
        .off_court
        .iter()
        .map(|&c| catalog.cards[c as usize].rank)
        .collect();
    core.player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&c| !used_ranks.contains(&catalog.cards[c as usize].rank))
        .collect()
}

pub(super) fn step_char_e1(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.def_key.as_str() {
        "lingyan" => step_lingyan(machine, ctx),
        "xunjian" => step_xunjian(machine, ctx),
        "humeng" => step_humeng(machine, ctx),
        "xianju" => step_xianju_gain(machine.owner, ctx),
        "xuanci" => step_xuanci(machine, ctx),
        "woxue" => step_woxue(machine, ctx),
        "yange" => step_yange(machine, ctx),
        "lianjing" => step_lianjing(machine, ctx),
        "cihong" => step_cihong(machine, ctx),
        _ => Flow::Done,
    }
}

pub(super) fn step_xianju_gain(owner: PlayerId, ctx: &mut Ctx<'_>) -> Flow {
    if ctx.core.turn.round % 2 != 0 || !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    let mut candidates: Vec<(bool, CardId)> = Vec::new();
    for &c in &ctx.core.deck {
        let def = &ctx.catalog.cards[c as usize];
        if is_jiguan_kind(def.kind) && !def.name.starts_with("yanjiadan_") {
            candidates.push((true, c));
        }
    }
    for &c in &ctx.core.discard {
        let def = &ctx.catalog.cards[c as usize];
        if is_jiguan_kind(def.kind) && !def.name.starts_with("yanjiadan_") {
            candidates.push((false, c));
        }
    }
    if candidates.is_empty() {
        return Flow::Done;
    }
    let pick = ctx.core.rng.gen_range(candidates.len() as u32) as usize;
    let (from_deck, cid) = candidates[pick];
    if from_deck {
        ctx.core.deck.retain(|x| *x != cid);
    } else {
        ctx.core.discard.retain(|x| *x != cid);
    }
    ctx.core.player_mut(owner).hand.push(cid);
    ctx.emit(EventData::CardsDrawn {
        player: owner,
        cards: vec![cid],
    });
    Flow::Done
}

fn step_lingyan(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            let opts: Vec<ChoiceOption> = lingyan_eligible_cards(ctx.core, ctx.catalog, machine.owner)
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
                ASK_LINGYAN_CARD,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_lingyan_card",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            )
        }
        HostStage::SelectCard => {
            let ChoiceOption::Card(cid) = answer_first_or_pass(ctx) else {
                if let Some(def) = ctx.catalog.skill_by_name("lingyan") {
                    if let Some(inst) = ctx.core.skills.get_mut(&(machine.owner, def)) {
                        inst.once_used = false;
                    }
                }
                return Flow::Done;
            };
            ctx.core.player_mut(machine.owner).hand.retain(|x| *x != cid);
            ctx.core.player_mut(machine.owner).off_court.push(cid);
            let suit_suffix = match ctx.catalog.cards[cid as usize].suit {
                crate::catalog::Suit::Heart => "heart",
                crate::catalog::Suit::Diamond => "diamond",
                crate::catalog::Suit::Club => "club",
                crate::catalog::Suit::Spade => "spade",
            };
            let target_name = format!("yanjiadan_{suit_suffix}");
            if let Some(pos) = ctx
                .core
                .detached_cards
                .iter()
                .position(|&(c, _)| ctx.catalog.cards[c as usize].name == target_name)
            {
                let egg = ctx.core.detached_cards.remove(pos).0;
                ctx.core.player_mut(machine.owner).hand.push(egg);
                ctx.emit(EventData::CardsDrawn {
                    player: machine.owner,
                    cards: vec![egg],
                });
            } else if let Some(pos) = ctx
                .core
                .discard
                .iter()
                .position(|&c| ctx.catalog.cards[c as usize].name == target_name)
            {
                let egg = ctx.core.discard.remove(pos);
                ctx.core.player_mut(machine.owner).hand.push(egg);
                ctx.emit(EventData::CardsDrawn {
                    player: machine.owner,
                    cards: vec![egg],
                });
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_xunjian(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(cid) = machine.card else {
        return Flow::Done;
    };
    let n_ly = ctx.core.player(machine.owner).off_court.len() as u32;
    if n_ly == 0 {
        return Flow::Done;
    }
    let roll = ctx.core.rng.gen_range(13);
    if roll >= n_ly {
        return Flow::Done;
    }
    let cname = ctx.catalog.cards[cid as usize].name;
    if let Some(pos) = ctx
        .core
        .deck
        .iter()
        .rposition(|&d| ctx.catalog.cards[d as usize].name == cname)
    {
        let gained = ctx.core.deck.remove(pos);
        ctx.core.player_mut(machine.owner).hand.push(gained);
        ctx.emit(EventData::CardsDrawn {
            player: machine.owner,
            cards: vec![gained],
        });
    }
    Flow::Done
}

fn step_humeng(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    match machine.stage {
        HostStage::Act => {
            if flag(ctx.core, owner, "gj_humeng_done") > 0 {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "gj_humeng_done", 1);
            let hand: Vec<CardId> = ctx.core.player_mut(owner).hand.drain(..).collect();
            let equips: Vec<CardId> = ctx.core.player_mut(owner).equips.drain(..).collect();
            let judge: Vec<CardId> = ctx.core.player_mut(owner).judge.drain(..).collect();
            for &e in &equips {
                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, e);
            }
            for &c in hand.iter().chain(equips.iter()).chain(judge.iter()) {
                ctx.to_discard(c);
            }
            ctx.core.player_mut(owner).linked = false;
            ctx.core.player_mut(owner).face_down = false;
            ctx.start_draw(owner, 4);
            for sk in ["humeng", "lingyan", "xunjian"] {
                if let Some(def) = ctx.catalog.skill_by_name(sk) {
                    ctx.core.skills.remove(&(owner, def));
                }
            }
            if let Some(chuqi) = general_id_by_name(ctx.catalog, "gjqt_chuqi") {
                ctx.core.player_mut(owner).general_slots[0] = Some(chuqi);
            }
            ctx.core.player_mut(owner).max_hp = 2;
            ctx.core.player_mut(owner).hp = 2;
            ctx.core.player_mut(owner).dying = false;
            if let Some(def) = ctx.catalog.skill_by_name("xuanci") {
                ctx.insert_skill(
                    (owner, def),
                    SkillInstance {
                        owner,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
            }
            super::sync_gujian_owner_flags(ctx.core, ctx.catalog, owner);
            if flag(ctx.core, owner, "gj_humeng_sub") > 0 {
                set_flag(ctx.core, owner, "gj_humeng_sub", 0);
                machine.stage = HostStage::DrawSettle;
                return Flow::Yield;
            }
            Flow::Done
        }
        HostStage::DrawSettle => {
            super::subplayer::enter_humeng_subplayer(ctx, owner);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_xuanci(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            let opts: Vec<ChoiceOption> = hand_and_equips(ctx.core, machine.owner)
                .into_iter()
                .filter(|&c| ctx.catalog.cards[c as usize].suit == crate::catalog::Suit::Club)
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
                ASK_XUANCI_CARD,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gj_xuanci_card",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            )
        }
        HostStage::SelectCard => {
            let ChoiceOption::Card(cid) = answer_first_or_pass(ctx) else {
                if let Some(def) = ctx.catalog.skill_by_name("xuanci") {
                    if let Some(inst) = ctx.core.skills.get_mut(&(machine.owner, def)) {
                        inst.once_used = false;
                    }
                }
                return Flow::Done;
            };
            machine.cards = vec![cid];
            let opts: Vec<ChoiceOption> = ctx
                .core
                .alive_players()
                .filter(|&p| {
                    p != machine.owner && can_target_card(ctx.core, ctx.catalog, machine.owner, p, CardKind::Feibiao)
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
                ASK_XUANCI_TARGET,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gj_xuanci_target",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            )
        }
        HostStage::TargetChoice => {
            let ChoiceOption::Player(t) = answer_first_or_pass(ctx) else {
                if let Some(def) = ctx.catalog.skill_by_name("xuanci") {
                    if let Some(inst) = ctx.core.skills.get_mut(&(machine.owner, def)) {
                        inst.once_used = false;
                    }
                }
                return Flow::Done;
            };
            let Some(cid) = machine.cards.first().copied() else {
                return Flow::Done;
            };
            if discard_card_from_he(ctx, machine.owner, cid) && ctx.core.player(t).alive {
                ctx.emit(EventData::CardUsed(card_use_virtual_with_nature(
                    machine.owner,
                    vec![t],
                    DamageNature::Normal,
                    CardKind::Feibiao,
                )));
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_woxue(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let cur = ctx.core.turn.current;
    match machine.stage {
        HostStage::Act => {
            if !ctx.core.player(cur).alive || cur == machine.owner {
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::SelectCard,
                machine.owner,
                ASK_WOXUE_CONFIRM,
                ChoiceSpec {
                    kind: ChoiceKind::Confirm,
                    prompt: "gj_woxue_confirm",
                    options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            )
        }
        HostStage::SelectCard => {
            if matches!(answer_first_or_pass(ctx), ChoiceOption::Confirm) && ctx.core.player(cur).alive {
                for key in [GW_CIGU_STATE_KEY, "gw_ciguhanshuang"] {
                    if let Some(def) = ctx.catalog.skill_by_name(key) {
                        ctx.insert_skill(
                            (cur, def),
                            SkillInstance {
                                owner: cur,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: false,
                                rounds: 0,
                            },
                        );
                        break;
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_yange(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            if flag(ctx.core, machine.owner, "gj_yange_inited") == 0 {
                let enemies = ctx
                    .core
                    .alive_players()
                    .filter(|&p| {
                        p != machine.owner && !ctx.core.is_friend_of(ctx.catalog, machine.owner, p)
                    })
                    .count() as u32;
                set_flag(ctx.core, machine.owner, "gj_yange_inited", 1);
                set_flag(ctx.core, machine.owner, "gj_yange_marks", enemies);
            }
            if flag(ctx.core, machine.owner, "gj_yange_marks") == 0 {
                return Flow::Done;
            }
            let opts = push_non_self_alive(ctx.core, machine.owner);
            if opts.is_empty() {
                return Flow::Done;
            }
            wait_ask(
                machine,
                ctx,
                HostStage::TargetChoice,
                machine.owner,
                ASK_YANGE_TARGET,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gj_yange_target",
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
            let marks = flag(ctx.core, machine.owner, "gj_yange_marks");
            if marks == 0 || !ctx.core.player(t).alive {
                return Flow::Done;
            }
            set_flag(ctx.core, machine.owner, "gj_yange_marks", marks - 1);
            super::subplayer::enter_yange_subplayer(ctx, machine.owner, t);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_lianjing(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            let opts = push_non_self_alive(ctx.core, machine.owner);
            if opts.is_empty() {
                return Flow::Done;
            }
            let max = opts.len().min(2);
            wait_ask(
                machine,
                ctx,
                HostStage::TargetChoice,
                machine.owner,
                ASK_LIANJING_TARGETS,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gj_lianjing_targets",
                    options: opts,
                    min: 1,
                    max,
                    allow_cancel: true,
                },
            )
        }
        HostStage::TargetChoice => {
            let picked: Vec<PlayerId> = ctx
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
            set_flag(
                ctx.core,
                machine.owner,
                "gj_lianjing_ready_round",
                ctx.core.turn.round + 2,
            );
            let mut list = vec![machine.owner];
            for p in picked {
                if ctx.core.player(p).alive && !list.contains(&p) {
                    list.push(p);
                }
            }
            list.sort_unstable();
            for p in ctx.core.alive_players().collect::<Vec<_>>() {
                if !list.contains(&p) {
                    set_flag(ctx.core, p, "u05_mod:gj_lianjing_out_of_play", 1);
                }
            }
            set_flag(ctx.core, 0, "gj_lianjing_len", list.len() as u32);
            set_flag(ctx.core, 0, "gj_lianjing_idx", 0);
            for (idx, p) in list.into_iter().enumerate() {
                set_flag(ctx.core, 0, &format!("gj_lianjing_q:{idx}"), u32::from(p));
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_cihong(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            let opts: Vec<ChoiceOption> = ctx
                .core
                .alive_players()
                .filter(|&p| {
                    p != machine.owner && can_target_card(ctx.core, ctx.catalog, machine.owner, p, CardKind::Sha)
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
                ASK_CIHONG_TARGET,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gj_cihong_target",
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
            set_flag(
                ctx.core,
                machine.owner,
                "gj_cihong_ready_round",
                ctx.core.turn.round + 3,
            );
            machine.target = Some(t);
            let red_opts: Vec<ChoiceOption> = hand_and_equips(ctx.core, machine.owner)
                .into_iter()
                .filter(|&c| is_red(ctx.catalog, c))
                .map(ChoiceOption::Card)
                .collect();
            if !red_opts.is_empty() && ctx.core.player(t).alive {
                return wait_ask(
                    machine,
                    ctx,
                    HostStage::SelectCard,
                    machine.owner,
                    ASK_CIHONG_RED,
                    ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "gj_cihong_red",
                        options: red_opts,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                );
            }
            ask_cihong_losehp(machine, ctx)
        }
        HostStage::SelectCard => {
            if let ChoiceOption::Card(cid) = answer_first_or_pass(ctx) {
                if discard_card_from_he(ctx, machine.owner, cid) {
                    if let Some(t) = machine.target {
                        spawn_virtual_sha(ctx, machine.owner, vec![t], DamageNature::Normal);
                        machine.stage = HostStage::DrawSettle;
                        return Flow::Yield;
                    }
                }
            }
            ask_cihong_losehp(machine, ctx)
        }
        HostStage::DrawSettle => ask_cihong_losehp(machine, ctx),
        HostStage::ComparePlayerPick => {
            if matches!(answer_first_or_pass(ctx), ChoiceOption::Confirm) {
                ctx.emit(EventData::LoseHp {
                    player: machine.owner,
                    amount: 1,
                });
                machine.stage = HostStage::SelectNature;
                return Flow::Yield;
            }
            ask_cihong_turnover(machine, ctx)
        }
        HostStage::SelectNature => {
            if let Some(t) = machine.target {
                if ctx.core.player(machine.owner).alive && ctx.core.player(t).alive {
                    spawn_virtual_sha(ctx, machine.owner, vec![t], DamageNature::Normal);
                    machine.stage = HostStage::HuanrenPick;
                    return Flow::Yield;
                }
            }
            ask_cihong_turnover(machine, ctx)
        }
        HostStage::HuanrenPick => ask_cihong_turnover(machine, ctx),
        HostStage::CompareTargetPick => {
            if matches!(answer_first_or_pass(ctx), ChoiceOption::Confirm) && !ctx.core.player(machine.owner).face_down {
                ctx.turn_over(machine.owner);
                machine.stage = HostStage::ReguhuoFinalPick;
                return Flow::Yield;
            }
            Flow::Done
        }
        HostStage::ReguhuoFinalPick => {
            if let Some(t) = machine.target {
                if ctx.core.player(machine.owner).alive && ctx.core.player(t).alive {
                    spawn_virtual_sha(ctx, machine.owner, vec![t], DamageNature::Normal);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn ask_cihong_losehp(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = machine.target else {
        return Flow::Done;
    };
    if !ctx.core.player(t).alive || !ctx.core.player(machine.owner).alive {
        return Flow::Done;
    }
    wait_ask(
        machine,
        ctx,
        HostStage::ComparePlayerPick,
        machine.owner,
        ASK_CIHONG_LOSEHP,
        ChoiceSpec {
            kind: ChoiceKind::Confirm,
            prompt: "gj_cihong_losehp",
            options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            min: 1,
            max: 1,
            allow_cancel: false,
        },
    )
}

fn ask_cihong_turnover(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = machine.target else {
        return Flow::Done;
    };
    if !ctx.core.player(t).alive
        || !ctx.core.player(machine.owner).alive
        || ctx.core.player(machine.owner).face_down
    {
        return Flow::Done;
    }
    wait_ask(
        machine,
        ctx,
        HostStage::CompareTargetPick,
        machine.owner,
        ASK_CIHONG_TURNOVER,
        ChoiceSpec {
            kind: ChoiceKind::Confirm,
            prompt: "gj_cihong_turnover",
            options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            min: 1,
            max: 1,
            allow_cancel: false,
        },
    )
}
