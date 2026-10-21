//! Command operations for `handle_view_as` and related helpers.

use super::*;

pub(crate) fn handle_view_as(
    tx: &mut Transaction,
    actor: PlayerId,
    skill: SkillId,
    inst: &crate::state::SkillInstance,
    views: &[ViewAsDef],
    targets: &[PlayerId],
    cards: &[CardId],
) -> Result<(), EffectError> {
    let card = cards[0];
    let c = tx
        .state()
        .cards
        .get(card)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("bad card".into()))?;
    // Pick the first rule matching every selected physical source.
    let token = tx
        .state()
        .turn
        .round
        .saturating_mul(16)
        .saturating_add(tx.state().turn.current.raw());
    let (rule_index, rule) = views
        .iter()
        .enumerate()
        .find(|(index, v)| {
            // once_per_turn_per_branch：本回合该规则位已置 → 跳过。
            !(v.once_per_turn_per_branch
                && inst.view_as_use_round == token
                && inst.view_as_used_branches & (1u32 << (*index as u32 % 32)) != 0)
                && view_as_sources_match(tx.state(), actor, v, cards)
                && window_ok(tx, actor, v)
        })
        .ok_or_else(|| EffectError::Illegal("no matching view_as for card".into()))?;

    if rule
        .uses_per_turn
        .is_some_and(|limit| inst.use_count >= u32::from(limit))
    {
        return Err(EffectError::Illegal(
            "view_as skill use limit reached".into(),
        ));
    }

    let as_kind = map_as_kind(&rule.as_kind)
        .ok_or_else(|| EffectError::Illegal(format!("bad as_kind {}", rule.as_kind)))?;
    let as_kind = match (as_kind, rule.nature.as_deref()) {
        (CardKind::Sha, Some("fire")) => CardKind::ShaFire,
        (CardKind::Sha, Some("thunder")) => CardKind::ShaThunder,
        (kind, None) => kind,
        _ => return Err(EffectError::ContentFault("bad view_as nature".into())),
    };
    let source_suit = crate::query::effective_card_suit(tx.state(), actor, &c);
    let force_unrespondable = rule.red_unrespondable
        && matches!(
            source_suit,
            engine_api::Suit::Heart | engine_api::Suit::Diamond
        );
    let ignore_use_count = rule.no_use_count
        || rule.black_no_use_count
            && matches!(
                source_suit,
                engine_api::Suit::Spade | engine_api::Suit::Club
            );
    let forced_unrespondable_targets: Vec<PlayerId> = if rule.target_lacks_source_suit_unrespondable
    {
        targets
            .iter()
            .copied()
            .filter(|target| {
                let Some(hand) = tx.state().zones.find(ZoneKind::Hand, *target) else {
                    return true;
                };
                !tx.state()
                    .zones
                    .get(hand)
                    .map_or(&[] as &[CardId], |zone| zone.cards.as_slice())
                    .iter()
                    .any(|card| {
                        tx.state().cards.get(*card).is_some_and(|card| {
                            crate::query::effective_card_suit(tx.state(), *target, card)
                                == source_suit
                        })
                    })
            })
            .collect()
    } else {
        Vec::new()
    };

    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, actor)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    let source_was_hand =
        c.zone == hand || crate::query::borrowed_hand_source(tx.state(), actor, card).is_some();
    for source in cards {
        let source_state = tx
            .state()
            .cards
            .get(*source)
            .cloned()
            .ok_or_else(|| EffectError::Illegal("bad view_as source".into()))?;
        if source_state.zone != hand
            && crate::query::borrowed_hand_source(tx.state(), actor, *source).is_none()
        {
            crate::effect::move_one_card(tx, *source, source_state.zone, hand, actor, false)?;
        }
    }

    if rule.once_per_turn_per_branch {
        let token = tx
            .state()
            .turn
            .round
            .saturating_mul(16)
            .saturating_add(tx.state().turn.current.raw());
        if let Some(s) = tx.state_mut().skills.get_mut(skill) {
            s.use_count = s.use_count.saturating_add(1);
            if s.view_as_use_round != token {
                s.view_as_use_round = token;
                s.view_as_used_branches = 0;
            }
            s.view_as_used_branches |= 1u32 << (rule_index as u32 % 32);
        }
    } else if let Some(s) = tx.state_mut().skills.get_mut(skill) {
        s.use_count = s.use_count.saturating_add(1);
    }

    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_CARD, card.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
    tx.state_mut().turn.set_flag(
        crate::state::turn_flag::VIEW_AS_LOGICAL_KIND,
        i32::from(as_kind.tag()),
    );
    let result = (|| {
        run_effects(
            tx,
            vec![EffectRequest::EmitRuleEvent {
                code: 120, // view_as used
                payload: vec![
                    skill.raw(),
                    inst.def.raw(),
                    actor.raw(),
                    card.raw(),
                    as_kind.tag() as u32,
                ],
            }],
        )?;
        let bonus = u32::from(
            cards.len() > 1
                && matches!(
                    rule.multi_card_bonus.as_deref(),
                    Some("damage_1" | "recover_1")
                ),
        );
        let no_damage_flag = if rule.refund_use_count_if_no_damage {
            tx.state_mut()
                .resolution
                .register_sha_no_damage_draw(card, engine_api::SkillId::NONE);
            rule_api::damage_card_flag::DRAW_IF_NO_DAMAGE
        } else {
            0
        };
        play_as_kind_with_extra_damage_flags(
            tx,
            actor,
            card,
            as_kind,
            targets,
            rule.ignore_distance,
            force_unrespondable,
            &forced_unrespondable_targets,
            ignore_use_count,
            source_was_hand,
            no_damage_flag,
            &cards[1..],
            bonus,
        )
    })();
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_CARD, 0);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, 0);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    result
}

