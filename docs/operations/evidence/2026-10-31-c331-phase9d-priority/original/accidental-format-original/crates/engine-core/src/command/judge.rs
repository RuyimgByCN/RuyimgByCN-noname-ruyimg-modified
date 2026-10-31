//! Judge phase resolution for delayed tricks plus package-defined rewrites.

use crate::effect::{run_effects, EffectError};
use crate::state::{turn_flag, CardKind, ZoneKind};
use crate::transaction::Transaction;
use engine_api::{CardId, DomainEvent, PlayerId, Suit};
use rule_api::EffectRequest;

/// Final physical judgment result after draw-pile selection or a rewrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct JudgmentResult {
    pub(crate) card: CardId,
    pub(crate) suit: Suit,
    pub(crate) rank: u8,
}

/// Resolve all delayed tricks in the current player's judge zone.
pub(super) fn resolve_judge_zone(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    let queue = collect_delayed_queue(tx, owner);
    resolve_judge_queue(tx, owner, &queue)
}

/// Continue resolving a remaining delayed-card queue after a rewrite barrier.
pub(crate) fn resolve_judge_queue(
    tx: &mut Transaction,
    owner: PlayerId,
    queue: &[CardId],
) -> Result<(), EffectError> {
    if queue.is_empty() {
        return Ok(());
    }
    let delayed = queue[0];
    let rest: Vec<CardId> = queue[1..].to_vec();

    let Some(c) = tx.state().cards.get(delayed).cloned() else {
        return resolve_judge_queue(tx, owner, &rest);
    };
    let Some(jz) = tx.state().zones.find(ZoneKind::Judge, owner) else {
        return Ok(());
    };
    if c.zone != jz || c.owner != owner {
        return resolve_judge_queue(tx, owner, &rest);
    }

    let kind = c.kind;
    let result = flip_judgment(tx, owner, JudgmentFavor::for_delayed(kind))?;

    let rewriters = judgment_rewriters(tx, owner);
    continue_judgment_rewriters(
        tx,
        owner,
        delayed,
        kind,
        result.card,
        result.suit,
        result.rank,
        &rewriters,
        &rest,
    )
}

/// Offer each remaining rewrite-capable player in seat order, then settle the judgment.
pub(crate) fn continue_judgment_rewriters(
    tx: &mut Transaction,
    owner: PlayerId,
    delayed: CardId,
    kind: CardKind,
    result_card: CardId,
    suit: Suit,
    rank: u8,
    rewriters: &[PlayerId],
    rest: &[CardId],
) -> Result<(), EffectError> {
    for (index, rewriter) in rewriters.iter().copied().enumerate() {
        let options = judgment_rewrite_cards(tx, rewriter);
        if options.is_empty() {
            continue;
        }
        let remaining = &rewriters[index..];
        // payload: [owner, delayed, result_card, suit, rank, kind, rewriter_len, rewriters...,
        //           rest_len, delayed_rest...]
        let mut payload = vec![
            owner.raw(),
            delayed.raw(),
            result_card.raw(),
            suit_tag(suit),
            rank as u32,
            kind.tag() as u32,
            remaining.len() as u32,
        ];
        payload.extend(remaining.iter().map(|player| player.raw()));
        payload.push(rest.len() as u32);
        payload.extend(rest.iter().map(|card| card.raw()));
        crate::effect::open_choice(
            tx,
            rewriter,
            0,
            1,
            true, // cancel = decline this rewrite; later rewriters still get their turn
            options.iter().map(|card| card.raw()).collect(),
            engine_api::ChoiceKind::SelectCards,
            crate::state::ContinuationKind::RewriteJudgment,
            payload,
        )?;
        return Err(EffectError::ChoiceRequested);
    }

    let resume = delayed_judgment_resume_payload(owner, delayed, kind, suit, rank, rest);
    match run_effects(
        tx,
        vec![EffectRequest::EmitJudgmentResolved {
            player: owner,
            result_card,
            suit,
            rank,
            cause_card: delayed,
        }],
    ) {
        Ok(_) => finish_delayed_judgment(tx, &resume),
        Err(EffectError::ChoiceRequested) => {
            let resolution = &mut tx.state_mut().resolution;
            resolution.resume_kind = crate::state::ContinuationKind::FinishDelayedJudgment;
            resolution.resume_payload = resume;
            Err(EffectError::ChoiceRequested)
        }
        Err(error) => Err(error),
    }
}

