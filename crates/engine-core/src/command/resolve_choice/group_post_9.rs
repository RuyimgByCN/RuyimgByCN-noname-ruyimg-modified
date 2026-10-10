//! Choice-continuation handlers (post 9).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

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

/// 举讹确认/选目标结算（ABI 583）。
/// 自己的结束阶段 payload=[magic, player, skill]；其他角色的结束阶段
/// payload=[magic, player, fixed_target, skill]。
pub(crate) fn resolve_dcjue_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if !matches!(payload.len(), 3 | 4) {
        return Err(EffectError::EngineFault("bad dcjue continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player || cancelled || (payload.len() == 3 && selections.is_empty()) {
        return Ok(());
    }
    let (target, skill) = if payload.len() == 3 {
        let [selected] = selections else {
            return Err(EffectError::Illegal("dcjue requires one target".into()));
        };
        let target = PlayerId::new(*selected);
        if target == player
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal("invalid dcjue target".into()));
        }
        (target, engine_api::SkillId::new(payload[2]))
    } else {
        if selections == [0] {
            return Ok(());
        }
        if selections != [1] {
            return Err(EffectError::Illegal("invalid dcjue confirmation".into()));
        }
        (
            PlayerId::new(payload[2]),
            engine_api::SkillId::new(payload[3]),
        )
    };
    run_effects(
        tx,
        vec![EffectRequest::ResolveDcjue {
            player,
            target,
            stage: 1,
            remaining: 0,
            skill,
        }],
    )
    .map(|_| ())
}

/// 阮籍「酒遁」上下文选择（ABI 584）。mode 0 确认摸牌饮酒；mode 1 可取消弃手牌并
/// 在弃置触发完成后令当前黑牌对自己无效。
pub(crate) fn resolve_ruanji_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [_, player, source, card, skill, mode] = payload else {
        return Err(EffectError::EngineFault("bad Ruanji continuation".into()));
    };
    let player = PlayerId::new(*player);
    let source = PlayerId::new(*source);
    let card = CardId::new(*card);
    let skill = engine_api::SkillId::new(*skill);
    if observer != player || cancelled {
        return Ok(());
    }
    match *mode {
        0 => {
            if selections == [0] {
                return Ok(());
            }
            if selections != [1] {
                return Err(EffectError::Illegal("invalid 酒遁 confirmation".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DrawCards { player, count: 1 },
                    EffectRequest::UseVirtualCardOn {
                        player,
                        kind: CardKind::Jiu.tag(),
                        target: player,
                    },
                ],
            )?;
        }
        1 => {
            let [selected] = selections else {
                return Ok(());
            };
            let selected = CardId::new(*selected);
            let legal = crate::query::discardable_hand_equip_cards(tx.state(), player)
                .into_iter()
                .any(|candidate| {
                    candidate == selected
                        && tx.state().cards.get(candidate).is_some_and(|card| {
                            tx.state()
                                .zones
                                .get(card.zone)
                                .is_some_and(|zone| zone.kind == ZoneKind::Hand)
                        })
                });
            if !legal {
                return Err(EffectError::Illegal(
                    "酒遁 discard card became illegal".into(),
                ));
            }
            let from = tx
                .state()
                .cards
                .get(selected)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("酒遁 discard card disappeared".into()))?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCardsWithReason {
                        cards: vec![selected],
                        to: discard,
                        from: Some(from),
                        reason: CardMoveReason::Discard,
                    },
                    EffectRequest::ResolveRuanji {
                        player,
                        source,
                        card,
                        action: 4,
                        cards: Vec::new(),
                        skill,
                    },
                ],
            )?;
        }
        _ => return Err(EffectError::EngineFault("bad Ruanji choice mode".into())),
    }
    Ok(())
}

