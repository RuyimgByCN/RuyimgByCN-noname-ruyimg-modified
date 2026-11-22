//! L25 模式与交互补全（`_anshaskill`、`_seadreammodeskill`、`_relifeskill`、
//! `_guanmodeskill`、L25 `gongxin`、以及 `wumou`/`_fromsay`/`_zuanCHANGEHPsayskill`/
//! `_zuanidentityshowskill`/`_bingletexiao1..2` 运行时钩子）。

use super::l25_batch_a_asks::wait_ask;
use super::*;
use crate::catalog::{Catalog, QueryCtx, Suit, TriggerMeta};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::{GameMode, GameStateCore, Role, SkillInstance};

fn mode_meta(event: EventKind, extra_events: Vec<EventKind>) -> TriggerMeta {
    TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 3,
        order_selectable: false,
        optional: false,
        extra_events,
    }
}

pub(super) fn is_l25_mode_skill(name: &str) -> bool {
    matches!(
        name,
        "_anshaskill"
            | "_seadreammodeskill"
            | "_relifeskill"
            | "_guanmodeskill"
            | "_dietime"
            | "ansha_zhong"
            | "ansha_nei"
            | "huanhaicls"
            | "guanfu"
            | "guifu_nei"
            | "guifu_fire"
            | "guifu_dam"
            | "gongxin"
    )
}

pub(super) fn is_l25_mode_active(state: &GameStateCore, expected: u32) -> bool {
    flag(state, 0, "l25_zuan_mode") == expected
}

fn is_identity_lord(state: &GameStateCore, p: PlayerId) -> bool {
    matches!(
        &state.mode,
        GameMode::Identity { roles } if roles.get(p as usize) == Some(&Role::Lord)
    )
}

pub(super) fn trigger_meta_mode_l25(name: &str) -> Option<TriggerMeta> {
    match name {
        "_anshaskill" => Some(mode_meta(EventKind::PhaseBegin, Vec::new())),
        "_seadreammodeskill" => Some(mode_meta(EventKind::PhaseBegin, Vec::new())),
        "_relifeskill" => Some(mode_meta(EventKind::TurnBegin, Vec::new())),
        "_guanmodeskill" => Some(mode_meta(EventKind::PhaseBegin, vec![EventKind::TurnBegin])),
        "_dietime" => Some(mode_meta(EventKind::TurnBegin, vec![EventKind::PhaseBegin])),
        "guanfu" => Some(mode_meta(EventKind::Death, Vec::new())),
        "guifu_nei" => Some(mode_meta(EventKind::Death, Vec::new())),
        "guifu_fire" => Some(mode_meta(EventKind::Dying, Vec::new())),
        _ => None,
    }
}

