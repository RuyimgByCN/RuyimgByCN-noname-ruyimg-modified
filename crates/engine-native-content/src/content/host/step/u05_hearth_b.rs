//! Batch U05-6a (`shahai-hearth` Part 1): Batch B 20 技能 (`ysjqisha` .. `zhuilie`) + 7 煞印子技。

use super::u05_hearth::{
    alive_others, discard_card_from_zone, flag, gain_card_to_hand, grant_skill, has_any_skill,
    has_skill_hearth, is_delayed_trick_hearth, is_hearth_enemy, is_max_hp, meta,
    pick_random_hearth_derivable, refund_once_used, remove_skill, rng_pick, rng_pick_mut,
    set_flag, spawn_card_by_kind,
};
use super::u05_hearth_a::use_gained_card;
use super::u05_hearth_asks::{
    set_active_ask, wait_hs, ASK_HS_AZAOWU_CARD, ASK_HS_AZAOWU_KIND, ASK_HS_AZAOWU_TARGET,
    ASK_HS_HLLINGXI_DISCARD, ASK_HS_HLLINGXI_TARGET, ASK_HS_LIEQI_GUESS, ASK_HS_LIEQI_TARGET,
    ASK_HS_TANSUO_CARD, ASK_HS_TANSUO_DISCOVER, ASK_HS_XIYONG_TARGET, ASK_HS_XIYONG_USE,
    ASK_HS_YINDAN_CARD, ASK_HS_ZHUILIE_DISCARD,
};
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, EventKind, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn trigger_meta_b(name: &str) -> Option<TriggerMeta> {
    Some(match name {
        "ysjqisha" => meta(EventKind::Damage, Timing::Normal, 0, false, Vec::new()),
        "ysjqisha_ju" | "ysjqisha_kuang" => meta(
            EventKind::CardUsed,
            Timing::Normal,
            0,
            false,
            vec![EventKind::TurnPhaseAfter],
        ),
        "ysjqisha_nu" => meta(
            EventKind::DamageBefore,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardUsed, EventKind::TurnPhaseAfter],
        ),
        "ysjqisha_hen" => meta(
            EventKind::Recover,
            Timing::Normal,
            0,
            false,
            vec![EventKind::TurnPhaseAfter],
        ),
        "ysjqisha_yi" | "ysjqisha_wang" | "ysjqisha_ao" => {
            meta(EventKind::TurnPhaseAfter, Timing::Normal, 0, false, Vec::new())
        }
        "hllingxi" => meta(EventKind::PhaseBegin, Timing::Normal, 0, true, Vec::new()),
        "zhaochao" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "xiyong" => meta(EventKind::PhaseBegin, Timing::Normal, 0, true, Vec::new()),
        "srjici" => meta(EventKind::Damage, Timing::Normal, 0, false, Vec::new()),
        "lieqi" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "qingtian" => meta(EventKind::DamageBefore, Timing::Normal, 0, false, Vec::new()),
        "qianfu" => meta(EventKind::Death, Timing::Normal, 0, false, Vec::new()),
        "qianfu2" => meta(
            EventKind::DamageBefore,
            Timing::Normal,
            0,
            false,
            vec![EventKind::Recover, EventKind::LoseHp],
        ),
        "shimo" => meta(EventKind::Damage, Timing::Normal, 0, false, Vec::new()),
        "lieyang" => meta(EventKind::CardUsed, Timing::Normal, 0, false, Vec::new()),
        "zhuilie" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        _ => return None,
    })
}

