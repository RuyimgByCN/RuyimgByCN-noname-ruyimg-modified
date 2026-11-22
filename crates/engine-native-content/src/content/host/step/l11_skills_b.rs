//! L11（太虚幻境）武将技能 Batch B（S44–S57：`l11_skills_b.rs`）：
//!
//! - `diyzhuiling`、`diyfeihua`、`txclanxieshu`、`txshixin`、`txshixin_red`
//! - `txmouduan`、`txkaikang`、`txhuangfu`、`txleizhen`、`txxiebi`
//! - `txlianbi`、`txfeiyan`、`txleili`、`cxyMoJun`、`cxyKuangXi`、`txbaobian`

use super::l11_common::*;
use super::l11_skills_b_asks::*;
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, SkillMachine, TriggerMeta};
use crate::event::{CardUseData, DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow};
use crate::rvm_bridge::modifiers::{d34_l28_mods, is_target_unequip2};

fn wait_b_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_l11_skills_b_ask(
        ask_id,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) {
        m.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        m.card = None;
        Flow::Done
    }
}

pub(in crate::content::host) fn trigger_meta_l11_skills_b(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "diyzhuiling" => Some(meta(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "diyfeihua" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new())),
        "txclanxieshu" => Some(meta(
            EventKind::DamageEnd,
            Timing::Normal,
            1,
            vec![EventKind::DamageSource],
        )),
        "txshixin" => Some(meta(EventKind::CardUsed, Timing::First, 9, Vec::new())),
        "txmouduan" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new())),
        "txkaikang" => Some(meta(EventKind::CardUsed, Timing::Normal, 3, Vec::new())),
        "txhuangfu" | "txxiebi" => {
            Some(meta(EventKind::DamageBegin4, Timing::Normal, 1, Vec::new()))
        }
        "txleizhen" => Some(meta(EventKind::Death, Timing::Normal, 1, Vec::new())),
        "txlianbi" => Some(meta(
            EventKind::TurnBegin,
            Timing::First,
            5,
            vec![EventKind::DamageAfter, EventKind::PhaseBegin],
        )),
        "txfeiyan" => Some(meta(EventKind::CardUsed, Timing::Normal, 2, Vec::new())),
        "txleili" => Some(meta(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "cxyMoJun" => Some(meta(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "cxyKuangXi" => Some(meta(EventKind::Dying, Timing::First, 15, Vec::new())),
        "txbaobian" => Some(meta(
            EventKind::TurnBegin,
            Timing::Normal,
            10,
            vec![
                EventKind::PhaseBegin,
                EventKind::DamageEnd,
                EventKind::LoseHp,
                EventKind::Recover,
            ],
        )),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_skills_b(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn activate_l11_skills_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    if skill.name != "cxyKuangXi" {
        return None;
    }
    let state = ctx.state;
    if state.turn.phase != Phase::Play
        || !is_unblocked_l11_owner(state, ctx.catalog, owner)
        || flag(state, owner, "l11_kuangxi_sealed_ts") == turn_stamp(state)
        || alive_others(state, owner).is_empty()
    {
        return None;
    }
    active_skill_machine("cxyKuangXi", HostBehavior::L11GeneralSkillsB, owner)
}

fn is_trick_or_delay_use(catalog: &crate::catalog::Catalog, u: &CardUseData) -> bool {
    effective_kind_of_use(catalog, u).is_some_and(|k| !k.is_basic_kind() && k.equip_slot().is_none())
}

pub(in crate::content::host) fn filter_l11_skills_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let catalog = ctx.catalog;
    if skill.name != "txleizhen" && !state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "diyzhuiling" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.target == owner
                && !d.prevented
                && d.amount > 0
                && d.source.is_some_and(|s| state.player(s).alive)
        }
        "diyfeihua" => {
            let EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } = event
            else {
                return false;
            };
            *player == owner && !alive_others(state, owner).is_empty()
        }
        "txclanxieshu" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            if d.prevented
                || d.amount <= 0
                || !(d.target == owner || d.source == Some(owner))
            {
                return false;
            }
            let Some(cid) = d.card else {
                return false;
            };
            he_cards(state, owner).len() >= card_name_len(catalog, cid)
        }
        "txshixin" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            u.player != owner
                && !u.cancelled
                && u.targets.contains(&owner)
                && is_trick_or_delay_use(catalog, u)
        }
        "txmouduan" => {
            let EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } = event
            else {
                return false;
            };
            *player == owner
                && (flag(state, owner, "l11_turn_used_suits_mask").count_ones() >= 4
                    || flag(state, owner, "l11_turn_used_types_mask").count_ones() >= 3)
                && !movable_field_sources(state, catalog, owner).is_empty()
        }
        "txkaikang" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            !u.cancelled
                && is_sha_use(catalog, u)
                && u.targets.iter().any(|&t| state.player(t).alive)
        }
        "txhuangfu" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.target == owner
                && !d.prevented
                && d.amount > 0
                && d.nature == DamageNature::Thunder
        }
        "txleizhen" => {
            let EventData::Death { player, killer } = event else {
                return false;
            };
            *player == owner && killer.is_some_and(|k| k != owner && state.player(k).alive)
        }
        "txxiebi" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.target == owner
                && !d.prevented
                && d.amount > 1
                && !is_target_unequip2(state, owner)
                && !d.source.is_some_and(|s| {
                    d34_l28_mods::source_ignores_armor(
                        state, catalog, s, owner, d.card, None, d.nature,
                    )
                })
        }
        "txlianbi" => match event {
            EventData::TurnBegin { .. } | EventData::Phase { .. } => {
                !state.player(owner).linked && flag(state, owner, "l11_lianbi_init") == 0
            }
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.nature != DamageNature::Normal
                    && !state.player(owner).linked
            }
            _ => false,
        },
        "txfeiyan" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            u.player != owner
                && !u.cancelled
                && is_sha_use(catalog, u)
                && u.targets.len() == 1
                && state.player(u.player).alive
                && state.computed_distance(catalog, owner, u.player)
                    <= state.attack_range(catalog, owner)
                && state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&c| effective_card_kind(state, catalog, c).is_sha_like())
        }
        "txleili" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.source == Some(owner)
                && !d.prevented
                && d.amount > 0
                && d.card.is_some_and(|c| catalog.card_kind(c).is_sha_like())
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| p != d.target && !state.is_friend_of(catalog, owner, p))
        }
        "cxyMoJun" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            !d.prevented
                && d.amount > 0
                && !d.from_chain
                && d.card.is_some_and(|c| catalog.card_kind(c).is_sha_like())
                && d.source.is_some_and(|s| {
                    state.player(s).alive && state.is_friend_of(catalog, owner, s)
                })
        }
        "cxyKuangXi" => {
            let EventData::Dying { .. } = event else {
                return false;
            };
            flag(state, owner, "l11_kuangxi_target") > 0
        }
        "txbaobian" => match event {
            EventData::TurnBegin { player }
            | EventData::Phase { player, .. }
            | EventData::LoseHp { player, .. }
            | EventData::Recover { player, .. } => *player == owner,
            EventData::Damage(d) => d.target == owner,
            _ => false,
        },
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_skills_b(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut targets = Vec::new();
    let mut cards = Vec::new();
    match event {
        EventData::Damage(d) => {
            target = if skill.name == "diyzhuiling" || skill.name == "cxyMoJun" {
                d.source
            } else {
                Some(d.target)
            };
            if let Some(c) = d.card {
                cards.push(c);
            }
        }
        EventData::CardUsed(u) => {
            target = Some(u.player);
            targets = u.targets.clone();
            if let Some(c) = u.card {
                cards.push(c);
            }
        }
        EventData::Death { killer, .. } => {
            target = *killer;
        }
        _ => {}
    }
    let stage = if skill.name == "cxyKuangXi" && matches!(event, EventData::Dying { .. }) {
        HostStage::CompareTargetPick
    } else {
        HostStage::Act
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11GeneralSkillsB,
        owner,
        target,
        card: None,
        cards,
        targets,
        stage,
    }
}

