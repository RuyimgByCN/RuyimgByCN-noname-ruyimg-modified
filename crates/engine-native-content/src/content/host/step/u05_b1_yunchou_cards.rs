//! Batch U05-1 运筹帷幄卡牌 (`card/yunchou/card.js`) 全量等价实现。

use super::super::*;
use super::l27_batch_a::{alive_others, discard_cards, move_cards_to_hand};
use crate::catalog::{CardKind, Catalog, Suit};
use crate::choice::ChoiceOption;
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;

fn is_red_suit(suit: Suit) -> bool {
    matches!(suit, Suit::Heart | Suit::Diamond)
}

fn finish_yunchou_card(m: &mut HostMachine, ctx: &mut Ctx<'_>) {
    if let Some(c) = m.card {
        ctx.core.table.retain(|x| *x != c);
        ctx.to_discard(c);
        m.card = None;
    }
}

pub(in crate::content::host) fn ask_u05_yunchou_card(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    _card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match behavior {
        HostBehavior::ChenhuodajieCard => match stage {
            HostStage::TargetChoice => {
                let opts: Vec<ChoiceOption> = core
                    .player(owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|&c| catalog.card_kind(c) == CardKind::Chenhuodajie)
                    .map(ChoiceOption::Card)
                    .collect();
                (!opts.is_empty())
                    .then(|| ask(owner, "g_chenhuodajie_use", ChoiceKind::SelectCards, opts, 1, 1, true))
            }
            HostStage::SelectCard => {
                let t = target?;
                let opts: Vec<ChoiceOption> = core
                    .player(t)
                    .hand
                    .iter()
                    .chain(core.player(t).equips.iter())
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                (!opts.is_empty())
                    .then(|| ask(owner, "u05_chenhuodajie_gain", ChoiceKind::SelectCards, opts, 1, 1, false))
            }
            _ => None,
        },
        HostBehavior::ShuigongCard if stage == HostStage::SelectCard => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core.player(t).equips.iter().copied().map(ChoiceOption::Card).collect();
            (!opts.is_empty()).then(|| ask(t, "u05_shuigong_discard", ChoiceKind::SelectCards, opts, 1, 1, false))
        }
        HostBehavior::CaochuanCard => match stage {
            HostStage::Act => {
                let t = target?;
                let has_sha = core.player(t).hand.iter().any(|&c| catalog.card_kind(c).is_sha_like());
                if has_sha {
                    Some(ask(
                        t,
                        WD_CAOCHUAN_BRANCH,
                        ChoiceKind::PickTag,
                        vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                        1,
                        1,
                        false,
                    ))
                } else {
                    let opts: Vec<ChoiceOption> = core.player(t).hand.iter().copied().map(ChoiceOption::Card).collect();
                    (!opts.is_empty()).then(|| {
                        ask(owner, "u05_caochuan_visible_discard", ChoiceKind::SelectCards, opts, 1, 1, false)
                    })
                }
            }
            HostStage::SelectCard => {
                let t = target?;
                let opts: Vec<ChoiceOption> = core.player(t).hand.iter().copied().map(ChoiceOption::Card).collect();
                (!opts.is_empty()).then(|| ask(owner, "u05_caochuan_visible_discard", ChoiceKind::SelectCards, opts, 1, 1, false))
            }
            _ => None,
        },
        HostBehavior::GeanguanhuoCompare => match stage {
            HostStage::ComparePlayerPick => {
                let t1 = *targets.first()?;
                let opts: Vec<ChoiceOption> = core.player(t1).hand.iter().copied().map(ChoiceOption::Card).collect();
                (!opts.is_empty()).then(|| ask(t1, YC_GEAN_T1_PICK, ChoiceKind::SelectCards, opts, 1, 1, false))
            }
            HostStage::Act => {
                let t2 = *targets.get(1)?;
                let opts: Vec<ChoiceOption> = core.player(t2).hand.iter().copied().map(ChoiceOption::Card).collect();
                (!opts.is_empty()).then(|| ask(t2, YC_GEAN_T2_PICK, ChoiceKind::SelectCards, opts, 1, 1, false))
            }
            HostStage::SelectCard => {
                let winner = target?;
                let loser = if Some(winner) == targets.first().copied() {
                    *targets.get(1)?
                } else {
                    *targets.first()?
                };
                let opts: Vec<ChoiceOption> = core
                    .player(loser)
                    .hand
                    .iter()
                    .chain(core.player(loser).equips.iter())
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                (!opts.is_empty()).then(|| ask(winner, "u05_gean_winner_gain", ChoiceKind::SelectCards, opts, 1, 1, false))
            }
            HostStage::CardsAsk => {
                let t1 = *targets.first()?;
                let opts: Vec<ChoiceOption> = core
                    .player(owner)
                    .hand
                    .iter()
                    .chain(core.player(owner).equips.iter())
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                (!opts.is_empty()).then(|| ask(t1, "u05_gean_tie_discard", ChoiceKind::SelectCards, opts, 0, 1, true))
            }
            _ => None,
        },
        HostBehavior::ShezhanqunruCompare => match stage {
            HostStage::TargetChoice => {
                let cand = *targets.first()?;
                Some(ask(
                    cand,
                    "u05_shezhan_respond",
                    ChoiceKind::RespondCard,
                    vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                    1,
                    1,
                    false,
                ))
            }
            HostStage::ComparePlayerPick => {
                let opts: Vec<ChoiceOption> = core.player(owner).hand.iter().copied().map(ChoiceOption::Card).collect();
                (!opts.is_empty()).then(|| ask(owner, "u05_shezhan_own_pick", ChoiceKind::SelectCards, opts, 1, 1, false))
            }
            HostStage::CompareTargetPick => {
                let idx = cards.len().saturating_sub(2);
                let opp = *targets.get(idx)?;
                let opts: Vec<ChoiceOption> = core.player(opp).hand.iter().copied().map(ChoiceOption::Card).collect();
                (!opts.is_empty()).then(|| ask(opp, "u05_shezhan_target_pick", ChoiceKind::SelectCards, opts, 1, 1, false))
            }
            _ => None,
        },
        _ => None,
    }
}

