//! Effect primitive implementations — shenhua 2 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

/// 奇制回合计数：技能状态 QZ1<round u32><count u8>。
pub(crate) fn qizhi_count(tx: &Transaction, skill: engine_api::SkillId) -> u32 {
    tx.state()
        .skills
        .get(skill)
        .map(|instance| qizhi_count_from_blob(tx, &instance.state_blob))
        .unwrap_or(0)
}

pub(crate) fn qizhi_count_from_blob(tx: &Transaction, blob: &[u8]) -> u32 {
    let round = tx.state().turn.round;
    (blob.len() == 8 && &blob[..3] == b"QZ1")
        .then(|| {
            let marked_round = u32::from_le_bytes(blob[3..7].try_into().unwrap());
            (marked_round == round).then(|| blob[7])
        })
        .flatten()
        .map(u32::from)
        .unwrap_or(0)
}

pub(crate) fn qizhi_increment(tx: &mut Transaction, skill: engine_api::SkillId) {
    let round = tx.state().turn.round;
    let count = qizhi_count(tx, skill).min(255) as u8;
    let mut blob = b"QZ1".to_vec();
    blob.extend_from_slice(&round.to_le_bytes());
    blob.push(count.saturating_add(1));
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = blob;
    }
}

/// 奇制：弃置一名非目标角色的一张手牌/装备牌并令其摸一张牌。
pub(crate) fn resolve_qizhi(
    tx: &mut Transaction,
    player: PlayerId,
    targets: &[PlayerId],
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
            {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| {
                    candidate != player
                        && !targets.contains(&candidate)
                        && (tx.state().hand_count(candidate) > 0
                            || tx
                                .state()
                                .zones
                                .find(ZoneKind::Equip, candidate)
                                .is_some_and(|zone| {
                                    tx.state()
                                        .zones
                                        .get(zone)
                                        .is_some_and(|zone| !zone.cards.is_empty())
                                }))
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![RESOLVE_QIZHI_TARGET, 1, player.raw(), skill.raw()],
            )
        }
        2 => {
            qizhi_increment(tx, skill);
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "奇制 stage must resolve through its choice continuation".into(),
        )),
    }
}

/// 进趋：出牌阶段结束时摸两张并将手牌弃置至本回合奇制次数张。
pub(crate) fn resolve_jinqu(
    tx: &mut Transaction,
    player: PlayerId,
    qizhi_def: engine_api::SkillDefId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::End
            {
                return Ok(Vec::new());
            }
            run_effects(tx, vec![EffectRequest::DrawCards { player, count: 2 }])?;
            let keep = skill_instance_by_def(tx, player, qizhi_def)
                .and_then(|id| tx.state().skills.get(id))
                .map(|instance| qizhi_count_from_blob(tx, &instance.state_blob))
                .unwrap_or(0);
            let discard = tx.state().hand_count(player).saturating_sub(keep);
            if discard == 0 {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                discard.min(u8::MAX as u32) as u8,
                discard.min(u8::MAX as u32) as u8,
                true,
                tx.state()
                    .hand_cards(player)
                    .into_iter()
                    .map(CardId::raw)
                    .collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_JINQU_DISCARD,
                    1,
                    player.raw(),
                    qizhi_def.raw(),
                    skill.raw(),
                ],
            )
        }
        _ => Err(EffectError::ContentFault(
            "进趋 stage must resolve through its choice continuation".into(),
        )),
    }
}

/// 荐相：令手牌数最少的一名角色摸一张牌。
pub(crate) fn resolve_jianxiang(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            let min_hand = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|candidate| tx.state().hand_count(candidate))
                .min()
                .unwrap_or(0);
            let fewest_hand: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| tx.state().hand_count(candidate) == min_hand)
                .collect();
            GuardedChoice::new(
                skill,
                Participants::owner(player),
                ContinuationKind::Custom,
                vec![RESOLVE_JIANXIANG_TARGET, 1, player.raw(), skill.raw()],
            )
            .shape(ChoiceShape::one_optional())
            .open_targets(tx, fewest_hand)
        }
        _ => Err(stage_must_resume_via_continuation("荐相")),
    }
}

/// 审时转换技分支读取/翻转。
pub(crate) fn shenshi_branch(tx: &Transaction, skill: engine_api::SkillId) -> u8 {
    tx.state()
        .skills
        .get(skill)
        .and_then(|instance| {
            (instance.state_blob.len() >= 4 && &instance.state_blob[..3] == b"SS1")
                .then(|| instance.state_blob[3])
        })
        .unwrap_or(0)
}

/// 审时：阳——交牌给手牌最多者并造成 1 点伤害，死亡可令一名角色摸至四张；
/// 阴——受伤后交来源一张牌，其回合结束时追踪牌仍在其手/装备区则你摸至四张。
pub(crate) fn resolve_shenshi(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 | 1 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
            {
                return Ok(Vec::new());
            }
            let branch = shenshi_branch(tx, skill);
            // 翻转转换技分支。
            let mut blob = b"SS1".to_vec();
            blob.push(if branch == 0 { 1 } else { 0 });
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob = blob;
            }
            let yang = stage == 0;
            if !yang {
                // 阴：受伤后观看来源手牌并交其一张牌（记录追踪牌）。
                if source == PlayerId::NONE
                    || !tx.state().players.get(source).is_some_and(|p| p.alive)
                {
                    return Ok(Vec::new());
                }
            }
            let options: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .chain(
                    tx.state()
                        .zones
                        .find(ZoneKind::Equip, player)
                        .map(|zone| {
                            tx.state()
                                .zones
                                .get(zone)
                                .map(|zone| zone.cards.clone())
                                .unwrap_or_default()
                        })
                        .unwrap_or_default(),
                )
                .collect();
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
                    RESOLVE_SHENSHI_GIVE,
                    if yang { 2 } else { 1 },
                    player.raw(),
                    source.raw(),
                    skill.raw(),
                ],
            )
        }
        4 => {
            // 回合结束检查（阴的追踪牌）。
            let blob = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| instance.state_blob.clone())
                .unwrap_or_default();
            let tracked = (blob.len() == 12 && &blob[..3] == b"SS1").then(|| {
                (
                    u32::from_le_bytes(blob[4..8].try_into().unwrap()),
                    CardId::new(u32::from_le_bytes(blob[8..12].try_into().unwrap())),
                )
            });
            if let Some((marked_source, card)) = tracked {
                let still_held = tx.state().cards.get(card).is_some_and(|value| {
                    tx.state().zones.get(value.zone).is_some_and(|zone| {
                        zone.kind == ZoneKind::Hand || zone.kind == ZoneKind::Equip
                    }) && value.owner == PlayerId::new(marked_source)
                });

                if still_held {
                    let draw = 4u32.saturating_sub(tx.state().hand_count(player));
                    if draw > 0 {
                        return run_effects(
                            tx,
                            vec![EffectRequest::DrawCards {
                                player,
                                count: draw,
                            }],
                        );
                    }
                }
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob.clear();
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "审时 stage must resolve through its choice continuation".into(),
        )),
    }
}

/// 「任」技能牌堆的牌（明任 def = 本技能 def）。
pub(crate) fn mingren_pile_cards(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Vec<CardId> {
    let def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .unwrap_or(0);
    tx.state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player && pile.skill_def == def)
        .and_then(|pile| tx.state().zones.get(pile.zone))
        .map(|zone| zone.cards.clone())
        .unwrap_or_default()
}

pub(crate) fn mingren_pile_cards_by_def(
    tx: &Transaction,
    player: PlayerId,
    def: engine_api::SkillDefId,
) -> Vec<CardId> {
    tx.state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player && pile.skill_def == def.raw())
        .and_then(|pile| tx.state().zones.get(pile.zone))
        .map(|zone| zone.cards.clone())
        .unwrap_or_default()
}

/// 「任」的颜色掩码（1 红 / 2 黑）。
pub(crate) fn mingren_color_mask(tx: &Transaction, player: PlayerId, skill: engine_api::SkillId) -> u8 {
    mingren_pile_cards(tx, player, skill)
        .first()
        .and_then(|card| tx.state().cards.get(*card))
        .map(|value| crate::query::card_color_mask_for_suit(value.suit))
        .unwrap_or(0)
}