/// 崔琰毛玠「征辟」选择续体（ABI 585）。
pub(crate) fn resolve_zhengbi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload[0] != crate::effect::RESOLVE_ZHENGBI_CHOICE || cancelled {
        return Ok(());
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(if stage == 0 { payload[3] } else { payload[4] });
    match stage {
        0 => {
            if observer != player {
                return Err(EffectError::Illegal("征辟目标选择者不匹配".into()));
            }
            let [target] = selections else {
                return Err(EffectError::Illegal("征辟需要一名目标".into()));
            };
            let target = PlayerId::new(*target);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("征辟目标已失效".into()));
            }
            let mut branches = vec![0];
            if tx.state().hand_cards(player).into_iter().any(|card| {
                tx.state().cards.get(card).is_some_and(|card| {
                    crate::query::effective_card_kind(tx.state(), player, card).is_basic()
                })
            }) {
                branches.push(1);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                branches,
                ChoiceKind::Confirm,
                ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_ZHENGBI_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )?;
        }
        1 => {
            if payload.len() != 5 || observer != player {
                return Err(EffectError::Illegal("征辟分支选择者不匹配".into()));
            }
            let target = PlayerId::new(payload[3]);
            let [branch] = selections else {
                return Err(EffectError::Illegal("征辟需要选择一项".into()));
            };
            match *branch {
                0 => {
                    run_effects(
                        tx,
                        vec![EffectRequest::ResolveZhengbi {
                            player,
                            target,
                            card: CardId::NONE,
                            action: 4,
                            skill,
                        }],
                    )?;
                }
                1 => {
                    let options: Vec<_> = tx
                        .state()
                        .hand_cards(player)
                        .into_iter()
                        .filter(|card| {
                            tx.state().cards.get(*card).is_some_and(|card| {
                                crate::query::effective_card_kind(tx.state(), player, card)
                                    .is_basic()
                            })
                        })
                        .map(CardId::raw)
                        .collect();
                    if options.is_empty() {
                        return Err(EffectError::Illegal("征辟已无可交出的基本牌".into()));
                    }
                    open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        options,
                        ChoiceKind::SelectCards,
                        ContinuationKind::Custom,
                        vec![
                            crate::effect::RESOLVE_ZHENGBI_CHOICE,
                            2,
                            player.raw(),
                            target.raw(),
                            skill.raw(),
                        ],
                    )?;
                }
                _ => return Err(EffectError::Illegal("征辟分支无效".into())),
            }
        }
        2 => {
            if payload.len() != 5 || observer != player {
                return Err(EffectError::Illegal("征辟赠牌选择者不匹配".into()));
            }
            let [card] = selections else {
                return Err(EffectError::Illegal("征辟需要交出一张基本牌".into()));
            };
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhengbi {
                    player,
                    target: PlayerId::new(payload[3]),
                    card: CardId::new(*card),
                    action: 2,
                    skill,
                }],
            )?;
        }
        3 => {
            if payload.len() != 5 {
                return Err(EffectError::EngineFault("征辟回礼分支数据损坏".into()));
            }
            let target = PlayerId::new(payload[3]);
            if observer != target {
                return Err(EffectError::Illegal("征辟回礼选择者不匹配".into()));
            }
            let [mode @ 0..=1] = selections else {
                return Err(EffectError::Illegal("征辟回礼分支无效".into()));
            };
            let hand = tx.state().zones.find(ZoneKind::Hand, target);
            let options: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), target)
                .into_iter()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card| {
                        let basic =
                            crate::query::effective_card_kind(tx.state(), target, card).is_basic();
                        if *mode == 0 {
                            !basic
                        } else {
                            basic && Some(card.zone) == hand
                        }
                    })
                })
                .map(CardId::raw)
                .collect();
            let count = if *mode == 0 { 1 } else { 2 };
            if options.len() < count as usize {
                return Err(EffectError::Illegal("征辟回礼牌不足".into()));
            }
            open_choice(
                tx,
                target,
                count,
                count,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_ZHENGBI_CHOICE,
                    4,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    *mode,
                ],
            )?;
        }
        4 => {
            if payload.len() != 6 {
                return Err(EffectError::EngineFault("征辟回礼数据损坏".into()));
            }
            let target = PlayerId::new(payload[3]);
            if observer != target {
                return Err(EffectError::Illegal("征辟回礼选择者不匹配".into()));
            }
            let mode = payload[5];
            let expected = if mode == 0 { 1 } else { 2 };
            if selections.len() != expected {
                return Err(EffectError::Illegal("征辟回礼数量无效".into()));
            }
            let hand = tx.state().zones.find(ZoneKind::Hand, target);
            let allowed: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), target)
                .into_iter()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card| {
                        let basic =
                            crate::query::effective_card_kind(tx.state(), target, card).is_basic();
                        if mode == 0 {
                            !basic
                        } else {
                            basic && Some(card.zone) == hand
                        }
                    })
                })
                .collect();
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            if cards.iter().any(|card| !allowed.contains(card)) {
                return Err(EffectError::Illegal("征辟回礼牌已失效".into()));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("征辟拥有者手牌区不存在".into()))?;
            let from = tx
                .state()
                .cards
                .get(cards[0])
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("征辟回礼牌不存在".into()))?;
            if cards.iter().any(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_none_or(|card| card.zone != from)
            }) {
                return Err(EffectError::Illegal("征辟回礼牌不在同一区域".into()));
            }
            run_effects(tx, vec![EffectRequest::GiveCards { cards, to, from }])?;
        }
        5 => {
            if payload.len() != 5 || observer != player {
                return Err(EffectError::Illegal("征辟获牌选择者不匹配".into()));
            }
            let target = PlayerId::new(payload[3]);
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            let allowed = crate::query::gainable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            if cards.is_empty() || cards.iter().any(|card| !allowed.contains(card)) {
                return Err(EffectError::Illegal("征辟所选牌已失效".into()));
            }
            let mut zones = std::collections::BTreeSet::new();
            for card in &cards {
                let zone = tx
                    .state()
                    .cards
                    .get(*card)
                    .map(|card| card.zone)
                    .ok_or_else(|| EffectError::Illegal("征辟所选牌不存在".into()))?;
                if !zones.insert(zone) {
                    return Err(EffectError::Illegal("征辟每个区域至多获得一张牌".into()));
                }
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("征辟拥有者手牌区不存在".into()))?;
            let effects = cards
                .into_iter()
                .map(|card| EffectRequest::MoveCards {
                    cards: vec![card],
                    to,
                    from: tx.state().cards.get(card).map(|card| card.zone),
                })
                .collect();
            run_effects(tx, effects)?;
        }
        _ => return Err(EffectError::EngineFault("未知征辟选择阶段".into())),
    }
    Ok(())
}

