//! Effect primitive implementations — ddd 2 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

pub(crate) const RESOLVE_DDD_XIANGLANG_CHOICE: u32 = 0xDCDC_6260;
pub(crate) const DDD_XUNXUN_DEF: engine_api::SkillDefId = engine_api::SkillDefId::new(1_673_829_014);
pub(crate) const DDD_FUSI_GLOBAL_DEF: engine_api::SkillDefId =
    engine_api::SkillDefId::new(crate::command::skill::DDD_FUSI_GLOBAL_DEF);
pub(crate) const DDD_TUOJI_DEF: engine_api::SkillDefId = engine_api::SkillDefId::new(1_700_500_735);

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

pub(crate) const RESOLVE_DDD_YUJIN_CHOICE: u32 = 0xDCDC_6270;
pub(crate) const DDD_ZHENGJUN_HP: u8 = 1;
pub(crate) const DDD_ZHENGJUN_HAND: u8 = 2;
pub(crate) const DDD_ZHENGJUN_EQUIP: u8 = 4;

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

pub(crate) const RESOLVE_DDD_LIUYE_CHOICE: u32 = 0xDCDC_6280;

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

pub(crate) const RESOLVE_DDD_BAOSANNIANG_CHOICE: u32 = 0xDCDC_6290;

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

pub(crate) fn resolve_ddd_jijian_kind_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled || selections.len() != 1 || payload.len() < 10 {
        return Err(EffectError::Illegal("赍剑须选择一个可使用牌名".into()));
    }
    let player = PlayerId::new(payload[2]);
    let current = PlayerId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    let used_kind = u8::try_from(payload[5]).unwrap_or(0);
    let card_count = payload[6] as usize;
    let cards_end = 7 + card_count;
    if observer != current || payload.len() <= cards_end {
        return Err(EffectError::Illegal("赍剑用牌角色已变化".into()));
    }
    let cards: Vec<_> = payload[7..cards_end]
        .iter()
        .copied()
        .map(CardId::new)
        .collect();
    let target_count = payload[cards_end] as usize;
    let targets_end = cards_end + 1 + target_count;
    if payload.len() <= targets_end {
        return Err(EffectError::EngineFault("truncated 赍剑 user queue".into()));
    }
    let targets: Vec<_> = payload[cards_end + 1..targets_end]
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    if targets.first() != Some(&current) {
        return Err(EffectError::Illegal("赍剑当前用牌角色已变化".into()));
    }
    let option_count = payload[targets_end] as usize;
    if payload.len() != targets_end + 1 + option_count
        || !payload[targets_end + 1..].contains(&selections[0])
        || !ddd_jijian_virtual_options(tx, current, &cards, used_kind)?.contains(&selections[0])
    {
        return Err(EffectError::Illegal("赍剑可用牌名已变化".into()));
    }
    let kind = u8::try_from(selections[0])
        .ok()
        .and_then(CardKind::from_tag)
        .ok_or_else(|| EffectError::Illegal("赍剑牌名无效".into()))?;
    let virtual_card = allocate_virtual_card_kind(tx, kind)?;
    let remaining: Vec<_> = targets.into_iter().skip(1).collect();
    match crate::command::play::skill_card_target_spec(tx, current, virtual_card)? {
        crate::command::play::SkillCardTargetSpec::Immediate(card_targets) => {
            ddd_jijian_play_virtual(
                tx,
                player,
                current,
                virtual_card,
                kind,
                &card_targets,
                cards,
                remaining,
                skill,
            )
        }
        crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
            let mut next = vec![
                RESOLVE_DDD_BAOSANNIANG_CHOICE,
                4,
                player.raw(),
                current.raw(),
                virtual_card.raw(),
                u32::from(kind.tag()),
                skill.raw(),
                cards.len() as u32,
            ];
            next.extend(cards.iter().map(|card| card.raw()));
            next.push(remaining.len() as u32);
            next.extend(remaining.iter().map(|target| target.raw()));
            next.push(u32::from(min));
            next.push(u32::from(max));
            next.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                current,
                min,
                max,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
        crate::command::play::SkillCardTargetSpec::Pairs(pairs) => {
            let mut next = vec![
                RESOLVE_DDD_BAOSANNIANG_CHOICE,
                5,
                player.raw(),
                current.raw(),
                virtual_card.raw(),
                u32::from(kind.tag()),
                skill.raw(),
                cards.len() as u32,
            ];
            next.extend(cards.iter().map(|card| card.raw()));
            next.push(remaining.len() as u32);
            next.extend(remaining.iter().map(|target| target.raw()));
            next.push(pairs.len() as u32);
            for (first, second) in &pairs {
                next.push(first.raw());
                next.push(second.raw());
            }
            open_choice(
                tx,
                current,
                1,
                1,
                false,
                (0..pairs.len().min(u32::MAX as usize) as u32).collect(),
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
    }
}

pub(crate) fn resolve_ddd_jijian_target_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
    paired: bool,
) -> Result<(), EffectError> {
    if cancelled || payload.len() < 11 {
        return Err(EffectError::Illegal("赍剑虚拟牌目标选择无效".into()));
    }
    let player = PlayerId::new(payload[2]);
    let current = PlayerId::new(payload[3]);
    let card = CardId::new(payload[4]);
    let kind = u8::try_from(payload[5])
        .ok()
        .and_then(CardKind::from_tag)
        .ok_or_else(|| EffectError::EngineFault("bad 赍剑 virtual kind".into()))?;
    let skill = engine_api::SkillId::new(payload[6]);
    let card_count = payload[7] as usize;
    let cards_end = 8 + card_count;
    if observer != current || payload.len() <= cards_end {
        return Err(EffectError::Illegal("赍剑虚拟牌使用者已变化".into()));
    }
    let cards: Vec<_> = payload[8..cards_end]
        .iter()
        .copied()
        .map(CardId::new)
        .collect();
    let remaining_count = payload[cards_end] as usize;
    let remaining_end = cards_end + 1 + remaining_count;
    if payload.len() <= remaining_end {
        return Err(EffectError::EngineFault(
            "truncated 赍剑 remaining users".into(),
        ));
    }
    let remaining: Vec<_> = payload[cards_end + 1..remaining_end]
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    if paired {
        let pair_count = payload[remaining_end] as usize;
        if selections.len() != 1
            || payload.len() != remaining_end + 1 + pair_count.saturating_mul(2)
            || selections[0] as usize >= pair_count
        {
            return Err(EffectError::Illegal("赍剑借刀目标组合无效".into()));
        }
        let index = remaining_end + 1 + selections[0] as usize * 2;
        let selected = (
            PlayerId::new(payload[index]),
            PlayerId::new(payload[index + 1]),
        );
        let crate::command::play::SkillCardTargetSpec::Pairs(current_pairs) =
            crate::command::play::skill_card_target_spec(tx, current, card)?
        else {
            return Err(EffectError::Illegal("赍剑借刀目标形态已变化".into()));
        };
        if !current_pairs.contains(&selected) {
            return Err(EffectError::Illegal("赍剑借刀目标已失效".into()));
        }
        ddd_jijian_play_virtual(
            tx,
            player,
            current,
            card,
            kind,
            &[selected.0, selected.1],
            cards,
            remaining,
            skill,
        )
    } else {
        if payload.len() < remaining_end + 3 {
            return Err(EffectError::EngineFault(
                "truncated 赍剑 target contract".into(),
            ));
        }
        let min = payload[remaining_end] as usize;
        let max = payload[remaining_end + 1] as usize;
        let offered: Vec<_> = payload[remaining_end + 2..]
            .iter()
            .copied()
            .map(PlayerId::new)
            .collect();
        let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
        let mut distinct = std::collections::BTreeSet::new();
        let crate::command::play::SkillCardTargetSpec::Select {
            min: current_min,
            max: current_max,
            options,
        } = crate::command::play::skill_card_target_spec(tx, current, card)?
        else {
            return Err(EffectError::Illegal("赍剑目标形态已变化".into()));
        };
        if selected.len() < min
            || selected.len() > max
            || selected.len() < usize::from(current_min)
            || selected.len() > usize::from(current_max)
            || selected.iter().any(|target| {
                !distinct.insert(*target) || !offered.contains(target) || !options.contains(target)
            })
        {
            return Err(EffectError::Illegal("赍剑虚拟牌目标已失效".into()));
        }
        ddd_jijian_play_virtual(
            tx, player, current, card, kind, &selected, cards, remaining, skill,
        )
    }
}

