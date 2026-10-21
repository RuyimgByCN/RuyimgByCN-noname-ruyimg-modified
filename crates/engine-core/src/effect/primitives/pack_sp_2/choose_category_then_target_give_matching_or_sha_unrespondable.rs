//! Effect primitives for `choose_category_then_target_give_matching_or_sha_unrespondable` and related helpers.

use super::*;

pub(crate) fn choose_category_then_target_give_matching_or_sha_unrespondable(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
    {
        return Ok(Vec::new());
    }
    crate::command::play::validate_saved_sha_target(tx, player, card, target)?;
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2, 3],
        ChoiceKind::Custom,
        ContinuationKind::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable,
        vec![0, player.raw(), card.raw(), target.raw(), skill.raw()],
    )
}

pub(crate) fn choose_category_then_target_give_matching_or_damage_plus_one(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    damage_id: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || damage_id == 0
        || tx.state().resolution.current_damage_id != damage_id
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
    {
        return Ok(Vec::new());
    }
    let has_hand_equip = [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| tx.state().zones.find(kind, target))
        .filter_map(|zone| tx.state().zones.get(zone))
        .any(|zone| !zone.cards.is_empty());
    if !has_hand_equip {
        increase_bound_damage_by_one(tx, damage_id)?;
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2, 3],
        ChoiceKind::Custom,
        ContinuationKind::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable,
        vec![
            0,
            player.raw(),
            damage_id,
            target.raw(),
            skill.raw(),
            CATEGORY_GIFT_DAMAGE_PLUS_ONE_MODE,
        ],
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn choose_lose_hp_use_virtual_sha_by_basic_variants(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    stage: u8,
    damage_bonus: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let damage_bonus = u32::from(tx.state().turn.distinct_basic_card_variants(player) > 1);
            choose_use_virtual_sha(tx, player, true, true, false, skill, 1, damage_bonus)
        }
        1 => {
            let processing = tx.state().zones.find(ZoneKind::Processing, PlayerId::NONE);
            let card_ready = tx
                .state()
                .cards
                .get(card)
                .is_some_and(|state| processing == Some(state.zone) && state.owner.is_none());
            if !card_ready {
                return Ok(Vec::new());
            }
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|state| state.alive && !state.dying)
            {
                let removed = tx
                    .state()
                    .zones
                    .find(ZoneKind::Removed, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("removed zone missing".into()))?;
                move_one_card(
                    tx,
                    card,
                    processing.expect("card_ready requires processing"),
                    removed,
                    PlayerId::NONE,
                    true,
                )?;
                return Ok(Vec::new());
            }
            crate::command::play::handle_virtual_sha_with_policies_and_bonus(
                tx,
                player,
                card,
                &targets,
                true,
                true,
                damage_bonus,
            )?;
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown lose-HP virtual Sha stage".into(),
        )),
    }
}

