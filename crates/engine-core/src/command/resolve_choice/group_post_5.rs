//! Choice-continuation handlers (post 5).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

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

/// 踞襄（张绣，主公技）装备牌续体：栏位空闲/禁用时置入主公装备区并恢复栏位，否则获得。
pub(crate) fn resolve_twjuxiang_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let lord = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let event_player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != event_player {
        return Ok(());
    }
    if cancelled || selections.len() != 1 {
        return Ok(());
    }
    let card = CardId::new(selections[0]);
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == lord && tx.state().skill_effectively_enabled(instance)
    }) || !tx.state().players.get(lord).is_some_and(|p| p.alive)
    {
        return Ok(());
    }
    let event_equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, event_player)
        .ok_or_else(|| EffectError::Illegal("no event equip zone".into()))?;
    let in_equip = tx
        .state()
        .zones
        .get(event_equip)
        .is_some_and(|zone| zone.cards.contains(&card));
    if !in_equip {
        return Err(EffectError::Illegal("踞襄 card moved".into()));
    }
    let kind = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.kind)
        .ok_or_else(|| EffectError::Illegal("踞襄 card missing".into()))?;
    if !kind.is_equip() {
        return Err(EffectError::Illegal(
            "踞襄 requires an equipment card".into(),
        ));
    }
    let lord_equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, lord)
        .ok_or_else(|| EffectError::Illegal("no lord equip zone".into()))?;
    let slot = kind.equip_slot();
    let slot_occupied = slot.is_some_and(|slot| {
        tx.state()
            .zones
            .get(lord_equip)
            .map(|zone| {
                zone.cards.iter().any(|existing| {
                    tx.state()
                        .cards
                        .get(*existing)
                        .is_some_and(|c| c.kind.equip_slot() == Some(slot))
                })
            })
            .unwrap_or(false)
    });
    let mut requests = Vec::new();
    if slot_occupied {
        let lord_hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, lord)
            .ok_or_else(|| EffectError::Illegal("no lord hand".into()))?;
        requests.push(EffectRequest::MoveCards {
            cards: vec![card],
            to: lord_hand,
            from: Some(event_equip),
        });
    } else {
        requests.push(EffectRequest::MoveCards {
            cards: vec![card],
            to: lord_equip,
            from: Some(event_equip),
        });
    }
    run_effects(tx, requests)?;
    if let Some(slot) = slot {
        if !tx
            .state()
            .players
            .get(lord)
            .is_some_and(|p| p.equip_slot_enabled(slot))
        {
            tx.push_event(engine_api::DomainEvent::RuleNote {
                code: 0x5901,
                payload: vec![lord.raw(), slot as u32],
            });
            tx.state_mut()
                .players
                .get_mut(lord)
                .expect("validated lord")
                .disabled_equip_slots[slot.index()] = 0;
        }
    }
    Ok(())
}

