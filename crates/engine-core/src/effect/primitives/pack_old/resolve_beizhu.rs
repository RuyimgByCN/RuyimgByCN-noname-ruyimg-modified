//! Effect primitives for `resolve_beizhu` and related helpers.

use super::*;

pub(crate) fn resolve_beizhu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    mut cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage == 0 {
        if !originating_skill_is_enabled(tx, player, skill) {
            return Ok(Vec::new());
        }
        if player == target
            || ![player, target].into_iter().all(|participant| {
                tx.state()
                    .players
                    .get(participant)
                    .is_some_and(|state| state.alive)
            })
            || tx.state().hand_cards(target).is_empty()
        {
            return Err(EffectError::Illegal(
                "Beizhu requires one living other target with hand cards".into(),
            ));
        }
        cards = tx
            .state()
            .hand_cards(target)
            .into_iter()
            .filter(|card| {
                tx.state().cards.get(*card).is_some_and(|card| {
                    crate::query::effective_card_kind(tx.state(), target, card).is_sha()
                })
            })
            .collect();
        if cards.is_empty() {
            let options = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            if options.is_empty() {
                return resolve_beizhu(tx, player, target, 1, Vec::new(), skill);
            }
            let mut payload = vec![
                0,
                player.raw(),
                target.raw(),
                skill.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            return open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveBeizhu,
                payload,
            );
        }
        let mut rng = RuleRng::from_state(tx.state().rng);
        rng.shuffle(&mut cards);
        tx.state_mut().rng = rng.into_state();
        return resolve_beizhu(tx, player, target, 2, cards, skill);
    }

    if stage == 1 {
        if ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        }) {
            return Ok(Vec::new());
        }
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("Beizhu draw pile is missing".into()))?;
        let sha = tx.state().zones.get(draw).and_then(|zone| {
            zone.cards.iter().rev().copied().find(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.kind.is_sha())
            })
        });
        let Some(sha) = sha else {
            return Ok(Vec::new());
        };
        return open_choice(
            tx,
            player,
            1,
            1,
            true,
            vec![1],
            ChoiceKind::Custom,
            ContinuationKind::ResolveBeizhu,
            vec![
                1,
                player.raw(),
                target.raw(),
                sha.raw(),
                draw.raw(),
                skill.raw(),
            ],
        );
    }

    if stage != 2 {
        return Err(EffectError::ContentFault(
            "unknown Beizhu resolution stage".into(),
        ));
    }
    while let Some(card) = cards.first().copied() {
        cards.remove(0);
        if !crate::command::play::beizhu_sha_usable(tx, target, player, card) {
            continue;
        }
        if !cards.is_empty() {
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveBeizhu {
                    player,
                    target,
                    stage: 2,
                    cards,
                    skill,
                });
        }
        crate::command::play::play_beizhu_sha_source(tx, target, player, card)?;
        return Ok(Vec::new());
    }
    Ok(Vec::new())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_anguo_flow(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    index: u8,
    remaining: u8,
    skill: engine_api::SkillId,
    allow_recast: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if stage == 0 {
        if target == player
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|target| target.alive)
        {
            return Err(EffectError::Illegal(
                "Anguo requires one living other target".into(),
            ));
        }
        return resolve_anguo_flow(tx, player, target, 1, 0, 0b111, skill, allow_recast);
    }
    if index >= 3 {
        if stage == 1 {
            return resolve_anguo_flow(tx, player, target, 2, 0, remaining, skill, allow_recast);
        }
        if allow_recast && remaining == 0 {
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let equip = tx.state().zones.find(ZoneKind::Equip, player);
            let mut options: Vec<_> = [hand, equip]
                .into_iter()
                .flatten()
                .flat_map(|zone| {
                    tx.state()
                        .zones
                        .get(zone)
                        .into_iter()
                        .flat_map(|zone| zone.cards.iter().copied())
                })
                .collect();
            options.sort_by_key(|card| card.raw());
            options.dedup();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let max = options.len().min(u8::MAX as usize) as u8;
            let mut payload = vec![player.raw(), skill.raw(), options.len() as u32];
            payload.extend(options.iter().map(|card| card.raw()));
            return open_choice(
                tx,
                player,
                1,
                max,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveReanguo,
                payload,
            );
        }
        return Ok(Vec::new());
    }
    let actor = if stage == 1 { target } else { player };
    let bit = 1u8 << index;
    if remaining & bit == 0 {
        return resolve_anguo_flow(
            tx,
            player,
            target,
            stage,
            index + 1,
            remaining,
            skill,
            allow_recast,
        );
    }
    let condition = xinanguo_metric_is_min(tx, actor, index)
        && (index != 1
            || tx
                .state()
                .players
                .get(actor)
                .is_some_and(|actor| actor.hp < actor.max_hp as i32));
    if !condition {
        return resolve_anguo_flow(
            tx,
            player,
            target,
            stage,
            index + 1,
            remaining,
            skill,
            allow_recast,
        );
    }
    let (events, performed) = match index {
        0 => (draw_cards(tx, actor, 1)?, true),
        1 => (recover_hp(tx, player, actor, 1)?, true),
        2 => equip_random_from_draw_pile(tx, actor, None)?,
        _ => unreachable!(),
    };
    if !performed {
        return resolve_anguo_flow(
            tx,
            player,
            target,
            stage,
            index + 1,
            remaining,
            skill,
            allow_recast,
        );
    }
    let remaining = remaining & !bit;
    let next = if allow_recast {
        EffectRequest::ResolveReanguo {
            player,
            target,
            stage,
            index: index + 1,
            remaining,
            skill,
        }
    } else {
        EffectRequest::ResolveXinanguo {
            player,
            target,
            stage,
            index: index + 1,
            remaining,
            skill,
        }
    };
    tx.state_mut().resolution.deferred_tail_effects.push(next);
    Ok(events)
}

