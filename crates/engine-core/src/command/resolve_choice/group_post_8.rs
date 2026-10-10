//! Choice-continuation handlers (post 8).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

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
        return Err(EffectError::Illegal("resolve-qianlong context changed".into()));
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
    crate::effect::resolve_jsrgzuozhan_gain_stage(
        tx,
        player,
        skill,
        event_player,
        selections,
    )?;
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
            let target = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-qiantun target required".into())
            })?;
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
            let k = *payload.get(5).ok_or_else(|| {
                EffectError::EngineFault("bad resolve-qiantun payload".into())
            })? as usize;
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
            let k = *payload.get(5).ok_or_else(|| {
                EffectError::EngineFault("bad resolve-qiantun payload".into())
            })? as usize;
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
        let card = selections.first().copied().ok_or_else(|| {
            EffectError::Illegal("resolve-liangyin discard card required".into())
        })?;
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
    let who = selections.first().copied().ok_or_else(|| {
        EffectError::Illegal("resolve-liangyin target required".into())
    })?;
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
    let target = selections.first().copied().ok_or_else(|| {
        EffectError::Illegal("resolve-zhengjian target required".into())
    })?;
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
            let target = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-shuaiyan target required".into())
            })?;
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
        _ => Err(EffectError::EngineFault("bad resolve-shuaiyan stage".into())),
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
        _ => Err(EffectError::EngineFault("bad resolve-mbbeiming stage".into())),
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
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&selected)
            {
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
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&selected)
            {
                return Err(EffectError::Illegal(
                    "resolve-gaoyuan discard cost is invalid or stale".into(),
                ));
            }
            // 与上游 content 顺序一致：先弃牌，转移提交作为尾随效果延后。
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::CommitGaoyuanRedirect {
                    player,
                    source,
                    card,
                    target,
                });
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
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&selected)
            {
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
                1 => crate::effect::resolve_duoduan_draw_and_cancel(
                    tx,
                    source,
                    player,
                    card,
                )
                .map(|_| ()),
                2 => {
                    let options =
                        crate::query::discardable_hand_equip_cards(tx.state(), source);
                    if options.is_empty() {
                        // 与上游一致：无可弃则仍走选项①语义。
                        crate::effect::resolve_duoduan_draw_and_cancel(
                            tx,
                            source,
                            player,
                            card,
                        )
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
            if !crate::query::discardable_hand_equip_cards(tx.state(), source).contains(&selected)
            {
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
            crate::command::play::make_saved_sha_target_unrespondable(
                tx,
                source,
                card,
                player,
            );
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
                run_effects(
                    tx,
                    vec![EffectRequest::DrawCards {
                        player,
                        count: 1,
                    }],
                )?;
            }
            let mut next = vec![
                0x434C_0001,
                2,
                player.raw(),
                cost.raw(),
                0u32,
            ];
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
        let other = if gained == first { second } else if gained == second { first } else {
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
        Err(EffectError::EngineFault("bad resolve-olqiaozhi stage".into()))
    }
}

pub(crate) fn resolve_resolve_olzhijie(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 || payload[0] != 0x5A4A_0001 || payload[1] != 1 {
        return Err(EffectError::EngineFault(
            "bad resolve-olzhijie continuation payload".into(),
        ));
    }
    let owner = PlayerId::new(payload[2]);
    if observer != owner {
        return Err(EffectError::Illegal(
            "resolve-olzhijie context changed".into(),
        ));
    }
    if cancelled {
        // forced 选目标不可取消；取消视为放弃（防御分支）。
        return Ok(());
    }
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "resolve-olzhijie requires exactly one target".into(),
        ));
    };
    let target = PlayerId::new(*selected);
    if !crate::query::player_in_attack_range(tx.state(), owner, target) {
        return Err(EffectError::Illegal(
            "resolve-olzhijie target is out of range".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )?;
    Ok(())
}

pub(crate) fn resolve_resolve_xinfu_qiai(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 || payload[0] != 0x5149_0001 || payload[1] != 1 {
        return Err(EffectError::EngineFault(
            "bad resolve-xinfu-qiai continuation payload".into(),
        ));
    }
    let owner = PlayerId::new(payload[2]);
    let giver = PlayerId::new(payload[3]);
    if observer != giver {
        return Err(EffectError::Illegal(
            "resolve-xinfu-qiai context changed".into(),
        ));
    }
    // 强制 1..1；取消视为跳过（防御分支，上游 chooseCard forced 不可取消）。
    if !cancelled {
        let [selected] = selections else {
            return Err(EffectError::Illegal(
                "resolve-xinfu-qiai requires exactly one card".into(),
            ));
        };
        let selected = rule_api::CardId::new(*selected);
        let mut allowed = crate::query::discardable_hand_equip_cards(tx.state(), giver);
        allowed.retain(|card| *card == selected);
        if allowed.is_empty() {
            return Err(EffectError::Illegal(
                "resolve-xinfu-qiai card is invalid or stale".into(),
            ));
        }
        // 交给 owner：移入其手牌区（上游 give → hand）。
        let hand = tx
            .state()
            .zones
            .find(crate::state::ZoneKind::Hand, owner)
            .ok_or_else(|| EffectError::Illegal("recipient hand zone missing".into()))?;
        let from = tx
            .state()
            .cards
            .get(selected)
            .map(|card| engine_api::ZoneId::new(card.zone.raw()));
        run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards: vec![selected],
                to: hand,
                from,
            }],
        )?;
    }
    // 循环下一 giver（座位序环绕，跳过 owner 与无牌者）。
    let Some(next) = crate::effect::xinfu_qiai_next_giver(tx.state(), owner, giver) else {
        return Ok(());
    };
    let options: Vec<u32> = crate::query::discardable_hand_equip_cards(tx.state(), next)
        .into_iter()
        .map(|card| card.raw())
        .collect();
    crate::effect::open_choice(
        tx,
        next,
        1,
        1,
        false,
        options,
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::ResolveXinfuQiaiStage,
        vec![0x5149_0001, 1, owner.raw(), next.raw()],
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_mbzhenann(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 || payload[0] != 0x5A4E_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-mbzhenann continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[2]);
    match payload[1] {
        1 if payload.len() == 3 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-mbzhenann context changed".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "resolve-mbzhenann requires exactly one card".into(),
                ));
            };
            let cost = rule_api::CardId::new(*selected);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&cost) {
                return Err(EffectError::Illegal(
                    "resolve-mbzhenann cost is invalid or stale".into(),
                ));
            }
            let candidates: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect();
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                candidates,
                engine_api::ChoiceKind::SelectTargets,
                crate::state::ContinuationKind::ResolveMbzhenannStage,
                vec![
                    0x5A4E_0001,
                    2,
                    player.raw(),
                    cost.raw(),
                ],
            )
            .map(|_| ())
        }
        2 if payload.len() == 4 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-mbzhenann context changed".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let [target_raw] = selections else {
                return Err(EffectError::Illegal(
                    "resolve-mbzhenann requires exactly one target".into(),
                ));
            };
            let target = PlayerId::new(*target_raw);
            let cost = rule_api::CardId::new(payload[3]);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&cost) {
                return Err(EffectError::Illegal(
                    "resolve-mbzhenann cost is invalid or stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![cost],
                }],
            )?;
            run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: player,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: 0,
                }],
            )?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault("bad resolve-mbzhenann stage".into())),
    }
}

pub(crate) fn resolve_resolve_meiyong(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 || payload[0] != 0x4D59_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-meiyong continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[2]);
    match payload[1] {
        1 if payload.len() == 3 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-meiyong context changed".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let [target_raw] = selections else {
                return Err(EffectError::Illegal(
                    "resolve-meiyong requires exactly one target".into(),
                ));
            };
            let target = PlayerId::new(*target_raw);
            let options: Vec<u32> = crate::effect::meiyong_hej_cards(tx, target)
                .into_iter()
                .map(CardId::raw)
                .collect();
            if options.is_empty() {
                return Ok(());
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::ResolveMeiyongStage,
                vec![0x4D59_0001, 2, player.raw(), target.raw()],
            )
            .map(|_| ())
        }
        2 if payload.len() == 4 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-meiyong context changed".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "resolve-meiyong requires exactly one card".into(),
                ));
            };
            let selected = rule_api::CardId::new(*selected);
            let target = PlayerId::new(payload[3]);
            if !crate::effect::meiyong_hej_cards(tx, target).contains(&selected) {
                return Err(EffectError::Illegal(
                    "resolve-meiyong card is not in the target area".into(),
                ));
            }
            // 移牌：目标 hej 区 → owner 手牌。
            let owner_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no owner hand".into()))?;
            let from_zone = tx.state().cards.get(selected).unwrap().zone;
            tx.state_mut()
                .zones
                .get_mut(from_zone)
                .unwrap()
                .cards
                .retain(|card| *card != selected);
            tx.state_mut()
                .zones
                .get_mut(owner_hand)
                .unwrap()
                .cards
                .push(selected);
            if let Some(card_state) = tx.state_mut().cards.get_mut(selected) {
                card_state.zone = owner_hand;
                card_state.owner = player;
            }
            run_effects(
                tx,
                vec![EffectRequest::DrawCards {
                    player: target,
                    count: 1,
                }],
            )?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault("bad resolve-meiyong stage".into())),
    }
}

pub(crate) fn resolve_resolve_dingce(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 || payload[0] != 0x4443_0002 {
        return Err(EffectError::EngineFault(
            "bad resolve-dingce continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[2]);
    let source = PlayerId::new(payload[3]);
    match payload[1] {
        1 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-dingce context changed".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "resolve-dingce requires exactly one card".into(),
                ));
            };
            let cost = rule_api::CardId::new(*selected);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&cost) {
                return Err(EffectError::Illegal(
                    "resolve-dingce cost is invalid or stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![cost],
                }],
            )?;
            // 来源无手牌可弃 → 结束（上游 hasDiscardableCards 早退）。
            let source_options: Vec<u32> =
                crate::query::discardable_hand_equip_cards(tx.state(), source)
                    .into_iter()
                    .map(CardId::raw)
                    .collect();
            if source_options.is_empty() {
                return Ok(());
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                source_options,
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::ResolveDingceStage,
                vec![0x4443_0002, 2, player.raw(), source.raw()],
            )
            .map(|_| ())
        }
        2 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-dingce context changed".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "resolve-dingce requires exactly one card".into(),
                ));
            };
            let source_card = rule_api::CardId::new(*selected);
            if !crate::query::discardable_hand_equip_cards(tx.state(), source)
                .contains(&source_card)
            {
                return Err(EffectError::Illegal(
                    "resolve-dingce source card is invalid or stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: source,
                    cards: vec![source_card],
                }],
            )?;
            // 同色分支（虚拟【洞烛先机】）登记债：卡牌不在牌池，暂不结算。
            Ok(())
        }
        _ => Err(EffectError::EngineFault("bad resolve-dingce stage".into())),
    }
}

pub(crate) fn resolve_resolve_dcqiaolve(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 || payload[0] != 0x514C_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-dcqiaolve continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    match payload[1] {
        1 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-dcqiaolve context changed".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "resolve-dcqiaolve requires exactly one card".into(),
                ));
            };
            let cost = rule_api::CardId::new(*selected);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&cost) {
                return Err(EffectError::Illegal(
                    "resolve-dcqiaolve cost is invalid or stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![cost],
                }],
            )?;
            run_effects(
                tx,
                vec![EffectRequest::TakeOneHandCard {
                    from: target,
                    to: player,
                }],
            )?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault("bad resolve-dcqiaolve stage".into())),
    }
}

pub(crate) fn resolve_resolve_qiangyong(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    _cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 || payload[0] != 0x5159_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-qiangyong continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    match payload[1] {
        1 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-qiangyong context changed".into(),
                ));
            }
            if selections.is_empty() {
                return Err(EffectError::Illegal(
                    "resolve-qiangyong requires at least one card".into(),
                ));
            }
            let cards: Vec<rule_api::CardId> = selections
                .iter()
                .map(|raw| rule_api::CardId::new(*raw))
                .collect();
            let target_options = crate::query::discardable_hand_equip_cards(tx.state(), target);
            for card in &cards {
                if !target_options.contains(card) {
                    return Err(EffectError::Illegal(
                        "resolve-qiangyong card is not in the target zones".into(),
                    ));
                }
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: cards.clone(),
                }],
            )?;
            crate::effect::qiangyong_bonus_if_no_hand(tx, player, target, skill)?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault("bad resolve-qiangyong stage".into())),
    }
}