/// 明任：首轮准备阶段摸两张并将一张手牌置于「任」；结束阶段可选手牌替换。
pub(crate) fn resolve_mingren(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || tx.state().turn.round != 1
                || !mingren_pile_cards(tx, player, skill).is_empty()
            {
                return Ok(Vec::new());
            }
            run_effects(tx, vec![EffectRequest::DrawCards { player, count: 2 }])?;
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![RESOLVE_MINGREN_PLACE, 1, player.raw(), skill.raw()],
            )
        }
        2 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || mingren_pile_cards(tx, player, skill).is_empty()
            {
                return Ok(Vec::new());
            }
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![RESOLVE_MINGREN_REPLACE, 3, player.raw(), skill.raw()],
            )
        }
        _ => Err(EffectError::ContentFault(
            "明任 stage must resolve through its choice continuation".into(),
        )),
    }
}

/// 贞良转换技分支读取/翻转。
pub(crate) fn zhenliang_branch(tx: &Transaction, skill: engine_api::SkillId) -> u8 {
    tx.state()
        .skills
        .get(skill)
        .and_then(|instance| {
            (instance.state_blob.len() >= 4 && &instance.state_blob[..3] == b"ZL1")
                .then(|| instance.state_blob[3])
        })
        .unwrap_or(0)
}

/// 贞良：阴——弃一张与「任」同色的牌并对攻击范围内角色造成 1 点伤害；
/// 阳——回合外使用/打出同色牌后令一名角色摸一张牌。
pub(crate) fn resolve_zhenliang(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    mingren_def: engine_api::SkillDefId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
            {
                return Ok(Vec::new());
            }
            let branch = zhenliang_branch(tx, skill);
            let mut blob = b"ZL1".to_vec();
            blob.push(if branch == 0 { 1 } else { 0 });
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob = blob;
            }
            if branch == 0 {
                // 阴：弃一张与「任」同色的手牌/装备牌。
                let color = mingren_pile_cards_by_def(tx, player, mingren_def)
                    .first()
                    .and_then(|card| tx.state().cards.get(*card))
                    .map(|value| crate::query::card_color_mask_for_suit(value.suit))
                    .unwrap_or(0);
                if color == 0 {
                    return Ok(Vec::new());
                }
                let options: Vec<_> = tx
                    .state()
                    .hand_cards(player)
                    .into_iter()
                    .chain(
                        tx.state()
                            .zones
                            .find(ZoneKind::Equip, player)
                            .map(|zone| {
                                tx.state()
                                    .zones
                                    .get(zone)
                                    .map(|zone| zone.cards.clone())
                                    .unwrap_or_default()
                            })
                            .unwrap_or_default(),
                    )
                    .filter(|&candidate| {
                        tx.state().cards.get(candidate).is_some_and(|value| {
                            crate::query::card_color_mask_for_suit(value.suit) == color
                        })
                    })
                    .collect();
                if options.is_empty() {
                    return Ok(Vec::new());
                }
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    true,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    vec![RESOLVE_ZHENLIANG_DISCARD, 1, player.raw(), skill.raw()],
                );
            }
            // 阳：回合外使用/打出的牌与「任」同色时令一名角色摸一张。
            if source != player || tx.state().turn.current == player || card == CardId::NONE {
                return Ok(Vec::new());
            }
            let color = mingren_pile_cards_by_def(tx, player, mingren_def)
                .first()
                .and_then(|card| tx.state().cards.get(*card))
                .map(|value| crate::query::card_color_mask_for_suit(value.suit))
                .unwrap_or(0);
            let matches_color =
                tx.state().cards.get(card).is_some_and(|value| {
                    crate::query::card_color_mask_for_suit(value.suit) == color
                });
            if color == 0 || !matches_color {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx.state().players.alive_by_seat();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![RESOLVE_ZHENLIANG_DRAW, 3, player.raw(), skill.raw()],
            )
        }
        _ => Err(EffectError::ContentFault(
            "贞良 stage must resolve through its choice continuation".into(),
        )),
    }
}

/// 本回合弃牌数（弃牌阶段权威计数）。
pub(crate) fn turn_discard_count(tx: &Transaction, player: PlayerId) -> u32 {
    tx.state()
        .turn
        .flag(turn_flag::DISCARD_PHASE_DISCARDED)
        .max(0) as u32
}

/// 溃诛：弃牌阶段结束后按本回合弃牌数 X 选择摸牌或伤害。
pub(crate) fn resolve_kuizhu(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || tx.state().turn.current != player
            {
                return Ok(Vec::new());
            }
            let x = turn_discard_count(tx, player);
            if x == 0 {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx.state().players.alive_by_seat();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                2,
                true,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![RESOLVE_KUIZHU_DRAW, 1, player.raw(), skill.raw(), x],
            )
        }
        1 => {
            // 摸牌分支：至多 X 名角色各摸一张。
            let x = *tx.state().resolution.resume_payload.get(4).unwrap_or(&0) as usize;
            let options: Vec<_> = tx.state().players.alive_by_seat();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                x.min(8) as u8,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![RESOLVE_KUIZHU_DRAW, 2, player.raw(), skill.raw(), x as u32],
            )
        }
        2 => {
            let x = *tx.state().resolution.resume_payload.get(4).unwrap_or(&0) as usize;
            let options: Vec<_> = tx.state().players.alive_by_seat();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                x.min(8) as u8,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_KUIZHU_DAMAGE,
                    3,
                    player.raw(),
                    skill.raw(),
                    x as u32,
                ],
            )
        }
        _ => Err(EffectError::ContentFault(
            "溃诛 stage must resolve through its choice continuation".into(),
        )),
    }
}

/// 掣政：出牌阶段结束若本回合用牌数少于攻击范围不包含你的角色数，弃置其一张牌。
pub(crate) fn resolve_zhizheng(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            let out_of_range: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| {
                    candidate != player
                        && !crate::effect::dcchiying_in_range(tx.state(), candidate, player)
                })
                .collect();
            if out_of_range.is_empty() {
                return Ok(Vec::new());
            }
            // 本回合已用牌数达到攻击范围外角色数时不再发动。
            let used = tx
                .state()
                .turn
                .flag(turn_flag::card_action_count_by_player(player))
                .max(0) as u32;
            if used >= out_of_range.len() as u32 {
                return Ok(Vec::new());
            }
            let with_cards: Vec<_> = out_of_range
                .into_iter()
                .filter(|&candidate| {
                    tx.state().hand_count(candidate) > 0
                        || tx
                            .state()
                            .zones
                            .find(ZoneKind::Equip, candidate)
                            .is_some_and(|zone| {
                                tx.state()
                                    .zones
                                    .get(zone)
                                    .is_some_and(|zone| !zone.cards.is_empty())
                            })
                })
                .collect();
            GuardedChoice::new(
                skill,
                Participants::owner(player),
                ContinuationKind::Custom,
                vec![RESOLVE_ZHIZHENG_CARD, 1, player.raw(), skill.raw()],
            )
            .during(TurnContext::OwnersTurn)
            .shape(ChoiceShape::one_optional())
            .open_targets(tx, with_cards)
        }
        _ => Err(stage_must_resume_via_continuation("掣政")),
    }
}

/// 立军（主公技）：吴势力角色出牌阶段使用实体【杀】后你获得该牌并令其摸一张。
pub(crate) fn resolve_lijun(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || !tx.state().players.get(source).is_some_and(|p| p.alive)
                || source == player
                || tx.state().turn.current != source
                || tx.state().turn.phase != Phase::Play
                || card == CardId::NONE
            {
                return Ok(Vec::new());
            }
            let is_wu = tx.state().player_group(source) == Some("wu");
            let is_sha = tx.state().cards.get(card).is_some_and(|value| {
                crate::query::effective_card_kind(tx.state(), source, value).is_sha()
            });
            if !is_wu || !is_sha {
                return Ok(Vec::new());
            }
            // 你获得该实体牌；其摸一张牌。
            let owner_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no lord hand zone".into()))?;
            let from_zone = tx.state().cards.get(card).map(|value| value.zone);
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: vec![card],
                        to: owner_hand,
                        from: from_zone,
                    },
                    EffectRequest::DrawCards {
                        player: source,
                        count: 1,
                    },
                ],
            )
        }
        _ => Err(EffectError::ContentFault(
            "立军 stage must resolve through its choice continuation".into(),
        )),
    }
}

/// 箜声牌堆（键 = 箜声 def）。
pub(crate) fn kongsheng_pile_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let def = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player)
        .map(|pile| pile.skill_def);
    match def {
        Some(def) => tx
            .state()
            .skill_piles
            .iter()
            .find(|pile| pile.owner == player && pile.skill_def == def)
            .and_then(|pile| tx.state().zones.get(pile.zone))
            .map(|zone| zone.cards.clone())
            .unwrap_or_default(),
        None => Vec::new(),
    }
}

