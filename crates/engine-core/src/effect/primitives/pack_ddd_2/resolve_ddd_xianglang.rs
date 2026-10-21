//! Effect primitives for `resolve_ddd_xianglang` and related helpers.

use super::*;

pub(crate) fn resolve_ddd_xianglang(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let count = tx
                .state()
                .players
                .get(actor)
                .map(|participant| participant.hp.max(0) as usize)
                .unwrap_or(0);
            let options: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|card| !card.face_up)
                })
                .collect();
            if player == actor || count == 0 || options.len() < count {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                count.min(u8::MAX as usize) as u8,
                count.min(u8::MAX as usize) as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_XIANGLANG_CHOICE,
                    0,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            let targets = tx.state().players.alive_by_seat();
            let mut events = Vec::new();
            for target in targets {
                let exists = tx.state().players.get(target).is_some_and(|participant| {
                    participant.skills.iter().any(|owned| {
                        tx.state()
                            .skills
                            .get(*owned)
                            .is_some_and(|instance| instance.def == DDD_FUSI_GLOBAL_DEF)
                    })
                });
                if !exists {
                    events.extend(
                        grant_skill_instance(tx, target, DDD_FUSI_GLOBAL_DEF, Vec::new())?.1,
                    );
                }
            }
            Ok(events)
        }
        2 => {
            let all_shown = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .all(|card| tx.state().cards.get(card).is_some_and(|card| card.face_up));
            let has_tuoji = tx.state().players.get(player).is_some_and(|participant| {
                participant.skills.iter().any(|owned| {
                    tx.state().skills.get(*owned).is_some_and(|instance| {
                        instance.def == DDD_TUOJI_DEF
                            && tx.state().skill_effectively_enabled(instance)
                    })
                })
            });
            if !all_shown || !has_tuoji {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_XIANGLANG_CHOICE,
                    2,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown 3D精选向朗 resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_xianglang_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DDD_XIANGLANG_CHOICE) || cancelled {
        return Err(EffectError::Illegal("bad 3D精选向朗 choice".into()));
    }
    let stage = *payload.get(1).unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let actor = PlayerId::new(*payload.get(3).unwrap_or(&0));
    match stage {
        0 if observer == player => {
            let count = tx
                .state()
                .players
                .get(actor)
                .map(|participant| participant.hp.max(0) as usize)
                .unwrap_or(0);
            let legal: std::collections::BTreeSet<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|card| !card.face_up)
                })
                .collect();
            let selected: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            let unique: std::collections::BTreeSet<_> = selected.iter().copied().collect();
            if count == 0
                || selected.len() != count
                || unique.len() != count
                || selected.iter().any(|card| !legal.contains(card))
            {
                return Err(EffectError::Illegal("恰化明置手牌选择已失效".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::RevealCards {
                    cards: selected.clone(),
                    to: Vec::new(),
                }],
            )?;
            for card in selected {
                tx.state_mut()
                    .cards
                    .get_mut(card)
                    .expect("validated 恰化 card")
                    .face_up = true;
            }
            add_skill_until_turn_end(tx, actor, DDD_XUNXUN_DEF).map(|_| ())
        }
        2 if observer == player => {
            let [decision] = selections else {
                return Err(EffectError::Illegal("拓籍须确认是否摸牌".into()));
            };
            if *decision == 0 {
                Ok(())
            } else if *decision == 1 {
                run_effects(tx, vec![EffectRequest::DrawCards { player, count: 3 }]).map(|_| ())
            } else {
                Err(EffectError::Illegal("拓籍选项无效".into()))
            }
        }
        _ => Err(EffectError::Illegal(
            "向朗 choice observer or stage changed".into(),
        )),
    }
}

