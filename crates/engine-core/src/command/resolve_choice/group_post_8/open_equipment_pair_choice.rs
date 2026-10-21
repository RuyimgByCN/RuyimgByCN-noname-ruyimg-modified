//! Command operations for `open_equipment_pair_choice` and related helpers.

use super::*;

pub(crate) fn open_equipment_pair_choice(
    tx: &mut Transaction,
    chooser: PlayerId,
    skill: engine_api::SkillId,
    target0: PlayerId,
    target1: PlayerId,
    first: CardId,
    options: Vec<CardId>,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    crate::effect::open_choice(
        tx,
        chooser,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        ContinuationKind::DiscardEquipmentPairWithArmor,
        vec![
            1,
            chooser.raw(),
            skill.raw(),
            target0.raw(),
            target1.raw(),
            first.raw(),
        ],
    )
}

pub(crate) fn resolve_discard_target_equipment_gain_mount(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    resolve_scoped_discard_target_card_gain_mount(
        tx,
        observer,
        payload,
        selections,
        cancelled,
        rule_api::TargetCardScope::Equip,
        true,
    )
}

pub(crate) fn resolve_discard_target_equipment_draw_weapon_range(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    let option_count = *payload
        .get(3)
        .ok_or_else(|| EffectError::EngineFault("truncated equipment options".into()))?
        as usize;
    if payload.len() != 4 + option_count {
        return Err(EffectError::EngineFault(
            "invalid equipment option count".into(),
        ));
    }
    if cancelled || observer != player || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one target equipment card".into(),
        ));
    }
    let selected = CardId::new(selections[0]);
    if !payload[4..].contains(&selected.raw())
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
        || !crate::query::discardable_equipment_cards(tx.state(), player, target)
            .contains(&selected)
    {
        return Err(EffectError::Illegal(
            "selected target equipment is stale".into(),
        ));
    }
    let draw_count = tx
        .state()
        .cards
        .get(selected)
        .filter(|card| card.kind == CardKind::Weapon)
        .map(|card| u32::from(card.weapon_range.max(1)))
        .unwrap_or(0);
    let mut effects = vec![EffectRequest::DiscardTargetCards {
        actor: player,
        target,
        cards: vec![selected],
        scope: rule_api::TargetCardScope::Equip,
    }];
    if draw_count > 0 {
        effects.push(EffectRequest::DrawCardsFromSkill {
            player: target,
            count: draw_count,
            skill,
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_choose_target_reveal_hand_damage_if_sha(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, amount, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad reveal-hand damage continuation".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let skill = engine_api::SkillId::new(*skill);
    if cancelled || observer != player || selections.len() != 1 || *amount == 0 {
        return Err(EffectError::Illegal(
            "choose exactly one reveal-hand target".into(),
        ));
    }
    let target = PlayerId::new(selections[0]);
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) || !tx
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
        return Err(EffectError::Illegal(
            "reveal-hand damage context changed".into(),
        ));
    }
    let cards = tx.state().hand_cards(target);
    if cards.is_empty() {
        return Err(EffectError::Illegal(
            "selected reveal-hand target has no hand cards".into(),
        ));
    }
    let contains_sha = cards.iter().any(|card| {
        tx.state().cards.get(*card).is_some_and(|card| {
            crate::query::effective_card_kind(tx.state(), target, card).is_sha()
        })
    });
    let mut effects = vec![EffectRequest::RevealCards {
        cards,
        to: Vec::new(),
    }];
    if contains_sha {
        effects.push(EffectRequest::DealDamage {
            source: player,
            target,
            amount: *amount,
            is_lose_hp: false,
            nature: 0,
        });
    }
    run_effects(tx, effects).map(|_| ())
}

/// ABI 681 guanxing_top: one round of the top/bottom pick loop.
pub(crate) fn resolve_choose_one_from_skill_pile_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 || selections.len() != 1 {
        return Err(EffectError::EngineFault(
            "bad choose-one-from-skill-pile-discard continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let pile_def = payload[1];
    let _skill = engine_api::SkillId::new(payload[2]);
    let card = CardId::new(selections[0]);
    if observer != player
        || cancelled
        || !tx
            .state()
            .skill_pile_cards(player, pile_def)
            .contains(&card)
    {
        return Err(EffectError::Illegal(
            "choose-one-from-skill-pile-discard context changed".into(),
        ));
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let pile = tx
        .state()
        .skill_piles
        .iter()
        .find(|p| p.owner == player && p.skill_def == pile_def)
        .map(|p| p.zone)
        .ok_or_else(|| EffectError::Illegal("skill pile missing".into()))?;
    tx.state_mut()
        .zones
        .get_mut(pile)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(card);
    if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
        card_state.zone = discard;
    }
    Ok(())
}

pub(crate) fn resolve_resolve_qianlong(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    _cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 6 || payload[0] != 0x51_4C_4E_01 {
        return Err(EffectError::EngineFault(
            "bad resolve-qianlong continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let _skill = engine_api::SkillId::new(payload[2]);
    let top3 = [payload[3], payload[4], payload[5]];
    if observer != player {
        return Err(EffectError::Illegal(
            "resolve-qianlong context changed".into(),
        ));
    }
    // selections = 获得牌（0..=min(3, 已损失体力)），须 ⊆ 顶三张。
    let participant = tx.state().players.get(player).unwrap();
    let lost = (participant.max_hp as i32 - participant.hp).max(0) as usize;
    if selections.len() > lost.min(3) {
        return Err(EffectError::Illegal(
            "resolve-qianlong selection exceeds lost-hp cap".into(),
        ));
    }
    let top3_set: std::collections::BTreeSet<u32> = top3.iter().copied().collect();
    if !selections.iter().all(|s| top3_set.contains(s)) {
        return Err(EffectError::Illegal(
            "resolve-qianlong selection outside top three".into(),
        ));
    }
    crate::effect::resolve_qianlong_stage(tx, player, top3, selections)?;
    Ok(())
}

pub(crate) fn resolve_resolve_jsrgzuozhan(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    _cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 || payload[0] != 0x5A55_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-jsrgzuozhan continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    if observer != player {
        return Err(EffectError::Illegal(
            "resolve-jsrgzuozhan context changed".into(),
        ));
    }
    // selections = 「坐瞻」角色（1..=2），须为其他存活角色。
    if selections.is_empty() || selections.len() > 2 {
        return Err(EffectError::Illegal(
            "resolve-jsrgzuozhan target count out of range".into(),
        ));
    }
    for raw in selections {
        let candidate = PlayerId::new(*raw);
        if candidate == player
            || !tx
                .state()
                .players
                .get(candidate)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal(
                "resolve-jsrgzuozhan target outside alive others".into(),
            ));
        }
    }
    crate::effect::resolve_jsrgzuozhan_stage(tx, player, skill, selections)?;
    Ok(())
}

pub(crate) fn resolve_resolve_jsrgzuozhan_gain(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    _cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 || payload[0] != 0x5A55_0003 {
        return Err(EffectError::EngineFault(
            "bad resolve-jsrgzuozhan-gain continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let event_player = PlayerId::new(payload[3]);
    if observer != player {
        return Err(EffectError::Illegal(
            "resolve-jsrgzuozhan-gain context changed".into(),
        ));
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "resolve-jsrgzuozhan-gain requires exactly one target".into(),
        ));
    }
    crate::effect::resolve_jsrgzuozhan_gain_stage(tx, player, skill, event_player, selections)?;
    Ok(())
}

pub(crate) fn resolve_resolve_twdaigui(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 || payload[0] != 0x5444_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-twdaigui continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    if observer != player {
        return Err(EffectError::Illegal(
            "resolve-twdaigui context changed".into(),
        ));
    }
    if cancelled {
        // 可取消 = 不发动待归。
        return Ok(());
    }
    if selections.is_empty() {
        return Err(EffectError::Illegal(
            "resolve-twdaigui requires at least one target".into(),
        ));
    }
    crate::effect::resolve_twdaigui_stage(tx, player, skill, selections)?;
    Ok(())
}

pub(crate) fn resolve_resolve_twmitu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 || payload[0] != 0x544D_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-twmitu continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    if observer != player {
        return Err(EffectError::Illegal(
            "resolve-twmitu context changed".into(),
        ));
    }
    if cancelled {
        // 可取消 = 不发动迷途。
        return Ok(());
    }
    if selections.is_empty() || selections.len() > 3 {
        return Err(EffectError::Illegal(
            "resolve-twmitu target count out of range".into(),
        ));
    }
    crate::effect::resolve_twmitu_stage(tx, player, skill, selections)?;
    Ok(())
}

/// 谦吞分段分派：0=目标已选 → 2=展示已定 → 3=owner 拼点牌 → 4=目标拼点牌
/// → 结算。
pub(crate) fn resolve_resolve_qiantun(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload[0] != 0x514E_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-qiantun continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let target_raw = *payload.get(4).unwrap_or(&0);
    match payload[3] {
        0 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-qiantun context changed".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let target = selections
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("resolve-qiantun target required".into()))?;
            crate::effect::resolve_qiantun_stage(tx, player, skill, target)?;
            Ok(())
        }
        2 => {
            let target = PlayerId::new(target_raw);
            if observer != target {
                return Err(EffectError::Illegal(
                    "resolve-qiantun show requires the target".into(),
                ));
            }
            if selections.is_empty() {
                return Err(EffectError::Illegal(
                    "resolve-qiantun show at least one card".into(),
                ));
            }
            let target_hand: std::collections::BTreeSet<u32> = tx
                .state()
                .hand_cards(target)
                .iter()
                .map(|card| card.raw())
                .collect();
            if !selections.iter().all(|card| target_hand.contains(card)) {
                return Err(EffectError::Illegal(
                    "resolve-qiantun show outside target hand".into(),
                ));
            }
            crate::effect::resolve_qiantun_shown(tx, player, skill, target_raw, selections)?;
            Ok(())
        }
        3 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-qiantun compare card requires owner".into(),
                ));
            }
            let k = *payload
                .get(5)
                .ok_or_else(|| EffectError::EngineFault("bad resolve-qiantun payload".into()))?
                as usize;
            if payload.len() < 6 + k {
                return Err(EffectError::EngineFault(
                    "bad resolve-qiantun payload layout".into(),
                ));
            }
            let shown: Vec<u32> = payload[6..6 + k].to_vec();
            let player_card = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-qiantun compare card required".into())
            })?;
            let player_hand: std::collections::BTreeSet<u32> = tx
                .state()
                .hand_cards(player)
                .iter()
                .map(|card| card.raw())
                .collect();
            if !player_hand.contains(&player_card) {
                return Err(EffectError::Illegal(
                    "resolve-qiantun compare card outside owner hand".into(),
                ));
            }
            crate::effect::resolve_qiantun_player_card(
                tx,
                player,
                skill,
                target_raw,
                &shown,
                player_card,
            )?;
            Ok(())
        }
        4 => {
            let target = PlayerId::new(target_raw);
            if observer != target {
                return Err(EffectError::Illegal(
                    "resolve-qiantun compare card requires the target".into(),
                ));
            }
            let k = *payload
                .get(5)
                .ok_or_else(|| EffectError::EngineFault("bad resolve-qiantun payload".into()))?
                as usize;
            if payload.len() < 7 + k {
                return Err(EffectError::EngineFault(
                    "bad resolve-qiantun payload layout".into(),
                ));
            }
            let shown: Vec<u32> = payload[6..6 + k].to_vec();
            let player_card = payload[6 + k];
            let target_card = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-qiantun compare card required".into())
            })?;
            crate::effect::resolve_qiantun_resolve(
                tx,
                player,
                skill,
                target_raw,
                &shown,
                player_card,
                target_card,
            )?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault("bad resolve-qiantun stage".into())),
    }
}