pub(crate) fn kongsheng_pile_cards_by_def(
    tx: &Transaction,
    player: PlayerId,
    def: engine_api::SkillDefId,
) -> Vec<CardId> {
    tx.state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player && pile.skill_def == def.raw())
        .and_then(|pile| tx.state().zones.get(pile.zone))
        .map(|zone| zone.cards.clone())
        .unwrap_or_default()
}

/// 良姻每回合一次计数（技能状态 LY1<round u32><count u8>）。
pub(crate) fn liangyin_used(tx: &Transaction, skill: engine_api::SkillId) -> bool {
    let round = tx.state().turn.round;
    tx.state()
        .skills
        .get(skill)
        .and_then(|instance| {
            (instance.state_blob.len() == 8 && &instance.state_blob[..3] == b"LY1").then(|| {
                let marked_round =
                    u32::from_le_bytes(instance.state_blob[3..7].try_into().unwrap());
                marked_round == round
            })
        })
        .unwrap_or(false)
}

pub(crate) fn liangyin_mark(tx: &mut Transaction, skill: engine_api::SkillId) {
    let round = tx.state().turn.round;
    let mut blob = b"LY1".to_vec();
    blob.extend_from_slice(&round.to_le_bytes());
    blob.push(1);
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = blob;
    }
}

/// 良姻：失牌后选一名其他角色双方各摸一张并可回复；获牌后弃一张牌并令一名
/// 其他角色弃一张牌。
/// 成略（许攸，转换技）：翻面后摸 1/2 张并弃 2/1 张手牌，本回合弃置花色的牌无距离与次数限制。
pub(crate) fn resolve_chenglve(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    suits: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
            {
                return Ok(Vec::new());
            }
            let branch = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| {
                    (instance.state_blob.len() >= 4 && &instance.state_blob[..3] == b"CL1")
                        .then(|| instance.state_blob[3])
                })
                .unwrap_or(0);
            let mut blob = b"CL1".to_vec();
            blob.push(if branch == 0 { 1 } else { 0 });
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob = blob;
            }
            // 翻面后的新分支：0=阴 摸 2 弃 1；1=阳 摸 1 弃 2。
            let new_branch = if branch == 0 { 1 } else { 0 };
            let draw_count = if new_branch == 0 { 2 } else { 1 };
            let discard_count = 3 - draw_count;
            if discard_count == 0 {
                return run_effects(
                    tx,
                    vec![EffectRequest::DrawCards {
                        player,
                        count: draw_count,
                    }],
                );
            }
            // 先压入弃牌步骤，使摸牌中途的触发选择恢复后能继续；摸牌无中断时直接续跑。
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ResolveChenglve {
                    player,
                    stage: 1,
                    suits,
                    skill,
                },
            );
            match run_effects(
                tx,
                vec![EffectRequest::DrawCards {
                    player,
                    count: draw_count,
                }],
            ) {
                Ok(_) => {
                    tx.state_mut().resolution.deferred_effects.retain(|effect| {
                        !matches!(effect, EffectRequest::ResolveChenglve { stage: 1, .. })
                    });
                    resolve_chenglve(tx, player, 1, suits, skill)
                }
                Err(e) => Err(e),
            }
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
            {
                return Ok(Vec::new());
            }
            let branch = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| {
                    (instance.state_blob.len() >= 4 && &instance.state_blob[..3] == b"CL1")
                        .then(|| instance.state_blob[3])
                })
                .unwrap_or(0);
            let discard_count = if branch == 0 { 1 } else { 2 } as u32;
            let options: Vec<_> = tx.state().hand_cards(player);
            if options.len() < discard_count as usize {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                discard_count.min(u8::MAX as u32) as u8,
                discard_count.min(u8::MAX as u32) as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_CHENGLVE_DISCARD,
                    1,
                    player.raw(),
                    skill.raw(),
                    discard_count,
                ],
            )
        }
        2 => {
            // 弃牌确认后：记录弃置花色为本回合无限制花色。
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
            {
                return Ok(Vec::new());
            }
            let key = turn_flag::CHENGLVE_UNLIMITED_SUITS;
            let existing = tx.state().turn.flag(key).max(0) as u32;
            tx.state_mut()
                .turn
                .set_flag(key, (existing | u32::from(suits)) as i32);
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "成略 stage must resolve through its choice continuation".into(),
        )),
    }
}

/// 恃才（许攸）：使用后若实体牌进弃牌堆且为本回合该类牌首用，可将它们以任意顺序
/// 置于牌堆顶，然后摸一张。
pub(crate) fn resolve_shicai(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    placed: Vec<CardId>,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
            {
                return Ok(Vec::new());
            }
            let Some(used) = tx.state().cards.get(card) else {
                return Ok(Vec::new());
            };
            let kind = crate::query::effective_card_kind(tx.state(), player, used);
            let category = if kind.is_equip() {
                1u32 << 2
            } else if kind.is_trick() {
                1u32 << 1
            } else {
                1u32 << 0
            };
            let key = turn_flag::shicai_used_category_by_player(player);
            let used_mask = tx.state().turn.flag(key).max(0) as u32;
            if used_mask & category != 0 {
                return Ok(Vec::new());
            }
            // 装备使用需包含自身为目标；实体牌需已进入弃牌堆。
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("discard pile missing".into()))?;
            let cards_in_discard: Vec<_> = tx
                .state()
                .zones
                .get(discard)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default();
            let used_physical: Vec<_> = tx
                .state()
                .cards
                .get(card)
                .map(|value| vec![card])
                .unwrap_or_default()
                .into_iter()
                .filter(|c| cards_in_discard.contains(c))
                .collect();
            if used_physical.is_empty() {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .turn
                .set_flag(key, (used_mask | category) as i32);
            open_choice(
                tx,
                player,
                used_physical.len().min(u8::MAX as usize) as u8,
                used_physical.len().min(u8::MAX as usize) as u8,
                true,
                used_physical.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_SHICAI_PLACE,
                    1,
                    player.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            // 放置续体：选中的牌以“最后一张在牌堆顶”的顺序置于牌堆顶，然后摸一张。
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
            {
                return Ok(Vec::new());
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("draw pile missing".into()))?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("discard pile missing".into()))?;
            let mut requests = Vec::new();
            for card in &placed {
                requests.push(EffectRequest::MoveCards {
                    cards: vec![*card],
                    to: draw,
                    from: Some(discard),
                });
            }
            requests.push(EffectRequest::DrawCards { player, count: 1 });
            run_effects(tx, requests)
        }
        _ => Err(EffectError::ContentFault(
            "恃才 stage must resolve through its choice continuation".into(),
        )),
    }
}

/// 闭月（界貂蝉）：结束阶段开始时，你摸 min(4, 本回合内受到伤害的不同角色数 + 1) 张牌。
/// ABI 599。
pub(crate) fn resolve_sbbiyue(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    let damaged = tx.state().turn.damaged_player_count();
    let count = damaged.saturating_add(1).min(4);
    run_effects(tx, vec![EffectRequest::DrawCards { player, count }])
}

/// 窃王（sxrm 严畯）：其他角色受到伤害后，若其与拥有者距离不大于 1，拥有者摸一张牌。
/// 手牌视为【无懈可击】的部分以文档化近似省略。ABI 596。
pub(crate) fn resolve_sxrmqiewang(
    tx: &mut Transaction,
    player: PlayerId,
    event_player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
        || event_player.is_none()
        || !tx
            .state()
            .players
            .get(event_player)
            .is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    if player == event_player {
        return Ok(Vec::new());
    }
    let distance = crate::query::query_explain(
        tx.state(),
        &rule_api::QueryRequest::Distance {
            from: player,
            to: event_player,
        },
        &mut crate::query::QueryContext::new(),
    );
    if !matches!(distance.response, rule_api::QueryResponse::Int(d) if d <= 1) {
        return Ok(Vec::new());
    }
    run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }])
}