pub(crate) fn ddd_zhengjun_category_candidates(
    tx: &Transaction,
    owner: PlayerId,
    from: PlayerId,
    to: PlayerId,
    move_mask: u8,
    from_bit: u8,
    to_bit: u8,
    equipment: bool,
) -> Vec<PlayerId> {
    let equip_count = |player| {
        tx.state()
            .zones
            .find(ZoneKind::Equip, player)
            .and_then(|zone| tx.state().zones.get(zone))
            .map(|zone| zone.cards.len() as u32)
            .unwrap_or(0)
    };
    let mut candidates = Vec::new();
    if move_mask & from_bit != 0 && from.is_some() {
        candidates.push(from);
    }
    if move_mask & to_bit != 0 && to.is_some() && !candidates.contains(&to) {
        candidates.push(to);
    }
    let owner_count = if equipment {
        equip_count(owner)
    } else {
        tx.state().hand_count(owner)
    };
    candidates.retain(|candidate| {
        tx.state()
            .players
            .get(*candidate)
            .is_some_and(|participant| participant.alive)
            && if equipment {
                equip_count(*candidate) == owner_count
            } else {
                tx.state().hand_count(*candidate) == owner_count
            }
    });
    candidates
}

pub(crate) fn ddd_zhengjun_mark_category(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    category: u8,
) -> Result<(), EffectError> {
    let valid = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player
            && tx.state().skill_effectively_enabled(instance)
            && tx.state().turn.current == player
            && tx.state().turn.phase == Phase::Play
            && instance.play_phase_category_mask & category == 0
    });
    if !valid {
        return Err(EffectError::Illegal("整军分类机会已失效".into()));
    }
    tx.state_mut()
        .skills
        .get_mut(skill)
        .expect("validated 整军 skill")
        .play_phase_category_mask |= category;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_ddd_yujin(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    target: PlayerId,
    move_mask: u8,
    action: u8,
    candidates: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let equal = tx
                .state()
                .players
                .get(actor)
                .zip(tx.state().players.get(player))
                .is_some_and(|(changed, owner)| changed.alive && changed.hp == owner.hp);
            if !equal {
                return Ok(Vec::new());
            }
            ddd_zhengjun_mark_category(tx, player, skill, DDD_ZHENGJUN_HP)?;
            let mut options = vec![1]; // lose 1 HP
            if tx
                .state()
                .players
                .get(actor)
                .is_some_and(|participant| participant.hp < participant.max_hp as i32)
            {
                options.insert(0, 0); // recover 1 HP
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_YUJIN_CHOICE,
                    0,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            let used = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| instance.play_phase_category_mask)
                .unwrap_or(u8::MAX);
            let hand = if used & DDD_ZHENGJUN_HAND == 0 {
                ddd_zhengjun_category_candidates(tx, player, actor, target, move_mask, 1, 2, false)
            } else {
                Vec::new()
            };
            let equipment = if used & DDD_ZHENGJUN_EQUIP == 0 {
                ddd_zhengjun_category_candidates(tx, player, actor, target, move_mask, 4, 8, true)
            } else {
                Vec::new()
            };
            if hand.is_empty() {
                return resolve_ddd_yujin(
                    tx,
                    player,
                    PlayerId::NONE,
                    PlayerId::NONE,
                    0,
                    2,
                    equipment,
                    skill,
                );
            }
            let mut payload = vec![
                RESOLVE_DDD_YUJIN_CHOICE,
                1,
                player.raw(),
                skill.raw(),
                hand.len() as u32,
            ];
            payload.extend(hand.iter().map(|candidate| candidate.raw()));
            payload.push(equipment.len() as u32);
            payload.extend(equipment.iter().map(|candidate| candidate.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                hand.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        2 => {
            let mut candidates = candidates;
            candidates.retain(|candidate| {
                tx.state()
                    .players
                    .get(*candidate)
                    .is_some_and(|participant| participant.alive)
            });
            candidates.sort_unstable_by_key(|candidate| candidate.raw());
            candidates.dedup();
            if candidates.is_empty()
                || tx.state().skills.get(skill).is_none_or(|instance| {
                    instance.play_phase_category_mask & DDD_ZHENGJUN_EQUIP != 0
                })
            {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_DDD_YUJIN_CHOICE,
                3,
                player.raw(),
                skill.raw(),
                candidates.len() as u32,
            ];
            payload.extend(candidates.iter().map(|candidate| candidate.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                candidates.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown 3D精选于禁 resolver action".into(),
        )),
    }
}

pub(crate) fn ddd_zhengjun_payload_candidates(
    payload: &[u32],
) -> Result<(Vec<PlayerId>, Vec<PlayerId>), EffectError> {
    let hand_count = *payload
        .get(4)
        .ok_or_else(|| EffectError::EngineFault("整军候选数据截断".into()))?
        as usize;
    let hand_end = 5usize.saturating_add(hand_count);
    let hand = payload
        .get(5..hand_end)
        .ok_or_else(|| EffectError::EngineFault("整军手牌候选截断".into()))?
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    let equipment_count = *payload
        .get(hand_end)
        .ok_or_else(|| EffectError::EngineFault("整军装备候选数据截断".into()))?
        as usize;
    let equipment = payload
        .get(hand_end + 1..hand_end + 1 + equipment_count)
        .ok_or_else(|| EffectError::EngineFault("整军装备候选截断".into()))?
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    Ok((hand, equipment))
}

pub(crate) fn resolve_ddd_yujin_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DDD_YUJIN_CHOICE) || payload.len() < 4 {
        return Err(EffectError::EngineFault("bad 3D精选于禁 choice".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    if observer != player {
        return Err(EffectError::Illegal("整军选择者已变化".into()));
    }
    match stage {
        0 => {
            let actor = PlayerId::new(payload[3]);
            let [branch] = selections else {
                return Err(EffectError::Illegal("整军体力选项无效".into()));
            };
            if cancelled || *branch > 1 {
                return Err(EffectError::Illegal("整军体力选项无效".into()));
            }
            let effect = if *branch == 0 {
                EffectRequest::RecoverHp {
                    source: player,
                    player: actor,
                    amount: 1,
                }
            } else {
                EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: actor,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                }
            };
            run_effects(tx, vec![effect]).map(|_| ())
        }
        1 => {
            let skill = engine_api::SkillId::new(payload[3]);
            let (hand, equipment) = ddd_zhengjun_payload_candidates(payload)?;
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal("取消整军仍包含目标".into()));
                }
                return run_effects(
                    tx,
                    vec![EffectRequest::ResolveDddYujin {
                        player,
                        actor: PlayerId::NONE,
                        target: PlayerId::NONE,
                        move_mask: 0,
                        action: 2,
                        candidates: equipment,
                        skill,
                    }],
                )
                .map(|_| ());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal("整军须选择一名手牌数变化角色".into()));
            };
            let target = PlayerId::new(*selected);
            if !hand.contains(&target) {
                return Err(EffectError::Illegal("整军手牌目标已失效".into()));
            }
            ddd_zhengjun_mark_category(tx, player, skill, DDD_ZHENGJUN_HAND)?;
            let mut next = vec![
                RESOLVE_DDD_YUJIN_CHOICE,
                2,
                player.raw(),
                skill.raw(),
                target.raw(),
                equipment.len() as u32,
            ];
            next.extend(equipment.iter().map(|candidate| candidate.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
        2 => {
            if cancelled || payload.len() < 6 {
                return Err(EffectError::Illegal("整军手牌选项无效".into()));
            }
            let [branch @ 0..=1] = selections else {
                return Err(EffectError::Illegal("整军手牌选项无效".into()));
            };
            let skill = engine_api::SkillId::new(payload[3]);
            let target = PlayerId::new(payload[4]);
            let count = payload[5] as usize;
            let equipment: Vec<_> = payload
                .get(6..6 + count)
                .ok_or_else(|| EffectError::EngineFault("整军装备续体截断".into()))?
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            let mut effects = Vec::new();
            if *branch == 0 {
                effects.push(EffectRequest::DrawCards {
                    player: target,
                    count: 1,
                });
            } else if !crate::query::discardable_hand_equip_cards(tx.state(), target).is_empty() {
                effects.push(EffectRequest::DiscardHandEquipCount {
                    player: target,
                    count: 1,
                    skill,
                });
            }
            effects.push(EffectRequest::ResolveDddYujin {
                player,
                actor: PlayerId::NONE,
                target: PlayerId::NONE,
                move_mask: 0,
                action: 2,
                candidates: equipment,
                skill,
            });
            run_effects(tx, effects).map(|_| ())
        }
        3 => {
            let skill = engine_api::SkillId::new(payload[3]);
            let count = *payload
                .get(4)
                .ok_or_else(|| EffectError::EngineFault("整军装备目标截断".into()))?
                as usize;
            let candidates: Vec<_> = payload
                .get(5..5 + count)
                .ok_or_else(|| EffectError::EngineFault("整军装备目标截断".into()))?
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            if cancelled {
                return if selections.is_empty() {
                    Ok(())
                } else {
                    Err(EffectError::Illegal("取消整军仍包含目标".into()))
                };
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal("整军须选择一名装备数变化角色".into()));
            };
            let target = PlayerId::new(*selected);
            if !candidates.contains(&target) {
                return Err(EffectError::Illegal("整军装备目标已失效".into()));
            }
            ddd_zhengjun_mark_category(tx, player, skill, DDD_ZHENGJUN_EQUIP)?;
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwtianyu {
                    player,
                    target,
                    helper_skill_def: 0,
                    action: 0,
                    stage: 2,
                    card: CardId::NONE,
                    branch: 2,
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown 3D精选于禁 choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_liuye(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            if actor == player
                || tx.state().turn.current != actor
                || tx.state().turn.phase != Phase::Play
            {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_LIUYE_CHOICE,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            let matched = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| instance.ddd_chashi_state())
                .is_some_and(|(target, suit, category)| {
                    let used_suit = tx
                        .state()
                        .cards
                        .get(card)
                        .map(|card| crate::query::effective_card_suit(tx.state(), actor, card))
                        .map(crate::command::judge::suit_tag)
                        .unwrap_or(0) as u8;
                    let used_category = effective_card_category(tx.state(), actor, card);
                    target == actor
                        && tx.state().turn.current == actor
                        && tx.state().turn.phase == Phase::Play
                        && ((used_suit != 0 && used_suit == suit)
                            || (used_category != 0 && used_category == category))
                });
            if !matched {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated 察势 skill")
                .clear_ddd_chashi_state();
            run_effects(
                tx,
                vec![
                    EffectRequest::DrawCards {
                        player: actor,
                        count: 1,
                    },
                    EffectRequest::DrawCards { player, count: 1 },
                ],
            )
        }
        2 => {
            let ready = tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.content_flag(0) == Some(false));
            if !ready || actor != player {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated 齐策 skill")
                .set_content_flag(0, true);
            run_effects(tx, vec![EffectRequest::DrawCards { player, count: 2 }])
        }
        3 => {
            let trick = actor == player
                && card.is_some()
                && tx.state().cards.get(card).is_some_and(|card| {
                    crate::query::effective_card_kind(tx.state(), actor, card).is_trick()
                });
            if trick {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .expect("validated 齐策 skill")
                    .set_content_flag(0, false);
            }
            Ok(Vec::new())
        }
        4 => {
            let pending = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| instance.ddd_chashi_state())
                .is_some_and(|(target, _, _)| target == actor);
            if pending {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .expect("validated 察势 skill")
                    .clear_ddd_chashi_state();
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown 3D精选刘晔 resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_liuye_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [tag, player, actor, skill] = payload else {
        return Err(EffectError::EngineFault("bad 3D精选刘晔 choice".into()));
    };
    let player = PlayerId::new(*player);
    let actor = PlayerId::new(*actor);
    let skill = engine_api::SkillId::new(*skill);
    if *tag != RESOLVE_DDD_LIUYE_CHOICE || observer != player {
        return Err(EffectError::Illegal("察势选择者已变化".into()));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal("取消察势仍包含牌".into()))
        };
    }
    let [selected] = selections else {
        return Err(EffectError::Illegal("察势须弃置一张牌".into()));
    };
    let card = CardId::new(*selected);
    if !originating_skill_is_enabled(tx, player, skill)
        || actor == player
        || tx.state().turn.current != actor
        || tx.state().turn.phase != Phase::Play
        || !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card)
    {
        return Err(EffectError::Illegal("察势弃牌条件已失效".into()));
    }
    let suit = tx
        .state()
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_suit(tx.state(), player, card))
        .map(crate::command::judge::suit_tag)
        .unwrap_or(0) as u8;
    let category = effective_card_category(tx.state(), player, card);
    if suit == 0 || category == 0 {
        return Err(EffectError::Illegal("察势弃牌没有有效花色或类型".into()));
    }
    discard_cards(tx, player, vec![card])?;
    tx.state_mut()
        .skills
        .get_mut(skill)
        .expect("validated 察势 skill")
        .set_ddd_chashi_state(actor, suit, category);
    Ok(())
}

pub(crate) fn ddd_baosanniang_is_male(state: &crate::state::GameState, player: PlayerId) -> bool {
    state
        .players
        .get(player)
        .and_then(|participant| state.content.character(participant.character_def))
        .is_some_and(|character| matches!(character.sex.as_str(), "male" | "double"))
}

pub(crate) fn ddd_zhilian_participants(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Option<Vec<PlayerId>> {
    let mut participants: Vec<_> = state
        .players
        .iter_live()
        .filter(|actor| {
            state
                .turn
                .flag(turn_flag::card_action_count_by_player(actor.id))
                > 0
        })
        .map(|actor| (actor.seat.raw(), actor.id, actor.alive))
        .collect();
    let alive = state.players.alive_by_seat();
    participants.sort_by_key(|(seat, actor, alive_actor)| {
        if *alive_actor {
            u32::from(crate::trigger::seat_rel_to_current(
                &alive,
                state.turn.current,
                *actor,
            ))
        } else {
            u32::from(*seat)
        }
    });
    let all_alive = participants.iter().all(|(_, _, alive)| *alive);
    let participants: Vec<_> = participants
        .into_iter()
        .map(|(_, actor, _)| actor)
        .collect();
    (participants.len() == 2
        && all_alive
        && participants.contains(&player)
        && participants
            .iter()
            .any(|actor| *actor != player && ddd_baosanniang_is_male(state, *actor)))
    .then_some(participants)
}

pub(crate) fn ddd_zhilian_gain_sha(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    let cards: Vec<_> = tx
        .state()
        .turn
        .entered_discard_cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|state| state.zone == discard && state.kind.is_sha())
        })
        .collect();
    if cards.is_empty() {
        Ok(Vec::new())
    } else {
        gain_event_cards(tx, player, &cards)
    }
}

