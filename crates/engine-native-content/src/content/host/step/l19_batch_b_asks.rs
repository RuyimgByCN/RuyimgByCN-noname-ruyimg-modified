//! L19 末日乱斗 Batch B 询问构造与长步骤辅助（`l19_batch_b_asks.rs`）。
//!
//! 承载提示词：
//! - `"_yuanqiskill:choose_mode"`（`extension.js:2422`）：选择消耗 100 经验获得技能（0）或移除技能摸 2 回 1（1）
//! - `"_yuanqiskill:choose_skill"`（`extension.js:2422`）：从技能池候选（`0..=2`）中选择 1 个技能获得
//! - `"_yuanqiskill:remove_skill"`（`extension.js:2422`）：选择自身 1 个可移除技能移除，摸 2 张牌并回复 1 点体力

use super::super::*;
use super::{
    flag, grant_rpg_pool_skill, grant_skill_named, has_skill_named, player_role,
    refund_once_used, removable_skills, remove_skill_def, revive_player_to, set_flag,
    set_player_role, turn_stamp, ASK_YUANQI_MODE, ASK_YUANQI_REMOVE, ASK_YUANQI_SKILL,
    RPG_SKILL_POOL,
};
use crate::catalog::Catalog;
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::ExpectedAsk;
use crate::state::{GameStateCore, Role};

pub(super) fn is_first_cancellable(ask_id: u32) -> bool {
    ask_id == ASK_YUANQI_MODE
}

fn build_ask(
    ask_id: u32,
    owner: PlayerId,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        ASK_YUANQI_MODE => {
            let mut opts = Vec::new();
            if flag(core, owner, "zuan_exp") >= 100 {
                opts.push(ChoiceOption::Tag(0));
            }
            if !removable_skills(core, catalog, owner).is_empty() {
                opts.push(ChoiceOption::Tag(1));
            }
            if opts.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "_yuanqiskill:choose_mode",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_YUANQI_SKILL => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "_yuanqiskill:choose_skill",
                options: vec![
                    ChoiceOption::Tag(0),
                    ChoiceOption::Tag(1),
                    ChoiceOption::Tag(2),
                ],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        )),
        ASK_YUANQI_REMOVE => {
            let rem = removable_skills(core, catalog, owner);
            if rem.is_empty() {
                return None;
            }
            let opts: Vec<ChoiceOption> = (0..rem.len())
                .map(|i| ChoiceOption::Tag(i as u8))
                .collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "_yuanqiskill:remove_skill",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        _ => None,
    }
}

