//! Batch U05-6a (`shahai-hearth` Part 1): Batch D 20 技能 (`midian` .. `wzhanyi`)。

use super::u05_hearth::{
    alive_others, discard_card_from_zone, emit_hearth_virtual, equip_card_to_player, flag,
    gain_card_to_hand, grant_skill, has_any_skill, in_attack_range_hearth, is_max_handcard, meta,
    refund_once_used, remove_skill, rng_pick, set_flag, spawn_card_by_base, spawn_card_by_kind,
    take_he_card_as,
};
use super::u05_hearth_a::use_gained_card;
use super::u05_hearth_asks::{
    hearth_card_needs_target, set_active_ask, wait_hs, ASK_HS_AOSHU_CARD, ASK_HS_BZHUIJI_CONFIRM,
    ASK_HS_LIANJIN_CARD, ASK_HS_MIDIAN_CARD, ASK_HS_PEIYU_TARGET,
    ASK_HS_WZHANYI_TARGET, ASK_HS_WZHANYI_USE, ASK_HS_XINGLUO_DISCARD, ASK_HS_XINGLUO_PICK,
    ASK_HS_XINGLUO_TARGETS, ASK_HS_YINGXI_CARD, ASK_HS_YINGXI_TARGET, ASK_HS_YUELU_CARD,
    ASK_HS_YUSHOU_CARD, ASK_HS_YUSHOU_HUOFU_CARD, ASK_HS_YUSHOU_HUOFU_TARGET,
};
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{EventData, EventKind, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

const YUSHOU_POOL: [&str; 3] = ["yushou_misha", "yushou_huofu", "yushou_leiouke"];

pub(super) fn trigger_meta_d(name: &str) -> Option<TriggerMeta> {
    Some(match name {
        "xingluo" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "yushou_lose" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "yushou_misha" => meta(EventKind::Damage, Timing::Normal, 0, false, Vec::new()),
        "yushou_leiouke" => meta(EventKind::DamageBefore, Timing::Normal, 0, false, Vec::new()),
        "qingzun" => meta(EventKind::PhaseBegin, Timing::Normal, 0, true, Vec::new()),
        "ayuling" => meta(EventKind::Damage, Timing::Normal, 0, true, Vec::new()),
        "lianjin" => meta(EventKind::CardsDiscarded, Timing::Normal, 0, false, Vec::new()),
        "yingxi" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "hsguimou" => meta(EventKind::Damage, Timing::Normal, 0, true, Vec::new()),
        "peiyu" => meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "peiyu2" => meta(EventKind::Damage, Timing::Normal, 0, false, Vec::new()),
        "wzhanyi" => meta(EventKind::PhaseBegin, Timing::First, 0, true, Vec::new()),
        _ => return None,
    })
}

pub(super) fn limits_d(name: &str) -> SkillLimits {
    match name {
        "midian" | "yushou_leiouke" | "aoshu" | "lianjin" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(super) fn has_active_d(name: &str) -> bool {
    matches!(name, "midian" | "yushou" | "yushou_huofu" | "aoshu" | "lianjin")
}

pub(super) fn activate_d(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<bool> {
    let pl = ctx.state.player(owner);
    Some(match skill.name.as_str() {
        "midian" => pl.hand.iter().any(|&c| {
            let k = ctx.catalog.card_kind(c);
            !k.is_basic_kind() && k.equip_slot().is_none()
        }),
        "yushou" => {
            (!pl.hand.is_empty() || !pl.equips.is_empty())
                && YUSHOU_POOL
                    .iter()
                    .any(|&s| !has_any_skill(ctx.state, ctx.catalog, owner, s))
        }
        "yushou_huofu" => {
            !alive_others(ctx.state, owner).is_empty()
                && pl.hand.iter().chain(pl.equips.iter()).any(|&c| {
                    matches!(
                        ctx.catalog.effective_card_suit(ctx.state, owner, c),
                        Suit::Spade | Suit::Club
                    )
                })
        }
        "aoshu" => pl
            .hand
            .iter()
            .chain(pl.equips.iter())
            .any(|&c| ctx.catalog.effective_card_suit(ctx.state, owner, c) == Suit::Spade),
        "lianjin" => pl.hand.iter().chain(pl.equips.iter()).any(|&c| {
            super::u05_hearth::exact_base_name(ctx.catalog.cards[c as usize].name) != "hsyaoshui"
        }),
        _ => return None,
    })
}

pub(super) fn filter_d(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.name.as_str() {
        "xingluo" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::Prepare,
                begin: true,
            } if *player == owner && !is_max_handcard(ctx.state, owner)
        ),
        "yushou_lose" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::Prepare,
                begin: true,
            } if *player == owner
                && YUSHOU_POOL
                    .iter()
                    .any(|&s| has_any_skill(ctx.state, ctx.catalog, owner, s))
        ),
        "yushou_misha" => matches!(
            event,
            EventData::Damage(d) if d.target == owner && !d.prevented && d.amount >= 1
        ),
        "yushou_leiouke" => matches!(
            event,
            EventData::Damage(d) if d.source == Some(owner) && !d.prevented && d.amount > 0
        ),
        "qingzun" => match event {
            EventData::Phase {
                player,
                phase: crate::event::Phase::Prepare,
                begin: true,
            } => *player == owner && flag(ctx.state, owner, "hs_qingzun_count") >= 2,
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } => *player == owner && flag(ctx.state, owner, "hs_qingzun_count") >= 6,
            _ => false,
        },
        "ayuling" => matches!(
            event,
            EventData::Damage(d) if d.target == owner && !d.prevented && d.amount > 0
        ),
        "lianjin" => matches!(
            event,
            EventData::CardsDiscarded { player, cards }
                if *player == owner
                    && cards.iter().any(|&c| {
                        super::u05_hearth::exact_base_name(ctx.catalog.cards[c as usize].name)
                            == "hsyaoshui"
                    })
        ),
        "yingxi" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::End,
                begin: true,
            } if *player == owner
                && ctx.state.player(owner).turn_damage == 0
                && !alive_others(ctx.state, owner)
                    .into_iter()
                    .filter(|&p| in_attack_range_hearth(ctx.state, ctx.catalog, owner, p))
                    .collect::<Vec<_>>()
                    .is_empty()
                && ctx.state.player(owner).hand.iter().chain(ctx.state.player(owner).equips.iter()).any(|&c| {
                    matches!(
                        ctx.catalog.effective_card_suit(ctx.state, owner, c),
                        Suit::Spade | Suit::Club
                    )
                })
        ),
        "hsguimou" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.source.is_some_and(|s| {
                        s != owner && ctx.state.player(s).alive && !ctx.state.player(s).hand.is_empty()
                    })
        ),
        "peiyu" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::Prepare,
                begin: true,
            } if *player == owner
                && ctx.state.alive_players().any(|p| {
                    !(1..=8).any(|i| has_any_skill(ctx.state, ctx.catalog, p, &format!("tuteng{i}")))
                })
        ),
        "peiyu2" => matches!(
            event,
            EventData::Damage(d) if d.target == owner && !d.prevented && d.amount > 0
        ),
        "wzhanyi" => matches!(
            event,
            EventData::Phase {
                player,
                phase: crate::event::Phase::Play,
                begin: true,
            } if *player == owner
        ),
        _ => false,
    }
}