fn guifu_dam_targets(state: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Vec<PlayerId> {
    alive_others(state, owner)
        .into_iter()
        .filter(|&p| {
            !catalog
                .skill_by_name("jingcheng")
                .is_some_and(|d| state.has_skill(p, d))
        })
        .collect()
}

pub(super) fn activate_mode_l25(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let state = ctx.state;
    match skill.name.as_str() {
        "gongxin" => {
            if flag(state, owner, "skill_turn:gongxin") >= 2 {
                return None;
            }
            let has_tgt = alive_others(state, owner)
                .into_iter()
                .any(|p| flag(state, p, "l25_gongxin_temp") == 0 && !state.player(p).hand.is_empty());
            if !has_tgt {
                return None;
            }
        }
        "guifu_dam" => {
            if flag(state, owner, "mark:guifu_dam") == 0
                || guifu_dam_targets(state, ctx.catalog, owner).is_empty()
            {
                return None;
            }
        }
        _ => return None,
    }
    Some(Box::new(HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

pub(super) fn filter_mode_l25(
    name: &str,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match name {
        "_dietime" => {
            flag(ctx.state, owner, "l25_dietime_init") == 0
                && is_l25_mode_active(ctx.state, 8)
                && is_identity_lord(ctx.state, owner)
                && matches!(
                    event,
                    EventData::TurnBegin { .. }
                        | EventData::Phase {
                            phase: Phase::Prepare,
                            begin: true,
                            ..
                        }
                )
        }
        "_anshaskill" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner
                && is_l25_mode_active(ctx.state, 8)
                && is_identity_lord(ctx.state, owner)
        ),
        "_seadreammodeskill" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if *player == owner
                && is_l25_mode_active(ctx.state, 7)
                && is_identity_lord(ctx.state, owner)
        ),
        "_relifeskill" => matches!(
            event,
            EventData::TurnBegin { .. }
                if (is_l25_mode_active(ctx.state, 2) || is_l25_mode_active(ctx.state, 4))
                    && is_identity_lord(ctx.state, owner)
                    && ctx.state.turn.round >= 2
                    && ctx.state.turn.round > flag(ctx.state, owner, "l25_relife_round_stamp")
        ),
        "_guanmodeskill" => {
            flag(ctx.state, owner, "l25_guanmode_init") == 0
                && is_l25_mode_active(ctx.state, 4)
                && is_identity_lord(ctx.state, owner)
                && matches!(
                    event,
                    EventData::TurnBegin { .. }
                        | EventData::Phase {
                            phase: Phase::Prepare,
                            begin: true,
                            ..
                        }
                )
        }
        "guanfu" => {
            !matches!(ctx.state.mode, GameMode::Kingdoms)
                && matches!(event, EventData::Death { .. })
        }
        "guifu_nei" => matches!(
            event,
            EventData::Death {
                player,
                killer: Some(k),
            } if *player == owner && *k != owner && ctx.state.player(*k).alive
        ),
        "guifu_fire" => matches!(event, EventData::Dying { player, .. } if *player == owner),
        _ => false,
    }
}

pub(super) fn ask_mode_l25(
    ask_code: u32,
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    target: Option<PlayerId>,
) -> Option<(PlayerId, ChoiceSpec)> {
    let tag_ask = |chooser: PlayerId, prompt: &'static str, range: std::ops::RangeInclusive<u8>, allow_cancel: bool| {
        let opts: Vec<ChoiceOption> = range.map(ChoiceOption::Tag).collect();
        Some((
            chooser,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt,
                options: opts,
                min: 1,
                max: 1,
                allow_cancel,
            },
        ))
    };
    match ask_code {
        ASK_NEW_GUIXIN_ZONE => tag_ask(owner, "new_guixin:choose_zone", 0..=2, false),
        ASK_L25_GONGXIN_TARGET => {
            let cands: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| flag(core, p, "l25_gongxin_temp") == 0 && !core.player(p).hand.is_empty())
                .collect();
            (!cands.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gongxin_target",
                    options: cands.into_iter().map(ChoiceOption::Player).collect(),
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_L25_GONGXIN_CARD => {
            let tgt = target?;
            let hearts: Vec<ChoiceOption> = core
                .player(tgt)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.effective_card_suit(core, tgt, c) == Suit::Heart)
                .map(ChoiceOption::Card)
                .collect();
            (!hearts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gongxin_card",
                    options: hearts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_L25_GONGXIN_CHOICE => tag_ask(owner, "gongxin_choice", 0..=1, false),
        ASK_L25_GUIFU_DAM_TARGET => {
            let cands = guifu_dam_targets(core, catalog, owner);
            (!cands.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "guifu_dam_target",
                    options: cands.into_iter().map(ChoiceOption::Player).collect(),
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        _ => None,
    }
}

pub(super) fn handle_mode_l25(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "_dietime" => step_l25_dietime(m, ctx),
        "_anshaskill" => step_l25_ansha(m, ctx),
        "_seadreammodeskill" => step_l25_seadream(m, ctx),
        "_relifeskill" => step_l25_relife(m, ctx),
        "_guanmodeskill" => step_l25_guanmode(m, ctx),
        "guanfu" => step_l25_guanfu(m, ctx),
        "guifu_nei" => step_l25_guifu_nei(m, ctx),
        "guifu_fire" => step_l25_guifu_fire(m, ctx),
        "guifu_dam" => step_l25_guifu_dam(m, ctx),
        "gongxin" => step_l25_gongxin(m, ctx),
        _ => Flow::Done,
    }
}

fn ensure_enabled_skill(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(d) = ctx.catalog.skill_by_name(name) {
        ctx.core.skills.entry((p, d)).or_insert(SkillInstance {
            owner: p,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        });
    }
}

fn step_l25_dietime(_m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = _m.owner;
    if flag(ctx.core, owner, "l25_dietime_init") > 0 {
        return Flow::Done;
    }
    set_flag(ctx.core, owner, "l25_dietime_init", 1);
    set_flag(ctx.core, owner, "l25_zhudie", 0);
    set_flag(ctx.core, owner, "mark:_dietime", 2);
    let balance = flag(ctx.core, 0, "l25_balance") > 0;
    let total = ctx.core.players.len() as PlayerId;
    for p in 0..total {
        let is_nei = matches!(
            &ctx.core.mode,
            GameMode::Identity { roles } if roles.get(p as usize) == Some(&Role::Traitor)
        );
        if is_nei {
            if let GameMode::Identity { roles } = &mut ctx.core.mode {
                if let Some(slot) = roles.get_mut(p as usize) {
                    *slot = if balance { Role::Loyalist } else { Role::Traitor };
                }
            }
            ensure_enabled_skill(ctx, p, "ansha_nei");
        }
    }
    let nei_def = ctx.catalog.skill_by_name("ansha_nei");
    for p in 0..total {
        let has_nei = nei_def.is_some_and(|d| ctx.core.has_skill(p, d));
        if !has_nei && !is_identity_lord(ctx.core, p) {
            if let GameMode::Identity { roles } = &mut ctx.core.mode {
                if let Some(slot) = roles.get_mut(p as usize) {
                    *slot = if balance { Role::Loyalist } else { Role::Traitor };
                }
            }
            ensure_enabled_skill(ctx, p, "ansha_zhong");
        }
    }
    Flow::Done
}

fn step_l25_ansha(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let t = flag(ctx.core, owner, "mark:_dietime");
    if t > 1 {
        set_flag(ctx.core, owner, "mark:_dietime", t - 1);
        return Flow::Done;
    }
    set_flag(ctx.core, owner, "mark:_dietime", 0);
    let zhong_def = ctx.catalog.skill_by_name("ansha_zhong");
    let nei_def = ctx.catalog.skill_by_name("ansha_nei");
    let zhongs: Vec<PlayerId> = alive_others(ctx.core, owner)
        .into_iter()
        .filter(|&p| zhong_def.is_some_and(|d| ctx.core.has_skill(p, d)))
        .collect();
    if !zhongs.is_empty() {
        let idx = ctx.core.rng.gen_range(zhongs.len() as u32) as usize;
        let victim = zhongs[idx];
        if flag(ctx.core, 0, "l25_balance") > 0 {
            if let GameMode::Identity { roles } = &mut ctx.core.mode {
                for &z in &zhongs {
                    if let Some(r) = roles.get_mut(z as usize) {
                        *r = Role::Traitor;
                    }
                }
            }
        }
        if let GameMode::Identity { roles } = &mut ctx.core.mode {
            if let Some(r) = roles.get_mut(victim as usize) {
                *r = Role::Loyalist;
            }
        }
        ctx.core.player_mut(victim).hp = 0;
        ctx.emit(EventData::Death {
            player: victim,
            killer: None,
        });
    }
    if zhongs.len() <= 1 {
        let zhudie = flag(ctx.core, owner, "l25_zhudie");
        if zhudie == 0 {
            let neis: Vec<PlayerId> = alive_others(ctx.core, owner)
                .into_iter()
                .filter(|&p| nei_def.is_some_and(|d| ctx.core.has_skill(p, d)))
                .collect();
            if let GameMode::Identity { roles } = &mut ctx.core.mode {
                for p in neis {
                    if let Some(r) = roles.get_mut(p as usize) {
                        *r = Role::Rebel;
                    }
                }
            }
            set_flag(ctx.core, owner, "mark:_dietime", 3);
            set_flag(ctx.core, owner, "l25_zhudie", 1);
        } else {
            ctx.core.player_mut(owner).hp = 0;
            ctx.emit(EventData::Death {
                player: owner,
                killer: None,
            });
        }
    }
    Flow::Done
}

fn clear_huanhaicls_on(ctx: &mut Ctx<'_>, p: PlayerId) {
    if let Some(hdef) = ctx.catalog.skill_by_name("huanhaicls") {
        ctx.core.skills.remove(&(p, hdef));
    }
    let keys: Vec<(String, u16)> = ctx
        .core
        .vm_aux
        .flags
        .keys()
        .filter_map(|(seat, k)| {
            if *seat == p {
                k.strip_prefix("huanhaicls_disabled:")
                    .and_then(|s| s.parse::<u16>().ok())
                    .map(|def| (k.clone(), def))
            } else {
                None
            }
        })
        .collect();
    for (k, def) in keys {
        ctx.core.vm_aux.flags.remove(&(p, k));
        if let Some(inst) = ctx.core.skills.get_mut(&(p, def)) {
            inst.enabled = true;
        }
    }
}

fn apply_seadream_hit(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let all = alive_all(ctx.core, owner);
    let hdef = ctx.catalog.skill_by_name("huanhaicls");
    for p in all {
        if let Some(def) = hdef {
            ctx.core.skills.entry((p, def)).or_insert(SkillInstance {
                owner: p,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            });
        }
        let to_disable: Vec<u16> = ctx
            .core
            .skills
            .iter()
            .filter_map(|((seat, def), inst)| {
                if *seat != p || !inst.enabled {
                    return None;
                }
                let sname = ctx.catalog.skill_name(*def);
                if matches!(sname, "huanhaicls" | "_seadreammodeskill")
                    || ctx.catalog.is_charlotte_skill(*def)
                    || ctx.catalog.is_forever_skill(*def)
                {
                    return None;
                }
                Some(*def)
            })
            .collect();
        for def in to_disable {
            if let Some(inst) = ctx.core.skills.get_mut(&(p, def)) {
                inst.enabled = false;
                ctx.core
                    .vm_aux
                    .flags
                    .insert((p, format!("huanhaicls_disabled:{def}")), 1);
            }
        }
    }
}

fn step_l25_seadream(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            for p in 0..ctx.core.players.len() as PlayerId {
                clear_huanhaicls_on(ctx, p);
            }
            let alive_cnt = ctx
                .core
                .seat_ring
                .iter()
                .filter(|&&p| ctx.core.player(p).alive)
                .count();
            for i in 0..alive_cnt {
                let pool: Vec<PlayerId> = ctx
                    .core
                    .seat_ring
                    .iter()
                    .copied()
                    .filter(|&p| ctx.core.player(p).alive)
                    .collect();
                if i < pool.len() {
                    let tar = pool[i];
                    let j = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                    let swap = pool[j];
                    ctx.core.swap_seat(tar, swap);
                }
            }
            let swaps = flag(ctx.core, owner, "l25_seadream_swap_runs") + 1;
            set_flag(ctx.core, owner, "l25_seadream_swap_runs", swaps);
            m.stage = HostStage::DrawSettle;
            if flag(ctx.core, 0, "l25_balance") > 0 {
                ctx.start_draw(owner, 1);
            } else {
                set_flag(ctx.core, owner, "last_judge_suit", 0);
                ctx.emit(EventData::OrdinaryJudge {
                    owner,
                    invocation: u32::MAX,
                });
            }
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if flag(ctx.core, 0, "l25_balance") > 0 {
                let roll = (ctx.core.rng.gen_range(6) + 1) as u8;
                set_flag(ctx.core, owner, "l25_seadream_last_dice", u32::from(roll));
                for p in alive_all(ctx.core, owner) {
                    let he_matched: Vec<CardId> = ctx
                        .core
                        .player(p)
                        .hand
                        .iter()
                        .chain(ctx.core.player(p).equips.iter())
                        .copied()
                        .filter(|&c| ctx.catalog.cards[c as usize].rank == roll)
                        .collect();
                    let j_matched: Vec<CardId> = ctx
                        .core
                        .player(p)
                        .judge
                        .iter()
                        .copied()
                        .filter(|&c| ctx.catalog.cards[c as usize].rank == roll)
                        .collect();
                    for &cid in &he_matched {
                        let was_equip = ctx.core.player(p).equips.contains(&cid);
                        ctx.take_owned_card(p, cid);
                        if was_equip {
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, p, cid);
                        }
                        ctx.to_discard(cid);
                    }
                    if !j_matched.is_empty() {
                        ctx.core
                            .player_mut(p)
                            .judge
                            .retain(|c| !j_matched.contains(c));
                        ctx.note_cards_lost(p, &j_matched);
                        for &cid in &j_matched {
                            ctx.to_discard(cid);
                        }
                    }
                    let mut all_disc = he_matched;
                    all_disc.extend(j_matched);
                    if !all_disc.is_empty() {
                        ctx.log(LogEvent::CardDiscarded {
                            player: p,
                            cards: all_disc.clone(),
                        });
                        (ctx.catalog.hooks.on_cards_to_discard)(ctx, p, &all_disc, false);
                    }
                }
                if roll == 6 {
                    apply_seadream_hit(ctx, owner);
                }
            } else if flag(ctx.core, owner, "last_judge_suit") == 4 {
                apply_seadream_hit(ctx, owner);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_l25_relife(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    set_flag(ctx.core, owner, "l25_relife_round_stamp", ctx.core.turn.round);
    let total = ctx.core.players.len() as PlayerId;
    for p in 0..total {
        if ctx.core.player(p).alive {
            continue;
        }
        let timer = flag(ctx.core, p, "l25_relife_timer");
        let new_timer = timer.saturating_sub(1);
        if new_timer >= 1 {
            set_flag(ctx.core, p, "l25_relife_timer", new_timer);
            continue;
        }
        {
            let pl = ctx.core.player_mut(p);
            pl.alive = true;
            pl.hp = pl.max_hp as i32;
            pl.dying = false;
            pl.death_cancelled = false;
        }
        let level = flag(ctx.core, p, "l25_level").max(1);
        let draw_n = (level + 1) / 2;
        let next_timer = ((level * 4 + 10) / 10).max(1);
        set_flag(ctx.core, p, "l25_relife_timer", next_timer);
        if ctx
            .catalog
            .skill_by_name("zuan_hudun")
            .is_some_and(|d| ctx.core.has_skill(p, d))
        {
            let max_hj = flag(ctx.core, p, "l25_maxhujia").max(1);
            set_flag(ctx.core, p, "l25_zuan_hudun", max_hj);
            set_flag(ctx.core, p, "mark:zuan_hudun", max_hj);
        }
        if draw_n > 0 {
            ctx.start_draw(p, draw_n);
        }
    }
    Flow::Done
}

fn step_l25_guanmode(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    set_flag(ctx.core, owner, "l25_guanmode_init", 1);
    set_flag(ctx.core, owner, "mark:zuan_guanbing", 0);
    set_flag(ctx.core, owner, "mark:zuan_panjun", 0);
    if let GameMode::Identity { roles } = &ctx.core.mode {
        let pairs: Vec<(PlayerId, Role)> = roles
            .iter()
            .enumerate()
            .map(|(i, r)| (i as PlayerId, *r))
            .collect();
        for (p, role) in pairs {
            if role == Role::Lord {
                ensure_enabled_skill(ctx, p, "guanfu");
            } else if role == Role::Traitor {
                ensure_enabled_skill(ctx, p, "guifu_nei");
            }
        }
    }
    Flow::Done
}

fn step_l25_guanfu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(dead) = m.target else {
        return Flow::Done;
    };
    if matches!(ctx.core.mode, GameMode::Kingdoms) {
        return Flow::Done;
    }
    if let Some(roles) = ctx.core.identity_roles() {
        match roles.get(dead as usize).copied() {
            Some(Role::Rebel) => {
                let n = flag(ctx.core, owner, "mark:zuan_guanbing") + 1;
                set_flag(ctx.core, owner, "mark:zuan_guanbing", n);
            }
            Some(Role::Loyalist) => {
                let n = flag(ctx.core, owner, "mark:zuan_panjun") + 1;
                set_flag(ctx.core, owner, "mark:zuan_panjun", n);
            }
            _ => {}
        }
    }
    Flow::Done
}

fn step_l25_guifu_nei(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(killer) = m.cards.first().copied().map(|v| v as PlayerId) else {
        return Flow::Done;
    };
    if killer == owner || !ctx.core.player(killer).alive {
        return Flow::Done;
    }
    ensure_enabled_skill(ctx, killer, "guifu_fire");
    ensure_enabled_skill(ctx, killer, "guifu_dam");
    let dam_marks = flag(ctx.core, killer, "mark:guifu_dam") + 1;
    let fire_marks = flag(ctx.core, killer, "mark:guifu_fire") + 3;
    set_flag(ctx.core, killer, "mark:guifu_dam", dam_marks);
    set_flag(ctx.core, killer, "mark:guifu_fire", fire_marks);
    if let GameMode::Identity { roles } = &ctx.core.mode {
        let loyalists: Vec<PlayerId> = (0..roles.len() as PlayerId)
            .filter(|&p| p != owner && ctx.core.player(p).alive && roles[p as usize] == Role::Loyalist)
            .collect();
        if !loyalists.is_empty() {
            let idx = ctx.core.rng.gen_range(loyalists.len() as u32) as usize;
            let him = loyalists[idx];
            if let GameMode::Identity { roles } = &mut ctx.core.mode {
                roles.swap(owner as usize, him as usize);
            }
        }
    }
    if flag(ctx.core, 0, "l25_balance") > 0 {
        ctx.change_hujia(killer, 1);
    }
    Flow::Done
}

fn remove_guifu_fire_and_dam(core: &mut GameStateCore, catalog: &Catalog, player: PlayerId) {
    if let Some(d) = catalog.skill_by_name("guifu_fire") {
        core.skills.remove(&(player, d));
    }
    if let Some(d) = catalog.skill_by_name("guifu_dam") {
        core.skills.remove(&(player, d));
    }
    set_flag(core, player, "mark:guifu_fire", 0);
}

fn step_l25_guifu_fire(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    remove_guifu_fire_and_dam(ctx.core, ctx.catalog, m.owner);
    Flow::Done
}

fn step_l25_guifu_dam(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_L25_GUIFU_DAM_TARGET),
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let cur = flag(ctx.core, owner, "mark:guifu_dam");
            if cur == 0 || !ctx.core.player(tgt).alive {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "mark:guifu_dam", cur - 1);
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(owner),
                tgt,
                1,
                DamageNature::Fire,
                None,
            )));
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