pub(crate) fn resolve_zhugemengxue_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 || payload[0] != crate::effect::RESOLVE_ZHUGEMENGXUE_CHOICE {
        return Err(EffectError::EngineFault(
            "bad 诸葛梦雪 choice payload".into(),
        ));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let skill = SkillId::new(payload[3]);
    if observer != player || cancelled {
        return Err(EffectError::Illegal("无效的诸葛梦雪选择".into()));
    }
    match stage {
        0 | 2 => {
            let card = CardId::new(payload[4]);
            let option_count = payload[5] as usize;
            if payload.len() != 6 + option_count || selections.len() != 1 {
                return Err(EffectError::Illegal("诸葛梦雪目标数量错误".into()));
            }
            let selected = selections[0];
            if !payload[6..].contains(&selected) {
                return Err(EffectError::Illegal("诸葛梦雪目标未在候选中".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhugemengxue {
                    player,
                    target: PlayerId::new(selected),
                    card,
                    action: if stage == 0 { 1 } else { 11 },
                    cards: Vec::new(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() < 7 {
                return Err(EffectError::EngineFault(
                    "bad 寄春弃牌 choice payload".into(),
                ));
            }
            let target = PlayerId::new(payload[4]);
            let max = payload[5] as usize;
            let option_count = payload[6] as usize;
            if payload.len() != 7 + option_count || selections.is_empty() || selections.len() > max
            {
                return Err(EffectError::Illegal("寄春弃牌数量错误".into()));
            }
            let offered: std::collections::BTreeSet<_> = payload[7..].iter().copied().collect();
            let current: std::collections::BTreeSet<_> =
                crate::command::play::discardable_target_zone_card_tokens(tx, player, target)
                    .into_iter()
                    .collect();
            let mut unique = std::collections::BTreeSet::new();
            if !selections.iter().all(|card| {
                unique.insert(*card) && offered.contains(card) && current.contains(card)
            }) {
                return Err(EffectError::Illegal("寄春弃牌已失效".into()));
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
        _ => Err(EffectError::EngineFault("bad 诸葛梦雪 choice stage".into())),
    }
}

pub(crate) fn resolve_bailingyun_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || payload[0] != crate::effect::RESOLVE_BAILINGYUN_CHOICE {
        return Err(EffectError::EngineFault("bad 柏灵筠 choice payload".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let skill = SkillId::new(payload[3]);
    if observer != player {
        return Err(EffectError::Illegal("无效的柏灵筠选择者".into()));
    }
    let skill_enabled = |tx: &Transaction| {
        tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    };
    match stage {
        0 => {
            let all_count = payload[4] as usize;
            let all_end = 5usize.saturating_add(all_count);
            if payload.len() <= all_end {
                return Err(EffectError::EngineFault("bad 灵慧选牌 payload".into()));
            }
            let cards: Vec<_> = payload[5..all_end]
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let offered_count = payload[all_end] as usize;
            if payload.len() != all_end + 1 + offered_count {
                return Err(EffectError::EngineFault("bad 灵慧候选牌 payload".into()));
            }
            if cancelled || selections.is_empty() {
                return run_effects(
                    tx,
                    vec![EffectRequest::ResolveBailingyun {
                        player,
                        target: PlayerId::NONE,
                        card: CardId::NONE,
                        action: 4,
                        cards,
                        skill,
                    }],
                )
                .map(|_| ());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal("灵慧至多选择一张牌".into()));
            };
            if !payload[all_end + 1..].contains(selected) {
                return Err(EffectError::Illegal("灵慧所选牌不在候选中".into()));
            }
            let selected = CardId::new(*selected);
            let remaining = cards.into_iter().filter(|card| *card != selected).collect();
            run_effects(
                tx,
                vec![EffectRequest::ResolveBailingyun {
                    player,
                    target: PlayerId::NONE,
                    card: selected,
                    action: 2,
                    cards: remaining,
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            if cancelled || payload.len() < 9 {
                return Err(EffectError::Illegal("灵慧用牌目标不可取消".into()));
            }
            let card = CardId::new(payload[4]);
            let min = payload[5] as usize;
            let max = payload[6] as usize;
            let remaining_count = payload[7] as usize;
            let remaining_end = 8usize.saturating_add(remaining_count);
            if payload.len() <= remaining_end {
                return Err(EffectError::EngineFault("bad 灵慧目标 payload".into()));
            }
            let remaining = payload[8..remaining_end]
                .iter()
                .copied()
                .map(CardId::new)
                .collect::<Vec<_>>();
            let option_count = payload[remaining_end] as usize;
            if payload.len() != remaining_end + 1 + option_count {
                return Err(EffectError::EngineFault("bad 灵慧目标候选 payload".into()));
            }
            let offered = &payload[remaining_end + 1..];
            let mut unique = std::collections::BTreeSet::new();
            if selections.len() < min
                || selections.len() > max
                || !selections
                    .iter()
                    .all(|target| unique.insert(*target) && offered.contains(target))
            {
                return Err(EffectError::Illegal("灵慧用牌目标非法".into()));
            }
            let crate::command::play::SkillCardTargetSpec::Select {
                min: current_min,
                max: current_max,
                options,
            } = crate::command::play::skill_card_target_spec(tx, player, card)?
            else {
                return Err(EffectError::Illegal("灵慧用牌目标形状已变化".into()));
            };
            let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            if targets.len() < current_min as usize
                || targets.len() > current_max as usize
                || targets.iter().any(|target| !options.contains(target))
            {
                return Err(EffectError::Illegal("灵慧用牌目标已失效".into()));
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveBailingyun {
                    player,
                    target: PlayerId::NONE,
                    card: CardId::NONE,
                    action: 3,
                    cards: remaining,
                    skill,
                },
            );
            crate::command::play::handle_skill_card(tx, player, card, &targets)
        }
        2 => {
            if cancelled || selections.len() != 1 || payload.len() < 8 {
                return Err(EffectError::Illegal("灵慧目标对选择非法".into()));
            }
            let card = CardId::new(payload[4]);
            let remaining_count = payload[5] as usize;
            let remaining_end = 6usize.saturating_add(remaining_count);
            if payload.len() <= remaining_end {
                return Err(EffectError::EngineFault("bad 灵慧目标对 payload".into()));
            }
            let remaining = payload[6..remaining_end]
                .iter()
                .copied()
                .map(CardId::new)
                .collect::<Vec<_>>();
            let pair_count = payload[remaining_end] as usize;
            if payload.len() != remaining_end + 1 + pair_count * 2 {
                return Err(EffectError::EngineFault(
                    "bad 灵慧目标对候选 payload".into(),
                ));
            }
            let crate::command::play::SkillCardTargetSpec::Pairs(pairs) =
                crate::command::play::skill_card_target_spec(tx, player, card)?
            else {
                return Err(EffectError::Illegal("灵慧目标对形状已变化".into()));
            };
            let index = selections[0]
                .checked_sub(1)
                .map(|value| value as usize)
                .ok_or_else(|| EffectError::Illegal("灵慧目标对索引非法".into()))?;
            let pair = pairs
                .get(index)
                .copied()
                .ok_or_else(|| EffectError::Illegal("灵慧目标对已失效".into()))?;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveBailingyun {
                    player,
                    target: PlayerId::NONE,
                    card: CardId::NONE,
                    action: 3,
                    cards: remaining,
                    skill,
                },
            );
            crate::command::play::handle_skill_card(tx, player, card, &[pair.0, pair.1])
        }
        3 => {
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal("黠策须选择一名角色".into()));
            };
            let count = payload[4] as usize;
            if payload.len() != 5 + count || !payload[5..].contains(selected) {
                return Err(EffectError::Illegal("黠策目标不在候选中".into()));
            }
            let target = PlayerId::new(*selected);
            let key = crate::state::turn_flag::xiace_used_by_player(player);
            let mask = tx.state().turn.flag(key).clamp(0, u8::MAX as i32) as u8;
            if !skill_enabled(tx)
                || mask & 1 != 0
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || tx
                    .state()
                    .turn
                    .non_locked_skill_suppressed_players
                    .contains(&target)
            {
                return Err(EffectError::Illegal("黠策目标已失效".into()));
            }
            tx.state_mut().turn.set_flag(key, i32::from(mask | 1));
            run_effects(
                tx,
                vec![EffectRequest::SuppressNonLockedSkillsUntilTurnEnd { player: target }],
            )
            .map(|_| ())
        }
        4 => {
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal("黠策须弃置一张牌".into()));
            };
            let count = payload[4] as usize;
            let card = CardId::new(*selected);
            let key = crate::state::turn_flag::xiace_used_by_player(player);
            let mask = tx.state().turn.flag(key).clamp(0, u8::MAX as i32) as u8;
            if payload.len() != 5 + count
                || !payload[5..].contains(selected)
                || !skill_enabled(tx)
                || mask & 2 != 0
                || !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card)
                || !tx.state().players.get(player).is_some_and(|participant| {
                    participant.alive && participant.hp < participant.max_hp as i32
                })
            {
                return Err(EffectError::Illegal("黠策弃牌已失效".into()));
            }
            tx.state_mut().turn.set_flag(key, i32::from(mask | 2));
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player,
                        cards: vec![card],
                    },
                    EffectRequest::RecoverHp {
                        source: player,
                        player,
                        amount: 1,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("bad 柏灵筠 choice stage".into())),
    }
}

pub(crate) fn resolve_dcdichou_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || payload[0] != crate::effect::RESOLVE_DCDICHOU_CHOICE {
        return Err(EffectError::EngineFault("bad 涤仇 choice payload".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    let helper_skill_def = engine_api::SkillDefId::new(payload[4]);
    if observer != player || cancelled {
        return Err(EffectError::Illegal("涤仇选择不可取消或代选".into()));
    }
    let mark = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .map(|instance| engine_api::MarkDefId::new(instance.def.raw()))
        .ok_or_else(|| EffectError::Illegal("涤仇技能已失效".into()))?;
    let mark_count = |tx: &Transaction, target: PlayerId| {
        tx.state()
            .players
            .get(target)
            .and_then(|participant| participant.marks.iter().find(|(id, _)| *id == mark.raw()))
            .map(|(_, count)| *count)
            .unwrap_or(0)
    };

    match stage {
        0 => {
            let [slot] = selections else {
                return Err(EffectError::Illegal("涤仇须废除一个装备栏".into()));
            };
            let slot = u8::try_from(*slot)
                .ok()
                .and_then(crate::state::EquipSlot::from_tag)
                .ok_or_else(|| EffectError::Illegal("涤仇装备栏非法".into()))?;
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|owner| owner.disabled_equip_slots[slot.index()] == 0)
            {
                return Err(EffectError::Illegal("涤仇装备栏已失效".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DisableEquipSlots {
                        player,
                        slots: vec![slot as u8],
                    },
                    EffectRequest::ResolveDcdichou {
                        player,
                        target: PlayerId::NONE,
                        card: CardId::NONE,
                        action: 1,
                        helper_skill_def,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        1 => {
            let [selected] = selections else {
                return Err(EffectError::Illegal("涤仇须选择一名有仇角色".into()));
            };
            let count = payload.get(5).copied().unwrap_or(0) as usize;
            if payload.len() != 6 + count || !payload[6..].contains(selected) {
                return Err(EffectError::Illegal("涤仇目标不在候选中".into()));
            }
            let target = PlayerId::new(*selected);
            if mark_count(tx, target) <= 0
                || crate::query::discardable_hand_equip_cards(tx.state(), target).is_empty()
            {
                return Err(EffectError::Illegal("涤仇目标已失效".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDcdichou {
                    player,
                    target,
                    card: CardId::NONE,
                    action: 2,
                    helper_skill_def,
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 => {
            let [selected] = selections else {
                return Err(EffectError::Illegal("涤仇须弃置一张牌".into()));
            };
            let target = PlayerId::new(*payload.get(5).unwrap_or(&0));
            let count = payload.get(6).copied().unwrap_or(0) as usize;
            let card = CardId::new(*selected);
            if payload.len() != 7 + count
                || !payload[7..].contains(selected)
                || mark_count(tx, target) <= 0
                || !crate::query::discardable_hand_equip_cards(tx.state(), target).contains(&card)
            {
                return Err(EffectError::Illegal("涤仇弃牌已失效".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player: target,
                        cards: vec![card],
                    },
                    EffectRequest::ResolveDcdichou {
                        player,
                        target,
                        card,
                        action: 3,
                        helper_skill_def,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        10 => {
            let count = payload.get(5).copied().unwrap_or(0) as usize;
            if payload.len() != 6 + count || selections.is_empty() {
                return Err(EffectError::Illegal("涤仇结束阶段目标非法".into()));
            }
            let offered = &payload[6..];
            let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
            if selected.len() != selections.len()
                || !selections.iter().all(|target| offered.contains(target))
            {
                return Err(EffectError::Illegal("涤仇结束阶段目标不在候选中".into()));
            }
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| selected.contains(&target.raw()) && mark_count(tx, *target) > 0)
                .collect();
            if targets.len() != selections.len() {
                return Err(EffectError::Illegal("涤仇结束阶段目标已失效".into()));
            }
            let mut effects = Vec::with_capacity(targets.len() * 2);
            for target in targets {
                effects.push(EffectRequest::DealDamage {
                    source: player,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::FIRE,
                });
                effects.push(EffectRequest::RemoveMark {
                    player: target,
                    mark,
                    amount: 0,
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        _ => Err(EffectError::EngineFault("bad 涤仇 choice stage".into())),
    }
}

pub(crate) fn resolve_dcdufeng_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 || payload[0] != crate::effect::RESOLVE_DCDUFENG_CHOICE {
        return Err(EffectError::EngineFault("bad 独锋 choice payload".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let helper_skill_def = engine_api::SkillDefId::new(payload[3]);
    let count = payload[4] as usize;
    let offered = &payload[5..];
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    if observer != player
        || cancelled
        || payload.len() != 5 + count
        || selections.is_empty()
        || selections.len() > 2
        || selected.len() != selections.len()
        || !selections.iter().all(|choice| offered.contains(choice))
        || selections.iter().filter(|choice| **choice != 0).count() > 1
    {
        return Err(EffectError::Illegal(
            "独锋须选择失去体力和/或一个装备栏".into(),
        ));
    }
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::Illegal("独锋技能已失效".into()));
    }

    let slot = selections.iter().copied().find(|choice| *choice != 0);
    if let Some(tag) = slot {
        let slot = u8::try_from(tag)
            .ok()
            .and_then(crate::state::EquipSlot::from_tag)
            .ok_or_else(|| EffectError::Illegal("独锋装备栏非法".into()))?;
        if !tx
            .state()
            .players
            .get(player)
            .is_some_and(|owner| owner.disabled_equip_slots[slot.index()] == 0)
        {
            return Err(EffectError::Illegal("独锋装备栏已失效".into()));
        }
    }

    let mut effects = Vec::with_capacity(3);
    if selections.contains(&0) {
        effects.push(EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: player,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        });
    }
    if let Some(slot) = slot {
        effects.push(EffectRequest::DisableEquipSlots {
            player,
            slots: vec![slot as u8],
        });
    }
    effects.push(EffectRequest::ResolveDcdufeng {
        player,
        action: 1,
        helper_skill_def,
        skill,
    });
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_kongsheng_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let count = *payload.get(4).unwrap_or(&0) as usize;
    if observer != player {
        return Err(EffectError::Illegal(
            "箜声 target choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "箜声 target is no longer legal".into(),
        ));
    }
    let def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::EngineFault("箜声 skill disappeared".into()))?;
    let pile_zone = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player && pile.skill_def == def)
        .map(|pile| pile.zone)
        .ok_or_else(|| EffectError::Illegal("no 箜 pile".into()))?;
    // 先压入第一件的移牌步骤，由 deferred 恢复驱动依次移入并使用。
    for i in 0..count {
        let card = CardId::new(*payload.get(5 + i).unwrap_or(&0));
        tx.state_mut()
            .resolution
            .deferred_effects
            .push(EffectRequest::ResolveKongsheng {
                player,
                stage: 4,
                target,
                card,
                skill,
            });
    }
    Ok(())
}

/// 巧变：跳过当前阶段（供各续体在收益结算后调用）。
pub(crate) fn qiaobian_skip_phase(tx: &mut Transaction, player: PlayerId) -> Result<(), EffectError> {
    crate::effect::run_effects(tx, vec![EffectRequest::SkipCurrentPhase { player }]).map(|_| ())
}

/// 巧变（弃牌阶段开始）：支付一张手牌，随后按阶段开启收益选择或直接跳过。
pub(crate) fn resolve_qiaobian_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let phase = *payload.get(2).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "巧变 choice must be made by the owner".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal("巧变 requires one hand card".into()));
    }
    let card = engine_api::CardId::new(selections[0]);
    let hand = tx.state().hand_cards(player);
    if !hand.contains(&card) {
        return Err(EffectError::Illegal(
            "巧变 card is no longer in hand".into(),
        ));
    }
    let hand_zone = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: discard,
            from: Some(hand_zone),
        }],
    )?;
    match phase {
        0 | 3 => qiaobian_skip_phase(tx, player),
        1 => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player && tx.state().hand_count(candidate) > 0)
                .collect();
            if options.is_empty() {
                return qiaobian_skip_phase(tx, player);
            }
            crate::effect::open_choice(
                tx,
                player,
                0,
                2,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                engine_api::ChoiceKind::SelectTargets,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_QIAOBIAN_DRAW_TARGETS,
                    2,
                    u32::from(phase),
                    player.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        2 => {
            let mut options = Vec::new();
            for candidate in tx.state().players.alive_by_seat() {
                for zone_kind in [crate::state::ZoneKind::Equip, crate::state::ZoneKind::Judge] {
                    if let Some(zone) = tx.state().zones.find(zone_kind, candidate) {
                        if let Some(zone) = tx.state().zones.get(zone) {
                            options.extend(zone.cards.iter().map(|card| card.raw()));
                        }
                    }
                }
            }
            if options.is_empty() {
                return qiaobian_skip_phase(tx, player);
            }
            crate::effect::open_choice(
                tx,
                player,
                0,
                1,
                true,
                options,
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_QIAOBIAN_PLAY_CARD,
                    2,
                    u32::from(phase),
                    player.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown 巧变 phase".into())),
    }
}

/// 巧变（摸牌阶段收益）：获至多两名其他角色各一张手牌，逐名开选牌。
pub(crate) fn resolve_qiaobian_draw_targets(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let phase = *payload.get(2).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "巧变 target choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return qiaobian_skip_phase(tx, player);
    }
    if selections.len() > 2 {
        return Err(EffectError::Illegal("巧变 at most two targets".into()));
    }
    let targets: Vec<_> = selections.iter().map(|&raw| PlayerId::new(raw)).collect();
    if targets.iter().any(|&target| {
        target == player
            || !tx.state().players.get(target).is_some_and(|p| p.alive)
            || tx.state().hand_count(target) == 0
    }) || targets
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len()
        != targets.len()
    {
        return Err(EffectError::Illegal(
            "巧变 targets are no longer legal".into(),
        ));
    }
    let first = targets[0];
    let mut choice_payload = vec![
        crate::effect::RESOLVE_QIAOBIAN_DRAW_PICK,
        3,
        u32::from(phase),
        player.raw(),
        skill.raw(),
        targets.len() as u32,
    ];
    choice_payload.extend(targets.iter().map(|target| target.raw()));
    choice_payload.push(0);
    crate::effect::open_choice(
        tx,
        first,
        1,
        1,
        false,
        tx.state()
            .hand_cards(first)
            .into_iter()
            .map(engine_api::CardId::raw)
            .collect(),
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::Custom,
        choice_payload,
    )
    .map(|_| ())
}

/// 巧变（摸牌阶段收益）：目标交一张手牌，随后进入下一目标或跳过阶段。
pub(crate) fn resolve_qiaobian_draw_pick(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let phase = *payload.get(2).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    let count = *payload.get(5).unwrap_or(&0) as usize;
    let index = *payload.get(6 + count).unwrap_or(&0) as usize;
    let target = PlayerId::new(*payload.get(6 + index).unwrap_or(&0));
    if observer != target {
        return Err(EffectError::Illegal(
            "巧变 pick must be made by the giving target".into(),
        ));
    }
    if cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "巧变 requires the target to give a card".into(),
        ));
    }
    let card = engine_api::CardId::new(selections[0]);
    let hand = tx.state().hand_cards(target);
    if !hand.contains(&card) {
        return Err(EffectError::Illegal(
            "巧变 given card is no longer in hand".into(),
        ));
    }
    let hand_zone = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let owner_hand = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no owner hand zone".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: owner_hand,
            from: Some(hand_zone),
        }],
    )?;
    if index + 1 >= count {
        return qiaobian_skip_phase(tx, player);
    }
    let next = PlayerId::new(*payload.get(6 + index + 1).unwrap_or(&0));
    let mut choice_payload = vec![
        crate::effect::RESOLVE_QIAOBIAN_DRAW_PICK,
        3,
        u32::from(phase),
        player.raw(),
        skill.raw(),
        count as u32,
    ];
    choice_payload.extend((0..count).map(|i| *payload.get(6 + i).unwrap_or(&0)));
    choice_payload.push((index + 1) as u32);
    crate::effect::open_choice(
        tx,
        next,
        1,
        1,
        false,
        tx.state()
            .hand_cards(next)
            .into_iter()
            .map(engine_api::CardId::raw)
            .collect(),
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::Custom,
        choice_payload,
    )
    .map(|_| ())
}

/// 巧变（出牌阶段收益）：选择场上的一张牌后，选择移动目的地。
pub(crate) fn resolve_qiaobian_play_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let phase = *payload.get(2).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "巧变 card choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return qiaobian_skip_phase(tx, player);
    }
    let card = engine_api::CardId::new(selections[0]);
    let Some(value) = tx.state().cards.get(card) else {
        return Err(EffectError::Illegal("巧变 field card vanished".into()));
    };
    if value.owner == PlayerId::NONE
        || !tx.state().players.get(value.owner).is_some_and(|p| p.alive)
    {
        return Err(EffectError::Illegal("巧变 field card owner is gone".into()));
    }
    let in_field = tx
        .state()
        .zones
        .get(value.zone)
        .map(|zone| {
            matches!(
                zone.kind,
                crate::state::ZoneKind::Equip | crate::state::ZoneKind::Judge
            ) && zone.cards.contains(&card)
        })
        .unwrap_or(false);
    if !in_field {
        return Err(EffectError::Illegal(
            "巧变 card is no longer on the field".into(),
        ));
    }
    let owner = value.owner;
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != owner)
        .collect();
    if options.is_empty() {
        return qiaobian_skip_phase(tx, player);
    }
    crate::effect::open_choice(
        tx,
        player,
        0,
        1,
        true,
        options.into_iter().map(PlayerId::raw).collect(),
        engine_api::ChoiceKind::SelectTargets,
        crate::state::ContinuationKind::Custom,
        vec![
            crate::effect::RESOLVE_QIAOBIAN_PLAY_TARGET,
            3,
            u32::from(phase),
            player.raw(),
            skill.raw(),
            card.raw(),
        ],
    )
    .map(|_| ())
}

/// 巧变（出牌阶段收益）：按牌类别移动到目的地角色的同类别区域，随后跳过阶段。
pub(crate) fn resolve_qiaobian_play_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let phase = *payload.get(2).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    let card = engine_api::CardId::new(*payload.get(5).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "巧变 move choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return qiaobian_skip_phase(tx, player);
    }
    let dest = PlayerId::new(selections[0]);
    let Some(value) = tx.state().cards.get(card) else {
        return Err(EffectError::Illegal("巧变 field card vanished".into()));
    };
    if dest == value.owner || !tx.state().players.get(dest).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "巧变 destination is no longer legal".into(),
        ));
    }
    let zone_kind = if value.kind.is_delayed_trick() {
        crate::state::ZoneKind::Judge
    } else {
        crate::state::ZoneKind::Equip
    };
    let dest_zone = tx
        .state()
        .zones
        .find(zone_kind, dest)
        .ok_or_else(|| EffectError::Illegal("no destination zone".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: dest_zone,
            from: Some(value.zone),
        }],
    )?;
    qiaobian_skip_phase(tx, player)
}

pub(crate) fn resolve_resolve_drlt_congjian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, stage(, target)]
    if payload.len() < 4 {
        return Err(EffectError::EngineFault("bad Congjian continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal("Congjian observer mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    let skill = engine_api::SkillId::new(payload[2]);
    let stage = payload[3];
    if stage == 0 {
        if selections.len() != 1 {
            return Err(EffectError::Illegal(
                "Congjian recipient selection invalid".into(),
            ));
        }
        let target = PlayerId::new(selections[0]);
        if !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal("Congjian context changed".into()));
        }
        return run_effects(
            tx,
            vec![EffectRequest::ResolveDrltCongjian {
                player,
                targets: Vec::new(),
                card: engine_api::CardId::NONE,
                target,
                stage: 1,
                skill,
            }],
        )
        .map(|_| ());
    }
    if stage == 1 {
        if selections.len() != 1 {
            return Err(EffectError::Illegal(
                "Congjian card selection invalid".into(),
            ));
        }
        let card = CardId::new(selections[0]);
        return run_effects(
            tx,
            vec![EffectRequest::ResolveDrltCongjian {
                player,
                targets: Vec::new(),
                card,
                target: PlayerId::new(payload[4]),
                stage: 2,
                skill,
            }],
        )
        .map(|_| ());
    }
    Err(EffectError::EngineFault(
        "bad Congjian continuation stage".into(),
    ))
}

