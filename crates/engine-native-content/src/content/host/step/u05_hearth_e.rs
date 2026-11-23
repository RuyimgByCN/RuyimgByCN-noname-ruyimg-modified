//! Batch U05-6a (`shahai-hearth` Part 1): Batch E 27 技能 (`shengteng` .. `hsbaowu_huangjinyuanhou2`)。

use super::u05_hearth::{
    alive_others, change_max_hp, discard_card_from_zone, emit_hearth_virtual, equip_card_to_player,
    exact_base_name, flag, gain_card_to_hand, grant_skill, has_any_skill, has_skill_hearth,
    hearth_card_type_tag, is_delayed_trick_hearth, meta, pop_detached_matching,
    refund_once_used, remove_skill, rng_pick, set_flag, spawn_card_by_base, spawn_card_by_kind,
    take_he_card_as,
};
use super::u05_hearth_a::use_gained_card;
use super::u05_hearth_asks::{
    hearth_card_needs_target, is_tanmi_card_usable, set_active_ask, wait_hs,
    ASK_HS_MAOXIAN_SKILL, ASK_HS_MOBAO_CARDS, ASK_HS_NUYAN2_KIND, ASK_HS_NUYAN2_TARGET,
    ASK_HS_NUYAN_CARD, ASK_HS_NUYAN_KIND, ASK_HS_NUYAN_TARGET, ASK_HS_QIANGHUAX_CARDS,
    ASK_HS_TANMI_TARGET, ASK_HS_TANMI_USE, ASK_HS_XIANJI2_DISCARD, ASK_HS_XUEREN_CONFIRM,
    ASK_HS_XUNBAO2_DISCARD, ASK_HS_YUANSU_CONFIRM, ASK_HS_ZENGLI_TARGET, ASK_HS_ZHUIZONG_CARDS,
    ASK_HS_ZHUIZONG_PICK,
};
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, EventKind, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn trigger_meta_e(name: &str) -> Option<TriggerMeta> {
    Some(match name {
        "shengteng" => meta(EventKind::Damage, Timing::Normal, 0, false, Vec::new()),
        "chouhuo" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "duxin" | "oldduxin" => meta(EventKind::PhaseBegin, Timing::Normal, 0, true, Vec::new()),
        "hsdusu_shinancao" => meta(EventKind::DamageBefore, Timing::Normal, 0, false, Vec::new()),
        "mobao" => meta(
            EventKind::Damage,
            Timing::Normal,
            0,
            false,
            vec![EventKind::PhaseBegin],
        ),
        "mobao2" => meta(EventKind::Damage, Timing::Normal, 0, false, Vec::new()),
        "mobao3" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "xianji" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "xianji3" => meta(EventKind::CardUsed, Timing::Normal, 0, false, Vec::new()),
        "tanmi" => meta(EventKind::PhaseBegin, Timing::Normal, 0, true, Vec::new()),
        "xueren" => meta(EventKind::Damage, Timing::Normal, 0, false, Vec::new()),
        "yiwen" => meta(
            EventKind::CardUsed,
            Timing::Normal,
            0,
            false,
            vec![EventKind::TurnPhaseAfter],
        ),
        "xunbao" => meta(EventKind::PhaseBegin, Timing::Normal, 1, true, Vec::new()),
        "xunbao2" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "hsbaowu_cangbaotu" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "hsbaowu_huangjinyuanhou2" => {
            meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new())
        }
        _ => return None,
    })
}