fn wait_ask_yc_card(m: &mut HostMachine, ctx: &mut Ctx<'_>, next_stage: HostStage) -> Flow {
    let Some(exp) = ask_u05_yunchou_card(
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

fn step_touliang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        return Flow::Done;
    };
    match m.stage {
        HostStage::SelectCard => {
            if ctx.core.player(t).hand.is_empty() {
                finish_yunchou_card(m, ctx);
                return Flow::Done;
            }
            if ctx.core.player(m.owner).hand.is_empty() {
                let revealed = ctx.core.player(t).hand.clone();
                ctx.log(LogEvent::CardsRevealed {
                    cards: revealed,
                    to: vec![m.owner],
                });
                let key = format!("u05_touliang_viewed:{t}");
                let prev = ctx.core.vm_aux.flags.get(&(m.owner, key.clone())).copied().unwrap_or(0);
                ctx.core.vm_aux.flags.insert((m.owner, key), prev + 1);
                finish_yunchou_card(m, ctx);
                return Flow::Done;
            }
            m.stage = HostStage::SelectNature;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: WD_TOULIANG_OWN,
                    options: ctx.core.player(m.owner).hand.iter().copied().map(ChoiceOption::Card).collect(),
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            }
        }
        HostStage::SelectNature => {
            let Some(ChoiceOption::Card(own_c)) = ctx.answer().first().copied() else {
                finish_yunchou_card(m, ctx);
                return Flow::Done;
            };
            m.cards = vec![own_c];
            m.stage = HostStage::Act;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: WD_TOULIANG_TARGET,
                    options: ctx.core.player(t).hand.iter().copied().map(ChoiceOption::Card).collect(),
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            }
        }
        HostStage::Act => {
            let (Some(ChoiceOption::Card(t_c)), Some(&own_c)) = (ctx.answer().first().copied(), m.cards.first()) else {
                finish_yunchou_card(m, ctx);
                return Flow::Done;
            };
            if ctx.core.player(m.owner).hand.contains(&own_c) && ctx.core.player(t).hand.contains(&t_c) {
                move_cards_to_hand(ctx, m.owner, t, &[own_c]);
                move_cards_to_hand(ctx, t, m.owner, &[t_c]);
                let s1 = ctx.catalog.cards[own_c as usize].suit;
                let s2 = ctx.catalog.cards[t_c as usize].suit;
                if is_red_suit(s1) == is_red_suit(s2) {
                    ctx.start_draw(m.owner, 1);
                }
            }
            finish_yunchou_card(m, ctx);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_fudichouxin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        return Flow::Done;
    };
    match m.stage {
        HostStage::CompareTargetPick => {
            if ctx.core.player(t).hand.is_empty() || ctx.core.player(m.owner).hand.is_empty() {
                finish_yunchou_card(m, ctx);
                return Flow::Done;
            }
            m.stage = HostStage::ComparePlayerPick;
            Flow::WaitChoice {
                observer: t,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: WD_FUDI_TARGET_PICK,
                    options: ctx.core.player(t).hand.iter().copied().map(ChoiceOption::Card).collect(),
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            }
        }
        HostStage::ComparePlayerPick => {
            let ChoiceOption::Card(tc) = ctx.answer_first() else {
                finish_yunchou_card(m, ctx);
                return Flow::Done;
            };
            m.cards = vec![tc];
            m.stage = HostStage::Act;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: WD_FUDI_PLAYER_PICK,
                    options: ctx.core.player(m.owner).hand.iter().copied().map(ChoiceOption::Card).collect(),
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            }
        }
        HostStage::Act => {
            let Some(ChoiceOption::Card(pc)) = ctx.answer().first().copied() else {
                finish_yunchou_card(m, ctx);
                return Flow::Done;
            };
            if let Some(&tc) = m.cards.first() {
                let pr = ctx.catalog.cards[pc as usize].rank;
                let tr = ctx.catalog.cards[tc as usize].rank;
                if pr > tr {
                    move_cards_to_hand(ctx, t, m.owner, &[tc]);
                    if ctx
                        .catalog
                        .cards
                        .get(pc as usize)
                        .is_some_and(|cd| cd.name.starts_with("mtg_du_"))
                        && ctx.core.player(m.owner).alive
                    {
                        ctx.emit(EventData::LoseHp {
                            player: m.owner,
                            amount: 1,
                        });
                    }
                    if ctx
                        .catalog
                        .cards
                        .get(tc as usize)
                        .is_some_and(|cd| cd.name.starts_with("mtg_du_"))
                        && ctx.core.player(t).alive
                    {
                        ctx.emit(EventData::LoseHp {
                            player: t,
                            amount: 1,
                        });
                    }
                } else {
                    discard_cards(ctx, m.owner, &[pc]);
                    discard_cards(ctx, t, &[tc]);
                }
            }
            finish_yunchou_card(m, ctx);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_geanguanhuo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.targets.len() != 2 {
        finish_yunchou_card(m, ctx);
        return Flow::Done;
    }
    let (t1, t2) = (m.targets[0], m.targets[1]);
    match m.stage {
        HostStage::CompareTargetPick => {
            if ctx.core.player(t1).hand.is_empty() || ctx.core.player(t2).hand.is_empty() {
                finish_yunchou_card(m, ctx);
                return Flow::Done;
            }
            wait_ask_yc_card(m, ctx, HostStage::ComparePlayerPick)
        }
        HostStage::ComparePlayerPick => {
            let ChoiceOption::Card(c1) = ctx.answer_first() else {
                finish_yunchou_card(m, ctx);
                return Flow::Done;
            };
            m.cards = vec![c1];
            wait_ask_yc_card(m, ctx, HostStage::Act)
        }
        HostStage::Act => {
            let (Some(ChoiceOption::Card(c2)), Some(&c1)) = (ctx.answer().first().copied(), m.cards.first()) else {
                finish_yunchou_card(m, ctx);
                return Flow::Done;
            };
            discard_cards(ctx, t1, &[c1]);
            discard_cards(ctx, t2, &[c2]);
            let r1 = ctx.catalog.cards[c1 as usize].rank;
            let r2 = ctx.catalog.cards[c2 as usize].rank;
            if r1 == r2 {
                if !ctx.core.player(m.owner).hand.is_empty() || !ctx.core.player(m.owner).equips.is_empty() {
                    return wait_ask_yc_card(m, ctx, HostStage::CardsAsk);
                }
            } else {
                let (winner, loser) = if r1 > r2 { (t1, t2) } else { (t2, t1) };
                m.target = Some(winner);
                if !ctx.core.player(loser).hand.is_empty() || !ctx.core.player(loser).equips.is_empty() {
                    return wait_ask_yc_card(m, ctx, HostStage::SelectCard);
                }
            }
            finish_yunchou_card(m, ctx);
            Flow::Done
        }
        HostStage::SelectCard => {
            if let (Some(winner), Some(ChoiceOption::Card(c))) = (m.target, ctx.answer().first().copied()) {
                let loser = if winner == t1 { t2 } else { t1 };
                move_cards_to_hand(ctx, loser, winner, &[c]);
            }
            finish_yunchou_card(m, ctx);
            Flow::Done
        }
        HostStage::CardsAsk => {
            if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                discard_cards(ctx, m.owner, &[c]);
            }
            finish_yunchou_card(m, ctx);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_shezhanqunru(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    // Polling (`TargetChoice`): `m.targets` = remaining candidates, `m.cards[0] = 50`, `m.cards[1..]` = confirmed seats.
    // Compare (`ComparePlayerPick`/`CompareTargetPick`): `m.targets` = confirmed responders,
    // `m.cards[0]` = `50` net marker, `m.cards[1]` = owner compare card, `m.cards[2..]` = responder picks.
    match m.stage {
        HostStage::Act => {
            let candidates: Vec<PlayerId> = alive_others(ctx.core, m.owner)
                .into_iter()
                .filter(|&p| !ctx.core.player(p).hand.is_empty())
                .collect();
            if candidates.is_empty() || ctx.core.player(m.owner).hand.is_empty() {
                ctx.start_draw(m.owner, 3);
                finish_yunchou_card(m, ctx);
                return Flow::Done;
            }
            m.targets = candidates;
            m.cards = vec![50];
            wait_ask_yc_card(m, ctx, HostStage::TargetChoice)
        }
        HostStage::TargetChoice => {
            let cand = m.targets.remove(0);
            if ctx.answer().first().copied() == Some(ChoiceOption::Confirm) {
                m.cards.push(u32::from(cand));
            }
            if !m.targets.is_empty() {
                return wait_ask_yc_card(m, ctx, HostStage::TargetChoice);
            }
            let responders: Vec<PlayerId> = m
                .cards
                .drain(1..)
                .map(|v| v as PlayerId)
                .filter(|&opp| ctx.core.player(opp).alive && !ctx.core.player(opp).hand.is_empty())
                .collect();
            if responders.is_empty() || ctx.core.player(m.owner).hand.is_empty() {
                ctx.start_draw(m.owner, 3);
                finish_yunchou_card(m, ctx);
                return Flow::Done;
            }
            m.targets = responders;
            m.cards = vec![50];
            wait_ask_yc_card(m, ctx, HostStage::ComparePlayerPick)
        }
        HostStage::ComparePlayerPick => {
            let ChoiceOption::Card(own_c) = ctx.answer_first() else {
                finish_yunchou_card(m, ctx);
                return Flow::Done;
            };
            m.cards = vec![50, own_c];
            advance_shezhan_next_target(m, ctx)
        }
        HostStage::CompareTargetPick => {
            if let ChoiceOption::Card(tgt_c) = ctx.answer_first() {
                m.cards.push(tgt_c);
                return advance_shezhan_next_target(m, ctx);
            }
            // Fallback if responder failed pick: remove current responder and continue
            let idx = m.cards.len().saturating_sub(2);
            if idx < m.targets.len() {
                m.targets.remove(idx);
            }
            advance_shezhan_next_target(m, ctx)
        }
        _ => Flow::Done,
    }
}

fn advance_shezhan_next_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    loop {
        let idx = m.cards.len().saturating_sub(2);
        if idx >= m.targets.len() {
            return finish_shezhan(m, ctx);
        }
        let opp = m.targets[idx];
        if ctx.core.player(opp).alive && !ctx.core.player(opp).hand.is_empty() {
            return wait_ask_yc_card(m, ctx, HostStage::CompareTargetPick);
        }
        m.targets.remove(idx);
    }
}

fn finish_shezhan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.cards.len() >= 2 && m.cards.len() >= 2 + m.targets.len() {
        let own_c = m.cards[1];
        let r_own = ctx.catalog.cards[own_c as usize].rank;
        discard_cards(ctx, m.owner, &[own_c]);
        let mut net: i32 = 0;
        for (i, &opp) in m.targets.iter().enumerate() {
            let tgt_c = m.cards[2 + i];
            discard_cards(ctx, opp, &[tgt_c]);
            let r_tgt = ctx.catalog.cards[tgt_c as usize].rank;
            if r_own > r_tgt {
                net += 1;
            } else {
                net -= 1;
            }
        }
        if net > 0 {
            ctx.start_draw(m.owner, 3);
        }
    } else if m.targets.is_empty() {
        ctx.start_draw(m.owner, 3);
    }
    finish_yunchou_card(m, ctx);
    Flow::Done
}