pub(crate) const RESOLVE_DDD_ZHENJI_CHOICE: u32 = 0xDCDC_6300;
pub(crate) const DDD_MIAOXING_PILE_1: u32 = 919_529_013;
pub(crate) const DDD_MIAOXING_PILE_2: u32 = 2_949_101_455;

pub(crate) fn ddd_zhenji_pile_def(index: u8) -> Result<u32, EffectError> {
    match index {
        1 => Ok(DDD_MIAOXING_PILE_1),
        2 => Ok(DDD_MIAOXING_PILE_2),
        _ => Err(EffectError::Illegal("水相组号无效".into())),
    }
}

pub(crate) fn ddd_zhenji_move_to_pile(
    tx: &mut Transaction,
    cards: Vec<CardId>,
    pile: ZoneId,
    from: ZoneId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let moved = cards.clone();
    let events = move_cards(tx, cards, pile, Some(from))?;
    for card in moved {
        tx.state_mut()
            .cards
            .get_mut(card)
            .expect("moved water card remains live")
            .face_up = false;
    }
    Ok(events)
}

pub(crate) fn ddd_zhenji_exchange_all(
    tx: &mut Transaction,
    owner: PlayerId,
    target: PlayerId,
    pile_index: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    let pile_def = ddd_zhenji_pile_def(pile_index)?;
    let hand_cards = tx.state().hand_cards(target);
    let pile_cards = tx.state().skill_pile_cards(owner, pile_def);
    if hand_cards.is_empty() && pile_cards.is_empty() {
        return Err(EffectError::Illegal("浮世不能交换两个空区域".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("浮世目标没有手牌区".into()))?;
    let pile = ensure_skill_pile_zone(tx, owner, pile_def)?;
    let mut events = if hand_cards.is_empty() {
        Vec::new()
    } else {
        ddd_zhenji_move_to_pile(tx, hand_cards, pile, hand)?
    };
    if !pile_cards.is_empty() {
        events.extend(move_cards(tx, pile_cards, hand, Some(pile))?);
    }
    Ok(events)
}

pub(crate) fn ddd_zhenji_continue_after_events(
    tx: &mut Transaction,
    events: Vec<DomainEvent>,
    next: EffectRequest,
) -> Result<Vec<DomainEvent>, EffectError> {
    if events.is_empty() {
        run_effects(tx, vec![next])?;
    } else {
        tx.state_mut().resolution.deferred_tail_effects.push(next);
    }
    Ok(events)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_ddd_zhenji(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    action: u8,
    stage: u8,
    mut piles: Vec<u8>,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage == 0 && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => {
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            let size = hand.len().div_ceil(3);
            let options: Vec<u32> = (0..3)
                .filter(|index| index * size < hand.len())
                .map(|index| index as u32)
                .collect();
            let mut payload = vec![
                RESOLVE_DDD_ZHENJI_CHOICE,
                0,
                player.raw(),
                skill.raw(),
                hand.len() as u32,
            ];
            payload.extend(hand.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                payload,
            )
        }
        (0, 1) => {
            let selected = piles
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("淼形未选择起始手牌组".into()))?
                as usize;
            if tx.state().hand_cards(player) != cards || cards.is_empty() {
                return Err(EffectError::Illegal("淼形起始手牌已变化".into()));
            }
            let size = cards.len().div_ceil(3);
            let mut groups = Vec::new();
            for index in 0..3 {
                let start = index * size;
                if start >= cards.len() {
                    break;
                }
                groups.push(cards[start..(start + size).min(cards.len())].to_vec());
            }
            if selected >= groups.len() {
                return Err(EffectError::Illegal("淼形起始手牌组越界".into()));
            }
            groups.remove(selected);
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("淼形没有手牌区".into()))?;
            let mut events = Vec::new();
            for (index, group) in groups.into_iter().enumerate() {
                if group.is_empty() {
                    continue;
                }
                let pile =
                    ensure_skill_pile_zone(tx, player, ddd_zhenji_pile_def(index as u8 + 1)?)?;
                events.extend(ddd_zhenji_move_to_pile(tx, group, pile, hand)?);
            }
            Ok(events)
        }
        (1, 0) => {
            let hand_count = tx.state().hand_count(player) as usize;
            let options: Vec<u32> = [1u8, 2]
                .into_iter()
                .filter(|index| {
                    ddd_zhenji_pile_def(*index).is_ok_and(|pile| {
                        tx.state().skill_pile_cards(player, pile).len() != hand_count
                    })
                })
                .map(u32::from)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let forced = options.len() == 2;
            open_choice(
                tx,
                player,
                1,
                options.len() as u8,
                !forced,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![RESOLVE_DDD_ZHENJI_CHOICE, 1, player.raw(), skill.raw()],
            )
        }
        (1, 1) => {
            while let Some(index) = piles.first().copied() {
                let pile_def = ddd_zhenji_pile_def(index)?;
                let pile_cards = tx.state().skill_pile_cards(player, pile_def);
                let hand_count = tx.state().hand_count(player) as usize;
                if pile_cards.len() == hand_count {
                    piles.remove(0);
                    continue;
                }
                if pile_cards.len() > hand_count {
                    let count = pile_cards.len() - hand_count;
                    let mut payload = vec![
                        RESOLVE_DDD_ZHENJI_CHOICE,
                        2,
                        player.raw(),
                        skill.raw(),
                        piles.len() as u32,
                    ];
                    payload.extend(piles.iter().map(|index| u32::from(*index)));
                    return open_choice(
                        tx,
                        player,
                        count as u8,
                        count as u8,
                        false,
                        pile_cards.into_iter().map(CardId::raw).collect(),
                        ChoiceKind::SelectCards,
                        ContinuationKind::Custom,
                        payload,
                    );
                }
                let count = hand_count - pile_cards.len();
                let draw = tx
                    .state()
                    .zones
                    .find(ZoneKind::DrawPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("淼形没有牌堆".into()))?;
                let pile = ensure_skill_pile_zone(tx, player, pile_def)?;
                let mut events = Vec::new();
                for _ in 0..count {
                    if tx
                        .state()
                        .zones
                        .get(draw)
                        .is_none_or(|zone| zone.cards.is_empty())
                    {
                        reshuffle_discard_into_draw(tx)?;
                    }
                    let Some(card) = tx
                        .state()
                        .zones
                        .get(draw)
                        .and_then(|zone| zone.cards.last())
                        .copied()
                    else {
                        break;
                    };
                    events.extend(ddd_zhenji_move_to_pile(tx, vec![card], pile, draw)?);
                }
                piles.remove(0);
                if piles.is_empty() {
                    return Ok(events);
                }
                return ddd_zhenji_continue_after_events(
                    tx,
                    events,
                    EffectRequest::ResolveDddZhenji {
                        player,
                        target: PlayerId::NONE,
                        action: 1,
                        stage: 1,
                        piles,
                        cards: Vec::new(),
                        skill,
                    },
                );
            }
            Ok(Vec::new())
        }
        (1, 2) => {
            let index = piles
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("淼形没有待缩减的水相".into()))?;
            let pile_def = ddd_zhenji_pile_def(index)?;
            let pile = ensure_skill_pile_zone(tx, player, pile_def)?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("淼形没有弃牌堆".into()))?;
            let current: std::collections::BTreeSet<_> = tx
                .state()
                .skill_pile_cards(player, pile_def)
                .into_iter()
                .collect();
            let selected: std::collections::BTreeSet<_> = cards.iter().copied().collect();
            let excess = current
                .len()
                .saturating_sub(tx.state().hand_count(player) as usize);
            if cards.is_empty()
                || cards.len() != excess
                || selected.len() != cards.len()
                || !selected.is_subset(&current)
            {
                return Err(EffectError::Illegal("淼形移除的水相已失效".into()));
            }
            let events = move_cards(tx, cards, discard, Some(pile))?;
            piles.remove(0);
            if piles.is_empty() {
                return Ok(events);
            }
            ddd_zhenji_continue_after_events(
                tx,
                events,
                EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::NONE,
                    action: 1,
                    stage: 1,
                    piles,
                    cards: Vec::new(),
                    skill,
                },
            )
        }
        (2, 0) => {
            let hand_nonempty = tx.state().hand_count(player) > 0;
            let options: Vec<u32> = [1u8, 2]
                .into_iter()
                .filter(|index| {
                    hand_nonempty
                        || ddd_zhenji_pile_def(*index)
                            .is_ok_and(|pile| !tx.state().skill_pile_cards(player, pile).is_empty())
                })
                .map(u32::from)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
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
                vec![RESOLVE_DDD_ZHENJI_CHOICE, 3, player.raw(), skill.raw()],
            )
        }
        (2, 1) => {
            let index = piles
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("浮世未选择水相".into()))?;
            let events = ddd_zhenji_exchange_all(tx, player, player, index)?;
            ddd_zhenji_continue_after_events(
                tx,
                events,
                EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::NONE,
                    action: 2,
                    stage: 2,
                    piles: Vec::new(),
                    cards: Vec::new(),
                    skill,
                },
            )
        }
        (2, 2) => {
            let has_water = !tx
                .state()
                .skill_pile_cards(player, DDD_MIAOXING_PILE_1)
                .is_empty()
                || !tx
                    .state()
                    .skill_pile_cards(player, DDD_MIAOXING_PILE_2)
                    .is_empty();
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|participant| {
                    ddd_baosanniang_is_male(tx.state(), *participant)
                        && (has_water || tx.state().hand_count(*participant) > 0)
                })
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![RESOLVE_DDD_ZHENJI_CHOICE, 4, player.raw(), skill.raw()],
            )
        }
        (2, 3) => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
                || !ddd_baosanniang_is_male(tx.state(), target)
            {
                return Err(EffectError::Illegal("浮世男性目标已失效".into()));
            }
            let target_has_hand = tx.state().hand_count(target) > 0;
            let options: Vec<u32> = [1u8, 2]
                .into_iter()
                .filter(|index| {
                    target_has_hand
                        || ddd_zhenji_pile_def(*index)
                            .is_ok_and(|pile| !tx.state().skill_pile_cards(player, pile).is_empty())
                })
                .map(u32::from)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                true,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_ZHENJI_CHOICE,
                    5,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        (2, 4) => {
            let index = piles
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("浮世男性角色未选择水相".into()))?;
            ddd_zhenji_exchange_all(tx, player, target, index)
        }
        _ => Err(EffectError::EngineFault(
            "unknown 3D精选甄宓 continuation".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_zhenji_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload.first() != Some(&RESOLVE_DDD_ZHENJI_CHOICE) {
        return Err(EffectError::EngineFault(
            "truncated 3D精选甄宓 choice".into(),
        ));
    }
    let choice = payload[1];
    let player = PlayerId::new(payload[2]);
    if observer != player && choice != 5 {
        return Err(EffectError::Illegal("甄宓选择观察者不匹配".into()));
    }
    match choice {
        0 => {
            let skill = engine_api::SkillId::new(payload[3]);
            let count = payload.get(4).copied().unwrap_or(0) as usize;
            if cancelled || selections.len() != 1 || payload.len() != 5 + count {
                return Err(EffectError::Illegal("淼形必须保留一组起始手牌".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::NONE,
                    action: 0,
                    stage: 1,
                    piles: vec![selections[0] as u8],
                    cards: payload[5..].iter().copied().map(CardId::new).collect(),
                    skill,
                }],
            )?;
        }
        1 => {
            let skill = engine_api::SkillId::new(payload[3]);
            let current_hand = tx.state().hand_count(player) as usize;
            let legal: std::collections::BTreeSet<_> = [1u8, 2]
                .into_iter()
                .filter(|index| {
                    ddd_zhenji_pile_def(*index).is_ok_and(|pile| {
                        tx.state().skill_pile_cards(player, pile).len() != current_hand
                    })
                })
                .map(u32::from)
                .collect();
            let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
            if cancelled {
                if !selections.is_empty() || legal.len() != 1 {
                    return Err(EffectError::Illegal("淼形不能取消强制平衡".into()));
                }
                return Ok(());
            }
            if selections.is_empty()
                || selections.len() > 2
                || selected.len() != selections.len()
                || !selected.is_subset(&legal)
            {
                return Err(EffectError::Illegal("淼形平衡组选择无效".into()));
            }
            let mut piles: Vec<_> = selections.iter().map(|value| *value as u8).collect();
            piles.sort_unstable();
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::NONE,
                    action: 1,
                    stage: 1,
                    piles,
                    cards: Vec::new(),
                    skill,
                }],
            )?;
        }
        2 => {
            let skill = engine_api::SkillId::new(payload[3]);
            let count = payload.get(4).copied().unwrap_or(0) as usize;
            if cancelled || count == 0 || payload.len() != 5 + count {
                return Err(EffectError::Illegal("淼形移除水相选择无效".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::NONE,
                    action: 1,
                    stage: 2,
                    piles: payload[5..].iter().map(|value| *value as u8).collect(),
                    cards: selections.iter().copied().map(CardId::new).collect(),
                    skill,
                }],
            )?;
        }
        3 => {
            let skill = engine_api::SkillId::new(payload[3]);
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("浮世必须选择一组水相".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::NONE,
                    action: 2,
                    stage: 1,
                    piles: vec![selections[0] as u8],
                    cards: Vec::new(),
                    skill,
                }],
            )?;
        }
        4 => {
            let skill = engine_api::SkillId::new(payload[3]);
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal("取消浮世目标时不得选人".into()));
                }
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("浮世必须选择一名男性角色".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target: PlayerId::new(selections[0]),
                    action: 2,
                    stage: 3,
                    piles: Vec::new(),
                    cards: Vec::new(),
                    skill,
                }],
            )?;
        }
        5 => {
            if payload.len() != 5 {
                return Err(EffectError::EngineFault("truncated 浮世目标选择".into()));
            }
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            if observer != target {
                return Err(EffectError::Illegal("浮世男性角色观察者不匹配".into()));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal("取消浮世交换时不得选水相".into()));
                }
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("浮世男性角色必须选择一组水相".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhenji {
                    player,
                    target,
                    action: 2,
                    stage: 4,
                    piles: vec![selections[0] as u8],
                    cards: Vec::new(),
                    skill,
                }],
            )?;
        }
        _ => {
            return Err(EffectError::EngineFault("unknown 3D精选甄宓 choice".into()));
        }
    }
    Ok(())
}

