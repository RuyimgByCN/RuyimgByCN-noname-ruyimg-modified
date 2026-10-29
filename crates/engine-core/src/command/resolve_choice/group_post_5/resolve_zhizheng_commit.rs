//! Command operations for `resolve_zhizheng_commit` and related helpers.

use super::*;

/// 掣政：弃置选定牌。
pub(crate) fn resolve_zhizheng_commit(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let target = PlayerId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "掣政 card choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let picked = CardId::new(selections[0]);
    let target_cards = tx.state().hand_cards(target);
    let equip_cards = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Equip, target)
        .map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    if !target_cards.contains(&picked) && !equip_cards.contains(&picked) {
        return Err(EffectError::Illegal("掣政 card is no longer legal".into()));
    }
    let from_zone = tx.state().cards.get(picked).map(|value| value.zone);
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![picked],
            to: discard,
            from: from_zone,
        }],
    )
    .map(|_| ())
}

/// 良姻（失牌分支）：双方各摸一张，任一方手牌等于箜数且受伤时可回复 1。
pub(crate) fn resolve_liangyin_draw(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "良姻 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if target == player || !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "良姻 target is no longer legal".into(),
        ));
    }
    // 先压入下一步（回复选择），使摸牌中途的触发选择（ChoiceRequested）恢复后能继续。
    tx.state_mut().resolution.deferred_effects.insert(
        0,
        EffectRequest::ResolveLiangyin {
            player,
            kongsheng_def: engine_api::SkillDefId::NONE,
            stage: 2,
            target,
            skill,
        },
    );
    crate::effect::run_effects(
        tx,
        vec![
            EffectRequest::DrawCards { player, count: 1 },
            EffectRequest::DrawCards {
                player: target,
                count: 1,
            },
        ],
    )
    .map(|_| ())
}

/// 良姻（失牌分支）：摸牌后，任一方手牌等于箜数且受伤时可回复 1。
pub(crate) fn resume_liangyin_recover_check(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    // 箜数 = 箜声牌堆张数。
    let kong = crate::effect::kongsheng_pile_cards(tx, player).len() as u32;
    let recoverable = |who: PlayerId| -> bool {
        tx.state().hand_count(who) == kong
            && tx
                .state()
                .players
                .get(who)
                .is_some_and(|p| p.hp < p.max_hp as i32)
    };
    if !recoverable(player) && !recoverable(target) {
        return Ok(());
    }
    crate::effect::open_choice(
        tx,
        player,
        0,
        1,
        true,
        vec![0, 1],
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::Custom,
        vec![
            crate::effect::RESOLVE_LIANGYIN_RECOVER,
            2,
            player.raw(),
            target.raw(),
            skill.raw(),
        ],
    )
    .map(|_| ())
}

/// 良姻：回复持有者或目标 1 点体力。
pub(crate) fn resolve_liangyin_recover(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let target = PlayerId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "良姻 recover choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let who = if selections[0] == 1 { target } else { player };
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::RecoverHp {
            player: who,
            amount: 1,
            source: PlayerId::NONE,
        }],
    )
    .map(|_| ())
}

/// 良姻（获牌分支）：弃一张牌并开目标选择。
pub(crate) fn resolve_liangyin_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "良姻 discard choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let picked = CardId::new(selections[0]);
    let owner_cards = tx.state().hand_cards(player);
    let equip_cards = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Equip, player)
        .map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    if !owner_cards.contains(&picked) && !equip_cards.contains(&picked) {
        return Err(EffectError::Illegal("良姻 card is no longer legal".into()));
    }
    let from_zone = tx.state().cards.get(picked).map(|value| value.zone);
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![picked],
            to: discard,
            from: from_zone,
        }],
    )?;
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| {
            candidate != player
                && (tx.state().hand_count(candidate) > 0
                    || tx
                        .state()
                        .zones
                        .find(crate::state::ZoneKind::Equip, candidate)
                        .is_some_and(|zone| {
                            tx.state()
                                .zones
                                .get(zone)
                                .is_some_and(|zone| !zone.cards.is_empty())
                        }))
        })
        .collect();
    if options.is_empty() {
        return Ok(());
    }
    crate::effect::open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(PlayerId::raw).collect(),
        engine_api::ChoiceKind::SelectTargets,
        crate::state::ContinuationKind::Custom,
        vec![
            crate::effect::RESOLVE_LIANGYIN_TARGET,
            5,
            player.raw(),
            skill.raw(),
        ],
    )
    .map(|_| ())
}