/// 追锋选目标结算（ABI 419，payload[0]=RESOLVE_DBZHUIFENG_TARGET）。
/// payload: [magic, player, skill]；selections[0] = 决斗目标。
/// 对目标视为使用虚拟【决斗】。
pub(crate) fn resolve_dbzhuifeng_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad Dbzhuifeng continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player || cancelled {
        return Ok(());
    }
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "Dbzhuifeng requires one target".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Dbzhuifeng target invalid".into()));
    }
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::UseVirtualCardOn {
            player,
            kind: u8::from(crate::state::CardKind::TrickJuedou.tag()),
            target,
        }],
    )
    .map(|_| ())
}

/// 曲迪三选一结算（ABI 420，payload[0]=RESOLVE_DBQUEDI_CHOICE）。
/// payload: [magic, player, target, card, skill]；selections[0] ∈ {0=获牌, 1=弃基本加伤, 2=背水}。
pub(crate) fn resolve_dbquedi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 {
        return Err(EffectError::EngineFault("bad Dbquedi continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let card = CardId::new(payload[3]);
    if observer != player || cancelled {
        return Ok(());
    }
    let [option] = selections else {
        return Err(EffectError::Illegal("Dbquedi requires one option".into()));
    };
    let backwater = *option == 2;
    let do_gain = *option == 0 || backwater;
    let do_discard_boost = *option == 1 || backwater;
    if backwater {
        crate::effect::run_effects(tx, vec![EffectRequest::ChangeMaxHp { player, delta: -1 }])?;
    }
    if do_gain {
        let options = crate::query::gainable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::Hand,
        );
        if let Some(gain) = options.into_iter().next() {
            let from = tx
                .state()
                .cards
                .get(gain)
                .map(|c| c.zone)
                .ok_or_else(|| EffectError::Illegal("Dbquedi card missing".into()))?;
            let to_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![gain],
                    to: to_hand,
                    from: Some(from),
                }],
            )?;
        }
    }
    if do_discard_boost {
        let basic = tx.state().hand_cards(player).into_iter().find(|candidate| {
            tx.state()
                .cards
                .get(*candidate)
                .is_some_and(|value| value.kind.is_basic())
        });
        if let Some(basic) = basic {
            crate::effect::discard_cards(tx, player, vec![basic])?;
            // 此牌对目标伤害 +1。
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::IncreaseCurrentCardBaseDamage {
                    player,
                    card,
                    amount: 1,
                    skill: engine_api::SkillId::NONE,
                }],
            )?;
        }
    }
    Ok(())
}

