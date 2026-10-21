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
