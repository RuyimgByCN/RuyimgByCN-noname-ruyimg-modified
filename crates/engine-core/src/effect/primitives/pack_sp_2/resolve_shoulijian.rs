//! Effect primitives for `resolve_shoulijian` and related helpers.

use super::*;

/// 欢乐卡牌【手里剑】使用结算（ABI 642, ADR 0607）：目标弃一张装备牌或受 1 点伤害。
///
/// 复用 `open_xiaoguo_target_equipment_or_damage` 的「弃装备或伤害」choice 结算。
pub(crate) fn resolve_shoulijian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if player == target
        || !tx
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
        return Ok(Vec::new());
    }
    // If the target has no equipment to discard, they take 1 damage directly;
    // otherwise they choose (handled by the Xiaoguo continuation).
    open_xiaoguo_target_equipment_or_damage(tx, player, target, skill)?;
    Ok(Vec::new())
}

/// 取消使用牌（ABI 643, ADR 0608）：gwent 杜达元帅1 特殊卡——当你成为其他角色使用牌
/// 的目标时，取消该牌对你生效，并获取对你使用的实体牌。
///
/// 触发上下文为 `trigger card_targeted`（牌使用结算中、逐目标触发）。此刻杀牌走
/// `sha_resolution_records`/ResumePlayedSha，锦囊走 WuxieWindow resume_payload，两者
/// 均已挂起，可经 `exclude_saved_card_target` 把 `player` 从当前牌结算目标中排除；
/// 不可排除的牌型（如无中生有类全目标锦囊）整效果视为未发动（graceful no-op）。
pub(crate) fn cancel_incoming_card_use(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if player == source
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(source)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    // The holder must still be a declared target of the suspended card use.
    let (_, removable) = crate::command::play::current_card_target_adjustments(tx, source, card)?;
    if !removable.contains(&player) {
        // 批 155：多目标锦囊（南蛮/万箭 AOE 队列）在触发效果执行时尚未挂起
        // （resume_kind=None）——把排除意图暂存，`advance_aoe_queue` 逐目标时消费。
        if tx.state().resolution.resume_kind == crate::state::ContinuationKind::None {
            tx.state_mut()
                .deferred_card_exclusions
                .push((card.raw(), player.raw()));
        }
        return Ok(Vec::new());
    }
    // 取消：把 holder 从当前牌结算目标中排除（杀走 sha 记录、锦囊走 Wuxie payload；
    // 批 155 起南蛮/万箭 AOE 队列经 deferred_card_exclusions 延迟排除——触发效果
    // 执行时 wuxie 载荷未挂起（resume_kind=None）或挂起但 NANMAN/WANJIAN 尚无
    // removable 计算（批 155 已补）皆可达此路径）。
    if crate::command::play::exclude_saved_card_target(tx, card, player).is_err() {
        // 该牌型无法被排除（全目标锦囊等）：整效果未发动。
        return Ok(Vec::new());
    }
    // 获得对你使用的实体牌（杜达元帅语义；啖酪共用本原语，多获实体牌为已注记残差）。
    let used = tx.state().resolution.card_source_cards(source, card);
    gain_event_cards(tx, player, &used)
}

/// 择一加技能（ABI 644, ADR 0612）：gwent 呼朋（hupeng）——玩家在两个技能定义间
/// 二选一，将所选技能置入目标（上游为 chooseVCardButton 择一「杜达元帅1/2」实体卡
/// 置入目标手牌；本引擎以技能近似杜达元帅卡，故置入所选技能）。
///
/// 复用 `ContinuationKind::Custom` 魔数分发（payload[0] 为本魔数），不新增
/// ContinuationKind 变体，保持回放序数稳定。
pub(crate) fn choose_other_gain_one_of_two_skills(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    first_def: engine_api::SkillDefId,
    second_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if player == target
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || first_def.is_none()
        || second_def.is_none()
        || first_def == second_def
    {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![
            CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS_CUSTOM_CHOICE,
            0,
            player.raw(),
            target.raw(),
            first_def.raw(),
            second_def.raw(),
            skill.raw(),
        ],
    )
}