/// 良姻：选择目标的一张牌弃置。
pub(crate) fn resolve_liangyin_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "良姻 target choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    let mut options = tx.state().hand_cards(target);
    options.extend(
        tx.state()
            .zones
            .find(crate::state::ZoneKind::Equip, target)
            .map(|zone| {
                tx.state()
                    .zones
                    .get(zone)
                    .map(|zone| zone.cards.clone())
                    .unwrap_or_default()
            })
            .unwrap_or_default(),
    );
    if options.is_empty() {
        return Ok(());
    }
    crate::effect::open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(engine_api::CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::Custom,
        vec![
            crate::effect::RESOLVE_LIANGYIN_COMMIT,
            6,
            player.raw(),
            target.raw(),
            skill.raw(),
        ],
    )
    .map(|_| ())
}

/// 良姻：弃置目标的选定牌。
pub(crate) fn resolve_liangyin_commit(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let target = PlayerId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "良姻 commit must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let picked = CardId::new(selections[0]);
    let target_cards = tx.state().hand_cards(target);
    let equip_cards = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Equip, target)
        .map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    if !target_cards.contains(&picked) && !equip_cards.contains(&picked) {
        return Err(EffectError::Illegal("良姻 card is no longer legal".into()));
    }
    let from_zone = tx.state().cards.get(picked).map(|value| value.zone);
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![picked],
            to: discard,
            from: from_zone,
        }],
    )
    .map(|_| ())
}

/// 箜声：将选定牌置为「箜」。
pub(crate) fn resolve_kongsheng_place(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "箜声 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::EngineFault("箜声 skill disappeared".into()))?;
    let pile = crate::effect::ensure_skill_pile_zone(tx, player, def)?;
    let mut requests = Vec::new();
    for &raw in selections {
        let card = CardId::new(raw);
        let from_zone = tx.state().cards.get(card).map(|value| value.zone);
        requests.push(EffectRequest::MoveCards {
            cards: vec![card],
            to: pile,
            from: from_zone,
        });
    }
    crate::effect::run_effects(tx, requests).map(|_| ())
}

