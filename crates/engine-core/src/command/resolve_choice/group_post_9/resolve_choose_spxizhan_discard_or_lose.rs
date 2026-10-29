//! Command operations for `resolve_choose_spxizhan_discard_or_lose` and related helpers.

use super::*;

pub(crate) fn resolve_choose_spxizhan_discard_or_lose(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad 嬉战 continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let phase_player = PlayerId::new(payload[1]);
    let fangzong_skill_def = SkillDefId::new(payload[2]);
    if observer != player
        || cancelled
        || selections.len() > 1
        || player == phase_player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "嬉战 discard-or-lose context changed".into(),
        ));
    }
    if selections.is_empty() {
        run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: player,
                amount: 1,
                is_lose_hp: true,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )?;
        return Ok(());
    }
    let card = CardId::new(selections[0]);
    let discardable = crate::query::discardable_target_cards(
        tx.state(),
        player,
        player,
        rule_api::TargetCardScope::HandEquip,
    );
    if !discardable.contains(&card) {
        return Err(EffectError::Illegal("嬉战 discard changed".into()));
    }
    let suit = tx
        .state()
        .cards
        .get(card)
        .map(|owned| crate::query::effective_card_suit(tx.state(), player, owned))
        .unwrap_or(engine_api::Suit::None);
    run_effects(
        tx,
        vec![
            EffectRequest::DiscardCards {
                player,
                cards: vec![card],
            },
            EffectRequest::DisableSkillByDefUntilTurnEnd {
                player,
                skill_def: fangzong_skill_def,
            },
        ],
    )?;
    apply_spxizhan_suit_use(tx, player, phase_player, suit)
}

/// 送枢：摸牌角色获得所选的仁库牌，并临时获得“不能指定其他角色”技能。
pub(crate) fn resolve_choose_spsongshu_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad 送枢 continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let phase_player = PlayerId::new(payload[1]);
    let renku_skill_def = SkillDefId::new(payload[2]);
    let block_skill_def = SkillDefId::new(payload[3]);
    if observer != phase_player
        || cancelled
        || player == phase_player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(phase_player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("送枢 choice context changed".into()));
    }
    let pile_cards = tx.state().skill_pile_cards(player, renku_skill_def.raw());
    let mut cards = Vec::new();
    for raw in selections {
        let card = CardId::new(*raw);
        if !pile_cards.contains(&card) {
            return Err(EffectError::Illegal("送枢 selection changed".into()));
        }
        cards.push(card);
    }
    if cards.is_empty() {
        return Ok(());
    }
    let hand_zone = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, phase_player)
        .ok_or_else(|| EffectError::EngineFault("missing hand zone".into()))?;
    let events = vec![
        EffectRequest::MoveCards {
            cards,
            to: hand_zone,
            from: None,
        },
        EffectRequest::AddSkillUntilTurnEnd {
            player: phase_player,
            skill_def: block_skill_def,
        },
    ];
    run_effects(tx, events)?;
    Ok(())
}

