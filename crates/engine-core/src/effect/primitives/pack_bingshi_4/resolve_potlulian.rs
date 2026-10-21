//! Effect primitives for `resolve_potlulian` and related helpers.

use super::*;

pub(crate) fn resolve_potlulian(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let Some(record) = tx
                .state()
                .resolution
                .pending_card_resolutions
                .iter()
                .rev()
                .find(|record| {
                    record.source == player && record.card == card && record.source_was_hand
                })
                .cloned()
            else {
                return Ok(Vec::new());
            };
            let mut records = potlulian_pending(tx, skill);
            records.retain(|pending| pending.card != card);
            records.push(PotlulianPending {
                card,
                kind: record.kind,
                targets: record.targets,
            });
            if records.len() > 8 {
                records.remove(0);
            }
            set_potlulian_pending(tx, skill, &records);
            Ok(Vec::new())
        }
        1 => {
            let mut records = potlulian_pending(tx, skill);
            let Some(position) = records.iter().position(|pending| pending.card == card) else {
                return Ok(Vec::new());
            };
            let record = records.remove(position);
            set_potlulian_pending(tx, skill, &records);
            let Some(kind) = CardKind::from_tag(record.kind) else {
                return Ok(Vec::new());
            };
            let category = potlulian_category(kind);
            let has_same_category = tx.state().hand_cards(player).into_iter().any(|hand_card| {
                tx.state()
                    .cards
                    .get(hand_card)
                    .map(|card| crate::query::effective_card_kind(tx.state(), player, card))
                    .is_some_and(|kind| potlulian_category(kind) == category)
            });
            if has_same_category {
                return Ok(Vec::new());
            }
            let owner = tx
                .state()
                .players
                .get(player)
                .ok_or_else(|| EffectError::Illegal("戮连拥有者不存在".into()))?;
            let owner_hp = owner.hp;
            let owner_equip = tx
                .state()
                .zones
                .find(ZoneKind::Equip, player)
                .and_then(|zone| tx.state().zones.get(zone))
                .map_or(0, |zone| zone.cards.len());
            let bool1 = record.targets.iter().any(|target| {
                tx.state()
                    .players
                    .get(*target)
                    .is_some_and(|target| target.hp <= owner_hp)
            });
            let bool2 = record.targets.iter().any(|target| {
                tx.state().players.get(*target).is_some_and(|_| {
                    tx.state()
                        .zones
                        .find(ZoneKind::Equip, *target)
                        .and_then(|zone| tx.state().zones.get(zone))
                        .map_or(0, |zone| zone.cards.len())
                        <= owner_equip
                })
            });
            let mut events = Vec::new();
            if bool1 {
                for target in record.targets.iter().copied() {
                    let unlinked = tx.state().players.get(target).is_some_and(|target| {
                        target.alive
                            && !target.marks.iter().any(|(mark, count)| {
                                *mark == crate::content::MARK_CHAIN && *count > 0
                            })
                    });
                    if unlinked {
                        events.extend(add_mark(tx, target, crate::content::MARK_CHAIN, 1)?);
                    }
                }
            }
            if bool2 {
                events.extend(draw_cards(tx, player, 1)?);
            }
            if bool1 && bool2 {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolvePotlulian {
                        player,
                        card,
                        target: PlayerId::NONE,
                        stage: 5,
                        skill,
                    },
                );
            }
            Ok(events)
        }
        2 => {
            let min_hp = tx
                .state()
                .players
                .iter_live()
                .filter(|participant| participant.alive)
                .map(|participant| participant.hp)
                .min();
            if !tx.state().players.get(target).is_some_and(|participant| {
                participant.alive && min_hp.is_some_and(|min_hp| participant.hp > min_hp)
            }) {
                return Err(EffectError::Illegal("戮连火焰伤害目标已不合法".into()));
            }
            deal_damage(
                tx,
                player,
                target,
                1,
                false,
                rule_api::damage_nature::FIRE,
                false,
                CardId::NONE,
                0,
                0,
            )
        }
        5 => {
            let Some(min_hp) = tx
                .state()
                .players
                .iter_live()
                .filter(|target| target.alive)
                .map(|target| target.hp)
                .min()
            else {
                return Ok(Vec::new());
            };
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|target| target.hp > min_hp)
                })
                .collect();
            if options.is_empty() {
                Ok(Vec::new())
            } else if options.len() == 1 {
                resolve_potlulian(tx, player, card, options[0], 2, skill)
            } else {
                open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    options.into_iter().map(PlayerId::raw).collect(),
                    ChoiceKind::SelectTargets,
                    ContinuationKind::Custom,
                    vec![
                        POTLULIAN_CUSTOM_CHOICE,
                        player.raw(),
                        card.raw(),
                        skill.raw(),
                    ],
                )
            }
        }
        _ => Err(EffectError::ContentFault("bad 势孙綝戮连 stage".into())),
    }
}