/// 隐据二选一结算（ABI 421，payload[0]=RESOLVE_SPYINJU_CHOICE）。
/// payload: [magic, player, target, skill]；selections[0] ∈ {0=用杀, 1=跳过}。
pub(crate) fn resolve_spyinju_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    const SKIP_MARK: u32 = 2_728_392_676;
    if payload.len() != 4 {
        return Err(EffectError::EngineFault("bad Spyinju continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    if observer != target || cancelled {
        return Ok(());
    }
    let [option] = selections else {
        return Err(EffectError::Illegal("Spyinju requires one option".into()));
    };
    match *option {
        0 => {
            // 对拥有者使用一张手牌【杀】（有杀时；无则视为跳过）。
            let sha = tx.state().hand_cards(target).into_iter().find(|candidate| {
                tx.state()
                    .cards
                    .get(*candidate)
                    .is_some_and(|value| value.kind.is_sha())
            });
            let Some(sha) = sha else {
                return Ok(());
            };
            let r = crate::effect::run_effects(
                tx,
                vec![EffectRequest::UseCard {
                    player: target,
                    card: sha,
                    targets: vec![player],
                }],
            );
            r.map(|_| ())
        }
        1 => crate::effect::run_effects(
            tx,
            vec![EffectRequest::AddMark {
                player: target,
                mark: engine_api::MarkDefId::new(SKIP_MARK),
                delta: 1,
            }],
        )
        .map(|_| ()),
        _ => Err(EffectError::Illegal("unknown Spyinju option".into())),
    }
}

/// 上议选牌结算（ABI 423，payload[0]=RESOLVE_SPSHANGYI_CHOICE）。
/// payload: [magic, player, target, skill]；selections[0] = 获得的目标手牌。
pub(crate) fn resolve_spshangyi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad Spshangyi continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    if observer != player || cancelled {
        return Ok(());
    }
    let [card_raw] = selections else {
        return Err(EffectError::Illegal("Spshangyi requires one card".into()));
    };
    let card = CardId::new(*card_raw);
    if !tx.state().hand_cards(target).contains(&card) {
        return Err(EffectError::Illegal(
            "Spshangyi card not in target hand".into(),
        ));
    }
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|c| c.zone)
        .ok_or_else(|| EffectError::Illegal("Spshangyi card missing".into()))?;
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
            from: Some(from),
        }],
    )
    .map(|_| ())
}

/// 合击二选一结算（ABI 427，payload[0]=RESOLVE_HEJI_CHOICE）。
/// payload: [magic, player, target, skill]；selections[0] ∈ {0=杀, 1=决斗}。
pub(crate) fn resolve_heji_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault("bad Heji continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    if observer != player || cancelled {
        return Ok(());
    }
    let [option] = selections else {
        return Err(EffectError::Illegal("Heji requires one option".into()));
    };
    let kind = if *option == 0 {
        crate::state::CardKind::Sha
    } else {
        crate::state::CardKind::TrickJuedou
    };
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::UseVirtualCardOn {
            player,
            kind: u8::from(kind.tag()),
            target,
        }],
    )
    .map(|_| ())
}

/// 清雨防伤弃牌续体（ABI 431，payload[0]=RESOLVE_QINGYU_CUSTOM_CHOICE）：
/// 弃置所选 2 张 he 并防止当前伤害。
pub(crate) fn resolve_qingyu_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault("bad Qingyu continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    if observer != player || cancelled {
        return Ok(());
    }
    let cards: Vec<_> = selections.iter().map(|raw| CardId::new(*raw)).collect();
    let allowed = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if cards.len() != 2 || cards.iter().any(|card| !allowed.contains(card)) {
        return Err(EffectError::Illegal("Qingyu discard card invalid".into()));
    }
    crate::effect::run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }])?;
    let damage_id = tx.state().resolution.current_damage_id;
    if damage_id != 0 {
        crate::effect::run_effects(tx, vec![EffectRequest::PreventDamage { damage_id }])?;
    }
    let _ = skill;
    Ok(())
}

/// 清雨使命失败授技目标续体（ABI 431，payload[0]=RESOLVE_QINGYU_FAIL_TARGET）。
pub(crate) fn resolve_qingyu_fail_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad Qingyu fail continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let mbyongjue_def = engine_api::SkillDefId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if observer != player || cancelled {
        return Ok(());
    }
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "Qingyu fail requires one target".into(),
        ));
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
            "Qingyu fail target no longer legal".into(),
        ));
    }
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::ResolveQingyu {
            player,
            action: 3,
            xuancun_def: engine_api::SkillDefId::NONE,
            mbyongjue_def,
            target,
            skill,
        }],
    )
    .map(|_| ())
}

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