/// 潜流（sxrm 伏皇后）：成为【杀】目标后取牌堆底四张；花色数大于 3 则获得之，
/// 否则置回牌堆顶（旧 JS 的拖动排序以置顶近似）。ABI 596。
pub(crate) fn resolve_sxrmqianliu(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
        || source == player
    {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile missing".into()))?;
    let bottom: Vec<_> = tx
        .state()
        .zones
        .get(draw)
        .map(|zone| zone.cards.iter().copied().take(4).collect())
        .unwrap_or_default();
    if bottom.len() < 4 {
        return Ok(Vec::new());
    }
    if stage == 0 {
        let mut suits = std::collections::BTreeSet::new();
        for card in &bottom {
            if let Some(value) = tx.state().cards.get(*card) {
                suits.insert(crate::command::judge::suit_tag(value.suit));
            }
        }
        let gain = suits.len() > 3;
        let mut events = Vec::new();
        events.push(engine_api::DomainEvent::RuleNote {
            code: 0x5960,
            payload: {
                let mut v = vec![player.raw(), u32::from(gain)];
                v.extend(bottom.iter().map(|card| card.raw()));
                v
            },
        });
        if gain {
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("hand missing".into()))?;
            events.extend(move_cards(tx, bottom, hand, Some(draw))?);
        } else {
            events.extend(move_cards(tx, bottom, draw, Some(draw))?);
        }
        Ok(events)
    } else {
        Err(EffectError::ContentFault("潜流 stage must be 0".into()))
    }
}

/// 旧不屈（shenhua 旧周泰）：体力降至 0 以下时把牌堆顶牌置入「创」技能牌堆并展示；
/// 点数均不同则置免死标记（伤害链消费后跳过濒死）。ABI 591。
pub(crate) fn resolve_gzbuqu(
    tx: &mut Transaction,
    player: PlayerId,
    delta: i32,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage != 0 {
        return Err(EffectError::ContentFault("旧不屈 stage must be 0".into()));
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
        || delta <= 0
    {
        return Ok(Vec::new());
    }
    let Some(hp) = tx.state().players.get(player).map(|p| p.hp) else {
        return Ok(Vec::new());
    };
    if hp > 0 {
        return Ok(Vec::new());
    }
    let skill_def = skill_pile_def(tx, player, skill)?;
    // 旧 JS：num = -trigger.num - max(player.hp - trigger.num, 1) + 1
    // （trigger.num 为负；等价于 delta - max(hp_before, 1) + 1，hp_before = hp + delta）。
    let count = (delta - (hp + delta).max(1) + 1).max(0) as usize;
    if count == 0 {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile missing".into()))?;
    let top: Vec<_> = tx
        .state()
        .zones
        .get(draw)
        .map(|zone| zone.cards.iter().rev().copied().take(count).collect())
        .unwrap_or_default();
    if top.len() < count {
        return Ok(Vec::new());
    }
    let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
    let mut events = Vec::new();
    events.extend(move_cards(tx, top.clone(), pile, Some(draw))?);
    // 展示创牌（RuleNote 投影）。
    events.push(engine_api::DomainEvent::RuleNote {
        code: 0x5910,
        payload: {
            let mut v = vec![player.raw()];
            v.extend(top.iter().map(|card| card.raw()));
            v
        },
    });
    // 点数均不同 → 免死标记。
    let mut ranks = std::collections::BTreeSet::new();
    let mut unique = true;
    for card in &top {
        let rank = tx
            .state()
            .cards
            .get(*card)
            .map(|card| card.rank)
            .unwrap_or(0);
        if !ranks.insert(rank) {
            unique = false;
        }
    }
    if unique {
        tx.state_mut().resolution.gzbuqu_nodying = true;
    }
    Ok(events)
}

/// 旧不屈·恢复：回复体力后按回复量移除「创」牌（点数重复者优先），移除后点数均不同
/// 则救回当前濒死。ABI 591。
pub(crate) fn resolve_gzbuqu_recover(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    let skill_def = skill_pile_def(tx, player, skill)?;
    let pile_cards = tx.state().skill_pile_cards(player, skill_def);
    if pile_cards.is_empty() {
        return Ok(Vec::new());
    }
    if stage == 0 {
        // 恢复量（event 不携带具体数值：按回复事件数量近似每次移除一张，点数重复者优先）。
        let options: Vec<_> = pile_cards
            .iter()
            .copied()
            .filter(|card| tx.state().cards.get(*card).is_some())
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        // 移除一张：优先点数重复的牌。
        let mut seen = std::collections::BTreeMap::new();
        for card in &options {
            let rank = tx.state().cards.get(*card).map(|c| c.rank).unwrap_or(0);
            *seen.entry(rank).or_insert(0u32) += 1;
        }
        let duplicate_first: Vec<_> = {
            let mut dup: Vec<_> = options
                .iter()
                .copied()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|c| seen.get(&c.rank).copied().unwrap_or(0) > 1)
                })
                .collect();
            if dup.is_empty() {
                dup = options.clone();
            }
            dup
        };
        if duplicate_first.len() == 1 {
            // 唯一候选：直接移除并结算救回。
            return resolve_gzbuqu_recover_commit(tx, player, skill_def, duplicate_first[0]);
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            duplicate_first.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            vec![RESOLVE_GZBUQU_RECOVER_CARD, player.raw(), skill.raw()],
        );
    }
    Err(EffectError::ContentFault(
        "旧不屈恢复 must resolve through its card continuation".into(),
    ))
}

pub(crate) fn resolve_gzbuqu_recover_commit(
    tx: &mut Transaction,
    player: PlayerId,
    skill_def: u32,
    card: CardId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("discard pile missing".into()))?;
    let mut events = run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: discard,
            from: Some(pile),
        }],
    )?;
    let remaining = tx.state().skill_pile_cards(player, skill_def);
    let mut ranks = std::collections::BTreeSet::new();
    let mut unique = true;
    for card in remaining {
        let rank = tx.state().cards.get(card).map(|c| c.rank).unwrap_or(0);
        if !ranks.insert(rank) {
            unique = false;
        }
    }
    if unique && tx.state().players.get(player).is_some_and(|p| p.dying) {
        tx.state_mut().resolution.gzbuqu_nodying = true;
    }
    Ok(events)
}

/// 雄乱（shenhua 张绣，限定技）：废除装备栏与判定区；直到回合结束，你对限定目标使用牌
/// 无距离与次数限制（回合 flag），其不能使用或打出手牌（临时禁手牌技能）。ABI 590。
pub(crate) fn resolve_xiongluan(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage != 1 {
        return Err(EffectError::ContentFault(
            "雄乱 must resolve through its target selection".into(),
        ));
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    // 限定技：整局一次。
    let used = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.state_blob.len() >= 3 && &instance.state_blob[..3] == b"XL1"
    });
    if used {
        return Ok(Vec::new());
    }
    if target.is_none()
        || target == player
        || !tx.state().players.get(target).is_some_and(|p| p.alive)
    {
        return Err(EffectError::Illegal(
            "雄乱 requires one other alive target".into(),
        ));
    }
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = b"XL1".to_vec();
    }
    let mut events = Vec::new();
    events.extend(disable_equip_slots(tx, player, vec![1, 2, 3, 4, 5])?);
    if !tx.state().disabled_judge_zones.contains(&player) {
        tx.state_mut().disabled_judge_zones.insert(player);
        events.push(engine_api::DomainEvent::RuleNote {
            code: 0x5900,
            payload: vec![player.raw()],
        });
    }
    tx.state_mut().turn.set_flag(
        turn_flag::xiongluan_target_by_player(player),
        target.raw() as i32,
    );
    events.extend(add_skill_until_turn_end(tx, target, DR_LT_XIONGLUAN_BAN)?);
    Ok(events)
}

/// 踞襄（shenhua 张绣，主公技）：其他群势力角色出牌阶段限一次，可将一张装备牌置入你的
/// 装备区（栏位空闲或禁用时直接装备并恢复栏位），否则你获得之。ABI 590。
/// 文档化近似：目标集合与“限一次”以回合 flag 位掩码表达，旧 JS 的 usable:1 按轮重置。
pub(crate) fn resolve_twjuxiang(
    tx: &mut Transaction,
    player: PlayerId,
    event_player: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    if stage == 0 {
        if event_player.is_none()
            || event_player == player
            || !tx
                .state()
                .players
                .get(event_player)
                .is_some_and(|p| p.alive)
            || tx.state().player_group(event_player) != Some("qun")
        {
            return Ok(Vec::new());
        }
        // 限一次（回合 flag 位掩码按 event_player 编号）。
        let key = turn_flag::twjuxiang_used_by_event_player(event_player);
        if tx.state().turn.flag(key) != 0 {
            return Ok(Vec::new());
        }
        let options: Vec<_> = tx
            .state()
            .zones
            .find(ZoneKind::Equip, event_player)
            .map(|zone| {
                tx.state()
                    .zones
                    .get(zone)
                    .map(|zone| zone.cards.clone())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .into_iter()
            .filter(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|c| c.kind.is_equip())
            })
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        tx.state_mut().turn.set_flag(key, 1);
        return open_choice(
            tx,
            event_player,
            1,
            1,
            true,
            options.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            vec![
                RESOLVE_TWJUXIANG_CARD,
                player.raw(),
                event_player.raw(),
                skill.raw(),
            ],
        );
    }
    if stage == 1 {
        // 续体：置入装备区或获得。
        let card = CardId::new(0); // resolved from continuation payload
        let _ = card;
        Err(EffectError::ContentFault(
            "踞襄 stage 1 must resolve through its card continuation".into(),
        ))
    } else {
        Err(EffectError::ContentFault("踞襄 unknown stage".into()))
    }
}

