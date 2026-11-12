//! L19 末日乱斗 Batch B（模式 4–10 RPG/祖安桥接 18 项技能，`末日乱斗/extension.js:2068-3360`）。
//!
//! 覆盖项：
//! 1. 共享 RPG 成长与复活（6 项）：`_levelskill`、`_expskill`、`_exp2skill`、`_useskill`、`_killskill`、`_relifeskill`
//! 2. 元气骑士（1 项）：`_yuanqiskill`
//! 3. 攻守京城（5 项）：`_zhongzhubuff`、`_jingchengadddie`、`_zhongfandieskill`、`_tamodeskill`、`_zhudieskill`
//! 4. 群雄争霸（1 项）：`_qunxiongzhengbaskill`
//! 5. 讨伐叛军（2 项）：`_guanmodeskill`、`_guandieskill`
//! 6. 天下归心（1 项）：`_guixinmodeskill`
//! 7. 幻海之诗（2 项）：`_huanhaiskill`、`_seadreammodeskill`
//! 8. 暗杀行动（2 项）：`_dietime`、`_anshaskill`

use super::super::*;
use crate::catalog::{Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::event::{EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::{GameMode, GameStateCore, Role, SkillInstance};

#[path = "l19_batch_b_asks.rs"]
mod l19_batch_b_asks;
pub(in crate::content::host) use l19_batch_b_asks::ask_l19_batch_b;

pub(super) const ASK_YUANQI_MODE: u32 = 30;
pub(super) const ASK_YUANQI_SKILL: u32 = 31;
pub(super) const ASK_YUANQI_REMOVE: u32 = 32;

pub(super) const RPG_SKILL_POOL: &[&str] = &[
    "zhiheng",
    "jianxiong",
    "rende",
    "paoxiao",
    "wusheng",
    "yingzi",
];

pub(super) fn flag(core: &GameStateCore, p: PlayerId, key: &str) -> u32 {
    core.vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

pub(super) fn set_flag(core: &mut GameStateCore, p: PlayerId, key: &str, val: u32) {
    if val == 0 {
        core.vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        core.vm_aux.flags.insert((p, key.to_string()), val);
    }
}

pub(super) fn turn_stamp(core: &GameStateCore) -> u32 {
    (core.turn.round.max(1) << 8) | u32::from(core.turn.current)
}

pub(super) fn player_role(core: &GameStateCore, p: PlayerId) -> Option<Role> {
    core.identity_roles().and_then(|r| r.get(p as usize).copied())
}

pub(super) fn set_player_role(core: &mut GameStateCore, p: PlayerId, role: Role) {
    if let GameMode::Identity { roles } = &mut core.mode {
        if let Some(slot) = roles.get_mut(p as usize) {
            *slot = role;
        }
    }
}

fn zh_skill_alias(name: &str) -> Option<&'static str> {
    match name {
        "zhiheng" => Some("制衡"),
        "jianxiong" => Some("奸雄"),
        "rende" => Some("仁德"),
        "paoxiao" => Some("咆哮"),
        "wusheng" => Some("武圣"),
        "yingzi" => Some("英姿"),
        "wushuang" => Some("无双"),
        "wansha" => Some("完杀"),
        _ => None,
    }
}

fn en_skill_alias(name: &str) -> Option<&'static str> {
    match name {
        "制衡" => Some("zhiheng"),
        "奸雄" => Some("jianxiong"),
        "仁德" => Some("rende"),
        "咆哮" => Some("paoxiao"),
        "武圣" => Some("wusheng"),
        "英姿" => Some("yingzi"),
        "无双" => Some("wushuang"),
        "完杀" => Some("wansha"),
        _ => None,
    }
}

pub(super) fn has_skill_named(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    name: &str,
) -> bool {
    if catalog.skill_by_name(name).is_some_and(|d| core.has_skill(p, d)) {
        return true;
    }
    if let Some(zh) = zh_skill_alias(name) {
        if catalog.skill_by_name(zh).is_some_and(|d| core.has_skill(p, d)) {
            return true;
        }
    }
    if let Some(en) = en_skill_alias(name) {
        if catalog.skill_by_name(en).is_some_and(|d| core.has_skill(p, d)) {
            return true;
        }
    }
    false
}

pub(super) fn grant_skill_named(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) -> Option<u16> {
    let mut granted = None;
    let inst = SkillInstance {
        owner: p,
        enabled: true,
        once_used: false,
        expires_end_of_turn: false,
        rounds: 0,
    };
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        ctx.core.skills.insert((p, def), inst.clone());
        granted = Some(def);
    }
    if let Some(zh) = zh_skill_alias(name) {
        if let Some(def) = ctx.catalog.skill_by_name(zh) {
            ctx.core.skills.insert((p, def), inst.clone());
            granted.get_or_insert(def);
        }
    }
    if let Some(en) = en_skill_alias(name) {
        if let Some(def) = ctx.catalog.skill_by_name(en) {
            ctx.core.skills.insert((p, def), inst);
            granted.get_or_insert(def);
        }
    }
    let canonical = en_skill_alias(name).unwrap_or(name);
    set_flag(ctx.core, p, &format!("zuan_skill:{canonical}"), 1);
    granted
}