pub(super) fn step_d(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "midian" => step_midian(m, ctx),
        "xingluo" => step_xingluo(m, ctx),
        "yuelu" => step_yuelu(m, ctx),
        "yushou" => step_yushou(m, ctx),
        "yushou_lose" => {
            let owned: Vec<&str> = YUSHOU_POOL
                .iter()
                .copied()
                .filter(|&s| has_any_skill(ctx.core, ctx.catalog, m.owner, s))
                .collect();
            if let Some(pick) = rng_pick(ctx, &owned) {
                remove_skill(ctx, m.owner, pick);
            }
            Flow::Done
        }
        "yushou_misha" => {
            ctx.change_hujia(m.owner, 1);
            Flow::Done
        }
        "yushou_huofu" => step_yushou_huofu(m, ctx),
        "yushou_leiouke" => {
            for f in ctx.res.stack.iter_mut().rev() {
                if let EventData::Damage(d) = &mut f.data {
                    if d.source == Some(m.owner) && !d.prevented {
                        d.amount += 1;
                        break;
                    }
                }
            }
            Flow::Done
        }
        "qingzun" => {
            ctx.start_draw(m.owner, 1);
            Flow::Done
        }
        "ayuling" => {
            let pool = [
                "hsqingyu_feibiao",
                "hsqingyu_zhanfang",
                "hsqingyu_hufu",
                "hsqingyu_shandian",
                "hsqingyu_zhao",
            ];
            if let Some(pick) = rng_pick(ctx, &pool) {
                if let Some(c) = spawn_card_by_base(ctx, pick) {
                    gain_card_to_hand(ctx, m.owner, c);
                }
            }
            Flow::Done
        }
        "aoshu" => step_aoshu(m, ctx),
        "bzhuiji" => step_bzhuiji(m, ctx),
        "lianjin" => step_lianjin(m, ctx),
        "yingxi" => step_yingxi(m, ctx),
        "hsguimou" => step_hsguimou(m, ctx),
        "peiyu" => step_peiyu(m, ctx),
        "peiyu2" => {
            remove_skill(ctx, m.owner, "peiyu2");
            for i in 1..=8 {
                remove_skill(ctx, m.owner, &format!("tuteng{i}"));
            }
            Flow::Done
        }
        "wzhanyi" => step_wzhanyi(m, ctx),
        _ => Flow::Done,
    }
}