pub(super) fn limits_b(name: &str) -> SkillLimits {
    match name {
        "yindan" | "tansuo" | "azaowu" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(super) fn has_active_b(name: &str) -> bool {
    matches!(name, "yindan" | "hllingxi" | "tansuo" | "azaowu")
}

pub(super) fn activate_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<bool> {
    let pl = ctx.state.player(owner);
    Some(match skill.name.as_str() {
        "yindan" => pl
            .hand
            .iter()
            .chain(pl.equips.iter())
            .any(|&c| ctx.catalog.effective_card_suit(ctx.state, owner, c) == Suit::Spade),
        "hllingxi" => alive_others(ctx.state, owner).into_iter().any(|p| {
            let t = ctx.state.player(p);
            t.hp < t.max_hp as i32
                && (t.hand.len() + t.equips.len()) >= 2
                && flag(ctx.state, p, "hs_hllingxi_used_turn") == 0
        }),
        "tansuo" => !pl.hand.is_empty() || !pl.equips.is_empty(),
        "azaowu" => {
            flag(ctx.state, owner, "hs_azaowu_used_turn") == 0
                && pl.hand.iter().any(|&c| {
                    ctx.catalog.card_kind(c).is_basic_kind()
                        && super::u05_hearth::exact_base_name(ctx.catalog.cards[c as usize].name)
                            != "du"
                })
                && (super::u05_hearth::hearth_sha_can_play(ctx.state, ctx.catalog, owner)
                    || super::u05_hearth::hearth_jiu_can_play(ctx.state, ctx.catalog, owner)
                    || super::u05_hearth::hearth_tao_can_play(ctx.state, ctx.catalog, owner))
        }
        _ => return None,
    })
}

pub(super) fn filter_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.name.as_str() {
        "ysjqisha" => match event {
            EventData::Damage(d) if !d.prevented && d.amount > 0 => {
                (d.target == owner && d.source.is_some_and(|s| s != owner && ctx.state.player(s).alive))
                    || (d.source == Some(owner) && d.target != owner && ctx.state.player(d.target).alive)
            }
            _ => false,
        },
        "ysjqisha_ju" => match event {
            EventData::CardUsed(u) if u.player == owner => {
                !ctx.state.player(owner).hand.is_empty() || !ctx.state.player(owner).equips.is_empty()
            }
            EventData::TurnEndWindow { player, after: true } => *player == owner,
            _ => false,
        },
        "ysjqisha_kuang" => match event {
            EventData::CardUsed(u) if u.player == owner && u.targets.len() == 1 => true,
            EventData::TurnEndWindow { player, after: true } => *player == owner,
            _ => false,
        },
        "ysjqisha_nu" => match event {
            EventData::Damage(d) => d.source == Some(owner) && !d.from_chain && d.card.is_some(),
            EventData::CardUsed(u) => u.player == owner,
            EventData::TurnEndWindow { player, after: true } => *player == owner,
            _ => false,
        },
        "ysjqisha_hen" => match event {
            EventData::Recover { player, amount, .. } => {
                *amount > 0 && is_hearth_enemy(ctx.state, ctx.catalog, owner, *player)
            }
            EventData::TurnEndWindow { player, after: true } => *player == owner,
            _ => false,
        },
        "ysjqisha_yi" | "ysjqisha_wang" | "ysjqisha_ao" => {
            matches!(event, EventData::TurnEndWindow { player, after: true } if *player == owner)
        }
        "hllingxi" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } if *player == owner && ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
        ),
        "zhaochao" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } if *player == owner && ctx.state.alive_players().any(|p| is_hearth_enemy(ctx.state, ctx.catalog, owner, p))
        ),
        "xiyong" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } if *player == owner
        ),
        "srjici" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && !d.settlement.notrigger
                    && d.amount > 0
        ),
        "lieqi" => {
            let ok_phase = matches!(
                event,
                EventData::Phase {
                    player,
                    phase: crate::event::Phase::Prepare | crate::event::Phase::End,
                    begin: true,
                } if *player == owner
            );
            ok_phase
                && alive_others(ctx.state, owner).into_iter().any(|p| {
                    !ctx.state.player(p).hand.is_empty()
                        && u32::from(p) + 1 != flag(ctx.state, owner, "hs_lieqi_last")
                })
        }
        "qingtian" => matches!(
            event,
            EventData::Damage(d) if d.target == owner && !d.prevented && d.amount > 0 && is_max_hp(ctx.state, owner)
        ),
        "qianfu" => matches!(
            event,
            EventData::Death { player, .. }
                if *player == owner
                    && !has_skill_hearth(ctx.state, ctx.catalog, owner, "qianfu2")
                    && ctx.state.player(owner).max_hp > 0
        ),
        "qianfu2" => match event {
            EventData::Damage(d) => d.target == owner && d.nature != DamageNature::Fire && !d.prevented,
            EventData::Recover { player, .. } | EventData::LoseHp { player, .. } if *player == owner => {
                let pl = ctx.state.player(owner);
                pl.hp >= 3 || pl.hp >= pl.max_hp as i32
            }
            _ => false,
        },
        "shimo" => matches!(
            event,
            EventData::Damage(d)
                if !d.prevented
                    && d.amount > 0
                    && d.target != owner
                    && ctx.state.player(d.target).alive
                    && ctx.state.computed_distance(ctx.catalog, owner, d.target) <= 1
        ),
        "lieyang" => match event {
            EventData::CardUsed(u) if u.player == owner && ctx.state.turn.current == owner => {
                let k = u.effective_kind(ctx.catalog);
                !k.is_basic_kind()
                    && k.equip_slot().is_none()
                    && !is_delayed_trick_hearth(k)
                    && flag(ctx.state, owner, "hs_lieyang_turn_used") < 3
            }
            _ => false,
        },
        "zhuilie" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::Prepare,
                begin: true,
            } if *player == owner
                && (!ctx.state.player(owner).hand.is_empty() || !ctx.state.player(owner).equips.is_empty())
        ),
        _ => false,
    }
}