pub(super) fn grant_rpg_pool_skill(ctx: &mut Ctx<'_>, owner: PlayerId) -> Option<u16> {
    let pick = RPG_SKILL_POOL
        .iter()
        .copied()
        .find(|&sk| !has_skill_named(ctx.core, ctx.catalog, owner, sk))
        .unwrap_or("zhiheng");
    let def = grant_skill_named(ctx, owner, pick);
    set_flag(ctx.core, owner, "_levelskill_granted", 1);
    let cnt = flag(ctx.core, owner, "zuan_skills_granted") + 1;
    set_flag(ctx.core, owner, "zuan_skills_granted", cnt);
    def
}

pub(super) fn removable_skills(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<u16> {
    let mut out = Vec::new();
    let mut seen_canonical = Vec::new();
    for (&(p, def), inst) in &core.skills {
        if p != owner || !inst.enabled {
            continue;
        }
        let Some(sk) = catalog.skills.get(def as usize) else {
            continue;
        };
        if sk.name == "_yuanqiskill" {
            continue;
        }
        let canon = en_skill_alias(sk.name).unwrap_or(sk.name);
        if !seen_canonical.contains(&canon) {
            seen_canonical.push(canon);
            out.push(def);
        }
    }
    out
}

pub(super) fn remove_skill_def(ctx: &mut Ctx<'_>, owner: PlayerId, def: u16) {
    ctx.core.skills.remove(&(owner, def));
    let Some(name) = ctx.catalog.skills.get(def as usize).map(|s| s.name) else {
        return;
    };
    if let Some(zh) = zh_skill_alias(name) {
        if let Some(zh_def) = ctx.catalog.skill_by_name(zh) {
            ctx.core.skills.remove(&(owner, zh_def));
        }
    }
    if let Some(en) = en_skill_alias(name) {
        if let Some(en_def) = ctx.catalog.skill_by_name(en) {
            ctx.core.skills.remove(&(owner, en_def));
        }
    }
}

pub(super) fn refund_once_used(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(p, def)) {
            inst.once_used = false;
        }
    }
}

pub(super) fn discard_cards(ctx: &mut Ctx<'_>, p: PlayerId, cards: &[CardId]) {
    let mut discarded = Vec::new();
    for &c in cards {
        if ctx.core.player(p).hand.contains(&c) {
            ctx.discard_from_hand(p, c);
            discarded.push(c);
        } else if ctx.core.player(p).equips.contains(&c) {
            ctx.take_owned_card(p, c);
            ctx.to_discard(c);
            discarded.push(c);
        }
    }
    if !discarded.is_empty() {
        ctx.emit(EventData::CardsDiscarded {
            player: p,
            cards: discarded,
        });
    }
}