/// 踞襄（张绣）禁手牌：本回合不能使用或打出手牌。
pub(crate) const DR_LT_XIONGLUAN_BAN: engine_api::SkillDefId = engine_api::SkillDefId::new(1234567905);

pub(crate) fn resolve_liangyin(
    tx: &mut Transaction,
    player: PlayerId,
    kongsheng_def: engine_api::SkillDefId,
    stage: u8,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || liangyin_used(tx, skill)
            {
                return Ok(Vec::new());
            }
            liangyin_mark(tx, skill);
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![RESOLVE_LIANGYIN_DRAW, 1, player.raw(), skill.raw()],
            )
        }
        3 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || liangyin_used(tx, skill)
            {
                return Ok(Vec::new());
            }
            liangyin_mark(tx, skill);
            let options: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .chain(
                    tx.state()
                        .zones
                        .find(ZoneKind::Equip, player)
                        .map(|zone| {
                            tx.state()
                                .zones
                                .get(zone)
                                .map(|zone| zone.cards.clone())
                                .unwrap_or_default()
                        })
                        .unwrap_or_default(),
                )
                .collect();
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
                vec![RESOLVE_LIANGYIN_DISCARD, 4, player.raw(), skill.raw()],
            )
        }
        2 => {
            // 摸牌后的回复选择（由 deferred 步骤继续，目标在摸牌中途的触发选择后恢复）。
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
            {
                return Ok(Vec::new());
            }
            crate::command::resolve_choice::resume_liangyin_recover_check(
                tx, player, target, skill,
            )?;
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "良姻 stage must resolve through its choice continuation".into(),
        )),
    }
}

/// 箜声：准备阶段将任意张牌置为「箜」；结束阶段非装备箜回手、装备箜令一名角色使用。
pub(crate) fn resolve_kongsheng(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    target: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || tx.state().turn.current != player
            {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .chain(
                    tx.state()
                        .zones
                        .find(ZoneKind::Equip, player)
                        .map(|zone| {
                            tx.state()
                                .zones
                                .get(zone)
                                .map(|zone| zone.cards.clone())
                                .unwrap_or_default()
                        })
                        .unwrap_or_default(),
                )
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                options.len().min(u8::MAX as usize) as u8,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![RESOLVE_KONGSHENG_PLACE, 1, player.raw(), skill.raw()],
            )
        }
        2 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || tx.state().turn.current != player
            {
                return Ok(Vec::new());
            }
            let pile = kongsheng_pile_cards(tx, player);
            if pile.is_empty() {
                return Ok(Vec::new());
            }
            let (equips, non_equips): (Vec<_>, Vec<_>) = pile.into_iter().partition(|&card| {
                tx.state()
                    .cards
                    .get(card)
                    .is_some_and(|value| value.kind.is_equip())
            });
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
            let pile_zone = tx
                .state()
                .skill_piles
                .iter()
                .find(|pile| pile.owner == player)
                .map(|pile| pile.zone)
                .ok_or_else(|| EffectError::Illegal("no 箜 pile".into()))?;
            let mut requests = Vec::new();
            for card in non_equips {
                requests.push(EffectRequest::MoveCards {
                    cards: vec![card],
                    to: hand,
                    from: Some(pile_zone),
                });
            }
            if !requests.is_empty() {
                // 先压入下一步（装备箜目标选择），使回手触发的中途选择恢复后能继续。
                tx.state_mut().resolution.deferred_effects.insert(
                    0,
                    EffectRequest::ResolveKongsheng {
                        player,
                        stage: 3,
                        target: PlayerId::NONE,
                        card: CardId::NONE,
                        skill,
                    },
                );
                match run_effects(tx, requests) {
                    Ok(_) => {
                        // 回手未触发选择：预压步骤作废，本步直接继续。
                        tx.state_mut().resolution.deferred_effects.retain(|effect| {
                            !matches!(effect, EffectRequest::ResolveKongsheng { stage: 3, .. })
                        });
                    }
                    Err(e) => return Err(e),
                }
            }
            if equips.is_empty() {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx.state().players.alive_by_seat();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_KONGSHENG_TARGET,
                3,
                player.raw(),
                skill.raw(),
                equips.len() as u32,
            ];
            payload.extend(equips.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        3 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || tx.state().turn.current != player
            {
                return Ok(Vec::new());
            }
            let pile = kongsheng_pile_cards(tx, player);
            let equips: Vec<_> = pile
                .into_iter()
                .filter(|&card| {
                    tx.state()
                        .cards
                        .get(card)
                        .is_some_and(|value| value.kind.is_equip())
                })
                .collect();
            if equips.is_empty() {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx.state().players.alive_by_seat();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_KONGSHENG_TARGET,
                3,
                player.raw(),
                skill.raw(),
                equips.len() as u32,
            ];
            payload.extend(equips.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        4 => {
            // 装备移入目标手牌：先压入使用步骤，再移牌，中途触发选择恢复后继续。
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || tx.state().turn.current != player
            {
                return Ok(Vec::new());
            }
            if card.is_none() || target.is_none() {
                return Ok(Vec::new());
            }
            let pile_zone = tx
                .state()
                .skill_piles
                .iter()
                .find(|pile| pile.owner == player)
                .map(|pile| pile.zone)
                .ok_or_else(|| EffectError::Illegal("no 箜 pile".into()))?;
            let target_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("no target hand zone".into()))?;
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ResolveKongsheng {
                    player,
                    stage: 5,
                    target,
                    card,
                    skill,
                },
            );
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to: target_hand,
                    from: Some(pile_zone),
                }],
            )?;
            Ok(Vec::new())
        }
        5 => {
            // 装备使用：先压入下一件移牌步骤，再使用当前件。
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx.state().players.get(player).is_some_and(|p| p.alive)
                || tx.state().turn.current != player
            {
                return Ok(Vec::new());
            }
            if card.is_none() || target.is_none() {
                return Ok(Vec::new());
            }
            let next = kongsheng_pile_cards(tx, player)
                .into_iter()
                .find(|&value| {
                    tx.state()
                        .cards
                        .get(value)
                        .is_some_and(|v| v.kind.is_equip())
                })
                .unwrap_or(CardId::NONE);
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ResolveKongsheng {
                    player,
                    stage: 4,
                    target,
                    card: next,
                    skill,
                },
            );
            crate::command::play::use_equipment_from_hand(tx, target, card)?;
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "箜声 stage must resolve through its choice continuation".into(),
        )),
    }
}

pub(crate) fn pottuntian_max_charge(tx: &Transaction, skill: engine_api::SkillId) -> u32 {
    tx.state()
        .skills
        .get(skill)
        .and_then(|skill| skill.state_blob.get(..4))
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
        .unwrap_or(0)
}

pub(crate) fn pottuntian_charge(tx: &Transaction, player: PlayerId, mark: u32) -> u32 {
    tx.state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.marks.iter())
        .find(|(existing, _)| *existing == mark)
        .map(|(_, count)| (*count).max(0) as u32)
        .unwrap_or(0)
}

pub(crate) fn zhugemengxue_card_in_hand_equip(tx: &Transaction, player: PlayerId, card: CardId) -> bool {
    tx.state().cards.get(card).is_some_and(|state| {
        state.owner == player
            && [
                tx.state().zones.find(ZoneKind::Hand, player),
                tx.state().zones.find(ZoneKind::Equip, player),
            ]
            .contains(&Some(state.zone))
    })
}