/// 良姻分段分派：0=受益/弃牌角色已选；9=mode1 子续体（弃牌选定）。
pub(crate) fn resolve_resolve_liangyin(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload[0] != 0x524C_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-liangyin continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let mode = payload[3];
    if mode == 9 {
        // mode 1 子续体：who 选定的手牌弃置。
        let who = PlayerId::new(payload[1]);
        if observer != who {
            return Err(EffectError::Illegal(
                "resolve-liangyin discard requires the player".into(),
            ));
        }
        let card = selections
            .first()
            .copied()
            .ok_or_else(|| EffectError::Illegal("resolve-liangyin discard card required".into()))?;
        crate::effect::resolve_reliangyin_discard(tx, who, card)?;
        return Ok(());
    }
    if observer != player {
        return Err(EffectError::Illegal(
            "resolve-liangyin context changed".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    let who = selections
        .first()
        .copied()
        .ok_or_else(|| EffectError::Illegal("resolve-liangyin target required".into()))?;
    match mode {
        0 => {
            crate::effect::draw_cards(tx, PlayerId::new(who), 1)?;
            Ok(())
        }
        1 => {
            eprintln!(
                "PROBE rl mode1: who={} hand_n={} options_n={}",
                who,
                tx.state().hand_cards(PlayerId::new(who)).len(),
                selections.len()
            );
            crate::effect::resolve_reliangyin_stage(tx, player, skill, 1, who)?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault("bad liangyin mode".into())),
    }
}

/// 诤荐分段分派：0=授标目标已选（可取消）→ 标 +1。
pub(crate) fn resolve_resolve_zhengjian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 || payload[0] != 0x5A4A_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-zhengjian continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    if observer != player {
        return Err(EffectError::Illegal(
            "resolve-zhengjian context changed".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    let target = selections
        .first()
        .copied()
        .ok_or_else(|| EffectError::Illegal("resolve-zhengjian target required".into()))?;
    crate::effect::add_mark(tx, PlayerId::new(target), crate::effect::ZHENGJIAN_MARK, 1)?;
    Ok(())
}

/// 率言分段分派：0=目标已选 → 2=目标选牌已定 → 交予 owner。
pub(crate) fn resolve_resolve_shuaiyan(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload[0] != 0x5348_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-shuaiyan continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    match payload[3] {
        0 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-shuaiyan context changed".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let target = selections
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("resolve-shuaiyan target required".into()))?;
            crate::effect::resolve_shuaiyan_stage(tx, player, skill, target)?;
            Ok(())
        }
        2 => {
            let target = PlayerId::new(payload[4]);
            if observer != target {
                return Err(EffectError::Illegal(
                    "resolve-shuaiyan give requires the target".into(),
                ));
            }
            crate::effect::resolve_shuaiyan_give(
                tx,
                player,
                skill,
                payload[4],
                selections.first().copied().ok_or_else(|| {
                    EffectError::Illegal("resolve-shuaiyan give card required".into())
                })?,
            )?;
            Ok(())
        }
        3 => {
            let target = PlayerId::new(payload[4]);
            if observer != target {
                return Err(EffectError::Illegal(
                    "resolve-shuaiyan give requires the target".into(),
                ));
            }
            let card = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-shuaiyan give card required".into())
            })?;
            crate::effect::resolve_shuaiyan_give(tx, player, skill, target.raw(), card)?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault(
            "bad resolve-shuaiyan stage".into(),
        )),
    }
}

pub(crate) fn resolve_resolve_mbbeiming(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 || payload[0] != 0x4242_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-mbbeiming continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    match payload[3] {
        0 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-mbbeiming context changed".into(),
                ));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            crate::effect::resolve_mbbeiming_grant(tx, player, skill, selections)?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault(
            "bad resolve-mbbeiming stage".into(),
        )),
    }
}

