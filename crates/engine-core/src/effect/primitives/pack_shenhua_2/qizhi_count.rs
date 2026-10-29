//! Effect primitives for `qizhi_count` and related helpers.

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
pub(crate) fn mingren_color_mask(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> u8 {
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