pub(super) fn revive_player_to(ctx: &mut Ctx<'_>, p: PlayerId, max_hp: u32, hp: u32) {
    ctx.core.player_mut(p).max_hp = max_hp;
    let cur = ctx.core.player(p).hp;
    ctx.core.player_mut(p).alive = true;
    ctx.core.player_mut(p).dying = false;
    if hp as i32 > cur {
        ctx.recover(p, (hp as i32 - cur) as u32, None);
    }
    ctx.core.player_mut(p).hp = hp as i32;
    ctx.core.player_mut(p).dying = false;
    if ctx
        .res
        .stack
        .iter()
        .any(|f| matches!(f.data, EventData::Death { player, .. } if player == p))
    {
        ctx.core.player_mut(p).death_cancelled = true;
    }
}

pub(super) fn check_and_apply_level_up(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let mut exp = flag(ctx.core, owner, "zuan_exp");
    let mut lvl = flag(ctx.core, owner, "zuan_level").max(1);
    set_flag(ctx.core, owner, "zuan_level", lvl);
    set_flag(ctx.core, owner, "_levelskill", lvl);
    while exp >= 100 {
        exp -= 100;
        lvl += 1;
        set_flag(ctx.core, owner, "zuan_exp", exp);
        set_flag(ctx.core, owner, "zuan_level", lvl);
        set_flag(ctx.core, owner, "_levelskill", lvl);
        ctx.core.player_mut(owner).max_hp += 1;
        ctx.recover(owner, 1, None);
        ctx.start_draw(owner, 2);
        if lvl == 2 || lvl == 4 {
            let _ = grant_rpg_pool_skill(ctx, owner);
        }
    }
}

fn find_zhu_seat(core: &GameStateCore, exclude: PlayerId) -> Option<PlayerId> {
    (0..core.players.len() as PlayerId)
        .find(|&p| {
            p != exclude
                && core.player(p).alive
                && (flag(core, p, "is_zhu") == 1 || player_role(core, p) == Some(Role::Lord))
        })
        .or_else(|| (exclude != 0 && core.player(0).alive).then_some(0))
}

