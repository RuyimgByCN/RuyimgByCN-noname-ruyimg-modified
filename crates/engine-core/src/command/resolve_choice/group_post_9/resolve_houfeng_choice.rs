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

/// 通夺续体（ABI 442，payload[0]=RESOLVE_TONGDUO_CHOICE）：0 选角色 → 1 选重铸牌。
pub(crate) fn resolve_tongduo_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.last().unwrap_or(&0));
    match payload.get(1).copied() {
        Some(0) => {
            if payload.len() != 4 || observer != player || cancelled {
                return Ok(());
            }
            let [target_raw] = selections else {
                return Err(EffectError::Illegal("Tongduo requires one actor".into()));
            };
            let target = PlayerId::new(*target_raw);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("Tongduo actor no longer legal".into()));
            }
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolveTongduo {
                    player,
                    target,
                    card: CardId::NONE,
                    stage: 1,
                    skill,
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() != 5 || observer != PlayerId::new(payload[3]) {
                return Err(EffectError::Illegal(
                    "Tongduo card must be chosen by the actor".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let target = PlayerId::new(payload[3]);
            let [card_raw] = selections else {
                return Err(EffectError::Illegal("Tongduo requires one card".into()));
            };
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolveTongduo {
                    player,
                    target,
                    card: CardId::new(*card_raw),
                    stage: 2,
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("bad Tongduo continuation".into())),
    }
}

/// 断笔续体（ABI 445，payload[0]=RESOLVE_DUANBI_CHOICE）：
/// 0 逐目标弃牌 → 全部完成后开选人获牌。
/// payload: [MAGIC, 0, player, skill, target0, target1, ...]
pub(crate) fn resolve_duanbi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 {
        return Err(EffectError::EngineFault("bad Duanbi continuation".into()));
    }
    let player = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if payload[1] == 1 {
        // 获牌阶段：选一人权威随机获得至多 3 张仍处弃牌堆的被弃牌。
        if observer != player || cancelled {
            return Ok(());
        }
        let [recipient_raw] = selections else {
            return Err(EffectError::Illegal("Duanbi requires one recipient".into()));
        };
        let recipient = PlayerId::new(*recipient_raw);
        if !tx
            .state()
            .players
            .get(recipient)
            .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal(
                "Duanbi recipient no longer legal".into(),
            ));
        }
        let recorded: Vec<CardId> = tx
            .state()
            .skills
            .get(skill)
            .and_then(|instance| instance.state_blob.strip_prefix(b"DB1"))
            .map(|rest| {
                rest.chunks_exact(4)
                    .map(|chunk| {
                        CardId::new(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
        let still: Vec<CardId> = recorded
            .into_iter()
            .filter(|card| {
                discard.is_some_and(|zone| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|value| value.zone == zone)
                })
            })
            .collect();
        if still.is_empty() {
            return Ok(());
        }
        // 权威随机取至多 3 张。
        let take = still.len().min(3);
        let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
        let mut picked = Vec::new();
        let mut pool = still;
        for _ in 0..take {
            let index = rng.gen_range_u32(pool.len() as u32) as usize;
            picked.push(pool.swap_remove(index));
        }
        tx.state_mut().rng = rng.into_state();
        // 清空记录。
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            instance.state_blob.clear();
        }
        let to_hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, recipient)
            .ok_or_else(|| EffectError::Illegal("recipient hand missing".into()))?;
        return crate::effect::run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards: picked,
                to: to_hand,
                from: None,
            }],
        )
        .map(|_| ());
    }
    let targets: Vec<PlayerId> = payload[4..].iter().copied().map(PlayerId::new).collect();
    let current = targets.first().copied().unwrap_or(PlayerId::NONE);
    if observer != current || cancelled {
        return Err(EffectError::Illegal(
            "Duanbi discard must be chosen by the current target".into(),
        ));
    }
    let expected = (tx.state().hand_count(current) as usize / 2).min(3);
    if selections.len() != expected {
        return Err(EffectError::Illegal("Duanbi discard count mismatch".into()));
    }
    let cards: Vec<_> = selections.iter().map(|raw| CardId::new(*raw)).collect();
    let allowed = tx.state().hand_cards(current);
    if cards.iter().any(|card| !allowed.contains(card)) {
        return Err(EffectError::Illegal("Duanbi discard card invalid".into()));
    }
    // 记录被弃牌到 blob "DB1"（追加）。
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        let mut blob = instance
            .state_blob
            .strip_prefix(b"DB1")
            .map(|r| r.to_vec())
            .unwrap_or_default();
        for card in &cards {
            blob.extend_from_slice(&card.raw().to_le_bytes());
        }
        instance.state_blob = [b"DB1".as_slice(), &blob].concat();
    }
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player: current,
            cards,
        }],
    )?;
    let remaining = &targets[1..];
    if remaining.is_empty() {
        // 全部弃完：选一人获牌（可取消）。
        return crate::effect::open_choice(
            tx,
            player,
            1,
            1,
            true,
            tx.state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect(),
            engine_api::ChoiceKind::SelectTargets,
            crate::state::ContinuationKind::Custom,
            vec![
                crate::effect::RESOLVE_DUANBI_CHOICE,
                1,
                player.raw(),
                skill.raw(),
            ],
        )
        .map(|_| ());
    }
    // 下一目标。
    let next = remaining[0];
    let count = (tx.state().hand_count(next) as usize / 2).min(3);
    if count == 0 {
        return Ok(());
    }
    crate::effect::open_choice(
        tx,
        next,
        count as u8,
        count as u8,
        false,
        tx.state()
            .hand_cards(next)
            .into_iter()
            .map(CardId::raw)
            .collect(),
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::Custom,
        vec![
            crate::effect::RESOLVE_DUANBI_CHOICE,
            0,
            player.raw(),
            skill.raw(),
            next.raw(),
        ]
        .into_iter()
        .chain(remaining.iter().map(|t| t.raw()))
        .collect(),
    )
    .map(|_| ())
}