pub(crate) fn apply_spxizhan_suit_use(
    tx: &mut Transaction,
    player: PlayerId,
    phase_player: PlayerId,
    suit: engine_api::Suit,
) -> Result<(), EffectError> {
    let phase_alive = tx
        .state()
        .players
        .get(phase_player)
        .is_some_and(|participant| participant.alive);
    let owner_alive = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive && !participant.dying);
    // 旧 JS：目标不在场时仅红心【无中生有】仍结算。
    if !phase_alive && suit != engine_api::Suit::Heart {
        return Ok(());
    }
    match suit {
        engine_api::Suit::Spade => {
            if !phase_alive {
                return Ok(());
            }
            run_effects(
                tx,
                vec![EffectRequest::UseVirtualCardOn {
                    player: phase_player,
                    kind: CardKind::Jiu.tag(),
                    target: phase_player,
                }],
            )
            .map(|_| ())
        }
        engine_api::Suit::Heart => {
            if !owner_alive {
                return Ok(());
            }
            run_effects(
                tx,
                vec![EffectRequest::UseVirtualCardOn {
                    player,
                    kind: CardKind::TrickWuzhong.tag(),
                    target: player,
                }],
            )
            .map(|_| ())
        }
        engine_api::Suit::Club => {
            if !phase_alive || !owner_alive {
                return Ok(());
            }
            if !spxizhan_can_use_virtual_on(
                tx.state(),
                player,
                phase_player,
                CardKind::TrickTiesuo,
                false,
            ) {
                return Ok(());
            }
            run_effects(
                tx,
                vec![EffectRequest::UseVirtualCardOn {
                    player,
                    kind: CardKind::TrickTiesuo.tag(),
                    target: phase_player,
                }],
            )
            .map(|_| ())
        }
        engine_api::Suit::Diamond => {
            if !phase_alive || !owner_alive {
                return Ok(());
            }
            if !spxizhan_can_use_virtual_on(
                tx.state(),
                player,
                phase_player,
                CardKind::ShaFire,
                true,
            ) {
                return Ok(());
            }
            let fire_sha = crate::effect::materialize_virtual_card(tx, player, CardKind::ShaFire)?;
            match crate::command::play::play_as_kind(
                tx,
                player,
                fire_sha,
                CardKind::ShaFire,
                std::slice::from_ref(&phase_player),
                true,
                false,
                false,
                false,
            ) {
                Ok(()) => Ok(()),
                Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
                Err(EffectError::Illegal(_)) => Ok(()),
                Err(err) => Err(err),
            }
        }
        _ => Ok(()),
    }
}

pub(crate) fn spxizhan_can_use_virtual_on(
    state: &GameState,
    actor: PlayerId,
    target: PlayerId,
    kind: CardKind,
    ignore_distance: bool,
) -> bool {
    let Some(hand) = state.zones.find(ZoneKind::Hand, actor) else {
        return false;
    };
    let card = crate::state::Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::NONE,
        kind,
        suit: engine_api::Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: hand,
        owner: actor,
        face_up: true,
    };
    if crate::query::card_use_cancelled_for_logical_kind(state, actor, &card, kind)
        || crate::query::card_target_cancelled_for_logical_kind(state, actor, target, &card, kind)
        || !state
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return false;
    }
    if kind.is_sha() && !ignore_distance {
        return crate::query::sha_distance_unlimited_for(state, actor)
            || crate::query::player_in_attack_range(state, actor, target);
    }
    true
}

