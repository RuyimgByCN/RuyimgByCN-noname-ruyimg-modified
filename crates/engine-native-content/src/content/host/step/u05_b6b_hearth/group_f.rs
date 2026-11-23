//! Batch U05-6b (`shahai-hearth` Part 2) Group F: 16 技能 (`tuteng_s` .. `hongxi`)。

use super::asks::{
    ASK_B6B_TUTENG7_TARGET, ASK_B6B_TUTENG_PICK, ASK_B6B_TUTENG_REPLACE, clear_b6b_ask, wait_b6b,
};
use super::{
    b6b_flag, clear_b6b_flag, hs_b6b_cnt, hs_b6b_inc, pop_stashed_card, set_b6b_flag,
    stash_card, tuteng_owned, tuteng_pool,
};
use super::super::u05_hearth::{
    alive_others, grant_skill, is_delayed_trick_hearth, meta, remove_skill, rng_pick,
};
use super::super::{HostMachine, HostSkillBehavior, HostStage};
use crate::catalog::{QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{EventData, EventKind, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow};

pub(super) fn trigger_meta_f(name: &str) -> Option<TriggerMeta> {
    match name {
        "tuteng_s" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new())),
        "s_tuteng" | "zuling" => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        "huanfeng" => Some(meta(
            EventKind::TurnBegin,
            Timing::Normal,
            10,
            false,
            vec![EventKind::TurnPhaseEnd, EventKind::Death],
        )),
        "tuteng_lose" => Some(meta(EventKind::DamageEnd, Timing::Normal, 0, false, Vec::new())),
        "tuteng1" | "tuteng5" | "tuteng7" => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        "tuteng2" => Some(meta(EventKind::DamageAfter, Timing::Normal, 0, false, Vec::new())),
        "tuteng3" | "tuteng4" | "tuteng6" => {
            Some(meta(EventKind::DamageBefore, Timing::Normal, 0, false, Vec::new()))
        }
        "hongxi" => Some(meta(EventKind::Death, Timing::Normal, 0, false, Vec::new())),
        // `tuteng` 为主动技；`tuteng_h` / `tuteng8` 为被动属性修订。
        _ => None,
    }
}

pub(super) fn limits_f(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(super) fn has_active_f(name: &str) -> bool {
    name == "tuteng"
}

pub(super) fn filter_f(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let st = ctx.state;
    match skill.name.as_str() {
        "tuteng_s" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Play, begin: true }
                if *player == owner && (1..=4u8).any(|i| {
                    !super::super::u05_hearth::has_any_skill(st, ctx.catalog, owner, &format!("tuteng{i}"))
                })
        ),
        "s_tuteng" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Prepare, begin: true } if *player == owner
        ),
        "zuling" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Prepare, begin: true }
                if *player == owner
                    && b6b_flag(st, owner, "hs_tuteng_awake") == 0
                    && tuteng_owned(st, ctx.catalog, owner).len() >= 3
        ),
        "huanfeng" => match event {
            EventData::TurnBegin { player } if *player == owner => (1..=4u8)
                .all(|i| super::super::u05_hearth::has_any_skill(st, ctx.catalog, owner, &format!("tuteng{i}"))),
            EventData::Death { player, .. } if *player == owner => {
                b6b_flag(st, owner, "hs_huanfeng_active") > 0
            }
            EventData::TurnEndWindow { player, after: false } if *player == owner => {
                b6b_flag(st, owner, "hs_huanfeng_active") > 0
                    || b6b_flag(st, owner, "hs_huanfeng_pending_extra") > 0
            }
            _ => false,
        },
        "tuteng_lose" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner && !tuteng_owned(st, ctx.catalog, owner).is_empty()
        ),
        "tuteng1" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true }
                if *player == owner && st.player(owner).hp < st.player(owner).max_hp as i32
        ),
        "tuteng2" => matches!(event, EventData::Damage(d) if d.source == Some(owner)),
        "tuteng3" => matches!(
            event,
            EventData::Damage(d) if d.target == owner && d.amount > 0
        ),
        "tuteng4" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && st.turn.current == owner
                    && !d.from_chain
                    && hs_b6b_cnt(st, owner, "tuteng4") == 0
                    && d.card.is_some_and(|c| {
                        let k = ctx.catalog.card_kind(c);
                        !k.is_basic_kind() && k.equip_slot().is_none() && !is_delayed_trick_hearth(k)
                    })
        ),
        "tuteng5" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true } if *player == owner
        ),
        "tuteng6" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && st.turn.current == owner
                    && !d.from_chain
                    && hs_b6b_cnt(st, owner, "tuteng6") == 0
                    && d.card.is_some_and(|c| ctx.catalog.card_kind(c).is_sha_like())
        ),
        "tuteng7" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true }
                if *player == owner
                    && alive_others(st, owner)
                        .into_iter()
                        .any(|p| st.player(p).hp < st.player(p).max_hp as i32)
        ),
        "hongxi" => matches!(
            event,
            EventData::Death { .. }
                if st.player(owner).alive && st.player(owner).hp < st.player(owner).max_hp as i32
        ),
        _ => false,
    }
}