fn delayed_judgment_resume_payload(
    owner: PlayerId,
    delayed: CardId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
    rest: &[CardId],
) -> Vec<u32> {
    let mut payload = vec![
        owner.raw(),
        delayed.raw(),
        kind.tag() as u32,
        suit_tag(suit),
        u32::from(rank),
        rest.len() as u32,
    ];
    payload.extend(rest.iter().map(|card| card.raw()));
    payload
}

/// Complete a delayed trick after its `JudgmentResolved` trigger queue.
pub(crate) fn finish_delayed_judgment(
    tx: &mut Transaction,
    payload: &[u32],
) -> Result<(), EffectError> {
    if payload.len() < 6 {
        return Err(EffectError::EngineFault(
            "truncated delayed judgment continuation".into(),
        ));
    }
    let owner = PlayerId::new(payload[0]);
    let delayed = CardId::new(payload[1]);
    let kind = kind_from_tag(payload[2]);
    let suit = suit_from_tag(payload[3]);
    let rank = payload[4] as u8;
    let rest_len = payload[5] as usize;
    if payload.len() != 6 + rest_len {
        return Err(EffectError::EngineFault(
            "truncated delayed judgment queue".into(),
        ));
    }
    apply_judge_result(tx, owner, delayed, kind, suit, rank)?;
    let rest: Vec<_> = payload[6..].iter().copied().map(CardId::new).collect();
    resolve_judge_queue(tx, owner, &rest)
}

/// Cards currently legal for this player's optional judgment rewrite.
pub(crate) fn judgment_rewrite_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let Some(state_player) = tx.state().players.get(player) else {
        return vec![];
    };
    if !state_player.alive
        || state_player.dying
        || !crate::query::can_rewrite_judgment(tx.state(), player)
        || crate::query::card_response_cancelled_for(tx.state(), player)
    {
        return vec![];
    }

    let mut cards: Vec<_> = [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Expansion]
        .into_iter()
        .filter_map(|kind| tx.state().zones.find(kind, player))
        .flat_map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
        })
        .collect();
    let sha_response_cancelled = crate::query::sha_response_cancelled_for(tx.state(), player);
    cards.retain(|card_id| {
        let Some(card) = tx.state().cards.get(*card_id) else {
            return false;
        };
        crate::query::judgment_rewrite_card_matches(tx.state(), player, card)
            && !crate::query::card_use_cancelled_for_card(tx.state(), player, *card_id)
            && !(sha_response_cancelled
                && crate::query::effective_card_kind(tx.state(), player, card).is_sha())
    });
    cards
}