fn chulao_open_target_pick(
    tx: &mut Transaction,
    player: PlayerId,
    payload: &mut [u32],
    target_raw: u32,
    idx: usize,
) -> Result<(), EffectError> {
    let target = PlayerId::new(target_raw);
    let options: Vec<u32> = crate::query::discardable_hand_equip_cards(tx.state(), target)
        .into_iter()
        .map(CardId::raw)
        .collect();
    if options.is_empty() {
        // 目标无牌可弃：跳过该目标（上游 choosePlayerCard 无可选项时中断，
        // 语义近似为跳过）。
        let targets_len = payload.len() - 5;
        if idx + 1 < targets_len {
            let next_target = payload[5 + idx + 1];
            payload[4] = (idx + 1) as u32;
            return chulao_open_target_pick(tx, player, payload, next_target, idx + 1);
        }
        return Ok(());
    }
    crate::effect::open_choice(
        tx,
        player,
        1,
        1,
        false,
        options,
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::ResolveChulaoStage,
        payload.to_vec(),
    )
    .map(|_| ())
}

/// 狐魅分段分派：0=目标已选 → 2=三选一 → 4=给牌子选择。
pub(crate) fn resolve_resolve_chumei(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload[0] != 0x4348_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-chumei continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let stage = payload[3];
    match stage {
        0 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-chumei context changed".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let target = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-chumei target required".into())
            })?;
            crate::effect::resolve_chumei_stage(tx, player, skill, target)?;
            Ok(())
        }
        2 => {
            let target_raw = payload[4];
            let target = PlayerId::new(target_raw);
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-chumei option choice requires owner".into(),
                ));
            }
            let choice = *selections.first().ok_or_else(|| {
                EffectError::Illegal("resolve-chumei option required".into())
            })?;
            match choice {
                1 => {
                    crate::effect::draw_cards(tx, target, 1)?;
                    Ok(())
                }
                2 => {
                    crate::effect::resolve_chumei_give_pick(
                        tx,
                        player,
                        skill,
                        target_raw,
                    )?;
                    Ok(())
                }
                3 => {
                    crate::effect::recover_hp(tx, player, target, 1)?;
                    Ok(())
                }
                _ => Err(EffectError::Illegal(
                    "resolve-chumei unknown option".into(),
                )),
            }
        }
        4 => {
            let target = PlayerId::new(payload[4]);
            if observer != target {
                return Err(EffectError::Illegal(
                    "resolve-chumei give requires the target".into(),
                ));
            }
            let card = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-chumei give card required".into())
            })?;
            crate::effect::resolve_chumei_give(tx, player, skill, payload[4], card)?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault("bad resolve-chumei stage".into())),
    }
}

/// 忠言分段分派：0=目标已选 → 3=亮牌已选 → 4=手牌已选 → 交换。
pub(crate) fn resolve_resolve_zhongyan(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload[0] != 0x5A5A_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-zhongyan continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    match payload[3] {
        0 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-zhongyan context changed".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let target = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-zhongyan target required".into())
            })?;
            crate::effect::resolve_zhongyan_stage(tx, player, skill, target)?;
            Ok(())
        }
        3 => {
            let target_raw = payload[4];
            let target = PlayerId::new(target_raw);
            if observer != target {
                return Err(EffectError::Illegal(
                    "resolve-zhongyan reveal pick requires the target".into(),
                ));
            }
            let k = *payload.get(5).ok_or_else(|| {
                EffectError::EngineFault("bad resolve-zhongyan payload".into())
            })? as usize;
            if payload.len() < 6 + k {
                return Err(EffectError::EngineFault(
                    "bad resolve-zhongyan payload layout".into(),
                ));
            }
            let revealed: Vec<u32> = payload[6..6 + k].to_vec();
            let pick = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-zhongyan reveal pick required".into())
            })?;
            if !revealed.contains(&pick) {
                return Err(EffectError::Illegal(
                    "resolve-zhongyan reveal pick outside revealed".into(),
                ));
            }
            crate::effect::resolve_zhongyan_pick(
                tx,
                player,
                skill,
                target_raw,
                &revealed,
                pick,
            )?;
            Ok(())
        }
        4 => {
            let target_raw = payload[4];
            let target = PlayerId::new(target_raw);
            if observer != target {
                return Err(EffectError::Illegal(
                    "resolve-zhongyan hand pick requires the target".into(),
                ));
            }
            let k = *payload.get(5).ok_or_else(|| {
                EffectError::EngineFault("bad resolve-zhongyan payload".into())
            })? as usize;
            if payload.len() < 7 + k {
                return Err(EffectError::EngineFault(
                    "bad resolve-zhongyan payload layout".into(),
                ));
            }
            let revealed: Vec<u32> = payload[6..6 + k].to_vec();
            let revealed_pick = payload[6 + k];
            let hand_card = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-zhongyan hand card required".into())
            })?;
            let target_hand: std::collections::BTreeSet<u32> = tx
                .state()
                .hand_cards(target)
                .iter()
                .map(|card| card.raw())
                .collect();
            if !target_hand.contains(&hand_card) {
                return Err(EffectError::Illegal(
                    "resolve-zhongyan hand card outside target hand".into(),
                ));
            }
            crate::effect::resolve_zhongyan_swap(
                tx,
                player,
                skill,
                target_raw,
                &revealed,
                revealed_pick,
                hand_card,
            )?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault("bad resolve-zhongyan stage".into())),
    }
}

/// 谋溃分段分派：2=知情选择已定 → 3=弃置目标手牌牌已定。
pub(crate) fn resolve_resolve_dcmoukui(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 || payload[0] != 0x4D4B_0001 {
        return Err(EffectError::EngineFault(
            "bad resolve-dcmoukui continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let target = PlayerId::new(payload[4]);
    if cancelled {
        return Ok(());
    }
    match payload[3] {
        2 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-dcmoukui context changed".into(),
                ));
            }
            crate::effect::resolve_dcmoukui_stage(tx, player, skill, target, selections)?;
            Ok(())
        }
        3 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-dcmoukui context changed".into(),
                ));
            }
            let card = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-dcmoukui discard card required".into())
            })?;
            crate::effect::resolve_dcmoukui_discard(tx, player, skill, target, card)?;
            Ok(())
        }
        6 => {
            // 追罚弃置：闪避者选定的 owner 手牌。
            let dodger = PlayerId::new(payload[4]);
            if observer != dodger {
                return Err(EffectError::Illegal(
                    "resolve-dcmoukui conseq pick requires the dodger".into(),
                ));
            }
            let card = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-dcmoukui conseq card required".into())
            })?;
            crate::effect::resolve_dcmoukui_conseq_discard(
                tx,
                player,
                skill,
                dodger,
                card,
            )?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault("bad resolve-dcmoukui stage".into())),
    }
}

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
    let target = selections.first().copied().ok_or_else(|| {
        EffectError::Illegal("resolve-biaozhao3 target required".into())
    })?;
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
            let card = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-canyun discard required".into())
            })?;
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
            crate::effect::resolve_canyun_resolve(
                tx,
                player,
                skill,
                discard_card,
                selections,
            )?;
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
    let target = selections.first().copied().ok_or_else(|| {
        EffectError::Illegal("resolve-yangming target required".into())
    })?;
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
        return Err(EffectError::Illegal("resolve-jsrgdaoren context changed".into()));
    }
    match stage {
        1 => {
            let card = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-jsrgdaoren card required".into())
            })?;
            crate::effect::jsrgdaoren_entry(tx, player, skill, 1, card, 0)?;
            Ok(())
        }
        2 => {
            let target = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-jsrgdaoren target required".into())
            })?;
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
        return Err(EffectError::Illegal("resolve-pxqixian context changed".into()));
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
            let material = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-pxqixian material required".into())
            })?;
            crate::effect::pxqixian_step(tx, player, skill, 3, kind, material, 0)?;
            Ok(())
        }
        // 阶段 3 目标：selections[0] = 目标。
        3 => {
            let target = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-pxqixian target required".into())
            })?;
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
        return Err(EffectError::Illegal("resolve-dcbingji context changed".into()));
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
            let target = selections.first().copied().ok_or_else(|| {
                EffectError::Illegal("resolve-dcbingji target required".into())
            })?;
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
    if selections.len() < min_count as usize
        || selections.len() > max_count as usize
    {
        return Err(EffectError::Illegal(
            "choose-hand-cards-to-skill-pile selection count out of range".into(),
        ));
    }
    let skill_def = crate::effect::skill_pile_def(tx, player, skill)?;
    let legal: std::collections::BTreeSet<_> =
        tx.state().hand_cards(player).into_iter().collect();
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
    if selections.len() != (count as usize).min(
        tx.state().skill_pile_cards(player, pile_def).len(),
    ) {
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
            return Err(EffectError::Illegal("selected card not in skill pile".into()));
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
            &next_top.iter().copied().map(CardId::new).collect::<Vec<_>>(),
            &next_bottom.iter().copied().map(CardId::new).collect::<Vec<_>>(),
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

pub(crate) fn resolve_choose_other_give_one_hand_equip(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad choose-other gift continuation".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let skill = engine_api::SkillId::new(*skill);
    if cancelled || observer != player || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one other gift target".into(),
        ));
    }
    let target = PlayerId::new(selections[0]);
    let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    let player_alive = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive);
    let target_alive = tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive);
    let has_card = [ZoneKind::Hand, ZoneKind::Equip].into_iter().any(|kind| {
        tx.state()
            .zones
            .find(kind, player)
            .and_then(|zone| tx.state().zones.get(zone))
            .is_some_and(|zone| !zone.cards.is_empty())
    });
    if !skill_enabled || !player_alive || !target_alive || player == target || !has_card {
        return Err(EffectError::Illegal(
            "choose-other gift context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::DrawThenGiveOne {
            player,
            target,
            count: 0,
            drawn_only: false,
            use_given_equipment: false,
            skill,
            drawn: Vec::new(),
            ready_to_give: true,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_discard_target_card_gain_mount(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let scope = payload
        .get(2)
        .and_then(|tag| u8::try_from(*tag).ok())
        .and_then(rule_api::TargetCardScope::from_tag)
        .ok_or_else(|| EffectError::Illegal("invalid target-card scope".into()))?;
    resolve_scoped_discard_target_card_gain_mount(
        tx, observer, payload, selections, cancelled, scope, true,
    )
}

pub(crate) fn resolve_choose_discard_target_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let scope = payload
        .get(2)
        .and_then(|tag| u8::try_from(*tag).ok())
        .and_then(rule_api::TargetCardScope::from_tag)
        .ok_or_else(|| EffectError::Illegal("invalid target-card scope".into()))?;
    resolve_scoped_discard_target_card_gain_mount(
        tx, observer, payload, selections, cancelled, scope, false,
    )
}

pub(crate) fn resolve_choose_target_reclaim_equipment_draw_if_range_reduced(
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
        .ok_or_else(|| EffectError::EngineFault("truncated reclamation options".into()))?
        as usize;
    if payload.len() != 4 + option_count {
        return Err(EffectError::EngineFault(
            "invalid reclamation option count".into(),
        ));
    }
    if cancelled || observer != player || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one target equipment card".into(),
        ));
    }
    let card = CardId::new(selections[0]);
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, target)
        .ok_or_else(|| EffectError::Illegal("missing target equipment zone".into()))?;
    if !payload[4..].contains(&card.raw())
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .zones
            .get(equip)
            .is_some_and(|zone| zone.cards.contains(&card))
    {
        return Err(EffectError::Illegal(
            "selected target equipment is stale".into(),
        ));
    }
    let before = crate::effect::attack_range_population(tx, target);
    run_effects(
        tx,
        vec![
            EffectRequest::CommitTargetReclaimEquipmentDrawIfRangeReduced {
                player,
                target,
                card,
                before,
                skill,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_twchenxun_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    let option_count = *payload
        .get(3)
        .ok_or_else(|| EffectError::EngineFault("truncated TW Chenxun options".into()))?
        as usize;
    if payload.len() != 4 + option_count || observer != player {
        return Err(EffectError::Illegal(
            "TW Chenxun target context changed".into(),
        ));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled TW Chenxun has selections".into(),
            ))
        };
    }
    if selections.len() != 1 || !payload[4..].contains(&selections[0]) {
        return Err(EffectError::Illegal(
            "TW Chenxun requires one offered target".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveTwchenxun {
            player,
            target: PlayerId::new(selections[0]),
            card: CardId::NONE,
            action: 3,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_twfenxian_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "TW Fenxian requires one offered choice".into(),
        ));
    }
    let stage = payload[1];
    let owner = PlayerId::new(payload[2]);
    let actor = PlayerId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    if observer != actor || !matches!((stage, payload.len()), (0 | 3, 6) | (2, 5)) {
        return Err(EffectError::Illegal(
            "TW Fenxian choice context changed".into(),
        ));
    }
    let request = match stage {
        0 if selections[0] <= 1 && payload[5] & (1 << selections[0]) != 0 => {
            EffectRequest::ResolveTwfenxian {
                player: owner,
                target: actor,
                card: CardId::NONE,
                action: if selections[0] == 0 { 2 } else { 1 },
                skill,
            }
        }
        2 => EffectRequest::ResolveTwfenxian {
            player: owner,
            target: actor,
            card: CardId::new(selections[0]),
            action: 3,
            skill,
        },
        3 => EffectRequest::ResolveTwfenxian {
            player: actor,
            target: PlayerId::new(selections[0]),
            card: CardId::new(payload[5]),
            action: 4,
            skill,
        },
        _ => {
            return Err(EffectError::Illegal(
                "TW Fenxian choice is not offered".into(),
            ));
        }
    };
    run_effects(tx, vec![request]).map(|_| ())
}

