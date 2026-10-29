//! Query operations for `bootstrap_modifier_defs` and related helpers.

use super::*;

pub(crate) fn resolved_value(state: &GameState, modifier: &ActiveModifier) -> i32 {
    resolved_value_with_context(state, modifier, None, None)
}

pub(crate) fn resolved_value_with_context(
    state: &GameState,
    modifier: &ActiveModifier,
    queried: Option<PlayerId>,
    other: Option<PlayerId>,
) -> i32 {
    let source = match modifier.value_source {
        None => return modifier.value,
        Some(source) => source,
    };
    let player = state.players.get(modifier.owner);
    let value = match source {
        ModifierValueSource::OwnerHp => player.map(|player| player.hp.max(0)).unwrap_or(0),
        ModifierValueSource::OwnerMaxHp => player
            .map(|player| player.max_hp.min(i32::MAX as u32) as i32)
            .unwrap_or(0),
        ModifierValueSource::OwnerHpMinusOne => player
            .map(|player| player.hp.saturating_sub(1).max(0))
            .unwrap_or(0),
        ModifierValueSource::OwnerLostHp => player
            .map(|player| (player.max_hp.min(i32::MAX as u32) as i32 - player.hp).max(0))
            .unwrap_or(0),
        ModifierValueSource::OwnerLostHpMinusOne => player
            .map(|player| {
                (player.max_hp.min(i32::MAX as u32) as i32)
                    .saturating_sub(player.hp)
                    .saturating_sub(1)
                    .max(0)
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerLostHpMinOne => player
            .map(|player| (player.max_hp.min(i32::MAX as u32) as i32 - player.hp).max(1))
            .unwrap_or(1),
        ModifierValueSource::OwnerSkillMark => player
            .and_then(|player| {
                player
                    .marks
                    .iter()
                    .find(|(mark, _)| *mark == modifier.skill_def.raw())
            })
            .map(|(_, count)| *count)
            .unwrap_or(0),
        ModifierValueSource::OwnerSkillPileCount => state
            .skill_pile_cards(modifier.owner, modifier.skill_def.raw())
            .len()
            .min(i32::MAX as usize) as i32,
        ModifierValueSource::OwnerSkillStateU32 => state
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == modifier.owner && skill.def == modifier.skill_def)
            .and_then(|skill| skill.state_blob.get(..4))
            .map(|raw| u32::from_le_bytes(raw.try_into().unwrap()).min(i32::MAX as u32) as i32)
            .unwrap_or(0),
        ModifierValueSource::OwnerSkillStateU32Negative => state
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == modifier.owner && skill.def == modifier.skill_def)
            .and_then(|skill| skill.state_blob.get(..4))
            .map(|raw| -(u32::from_le_bytes(raw.try_into().unwrap()).min(i32::MAX as u32) as i32))
            .unwrap_or(0),
        ModifierValueSource::QueriedPlayerQunOwnerLordAlive => queried
            .filter(|queried| {
                state
                    .players
                    .get(*queried)
                    .is_some_and(|player| player.alive)
                    && state.player_group(*queried) == Some("qun")
                    && state
                        .players
                        .get(modifier.owner)
                        .is_some_and(|owner| owner.alive && owner.identity == Identity::Lord)
            })
            .map_or(0, |_| 1),
        ModifierValueSource::OwnerSkillUseCount => state
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == modifier.owner && skill.def == modifier.skill_def)
            .map(|skill| skill.use_count.min(i32::MAX as u32) as i32)
            .unwrap_or(0),
        ModifierValueSource::OwnerLinked => i32::from(player.is_some_and(|participant| {
            participant.alive
                && participant
                    .marks
                    .iter()
                    .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
        })),
        ModifierValueSource::AliveWoundedOrGuimingWuCountMinOne => {
            alive_wounded_or_guiming_wu_count(state, modifier.owner).max(1)
        }
        ModifierValueSource::OwnerMark(mark) => player
            .and_then(|player| player.marks.iter().find(|(id, _)| *id == mark))
            .map(|(_, count)| *count)
            .unwrap_or(0),
        ModifierValueSource::AliveGroupCount => super::super::alive_group_count(state),
        ModifierValueSource::AliveFemaleCount => state
            .players
            .iter_live()
            .filter(|player| player.alive)
            .filter_map(|player| state.content.character(player.character_def))
            .filter(|character| character.sex == "female")
            .count()
            .min(i32::MAX as usize) as i32,
        ModifierValueSource::AliveOtherCountCappedFour => state
            .players
            .iter_live()
            .filter(|player| player.alive && player.id != modifier.owner)
            .count()
            .min(4) as i32,
        ModifierValueSource::OwnerNoMount => player
            .map(|_| {
                let has_mount = state
                    .zones
                    .find(ZoneKind::Equip, modifier.owner)
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| {
                        zone.cards.iter().any(|card| {
                            state.cards.get(*card).is_some_and(|card| {
                                matches!(card.kind, CardKind::HorseOff | CardKind::HorseDef)
                            })
                        })
                    });
                i32::from(!has_mount)
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerHandEmpty => player
            .map(|_| i32::from(state.hand_count(modifier.owner) == 0))
            .unwrap_or(0),
        ModifierValueSource::OwnerHasTreasure => player
            .map(|_| {
                i32::from(
                    state
                        .zones
                        .find(ZoneKind::Equip, modifier.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| {
                            zone.cards.iter().any(|card| {
                                state
                                    .cards
                                    .get(*card)
                                    .is_some_and(|card| card.kind == CardKind::Treasure)
                            })
                        }),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::AliveMountedPlayerCountMinOne => state
            .players
            .iter_live()
            .filter(|player| player.alive)
            .filter(|player| {
                state
                    .zones
                    .find(ZoneKind::Equip, player.id)
                    .and_then(|zone| state.zones.get(zone))
                    .is_some_and(|zone| {
                        zone.cards.iter().any(|card| {
                            state.cards.get(*card).is_some_and(|card| {
                                matches!(card.kind, CardKind::HorseOff | CardKind::HorseDef)
                            })
                        })
                    })
            })
            .count()
            .max(1)
            .min(i32::MAX as usize)
            as i32,
        ModifierValueSource::OwnerEquipCountOdd => player
            .map(|_| i32::from(owner_equip_count(state, modifier.owner) % 2 == 1))
            .unwrap_or(0),
        ModifierValueSource::OwnerEquipCountEven => player
            .map(|_| i32::from(owner_equip_count(state, modifier.owner) % 2 == 0))
            .unwrap_or(0),
        ModifierValueSource::OwnerEquipCountHalfFloor => player
            .map(|_| {
                owner_equip_count(state, modifier.owner)
                    .saturating_div(2)
                    .min(i32::MAX as usize) as i32
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerEquipCount => player
            .map(|_| owner_equip_count(state, modifier.owner).min(i32::MAX as usize) as i32)
            .unwrap_or(0),
        ModifierValueSource::OwnerSkillPileCountNegative(skill_def) => {
            -(state
                .skill_pile_cards(modifier.owner, skill_def)
                .len()
                .min(i32::MAX as usize) as i32)
        }
        ModifierValueSource::OwnerSkillPileCountOf(skill_def) => state
            .skill_pile_cards(modifier.owner, skill_def)
            .len()
            .min(i32::MAX as usize)
            as i32,
        ModifierValueSource::OwnerEquipCountAtLeast(threshold) => player
            .map(|_| i32::from(owner_equip_count(state, modifier.owner) >= threshold as usize))
            .unwrap_or(0),
        // Count-gated floor: the equip count itself when it reaches the threshold,
        // otherwise 0 (upstream 白虎骁骑 cardUsable/globalFrom/draw thresholds).
        ModifierValueSource::OwnerEquipCountFloor(threshold) => player
            .map(|_| {
                let count = owner_equip_count(state, modifier.owner);
                if count >= threshold as usize {
                    count.min(i32::MAX as usize) as i32
                } else {
                    0
                }
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerEquipCountExactlyOne => player
            .map(|_| i32::from(owner_equip_count(state, modifier.owner) == 1))
            .unwrap_or(0),
        ModifierValueSource::OwnerHpGtTwo => {
            player.map(|player| i32::from(player.hp > 2)).unwrap_or(0)
        }
        ModifierValueSource::OwnerHpLeTwo => {
            player.map(|player| i32::from(player.hp <= 2)).unwrap_or(0)
        }
        ModifierValueSource::OwnerLostHpAtLeastTwo => player
            .map(|player| {
                i32::from((player.max_hp.min(i32::MAX as u32) as i32 - player.hp).max(0) >= 2)
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerAllOthersInAttackRange => player
            .map(|_| {
                i32::from(super::super::all_other_players_in_attack_range(
                    state,
                    modifier.owner,
                ))
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerShaUsedZero => {
            i32::from(state.turn.flag(crate::state::turn_flag::SHA_PLAYED) == 0)
        }
        ModifierValueSource::OwnerShaUsedNonzero => i32::from(
            modifier.owner == state.turn.current
                && state.turn.flag(crate::state::turn_flag::SHA_PLAYED) != 0,
        ),
        ModifierValueSource::OwnerShaUsedExactlyOne => i32::from(
            modifier.owner == state.turn.current
                && state.turn.flag(crate::state::turn_flag::SHA_PLAYED) == 1,
        ),
        ModifierValueSource::OwnerShaUsedExactlyTwo => i32::from(
            modifier.owner == state.turn.current
                && state.turn.flag(crate::state::turn_flag::SHA_PLAYED) == 2,
        ),
        ModifierValueSource::OwnerMaxHandBeforeSkillFiresThisTurn => player
            .filter(|player| player.alive)
            .map(|_| {
                let owner_hand = state.hand_count(modifier.owner);
                i32::from(
                    state
                        .skills
                        .get(modifier.skill_id)
                        .is_some_and(|skill| skill.turn_fire_count == 0)
                        && state
                            .players
                            .iter_live()
                            .filter(|other| other.alive && other.id != modifier.owner)
                            .all(|other| state.hand_count(other.id) <= owner_hand),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerNonDamageHandTypeCount => state
            .hand_cards(modifier.owner)
            .into_iter()
            .filter_map(|card| state.cards.get(card))
            .filter(|card| !card.kind.is_sha() && !card.kind.is_damage_trick())
            .filter_map(|card| {
                if card.kind.is_equip() {
                    Some(2u8)
                } else if card.kind.is_trick() {
                    Some(1u8)
                } else if matches!(
                    card.kind,
                    CardKind::Sha
                        | CardKind::ShaFire
                        | CardKind::ShaThunder
                        | CardKind::Tao
                        | CardKind::Shan
                        | CardKind::Jiu
                ) {
                    Some(0u8)
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>()
            .len()
            .min(i32::MAX as usize)
            as i32,
        ModifierValueSource::OwnerShaHandCount => state
            .hand_cards(modifier.owner)
            .into_iter()
            .filter(|card| {
                state
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.kind.is_sha())
            })
            .count()
            .min(i32::MAX as usize) as i32,
        ModifierValueSource::OwnerTurnDamageDealt => {
            if modifier.owner == state.turn.current {
                state
                    .turn
                    .flag(crate::state::turn_flag::DAMAGE_DEALT)
                    .max(0)
            } else {
                0
            }
        }
        ModifierValueSource::OwnerDistinctHandCardSuitsUsedThisTurn => state
            .turn
            .distinct_hand_card_suits_used(modifier.owner)
            .min(i32::MAX as u32)
            as i32,
        ModifierValueSource::OwnerCurrentTurnCardUseCount => {
            if modifier.owner == state.turn.current {
                state
                    .turn
                    .flag(crate::state::turn_flag::card_used_by_player(modifier.owner))
                    .max(0)
            } else {
                0
            }
        }
        ModifierValueSource::OwnerPlayPhaseCardUseCountZero => i32::from(
            modifier.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && state
                    .turn
                    .flag(crate::state::turn_flag::play_phase_card_used_by_player(
                        modifier.owner,
                    ))
                    == 0,
        ),
        ModifierValueSource::OwnerSkillPlayPhaseFireCountBelow(limit) => i32::from(
            limit > 0
                && modifier.owner == state.turn.current
                && state.turn.phase == Phase::Play
                && state
                    .skills
                    .get(modifier.skill_id)
                    .is_some_and(|skill| skill.play_phase_fire_count < limit),
        ),
        ModifierValueSource::OwnerCurrentTurnAllOthersDistanceAtMostOne => i32::from(
            modifier.owner == state.turn.current
                && owner_all_others_distance_at_most_one(state, modifier.owner),
        ),
        ModifierValueSource::OwnerHasHeartEquip => player
            .map(|_| {
                i32::from(
                    state
                        .zones
                        .find(ZoneKind::Equip, modifier.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| {
                            zone.cards.iter().any(|card| {
                                state.cards.get(*card).is_some_and(|card| {
                                    super::super::effective_card_suit(state, modifier.owner, card)
                                        == Suit::Heart
                                })
                            })
                        }),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::AliveOtherQunCount => state
            .players
            .iter_live()
            .filter(|other| other.alive && other.id != modifier.owner)
            .filter(|other| state.player_group(other.id) == Some("qun"))
            .count()
            .min(i32::MAX as usize) as i32,
        ModifierValueSource::OwnerHasWeapon => player
            .map(|_| {
                i32::from(
                    state
                        .zones
                        .find(ZoneKind::Equip, modifier.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| {
                            zone.cards.iter().any(|card| {
                                state
                                    .cards
                                    .get(*card)
                                    .is_some_and(|card| card.kind == CardKind::Weapon)
                            })
                        }),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerNoWeapon => player
            .map(|_| {
                i32::from(
                    !state
                        .zones
                        .find(ZoneKind::Equip, modifier.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| {
                            zone.cards.iter().any(|card| {
                                state
                                    .cards
                                    .get(*card)
                                    .is_some_and(|card| card.kind == CardKind::Weapon)
                            })
                        }),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerHasArmorEquip => player
            .map(|_| {
                i32::from(
                    state
                        .zones
                        .find(ZoneKind::Equip, modifier.owner)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| {
                            zone.cards.iter().any(|card| {
                                state
                                    .cards
                                    .get(*card)
                                    .is_some_and(|card| card.kind == CardKind::Armor)
                            })
                        }),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::RoundNumberCappedFive => state.turn.round.min(5) as i32,
        ModifierValueSource::RoundAtLeastAliveQunCount => {
            let qun = alive_qun_count(state);
            i32::from(state.turn.round as usize >= qun)
        }
        ModifierValueSource::RoundBelowAliveQunCount => {
            let qun = alive_qun_count(state);
            i32::from((state.turn.round as usize) < qun)
        }
        ModifierValueSource::AliveFieldRankThirteenPlayerCount => state
            .players
            .iter_live()
            .filter(|other| other.alive)
            .filter(|other| {
                [ZoneKind::Equip, ZoneKind::Judge].into_iter().any(|kind| {
                    state
                        .zones
                        .find(kind, other.id)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| {
                            zone.cards.iter().any(|card| {
                                state.cards.get(*card).is_some_and(|card| card.rank == 13)
                            })
                        })
                })
            })
            .count()
            .min(i32::MAX as usize)
            as i32,
        ModifierValueSource::FieldMountCardCountCeilHalf => {
            let mounts = state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .filter_map(|player| state.zones.find(ZoneKind::Equip, player.id))
                .filter_map(|zone| state.zones.get(zone))
                .flat_map(|zone| zone.cards.iter())
                .filter(|card| {
                    state.cards.get(**card).is_some_and(|card| {
                        matches!(card.kind, CardKind::HorseOff | CardKind::HorseDef)
                    })
                })
                .count();
            mounts.div_ceil(2).min(i32::MAX as usize) as i32
        }
        ModifierValueSource::OwnerUniqueMaxHand => player
            .filter(|player| player.alive)
            .map(|_| {
                let owner_hand = state.hand_count(modifier.owner);
                i32::from(
                    state
                        .players
                        .iter_live()
                        .filter(|other| other.alive && other.id != modifier.owner)
                        .all(|other| state.hand_count(other.id) < owner_hand),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerArmorEmpty => player
            .map(|_| {
                i32::from(!super::super::has_equip_kind(
                    state,
                    modifier.owner,
                    CardKind::Armor,
                ))
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerHealthy => player
            .map(|player| i32::from(player.hp >= player.max_hp as i32))
            .unwrap_or(0),
        ModifierValueSource::AliveWoundedCountZeroOrAtLeastFour => {
            let wounded = super::super::alive_wounded_count(state);
            i32::from(wounded == 0 || wounded >= 4)
        }
        ModifierValueSource::AliveWoundedCountAtLeast(threshold) => {
            i32::from(super::super::alive_wounded_count(state) >= threshold as usize)
        }
        ModifierValueSource::OwnerHandGreaterThanOther => other
            .filter(|other| other.is_some())
            .filter(|other| {
                player.is_some_and(|player| player.alive)
                    && state.players.get(*other).is_some_and(|player| player.alive)
            })
            .map(|other| i32::from(state.hand_count(modifier.owner) > state.hand_count(other)))
            .unwrap_or(0),
        ModifierValueSource::OwnerHandLessThanOther => other
            .filter(|other| other.is_some())
            .filter(|other| {
                player.is_some_and(|player| player.alive)
                    && state.players.get(*other).is_some_and(|player| player.alive)
            })
            .map(|other| i32::from(state.hand_count(modifier.owner) < state.hand_count(other)))
            .unwrap_or(0),
        ModifierValueSource::OwnerQunOtherShu => other
            .filter(|other| other.is_some())
            .filter(|other| {
                player.is_some_and(|player| player.alive)
                    && state.players.get(*other).is_some_and(|player| player.alive)
            })
            .map(|other| {
                i32::from(
                    state.player_group(modifier.owner) == Some("qun")
                        && state.player_group(other) == Some("shu"),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerTurnedOverDiffOther => other
            .filter(|other| other.is_some())
            .filter(|other| {
                player.is_some_and(|player| player.alive)
                    && state.players.get(*other).is_some_and(|player| player.alive)
            })
            .map(|other| {
                i32::from(
                    player.is_some_and(|player| player.turned_over)
                        != state
                            .players
                            .get(other)
                            .is_some_and(|player| player.turned_over),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerHandEquipGteOther => other
            .filter(|other| other.is_some())
            .filter(|other| {
                player.is_some_and(|player| player.alive)
                    && state.players.get(*other).is_some_and(|player| player.alive)
            })
            .map(|other| {
                i32::from(
                    state.hand_count(modifier.owner) >= state.hand_count(other)
                        && owner_equip_count(state, modifier.owner)
                            >= owner_equip_count(state, other),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerShaDamageDealtZeroThisTurn => i32::from(
            state
                .turn
                .flag(crate::state::turn_flag::sha_damage_dealt_by_player(
                    modifier.owner,
                ))
                == 0,
        ),
        ModifierValueSource::QueriedPlayerHandGtHpSigned => queried
            .and_then(|queried| state.players.get(queried).map(|player| (queried, player)))
            .map(|(queried, player)| {
                if state.hand_count(queried).min(i32::MAX as u32) as i32 > player.hp {
                    1
                } else {
                    -1
                }
            })
            .unwrap_or(0),
        ModifierValueSource::OwnerLinkedDiffOther => other
            .filter(|other| other.is_some())
            .filter(|other| {
                player.is_some_and(|player| player.alive)
                    && state.players.get(*other).is_some_and(|player| player.alive)
            })
            .map(|other| {
                let linked = |player: PlayerId| {
                    state.players.get(player).is_some_and(|player| {
                        player
                            .marks
                            .iter()
                            .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                    })
                };
                i32::from(linked(modifier.owner) != linked(other))
            })
            .unwrap_or(0),
        ModifierValueSource::QueriedPlayerJudgeNonemptyOtherThanOwner => queried
            .filter(|queried| *queried != modifier.owner)
            .filter(|queried| {
                state
                    .players
                    .get(*queried)
                    .is_some_and(|player| player.alive)
            })
            .map(|queried| {
                i32::from(
                    state
                        .zones
                        .find(ZoneKind::Judge, queried)
                        .and_then(|zone| state.zones.get(zone))
                        .is_some_and(|zone| !zone.cards.is_empty()),
                )
            })
            .unwrap_or(0),
        ModifierValueSource::QueriedPlayerLinkedOtherThanOwner => queried
            .filter(|queried| *queried != modifier.owner)
            .filter(|queried| {
                state
                    .players
                    .get(*queried)
                    .is_some_and(|player| player.alive)
            })
            .map(|queried| {
                i32::from(state.players.get(queried).is_some_and(|player| {
                    player
                        .marks
                        .iter()
                        .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                }))
            })
            .unwrap_or(0),
        ModifierValueSource::QueriedPlayerLinkedFirstLivingSkillOwner => {
            let first_owner = state
                .skills
                .sorted_live()
                .into_iter()
                .filter(|skill| {
                    skill.def == modifier.skill_def
                        && state.skill_effectively_enabled(skill)
                        && state
                            .players
                            .get(skill.owner)
                            .is_some_and(|owner| owner.alive)
                })
                .min_by_key(|skill| {
                    state
                        .players
                        .get(skill.owner)
                        .map(|owner| owner.seat.raw())
                        .unwrap_or(u8::MAX)
                })
                .map(|skill| skill.owner);
            if first_owner != Some(modifier.owner) {
                0
            } else {
                queried
                    .filter(|queried| {
                        state
                            .players
                            .get(*queried)
                            .is_some_and(|player| player.alive)
                    })
                    .map(|queried| {
                        i32::from(state.players.get(queried).is_some_and(|player| {
                            player.marks.iter().any(|(mark, count)| {
                                *mark == crate::content::MARK_CHAIN && *count > 0
                            })
                        }))
                    })
                    .unwrap_or(0)
            }
        }
        ModifierValueSource::QueriedPlayerMarkPositive(mark) => queried
            .and_then(|queried| state.players.get(queried))
            .filter(|player| player.alive)
            .and_then(|player| {
                player
                    .marks
                    .iter()
                    .find(|(found, _)| *found == mark)
                    .map(|(_, count)| i32::from(*count > 0))
            })
            .unwrap_or(0)
            .max(0),
        ModifierValueSource::QueriedPlayerOtherThanOwner => queried
            .filter(|queried| *queried != modifier.owner)
            .filter(|queried| {
                state
                    .players
                    .get(*queried)
                    .is_some_and(|player| player.alive)
            })
            .map(|_| 1)
            .unwrap_or(0),
        ModifierValueSource::QueriedPlayerWoundedInOwnerAttackRangeOther => queried
            .filter(|queried| *queried != modifier.owner)
            .filter(|queried| {
                state
                    .players
                    .get(*queried)
                    .is_some_and(|player| player.alive && player.hp < player.max_hp as i32)
            })
            .filter(|queried| crate::query::player_in_attack_range(state, modifier.owner, *queried))
            .map(|_| 1)
            .unwrap_or(0),
        ModifierValueSource::QueriedSourceEnemyOwnerAllyTargetOtherThanOwner => queried
            .filter(|target| *target != modifier.owner)
            .zip(other)
            .filter(|(target, source)| {
                state
                    .players
                    .get(*target)
                    .is_some_and(|player| player.alive)
                    && state
                        .players
                        .get(*source)
                        .is_some_and(|player| player.alive)
            })
            .map(|(target, source)| {
                let owner_allied_with_target = matches!(
                    super::super::relation(state, modifier.owner, target),
                    QueryResponse::Relation(RelationTag::Ally)
                );
                let source_enemy_of_target = matches!(
                    super::super::relation(state, source, target),
                    QueryResponse::Relation(RelationTag::Enemy)
                );
                i32::from(owner_allied_with_target && source_enemy_of_target)
            })
            .unwrap_or(0),
    };
    modifier.value.saturating_mul(value)
}

/// Positive optional draw bonuses grouped by owning skill in stable skill-id order.
pub fn optional_draw_bonuses_for(state: &GameState, owner: PlayerId) -> Vec<u32> {
    let mut grouped = BTreeMap::<SkillId, i32>::new();
    for modifier in state
        .modifiers
        .for_target(ModifierTarget::OptionalDrawCount)
        .into_iter()
        .filter(|modifier| modifier.owner == owner && modifier.mode == CombineMode::Add)
    {
        let total = grouped.entry(modifier.skill_id).or_default();
        *total = total.saturating_add(resolved_value(state, modifier));
    }
    grouped
        .into_values()
        .filter(|bonus| *bonus > 0)
        .map(|bonus| bonus as u32)
        .collect()
}

/// Count living wounded players, plus other living Wu players while the owner is the Lord and
/// has enabled Guiming.
pub fn alive_wounded_or_guiming_wu_count(state: &GameState, owner: PlayerId) -> i32 {
    const GUIMING: u32 = 3_828_399_214;
    let guiming_enabled = state
        .players
        .get(owner)
        .is_some_and(|player| player.alive && player.identity == Identity::Lord)
        && state.skills.sorted_live().into_iter().any(|skill| {
            skill.owner == owner
                && skill.def.raw() == GUIMING
                && state.skill_effectively_enabled(skill)
        });
    state
        .players
        .iter_live()
        .filter(|other| {
            other.alive
                && (other.hp < other.max_hp as i32
                    || (guiming_enabled
                        && other.id != owner
                        && state.player_group(other.id) == Some("wu")))
        })
        .count()
        .min(i32::MAX as usize) as i32
}

pub(crate) fn owner_all_others_distance_at_most_one(state: &GameState, owner: PlayerId) -> bool {
    state
        .players
        .iter_live()
        .filter(|player| player.alive && player.id != owner)
        .all(
            |player| match super::super::distance_explain(state, owner, player.id).response {
                rule_api::QueryResponse::Int(value) => value <= 1,
                _ => false,
            },
        )
}

pub(crate) fn owner_equip_count(state: &GameState, owner: PlayerId) -> usize {
    state
        .zones
        .find(ZoneKind::Equip, owner)
        .and_then(|zone| state.zones.get(zone))
        .map(|zone| zone.cards.len())
        .unwrap_or(0)
}

pub(crate) fn alive_qun_count(state: &GameState) -> usize {
    state
        .players
        .iter_live()
        .filter(|player| player.alive)
        .filter(|player| state.player_group(player.id) == Some("qun"))
        .count()
}

pub(crate) fn src(m: &ActiveModifier, value: i32) -> ModifierSource {
    ModifierSource {
        source_id: m.skill_id.raw(),
        label_key: m.label_key.clone(),
        value,
    }
}

/// Debug helper: group sources by label.
pub fn sources_by_label(sources: &[ModifierSource]) -> BTreeMap<String, i32> {
    let mut map = BTreeMap::new();
    for s in sources {
        *map.entry(s.label_key.clone()).or_insert(0) += s.value;
    }
    map
}