fn step_midian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_MIDIAN_CARD),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, m.owner, "midian");
                return Flow::Done;
            };
            discard_card_from_zone(ctx, m.owner, picked);
            ctx.log(LogEvent::CardDiscarded {
                player: m.owner,
                cards: vec![picked],
            });
            let tricks = [
                CardKind::Duel,
                CardKind::Dismantle,
                CardKind::Snatch,
                CardKind::Wuzhong,
                CardKind::Taoyuan,
                CardKind::Wugu,
                CardKind::SavageAttack,
                CardKind::ArcheryAttack,
                CardKind::Chain,
                CardKind::Jiedao,
                CardKind::Le,
                CardKind::Bing,
                CardKind::ShanDian,
                CardKind::Nullify,
            ];
            for _ in 0..3 {
                if let Some(k) = rng_pick(ctx, &tricks) {
                    if let Some(c) = spawn_card_by_kind(ctx, k) {
                        gain_card_to_hand(ctx, m.owner, c);
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_xingluo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_XINGLUO_TARGETS),
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            let chosen: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .collect();
            if chosen.is_empty() {
                return Flow::Done;
            }
            m.targets = chosen;
            set_flag(ctx.core, m.owner, "hs_xingluo_idx", 0);
            m.cards.clear();
            let first = m.targets[0];
            m.target = Some(first);
            wait_hs(m, ctx, HostStage::ComparePlayerPick, ASK_HS_XINGLUO_DISCARD)
        }
        HostStage::ComparePlayerPick => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::ComparePlayerPick, 0);
            let idx = flag(ctx.core, m.owner, "hs_xingluo_idx") as usize;
            if let (Some(&t), Some(ChoiceOption::Card(dc))) =
                (m.targets.get(idx), ctx.answer().first().copied())
            {
                discard_card_from_zone(ctx, t, dc);
                ctx.log(LogEvent::CardDiscarded {
                    player: t,
                    cards: vec![dc],
                });
                m.cards.push(dc);
            }
            let next_idx = idx + 1;
            if next_idx < m.targets.len() {
                set_flag(ctx.core, m.owner, "hs_xingluo_idx", next_idx as u32);
                m.target = Some(m.targets[next_idx]);
                return wait_hs(m, ctx, HostStage::ComparePlayerPick, ASK_HS_XINGLUO_DISCARD);
            }
            let valid: Vec<CardId> = m
                .cards
                .iter()
                .copied()
                .filter(|&c| ctx.core.discard.contains(&c))
                .collect();
            m.cards = valid;
            if m.cards.is_empty() {
                return Flow::Done;
            }
            wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_XINGLUO_PICK)
        }
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            if let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() {
                if let Some(pos) = ctx.core.discard.iter().rposition(|&c| c == picked) {
                    ctx.core.discard.remove(pos);
                    gain_card_to_hand(ctx, m.owner, picked);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_yuelu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_YUELU_CARD),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let dying = m.target.unwrap_or(m.owner);
            discard_card_from_zone(ctx, m.owner, picked);
            ctx.log(LogEvent::CardDiscarded {
                player: m.owner,
                cards: vec![picked],
            });
            ctx.recover(dying, 1, None);
            ctx.change_hujia(dying, 1);
            set_flag(ctx.core, m.owner, "hs_yuelu_saved", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_yushou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_YUSHOU_CARD),
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
            let missing: Vec<&str> = YUSHOU_POOL
                .iter()
                .copied()
                .filter(|&s| !has_any_skill(ctx.core, ctx.catalog, m.owner, s))
                .collect();
            if let Some(sk) = rng_pick(ctx, &missing) {
                grant_skill(ctx, m.owner, sk, false);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_yushou_huofu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_YUSHOU_HUOFU_CARD),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.cards = vec![picked];
            wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_YUSHOU_HUOFU_TARGET)
        }
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            let (Some(&card), Some(ChoiceOption::Player(t))) =
                (m.cards.first(), ctx.answer().first().copied())
            else {
                return Flow::Done;
            };
            take_he_card_as(ctx, m.owner, card, vec![t], CardKind::Duel);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_aoshu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_AOSHU_CARD),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, m.owner, "aoshu");
                return Flow::Done;
            };
            take_he_card_as(ctx, m.owner, picked, vec![m.owner], CardKind::Wuzhong);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_bzhuiji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::CompareTargetPick, ASK_HS_BZHUIJI_CONFIRM),
        HostStage::CompareTargetPick => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::CompareTargetPick, 0);
            if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                return Flow::Done;
            }
            ctx.start_draw(m.owner, 2);
            if let Some(killer) = m.target {
                if killer != m.owner && ctx.core.player(killer).alive {
                    emit_hearth_virtual(ctx, m.owner, CardKind::Duel, vec![killer]);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_lianjin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let in_discard = ctx.res.stack.last().and_then(|f| match &f.data {
        EventData::CardsDiscarded { player, cards } if *player == m.owner => Some(cards.clone()),
        _ => None,
    });
    if let Some(cards) = in_discard {
        for c in cards {
            if super::u05_hearth::exact_base_name(ctx.catalog.cards[c as usize].name) == "hsyaoshui"
            {
                let effs: Vec<u32> = (0..3)
                    .filter_map(|i| {
                        let v = flag(ctx.core, 0, &format!("hs_yaoshui_eff_{i}:{c}"));
                        (v > 0).then_some(v)
                    })
                    .collect();
                if let Some(code) = rng_pick(ctx, &effs) {
                    if let Ok(kind) = crate::archive::kind_from_code(code as u16) {
                        if let Some(g) = spawn_card_by_kind(ctx, kind) {
                            gain_card_to_hand(ctx, m.owner, g);
                        }
                    }
                }
            }
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_LIANJIN_CARD),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                refund_once_used(ctx, m.owner, "lianjin");
                return Flow::Done;
            };
            discard_card_from_zone(ctx, m.owner, picked);
            ctx.log(LogEvent::CardDiscarded {
                player: m.owner,
                cards: vec![picked],
            });
            if let Some(potion) = spawn_card_by_base(ctx, "hsyaoshui") {
                let mut pool = vec![
                    CardKind::Duel,
                    CardKind::Dismantle,
                    CardKind::Snatch,
                    CardKind::Wuzhong,
                    CardKind::SavageAttack,
                    CardKind::ArcheryAttack,
                    CardKind::Taoyuan,
                    CardKind::Wugu,
                    CardKind::Sha,
                    CardKind::Tao,
                ];
                let want_equip = ctx.core.rng.gen_range(2) == 0;
                let mut chosen = Vec::new();
                while chosen.len() < 2 && !pool.is_empty() {
                    let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                    chosen.push(pool.swap_remove(idx));
                }
                if want_equip {
                    let eq_pool = [
                        CardKind::Weapon,
                        CardKind::Armor,
                        CardKind::HorsePlus,
                        CardKind::HorseMinus,
                    ];
                    chosen.push(rng_pick(ctx, &eq_pool).unwrap_or(CardKind::Weapon));
                } else {
                    let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                    chosen.push(pool.swap_remove(idx));
                }
                for (i, k) in chosen.into_iter().enumerate() {
                    let code = crate::archive::kind_code(k) as u32;
                    set_flag(ctx.core, 0, &format!("hs_yaoshui_eff_{i}:{potion}"), code);
                }
                gain_card_to_hand(ctx, m.owner, potion);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_yingxi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_YINGXI_CARD),
        HostStage::SelectCard => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.cards = vec![picked];
            wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_YINGXI_TARGET)
        }
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            let (Some(&card), Some(ChoiceOption::Player(t))) =
                (m.cards.first(), ctx.answer().first().copied())
            else {
                return Flow::Done;
            };
            set_flag(ctx.core, m.owner, "hs_yingxi_sha_active", 1);
            take_he_card_as(ctx, m.owner, card, vec![t], CardKind::Sha);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_hsguimou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let src_opt = ctx.res.stack.iter().rev().find_map(|f| match &f.data {
        EventData::Damage(d) if d.target == m.owner => d.source,
        _ => None,
    });
    let Some(src) = src_opt else {
        return Flow::Done;
    };
    while src != m.owner && ctx.core.player(src).alive && !ctx.core.player(src).hand.is_empty() {
        let hand = ctx.core.player(src).hand.clone();
        let Some(c) = rng_pick(ctx, &hand) else {
            break;
        };
        let is_black = matches!(
            ctx.catalog.effective_card_suit(ctx.core, src, c),
            Suit::Spade | Suit::Club
        );
        ctx.give_hand_card(src, m.owner, c);
        if !is_black {
            break;
        }
    }
    Flow::Done
}

