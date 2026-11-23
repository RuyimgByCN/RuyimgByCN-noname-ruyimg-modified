//! Batch U05-6b (`shahai-hearth` Part 2) Group C: 18 技能 (`lingzhou` .. `shengdun`)。

use super::asks::{
    ASK_B6B_ENZE_DISCARD, ASK_B6B_ENZE_TARGET, ASK_B6B_GUOZAI2_DISCARD, ASK_B6B_HUANWU_TARGET,
    ASK_B6B_LINGZHOU_CHOICE, ASK_B6B_LINGZHOU_TARGET, ASK_B6B_OLDENZE_TARGET,
    ASK_B6B_XMOJIAN_TARGET, clear_b6b_ask, wait_b6b,
};
use super::{
    b6b_flag, clear_b6b_flag, emit_free_sha_no_count, hs_b6b_cnt, hs_b6b_inc, set_b6b_flag,
    sha_target_options,
};
use super::super::u05_hearth::{
    alive_others, change_max_hp, discard_cards_from_zone, flag, grant_skill, make_damage, meta,
    remove_skill, rng_pick, set_flag,
};
use super::super::{HostMachine, HostSkillBehavior, HostStage};
use crate::catalog::{QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn trigger_meta_c(name: &str) -> Option<TriggerMeta> {
    match name {
        "lingzhou" | "fengnu" => {
            Some(meta(EventKind::CardUsed, Timing::Normal, 0, false, Vec::new()))
        }
        "mieshi" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new())),
        "xmojian" => Some(meta(EventKind::TurnOvered, Timing::Normal, 0, false, Vec::new())),
        "xshixin" | "hanshuang" | "oldhanshuang" => {
            Some(meta(EventKind::DamageEnd, Timing::Normal, 0, false, Vec::new()))
        }
        "guozai2" | "guozaix2" => {
            Some(meta(EventKind::PhaseEnd, Timing::First, 0, false, Vec::new()))
        }
        "shengdun" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new())),
        // `chongsheng` 由 dying save 挂点处理；`bingshi` / `bingshi2` 由 `b6b_on_player_died` 处理。
        _ => None,
    }
}

pub(super) fn limits_c(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(super) fn has_active_c(name: &str) -> bool {
    matches!(
        name,
        "enze" | "oldenze" | "guozai" | "guozaix" | "huanwu"
    )
}

pub(super) fn filter_c(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let st = ctx.state;
    match skill.name.as_str() {
        "lingzhou" => match event {
            EventData::CardUsed(u) if u.player == owner && !u.cancelled && u.card.is_some() => {
                let eff = u.effective_kind(ctx.catalog);
                !eff.is_basic_kind() && eff.equip_slot().is_none()
            }
            _ => false,
        },
        "mieshi" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true } if *player == owner
        ),
        "xmojian" => matches!(
            event,
            EventData::TurnOvered { player, face_down }
                if *player == owner
                    && !*face_down
                    && !sha_target_options(st, ctx.catalog, owner).is_empty()
        ),
        "xshixin" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner) && d.target != owner && st.player(d.target).alive
        ),
        "guozai2" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Play, begin: false } if *player == owner
        ),
        "guozaix2" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Play, begin: false } if *player == owner
        ),
        "hanshuang" | "oldhanshuang" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && st.player(d.target).alive
                    && !st.player(d.target).face_down
                    && d.card.is_some_and(|c| {
                        !super::is_b6b_red_suit(ctx.catalog.cards[c as usize].suit)
                    })
        ),
        "fengnu" => match event {
            EventData::CardUsed(u)
                if u.player == owner
                    && st.turn.current == owner
                    && !u.cancelled
                    && hs_b6b_cnt(st, owner, "fengnu") < 3 =>
            {
                let key = super::b6b_card_name_key(st, ctx.catalog, u, 0);
                b6b_flag(st, owner, &format!("hs_b6b_name_used:{key}")) > 0
            }
            _ => false,
        },
        "shengdun" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Prepare, begin: true }
                if *player == owner && st.player(owner).hujia == 0
        ),
        _ => false,
    }
}