/// Apply delayed-trick effect with final suit/rank, then continue rest queue via payload.
pub(crate) fn apply_judge_result(
    tx: &mut Transaction,
    owner: PlayerId,
    delayed: CardId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> Result<(), EffectError> {
    tx.push_event(DomainEvent::RuleNote {
        code: 200, // judgment result
        payload: vec![
            owner.raw(),
            delayed.raw(),
            suit_tag(suit),
            rank as u32,
            kind.tag() as u32,
        ],
    });

    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
    let judge_z = tx
        .state()
        .zones
        .find(ZoneKind::Judge, owner)
        .ok_or_else(|| EffectError::Illegal("no judge".into()))?;

    // Delayed may already have left the zone
    let still_there = tx
        .state()
        .cards
        .get(delayed)
        .map(|c| c.zone == judge_z)
        .unwrap_or(false);

    match kind {
        CardKind::TrickLe => {
            if suit != Suit::Heart {
                tx.state_mut().turn.set_flag(turn_flag::SKIP_PLAY, 1);
            }
            if still_there {
                run_effects(
                    tx,
                    vec![EffectRequest::MoveCards {
                        cards: vec![delayed],
                        to: discard,
                        from: Some(judge_z),
                    }],
                )?;
            }
        }
        CardKind::TrickBingliang => {
            if suit != Suit::Club {
                tx.state_mut().turn.set_flag(turn_flag::SKIP_DRAW, 1);
            }
            if still_there {
                run_effects(
                    tx,
                    vec![EffectRequest::MoveCards {
                        cards: vec![delayed],
                        to: discard,
                        from: Some(judge_z),
                    }],
                )?;
            }
        }
        CardKind::TrickShandian => {
            let hit = suit == Suit::Spade && (2..=9).contains(&rank);
            if hit {
                if still_there {
                    run_effects(
                        tx,
                        vec![EffectRequest::MoveCards {
                            cards: vec![delayed],
                            to: discard,
                            from: Some(judge_z),
                        }],
                    )?;
                }
                run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: owner,
                        amount: 3,
                        is_lose_hp: false,
                        nature: rule_api::damage_nature::THUNDER,
                    }],
                )?;
            } else if still_there {
                pass_lightning(tx, owner, delayed)?;
            }
        }
        _ => {
            if still_there {
                // 蓄谋牌（jsrg）：视为使用原牌（恢复原 def 的 kind 表达）——
                // 上游 get.autoViewAs(event.cards[0]) chooseUseTarget。
                let is_xumou = tx
                    .state()
                    .content_card_tags
                    .get(&delayed)
                    .is_some_and(|tags| tags.contains(&crate::content::XUMOU_TAG));
                if is_xumou {
                    let original = tx.state().cards.get(delayed).map(|c| c.kind);
                    tx.state_mut().cards.get_mut(delayed).map(|c| {
                        c.kind = crate::state::CardKind::Other;
                        c
                    });
                    // 判定区蓄谋牌结算：移入弃牌堆并触发视为使用（由技能
                    // 侧 use 事件；此处保持原牌在弃牌堆可追溯）。
                    run_effects(
                        tx,
                        vec![
                            EffectRequest::MoveCards {
                                cards: vec![delayed],
                                to: discard,
                                from: Some(judge_z),
                            },
                            EffectRequest::EmitRuleEvent {
                                code: 55, // xumou settled: original kind
                                payload: vec![
                                    owner.raw(),
                                    delayed.raw(),
                                    original.map(|k| k.tag() as u32).unwrap_or(0),
                                ],
                            },
                        ],
                    )?;
                    return Ok(());
                }
                run_effects(
                    tx,
                    vec![EffectRequest::MoveCards {
                        cards: vec![delayed],
                        to: discard,
                        from: Some(judge_z),
                    }],
                )?;
            }
        }
    }
    Ok(())
}

fn collect_delayed_queue(tx: &Transaction, owner: PlayerId) -> Vec<CardId> {
    let Some(judge_z) = tx.state().zones.find(ZoneKind::Judge, owner) else {
        return vec![];
    };
    let cards: Vec<CardId> = tx
        .state()
        .zones
        .get(judge_z)
        .map(|z| z.cards.clone())
        .unwrap_or_default();
    let mut delayed = Vec::new();
    let mut lightnings = Vec::new();
    for cid in cards {
        let kind = tx
            .state()
            .cards
            .get(cid)
            .map(|c| c.kind)
            .unwrap_or(CardKind::Other);
        if kind == CardKind::TrickShandian {
            lightnings.push(cid);
        } else if kind.is_delayed_trick() {
            delayed.push(cid);
        }
    }
    delayed.extend(lightnings);
    delayed
}

pub(crate) fn judgment_rewriters(tx: &Transaction, start: PlayerId) -> Vec<PlayerId> {
    let order = tx.state().players.alive_by_seat();
    if order.is_empty() {
        return vec![];
    }
    let start_idx = order.iter().position(|p| *p == start).unwrap_or(0);
    let mut rewriters = Vec::new();
    for i in 0..order.len() {
        let p = order[(start_idx + i) % order.len()];
        if crate::query::can_rewrite_judgment(tx.state(), p) {
            rewriters.push(p);
        }
    }
    rewriters
}