/// 霆威选择该杀目标阶段（ABI 410，payload[0]=RESOLVE_TINGWEI_SELECT）。
/// 霆威选择该杀目标阶段（ABI 410，payload[0]=RESOLVE_TINGWEI_SELECT）。
/// payload: [magic, player, card, ...sha_targets]；selections[0] = 目标 PlayerId。
/// 校验后打开目标四选一 choice（观察者=所选目标）。
pub(crate) fn resolve_tingwei_select(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 {
        return Err(EffectError::EngineFault(
            "bad Tingwei select payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let card = CardId::new(payload[2]);
    if observer != player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Tingwei observer mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    let [target_raw] = selections else {
        return Err(EffectError::Illegal("Tingwei requires one target".into()));
    };
    let target = PlayerId::new(*target_raw);
    let legal = payload
        .iter()
        .skip(3)
        .any(|candidate| PlayerId::new(*candidate) == target)
        && tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive);
    if !legal {
        return Err(EffectError::Illegal(
            "Tingwei target is no longer legal".into(),
        ));
    }
    crate::effect::open_choice(
        tx,
        target,
        1,
        1,
        true,
        vec![0, 1, 2, 3],
        engine_api::ChoiceKind::Custom,
        crate::state::ContinuationKind::Custom,
        vec![
            crate::effect::RESOLVE_TINGWEI_OPTION,
            player.raw(),
            target.raw(),
            card.raw(),
        ],
    )?;
    Err(EffectError::ChoiceRequested)
}

/// 霆威目标四选一阶段（ABI 410，payload[0]=RESOLVE_TINGWEI_OPTION）。
/// payload: [magic, player, target, card]；selections[0] ∈ 0..3：
/// 0=非锁定技失效至其回合结束，1=交给拥有者一张装备牌，2=此【杀】对其伤害+1，
/// 3=随机弃一张牌；取消则目标横置。
pub(crate) fn resolve_tingwei_option(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad Tingwei option payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let card = CardId::new(payload[3]);
    if observer != target
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Tingwei observer mismatch".into()));
    }
    if cancelled {
        return crate::effect::run_effects(
            tx,
            vec![EffectRequest::AddMark {
                player: target,
                mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
                delta: 1,
            }],
        )
        .map(|_| ());
    }
    let [option] = selections else {
        return Err(EffectError::Illegal("Tingwei requires one option".into()));
    };
    match *option {
        0 => {
            // 非锁定技失效至其下个回合结束。
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::SuppressNonLockedSkillsUntilTurnEnd { player: target }],
            )
            .map(|_| ())
        }
        1 => {
            // 交给拥有者一张装备牌（无则视为未选择）。
            let equip = crate::query::gainable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::Equip,
            )
            .into_iter()
            .next();
            let Some(card) = equip else {
                return Ok(());
            };
            let to_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("Tingwei equip missing".into()))?;
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to: to_hand,
                    from: Some(from),
                }],
            )
            .map(|_| ())
        }
        2 => {
            // 此【杀】对其伤害+1。
            if !tx.state().cards.get(card).is_some_and(|value| {
                crate::query::effective_card_kind(tx.state(), player, value).is_sha()
            }) {
                return Ok(());
            }
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::IncreaseCurrentShaTargetDamage {
                    source: player,
                    card,
                    target,
                    amount: 1,
                }],
            )
            .map(|_| ())
        }
        3 => {
            // 随机弃一张牌（权威 RNG）。
            let discardable = crate::query::discardable_hand_equip_cards(tx.state(), target);
            let Some(selected) = (if discardable.is_empty() {
                None
            } else {
                let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
                let index = rng.next_u64() as usize % discardable.len();
                tx.state_mut().rng = rng.into_state();
                Some(discardable[index])
            }) else {
                return Ok(());
            };
            crate::effect::discard_cards(tx, target, vec![selected]).map(|_| ())
        }
        _ => Err(EffectError::Illegal("unknown Tingwei option".into())),
    }
}

/// 新名士弃牌结算（ABI 411，payload[0]=RESOLVE_XINMINGSHI_CUSTOM_CHOICE）。
/// payload: [magic, player, skill]；selections[0] = 来源弃置的 hej 牌。
/// 红色弃牌 → 拥有者回复 1；弃牌进入弃牌堆 → 拥有者获得。
pub(crate) fn resolve_xinmingshi_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad Xinmingshi continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    if cancelled {
        return Ok(());
    }
    let [card_raw] = selections else {
        return Err(EffectError::Illegal(
            "Xinmingshi requires one discarded card".into(),
        ));
    };
    let card = CardId::new(*card_raw);
    let legal = crate::query::gainable_target_cards(
        tx.state(),
        observer,
        observer,
        rule_api::TargetCardScope::HandEquipJudge,
    )
    .contains(&card);
    if !legal {
        return Err(EffectError::Illegal(
            "Xinmingshi discarded card invalid".into(),
        ));
    }
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("Xinmingshi card missing".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("discard pile missing".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: discard,
            from: Some(from),
        }],
    )?;
    let discarded = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.suit)
        .unwrap_or(engine_api::Suit::None);
    let is_red = matches!(
        discarded,
        engine_api::Suit::Heart | engine_api::Suit::Diamond
    );
    if is_red {
        crate::effect::run_effects(
            tx,
            vec![EffectRequest::RecoverHp {
                source: PlayerId::NONE,
                player,
                amount: 1,
            }],
        )?;
    } else if tx
        .state()
        .zones
        .get(discard)
        .is_some_and(|zone| zone.cards.contains(&card))
    {
        let to_hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
        crate::effect::run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards: vec![card],
                to: to_hand,
                from: Some(discard),
            }],
        )?;
    }
    Ok(())
}