pub(super) fn wait_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    next_stage: HostStage,
    ask_id: u32,
) -> Flow {
    set_flag(ctx.core, m.owner, "l19_ask_id", ask_id);
    if let Some((observer, spec)) = build_ask(ask_id, m.owner, ctx.core, ctx.catalog) {
        m.stage = next_stage;
        Flow::WaitChoice { observer, spec }
    } else {
        set_flag(ctx.core, m.owner, "l19_ask_id", 0);
        Flow::Done
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_l19_batch_b(
    stage: HostStage,
    owner: PlayerId,
    _target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<CardId>,
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if stage == HostStage::Act {
        return None;
    }
    let ask_id = flag(core, owner, "l19_ask_id");
    let (observer, spec) = build_ask(ask_id, owner, core, catalog)?;
    Some(crate::snap::ask(
        observer,
        spec.prompt,
        spec.kind,
        spec.options,
        spec.min,
        spec.max,
        spec.allow_cancel,
    ))
}

pub(super) fn step_yuanqiskill(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if flag(ctx.core, owner, "_yuanqiskill_used_stamp") == turn_stamp(ctx.core)
                || (flag(ctx.core, owner, "zuan_exp") < 100
                    && removable_skills(ctx.core, ctx.catalog, owner).is_empty())
            {
                refund_once_used(ctx, owner, "_yuanqiskill");
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectNature, ASK_YUANQI_MODE)
        }
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l19_ask_id", 0);
            let mode = ctx.answer().iter().find_map(|o| match *o {
                ChoiceOption::Tag(i) => Some(i),
                _ => None,
            });
            match mode {
                Some(0) if flag(ctx.core, owner, "zuan_exp") >= 100 => {
                    let next_exp = flag(ctx.core, owner, "zuan_exp").saturating_sub(100);
                    set_flag(ctx.core, owner, "zuan_exp", next_exp);
                    wait_ask(m, ctx, HostStage::SelectCard, ASK_YUANQI_SKILL)
                }
                Some(1) if !removable_skills(ctx.core, ctx.catalog, owner).is_empty() => {
                    wait_ask(m, ctx, HostStage::TargetChoice, ASK_YUANQI_REMOVE)
                }
                _ => {
                    refund_once_used(ctx, owner, "_yuanqiskill");
                    Flow::Done
                }
            }
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l19_ask_id", 0);
            let idx = ctx
                .answer()
                .iter()
                .find_map(|o| match *o {
                    ChoiceOption::Tag(i) => Some(i as usize),
                    _ => None,
                })
                .unwrap_or(0);
            let pool = ["zhiheng", "jianxiong", "yingzi", "rende", "paoxiao", "wusheng"];
            let chosen = pool.get(idx).copied().unwrap_or("zhiheng");
            if has_skill_named(ctx.core, ctx.catalog, owner, chosen) {
                let _ = grant_rpg_pool_skill(ctx, owner);
            } else {
                let _ = grant_skill_named(ctx, owner, chosen);
            }
            set_flag(ctx.core, owner, "_yuanqiskill_gained", 1);
            set_flag(ctx.core, owner, "_yuanqiskill_used_stamp", turn_stamp(ctx.core));
            Flow::Done
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l19_ask_id", 0);
            let idx = ctx
                .answer()
                .iter()
                .find_map(|o| match *o {
                    ChoiceOption::Tag(i) => Some(i as usize),
                    _ => None,
                })
                .unwrap_or(0);
            let rem = removable_skills(ctx.core, ctx.catalog, owner);
            if let Some(&def) = rem.get(idx).or_else(|| rem.first()) {
                remove_skill_def(ctx, owner, def);
                ctx.start_draw(owner, 2);
                ctx.recover(owner, 1, None);
                set_flag(ctx.core, owner, "_yuanqiskill_removed", 1);
                set_flag(ctx.core, owner, "_yuanqiskill_used_stamp", turn_stamp(ctx.core));
            } else {
                refund_once_used(ctx, owner, "_yuanqiskill");
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn apply_guandie_zuan_exp(ctx: &mut Ctx<'_>, src: PlayerId, base_exp: u32) {
    let up = flag(ctx.core, src, "zuan_upspeed");
    let adds = if up > 0 {
        (base_exp.saturating_mul(up) + 9) / 10
    } else {
        base_exp
    };
    let mut level = flag(ctx.core, src, "_levelskill")
        .max(flag(ctx.core, src, "zuan_level"))
        .max(1);
    let mut exp = flag(ctx.core, src, "_expskill")
        .max(flag(ctx.core, src, "zuan_exp"))
        .saturating_add(adds);
    let maxexp = 20 + 2 * level;
    if exp >= maxexp && level < 10 {
        exp -= maxexp;
        level += 1;
        if flag(ctx.core, src, "zuan_balance") == 0 {
            ctx.start_draw(src, 1);
        }
        if level == 8 {
            ctx.core.player_mut(src).max_hp = ctx.core.player(src).max_hp.saturating_add(1);
        }
        let relife = (level * 4 + 1 + 9) / 10;
        set_flag(ctx.core, src, "zuan_relife", relife);
    }
    set_flag(ctx.core, src, "zuan_exp", exp);
    set_flag(ctx.core, src, "_expskill", exp);
    set_flag(ctx.core, src, "zuan_level", level);
    set_flag(ctx.core, src, "_levelskill", level);
}

pub(super) fn step_guandieskill(m: &HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if has_skill_named(ctx.core, ctx.catalog, owner, "jingcheng")
        || flag(ctx.core, owner, "_guandieskill_jingcheng") > 0
        || player_role(ctx.core, owner) != Some(Role::Lord)
    {
        return Flow::Done;
    }
    if let Some(src) = m.target {
        if src != owner && ctx.core.player(src).alive {
            apply_guandie_zuan_exp(ctx, src, 25);
        }
    }
    let skill_defs: Vec<u16> = ctx
        .core
        .skills
        .keys()
        .filter_map(|&(p, d)| (p == owner).then_some(d))
        .collect();
    for def in skill_defs {
        let name = ctx
            .catalog
            .skills
            .get(def as usize)
            .map(|s| s.name)
            .unwrap_or("");
        if !matches!(
            name,
            "thundershow" | "jingcheng" | "bazhu" | "guanfu" | "notarget"
        ) && !ctx.catalog.is_charlotte_skill(def)
        {
            if let Some(inst) = ctx.core.skills.get_mut(&(owner, def)) {
                inst.enabled = false;
            }
        }
    }
    let pj = flag(ctx.core, owner, "zuan_panjun") + 2;
    set_flag(ctx.core, owner, "zuan_panjun", pj);
    if let Some(def) = ctx.catalog.skill_by_name("zuan_panjun") {
        let entry = ctx
            .core
            .vm_aux
            .marks
            .entry((owner, u32::from(def)))
            .or_insert(0);
        *entry += 2;
    }
    let gb = flag(ctx.core, owner, "zuan_guanbing");
    let mut lethal_fan = false;
    if gb >= 10 {
        set_flag(ctx.core, owner, "taofa_guan_win", 1);
        set_flag(ctx.core, 0, "taofa_guan_win", 1);
        for p in 0..ctx.core.players.len() as PlayerId {
            if p != owner
                && ctx.core.player(p).alive
                && !matches!(player_role(ctx.core, p), Some(Role::Lord | Role::Loyalist))
            {
                ctx.core.player_mut(p).hp = 0;
                ctx.emit(EventData::Death {
                    player: p,
                    killer: None,
                });
            }
        }
    } else if pj >= 10 {
        set_flag(ctx.core, owner, "taofa_fan_win", 1);
        set_flag(ctx.core, 0, "taofa_fan_win", 1);
        lethal_fan = true;
    }
    let old_max = ctx.core.player(owner).max_hp;
    set_flag(ctx.core, owner, "zuan_zhumaxhp", old_max);
    let tamax = flag(ctx.core, owner, "zuan_tamaxhp");
    let new_max = if tamax > 0 {
        tamax
    } else {
        crate::state::INFINITE_HP
    };
    let tahp = flag(ctx.core, owner, "zuan_tahp");
    let new_hp = if tahp > 0 {
        tahp as i32
    } else if new_max == crate::state::INFINITE_HP {
        crate::state::INFINITE_HP as i32
    } else {
        new_max as i32
    };
    {
        let p = ctx.core.player_mut(owner);
        p.max_hp = new_max;
        p.hp = new_hp;
        p.linked = false;
        p.face_down = false;
        if !lethal_fan {
            p.death_cancelled = true;
            p.alive = true;
            p.dying = false;
        }
    }
    let _ = grant_skill_named(ctx, owner, "jingcheng");
    set_flag(ctx.core, owner, "_guandieskill_jingcheng", 1);
    set_flag(ctx.core, owner, "has_jingcheng", 1);
    let _ = grant_skill_named(ctx, owner, "notarget");
    set_flag(ctx.core, owner, "notarget_active", 1);
    set_player_role(ctx.core, owner, Role::Lord);
    set_flag(ctx.core, owner, "zuan_identity_wang", 1);
    let mut hej: Vec<CardId> = ctx.core.player(owner).hand.clone();
    hej.extend(ctx.core.player(owner).equips.iter().copied());
    hej.extend(ctx.core.player(owner).judge.iter().copied());
    for &c in &hej {
        ctx.take_owned_card(owner, c);
        ctx.to_discard(c);
    }
    if !hej.is_empty() {
        ctx.log(LogEvent::CardDiscarded {
            player: owner,
            cards: hej.clone(),
        });
        ctx.emit(EventData::CardsDiscarded {
            player: owner,
            cards: hej,
        });
    }
    Flow::Done
}

pub(super) fn step_guixinmodeskill(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(&victim) = m.targets.first() else {
        return Flow::Done;
    };
    if victim == owner {
        return Flow::Done;
    }
    let owner_camp = flag(ctx.core, owner, "guixin_camp").max(1);
    set_flag(ctx.core, owner, "guixin_camp", owner_camp);
    set_flag(ctx.core, victim, "guixin_camp", owner_camp);
    if player_role(ctx.core, owner) == Some(Role::Lord) {
        set_player_role(ctx.core, victim, Role::Loyalist);
    }
    let max_hp = ctx.core.player(victim).max_hp.max(1);
    revive_player_to(ctx, victim, max_hp, max_hp);
    ctx.start_draw(victim, 3);
    let conv = flag(ctx.core, owner, "_guixinmodeskill_converted") + 1;
    set_flag(ctx.core, owner, "_guixinmodeskill_converted", conv);

    let all_same = (0..ctx.core.players.len() as PlayerId)
        .filter(|&p| ctx.core.player(p).alive)
        .all(|p| flag(ctx.core, p, "guixin_camp") == owner_camp);
    if all_same {
        set_flag(ctx.core, owner, "guixin_all_converted_win", 1);
        set_flag(ctx.core, 0, "guixin_all_converted_win", 1);
    }
    Flow::Done
}

pub(super) fn step_seadreammodeskill(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let enemy = m.targets.first().copied().unwrap_or(owner);
    if ctx.core.player(enemy).hp > 0 {
        return Flow::Done;
    }
    let cur_stage = flag(ctx.core, owner, "huanhai_stage")
        .max(flag(ctx.core, enemy, "huanhai_stage"))
        .max(flag(ctx.core, 0, "huanhai_stage"))
        .max(1);
    if cur_stage < 4 {
        let next_stage = cur_stage + 1;
        for p in [owner, enemy, 0] {
            set_flag(ctx.core, p, "huanhai_stage", next_stage);
            set_flag(ctx.core, p, "_seadreammodeskill_stage", next_stage);
        }
        let new_max = 3 + next_stage;
        revive_player_to(ctx, enemy, new_max, new_max);
        for &sk in RPG_SKILL_POOL.iter().take(next_stage as usize) {
            let _ = grant_skill_named(ctx, enemy, sk);
        }
    } else {
        for p in [owner, enemy, 0] {
            set_flag(ctx.core, p, "huanhai_win", 1);
        }
    }
    Flow::Done
}

pub(super) fn step_anshaskill(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if ctx.core.player(owner).hp > 0 {
        return Flow::Done;
    }
    let killer = m.target.filter(|&k| k != owner).or_else(|| {
        (0..ctx.core.players.len() as PlayerId)
            .find(|&p| p != owner && ctx.core.player(p).alive)
    });
    let is_zhu = flag(ctx.core, owner, "ansha_is_zhu") == 1
        || (flag(ctx.core, owner, "ansha_is_fan") == 0
            && player_role(ctx.core, owner) == Some(Role::Lord));
    if is_zhu {
        if let Some(k) = killer {
            set_flag(ctx.core, k, "ansha_is_zhu", 1);
            set_flag(ctx.core, k, "ansha_is_fan", 0);
            set_player_role(ctx.core, k, Role::Lord);
            ctx.core.player_mut(k).max_hp += 1;
            let k_max = ctx.core.player(k).max_hp;
            let k_hp = ctx.core.player(k).hp;
            if k_max as i32 > k_hp {
                ctx.recover(k, (k_max as i32 - k_hp) as u32, None);
            }
            ctx.core.player_mut(k).hp = k_max as i32;
            set_flag(ctx.core, k, "dietime", 0);
        }
        set_flag(ctx.core, owner, "ansha_is_zhu", 0);
        set_flag(ctx.core, owner, "ansha_is_fan", 1);
        set_player_role(ctx.core, owner, Role::Rebel);
        set_flag(ctx.core, owner, "_anshaskill_zhu_swapped", 1);
        let lives = flag(ctx.core, owner, "ansha_lives");
        if lives > 0 {
            set_flag(ctx.core, owner, "ansha_lives", lives - 1);
            let max_hp = ctx.core.player(owner).max_hp.max(1);
            revive_player_to(ctx, owner, max_hp, max_hp);
            ctx.start_draw(owner, 4);
        }
    } else {
        if let Some(k) = killer {
            ctx.start_draw(k, 3);
            set_flag(ctx.core, k, "dietime", 0);
        }
        let lives = flag(ctx.core, owner, "ansha_lives");
        if lives > 0 {
            set_flag(ctx.core, owner, "ansha_lives", lives - 1);
            let max_hp = ctx.core.player(owner).max_hp.max(1);
            revive_player_to(ctx, owner, max_hp, max_hp);
            ctx.start_draw(owner, 4);
            set_flag(ctx.core, owner, "_anshaskill_fan_respawned", 1);
        }
    }
    Flow::Done
}
