//! L19 末日乱斗 Batch A 询问构造与交互步骤实现（`l19_batch_a_asks.rs`）。
//!
//! 承载提示词：
//! - `"ganran2:choose_equip"`（`extension.js:435`）：出牌阶段选择一张装备区外的手牌装备重铸
//! - `"_huzhu:discard_tao"`（`extension.js:698`）：出牌阶段限一次，弃置一张手牌【桃】
//! - `"_huzhu:choose_target"`（`extension.js:698`）：选择距离 1 以内一名已受伤人类盟友令其回复 1 点体力

use super::super::*;
use super::{
    discard_cards, equip_hand_cards, flag, grant_skill_named, grant_zombie_skills, huzhu_allies,
    huzhu_play_stamp, is_huzhu_human, player_role, refund_once_used, set_flag, set_player_role,
    tao_hand_cards, ASK_GANRAN2_EQUIP, ASK_HUZHU_DISCARD_TAO, ASK_HUZHU_TARGET,
    ASK_WUJIN_RE1_CONFIRM, ASK_WUJIN_RE2_SKILL,
};
use super::super::l19_fixes::collect_wujin_stealable_skills;
use crate::catalog::{CardKind, Catalog};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::ExpectedAsk;
use crate::state::{GameStateCore, Role};

pub(super) fn is_first_cancellable(ask_id: u32) -> bool {
    matches!(
        ask_id,
        ASK_GANRAN2_EQUIP | ASK_HUZHU_DISCARD_TAO | ASK_HUZHU_TARGET | ASK_WUJIN_RE1_CONFIRM
    )
}