pub(crate) fn resolve_ganglie_three(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let enemies: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|candidate| {
            matches!(
                crate::query::query(
                    tx.state(),
                    &QueryRequest::Relation {
                        a: player,
                        b: *candidate,
                    },
                    &mut crate::query::QueryContext::default(),
                ),
                QueryResponse::Relation(RelationTag::Enemy)
            )
        })
        .collect();
    if target.is_none() {
        return GuardedChoice::new(
            skill,
            Participants::owner_any_state(player),
            ContinuationKind::ResolveGanglieThree,
            vec![player.raw(), skill.raw()],
        )
        .open_targets(tx, enemies);
    }
    if !enemies.contains(&target) {
        return Err(EffectError::Illegal(
            "Ganglie Three target must be a living enemy".into(),
        ));
    }
    judge_target_discard_two_or_damage(tx, player, target, false)
}

pub(crate) fn resolve_zhongyi(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    helper_skill_def: engine_api::SkillDefId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let card_state =
                tx.state().cards.get(card).ok_or_else(|| {
                    EffectError::Illegal("Zhongyi selected card is missing".into())
                })?;
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let equip = tx.state().zones.find(ZoneKind::Equip, player);
            if card_state.owner != player
                || (Some(card_state.zone) != hand && Some(card_state.zone) != equip)
                || !tx
                    .state()
                    .skill_pile_cards(player, helper_skill_def.raw())
                    .is_empty()
            {
                return Err(EffectError::Illegal(
                    "Zhongyi requires one current hand/equipment card and an empty pile".into(),
                ));
            }
            let from = card_state.zone;
            let round = tx.state().turn.round;
            let pile = ensure_skill_pile_zone(tx, player, helper_skill_def.raw())?;
            let mut events = add_skill_with_state(
                tx,
                player,
                helper_skill_def,
                Some(round.to_le_bytes().into()),
            )?;
            events.extend(move_cards(tx, vec![card], pile, Some(from))?);
            Ok(events)
        }
        1 => {
            let valid = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player
                    && instance.def == helper_skill_def
                    && tx.state().skill_effectively_enabled(instance)
            }) && !tx
                .state()
                .skill_pile_cards(player, helper_skill_def.raw())
                .is_empty();
            if !valid {
                return Ok(Vec::new());
            }
            let damage_id = tx.state().resolution.current_damage_id;
            increase_bound_damage(tx, damage_id, 1)?;
            Ok(Vec::new())
        }
        2 => {
            let Some(instance) = tx.state().skills.get(skill) else {
                return Ok(Vec::new());
            };
            if instance.owner != player
                || instance.def != helper_skill_def
                || instance.state_blob.len() != 4
            {
                return Ok(Vec::new());
            }
            let activated_round =
                u32::from_le_bytes(instance.state_blob.as_slice().try_into().unwrap());
            if tx.state().turn.round <= activated_round {
                return Ok(Vec::new());
            }
            let cards = tx.state().skill_pile_cards(player, helper_skill_def.raw());
            let mut events = if cards.is_empty() {
                Vec::new()
            } else {
                let pile = ensure_skill_pile_zone(tx, player, helper_skill_def.raw())?;
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
                move_cards(tx, cards, discard, Some(pile))?
            };
            events.extend(remove_skill(tx, skill)?);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("unknown Zhongyi action".into())),
    }
}