pub(super) fn limits_e(name: &str) -> SkillLimits {
    match name {
        "yuansu" | "nuyan" | "zengli" | "qianghuax" | "zhuizong" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(super) fn has_active_e(name: &str) -> bool {
    matches!(
        name,
        "yuansu" | "nuyan" | "nuyan2" | "zengli" | "mobao" | "maoxian" | "qianghuax" | "zhuizong"
    )
}

pub(super) fn activate_e(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<bool> {
    let pl = ctx.state.player(owner);
    Some(match skill.name.as_str() {
        "yuansu" => pl.max_hp as i32 - pl.hp >= 3,
        "nuyan" => {
            flag(ctx.state, owner, "hs_nuyan_used_mask") != 0b11_1111
                && pl.hand.iter().chain(pl.equips.iter()).any(|&c| {
                    matches!(
                        ctx.catalog.effective_card_suit(ctx.state, owner, c),
                        Suit::Heart | Suit::Diamond
                    )
                })
        }
        "nuyan2" => pl.hp > 0 && flag(ctx.state, owner, "hs_nuyan2_used_turn") < 3,
        "zengli" => !alive_others(ctx.state, owner).is_empty(),
        "mobao" => {
            flag(ctx.state, owner, "hs_mobao_used_turn") == 0
                && flag(ctx.state, owner, "hs_mobao_mask") != 0
                && alive_others(ctx.state, owner)
                    .into_iter()
                    .any(|p| (flag(ctx.state, owner, "hs_mobao_mask") & (1 << p)) != 0)
                && pl.hand.iter().chain(pl.equips.iter()).any(|&c| {
                    matches!(
                        ctx.catalog.effective_card_suit(ctx.state, owner, c),
                        Suit::Spade | Suit::Club
                    )
                })
        }
        "maoxian" => flag(ctx.state, owner, "hs_maoxian_used_turn") < 2,
        "qianghuax" | "zhuizong" => !pl.hand.is_empty() || !pl.equips.is_empty(),
        _ => return None,
    })
}

pub(super) fn filter_e(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.name.as_str() {
        "shengteng" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.card.is_some_and(|c| {
                        let k = ctx.catalog.card_kind(c);
                        !k.is_basic_kind() && k.equip_slot().is_none() && !is_delayed_trick_hearth(k)
                    })
        ),
        "chouhuo" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::Prepare,
                begin: true,
            } if *player == owner
                && has_skill_hearth(ctx.state, ctx.catalog, owner, "nuyan")
                && flag(ctx.state, owner, "hs_nuyan_used_mask") == 0b11_1111
        ),
        "duxin" => match event {
            EventData::Phase {
                player,
                phase: crate::event::Phase::Prepare | crate::event::Phase::End,
                begin: true,
            } => {
                *player == owner
                    && !ctx.state.player(owner).hand.iter().any(|&c| {
                        exact_base_name(ctx.catalog.cards[c as usize].name).starts_with("hsdusu_")
                    })
            }
            _ => false,
        },
        "oldduxin" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::Prepare | crate::event::Phase::End,
                begin: true,
            } if *player == owner
        ),
        "hsdusu_shinancao" => matches!(
            event,
            EventData::Damage(d) if d.source == Some(owner) && !d.prevented && d.amount > 0
        ),
        "mobao" => match event {
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.source.is_some_and(|s| s != owner)
            }
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } => *player == owner && flag(ctx.state, owner, "hs_mobao_mask") != 0,
            _ => false,
        },
        "mobao2" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.source.is_some_and(|s| s != owner)
        ),
        "mobao3" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } if *player == owner && flag(ctx.state, owner, "hs_mobao_mask") != 0
        ),
        "xianji" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } if *player != owner
                && !has_skill_hearth(ctx.state, ctx.catalog, *player, "xianji")
                && (!ctx.state.player(*player).hand.is_empty()
                    || has_any_skill(ctx.state, ctx.catalog, *player, "xianji3"))
        ),
        "xianji3" => matches!(
            event,
            EventData::CardUsed(u)
                if u.targets.contains(&owner)
                    && flag(ctx.state, owner, "hs_xianji3_source") == u32::from(u.player) + 1
        ),
        "tanmi" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } if *player != owner && ctx.state.player(owner).hand.is_empty()
        ),
        "xueren" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.player(d.target).alive
                    && d.card.is_some_and(|c| ctx.catalog.card_kind(c).is_sha_like())
        ),
        "yiwen" => match event {
            EventData::CardUsed(u) => {
                u.targets.len() == 1
                    && u.targets[0] == owner
                    && u.player != owner
                    && ctx.state.turn.current == u.player
                    && flag(ctx.state, u.player, "hs_yiwen2_turn") == 0
            }
            EventData::TurnEndWindow { after: true, .. } => true,
            _ => false,
        },
        "xunbao" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::Prepare,
                begin: true,
            } if *player == owner && !has_any_skill(ctx.state, ctx.catalog, owner, "xunbao2")
        ),
        "xunbao2" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::Prepare,
                begin: true,
            } if *player == owner && {
                let want = flag(ctx.state, owner, "hs_xunbao_rank") as u8;
                want > 0
                    && ctx
                        .state
                        .player(owner)
                        .hand
                        .iter()
                        .chain(ctx.state.player(owner).equips.iter())
                        .any(|&c| ctx.catalog.cards[c as usize].rank == want)
            }
        ),
        "hsbaowu_cangbaotu" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } if *player == owner
        ),
        "hsbaowu_huangjinyuanhou2" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::Prepare,
                begin: true,
            } if *player == owner
        ),
        _ => false,
    }
}