pub(crate) fn resolve_resolve_xinkuangfu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, target, sha_target, stage]
    if payload.len() != 6 {
        return Err(EffectError::EngineFault(
            "bad Xinkuangfu continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal("Xinkuangfu observer mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Xinkuangfu context changed".into()));
    }
    let stage = payload[5];
    match stage {
        1 => {
            // The owner picked the equipment target.
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Xinkuangfu target selection invalid".into(),
                ));
            }
            let target = PlayerId::new(selections[0]);
            if target != player
                && (!tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                    || crate::query::discardable_target_cards(
                        tx.state(),
                        player,
                        target,
                        rule_api::TargetCardScope::Equip,
                    )
                    .is_empty())
            {
                return Err(EffectError::Illegal(
                    "Xinkuangfu target has no equipment".into(),
                ));
            }
            return run_effects(
                tx,
                vec![EffectRequest::ResolveXinkuangfu {
                    player,
                    target,
                    sha_target: PlayerId::NONE,
                    stage: 2,
                    skill: engine_api::SkillId::new(payload[2]),
                }],
            )
            .map(|_| ());
        }
        2 => {
            // The owner picked the equipment card to discard.
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Xinkuangfu equipment selection invalid".into(),
                ));
            }
            let target = PlayerId::new(payload[3]);
            let card = CardId::new(selections[0]);
            let allowed = crate::query::discardable_target_cards(
                tx.state(),
                player,
                if target.is_none() || target == player {
                    player
                } else {
                    target
                },
                rule_api::TargetCardScope::Equip,
            );
            if !allowed.contains(&card) {
                return Err(EffectError::Illegal(
                    "Xinkuangfu equipment card invalid".into(),
                ));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("Xinkuangfu card missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to: tx
                        .state()
                        .zones
                        .find(ZoneKind::DiscardPile, PlayerId::NONE)
                        .unwrap(),
                    from: Some(from),
                }],
            )?;
            return run_effects(
                tx,
                vec![EffectRequest::ResolveXinkuangfu {
                    player,
                    target,
                    sha_target: PlayerId::NONE,
                    stage: 3,
                    skill: engine_api::SkillId::new(payload[2]),
                }],
            )
            .map(|_| ());
        }
        3 => {
            // The owner picked the Sha target; run the Sha and branch on damage.
            if selections.len() != 1 {
                return Err(EffectError::Illegal("Xinkuangfu Sha target invalid".into()));
            }
            let sha_target = PlayerId::new(selections[0]);
            if sha_target == player
                || !tx
                    .state()
                    .players
                    .get(sha_target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("Xinkuangfu Sha target stale".into()));
            }
            let equip_target = PlayerId::new(payload[3]);
            let owner_hp_before = tx
                .state()
                .players
                .get(sha_target)
                .map(|p| p.hp)
                .unwrap_or(0);
            if let Err(crate::effect::EffectError::ChoiceRequested) = run_effects(
                tx,
                vec![EffectRequest::UseVirtualCardOn {
                    player,
                    kind: crate::state::CardKind::Sha.tag(),
                    target: sha_target,
                }],
            ) {
                return Ok(());
            }
            let owner_hp_after = tx
                .state()
                .players
                .get(sha_target)
                .map(|p| p.hp)
                .unwrap_or(0);
            let damaged = owner_hp_after < owner_hp_before;
            let mut tail = Vec::new();
            if equip_target == player {
                if damaged {
                    tail.push(EffectRequest::DrawCardsFromSkill {
                        player,
                        count: 2,
                        skill: engine_api::SkillId::new(payload[2]),
                    });
                }
            } else if !damaged {
                // The owner discards two hand cards.
                let options: Vec<_> =
                    crate::query::discardable_hand_equip_cards(tx.state(), player)
                        .into_iter()
                        .map(CardId::raw)
                        .collect();
                if options.len() < 2 {
                    return Ok(());
                }
                let mut next_payload = vec![
                    crate::effect::RESOLVE_XINKUANGFU_CUSTOM_CHOICE,
                    player.raw(),
                    engine_api::SkillId::new(payload[2]).raw(),
                    equip_target.raw(),
                    sha_target.raw(),
                    4,
                ];
                return crate::effect::open_choice(
                    tx,
                    player,
                    2,
                    2,
                    false,
                    options,
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    next_payload,
                )
                .map(|_| ());
            }
            run_effects(tx, tail).map(|_| ())
        }
        4 => {
            // Discard two hand cards.
            if selections.len() != 2 {
                return Err(EffectError::Illegal(
                    "Xinkuangfu hand discard invalid".into(),
                ));
            }
            let allowed = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let mut moves = Vec::new();
            for raw in selections {
                let card = CardId::new(*raw);
                if !allowed.contains(&card) {
                    return Err(EffectError::Illegal(
                        "Xinkuangfu hand discard card invalid".into(),
                    ));
                }
                let from = tx
                    .state()
                    .cards
                    .get(card)
                    .map(|card| card.zone)
                    .ok_or_else(|| EffectError::Illegal("Xinkuangfu card missing".into()))?;
                moves.push(EffectRequest::MoveCards {
                    cards: vec![card],
                    to: tx
                        .state()
                        .zones
                        .find(ZoneKind::DiscardPile, PlayerId::NONE)
                        .unwrap(),
                    from: Some(from),
                });
            }
            run_effects(tx, moves).map(|_| ())
        }
        _ => Err(EffectError::Illegal("Xinkuangfu stage out of range".into())),
    }
}