pub(crate) fn discard_equipment_slot(
    tx: &mut Transaction,
    player: PlayerId,
    slot: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    let slot = EquipSlot::from_tag(slot)
        .ok_or_else(|| EffectError::ContentFault("invalid equipment slot".into()))?;
    let Some(equip) = tx.state().zones.find(ZoneKind::Equip, player) else {
        return Ok(Vec::new());
    };
    let cards: Vec<_> = tx
        .state()
        .zones
        .get(equip)
        .into_iter()
        .flat_map(|zone| zone.cards.iter())
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .and_then(|card| card.kind.equip_slot())
                == Some(slot)
        })
        .collect();
    if cards.is_empty() {
        Ok(Vec::new())
    } else {
        discard_cards(tx, player, cards)
    }
}

pub(crate) fn record_active_target(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    let instance = state
        .skills
        .get(skill)
        .ok_or_else(|| EffectError::ContentFault("active target recorder missing skill".into()))?;
    let contract = state
        .content
        .first_active_for_skill(instance.def)
        .ok_or_else(|| {
            EffectError::ContentFault("active target recorder missing contract".into())
        })?;
    let target_contract = contract.targets.as_ref().ok_or_else(|| {
        EffectError::ContentFault("active target recorder missing targets".into())
    })?;
    let turn_scoped = target_contract.unused_by_skill_this_turn;
    let play_phase_scoped = target_contract.unused_by_skill_this_play_phase;
    if instance.owner != player
        || !state.skill_effectively_enabled(instance)
        || state.turn.current != player
        || state.turn.phase != Phase::Play
        || turn_scoped == play_phase_scoped
    {
        return Err(EffectError::ContentFault(
            "active target recorder requires an unused-target contract".into(),
        ));
    }
    if (target == player && !target_contract.allow_self)
        || !state
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "active target recorder requires another living target".into(),
        ));
    }
    let recorded = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .is_some_and(|instance| {
            if play_phase_scoped {
                instance.record_active_play_phase_target(target)
            } else {
                instance.record_active_target(target)
            }
        });
    if !recorded {
        return Err(EffectError::Illegal(
            "active target was already selected in this uniqueness window".into(),
        ));
    }
    Ok(Vec::new())
}

pub(crate) fn resolve_dingpin(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::ContentFault(
            "Dingpin requires its enabled owner and a living Play-phase target".into(),
        ));
    }
    let result = crate::command::judge::flip_judgment(
        tx,
        target,
        crate::command::judge::JudgmentFavor::Neutral,
    )?;
    let judgment_event = completed_judgment_event(target, result, CardId::NONE);
    tx.push_event(DomainEvent::RuleNote {
        code: 232,
        payload: vec![
            target.raw(),
            crate::command::judge::suit_tag(result.suit),
            u32::from(result.rank),
        ],
    });
    match result.suit {
        Suit::Spade | Suit::Club => {
            record_active_target(tx, player, target, skill)?;
            let lost_hp = tx
                .state()
                .players
                .get(target)
                .map(|participant| {
                    (i64::from(participant.max_hp) - i64::from(participant.hp)).max(0) as u32
                })
                .unwrap_or(0);
            draw_cards(tx, target, lost_hp).map(|events| prepend_event(judgment_event, events))
        }
        Suit::Heart | Suit::Diamond => {
            turn_over(tx, player).map(|events| prepend_event(judgment_event, events))
        }
        Suit::None => Ok(vec![judgment_event]),
    }
}

pub(crate) fn discardable_weapon_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    crate::query::discardable_hand_equip_cards(tx.state(), player)
        .into_iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.kind == CardKind::Weapon)
        })
        .collect()
}