/// 殉义选目标结算（ABI 412，payload[0]=RESOLVE_XUNYI_SELECT）。
/// payload: [magic, player, skill]；selections[0] = 义目标 PlayerId。
/// 目标获得“义”标记。
pub(crate) fn resolve_xunyi_select(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    const YI_MARK: u32 = 2_517_009_871;
    if payload.len() != 3 {
        return Err(EffectError::EngineFault("bad Xunyi select payload".into()));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player || cancelled {
        return Ok(());
    }
    let [target_raw] = selections else {
        return Err(EffectError::Illegal("Xunyi requires one target".into()));
    };
    let target = PlayerId::new(*target_raw);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Xunyi target invalid".into()));
    }
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: engine_api::MarkDefId::new(YI_MARK),
            delta: 1,
        }],
    )
    .map(|_| ())
}

/// 恶荐二选一结算（ABI 414，payload[0]=RESOLVE_EJIAN_CHOICE）。
/// payload: [magic, player, target, card]；selections[0] ∈ {0=受 1 点伤害, 1=展示弃同类}。
pub(crate) fn resolve_ejian_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault("bad Ejian continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let card = CardId::new(payload[3]);
    if observer != target || cancelled {
        return Ok(());
    }
    let [option] = selections else {
        return Err(EffectError::Illegal("Ejian requires one option".into()));
    };
    match *option {
        0 => crate::effect::run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: player,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .map(|_| ()),
        1 => {
            // 展示手牌并弃所有与赠牌同类的牌。
            let category = crate::effect::card_category_of(tx.state(), card);
            let discardable: Vec<_> = [ZoneKind::Hand, ZoneKind::Equip]
                .into_iter()
                .filter_map(|kind| tx.state().zones.find(kind, target))
                .filter_map(|zone| tx.state().zones.get(zone))
                .flat_map(|zone| zone.cards.iter().copied())
                .filter(|candidate| {
                    *candidate != card
                        && crate::effect::card_category_of(tx.state(), *candidate) == category
                })
                .collect();
            if discardable.is_empty() {
                return Ok(());
            }
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::RevealCards {
                    cards: discardable.clone(),
                    to: Vec::new(),
                }],
            )?;
            crate::effect::discard_cards(tx, target, discardable).map(|_| ())
        }
        _ => Err(EffectError::Illegal("unknown Ejian option".into())),
    }
}

/// 舍医选牌防伤结算（ABI 417，payload[0]=RESOLVE_SHEYI_CHOICE）。
/// payload: [magic, player, target, skill]；selections = 交给目标的牌。
/// 赠牌后防止当前伤害。
pub(crate) fn resolve_sheyi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault("bad Sheyi continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    if observer != player || cancelled || selections.is_empty() {
        return Ok(());
    }
    let cards: Vec<_> = selections.iter().map(|raw| CardId::new(*raw)).collect();
    let allowed = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if cards.iter().any(|card| !allowed.contains(card)) {
        return Err(EffectError::Illegal("Sheyi gift card invalid".into()));
    }
    let to_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("recipient hand missing".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCardsWithReason {
            cards,
            to: to_hand,
            from: None,
            reason: engine_api::CardMoveReason::Give,
        }],
    )?;
    let damage_id = tx.state().resolution.current_damage_id;
    if damage_id != 0 {
        crate::effect::run_effects(tx, vec![EffectRequest::PreventDamage { damage_id }])?;
    }
    Ok(())
}