fn find_tower_nei(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Option<PlayerId> {
    (0..core.players.len() as PlayerId).find(|&p| {
        p != owner
            && core.player(p).alive
            && core.player(p).max_hp >= 1
            && (has_skill_named(core, catalog, p, "_tamodeskill")
                || flag(core, p, "is_tower_nei") == 1
                || flag(core, p, "_tamodeskill_untouchable") == 1)
    })
}

pub(in crate::content::host) fn trigger_meta_l19_batch_b(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, priority: i32, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "_levelskill" => Some(meta(
            EventKind::TurnBegin,
            20,
            vec![
                EventKind::PhaseBegin,
                EventKind::PhaseEnd,
                EventKind::Damage,
                EventKind::CardUsed,
                EventKind::Death,
                EventKind::TurnEnded,
            ],
        )),
        "_expskill" => Some(meta(EventKind::Damage, 25, Vec::new())),
        "_exp2skill" => Some(meta(
            EventKind::PhaseBegin,
            25,
            vec![EventKind::PhaseEnd, EventKind::TurnEnded],
        )),
        "_useskill" => Some(meta(EventKind::CardUsed, 25, Vec::new())),
        "_killskill" => Some(meta(EventKind::Death, 25, Vec::new())),
        "_relifeskill" => Some(meta(EventKind::Dying, 24, vec![EventKind::Death])),
        "_yuanqiskill" => Some(meta(EventKind::PhaseBegin, 10, Vec::new())),
        "_zhongzhubuff" => Some(meta(EventKind::Dying, 23, vec![EventKind::Death])),
        "_jingchengadddie" => Some(meta(EventKind::Death, 25, Vec::new())),
        "_zhongfandieskill" => Some(meta(EventKind::Dying, 18, vec![EventKind::Death])),
        "_tamodeskill" => Some(meta(EventKind::PhaseBegin, 25, vec![EventKind::TurnBegin])),
        "_zhudieskill" => Some(meta(EventKind::Dying, 25, vec![EventKind::Death])),
        "_qunxiongzhengbaskill" => Some(meta(EventKind::Dying, 22, vec![EventKind::Death])),
        "_guanmodeskill" => Some(meta(
            EventKind::TurnBegin,
            25,
            vec![
                EventKind::PhaseBegin,
                EventKind::TurnOverBefore,
                EventKind::TurnOvered,
            ],
        )),
        "_guandieskill" => Some(meta(EventKind::Dying, 25, vec![EventKind::Death])),
        "_guixinmodeskill" => Some(meta(EventKind::Dying, 25, vec![EventKind::Death])),
        "_huanhaiskill" => Some(meta(EventKind::TurnBegin, 22, vec![EventKind::PhaseBegin])),
        "_seadreammodeskill" => Some(meta(EventKind::Dying, 24, vec![EventKind::Death])),
        "_dietime" => Some(meta(
            EventKind::PhaseBegin,
            15,
            vec![EventKind::PhaseEnd, EventKind::TurnEnded],
        )),
        "_anshaskill" => Some(meta(EventKind::Dying, 24, vec![EventKind::Death])),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l19_batch_b(name: &str) -> SkillLimits {
    match name {
        "_yuanqiskill" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn filter_l19_batch_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive
        && !matches!(
            skill.name.as_str(),
            "_relifeskill"
                | "_zhongzhubuff"
                | "_jingchengadddie"
                | "_zhongfandieskill"
                | "_zhudieskill"
                | "_qunxiongzhengbaskill"
                | "_guandieskill"
                | "_seadreammodeskill"
                | "_anshaskill"
        )
    {
        return false;
    }
    match skill.name.as_str() {
        "_levelskill" => flag(ctx.state, owner, "zuan_exp") >= 100,
        "_expskill" => matches!(
            event,
            EventData::Damage(d) if !d.prevented && d.amount > 0 && d.source == Some(owner)
        ),
        "_exp2skill" => match event {
            EventData::Phase {
                player,
                phase: Phase::End,
                ..
            }
            | EventData::TurnEnded { player } => {
                *player == owner
                    && flag(ctx.state, owner, "_exp2skill_stamp") != turn_stamp(ctx.state)
            }
            _ => false,
        },
        "_useskill" => matches!(
            event,
            EventData::CardUsed(u) if u.player == owner && !u.cancelled
        ),
        "_killskill" => matches!(
            event,
            EventData::Death {
                killer: Some(k),
                ..
            } if *k == owner
        ),
        "_relifeskill" => match event {
            EventData::Dying { player, .. } | EventData::Death { player, .. } => {
                *player == owner
                    && ctx.state.player(owner).hp <= 0
                    && flag(ctx.state, owner, "zuan_relife") > 0
            }
            _ => false,
        },
        "_yuanqiskill" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } if *player == owner
                && flag(ctx.state, owner, "_yuanqiskill_used_stamp") != turn_stamp(ctx.state)
                && (flag(ctx.state, owner, "zuan_exp") >= 100
                    || !removable_skills(ctx.state, ctx.catalog, owner).is_empty())
        ),
        "_zhongzhubuff" => match event {
            EventData::Dying { player, .. } | EventData::Death { player, .. } => {
                *player == owner
                    && ctx.state.player(owner).hp <= 0
                    && find_zhu_seat(ctx.state, owner)
                        .is_some_and(|z| ctx.state.player(z).max_hp > 3)
            }
            _ => false,
        },
        "_jingchengadddie" => match event {
            EventData::Death { player, .. } => {
                flag(ctx.state, *player, "is_zhu") == 0
                    && !matches!(
                        player_role(ctx.state, *player),
                        Some(Role::Lord | Role::Loyalist)
                    )
            }
            _ => false,
        },
        "_zhongfandieskill" => match event {
            EventData::Dying { player, .. } | EventData::Death { player, .. } => {
                *player == owner && ctx.state.player(owner).hp <= 0
            }
            _ => false,
        },
        "_tamodeskill" => match event {
            EventData::TurnBegin { player } => *player == owner,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => *player == owner,
            _ => false,
        },
        "_zhudieskill" => match event {
            EventData::Dying { player, .. } | EventData::Death { player, .. } => {
                *player == owner
                    && ctx.state.player(owner).hp <= 0
                    && find_tower_nei(ctx.state, ctx.catalog, owner).is_some()
            }
            _ => false,
        },
        "_qunxiongzhengbaskill" => match event {
            EventData::Dying { player, .. } | EventData::Death { player, .. } => {
                *player == owner && ctx.state.player(owner).hp <= 0
            }
            _ => false,
        },
        "_guanmodeskill" => match event {
            EventData::TurnOverBefore { player, .. } | EventData::TurnOvered { player, .. } => {
                *player == owner
            }
            EventData::TurnBegin { player }
            | EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                *player == owner
                    && flag(ctx.state, owner, "_guanmodeskill_stamp") != turn_stamp(ctx.state)
            }
            _ => false,
        },
        "_guandieskill" => match event {
            EventData::Dying { player, .. } | EventData::Death { player, .. } => {
                *player == owner && ctx.state.player(owner).hp <= 0
            }
            _ => false,
        },
        "_guixinmodeskill" => match event {
            EventData::Dying {
                player,
                source: Some(src),
            } => *src == owner && *player != owner && ctx.state.player(*player).hp <= 0,
            EventData::Death {
                player,
                killer: Some(src),
            } => *src == owner && *player != owner,
            _ => false,
        },
        "_huanhaiskill" => match event {
            EventData::TurnBegin { player }
            | EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                *player == owner
                    && flag(ctx.state, owner, "_huanhaiskill_stamp") != turn_stamp(ctx.state)
            }
            _ => false,
        },
        "_seadreammodeskill" => match event {
            EventData::Dying { player, .. } | EventData::Death { player, .. } => {
                ctx.state.player(*player).hp <= 0
                    && (*player == owner
                        || player_role(ctx.state, *player) == Some(Role::Rebel)
                        || flag(ctx.state, *player, "is_fan") == 1)
            }
            _ => false,
        },
        "_dietime" => match event {
            EventData::Phase {
                player,
                phase: Phase::End,
                ..
            }
            | EventData::TurnEnded { player } => {
                *player == owner
                    && flag(ctx.state, owner, "_dietime_stamp") != turn_stamp(ctx.state)
            }
            _ => false,
        },
        "_anshaskill" => match event {
            EventData::Dying { player, .. } | EventData::Death { player, .. } => {
                *player == owner && ctx.state.player(owner).hp <= 0
            }
            _ => false,
        },
        _ => false,
    }
}