pub(crate) fn choose_discard_weapon_or_lose_hp(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Err(EffectError::Illegal(
            "invalid discard-weapon-or-lose-HP context".into(),
        ));
    }
    let can_lose_hp = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive && participant.hp >= 1);
    let weapons = discardable_weapon_cards(tx, player);
    match (can_lose_hp, weapons.is_empty()) {
        (false, true) => Err(EffectError::Illegal(
            "discard-weapon-or-lose-HP cost is unavailable".into(),
        )),
        (true, true) => deal_damage(
            tx,
            PlayerId::NONE,
            player,
            1,
            true,
            rule_api::damage_nature::NORMAL,
            false,
            CardId::NONE,
            0,
            0,
        ),
        (false, false) => open_choice(
            tx,
            player,
            1,
            1,
            false,
            weapons.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ChooseDiscardWeaponOrLoseHp,
            vec![player.raw(), skill.raw(), 1],
        ),
        (true, false) => open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::ChooseDiscardWeaponOrLoseHp,
            vec![player.raw(), skill.raw(), 0],
        ),
    }
}

/// 弃置一张手牌/装备牌，或失去 `lose_amount` 点体力（庸肆 xinyongsi2 弃一张或失去 1 点）。
/// 与 `choose_discard_weapon_or_lose_hp` 同构：可选弃牌时开二选一（0=失去体力,1=选牌），
/// 无可弃牌时直接失去体力。Choice 恢复时重验上下文。
pub(crate) fn choose_discard_one_he_or_lose_hp(
    tx: &mut Transaction,
    player: PlayerId,
    lose_amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Err(EffectError::Illegal(
            "invalid discard-one-or-lose-HP context".into(),
        ));
    }
    let can_lose_hp = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive && participant.hp >= lose_amount as i32);
    let discardable = crate::query::discardable_hand_equip_cards(tx.state(), player);
    match (can_lose_hp, discardable.is_empty()) {
        (false, true) => Err(EffectError::Illegal(
            "discard-one-or-lose-HP cost is unavailable".into(),
        )),
        (true, true) => deal_damage(
            tx,
            PlayerId::NONE,
            player,
            lose_amount,
            true,
            rule_api::damage_nature::NORMAL,
            false,
            CardId::NONE,
            0,
            0,
        ),
        (false, false) => open_choice(
            tx,
            player,
            1,
            1,
            false,
            discardable.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ChooseDiscardOneHeOrLoseHp,
            vec![player.raw(), skill.raw(), lose_amount, 1],
        ),
        (true, false) => open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::ChooseDiscardOneHeOrLoseHp,
            vec![player.raw(), skill.raw(), lose_amount, 0],
        ),
    }
}

/// 随势（suishi2）：其他角色死亡后，其伤害来源选择是否令拥有者失去 1 点体力。
/// 上游：trigger.source.chooseBool() 确认后 player.loseHp()。Choice 恢复时重验来源存活。
pub(crate) fn choose_source_confirm_owner_lose_hp(
    tx: &mut Transaction,
    owner: PlayerId,
    source: PlayerId,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, owner, skill)
        || source == PlayerId::NONE
        || source == owner
        || !tx
            .state()
            .players
            .get(owner)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "invalid source-confirm-owner-lose-HP context".into(),
        ));
    }
    open_choice(
        tx,
        source,
        0,
        1,
        false,
        vec![1],
        ChoiceKind::Confirm,
        ContinuationKind::ChooseSourceConfirmOwnerLoseHp,
        vec![owner.raw(), source.raw(), skill.raw(), amount],
    )
}

/// 峻攻：弃置至多 `use_count + 1` 张手牌/装备牌或失去等量体力，然后对目标使用虚拟【杀】。
///
/// 旧 JS 的可用过滤 `num < hp || num <= countCards("he")` 与选牌 `[0, num]`（选 0 张即
/// 失去体力）由本原语校验并在 Choice 恢复时重验；虚拟【杀】沿普通结算队列运行，命中目标
/// 时由配套触发 `owner_source_sha_damage_recorded_by_skill` 禁用本主动技至回合结束。
pub(crate) fn spjungong_discard_or_lose_then_sha(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "invalid 峻攻 discard-or-lose context".into(),
        ));
    }
    let instance = tx
        .state()
        .skills
        .get(skill)
        .ok_or_else(|| EffectError::ContentFault("峻攻 active skill missing".into()))?;
    let n = instance.use_count.saturating_add(1);
    let hp = tx
        .state()
        .players
        .get(player)
        .map(|participant| participant.hp)
        .unwrap_or(0);
    let cards = crate::query::discardable_target_cards(
        tx.state(),
        player,
        player,
        rule_api::TargetCardScope::HandEquip,
    );
    if !(hp > n as i32) && cards.len() < n as usize {
        // 旧 JS filter：num < hp || num <= countCards("he")。
        return Ok(Vec::new());
    }
    let max_select = (n as usize).min(cards.len()).min(u8::MAX as usize) as u8;
    open_choice(
        tx,
        player,
        0,
        max_select,
        false,
        cards.into_iter().map(CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        ContinuationKind::ChooseSpjungongDiscardOrLose,
        vec![player.raw(), skill.raw(), target.raw(), n],
    )
}

