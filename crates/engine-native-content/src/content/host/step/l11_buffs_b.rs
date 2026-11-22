//! L11（太虚幻境）Batch L11-3 Buffs Group B（`L11BuffsB` tag 734）：
//!
//! 承载 `extension_buff.js:231..431` 及 `extension_skill.js:2038..2529` 后组增益（20 组）：
//! - `dunjiazhoufa`、`ziyideyongzhan`、`yanhuozhiren`、`jingleizhiren`、`yingjicuoshijia`/`yi`
//! - `xihuozhikai`、`dingleizhikai`、`yueyingdejiqiao`（`host_judge_settled`）、`taipingzhoufa`、`baijujianchu`
//! - `juejingzhice`、`xumingshenyao`、`wuweizhile`（`host_judge_settled`）、`yuanduandilue`、`jiezizhidao`（`host_judge_settled`）
//! - `huangtiannizhuan`、`dongchazhiyan`、`mingjingzhiyan`、`juxiangzhifeng`

use super::l11_common::*;
use super::l11_common_b2::damage_effective_kind;
use super::l11_common_b3::*;
use super::*;
use crate::catalog::{CardKind, EquipSlot, QueryCtx, SkillLimits, SkillMachine, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow};

pub(in crate::content::host) fn trigger_meta_l11_buffs_b(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, optional: bool, extra_events| {
        TriggerMeta {
            event,
            timing,
            priority,
            order_selectable: false,
            optional,
            extra_events,
        }
    };
    match name {
        "buff_txhj_ziyideyongzhan" | "_buff_txhj_ziyideyongzhan" => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            2,
            false,
            Vec::new(),
        )),
        "buff_txhj_yanhuozhiren"
        | "_buff_txhj_yanhuozhiren"
        | "buff_txhj_jingleizhiren"
        | "_buff_txhj_jingleizhiren" => Some(meta(
            EventKind::DamageBegin1,
            Timing::Normal,
            4,
            false,
            Vec::new(),
        )),
        "buff_txhj_yingjicuoshijia"
        | "_buff_txhj_yingjichuoshijia"
        | "buff_txhj_yingjicuoshiyi"
        | "_buff_txhj_yingjichuoshiyi" => Some(meta(
            EventKind::CardsLost,
            Timing::Normal,
            0,
            true,
            vec![
                EventKind::CardsDiscarded,
                EventKind::CardsMoved,
                EventKind::Equipped,
            ],
        )),
        "buff_txhj_xihuozhikai"
        | "_buff_txhj_xihuozhikai"
        | "buff_txhj_dingleizhikai"
        | "_buff_txhj_dingleizhikai" => Some(meta(
            EventKind::DamageBegin3,
            Timing::Normal,
            0,
            false,
            Vec::new(),
        )),
        "buff_txhj_taipingzhoufa" | "_buff_txhj_taipingzhoufa" => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            3,
            false,
            Vec::new(),
        )),
        "buff_txhj_baijujianchu" | "_buff_txhj_baijujianchu" => Some(meta(
            EventKind::DamageEnd,
            Timing::Normal,
            0,
            false,
            Vec::new(),
        )),
        "buff_txhj_juejingzhice" | "_buff_txhj_juejingzhice" => Some(meta(
            EventKind::CardResponded,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardUsed],
        )),
        "buff_txhj_xumingshenyao" | "_buff_txhj_xumingshenyao" => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            0,
            false,
            Vec::new(),
        )),
        "buff_txhj_yuanduandilue"
        | "_buff_txhj_yuanduandilue"
        | "buff_txhj_huangtiannizhuan"
        | "_buff_txhj_huangtiannizhuan" => Some(meta(
            EventKind::TurnBegin,
            Timing::First,
            10,
            false,
            Vec::new(),
        )),
        "buff_txhj_dongchazhiyan"
        | "_buff_txhj_dongchazhiyan"
        | "buff_txhj_mingjingzhiyan"
        | "_buff_txhj_mingjingzhiyan" => Some(meta(
            EventKind::CardUsed,
            Timing::First,
            10,
            false,
            Vec::new(),
        )),
        "buff_txhj_juxiangzhifeng" | "_buff_txhj_juxiangzhifeng" => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            0,
            false,
            Vec::new(),
        )),
        // `dunjiazhoufa` (d34_flags), `yueyingdejiqiao`/`wuweizhile`/`jiezizhidao` (`host_judge_settled`)
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_buffs_b(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn activate_l11_buffs_b(
    _skill: &HostSkillBehavior,
    _ctx: &QueryCtx<'_>,
    _owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    None
}

pub(in crate::content::host) fn filter_l11_buffs_b_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[crate::event::Frame],
    frame: Option<&crate::event::Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let is_optional = matches!(
        skill.name.as_str(),
        "buff_txhj_yingjicuoshijia"
            | "_buff_txhj_yingjichuoshijia"
            | "buff_txhj_yingjicuoshiyi"
            | "_buff_txhj_yingjichuoshiyi"
    );
    let owner_ok = if is_optional {
        is_unblocked_l11_owner(state, ctx.catalog, owner)
    } else {
        is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
    };
    if !owner_ok || buff_yields_to_companion(state, ctx.catalog, owner, &skill.name) {
        return false;
    }
    match skill.name.as_str() {
        "buff_txhj_ziyideyongzhan" | "_buff_txhj_ziyideyongzhan" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && state.turn.current == owner
                    && state.turn.phase == Phase::Play
                    && u.effective_kind(ctx.catalog).is_sha_like()
                    && has_weapon_slot(state, ctx.catalog, owner)
        ),
        "buff_txhj_yanhuozhiren" | "_buff_txhj_yanhuozhiren" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.nature == DamageNature::Fire
                    && !d.from_chain
                    && has_weapon_slot(state, ctx.catalog, owner)
        ),
        "buff_txhj_jingleizhiren" | "_buff_txhj_jingleizhiren" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.nature == DamageNature::Thunder
                    && !d.from_chain
                    && has_weapon_slot(state, ctx.catalog, owner)
        ),
        "buff_txhj_yingjicuoshijia"
        | "_buff_txhj_yingjichuoshijia"
        | "buff_txhj_yingjicuoshiyi"
        | "_buff_txhj_yingjichuoshiyi" => match event {
            EventData::CardsLost { player, cards, .. } => {
                *player == owner
                    && cards.iter().any(|&c| {
                        ctx.catalog.card_slot_of(state, c) == Some(EquipSlot::Armor)
                    })
            }
            EventData::CardsDiscarded { player, cards } => {
                *player == owner
                    && cards.iter().any(|&c| {
                        flag(state, owner, &format!("l11_hand_dismantle_card:{c}")) > 0
                            && ctx.catalog.card_slot_of(state, c) == Some(EquipSlot::Armor)
                    })
            }
            EventData::CardsMoved { from, cards, .. } => {
                *from == owner
                    && cards.iter().any(|&c| {
                        flag(state, owner, &format!("l11_hand_snatch_card:{c}")) > 0
                            && ctx.catalog.card_slot_of(state, c) == Some(EquipSlot::Armor)
                    })
            }
            EventData::Equipped { player, card, .. } => {
                *player == owner
                    && flag(state, owner, &format!("l11_equip_from_hand:{card}")) > 0
                    && ctx.catalog.card_slot_of(state, *card) == Some(EquipSlot::Armor)
            }
            _ => false,
        },
        "buff_txhj_xihuozhikai" | "_buff_txhj_xihuozhikai" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.nature == DamageNature::Fire
                    && has_armor_slot(state, ctx.catalog, owner)
        ),
        "buff_txhj_dingleizhikai" | "_buff_txhj_dingleizhikai" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.nature == DamageNature::Thunder
                    && has_armor_slot(state, ctx.catalog, owner)
        ),
        "buff_txhj_taipingzhoufa" | "_buff_txhj_taipingzhoufa" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && u.effective_kind(ctx.catalog).is_sha_like()
                    && sha_use_is_elemental(ctx.catalog, u)
        ),
        "buff_txhj_baijujianchu" | "_buff_txhj_baijujianchu" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.nature == DamageNature::Normal
                    && state.player(d.target).alive
                    && damage_effective_kind(state, frames, frame, ctx.catalog, d)
                        .is_some_and(CardKind::is_sha_like)
        ),
        "buff_txhj_juejingzhice" | "_buff_txhj_juejingzhice" => {
            let is_sha_shan = match event {
                EventData::CardResponded { card, player } => {
                    *player == owner
                        && (ctx.catalog.card_kind(*card) == CardKind::Shan
                            || state.declared_kinds.get(card) == Some(&CardKind::Shan))
                        && frames
                            .iter()
                            .rev()
                            .find(|f| matches!(f.data, EventData::CardUsed(_)))
                            .is_some_and(|f| {
                                matches!(
                                    &f.data,
                                    EventData::CardUsed(u) if u.effective_kind(ctx.catalog).is_sha_like()
                                ) && flag(
                                    state,
                                    owner,
                                    &format!("l11_sha_dodge_ok:{}:{card}", f.id),
                                ) > 0
                            })
                }
                EventData::CardUsed(u) => {
                    u.player == owner
                        && !u.cancelled
                        && u.effective_kind(ctx.catalog) == CardKind::Shan
                        && frames
                            .iter()
                            .rev()
                            .find(|f| {
                                matches!(f.data, EventData::CardUsed(_))
                                    && frame.is_none_or(|cf| f.id != cf.id)
                            })
                            .is_some_and(|f| {
                                matches!(
                                    &f.data,
                                    EventData::CardUsed(sha) if sha.effective_kind(ctx.catalog).is_sha_like()
                                )
                            })
                }
                _ => false,
            };
            is_sha_shan && state.player(owner).hand.is_empty()
        }
        "buff_txhj_xumingshenyao" | "_buff_txhj_xumingshenyao" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && u.effective_kind(ctx.catalog) == CardKind::Tao
                    && (u.targets.contains(&owner) || u.targets.is_empty())
                    && state.player(owner).hp <= 0
        ),
        "buff_txhj_yuanduandilue" | "_buff_txhj_yuanduandilue" => {
            matches!(event, EventData::TurnBegin { .. })
                && !has_named_skill(state, ctx.catalog, owner, "txhj_yuanduandilue")
        }
        "buff_txhj_huangtiannizhuan" | "_buff_txhj_huangtiannizhuan" => {
            matches!(event, EventData::TurnBegin { .. })
                && flag(state, owner, "l11_huangtiannizhuan_applied") == 0
        }
        "buff_txhj_dongchazhiyan" | "_buff_txhj_dongchazhiyan" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && state.turn.current == owner
                    && u.effective_kind(ctx.catalog) == CardKind::Snatch
        ),
        "buff_txhj_mingjingzhiyan" | "_buff_txhj_mingjingzhiyan" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && state.turn.current == owner
                    && u.effective_kind(ctx.catalog) == CardKind::Dismantle
        ),
        "buff_txhj_juxiangzhifeng" | "_buff_txhj_juxiangzhifeng" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && u.effective_kind(ctx.catalog) == CardKind::SavageAttack
                    && !u.targets.is_empty()
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_buffs_b(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let (target, card, targets) = match event {
        EventData::Damage(d) => (Some(d.target), None, Vec::new()),
        EventData::CardUsed(u) => (None, u.card, u.targets.clone()),
        _ => (None, None, Vec::new()),
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11BuffsB,
        owner,
        target,
        card,
        cards: Vec::new(),
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l11_buffs_b(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "buff_txhj_ziyideyongzhan" | "_buff_txhj_ziyideyongzhan" => {
            if play_phase_sha_index(ctx, owner) == 2 {
                let cid = m.card.unwrap_or(0);
                let prev = ctx
                    .core
                    .vm_aux
                    .flags
                    .get(&(owner, format!("card_damage_bonus:{cid}")))
                    .copied()
                    .unwrap_or(0);
                ctx.core
                    .vm_aux
                    .flags
                    .insert((owner, format!("card_damage_bonus:{cid}")), prev + 1);
            }
            Flow::Done
        }
        "buff_txhj_yanhuozhiren"
        | "_buff_txhj_yanhuozhiren"
        | "buff_txhj_jingleizhiren"
        | "_buff_txhj_jingleizhiren" => {
            with_current_damage_mut(ctx, |dm| {
                dm.amount += 1;
            });
            Flow::Done
        }
        "buff_txhj_yingjicuoshijia" | "_buff_txhj_yingjichuoshijia" => {
            ctx.recover(owner, 1, None);
            Flow::Done
        }
        "buff_txhj_yingjicuoshiyi" | "_buff_txhj_yingjichuoshiyi" => {
            ctx.start_draw(owner, 2);
            Flow::Done
        }
        "buff_txhj_xihuozhikai"
        | "_buff_txhj_xihuozhikai"
        | "buff_txhj_dingleizhikai"
        | "_buff_txhj_dingleizhikai" => {
            with_current_damage_mut(ctx, |dm| {
                dm.amount = dm.amount.saturating_sub(1);
                if dm.amount == 0 {
                    dm.prevented = true;
                }
            });
            Flow::Done
        }
        "buff_txhj_taipingzhoufa" | "_buff_txhj_taipingzhoufa" => {
            let cid = m.card.unwrap_or(0);
            let prev = ctx
                .core
                .vm_aux
                .flags
                .get(&(owner, format!("card_damage_bonus:{cid}")))
                .copied()
                .unwrap_or(0);
            ctx.core
                .vm_aux
                .flags
                .insert((owner, format!("card_damage_bonus:{cid}")), prev + 1);
            Flow::Done
        }
        "buff_txhj_baijujianchu" | "_buff_txhj_baijujianchu" => {
            if let Some(t) = m.target {
                let _ = random_discard_he(ctx, t);
            }
            Flow::Done
        }
        "buff_txhj_juejingzhice" | "_buff_txhj_juejingzhice" => {
            if ctx.core.player(owner).hand.is_empty() {
                ctx.start_draw(owner, 2);
            }
            Flow::Done
        }
        "buff_txhj_xumingshenyao" | "_buff_txhj_xumingshenyao" => {
            ctx.recover(owner, 1, None);
            Flow::Done
        }
        "buff_txhj_yuanduandilue" | "_buff_txhj_yuanduandilue" => {
            ensure_perm_skill(ctx, owner, "txhj_yuanduandilue");
            Flow::Done
        }
        "buff_txhj_huangtiannizhuan" | "_buff_txhj_huangtiannizhuan" => {
            set_flag(ctx.core, owner, "l11_huangtiannizhuan_applied", 1);
            let order = ctx.core.seat_order_from(owner);
            for t in order {
                if !ctx.core.is_friend_of(ctx.catalog, owner, t) {
                    ensure_perm_skill(ctx, t, "txhj_huangtiannizhuan");
                    let prev = flag(ctx.core, t, "l11_huangtiannizhuan_marks");
                    set_flag(ctx.core, t, "l11_huangtiannizhuan_marks", prev + 1);
                }
            }
            Flow::Done
        }
        "buff_txhj_dongchazhiyan"
        | "_buff_txhj_dongchazhiyan"
        | "buff_txhj_mingjingzhiyan"
        | "_buff_txhj_mingjingzhiyan" => {
            if !has_named_skill(ctx.core, ctx.catalog, owner, "txhj_kapaikeshi") {
                ensure_perm_skill(ctx, owner, "txhj_kapaikeshi");
                set_flag(ctx.core, owner, "l11_temp_kapaikeshi", 1);
            }
            Flow::Done
        }
        "buff_txhj_juxiangzhifeng" | "_buff_txhj_juxiangzhifeng" => handle_juxiangzhifeng(m, ctx),
        _ => Flow::Done,
    }
}

fn handle_juxiangzhifeng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let cid = m.card.unwrap_or(0);
    if m.stage == HostStage::SelectCard {
        let mut yielded_for_discard = false;
        if let Some(t) = m.target {
            let picked = match ctx.answer().first().copied() {
                Some(ChoiceOption::Card(c)) if he_cards(ctx.core, t).contains(&c) => Some(c),
                _ => None,
            };
            if let Some(c) = picked {
                discard_player_cards(ctx, t, &[c]);
                yielded_for_discard = true;
            } else {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((t, format!("aoe_direct_hit:{cid}")), 1);
            }
        }
        if !m.targets.is_empty() {
            m.targets.remove(0);
        }
        m.target = None;
        m.stage = HostStage::Act;
        if yielded_for_discard {
            return Flow::Yield;
        }
    }
    while let Some(&t) = m.targets.first() {
        if !ctx.core.player(t).alive
            || crate::kernel::dushiwang_remen_blocks_core(ctx.core, ctx.catalog, t)
            || crate::kernel::weimu_blocks_aoe_core(ctx.core, ctx.catalog, cid, t)
            || ctx.catalog.aoe_excludes_target(ctx.core, t)
            || crate::rvm_bridge::modifiers::eval_card_target_canceled(
                ctx.core,
                ctx.catalog,
                m.owner,
                t,
                cid,
                CardKind::SavageAttack,
                DamageNature::Normal,
            )
        {
            m.targets.remove(0);
            continue;
        }
        if he_cards(ctx.core, t).is_empty() {
            ctx.core
                .vm_aux
                .flags
                .insert((t, format!("aoe_direct_hit:{cid}")), 1);
            m.targets.remove(0);
            continue;
        }
        m.target = Some(t);
        m.stage = HostStage::SelectCard;
        if let Some((observer, spec)) = super::l11_buffs_b_asks::build_l11_buffs_b_ask(
            m.stage,
            m.owner,
            m.target,
            &m.targets,
            m.card,
            &m.cards,
            ctx.core,
            ctx.catalog,
        ) {
            return Flow::WaitChoice { observer, spec };
        }
        m.target = None;
        m.targets.remove(0);
    }
    Flow::Done
}