pub(crate) fn resolve_choose_use_virtual_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let card = CardId::new(*payload.get(1).unwrap_or(&0));
    let kind = payload
        .get(2)
        .and_then(|value| u8::try_from(*value).ok())
        .and_then(CardKind::from_tag)
        .filter(|kind| matches!(kind, CardKind::TrickWanjian | CardKind::TrickJuedou))
        .ok_or_else(|| EffectError::EngineFault("invalid virtual-card choice kind".into()))?;
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let option_count = *payload
        .get(4)
        .ok_or_else(|| EffectError::EngineFault("truncated virtual-card options".into()))?
        as usize;
    let option_end = 5 + option_count;
    let dcctjiuxian = payload.get(option_end) == Some(&1);
    if payload.len() != option_end + usize::from(dcctjiuxian) {
        return Err(EffectError::EngineFault(
            "invalid virtual-card option count".into(),
        ));
    }
    if cancelled || observer != player {
        return Err(EffectError::Illegal(
            "mandatory virtual-card target choice cannot be cancelled".into(),
        ));
    }
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) || tx
        .state()
        .cards
        .get(card)
        .is_none_or(|state| state.kind != kind)
    {
        return Err(EffectError::Illegal(
            "virtual-card use context is stale".into(),
        ));
    }
    let crate::command::play::SkillCardTargetSpec::Select { min, max, options } =
        crate::command::play::skill_card_target_spec(tx, player, card)?
    else {
        return Err(EffectError::Illegal(
            "virtual-card target contract changed".into(),
        ));
    };
    let mut distinct = std::collections::BTreeSet::new();
    let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
    if selected.len() < usize::from(min)
        || selected.len() > usize::from(max)
        || selected
            .iter()
            .any(|target| !distinct.insert(*target) || !options.contains(target))
        || selections
            .iter()
            .any(|target| !payload[5..option_end].contains(target))
    {
        return Err(EffectError::Illegal(
            "virtual-card target selection is invalid or stale".into(),
        ));
    }
    if dcctjiuxian {
        if kind != CardKind::TrickJuedou
            || selected.len() != 1
            || !tx
                .state_mut()
                .skills
                .get_mut(skill)
                .is_some_and(|instance| instance.record_active_play_phase_target(selected[0]))
        {
            return Err(EffectError::Illegal(
                "Dcctjiuxian Duel target context changed".into(),
            ));
        }
    }
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_CARD, card.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
    tx.state_mut().turn.set_flag(
        crate::state::turn_flag::VIEW_AS_LOGICAL_KIND,
        i32::from(kind.tag()),
    );
    let result = crate::command::play::handle_skill_card(tx, player, card, &selected);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_CARD, 0);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, 0);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    result
}