/// 把主动技实例的 `active_turn_banned` 置位，禁用该主动技直到当前回合结束。
pub(crate) fn ban_active_until_turn_end(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::ContentFault("turn-end active ban skill missing".into()))?;
    if instance.owner != player {
        return Err(EffectError::ContentFault(
            "turn-end active ban owner mismatch".into(),
        ));
    }
    instance.active_turn_banned = true;
    Ok(vec![DomainEvent::RuleNote {
        code: 241,
        payload: vec![player.raw(), skill.raw(), 1],
    }])
}

pub(crate) fn force_others_use_sha_on_nearest_or_lose_hp(
    tx: &mut Transaction,
    owner: PlayerId,
    remaining: Vec<PlayerId>,
    started: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut remaining = if started {
        remaining
    } else {
        if !originating_skill_is_enabled(tx, owner, skill)
            || !tx
                .state()
                .players
                .get(owner)
                .is_some_and(|player| player.alive)
        {
            return Ok(Vec::new());
        }
        let order = tx.state().players.alive_by_seat();
        let Some(start) = order.iter().position(|&player| player == owner) else {
            return Ok(Vec::new());
        };
        (1..order.len())
            .map(|offset| order[(start + offset) % order.len()])
            .collect()
    };

    while remaining.first().is_some_and(|player| {
        !tx.state()
            .players
            .get(*player)
            .is_some_and(|participant| participant.alive)
    }) {
        remaining.remove(0);
    }
    if remaining.is_empty() {
        return Ok(Vec::new());
    }

    let current = remaining.remove(0);
    let target_limit = crate::query::sha_target_limit_for(tx.state(), current);
    let cards = if target_limit == 0 {
        Vec::new()
    } else {
        crate::command::play::prompted_sha_source_cards(tx.state(), current)
            .into_iter()
            .filter(|card| !crate::command::play::luanwu_sha_targets(tx, current, *card).is_empty())
            .collect()
    };
    if cards.is_empty() {
        crate::effect::continue_luanwu_after_hp_loss(tx, owner, current, skill, remaining)?;
        return Ok(Vec::new());
    }

    let mut payload = vec![
        0,
        owner.raw(),
        current.raw(),
        skill.raw(),
        remaining.len() as u32,
    ];
    payload.extend(remaining.into_iter().map(PlayerId::raw));
    open_choice(
        tx,
        current,
        0,
        1,
        true,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ForceOthersUseShaOnNearestOrLoseHp,
        payload,
    )
}

pub(crate) fn discard_selected_record_unique_category_target(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    let instance = state
        .skills
        .get(skill)
        .ok_or_else(|| EffectError::ContentFault("missing originating active skill".into()))?;
    let contract = state
        .content
        .first_active_for_skill(instance.def)
        .ok_or_else(|| EffectError::ContentFault("missing originating active contract".into()))?;
    let targets = contract.targets.as_ref();
    let selection = contract.selection.as_ref();
    let turn_scoped = targets.is_some_and(|targets| {
        targets.min == 1
            && targets.max == 1
            && !targets.allow_self
            && targets.unused_by_skill_this_turn
    }) && selection.is_some_and(|selection| {
        selection.min == 1 && selection.max == 1 && selection.unused_category_by_skill_this_turn
    });
    let play_phase_scoped = targets.is_some_and(|targets| {
        targets.min == 1
            && targets.max == 1
            && targets.allow_self
            && targets.unused_by_skill_this_play_phase
    }) && selection.is_some_and(|selection| {
        selection.min == 1
            && selection.max == 1
            && selection.unused_category_by_skill_this_play_phase
    });
    if instance.owner != player
        || !state.skill_effectively_enabled(instance)
        || state.turn.current != player
        || state.turn.phase != Phase::Play
        || turn_scoped == play_phase_scoped
    {
        return Err(EffectError::ContentFault(
            "active uniqueness effect requires its declared contract".into(),
        ));
    }
    if (target == player && !targets.is_some_and(|targets| targets.allow_self))
        || !state
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !crate::query::discardable_hand_equip_cards(state, player).contains(&card)
    {
        return Err(EffectError::Illegal(
            "active uniqueness target or selected card is no longer legal".into(),
        ));
    }
    let category_bit = state
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(state, player, card))
        .map(|kind| {
            if kind.is_basic() {
                1
            } else if kind.is_trick() {
                2
            } else if kind.is_equip() {
                4
            } else {
                0
            }
        })
        .unwrap_or(0);
    let recorded = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .is_some_and(|instance| {
            if turn_scoped {
                instance.record_active_unique_use(category_bit, target)
            } else if category_bit != 0
                && instance.play_phase_category_mask & category_bit == 0
                && !instance
                    .active_play_phase_targets()
                    .unwrap_or_default()
                    .contains(&target)
                && instance.record_active_play_phase_target(target)
            {
                instance.play_phase_category_mask |= category_bit;
                true
            } else {
                false
            }
        });
    if !recorded {
        return Err(EffectError::Illegal(
            "active target or logical card category was already used in this uniqueness window"
                .into(),
        ));
    }
    discard_cards(tx, player, vec![card])
}