pub(crate) fn open_xiaoguo_target_equipment_or_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
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
        return Ok(());
    }
    let options = crate::query::standard_xiaoguo_equipment_cards(tx.state(), target);
    if options.is_empty() {
        return super::super::run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: player,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .map(|_| ());
    }
    open_choice(
        tx,
        target,
        0,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::XiaoguoDiscardBasicThenEquipOrDamage,
        vec![2, player.raw(), target.raw(), skill.raw()],
    )
    .map(|_| ())
}

pub(crate) fn judge_prevent_damage_lose_max_hp_on_nonheart(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    damage_id: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().players.get(player).is_none() || tx.state().players.get(target).is_none() {
        return Err(EffectError::Illegal("bad judgment participant".into()));
    }
    if damage_id == 0 || tx.state().resolution.current_damage_id != damage_id {
        return Err(EffectError::ContentFault(
            "judgment damage prevention used outside its damage_before event".into(),
        ));
    }
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
        return Ok(Vec::new());
    }

    let result = crate::command::judge::flip_judgment(
        tx,
        player,
        crate::command::judge::JudgmentFavor::NonSuit(engine_api::Suit::Heart),
    )?;
    let suit = result.suit;
    let rank = result.rank;
    let judgment_event = completed_judgment_event(player, result, CardId::NONE);
    tx.push_event(DomainEvent::RuleNote {
        code: 234,
        payload: vec![
            player.raw(),
            target.raw(),
            damage_id,
            crate::command::judge::suit_tag(suit),
            u32::from(rank),
        ],
    });
    if suit == engine_api::Suit::Heart {
        return Ok(vec![judgment_event]);
    }
    if !tx
        .state()
        .resolution
        .prevented_damage_ids
        .contains(&damage_id)
    {
        tx.state_mut()
            .resolution
            .prevented_damage_ids
            .push(damage_id);
    }
    change_max_hp(tx, target, -1).map(|events| prepend_event(judgment_event, events))
}

pub(crate) fn make_current_sha_target_unrespondable(
    tx: &mut Transaction,
    source: PlayerId,
    card: CardId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(source)
        .is_some_and(|player| player.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|player| player.alive)
        || !tx.state().cards.get(card).is_some_and(|value| {
            crate::query::effective_card_kind(tx.state(), source, value).is_sha()
        })
    {
        return Err(EffectError::Illegal(
            "invalid current Sha response participant".into(),
        ));
    }
    if !tx
        .state_mut()
        .resolution
        .make_saved_sha_target_unrespondable(source, card, target)
    {
        return Err(EffectError::Illegal(
            "no matching currently resolving Sha target".into(),
        ));
    }
    Ok(vec![DomainEvent::RuleNote {
        code: 234,
        payload: vec![source.raw(), card.raw(), target.raw()],
    }])
}

pub(crate) fn increase_current_sha_target_damage(
    tx: &mut Transaction,
    source: PlayerId,
    card: CardId,
    target: PlayerId,
    amount: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if amount == 0 {
        return Ok(Vec::new());
    }
    if !tx
        .state()
        .players
        .get(source)
        .is_some_and(|player| player.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|player| player.alive)
        || !tx.state().cards.get(card).is_some_and(|value| {
            crate::query::effective_card_kind(tx.state(), source, value).is_sha()
        })
    {
        return Err(EffectError::Illegal(
            "invalid current Sha damage participant".into(),
        ));
    }
    crate::command::play::increase_saved_sha_target_damage(tx, source, card, target, amount)?;
    Ok(vec![DomainEvent::RuleNote {
        code: 235,
        payload: vec![source.raw(), card.raw(), target.raw(), amount],
    }])
}

pub(crate) fn choose_discard_target_card_resolve_current_sha(
    tx: &mut Transaction,
    source: PlayerId,
    card: CardId,
    target: PlayerId,
    nonbasic_bonus_skill: Option<engine_api::SkillDefId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    crate::command::play::validate_saved_sha_target(tx, source, card, target)?;
    open_discard_target_card_choice(
        tx,
        source,
        target,
        rule_api::TargetCardScope::HandEquip,
        ContinuationKind::ChooseDiscardTargetCardResolveCurrentSha,
        false,
        nonbasic_bonus_skill,
    )
}