pub(super) fn on_draw_begin_guifu_fire(ctx: &mut Ctx<'_>, player: PlayerId, pending: &mut i32) {
    if !ctx.core.player(player).alive {
        return;
    }
    let has_fire = ctx
        .catalog
        .skill_by_name("guifu_fire")
        .is_some_and(|d| ctx.core.has_skill(player, d));
    if !has_fire {
        return;
    }
    let cur = flag(ctx.core, player, "mark:guifu_fire");
    if cur > 0 {
        *pending += 1;
        let rem = cur - 1;
        set_flag(ctx.core, player, "mark:guifu_fire", rem);
        if rem == 0 {
            remove_guifu_fire_and_dam(ctx.core, ctx.catalog, player);
        }
    } else {
        remove_guifu_fire_and_dam(ctx.core, ctx.catalog, player);
    }
}

fn step_l25_gongxin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_L25_GONGXIN_TARGET),
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let used = flag(ctx.core, owner, "skill_turn:gongxin") + 1;
            set_flag(ctx.core, owner, "skill_turn:gongxin", used);
            set_flag(ctx.core, tgt, "l25_gongxin_temp", 1);
            let has_heart = ctx
                .core
                .player(tgt)
                .hand
                .iter()
                .any(|&c| ctx.catalog.effective_card_suit(ctx.core, tgt, c) == Suit::Heart);
            if !has_heart {
                set_flag(ctx.core, owner, "l25_gongxin_no_heart", 1);
                return Flow::Done;
            }
            m.target = Some(tgt);
            wait_ask(m, ctx, HostStage::SelectCard, ASK_L25_GONGXIN_CARD)
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.card = Some(cid);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_L25_GONGXIN_CHOICE)
        }
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l25_ask_id", 0);
            let Some(tgt) = m.target else {
                return Flow::Done;
            };
            let Some(cid) = m.card else {
                return Flow::Done;
            };
            let choice = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(t)) => t,
                _ => 0,
            };
            if choice == 0 {
                if ctx.core.player(tgt).hand.contains(&cid) {
                    ctx.take_hand_card(tgt, cid);
                    ctx.core.player_mut(owner).hand.push(cid);
                    ctx.log(LogEvent::CardsDrawn {
                        player: owner,
                        cards: vec![cid],
                    });
                    set_flag(ctx.core, owner, "l25_gongxin_obtained_heart", 1);
                }
            } else {
                set_flag(ctx.core, tgt, "l25_gongxin_red_sealed", u32::from(owner) + 1);
                set_flag(ctx.core, owner, "l25_gongxin_seal_mode", 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn zuan_hp_say_enabled(state: &GameStateCore) -> bool {
    if flag(state, 0, "l25_config_applied") == 0 {
        return true;
    }
    flag(state, 0, "l25_zuantype_new") > 0 && flag(state, 0, "l25_ransay") > 0
}

pub(super) fn zuan_attitude_positive(state: &GameStateCore, owner: PlayerId, other: PlayerId) -> bool {
    match &state.mode {
        GameMode::Identity { roles } => {
            let r1 = roles.get(owner as usize).copied();
            let r2 = roles.get(other as usize).copied();
            matches!(
                (r1, r2),
                (
                    Some(Role::Lord | Role::Loyalist),
                    Some(Role::Lord | Role::Loyalist)
                ) | (Some(Role::Rebel), Some(Role::Rebel))
            )
        }
        GameMode::Versus { teams, .. } => teams
            .iter()
            .any(|t| t.contains(&owner) && t.contains(&other)),
        _ => true,
    }
}

pub(super) fn has_maixie_tag(state: &GameStateCore, catalog: &Catalog, p: PlayerId) -> bool {
    [
        "jianxiong",
        "yiji",
        "fankui",
        "ganglie",
        "奸雄",
        "遗计",
        "反馈",
        "刚烈",
        "new_guixin",
        "ssr_newganglie",
    ]
    .iter()
    .any(|name| {
        catalog
            .skill_by_name(name)
            .is_some_and(|d| state.has_skill(p, d))
    })
}

pub(super) fn update_zuan_identity_show(core: &mut GameStateCore, owner: PlayerId) {
    if owner == 0 || flag(core, owner, "l25_hidentity") == 0 {
        return;
    }
    let real = flag(core, owner, "l25_sidentity");
    let cur = flag(core, owner, "l25_shown_role");
    let att = flag(core, owner, "l25_zhu_att") as i32;
    if real > 0 && cur == real && att.abs() > 80 {
        set_flag(core, owner, "l25_hidentity", 0);
        set_flag(core, owner, "l25_shown_role", real);
        return;
    }
    let shown = if att > 0 {
        2
    } else if att < 0 {
        3
    } else if real > 0 {
        real
    } else {
        4
    };
    set_flag(core, owner, "l25_shown_role", shown);
}

pub(super) fn reveal_zuan_identity_on_dying(core: &mut GameStateCore, owner: PlayerId) {
    let real = flag(core, owner, "l25_sidentity");
    if real > 0 {
        set_flag(core, owner, "l25_shown_role", real);
    }
}

fn bump_ui_hook_flags(core: &mut GameStateCore, owner: PlayerId, hook: &str) {
    let key = format!("l25_ui_hook:{hook}");
    let cnt = flag(core, owner, &key) + 1;
    set_flag(core, owner, &key, cnt);
    set_flag(core, owner, &format!("l25_vfx_fired:{hook}"), cnt);
    let total = flag(core, owner, "l25_ui_hook_invoked") + 1;
    set_flag(core, owner, "l25_ui_hook_invoked", total);
}

/// 保持历史内容锁不变的宿主事件钩子分派：
/// - `wumou`：`Phase::End { begin: true }` 结算阶段触发，非延时锦囊仅扣暴怒/体力各至多 1 次
/// - `gongxin` 阶段临时目标清理于 `TurnEnded`（`phaseAfter`），`ssr_gongxin_break` 于使用方 `TurnBegin`/`Death` 解除
/// - `_zuanCHANGEHPsayskill` 覆盖 Damage/Recover/LoseHp/CardsDrawn/Death 5 事件（每事件仅触发 1 次）
pub(crate) fn on_l25_host_event(ctx: &mut Ctx<'_>, event: &EventData) {
    match event {
        EventData::Phase {
            player,
            phase: Phase::End,
            begin: true,
        } => {
            let p = *player;
            if ctx.core.player(p).alive
                && flag(ctx.core, p, "l25_wumou_trick_in_play") > 0
                && ctx
                    .catalog
                    .skill_by_name("wumou")
                    .is_some_and(|d| ctx.core.has_skill(p, d))
                && !ctx.catalog.is_tieji_sealed(ctx.core, p)
            {
                set_flag(ctx.core, p, "l25_wumou_trick_in_play", 0);
                ctx.res.parks.push(crate::resolution::Park {
                    resume_depth: ctx.res.stack.len(),
                    slot: crate::resolution::Slot::Intrinsic {
                        frame: crate::resolution::NO_FRAME,
                    },
                    awaiting_answer: false,
                    machine: Box::new(HostMachine {
                        def_key: "wumou".to_string(),
                        behavior: HostBehavior::L25ZuanBatchA,
                        owner: p,
                        target: None,
                        card: Some(105),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: HostStage::Act,
                    }),
                });
            }
        }
        EventData::TurnEnded { .. } => {
            for p in 0..ctx.core.players.len() as PlayerId {
                set_flag(ctx.core, p, "l25_gongxin_temp", 0);
            }
        }
        EventData::TurnBegin { player } => {
            let cur = u32::from(*player) + 1;
            for p in 0..ctx.core.players.len() as PlayerId {
                if flag(ctx.core, p, "l25_gongxin_red_sealed") == cur {
                    set_flag(ctx.core, p, "l25_gongxin_red_sealed", 0);
                    set_flag(ctx.core, p, "l25_gongxin_heart_sealed", 0);
                }
            }
        }
        _ => {}
    }

    let is_hp_say_event = match event {
        EventData::Damage(d) => !d.prevented && d.amount > 0,
        EventData::Recover { amount, .. } | EventData::LoseHp { amount, .. } => *amount > 0,
        EventData::CardsDrawn { cards, .. } => !cards.is_empty(),
        EventData::Death { .. } => true,
        _ => false,
    };
    if is_hp_say_event && zuan_hp_say_enabled(ctx.core) {
        if let Some(say_def) = ctx.catalog.skill_by_name("_zuanCHANGEHPsayskill") {
            for seat in 0..ctx.core.players.len() as PlayerId {
                if ctx.core.player(seat).alive && ctx.core.has_skill(seat, say_def) {
                    bump_ui_hook_flags(ctx.core, seat, "_zuanCHANGEHPsayskill");
                }
            }
        }
    }
    if let EventData::Death { player, .. } = event {
        let dead = u32::from(*player) + 1;
        for p in 0..ctx.core.players.len() as PlayerId {
            if flag(ctx.core, p, "l25_gongxin_red_sealed") == dead {
                set_flag(ctx.core, p, "l25_gongxin_red_sealed", 0);
                set_flag(ctx.core, p, "l25_gongxin_heart_sealed", 0);
            }
        }
    }
}
