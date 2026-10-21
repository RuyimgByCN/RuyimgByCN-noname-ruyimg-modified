//! Command operations for `resolve_resolve_olzhijie` and related helpers.

use super::*;

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
                vec![0x5A4E_0001, 2, player.raw(), cost.raw()],
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
        _ => Err(EffectError::EngineFault(
            "bad resolve-mbzhenann stage".into(),
        )),
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
        _ => Err(EffectError::EngineFault(
            "bad resolve-dcqiaolve stage".into(),
        )),
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
        _ => Err(EffectError::EngineFault(
            "bad resolve-qiangyong stage".into(),
        )),
    }
}

pub(crate) fn chulao_open_target_pick(
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
            let target = selections
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("resolve-chumei target required".into()))?;
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
            let choice = *selections
                .first()
                .ok_or_else(|| EffectError::Illegal("resolve-chumei option required".into()))?;
            match choice {
                1 => {
                    crate::effect::draw_cards(tx, target, 1)?;
                    Ok(())
                }
                2 => {
                    crate::effect::resolve_chumei_give_pick(tx, player, skill, target_raw)?;
                    Ok(())
                }
                3 => {
                    crate::effect::recover_hp(tx, player, target, 1)?;
                    Ok(())
                }
                _ => Err(EffectError::Illegal("resolve-chumei unknown option".into())),
            }
        }
        4 => {
            let target = PlayerId::new(payload[4]);
            if observer != target {
                return Err(EffectError::Illegal(
                    "resolve-chumei give requires the target".into(),
                ));
            }
            let card = selections
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("resolve-chumei give card required".into()))?;
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
            let target = selections
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("resolve-zhongyan target required".into()))?;
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
            let k = *payload
                .get(5)
                .ok_or_else(|| EffectError::EngineFault("bad resolve-zhongyan payload".into()))?
                as usize;
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
            crate::effect::resolve_zhongyan_pick(tx, player, skill, target_raw, &revealed, pick)?;
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
            let k = *payload
                .get(5)
                .ok_or_else(|| EffectError::EngineFault("bad resolve-zhongyan payload".into()))?
                as usize;
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
        _ => Err(EffectError::EngineFault(
            "bad resolve-zhongyan stage".into(),
        )),
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
            crate::effect::resolve_dcmoukui_conseq_discard(tx, player, skill, dodger, card)?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault(
            "bad resolve-dcmoukui stage".into(),
        )),
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