pub(crate) const RESOLVE_DDD_ZHAOANG_CHOICE: u32 = 0xDCDC_6310;

pub(crate) fn ddd_fenji_total_damage(state: &crate::state::GameState) -> u32 {
    state
        .damage_dealt_totals
        .values()
        .copied()
        .fold(0u32, u32::saturating_add)
}

pub(crate) fn ddd_fenji_phase_valid(tx: &Transaction, player: PlayerId, action: u8) -> bool {
    tx.state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        && tx.state().turn.current == player
        && match action {
            0 => {
                tx.state().turn.phase == Phase::Draw
                    && tx.state().turn.draw_phase_fixed_count().is_none()
            }
            1 => tx.state().turn.phase == Phase::Discard,
            _ => false,
        }
}

pub(crate) fn ddd_fenji_play(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    card: CardId,
    kind: CardKind,
    targets: &[PlayerId],
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    let count = if action == 0 {
        ddd_fenji_total_damage(tx.state())
    } else {
        targets.len().min(u32::MAX as usize) as u32
    };
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveDddZhaoang {
            player,
            action,
            stage: 2,
            card,
            kind: kind.tag(),
            count,
            skill,
        });
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_CARD, card.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, i32::from(kind.tag()));
    let result = crate::command::play::handle_skill_card(tx, player, card, targets);
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_CARD, 0);
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_SKILL, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    result
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_ddd_zhaoang(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    stage: u8,
    card: CardId,
    kind: u8,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !ddd_fenji_phase_valid(tx, player, action)
            {
                return Ok(Vec::new());
            }
            let used_mask = tx
                .state()
                .skills
                .get(skill)
                .map_or(0, |instance| instance.content_flags);
            let options = crate::command::play::source_free_ordinary_trick_kinds(
                tx.state(),
                player,
                used_mask,
            );
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options
                    .into_iter()
                    .map(|candidate| u32::from(candidate.tag()))
                    .collect(),
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_ZHAOANG_CHOICE,
                    0,
                    player.raw(),
                    u32::from(action),
                    skill.raw(),
                ],
            )
        }
        1 => {
            if !ddd_fenji_phase_valid(tx, player, action) {
                return Err(EffectError::Illegal("奋计阶段已经变化".into()));
            }
            let kind = CardKind::from_tag(kind)
                .ok_or_else(|| EffectError::Illegal("奋计锦囊牌名无效".into()))?;
            let used_mask = tx
                .state()
                .skills
                .get(skill)
                .map_or(0, |instance| instance.content_flags);
            if !crate::command::play::source_free_ordinary_trick_kinds(
                tx.state(),
                player,
                used_mask,
            )
            .contains(&kind)
            {
                return Err(EffectError::Illegal("奋计锦囊已使用或已无合法目标".into()));
            }
            if !tx
                .state_mut()
                .skills
                .get_mut(skill)
                .is_some_and(|instance| instance.set_content_flag(kind.tag(), true))
            {
                return Err(EffectError::ContentFault("奋计无法记录已用牌名".into()));
            }
            if action == 0 {
                tx.state_mut().turn.set_draw_phase_fixed_count(0);
            } else {
                tx.state_mut().turn.set_flag(turn_flag::SKIP_DISCARD, 1);
            }
            let card = allocate_virtual_card_kind(tx, kind)?;
            match crate::command::play::skill_card_target_spec(tx, player, card)? {
                crate::command::play::SkillCardTargetSpec::Immediate(targets) => {
                    ddd_fenji_play(tx, player, action, card, kind, &targets, skill)?;
                    Ok(Vec::new())
                }
                crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
                    let mut payload = vec![
                        RESOLVE_DDD_ZHAOANG_CHOICE,
                        1,
                        player.raw(),
                        u32::from(action),
                        card.raw(),
                        u32::from(kind.tag()),
                        skill.raw(),
                        u32::from(min),
                        u32::from(max),
                    ];
                    payload.extend(options.iter().map(|target| target.raw()));
                    open_choice(
                        tx,
                        player,
                        min,
                        max,
                        false,
                        options.into_iter().map(PlayerId::raw).collect(),
                        ChoiceKind::SelectTargets,
                        ContinuationKind::Custom,
                        payload,
                    )
                }
                crate::command::play::SkillCardTargetSpec::Pairs(pairs) => {
                    let mut payload = vec![
                        RESOLVE_DDD_ZHAOANG_CHOICE,
                        2,
                        player.raw(),
                        u32::from(action),
                        card.raw(),
                        u32::from(kind.tag()),
                        skill.raw(),
                        pairs.len().min(u32::MAX as usize) as u32,
                    ];
                    for (first, second) in &pairs {
                        payload.push(first.raw());
                        payload.push(second.raw());
                    }
                    open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        (0..pairs.len().min(u32::MAX as usize) as u32).collect(),
                        ChoiceKind::Custom,
                        ContinuationKind::Custom,
                        payload,
                    )
                }
            }
        }
        2 => {
            if action == 0 {
                let damage = ddd_fenji_total_damage(tx.state()).saturating_sub(count);
                if damage == 0 {
                    Ok(Vec::new())
                } else {
                    run_effects(
                        tx,
                        vec![EffectRequest::DrawCards {
                            player,
                            count: damage,
                        }],
                    )
                }
            } else if action == 1 {
                let actual = count.min(
                    crate::query::discardable_hand_equip_cards(tx.state(), player)
                        .len()
                        .min(u32::MAX as usize) as u32,
                );
                if actual == 0 {
                    Ok(Vec::new())
                } else {
                    run_effects(
                        tx,
                        vec![EffectRequest::DiscardHandEquipCount {
                            player,
                            count: actual,
                            skill,
                        }],
                    )
                }
            } else {
                Err(EffectError::EngineFault("unknown 奋计 phase action".into()))
            }
        }
        _ => Err(EffectError::EngineFault(
            "unknown 3D精选赵昂 continuation".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_zhaoang_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5
        || payload.first() != Some(&RESOLVE_DDD_ZHAOANG_CHOICE)
        || observer != PlayerId::new(payload[2])
        || cancelled
    {
        return Err(EffectError::Illegal("奋计选择无效".into()));
    }
    let player = PlayerId::new(payload[2]);
    let action = payload[3] as u8;
    match payload[1] {
        0 => {
            let [kind] = selections else {
                return Err(EffectError::Illegal("奋计必须选择一个锦囊牌名".into()));
            };
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhaoang {
                    player,
                    action,
                    stage: 1,
                    card: CardId::NONE,
                    kind: *kind as u8,
                    count: 0,
                    skill: engine_api::SkillId::new(payload[4]),
                }],
            )?;
        }
        1 => {
            if payload.len() < 9 {
                return Err(EffectError::EngineFault("truncated 奋计目标续体".into()));
            }
            let card = CardId::new(payload[4]);
            let kind = CardKind::from_tag(payload[5] as u8)
                .ok_or_else(|| EffectError::Illegal("奋计目标牌名无效".into()))?;
            let skill = engine_api::SkillId::new(payload[6]);
            let min = payload[7] as usize;
            let max = payload[8] as usize;
            let offered: Vec<_> = payload[9..].iter().copied().map(PlayerId::new).collect();
            let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            let mut distinct = std::collections::BTreeSet::new();
            let crate::command::play::SkillCardTargetSpec::Select {
                min: current_min,
                max: current_max,
                options,
            } = crate::command::play::skill_card_target_spec(tx, player, card)?
            else {
                return Err(EffectError::Illegal("奋计目标形态已变化".into()));
            };
            if selected.len() < min
                || selected.len() > max
                || selected.len() < usize::from(current_min)
                || selected.len() > usize::from(current_max)
                || selected.iter().any(|target| {
                    !distinct.insert(*target)
                        || !offered.contains(target)
                        || !options.contains(target)
                })
            {
                return Err(EffectError::Illegal("奋计目标已失效".into()));
            }
            ddd_fenji_play(tx, player, action, card, kind, &selected, skill)?;
        }
        2 => {
            if payload.len() < 8 || selections.len() != 1 {
                return Err(EffectError::EngineFault("truncated 奋计借刀续体".into()));
            }
            let card = CardId::new(payload[4]);
            let kind = CardKind::from_tag(payload[5] as u8)
                .ok_or_else(|| EffectError::Illegal("奋计借刀牌名无效".into()))?;
            let skill = engine_api::SkillId::new(payload[6]);
            let pair_count = payload[7] as usize;
            if payload.len() != 8 + pair_count.saturating_mul(2)
                || selections[0] as usize >= pair_count
            {
                return Err(EffectError::Illegal("奋计借刀目标组合无效".into()));
            }
            let index = 8 + selections[0] as usize * 2;
            let selected = (
                PlayerId::new(payload[index]),
                PlayerId::new(payload[index + 1]),
            );
            let crate::command::play::SkillCardTargetSpec::Pairs(current) =
                crate::command::play::skill_card_target_spec(tx, player, card)?
            else {
                return Err(EffectError::Illegal("奋计借刀目标形态已变化".into()));
            };
            if !current.contains(&selected) {
                return Err(EffectError::Illegal("奋计借刀目标已失效".into()));
            }
            ddd_fenji_play(
                tx,
                player,
                action,
                card,
                kind,
                &[selected.0, selected.1],
                skill,
            )?;
        }
        _ => return Err(EffectError::EngineFault("unknown 奋计 choice".into())),
    }
    Ok(())
}