pub(crate) fn resolve_dcqiaomeng(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    target: PlayerId,
    selected: CardId,
    helper_skill_def: engine_api::SkillDefId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let context_valid = originating_skill_is_enabled(tx, player, skill)
        && !helper_skill_def.is_none()
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        && crate::command::play::validate_saved_targeted_card_use(tx, player, card).is_ok()
        && tx
            .state()
            .cards
            .get(card)
            .map(|card| crate::query::effective_card_suit(tx.state(), player, card))
            .is_some_and(|suit| matches!(suit, Suit::Spade | Suit::Club));
    if !context_valid {
        return if stage == 0 {
            Ok(Vec::new())
        } else {
            Err(EffectError::Illegal(
                "Dcqiaomeng current-card context changed".into(),
            ))
        };
    }
    let eligible_targets = || {
        let mut seen = std::collections::BTreeSet::new();
        targets
            .iter()
            .copied()
            .filter(|candidate| seen.insert(*candidate))
            .filter(|candidate| *candidate != player)
            .filter(|candidate| {
                tx.state()
                    .players
                    .get(*candidate)
                    .is_some_and(|participant| participant.alive)
            })
            .filter(|candidate| {
                !crate::query::discardable_target_cards(
                    tx.state(),
                    player,
                    *candidate,
                    TargetCardScope::HandEquip,
                )
                .is_empty()
            })
            .collect::<Vec<_>>()
    };
    let mut payload = vec![
        stage as u32,
        player.raw(),
        card.raw(),
        target.raw(),
        helper_skill_def.raw(),
        skill.raw(),
        targets.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(targets.iter().copied().map(PlayerId::raw));
    match stage {
        0 => {
            let options = eligible_targets();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveDcqiaomeng,
                payload,
            )
        }
        1 => {
            if !eligible_targets().contains(&target) {
                return Err(EffectError::Illegal(
                    "Dcqiaomeng selected target is no longer legal".into(),
                ));
            }
            let options = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                TargetCardScope::HandEquip,
            );
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveDcqiaomeng,
                payload,
            )
        }
        2 => {
            if !targets.contains(&target)
                || !crate::query::discardable_target_cards(
                    tx.state(),
                    player,
                    target,
                    TargetCardScope::HandEquip,
                )
                .contains(&selected)
            {
                return Err(EffectError::Illegal(
                    "Dcqiaomeng target card is no longer legal".into(),
                ));
            }
            let selected_kind = tx
                .state()
                .cards
                .get(selected)
                .map(|card| card.kind)
                .ok_or_else(|| EffectError::Illegal("Dcqiaomeng card is missing".into()))?;
            let mut effects = vec![EffectRequest::DiscardTargetCards {
                actor: player,
                target,
                cards: vec![selected],
                scope: TargetCardScope::HandEquip,
            }];
            if selected_kind.is_equip() {
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("missing discard pile".into()))?;
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, player)
                    .ok_or_else(|| EffectError::EngineFault("missing Dcqiaomeng hand".into()))?;
                effects.push(EffectRequest::MoveCards {
                    cards: vec![selected],
                    to: hand,
                    from: Some(discard),
                });
            } else if selected_kind.is_trick() {
                effects.push(EffectRequest::AddCardUsePolicySkill {
                    player,
                    card,
                    policy_mask: 1 << 1,
                    skill_def: helper_skill_def,
                });
            }
            run_effects(tx, effects).map(|_| Vec::new())
        }
        _ => Err(EffectError::EngineFault("invalid Dcqiaomeng stage".into())),
    }
}

pub(crate) fn take_from_hand(
    tx: &mut Transaction,
    from: PlayerId,
    to: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if from == to {
        return Ok(vec![]);
    }
    let hand = tx.state().hand_cards(from);
    let Some(card) = hand.first().copied() else {
        return Ok(vec![]);
    };
    let to_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, to)
        .ok_or_else(|| EffectError::Illegal("no dest hand".into()))?;
    move_cards(tx, vec![card], to_hand, None)
}