pub(crate) fn resolve_resolve_gaoyuan(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || payload[0] != 0x4747_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-gaoyuan continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let source = PlayerId::new(payload[2]);
    let card = rule_api::CardId::new(payload[3]);
    match payload[4] {
        1 if payload.len() == 5 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-gaoyuan context changed".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "resolve-gaoyuan requires exactly one card".into(),
                ));
            };
            let selected = rule_api::CardId::new(*selected);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&selected) {
                return Err(EffectError::Illegal(
                    "resolve-gaoyuan discard cost is invalid or stale".into(),
                ));
            }
            let candidates = crate::effect::gaoyuan_redirect_candidates(tx, player, source, card)?;
            crate::effect::open_choice(
                tx,
                player,
                0,
                1,
                true,
                candidates.into_iter().map(PlayerId::raw).collect(),
                engine_api::ChoiceKind::SelectTargets,
                crate::state::ContinuationKind::ResolveGaoyuanStage,
                vec![
                    0x4747_0001,
                    player.raw(),
                    source.raw(),
                    card.raw(),
                    2,
                    selected.raw(),
                ],
            )
            .map(|_| ())
        }
        2 if payload.len() == 6 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-gaoyuan context changed".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let [target_raw] = selections else {
                return Err(EffectError::Illegal(
                    "resolve-gaoyuan requires exactly one target".into(),
                ));
            };
            let target = PlayerId::new(*target_raw);
            let selected = rule_api::CardId::new(payload[5]);
            let candidates = crate::effect::gaoyuan_redirect_candidates(tx, player, source, card)?;
            if !candidates.contains(&target) {
                return Err(EffectError::Illegal(
                    "resolve-gaoyuan target is invalid or stale".into(),
                ));
            }
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&selected) {
                return Err(EffectError::Illegal(
                    "resolve-gaoyuan discard cost is invalid or stale".into(),
                ));
            }
            // 与上游 content 顺序一致：先弃牌，转移提交作为尾随效果延后。
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::CommitGaoyuanRedirect {
                    player,
                    source,
                    card,
                    target,
                },
            );
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![selected],
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("bad resolve-gaoyuan stage".into())),
    }
}