pub(crate) fn zhugemengxue_branch_mask(tx: &Transaction, skill: engine_api::SkillId) -> u8 {
    tx.state()
        .skills
        .get(skill)
        .and_then(|instance| {
            (instance.state_blob.len() == 5 && &instance.state_blob[..4] == b"JCH1")
                .then(|| instance.state_blob[4])
        })
        .unwrap_or(0)
}

pub(crate) fn set_zhugemengxue_branch_mask(tx: &mut Transaction, skill: engine_api::SkillId, mask: u8) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = [b"JCH1".as_slice(), &[mask]].concat();
    }
}

pub(crate) fn zhugemengxue_jichun_target_is_legal(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    mask: u8,
) -> bool {
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return false;
    }
    let owner_hand = tx.state().hand_count(player);
    let target_hand = tx.state().hand_count(target);
    (target_hand < owner_hand && mask & 1 == 0)
        || (target_hand > owner_hand
            && mask & 2 == 0
            && crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card)
            && !crate::command::play::discardable_target_zone_card_tokens(tx, player, target)
                .is_empty())
}

pub(crate) fn zhugemengxue_hanying_targets(tx: &Transaction, player: PlayerId, card: CardId) -> Vec<PlayerId> {
    let Some(card_state) = tx.state().cards.get(card) else {
        return Vec::new();
    };
    let Some(draw) = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE) else {
        return Vec::new();
    };
    if card_state.zone != draw {
        return Vec::new();
    }
    let kind = crate::query::effective_card_kind(tx.state(), player, card_state);
    let Some(slot) = kind.equip_slot() else {
        return Vec::new();
    };
    let hand = tx.state().hand_count(player);
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            tx.state().hand_count(*target) == hand
                && tx.state().players.get(*target).is_some_and(|participant| {
                    !participant.dying && participant.equip_slot_enabled(slot)
                })
                && !crate::query::card_use_cancelled_for_card(tx.state(), *target, card)
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
/// XIANDING 诸葛梦雪「寄春 / 寒英」（ABI 588）。
pub(crate) fn resolve_zhugemengxue(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
            {
                return Ok(Vec::new());
            }
            let [card] = cards.as_slice() else {
                return Err(EffectError::Illegal("寄春须选择一张牌".into()));
            };
            let card = *card;
            if !zhugemengxue_card_in_hand_equip(tx, player, card)
                || crate::query::card_name_length(tx.state(), player, card).is_none()
            {
                return Err(EffectError::Illegal("寄春所选牌已失效".into()));
            }
            let mask = zhugemengxue_branch_mask(tx, skill);
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    zhugemengxue_jichun_target_is_legal(tx, player, *target, card, mask)
                })
                .collect();
            if options.is_empty() {
                return Err(EffectError::Illegal("寄春没有合法目标".into()));
            }
            let mut payload = vec![
                RESOLVE_ZHUGEMENGXUE_CHOICE,
                0,
                player.raw(),
                skill.raw(),
                card.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        1 => {
            let mask = zhugemengxue_branch_mask(tx, skill);
            if !originating_skill_is_enabled(tx, player, skill)
                || !zhugemengxue_card_in_hand_equip(tx, player, card)
                || crate::query::card_name_length(tx.state(), player, card).is_none()
                || !zhugemengxue_jichun_target_is_legal(tx, player, target, card, mask)
            {
                return Err(EffectError::Illegal("寄春目标或牌已失效".into()));
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveZhugemengxue {
                    player,
                    target,
                    card,
                    action: 3,
                    cards: Vec::new(),
                    skill,
                },
            );
            Ok(vec![DomainEvent::CardsRevealed {
                cards: vec![card],
                to: Vec::new(),
            }])
        }
        2 => {
            if tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player && instance.state_blob.get(..4) == Some(b"JCH1")
            }) {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .unwrap()
                    .state_blob
                    .clear();
            }
            Ok(Vec::new())
        }
        3 => {
            let length = crate::query::card_name_length(tx.state(), player, card)
                .ok_or_else(|| EffectError::Illegal("寄春牌名字数不可用".into()))?;
            let mask = zhugemengxue_branch_mask(tx, skill);
            if !originating_skill_is_enabled(tx, player, skill)
                || !zhugemengxue_card_in_hand_equip(tx, player, card)
                || !zhugemengxue_jichun_target_is_legal(tx, player, target, card, mask)
            {
                return Err(EffectError::Illegal("寄春结算条件已变化".into()));
            }
            if tx.state().hand_count(target) < tx.state().hand_count(player) {
                set_zhugemengxue_branch_mask(tx, skill, mask | 1);
                let from = tx.state().cards.get(card).unwrap().zone;
                let to = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, target)
                    .ok_or_else(|| EffectError::EngineFault("寄春目标手牌区不存在".into()))?;
                tx.state_mut()
                    .resolution
                    .deferred_tail_effects
                    .push(EffectRequest::DrawCards {
                        player,
                        count: u32::from(length),
                    });
                give_cards_as(tx, player, vec![card], to, from)
            } else {
                set_zhugemengxue_branch_mask(tx, skill, mask | 2);
                tx.state_mut().resolution.deferred_effects.push(
                    EffectRequest::ResolveZhugemengxue {
                        player,
                        target,
                        card,
                        action: 4,
                        cards: Vec::new(),
                        skill,
                    },
                );
                discard_cards(tx, player, vec![card])
            }
        }
        4 => {
            let Some(length) = crate::query::card_name_length(tx.state(), player, card) else {
                return Ok(Vec::new());
            };
            let options =
                crate::command::play::discardable_target_zone_card_tokens(tx, player, target);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let max = usize::from(length)
                .min(options.len())
                .min(usize::from(u8::MAX)) as u8;
            let mut payload = vec![
                RESOLVE_ZHUGEMENGXUE_CHOICE,
                1,
                player.raw(),
                skill.raw(),
                target.raw(),
                u32::from(max),
                options.len() as u32,
            ];
            payload.extend(options.iter().copied());
            open_choice(
                tx,
                player,
                1,
                max,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        10 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let Some(draw) = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE) else {
                return Ok(Vec::new());
            };
            let card = tx
                .state()
                .zones
                .get(draw)
                .into_iter()
                .flat_map(|zone| zone.cards.iter().rev().copied())
                .find(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| state.kind.is_equip())
                        && !crate::query::card_is_gift(tx.state(), *card)
                });
            let Some(card) = card else {
                return Ok(Vec::new());
            };
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveZhugemengxue {
                    player,
                    target: PlayerId::NONE,
                    card,
                    action: 12,
                    cards: Vec::new(),
                    skill,
                },
            );
            Ok(vec![DomainEvent::CardsRevealed {
                cards: vec![card],
                to: Vec::new(),
            }])
        }
        11 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !zhugemengxue_hanying_targets(tx, player, card).contains(&target)
            {
                return Err(EffectError::Illegal("寒英目标或装备已失效".into()));
            }
            crate::command::play::use_equipment_from_shared_zone(tx, target, card)?;
            Ok(Vec::new())
        }
        12 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let options = zhugemengxue_hanying_targets(tx, player, card);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_ZHUGEMENGXUE_CHOICE,
                2,
                player.raw(),
                skill.raw(),
                card.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault("诸葛梦雪 action 非法".into())),
    }
}

pub(crate) fn bailingyun_card_in_processing_or_discard(tx: &Transaction, card: CardId) -> bool {
    tx.state().cards.get(card).is_some_and(|state| {
        tx.state()
            .zones
            .get(state.zone)
            .is_some_and(|zone| matches!(zone.kind, ZoneKind::Processing | ZoneKind::DiscardPile))
    })
}

pub(crate) fn return_bailingyun_cards_to_top(
    tx: &mut Transaction,
    cards: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("灵慧牌堆不存在".into()))?;
    let mut events = Vec::new();
    let filtered: Vec<_> = cards
        .into_iter()
        .filter(|card| bailingyun_card_in_processing_or_discard(tx, *card))
        .rev()
        .collect();
    for card in filtered {
        let from = tx.state().cards.get(card).unwrap().zone;
        events.extend(move_cards(tx, vec![card], draw, Some(from))?);
    }
    Ok(events)
}

pub(crate) fn bailingyun_xiace_mask(tx: &Transaction, player: PlayerId) -> u8 {
    tx.state()
        .turn
        .flag(turn_flag::xiace_used_by_player(player))
        .clamp(0, u8::MAX as i32) as u8
}

