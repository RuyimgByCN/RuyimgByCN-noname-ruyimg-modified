//! Command operations for `resolve_houfeng_choice` and related helpers.

use super::*;

/// 侯封 / 燕集续体（ABI 433，payload[0]=RESOLVE_HOUFENG_CHOICE）：
/// 选中整肃类型（1 垒金 / 2 变阵 / 3 明志）后重入 resolve_houfeng 提交授予。
pub(crate) fn resolve_houfeng_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 {
        return Err(EffectError::EngineFault("bad Houfeng continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let mode = payload[3] as u8;
    let skill = engine_api::SkillId::new(payload[4]);
    if observer != player || cancelled {
        return Ok(());
    }
    let [choice_raw] = selections else {
        return Err(EffectError::Illegal(
            "Houfeng requires one zhengsu type".into(),
        ));
    };
    if !matches!(*choice_raw, 1 | 2 | 3) {
        return Err(EffectError::Illegal("invalid Houfeng zhengsu type".into()));
    }
    // 选项 1 垒金 / 2 变阵 / 3 明志 → resolve_houfeng 内部 mode 0/1/2。
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::ResolveHoufeng {
            player,
            target,
            mode: (*choice_raw - 1) as u8,
            stage: 1,
            skill,
        }],
    )
    .map(|_| ())
}

/// 战海续体（ABI 435，payload[0]=RESOLVE_RECHUHAI_CHOICE，stage 在 payload[1]）：
/// 0 拥有者选拼点牌 → 1 目标选拼点牌。
pub(crate) fn resolve_rechuhai_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let target = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.last().unwrap_or(&0));
    match payload.get(1).copied() {
        Some(0) => {
            if payload.len() != 5 || observer != player || cancelled {
                return Err(EffectError::Illegal(
                    "Rechuhai card must be chosen by the owner".into(),
                ));
            }
            let [card_raw] = selections else {
                return Err(EffectError::Illegal("Rechuhai requires one card".into()));
            };
            let card = CardId::new(*card_raw);
            if !tx.state().hand_cards(player).contains(&card)
                || tx.state().hand_cards(target).is_empty()
            {
                return Err(EffectError::Illegal(
                    "Rechuhai comparison context changed".into(),
                ));
            }
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolveRechuhai {
                    player,
                    target,
                    card,
                    target_card: CardId::NONE,
                    stage: 1,
                    skill,
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() != 6 || observer != target || cancelled {
                return Err(EffectError::Illegal(
                    "Rechuhai target card must be chosen by the target".into(),
                ));
            }
            let card = CardId::new(payload[4]);
            let [target_card_raw] = selections else {
                return Err(EffectError::Illegal(
                    "Rechuhai target requires one card".into(),
                ));
            };
            let target_card = CardId::new(*target_card_raw);
            if !tx.state().hand_cards(player).contains(&card)
                || !tx.state().hand_cards(target).contains(&target_card)
            {
                return Err(EffectError::Illegal(
                    "Rechuhai comparison cards left hand".into(),
                ));
            }
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolveRechuhai {
                    player,
                    target,
                    card,
                    target_card,
                    stage: 2,
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("bad Rechuhai continuation".into())),
    }
}

/// 凶竖续体（ABI 436，payload[0]=RESOLVE_XIONGSHU_CHOICE，stage 在 payload[1]）：
/// 1 选展示牌 → 2 预测。
pub(crate) fn resolve_xiongshu_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let target = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.last().unwrap_or(&0));
    match payload.get(1).copied() {
        Some(0) => {
            if payload.len() != 6 || observer != player || cancelled {
                return Err(EffectError::Illegal(
                    "Xiongshu discard must be chosen by the owner".into(),
                ));
            }
            let mark = payload[4] as u32;
            if selections.len() as u32 != mark || mark == 0 {
                return Err(EffectError::Illegal(
                    "Xiongshu discard count mismatch".into(),
                ));
            }
            let cards: Vec<_> = selections.iter().map(|raw| CardId::new(*raw)).collect();
            let allowed = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if cards.iter().any(|card| !allowed.contains(card)) {
                return Err(EffectError::Illegal("Xiongshu discard card invalid".into()));
            }
            crate::effect::run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards { player, cards },
                    EffectRequest::AddMark {
                        player,
                        mark: engine_api::MarkDefId::new(crate::effect::XIONGSHU_MARK),
                        delta: 1,
                    },
                    EffectRequest::ResolveXiongshu {
                        player,
                        target,
                        card: CardId::NONE,
                        action: 0,
                        stage: 1,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() != 5 || observer != player || cancelled {
                return Err(EffectError::Illegal(
                    "Xiongshu shown card must be chosen by the owner".into(),
                ));
            }
            let [card_raw] = selections else {
                return Err(EffectError::Illegal("Xiongshu requires one card".into()));
            };
            let card = CardId::new(*card_raw);
            if !tx.state().hand_cards(target).contains(&card) {
                return Err(EffectError::Illegal(
                    "Xiongshu shown card left target hand".into(),
                ));
            }
            // 展示所选牌。
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::RevealCards {
                    cards: vec![card],
                    to: Vec::new(),
                }],
            )?;
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolveXiongshu {
                    player,
                    target,
                    card,
                    action: 0,
                    stage: 2,
                    skill,
                }],
            )
            .map(|_| ())
        }
        Some(2) => {
            if payload.len() != 6 || observer != player {
                return Err(EffectError::Illegal("bad Xiongshu prediction".into()));
            }
            if cancelled {
                return Ok(());
            }
            let card = CardId::new(payload[4]);
            let [action_raw] = selections else {
                return Err(EffectError::Illegal(
                    "Xiongshu requires one prediction".into(),
                ));
            };
            if !matches!(*action_raw, 0 | 1) {
                return Err(EffectError::Illegal("invalid Xiongshu prediction".into()));
            }
            // 记录预测状态 blob "XS1" + target + card + action。
            let mut blob = Vec::with_capacity(13);
            blob.extend_from_slice(b"XS1");
            blob.extend_from_slice(&target.raw().to_le_bytes());
            blob.extend_from_slice(&card.raw().to_le_bytes());
            blob.push(*action_raw as u8);
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob = blob;
            }
            Ok(())
        }
        _ => Err(EffectError::EngineFault("bad Xiongshu continuation".into())),
    }
}