pub(super) fn step_e(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "shengteng" => {
            change_max_hp(ctx, m.owner, 1);
            ctx.recover(m.owner, 1, None);
            Flow::Done
        }
        "yuansu" => step_yuansu(m, ctx),
        "chouhuo" => {
            change_max_hp(ctx, m.owner, -1);
            ctx.change_hujia(m.owner, 2);
            remove_skill(ctx, m.owner, "nuyan");
            grant_skill(ctx, m.owner, "nuyan2", false);
            Flow::Done
        }
        "nuyan" | "nuyan2" => step_nuyan(m, ctx, m.def_key == "nuyan2"),
        "duxin" | "oldduxin" => {
            let pool = [
                "hsdusu_xueji",
                "hsdusu_huangxuecao",
                "hsdusu_kuyecao",
                "hsdusu_shinancao",
                "hsdusu_huoyanhua",
            ];
            if let Some(pick) = rng_pick(ctx, &pool) {
                if let Some(c) = spawn_card_by_base(ctx, pick) {
                    gain_card_to_hand(ctx, m.owner, c);
                }
            }
            Flow::Done
        }
        "hsdusu_shinancao" => {
            for f in ctx.res.stack.iter_mut().rev() {
                if let EventData::Damage(d) = &mut f.data {
                    if d.source == Some(m.owner) && !d.prevented {
                        d.amount += 1;
                        break;
                    }
                }
            }
            remove_skill(ctx, m.owner, "hsdusu_shinancao");
            Flow::Done
        }
        "xiubu" => Flow::Done,
        "zengli" => step_zengli(m, ctx),
        "mobao" => {
            if let Ok(d) = ctx.current_damage().cloned() {
                if let Some(src) = d.source {
                    let mask = flag(ctx.core, m.owner, "hs_mobao_mask");
                    set_flag(ctx.core, m.owner, "hs_mobao_mask", mask | (1 << src));
                }
                Flow::Done
            } else if matches!(
                ctx.res.stack.last().map(|f| &f.data),
                Some(EventData::Phase { .. })
            ) {
                set_flag(ctx.core, m.owner, "hs_mobao_mask", 0);
                Flow::Done
            } else {
                step_mobao(m, ctx)
            }
        }
        "mobao2" => {
            if let Some(src) = ctx.res.stack.iter().rev().find_map(|f| match &f.data {
                EventData::Damage(d) if d.target == m.owner => d.source,
                _ => None,
            }) {
                let mask = flag(ctx.core, m.owner, "hs_mobao_mask");
                set_flag(ctx.core, m.owner, "hs_mobao_mask", mask | (1 << src));
            }
            Flow::Done
        }
        "mobao3" => {
            set_flag(ctx.core, m.owner, "hs_mobao_mask", 0);
            Flow::Done
        }
        "xianji" | "xianji2" => step_xianji(m, ctx),
        "xianji3" => {
            ctx.start_draw(m.owner, 1);
            Flow::Done
        }
        "tanmi" => step_tanmi(m, ctx),
        "xueren" => step_xueren(m, ctx),
        "maoxian" => step_maoxian(m, ctx),
        "yiwen" => step_yiwen(m, ctx),
        "qianghuax" => step_qianghuax(m, ctx),
        "zhuizong" => step_zhuizong(m, ctx),
        "xunbao" => {
            if let Some(c) = spawn_card_by_base(ctx, "hsbaowu_cangbaotu") {
                let rank = u32::from(ctx.catalog.cards[c as usize].rank);
                ctx.core
                    .detached_cards
                    .push((c, crate::event::DetachedKind::Unclaimed));
                set_flag(ctx.core, m.owner, "hs_xunbao_card_id", u32::from(c) + 1);
                set_flag(ctx.core, m.owner, "hs_xunbao_rank", rank);
                grant_skill(ctx, m.owner, "xunbao2", false);
                if let Some(def) = ctx.catalog.skill_by_name("xunbao2") {
                    ctx.core
                        .vm_aux
                        .marks
                        .insert((m.owner, u32::from(def)), rank as i32);
                }
            }
            Flow::Done
        }
        "xunbao2" => step_xunbao2(m, ctx),
        "hsbaowu_cangbaotu" => {
            if let Some(c) = spawn_card_by_base(ctx, "hsbaowu_huangjinyuanhou") {
                gain_card_to_hand(ctx, m.owner, c);
            }
            remove_skill(ctx, m.owner, "hsbaowu_cangbaotu");
            Flow::Done
        }
        "hsbaowu_huangjinyuanhou" | "hsbaowu_huangjinyuanhou2" => {
            remove_skill(ctx, m.owner, "hsbaowu_huangjinyuanhou");
            remove_skill(ctx, m.owner, "hsbaowu_huangjinyuanhou2");
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_yuansu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::CompareTargetPick, ASK_HS_YUANSU_CONFIRM),
        HostStage::CompareTargetPick => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::CompareTargetPick, 0);
            if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                refund_once_used(ctx, m.owner, "yuansu");
                return Flow::Done;
            }
            let hp = ctx.core.player(m.owner).hp.max(1) as u32;
            ctx.core.player_mut(m.owner).max_hp = hp;
            let targets = alive_others(ctx.core, m.owner);
            emit_hearth_virtual(ctx, m.owner, CardKind::Yuansuhuimie, targets);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn nuyan_kind_from_tag(tag: u32) -> CardKind {
    match tag {
        0 => CardKind::Sha,
        1 => CardKind::Duel,
        2 => CardKind::SavageAttack,
        3 => CardKind::ArcheryAttack,
        4 => CardKind::FireSha,
        _ => CardKind::Yuansuhuimie,
    }
}