/// 躁厉续体（ABI 446，payload[0]=RESOLVE_ZAOLI_CHOICE）：弃至少一张牌 →
/// 摸（标记数+弃牌数）张，总数 > 2 失去 1 点体力。
pub(crate) fn resolve_zaoli_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 {
        return Err(EffectError::EngineFault("bad Zaoli continuation".into()));
    }
    let player = PlayerId::new(payload[2]);
    let mark = payload[3];
    let skill = engine_api::SkillId::new(payload[4]);
    if observer != player || cancelled || selections.is_empty() {
        return Ok(());
    }
    let cards: Vec<_> = selections.iter().map(|raw| CardId::new(*raw)).collect();
    let allowed = tx.state().hand_cards(player);
    if cards.iter().any(|card| !allowed.contains(card)) {
        return Err(EffectError::Illegal("Zaoli discard card invalid".into()));
    }
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player,
            cards: cards.clone(),
        }],
    )?;
    let mut effects = vec![EffectRequest::DrawCards {
        player,
        count: mark + cards.len() as u32,
    }];
    if mark + cards.len() as u32 > 2 {
        effects.push(EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: player,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        });
    }
    let _ = skill;
    crate::effect::run_effects(tx, effects).map(|_| ())
}

/// 孤影续体（ABI 447，payload[0]=RESOLVE_GUYING_CHOICE）：当前回合角色二选一。
pub(crate) fn resolve_guying_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 {
        return Err(EffectError::EngineFault("bad Guying continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let actor = PlayerId::new(payload[2]);
    let card = CardId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    if observer != actor || cancelled {
        return Ok(());
    }
    let [branch] = selections else {
        return Err(EffectError::Illegal("Guying requires one option".into()));
    };
    if !matches!(*branch, 0 | 1) {
        return Err(EffectError::Illegal("invalid Guying option".into()));
    }
    crate::effect::resolve_guying_commit(tx, player, actor, card, *branch).map(|_| ())
}

/// 佯解续体（ABI 430，payload[0]=RESOLVE_XINYANGJIE_CUSTOM_CHOICE，stage 在
/// payload[1]）：0 拥有者选拼点牌 → 1 目标选拼点牌 → 2 选火杀来源。
pub(crate) fn resolve_xinyangjie_choice(
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
                    "Xinyangjie card must be chosen by the owner".into(),
                ));
            }
            let [card_raw] = selections else {
                return Err(EffectError::Illegal("Xinyangjie requires one card".into()));
            };
            let card = CardId::new(*card_raw);
            if !tx.state().hand_cards(player).contains(&card)
                || tx.state().hand_cards(target).is_empty()
            {
                return Err(EffectError::Illegal(
                    "Xinyangjie comparison context changed".into(),
                ));
            }
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolveXinyangjie {
                    player,
                    target,
                    card,
                    target_card: CardId::NONE,
                    source: PlayerId::NONE,
                    stage: 1,
                    skill,
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() != 6 || observer != target || cancelled {
                return Err(EffectError::Illegal(
                    "Xinyangjie target card must be chosen by the target".into(),
                ));
            }
            let card = CardId::new(payload[4]);
            let [target_card_raw] = selections else {
                return Err(EffectError::Illegal(
                    "Xinyangjie target requires one card".into(),
                ));
            };
            let target_card = CardId::new(*target_card_raw);
            if !tx.state().hand_cards(player).contains(&card)
                || !tx.state().hand_cards(target).contains(&target_card)
            {
                return Err(EffectError::Illegal(
                    "Xinyangjie comparison cards left hand".into(),
                ));
            }
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolveXinyangjie {
                    player,
                    target,
                    card,
                    target_card,
                    source: PlayerId::NONE,
                    stage: 2,
                    skill,
                }],
            )
            .map(|_| ())
        }
        Some(2) => {
            if payload.len() != 5 || observer != player {
                return Err(EffectError::Illegal(
                    "bad Xinyangjie fire-sha choice".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let [source_raw] = selections else {
                return Err(EffectError::Illegal(
                    "Xinyangjie requires one fire-sha source".into(),
                ));
            };
            let source = PlayerId::new(*source_raw);
            if source == player
                || source == target
                || !tx
                    .state()
                    .players
                    .get(source)
                    .is_some_and(|participant| participant.alive)
                || !crate::effect::dcchiying_in_range(tx.state(), source, target)
            {
                return Err(EffectError::Illegal(
                    "Xinyangjie fire-sha source no longer legal".into(),
                ));
            }
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolveXinyangjie {
                    player,
                    target,
                    card: CardId::NONE,
                    target_card: CardId::NONE,
                    source,
                    stage: 4,
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "bad Xinyangjie continuation".into(),
        )),
    }
}

/// 压军续体（ABI 428，payload[0]=RESOLVE_SPYAJUN_CUSTOM_CHOICE，stage 在 payload[1]）。
/// stage 0 选新获牌 → 1 选拼点目标 → 2 目标选拼点牌 → 3 赢家可选置顶（0..1 张）。
/// 每个 stage 校验选择后重入 resolve_spyajun 的下一阶段。
pub(crate) fn resolve_spyajun_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let less_def = engine_api::SkillDefId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.last().unwrap_or(&0));
    match payload.get(1).copied() {
        Some(0) => {
            if payload.len() != 5 || observer != player || cancelled {
                return Ok(());
            }
            let [card_raw] = selections else {
                return Err(EffectError::Illegal("Spyajun requires one card".into()));
            };
            let card = CardId::new(*card_raw);
            if !tx.state().turn.gained_hand_card(player, card)
                || !tx.state().hand_cards(player).contains(&card)
            {
                return Err(EffectError::Illegal(
                    "Spyajun card not gained this turn".into(),
                ));
            }
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolveSpyajun {
                    player,
                    less_def,
                    card,
                    target: PlayerId::NONE,
                    target_card: CardId::NONE,
                    stage: 1,
                    skill,
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() != 6 || observer != player || cancelled {
                return Ok(());
            }
            let card = CardId::new(payload[4]);
            let [target_raw] = selections else {
                return Err(EffectError::Illegal("Spyajun requires one target".into()));
            };
            let target = PlayerId::new(*target_raw);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || tx.state().hand_cards(target).is_empty()
            {
                return Err(EffectError::Illegal(
                    "Spyajun target no longer legal".into(),
                ));
            }
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolveSpyajun {
                    player,
                    less_def,
                    card,
                    target,
                    target_card: CardId::NONE,
                    stage: 2,
                    skill,
                }],
            )
            .map(|_| ())
        }
        Some(2) => {
            if payload.len() != 7 {
                return Err(EffectError::EngineFault(
                    "bad Spyajun compare-card payload".into(),
                ));
            }
            let card = CardId::new(payload[4]);
            let target = PlayerId::new(payload[5]);
            if observer != target || cancelled {
                return Err(EffectError::Illegal(
                    "Spyajun compare card must be chosen by the target".into(),
                ));
            }
            let [target_card_raw] = selections else {
                return Err(EffectError::Illegal(
                    "Spyajun target requires one card".into(),
                ));
            };
            let target_card = CardId::new(*target_card_raw);
            if !tx.state().hand_cards(target).contains(&target_card)
                || !tx.state().hand_cards(player).contains(&card)
            {
                return Err(EffectError::Illegal(
                    "Spyajun comparison context changed".into(),
                ));
            }
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolveSpyajun {
                    player,
                    less_def,
                    card,
                    target,
                    target_card,
                    stage: 3,
                    skill,
                }],
            )
            .map(|_| ())
        }
        Some(3) => {
            if payload.len() != 6 || observer != player {
                return Err(EffectError::Illegal("bad Spyajun place-top choice".into()));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            let card = CardId::new(payload[4]);
            if selections != [card.raw()] {
                return Err(EffectError::Illegal(
                    "Spyajun place-top card mismatch".into(),
                ));
            }
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolveSpyajun {
                    player,
                    less_def,
                    card,
                    target: PlayerId::NONE,
                    target_card: CardId::NONE,
                    stage: 5,
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("bad Spyajun continuation".into())),
    }
}