pub(crate) const RESOLVE_DDD_LIUBA_CHOICE: u32 = 0xDCDC_6320;
pub(crate) const DDD_YULV_DEF: u32 = 694_263_366;

pub(crate) fn ddd_liuba_he_cards(state: &crate::state::GameState, player: PlayerId) -> Vec<CardId> {
    let mut cards = state.hand_cards(player);
    if let Some(zone) = state.zones.find(ZoneKind::Equip, player) {
        cards.extend(
            state
                .zones
                .get(zone)
                .into_iter()
                .flat_map(|zone| zone.cards.iter().copied()),
        );
    }
    cards.sort_by_key(|card| card.raw());
    cards
}

pub(crate) fn ddd_liuba_open_distribution_subset(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    remaining: Vec<CardId>,
    assigned: Vec<(CardId, PlayerId)>,
) -> Result<Vec<DomainEvent>, EffectError> {
    if remaining.is_empty() {
        let (cards, targets): (Vec<_>, Vec<_>) = assigned.into_iter().unzip();
        return run_effects(
            tx,
            vec![EffectRequest::ResolveDddLiuba {
                player,
                actor: player,
                action: 14,
                cards,
                targets,
                skill,
            }],
        );
    }
    if remaining.iter().any(|card| {
        !tx.state().cards.get(*card).is_some_and(|card| {
            tx.state()
                .zones
                .get(card.zone)
                .is_some_and(|zone| zone.kind == ZoneKind::Processing)
        })
    }) {
        return Err(EffectError::Illegal("丰政待分配牌已离开处理区".into()));
    }
    let mut payload = vec![
        RESOLVE_DDD_LIUBA_CHOICE,
        1,
        player.raw(),
        skill.raw(),
        remaining.len() as u32,
    ];
    payload.extend(remaining.iter().map(|card| card.raw()));
    payload.push(assigned.len() as u32);
    for (card, target) in assigned {
        payload.extend([card.raw(), target.raw()]);
    }
    if remaining.len() == 1 {
        payload[1] = 2;
        payload.push(1);
        payload.push(remaining[0].raw());
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            tx.state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            payload,
        );
    }
    open_choice(
        tx,
        player,
        1,
        remaining.len().min(u8::MAX as usize) as u8,
        false,
        remaining.iter().map(|card| card.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn ddd_liuba_finish_distribution(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    assigned: Vec<(CardId, PlayerId)>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("丰政处理区不存在".into()))?;
    let mut groups = std::collections::BTreeMap::<PlayerId, Vec<CardId>>::new();
    for (card, target) in assigned {
        if !tx
            .state()
            .cards
            .get(card)
            .is_some_and(|card| card.zone == processing)
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|target| target.alive)
        {
            return Err(EffectError::Illegal("丰政最终分配状态失效".into()));
        }
        groups.entry(target).or_default().push(card);
    }
    let lose = groups.values().any(|cards| cards.len() > 2);
    let mut events = Vec::new();
    for (target, cards) in groups {
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, target)
            .ok_or_else(|| EffectError::EngineFault("丰政目标手牌区不存在".into()))?;
        events.extend(move_cards(tx, cards, hand, Some(processing))?);
    }
    if lose {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::RemoveSkill { skill });
    }
    let _ = player;
    Ok(events)
}