pub(in crate::content::host) fn step_u05_yunchou_cards(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    match m.behavior {
        HostBehavior::ChenhuodajieCard => match m.stage {
            HostStage::Act => {
                let Some(t) = m.target else {
                    finish_yunchou_card(m, ctx);
                    return Flow::Done;
                };
                if ctx.core.player(t).hand.is_empty() && ctx.core.player(t).equips.is_empty() {
                    finish_yunchou_card(m, ctx);
                    return Flow::Done;
                }
                if m.card.is_none() {
                    return wait_ask_yc_card(m, ctx, HostStage::TargetChoice);
                }
                wait_ask_yc_card(m, ctx, HostStage::SelectCard)
            }
            HostStage::TargetChoice => {
                if let (Some(t), Some(ChoiceOption::Card(chd))) =
                    (m.target, ctx.answer().first().copied())
                {
                    if ctx.core.player(m.owner).hand.contains(&chd)
                        && ctx.catalog.card_kind(chd) == CardKind::Chenhuodajie
                    {
                        ctx.emit(EventData::CardUsed(crate::machine::card_use(
                            chd,
                            m.owner,
                            vec![t],
                        )));
                    }
                }
                Flow::Done
            }
            HostStage::SelectCard => {
                if let (Some(t), Some(ChoiceOption::Card(c))) = (m.target, ctx.answer().first().copied()) {
                    move_cards_to_hand(ctx, t, m.owner, &[c]);
                }
                finish_yunchou_card(m, ctx);
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::ShuigongCard => match m.stage {
            HostStage::Act => {
                while !m.targets.is_empty() {
                    let t = m.targets.remove(0);
                    if ctx.core.player(t).alive && !ctx.core.player(t).equips.is_empty() {
                        m.target = Some(t);
                        return wait_ask_yc_card(m, ctx, HostStage::SelectCard);
                    }
                }
                finish_yunchou_card(m, ctx);
                Flow::Done
            }
            HostStage::SelectCard => {
                if let (Some(t), Some(ChoiceOption::Card(c))) = (m.target, ctx.answer().first().copied()) {
                    discard_cards(ctx, t, &[c]);
                }
                m.stage = HostStage::Act;
                Flow::Yield
            }
            _ => Flow::Done,
        },
        HostBehavior::CaochuanCard => match m.stage {
            HostStage::SelectNature => {
                let Some(t) = m.target else {
                    finish_yunchou_card(m, ctx);
                    return Flow::Done;
                };
                if ctx.core.player(t).hand.is_empty() {
                    finish_yunchou_card(m, ctx);
                    return Flow::Done;
                }
                wait_ask_yc_card(m, ctx, HostStage::Act)
            }
            HostStage::Act => {
                let Some(t) = m.target else {
                    finish_yunchou_card(m, ctx);
                    return Flow::Done;
                };
                match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(0)) => {
                        let shas: Vec<CardId> = ctx
                            .core
                            .player(t)
                            .hand
                            .iter()
                            .copied()
                            .filter(|&c| ctx.catalog.card_kind(c).is_sha_like())
                            .collect();
                        move_cards_to_hand(ctx, t, m.owner, &shas);
                        finish_yunchou_card(m, ctx);
                        ctx.use_virtual_card(t, CardKind::Sha, vec![m.owner]);
                        Flow::Done
                    }
                    Some(ChoiceOption::Tag(1)) => wait_ask_yc_card(m, ctx, HostStage::SelectCard),
                    Some(ChoiceOption::Card(c)) => {
                        discard_cards(ctx, t, &[c]);
                        finish_yunchou_card(m, ctx);
                        Flow::Done
                    }
                    _ => {
                        finish_yunchou_card(m, ctx);
                        Flow::Done
                    }
                }
            }
            HostStage::SelectCard => {
                if let (Some(t), Some(ChoiceOption::Card(c))) = (m.target, ctx.answer().first().copied()) {
                    discard_cards(ctx, t, &[c]);
                }
                finish_yunchou_card(m, ctx);
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::TouliangSwap => step_touliang(m, ctx),
        HostBehavior::FudichouxinCompare => step_fudichouxin(m, ctx),
        HostBehavior::GeanguanhuoCompare => step_geanguanhuo(m, ctx),
        HostBehavior::ShezhanqunruCompare => step_shezhanqunru(m, ctx),
        _ => Flow::Done,
    }
}