pub(super) fn step_b(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "ysjqisha" => step_ysjqisha(m, ctx),
        "ysjqisha_ju" | "ysjqisha_kuang" | "ysjqisha_nu" | "ysjqisha_yi" | "ysjqisha_wang"
        | "ysjqisha_hen" | "ysjqisha_ao" => step_ysjqisha_debuff(m, ctx),
        "yindan" => step_yindan(m, ctx),
        "hllingxi" => step_hllingxi(m, ctx),
        "zhaochao" => step_zhaochao(m, ctx),
        "xiyong" => step_xiyong(m, ctx),
        "srjici" => step_srjici(m, ctx),
        "tansuo" => step_tansuo(m, ctx),
        "lieqi" => step_lieqi(m, ctx),
        "azaowu" => step_azaowu(m, ctx),
        "qingtian" => {
            let _ = ctx.modify_damage_amount(1);
            Flow::Done
        }
        "qianfu" => {
            ctx.core.player_mut(m.owner).death_cancelled = true;
            ctx.core.player_mut(m.owner).hp = 1;
            let cards: Vec<CardId> = ctx
                .core
                .player(m.owner)
                .hand
                .iter()
                .chain(ctx.core.player(m.owner).equips.iter())
                .copied()
                .collect();
            for c in cards {
                discard_card_from_zone(ctx, m.owner, c);
            }
            grant_skill(ctx, m.owner, "qianfu2", false);
            set_flag(ctx.core, m.owner, "hs_qianfu2_active", 1);
            Flow::Done
        }
        "qianfu2" => {
            if ctx.current_damage().is_ok() {
                let d = ctx.current_damage_mut();
                d.prevented = true;
                d.settlement.outcome = crate::event::DamageOutcome::Cancelled;
            } else {
                remove_skill(ctx, m.owner, "qianfu2");
                set_flag(ctx.core, m.owner, "hs_qianfu2_active", 0);
                ctx.start_draw(m.owner, 3);
            }
            Flow::Done
        }
        "shimo" => {
            if ctx.core.player(m.owner).hp < ctx.core.player(m.owner).max_hp as i32 {
                ctx.recover(m.owner, 1, None);
            } else {
                ctx.start_draw(m.owner, 1);
            }
            Flow::Done
        }
        "lieyang" => step_lieyang(m, ctx),
        "zhuilie" => step_zhuilie(m, ctx),
        _ => Flow::Done,
    }
}

fn step_ysjqisha(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let other = ctx
        .res
        .stack
        .iter()
        .rev()
        .find_map(|f| match &f.data {
            EventData::Damage(d) => {
                if d.source == Some(m.owner) {
                    Some(d.target)
                } else {
                    d.source
                }
            }
            _ => None,
        });
    let Some(t) = other else {
        return Flow::Done;
    };
    if t == m.owner || !ctx.core.player(t).alive {
        return Flow::Done;
    }
    const DEBUFFS: [&str; 7] = [
        "ysjqisha_ju",
        "ysjqisha_kuang",
        "ysjqisha_nu",
        "ysjqisha_yi",
        "ysjqisha_wang",
        "ysjqisha_hen",
        "ysjqisha_ao",
    ];
    let avail: Vec<&str> = DEBUFFS
        .iter()
        .copied()
        .filter(|&s| !has_any_skill(ctx.core, ctx.catalog, t, s))
        .collect();
    if let Some(pick) = rng_pick(ctx, &avail) {
        grant_skill(ctx, t, pick, false);
        match pick {
            "ysjqisha_wang" => set_flag(ctx.core, t, "hs_ysjqisha_wang", 1),
            "ysjqisha_yi" => set_flag(ctx.core, t, "hs_ysjqisha_yi", 1),
            "ysjqisha_ao" => set_flag(ctx.core, t, "hs_ysjqisha_ao", 1),
            _ => {}
        }
    }
    Flow::Done
}