pub(crate) fn resolve_resolve_dcchiying(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, target, stage, accum_len, accum...] (+ picked at stage 3)
    if payload.len() < 6 {
        return Err(EffectError::EngineFault(
            "bad Dcchiying continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[3]);
    let stage = payload[4];
    let accum_len = payload[5] as usize;
    if payload.len() < 6 + accum_len {
        return Err(EffectError::EngineFault(
            "bad Dcchiying accumulated-card payload".into(),
        ));
    }
    let mut accum: Vec<CardId> = payload[6..6 + accum_len]
        .iter()
        .map(|id| CardId::new(*id))
        .collect();
    if stage == 3 {
        // Stage 3: the target discarded a card from the picked player.
        if payload.len() != 7 + accum_len {
            return Err(EffectError::EngineFault(
                "bad Dcchiying stage-3 payload".into(),
            ));
        }
        if observer != target || cancelled || selections.len() != 1 {
            return Err(EffectError::Illegal(
                "Dcchiying card discard context changed".into(),
            ));
        }
        let picked = PlayerId::new(payload[6 + accum_len]);
        let card = CardId::new(selections[0]);
        let allowed = crate::query::discardable_target_cards(
            tx.state(),
            target,
            picked,
            rule_api::TargetCardScope::HandEquip,
        );
        if !allowed.contains(&card) {
            return Err(EffectError::Illegal(
                "Dcchiying discarded card invalid".into(),
            ));
        }
        let from = tx
            .state()
            .cards
            .get(card)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("Dcchiying card missing".into()))?;
        run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards: vec![card],
                to: tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .unwrap(),
                from: Some(from),
            }],
        )?;
        accum.push(card);
        return run_effects(
            tx,
            vec![EffectRequest::ResolveDcchiying {
                player,
                target,
                stage: 2,
                accum,
                skill: engine_api::SkillId::new(payload[2]),
            }],
        )
        .map(|_| ());
    }
    if payload.len() != 6 + accum_len {
        return Err(EffectError::EngineFault(
            "bad Dcchiying payload length".into(),
        ));
    }
    if stage == 1 {
        // The owner chose the target.
        if observer != player {
            return Err(EffectError::Illegal(
                "Dcchiying target observer mismatch".into(),
            ));
        }
        if cancelled {
            return Ok(());
        }
        if selections.len() != 1 {
            return Err(EffectError::Illegal(
                "Dcchiying target selection invalid".into(),
            ));
        }
        let target = PlayerId::new(selections[0]);
        if target == player
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal("Dcchiying target stale".into()));
        }
        return run_effects(
            tx,
            vec![EffectRequest::ResolveDcchiying {
                player,
                target,
                stage: 2,
                accum: Vec::new(),
                skill: engine_api::SkillId::new(payload[2]),
            }],
        )
        .map(|_| ());
    }
    // Stage 2: the target picked a player (or cancelled).
    if observer != target {
        return Err(EffectError::Illegal(
            "Dcchiying discard observer mismatch".into(),
        ));
    }
    if cancelled {
        return crate::effect::dcchiying_gain_basics(tx, target, accum).map(|_| ());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "Dcchiying pick selection invalid".into(),
        ));
    }
    let picked = PlayerId::new(selections[0]);
    if picked == target
        || !tx
            .state()
            .players
            .get(picked)
            .is_some_and(|participant| participant.alive)
        || !crate::effect::dcchiying_in_range(tx.state(), target, picked)
    {
        return Err(EffectError::Illegal("Dcchiying picked player stale".into()));
    }
    // The target picks one card of the picked player to discard.
    let options: Vec<_> = crate::query::discardable_target_cards(
        tx.state(),
        target,
        picked,
        rule_api::TargetCardScope::HandEquip,
    )
    .into_iter()
    .map(CardId::raw)
    .collect();
    if options.is_empty() {
        return Ok(());
    }
    let mut next_payload = vec![
        crate::effect::RESOLVE_DCCHIYING_CUSTOM_CHOICE,
        player.raw(),
        engine_api::SkillId::new(payload[2]).raw(),
        target.raw(),
        3,
        accum.len() as u32,
    ];
    next_payload.extend(accum.iter().map(|card| card.raw()));
    next_payload.push(picked.raw());
    crate::effect::open_choice(
        tx,
        target,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        next_payload,
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_xuxie(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, stage, count, targets..., index]
    if payload.len() < 6 {
        return Err(EffectError::EngineFault("bad Xuxie continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal(
            "Xuxie choice observer mismatch".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Xuxie context changed".into()));
    }
    let stage = payload[3];
    let count = payload[4] as usize;
    let expected = if stage == 0 { 5 + count } else { 5 + count + 1 };
    if payload.len() != expected {
        return Err(EffectError::EngineFault("bad Xuxie payload length".into()));
    }
    let targets: Vec<PlayerId> = payload[5..5 + count]
        .iter()
        .map(|id| PlayerId::new(*id))
        .collect();
    if stage == 0 {
        if selections.len() != 1 {
            return Err(EffectError::Illegal(
                "Xuxie branch selection invalid".into(),
            ));
        }
        let branch = *selections.first().unwrap();
        if branch == 1 {
            let draws: Vec<_> = targets
                .iter()
                .map(|target| EffectRequest::DrawCardsFromSkill {
                    player: *target,
                    count: 1,
                    skill: engine_api::SkillId::new(payload[2]),
                })
                .collect();
            return run_effects(tx, draws).map(|_| ());
        }
        if branch != 0 {
            return Err(EffectError::Illegal("Xuxie branch out of range".into()));
        }
        // Discard branch: start the per-target loop with the marker choice.
        let mut next_payload = payload.to_vec();
        next_payload[3] = 1;
        next_payload.push(0); // current index
        return crate::effect::open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            next_payload,
        )
        .map(|_| ());
    }
    // Stage 1: the discard loop. index = payload[5 + count].
    let index = payload[5 + count] as usize;
    if index >= count {
        return Ok(());
    }
    let target = targets[index];
    let alive = tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive);
    let discardable = if alive {
        crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::HandEquip,
        )
    } else {
        Vec::new()
    };
    // Marker selection [0] or an unavailable target: advance.
    let is_marker = selections.len() == 1 && *selections.first().unwrap() == 0;
    if is_marker || discardable.is_empty() {
        let mut next_payload = payload.to_vec();
        next_payload[5 + count] = (index + 1) as u32;
        return crate::effect::open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            next_payload,
        )
        .map(|_| ());
    }
    // A card selection: discard it, then advance.
    if selections.len() != 1 || !discardable.contains(&CardId::new(selections[0])) {
        return Err(EffectError::Illegal("Xuxie discard card invalid".into()));
    }
    let card = CardId::new(selections[0]);
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("Xuxie card missing".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap(),
            from: Some(from),
        }],
    )?;
    let mut next_payload = payload.to_vec();
    next_payload[5 + count] = (index + 1) as u32;
    crate::effect::open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![0],
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        next_payload,
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_xizhen(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // Stage 0 payload: [magic, player, skill] -> the owner picked a target.
    // Stage 1 payload: [magic, player, skill, target] -> the owner picked the branch.
    if payload.len() == 3 {
        let player = PlayerId::new(payload[1]);
        if observer != player {
            return Err(EffectError::Illegal(
                "Xizhen choice observer mismatch".into(),
            ));
        }
        if cancelled {
            return Ok(());
        }
        if selections.len() != 1
            || !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal("Xizhen context changed".into()));
        }
        let target = PlayerId::new(selections[0]);
        if target == player
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal("Xizhen target stale".into()));
        }
        // Open the branch choice: 1 = Sha, 2 = Duel.
        return crate::effect::open_choice(
            tx,
            player,
            1,
            1,
            true,
            vec![1, 2],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            vec![
                crate::effect::RESOLVE_XIZHEN_CUSTOM_CHOICE,
                player.raw(),
                engine_api::SkillId::new(payload[2]).raw(),
                target.raw(),
            ],
        )
        .map(|_| ());
    }
    if payload.len() == 4 {
        let player = PlayerId::new(payload[1]);
        let target = PlayerId::new(payload[3]);
        if observer != player {
            return Err(EffectError::Illegal(
                "Xizhen branch observer mismatch".into(),
            ));
        }
        if cancelled {
            return Ok(());
        }
        if selections.len() != 1
            || !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal("Xizhen branch context changed".into()));
        }
        let branch = *selections.first().unwrap();
        let kind = match branch {
            1 => crate::state::CardKind::Sha,
            2 => crate::state::CardKind::TrickJuedou,
            _ => return Err(EffectError::Illegal("Xizhen branch out of range".into())),
        };
        return run_effects(
            tx,
            vec![EffectRequest::UseVirtualCardOn {
                player,
                kind: kind.tag(),
                target,
            }],
        )
        .map(|_| ());
    }
    Err(EffectError::EngineFault("bad Xizhen continuation".into()))
}