pub(super) fn activate_f(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<bool> {
    let st = ctx.state;
    if skill.name != "tuteng" || hs_b6b_cnt(st, owner, "tuteng") > 0 {
        return (skill.name == "tuteng").then_some(false);
    }
    let awake = b6b_flag(st, owner, "hs_tuteng_awake") > 0;
    let owned = tuteng_owned(st, ctx.catalog, owner);
    let has_missing = tuteng_pool(awake).into_iter().any(|i| !owned.contains(&i));
    Some(awake || has_missing)
}

fn begin_huanfeng(ctx: &mut Ctx<'_>, owner: PlayerId) {
    for i in 1..=4 {
        remove_skill(ctx, owner, &format!("tuteng{i}"));
    }
    let stashed_hand = std::mem::take(&mut ctx.core.player_mut(owner).hand);
    for c in stashed_hand {
        stash_card(ctx.core, owner, "hs_huanfeng_hand", c);
    }
    let stashed_eq = std::mem::take(&mut ctx.core.player_mut(owner).equips);
    for c in stashed_eq {
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, c);
        stash_card(ctx.core, owner, "hs_huanfeng_eq", c);
    }
    let orig_hp = ctx.core.player(owner).hp;
    let orig_max = ctx.core.player(owner).max_hp;
    let orig_hujia = ctx.core.player(owner).hujia;
    set_b6b_flag(
        ctx.core,
        owner,
        "hs_huanfeng_orig_hp",
        (orig_hp.max(-500) + 1000) as u32,
    );
    set_b6b_flag(ctx.core, owner, "hs_huanfeng_orig_max", orig_max);
    set_b6b_flag(ctx.core, owner, "hs_huanfeng_orig_hujia", u32::from(orig_hujia));

    let keep_self = ctx.catalog.skill_by_name("huanfeng");
    let stashed_skills: Vec<(crate::ids::SkillDefId, bool, bool, u8)> = ctx
        .core
        .skills
        .iter()
        .filter(|(&(o, d), _)| o == owner && Some(d) != keep_self)
        .map(|(&(_, d), inst)| (d, inst.enabled, inst.once_used, inst.rounds))
        .collect();
    for (d, en, ou, r) in stashed_skills {
        ctx.core.skills.remove(&(owner, d));
        let packed = (u32::from(r) << 4) | (u32::from(ou) << 2) | (u32::from(en) << 1) | 1;
        set_b6b_flag(ctx.core, owner, &format!("hs_huanfeng_sk:{d}"), packed);
    }

    let p = ctx.core.player_mut(owner);
    p.max_hp = 3;
    p.hp = 3;
    p.hujia = 0;
    grant_skill(ctx, owner, "fengnu", true);
    grant_skill(ctx, owner, "shengdun", true);
    set_b6b_flag(ctx.core, owner, "hs_huanfeng_active", 1);
    if b6b_flag(ctx.core, owner, "hs_active_extra_turn_source") != 3 {
        set_b6b_flag(ctx.core, owner, "hs_huanfeng_pending_extra", 1);
    }
    ctx.start_draw(owner, 4);
}