pub(crate) fn resolve_resolve_duoduan(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 || payload[0] != 0x4444_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-duoduan continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let source = PlayerId::new(payload[2]);
    let card = rule_api::CardId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    match payload[5] {
        1 if payload.len() == 6 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-duoduan context changed".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "resolve-duoduan requires exactly one card".into(),
                ));
            };
            let selected = rule_api::CardId::new(*selected);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&selected) {
                return Err(EffectError::Illegal(
                    "resolve-duoduan recast cost is invalid or stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::RecastCards {
                    player,
                    cards: vec![selected],
                }],
            )?;
            // 使用者无可弃牌 → 强制选项①（上游 result 直接落 {index:0}）。
            if crate::query::discardable_hand_equip_cards(tx.state(), source).is_empty() {
                crate::effect::resolve_duoduan_draw_and_cancel(tx, source, player, card)?;
                return Ok(());
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![1, 2],
                engine_api::ChoiceKind::Custom,
                crate::state::ContinuationKind::ResolveDuoduanStage,
                vec![
                    0x4444_0001,
                    player.raw(),
                    source.raw(),
                    card.raw(),
                    skill.raw(),
                    2,
                ],
            )
            .map(|_| ())
        }
        2 if payload.len() == 6 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-duoduan context changed".into(),
                ));
            }
            let [choice] = selections else {
                return Err(EffectError::Illegal(
                    "resolve-duoduan requires exactly one option".into(),
                ));
            };
            match *choice {
                1 => crate::effect::resolve_duoduan_draw_and_cancel(tx, source, player, card)
                    .map(|_| ()),
                2 => {
                    let options = crate::query::discardable_hand_equip_cards(tx.state(), source);
                    if options.is_empty() {
                        // 与上游一致：无可弃则仍走选项①语义。
                        crate::effect::resolve_duoduan_draw_and_cancel(tx, source, player, card)
                            .map(|_| ())
                    } else {
                        crate::effect::open_choice(
                            tx,
                            source,
                            1,
                            1,
                            false,
                            options.into_iter().map(CardId::raw).collect(),
                            engine_api::ChoiceKind::SelectCards,
                            crate::state::ContinuationKind::ResolveDuoduanStage,
                            vec![
                                0x4444_0001,
                                player.raw(),
                                source.raw(),
                                card.raw(),
                                skill.raw(),
                                3,
                            ],
                        )
                        .map(|_| ())
                    }
                }
                _ => Err(EffectError::Illegal(
                    "resolve-duoduan unknown option".into(),
                )),
            }
        }
        3 if payload.len() == 6 => {
            if observer != source {
                return Err(EffectError::Illegal(
                    "resolve-duoduan discard requires the sha user".into(),
                ));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "resolve-duoduan requires exactly one card".into(),
                ));
            };
            let selected = rule_api::CardId::new(*selected);
            if !crate::query::discardable_hand_equip_cards(tx.state(), source).contains(&selected) {
                return Err(EffectError::Illegal(
                    "resolve-duoduan discard cost is invalid or stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: source,
                    cards: vec![selected],
                }],
            )?;
            crate::command::play::make_saved_sha_target_unrespondable(tx, source, card, player);
            Ok(())
        }
        _ => Err(EffectError::EngineFault("bad resolve-duoduan stage".into())),
    }
}