fn pass_lightning(tx: &mut Transaction, owner: PlayerId, card: CardId) -> Result<(), EffectError> {
    let order = tx.state().players.alive_by_seat();
    if order.is_empty() {
        return Ok(());
    }
    let idx = order.iter().position(|p| *p == owner).unwrap_or(0);
    let next = order[(idx + 1) % order.len()];
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Judge, owner)
        .ok_or_else(|| EffectError::Illegal("no judge".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Judge, next)
        .ok_or_else(|| EffectError::Illegal("no next judge".into()))?;
    let next_has = tx
        .state()
        .zones
        .get(to)
        .map(|z| {
            z.cards.iter().any(|cid| {
                tx.state()
                    .cards
                    .get(*cid)
                    .map(|c| c.kind == CardKind::TrickShandian)
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);
    if next_has {
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
        run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(from),
            }],
        )?;
    } else {
        run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards: vec![card],
                to,
                from: Some(from),
            }],
        )?;
    }
    Ok(())
}

/// Result shape whose highest-scoring draw-pile card is favorable to the judged player.
#[derive(Debug, Clone, Copy)]
pub(crate) enum JudgmentFavor {
    /// Every physical result has the same score.
    Neutral,
    /// One suit is favorable.
    Suit(Suit),
    /// Either red suit is favorable.
    Red,
    /// Every suit except one is favorable.
    NonSuit(Suit),
    /// Any result that does not trigger Lightning is favorable.
    AvoidLightningHit,
}

impl JudgmentFavor {
    fn for_delayed(kind: CardKind) -> Self {
        match kind {
            CardKind::TrickLe => Self::Suit(Suit::Heart),
            CardKind::TrickBingliang => Self::Suit(Suit::Club),
            CardKind::TrickShandian => Self::AvoidLightningHit,
            _ => Self::Neutral,
        }
    }

    fn score(self, suit: Suit, rank: u8) -> i8 {
        match self {
            Self::Neutral => 0,
            Self::Suit(expected) => i8::from(suit == expected),
            Self::Red => i8::from(matches!(suit, Suit::Heart | Suit::Diamond)),
            Self::NonSuit(excluded) => i8::from(suit != excluded),
            Self::AvoidLightningHit => i8::from(!(suit == Suit::Spade && (2..=9).contains(&rank))),
        }
    }
}

/// Flip one judgment card from the draw pile into discard.
pub(crate) fn flip_judgment(
    tx: &mut Transaction,
    judged_player: PlayerId,
    favor: JudgmentFavor,
) -> Result<JudgmentResult, EffectError> {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw".into()))?;
    let empty = tx
        .state()
        .zones
        .get(draw)
        .map(|z| z.cards.is_empty())
        .unwrap_or(true);
    if empty {
        crate::effect::reshuffle_discard_into_draw(tx)?;
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw".into()))?;
    let cards = tx
        .state()
        .zones
        .get(draw)
        .map(|z| z.cards.clone())
        .ok_or_else(|| EffectError::Illegal("draw empty after reshuffle".into()))?;
    let mut top = cards
        .last()
        .copied()
        .ok_or_else(|| EffectError::Illegal("draw empty after reshuffle".into()))?;
    if crate::query::judgment_uses_best_draw_pile_card(tx.state(), judged_player) {
        let mut best_score = i8::MIN;
        for candidate in cards.iter().rev().copied() {
            let Some(card) = tx.state().cards.get(candidate) else {
                continue;
            };
            let suit = crate::query::effective_card_suit(tx.state(), judged_player, card);
            let score = favor.score(suit, card.rank);
            if score > best_score {
                top = candidate;
                best_score = score;
            }
        }
    }
    let card = tx
        .state()
        .cards
        .get(top)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("bad top card".into()))?;
    let suit = crate::query::effective_card_suit(tx.state(), judged_player, &card);
    let rank = card.rank;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![top],
            to: discard,
            from: Some(draw),
        }],
    )?;
    Ok(JudgmentResult {
        card: top,
        suit,
        rank,
    })
}

pub(crate) fn suit_tag(s: Suit) -> u32 {
    match s {
        Suit::None => 0,
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 3,
        Suit::Diamond => 4,
    }
}

pub(crate) fn suit_from_tag(t: u32) -> Suit {
    match t {
        0 => Suit::None,
        2 => Suit::Heart,
        3 => Suit::Club,
        4 => Suit::Diamond,
        _ => Suit::Spade,
    }
}

pub(crate) fn kind_from_tag(t: u32) -> CardKind {
    match t {
        30 => CardKind::TrickLe,
        31 => CardKind::TrickBingliang,
        32 => CardKind::TrickShandian,
        _ => CardKind::Other,
    }
}