fn step_peiyu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_PEIYU_TARGET),
        HostStage::TargetChoice => {
            set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let idx = (ctx.core.rng.gen_range(8) + 1) as u32;
            grant_skill(ctx, t, "peiyu2", false);
            grant_skill(ctx, t, &format!("tuteng{idx}"), false);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_wzhanyi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    loop {
        match m.stage {
            HostStage::Act => {
                ctx.core.turn.skip_play_phase = true;
                for _ in 0..3 {
                    let Some(c) = ctx.core.deck.pop() else {
                        break;
                    };
                    if ctx.catalog.card_kind(c).is_equip() {
                        equip_card_to_player(ctx, m.owner, c);
                    } else {
                        gain_card_to_hand(ctx, m.owner, c);
                    }
                }
                m.stage = HostStage::DrawSettle;
            }
            HostStage::DrawSettle => {
                let has_sha = ctx
                    .core
                    .player(m.owner)
                    .hand
                    .iter()
                    .any(|&c| ctx.catalog.card_kind(c).is_sha_like());
                let has_target = !alive_others(ctx.core, m.owner)
                    .into_iter()
                    .filter(|&p| in_attack_range_hearth(ctx.core, ctx.catalog, m.owner, p))
                    .collect::<Vec<_>>()
                    .is_empty();
                if !has_sha || !has_target {
                    return Flow::Done;
                }
                return wait_hs(m, ctx, HostStage::SelectCard, ASK_HS_WZHANYI_USE);
            }
            HostStage::SelectCard => {
                set_active_ask(ctx.core, m.behavior, m.owner, HostStage::SelectCard, 0);
                let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.cards = vec![picked];
                return wait_hs(m, ctx, HostStage::TargetChoice, ASK_HS_WZHANYI_TARGET);
            }
            HostStage::TargetChoice => {
                set_active_ask(ctx.core, m.behavior, m.owner, HostStage::TargetChoice, 0);
                let (Some(&card), Some(ChoiceOption::Player(t))) =
                    (m.cards.first(), ctx.answer().first().copied())
                else {
                    return Flow::Done;
                };
                if hearth_card_needs_target(ctx.core, ctx.catalog, m.owner, card) {
                    use_gained_card(ctx, m.owner, card, vec![t]);
                } else {
                    use_gained_card(ctx, m.owner, card, Vec::new());
                }
                m.stage = HostStage::DrawSettle;
                return Flow::Yield;
            }
            _ => return Flow::Done,
        }
    }
}