fn step_ysjqisha_debuff(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let cur = ctx.res.stack.last().map(|f| f.data.clone());
    if matches!(cur, Some(EventData::TurnEndWindow { after: true, .. })) {
        let name = m.def_key.clone();
        remove_skill(ctx, m.owner, &name);
        set_flag(ctx.core, m.owner, &format!("hs_{name}"), 0);
        return Flow::Done;
    }
    match m.def_key.as_str() {
        "ysjqisha_ju" => {
            let mut pool: Vec<CardId> = ctx
                .core
                .player(m.owner)
                .hand
                .iter()
                .chain(ctx.core.player(m.owner).equips.iter())
                .copied()
                .collect();
            if let Some(rc) = rng_pick_mut(ctx, &mut pool) {
                discard_card_from_zone(ctx, m.owner, rc);
                ctx.log(LogEvent::CardDiscarded {
                    player: m.owner,
                    cards: vec![rc],
                });
            }
        }
        "ysjqisha_kuang" => {
            if ctx.core.rng.gen_range(2) == 0 {
                let fid = ctx.current_frame_id();
                let old_t_opt = ctx.res.frame(fid).and_then(|f| match &f.data {
                    EventData::CardUsed(u) => u.targets.first().copied(),
                    _ => None,
                });
                if let Some(old_t) = old_t_opt {
                    let candidates: Vec<PlayerId> = ctx
                        .core
                        .alive_players()
                        .filter(|&p| p != old_t && p != m.owner)
                        .collect();
                    if let Some(new_t) = rng_pick(ctx, &candidates) {
                        if let Some(frame) = ctx.res.frame_mut(fid) {
                            frame.card_use_mut().targets[0] = new_t;
                        }
                    }
                }
            }
        }
        "ysjqisha_nu" => {
            if ctx.current_damage().is_ok() {
                let _ = ctx.modify_damage_amount(1);
            } else if matches!(cur, Some(EventData::CardUsed(_))) && ctx.core.rng.gen_range(100) < 65 {
                let fid = ctx.current_frame_id();
                if let Some(frame) = ctx.res.frame_mut(fid) {
                    frame.card_use_mut().targets.clear();
                }
            }
        }
        "ysjqisha_hen" => {
            ctx.emit(EventData::LoseHp {
                player: m.owner,
                amount: 1,
            });
        }
        _ => {}
    }
    Flow::Done
}

