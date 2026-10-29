//! Command operations for `resolve_resolve_olzhijie` and related helpers.

use super::*;

/// 表召·归分派：0=目标已选（含取消）→ 结算。
pub(crate) fn resolve_resolve_biaozhao3(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 || payload[0] != 0x425A_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-biaozhao3 continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let num = payload[4];
    if observer != player {
        return Err(EffectError::Illegal(
            "resolve-biaozhao3 context changed".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    let target = selections
        .first()
        .copied()
        .ok_or_else(|| EffectError::Illegal("resolve-biaozhao3 target required".into()))?;
    crate::effect::resolve_biaozhao3_stage(tx, player, skill, num, &[target])?;
    Ok(())
}

/// 残殒分段分派：2=弃牌已定 → 3=目标已定（1-2 名）→ 结算。
pub(crate) fn resolve_resolve_canyun(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload[0] != 0x4359_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-canyun continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    if observer != player {
        return Err(EffectError::Illegal(
            "resolve-canyun context changed".into(),
        ));
    }
    match payload[3] {
        2 => {
            if cancelled {
                return Ok(());
            }
            let card = selections
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("resolve-canyun discard required".into()))?;
            crate::effect::resolve_canyun_discarded(tx, player, skill, card)?;
            Ok(())
        }
        3 => {
            if cancelled {
                return Ok(());
            }
            let discard_card = payload[4];
            if selections.is_empty() || selections.len() > 2 {
                return Err(EffectError::Illegal(
                    "resolve-canyun target count out of range".into(),
                ));
            }
            crate::effect::resolve_canyun_resolve(tx, player, skill, discard_card, selections)?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault("bad resolve-canyun stage".into())),
    }
}

/// 养名循环分派：0=目标已选；>=2 为拼点/知情轮。
pub(crate) fn resolve_resolve_yangming(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 || payload[0] != 0x594D_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-yangming continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    if observer != player {
        return Err(EffectError::Illegal(
            "resolve-yangming context changed".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    let target = selections
        .first()
        .copied()
        .ok_or_else(|| EffectError::Illegal("resolve-yangming target required".into()))?;
    crate::effect::resolve_yangming_stage(tx, player, skill, target, 0)?;
    Ok(())
}

/// 养名拼点/知情轮分派：payload[3] >= 2（2 owner 选牌/3 目标选牌/4 重复知情）。
pub(crate) fn resolve_resolve_yangming_round(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
) -> Result<(), EffectError> {
    if payload.len() < 6 || payload[0] != 0x594D_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-yangming-round continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    crate::effect::resolve_yangming_round(tx, observer, player, skill, payload, selections)?;
    Ok(())
}

/// 迷途拼点轮分派：payload[3] >= 3（3 选对手/4 知情/5 目标选牌/6 对手选牌）。
pub(crate) fn resolve_resolve_twmitu_round(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
) -> Result<(), EffectError> {
    if payload.len() < 6 || payload[0] != 0x544D_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-twmitu-round continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    crate::effect::resolve_twmitu_round(tx, observer, player, skill, payload, selections)?;
    Ok(())
}

/// 待归 stage 3 轮次分派：payload[3] == 3 时为逐目标选获轮（异观察者）。
pub(crate) fn resolve_resolve_twdaigui_round(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload[0] != 0x5444_0001 || payload[3] != 3 {
        return Err(EffectError::EngineFault(
            "bad resolve-twdaigui-round continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    crate::effect::resolve_twdaigui_round(tx, observer, player, skill, payload, selections)
}

pub(crate) fn resolve_resolve_jsrgdaoren(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 6 || payload[0] != 0x4A53_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-jsrgdaoren continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let stage = payload[3];
    let card = payload[4];
    let target = payload[5];
    if observer != player || cancelled {
        return Err(EffectError::Illegal(
            "resolve-jsrgdaoren context changed".into(),
        ));
    }
    match stage {
        1 => {
            let card = selections
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("resolve-jsrgdaoren card required".into()))?;
            crate::effect::jsrgdaoren_entry(tx, player, skill, 1, card, 0)?;
            Ok(())
        }
        2 => {
            let target = selections
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("resolve-jsrgdaoren target required".into()))?;
            crate::effect::jsrgdaoren_entry(tx, player, skill, 2, card, target)?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault(
            "resolve-jsrgdaoren unknown stage".into(),
        )),
    }
}

pub(crate) fn resolve_resolve_pxqixian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 7 || payload[0] != 0x5078_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-pxqixian continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let stage = payload[3];
    let kind = payload[4];
    let material = payload[5];
    let target = payload[6];
    if observer != player || cancelled {
        return Err(EffectError::Illegal(
            "resolve-pxqixian context changed".into(),
        ));
    }
    match stage {
        // 阶段 1 确认：selections[0] = 1(杀) / 2(拆)。
        1 => {
            let kind = match selections.first().copied() {
                Some(1) => 1u32,
                Some(2) => 2u32,
                _ => {
                    return Err(EffectError::Illegal(
                        "resolve-pxqixian kind must be 1 or 2".into(),
                    ))
                }
            };
            crate::effect::pxqixian_step(tx, player, skill, 2, kind, 0, 0)?;
            Ok(())
        }
        // 阶段 2 素材：selections[0] = 素材牌。
        2 => {
            let material = selections
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("resolve-pxqixian material required".into()))?;
            crate::effect::pxqixian_step(tx, player, skill, 3, kind, material, 0)?;
            Ok(())
        }
        // 阶段 3 目标：selections[0] = 目标。
        3 => {
            let target = selections
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("resolve-pxqixian target required".into()))?;
            crate::effect::pxqixian_step(tx, player, skill, 4, kind, material, target)?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault(
            "resolve-pxqixian unknown stage".into(),
        )),
    }
}

pub(crate) fn resolve_resolve_dcbingji(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 || payload[0] != 0xDCB0_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-dcbingji continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let stage = payload[3];
    let kind = payload[4];
    if observer != player || cancelled {
        return Err(EffectError::Illegal(
            "resolve-dcbingji context changed".into(),
        ));
    }
    match stage {
        // 阶段 1 确认：selections[0] = 1(杀) / 2(桃)。
        1 => {
            let kind = match selections.first().copied() {
                Some(1) => 1u32,
                Some(2) => 2u32,
                _ => {
                    return Err(EffectError::Illegal(
                        "resolve-dcbingji kind must be 1 or 2".into(),
                    ))
                }
            };
            crate::effect::resolve_dcbingji_entry(tx, player, skill, 1, kind)?;
            Ok(())
        }
        // 阶段 2 选目标（杀）：selections[0] = 目标。
        2 => {
            let target = selections
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("resolve-dcbingji target required".into()))?;
            crate::effect::resolve_dcbingji_entry(tx, player, skill, 2, target)?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault(
            "resolve-dcbingji unknown stage".into(),
        )),
    }
}

pub(crate) fn resolve_choose_hand_cards_to_skill_pile(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad choose-hand-cards-to-skill-pile continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let skill = engine_api::SkillId::new(payload[1]);
    let min_count = payload[2];
    let max_count = payload[3];
    if observer != player {
        return Err(EffectError::Illegal(
            "choose-hand-cards-to-skill-pile context changed".into(),
        ));
    }
    // 取消 = 不置入（上游 chooseCard bool=false 的合法路径）。
    if cancelled {
        return Ok(());
    }
    if selections.len() < min_count as usize || selections.len() > max_count as usize {
        return Err(EffectError::Illegal(
            "choose-hand-cards-to-skill-pile selection count out of range".into(),
        ));
    }
    let skill_def = crate::effect::skill_pile_def(tx, player, skill)?;
    let legal: std::collections::BTreeSet<_> = tx.state().hand_cards(player).into_iter().collect();
    let selected: std::collections::BTreeSet<_> =
        selections.iter().map(|&card| CardId::new(card)).collect();
    if selected.len() != selections.len() || !selected.is_subset(&legal) {
        return Err(EffectError::Illegal(
            "skill-pile placement must select distinct current hand cards".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let pile = crate::effect::ensure_skill_pile_zone(tx, player, skill_def)?;
    let cards: Vec<CardId> = selections.iter().map(|&card| CardId::new(card)).collect();
    crate::effect::move_cards(tx, cards, pile, Some(hand))?;
    Ok(())
}

pub(crate) fn resolve_store_selected_in_skill_pile(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 2 {
        return Err(EffectError::EngineFault(
            "bad store-selected-in-skill-pile continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let skill = engine_api::SkillId::new(payload[1]);
    if observer != player || cancelled {
        return Err(EffectError::Illegal(
            "store-selected-in-skill-pile context changed".into(),
        ));
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "store-selected-in-skill-pile requires exactly one hand card".into(),
        ));
    }
    let skill_def = crate::effect::skill_pile_def(tx, player, skill)?;
    let legal: std::collections::BTreeSet<_> = tx.state().hand_cards(player).into_iter().collect();
    let card = CardId::new(selections[0]);
    if !legal.contains(&card) {
        return Err(EffectError::Illegal(
            "skill-pile storage must select a current hand card".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let pile = crate::effect::ensure_skill_pile_zone(tx, player, skill_def)?;
    crate::effect::move_cards(tx, vec![card], pile, Some(hand))?;
    Ok(())
}

pub(crate) fn resolve_rearrange_pile_to_deck_top(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad rearrange-pile-to-deck-top continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let _skill = engine_api::SkillId::new(payload[1]);
    let pile_def = payload[2];
    let count = payload[3];
    if observer != player || cancelled {
        return Err(EffectError::Illegal(
            "rearrange-pile-to-deck-top context changed".into(),
        ));
    }
    if selections.len() != (count as usize).min(tx.state().skill_pile_cards(player, pile_def).len())
    {
        return Err(EffectError::Illegal(
            "rearrange-pile-to-deck-top selection count mismatch".into(),
        ));
    }
    let pile = tx
        .state()
        .skill_piles
        .iter()
        .find(|p| p.owner == player && p.skill_def == pile_def)
        .map(|p| p.zone)
        .ok_or_else(|| EffectError::Illegal("skill pile missing".into()))?;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    // validate all selections are in the pile
    let pile_set: std::collections::BTreeSet<_> = tx
        .state()
        .skill_pile_cards(player, pile_def)
        .into_iter()
        .collect();
    for &sel in selections {
        if !pile_set.contains(&CardId::new(sel)) {
            return Err(EffectError::Illegal(
                "selected card not in skill pile".into(),
            ));
        }
    }
    for &sel in selections {
        crate::effect::move_one_card(tx, CardId::new(sel), pile, draw, PlayerId::NONE, true)?;
    }
    Ok(())
}

pub(crate) fn resolve_place_hand_cards_to_deck_top(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad place-hand-cards-to-deck-top continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let _skill = engine_api::SkillId::new(payload[1]);
    let count = payload[2];
    if observer != player || cancelled {
        return Err(EffectError::Illegal(
            "place-hand-cards-to-deck-top context changed".into(),
        ));
    }
    if selections.len() != count as usize {
        return Err(EffectError::Illegal(
            "place-hand-cards-to-deck-top selection count mismatch".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    // validate all selections are in hand
    let hand_set: std::collections::BTreeSet<_> = tx
        .state()
        .zones
        .get(hand)
        .unwrap()
        .cards
        .iter()
        .copied()
        .collect();
    for &sel in selections {
        if !hand_set.contains(&CardId::new(sel)) {
            return Err(EffectError::Illegal("selected card not in hand".into()));
        }
    }
    // remove from hand (reverse order to keep indices valid)
    for &sel in selections.iter().rev() {
        tx.state_mut()
            .zones
            .get_mut(hand)
            .unwrap()
            .cards
            .retain(|c| *c != CardId::new(sel));
    }
    // push onto draw pile top (end of vec = top), in reverse selection order
    // so that first-selected ends up highest
    for &sel in selections.iter() {
        tx.state_mut()
            .zones
            .get_mut(draw)
            .unwrap()
            .cards
            .push(CardId::new(sel));
    }
    // update card zones
    for &sel in selections {
        if let Some(card_state) = tx.state_mut().cards.get_mut(CardId::new(sel)) {
            card_state.zone = draw;
            card_state.owner = PlayerId::NONE;
        }
    }
    Ok(())
}

pub(crate) fn resolve_guanxing_pick(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let count = *payload.get(1).unwrap_or(&0);
    let round = *payload.get(2).unwrap_or(&0);
    let flipped_len = *payload.get(3).unwrap_or(&0) as usize;
    let skill = engine_api::SkillId::new(*payload.last().unwrap_or(&0));
    let top_start = 4 + flipped_len;
    let bottom_start = top_start + 1; // top_len occupies one slot
    if cancelled
        || observer != player
        || selections.len() != 1
        || round >= count
        || payload.len() < bottom_start + 2
    {
        return Err(EffectError::Illegal("bad guanxing pick context".into()));
    }
    let Some(current) = payload.get(4 + round as usize).copied().map(CardId::new) else {
        return Err(EffectError::Illegal("guanxing card missing".into()));
    };
    let top_len = *payload.get(top_start).unwrap_or(&0) as usize;
    let top: Vec<_> = payload
        .get(top_start + 1..top_start + 1 + top_len)
        .unwrap_or_default()
        .to_vec();
    let bottom_len = *payload.get(bottom_start).unwrap_or(&0) as usize;
    let bottom: Vec<_> = payload
        .get(bottom_start + 1..bottom_start + 1 + bottom_len)
        .unwrap_or_default()
        .to_vec();
    let pick_top = selections[0] == 1;
    let mut next_top = top.clone();
    let mut next_bottom = bottom.clone();
    if pick_top {
        // 近似堆叠序：后选者叠在先选者之上。
        next_top.insert(0, current.raw());
    } else {
        next_bottom.push(current.raw());
    }
    let round_next = round + 1;
    let mut next_payload = vec![player.raw(), count, round_next, flipped_len as u32];
    next_payload.extend(payload.get(4..4 + flipped_len).unwrap_or_default().to_vec());
    next_payload.push(next_top.len() as u32);
    next_payload.extend(next_top.iter().copied());
    next_payload.push(next_bottom.len() as u32);
    next_payload.extend(next_bottom.iter().copied());
    next_payload.push(skill.raw());
    if round_next < count {
        crate::effect::open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![1, 2],
            engine_api::ChoiceKind::SelectTargets,
            crate::state::ContinuationKind::GuanxingPick,
            next_payload,
        )
        .map(|_| ())
    } else {
        crate::effect::commit_guanxing_top(
            tx,
            player,
            &next_top
                .iter()
                .copied()
                .map(CardId::new)
                .collect::<Vec<_>>(),
            &next_bottom
                .iter()
                .copied()
                .map(CardId::new)
                .collect::<Vec<_>>(),
            skill,
        )
        .map(|_| ())
    }
}

/// ABI 682 (batch 180): resume — giver chosen; they stash one hand card to the chooser,
/// then recover 1 HP themselves (psdaohe).
pub(crate) fn resolve_choose_other_give_then_recover(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(1).unwrap_or(&0));
    if cancelled || observer != player || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose-other give-then-recover must choose exactly one giver".into(),
        ));
    }
    let giver = PlayerId::new(selections[0]);
    let state = tx.state();
    if !state.skills.get(skill).is_some_and(|instance| {
        instance.owner == player && state.skill_effectively_enabled(instance)
    }) || giver == player
        || !state
            .players
            .get(giver)
            .is_some_and(|participant| participant.alive)
        || state.hand_cards(giver).is_empty()
    {
        return Err(EffectError::Illegal(
            "choose-other give-then-recover context changed".into(),
        ));
    }
    // giver 交一张手牌给 player（DrawThenGiveOne：第一参为交牌者），
    // 然后 giver 自回 1。
    run_effects(
        tx,
        vec![
            EffectRequest::DrawThenGiveOne {
                player: giver,
                target: player,
                count: 0,
                drawn_only: false,
                use_given_equipment: false,
                skill,
                drawn: Vec::new(),
                ready_to_give: true,
            },
            EffectRequest::RecoverHp {
                source: player,
                player: giver,
                amount: 1,
            },
        ],
    )
    .map(|_| ())
}