pub(crate) fn resolve_resolve_dcanxu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    _cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 || payload[0] != 0x4443_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-dcanxu continuation payload".into(),
        ));
    }
    let owner = PlayerId::new(payload[1]);
    let gainner = PlayerId::new(payload[2]);
    let giver = PlayerId::new(payload[3]);
    match payload[4] {
        1 => {
            if observer != gainner {
                return Err(EffectError::Illegal(
                    "resolve-dcanxu gain requires the lower-hand player".into(),
                ));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "resolve-dcanxu requires exactly one card".into(),
                ));
            };
            let selected = rule_api::CardId::new(*selected);
            let giver_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, giver)
                .ok_or_else(|| EffectError::Illegal("no giver hand".into()))?;
            if !tx
                .state()
                .zones
                .get(giver_hand)
                .unwrap()
                .cards
                .contains(&selected)
            {
                return Err(EffectError::Illegal(
                    "resolve-dcanxu card is not in the giver hand".into(),
                ));
            }
            let suit = tx.state().cards.get(selected).map(|card| card.suit);
            // 移牌：多者手牌 → 少者手牌。
            let gainner_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, gainner)
                .ok_or_else(|| EffectError::Illegal("no gainner hand".into()))?;
            tx.state_mut()
                .zones
                .get_mut(giver_hand)
                .unwrap()
                .cards
                .retain(|card| *card != selected);
            tx.state_mut()
                .zones
                .get_mut(gainner_hand)
                .unwrap()
                .cards
                .push(selected);
            if let Some(card_state) = tx.state_mut().cards.get_mut(selected) {
                card_state.zone = gainner_hand;
                card_state.owner = gainner;
            }
            // 非黑桃所得 → owner 摸 1。
            if suit != Some(engine_api::Suit::Spade) {
                run_effects(
                    tx,
                    vec![EffectRequest::DrawCards {
                        player: owner,
                        count: 1,
                    }],
                )?;
            }
            // 手牌数相等 → owner 回 1。
            if tx.state().hand_count(gainner) == tx.state().hand_count(giver) {
                run_effects(
                    tx,
                    vec![EffectRequest::RecoverHp {
                        source: engine_api::PlayerId::NONE,
                        player: owner,
                        amount: 1,
                    }],
                )?;
            }
            Ok(())
        }
        _ => Err(EffectError::EngineFault("bad resolve-dcanxu stage".into())),
    }
}