/// 叶舞（gwent, ABI 645, ADR 0627）：弃置费用手牌后按种子 RNG 循环——随机敌人随机弃
/// 其一张「he」牌，弃牌颜色与费用相同则重复；每以此法弃敌方手牌加一张雾妖
/// （wuyao_def），每弃敌方装备牌加一张狼（lang_def）。雾妖/狼以技能近似（add_skill，
/// 造卡入口）。循环内模拟已弃牌（从候选中排除），保证终止且消耗 RNG 确定性。
pub(crate) fn resolve_gwyewu_chain(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    cost_cards: Vec<engine_api::CardId>,
    wuyao_def: engine_api::SkillDefId,
    lang_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let he_of = |tx: &Transaction, who: PlayerId| -> Vec<(engine_api::CardId, bool)> {
        let mut out = Vec::new();
        for cid in tx.state().hand_cards(who) {
            out.push((cid, true));
        }
        if let Some(eq) = tx.state().zones.find(ZoneKind::Equip, who) {
            if let Some(zone) = tx.state().zones.get(eq) {
                for cid in &zone.cards {
                    out.push((*cid, false));
                }
            }
        }
        out
    };
    // 费用：弃置所选手牌并记录颜色。
    let mut effects: Vec<EffectRequest> = Vec::new();
    let mut cost_color = 0u8;
    if let Some(cost) = cost_cards.first().copied() {
        if tx.state().hand_cards(player).contains(&cost) {
            cost_color = crate::query::effective_card_color_mask(tx.state(), player, cost);
            effects.push(EffectRequest::DiscardCards {
                player,
                cards: vec![cost],
            });
        }
    }
    // 循环：随机敌人 → 随机 he 牌 → 弃 → 颜色相同则重复（模拟弃牌从候选排除）。
    let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
    let mut nh = 0u32;
    let mut ne = 0u32;
    let mut discarded: Vec<engine_api::CardId> = Vec::new();
    let mut per_owner: Vec<(PlayerId, Vec<engine_api::CardId>)> = Vec::new();
    loop {
        let candidates: Vec<PlayerId> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|candidate| *candidate != player)
            .filter(|candidate| !crate::mode::same_side(tx.state(), player, *candidate))
            .filter(|candidate| {
                he_of(tx, *candidate)
                    .iter()
                    .any(|(cid, _)| !discarded.contains(cid))
            })
            .collect();
        if candidates.is_empty() {
            break;
        }
        let enemy = candidates[rng.gen_range_u32(candidates.len() as u32) as usize];
        let available: Vec<_> = he_of(tx, enemy)
            .into_iter()
            .filter(|(cid, _)| !discarded.contains(cid))
            .collect();
        let (cid, in_hand) = available[rng.gen_range_u32(available.len() as u32) as usize];
        if in_hand {
            nh += 1;
        } else {
            ne += 1;
        }
        discarded.push(cid);
        if let Some((_, cards)) = per_owner.iter_mut().find(|(owner, _)| *owner == enemy) {
            cards.push(cid);
        } else {
            per_owner.push((enemy, vec![cid]));
        }
        let color = crate::query::effective_card_color_mask(tx.state(), enemy, cid);
        if cost_color == 0 || color != cost_color {
            break;
        }
    }
    tx.state_mut().rng = rng.into_state();
    for (owner, cards) in per_owner {
        effects.push(EffectRequest::DiscardCards {
            player: owner,
            cards,
        });
    }
    for _ in 0..nh {
        effects.push(EffectRequest::AddSkill {
            player,
            skill_def: wuyao_def,
        });
    }
    for _ in 0..ne {
        effects.push(EffectRequest::AddSkill {
            player,
            skill_def: lang_def,
        });
    }
    run_effects(tx, effects)
}

/// 起乱 (qiluan): draw Σ alive players' current-turn kill counts × (3 if the killer is
/// the owner else 1). 何太后 end-phase draw, 上游 skill.js:42081.
pub(crate) fn resolve_qiluan(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let mut total: u32 = 0;
    for killed in tx.state().players.iter_live() {
        if !killed.alive {
            continue;
        }
        let kills = tx
            .state()
            .turn
            .flag(turn_flag::kill_count_by_player(killed.id));
        if kills > 0 {
            total = total.saturating_add(if killed.id == player {
                (kills as u32).saturating_mul(3)
            } else {
                kills as u32
            });
        }
    }
    if total == 0 {
        return Ok(Vec::new());
    }
    let _ = skill;
    draw_cards(tx, player, total)
}

/// 灵杉 (huamu): store the triggering event's used card onto the owner's skill pile
/// (武将牌) when the card currently sits in the shared discard pile. 上游 skill.js:24085.
pub(crate) fn store_event_card_in_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let skill_def = crate::effect::primitives::pack_bingshi_1::skill_pile_def(tx, player, skill)?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let in_discard = tx
        .state()
        .cards
        .get(card)
        .is_some_and(|state| state.zone == discard);
    if !in_discard {
        return Ok(Vec::new());
    }
    let pile =
        crate::effect::primitives::pack_bingshi_1::ensure_skill_pile_zone(tx, player, skill_def)?;
    move_cards(tx, vec![card], pile, Some(discard))
}