pub(super) fn activate_c(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<bool> {
    let st = ctx.state;
    match skill.name.as_str() {
        "enze" | "oldenze" => {
            let my_h = st.player(owner).hand.len();
            let used = hs_b6b_cnt(st, owner, &skill.name) > 0;
            Some(!used && alive_others(st, owner).into_iter().any(|p| st.player(p).hand.len() != my_h))
        }
        "guozai" => Some(hs_b6b_cnt(st, owner, "guozai") == 0 && st.player(owner).hand.len() < 4),
        "guozaix" => Some(hs_b6b_cnt(st, owner, "guozaix") < 2 && st.player(owner).hand.len() < 4),
        "huanwu" => Some(
            hs_b6b_cnt(st, owner, "huanwu") == 0
                && st.alive_players().any(|p| flag(st, p, "hs_huanwu_marked") == 0),
        ),
        _ => None,
    }
}

fn step_enze(m: &mut HostMachine, ctx: &mut Ctx<'_>, capped: bool) -> Flow {
    let sk = if capped { "enze" } else { "oldenze" };
    let ask_id = if capped {
        ASK_B6B_ENZE_TARGET
    } else {
        ASK_B6B_OLDENZE_TARGET
    };
    match m.stage {
        HostStage::Act => wait_b6b(m, ctx, HostStage::TargetChoice, ask_id),
        HostStage::TargetChoice => {
            clear_b6b_ask(m, ctx, HostStage::TargetChoice);
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(t).alive || t == m.owner {
                return Flow::Done;
            }
            let diff = ctx.core.player(m.owner).hand.len() as isize
                - ctx.core.player(t).hand.len() as isize;
            if diff == 0 {
                return Flow::Done;
            }
            hs_b6b_inc(ctx.core, m.owner, sk);
            if diff > 0 {
                let n = if capped { diff.min(3) } else { diff } as u32;
                ctx.start_draw(t, n);
                return Flow::Done;
            }
            let want = if capped { (-diff).min(3) } else { -diff } as usize;
            let th = ctx.core.player(t).hand.len();
            if th <= want {
                let all_h = ctx.core.player(t).hand.clone();
                for &c in &all_h {
                    ctx.discard_from_hand(t, c);
                }
                return Flow::Done;
            }
            m.target = Some(t);
            m.card = Some(want as u32);
            wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_ENZE_DISCARD)
        }
        HostStage::SelectCard => {
            clear_b6b_ask(m, ctx, HostStage::SelectCard);
            if let Some(t) = m.target {
                let chosen: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                for &c in &chosen {
                    if ctx.core.player(t).hand.contains(&c) {
                        ctx.discard_from_hand(t, c);
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_guozai2(m: &mut HostMachine, ctx: &mut Ctx<'_>, sk: &str) -> Flow {
    let flag_key = if sk == "guozai2" {
        "hs_guozai2_cnt"
    } else {
        "hs_guozaix2_cnt"
    };
    match m.stage {
        HostStage::Act => {
            let n = b6b_flag(ctx.core, m.owner, flag_key) as usize;
            clear_b6b_flag(ctx.core, m.owner, flag_key);
            let pl = ctx.core.player(m.owner);
            let he: Vec<CardId> = pl.hand.iter().chain(pl.equips.iter()).copied().collect();
            if n == 0 || he.is_empty() {
                remove_skill(ctx, m.owner, sk);
                return Flow::Done;
            }
            if he.len() <= n {
                discard_cards_from_zone(ctx, m.owner, &he);
                ctx.log(LogEvent::CardDiscarded {
                    player: m.owner,
                    cards: he,
                });
                remove_skill(ctx, m.owner, sk);
                return Flow::Done;
            }
            m.card = Some(n as u32);
            wait_b6b(m, ctx, HostStage::SelectCard, ASK_B6B_GUOZAI2_DISCARD)
        }
        HostStage::SelectCard => {
            clear_b6b_ask(m, ctx, HostStage::SelectCard);
            let chosen: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if !chosen.is_empty() {
                discard_cards_from_zone(ctx, m.owner, &chosen);
                ctx.log(LogEvent::CardDiscarded {
                    player: m.owner,
                    cards: chosen,
                });
            }
            remove_skill(ctx, m.owner, sk);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_c(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "lingzhou" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::TargetChoice, ASK_B6B_LINGZHOU_TARGET),
            HostStage::TargetChoice => {
                clear_b6b_ask(m, ctx, HostStage::TargetChoice);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive {
                    return Flow::Done;
                }
                m.target = Some(t);
                wait_b6b(m, ctx, HostStage::SelectNature, ASK_B6B_LINGZHOU_CHOICE)
            }
            HostStage::SelectNature => {
                clear_b6b_ask(m, ctx, HostStage::SelectNature);
                let Some(t) = m.target else {
                    return Flow::Done;
                };
                match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(1))
                        if ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32 =>
                    {
                        ctx.recover(t, 1, None);
                    }
                    _ => ctx.start_draw(t, 1),
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "mieshi" => match m.stage {
            HostStage::Act => {
                ctx.emit(EventData::LoseHp {
                    player: m.owner,
                    amount: 1,
                });
                m.stage = HostStage::DrawSettle;
                Flow::Yield
            }
            HostStage::DrawSettle => {
                let pool = alive_others(ctx.core, m.owner);
                if let Some(t) = rng_pick(ctx, &pool) {
                    ctx.emit(EventData::Damage(make_damage(
                        Some(m.owner),
                        t,
                        1,
                        DamageNature::Fire,
                        None,
                    )));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xmojian" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::TargetChoice, ASK_B6B_XMOJIAN_TARGET),
            HostStage::TargetChoice => {
                clear_b6b_ask(m, ctx, HostStage::TargetChoice);
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    if ctx.core.player(t).alive && t != m.owner {
                        emit_free_sha_no_count(ctx, m.owner, t);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xshixin" => match m.stage {
            HostStage::Act => {
                let Some(t) = m.target else {
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive || t == m.owner {
                    return Flow::Done;
                }
                ctx.emit(EventData::LoseHp {
                    player: t,
                    amount: 1,
                });
                m.stage = HostStage::DrawSettle;
                Flow::Yield
            }
            HostStage::DrawSettle => {
                if ctx.core.player(m.owner).alive {
                    ctx.emit(EventData::LoseHp {
                        player: m.owner,
                        amount: 1,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "enze" => step_enze(m, ctx, true),
        "oldenze" => step_enze(m, ctx, false),
        "guozai" => {
            let cur = ctx.core.player(m.owner).hand.len();
            if cur < 4 && hs_b6b_cnt(ctx.core, m.owner, "guozai") == 0 {
                let n = (4 - cur) as u32;
                hs_b6b_inc(ctx.core, m.owner, "guozai");
                ctx.start_draw(m.owner, n);
                grant_skill(ctx, m.owner, "guozai2", false);
                let prev = b6b_flag(ctx.core, m.owner, "hs_guozai2_cnt");
                set_b6b_flag(ctx.core, m.owner, "hs_guozai2_cnt", prev + n);
            }
            Flow::Done
        }
        "guozaix" => {
            let cur = ctx.core.player(m.owner).hand.len();
            if cur < 4 && hs_b6b_cnt(ctx.core, m.owner, "guozaix") < 2 {
                let n = (4 - cur) as u32;
                hs_b6b_inc(ctx.core, m.owner, "guozaix");
                ctx.start_draw(m.owner, n);
                grant_skill(ctx, m.owner, "guozaix2", false);
                let prev = b6b_flag(ctx.core, m.owner, "hs_guozaix2_cnt");
                set_b6b_flag(ctx.core, m.owner, "hs_guozaix2_cnt", prev + n);
            }
            Flow::Done
        }
        "guozai2" => step_guozai2(m, ctx, "guozai2"),
        "guozaix2" => step_guozai2(m, ctx, "guozaix2"),
        "hanshuang" | "oldhanshuang" => match m.stage {
            HostStage::Act => {
                let lose_self = m.def_key == "hanshuang";
                if let Ok(d) = ctx.current_damage() {
                    let tgt = d.target;
                    if ctx.core.player(tgt).alive && !ctx.core.player(tgt).face_down {
                        ctx.turn_over(tgt);
                        if lose_self {
                            m.stage = HostStage::DrawSettle;
                            return Flow::Yield;
                        }
                    }
                }
                Flow::Done
            }
            HostStage::DrawSettle => {
                if ctx.core.player(m.owner).alive {
                    ctx.emit(EventData::LoseHp {
                        player: m.owner,
                        amount: 1,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "huanwu" => match m.stage {
            HostStage::Act => wait_b6b(m, ctx, HostStage::TargetChoice, ASK_B6B_HUANWU_TARGET),
            HostStage::TargetChoice => {
                clear_b6b_ask(m, ctx, HostStage::TargetChoice);
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    if ctx.core.player(t).alive && flag(ctx.core, t, "hs_huanwu_marked") == 0 {
                        hs_b6b_inc(ctx.core, m.owner, "huanwu");
                        set_flag(ctx.core, t, "hs_huanwu_marked", 1);
                        change_max_hp(ctx, t, 1);
                        ctx.recover(t, 1, None);
                        ctx.start_draw(t, 2);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "fengnu" => {
            if hs_b6b_cnt(ctx.core, m.owner, "fengnu") < 3 {
                hs_b6b_inc(ctx.core, m.owner, "fengnu");
                ctx.start_draw(m.owner, 1);
            }
            Flow::Done
        }
        "shengdun" => {
            if ctx.core.player(m.owner).hujia == 0 {
                ctx.change_hujia(m.owner, 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
