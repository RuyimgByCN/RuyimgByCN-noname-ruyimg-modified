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

pub(crate) fn resolve_twshanghe(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("bad TW Shanghe card choice".into()));
    }
    let player = PlayerId::new(payload[0]);
    let giver = PlayerId::new(payload[1]);
    let skill = SkillId::new(payload[2]);
    let count = payload[4] as usize;
    let remaining: Vec<_> = payload
        .get(5..5 + count)
        .ok_or_else(|| EffectError::EngineFault("truncated TW Shanghe queue".into()))?
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    let card = CardId::new(selections[0]);
    let in_giver_zone = [ZoneKind::Hand, ZoneKind::Equip].into_iter().any(|kind| {
        tx.state().zones.find(kind, giver).is_some_and(|zone| {
            tx.state()
                .cards
                .get(card)
                .is_some_and(|card_state| card_state.owner == giver && card_state.zone == zone)
        })
    });
    if observer != giver
        || !in_giver_zone
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|owner| owner.alive)
    {
        return Err(EffectError::Illegal("TW Shanghe context changed".into()));
    }
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("TW Shanghe card missing".into()))?;
    let from = card_state.zone;
    let got_jiu = payload[3] != 0
        || crate::query::effective_card_kind(tx.state(), player, card_state) == CardKind::Jiu;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("TW Shanghe owner hand missing".into()))?;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveTwshanghe {
            player,
            stage: 1,
            remaining,
            got_jiu,
            skill,
        });
    run_effects(
        tx,
        vec![EffectRequest::GiveCards {
            cards: vec![card],
            to,
            from,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_twwangchang(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let mode = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = SkillId::new(if mode == 3 {
        *payload.get(2).unwrap_or(&0)
    } else {
        *payload.get(4).unwrap_or(&0)
    });
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal("TW Wangchang context changed".into()));
    }
    if mode == 3 {
        if cancelled || payload.len() != 3 || selections.is_empty() {
            return Err(EffectError::Illegal("bad TW Kaiji target choice".into()));
        }
        let max = 1 + tx.state().lifetime_dying_players.len();
        let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
        let mut unique = targets.clone();
        unique.sort_unstable_by_key(|target| target.raw());
        unique.dedup();
        if targets.len() > max
            || unique.len() != targets.len()
            || targets.iter().any(|target| {
                !tx.state()
                    .players
                    .get(*target)
                    .is_some_and(|participant| participant.alive)
            })
        {
            return Err(EffectError::Illegal("invalid TW Kaiji targets".into()));
        }
        return run_effects(
            tx,
            vec![EffectRequest::ResolveTwwangchang {
                player,
                target: PlayerId::NONE,
                card: CardId::NONE,
                selected_card: CardId::NONE,
                action: 0,
                stage: 1,
                targets,
                skill,
            }],
        )
        .map(|_| ());
    }
    if payload.len() != 5 {
        return Err(EffectError::EngineFault(
            "bad TW Shepan continuation".into(),
        ));
    }
    let target = PlayerId::new(payload[2]);
    let card = CardId::new(payload[3]);
    match mode {
        0 => {
            if cancelled || selections.len() != 1 || !matches!(selections[0], 0 | 1) {
                return Err(EffectError::Illegal("bad TW Shepan branch choice".into()));
            }
            if selections[0] == 0 {
                return run_effects(
                    tx,
                    vec![EffectRequest::ResolveTwwangchang {
                        player,
                        target,
                        card,
                        selected_card: CardId::NONE,
                        action: 1,
                        stage: 2,
                        targets: Vec::new(),
                        skill,
                    }],
                )
                .map(|_| ());
            }
            let options =
                crate::command::play::target_zone_card_tokens(tx, target, true, true, true);
            if options.is_empty() {
                return Err(EffectError::Illegal("TW Shepan target region empty".into()));
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwwangchang,
                vec![1, player.raw(), target.raw(), card.raw(), skill.raw()],
            )?;
            Ok(())
        }
        1 => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("bad TW Shepan card choice".into()));
            }
            let selected_card = CardId::new(selections[0]);
            if !crate::command::play::target_zone_card_tokens(tx, target, true, true, true)
                .contains(&selected_card.raw())
            {
                return Err(EffectError::Illegal("invalid TW Shepan target card".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwwangchang {
                    player,
                    target,
                    card,
                    selected_card,
                    action: 1,
                    stage: 4,
                    targets: Vec::new(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 => {
            if cancelled {
                return Ok(());
            }
            if selections != [1] {
                return Err(EffectError::Illegal("bad TW Shepan nullify choice".into()));
            }
            crate::command::play::exclude_saved_card_target(tx, card, player)
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Wangchang continuation".into(),
        )),
    }
}

pub(crate) fn resolve_twcaozhao(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let mode = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = SkillId::new(*payload.last().unwrap_or(&0));
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::Illegal("TW Caozhao context changed".into()));
    }
    match mode {
        0 => {
            if observer != player || cancelled || payload.len() != 5 || selections.len() != 1 {
                return Err(EffectError::Illegal("bad TW Fuzuan target choice".into()));
            }
            let target = PlayerId::new(selections[0]);
            let defs = crate::query::conversion_skill_defs(tx.state(), target);
            if defs.is_empty() {
                return Err(EffectError::Illegal(
                    "TW Fuzuan target has no conversion skill".into(),
                ));
            }
            if defs.len() == 1 {
                return run_effects(
                    tx,
                    vec![EffectRequest::ResolveTwcaozhao {
                        player,
                        target,
                        source: PlayerId::NONE,
                        card: CardId::NONE,
                        selected_skill_def: defs[0].raw(),
                        twfuzuan_skill_def: payload[2],
                        twfeifu_skill_def: payload[3],
                        action: 0,
                        stage: 1,
                        skill,
                    }],
                )
                .map(|_| ());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                defs.into_iter().map(engine_api::SkillDefId::raw).collect(),
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwcaozhao,
                vec![
                    1,
                    player.raw(),
                    target.raw(),
                    payload[2],
                    payload[3],
                    skill.raw(),
                ],
            )?;
            Ok(())
        }
        1 => {
            if observer != player || cancelled || payload.len() != 6 || selections.len() != 1 {
                return Err(EffectError::Illegal("bad TW Fuzuan skill choice".into()));
            }
            let target = PlayerId::new(payload[2]);
            if !crate::query::conversion_skill_defs(tx.state(), target)
                .iter()
                .any(|skill| skill.raw() == selections[0])
            {
                return Err(EffectError::Illegal(
                    "invalid TW Fuzuan conversion skill".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwcaozhao {
                    player,
                    target,
                    source: PlayerId::NONE,
                    card: CardId::NONE,
                    selected_skill_def: selections[0],
                    twfuzuan_skill_def: payload[3],
                    twfeifu_skill_def: payload[4],
                    action: 0,
                    stage: 1,
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 => {
            if observer != player || payload.len() != 5 {
                return Err(EffectError::Illegal("bad TW Chongqi target choice".into()));
            }
            if cancelled {
                return if selections.is_empty() {
                    Ok(())
                } else {
                    Err(EffectError::Illegal(
                        "cancelled TW Chongqi has a target".into(),
                    ))
                };
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("TW Chongqi needs one target".into()));
            }
            let target = PlayerId::new(selections[0]);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid TW Chongqi target".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::ChangeMaxHp { player, delta: -1 },
                    EffectRequest::AddSkill {
                        player: target,
                        skill_def: engine_api::SkillDefId::new(payload[2]),
                    },
                ],
            )
            .map(|_| ())
        }
        3 => {
            if payload.len() != 6 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("bad TW Feifu card choice".into()));
            }
            let giver = PlayerId::new(payload[2]);
            let source = PlayerId::new(payload[3]);
            if observer != giver
                || !tx
                    .state()
                    .players
                    .get(source)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("TW Feifu participants changed".into()));
            }
            let card = CardId::new(selections[0]);
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|state| state.zone)
                .ok_or_else(|| EffectError::Illegal("TW Feifu selected card missing".into()))?;
            if ![ZoneKind::Hand, ZoneKind::Equip]
                .into_iter()
                .any(|kind| tx.state().zones.find(kind, giver) == Some(from))
            {
                return Err(EffectError::Illegal("TW Feifu card left giver".into()));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, source)
                .ok_or_else(|| {
                    EffectError::EngineFault("TW Feifu recipient hand missing".into())
                })?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwcaozhao {
                    player,
                    target: giver,
                    source,
                    card,
                    selected_skill_def: 0,
                    twfuzuan_skill_def: 0,
                    twfeifu_skill_def: 0,
                    action: 2,
                    stage: 1,
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: vec![card],
                    to,
                    from,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Caozhao continuation".into(),
        )),
    }
}