pub(crate) fn record_selected_play_phase_category(
    tx: &mut Transaction,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    let instance = state
        .skills
        .get(skill)
        .ok_or_else(|| EffectError::ContentFault("missing originating active skill".into()))?;
    let contract = state
        .content
        .first_active_for_skill(instance.def)
        .ok_or_else(|| EffectError::ContentFault("missing originating active contract".into()))?;
    if !state.skill_effectively_enabled(instance)
        || state.turn.current != instance.owner
        || state.turn.phase != Phase::Play
        || !contract.selection.as_ref().is_some_and(|selection| {
            selection.min == 1
                && selection.max == 1
                && selection.unused_category_by_skill_this_play_phase
        })
    {
        return Err(EffectError::ContentFault(
            "play-phase category effect requires its declared active contract".into(),
        ));
    }
    let owner = instance.owner;
    if !crate::query::discardable_hand_equip_cards(state, owner).contains(&card) {
        return Err(EffectError::Illegal(
            "selected card is no longer legal for play-phase category recording".into(),
        ));
    }
    let category_bit = state
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(state, owner, card))
        .map(|kind| {
            if kind.is_basic() {
                1
            } else if kind.is_trick() {
                2
            } else if kind.is_equip() {
                4
            } else {
                0
            }
        })
        .unwrap_or(0);
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .expect("originating active skill validated");
    if category_bit == 0 || instance.play_phase_category_mask & category_bit != 0 {
        return Err(EffectError::Illegal(
            "logical card category was already used in this play phase".into(),
        ));
    }
    instance.play_phase_category_mask |= category_bit;
    Ok(Vec::new())
}

pub(crate) fn choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    toggle_link: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
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
    let count = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.use_count)
        .unwrap_or(0);
    if count == 0 {
        return Err(EffectError::ContentFault(
            "active draw-or-discard effect requires a recorded use".into(),
        ));
    }
    if crate::query::discardable_hand_equip_cards(tx.state(), target).is_empty() {
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::FinishLinkIfTargetWounded {
                player,
                target,
                toggle_link,
                skill,
            },
        );
        return draw_cards(tx, target, count);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded,
        vec![
            0,
            player.raw(),
            target.raw(),
            skill.raw(),
            count,
            u32::from(toggle_link),
        ],
    )
}

pub(crate) fn finish_link_if_target_wounded(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    toggle_link: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target_wounded = tx.state().players.get(target).is_some_and(|participant| {
        participant.alive && i64::from(participant.hp) < i64::from(participant.max_hp)
    });
    let player_unlinked = tx.state().players.get(player).is_some_and(|participant| {
        participant.alive
            && !participant
                .marks
                .iter()
                .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
    });
    if !target_wounded {
        Ok(Vec::new())
    } else if toggle_link && !player_unlinked {
        remove_mark(tx, player, crate::content::MARK_CHAIN, 0)
    } else if player_unlinked {
        add_mark(tx, player, crate::content::MARK_CHAIN, 1)
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn choose_reveal_other_hand_card_remember_category(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player && tx.state().hand_count(target) > 0)
        .map(PlayerId::raw)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseRevealOtherHandCardRememberCategory,
        vec![0, player.raw(), skill.raw()],
    )
}