#[allow(clippy::too_many_arguments)]
/// XIANDING 柏灵筠「灵慧 / 黠策」（ABI 593）。
pub(crate) fn resolve_bailingyun(
    tx: &mut Transaction,
    player: PlayerId,
    _target: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.phase != Phase::End
                || (tx.state().turn.current != player
                    && tx.state().turn.entered_dying_players.is_empty())
            {
                return Ok(Vec::new());
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("灵慧牌堆不存在".into()))?;
            while tx
                .state()
                .zones
                .get(draw)
                .map_or(0, |zone| zone.cards.len())
                < 3
            {
                let before = tx
                    .state()
                    .zones
                    .get(draw)
                    .map_or(0, |zone| zone.cards.len());
                reshuffle_discard_into_draw(tx)?;
                if tx
                    .state()
                    .zones
                    .get(draw)
                    .map_or(0, |zone| zone.cards.len())
                    == before
                {
                    break;
                }
            }
            let cards: Vec<_> = tx
                .state()
                .zones
                .get(draw)
                .map(|zone| zone.cards.iter().rev().take(3).copied().collect())
                .unwrap_or_default();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("灵慧处理区不存在".into()))?;
            let mut events = move_cards(tx, cards.clone(), processing, Some(draw))?;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveBailingyun {
                    player,
                    target: PlayerId::NONE,
                    card: CardId::NONE,
                    action: 1,
                    cards: cards.clone(),
                    skill,
                },
            );
            events.push(DomainEvent::CardsRevealed {
                cards,
                to: vec![player],
            });
            Ok(events)
        }
        1 => {
            let available: Vec<_> = cards
                .iter()
                .copied()
                .filter(|card| {
                    bailingyun_card_in_processing_or_discard(tx, *card)
                        && crate::command::play::skill_card_target_spec(tx, player, *card).is_ok()
                })
                .collect();
            if !originating_skill_is_enabled(tx, player, skill) || available.is_empty() {
                return return_bailingyun_cards_to_top(tx, cards);
            }
            let mut payload = vec![
                RESOLVE_BAILINGYUN_CHOICE,
                0,
                player.raw(),
                skill.raw(),
                cards.len() as u32,
            ];
            payload.extend(cards.iter().map(|card| card.raw()));
            payload.push(available.len() as u32);
            payload.extend(available.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                available.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        2 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !bailingyun_card_in_processing_or_discard(tx, card)
            {
                return return_bailingyun_cards_to_top(tx, cards);
            }
            match crate::command::play::skill_card_target_spec(tx, player, card)? {
                crate::command::play::SkillCardTargetSpec::Immediate(targets) => {
                    tx.state_mut().resolution.deferred_tail_effects.push(
                        EffectRequest::ResolveBailingyun {
                            player,
                            target: PlayerId::NONE,
                            card: CardId::NONE,
                            action: 3,
                            cards,
                            skill,
                        },
                    );
                    crate::command::play::handle_skill_card(tx, player, card, &targets)?;
                    Ok(Vec::new())
                }
                crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
                    let mut payload = vec![
                        RESOLVE_BAILINGYUN_CHOICE,
                        1,
                        player.raw(),
                        skill.raw(),
                        card.raw(),
                        u32::from(min),
                        u32::from(max),
                        cards.len() as u32,
                    ];
                    payload.extend(cards.iter().map(|card| card.raw()));
                    payload.push(options.len() as u32);
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
                        RESOLVE_BAILINGYUN_CHOICE,
                        2,
                        player.raw(),
                        skill.raw(),
                        card.raw(),
                        cards.len() as u32,
                    ];
                    payload.extend(cards.iter().map(|card| card.raw()));
                    payload.push(pairs.len() as u32);
                    for (left, right) in &pairs {
                        payload.push(left.raw());
                        payload.push(right.raw());
                    }
                    open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        (1..=pairs.len() as u32).collect(),
                        ChoiceKind::Custom,
                        ContinuationKind::Custom,
                        payload,
                    )
                }
            }
        }
        3 => {
            let mut available: Vec<_> = cards
                .iter()
                .copied()
                .filter(|card| bailingyun_card_in_processing_or_discard(tx, *card))
                .collect();
            if available.is_empty()
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.alive)
            {
                return return_bailingyun_cards_to_top(tx, cards);
            }
            let mut rng = RuleRng::from_state(tx.state().rng);
            tx.journal(JournalEntry::RngCheckpoint);
            let index = rng.gen_range_u32(available.len() as u32) as usize;
            tx.state_mut().rng = rng.into_state();
            let gained = available.remove(index);
            let remaining: Vec<_> = cards.into_iter().filter(|card| *card != gained).collect();
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveBailingyun {
                    player,
                    target: PlayerId::NONE,
                    card: CardId::NONE,
                    action: 4,
                    cards: remaining,
                    skill,
                },
            );
            gain_cards_by_zone(tx, player, &[gained])
        }
        4 => return_bailingyun_cards_to_top(tx, cards),
        10 => {
            let mask = bailingyun_xiace_mask(tx, player);
            if !originating_skill_is_enabled(tx, player, skill) || mask & 1 != 0 {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player)
                .filter(|target| {
                    !tx.state()
                        .turn
                        .non_locked_skill_suppressed_players
                        .contains(target)
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_BAILINGYUN_CHOICE,
                3,
                player.raw(),
                skill.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        11 => {
            let mask = bailingyun_xiace_mask(tx, player);
            let wounded = tx.state().players.get(player).is_some_and(|participant| {
                participant.alive && participant.hp < participant.max_hp as i32
            });
            if !originating_skill_is_enabled(tx, player, skill) || mask & 2 != 0 || !wounded {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_BAILINGYUN_CHOICE,
                4,
                player.raw(),
                skill.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault("柏灵筠 action 非法".into())),
    }
}

pub(crate) fn resolve_dcshaowei(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if action == 1 {
        let options: Vec<_> = tx
            .state()
            .players
            .get(player)
            .into_iter()
            .flat_map(|owner| owner.disabled_equip_slots.iter().enumerate())
            .filter_map(|(index, count)| (*count > 0).then_some(index as u32 + 1))
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options,
            ChoiceKind::Custom,
            ContinuationKind::ChooseDrltZhitiEnableSlot,
            vec![player.raw(), skill.raw()],
        );
    }
    if action != 0 {
        return Err(EffectError::ContentFault("绍威 action 非法".into()));
    }

    let key = turn_flag::dcshaowei_used_by_player(player);
    let used = tx.state().turn.flag(key).clamp(0, u8::MAX as i32) as u8;
    let mut suits = 0u8;
    for card in cards {
        let Some(card) = tx.state().cards.get(card) else {
            continue;
        };
        match crate::query::effective_card_suit(tx.state(), player, card) {
            Suit::Diamond => suits |= 1,
            Suit::Heart => suits |= 2,
            _ => {}
        }
    }
    let pending = suits & !used;
    if pending == 0 {
        return Ok(Vec::new());
    }

    let mut events = Vec::new();
    let mut next_used = used;
    if pending & 1 != 0 {
        let mut red: Vec<_> =
            tx.state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .and_then(|draw| tx.state().zones.get(draw))
                .into_iter()
                .flat_map(|zone| zone.cards.iter().copied())
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card| {
                        crate::query::card_color_mask_for_suit(card.suit) == 0b01
                    })
                })
                .collect();
        if !red.is_empty() {
            let mut rng = RuleRng::from_state(tx.state().rng);
            tx.journal(JournalEntry::RngCheckpoint);
            rng.shuffle(&mut red);
            tx.state_mut().rng = rng.into_state();
            red.truncate(3);
            events.extend(gain_cards_by_zone(tx, player, &red)?);
        }
        next_used |= 1;
    }
    if pending & 2 != 0 {
        next_used |= 2;
        tx.state_mut().resolution.deferred_tail_effects.extend([
            EffectRequest::RecoverHp {
                source: player,
                player,
                amount: 1,
            },
            EffectRequest::ResolveDcshaowei {
                player,
                action: 1,
                cards: Vec::new(),
                skill,
            },
        ]);
    }
    tx.state_mut().turn.set_flag(key, i32::from(next_used));
    Ok(events)
}

pub(crate) fn dcdichou_mark(tx: &Transaction, skill: engine_api::SkillId) -> Option<engine_api::MarkDefId> {
    tx.state()
        .skills
        .get(skill)
        .map(|instance| engine_api::MarkDefId::new(instance.def.raw()))
}