/// 尊帝续体（ABI 429，payload[0]=RESOLVE_SPZUNDI_CUSTOM_CHOICE，stage 在 payload[1]）。
/// stage 0 目标选场上牌 → 1 选落点 → 2 提交移动。
pub(crate) fn resolve_spzundi_choice(
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
            if payload.len() != 5 || observer != target || cancelled {
                return Err(EffectError::Illegal(
                    "Spzundi card must be chosen by the target".into(),
                ));
            }
            let [card_raw] = selections else {
                return Err(EffectError::Illegal("Spzundi requires one card".into()));
            };
            let card = CardId::new(*card_raw);
            let movable = tx
                .state()
                .players
                .iter_live()
                .flat_map(|participant| {
                    crate::query::movable_field_cards_from(tx.state(), participant.id)
                })
                .any(|candidate| candidate == card);
            if !movable {
                return Err(EffectError::Illegal(
                    "Spzundi field card no longer legal".into(),
                ));
            }
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolveSpzundi {
                    player,
                    target,
                    card,
                    destination: PlayerId::NONE,
                    stage: 1,
                    skill,
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() != 6 || observer != target || cancelled {
                return Err(EffectError::Illegal(
                    "Spzundi destination must be chosen by the target".into(),
                ));
            }
            let card = CardId::new(payload[4]);
            let [destination_raw] = selections else {
                return Err(EffectError::Illegal(
                    "Spzundi requires one destination".into(),
                ));
            };
            let destination = PlayerId::new(*destination_raw);
            if !crate::query::field_card_move_destinations(tx.state(), card).contains(&destination)
            {
                return Err(EffectError::Illegal(
                    "Spzundi destination no longer legal".into(),
                ));
            }
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolveSpzundi {
                    player,
                    target,
                    card,
                    destination,
                    stage: 2,
                    skill,
                }],
            )
            .map(|_| ())
        }
        Some(2) => {
            if payload.len() != 6 || observer != target || cancelled {
                return Err(EffectError::Illegal("bad Spzundi commit".into()));
            }
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolveSpzundi {
                    player,
                    target,
                    card: CardId::new(payload[4]),
                    destination: PlayerId::new(payload[5]),
                    stage: 2,
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("bad Spzundi continuation".into())),
    }
}