pub(crate) fn resolve_resolve_chulao(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 || payload[0] != 0x434C_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-chulao continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[2]);
    match payload[1] {
        1 if payload.len() >= 4 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-chulao context changed".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "resolve-chulao requires exactly one card".into(),
                ));
            };
            let cost = rule_api::CardId::new(*selected);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&cost) {
                return Err(EffectError::Illegal(
                    "resolve-chulao cost is invalid or stale".into(),
                ));
            }
            // 弃置成本牌；成本牌为黑桃 → owner 摸 1（上游 step 0 判定）。
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![cost],
                }],
            )?;
            if tx
                .state()
                .cards
                .get(cost)
                .is_some_and(|card| card.suit == engine_api::Suit::Spade)
            {
                run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }])?;
            }
            let mut next = vec![0x434C_0001, 2, player.raw(), cost.raw(), 0u32];
            let targets_raw = payload[3..].to_vec();
            next.extend_from_slice(&targets_raw);
            // 打开对首目标的选牌。
            let first = targets_raw
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("resolve-chulao no targets".into()))?;
            chulao_open_target_pick(tx, player, &mut next, first, 0)
        }
        2 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-chulao context changed".into(),
                ));
            }
            if payload.len() < 5 {
                return Err(EffectError::EngineFault(
                    "bad resolve-chulao target payload".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "resolve-chulao requires exactly one card".into(),
                ));
            };
            let selected = rule_api::CardId::new(*selected);
            let idx = payload[4] as usize;
            let target = PlayerId::new(payload[5 + idx]);
            // 目标的手牌+装备区。
            let mut options = crate::query::discardable_hand_equip_cards(tx.state(), target);
            options.retain(|card| *card == selected);
            if options.is_empty() {
                return Err(EffectError::Illegal(
                    "resolve-chulao pick is not in the target zones".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: vec![selected],
                }],
            )?;
            // 被弃牌黑桃 → 该目标摸 1。
            let mut events = Vec::new();
            if tx
                .state()
                .cards
                .get(selected)
                .is_some_and(|card| card.suit == engine_api::Suit::Spade)
            {
                events.push(EffectRequest::DrawCards {
                    player: target,
                    count: 1,
                });
            }
            run_effects(tx, events)?;
            // 下一目标或收尾。
            let targets_len = payload.len() - 5;
            if idx + 1 < targets_len {
                let next_target = payload[5 + idx + 1];
                let mut next = payload.to_vec();
                next[4] = (idx + 1) as u32;
                chulao_open_target_pick(tx, player, &mut next, next_target, idx + 1)
            } else {
                Ok(())
            }
        }
        _ => Err(EffectError::EngineFault("bad resolve-chulao stage".into())),
    }
}