fn step_nuyan(m: &mut HostMachine, ctx: &mut Ctx<'_>, is_nuyan2: bool) -> Flow {
    let refund = |ctx: &mut Ctx<'_>| {
        if !is_nuyan2 {
            refund_once_used(ctx, m.owner, "nuyan");
        }
    };
    match m.stage {
        HostStage::Act => {
            let ask = if is_nuyan2 {
                ASK_HS_NUYAN2_KIND
            } else {
                ASK_HS_NUYAN_KIND
            };
            wait_hs(m, ctx, HostStage::SelectNature, ask)
        }
        HostStage::SelectNature => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectNature, 0);
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                refund(ctx);
                return Flow::Done;
            };
            set_flag(ctx.core, m.owner, "hs_nuyan_picked_kind", u32::from(tag));
            if is_nuyan2 {
                if matches!(tag, 2 | 3 | 5) {
                    let used = flag(ctx.core, m.owner, "hs_nuyan2_used_turn") + 1;
                    set_flag(ctx.core, m.owner, "hs_nuyan2_used_turn", used);
                    m.stage = HostStage::DrawSettle;
                    ctx.emit(EventData::LoseHp {
                        player: m.owner,
                        amount: 1,
                    });
                    return Flow::Yield;
                }
                return wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_NUYAN2_TARGET);
            }
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_NUYAN_CARD)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                refund(ctx);
                return Flow::Done;
            };
            m.cards = vec![picked];
            let tag = flag(ctx.core, m.owner, "hs_nuyan_picked_kind");
            if matches!(tag, 2 | 3 | 5) {
                let mask = flag(ctx.core, m.owner, "hs_nuyan_used_mask");
                set_flag(ctx.core, m.owner, "hs_nuyan_used_mask", mask | (1 << tag));
                let kind = nuyan_kind_from_tag(tag);
                let others = alive_others(ctx.core, m.owner);
                take_he_card_as(ctx, m.owner, picked, others, kind);
                return Flow::Done;
            }
            wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_NUYAN_TARGET)
        }
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                refund(ctx);
                return Flow::Done;
            };
            let tag = flag(ctx.core, m.owner, "hs_nuyan_picked_kind");
            let kind = nuyan_kind_from_tag(tag);
            if is_nuyan2 {
                let used = flag(ctx.core, m.owner, "hs_nuyan2_used_turn") + 1;
                set_flag(ctx.core, m.owner, "hs_nuyan2_used_turn", used);
                m.target = Some(t);
                m.stage = HostStage::DrawSettle;
                ctx.emit(EventData::LoseHp {
                    player: m.owner,
                    amount: 1,
                });
                return Flow::Yield;
            } else if let Some(&card) = m.cards.first() {
                let mask = flag(ctx.core, m.owner, "hs_nuyan_used_mask");
                set_flag(ctx.core, m.owner, "hs_nuyan_used_mask", mask | (1 << tag));
                take_he_card_as(ctx, m.owner, card, vec![t], kind);
            }
            Flow::Done
        }
        HostStage::DrawSettle => {
            if !ctx.core.player(m.owner).alive {
                return Flow::Done;
            }
            let tag = flag(ctx.core, m.owner, "hs_nuyan_picked_kind");
            let kind = nuyan_kind_from_tag(tag);
            let targets = if matches!(tag, 2 | 3 | 5) {
                alive_others(ctx.core, m.owner)
            } else {
                m.target.into_iter().collect()
            };
            emit_hearth_virtual(ctx, m.owner, kind, targets);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_zengli(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_ZENGLI_TARGET),
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, m.owner, "zengli");
                return Flow::Done;
            };
            m.target = Some(t);
            m.stage = HostStage::DrawSettle;
            if let Some(w) = super::u05_hearth::pop_random_detached_weapon(ctx) {
                equip_card_to_player(ctx, m.owner, w);
            }
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if let Some(t) = m.target {
                if ctx.core.player(t).alive {
                    if let Some(w) = super::u05_hearth::pop_random_detached_weapon(ctx) {
                        equip_card_to_player(ctx, t, w);
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_mobao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_MOBAO_CARDS),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let picked: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if picked.is_empty() {
                return Flow::Done;
            }
            set_flag(ctx.core, m.owner, "hs_mobao_used_turn", 1);
            let n = picked.len() as i32;
            for &dc in &picked {
                discard_card_from_zone(ctx, m.owner, dc);
            }
            ctx.log(LogEvent::CardDiscarded {
                player: m.owner,
                cards: picked,
            });
            let mask = flag(ctx.core, m.owner, "hs_mobao_mask");
            for p in alive_others(ctx.core, m.owner) {
                if (mask & (1 << p)) != 0 {
                    ctx.emit(EventData::Damage(super::u05_hearth::make_damage(
                        Some(m.owner),
                        p,
                        n,
                        DamageNature::Thunder,
                        None,
                    )));
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_xianji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let Some(t) = m.target else {
                return Flow::Done;
            };
            set_flag(ctx.core, t, "hs_xianji3_source", 0);
            remove_skill(ctx, t, "xianji3");
            if ctx.core.player(t).hand.is_empty() {
                return Flow::Done;
            }
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_XIANJI2_DISCARD)
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
            if picked.is_empty() {
                return Flow::Done;
            }
            let n = picked.len() as u32;
            for &dc in &picked {
                discard_card_from_zone(ctx, t, dc);
            }
            ctx.log(LogEvent::CardDiscarded {
                player: t,
                cards: picked,
            });
            ctx.start_draw(m.owner, n);
            set_flag(ctx.core, t, "hs_xianji3_source", u32::from(m.owner) + 1);
            grant_skill(ctx, t, "xianji3", false);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_tanmi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    loop {
        match m.stage {
            HostStage::Act => {
                ctx.start_draw(m.owner, 2);
                set_flag(ctx.core, m.owner, "hs_tanmi_used_count", 0);
                m.stage = HostStage::DrawSettle;
                return Flow::Yield;
            }
            HostStage::DrawSettle => {
                let cnt = flag(ctx.core, m.owner, "hs_tanmi_used_count");
                if cnt >= 2 {
                    return Flow::Done;
                }
                let has_usable = ctx
                    .core
                    .player(m.owner)
                    .hand
                    .iter()
                    .any(|&c| is_tanmi_card_usable(ctx.core, ctx.catalog, m.owner, c));
                if !has_usable {
                    return Flow::Done;
                }
                return wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_TANMI_USE);
            }
            HostStage::SelectCard => {
                set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
                let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.cards = vec![picked];
                if hearth_card_needs_target(ctx.core, ctx.catalog, m.owner, picked) {
                    return wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_TANMI_TARGET);
                }
                let next_cnt = flag(ctx.core, m.owner, "hs_tanmi_used_count") + 1;
                set_flag(ctx.core, m.owner, "hs_tanmi_used_count", next_cnt);
                use_gained_card(ctx, m.owner, picked, Vec::new());
                m.stage = HostStage::DrawSettle;
                return Flow::Yield;
            }
            HostStage::TargetChoice => {
                set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
                let picked_targets = super::u05_hearth_asks::collect_player_answers(ctx.answer());
                let Some(&card) = m.cards.first() else {
                    return Flow::Done;
                };
                if picked_targets.is_empty() {
                    return Flow::Done;
                }
                let next_cnt = flag(ctx.core, m.owner, "hs_tanmi_used_count") + 1;
                set_flag(ctx.core, m.owner, "hs_tanmi_used_count", next_cnt);
                use_gained_card(ctx, m.owner, card, picked_targets);
                m.stage = HostStage::DrawSettle;
                return Flow::Yield;
            }
            _ => return Flow::Done,
        }
    }
}