/// 傅干「巧谏」：衔接双方暗选、分支选择与额外结算确认。
pub(crate) fn resolve_qiaojian_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .get(1)
        .ok_or_else(|| EffectError::EngineFault("bad 巧谏 choice payload".into()))?;
    match stage {
        0 | 1 => {
            if payload.len() < if stage == 0 { 6 } else { 7 } {
                return Err(EffectError::EngineFault("bad 巧谏 discard payload".into()));
            }
            let player = PlayerId::new(payload[2]);
            let target = PlayerId::new(payload[3]);
            let chooser = if stage == 0 { player } else { target };
            if observer != chooser {
                return Err(EffectError::Illegal("巧谏选择者不匹配".into()));
            }
            let owner_count = if stage == 0 { 0 } else { payload[6] as usize };
            if stage == 1 && payload.len() != 7 + owner_count {
                return Err(EffectError::EngineFault("bad 巧谏 target payload".into()));
            }
            let mut cards = if stage == 0 {
                Vec::new()
            } else {
                payload[7..].iter().copied().map(CardId::new).collect()
            };
            if !cancelled {
                cards.extend(selections.iter().copied().map(CardId::new));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveQiaojian {
                    player,
                    target,
                    helper_def: SkillDefId::new(payload[4]),
                    action: if stage == 0 { 1 } else { 2 },
                    cards,
                    split: if stage == 0 { 0 } else { owner_count as u32 },
                    skill: SkillId::new(payload[5]),
                }],
            )
            .map(|_| ())
        }
        2 => {
            let [_, _, player_raw, target_raw, helper_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault("bad 巧谏 branch payload".into()));
            };
            let [branch] = selections else {
                return Err(EffectError::Illegal("巧谏需要选择一项效果".into()));
            };
            let player = PlayerId::new(*player_raw);
            if cancelled || observer != player || !matches!(*branch, 1 | 2) {
                return Err(EffectError::Illegal("无效的巧谏效果选择".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveQiaojian {
                    player,
                    target: PlayerId::new(*target_raw),
                    helper_def: SkillDefId::new(*helper_raw),
                    action: 10,
                    cards: Vec::new(),
                    split: *branch,
                    skill: SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        3 => {
            let [_, _, player_raw, card_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault("bad 巧谏 repeat payload".into()));
            };
            let player = PlayerId::new(*player_raw);
            if observer != player {
                return Err(EffectError::Illegal("巧谏确认者不匹配".into()));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            if selections != [1] {
                return Err(EffectError::Illegal("无效的巧谏额外结算确认".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveQiaojian {
                    player,
                    target: PlayerId::NONE,
                    helper_def: SkillDefId::NONE,
                    action: 8,
                    cards: vec![CardId::new(*card_raw)],
                    split: 0,
                    skill: SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown 巧谏 choice stage".into())),
    }
}

/// 傅干「析察」：保存从伤害来源手牌中秘密选中的实体牌。
pub(crate) fn resolve_xicha_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [_, player_raw, source_raw, skill_raw] = payload else {
        return Err(EffectError::EngineFault("bad 析察 choice payload".into()));
    };
    let player = PlayerId::new(*player_raw);
    let source = PlayerId::new(*source_raw);
    let required = tx.state().hand_cards(source).len().min(2);
    if cancelled || observer != player || required == 0 || selections.len() != required {
        return Err(EffectError::Illegal("无效的析察选牌".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveXicha {
            player,
            source,
            card: CardId::NONE,
            action: 2,
            cards: selections.iter().copied().map(CardId::new).collect(),
            skill: SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}

/// 胡遵「安境」：提交当前仍受伤的目标集合。
pub(crate) fn resolve_huzun_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::EngineFault("bad 安境 choice payload".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = SkillId::new(payload[2]);
    let max = payload[3] as usize;
    let option_count = payload[4] as usize;
    if observer != player
        || cancelled
        || payload.len() != 5 + option_count
        || selections.is_empty()
        || selections.len() > max
    {
        return Err(EffectError::Illegal("无效的安境目标".into()));
    }
    let offered: std::collections::BTreeSet<_> = payload[5..].iter().copied().collect();
    let mut unique = std::collections::BTreeSet::new();
    if !selections.iter().all(|target| {
        unique.insert(*target)
            && offered.contains(target)
            && tx
                .state()
                .players
                .get(PlayerId::new(*target))
                .is_some_and(|target| target.alive && target.hp < target.max_hp as i32)
    }) {
        return Err(EffectError::Illegal("安境目标已失效".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveHuzun {
            player,
            source: PlayerId::NONE,
            card: CardId::NONE,
            action: 2,
            cards: Vec::new(),
            targets: selections.iter().copied().map(PlayerId::new).collect(),
            skill,
        }],
    )
    .map(|_| ())
}

/// 诸葛梦雪「寄春 / 寒英」：提交权威目标或目标区域牌。
/// 成略弃牌确认：弃置选中手牌并记录其花色为无限制花色。
pub(crate) fn resolve_chenglve_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let discard_count = *payload.get(4).unwrap_or(&0) as usize;
    if observer != player || cancelled {
        return Ok(());
    }
    if selections.len() != discard_count {
        return Err(EffectError::Illegal(
            "成略 must discard exactly the required hand cards".into(),
        ));
    }
    let mut suits = 0u32;
    for raw in selections {
        let card = CardId::new(*raw);
        let suit = tx
            .state()
            .cards
            .get(card)
            .map(|value| value.suit)
            .ok_or_else(|| EffectError::Illegal("成略 discard card missing".into()))?;
        suits |= match suit {
            Suit::Spade => 1u32 << 1,
            Suit::Heart => 1u32 << 2,
            Suit::Club => 1u32 << 3,
            Suit::Diamond => 1u32 << 4,
            _ => 0,
        };
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
    run_effects(
        tx,
        vec![
            EffectRequest::MoveCards {
                cards,
                to: discard,
                from: Some(hand),
            },
            EffectRequest::ResolveChenglve {
                player,
                stage: 2,
                suits: suits as u8,
                skill,
            },
        ],
    )
    .map(|_| ())
}

/// 恃才放置：以“最后一张在牌堆顶”的顺序将选中的牌置于牌堆顶，然后摸一张。
pub(crate) fn resolve_shicai_place(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let card = CardId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Ok(());
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let placed: Vec<_> = selections.iter().copied().map(CardId::new).collect();
    run_effects(
        tx,
        vec![EffectRequest::ResolveShicai {
            player,
            card,
            placed,
            stage: 1,
            skill,
        }],
    )
    .map(|_| ())
}

/// 旧不屈·恢复（张绣）创牌移除续体。
pub(crate) fn resolve_gzbuqu_recover_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    if observer != player || cancelled || selections.len() != 1 {
        return Ok(());
    }
    let skill_def = crate::effect::skill_pile_def(tx, player, skill)?;
    let card = CardId::new(selections[0]);
    if !tx
        .state()
        .skill_pile_cards(player, skill_def)
        .contains(&card)
    {
        return Err(EffectError::Illegal("旧不屈创牌已移动".into()));
    }
    crate::effect::resolve_gzbuqu_recover_commit(tx, player, skill_def, card).map(|_| ())
}