pub(in crate::content::host) fn activate_l19_batch_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    if !ctx.state.player(owner).alive {
        return None;
    }
    let can_activate = match skill.name.as_str() {
        "_yuanqiskill" => {
            flag(ctx.state, owner, "_yuanqiskill_used_stamp") != turn_stamp(ctx.state)
                && (flag(ctx.state, owner, "zuan_exp") >= 100
                    || !removable_skills(ctx.state, ctx.catalog, owner).is_empty())
        }
        _ => false,
    };
    if !can_activate {
        return None;
    }
    Some(Box::new(HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L19DoomsdayBatchB,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

pub(in crate::content::host) fn machine_l19_batch_b(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut targets = Vec::new();
    match event {
        EventData::Dying { player, source } => {
            target = *source;
            targets.push(*player);
        }
        EventData::Death { player, killer } => {
            target = *killer;
            targets.push(*player);
        }
        EventData::Damage(d) => {
            target = Some(d.target);
            card = Some(d.amount.max(1) as u32);
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L19DoomsdayBatchB,
        owner,
        target,
        card,
        cards: Vec::new(),
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l19_batch_b(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    let ask_id = flag(ctx.core, owner, "l19_ask_id");
    if m.stage != HostStage::Act && ctx.answer().is_empty() {
        if l19_batch_b_asks::is_first_cancellable(ask_id) {
            refund_once_used(ctx, owner, &m.def_key);
        }
        set_flag(ctx.core, owner, "l19_ask_id", 0);
        return Flow::Done;
    }
    match m.def_key.as_str() {
        "_levelskill" => {
            check_and_apply_level_up(ctx, owner);
            Flow::Done
        }
        "_expskill" => {
            let amt = m.card.unwrap_or(1).max(1);
            let exp = flag(ctx.core, owner, "zuan_exp") + 20 * amt;
            let score = flag(ctx.core, owner, "zuan_score") + 10 * amt;
            set_flag(ctx.core, owner, "zuan_exp", exp);
            set_flag(ctx.core, owner, "zuan_score", score);
            check_and_apply_level_up(ctx, owner);
            Flow::Done
        }
        "_exp2skill" => {
            set_flag(ctx.core, owner, "_exp2skill_stamp", turn_stamp(ctx.core));
            let exp = flag(ctx.core, owner, "zuan_exp") + 10;
            let score = flag(ctx.core, owner, "zuan_score") + 5;
            set_flag(ctx.core, owner, "zuan_exp", exp);
            set_flag(ctx.core, owner, "zuan_score", score);
            check_and_apply_level_up(ctx, owner);
            Flow::Done
        }
        "_useskill" => {
            let exp = flag(ctx.core, owner, "zuan_exp") + 5;
            let score = flag(ctx.core, owner, "zuan_score") + 5;
            set_flag(ctx.core, owner, "zuan_exp", exp);
            set_flag(ctx.core, owner, "zuan_score", score);
            check_and_apply_level_up(ctx, owner);
            Flow::Done
        }
        "_killskill" => {
            ctx.start_draw(owner, 3);
            let exp = flag(ctx.core, owner, "zuan_exp") + 100;
            let score = flag(ctx.core, owner, "zuan_score") + 100;
            set_flag(ctx.core, owner, "zuan_exp", exp);
            set_flag(ctx.core, owner, "zuan_score", score);
            check_and_apply_level_up(ctx, owner);
            Flow::Done
        }
        "_relifeskill" => l19_batch_b_asks::step_relifeskill(owner, ctx),
        "_yuanqiskill" => l19_batch_b_asks::step_yuanqiskill(m, ctx),
        "_zhongzhubuff" => {
            if ctx.core.player(owner).hp <= 0 {
                if let Some(zhu) = find_zhu_seat(ctx.core, owner) {
                    if ctx.core.player(zhu).max_hp > 3 {
                        let zhu_hp_before = ctx.core.player(zhu).hp;
                        ctx.lose_max_hp(zhu, 1);
                        ctx.core.player_mut(zhu).hp = (zhu_hp_before - 1)
                            .min(ctx.core.player(zhu).max_hp as i32)
                            .max(1);
                        let max_hp = ctx.core.player(owner).max_hp.max(2);
                        revive_player_to(ctx, owner, max_hp, 2);
                        ctx.start_draw(owner, 2);
                        set_flag(ctx.core, owner, "_zhongzhubuff_triggered", 1);
                    }
                }
            }
            Flow::Done
        }
        "_jingchengadddie" => {
            let cur = flag(ctx.core, owner, "jingcheng_fan_kills")
                .max(flag(ctx.core, 0, "jingcheng_fan_kills"));
            let next = cur + 1;
            set_flag(ctx.core, owner, "jingcheng_fan_kills", next);
            set_flag(ctx.core, 0, "jingcheng_fan_kills", next);
            if next >= 7 {
                set_flag(ctx.core, owner, "jingcheng_humans_win", 1);
                set_flag(ctx.core, 0, "jingcheng_humans_win", 1);
            }
            Flow::Done
        }
        "_zhongfandieskill" => {
            if ctx.core.player(owner).hp <= 0 {
                let max_hp = ctx.core.player(owner).max_hp.max(4);
                revive_player_to(ctx, owner, max_hp, max_hp);
                ctx.start_draw(owner, 4);
                let cnt = flag(ctx.core, owner, "_zhongfandieskill_respawned") + 1;
                set_flag(ctx.core, owner, "_zhongfandieskill_respawned", cnt);
            }
            Flow::Done
        }
        "_tamodeskill" => {
            set_flag(ctx.core, owner, "_tamodeskill_untouchable", 1);
            set_flag(ctx.core, owner, "is_tower_nei", 1);
            if ctx.core.turn.current == owner && ctx.core.turn.phase == Phase::Play {
                ctx.core.turn.skip_play_phase = true;
                ctx.record_phase_skipped(owner);
                ctx.core.turn.phase = Phase::Discard;
                set_flag(ctx.core, owner, "_tamodeskill_fired", 1);
                let enemy = ctx
                    .core
                    .seat_order_from(owner)
                    .into_iter()
                    .find(|&p| {
                        p != owner
                            && ctx.core.player(p).alive
                            && (player_role(ctx.core, p) == Some(Role::Rebel)
                                || flag(ctx.core, p, "is_fan") == 1)
                    })
                    .or_else(|| {
                        ctx.core
                            .seat_order_from(owner)
                            .into_iter()
                            .find(|&p| p != owner && ctx.core.player(p).alive)
                    });
                if let Some(target) = enemy {
                    let first_card = ctx
                        .core
                        .player(target)
                        .hand
                        .first()
                        .copied()
                        .or_else(|| ctx.core.player(target).equips.first().copied());
                    if let Some(cid) = first_card {
                        discard_cards(ctx, target, &[cid]);
                    }
                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                        Some(owner),
                        target,
                        1,
                    )));
                }
            }
            Flow::Done
        }
        "_zhudieskill" => {
            if ctx.core.player(owner).hp <= 0 {
                if let Some(tower) = find_tower_nei(ctx.core, ctx.catalog, owner) {
                    let t_hp = ctx.core.player(tower).hp;
                    if ctx.core.player(tower).max_hp > 1 {
                        ctx.lose_max_hp(tower, 1);
                        ctx.core.player_mut(tower).hp = (t_hp - 1)
                            .min(ctx.core.player(tower).max_hp as i32)
                            .max(0);
                    } else {
                        ctx.core.player_mut(tower).max_hp = 0;
                        ctx.core.player_mut(tower).hp = 0;
                    }
                    let new_max = ctx.core.player(owner).max_hp + 1;
                    let new_hp = (ctx.core.player(owner).hp + 1).max(1) as u32;
                    revive_player_to(ctx, owner, new_max, new_hp);
                    set_flag(ctx.core, owner, "_zhudieskill_saved", 1);
                }
            }
            Flow::Done
        }
        "_qunxiongzhengbaskill" => {
            if ctx.core.player(owner).hp <= 0 {
                let other_alive: Vec<PlayerId> = (0..ctx.core.players.len() as PlayerId)
                    .filter(|&p| p != owner && ctx.core.player(p).alive && ctx.core.player(p).hp > 0)
                    .collect();
                let respawns_left = flag(ctx.core, owner, "zhengba_respawns_left");
                if other_alive.len() <= 1 && respawns_left == 0 {
                    set_flag(ctx.core, owner, "zhengba_win", 1);
                    set_flag(ctx.core, 0, "zhengba_win", 1);
                    if let Some(&surv) = other_alive.first() {
                        set_flag(ctx.core, surv, "zhengba_win", 1);
                    }
                } else if respawns_left > 0
                    || flag(ctx.core, owner, "_qunxiongzhengbaskill_respawned") == 0
                {
                    set_flag(
                        ctx.core,
                        owner,
                        "zhengba_respawns_left",
                        respawns_left.saturating_sub(1),
                    );
                    let max_hp = ctx.core.player(owner).max_hp.max(4);
                    revive_player_to(ctx, owner, max_hp, 4);
                    ctx.start_draw(owner, 4);
                    set_flag(ctx.core, owner, "_qunxiongzhengbaskill_respawned", 1);
                }
            }
            Flow::Done
        }
        "_guanmodeskill" => {
            set_flag(ctx.core, owner, "_guanmodeskill_immune", 1);
            ctx.core.player_mut(owner).face_down = false;
            if ctx.res.stack.last().is_some_and(|f| {
                matches!(f.data, EventData::TurnOverBefore { player, .. } if player == owner)
            }) {
                let (_, cancelled) = ctx.current_turn_over_before_mut();
                *cancelled = true;
                return Flow::Done;
            }
            if ctx.res.stack.last().is_some_and(|f| {
                matches!(f.data, EventData::TurnOvered { player, .. } if player == owner)
            }) {
                ctx.core.player_mut(owner).face_down = false;
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "_guanmodeskill_stamp", turn_stamp(ctx.core));
            let alive_count = ctx.core.alive_players().count() as u32;
            let draw_n = alive_count.saturating_sub(1);
            if draw_n > 0 {
                ctx.start_draw(owner, draw_n);
            }
            set_flag(ctx.core, owner, "_guanmodeskill_fired", 1);
            if let Some(target) = ctx
                .core
                .seat_order_from(owner)
                .into_iter()
                .find(|&p| p != owner && ctx.core.player(p).alive)
            {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    target,
                    1,
                )));
            }
            Flow::Done
        }
        "_guandieskill" => l19_batch_b_asks::step_guandieskill(owner, ctx),
        "_guixinmodeskill" => l19_batch_b_asks::step_guixinmodeskill(m, ctx),
        "_huanhaiskill" => {
            set_flag(ctx.core, owner, "_huanhaiskill_stamp", turn_stamp(ctx.core));
            let stage = flag(ctx.core, owner, "huanhai_stage")
                .max(flag(ctx.core, 0, "huanhai_stage"))
                .max(1);
            match stage {
                2 => {
                    ctx.recover(owner, 1, None);
                    set_flag(ctx.core, owner, "_huanhaiskill_stage_applied", 2);
                }
                3 => {
                    set_flag(ctx.core, owner, "_huanhaiskill_stage_applied", 3);
                    ctx.emit(EventData::LoseHp {
                        player: owner,
                        amount: 1,
                    });
                }
                s if s >= 4 => {
                    let _ = grant_rpg_pool_skill(ctx, owner);
                    ctx.core.player_mut(owner).max_hp += 1;
                    ctx.recover(owner, 1, None);
                    ctx.start_draw(owner, 1);
                    set_flag(ctx.core, owner, "_huanhaiskill_stage_applied", s);
                }
                _ => {
                    set_flag(ctx.core, owner, "_huanhaiskill_stage_applied", 1);
                }
            }
            Flow::Done
        }
        "_seadreammodeskill" => l19_batch_b_asks::step_seadreammodeskill(m, ctx),
        "_dietime" => {
            set_flag(ctx.core, owner, "_dietime_stamp", turn_stamp(ctx.core));
            let next = flag(ctx.core, owner, "dietime") + 1;
            set_flag(ctx.core, owner, "dietime", next);
            if next >= 5 {
                set_flag(ctx.core, owner, "_dietime_expired", 1);
                ctx.core.player_mut(owner).hp = 0;
                ctx.emit(EventData::Death {
                    player: owner,
                    killer: None,
                });
            }
            Flow::Done
        }
        "_anshaskill" => l19_batch_b_asks::step_anshaskill(m, ctx),
        _ => Flow::Done,
    }
}