pub(crate) fn resolve_twguohuai(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 6 || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("bad TW Guohuai choice".into()));
    }
    let mode = payload[0];
    let player = PlayerId::new(payload[1]);
    let source = PlayerId::new(payload[2]);
    let mark_def = payload[3];
    let helper_skill_def = payload[4];
    let skill = SkillId::new(payload[5]);
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal("TW Guohuai context changed".into()));
    }
    match (mode, selections[0]) {
        (0, 1) => run_effects(
            tx,
            vec![
                EffectRequest::RemoveMark {
                    player,
                    mark: engine_api::MarkDefId::new(mark_def),
                    amount: 1,
                },
                EffectRequest::AddSkillUntilTurnEnd {
                    player: source,
                    skill_def: engine_api::SkillDefId::new(helper_skill_def),
                },
            ],
        )
        .map(|_| ()),
        (0, 2) => {
            let cards = crate::query::discardable_hand_equip_cards(tx.state(), source);
            if cards.is_empty() {
                return Err(EffectError::Illegal(
                    "TW Yuzhang source has no cards".into(),
                ));
            }
            let count = cards.len().min(2) as u8;
            run_effects(
                tx,
                vec![EffectRequest::RemoveMark {
                    player,
                    mark: engine_api::MarkDefId::new(mark_def),
                    amount: 1,
                }],
            )?;
            open_choice(
                tx,
                source,
                count,
                count,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwguohuai,
                vec![
                    1,
                    player.raw(),
                    source.raw(),
                    mark_def,
                    helper_skill_def,
                    skill.raw(),
                ],
            )?;
            Ok(())
        }
        (1, _) => {
            let allowed: std::collections::BTreeSet<_> =
                crate::query::discardable_hand_equip_cards(tx.state(), source)
                    .into_iter()
                    .collect();
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            if cards.len() != allowed.len().min(2)
                || cards.iter().any(|card| !allowed.contains(card))
            {
                return Err(EffectError::Illegal("invalid TW Yuzhang discard".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: source,
                    cards,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal("invalid TW Guohuai branch".into())),
    }
}

pub(crate) fn resolve_twchengpu_custom(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::TWCHENGPU_CUSTOM_CHOICE)
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("bad TW Chengpu choice".into()));
    }
    let mode = *payload.get(1).unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = SkillId::new(*payload.last().unwrap_or(&0));
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::Illegal("TW Chengpu context changed".into()));
    }
    match mode {
        0 => {
            if observer != player || payload.len() != 5 {
                return Err(EffectError::Illegal("bad TW Chunlao target choice".into()));
            }
            let target = PlayerId::new(selections[0]);
            let cards = crate::effect::twchengpu_field_cards(tx.state(), target);
            if cards.is_empty() {
                return Err(EffectError::Illegal(
                    "TW Chunlao target has no field card".into(),
                ));
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    crate::effect::TWCHENGPU_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    payload[3],
                    skill.raw(),
                ],
            )?;
            Ok(())
        }
        1 => {
            if observer != player || payload.len() != 6 {
                return Err(EffectError::Illegal("bad TW Chunlao card choice".into()));
            }
            let target = PlayerId::new(payload[3]);
            let skill_def = payload[4];
            let card = CardId::new(selections[0]);
            if !crate::effect::twchengpu_field_cards(tx.state(), target).contains(&card) {
                return Err(EffectError::Illegal("TW Chunlao card moved".into()));
            }
            let from = tx.state().cards.get(card).unwrap().zone;
            let to = crate::effect::ensure_skill_pile_zone(tx, target, skill_def)?;
            crate::effect::move_cards_with_reason(
                tx,
                vec![card],
                to,
                Some(from),
                engine_api::CardMoveReason::Give,
            )?;
            Ok(())
        }
        2 => {
            if payload.len() != 7 {
                return Err(EffectError::Illegal("bad TW Chunlao gift choice".into()));
            }
            let source = PlayerId::new(payload[3]);
            let sha = CardId::new(payload[4]);
            let card = CardId::new(selections[0]);
            if observer != source
                || !crate::effect::twchengpu_hand_equip_cards(tx.state(), source).contains(&card)
            {
                return Err(EffectError::Illegal("TW Chunlao gift card moved".into()));
            }
            let from = tx.state().cards.get(card).unwrap().zone;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("TW Chunlao owner hand missing".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwchengpu {
                    player,
                    target: source,
                    source,
                    card: sha,
                    chunlao_skill_def: payload[5],
                    action: 2,
                    stage: 1,
                    selected_card: card,
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: vec![card],
                    to,
                    from,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "unknown TW Chengpu custom choice".into(),
        )),
    }
}