pub(crate) fn draw_cards(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().players.get(player).is_none() {
        return Err(EffectError::Illegal(format!("no player {}", player.raw())));
    }
    let draw_zone = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let hand_zone = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;

    let mut drawn = Vec::new();
    for _ in 0..count {
        // Reshuffle if empty (noname: 洗牌)
        let empty = tx
            .state()
            .zones
            .get(draw_zone)
            .map(|z| z.cards.is_empty())
            .unwrap_or(true);
        if empty {
            reshuffle_discard_into_draw(tx)?;
        }
        let card = {
            let z = tx
                .state()
                .zones
                .get(draw_zone)
                .ok_or_else(|| EffectError::EngineFault("draw zone missing".into()))?;
            match z.cards.last().copied() {
                Some(c) => c,
                None => break,
            }
        };
        move_one_card(tx, card, draw_zone, hand_zone, player, false)?;
        drawn.push(card);
    }
    if drawn.is_empty() && count > 0 {
        return Ok(vec![]);
    }
    if !drawn.is_empty() && tx.state().turn.phase == Phase::Play {
        tx.state_mut().turn.record_play_phase_draw(player);
    }
    Ok(vec![DomainEvent::CardsDrawn {
        player,
        cards: drawn,
    }])
}

pub(crate) fn draw_remembered_hand_cards(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0 || !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let events = draw_cards(tx, player, count)?;
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let cards: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::CardsDrawn {
                player: drawer,
                cards,
            } if *drawer == player => Some(cards.as_slice()),
            _ => None,
        })
        .flatten()
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|state| state.owner == player && Some(state.zone) == hand)
        })
        .collect();
    if !tx
        .state_mut()
        .skills
        .get_mut(skill)
        .expect("validated originating skill")
        .remember_hand_cards(&cards)
    {
        return Err(EffectError::ContentFault(
            "remembered hand cards conflict with existing skill state".into(),
        ));
    }
    Ok(events)
}

pub(crate) fn discard_remembered_hand_cards(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let remembered = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .expect("validated originating skill")
        .take_remembered_hand_cards()
        .ok_or_else(|| {
            EffectError::ContentFault(
                "remembered hand cards conflict with existing skill state".into(),
            )
        })?;
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let discardable: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .collect();
    let cards: Vec<_> = remembered
        .into_iter()
        .filter(|card| {
            discardable.contains(card)
                && tx
                    .state()
                    .cards
                    .get(*card)
                    .is_some_and(|state| state.owner == player && Some(state.zone) == hand)
        })
        .collect();
    if cards.is_empty() {
        Ok(Vec::new())
    } else {
        discard_cards(tx, player, cards)
    }
}

pub(crate) fn draw_each_by_seat(
    tx: &mut Transaction,
    first: PlayerId,
    second: PlayerId,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if first == second {
        return Err(EffectError::Illegal(
            "draw_each_by_seat requires distinct players".into(),
        ));
    }
    let seats = tx.state().players.alive_by_seat();
    if !seats.contains(&first) || !seats.contains(&second) {
        return Ok(Vec::new());
    }
    let mut players = [first, second];
    players.sort_by_key(|player| {
        seats
            .iter()
            .position(|seated| seated == player)
            .unwrap_or(usize::MAX)
    });
    let mut events = draw_cards(tx, players[0], count)?;
    events.extend(draw_cards(tx, players[1], count)?);
    Ok(events)
}

pub(crate) fn draw_selected_targets(
    tx: &mut Transaction,
    targets: &[PlayerId],
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if targets.is_empty() {
        return Err(EffectError::ContentFault(
            "selected-target draw requires at least one target".into(),
        ));
    }
    let order = tx.state().players.alive_by_seat();
    let mut sorted = Vec::with_capacity(targets.len());
    for &target in targets {
        if sorted.contains(&target) {
            return Err(EffectError::ContentFault(
                "selected-target draw received duplicate targets".into(),
            ));
        }
        if !order.contains(&target) {
            return Err(EffectError::Illegal(
                "selected-target draw requires living targets".into(),
            ));
        }
        sorted.push(target);
    }
    sorted.sort_by_key(|target| {
        order
            .iter()
            .position(|candidate| candidate == target)
            .expect("validated living target")
    });
    let mut events = Vec::new();
    for target in sorted {
        events.extend(draw_cards(tx, target, count)?);
    }
    Ok(events)
}

pub(crate) fn draw_linked_players(
    tx: &mut Transaction,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let players: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&player| {
            tx.state().players.get(player).is_some_and(|state| {
                state
                    .marks
                    .iter()
                    .any(|(mark, value)| *mark == crate::content::MARK_CHAIN && *value > 0)
            })
        })
        .collect();
    let mut events = Vec::new();
    for player in players {
        events.extend(draw_cards(tx, player, count)?);
    }
    Ok(events)
}
