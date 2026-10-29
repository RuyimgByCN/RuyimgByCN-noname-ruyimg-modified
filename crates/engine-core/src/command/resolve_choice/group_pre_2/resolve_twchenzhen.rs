//! Command operations for `resolve_twchenzhen` and related helpers.

use super::*;

pub(crate) fn resolve_twchenzhen(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) => {
            let [_, player, helper, skill] = payload else {
                return Err(EffectError::EngineFault("bad TW Muyue kind payload".into()));
            };
            if cancelled || observer.raw() != *player {
                return Err(EffectError::Illegal(
                    "TW Muyue kind cannot be cancelled".into(),
                ));
            }
            let [kind] = selections else {
                return Err(EffectError::Illegal(
                    "TW Muyue requires one card kind".into(),
                ));
            };
            let kind = u8::try_from(*kind)
                .ok()
                .and_then(CardKind::from_tag)
                .filter(|kind| kind.is_basic() || kind.is_trick())
                .ok_or_else(|| EffectError::Illegal("invalid TW Muyue kind".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwchenzhen {
                    player: PlayerId::new(*player),
                    target: PlayerId::NONE,
                    helper_skill_def: *helper,
                    record_skill_def: 0,
                    action: 0,
                    stage: 1,
                    card_kind: kind.tag(),
                    card: CardId::NONE,
                    branch: 0,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() < 6 || cancelled || observer.raw() != payload[1] {
                return Err(EffectError::Illegal("bad TW Muyue discard choice".into()));
            }
            let [card] = selections else {
                return Err(EffectError::Illegal("TW Muyue requires one discard".into()));
            };
            if !payload[5..].contains(card) {
                return Err(EffectError::Illegal("invalid TW Muyue discard".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwchenzhen {
                    player: PlayerId::new(payload[1]),
                    target: PlayerId::NONE,
                    helper_skill_def: payload[2],
                    record_skill_def: 0,
                    action: 0,
                    stage: 2,
                    card_kind: payload[4] as u8,
                    card: CardId::new(*card),
                    branch: 0,
                    skill: SkillId::new(payload[3]),
                }],
            )
            .map(|_| ())
        }
        Some(3) => {
            let [_, player, helper, _skill, kind, matching] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Muyue target payload".into(),
                ));
            };
            if cancelled || observer.raw() != *player {
                return Err(EffectError::Illegal(
                    "TW Muyue target cannot be cancelled".into(),
                ));
            }
            let [target] = selections else {
                return Err(EffectError::Illegal("TW Muyue requires one target".into()));
            };
            let target = PlayerId::new(*target);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid TW Muyue target".into()));
            }
            let mut effects = vec![EffectRequest::GainCardByKind {
                player: target,
                kind: *kind as u8,
            }];
            if *matching != 0 {
                effects.push(EffectRequest::AddSkill {
                    player: PlayerId::new(*player),
                    skill_def: SkillDefId::new(*helper),
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        Some(10) => {
            let [_, owner, effect, record, _skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Chayi target payload".into(),
                ));
            };
            if observer.raw() != *owner {
                return Err(EffectError::Illegal("wrong TW Chayi chooser".into()));
            }
            if cancelled {
                return Ok(());
            }
            let [target] = selections else {
                return Err(EffectError::Illegal("TW Chayi requires one target".into()));
            };
            let target = PlayerId::new(*target);
            if target.raw() == *owner
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid TW Chayi target".into()));
            }
            if tx.state().hand_cards(target).is_empty() {
                return crate::effect::apply_twchayi_initial(tx, target, 1, *effect, *record)
                    .map(|_| ());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwchenzhen,
                vec![11, *owner, target.raw(), *effect, *record],
            )?;
            Ok(())
        }
        Some(11) => {
            let [_, _owner, target, effect, record] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Chayi branch payload".into(),
                ));
            };
            if cancelled || observer.raw() != *target {
                return Err(EffectError::Illegal(
                    "TW Chayi branch cannot be cancelled".into(),
                ));
            }
            let [branch @ 0..=1] = selections else {
                return Err(EffectError::Illegal("invalid TW Chayi branch".into()));
            };
            crate::effect::apply_twchayi_initial(
                tx,
                PlayerId::new(*target),
                *branch as u8,
                *effect,
                *record,
            )
            .map(|_| ())
        }
        Some(12) => {
            if payload.len() < 6 || cancelled || observer.raw() != payload[1] {
                return Err(EffectError::Illegal("bad TW Chayi discard payload".into()));
            }
            let player = PlayerId::new(payload[1]);
            let skill = SkillId::new(payload[2]);
            let mark = engine_api::MarkDefId::new(payload[3]);
            let need = payload[4] as usize;
            let mut unique = std::collections::BTreeSet::new();
            if selections.len() != need
                || selections
                    .iter()
                    .any(|card| !payload[5..].contains(card) || !unique.insert(*card))
            {
                return Err(EffectError::Illegal("invalid TW Chayi discard".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::RemoveSkill { skill },
                    EffectRequest::RemoveMark {
                        player,
                        mark,
                        amount: need as u32,
                    },
                    EffectRequest::DiscardCards {
                        player,
                        cards: selections.iter().copied().map(CardId::new).collect(),
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Chenzhen choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_twqingxi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || cancelled || observer.raw() != payload[2] {
        return Err(EffectError::Illegal("bad TW Qingxi choice".into()));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let card = CardId::new(payload[3]);
    let skill = SkillId::new(payload[4]);
    match payload[0] {
        0 => {
            let [branch] = selections else {
                return Err(EffectError::Illegal("TW Qingxi requires one branch".into()));
            };
            if *branch > 1 {
                return Err(EffectError::Illegal("invalid TW Qingxi branch".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwqingxi {
                    player,
                    target,
                    card,
                    stage: if *branch == 0 { 1 } else { 2 },
                    count: 0,
                    cards: Vec::new(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() < 6 {
                return Err(EffectError::EngineFault(
                    "bad TW Qingxi discard payload".into(),
                ));
            }
            let need = usize::from(payload[5].min(u8::MAX.into()) as u8)
                .min(payload.len().saturating_sub(6));
            if selections.len() != need
                || selections.iter().any(|card| !payload[6..].contains(card))
            {
                return Err(EffectError::Illegal(
                    "invalid TW Qingxi equipment choice".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwqingxi {
                    player,
                    target,
                    card,
                    stage: 6,
                    count: payload[5] as u8,
                    cards: selections.iter().copied().map(CardId::new).collect(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Qingxi choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_twzaoli(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 || cancelled || observer.raw() != payload[0] {
        return Err(EffectError::Illegal("bad TW Zaoli choice".into()));
    }
    let mandatory_len = payload[2] as usize;
    let split = 3usize
        .checked_add(mandatory_len)
        .filter(|split| *split <= payload.len())
        .ok_or_else(|| EffectError::EngineFault("bad TW Zaoli payload".into()))?;
    if selections.is_empty()
        || selections
            .iter()
            .any(|card| !payload[split..].contains(card))
    {
        return Err(EffectError::Illegal(
            "invalid TW Zaoli discard choice".into(),
        ));
    }
    let cards = payload[3..split]
        .iter()
        .chain(selections)
        .copied()
        .map(CardId::new)
        .collect();
    run_effects(
        tx,
        vec![EffectRequest::ResolveTwzaoli {
            player: PlayerId::new(payload[0]),
            stage: 1,
            cards,
            slots: Vec::new(),
            count: 0,
            skill: SkillId::new(payload[1]),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_twzhengjian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled {
        return Err(EffectError::Illegal(
            "TW Zhengjian choice cannot be cancelled".into(),
        ));
    }
    match payload.first().copied() {
        Some(0) => {
            let [_, player, _helper, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Zhengjian setup payload".into(),
                ));
            };
            let [branch @ 0..=1] = selections else {
                return Err(EffectError::Illegal("invalid TW Zhengjian branch".into()));
            };
            if observer.raw() != *player {
                return Err(EffectError::Illegal("wrong TW Zhengjian chooser".into()));
            }
            tx.state_mut()
                .skills
                .get_mut(SkillId::new(*skill))
                .ok_or_else(|| EffectError::Illegal("TW Zhengjian skill disappeared".into()))?
                .set_twzhengjian_state(*branch as u8, false, Vec::new());
            Ok(())
        }
        Some(1) => {
            if payload.len() < 6 || observer.raw() != payload[2] {
                return Err(EffectError::Illegal("bad TW Zhengjian give choice".into()));
            }
            let [card] = selections else {
                return Err(EffectError::Illegal(
                    "TW Zhengjian requires one card".into(),
                ));
            };
            if !payload[5..].contains(card) {
                return Err(EffectError::Illegal("invalid TW Zhengjian card".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwzhengjian {
                    player: PlayerId::new(payload[1]),
                    target: PlayerId::new(payload[2]),
                    helper_skill_def: payload[3],
                    action: 1,
                    stage: 1,
                    card: CardId::new(*card),
                    branch: 0,
                    skill: SkillId::new(payload[4]),
                }],
            )
            .map(|_| ())
        }
        Some(2) => {
            let [_, player, target, helper, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Zhengjian damage payload".into(),
                ));
            };
            let [choice @ 0..=3] = selections else {
                return Err(EffectError::Illegal(
                    "invalid TW Zhengjian damage choice".into(),
                ));
            };
            if observer.raw() != *player {
                return Err(EffectError::Illegal("wrong TW Zhengjian chooser".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwzhengjian {
                    player: PlayerId::new(*player),
                    target: PlayerId::new(*target),
                    helper_skill_def: *helper,
                    action: 1,
                    stage: 3,
                    card: CardId::NONE,
                    branch: *choice as u8,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        Some(3) => {
            let [_, player, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Zhengjian switch payload".into(),
                ));
            };
            let [choice @ 0..=1] = selections else {
                return Err(EffectError::Illegal("invalid TW Zhengjian switch".into()));
            };
            if observer.raw() != *player {
                return Err(EffectError::Illegal("wrong TW Zhengjian chooser".into()));
            }
            if *choice != 0 {
                let instance = tx
                    .state_mut()
                    .skills
                    .get_mut(SkillId::new(*skill))
                    .ok_or_else(|| EffectError::Illegal("TW Zhengjian skill disappeared".into()))?;
                let (branch, awakened, contributors) = instance.twzhengjian_state();
                instance.set_twzhengjian_state(1 - branch.unwrap_or(0), awakened, contributors);
            }
            Ok(())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Zhengjian choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_twtianyu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled {
        return Err(EffectError::Illegal(
            "TW Tianyu choice cannot be cancelled".into(),
        ));
    }
    match payload.first().copied() {
        Some(0) => {
            let [_, player, target, helper, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Zhenxi branch payload".into(),
                ));
            };
            let [branch @ 1..=3] = selections else {
                return Err(EffectError::Illegal("invalid TW Zhenxi branch".into()));
            };
            if observer.raw() != *player {
                return Err(EffectError::Illegal("wrong TW Zhenxi chooser".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwtianyu {
                    player: PlayerId::new(*player),
                    target: PlayerId::new(*target),
                    helper_skill_def: *helper,
                    action: 0,
                    stage: 1,
                    card: CardId::NONE,
                    branch: *branch as u8,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() < 7 || observer.raw() != payload[1] || selections.is_empty() {
                return Err(EffectError::Illegal("bad TW Zhenxi discard choice".into()));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = SkillId::new(payload[4]);
            let branch = payload[5] as u8;
            let legal = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::Hand,
            );
            if selections
                .iter()
                .any(|card| !payload[6..].contains(card) || !legal.contains(&CardId::new(*card)))
            {
                return Err(EffectError::Illegal(
                    "invalid TW Zhenxi discard card".into(),
                ));
            }
            if branch & 2 != 0 {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveTwtianyu {
                        player,
                        target,
                        helper_skill_def: payload[3],
                        action: 0,
                        stage: 2,
                        card: CardId::NONE,
                        branch,
                        skill,
                    },
                );
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: selections.iter().copied().map(CardId::new).collect(),
                }],
            )
            .map(|_| ())
        }
        Some(2) => {
            let [_, player, source, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Zhenxi field-card payload".into(),
                ));
            };
            let [card] = selections else {
                return Err(EffectError::Illegal(
                    "TW Zhenxi requires one field card".into(),
                ));
            };
            if observer.raw() != *player
                || !crate::query::movable_field_cards_from(tx.state(), PlayerId::new(*source))
                    .contains(&CardId::new(*card))
            {
                return Err(EffectError::Illegal("invalid TW Zhenxi field card".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwtianyu {
                    player: PlayerId::new(*player),
                    target: PlayerId::new(*source),
                    helper_skill_def: 0,
                    action: 0,
                    stage: 3,
                    card: CardId::new(*card),
                    branch: 2,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        Some(3) => {
            let [_, player, source, card, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Zhenxi destination payload".into(),
                ));
            };
            let [destination] = selections else {
                return Err(EffectError::Illegal(
                    "TW Zhenxi requires one destination".into(),
                ));
            };
            let player = PlayerId::new(*player);
            let source = PlayerId::new(*source);
            let card = CardId::new(*card);
            let destination = PlayerId::new(*destination);
            if observer != player
                || !tx
                    .state()
                    .skills
                    .get(SkillId::new(*skill))
                    .is_some_and(|instance| {
                        instance.owner == player && tx.state().skill_effectively_enabled(instance)
                    })
                || !crate::query::movable_field_cards_from(tx.state(), source).contains(&card)
                || !crate::query::field_card_move_destinations(tx.state(), card)
                    .contains(&destination)
            {
                return Err(EffectError::Illegal("invalid TW Zhenxi destination".into()));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("TW Zhenxi field card missing".into()))?;
            let kind = tx
                .state()
                .zones
                .get(from)
                .map(|zone| zone.kind)
                .ok_or_else(|| EffectError::Illegal("TW Zhenxi field zone missing".into()))?;
            let to =
                tx.state().zones.find(kind, destination).ok_or_else(|| {
                    EffectError::Illegal("TW Zhenxi destination zone missing".into())
                })?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to,
                    from: Some(from),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "bad TW Tianyu choice payload".into(),
        )),
    }
}

pub(crate) fn resolve_twwujing(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    if cancelled {
        return Ok(());
    }
    match payload.first().copied() {
        Some(0) => {
            if payload.len() != 3 || observer != player || selections.is_empty() {
                return Err(EffectError::Illegal("bad TW Congji card choice".into()));
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("shared discard pile missing".into()))?;
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            if cards.iter().any(|card| {
                tx.state().cards.get(*card).is_none_or(|card_state| {
                    card_state.zone != discard
                        || crate::query::effective_card_color_mask(tx.state(), player, *card)
                            != 0b01
                })
            }) {
                return Err(EffectError::Illegal("TW Congji card left discard".into()));
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player)
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(());
            }
            let mut next = vec![1, player.raw(), payload[2], cards.len() as u32];
            next.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwwujing,
                next,
            )?;
            Ok(())
        }
        Some(1) => {
            if payload.len() < 5 || observer != player {
                return Err(EffectError::Illegal("bad TW Congji target choice".into()));
            }
            let [target] = selections else {
                return Err(EffectError::Illegal(
                    "TW Congji requires one recipient".into(),
                ));
            };
            let target = PlayerId::new(*target);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid TW Congji recipient".into()));
            }
            let count = payload[3] as usize;
            let cards: Vec<_> = payload
                .get(4..4 + count)
                .ok_or_else(|| EffectError::EngineFault("truncated TW Congji cards".into()))?
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("shared discard pile missing".into()))?;
            if cards.iter().any(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_none_or(|card_state| card_state.zone != discard)
            }) {
                return Err(EffectError::Illegal("TW Congji card left discard".into()));
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("recipient hand missing".into()))?;
            crate::effect::move_cards(tx, cards, hand, Some(discard)).map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "bad TW Wujing choice payload".into(),
        )),
    }
}