pub(crate) fn resolve_twcaoang_custom(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 6
        || payload.first() != Some(&crate::effect::TWCAOANG_CUSTOM_CHOICE)
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("bad TW Caoang horse choice".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let helper_skill_def = payload[4];
    let skill = SkillId::new(payload[5]);
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|target| target.alive)
    {
        return Err(EffectError::Illegal(
            "TW Caoang horse context changed".into(),
        ));
    }
    if stage == 0 {
        if cancelled || selections == [0] {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .clear_twxiaolian_redirect();
            return Ok(());
        }
        if selections != [1] {
            return Err(EffectError::Illegal("bad TW Caoang confirmation".into()));
        }
        let options = crate::effect::twchengpu_hand_equip_cards(tx.state(), player);
        let mut next = payload.to_vec();
        next[1] = 1;
        return crate::effect::open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.into_iter().map(CardId::raw).collect(),
            engine_api::ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            next,
        )
        .map(|_| ());
    }
    if stage != 1 || cancelled {
        return Err(EffectError::Illegal("bad TW Caoang horse stage".into()));
    }
    let card = CardId::new(selections[0]);
    if !crate::effect::twchengpu_hand_equip_cards(tx.state(), player).contains(&card) {
        return Err(EffectError::Illegal("TW Caoang horse card moved".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: engine_api::SkillDefId::new(helper_skill_def),
        }],
    )?;
    let from = tx.state().cards.get(card).unwrap().zone;
    let to = crate::effect::ensure_skill_pile_zone(tx, target, helper_skill_def)?;
    crate::effect::move_cards_with_reason(
        tx,
        vec![card],
        to,
        Some(from),
        engine_api::CardMoveReason::Unspecified,
    )?;
    tx.state_mut()
        .skills
        .get_mut(skill)
        .unwrap()
        .clear_twxiaolian_redirect();
    Ok(())
}