pub(crate) fn resolve_choose_discard_target_card_current_sha(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if !matches!(payload, [_, _, _] | [_, _, _, _]) {
        return Err(EffectError::EngineFault(
            "bad current Sha target-card payload".into(),
        ));
    }
    if cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one target card for current Sha".into(),
        ));
    }
    let source = PlayerId::new(*payload.first().unwrap_or(&0));
    let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let card = CardId::new(*payload.get(2).unwrap_or(&0));
    let nonbasic_bonus_skill = payload.get(3).copied().map(engine_api::SkillDefId::new);
    if observer != source {
        return Err(EffectError::Illegal(
            "only the current Sha user may resolve the target-card choice".into(),
        ));
    }
    crate::command::play::validate_saved_sha_target(tx, source, card, target)?;
    let selected = CardId::new(selections[0]);
    if !crate::query::discardable_target_cards(
        tx.state(),
        source,
        target,
        rule_api::TargetCardScope::HandEquip,
    )
    .contains(&selected)
    {
        return Err(EffectError::Illegal(
            "current Sha target-card selection is no longer legal".into(),
        ));
    }
    let selected_kind = tx
        .state()
        .cards
        .get(selected)
        .map(|selected| crate::query::effective_card_kind(tx.state(), source, selected))
        .ok_or_else(|| EffectError::Illegal("selected target card no longer exists".into()))?;
    let blocks_response = nonbasic_bonus_skill
        .map_or_else(|| selected_kind.is_equip(), |_| !selected_kind.is_basic());
    let mut effects = vec![EffectRequest::DiscardTargetCards {
        actor: source,
        target,
        cards: vec![selected],
        scope: rule_api::TargetCardScope::HandEquip,
    }];
    if blocks_response {
        effects.push(EffectRequest::MakeCurrentShaTargetUnrespondable {
            source,
            card,
            target,
        });
        if let Some(skill_def) = nonbasic_bonus_skill {
            effects.push(EffectRequest::AddSkill {
                player: source,
                skill_def,
            });
            effects.push(EffectRequest::AddMark {
                player: source,
                mark: engine_api::MarkDefId::new(skill_def.raw()),
                delta: 1,
            });
        }
    } else {
        effects.push(EffectRequest::GainEventCards {
            player: target,
            cards: vec![card],
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_dcqiaomeng(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 7 {
        return Err(EffectError::EngineFault(
            "bad Dcqiaomeng choice payload".into(),
        ));
    }
    let target_count = payload[6] as usize;
    let Some(targets_raw) = payload.get(7..7 + target_count) else {
        return Err(EffectError::EngineFault(
            "bad Dcqiaomeng target snapshot".into(),
        ));
    };
    if payload.len() != 7 + target_count {
        return Err(EffectError::EngineFault(
            "bad Dcqiaomeng choice payload length".into(),
        ));
    }
    let [selection] = selections else {
        return Err(EffectError::Illegal(
            "Dcqiaomeng requires exactly one choice".into(),
        ));
    };
    let player = PlayerId::new(payload[1]);
    if cancelled || observer != player {
        return Err(EffectError::Illegal(
            "only the Dcqiaomeng owner may resolve its choice".into(),
        ));
    }
    let card = CardId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let helper_skill_def = engine_api::SkillDefId::new(payload[4]);
    let skill = engine_api::SkillId::new(payload[5]);
    let targets = targets_raw.iter().copied().map(PlayerId::new).collect();
    let effect = match payload[0] {
        0 => EffectRequest::ResolveDcqiaomeng {
            player,
            card,
            targets,
            target: PlayerId::new(*selection),
            selected: CardId::NONE,
            helper_skill_def,
            stage: 1,
            skill,
        },
        1 => EffectRequest::ResolveDcqiaomeng {
            player,
            card,
            targets,
            target,
            selected: CardId::new(*selection),
            helper_skill_def,
            stage: 2,
            skill,
        },
        _ => {
            return Err(EffectError::Illegal(
                "invalid Dcqiaomeng choice stage".into(),
            ));
        }
    };
    run_effects(tx, vec![effect]).map(|_| ())
}

pub(crate) fn resolve_retieji(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [source, card, target, suit] = payload else {
        return Err(EffectError::EngineFault(
            "bad Retieji choice payload".into(),
        ));
    };
    let source = PlayerId::new(*source);
    let card = CardId::new(*card);
    let target = PlayerId::new(*target);
    let suit = crate::command::judge::suit_from_tag(*suit);
    if observer != target
        || ![source, target].into_iter().all(|player| {
            tx.state()
                .players
                .get(player)
                .is_some_and(|state| state.alive)
        })
    {
        return Err(EffectError::Illegal(
            "Retieji choice context changed".into(),
        ));
    }
    crate::command::play::validate_saved_sha_target(tx, source, card, target)?;
    if cancelled || selections.is_empty() {
        if !selections.is_empty() {
            return Err(EffectError::Illegal(
                "cancelled Retieji choice contains a card".into(),
            ));
        }
        return run_effects(
            tx,
            vec![
                EffectRequest::MakeCurrentShaTargetUnrespondable {
                    source,
                    card,
                    target,
                },
                EffectRequest::IncreaseCurrentShaTargetDamage {
                    source,
                    card,
                    target,
                    amount: 1,
                },
            ],
        )
        .map(|_| ());
    }
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "Retieji accepts at most one card".into(),
        ));
    };
    let selected = CardId::new(*selected);
    if !crate::query::discardable_hand_equip_cards(tx.state(), target).contains(&selected)
        || !tx
            .state()
            .cards
            .get(selected)
            .is_some_and(|card| crate::query::effective_card_suit(tx.state(), target, card) == suit)
    {
        return Err(EffectError::Illegal(
            "Retieji card is no longer eligible".into(),
        ));
    }
    let from = response_card_zone(tx, target, selected)?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, source)
        .ok_or_else(|| EffectError::EngineFault("Retieji recipient hand is missing".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::GiveCards {
            cards: vec![selected],
            to,
            from,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_new_reluoyi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, buff_skill_def, skill, card_count, cards @ ..] = payload else {
        return Err(EffectError::EngineFault(
            "bad New Reluoyi choice payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let card_count = usize::try_from(*card_count)
        .map_err(|_| EffectError::EngineFault("bad New Reluoyi card count".into()))?;
    if observer != player
        || cancelled
        || selections.len() != 1
        || cards.len() != card_count
        || !matches!(selections[0], 0 | 1)
    {
        return Err(EffectError::Illegal(
            "New Reluoyi choice context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveNewReluoyi {
            player,
            buff_skill_def: SkillDefId::new(*buff_skill_def),
            stage: 2,
            cards: cards.iter().copied().map(CardId::new).collect(),
            accepted: selections[0] == 1,
            skill: SkillId::new(*skill),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_xinyaoming(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing Xinyaoming stage".into()))?;
    match stage {
        0 => {
            let option_count = *payload.get(3).ok_or_else(|| {
                EffectError::EngineFault("truncated Xinyaoming target payload".into())
            })? as usize;
            if payload.len() != 4 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Xinyaoming target payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let skill = SkillId::new(payload[2]);
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Xinyaoming requires exactly one target".into(),
                ));
            };
            let target = PlayerId::new(*selected);
            let used_mask = tx
                .state()
                .skills
                .get(skill)
                .filter(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                })
                .map_or(u8::MAX, |instance| instance.xinyaoming_used_mask());
            if cancelled
                || observer != player
                || !payload[4..].contains(selected)
                || crate::query::xinyaoming_available_actions(tx.state(), player, target, used_mask)
                    .is_empty()
            {
                return Err(EffectError::Illegal(
                    "Xinyaoming target choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveXinyaoming {
                    player,
                    target,
                    action: 0,
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            let option_count = *payload.get(4).ok_or_else(|| {
                EffectError::EngineFault("truncated Xinyaoming action payload".into())
            })? as usize;
            if payload.len() != 5 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Xinyaoming action payload".into(),
                ));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Xinyaoming requires exactly one action".into(),
                ));
            };
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = SkillId::new(payload[3]);
            let used_mask = tx
                .state()
                .skills
                .get(skill)
                .filter(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                })
                .map_or(u8::MAX, |instance| instance.xinyaoming_used_mask());
            let action = u8::try_from(*selected)
                .map_err(|_| EffectError::Illegal("bad Xinyaoming action token".into()))?;
            if cancelled
                || observer != player
                || !payload[5..].contains(selected)
                || !crate::query::xinyaoming_available_actions(
                    tx.state(),
                    player,
                    target,
                    used_mask,
                )
                .contains(&action)
            {
                return Err(EffectError::Illegal(
                    "Xinyaoming action choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveXinyaoming {
                    player,
                    target,
                    action,
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 => {
            let option_count = *payload.get(4).ok_or_else(|| {
                EffectError::EngineFault("truncated Xinyaoming discard payload".into())
            })? as usize;
            if payload.len() != 5 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Xinyaoming discard payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = SkillId::new(payload[3]);
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            let current = crate::query::discardable_target_cards(
                tx.state(),
                target,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player && tx.state().skill_effectively_enabled(instance)
            });
            if cancelled
                || observer != target
                || !skill_enabled
                || cards.is_empty()
                || cards.len() > 2
                || cards.iter().enumerate().any(|(index, card)| {
                    cards[..index].contains(card)
                        || !payload[5..].contains(&card.raw())
                        || !current.contains(card)
                })
            {
                return Err(EffectError::Illegal(
                    "Xinyaoming discard choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player: target,
                        cards: cards.clone(),
                    },
                    EffectRequest::DrawCardsFromSkill {
                        player: target,
                        count: cards.len() as u32,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Xinyaoming choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_rehuaiyi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing Rehuaiyi stage".into()))?;
    match stage {
        0 => {
            let option_count = *payload.get(3).ok_or_else(|| {
                EffectError::EngineFault("truncated Rehuaiyi color payload".into())
            })? as usize;
            if payload.len() != 4 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Rehuaiyi color payload".into(),
                ));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Rehuaiyi requires exactly one color".into(),
                ));
            };
            let player = PlayerId::new(payload[1]);
            let skill = SkillId::new(payload[2]);
            let color = u8::try_from(*selected)
                .map_err(|_| EffectError::Illegal("bad Rehuaiyi color token".into()))?;
            if cancelled
                || observer != player
                || !matches!(color, 0b01 | 0b10)
                || !payload[4..].contains(selected)
            {
                return Err(EffectError::Illegal(
                    "Rehuaiyi color choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveRehuaiyi {
                    player,
                    stage: 0,
                    color,
                    discarded_count: 0,
                    targets: Vec::new(),
                    index: 0,
                    gained_count: 0,
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            let discarded_count = *payload.get(3).ok_or_else(|| {
                EffectError::EngineFault("truncated Rehuaiyi target payload".into())
            })?;
            let option_count = *payload.get(4).ok_or_else(|| {
                EffectError::EngineFault("truncated Rehuaiyi target options".into())
            })? as usize;
            if payload.len() != 5 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Rehuaiyi target payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let skill = SkillId::new(payload[2]);
            if observer != player || (cancelled && !selections.is_empty()) {
                return Err(EffectError::Illegal(
                    "invalid Rehuaiyi target cancellation".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let max = (discarded_count as usize).min(option_count);
            let mut unique = std::collections::BTreeSet::new();
            if selections.is_empty()
                || selections.len() > max
                || !selections.iter().all(|selected| {
                    unique.insert(*selected)
                        && payload[5..].contains(selected)
                        && PlayerId::new(*selected) != player
                        && !crate::query::gainable_target_cards(
                            tx.state(),
                            player,
                            PlayerId::new(*selected),
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .is_empty()
                })
            {
                return Err(EffectError::Illegal(
                    "Rehuaiyi target choice context changed".into(),
                ));
            }
            let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| selected.contains(&target.raw()))
                .collect();
            run_effects(
                tx,
                vec![EffectRequest::ResolveRehuaiyi {
                    player,
                    stage: 2,
                    color: 0,
                    discarded_count: 0,
                    targets,
                    index: 0,
                    gained_count: 0,
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 => {
            let target_count = *payload
                .get(5)
                .ok_or_else(|| EffectError::EngineFault("truncated Rehuaiyi gain payload".into()))?
                as usize;
            let options_index = 6usize
                .checked_add(target_count)
                .ok_or_else(|| EffectError::EngineFault("Rehuaiyi payload overflow".into()))?;
            let option_count = *payload
                .get(options_index)
                .ok_or_else(|| EffectError::EngineFault("truncated Rehuaiyi card options".into()))?
                as usize;
            if payload.len() != options_index + 1 + option_count {
                return Err(EffectError::EngineFault("bad Rehuaiyi gain payload".into()));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Rehuaiyi requires exactly one target card".into(),
                ));
            };
            let player = PlayerId::new(payload[1]);
            let skill = SkillId::new(payload[2]);
            let index = payload[3] as usize;
            let gained_count = payload[4];
            let targets: Vec<_> = payload[6..options_index]
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            let target = targets.get(index).copied().ok_or_else(|| {
                EffectError::EngineFault("Rehuaiyi target index out of bounds".into())
            })?;
            let card = CardId::new(*selected);
            let current = crate::query::gainable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            let skill_valid = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player && tx.state().skill_effectively_enabled(instance)
            });
            if cancelled
                || observer != player
                || !skill_valid
                || !payload[options_index + 1..].contains(selected)
                || !current.contains(&card)
            {
                return Err(EffectError::Illegal(
                    "Rehuaiyi target-card choice context changed".into(),
                ));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("Rehuaiyi card disappeared".into()))?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("Rehuaiyi owner hand missing".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: vec![card],
                        to,
                        from: Some(from),
                    },
                    EffectRequest::ResolveRehuaiyi {
                        player,
                        stage: 2,
                        color: 0,
                        discarded_count: 0,
                        targets,
                        index: (index as u32).saturating_add(1),
                        gained_count: gained_count.saturating_add(1),
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Rehuaiyi choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_reanguo(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw, option_count, options @ ..] = payload else {
        return Err(EffectError::EngineFault(
            "truncated Reanguo recast payload".into(),
        ));
    };
    if options.len() != *option_count as usize {
        return Err(EffectError::EngineFault(
            "bad Reanguo recast payload".into(),
        ));
    }
    let player = PlayerId::new(*player_raw);
    let skill = SkillId::new(*skill_raw);
    if observer != player || (cancelled && !selections.is_empty()) {
        return Err(EffectError::Illegal(
            "invalid Reanguo recast cancellation".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let equip = tx.state().zones.find(ZoneKind::Equip, player);
    let max = options.len().min(u8::MAX as usize);
    let mut unique = std::collections::BTreeSet::new();
    let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
    let skill_valid = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    if !skill_valid
        || cards.is_empty()
        || cards.len() > max
        || cards.iter().any(|card| {
            !unique.insert(card.raw())
                || !options.contains(&card.raw())
                || tx.state().cards.get(*card).is_none_or(|state| {
                    state.owner != player || (Some(state.zone) != hand && Some(state.zone) != equip)
                })
        })
    {
        return Err(EffectError::Illegal(
            "Reanguo recast choice context changed".into(),
        ));
    }
    run_effects(tx, vec![EffectRequest::RecastCards { player, cards }]).map(|_| ())
}

pub(crate) fn resolve_scoped_discard_target_card_gain_mount(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
    scope: rule_api::TargetCardScope,
    gain_mount: bool,
) -> Result<(), EffectError> {
    if cancelled {
        if !gain_mount && payload.get(3).copied().unwrap_or(0) == 1 {
            return Ok(());
        }
        return Err(EffectError::Illegal(
            "choose exactly one target card".into(),
        ));
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one target card".into(),
        ));
    }
    let chooser = PlayerId::new(*payload.first().unwrap_or(&0));
    let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
    if observer != chooser {
        return Err(EffectError::Illegal(
            "only the target-card discard chooser may resolve this choice".into(),
        ));
    }
    if ![chooser, target].into_iter().all(|player| {
        tx.state()
            .players
            .get(player)
            .is_some_and(|state| state.alive)
    }) {
        return Err(EffectError::Illegal(
            "target-card discard participants are no longer alive".into(),
        ));
    }
    let selected = CardId::new(selections[0]);
    if !crate::query::discardable_target_cards(tx.state(), chooser, target, scope)
        .contains(&selected)
    {
        return Err(EffectError::Illegal(
            "target card selection is no longer legal".into(),
        ));
    }
    let is_mount = tx
        .state()
        .cards
        .get(selected)
        .is_some_and(|card| matches!(card.kind, CardKind::HorseOff | CardKind::HorseDef));
    let mut effects = vec![EffectRequest::DiscardTargetCards {
        actor: chooser,
        target,
        cards: vec![selected],
        scope,
    }];
    if gain_mount && is_mount {
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("missing discard pile".into()))?;
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, chooser)
            .ok_or_else(|| EffectError::EngineFault("missing chooser hand".into()))?;
        effects.push(EffectRequest::MoveCards {
            cards: vec![selected],
            to: hand,
            from: Some(discard),
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_beizhu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) => {
            if payload.len() < 5 {
                return Err(EffectError::EngineFault(
                    "bad Beizhu discard payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            let option_count = payload[4] as usize;
            if cancelled
                || observer != player
                || selections.len() != 1
                || payload.len() != 5 + option_count
            {
                return Err(EffectError::Illegal(
                    "Beizhu requires one target card".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            if !payload[5..].contains(&card.raw())
                || !tx
                    .state()
                    .skills
                    .get(skill)
                    .is_some_and(|instance| instance.owner == player)
                || !crate::query::discardable_target_cards(
                    tx.state(),
                    player,
                    target,
                    rule_api::TargetCardScope::HandEquip,
                )
                .contains(&card)
            {
                return Err(EffectError::Illegal(
                    "Beizhu target-card choice context changed".into(),
                ));
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveBeizhu {
                    player,
                    target,
                    stage: 1,
                    cards: Vec::new(),
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::DiscardTargetCards {
                    actor: player,
                    target,
                    cards: vec![card],
                    scope: rule_api::TargetCardScope::HandEquip,
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            let [_, player_raw, target_raw, card_raw, draw_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault(
                    "bad Beizhu draw-pile Sha payload".into(),
                ));
            };
            let player = PlayerId::new(*player_raw);
            if observer != player {
                return Err(EffectError::Illegal(
                    "only the Beizhu owner may resolve this choice".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            if selections != [1] {
                return Err(EffectError::Illegal(
                    "invalid Beizhu draw-pile Sha choice".into(),
                ));
            }
            let target = PlayerId::new(*target_raw);
            let card = CardId::new(*card_raw);
            let draw = engine_api::ZoneId::new(*draw_raw);
            let skill = engine_api::SkillId::new(*skill_raw);
            if ![player, target].into_iter().all(|participant| {
                tx.state()
                    .players
                    .get(participant)
                    .is_some_and(|state| state.alive)
            }) || !tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.owner == player)
                || !tx.state().zones.get(draw).is_some_and(|zone| {
                    zone.kind == ZoneKind::DrawPile
                        && zone.owner.is_none()
                        && zone.cards.contains(&card)
                })
                || !tx
                    .state()
                    .cards
                    .get(card)
                    .is_some_and(|state| state.zone == draw && state.kind.is_sha())
            {
                return Err(EffectError::Illegal(
                    "Beizhu draw-pile Sha choice context changed".into(),
                ));
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("Beizhu target hand is missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: vec![card],
                    to: hand,
                    from: draw,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Beizhu choice stage".into(),
        )),
    }
}

pub(crate) fn vm_trigger_marker(marker: [u32; 3]) -> EffectRequest {
    EffectRequest::EmitRuleEvent {
        code: 100,
        payload: marker.to_vec(),
    }
}

/// Store a VM pause into resolution state and open the corresponding choice.
pub(crate) fn open_vm_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    module_name: &str,
    skill_id: u32,
    ctx: [u32; 4],
    trigger_marker: [u32; 3],
    pause: crate::vm_host::VmPause,
) -> Result<(), EffectError> {
    {
        let r = &mut tx.state_mut().resolution;
        r.vm_cont_frames = pause.frames;
        r.vm_cont_module = module_name.to_string();
        r.vm_cont_pc = pause.pc;
        r.vm_cont_fuel = pause.fuel_remaining;
        r.vm_cont_ctx = ctx;
        r.vm_cont_skill = skill_id;
        r.vm_cont_trigger_marker = trigger_marker;
    }
    crate::effect::open_choice(
        tx,
        observer,
        1,
        1,
        false,
        pause.options,
        engine_api::ChoiceKind::Custom,
        ContinuationKind::VmResume,
        Vec::new(),
    )?;
    Ok(())
}

pub(crate) fn jiedao_give_weapon(
    tx: &mut Transaction,
    actor: PlayerId,
    holder: PlayerId,
) -> Result<(), EffectError> {
    use crate::state::CardKind;
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, holder)
        .ok_or_else(|| EffectError::Illegal("no equip".into()))?;
    let weapon = tx
        .state()
        .zones
        .get(equip)
        .and_then(|z| {
            z.cards.iter().copied().find(|cid| {
                tx.state()
                    .cards
                    .get(*cid)
                    .map(|c| c.kind == CardKind::Weapon)
                    .unwrap_or(false)
            })
        })
        .ok_or_else(|| EffectError::Illegal("no weapon".into()))?;
    let to_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, actor)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon],
            to: to_hand,
            from: Some(equip),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_multi_card_view_as_response(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(1).unwrap_or(&0));
    let kind = payload
        .get(2)
        .and_then(|tag| CardKind::from_tag(*tag as u8))
        .ok_or_else(|| EffectError::EngineFault("bad multi-card response kind".into()))?;
    if cancelled || observer != player || selections.is_empty() {
        return Err(EffectError::Illegal(
            "multi-card response source choice cannot cancel".into(),
        ));
    }
    let rule = crate::command::skill::multi_card_response_rule(tx.state(), player, skill, kind)
        .ok_or_else(|| EffectError::Illegal("multi-card response conversion unavailable".into()))?;
    let cards: Vec<CardId> = selections.iter().copied().map(CardId::new).collect();
    if !crate::command::skill::view_as_sources_match(tx.state(), player, &rule, &cards)
        || cards
            .iter()
            .any(|card| crate::query::card_color_locked_for_card(tx.state(), player, *card))
    {
        return Err(EffectError::Illegal(
            "invalid multi-card response sources".into(),
        ));
    }
    let instance = tx
        .state()
        .skills
        .get(skill)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("unknown response conversion skill".into()))?;
    if rule
        .uses_per_turn
        .is_some_and(|limit| instance.use_count >= u32::from(limit))
    {
        return Err(EffectError::Illegal(
            "view_as skill use limit reached".into(),
        ));
    }
    if rule.cost.as_deref() == Some("potqingyan") {
        run_effects(
            tx,
            vec![EffectRequest::RevealCards {
                cards: cards.clone(),
                to: Vec::new(),
            }],
        )?;
        let round = tx.state().turn.round;
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            if instance.view_as_use_round != round {
                instance.view_as_use_round = round;
                instance.view_as_used_branches = 0;
            }
            instance.view_as_used_branches = instance.view_as_used_branches.saturating_add(1);
            instance.state_blob = cards
                .iter()
                .flat_map(|card| card.raw().to_le_bytes())
                .collect();
        }
    }
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.use_count = instance.use_count.saturating_add(1);
    }
    let bonus = (cards.len() > 1)
        .then_some(rule.multi_card_bonus.as_deref())
        .flatten();
    if rule.cost.as_deref() == Some("potqingyan") {
        complete_view_as_response(tx, payload, &[], bonus)
    } else {
        complete_view_as_response(tx, payload, &cards, bonus)
    }
}

/// Complete a source-free conversion after its cost has resolved.
pub(crate) fn resume_source_free_response(
    tx: &mut Transaction,
    payload: &[u32],
) -> Result<(), EffectError> {
    let skill = engine_api::SkillId::new(*payload.get(1).unwrap_or(&0));
    let source = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .and_then(|skill| skill.take_source_free_physical_card());
    match source {
        Some(source) => complete_view_as_response(tx, payload, &[source], None),
        None => complete_view_as_response(tx, payload, &[], None),
    }
}

pub(crate) fn complete_view_as_response(
    tx: &mut Transaction,
    payload: &[u32],
    physical_sources: &[CardId],
    multi_card_bonus: Option<&str>,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(1).unwrap_or(&0));
    let kind = payload
        .get(2)
        .and_then(|tag| CardKind::from_tag(*tag as u8))
        .ok_or_else(|| EffectError::EngineFault("bad source-free response kind".into()))?;
    let action = *payload.get(3).unwrap_or(&0);
    let original = payload.get(4..).unwrap_or_default();
    let skill_def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .unwrap_or(0);
    let response_card = physical_sources.first().copied().unwrap_or(CardId::NONE);
    let fusi_provider = physical_sources
        .first()
        .and_then(|card| crate::command::skill::dddfusi_approved_source(tx.state(), player, *card));
    let post_draw = crate::command::skill::source_free_post_draw(tx.state(), skill, kind);
    let view_events = |tx: &Transaction| -> Result<Vec<EffectRequest>, EffectError> {
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
        let mut effects = Vec::new();
        for source in physical_sources {
            effects.push(EffectRequest::MoveCardsWithReason {
                cards: vec![*source],
                to: discard,
                from: Some(response_card_zone(tx, player, *source)?),
                reason: CardMoveReason::Respond,
            });
        }
        append_pothaoshi_refills(tx.state(), player, physical_sources, &mut effects);
        effects.push(EffectRequest::EmitRuleEvent {
            code: 120,
            payload: vec![
                skill.raw(),
                skill_def,
                player.raw(),
                response_card.raw(),
                kind.tag() as u32,
            ],
        });
        if let Some(provider) = fusi_provider {
            let fusi_skill = tx
                .state()
                .players
                .get(provider)
                .and_then(|participant| {
                    participant.skills.iter().copied().find(|owned| {
                        tx.state().skills.get(*owned).is_some_and(|instance| {
                            instance.def.raw() == crate::command::skill::DDD_FUSI_DEF
                        })
                    })
                })
                .unwrap_or(engine_api::SkillId::NONE);
            effects.push(EffectRequest::ResolveDddXianglang {
                player: provider,
                actor: player,
                action: 2,
                skill: fusi_skill,
            });
        }
        Ok(effects)
    };
    let append_multi_card_bonus = |tx: &Transaction, effects: &mut Vec<EffectRequest>| {
        if multi_card_bonus != Some("discard_current_hand_equip") {
            return;
        }
        let current = tx.state().turn.current;
        if current != player
            && tx
                .state()
                .players
                .get(current)
                .is_some_and(|participant| participant.alive)
            && !crate::query::discardable_target_cards(
                tx.state(),
                player,
                current,
                rule_api::TargetCardScope::HandEquip,
            )
            .is_empty()
        {
            effects.push(EffectRequest::ChooseDiscardTargetCard {
                player,
                target: current,
                scope: rule_api::TargetCardScope::HandEquip,
            });
        }
    };
    let append_post_draw = |effects: &mut Vec<EffectRequest>| {
        if post_draw > 0 {
            effects.push(EffectRequest::DrawCards {
                player,
                count: u32::from(post_draw),
            });
        }
    };

    tx.state_mut().turn.set_flag(
        crate::state::turn_flag::VIEW_AS_CARD,
        response_card.raw() as i32,
    );
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
    tx.state_mut().turn.set_flag(
        crate::state::turn_flag::VIEW_AS_LOGICAL_KIND,
        i32::from(kind.tag()),
    );
    let result = (|| {
        match action {
            7 => {
                let owner = PlayerId::new(*original.first().unwrap_or(&0));
                let mengmou = engine_api::SkillId::new(*original.get(1).unwrap_or(&0));
                let mut effects = view_events(tx)?;
                effects.push(EffectRequest::CardResponded {
                    player,
                    respond_to: owner,
                    card: response_card,
                    response_kind: CardKind::Sha.tag(),
                });
                append_post_draw(&mut effects);
                append_multi_card_bonus(tx, &mut effects);
                effects.push(EffectRequest::ResolveDcSbLusu {
                    player: owner,
                    source: player,
                    card: response_card,
                    action: 22,
                    cards: Vec::new(),
                    amount: 0,
                    skill: mengmou,
                });
                run_response_effects(tx, effects, ContinuationKind::None, Vec::new())
            }
            1 => {
                // RespondToSha payload: [source, target, amount, need, have, context, respondable, card]
                let source = PlayerId::new(*original.first().unwrap_or(&0));
                let target = PlayerId::new(*original.get(1).unwrap_or(&0));
                let amount = *original.get(2).unwrap_or(&1);
                let need = *original.get(3).unwrap_or(&1);
                let have = original.get(4).copied().unwrap_or(0).saturating_add(1);
                let context = *original.get(5).unwrap_or(&0);
                let respondable = *original.get(6).unwrap_or(&1);
                let card = *original.get(7).unwrap_or(&0);
                let resume = if have < need {
                    vec![
                        source.raw(),
                        target.raw(),
                        amount,
                        need,
                        have,
                        context,
                        respondable,
                        card,
                    ]
                } else {
                    Vec::new()
                };
                let mut response_effects = view_events(tx)?;
                response_effects.extend(sha_response_effects(
                    target,
                    source,
                    response_card,
                    CardId::new(card),
                    have,
                    need,
                ));
                response_effects.push(EffectRequest::EmitRuleEvent {
                    code: 121,
                    payload: vec![source.raw(), target.raw(), response_card.raw(), have, need],
                });
                append_post_draw(&mut response_effects);
                append_multi_card_bonus(tx, &mut response_effects);
                if have >= need {
                    crate::command::play::save_played_sha_finish(tx, source, CardId::new(card), context)?;
                }
                run_response_effects(tx, response_effects, ContinuationKind::RespondToSha, resume)?;
                if have < need {
                    let (nature, card_flags) = crate::command::play::unpack_damage_card_context(context);
                    return crate::command::play::open_sha_response_step(
                        tx,
                        source,
                        target,
                        amount,
                        need,
                        have,
                        nature,
                        card_flags,
                        respondable != 0,
                        CardId::new(card),
                    );
                }
                crate::command::play::finish_saved_played_sha(tx, source, CardId::new(card))
            }
            2 => {
                // RespondAoeQueue, including its kind=3 multi-target Sha form.
                let queue_kind = *original.first().unwrap_or(&0);
                let source = PlayerId::new(*original.get(1).unwrap_or(&0));
                let idx = *original.get(3).unwrap_or(&0) as usize;
                if queue_kind == 3 {
                    let have = original.get(7).copied().unwrap_or(0).saturating_add(1);
                    let need = *original.get(6).unwrap_or(&1);
                    let target = PlayerId::new(*original.get(8 + idx).unwrap_or(&0));
                    let len = *original.get(4).unwrap_or(&0) as usize;
                    let card = CardId::new(*original.get(8 + 2 * len).unwrap_or(&0));
                    let next = multi_sha_queue_next(original, idx, have);
                    let mut response_effects = view_events(tx)?;
                    response_effects.extend(sha_response_effects(
                        target,
                        source,
                        response_card,
                        card,
                        have,
                        need,
                    ));
                    append_post_draw(&mut response_effects);
                    append_multi_card_bonus(tx, &mut response_effects);
                    run_response_effects(
                        tx,
                        response_effects,
                        ContinuationKind::RespondAoeQueue,
                        next.clone(),
                    )?;
                    return continue_response_queue(tx, next);
                }
                let target = PlayerId::new(*original.get(5 + idx).unwrap_or(&0));
                let response_kind = if queue_kind == 1 {
                    CardKind::Sha
                } else {
                    CardKind::Shan
                };
                let mut next = original.to_vec();
                if next.len() > 3 {
                    next[3] = (idx + 1) as u32;
                }
                let mut response_effects = view_events(tx)?;
                response_effects.push(EffectRequest::CardResponded {
                    player: target,
                    respond_to: source,
                    card: response_card,
                    response_kind: response_kind.tag(),
                });
                response_effects.push(EffectRequest::EmitRuleEvent {
                    code: 123,
                    payload: vec![queue_kind, source.raw(), target.raw(), response_card.raw()],
                });
                append_post_draw(&mut response_effects);
                append_multi_card_bonus(tx, &mut response_effects);
                run_response_effects(
                    tx,
                    response_effects,
                    ContinuationKind::RespondAoeQueue,
                    next.clone(),
                )?;
                tx.state_mut().resolution.continuation_kind = ContinuationKind::RespondAoeQueue;
                tx.state_mut().resolution.continuation_payload = next;
                crate::command::play::advance_aoe_queue(tx)
            }
            3 => {
                // JuedouDuel payload:
                // [source, target, next_responder, exchange_count, need, have, tail...]
                let source = PlayerId::new(*original.first().unwrap_or(&0));
                let target = PlayerId::new(*original.get(1).unwrap_or(&0));
                let next = PlayerId::new(*original.get(2).unwrap_or(&0));
                let exchanges = *original.get(3).unwrap_or(&0);
                let need = *original.get(4).unwrap_or(&1);
                let have = *original.get(5).unwrap_or(&0);
                let tail = original.get(6..).unwrap_or_default();
                let resume = crate::command::play::juedou_payload_after_response(
                    source, target, next, exchanges, need, have, tail,
                );
                let mut response_effects = view_events(tx)?;
                response_effects.push(EffectRequest::CardResponded {
                    player: next,
                    respond_to: source,
                    card: response_card,
                    response_kind: CardKind::Sha.tag(),
                });
                response_effects.push(EffectRequest::EmitRuleEvent {
                    code: 124,
                    payload: vec![source.raw(), target.raw(), next.raw(), response_card.raw()],
                });
                append_post_draw(&mut response_effects);
                run_response_effects(
                    tx,
                    response_effects,
                    ContinuationKind::JuedouDuel,
                    resume.clone(),
                )?;
                crate::command::play::continue_juedou_payload(tx, &resume)
            }
            4 => {
                // WuxieWindow payload is preserved verbatim from the original ask.
                let ask_len = *original.get(2).unwrap_or(&0) as usize;
                let effect_off = 3 + ask_len;
                let nullified = original.first().copied().unwrap_or(0) ^ 1;
                let effect_tag = *original.get(effect_off).unwrap_or(&0);
                let actor = PlayerId::new(*original.get(effect_off + 1).unwrap_or(&0));
                let respond_to = PlayerId::new(*original.get(effect_off + 2).unwrap_or(&0));
                let effect_args = original.get(effect_off + 5..).unwrap_or_default().to_vec();
                let order = tx.state().players.alive_by_seat();
                let pos = order
                    .iter()
                    .position(|candidate| *candidate == player)
                    .unwrap_or(0);
                let start = if order.is_empty() {
                    player
                } else {
                    order[(pos + 1) % order.len()]
                };
                let response_suit = tx
                    .state()
                    .cards
                    .get(response_card)
                    .map(|card| crate::query::effective_card_suit(tx.state(), player, card))
                    .unwrap_or(engine_api::Suit::None);
                let mut resume = vec![
                    start.raw(),
                    nullified,
                    effect_tag,
                    actor.raw(),
                    player.raw(),
                    CardKind::TrickWuxie.tag() as u32,
                    crate::command::judge::suit_tag(response_suit),
                ];
                resume.extend_from_slice(&effect_args);
                let mut response_effects = view_events(tx)?;
                response_effects.push(EffectRequest::CardResponded {
                    player,
                    respond_to,
                    card: response_card,
                    response_kind: CardKind::TrickWuxie.tag(),
                });
                response_effects.push(EffectRequest::EmitRuleEvent {
                    code: 221,
                    payload: vec![player.raw(), response_card.raw()],
                });
                append_post_draw(&mut response_effects);
                append_multi_card_bonus(tx, &mut response_effects);
                run_response_effects(tx, response_effects, ContinuationKind::WuxieWindow, resume)?;
                crate::command::play::begin_wuxie_window_with_parity(
                    tx,
                    start,
                    nullified,
                    effect_tag,
                    actor,
                    player,
                    CardKind::TrickWuxie,
                    response_suit,
                    &effect_args,
                )
            }
            5 => {
                // DyingSave payload: [dying, killer, responder, rest_len, rest...].
                let dying = PlayerId::new(*original.first().unwrap_or(&0));
                let killer = PlayerId::new(*original.get(1).unwrap_or(&0));
                let responder = PlayerId::new(*original.get(2).unwrap_or(&0));
                let len = *original.get(3).unwrap_or(&u32::MAX) as usize;
                let remaining = original
                    .get(4..)
                    .filter(|rest| rest.len() == len)
                    .ok_or_else(|| {
                        EffectError::EngineFault("truncated source-free dying-save queue".into())
                    })?;
                if player != responder
                    || crate::query::card_save_cancelled_for_kind_to(
                        tx.state(),
                        player,
                        dying,
                        kind,
                    )
                    || (kind == CardKind::Jiu && player != dying)
                {
                    return Err(EffectError::Illegal("card save disabled".into()));
                }
                let mut effects = view_events(tx)?;
                effects.push(EffectRequest::CardResponded {
                    player,
                    respond_to: dying,
                    card: response_card,
                    response_kind: kind.tag(),
                });
                append_post_draw(&mut effects);
                if tx
                    .state()
                    .players
                    .get(dying)
                    .is_some_and(|state| state.hp <= 0)
                {
                    effects.push(EffectRequest::RecoverHp {
                        source: player,
                        player: dying,
                        amount: 1 + u32::from(multi_card_bonus == Some("recover_1")),
                    });
                }
                let mut responders = vec![responder];
                responders.extend(remaining.iter().copied().map(PlayerId::new));
                effects.push(EffectRequest::ContinueDyingSave {
                    player: dying,
                    source: killer,
                    responders,
                });
                run_effects(tx, effects)?;
                resume_interrupted_continuation(tx)
            }
            6 => {
                // Proactive delegated Sha payload: [target_count, targets...].
                if kind != CardKind::Sha {
                    return Err(EffectError::ContentFault(
                        "proactive delegated card must be Sha".into(),
                    ));
                }
                let target_count = *original.first().unwrap_or(&u32::MAX) as usize;
                let targets = original
                    .get(1..)
                    .filter(|targets| targets.len() == target_count)
                    .ok_or_else(|| {
                        EffectError::EngineFault("truncated proactive delegated Sha targets".into())
                    })?;
                let mut play = vec![
                    player.raw(),
                    skill.raw(),
                    kind.tag() as u32,
                    target_count as u32,
                ];
                play.extend_from_slice(targets);
                crate::command::skill::resume_source_free_play(tx, &play)
            }
            _ => Err(EffectError::EngineFault(
                "unknown source-free response continuation".into(),
            )),
        }
    })();
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_CARD, 0);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, 0);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    result
}

pub(crate) fn begin_delegated_group_response(
    tx: &mut Transaction,
    owner: PlayerId,
    skill: engine_api::SkillId,
    kind: CardKind,
    action: u32,
    action_payload: &[u32],
    group: &str,
) -> Result<(), EffectError> {
    let providers = crate::command::skill::delegated_response_candidates(tx.state(), owner, group, kind);
    if providers.is_empty() {
        return Err(EffectError::Illegal(
            "no group member can provide the requested response".into(),
        ));
    }
    let mut payload = vec![
        owner.raw(),
        skill.raw(),
        kind.tag() as u32,
        action,
        0,
        providers.len() as u32,
    ];
    payload.extend(providers.iter().map(|player| player.raw()));
    payload.extend_from_slice(action_payload);
    open_delegated_group_response(tx, payload)
}

pub(crate) fn resolve_delegated_group_response(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let (_, _, kind, provider_index, providers, _) = delegated_response_payload(payload)?;
    let provider = *providers
        .get(provider_index)
        .ok_or_else(|| EffectError::Illegal("delegated response provider is missing".into()))?;
    if observer != provider {
        return Err(EffectError::Illegal(
            "only the current provider may resolve delegated response".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        let mut next = payload.to_vec();
        next[4] = (provider_index + 1) as u32;
        return open_delegated_group_response(tx, next);
    }
    if selections.len() != 1 || !matches!(kind, CardKind::Sha | CardKind::Shan) {
        return Err(EffectError::Illegal(
            "delegated response requires one card".into(),
        ));
    }
    let token = selections[0];
    let legal = match kind {
        CardKind::Sha => crate::command::play::delegated_sha_options(tx.state(), provider),
        CardKind::Shan => crate::command::play::delegated_shan_options(tx.state(), provider),
        _ => Vec::new(),
    };
    if !legal.contains(&token) {
        return Err(EffectError::Illegal(
            "not a legal delegated card response".into(),
        ));
    }
    if kind == CardKind::Shan && token == crate::content::BAGUA_RESPONSE_TOKEN {
        let (ok, judgment) = crate::command::play::try_bagua_as_shan(tx, provider)?;
        if ok {
            return complete_delegated_group_response(
                tx,
                payload,
                vec![
                    judgment,
                    EffectRequest::CardResponded {
                        player: provider,
                        respond_to: PlayerId::new(payload[0]),
                        card: CardId::NONE,
                        response_kind: CardKind::Shan.tag(),
                    },
                ],
            );
        }
        let mut next = payload.to_vec();
        next[4] = (provider_index + 1) as u32;
        run_response_effects(
            tx,
            vec![judgment],
            ContinuationKind::DelegatedGroupResponse,
            next.clone(),
        )?;
        return open_delegated_group_response(tx, next);
    }
    let card = CardId::new(token);
    let from = response_card_zone(tx, provider, card)?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
    let mut effects = vec![EffectRequest::MoveCardsWithReason {
        cards: vec![card],
        to: discard,
        from: Some(from),
        reason: CardMoveReason::Respond,
    }];
    append_pothaoshi_refills(tx.state(), provider, &[card], &mut effects);
    effects.push(EffectRequest::CardResponded {
        player: provider,
        respond_to: PlayerId::new(payload[0]),
        card,
        response_kind: kind.tag(),
    });
    effects.push(EffectRequest::EmitRuleEvent {
        code: 126,
        payload: vec![payload[0], provider.raw(), card.raw(), kind.tag() as u32],
    });
    complete_delegated_group_response(tx, payload, effects)
}

pub(crate) fn delegated_response_payload(
    payload: &[u32],
) -> Result<
    (
        PlayerId,
        engine_api::SkillId,
        CardKind,
        usize,
        Vec<PlayerId>,
        &[u32],
    ),
    EffectError,
> {
    let provider_count = *payload
        .get(5)
        .ok_or_else(|| EffectError::EngineFault("truncated delegated response".into()))?
        as usize;
    let original_offset = 6 + provider_count;
    if payload.len() < original_offset {
        return Err(EffectError::EngineFault(
            "truncated delegated response providers".into(),
        ));
    }
    let kind = payload
        .get(2)
        .and_then(|tag| CardKind::from_tag(*tag as u8))
        .ok_or_else(|| EffectError::EngineFault("bad delegated response kind".into()))?;
    Ok((
        PlayerId::new(payload[0]),
        engine_api::SkillId::new(payload[1]),
        kind,
        payload[4] as usize,
        payload[6..original_offset]
            .iter()
            .copied()
            .map(PlayerId::new)
            .collect(),
        &payload[original_offset..],
    ))
}

pub(crate) fn open_delegated_group_response(
    tx: &mut Transaction,
    mut payload: Vec<u32>,
) -> Result<(), EffectError> {
    loop {
        let (owner, _, kind, provider_index, providers, _) = delegated_response_payload(&payload)?;
        let Some(provider) = providers.get(provider_index).copied() else {
            return cancel_delegated_group_response(tx, &payload);
        };
        let options = match kind {
            CardKind::Sha => crate::command::play::delegated_sha_options(tx.state(), provider),
            CardKind::Shan => crate::command::play::delegated_shan_options(tx.state(), provider),
            _ => Vec::new(),
        };
        if options.is_empty()
            || provider == owner
            || !tx
                .state()
                .players
                .get(provider)
                .is_some_and(|player| player.alive)
        {
            payload[4] = (provider_index + 1) as u32;
            continue;
        }
        crate::effect::open_choice(
            tx,
            provider,
            0,
            1,
            true,
            options,
            engine_api::ChoiceKind::SelectCards,
            ContinuationKind::DelegatedGroupResponse,
            payload,
        )?;
        return Err(EffectError::ChoiceRequested);
    }
}

pub(crate) fn complete_delegated_group_response(
    tx: &mut Transaction,
    payload: &[u32],
    effects: Vec<EffectRequest>,
) -> Result<(), EffectError> {
    let (owner, skill, kind, _, _, original) = delegated_response_payload(payload)?;
    let action = payload[3];
    let mut resume = vec![owner.raw(), skill.raw(), kind.tag() as u32, action];
    resume.extend_from_slice(original);
    run_response_effects(
        tx,
        effects,
        ContinuationKind::SourceFreeViewAsResponse,
        resume.clone(),
    )?;
    resume_source_free_response(tx, &resume)
}

pub(crate) fn cancel_delegated_group_response(
    tx: &mut Transaction,
    payload: &[u32],
) -> Result<(), EffectError> {
    let (_, skill, _, _, _, original) = delegated_response_payload(payload)?;
    match payload[3] {
        1 => {
            let source = PlayerId::new(*original.first().unwrap_or(&0));
            let target = PlayerId::new(*original.get(1).unwrap_or(&0));
            let amount = *original.get(2).unwrap_or(&1);
            let (nature, card_flags) =
                crate::command::play::unpack_damage_card_context(*original.get(5).unwrap_or(&0));
            let card = CardId::new(*original.get(7).unwrap_or(&0));
            crate::command::play::deal_single_sha_damage(
                tx, source, target, amount, nature, card_flags, card,
            )
        }
        2 if original.first() == Some(&3) => {
            resolve_multi_sha_queue_choice(tx, original, &[], true)
        }
        2 => cancel_ordinary_aoe_response(tx, original),
        3 => {
            let source = PlayerId::new(*original.first().unwrap_or(&0));
            let target = PlayerId::new(*original.get(1).unwrap_or(&0));
            let next = PlayerId::new(*original.get(2).unwrap_or(&0));
            let tail = original.get(6..).unwrap_or_default();
            let dmg_source = if next == target { source } else { target };
            let amount = crate::query::sha_damage_for(tx.state(), dmg_source, 1);
            crate::command::play::finish_juedou_with_tail(tx, source, target, next, amount, tail)
        }
        6 => {
            let revision = tx.state().revision.saturating_add(1);
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.set_delegated_retry_lock_revision(revision);
            }
            Ok(())
        }
        7 => {
            let owner = PlayerId::new(*original.first().unwrap_or(&0));
            let mengmou = engine_api::SkillId::new(*original.get(1).unwrap_or(&0));
            run_effects(
                tx,
                vec![EffectRequest::ResolveDcSbLusu {
                    player: owner,
                    source: owner,
                    card: CardId::NONE,
                    action: 23,
                    cards: Vec::new(),
                    amount: 0,
                    skill: mengmou,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::ContentFault(
            "unsupported delegated response context".into(),
        )),
    }
}

pub(crate) fn cancel_ordinary_aoe_response(tx: &mut Transaction, payload: &[u32]) -> Result<(), EffectError> {
    let source = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let amount = *payload.get(2).unwrap_or(&1);
    let idx = *payload.get(3).unwrap_or(&0) as usize;
    let target = PlayerId::new(*payload.get(5 + idx).unwrap_or(&0));
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature: 0,
        }],
    )?;
    if tx.state().resolution.continuation_kind == ContinuationKind::DyingSave {
        if tx.state().resolution.resume_kind == ContinuationKind::RespondAoeQueue
            && tx.state().resolution.resume_payload.len() > 3
        {
            tx.state_mut().resolution.resume_payload[3] = (idx + 1) as u32;
        }
        return Ok(());
    }
    let mut next = payload.to_vec();
    next[3] = (idx + 1) as u32;
    tx.state_mut().resolution.continuation_kind = ContinuationKind::RespondAoeQueue;
    tx.state_mut().resolution.continuation_payload = next;
    crate::command::play::advance_aoe_queue(tx)
}

pub(crate) fn run_response_effects(
    tx: &mut Transaction,
    effects: Vec<EffectRequest>,
    resume_kind: ContinuationKind,
    resume_payload: Vec<u32>,
) -> Result<(), EffectError> {
    match run_effects(tx, effects) {
        Ok(_) => Ok(()),
        Err(EffectError::ChoiceRequested) => {
            if resume_kind != ContinuationKind::None && !resume_payload.is_empty() {
                let resolution = &mut tx.state_mut().resolution;
                resolution.resume_kind = resume_kind;
                resolution.resume_payload = resume_payload;
            }
            Err(EffectError::ChoiceRequested)
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn run_physical_response_effects(
    tx: &mut Transaction,
    mut effects: Vec<EffectRequest>,
    resume_kind: ContinuationKind,
    resume_payload: Vec<u32>,
    player: PlayerId,
    card: CardId,
    response_kind: CardKind,
) -> Result<(), EffectError> {
    for effect in &mut effects {
        if matches!(effect, EffectRequest::MoveCards { cards, .. } if cards.contains(&card)) {
            let EffectRequest::MoveCards { cards, to, from } = effect.clone() else {
                unreachable!();
            };
            *effect = EffectRequest::MoveCardsWithReason {
                cards,
                to,
                from,
                reason: CardMoveReason::Respond,
            };
        }
    }
    let refills = crate::effect::pothaoshi_refill_effects(tx.state(), player, &[card]);
    if !refills.is_empty() {
        let insert_at = effects
            .iter()
            .rposition(|effect| match effect {
                EffectRequest::MoveCards { cards, .. }
                | EffectRequest::MoveCardsWithReason { cards, .. } => cards.contains(&card),
                _ => false,
            })
            .map_or(0, |index| index + 1);
        effects.splice(insert_at..insert_at, refills);
    }
    let view_skill = tx.state().cards.get(card).and_then(|card_state| {
        let physical_kind = crate::query::effective_card_kind(tx.state(), player, card_state);
        let natural = if response_kind == CardKind::Shan {
            physical_kind == CardKind::Shan
        } else {
            response_kind.is_sha() && physical_kind.is_sha()
        };
        (!natural)
            .then(|| {
                crate::command::play::view_as_response_skill(
                    tx.state(),
                    player,
                    card_state,
                    if response_kind == CardKind::Shan {
                        "shan"
                    } else {
                        "sha"
                    },
                )
            })
            .flatten()
    });
    if let Some(skill) = view_skill {
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::VIEW_AS_CARD, card.raw() as i32);
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
        tx.state_mut().turn.set_flag(
            crate::state::turn_flag::VIEW_AS_LOGICAL_KIND,
            i32::from(response_kind.tag()),
        );
    }
    let result = run_response_effects(tx, effects, resume_kind, resume_payload);
    if view_skill.is_some() {
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::VIEW_AS_CARD, 0);
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, 0);
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    }
    result
}

pub(crate) fn sha_response_effects(
    target: PlayerId,
    source: PlayerId,
    response_card: CardId,
    sha_card: CardId,
    have: u32,
    need: u32,
) -> Vec<EffectRequest> {
    let mut effects = vec![EffectRequest::CardResponded {
        player: target,
        respond_to: source,
        card: response_card,
        response_kind: CardKind::Shan.tag(),
    }];
    if have >= need {
        effects.push(EffectRequest::ShaDodged {
            source,
            target,
            card: sha_card,
        });
    }
    effects
}

pub(crate) fn resolve_multi_sha_queue_choice(
    tx: &mut Transaction,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 9 {
        return Err(EffectError::EngineFault(
            "malformed multi-target sha queue".into(),
        ));
    }
    let source = PlayerId::new(payload[1]);
    let idx = payload[3] as usize;
    let len = payload[4] as usize;
    let amount = payload[2].saturating_add(crate::command::play::sha_queue_target_damage_bonus(
        payload, 8, len, idx,
    ));
    let (nature, card_flags) = crate::command::play::unpack_damage_card_context(payload[5]);
    let need = payload[6];
    let have = payload[7];
    let card = CardId::new(*payload.get(8 + 2 * len).unwrap_or(&0));
    if idx >= len || 8 + idx >= payload.len() {
        return Err(EffectError::EngineFault(
            "multi-target sha queue index out of bounds".into(),
        ));
    }
    let target = PlayerId::new(payload[8 + idx]);

    if cancelled || selections.is_empty() {
        let next = multi_sha_queue_next(payload, idx, need);
        {
            let resolution = &mut tx.state_mut().resolution;
            resolution.resume_kind = ContinuationKind::RespondAoeQueue;
            resolution.resume_payload = next;
        }
        return match crate::command::play::deal_sha_damage(
            tx, source, target, amount, nature, card_flags, card,
        ) {
            Ok(_) => {
                let resolution = &mut tx.state_mut().resolution;
                let next = std::mem::take(&mut resolution.resume_payload);
                resolution.resume_kind = ContinuationKind::None;
                continue_response_queue(tx, next)
            }
            Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
            Err(error) => Err(error),
        };
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal("play at most one 闪".into()));
    }

    let token = selections[0];
    if have == 0 && crate::command::play::response_youdishenru_tokens(tx.state(), target).contains(&token) {
        return crate::command::play::use_youdishenru_for_saved_sha(
            tx,
            target,
            source,
            card,
            CardId::new(token),
            ContinuationKind::RespondAoeQueue,
            payload,
        );
    }
    if token == crate::content::BAGUA_RESPONSE_TOKEN {
        let (ok, judgment) = crate::command::play::try_bagua_as_shan(tx, target)?;
        if !ok {
            run_response_effects(
                tx,
                vec![judgment],
                ContinuationKind::RespondAoeQueue,
                payload.to_vec(),
            )?;
            return continue_response_queue(tx, payload.to_vec());
        }
        let next_have = have.saturating_add(1);
        let next = multi_sha_queue_next(payload, idx, next_have);
        let mut effects = vec![judgment];
        effects.extend(sha_response_effects(
            target,
            source,
            CardId::NONE,
            card,
            next_have,
            need,
        ));
        run_response_effects(tx, effects, ContinuationKind::RespondAoeQueue, next.clone())?;
        return continue_response_queue(tx, next);
    }

    if let Some(skill) =
        crate::command::skill::source_free_response_skill(tx.state(), target, CardKind::Shan, token)
    {
        return crate::command::skill::begin_source_free_response(
            tx,
            target,
            skill,
            CardKind::Shan,
            2,
            payload,
        );
    }
    if let Some(skill) =
        crate::command::skill::multi_card_response_skill(tx.state(), target, CardKind::Shan, token)
    {
        return crate::command::skill::begin_multi_card_response(
            tx,
            target,
            skill,
            CardKind::Shan,
            2,
            payload,
        );
    }

    let cid = CardId::new(token);
    if !crate::command::play::response_shan_tokens(tx.state(), target).contains(&cid.raw()) {
        return Err(EffectError::Illegal("not a legal 闪 response".into()));
    }
    let from = response_card_zone(tx, target, cid)?;
    tx.state_mut()
        .resolution
        .register_sha_response_card(source, card, cid);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
    let next_have = have.saturating_add(1);
    let next = multi_sha_queue_next(payload, idx, next_have);
    let mut response_effects = vec![EffectRequest::MoveCards {
        cards: vec![cid],
        to: discard,
        from: Some(from),
    }];
    response_effects.extend(sha_response_effects(
        target, source, cid, card, next_have, need,
    ));
    response_effects.push(EffectRequest::EmitRuleEvent {
        code: 121,
        payload: vec![source.raw(), target.raw(), cid.raw(), next_have, need],
    });
    run_physical_response_effects(
        tx,
        response_effects,
        ContinuationKind::RespondAoeQueue,
        next.clone(),
        target,
        cid,
        CardKind::Shan,
    )?;
    continue_response_queue(tx, next)
}

pub(crate) fn multi_sha_queue_next(payload: &[u32], idx: usize, have: u32) -> Vec<u32> {
    let mut next = payload.to_vec();
    if have < payload[6] {
        next[7] = have;
    } else {
        next[3] = (idx + 1) as u32;
        next[7] = 0;
    }
    next
}

pub(crate) fn continue_response_queue(tx: &mut Transaction, payload: Vec<u32>) -> Result<(), EffectError> {
    if tx.state().resolution.continuation_kind == ContinuationKind::DyingSave {
        let resolution = &mut tx.state_mut().resolution;
        resolution.resume_kind = ContinuationKind::RespondAoeQueue;
        resolution.resume_payload = payload;
        return Ok(());
    }
    let resolution = &mut tx.state_mut().resolution;
    resolution.continuation_kind = ContinuationKind::RespondAoeQueue;
    resolution.continuation_payload = payload;
    crate::command::play::advance_aoe_queue(tx)
}

pub(crate) fn resume_interrupted_continuation(tx: &mut Transaction) -> Result<(), EffectError> {
    let kind = tx.state().resolution.resume_kind;
    let payload = tx.state().resolution.resume_payload.clone();
    if kind == ContinuationKind::None || payload.is_empty() {
        return Ok(());
    }
    tx.state_mut().resolution.resume_kind = ContinuationKind::None;
    tx.state_mut().resolution.resume_payload.clear();
    match kind {
        ContinuationKind::RespondAoeQueue => {
            tx.state_mut().resolution.continuation_kind = kind;
            tx.state_mut().resolution.continuation_payload = payload;
            crate::command::play::advance_aoe_queue(tx)
        }
        ContinuationKind::JuedouDuel => crate::command::play::continue_juedou_payload(tx, &payload),
        ContinuationKind::IndependentTrickQueue => {
            crate::command::play::advance_independent_trick_queue(tx, &payload)
        }
        ContinuationKind::RespondToSha => {
            let source = PlayerId::new(*payload.first().unwrap_or(&0));
            let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let amount = *payload.get(2).unwrap_or(&1);
            let need = *payload.get(3).unwrap_or(&1);
            let have = *payload.get(4).unwrap_or(&0);
            let (nature, card_flags) =
                crate::command::play::unpack_damage_card_context(*payload.get(5).unwrap_or(&0));
            let respondable = *payload.get(6).unwrap_or(&1) != 0;
            let card = CardId::new(*payload.get(7).unwrap_or(&0));
            crate::command::play::open_sha_response_step(
                tx,
                source,
                target,
                amount,
                need,
                have,
                nature,
                card_flags,
                respondable,
                card,
            )
        }
        ContinuationKind::ResumePlayedSha | ContinuationKind::ResumePlayedShaQueue => {
            crate::command::play::resume_played_sha_from_payload(tx, kind, &payload)
        }
        ContinuationKind::FinishPlayedSha => {
            let source = PlayerId::new(*payload.first().unwrap_or(&0));
            let card = CardId::new(*payload.get(1).unwrap_or(&0));
            crate::command::play::finish_played_sha(tx, source, card, *payload.get(2).unwrap_or(&0))
        }
        ContinuationKind::FinishDelayedJudgment => {
            crate::command::judge::finish_delayed_judgment(tx, &payload)
        }
        ContinuationKind::JudgeDamageOtherOnSuit => {
            let player = PlayerId::new(*payload.first().unwrap_or(&0));
            let amount = *payload.get(1).unwrap_or(&0);
            let nature = *payload.get(2).unwrap_or(&0) as u8;
            crate::effect::open_judgment_damage_other_choice(tx, player, amount, nature).map(|_| ())
        }
        ContinuationKind::GanglieSourceDiscardOrDamage => {
            let player = PlayerId::new(*payload.first().unwrap_or(&0));
            let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let hand_only = *payload.get(2).unwrap_or(&1) != 0;
            crate::effect::open_ganglie_target_discard_or_damage(tx, player, target, hand_only)
                .map(|_| ())
        }
        ContinuationKind::XiaoguoDiscardBasicThenEquipOrDamage => {
            let [stage, player, target, skill] = payload.as_slice() else {
                return Err(EffectError::EngineFault(
                    "bad saved Xiaoguo continuation".into(),
                ));
            };
            if *stage != 1 {
                return Err(EffectError::EngineFault(
                    "bad saved Xiaoguo continuation stage".into(),
                ));
            }
            crate::effect::open_xiaoguo_target_equipment_or_damage(
                tx,
                PlayerId::new(*player),
                PlayerId::new(*target),
                engine_api::SkillId::new(*skill),
            )
        }
        ContinuationKind::TargetDiscardThenDuelIfNonShaAndHpGteOwner => {
            let [player, target, _skill, discarded_was_sha] = payload.as_slice() else {
                return Err(EffectError::EngineFault(
                    "bad saved target-discard Duel continuation".into(),
                ));
            };
            crate::effect::resolve_target_discard_duel(
                tx,
                PlayerId::new(*player),
                PlayerId::new(*target),
                *discarded_was_sha != 0,
            )
            .map(|_| ())
        }
        ContinuationKind::WuxieWindow => {
            let start = PlayerId::new(*payload.first().unwrap_or(&0));
            let nullified = *payload.get(1).unwrap_or(&0);
            let effect_tag = *payload.get(2).unwrap_or(&0);
            let actor = PlayerId::new(*payload.get(3).unwrap_or(&0));
            let respond_to = PlayerId::new(*payload.get(4).unwrap_or(&0));
            let response_kind = payload
                .get(5)
                .and_then(|tag| CardKind::from_tag(*tag as u8))
                .unwrap_or(CardKind::Other);
            let response_suit = crate::command::judge::suit_from_tag(*payload.get(6).unwrap_or(&1));
            crate::command::play::resume_wuxie_after_card_used_triggers(
                tx,
                start,
                nullified,
                effect_tag,
                actor,
                respond_to,
                response_kind,
                response_suit,
                payload.get(7..).unwrap_or_default(),
            )
        }
        ContinuationKind::SourceFreeViewAsResponse => resume_source_free_response(tx, &payload),
        ContinuationKind::DelegatedGroupResponse => open_delegated_group_response(tx, payload),
        ContinuationKind::SourceFreeViewAsPlay => {
            crate::command::skill::resume_source_free_play(tx, &payload)
        }
        ContinuationKind::ContinueEnteredDrawPhase => {
            let player = PlayerId::new(*payload.first().unwrap_or(&0));
            crate::command::turn::continue_entered_draw_phase(tx, player)
        }
        _ => Err(EffectError::EngineFault(format!(
            "unsupported saved continuation {kind:?}"
        ))),
    }
}

pub(crate) fn resolve_twfupan(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [stage, player, damage_amount, card, target, skill] = payload else {
        return Err(EffectError::EngineFault("bad twfupan payload".into()));
    };
    let player = PlayerId::new(*player);
    let card = CardId::new(*card);
    let target = PlayerId::new(*target);
    let skill = SkillId::new(*skill);
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "twfupan owner or skill changed".into(),
        ));
    }
    let card_zone = |tx: &Transaction, card: CardId| {
        tx.state().cards.get(card).and_then(|state| {
            (state.owner == player
                && tx
                    .state()
                    .zones
                    .get(state.zone)
                    .is_some_and(|zone| matches!(zone.kind, ZoneKind::Hand | ZoneKind::Equip)))
            .then_some(state.zone)
        })
    };
    let target_eligible = |tx: &Transaction, target: PlayerId| {
        target != player
            && tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            && tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.twfupan_target_state(target) < 2)
    };
    match *stage {
        1 => {
            if cancelled {
                return Err(EffectError::Illegal(
                    "twfupan card choice cannot cancel".into(),
                ));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal("twfupan requires one card".into()));
            };
            let selected = CardId::new(*selected);
            if card_zone(tx, selected).is_none() {
                return Err(EffectError::Illegal("twfupan selected card moved".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwfupan {
                    player,
                    damage_amount: *damage_amount,
                    card: selected,
                    target: PlayerId::NONE,
                    stage: 2,
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 => {
            if cancelled {
                return Err(EffectError::Illegal(
                    "twfupan target choice cannot cancel".into(),
                ));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal("twfupan requires one target".into()));
            };
            let selected = PlayerId::new(*selected);
            let from = card_zone(tx, card)
                .ok_or_else(|| EffectError::Illegal("twfupan selected card moved".into()))?;
            if !target_eligible(tx, selected) {
                return Err(EffectError::Illegal(
                    "twfupan target became ineligible".into(),
                ));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, selected)
                .ok_or_else(|| EffectError::EngineFault("twfupan target hand missing".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::GiveCards {
                        cards: vec![card],
                        to,
                        from,
                    },
                    EffectRequest::ResolveTwfupan {
                        player,
                        damage_amount: *damage_amount,
                        card: CardId::NONE,
                        target: selected,
                        stage: 3,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        3 => {
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            if selections != [1] || !target_eligible(tx, target) {
                return Err(EffectError::Illegal("invalid twfupan damage choice".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwfupan {
                    player,
                    damage_amount: *damage_amount,
                    card: CardId::NONE,
                    target,
                    stage: 4,
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown twfupan choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_choose_spjungong_discard_or_lose(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad 峻攻 continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let skill = engine_api::SkillId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let n = payload[3];
    if observer != player
        || cancelled
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "峻攻 discard-or-lose context changed".into(),
        ));
    }
    let selected: Vec<CardId> = selections.iter().map(|c| CardId::new(*c)).collect();
    let mut queue: Vec<EffectRequest> = Vec::new();
    if selected.is_empty() {
        // 选 0 张 → 失去等量体力
        if tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.hp >= 1)
        {
            queue.push(EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: player,
                amount: n,
                is_lose_hp: true,
                nature: rule_api::damage_nature::NORMAL,
            });
        }
    } else {
        // 重验所选为可弃手牌/装备
        let discardable = crate::query::discardable_target_cards(
            tx.state(),
            player,
            player,
            rule_api::TargetCardScope::HandEquip,
        );
        if !selected.iter().all(|c| discardable.contains(c)) {
            return Err(EffectError::Illegal("峻攻 discard changed".into()));
        }
        queue.push(EffectRequest::DiscardCards {
            player,
            cards: selected,
        });
    }
    run_effects(tx, queue)?;
    // 使用虚拟【杀】：先 materialize 卡，预注册 Sha 结算记录与技能绑定，
    // 再以普通结算队列使用命中目标时由 `owner_source_sha_damage_recorded_by_skill`
    // 触发禁用本主动技至回合结束。旧 JS 的 `tempBanSkill` 只在该杀造成伤害时触发。
    let card = crate::effect::materialize_virtual_card(tx, player, CardKind::Sha)?;
    tx.state_mut().resolution.register_sha_resolution(
        player,
        card,
        std::slice::from_ref(&target),
        false,
    );
    tx.state_mut()
        .resolution
        .record_sha_skill_target(player, card, skill, target)
        .then_some(())
        .ok_or_else(|| EffectError::Illegal("峻攻 Sha target disappeared".into()))?;
    crate::command::play::play_as_kind(
        tx,
        player,
        card,
        CardKind::Sha,
        std::slice::from_ref(&target),
        false,
        false,
        false,
        false,
    )?;
    Ok(())
}
