//! L11（太虚幻境）Batch L11-3 Buffs Group A（`L11BuffsA` tag 733）：
//!
//! 承载 `extension_buff.js:2..230` 及 `extension_skill.js:1633..2035` 前组增益（20 组）：
//! - `aozhan`（战役后不回血；战内无修饰）、`yanhuo`、`jinglei`、`xihuo`、`dinglei`
//! - `wuzhongshengsha`、`wuzhongshengshan`、`fengyushengcai`、`jipodizhen`、`wenhezhenlue`
//! - `wenhejibei`、`yueyingdezhihui`、`changbingguangren`、`fuzu`、`xiaojideyongwu`
//! - `xiongziyingfa`（由 `l11_common_b3::on_draw_begin_l11_b3` 驱动）、`wuxiangdexinxue`、`juguzhidao`（商店仅）、`pozhenzhifeng`、`yanrenzhinu`

use super::l11_common::*;
use super::l11_common_b2::is_ordinary_trick_kind;
use super::l11_common_b3::*;
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, SkillMachine, TriggerMeta};
use crate::event::{DamageNature, EventData, EventKind, LostOrigin, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow};

pub(in crate::content::host) fn trigger_meta_l11_buffs_a(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "buff_txhj_yanhuo" | "_buff_txhj_yanhuo" => {
            Some(meta(EventKind::DamageBegin1, Timing::Normal, 4, Vec::new()))
        }
        "buff_txhj_jinglei" | "_buff_txhj_jinglei" => {
            Some(meta(EventKind::DamageBegin1, Timing::Normal, 4, Vec::new()))
        }
        "buff_txhj_xihuo" | "_buff_txhj_xihuo" => {
            Some(meta(EventKind::DamageBegin3, Timing::Normal, 0, Vec::new()))
        }
        "buff_txhj_dinglei" | "_buff_txhj_dinglei" => {
            Some(meta(EventKind::DamageBegin3, Timing::Normal, 0, Vec::new()))
        }
        "buff_txhj_wuzhongshengsha" | "_buff_txhj_wuzhongshengsha" => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, Vec::new()))
        }
        "buff_txhj_wuzhongshengshan" | "_buff_txhj_wuzhongshengshan" => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, Vec::new()))
        }
        "buff_txhj_fengyushengcai" | "_buff_txhj_fengyushengcai" => {
            Some(meta(EventKind::TurnBegin, Timing::Normal, 0, Vec::new()))
        }
        "buff_txhj_jipodizhen" | "_buff_txhj_jipodizhen" => {
            Some(meta(EventKind::DamageEnd, Timing::Normal, 0, Vec::new()))
        }
        "buff_txhj_wenhezhenlue" | "_buff_txhj_wenhezhenlue" => {
            Some(meta(EventKind::CardUsed, Timing::First, 6, Vec::new()))
        }
        "buff_txhj_wenhejibei"
        | "_buff_txhj_wenhejibei"
        | "buff_txhj_yueyingdezhihui"
        | "_buff_txhj_yueyingdezhihui"
        | "buff_txhj_changbingguangren"
        | "_buff_txhj_changbingguangren"
        | "buff_txhj_wuxiangdexinxue"
        | "_buff_txhj_wuxiangdexinxue" => {
            Some(meta(EventKind::TurnBegin, Timing::First, 10, Vec::new()))
        }
        "buff_txhj_fuzu" | "_buff_txhj_fuzu" => {
            Some(meta(EventKind::TurnBegin, Timing::First, 2, Vec::new()))
        }
        "buff_txhj_xiaojideyongwu" | "_buff_txhj_xiaojideyongwu" => {
            Some(meta(EventKind::CardsLost, Timing::Normal, 0, Vec::new()))
        }
        "buff_txhj_pozhenzhifeng" | "_buff_txhj_pozhenzhifeng" => Some(meta(
            EventKind::PhaseBegin,
            Timing::First,
            10,
            vec![EventKind::Equipped, EventKind::CardsLost, EventKind::PhaseEnd],
        )),
        "buff_txhj_yanrenzhinu" | "_buff_txhj_yanrenzhinu" => {
            Some(meta(EventKind::CardUsed, Timing::Normal, 2, Vec::new()))
        }
        // `buff_txhj_aozhan`, `buff_txhj_juguzhidao`, `buff_txhj_xiongziyingfa`, `_buff_txhj_xiongziyingfa`
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_buffs_a(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn activate_l11_buffs_a(
    _skill: &HostSkillBehavior,
    _ctx: &QueryCtx<'_>,
    _owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    None
}

pub(in crate::content::host) fn filter_l11_buffs_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    if !is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
        || buff_yields_to_companion(state, ctx.catalog, owner, &skill.name)
    {
        return false;
    }
    match skill.name.as_str() {
        "buff_txhj_yanhuo" | "_buff_txhj_yanhuo" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.nature == DamageNature::Fire
                    && !d.from_chain
        ),
        "buff_txhj_jinglei" | "_buff_txhj_jinglei" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.nature == DamageNature::Thunder
                    && !d.from_chain
        ),
        "buff_txhj_xihuo" | "_buff_txhj_xihuo" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.nature == DamageNature::Fire
        ),
        "buff_txhj_dinglei" | "_buff_txhj_dinglei" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.nature == DamageNature::Thunder
        ),
        "buff_txhj_wuzhongshengsha" | "_buff_txhj_wuzhongshengsha" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if *player == owner
        ),
        "buff_txhj_wuzhongshengshan" | "_buff_txhj_wuzhongshengshan" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner
        ),
        "buff_txhj_fengyushengcai" | "_buff_txhj_fengyushengcai" => matches!(
            event,
            EventData::TurnBegin { .. }
                if flag(state, owner, "l11_fengyushengcai_round") != state.turn.round.max(1)
                    && state.player(owner).hp >= state.player(owner).max_hp as i32
        ),
        "buff_txhj_jipodizhen" | "_buff_txhj_jipodizhen" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && state.player(d.target).alive
        ),
        "buff_txhj_wenhezhenlue" | "_buff_txhj_wenhezhenlue" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && is_ordinary_trick_kind(u.effective_kind(ctx.catalog))
        ),
        "buff_txhj_wenhejibei" | "_buff_txhj_wenhejibei" => {
            matches!(event, EventData::TurnBegin { .. })
                && !has_named_skill(state, ctx.catalog, owner, "txhj_wenhejibei")
        }
        "buff_txhj_yueyingdezhihui" | "_buff_txhj_yueyingdezhihui" => {
            matches!(event, EventData::TurnBegin { .. })
                && !has_named_skill(state, ctx.catalog, owner, "txhj_yueyingdezhihui")
        }
        "buff_txhj_changbingguangren" | "_buff_txhj_changbingguangren" => {
            matches!(event, EventData::TurnBegin { .. })
                && !has_named_skill(state, ctx.catalog, owner, "txhj_changbingguangren")
        }
        "buff_txhj_wuxiangdexinxue" | "_buff_txhj_wuxiangdexinxue" => {
            matches!(event, EventData::TurnBegin { .. })
                && !has_named_skill(state, ctx.catalog, owner, "txhj_wuxiangdexinxue")
        }
        "buff_txhj_fuzu" | "_buff_txhj_fuzu" => {
            matches!(event, EventData::TurnBegin { .. })
                && flag(state, owner, "l11_fuzu_done") == 0
                && state.player(owner).hand.len() != state.player(owner).max_hp as usize
        }
        "buff_txhj_xiaojideyongwu" | "_buff_txhj_xiaojideyongwu" => matches!(
            event,
            EventData::CardsLost { player, origins, .. }
                if *player == owner
                    && !has_named_skill(state, ctx.catalog, owner, "xiaoji")
                    && !has_named_skill(
                        state,
                        ctx.catalog,
                        owner,
                        crate::vocabulary::names::XIAO_JI
                    )
                    && origins.contains(&LostOrigin::Equip)
        ),
        "buff_txhj_pozhenzhifeng" | "_buff_txhj_pozhenzhifeng" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => {
                *player == owner
                    && (has_weapon_slot(state, ctx.catalog, owner)
                        || has_equip_named(state, ctx.catalog, owner, "txhj_feilong"))
                    && !has_named_skill(state, ctx.catalog, owner, "txhj_pozhenzhifeng")
            }
            EventData::Equipped { player, .. } => {
                *player == owner
                    && state.turn.current == owner
                    && state.turn.phase == Phase::Play
                    && (has_weapon_slot(state, ctx.catalog, owner)
                        || has_equip_named(state, ctx.catalog, owner, "txhj_feilong"))
                    && !has_named_skill(state, ctx.catalog, owner, "txhj_pozhenzhifeng")
            }
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: false,
            } => {
                *player == owner && flag(state, owner, "l11_temp_pozhenzhifeng") > 0
            }
            EventData::CardsLost {
                player, origins, ..
            } => {
                *player == owner
                    && origins.contains(&LostOrigin::Equip)
                    && flag(state, owner, "l11_temp_pozhenzhifeng") > 0
                    && !has_weapon_slot(state, ctx.catalog, owner)
                    && !has_equip_named(state, ctx.catalog, owner, "txhj_feilong")
            }
            _ => false,
        },
        "buff_txhj_yanrenzhinu" | "_buff_txhj_yanrenzhinu" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && state.turn.current == owner
                    && state.turn.phase == Phase::Play
                    && u.effective_kind(ctx.catalog).is_sha_like()
                    && has_weapon_slot(state, ctx.catalog, owner)
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_buffs_a(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let (target, card, cards, targets) = match event {
        EventData::Damage(d) => (Some(d.target), None, Vec::new(), Vec::new()),
        EventData::CardUsed(u) => (None, u.card, Vec::new(), u.targets.clone()),
        EventData::CardsLost { cards, .. } => (None, None, cards.clone(), Vec::new()),
        EventData::Phase { begin, .. } => {
            (None, Some(u32::from(*begin)), Vec::new(), Vec::new())
        }
        _ => (None, None, Vec::new(), Vec::new()),
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11BuffsA,
        owner,
        target,
        card,
        cards,
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l11_buffs_a(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "buff_txhj_yanhuo"
        | "_buff_txhj_yanhuo"
        | "buff_txhj_jinglei"
        | "_buff_txhj_jinglei" => {
            with_current_damage_mut(ctx, |dm| {
                dm.amount += 1;
            });
            Flow::Done
        }
        "buff_txhj_xihuo"
        | "_buff_txhj_xihuo"
        | "buff_txhj_dinglei"
        | "_buff_txhj_dinglei" => {
            with_current_damage_mut(ctx, |dm| {
                dm.amount = dm.amount.saturating_sub(1);
                if dm.amount == 0 {
                    dm.prevented = true;
                }
            });
            Flow::Done
        }
        "buff_txhj_wuzhongshengsha" | "_buff_txhj_wuzhongshengsha" => {
            let _ = gain_first_from_deck_or_discard(ctx, owner, CardKind::is_sha_like);
            Flow::Done
        }
        "buff_txhj_wuzhongshengshan" | "_buff_txhj_wuzhongshengshan" => {
            let _ = gain_first_from_deck_or_discard(ctx, owner, |k| k == CardKind::Shan);
            Flow::Done
        }
        "buff_txhj_fengyushengcai" | "_buff_txhj_fengyushengcai" => {
            let round = ctx.core.turn.round.max(1);
            set_flag(ctx.core, owner, "l11_fengyushengcai_round", round);
            if ctx.core.player(owner).hp >= ctx.core.player(owner).max_hp as i32 {
                ctx.start_draw(owner, 1);
            }
            Flow::Done
        }
        "buff_txhj_jipodizhen" | "_buff_txhj_jipodizhen" => {
            if let Some(t) = m.target {
                let _ = random_discard_he(ctx, t);
            }
            Flow::Done
        }
        "buff_txhj_wenhezhenlue" | "_buff_txhj_wenhezhenlue" => {
            let cid = m.card.unwrap_or(0);
            ctx.core
                .vm_aux
                .flags
                .insert((owner, format!("card_nowuxie:{cid}")), 1);
            Flow::Done
        }
        "buff_txhj_wenhejibei" | "_buff_txhj_wenhejibei" => {
            ensure_perm_skill(ctx, owner, "txhj_wenhejibei");
            Flow::Done
        }
        "buff_txhj_yueyingdezhihui" | "_buff_txhj_yueyingdezhihui" => {
            ensure_perm_skill(ctx, owner, "txhj_yueyingdezhihui");
            Flow::Done
        }
        "buff_txhj_changbingguangren" | "_buff_txhj_changbingguangren" => {
            ensure_perm_skill(ctx, owner, "txhj_changbingguangren");
            Flow::Done
        }
        "buff_txhj_wuxiangdexinxue" | "_buff_txhj_wuxiangdexinxue" => {
            ensure_perm_skill(ctx, owner, "txhj_wuxiangdexinxue");
            Flow::Done
        }
        "buff_txhj_fuzu" | "_buff_txhj_fuzu" => {
            set_flag(ctx.core, owner, "l11_fuzu_done", 1);
            let max_hp = ctx.core.player(owner).max_hp as usize;
            let hand_len = ctx.core.player(owner).hand.len();
            if hand_len < max_hp {
                ctx.start_draw(owner, (max_hp - hand_len) as u32);
            } else if hand_len > max_hp {
                let excess = hand_len - max_hp;
                for _ in 0..excess {
                    if let Some(c) = ctx.core.player_mut(owner).hand.pop() {
                        ctx.core.deck.push(c);
                    }
                }
            }
            Flow::Done
        }
        "buff_txhj_xiaojideyongwu" | "_buff_txhj_xiaojideyongwu" => {
            let eq_cnt = m
                .cards
                .iter()
                .filter(|&&c| ctx.catalog.card_kind(c).equip_slot().is_some())
                .count()
                .max(1) as u32;
            ctx.start_draw(owner, eq_cnt * 2);
            Flow::Done
        }
        "buff_txhj_pozhenzhifeng" | "_buff_txhj_pozhenzhifeng" => {
            let is_equip_or_begin = m.card == Some(1)
                || (m.cards.is_empty()
                    && m.card.is_none()
                    && (has_weapon_slot(ctx.core, ctx.catalog, owner)
                        || has_equip_named(ctx.core, ctx.catalog, owner, "txhj_feilong")));
            if is_equip_or_begin {
                if !has_named_skill(ctx.core, ctx.catalog, owner, "txhj_pozhenzhifeng") {
                    ensure_perm_skill(ctx, owner, "txhj_pozhenzhifeng");
                    set_flag(ctx.core, owner, "l11_temp_pozhenzhifeng", 1);
                }
            } else if flag(ctx.core, owner, "l11_temp_pozhenzhifeng") > 0 {
                set_flag(ctx.core, owner, "l11_temp_pozhenzhifeng", 0);
                remove_named_skill(ctx, owner, "txhj_pozhenzhifeng");
            }
            Flow::Done
        }
        "buff_txhj_yanrenzhinu" | "_buff_txhj_yanrenzhinu" => {
            if play_phase_sha_index(ctx, owner) == 2 {
                let cid = m.card.unwrap_or(0);
                for &t in &m.targets {
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((owner, format!("sha_unrespondable:{cid}:{t}")), 1);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