pub(crate) fn choose_reveal_hand_card_bind_category_target(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != target
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .into_iter()
        .map(CardId::raw)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseRevealOtherHandCardRememberCategory,
        vec![2, player.raw(), skill.raw(), target.raw()],
    )
}

pub(crate) fn recover_to_max_then_optional_exchange_skill(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    removable_skill_def: engine_api::SkillDefId,
    restore_skill_def: engine_api::SkillDefId,
    ready_to_choose: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if ready_to_choose {
        if restore_skill_def.raw() == 0
            || !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
        {
            return Ok(Vec::new());
        }
        let has_removable = tx.state().players.get(player).is_some_and(|participant| {
            participant.skills.iter().any(|skill| {
                tx.state()
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def == removable_skill_def)
            })
        });
        if !has_removable {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Confirm,
            ContinuationKind::RecoverToMaxThenOptionalExchangeSkill,
            vec![removable_skill_def.raw(), restore_skill_def.raw()],
        );
    }

    let Some(instance) = tx.state().skills.get(skill).cloned() else {
        return Ok(Vec::new());
    };
    if instance.owner != player || !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let Some(participant) = tx
        .state()
        .players
        .get(player)
        .filter(|participant| participant.alive)
    else {
        return Ok(Vec::new());
    };
    let amount = (i64::from(participant.max_hp) - i64::from(participant.hp))
        .max(0)
        .min(i64::from(u32::MAX)) as u32;
    let restore_skill_def = instance.def;
    let mut events = remove_skill(tx, skill)?;
    tx.state_mut().resolution.deferred_tail_effects.push(
        EffectRequest::RecoverToMaxThenOptionalExchangeSkill {
            player,
            skill: engine_api::SkillId::NONE,
            removable_skill_def,
            restore_skill_def,
            ready_to_choose: true,
        },
    );
    events.extend(recover_hp(tx, player, player, amount)?);
    Ok(events)
}

pub(crate) fn draw_then_give_exact_record_phase_target(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
    ready_to_give: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let valid = originating_skill_is_enabled(tx, player, skill)
        && count > 0
        && player != target
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        && tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        && tx.state().turn.current == target
        && tx.state().turn.phase == Phase::Play;
    if !valid {
        return Ok(Vec::new());
    }
    if !ready_to_give {
        let events = draw_cards(tx, player, count)?;
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::DrawThenGiveExactRecordPhaseTarget {
                player,
                target,
                count,
                skill,
                ready_to_give: true,
            },
        );
        return Ok(events);
    }
    let mut options: Vec<CardId> = [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| tx.state().zones.find(kind, player))
        .flat_map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|state| state.cards.clone())
                .unwrap_or_default()
        })
        .collect();
    options.sort_by_key(|card| card.raw());
    options.dedup();
    if options.len() < count as usize || count > u8::MAX as u32 {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        count as u8,
        count as u8,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::GiveExactRecordPhaseTarget,
        vec![player.raw(), target.raw(), skill.raw(), count],
    )
}

pub(crate) fn hand_cards_sorted(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let mut cards = tx.state().hand_cards(player);
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

pub(crate) fn discardable_hand_cards_sorted(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let mut cards: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), player)
        .into_iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|state| Some(state.zone) == hand)
        })
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

pub(crate) fn standard_ganglie_discardable_hand_cards(
    tx: &Transaction,
    player: PlayerId,
) -> Vec<CardId> {
    discardable_hand_cards_sorted(tx, player)
}

pub(crate) fn ganglie_discardable_cards(
    tx: &Transaction,
    player: PlayerId,
    hand_only: bool,
) -> Vec<CardId> {
    if hand_only {
        return standard_ganglie_discardable_hand_cards(tx, player);
    }
    let mut cards = crate::query::discardable_hand_equip_cards(tx.state(), player);
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

pub(crate) fn open_draw_give_or_discard_hand_discard(
    tx: &mut Transaction,
    player: PlayerId,
    payment_count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = discardable_hand_cards_sorted(tx, player);
    let required = options.len().min(payment_count as usize);
    if required == 0 {
        return Ok(Vec::new());
    }
    if required == options.len() {
        return discard_cards(tx, player, options);
    }
    open_choice(
        tx,
        player,
        required as u8,
        required as u8,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::DrawThenChooseGiveOrDiscardHand,
        vec![3, player.raw(), required as u32, skill.raw()],
    )
}