pub(crate) fn ddd_liuba_open_yulv_exchange(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let hand = tx.state().hand_cards(player);
    if hand.is_empty() {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ResolveDddLiuba {
                player,
                actor: player,
                action: 13,
                cards: Vec::new(),
                targets: Vec::new(),
                skill,
            });
        return run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }]);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        hand.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![RESOLVE_DDD_LIUBA_CHOICE, 6, player.raw(), skill.raw()],
    )
}

pub(crate) fn resolve_ddd_liuba(
    tx: &mut Transaction,
    player: PlayerId,
    _actor: PlayerId,
    action: u8,
    mut cards: Vec<CardId>,
    _targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if action <= 4 && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        // 丰政：点数和为 13 的任意张手牌作为完整【无中生有】使用。
        0 => {
            if !crate::command::skill::ddd_fengzheng_provider_exists(tx.state())
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || cards.is_empty()
            {
                return Err(EffectError::Illegal("丰政转化时机无效".into()));
            }
            cards.sort_by_key(|card| card.raw());
            cards.dedup();
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let total = cards.iter().try_fold(0u16, |sum, card| {
                tx.state().cards.get(*card).and_then(|card| {
                    (Some(card.zone) == hand).then(|| {
                        sum.saturating_add(u16::from(crate::query::effective_card_number(
                            tx.state(),
                            player,
                            card,
                        )))
                    })
                })
            });
            if total != Some(13) {
                return Err(EffectError::Illegal("丰政手牌点数和必须为 13".into()));
            }
            let round = tx.state().turn.round;
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.round_fire_round = round;
            }
            let carrier = cards[0];
            let extras = &cards[1..];
            tx.state_mut()
                .turn
                .set_flag(turn_flag::VIEW_AS_CARD, carrier.raw() as i32);
            tx.state_mut()
                .turn
                .set_flag(turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
            tx.state_mut().turn.set_flag(
                turn_flag::VIEW_AS_LOGICAL_KIND,
                i32::from(CardKind::TrickWuzhong.tag()),
            );
            let result = crate::command::play::play_as_kind_with_extra_damage_flags(
                tx,
                player,
                carrier,
                CardKind::TrickWuzhong,
                &[player],
                false,
                false,
                &[],
                false,
                true,
                0,
                extras,
                0,
            );
            tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_CARD, 0);
            tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_SKILL, 0);
            tx.state_mut()
                .turn
                .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, 0);
            result.map(|_| Vec::new())
        }
        // 丰政轮首：本轮只提示一次，选择是否开始分配。
        1 => {
            let round = tx.state().turn.round;
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.round_fire_round = round;
            }
            let count = tx
                .state()
                .skills
                .sorted_live()
                .iter()
                .filter(|instance| {
                    instance.def.raw() == crate::command::skill::DDD_FENGZHENG_GLOBAL_DEF
                        && instance.round_fire_round != 0
                        && instance.round_fire_round.saturating_add(1) == round
                })
                .count();
            if count == 0 {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                vec![1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_LIUBA_CHOICE,
                    0,
                    player.raw(),
                    skill.raw(),
                    count as u32,
                ],
            )
        }
        // 安装每名角色自己的 non-gainable 全局技实例。
        2 => {
            let mut events = Vec::new();
            for target in tx.state().players.alive_by_seat() {
                let exists = tx.state().players.get(target).is_some_and(|participant| {
                    participant.skills.iter().any(|owned| {
                        tx.state().skills.get(*owned).is_some_and(|instance| {
                            instance.def.raw() == crate::command::skill::DDD_FENGZHENG_GLOBAL_DEF
                        })
                    })
                });
                if !exists {
                    events.extend(
                        grant_skill_instance(
                            tx,
                            target,
                            engine_api::SkillDefId::new(
                                crate::command::skill::DDD_FENGZHENG_GLOBAL_DEF,
                            ),
                            Vec::new(),
                        )?
                        .1,
                    );
                }
            }
            Ok(events)
        }
        // 玉律起始：先摸一张，嵌套事件结束后再置牌。
        3 => {
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveDddLiuba {
                    player,
                    actor: player,
                    action: 11,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    skill,
                });
            run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }])
        }
        // 玉律②：记录本回合序号后由拥有者选择摸牌或弃牌。
        4 => {
            let actor = tx.state().turn.current;
            let ordinal = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .map(|instance| {
                    instance.turn_fire_count = instance.turn_fire_count.saturating_add(1);
                    instance.turn_fire_count
                })
                .unwrap_or(0);
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
                    RESOLVE_DDD_LIUBA_CHOICE,
                    3,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                    ordinal,
                ],
            )
        }
        // 丰政顶牌进入处理区后的第一次分配。
        10 => ddd_liuba_open_distribution_subset(tx, player, skill, cards, Vec::new()),
        // 玉律摸牌后选择一张手牌置于公开技能牌堆。
        11 => {
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![RESOLVE_DDD_LIUBA_CHOICE, 4, player.raw(), skill.raw()],
            )
        }
        // 第二次玉律后的交换入口。
        12 => ddd_liuba_open_yulv_exchange(tx, player, skill),
        // 无手牌先摸一张后的交换入口。
        13 => {
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() {
                Ok(Vec::new())
            } else {
                open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    hand.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    vec![RESOLVE_DDD_LIUBA_CHOICE, 6, player.raw(), skill.raw()],
                )
            }
        }
        // 分配选择全部完成后，在一个效果段内统一移动全部牌。
        14 => {
            if cards.len() != _targets.len() {
                return Err(EffectError::EngineFault("丰政最终分配长度不一致".into()));
            }
            ddd_liuba_finish_distribution(
                tx,
                player,
                skill,
                cards.into_iter().zip(_targets).collect(),
            )
        }
        // 交换“玉律”，两次移动属于同一效果段。
        15 => {
            let [new_card] = cards.as_slice() else {
                return Err(EffectError::EngineFault("玉律交换牌数量无效".into()));
            };
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("玉律拥有者手牌区不存在".into()))?;
            if !tx
                .state()
                .cards
                .get(*new_card)
                .is_some_and(|card| card.zone == hand)
            {
                return Err(EffectError::Illegal("玉律交换手牌已失效".into()));
            }
            let old = tx
                .state()
                .skill_pile_cards(player, DDD_YULV_DEF)
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("玉律牌不存在".into()))?;
            let pile = ensure_skill_pile_zone(tx, player, DDD_YULV_DEF)?;
            let mut events = move_cards(tx, vec![old], hand, Some(pile))?;
            events.extend(move_cards(tx, vec![*new_card], pile, Some(hand))?);
            Ok(events)
        }
        _ => Err(EffectError::EngineFault("unknown 3D精选刘巴 action".into())),
    }
}