pub(crate) fn ddd_jijian_name_kind(kind: CardKind) -> CardKind {
    if kind.is_sha() {
        CardKind::Sha
    } else {
        kind
    }
}

pub(crate) fn ddd_jijian_reveal_kind(
    state: &crate::state::GameState,
    holder: PlayerId,
    card: CardId,
) -> Option<CardKind> {
    let card = state.cards.get(card)?;
    let kind = crate::query::effective_card_kind(state, holder, card);
    (kind.is_sha() || (kind.is_trick() && !kind.is_delayed_trick())).then_some(kind)
}

pub(crate) fn ddd_jijian_virtual_options(
    tx: &mut Transaction,
    current: PlayerId,
    cards: &[CardId],
    used_kind: u8,
) -> Result<Vec<u32>, EffectError> {
    let holder = cards
        .iter()
        .find_map(|card| tx.state().cards.get(*card).map(|card| card.owner))
        .filter(|holder| holder.is_some())
        .unwrap_or(current);
    let used = CardKind::from_tag(used_kind).map(ddd_jijian_name_kind);
    let mut kinds = Vec::new();
    for card in cards {
        let Some(kind) = ddd_jijian_reveal_kind(tx.state(), holder, *card) else {
            continue;
        };
        if used == Some(ddd_jijian_name_kind(kind)) || kinds.contains(&kind) {
            continue;
        }
        let virtual_card = allocate_virtual_card_kind(tx, kind)?;
        let legal = crate::command::play::skill_card_target_spec(tx, current, virtual_card).is_ok();
        remove_unused_virtual_card(tx, virtual_card)?;
        if legal {
            kinds.push(kind);
        }
    }
    Ok(kinds
        .into_iter()
        .map(|kind| u32::from(kind.tag()))
        .collect())
}

