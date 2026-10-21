//! Query operations for `target_legal_explain` and related helpers.

use super::*;

pub(crate) fn target_legal_explain(
    state: &GameState,
    user: PlayerId,
    card: engine_api::CardId,
    target: PlayerId,
    ctx: &mut QueryContext,
) -> QueryExplanation {
    let Some(c) = state.cards.get(card) else {
        return err(QueryErrorCode::InvalidId);
    };
    let Some(t) = state.players.get(target) else {
        return err(QueryErrorCode::InvalidId);
    };
    if !t.alive {
        return QueryExplanation {
            response: QueryResponse::Bool(false),
            base: None,
            sources: vec![],
        };
    }
    // 雄乱（张绣）：本回合使用者只能对限定目标使用牌。
    let restricted = state
        .turn
        .flag(crate::state::turn_flag::xiongluan_target_by_player(user))
        .max(0) as u32;
    if restricted != 0 && target.raw() != restricted {
        return QueryExplanation {
            response: QueryResponse::Bool(false),
            base: None,
            sources: vec![],
        };
    }
    let kind = effective_card_kind(state, user, c);
    if card_target_cancelled_for_logical_kind(state, user, target, c, kind) {
        return QueryExplanation {
            response: QueryResponse::Bool(false),
            base: None,
            sources: vec![],
        };
    }
    match kind {
        CardKind::Sha
        | CardKind::ShaFire
        | CardKind::ShaThunder
        | CardKind::TrickGuohe
        | CardKind::TrickShunshou
        | CardKind::TrickJuedou
        | CardKind::TrickHuogong => {
            if target == user && kind.is_sha() {
                return QueryExplanation {
                    response: QueryResponse::Bool(false),
                    base: None,
                    sources: vec![],
                };
            }
            if target == user
                && matches!(
                    kind,
                    CardKind::TrickGuohe | CardKind::TrickShunshou | CardKind::TrickJuedou
                )
            {
                return QueryExplanation {
                    response: QueryResponse::Bool(false),
                    base: None,
                    sources: vec![],
                };
            }
            // 火攻可指定自己（有手牌时）
            if kind.is_sha() {
                if card_distance_unlimited_for_target(state, user, card, target) {
                    return QueryExplanation {
                        response: QueryResponse::Bool(true),
                        base: None,
                        sources: vec![],
                    };
                }
                let dist = query(
                    state,
                    &QueryRequest::Distance {
                        from: user,
                        to: target,
                    },
                    ctx,
                );
                let range = query(state, &QueryRequest::AttackRange { player: user }, ctx);
                let ok = matches!(
                    (dist, range),
                    (QueryResponse::Int(d), QueryResponse::Int(r)) if d <= r
                );
                return QueryExplanation {
                    response: QueryResponse::Bool(ok),
                    base: None,
                    sources: vec![],
                };
            }
            if kind == CardKind::TrickShunshou {
                return QueryExplanation {
                    response: QueryResponse::Bool(card_target_within_distance(
                        state, user, card, target, 1,
                    )),
                    base: None,
                    sources: vec![],
                };
            }
            if kind == CardKind::TrickHuogong {
                let alive = state.players.get(target).map(|p| p.alive).unwrap_or(false);
                let has_hand = !state.hand_cards(target).is_empty();
                return QueryExplanation {
                    response: QueryResponse::Bool(alive && has_hand),
                    base: None,
                    sources: vec![],
                };
            }
            QueryExplanation {
                response: QueryResponse::Bool(
                    state.players.get(target).map(|p| p.alive).unwrap_or(false),
                ),
                base: None,
                sources: vec![],
            }
        }
        CardKind::TrickLe | CardKind::TrickBingliang => {
            let ok = target != user
                && t.alive
                && (kind != CardKind::TrickBingliang
                    || card_target_within_distance(state, user, card, target, 1));
            QueryExplanation {
                response: QueryResponse::Bool(ok),
                base: None,
                sources: vec![],
            }
        }
        CardKind::Tao | CardKind::Jiu => QueryExplanation {
            response: QueryResponse::Bool(target == user),
            base: None,
            sources: vec![],
        },
        _ => QueryExplanation {
            response: QueryResponse::Bool(true),
            base: None,
            sources: vec![],
        },
    }
}

pub(crate) fn skill_uses_explain(
    state: &GameState,
    skill: engine_api::SkillId,
) -> QueryExplanation {
    let Some(s) = state.skills.get(skill) else {
        return err(QueryErrorCode::InvalidId);
    };
    // Phase 3: unlimited unless state_blob encodes max; report 99 - use_count
    let remaining = 99i32.saturating_sub(s.use_count as i32);
    QueryExplanation {
        response: QueryResponse::Int(remaining.max(0)),
        base: Some(99),
        sources: vec![],
    }
}

pub(crate) fn card_attr_explain(
    state: &GameState,
    card: engine_api::CardId,
    attr: u32,
) -> QueryExplanation {
    let Some(c) = state.cards.get(card) else {
        return err(QueryErrorCode::InvalidId);
    };
    // attr 0 = kind tag, 1 = rank, 2 = suit tag
    let base = match attr {
        0 => c.kind.tag() as i32,
        1 => c.rank as i32,
        2 => match c.suit {
            engine_api::Suit::None => 0,
            engine_api::Suit::Spade => 1,
            engine_api::Suit::Heart => 2,
            engine_api::Suit::Club => 3,
            engine_api::Suit::Diamond => 4,
        },
        _ => 0,
    };
    let mods = state.modifiers.for_target(ModifierTarget::CardAttr(attr));
    // Card mods: no owner filter
    let (value, sources, _) = apply_int_modifiers(base, &mods, None);
    QueryExplanation {
        response: QueryResponse::Int(value),
        base: Some(base),
        sources,
    }
}

pub(crate) fn err(code: QueryErrorCode) -> QueryExplanation {
    QueryExplanation {
        response: QueryResponse::Error { code },
        base: None,
        sources: vec![],
    }
}