pub(crate) fn resolve_resolve_dcmoyu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault("bad Dcmoyu continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal(
            "Dcmoyu choice observer mismatch".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Dcmoyu context changed".into()));
    }
    let target = PlayerId::new(selections[0]);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Dcmoyu target stale".into()));
    }
    let skill = engine_api::SkillId::new(payload[2]);
    // Gain count: 1, plus 1 when the add-on mark from a previous use is set.
    let add = tx
        .state()
        .players
        .get(player)
        .map(|p| {
            p.marks
                .iter()
                .find(|(mark, _)| *mark == crate::effect::DCMOYU_ADD_MARK)
                .map(|(_, count)| *count)
                .unwrap_or(0)
        })
        .unwrap_or(0);
    let count = 1 + add.min(1) as usize;
    let available = crate::query::discardable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::HandEquipJudge,
    );
    let mut moves = Vec::new();
    for card in available.into_iter().take(count) {
        let from = tx
            .state()
            .cards
            .get(card)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("Dcmoyu card missing".into()))?;
        moves.push(EffectRequest::MoveCards {
            cards: vec![card],
            to: tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?,
            from: Some(from),
        });
    }
    if add > 0 {
        moves.push(EffectRequest::RemoveMark {
            player,
            mark: engine_api::MarkDefId::new(crate::effect::DCMOYU_ADD_MARK),
            amount: add.max(0) as u32,
        });
    }
    run_effects(tx, moves)?;
    // The target uses a virtual Sha on the owner; snapshot HP for the outcome.
    let owner_hp_before = tx.state().players.get(player).map(|p| p.hp).unwrap_or(0);
    if let Err(crate::effect::EffectError::ChoiceRequested) = run_effects(
        tx,
        vec![EffectRequest::UseVirtualCardOn {
            player: target,
            kind: crate::state::CardKind::Sha.tag(),
            target: player,
        }],
    ) {
        // The Sha opened a response window; skip the branch outcome.
        return Ok(());
    }
    let owner_hp_after = tx.state().players.get(player).map(|p| p.hp).unwrap_or(0);
    let mut tail = vec![EffectRequest::AddMark {
        player,
        mark: engine_api::MarkDefId::new(target.raw()),
        delta: 1,
    }];
    if owner_hp_after < owner_hp_before {
        // Damaged: ban the active skill until the turn ends.
        tail.push(EffectRequest::BanActiveUntilTurnEnd { player, skill });
    } else {
        // Not damaged: the next use gains one more card.
        tail.push(EffectRequest::AddMark {
            player,
            mark: engine_api::MarkDefId::new(crate::effect::DCMOYU_ADD_MARK),
            delta: 1,
        });
    }
    run_effects(tx, tail).map(|_| ())
}