fn step_xueren(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::CompareTargetPick, ASK_HS_XUEREN_CONFIRM),
        HostStage::CompareTargetPick => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::CompareTargetPick, 0);
            if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                return Flow::Done;
            }
            if let Some(t) = m.target {
                if ctx.core.player(t).alive {
                    ctx.emit(EventData::LoseHp {
                        player: t,
                        amount: 1,
                    });
                }
            }
            ctx.emit(EventData::LoseHp {
                player: m.owner,
                amount: 1,
            });
            ctx.start_draw(m.owner, 2);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_maoxian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let mut pool = super::u05_hearth::collect_hearth_gainable_skills(ctx.catalog, "maoxian");
            pool.retain(|&d| !has_any_skill(ctx.core, ctx.catalog, m.owner, ctx.catalog.skill_name(d)));
            if pool.is_empty() {
                return Flow::Done;
            }
            let mut offer = Vec::new();
            while !pool.is_empty() && offer.len() < 3 {
                let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                offer.push(pool.swap_remove(idx));
            }
            set_flag(ctx.core, m.owner, "hs_choice_count", offer.len() as u32);
            for (i, d) in offer.iter().enumerate() {
                set_flag(ctx.core, m.owner, &format!("hs_choice_sk:{i}"), u32::from(*d) + 1);
            }
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_MAOXIAN_SKILL)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Tag(idx)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let raw = flag(ctx.core, m.owner, &format!("hs_choice_sk:{idx}"));
            if raw > 0 {
                let used = flag(ctx.core, m.owner, "hs_maoxian_used_turn") + 1;
                set_flag(ctx.core, m.owner, "hs_maoxian_used_turn", used);
                let old = flag(ctx.core, m.owner, "hs_maoxian_sk");
                if old > 0 {
                    let old_name = ctx.catalog.skill_name((old - 1) as u16).to_string();
                    remove_skill(ctx, m.owner, &old_name);
                }
                let name = ctx.catalog.skill_name((raw - 1) as u16).to_string();
                grant_skill(ctx, m.owner, &name, false);
                set_flag(ctx.core, m.owner, "hs_maoxian_sk", raw);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_yiwen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if matches!(
        ctx.res.stack.last().map(|f| &f.data),
        Some(EventData::TurnEndWindow { after: true, .. })
    ) {
        let seats = ctx.core.players.len() as PlayerId;
        for p in 0..seats {
            set_flag(ctx.core, p, "hs_yiwen2_turn", 0);
        }
        return Flow::Done;
    }
    if let Some((user, kind)) = ctx.res.stack.iter().rev().find_map(|f| match &f.data {
        EventData::CardUsed(u) if u.targets.contains(&m.owner) => {
            Some((u.player, u.effective_kind(ctx.catalog)))
        }
        _ => None,
    }) {
        if let Some(copy) = spawn_card_by_kind(ctx, kind) {
            gain_card_to_hand(ctx, m.owner, copy);
        }
        set_flag(ctx.core, user, "hs_yiwen2_turn", 1);
    }
    Flow::Done
}