pub(crate) fn dcdichou_mark_count(tx: &Transaction, target: PlayerId, mark: engine_api::MarkDefId) -> i32 {
    tx.state()
        .players
        .get(target)
        .and_then(|player| player.marks.iter().find(|(id, _)| *id == mark.raw()))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_dcdichou(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    helper_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let mark = dcdichou_mark(tx, skill)
        .ok_or_else(|| EffectError::EngineFault("涤仇技能不存在".into()))?;
    match action {
        0 => {
            let options: Vec<_> = tx
                .state()
                .players
                .get(player)
                .into_iter()
                .flat_map(|owner| owner.disabled_equip_slots.iter().enumerate())
                .filter_map(|(index, count)| (*count == 0).then_some(index as u32 + 1))
                .collect();
            if options.is_empty() {
                return resolve_dcdichou(
                    tx,
                    player,
                    PlayerId::NONE,
                    CardId::NONE,
                    1,
                    helper_skill_def,
                    skill,
                );
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
                    RESOLVE_DCDICHOU_CHOICE,
                    0,
                    player.raw(),
                    skill.raw(),
                    helper_skill_def.raw(),
                ],
            )
        }
        1 => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    dcdichou_mark_count(tx, *target, mark) > 0
                        && !crate::query::discardable_hand_equip_cards(tx.state(), *target)
                            .is_empty()
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_DCDICHOU_CHOICE,
                1,
                player.raw(),
                skill.raw(),
                helper_skill_def.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        2 => {
            if dcdichou_mark_count(tx, target, mark) <= 0 {
                return Err(EffectError::Illegal("涤仇目标已失效".into()));
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_DCDICHOU_CHOICE,
                2,
                player.raw(),
                skill.raw(),
                helper_skill_def.raw(),
                target.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        3 => {
            let Some(length) = crate::query::card_name_length(tx.state(), player, card) else {
                return Ok(Vec::new());
            };
            tx.state_mut().resolution.deferred_tail_effects.extend([
                EffectRequest::RemoveMark {
                    player,
                    mark: engine_api::MarkDefId::new(helper_skill_def.raw()),
                    amount: 0,
                },
                EffectRequest::AddSkillUntilTurnEnd {
                    player,
                    skill_def: helper_skill_def,
                },
                EffectRequest::AddMark {
                    player,
                    mark: engine_api::MarkDefId::new(helper_skill_def.raw()),
                    delta: i32::from(length),
                },
            ]);
            Ok(Vec::new())
        }
        10 => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| dcdichou_mark_count(tx, *target, mark) > 0)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_DCDICHOU_CHOICE,
                10,
                player.raw(),
                skill.raw(),
                helper_skill_def.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                options.len().min(u8::MAX as usize) as u8,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault("涤仇 action 非法".into())),
    }
}

pub(crate) fn resolve_dcdufeng(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    helper_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|owner| owner.alive)
    {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let slots: Vec<_> = tx
                .state()
                .players
                .get(player)
                .into_iter()
                .flat_map(|owner| owner.disabled_equip_slots.iter().enumerate())
                .filter_map(|(index, count)| (*count == 0).then_some(index as u32 + 1))
                .collect();
            let mut options = vec![0];
            options.extend(slots);
            let mut payload = vec![
                RESOLVE_DCDUFENG_CHOICE,
                player.raw(),
                skill.raw(),
                helper_skill_def.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().copied());
            open_choice(
                tx,
                player,
                1,
                if options.len() > 1 { 2 } else { 1 },
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                payload,
            )
        }
        1 => {
            let Some(owner) = tx.state().players.get(player) else {
                return Ok(Vec::new());
            };
            let disabled = owner
                .disabled_equip_slots
                .iter()
                .filter(|count| **count > 0)
                .count() as u32;
            let lost_hp = (i64::from(owner.max_hp) - i64::from(owner.hp)).max(0) as u32;
            let amount = disabled.saturating_add(lost_hp).min(owner.max_hp);
            if amount == 0 {
                return Ok(Vec::new());
            }
            let mut effects = vec![EffectRequest::DrawCards {
                player,
                count: amount,
            }];
            if amount > 1 {
                effects.extend([
                    EffectRequest::RemoveMark {
                        player,
                        mark: engine_api::MarkDefId::new(helper_skill_def.raw()),
                        amount: 0,
                    },
                    EffectRequest::AddSkillUntilTurnEnd {
                        player,
                        skill_def: helper_skill_def,
                    },
                    EffectRequest::AddMark {
                        player,
                        mark: engine_api::MarkDefId::new(helper_skill_def.raw()),
                        delta: amount.saturating_sub(1) as i32,
                    },
                ]);
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .extend(effects);
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("独锋 action 非法".into())),
    }
}

pub(crate) fn resolve_pottuntian(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    amount: u8,
    targets: Vec<PlayerId>,
    zaoxian_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let mark = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::EngineFault("势屯田技能不存在".into()))?;
    let charge = pottuntian_charge(tx, player, mark);
    let max_charge = pottuntian_max_charge(tx, skill);
    match action {
        0 => {
            if charge >= max_charge {
                return Ok(Vec::new());
            }
            add_mark(tx, player, mark, 1)
        }
        1 => {
            if charge < max_charge {
                return Ok(Vec::new());
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob = max_charge.saturating_add(1).to_le_bytes().to_vec();
            }
            draw_cards(tx, player, 1)
        }
        2 => {
            let distinct: std::collections::BTreeSet<_> = targets.iter().copied().collect();
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || charge == 0
                || targets.is_empty()
                || targets.len() > charge as usize
                || distinct.len() != targets.len()
                || targets.iter().any(|target| {
                    !tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|participant| participant.alive)
                })
            {
                return Err(EffectError::Illegal("势屯田发动条件已变化".into()));
            }
            let mut payload = vec![
                POTTUNTIAN_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                zaoxian_def.raw(),
                targets.len() as u32,
            ];
            payload.extend(targets.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                (targets.len() as u32..=charge).collect(),
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                payload,
            )
        }
        3 => {
            if amount == 0
                || u32::from(amount) > charge
                || targets.is_empty()
                || targets.len() > usize::from(amount)
            {
                return Err(EffectError::Illegal("势屯田蓄力消耗已变化".into()));
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("弃牌堆不存在".into()))?;
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("牌堆不存在".into()))?;
            let owner_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("势屯田手牌区不存在".into()))?;
            let has_zaoxian = player_has_skill_def(tx, player, zaoxian_def, true);
            let thresholds = [
                (2, CardKind::TrickWuzhong),
                (5, CardKind::TrickWuxie),
                (7, CardKind::TrickWugu),
            ];
            let mut reserved = std::collections::BTreeSet::new();
            let mut effects = vec![EffectRequest::RemoveMark {
                player,
                mark: engine_api::MarkDefId::new(mark),
                amount: u32::from(amount),
            }];
            if has_zaoxian {
                for (threshold, kind) in thresholds {
                    if amount < threshold {
                        continue;
                    }
                    let card = tx
                        .state()
                        .zones
                        .get(discard)
                        .into_iter()
                        .flat_map(|zone| zone.cards.iter().rev().copied())
                        .find(|card| {
                            !reserved.contains(card)
                                && tx
                                    .state()
                                    .cards
                                    .get(*card)
                                    .is_some_and(|card| card.kind == kind)
                        });
                    if let Some(card) = card {
                        reserved.insert(card);
                        effects.push(EffectRequest::MoveCards {
                            cards: vec![card],
                            to: owner_hand,
                            from: Some(discard),
                        });
                    }
                }
            }
            let heart_cards: Vec<_> = [draw, discard]
                .into_iter()
                .flat_map(|zone| {
                    tx.state()
                        .zones
                        .get(zone)
                        .into_iter()
                        .flat_map(|zone| zone.cards.iter().rev().copied())
                        .map(move |card| (zone, card))
                })
                .filter(|(_, card)| {
                    !reserved.contains(card)
                        && tx
                            .state()
                            .cards
                            .get(*card)
                            .is_some_and(|card| card.suit == engine_api::Suit::Heart)
                })
                .take(targets.len())
                .collect();
            for (target, (from, card)) in targets.into_iter().zip(heart_cards) {
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, target)
                    .ok_or_else(|| EffectError::EngineFault("势屯田目标手牌区不存在".into()))?;
                effects.push(EffectRequest::MoveCards {
                    cards: vec![card],
                    to: hand,
                    from: Some(from),
                });
            }
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault("势屯田 action 非法".into())),
    }
}