fn sync_txbaobian(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let hp = ctx.core.player(owner).hp;
    let paoxiao_key = if ctx.catalog.skill_by_name("paoxiao").is_some() {
        "paoxiao"
    } else {
        "咆哮"
    };
    let want = [
        ("tiaoxin", "l11_baobian_added:tiaoxin", hp <= 3),
        (paoxiao_key, "l11_baobian_added:paoxiao", hp <= 2),
        ("shensu", "l11_baobian_added:shensu", hp <= 1),
    ];
    for (skey, fkey, active) in want {
        if active {
            if !has_named_skill(ctx.core, ctx.catalog, owner, skey) {
                ensure_perm_skill(ctx, owner, skey);
                set_flag(ctx.core, owner, fkey, 1);
            }
        } else if flag(ctx.core, owner, fkey) > 0 {
            set_flag(ctx.core, owner, fkey, 0);
            remove_named_skill(ctx, owner, skey);
        }
    }
}

pub(in crate::content::host) fn handle_l11_skills_b(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "diyzhuiling" => match m.stage {
            HostStage::Act => wait_b_ask(m, ctx, HostStage::TargetChoice, ASK_ZHUILING_CONFIRM),
            HostStage::TargetChoice => {
                if is_confirmed(ctx) {
                    if let Some(s) = m.target.filter(|&p| ctx.core.player(p).alive) {
                        let nat = if ctx.core.rng.gen_range(2) == 0 {
                            DamageNature::Fire
                        } else {
                            DamageNature::Thunder
                        };
                        m.stage = HostStage::DrawSettle;
                        emit_damage(ctx, Some(owner), s, 1, nat, None);
                        return Flow::Yield;
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "diyfeihua" => {
            let pick = ctx.core.rng.gen_range(2);
            if pick == 0 {
                let others = alive_others(ctx.core, owner);
                if !others.is_empty() {
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: owner,
                        targets: others,
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::ArcheryAttack),
                        cancelled: false,
                    }));
                }
            } else if let Some(zb_cid) = ctx.catalog.card_by_name("txhj_zhijizhibi") {
                if let Some(t) = alive_others(ctx.core, owner)
                    .into_iter()
                    .find(|&p| !ctx.core.player(p).hand.is_empty())
                    .or_else(|| alive_others(ctx.core, owner).into_iter().next())
                {
                    set_flag(ctx.core, owner, VIRTUAL_TRICK_CID_FLAG, zb_cid + 1);
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: owner,
                        targets: vec![t],
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Wuzhong),
                        cancelled: false,
                    }));
                }
            }
            Flow::Done
        }
        "txclanxieshu" => match m.stage {
            HostStage::Act => {
                let Some(cid) = m.cards.first().copied() else {
                    return Flow::Done;
                };
                let need = card_name_len(ctx.catalog, cid) as u32;
                if he_cards(ctx.core, owner).len() < need as usize {
                    return Flow::Done;
                }
                m.cards = vec![need];
                wait_b_ask(m, ctx, HostStage::SelectCard, ASK_XIESHU_DISCARD)
            }
            HostStage::SelectCard => {
                let chosen = answered_cards(ctx);
                let need = m.cards.first().copied().unwrap_or(1) as usize;
                if chosen.len() == need {
                    discard_player_cards(ctx, owner, &chosen);
                    let pl = ctx.core.player(owner);
                    if pl.hp < pl.max_hp as i32 {
                        let lost = (pl.max_hp as i32 - pl.hp) as u32;
                        ctx.start_draw(owner, lost);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txshixin" => match m.stage {
            HostStage::Act => wait_b_ask(m, ctx, HostStage::TargetChoice, ASK_SHIXIN_CONFIRM),
            HostStage::TargetChoice => {
                if !is_confirmed(ctx) {
                    return Flow::Done;
                }
                m.stage = HostStage::CompareTargetPick;
                ctx.emit(EventData::OrdinaryJudge {
                    owner,
                    invocation: u32::MAX,
                });
                Flow::Yield
            }
            HostStage::CompareTargetPick => {
                let suit = flag(ctx.core, owner, "last_judge_suit");
                set_flag(ctx.core, owner, "last_judge_suit", 0);
                set_flag(ctx.core, owner, "last_judge_rank", 0);
                if suit == 1 || suit == 3 {
                    if let Some(u) = current_use_card_mut(ctx) {
                        u.targets.retain(|&p| p != owner);
                        if u.targets.is_empty() {
                            u.cancelled = true;
                        }
                    }
                } else if suit == 2 || suit == 4 {
                    if let Some(cid) = m.cards.first().copied() {
                        set_flag(ctx.core, owner, "l11_shixin_steal_cid", cid + 1);
                        ensure_perm_skill(ctx, owner, "txshixin_red");
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txmouduan" => match m.stage {
            HostStage::Act => wait_b_ask(m, ctx, HostStage::TargetChoice, ASK_MOUDUAN_CONFIRM),
            HostStage::TargetChoice => {
                if !is_confirmed(ctx) {
                    return Flow::Done;
                }
                wait_b_ask(m, ctx, HostStage::JiefenTarget, ASK_MOUDUAN_FROM)
            }
            HostStage::JiefenTarget => {
                let Some(from) = answered_players(ctx).into_iter().next() else {
                    return Flow::Done;
                };
                m.target = Some(from);
                wait_b_ask(m, ctx, HostStage::SelectCard, ASK_MOUDUAN_CARD)
            }
            HostStage::SelectCard => {
                let Some(card) = answered_cards(ctx).into_iter().next() else {
                    return Flow::Done;
                };
                m.cards = vec![card];
                wait_b_ask(m, ctx, HostStage::SelectNature, ASK_MOUDUAN_TO)
            }
            HostStage::SelectNature => {
                let Some(from) = m.target else {
                    return Flow::Done;
                };
                let Some(card) = m.cards.first().copied() else {
                    return Flow::Done;
                };
                let Some(to) = answered_players(ctx).into_iter().next() else {
                    return Flow::Done;
                };
                if can_receive_moved_card(ctx.core, ctx.catalog, from, to, card) {
                    let was_equip = ctx.core.player(from).equips.contains(&card);
                    if was_equip {
                        ctx.core.player_mut(from).equips.retain(|&c| c != card);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, from, card);
                        ctx.core.player_mut(to).equips.push(card);
                        crate::kernel::grant_equip_skills(ctx, to, card);
                    } else {
                        ctx.core.player_mut(from).judge.retain(|&c| c != card);
                        ctx.note_cards_lost(from, &[card]);
                        ctx.core.player_mut(to).judge.push(card);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txkaikang" => match m.stage {
            HostStage::Act => {
                if let Some(next_t) = m.targets.first().copied() {
                    m.targets.remove(0);
                    if !ctx.core.player(next_t).alive {
                        return handle_l11_skills_b(m, ctx);
                    }
                    m.target = Some(next_t);
                    return wait_b_ask(m, ctx, HostStage::TargetChoice, ASK_KAIKANG_CONFIRM);
                }
                Flow::Done
            }
            HostStage::TargetChoice => {
                if is_confirmed(ctx) {
                    m.stage = HostStage::DrawSettle;
                    ctx.start_draw(owner, 1);
                    return Flow::Yield;
                }
                m.stage = HostStage::Act;
                handle_l11_skills_b(m, ctx)
            }
            HostStage::DrawSettle => {
                let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                    m.stage = HostStage::Act;
                    return handle_l11_skills_b(m, ctx);
                };
                if t == owner || he_cards(ctx.core, owner).is_empty() {
                    m.stage = HostStage::Act;
                    return handle_l11_skills_b(m, ctx);
                }
                wait_b_ask(m, ctx, HostStage::SelectCard, ASK_KAIKANG_GIVE)
            }
            HostStage::SelectCard => {
                let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                    m.stage = HostStage::Act;
                    return handle_l11_skills_b(m, ctx);
                };
                if let Some(cid) = answered_cards(ctx).into_iter().next() {
                    move_cards_to_hand(ctx, owner, t, &[cid]);
                    if ctx.catalog.card_kind(cid).equip_slot().is_some()
                        && ctx.core.player(t).hand.contains(&cid)
                    {
                        m.cards = vec![cid];
                        return wait_b_ask(m, ctx, HostStage::SelectNature, ASK_KAIKANG_EQUIP);
                    }
                }
                m.stage = HostStage::Act;
                handle_l11_skills_b(m, ctx)
            }
            HostStage::SelectNature => {
                if is_confirmed(ctx) {
                    if let (Some(t), Some(cid)) = (
                        m.target.filter(|&p| ctx.core.player(p).alive),
                        m.cards.first().copied(),
                    ) {
                        if ctx.core.player(t).hand.contains(&cid) {
                            ctx.emit(EventData::Equipped {
                                player: t,
                                card: cid,
                                replaced: None,
                            });
                            m.stage = HostStage::Act;
                            return Flow::Yield;
                        }
                    }
                }
                m.stage = HostStage::Act;
                handle_l11_skills_b(m, ctx)
            }
            _ => Flow::Done,
        },
        "txhuangfu" => {
            with_current_damage_mut(ctx, |dm| {
                dm.prevented = true;
                dm.amount = 0;
            });
            Flow::Done
        }
        "txleizhen" => {
            if let Some(k) = m.target.filter(|&p| ctx.core.player(p).alive) {
                emit_damage(ctx, None, k, 1, DamageNature::Thunder, None);
            }
            Flow::Done
        }
        "txxiebi" => {
            with_current_damage_mut(ctx, |dm| {
                if dm.amount > 1 {
                    dm.amount = 1;
                }
            });
            Flow::Done
        }
        "txlianbi" => {
            set_flag(ctx.core, owner, "l11_lianbi_init", 1);
            if !ctx.core.player(owner).linked {
                ctx.emit(EventData::Linked {
                    player: owner,
                    linked: true,
                });
            }
            Flow::Done
        }
        "txfeiyan" => match m.stage {
            HostStage::Act => wait_b_ask(m, ctx, HostStage::TargetChoice, ASK_FEIYAN_CONFIRM),
            HostStage::TargetChoice => {
                if !is_confirmed(ctx) {
                    return Flow::Done;
                }
                wait_b_ask(m, ctx, HostStage::SelectCard, ASK_FEIYAN_SHA)
            }
            HostStage::SelectCard => {
                let Some(sha_c) = answered_cards(ctx).into_iter().next() else {
                    return Flow::Done;
                };
                let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                    return Flow::Done;
                };
                if ctx.core.player(owner).hand.contains(&sha_c) {
                    ctx.core.player_mut(owner).hand.retain(|&c| c != sha_c);
                    ctx.core.table.push(sha_c);
                    ctx.note_cards_lost(owner, &[sha_c]);
                    let eff = effective_card_kind(ctx.core, ctx.catalog, sha_c);
                    m.stage = HostStage::DrawSettle;
                    ctx.emit(EventData::CardUsed(super::sha_card_use_data(
                        ctx,
                        owner,
                        sha_c,
                        vec![t],
                        Some(eff),
                    )));
                    return Flow::Yield;
                }
                Flow::Done
            }
            HostStage::DrawSettle => {
                ctx.start_draw(owner, 2);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txleili" => match m.stage {
            HostStage::Act => wait_b_ask(m, ctx, HostStage::TargetChoice, ASK_LEILI_TARGET),
            HostStage::TargetChoice => {
                if let Some(e) = answered_players(ctx)
                    .into_iter()
                    .next()
                    .filter(|&p| ctx.core.player(p).alive)
                {
                    m.stage = HostStage::DrawSettle;
                    emit_damage(ctx, Some(owner), e, 1, DamageNature::Thunder, None);
                    return Flow::Yield;
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "cxyMoJun" => match m.stage {
            HostStage::Act => {
                let Some(src) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                    return Flow::Done;
                };
                m.stage = HostStage::CompareTargetPick;
                ctx.emit(EventData::OrdinaryJudge {
                    owner: src,
                    invocation: u32::MAX,
                });
                Flow::Yield
            }
            HostStage::CompareTargetPick => {
                let Some(src) = m.target else {
                    return Flow::Done;
                };
                let suit = flag(ctx.core, src, "last_judge_suit");
                set_flag(ctx.core, src, "last_judge_suit", 0);
                set_flag(ctx.core, src, "last_judge_rank", 0);
                if suit == 1 || suit == 3 {
                    let friends: Vec<PlayerId> = alive_all(ctx.core, owner)
                        .into_iter()
                        .filter(|&p| ctx.core.is_friend_of(ctx.catalog, owner, p))
                        .collect();
                    for f in friends {
                        ctx.start_draw(f, 1);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "cxyKuangXi" => match m.stage {
            HostStage::CompareTargetPick => {
                // Any Dying raised during cxyKuangXi (including owner self-dying on step-0 loseHp) seals it for this turn.
                set_flag(ctx.core, owner, "l11_kuangxi_sealed_ts", turn_stamp(ctx.core));
                Flow::Done
            }
            HostStage::Act => wait_b_ask(m, ctx, HostStage::TargetChoice, ASK_KUANGXI_TARGET),
            HostStage::TargetChoice => {
                let Some(t) = answered_players(ctx)
                    .into_iter()
                    .next()
                    .filter(|&p| ctx.core.player(p).alive)
                else {
                    return Flow::Done;
                };
                m.target = Some(t);
                set_flag(ctx.core, owner, "l11_kuangxi_target", u32::from(t) + 1);
                m.stage = HostStage::SelectNature;
                emit_lose_hp(ctx, owner, 1);
                Flow::Yield
            }
            HostStage::SelectNature => {
                let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                    set_flag(ctx.core, owner, "l11_kuangxi_target", 0);
                    return Flow::Done;
                };
                m.stage = HostStage::DrawSettle;
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                Flow::Yield
            }
            _ => {
                set_flag(ctx.core, owner, "l11_kuangxi_target", 0);
                Flow::Done
            }
        },
        "txbaobian" => {
            sync_txbaobian(ctx, owner);
            Flow::Done
        }
        _ => Flow::Done,
    }
}