pub(crate) fn resolve_resolve_rekuanshi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad Rekuanshi continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal(
            "Rekuanshi choice observer mismatch".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Rekuanshi context changed".into()));
    }
    let target = PlayerId::new(selections[0]);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Rekuanshi target stale".into()));
    }
    // Mark the watched player; the damage trigger resolves the recovery.
    run_effects(
        tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: engine_api::MarkDefId::new(crate::effect::REKUANSHI_WATCH_MARK),
            delta: 1,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_sxrmlanjiao(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, stage, 0, target, view_len, view...]
    if payload.len() < 7 {
        return Err(EffectError::EngineFault(
            "bad Sxrmlanjiao continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let stage = payload[3];
    let target = PlayerId::new(payload[5]);
    let view_len = payload[6] as usize;
    if payload.len() != 7 + view_len {
        return Err(EffectError::EngineFault(
            "bad Sxrmlanjiao payload length".into(),
        ));
    }
    let view: Vec<CardId> = payload[7..].iter().map(|id| CardId::new(*id)).collect();
    if stage == 0 {
        // Target choice made by the owner.
        if observer != player {
            return Err(EffectError::Illegal(
                "Sxrmlanjiao target choice observer mismatch".into(),
            ));
        }
        if cancelled || selections.len() != 1 {
            return Ok(());
        }
        let target = PlayerId::new(selections[0]);
        if target == player
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            || tx.state().hand_count(target) < 2
        {
            return Err(EffectError::Illegal("Sxrmlanjiao target stale".into()));
        }
        return run_effects(
            tx,
            vec![EffectRequest::ResolveSxrmlanjiao {
                player,
                target,
                stage: 1,
                view: Vec::new(),
                skill: engine_api::SkillId::new(payload[2]),
            }],
        )
        .map(|_| ());
    }
    // Stage 1: the target's rechoose choice.
    if observer != target {
        return Err(EffectError::Illegal(
            "Sxrmlanjiao rechoose observer mismatch".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Sxrmlanjiao context changed".into()));
    }
    let rechoose = if cancelled {
        false
    } else if selections.len() == 1 {
        *selections.first().unwrap() == 1
    } else {
        false
    };
    if rechoose {
        // The target loses 1 HP and draws one, then the owner re-views.
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            return Ok(());
        }
        return run_effects(
            tx,
            vec![
                EffectRequest::DrawCardsFromSkill {
                    player: target,
                    count: 1,
                    skill: engine_api::SkillId::new(payload[2]),
                },
                EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                },
            ],
        )
        .and_then(|_| {
            run_effects(
                tx,
                vec![EffectRequest::ResolveSxrmlanjiao {
                    player,
                    target,
                    stage: 1,
                    view: Vec::new(),
                    skill: engine_api::SkillId::new(payload[2]),
                }],
            )
        })
        .map(|_| ());
    }
    // Stop: the owner gains the viewed cards; mark the target as used this turn.
    if view.is_empty() {
        return Ok(());
    }
    let mut moves = Vec::new();
    for card in &view {
        if !tx.state().hand_cards(target).contains(card) {
            continue;
        }
        let from = tx
            .state()
            .cards
            .get(*card)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("Sxrmlanjiao card missing".into()))?;
        moves.push(EffectRequest::MoveCards {
            cards: vec![*card],
            to: tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?,
            from: Some(from),
        });
    }
    moves.push(EffectRequest::AddMark {
        player,
        mark: engine_api::MarkDefId::new(target.raw()),
        delta: 1,
    });
    run_effects(tx, moves).map(|_| ())
}

pub(crate) fn resolve_resolve_nsdingzhou(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad Nsdingzhou continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal(
            "Nsdingzhou choice observer mismatch".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Nsdingzhou context changed".into()));
    }
    let target = PlayerId::new(selections[0]);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Nsdingzhou target stale".into()));
    }
    // Gain one card from the target's zones (first by seat order, deterministic).
    let gained = crate::query::discardable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::HandEquipJudge,
    )
    .into_iter()
    .next();
    let Some(gained) = gained else {
        return Ok(());
    };
    let gained_color = tx
        .state()
        .cards
        .get(gained)
        .map(|card| card.suit as u32 % 2)
        .unwrap_or(0);
    let from = tx
        .state()
        .cards
        .get(gained)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("Nsdingzhou card missing".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![gained],
            to: hand,
            from: Some(from),
        }],
    )?;
    // Draw one; the drawn card is the newest hand card (after the gain).
    run_effects(
        tx,
        vec![EffectRequest::DrawCardsFromSkill {
            player,
            count: 1,
            skill: engine_api::SkillId::new(payload[2]),
        }],
    )?;
    let drawn = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .find(|card| *card != gained);
    if let Some(drawn) = drawn {
        let drawn_color = tx
            .state()
            .cards
            .get(drawn)
            .map(|card| card.suit as u32 % 2)
            .unwrap_or(0);
        if drawn_color != gained_color {
            return run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .map(|_| ());
        }
    }
    Ok(())
}