pub(crate) fn resolve_resolve_zhongjie(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 || payload[0] != 0x5A4A_0002 || payload[1] != 1 {
        return Err(EffectError::EngineFault(
            "bad resolve-zhongjie continuation payload".into(),
        ));
    }
    let owner = PlayerId::new(payload[2]);
    if observer != owner {
        return Err(EffectError::Illegal(
            "resolve-zhongjie context changed".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "resolve-zhongjie requires exactly one target".into(),
        ));
    };
    let target = PlayerId::new(*selected);
    if !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "resolve-zhongjie target must be alive".into(),
        ));
    }
    // +1 体力上限 → 回复 1 → 摸 1（change_max_hp 自带上限变更事件）。
    run_effects(
        tx,
        vec![
            EffectRequest::ChangeMaxHp {
                player: target,
                delta: 1,
            },
            EffectRequest::RecoverHp {
                source: owner,
                player: target,
                amount: 1,
            },
            EffectRequest::DrawCards {
                player: target,
                count: 1,
            },
        ],
    )?;
    Ok(())
}

pub(crate) fn resolve_resolve_nhlundao(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 || payload[0] != 0x4C44_0001 || payload[1] != 1 {
        return Err(EffectError::EngineFault(
            "bad resolve-nhlundao continuation payload".into(),
        ));
    }
    let owner = PlayerId::new(payload[2]);
    let source = PlayerId::new(payload[3]);
    if observer != owner {
        return Err(EffectError::Illegal(
            "resolve-nhlundao context changed".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "resolve-nhlundao requires exactly one card".into(),
        ));
    };
    let card = rule_api::CardId::new(*selected);
    // 所选牌必须仍在来源 he 区。
    let mut allowed: Vec<CardId> = Vec::new();
    for kind in [crate::state::ZoneKind::Hand, crate::state::ZoneKind::Equip] {
        if let Some(zone) = tx.state().zones.find(kind, source) {
            allowed.extend(tx.state().zones.get(zone).unwrap().cards.iter().copied());
        }
    }
    if !allowed.contains(&card) {
        return Err(EffectError::Illegal(
            "resolve-nhlundao card is invalid or stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player: source,
            cards: vec![card],
        }],
    )?;
    Ok(())
}