fn step_qianghuax(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_QIANGHUAX_CARDS),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let raw: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            let mut seen_types = 0u8;
            let mut uniq = Vec::new();
            for c in raw {
                let bit = 1u8 << hearth_card_type_tag(ctx.catalog, c);
                if (seen_types & bit) == 0 {
                    seen_types |= bit;
                    uniq.push(c);
                }
            }
            if uniq.is_empty() {
                refund_once_used(ctx, m.owner, "qianghuax");
                return Flow::Done;
            }
            let tags: Vec<(u8, CardKind)> = uniq
                .iter()
                .map(|&c| (hearth_card_type_tag(ctx.catalog, c), ctx.catalog.card_kind(c)))
                .collect();
            for &dc in &uniq {
                discard_card_from_zone(ctx, m.owner, dc);
            }
            ctx.log(LogEvent::CardDiscarded {
                player: m.owner,
                cards: uniq,
            });
            for (ty, orig_kind) in tags {
                let mut cand: Vec<CardKind> = match ty {
                    0 => [CardKind::Tao, CardKind::Jiu, CardKind::FireSha, CardKind::LeiSha, CardKind::Sha]
                        .into_iter()
                        .filter(|&k| k != orig_kind)
                        .collect(),
                    2 => [CardKind::Weapon, CardKind::Armor, CardKind::HorsePlus, CardKind::HorseMinus]
                        .into_iter()
                        .filter(|&k| k != orig_kind)
                        .collect(),
                    _ => [
                        CardKind::Wuzhong,
                        CardKind::Taoyuan,
                        CardKind::Duel,
                        CardKind::Snatch,
                        CardKind::Dismantle,
                        CardKind::SavageAttack,
                        CardKind::ArcheryAttack,
                        CardKind::Yuansuhuimie,
                        CardKind::Shijieshu,
                    ]
                    .into_iter()
                    .filter(|&k| k != orig_kind)
                    .collect(),
                };
                while !cand.is_empty() {
                    let i = ctx.core.rng.gen_range(cand.len() as u32) as usize;
                    let k = cand.swap_remove(i);
                    if let Some(g) = spawn_card_by_kind(ctx, k) {
                        gain_card_to_hand(ctx, m.owner, g);
                        break;
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_zhuizong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_ZHUIZONG_CARDS),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let picked: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if picked.is_empty() {
                refund_once_used(ctx, m.owner, "zhuizong");
                return Flow::Done;
            }
            let draw_n = 4 * picked.len();
            for &dc in &picked {
                discard_card_from_zone(ctx, m.owner, dc);
            }
            ctx.log(LogEvent::CardDiscarded {
                player: m.owner,
                cards: picked,
            });
            let mut looked = Vec::new();
            for _ in 0..draw_n {
                if let Some(c) = ctx.core.deck.pop() {
                    looked.push(c);
                }
            }
            if looked.is_empty() {
                return Flow::Done;
            }
            ctx.core
                .vm_aux
                .piles
                .insert((m.owner, "hs_zhuizong_holding".to_string()), looked);
            wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_ZHUIZONG_PICK)
        }
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            let held = ctx
                .core
                .vm_aux
                .piles
                .remove(&(m.owner, "hs_zhuizong_holding".to_string()))
                .unwrap_or_default();
            let chosen = match ctx.answer().first().copied() {
                Some(ChoiceOption::Card(c)) if held.contains(&c) => c,
                _ => held.first().copied().unwrap_or(0),
            };
            for c in held {
                if c == chosen {
                    gain_card_to_hand(ctx, m.owner, c);
                } else {
                    ctx.to_discard(c);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_xunbao2(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_XUNBAO2_DISCARD),
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
            let raw = flag(ctx.core, m.owner, "hs_xunbao_card_id");
            set_flag(ctx.core, m.owner, "hs_xunbao_card_id", 0);
            set_flag(ctx.core, m.owner, "hs_xunbao_rank", 0);
            if raw > 0 {
                let target_cid = (raw - 1) as CardId;
                if let Some(c) = pop_detached_matching(ctx.core, |cid| cid == target_cid) {
                    gain_card_to_hand(ctx, m.owner, c);
                }
            }
            remove_skill(ctx, m.owner, "xunbao2");
            if let Some(def) = ctx.catalog.skill_by_name("xunbao2") {
                ctx.core.vm_aux.marks.remove(&(m.owner, u32::from(def)));
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
