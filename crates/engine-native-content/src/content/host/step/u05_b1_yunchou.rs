//! Batch U05-1 运筹帷幄武将全量技能 (`character/yunchou/skill.js`) 与精械实现。

use super::super::*;
use super::l27_batch_a::{alive_all, alive_others, discard_cards, hand_and_equips, move_cards_to_hand, set_flag};
use super::l28_batch_b::jingxie_candidates;
use super::u08_batch_b::{is_black_card, is_red_card};
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, EventKind, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;

pub(in crate::content::host) fn trigger_meta_u05_yunchou_char(name: &str) -> Option<TriggerMeta> {
    match name {
        "ycndiyjianxiong" => Some(TriggerMeta {
            event: EventKind::Death,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: true,
            optional: false,
            extra_events: Vec::new(),
        }),
        "xinfu_jingxie" => Some(TriggerMeta {
            event: EventKind::Dying,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: true,
            optional: false,
            extra_events: Vec::new(),
        }),
        // Classic `beige` (`character/shenhua/skill.js:4814`): cancelable discard cost before judge, no `beige_gains`.
        "ycn_beige" => Some(TriggerMeta {
            event: EventKind::Damage,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: true,
            optional: false,
            extra_events: Vec::new(),
        }),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_u05_yunchou_char(name: &str) -> SkillLimits {
    if matches!(name, "ycnxinfu_qiaosi" | "xinfu_qiaosi") {
        SkillLimits {
            once_per_turn: true,
            ..Default::default()
        }
    } else {
        SkillLimits::default()
    }
}

pub(in crate::content::host) fn filter_u05_yunchou_char(
    m: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if ctx.catalog.is_tieji_sealed(ctx.state, owner) || !ctx.state.player(owner).alive {
        return false;
    }
    match m.name.as_str() {
        "ycndiyjianxiong" => match event {
            EventData::Death { player, .. } => {
                ctx.state.turn.current == owner
                    && *player != owner
                    && match &ctx.state.mode {
                        crate::state::GameMode::Identity { roles } => {
                            roles.get(*player as usize).copied() != Some(crate::state::Role::Lord)
                        }
                        crate::state::GameMode::Kingdoms => ctx.state.is_friend_of(ctx.catalog, *player, owner),
                        _ => false,
                    }
            }
            _ => false,
        },
        "xinfu_jingxie" => match event {
            EventData::Dying { player, .. } => {
                *player == owner && !jingxie_candidates(ctx.state, ctx.catalog, owner, true).is_empty()
            }
            _ => false,
        },
        "ycn_beige" => matches!(
            event,
            EventData::Damage(d)
                if !d.prevented
                    && d.amount > 0
                    && d.source.is_some()
                    && d.card
                        .is_some_and(|c| ctx.catalog.resolved_card_kind(ctx.state, c).is_sha_like())
                    && ctx.state.player(d.target).alive
                    && (!ctx.state.player(owner).hand.is_empty()
                        || !ctx.state.player(owner).equips.is_empty())
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn activate_u05_yunchou_char(
    m: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<HostMachine> {
    if ctx.catalog.is_tieji_sealed(ctx.state, owner) || !ctx.state.player(owner).alive {
        return None;
    }
    let ok = match m.name.as_str() {
        "xinfu_jingxie" => !jingxie_candidates(ctx.state, ctx.catalog, owner, false).is_empty(),
        "ycnxinfu_qiaosi" | "xinfu_qiaosi" => !ctx.state.deck.is_empty() || !ctx.state.discard.is_empty(),
        _ => false,
    };
    ok.then(|| HostMachine {
        def_key: m.name.clone(),
        behavior: m.behavior,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    })
}

pub(in crate::content::host) fn filter_yunchou_existing(
    behavior: HostBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if ctx.catalog.is_tieji_sealed(ctx.state, owner) || !ctx.state.player(owner).alive {
        return false;
    }
    match behavior {
        HostBehavior::YunchouDiykuanggu => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && (ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
                        || ctx.state.computed_distance(ctx.catalog, d.target, owner) > 1)
        ),
        HostBehavior::YunchouDiychanyuan => matches!(
            event,
            EventData::Death { player, killer: Some(_), .. } if *player == owner
        ),
        HostBehavior::YunchouXicai => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.card.is_some_and(|c| ctx.state.table.contains(&c) || ctx.state.discard.contains(&c))
        ),
        HostBehavior::YunchouHonglian => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.source.is_some_and(|src| {
                        src != owner
                            && ctx.state.player(src).alive
                            && ctx
                                .state
                                .player(src)
                                .hand
                                .iter()
                                .chain(ctx.state.player(src).equips.iter())
                                .any(|&c| is_red_card(ctx.state, ctx.catalog, src, c))
                    })
        ),
        HostBehavior::YunchouZonghuo => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.nature != DamageNature::Fire
                    && !ctx.state.player(owner).hand.is_empty()
        ),
        HostBehavior::YunchouShaoying => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.nature == DamageNature::Fire
                    && alive_others(ctx.state, d.target).into_iter().any(|cand| {
                        ctx.state.computed_distance(ctx.catalog, d.target, cand) <= 1
                    })
        ),
        HostBehavior::YunchouLuweiyan2 => match event {
            EventData::CardUsed(u) if !u.cancelled && u.player == owner => {
                crate::content::host::support::u_matches_lw2(event, owner, ctx)
                    && u.targets.iter().any(|&t| t != owner && ctx.state.player(t).alive)
            }
            _ => false,
        },
        HostBehavior::YunchouJiaoxia => matches!(
            event,
            EventData::CardUsed(u)
                if !u.cancelled
                    && u.targets.contains(&owner)
                    && u.card.is_some_and(|c| is_red_card(ctx.state, ctx.catalog, u.player, c))
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn ask_u05_yunchou_char(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let key = if behavior == HostBehavior::U05YunchouCharBatch {
        if card.is_none() && targets.len() >= 2 {
            "ycn_beige"
        } else if card.is_none() {
            "xinfu_jingxie"
        } else {
            "ycnxinfu_qiaosi"
        }
    } else {
        ""
    };
    match (behavior, key, stage) {
        (HostBehavior::U05YunchouCharBatch, "xinfu_jingxie", HostStage::SelectCard) => {
            let opts: Vec<ChoiceOption> = jingxie_candidates(core, catalog, owner, target == Some(owner))
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| ask(owner, "u05_xinfu_jingxie_card", ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        (HostBehavior::U05YunchouCharBatch, "ycnxinfu_qiaosi" | "xinfu_qiaosi", HostStage::SelectNature) => {
            let mut opts = Vec::new();
            if !alive_others(core, owner).is_empty() {
                opts.push(ChoiceOption::Tag(0));
            }
            opts.push(ChoiceOption::Tag(1));
            Some(ask(owner, "u05_qiaosi_mode", ChoiceKind::PickTag, opts, 1, 1, false))
        }
        (HostBehavior::U05YunchouCharBatch, "ycnxinfu_qiaosi" | "xinfu_qiaosi", HostStage::SelectCard) => {
            let he = hand_and_equips(core, owner);
            let n = card.unwrap_or(1).min(he.len() as u32).max(1) as usize;
            let opts = he.into_iter().map(ChoiceOption::Card).collect();
            Some(ask(owner, "u05_qiaosi_cards", ChoiceKind::SelectCards, opts, n, n, false))
        }
        (HostBehavior::U05YunchouCharBatch, "ycnxinfu_qiaosi" | "xinfu_qiaosi", HostStage::TargetChoice) => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
            (!opts.is_empty()).then(|| ask(owner, "u05_qiaosi_target", ChoiceKind::SelectPlayers, opts, 1, 1, false))
        }
        (HostBehavior::U05YunchouCharBatch, "ycnxinfu_qiaosi" | "xinfu_qiaosi", HostStage::CardsAsk) => {
            let he = hand_and_equips(core, owner);
            let n = card.unwrap_or(1).min(he.len() as u32).max(1) as usize;
            let opts = he.into_iter().map(ChoiceOption::Card).collect();
            Some(ask(owner, "u05_qiaosi_discard", ChoiceKind::SelectCards, opts, n, n, false))
        }
        (HostBehavior::YunchouJieyan, _, HostStage::SelectCard) => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| is_red_card(core, catalog, owner, c))
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| ask(owner, "u05_jieyan_red", ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        (HostBehavior::YunchouDiyguhuo, _, HostStage::CardsAsk) => {
            let pl = core.player(owner);
            let all: Vec<CardId> = pl
                .hand
                .iter()
                .chain(pl.equips.iter())
                .chain(pl.judge.iter())
                .copied()
                .collect();
            let n = all.len().min(2);
            (n > 0).then(|| {
                ask(
                    owner,
                    "u05_diyguhuo_discard",
                    ChoiceKind::SelectCards,
                    all.into_iter().map(ChoiceOption::Card).collect(),
                    n,
                    n,
                    false,
                )
            })
        }
        (HostBehavior::YunchouZonghuo, _, HostStage::SelectCard) => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| ask(owner, "u05_zonghuo_discard", ChoiceKind::SelectCards, opts, 0, 1, true))
        }
        (HostBehavior::YunchouShaoying, _, HostStage::TargetChoice) => {
            let victim = target.unwrap_or(owner);
            let opts: Vec<ChoiceOption> = alive_others(core, victim)
                .into_iter()
                .filter(|&cand| core.computed_distance(catalog, victim, cand) <= 1)
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then(|| ask(owner, "u05_shaoying_target", ChoiceKind::SelectPlayers, opts, 0, 1, true))
        }
        (HostBehavior::YunchouLuweiyan2, _, HostStage::TargetChoice) => {
            let opts: Vec<ChoiceOption> = targets
                .iter()
                .copied()
                .filter(|&t| t != owner && core.player(t).alive)
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then(|| ask(owner, "u05_luweiyan2_target", ChoiceKind::SelectPlayers, opts, 0, 1, true))
        }
        (HostBehavior::YunchouBeige, _, HostStage::SelectCard) => {
            let he = hand_and_equips(core, owner);
            (!he.is_empty()).then(|| {
                ask(
                    owner,
                    "beige_discard",
                    ChoiceKind::SelectCards,
                    he.into_iter().map(ChoiceOption::Card).collect(),
                    1,
                    1,
                    false,
                )
            })
        }
        (HostBehavior::YunchouBeige | HostBehavior::U05YunchouCharBatch, key, HostStage::CardsAsk)
            if behavior == HostBehavior::YunchouBeige || key == "ycn_beige" =>
        {
            let src = (targets.len() >= 2).then(|| targets[0])?;
            let he = hand_and_equips(core, src);
            let n = he.len().min(2);
            (n > 0).then(|| {
                ask(
                    src,
                    "u05_beige_club_discard",
                    ChoiceKind::SelectCards,
                    he.into_iter().map(ChoiceOption::Card).collect(),
                    n,
                    n,
                    false,
                )
            })
        }
        (HostBehavior::U05YunchouCharBatch, "ycn_beige", HostStage::SelectCard) => {
            let he = hand_and_equips(core, owner);
            (!he.is_empty()).then(|| {
                ask(
                    owner,
                    "u05_ycn_beige_discard",
                    ChoiceKind::SelectCards,
                    he.into_iter().map(ChoiceOption::Card).collect(),
                    0,
                    1,
                    true,
                )
            })
        }
        (HostBehavior::YunchouYaliang, _, HostStage::SelectNature) => Some(ask(
            owner,
            "trigger_confirm",
            crate::choice::ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        )),
        (HostBehavior::YunchouYiesheng, _, HostStage::SelectCard) => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| is_black_card(core, catalog, owner, c))
                .map(ChoiceOption::Card)
                .collect();
            let max = opts.len();
            (max > 0).then(|| ask(owner, "yiesheng_discard", ChoiceKind::SelectCards, opts, 1, max, true))
        }
        _ => None,
    }
}

fn wait_ask_yc_char(m: &mut HostMachine, ctx: &mut Ctx<'_>, next_stage: HostStage) -> Flow {
    let Some(exp) = ask_u05_yunchou_char(
        m.behavior,
        next_stage,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) else {
        return Flow::Done;
    };
    m.stage = next_stage;
    Flow::WaitChoice {
        observer: exp.observer,
        spec: ChoiceSpec {
            kind: exp.kind,
            prompt: exp.prompt,
            options: exp.options,
            min: exp.min,
            max: exp.max,
            allow_cancel: exp.allow_cancel,
        },
    }
}

fn step_qiaosi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let num = (ctx.core.rng.gen_range(6) + 1) as u32;
            m.card = Some(num);
            ctx.start_draw(owner, num);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => wait_ask_yc_char(m, ctx, HostStage::SelectNature),
        HostStage::SelectNature => match ctx.answer().first().copied() {
            Some(ChoiceOption::Tag(0)) => {
                if hand_and_equips(ctx.core, owner).is_empty() || alive_others(ctx.core, owner).is_empty() {
                    return Flow::Done;
                }
                wait_ask_yc_char(m, ctx, HostStage::SelectCard)
            }
            _ => {
                if hand_and_equips(ctx.core, owner).is_empty() {
                    return Flow::Done;
                }
                wait_ask_yc_char(m, ctx, HostStage::CardsAsk)
            }
        },
        HostStage::SelectCard => {
            let cards: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if cards.is_empty() {
                return Flow::Done;
            }
            m.cards = cards;
            wait_ask_yc_char(m, ctx, HostStage::TargetChoice)
        }
        HostStage::TargetChoice => {
            if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                move_cards_to_hand(ctx, owner, t, &m.cards);
            }
            Flow::Done
        }
        HostStage::CardsAsk => {
            let cards: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            discard_cards(ctx, owner, &cards);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_xinfu_jingxie(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let is_dying = ctx.res.stack.last().is_some_and(|f| matches!(f.data, EventData::Dying { .. }));
            m.target = is_dying.then_some(owner);
            wait_ask_yc_char(m, ctx, HostStage::SelectCard)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if m.target == Some(owner) {
                ctx.take_owned_card(owner, c);
                ctx.to_discard(c);
                ctx.start_draw(owner, 1);
                m.stage = HostStage::DrawSettle;
                Flow::Yield
            } else {
                set_flag(ctx.core, 0, &format!("th_jingxie_rewritten:{c}"), 1);
                set_flag(ctx.core, owner, &format!("th_jingxie_upgraded:{c}"), 1);
                Flow::Done
            }
        }
        HostStage::DrawSettle => {
            if ctx.core.player(owner).alive {
                let hp = ctx.core.player(owner).hp;
                if hp < 1 {
                    ctx.recover(owner, (1 - hp) as u32, None);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_diyguhuo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            ctx.start_draw(owner, 2);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => wait_ask_yc_char(m, ctx, HostStage::CardsAsk),
        HostStage::CardsAsk => {
            let cards: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            let mut judge_discards = Vec::new();
            let mut hand_or_eq = Vec::new();
            for c in cards {
                if let Some(pos) = ctx.core.player(owner).judge.iter().position(|&x| x == c) {
                    ctx.core.player_mut(owner).judge.remove(pos);
                    ctx.to_discard(c);
                    judge_discards.push(c);
                } else {
                    hand_or_eq.push(c);
                }
            }
            if !judge_discards.is_empty() {
                ctx.log(crate::machine::LogEvent::CardDiscarded {
                    player: owner,
                    cards: judge_discards,
                });
            }
            discard_cards(ctx, owner, &hand_or_eq);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn apply_ycn_beige_suit(m: &mut HostMachine, ctx: &mut Ctx<'_>, suit: Suit) -> Flow {
    let [src, victim] = [m.targets[0], m.targets[1]];
    match suit {
        Suit::Heart => {
            let pl = ctx.core.player(victim);
            if pl.alive && pl.hp < pl.max_hp as i32 {
                ctx.recover(victim, 1, None);
            }
            Flow::Done
        }
        Suit::Diamond => {
            if ctx.core.player(victim).alive {
                ctx.start_draw(victim, 2);
            }
            Flow::Done
        }
        Suit::Spade => {
            if ctx.core.player(src).alive {
                ctx.emit(EventData::TurnOverBefore {
                    player: src,
                    cancelled: false,
                });
            }
            Flow::Done
        }
        Suit::Club => {
            if ctx.core.player(src).alive && !hand_and_equips(ctx.core, src).is_empty() {
                return wait_ask_yc_char(m, ctx, HostStage::CardsAsk);
            }
            Flow::Done
        }
    }
}

/// Classic `beige` (`shenhua/skill.js:4814`): cancelable discard cost before judge, NO `beige_gains`.
fn step_ycn_beige(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.targets.len() < 2 || hand_and_equips(ctx.core, owner).is_empty() {
                return Flow::Done;
            }
            wait_ask_yc_char(m, ctx, HostStage::SelectCard)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !hand_and_equips(ctx.core, owner).contains(&c) {
                return Flow::Done;
            }
            discard_cards(ctx, owner, &[c]);
            let judged = ctx.draw_judge_card_with_elo(
                owner,
                engine_native_kernel::kernel::EloJudgePref::Beige,
            );
            let victim = m.targets[1];
            let (suit, _) = ctx.inline_judge_suit_rank(victim, judged);
            m.cards = vec![judged];
            if ctx.settle_inline_judge_card(victim, suit, judged) {
                m.stage = HostStage::SelectNature;
                return Flow::Yield;
            }
            apply_ycn_beige_suit(m, ctx, suit)
        }
        HostStage::SelectNature => {
            let Some(&judged) = m.cards.first() else {
                return Flow::Done;
            };
            let victim = m.targets[1];
            let (suit, _) = ctx.inline_judge_suit_rank(victim, judged);
            apply_ycn_beige_suit(m, ctx, suit)
        }
        HostStage::CardsAsk => {
            let cards: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if let Some(&src) = m.targets.first() {
                discard_cards(ctx, src, &cards);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_beige(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if hand_and_equips(ctx.core, owner).is_empty() {
                return Flow::Done;
            }
            wait_ask_yc_char(m, ctx, HostStage::SelectCard)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            discard_cards(ctx, owner, &[c]);
            let judged = ctx.draw_judge_card();
            let suit = ctx.catalog.cards[judged as usize].suit;
            ctx.to_discard(judged);
            m.cards = vec![judged, c];
            let (source, victim) = if m.targets.len() >= 2 {
                (Some(m.targets[0]), m.targets[1])
            } else {
                (None, m.targets.first().copied().unwrap_or(owner))
            };
            match suit {
                Suit::Heart => {
                    let pl = ctx.core.player(victim);
                    if pl.alive && pl.hp < pl.max_hp as i32 {
                        ctx.recover(victim, 1, None);
                    }
                    super::yunchou::beige_gains(ctx, owner, judged, c);
                    Flow::Done
                }
                Suit::Diamond => {
                    if ctx.core.player(victim).alive {
                        ctx.start_draw(victim, 2);
                        m.stage = HostStage::DrawSettle;
                        return Flow::Yield;
                    }
                    super::yunchou::beige_gains(ctx, owner, judged, c);
                    Flow::Done
                }
                Suit::Club => {
                    if let Some(src) = source {
                        if ctx.core.player(src).alive && !hand_and_equips(ctx.core, src).is_empty() {
                            return wait_ask_yc_char(m, ctx, HostStage::CardsAsk);
                        }
                    }
                    super::yunchou::beige_gains(ctx, owner, judged, c);
                    Flow::Done
                }
                Suit::Spade => {
                    if let Some(src) = source {
                        if ctx.core.player(src).alive {
                            ctx.emit(EventData::TurnOverBefore {
                                player: src,
                                cancelled: false,
                            });
                        }
                    }
                    super::yunchou::beige_gains(ctx, owner, judged, c);
                    Flow::Done
                }
            }
        }
        HostStage::CardsAsk => {
            let cards: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if m.targets.len() >= 2 {
                let src = m.targets[0];
                discard_cards(ctx, src, &cards);
            }
            if let [judged, c] = m.cards.as_slice() {
                super::yunchou::beige_gains(ctx, owner, *judged, *c);
            }
            Flow::Done
        }
        HostStage::DrawSettle => {
            if let [judged, c] = m.cards.as_slice() {
                super::yunchou::beige_gains(ctx, owner, *judged, *c);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn is_red_suit(suit: Suit) -> bool {
    matches!(suit, Suit::Heart | Suit::Diamond)
}

fn draw_yaliang_pair(ctx: &mut Ctx<'_>, a: PlayerId, b: PlayerId) {
    let order = ctx.core.seat_order_from(ctx.core.turn.current);
    for p in order.into_iter().rev() {
        if (p == a || p == b) && ctx.core.player(p).alive {
            ctx.start_draw(p, 1);
        }
    }
}

pub(in crate::content::host) fn handle_u05_yunchou_char(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match (m.behavior, m.def_key.as_str()) {
        (HostBehavior::U05YunchouCharBatch, "ycndiyjianxiong") => {
            let dead_player = ctx.res.stack.iter().rev().find_map(|f| match f.data {
                EventData::Death { player, .. } => Some(player),
                _ => None,
            });
            if let Some(player) = dead_player {
                match &mut ctx.core.mode {
                    crate::state::GameMode::Identity { roles } => {
                        if let Some(slot) = roles.get_mut(player as usize) {
                            if *slot != crate::state::Role::Lord {
                                *slot = crate::state::Role::Rebel;
                            }
                        }
                    }
                    crate::state::GameMode::Kingdoms => {
                        let owner_group: Option<String> = match &ctx.core.player(owner).kingdom_identity
                        {
                            crate::state::KingdomIdentity::Declared(n) => {
                                Some(GameStateCore::jun_group(n).unwrap_or(n).to_string())
                            }
                            crate::state::KingdomIdentity::Ye
                            | crate::state::KingdomIdentity::Unknown => ctx
                                .core
                                .declared_faction(ctx.catalog, owner)
                                .map(|f| GameStateCore::faction_name(f).to_string()),
                        };
                        let choices: Vec<&str> = ["wei", "shu", "wu", "qun"]
                            .into_iter()
                            .filter(|&g| owner_group.as_deref() != Some(g))
                            .collect();
                        if !choices.is_empty() {
                            let picked =
                                choices[ctx.core.rng.gen_range(choices.len() as u32) as usize];
                            ctx.core.player_mut(player).yexinjia_friends.clear();
                            ctx.core
                                .player_mut(owner)
                                .yexinjia_friends
                                .retain(|&p| p != player);
                            let _ = ctx.core.set_kingdom_identity(
                                player,
                                crate::state::KingdomIdentity::Declared(picked.to_string()),
                            );
                        }
                    }
                    _ => {}
                }
            }
            Flow::Done
        }
        (HostBehavior::U05YunchouCharBatch, "xinfu_jingxie") => step_xinfu_jingxie(m, ctx),
        (HostBehavior::U05YunchouCharBatch, "ycnxinfu_qiaosi" | "xinfu_qiaosi") => step_qiaosi(m, ctx),
        (HostBehavior::U05YunchouCharBatch, "ycn_beige") => step_ycn_beige(m, ctx),
        (HostBehavior::YunchouXicai, _) => {
            if let Ok(d) = ctx.current_damage() {
                if let Some(c) = d.card {
                    if ctx.core.table.contains(&c) || ctx.core.discard.contains(&c) {
                        ctx.core.table.retain(|&x| x != c);
                        ctx.core.discard.retain(|&x| x != c);
                        ctx.core.player_mut(owner).hand.push(c);
                    }
                }
            }
            Flow::Done
        }
        (HostBehavior::YunchouDiykuanggu, _) => {
            if let Ok(d) = ctx.current_damage() {
                let times = d.amount.max(0) as u32;
                let dist = ctx.core.computed_distance(ctx.catalog, d.target, owner);
                for _ in 0..times {
                    if dist > 1 {
                        ctx.start_draw(owner, 1);
                    } else if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                        ctx.recover(owner, 1, None);
                    }
                }
            }
            Flow::Done
        }
        (HostBehavior::YunchouXiongzi, _) => {
            let n = 1 + ctx.core.player(owner).equips.len() / 2;
            ctx.start_draw(owner, n as u32);
            Flow::Done
        }
        (HostBehavior::YunchouYaliang, _) => match m.stage {
            HostStage::Act => {
                if m.targets.len() != 2 {
                    return Flow::Done;
                }
                let num = ctx
                    .current_damage()
                    .map(|d| d.amount.max(1) as u32)
                    .unwrap_or(1);
                let (a, b) = (m.targets[0], m.targets[1]);
                draw_yaliang_pair(ctx, a, b);
                if num > 1 {
                    m.card = Some(num - 1);
                    m.stage = HostStage::DrawSettle;
                    return Flow::Yield;
                }
                Flow::Done
            }
            HostStage::DrawSettle => {
                if m.targets.len() == 2
                    && ctx.core.player(m.targets[0]).alive
                    && ctx.core.player(m.targets[1]).alive
                {
                    return wait_ask_yc_char(m, ctx, HostStage::SelectNature);
                }
                Flow::Done
            }
            HostStage::SelectNature => {
                if ctx.answer().first().copied() == Some(ChoiceOption::Confirm)
                    && m.targets.len() == 2
                {
                    draw_yaliang_pair(ctx, m.targets[0], m.targets[1]);
                }
                let rem = m.card.unwrap_or(1).saturating_sub(1);
                if rem > 0 {
                    m.card = Some(rem);
                    m.stage = HostStage::DrawSettle;
                    return Flow::Yield;
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        (HostBehavior::YunchouDiychanyuan, _) => {
            if let Some(f) = ctx.res.stack.last() {
                if let EventData::Death { killer: Some(k), .. } = f.data {
                    if ctx.core.player(k).alive {
                        ctx.lose_max_hp(k, 1);
                    }
                }
            }
            Flow::Done
        }
        (HostBehavior::YunchouBeige, _) => step_beige(m, ctx),
        (HostBehavior::YunchouJieyan, _) => match m.stage {
            HostStage::Act => wait_ask_yc_char(m, ctx, HostStage::SelectCard),
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                discard_cards(ctx, owner, &[c]);
                for t in alive_all(ctx.core, owner).into_iter().rev() {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(owner),
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
        (HostBehavior::YunchouYiesheng, _) => match m.stage {
            HostStage::Act => wait_ask_yc_char(m, ctx, HostStage::SelectCard),
            HostStage::SelectCard => {
                let cards: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                let cnt = cards.len() as u32;
                discard_cards(ctx, owner, &cards);
                if cnt > 0 {
                    ctx.start_draw(owner, cnt);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        (HostBehavior::YunchouLuweiyan2, _) => match m.stage {
            HostStage::Act => wait_ask_yc_char(m, ctx, HostStage::TargetChoice),
            HostStage::TargetChoice => {
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    if ctx.core.player(t).alive {
                        ctx.use_virtual_card(owner, CardKind::Sha, vec![t]);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        (HostBehavior::YunchouDiyguhuo, _) => step_diyguhuo(m, ctx),
        (HostBehavior::YunchouJiaoxia, _) => {
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        (HostBehavior::YunchouHonglian, _) => {
            if let Ok(d) = ctx.current_damage() {
                if let Some(src) = d.source {
                    if src != owner && ctx.core.player(src).alive {
                        let reds: Vec<CardId> = hand_and_equips(ctx.core, src)
                            .into_iter()
                            .filter(|&c| is_red_card(ctx.core, ctx.catalog, src, c))
                            .collect();
                        discard_cards(ctx, src, &reds);
                    }
                }
            }
            Flow::Done
        }
        (HostBehavior::YunchouZonghuo, _) => match m.stage {
            HostStage::Act => wait_ask_yc_char(m, ctx, HostStage::SelectCard),
            HostStage::SelectCard => {
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    if ctx.core.player(owner).hand.contains(&c) {
                        ctx.current_damage_mut().nature = DamageNature::Fire;
                        discard_cards(ctx, owner, &[c]);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        (HostBehavior::YunchouShaoying, _) => match m.stage {
            HostStage::Act => {
                let victim = ctx.current_damage().map(|d| d.target).unwrap_or(owner);
                m.target = Some(victim);
                wait_ask_yc_char(m, ctx, HostStage::TargetChoice)
            }
            HostStage::TargetChoice => {
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    let top = ctx.draw_judge_card();
                    let is_red = is_red_suit(ctx.catalog.cards[top as usize].suit);
                    ctx.to_discard(top);
                    if is_red && ctx.core.player(t).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(owner),
                            t,
                            1,
                            DamageNature::Fire,
                            None,
                        )));
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}