/// 弃哀续体（ABI 439，payload[0]=RESOLVE_SPQIAI_CHOICE）：目标二选一——
/// 0 令拥有者回复 1 / 1 令拥有者摸两张。
pub(crate) fn resolve_spqiai_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault("bad Spqiai continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if observer != target || cancelled {
        return Ok(());
    }
    let [choice] = selections else {
        return Err(EffectError::Illegal("Spqiai requires one option".into()));
    };
    if !matches!(*choice, 0 | 1) {
        return Err(EffectError::Illegal("invalid Spqiai option".into()));
    }
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::ResolveSpqiai {
            player,
            target,
            card: CardId::NONE,
            stage: 1,
            skill,
        }],
    )
    .map(|_| ())
}

/// 擅袭续体（ABI 440，payload[0]=RESOLVE_SPSHANXI_CHOICE）：选中目标后提交标记。
pub(crate) fn resolve_spshanxi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault("bad Spshanxi continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    if observer != player || cancelled {
        return Ok(());
    }
    let [target_raw] = selections else {
        return Err(EffectError::Illegal("Spshanxi requires one target".into()));
    };
    let target = PlayerId::new(*target_raw);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Spshanxi target no longer legal".into(),
        ));
    }
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::ResolveSpshanxi {
            player,
            target,
            stage: 1,
            skill,
        }],
    )
    .map(|_| ())
}

/// 索命续体（ABI 441，payload[0]=RESOLVE_SUOMING_CHOICE）：选中 2 张牌记录后提交；
/// 取消 → 失去 1 点体力。
pub(crate) fn resolve_suoming_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault("bad Suoming continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let actor = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if observer != actor {
        return Err(EffectError::Illegal(
            "Suoming cards must be chosen by the actor".into(),
        ));
    }
    if cancelled {
        // 未交牌 → 失去 1 点体力。
        return crate::effect::run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: actor,
                amount: 1,
                is_lose_hp: true,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .map(|_| ());
    }
    let cards: Vec<_> = selections.iter().map(|raw| CardId::new(*raw)).collect();
    let allowed = crate::query::discardable_hand_equip_cards(tx.state(), actor);
    if cards.len() != 2 || cards.iter().any(|card| !allowed.contains(card)) {
        return Err(EffectError::Illegal("Suoming gift card invalid".into()));
    }
    // 记录所选牌到 blob "SM1"。
    let mut blob = Vec::with_capacity(8);
    for card in &cards {
        blob.extend_from_slice(&card.raw().to_le_bytes());
    }
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = [b"SM1".as_slice(), &blob].concat();
    }
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::ResolveSpshanxiSuoming {
            player,
            actor,
            stage: 1,
            skill,
        }],
    )
    .map(|_| ())
}