fn build_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        ASK_GANRAN2_EQUIP => {
            let opts: Vec<ChoiceOption> = equip_hand_cards(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            if opts.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "ganran2:choose_equip",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_HUZHU_DISCARD_TAO => {
            let opts: Vec<ChoiceOption> = tao_hand_cards(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            if opts.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "_huzhu:discard_tao",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_HUZHU_TARGET => {
            let opts: Vec<ChoiceOption> = huzhu_allies(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            if opts.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "_huzhu:choose_target",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        ASK_WUJIN_RE1_CONFIRM => Some((
            0,
            ChoiceSpec {
                kind: ChoiceKind::Confirm,
                prompt: "wujinmoshiRE1:confirm",
                options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                min: 1,
                max: 1,
                allow_cancel: true,
            },
        )),
        ASK_WUJIN_RE2_SKILL => {
            let victim = target?;
            let pool = collect_wujin_stealable_skills(core, catalog, 0, victim);
            if pool.is_empty() {
                return None;
            }
            let opts: Vec<ChoiceOption> = (0..pool.len()).map(|i| ChoiceOption::Tag(i as u8)).collect();
            Some((
                0,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "wujinmoshiRE2:choose_skill",
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
    if let Some((observer, spec)) =
        build_ask(ask_id, m.owner, m.targets.first().copied(), ctx.core, ctx.catalog)
    {
        m.stage = next_stage;
        Flow::WaitChoice { observer, spec }
    } else {
        set_flag(ctx.core, m.owner, "l19_ask_id", 0);
        Flow::Done
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_l19_batch_a(
    stage: HostStage,
    owner: PlayerId,
    _target: Option<PlayerId>,
    targets: &[PlayerId],
    _card: Option<CardId>,
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if stage == HostStage::Act {
        return None;
    }
    let ask_id = flag(core, owner, "l19_ask_id");
    let (observer, spec) = build_ask(ask_id, owner, targets.first().copied(), core, catalog)?;
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

pub(super) fn step_ganran2(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if equip_hand_cards(ctx.core, ctx.catalog, owner).is_empty() {
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_GANRAN2_EQUIP)
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l19_ask_id", 0);
            let picked = ctx.answer().iter().find_map(|o| match *o {
                ChoiceOption::Card(c) => Some(c),
                _ => None,
            });
            let Some(cid) = picked else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).hand.contains(&cid) {
                return Flow::Done;
            }
            discard_cards(ctx, owner, &[cid]);
            ctx.start_draw(owner, 1);
            let c = flag(ctx.core, owner, "ganran2_recast") + 1;
            set_flag(ctx.core, owner, "ganran2_recast", c);
            set_flag(ctx.core, owner, "ganran_active", 1);
            set_flag(ctx.core, owner, "doomsday_mutated", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_huzhu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if !is_huzhu_human(ctx.core, owner)
                || flag(ctx.core, owner, "_huzhu_used_stamp") == huzhu_play_stamp(ctx.core, owner)
                || tao_hand_cards(ctx.core, ctx.catalog, owner).is_empty()
                || huzhu_allies(ctx.core, ctx.catalog, owner).is_empty()
            {
                refund_once_used(ctx, owner, "_huzhu");
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_HUZHU_DISCARD_TAO)
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, owner, "l19_ask_id", 0);
            let picked = ctx.answer().iter().find_map(|o| match *o {
                ChoiceOption::Card(c) => Some(c),
                _ => None,
            });
            let Some(tao_id) = picked else {
                refund_once_used(ctx, owner, "_huzhu");
                return Flow::Done;
            };
            if !ctx.core.player(owner).hand.contains(&tao_id)
                || ctx.catalog.card_kind(tao_id) != CardKind::Tao
            {
                refund_once_used(ctx, owner, "_huzhu");
                return Flow::Done;
            }
            m.card = Some(tao_id);
            if huzhu_allies(ctx.core, ctx.catalog, owner).is_empty() {
                refund_once_used(ctx, owner, "_huzhu");
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_HUZHU_TARGET)
        }
        HostStage::TargetChoice => {
            set_flag(ctx.core, owner, "l19_ask_id", 0);
            let allies = huzhu_allies(ctx.core, ctx.catalog, owner);
            let ally = ctx.answer().iter().find_map(|o| match *o {
                ChoiceOption::Player(p) if allies.contains(&p) => Some(p),
                _ => None,
            });
            let Some(target) = ally else {
                refund_once_used(ctx, owner, "_huzhu");
                return Flow::Done;
            };
            let Some(tao_id) = m.card.filter(|id| ctx.core.player(owner).hand.contains(id)) else {
                refund_once_used(ctx, owner, "_huzhu");
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "_huzhu_used_stamp", huzhu_play_stamp(ctx.core, owner));
            set_flag(ctx.core, owner, "_huzhu_healed", 1);
            set_flag(ctx.core, owner, "l19_huzhu_healed", 1);
            set_flag(ctx.core, target, "_huzhu_recovered", 1);
            // LIFO 解析栈：先压入虚拟【桃】CardUsed（target），再弃手桃，保证弃牌后立刻走真实 CardUsed(Tao) 链路
            let mut virtual_tao =
                crate::machine::card_use_as(0, owner, vec![target], Some(CardKind::Tao));
            virtual_tao.card = None;
            ctx.emit(EventData::CardUsed(virtual_tao));
            discard_cards(ctx, owner, &[tao_id]);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn transform_player_into_zombie(
    ctx: &mut Ctx<'_>,
    victim: PlayerId,
    killer: Option<PlayerId>,
) {
    let mut all_hej = ctx.core.player(victim).hand.clone();
    all_hej.extend(ctx.core.player(victim).equips.iter().copied());
    discard_cards(ctx, victim, &all_hej);
    let judges = ctx.core.player(victim).judge.clone();
    for j in judges {
        if let Some(pos) = ctx.core.player(victim).judge.iter().position(|&x| x == j) {
            ctx.core.player_mut(victim).judge.remove(pos);
            ctx.to_discard(j);
        }
    }

    let killed_by_zombie_or_fan = killer.is_some_and(|k| {
        k != victim
            && (player_role(ctx.core, k) == Some(Role::Rebel)
                || flag(ctx.core, k, "zombie_role") == 1
                || flag(ctx.core, k, "_jiangshi_transformed") > 0
                || flag(ctx.core, k, "_jiangshi2_transformed") > 0)
    });
    let force_fz = flag(ctx.core, victim, "_jiangshi_force_fz") == 1
        || flag(ctx.core, victim, "_jiangshi_storage") == 1;
    let force_nj = flag(ctx.core, victim, "_jiangshi_force_nj") == 1;
    let is_fz = !force_nj && (killed_by_zombie_or_fan || force_fz);

    let new_max = if is_fz { 5u32 } else { 3u32 };
    let z_role = if is_fz { 1u32 } else { 2u32 };
    {
        let p = ctx.core.player_mut(victim);
        p.max_hp = new_max;
        p.hp = new_max as i32;
        p.dying = false;
        p.alive = true;
    }
    if ctx
        .res
        .stack
        .iter()
        .any(|f| matches!(f.data, EventData::Death { player, .. } if player == victim))
    {
        ctx.core.player_mut(victim).death_cancelled = true;
    }
    if is_fz {
        set_player_role(ctx.core, victim, Role::Rebel);
        grant_zombie_skills(ctx, victim, true);
    } else {
        set_player_role(ctx.core, victim, Role::Traitor);
        grant_zombie_skills(ctx, victim, false);
    }
    set_flag(ctx.core, victim, "_jiangshi_infected", 1);
    set_flag(ctx.core, victim, "_jiangshi_transformed", 1);
    set_flag(ctx.core, victim, "zombie_role", z_role);
    ctx.start_draw(victim, 4);
}

pub(super) fn step_jiangshi4(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if ctx.core.player(owner).hp > 0 || flag(ctx.core, owner, "_jiangshi4_transferred") > 0 {
        return Flow::Done;
    }
    let candidate = (0..ctx.core.players.len() as PlayerId)
        .find(|&p| {
            p != owner
                && ctx.core.player(p).alive
                && flag(ctx.core, p, "zombie_role") == 0
                && player_role(ctx.core, p) == Some(Role::Loyalist)
        })
        .or_else(|| {
            (0..ctx.core.players.len() as PlayerId).find(|&p| {
                p != owner
                    && ctx.core.player(p).alive
                    && flag(ctx.core, p, "zombie_role") == 0
                    && !matches!(
                        player_role(ctx.core, p),
                        Some(Role::Rebel | Role::Traitor)
                    )
            })
        });
    let Some(zhong) = candidate else {
        return Flow::Done;
    };
    let old_tz = flag(ctx.core, owner, "_tuizhi").max(flag(ctx.core, owner, "l19_tuizhi_turns"));
    let inherited_tz = old_tz.saturating_sub(1);
    set_player_role(ctx.core, zhong, Role::Lord);
    ctx.core.player_mut(zhong).max_hp += 1;
    ctx.recover(zhong, 1, None);
    set_flag(ctx.core, zhong, "_tuizhi", inherited_tz);
    set_flag(ctx.core, zhong, "l19_tuizhi_turns", inherited_tz);
    set_flag(ctx.core, zhong, "_jiangshi4_new_zhu", 1);
    set_flag(ctx.core, owner, "_tuizhi", 0);
    set_flag(ctx.core, owner, "l19_tuizhi_turns", 0);
    set_flag(ctx.core, owner, "_jiangshi4_transferred", 1);
    set_flag(ctx.core, owner, "_jiangshi_force_fz", 1);
    transform_player_into_zombie(ctx, owner, m.target);
    Flow::Done
}

pub(super) fn step_wujin_re(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(&victim) = m.targets.first() else {
        return Flow::Done;
    };
    match m.stage {
        HostStage::Act => {
            let fid = ctx.current_event_frame_id();
            let guard = format!("_wujin_re_frame:{fid}");
            if flag(ctx.core, 0, &guard) > 0 {
                return Flow::Done;
            }
            set_flag(ctx.core, 0, &guard, 1);
            if victim == 0 {
                set_flag(ctx.core, 0, "_wujinmoshiRE_reset", 1);
                return Flow::Done;
            }
            if !ctx.core.player(0).alive {
                return Flow::Done;
            }
            set_flag(ctx.core, 0, "wujinmoshiESkillE", 0);
            let dxb = flag(ctx.core, 0, "wujinmoshiDXB") + 1;
            let ds = flag(ctx.core, 0, "wujinmoshiDS") + 1;
            set_flag(ctx.core, 0, "wujinmoshiDXB", dxb);
            set_flag(ctx.core, 0, "wujinmoshiDS", ds);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_WUJIN_RE1_CONFIRM)
        }
        HostStage::SelectNature => {
            set_flag(ctx.core, m.owner, "l19_ask_id", 0);
            if ctx.answer().first().copied() != Some(ChoiceOption::Confirm) {
                set_flag(ctx.core, 0, "_wujinmoshiRE1_declined", 1);
                return Flow::Done;
            }
            if collect_wujin_stealable_skills(ctx.core, ctx.catalog, 0, victim).is_empty() {
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_WUJIN_RE2_SKILL)
        }
        HostStage::SelectCard => {
            set_flag(ctx.core, m.owner, "l19_ask_id", 0);
            let Some(ChoiceOption::Tag(idx)) =
                ctx.answer().iter().find(|o| matches!(o, ChoiceOption::Tag(_))).copied()
            else {
                return Flow::Done;
            };
            let pool = collect_wujin_stealable_skills(ctx.core, ctx.catalog, 0, victim);
            if let Some(&chosen) = pool.get(idx as usize) {
                let _ = grant_skill_named(ctx, 0, chosen);
                set_flag(ctx.core, 0, "_wujinmoshiRE_used", 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