fn restore_huanfeng_subplayer(ctx: &mut Ctx<'_>, owner: PlayerId) {
    clear_b6b_flag(ctx.core, owner, "hs_huanfeng_active");
    remove_skill(ctx, owner, "fengnu");
    remove_skill(ctx, owner, "shengdun");
    let sub_hand = std::mem::take(&mut ctx.core.player_mut(owner).hand);
    for c in sub_hand {
        ctx.to_discard(c);
    }
    let sub_eq = std::mem::take(&mut ctx.core.player_mut(owner).equips);
    for c in sub_eq {
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, c);
        ctx.to_discard(c);
    }
    while let Some(c) = pop_stashed_card(ctx.core, owner, "hs_huanfeng_hand") {
        ctx.core.player_mut(owner).hand.push(c);
    }
    while let Some(c) = pop_stashed_card(ctx.core, owner, "hs_huanfeng_eq") {
        ctx.core.player_mut(owner).equips.push(c);
        crate::kernel::grant_equip_skills(ctx, owner, c);
    }
    let orig_max = b6b_flag(ctx.core, owner, "hs_huanfeng_orig_max").max(1);
    let orig_hp = b6b_flag(ctx.core, owner, "hs_huanfeng_orig_hp") as i32 - 1000;
    let orig_hujia = b6b_flag(ctx.core, owner, "hs_huanfeng_orig_hujia") as u8;
    clear_b6b_flag(ctx.core, owner, "hs_huanfeng_orig_max");
    clear_b6b_flag(ctx.core, owner, "hs_huanfeng_orig_hp");
    clear_b6b_flag(ctx.core, owner, "hs_huanfeng_orig_hujia");
    let p = ctx.core.player_mut(owner);
    p.max_hp = orig_max;
    p.hp = orig_hp.min(orig_max as i32);
    p.hujia = orig_hujia;

    let keys: Vec<(crate::ids::SkillDefId, u32)> = ctx
        .core
        .vm_aux
        .flags
        .iter()
        .filter_map(|((s, k), &v)| {
            if *s == owner && v > 0 {
                k.strip_prefix("hs_huanfeng_sk:")
                    .and_then(|n| n.parse::<crate::ids::SkillDefId>().ok())
                    .map(|d| (d, v))
            } else {
                None
            }
        })
        .collect();
    for (d, packed) in keys {
        ctx.core
            .vm_aux
            .flags
            .remove(&(owner, format!("hs_huanfeng_sk:{d}")));
        ctx.ensure_skill(
            (owner, d),
                crate::state::SkillInstance {
                    owner,
                    enabled: (packed & 2) != 0,
                    once_used: (packed & 4) != 0,
                    expires_end_of_turn: false,
                    rounds: ((packed >> 4) & 0xff) as u8,
                },
            );
        }
}

fn queue_huanfeng_extra_turn(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if b6b_flag(ctx.core, owner, "hs_huanfeng_pending_extra") > 0 {
        clear_b6b_flag(ctx.core, owner, "hs_huanfeng_pending_extra");
        if b6b_flag(ctx.core, owner, "hs_active_extra_turn_source") != 3 {
            set_b6b_flag(ctx.core, owner, "extra_turn_insert_queued", 1);
            set_b6b_flag(ctx.core, owner, "hs_queued_extra_turn_source", 3);
        }
    }
}