fn step_yindan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_YINDAN_CARD),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, m.owner, "yindan");
                return Flow::Done;
            };
            discard_card_from_zone(ctx, m.owner, picked);
            ctx.log(LogEvent::CardDiscarded {
                player: m.owner,
                cards: vec![picked],
            });
            ctx.emit(EventData::LoseHp {
                player: m.owner,
                amount: 1,
            });
            for _ in 0..2 {
                if let Some(bomb) = super::u05_hearth::spawn_card_by_base(ctx, "hsjixie_zhadan") {
                    gain_card_to_hand(ctx, m.owner, bomb);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_hllingxi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let in_phase_end = ctx
        .res
        .stack
        .last()
        .is_some_and(|f| matches!(f.data, EventData::Phase { .. }));
    if in_phase_end {
        ctx.recover(m.owner, 1, None);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_HLLINGXI_TARGET),
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(t);
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_HLLINGXI_DISCARD)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(t) = m.target else {
                return Flow::Done;
            };
            let picked: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            for &dc in &picked {
                discard_card_from_zone(ctx, t, dc);
            }
            if !picked.is_empty() {
                ctx.log(LogEvent::CardDiscarded {
                    player: t,
                    cards: picked,
                });
            }
            ctx.recover(t, 1, None);
            set_flag(ctx.core, t, "hs_hllingxi_used_turn", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_zhaochao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let mut enemies: Vec<PlayerId> = ctx
                .core
                .alive_players()
                .filter(|&p| is_hearth_enemy(ctx.core, ctx.catalog, m.owner, p))
                .collect();
            let Some(e1) = rng_pick_mut(ctx, &mut enemies) else {
                return Flow::Done;
            };
            m.targets = enemies;
            set_flag(ctx.core, m.owner, "hs_zhaochao_sha_pending", 1);
            set_flag(ctx.core, m.owner, "hs_zhaochao2_missed", 0);
            m.stage = HostStage::DrawSettle;
            ctx.use_virtual_card(m.owner, CardKind::Sha, vec![e1]);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            set_flag(ctx.core, m.owner, "hs_zhaochao_sha_pending", 0);
            let missed = flag(ctx.core, m.owner, "hs_zhaochao2_missed") > 0;
            set_flag(ctx.core, m.owner, "hs_zhaochao2_missed", 0);
            if missed {
                let remaining: Vec<PlayerId> = m
                    .targets
                    .iter()
                    .copied()
                    .filter(|&p| ctx.core.player(p).alive)
                    .collect();
                if let Some(e2) = rng_pick(ctx, &remaining) {
                    ctx.use_virtual_card(m.owner, CardKind::Sha, vec![e2]);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_xiyong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            m.cards = ctx.core.player(m.owner).hand.clone();
            ctx.start_draw(m.owner, 1);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let gained = ctx
                .core
                .player(m.owner)
                .hand
                .iter()
                .copied()
                .rfind(|c| !m.cards.contains(c));
            m.cards.clear();
            let Some(gained) = gained else {
                return Flow::Done;
            };
            if !super::u05_hearth_asks::is_tanmi_card_usable(
                ctx.core,
                ctx.catalog,
                m.owner,
                gained,
            ) {
                return Flow::Done;
            }
            m.cards = vec![gained];
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_XIYONG_USE)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if super::u05_hearth_asks::hearth_card_needs_target(
                ctx.core,
                ctx.catalog,
                m.owner,
                picked,
            ) {
                return wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_XIYONG_TARGET);
            }
            use_gained_card(ctx, m.owner, picked, Vec::new());
            ctx.start_draw(m.owner, 1);
            Flow::Done
        }
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            let picked_targets = super::u05_hearth_asks::collect_player_answers(ctx.answer());
            if let Some(&picked) = m.cards.first() {
                if !picked_targets.is_empty() {
                    use_gained_card(ctx, m.owner, picked, picked_targets);
                    ctx.start_draw(m.owner, 1);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_srjici(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(m.owner, 1);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if let Some(t) = m.target {
                if ctx.core.player(t).alive {
                    let he: Vec<CardId> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .chain(ctx.core.player(t).equips.iter())
                        .copied()
                        .collect();
                    if let Some(rc) = rng_pick(ctx, &he) {
                        discard_card_from_zone(ctx, t, rc);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![rc],
                        });
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_tansuo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_TANSUO_CARD),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, m.owner, "tansuo");
                return Flow::Done;
            };
            discard_card_from_zone(ctx, m.owner, picked);
            ctx.log(LogEvent::CardDiscarded {
                player: m.owner,
                cards: vec![picked],
            });
            let offered = pick_random_hearth_derivable(ctx, 3);
            if offered.is_empty() {
                ctx.start_draw(m.owner, 1);
                return Flow::Done;
            }
            m.cards = offered;
            wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_TANSUO_DISCOVER)
        }
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            if let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() {
                ctx.core.detached_cards.retain(|&(x, _)| x != picked);
                gain_card_to_hand(ctx, m.owner, picked);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_lieqi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let is_prep = ctx.res.stack.last().is_some_and(|f| {
                matches!(
                    f.data,
                    EventData::Phase {
                        phase: crate::event::Phase::Prepare,
                        ..
                    }
                )
            });
            set_flag(ctx.core, m.owner, "hs_lieqi_in_prep", u32::from(is_prep));
            wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_LIEQI_TARGET)
        }
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            let is_prep = flag(ctx.core, m.owner, "hs_lieqi_in_prep") > 0;
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                if !is_prep {
                    set_flag(ctx.core, m.owner, "hs_lieqi_last", 0);
                }
                return Flow::Done;
            };
            if is_prep {
                set_flag(ctx.core, m.owner, "hs_lieqi_last", u32::from(t) + 1);
            } else {
                set_flag(ctx.core, m.owner, "hs_lieqi_last", 0);
            }
            let hand = ctx.core.player(t).hand.clone();
            let Some(real_c) = rng_pick(ctx, &hand) else {
                return Flow::Done;
            };
            let real_names: Vec<&str> = hand
                .iter()
                .map(|&c| ctx.catalog.cards[c as usize].name)
                .collect();
            let decoys: Vec<CardId> = ctx
                .core
                .detached_cards
                .iter()
                .map(|&(c, _)| c)
                .filter(|&c| !real_names.contains(&ctx.catalog.cards[c as usize].name))
                .take(2)
                .collect();
            let mut opts = vec![real_c];
            opts.extend(decoys);
            // Deterministic RNG shuffle of 3 options
            for i in (1..opts.len()).rev() {
                let j = ctx.core.rng.gen_range((i + 1) as u32) as usize;
                opts.swap(i, j);
            }
            m.card = Some(real_c);
            m.cards = opts;
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_LIEQI_GUESS)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            if let (Some(real_c), Some(ChoiceOption::Card(picked))) =
                (m.card, ctx.answer().first().copied())
            {
                if picked == real_c {
                    let k = ctx.catalog.card_kind(real_c);
                    if let Some(copy) = spawn_card_by_kind(ctx, k) {
                        gain_card_to_hand(ctx, m.owner, copy);
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_azaowu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectNature, ASK_HS_AZAOWU_KIND),
        HostStage::SelectNature => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectNature, 0);
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, m.owner, "azaowu");
                return Flow::Done;
            };
            set_flag(ctx.core, m.owner, "hs_azaowu_picked_kind", u32::from(tag));
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_AZAOWU_CARD)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, m.owner, "azaowu");
                return Flow::Done;
            };
            m.cards = vec![card];
            wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_AZAOWU_TARGET)
        }
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            let (Some(&card), Some(ChoiceOption::Player(t))) =
                (m.cards.first(), ctx.answer().first().copied())
            else {
                refund_once_used(ctx, m.owner, "azaowu");
                return Flow::Done;
            };
            let kind_tag = flag(ctx.core, m.owner, "hs_azaowu_picked_kind");
            let as_kind = match kind_tag {
                0 => CardKind::Sha,
                1 => CardKind::FireSha,
                2 => CardKind::LeiSha,
                3 => CardKind::Jiu,
                _ => CardKind::Tao,
            };
            set_flag(ctx.core, m.owner, "hs_azaowu_used_turn", 1);
            ctx.take_hand_card(m.owner, card);
            ctx.core.table.push(card);
            ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                card,
                m.owner,
                if matches!(as_kind, CardKind::Jiu) {
                    Vec::new()
                } else {
                    vec![t]
                },
                Some(as_kind),
            )));
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_lieyang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let used = flag(ctx.core, m.owner, "hs_lieyang_turn_used") + 1;
    set_flag(ctx.core, m.owner, "hs_lieyang_turn_used", used);
    let pool = [
        CardKind::Wuzhong,
        CardKind::Duel,
        CardKind::Dismantle,
        CardKind::Snatch,
        CardKind::Chain,
        CardKind::SavageAttack,
        CardKind::ArcheryAttack,
        CardKind::Taoyuan,
        CardKind::Wugu,
    ];
    if let Some(pick) = rng_pick(ctx, &pool) {
        if let Some(card) = spawn_card_by_kind(ctx, pick) {
            gain_card_to_hand(ctx, m.owner, card);
        }
    }
    if used >= 3 {
        grant_skill(ctx, m.owner, "lieyang2", true);
    }
    Flow::Done
}

fn step_zhuilie(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_ZHUILIE_DISCARD),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            discard_card_from_zone(ctx, m.owner, picked);
            ctx.log(LogEvent::CardDiscarded {
                player: m.owner,
                cards: vec![picked],
            });
            let mut discarded_basics = 0usize;
            let n = ctx.core.deck.len().min(6);
            let top_start = ctx.core.deck.len() - n;
            let top6: Vec<CardId> = ctx.core.deck[top_start..].iter().copied().collect();
            for c in top6.into_iter().rev() {
                if ctx.catalog.card_kind(c).is_basic_kind() {
                    if let Some(pos) = ctx.core.deck.iter().rposition(|&x| x == c) {
                        ctx.core.deck.remove(pos);
                    }
                    ctx.to_discard(c);
                    discarded_basics += 1;
                }
            }
            if discarded_basics > 3 {
                ctx.start_draw(m.owner, 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