pub(crate) fn ddd_jijian_schedule_next(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
    used_kind: CardKind,
    skill: engine_api::SkillId,
) {
    if !targets.is_empty() {
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::ResolveDddBaosanniang {
                player,
                target: targets[0],
                card: CardId::NONE,
                action: 12,
                cards,
                targets,
                used_kind: ddd_jijian_name_kind(used_kind).tag(),
                skill,
            },
        );
    }
}

pub(crate) fn ddd_jijian_play_virtual(
    tx: &mut Transaction,
    player: PlayerId,
    current: PlayerId,
    card: CardId,
    kind: CardKind,
    targets_for_card: &[PlayerId],
    cards: Vec<CardId>,
    remaining: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    ddd_jijian_schedule_next(tx, player, cards, remaining, kind, skill);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_CARD, card.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, i32::from(kind.tag()));
    let result = crate::command::play::handle_skill_card(tx, current, card, targets_for_card);
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_CARD, 0);
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_SKILL, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    result
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_ddd_baosanniang(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
    mut targets: Vec<PlayerId>,
    used_kind: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if matches!(action, 0 | 1) && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let Some(participants) = ddd_zhilian_participants(tx.state(), player) else {
                return Ok(Vec::new());
            };
            let can_discard = participants.iter().any(|participant| {
                !crate::query::discardable_hand_equip_cards(tx.state(), *participant).is_empty()
            });
            let mut options = vec![0];
            if can_discard {
                options.push(1);
            }
            let mut payload = vec![RESOLVE_DDD_BAOSANNIANG_CHOICE, 0, player.raw(), skill.raw()];
            payload.extend(participants.iter().map(|participant| participant.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                payload,
            )
        }
        1 => {
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || !ddd_baosanniang_is_male(tx.state(), target)
            {
                return Ok(Vec::new());
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("赍剑发动者手牌区不存在".into()))?;
            let selected = cards.first().copied().unwrap_or(card);
            if !tx.state().cards.get(selected).is_some_and(|state| {
                state.owner == player
                    && state.zone == hand
                    && crate::query::effective_card_kind(tx.state(), player, state).is_sha()
            }) {
                return Err(EffectError::Illegal("赍剑须交出一张当前手牌【杀】".into()));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("赍剑目标手牌区不存在".into()))?;
            let mut events = vec![DomainEvent::CardsRevealed {
                cards: vec![selected],
                to: Vec::new(),
            }];
            events.extend(give_cards_as(tx, player, vec![selected], to, hand)?);
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveDddBaosanniang {
                    player,
                    target,
                    card: CardId::NONE,
                    action: 10,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    used_kind: 0,
                    skill,
                },
            );
            Ok(events)
        }
        2 => ddd_zhilian_gain_sha(tx, player),
        3 => {
            while let Some(current) = targets.first().copied() {
                let options = crate::query::discardable_hand_equip_cards(tx.state(), current);
                if options.is_empty() {
                    targets.remove(0);
                    continue;
                }
                let mut payload = vec![
                    RESOLVE_DDD_BAOSANNIANG_CHOICE,
                    1,
                    player.raw(),
                    current.raw(),
                    skill.raw(),
                    targets.len() as u32,
                ];
                payload.extend(targets.iter().map(|target| target.raw()));
                payload.push(options.len() as u32);
                payload.extend(options.iter().map(|card| card.raw()));
                return open_choice(
                    tx,
                    current,
                    1,
                    1,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    payload,
                );
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddBaosanniang {
                    player,
                    target: PlayerId::NONE,
                    card: CardId::NONE,
                    action: 2,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    used_kind: 0,
                    skill,
                }],
            )
        }
        10 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("赍剑目标手牌区不存在".into()))?;
            let options: Vec<_> = tx
                .state()
                .hand_cards(target)
                .into_iter()
                .filter(|card| ddd_jijian_reveal_kind(tx.state(), target, *card).is_some())
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_DDD_BAOSANNIANG_CHOICE,
                2,
                player.raw(),
                target.raw(),
                skill.raw(),
                hand.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                1,
                options.len().min(2) as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        11 => {
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let alive = tx.state().players.alive_by_seat();
            let anchor = if tx.state().turn.current.is_some() {
                tx.state().turn.current
            } else {
                player
            };
            targets = vec![player, target];
            targets.sort_by_key(|participant| {
                crate::trigger::seat_rel_to_current(&alive, anchor, *participant)
            });
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddBaosanniang {
                    player,
                    target: targets[0],
                    card: CardId::NONE,
                    action: 12,
                    cards,
                    targets,
                    used_kind: 0,
                    skill,
                }],
            )
        }
        12 => {
            while let Some(current) = targets.first().copied() {
                if tx
                    .state()
                    .players
                    .get(current)
                    .is_some_and(|participant| participant.alive)
                {
                    let options = ddd_jijian_virtual_options(tx, current, &cards, used_kind)?;
                    if options.is_empty() {
                        return Ok(Vec::new());
                    }
                    let mut payload = vec![
                        RESOLVE_DDD_BAOSANNIANG_CHOICE,
                        3,
                        player.raw(),
                        current.raw(),
                        skill.raw(),
                        used_kind.into(),
                        cards.len() as u32,
                    ];
                    payload.extend(cards.iter().map(|card| card.raw()));
                    payload.push(targets.len() as u32);
                    payload.extend(targets.iter().map(|target| target.raw()));
                    payload.push(options.len() as u32);
                    payload.extend(options.iter().copied());
                    return open_choice(
                        tx,
                        current,
                        1,
                        1,
                        false,
                        options,
                        ChoiceKind::Custom,
                        ContinuationKind::Custom,
                        payload,
                    );
                }
                targets.remove(0);
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown 3D精选鲍三娘 resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_baosanniang_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload.first() != Some(&RESOLVE_DDD_BAOSANNIANG_CHOICE) {
        return Err(EffectError::EngineFault(
            "truncated 3D精选鲍三娘 choice".into(),
        ));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    match stage {
        0 => {
            if observer != player {
                return Err(EffectError::Illegal("枝连选择者已变化".into()));
            }
            if cancelled && selections.is_empty() {
                return Ok(());
            }
            if cancelled || selections.len() != 1 || payload.len() != 6 {
                return Err(EffectError::Illegal("枝连须选择摸牌、弃牌或取消".into()));
            }
            let skill = engine_api::SkillId::new(payload[3]);
            let participants: Vec<_> = payload[4..6].iter().copied().map(PlayerId::new).collect();
            if ddd_zhilian_participants(tx.state(), player).as_deref()
                != Some(participants.as_slice())
            {
                return Err(EffectError::Illegal("枝连参与角色已变化".into()));
            }
            match selections[0] {
                0 => {
                    let mut effects: Vec<_> = participants
                        .iter()
                        .copied()
                        .map(|participant| EffectRequest::DrawCards {
                            player: participant,
                            count: 1,
                        })
                        .collect();
                    effects.push(EffectRequest::ResolveDddBaosanniang {
                        player,
                        target: PlayerId::NONE,
                        card: CardId::NONE,
                        action: 2,
                        cards: Vec::new(),
                        targets: Vec::new(),
                        used_kind: 0,
                        skill,
                    });
                    run_effects(tx, effects).map(|_| ())
                }
                1 if participants.iter().any(|participant| {
                    !crate::query::discardable_hand_equip_cards(tx.state(), *participant).is_empty()
                }) =>
                {
                    run_effects(
                        tx,
                        vec![EffectRequest::ResolveDddBaosanniang {
                            player,
                            target: participants[0],
                            card: CardId::NONE,
                            action: 3,
                            cards: Vec::new(),
                            targets: participants,
                            used_kind: 0,
                            skill,
                        }],
                    )
                    .map(|_| ())
                }
                _ => Err(EffectError::Illegal("枝连分支已失效".into())),
            }
        }
        1 => {
            if cancelled || selections.len() != 1 || payload.len() < 8 {
                return Err(EffectError::Illegal("枝连弃牌选择无效".into()));
            }
            let current = PlayerId::new(payload[3]);
            if observer != current {
                return Err(EffectError::Illegal("枝连弃牌角色已变化".into()));
            }
            let skill = engine_api::SkillId::new(payload[4]);
            let target_count = payload[5] as usize;
            let options_index = 6 + target_count;
            if payload.len() <= options_index {
                return Err(EffectError::EngineFault(
                    "truncated 枝连 discard queue".into(),
                ));
            }
            let option_count = payload[options_index] as usize;
            if payload.len() != options_index + 1 + option_count
                || !payload[options_index + 1..].contains(&selections[0])
            {
                return Err(EffectError::Illegal("枝连弃牌候选已变化".into()));
            }
            let card = CardId::new(selections[0]);
            if !crate::query::discardable_hand_equip_cards(tx.state(), current).contains(&card) {
                return Err(EffectError::Illegal("枝连所弃牌已失效".into()));
            }
            let remaining: Vec<_> = payload[6..options_index]
                .iter()
                .copied()
                .map(PlayerId::new)
                .skip(1)
                .collect();
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player: current,
                        cards: vec![card],
                    },
                    EffectRequest::ResolveDddBaosanniang {
                        player,
                        target: remaining.first().copied().unwrap_or(PlayerId::NONE),
                        card: CardId::NONE,
                        action: 3,
                        cards: Vec::new(),
                        targets: remaining,
                        used_kind: 0,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        2 => {
            if cancelled || selections.is_empty() || selections.len() > 2 || payload.len() < 8 {
                return Err(EffectError::Illegal("赍剑须展示一至两张牌".into()));
            }
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let hand = ZoneId::new(payload[5]);
            let option_count = payload[6] as usize;
            if observer != target
                || payload.len() != 7 + option_count
                || selections.iter().any(|card| !payload[7..].contains(card))
            {
                return Err(EffectError::Illegal("赍剑展示候选已变化".into()));
            }
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            let mut distinct = std::collections::BTreeSet::new();
            if cards.iter().any(|card| {
                !distinct.insert(*card)
                    || !tx.state().cards.get(*card).is_some_and(|state| {
                        state.owner == target
                            && state.zone == hand
                            && ddd_jijian_reveal_kind(tx.state(), target, *card).is_some()
                    })
            }) {
                return Err(EffectError::Illegal("赍剑展示牌已离开目标手牌".into()));
            }
            let colors: std::collections::BTreeSet<_> = cards
                .iter()
                .map(|card| crate::query::effective_card_color_mask(tx.state(), target, *card))
                .collect();
            if colors.len() != 1 || colors.contains(&0) {
                return Err(EffectError::Illegal("赍剑展示牌必须颜色相同".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::RevealCards {
                        cards: cards.clone(),
                        to: Vec::new(),
                    },
                    EffectRequest::ResolveDddBaosanniang {
                        player,
                        target,
                        card: CardId::NONE,
                        action: 11,
                        cards,
                        targets: Vec::new(),
                        used_kind: 0,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        3 => resolve_ddd_jijian_kind_choice(tx, observer, payload, selections, cancelled),
        4 => resolve_ddd_jijian_target_choice(tx, observer, payload, selections, cancelled, false),
        5 => resolve_ddd_jijian_target_choice(tx, observer, payload, selections, cancelled, true),
        _ => Err(EffectError::EngineFault(
            "unknown 3D精选鲍三娘 choice stage".into(),
        )),
    }
}
