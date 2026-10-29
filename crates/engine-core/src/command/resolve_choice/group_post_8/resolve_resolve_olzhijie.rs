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