pub(super) fn step_f(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "tuteng_s" => {
            let owned = tuteng_owned(ctx.core, ctx.catalog, m.owner);
            let avail: Vec<u8> = (1..=4u8).filter(|i| !owned.contains(i)).collect();
            if let Some(pick) = rng_pick(ctx, &avail) {
                grant_skill(ctx, m.owner, &format!("tuteng{pick}"), false);
            }
            Flow::Done
        }
        "s_tuteng" => {
            let owned = tuteng_owned(ctx.core, ctx.catalog, m.owner);
            if owned.len() >= 3 {
                if let Some(rm) = rng_pick(ctx, &owned) {
                    remove_skill(ctx, m.owner, &format!("tuteng{rm}"));
                }
            }
            let cur_owned = tuteng_owned(ctx.core, ctx.catalog, m.owner);
            let avail: Vec<u8> = (1..=8u8).filter(|i| !cur_owned.contains(i)).collect();
            // `rand.randomGet("tuteng1", "tuteng3")`：优先排除 `1` 和 `3`。
            let preferred: Vec<u8> = avail
                .iter()
                .copied()
                .filter(|&i| i != 1 && i != 3)
                .collect();
            let pool = if preferred.is_empty() { &avail } else { &preferred };
            if let Some(pick) = rng_pick(ctx, pool) {
                grant_skill(ctx, m.owner, &format!("tuteng{pick}"), false);
            }
            Flow::Done
        }
        "tuteng" => match m.stage {
            HostStage::Act => {
                if hs_b6b_cnt(ctx.core, m.owner, "tuteng") > 0 {
                    return Flow::Done;
                }
                let awake = b6b_flag(ctx.core, m.owner, "hs_tuteng_awake") > 0;
                let owned = tuteng_owned(ctx.core, ctx.catalog, m.owner);
                let avail: Vec<u8> = tuteng_pool(awake)
                    .into_iter()
                    .filter(|i| !owned.contains(i))
                    .collect();
                if avail.is_empty() {
                    return Flow::Done;
                }
                if !awake {
                    hs_b6b_inc(ctx.core, m.owner, "tuteng");
                    if let Some(pick) = rng_pick(ctx, &avail) {
                        grant_skill(ctx, m.owner, &format!("tuteng{pick}"), false);
                    }
                    return Flow::Done;
                }
                if avail.len() == 1 && owned.len() < 4 {
                    hs_b6b_inc(ctx.core, m.owner, "tuteng");
                    grant_skill(ctx, m.owner, &format!("tuteng{}", avail[0]), false);
                    return Flow::Done;
                }
                wait_b6b(m, ctx, HostStage::SelectNature, ASK_B6B_TUTENG_PICK)
            }
            HostStage::SelectNature => {
                clear_b6b_ask(m, ctx, HostStage::SelectNature);
                let Some(ChoiceOption::Tag(idx)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let owned = tuteng_owned(ctx.core, ctx.catalog, m.owner);
                if owned.len() >= 4 {
                    m.card = Some(u32::from(idx));
                    return wait_b6b(m, ctx, HostStage::TargetChoice, ASK_B6B_TUTENG_REPLACE);
                }
                hs_b6b_inc(ctx.core, m.owner, "tuteng");
                grant_skill(ctx, m.owner, &format!("tuteng{idx}"), false);
                Flow::Done
            }
            HostStage::TargetChoice => {
                clear_b6b_ask(m, ctx, HostStage::TargetChoice);
                let Some(new_idx) = m.card else {
                    return Flow::Done;
                };
                if let Some(ChoiceOption::Tag(old_idx)) = ctx.answer().first().copied() {
                    hs_b6b_inc(ctx.core, m.owner, "tuteng");
                    remove_skill(ctx, m.owner, &format!("tuteng{old_idx}"));
                    grant_skill(ctx, m.owner, &format!("tuteng{new_idx}"), false);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "zuling" => {
            if b6b_flag(ctx.core, m.owner, "hs_tuteng_awake") == 0
                && tuteng_owned(ctx.core, ctx.catalog, m.owner).len() >= 3
            {
                set_b6b_flag(ctx.core, m.owner, "hs_tuteng_awake", 1);
                ctx.lose_max_hp(m.owner, 1);
            }
            Flow::Done
        }
        "huanfeng" => {
            let cur = ctx.res.stack.last().map(|f| f.data.clone());
            match cur {
                Some(EventData::TurnBegin { player }) if player == m.owner => {
                    if (1..=4u8).all(|i| {
                        super::super::u05_hearth::has_any_skill(
                            ctx.core,
                            ctx.catalog,
                            m.owner,
                            &format!("tuteng{i}"),
                        )
                    }) {
                        begin_huanfeng(ctx, m.owner);
                    }
                }
                Some(EventData::Death { player, .. }) if player == m.owner => {
                    if b6b_flag(ctx.core, m.owner, "hs_huanfeng_active") > 0 {
                        ctx.core.player_mut(m.owner).death_cancelled = true;
                        restore_huanfeng_subplayer(ctx, m.owner);
                    }
                }
                Some(EventData::TurnEndWindow {
                    player,
                    after: false,
                }) if player == m.owner => {
                    if b6b_flag(ctx.core, m.owner, "hs_huanfeng_active") > 0 {
                        restore_huanfeng_subplayer(ctx, m.owner);
                    }
                    queue_huanfeng_extra_turn(ctx, m.owner);
                }
                _ => {}
            }
            Flow::Done
        }
        "tuteng_lose" => {
            let owned = tuteng_owned(ctx.core, ctx.catalog, m.owner);
            if let Some(rm) = rng_pick(ctx, &owned) {
                remove_skill(ctx, m.owner, &format!("tuteng{rm}"));
            }
            Flow::Done
        }
        "tuteng1" => {
            if ctx.core.player(m.owner).hp < ctx.core.player(m.owner).max_hp as i32 {
                ctx.recover(m.owner, 1, None);
            }
            Flow::Done
        }
        "tuteng2" => {
            ctx.start_draw(m.owner, 1);
            Flow::Done
        }
        "tuteng3" => {
            if ctx.current_damage().is_ok_and(|d| d.amount > 0) {
                ctx.modify_damage_amount(-1);
                remove_skill(ctx, m.owner, "tuteng3");
            }
            Flow::Done
        }
        "tuteng4" => {
            if hs_b6b_cnt(ctx.core, m.owner, "tuteng4") == 0 && ctx.current_damage().is_ok() {
                hs_b6b_inc(ctx.core, m.owner, "tuteng4");
                ctx.modify_damage_amount(1);
            }
            Flow::Done
        }
        "tuteng5" => {
            ctx.start_draw(m.owner, 1);
            Flow::Done
        }
        "tuteng6" => {
            if hs_b6b_cnt(ctx.core, m.owner, "tuteng6") == 0 && ctx.current_damage().is_ok() {
                hs_b6b_inc(ctx.core, m.owner, "tuteng6");
                ctx.modify_damage_amount(1);
            }
            Flow::Done
        }
        "tuteng7" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::TargetChoice, ASK_B6B_TUTENG7_TARGET),
            HostStage::TargetChoice => {
                clear_b6b_ask(m, ctx, HostStage::TargetChoice);
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    if ctx.core.player(t).alive && ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32 {
                        ctx.recover(t, 1, None);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "hongxi" => {
            let p = ctx.core.player(m.owner);
            let miss = (p.max_hp as i32 - p.hp).max(0) as u32;
            if miss > 0 {
                ctx.recover(m.owner, miss, None);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