pub(crate) fn resolve_resolve_olqiaozhi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    const MAGIC: u32 = 0x515A_0001;
    let player = PlayerId::new(payload[2]);
    if observer != player {
        return Err(EffectError::Illegal(
            "resolve-olqiaozhi context changed".into(),
        ));
    }
    if payload[1] == 1 {
        // stage 1：弃成本 → 亮牌堆顶 2 张 → 选 1。
        if cancelled {
            return Ok(());
        }
        let [selected] = selections else {
            return Err(EffectError::Illegal(
                "resolve-olqiaozhi requires exactly one cost card".into(),
            ));
        };
        let cost = rule_api::CardId::new(*selected);
        if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&cost) {
            return Err(EffectError::Illegal(
                "resolve-olqiaozhi cost is invalid or stale".into(),
            ));
        }
        run_effects(
            tx,
            vec![EffectRequest::DiscardCards {
                player,
                cards: vec![cost],
            }],
        )?;
        let draw = tx
            .state()
            .zones
            .find(crate::state::ZoneKind::DrawPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("draw pile missing".into()))?;
        let two: Vec<u32> = tx
            .state()
            .zones
            .get(draw)
            .unwrap()
            .cards
            .iter()
            .rev()
            .copied()
            .take(2)
            .map(|card| card.raw())
            .collect();
        if two.is_empty() {
            return Ok(());
        }
        crate::effect::open_choice(
            tx,
            player,
            1,
            1,
            false,
            two.clone(),
            engine_api::ChoiceKind::SelectCards,
            crate::state::ContinuationKind::ResolveOlqiaozhiStage,
            if two.len() == 2 {
                vec![MAGIC, 2, player.raw(), two[0], two[1]]
            } else {
                vec![MAGIC, 2, player.raw(), two[0], 0]
            },
        )
        .map(|_| ())
    } else if payload[1] == 2 {
        // stage 2：所选打「巧织」标入手牌；未选（牌堆顶第 2 张）入弃牌堆。
        if cancelled {
            return Ok(());
        }
        if payload.len() != 5 {
            return Err(EffectError::EngineFault(
                "bad resolve-olqiaozhi reveal payload".into(),
            ));
        }
        let [selected] = selections else {
            return Err(EffectError::Illegal(
                "resolve-olqiaozhi requires exactly one gained card".into(),
            ));
        };
        let gained = rule_api::CardId::new(*selected);
        // 另一张 = 亮出两张中未被选中的。
        let first = rule_api::CardId::new(payload[3]);
        let second = rule_api::CardId::new(payload[4]);
        let other = if gained == first {
            second
        } else if gained == second {
            first
        } else {
            return Err(EffectError::Illegal(
                "resolve-olqiaozhi pick is not among revealed cards".into(),
            ));
        };
        let hand = tx
            .state()
            .zones
            .find(crate::state::ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("owner hand zone missing".into()))?;
        let from = tx
            .state()
            .cards
            .get(gained)
            .map(|card| engine_api::ZoneId::new(card.zone.raw()));
        run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards: vec![gained],
                to: hand,
                from,
            }],
        )?;
        tx.state_mut().add_content_card_tag(gained, 1_909_178_934);
        // 未选牌 → 弃牌堆（上游 cardsGotoOrdering 收尾）。
        if other != gained && other != rule_api::CardId::NONE {
            let discard = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("discard pile missing".into()))?;
            let from = tx
                .state()
                .cards
                .get(other)
                .map(|card| engine_api::ZoneId::new(card.zone.raw()));
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![other],
                    to: discard,
                    from,
                }],
            )?;
        }
        Ok(())
    } else {
        Err(EffectError::EngineFault(
            "bad resolve-olqiaozhi stage".into(),
        ))
    }
}