pub(crate) fn view_as_sources_match(
    state: &crate::state::GameState,
    player: PlayerId,
    rule: &ViewAsDef,
    cards: &[CardId],
) -> bool {
    let (min_cards, max_cards) = view_as_card_count(state, player, rule);
    if cards.len() < usize::from(min_cards) || cards.len() > usize::from(max_cards) {
        return false;
    }
    if rule.cost.as_deref() == Some("dddzhijie") {
        if cards.len() != 2
            || !cards.iter().all(|card| {
                state
                    .skill_pile_cards(player, crate::effect::DDD_DONGCHA_DEF)
                    .contains(card)
            })
        {
            return false;
        }
        let same = crate::query::effective_card_color_mask(state, player, cards[0])
            == crate::query::effective_card_color_mask(state, player, cards[1]);
        if (rule.as_kind == "shan") != same {
            return false;
        }
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut suit = None;
    let mut distinct_suits = 0u8;
    cards.iter().all(|card_id| {
        if !seen.insert(*card_id) {
            return false;
        }
        if rule.cost.as_deref() != Some("dddzhijie")
            && !view_as_source_matches(state, player, rule, *card_id)
        {
            return false;
        }
        let Some(card) = state.cards.get(*card_id) else {
            return false;
        };
        let current = crate::query::effective_card_suit(state, player, card);
        if rule.cost.as_deref() == Some("dddtongyu") {
            let bit = match current {
                engine_api::Suit::Spade => 1,
                engine_api::Suit::Heart => 2,
                engine_api::Suit::Club => 4,
                engine_api::Suit::Diamond => 8,
                engine_api::Suit::None => 0,
            };
            if bit == 0 || distinct_suits & bit != 0 {
                return false;
            }
            distinct_suits |= bit;
        }
        if rule.same_suit && suit.is_some_and(|expected| expected != current) {
            return false;
        }
        suit = Some(current);
        true
    })
}

pub(crate) fn view_as_card_count(
    state: &crate::state::GameState,
    player: PlayerId,
    rule: &ViewAsDef,
) -> (u8, u8) {
    if rule.cost.as_deref() == Some("potqingyan") {
        let count = state
            .players
            .get(player)
            .into_iter()
            .flat_map(|owner| owner.skills.iter())
            .filter_map(|skill| state.skills.get(*skill))
            .find(|instance| instance.def == rule.skill_def)
            .map(|instance| {
                if instance.view_as_use_round == state.turn.round {
                    instance.view_as_used_branches
                } else {
                    0
                }
            })
            .unwrap_or(0)
            .saturating_add(1)
            .min(5) as u8;
        return (count, count);
    }
    if rule.owner_basic_uses_plus_one_cards {
        // 至少 1+本局基本牌使用数张，上限不封顶（截到 u8::MAX）。
        let min = state
            .lifetime_basic_card_uses
            .saturating_add(1)
            .min(u8::MAX as u32) as u8;
        return (min, u8::MAX);
    }
    if !rule.owner_hp_min_one_cards {
        return (rule.min_cards, rule.max_cards);
    }
    let count = state
        .players
        .get(player)
        .map(|participant| u8::try_from(participant.hp.max(1)).unwrap_or(u8::MAX))
        .unwrap_or(1);
    (count, count)
}

pub(crate) fn potqingyan_has_tracked_hand_card(
    state: &crate::state::GameState,
    player: PlayerId,
    instance: &crate::state::SkillInstance,
) -> bool {
    let hand = state.zones.find(ZoneKind::Hand, player);
    instance.state_blob.chunks_exact(4).any(|bytes| {
        state
            .cards
            .get(CardId::new(u32::from_le_bytes(bytes.try_into().unwrap())))
            .is_some_and(|card| Some(card.zone) == hand)
    })
}

pub(crate) fn view_as_source_matches(
    state: &crate::state::GameState,
    player: PlayerId,
    rule: &ViewAsDef,
    card_id: CardId,
) -> bool {
    state.cards.get(card_id).is_some_and(|card| {
        crate::query::view_as_source_zone_matches(state, player, rule.source_zone.as_deref(), card)
            && crate::query::view_as_filter_matches(state, player, &rule.filter, card)
    })
}

pub(crate) fn window_ok(tx: &Transaction, actor: PlayerId, v: &ViewAsDef) -> bool {
    match v.window.as_str() {
        "play_phase" => tx.state().turn.current == actor && tx.state().turn.phase == Phase::Play,
        "any" => true,
        _ => false,
    }
}

pub(crate) fn map_as_kind(s: &str) -> Option<CardKind> {
    match s {
        "sha" => Some(CardKind::Sha),
        "shan" => Some(CardKind::Shan),
        "tao" => Some(CardKind::Tao),
        "jiu" => Some(CardKind::Jiu),
        // Trick view-as (奇袭 → 过河拆桥, 国色 → 乐不思蜀, …).
        "guohe" => Some(CardKind::TrickGuohe),
        "shunshou" => Some(CardKind::TrickShunshou),
        "juedou" => Some(CardKind::TrickJuedou),
        "wuzhong" => Some(CardKind::TrickWuzhong),
        "le" => Some(CardKind::TrickLe),
        "bingliang" => Some(CardKind::TrickBingliang),
        "huogong" => Some(CardKind::TrickHuogong),
        "jiedao" => Some(CardKind::TrickJiedao),
        "tiesuo" => Some(CardKind::TrickTiesuo),
        "wanjian" => Some(CardKind::TrickWanjian),
        "wugu" => Some(CardKind::TrickWugu),
        "nanman" => Some(CardKind::TrickNanman),
        "chuqibuyi" => Some(CardKind::TrickChuqibuyi),
        _ => None,
    }
}
