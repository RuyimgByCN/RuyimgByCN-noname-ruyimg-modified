//! Command operations for `skill_card_target_spec_with_options` and related helpers.

use super::*;

/// Resolve the ordinary target shape for a hand card used by a skill, optionally ignoring
/// distance while preserving every other target and use-count rule.
pub(crate) fn skill_card_target_spec_with_options(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    ignore_distance: bool,
    ignore_use_count: bool,
) -> Result<SkillCardTargetSpec, EffectError> {
    let player = tx
        .state()
        .players
        .get(actor)
        .ok_or_else(|| EffectError::Illegal("bad actor".into()))?;
    if !player.alive || player.dying {
        return Err(EffectError::Illegal("actor not able to play".into()));
    }
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad card".into()))?;
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let hand = tx.state().zones.find(ZoneKind::Hand, actor);
    let equip = tx.state().zones.find(ZoneKind::Equip, actor);
    let playable_as_hand = tx.state().playable_hand_cards(actor).contains(&card);
    let view_as_material =
        tx.state().turn.flag(crate::state::turn_flag::VIEW_AS_CARD) == card.raw() as i32;
    if !((card_state.zone == processing && card_state.owner.is_none())
        || (hand == Some(card_state.zone) && card_state.owner == actor)
        || (view_as_material && equip == Some(card_state.zone) && card_state.owner == actor)
        || playable_as_hand)
    {
        return Err(EffectError::Illegal(
            "skill card not in processing or user hand".into(),
        ));
    }
    if crate::query::card_use_cancelled_for_card(tx.state(), actor, card) {
        return Err(EffectError::Illegal("card use disabled".into()));
    }
    let kind = crate::query::effective_card_kind(tx.state(), actor, card_state);
    let alive = tx.state().players.alive_by_seat();
    let target_allowed = |target: PlayerId| {
        validate_explicit_target_policy(tx, actor, card_state, kind, &[target]).is_ok()
    };
    let select_one = |options: Vec<PlayerId>| {
        if options.is_empty() {
            Err(EffectError::Illegal("card has no legal target".into()))
        } else {
            Ok(SkillCardTargetSpec::Select {
                min: 1,
                max: 1,
                options,
            })
        }
    };
    let select_independent = |options: Vec<PlayerId>| {
        if options.is_empty() {
            return Err(EffectError::Illegal("card has no legal target".into()));
        }
        let max = crate::query::card_target_limit_for_card(tx.state(), actor, card, 1)
            .min(options.len() as u32)
            .min(u32::from(u8::MAX)) as u8;
        if max == 0 {
            Err(EffectError::Illegal("card has no legal target".into()))
        } else {
            Ok(SkillCardTargetSpec::Select {
                min: 1,
                max,
                options,
            })
        }
    };

    match kind {
        CardKind::Ying => Err(EffectError::Illegal("shadow card cannot be played".into())),
        CardKind::Shan | CardKind::TrickWuxie => {
            Err(EffectError::Illegal("response-only card".into()))
        }
        CardKind::Sha | CardKind::ShaFire | CardKind::ShaThunder => {
            let within_use_count = ignore_use_count
                || crate::query::sha_play_phase_use_count(tx.state())
                    < crate::query::sha_use_limit_for_card(tx.state(), actor, card);
            let options: Vec<_> = alive
                .iter()
                .copied()
                .filter(|target| {
                    within_use_count
                        || crate::query::sha_use_unlimited_for_target(tx.state(), actor, *target)
                })
                .filter(|target| {
                    validate_sha_target_with_options(tx, actor, card, *target, ignore_distance)
                        .is_ok()
                })
                .collect();
            if crate::query::sha_targets_all_for(tx.state(), actor) {
                if options.is_empty() {
                    Err(EffectError::Illegal("sha has no legal target".into()))
                } else {
                    Ok(SkillCardTargetSpec::Immediate(options))
                }
            } else {
                let max = usize::try_from(crate::query::sha_target_limit_for(tx.state(), actor))
                    .unwrap_or(usize::MAX)
                    .min(options.len())
                    .min(u8::MAX as usize) as u8;
                if max == 0 {
                    Err(EffectError::Illegal("sha has no legal target".into()))
                } else {
                    Ok(SkillCardTargetSpec::Select {
                        min: 1,
                        max,
                        options,
                    })
                }
            }
        }
        CardKind::Tao => {
            if player.hp >= player.max_hp as i32 {
                return Err(EffectError::Illegal("hp full".into()));
            }
            validate_explicit_target_policy(tx, actor, card_state, kind, &[actor])?;
            Ok(SkillCardTargetSpec::Immediate(vec![actor]))
        }
        CardKind::Jiu => {
            validate_explicit_target_policy(tx, actor, card_state, kind, &[actor])?;
            Ok(SkillCardTargetSpec::Immediate(vec![actor]))
        }
        CardKind::Weapon
        | CardKind::Armor
        | CardKind::HorseOff
        | CardKind::HorseDef
        | CardKind::Treasure
        | CardKind::TrickWuzhong
        | CardKind::TrickYoudishenru
        | CardKind::Other => Ok(SkillCardTargetSpec::Immediate(Vec::new())),
        CardKind::TrickGuohe => select_independent(
            alive
                .iter()
                .copied()
                .filter(|target| target_allowed(*target))
                .filter(|target| {
                    !discardable_target_zone_card_tokens(tx, actor, *target).is_empty()
                })
                .collect(),
        ),
        CardKind::TrickShunshou => select_independent(
            alive
                .iter()
                .copied()
                .filter(|target| validate_alive_other(tx, actor, *target).is_ok())
                .filter(|target| target_allowed(*target))
                .filter(|target| {
                    ignore_distance
                        || crate::query::card_target_within_distance(
                            tx.state(),
                            actor,
                            card,
                            *target,
                            1,
                        )
                })
                .filter(|target| {
                    !crate::query::target_immune(
                        tx.state(),
                        *target,
                        crate::query::ModifierTarget::ImmuneShunshou,
                    )
                })
                .filter(|target| {
                    !target_zone_card_tokens(tx, *target, true, true, false).is_empty()
                })
                .collect(),
        ),
        CardKind::TrickHuogong => select_independent(
            alive
                .iter()
                .copied()
                .filter(|target| target_allowed(*target))
                .filter(|target| !tx.state().hand_cards(*target).is_empty())
                .collect(),
        ),
        CardKind::TrickJuedou => select_independent(
            alive
                .iter()
                .copied()
                .filter(|target| validate_alive_other(tx, actor, *target).is_ok())
                .filter(|target| target_allowed(*target))
                .collect(),
        ),
        CardKind::TrickYuanjiao => select_one(
            alive
                .iter()
                .copied()
                .filter(|target| validate_alive_other(tx, actor, *target).is_ok())
                .filter(|target| target_allowed(*target))
                .filter(|target| {
                    let actor_group = tx.state().player_group(actor);
                    let target_group = tx.state().player_group(*target);
                    actor_group.is_none() || actor_group != target_group
                })
                .collect(),
        ),
        CardKind::TrickWangmeizhike => select_one(
            alive
                .iter()
                .copied()
                .filter(|target| target_allowed(*target))
                .collect(),
        ),
        CardKind::TrickChuqibuyi => select_one(
            alive
                .iter()
                .copied()
                .filter(|target| {
                    *target != actor
                        && target_allowed(*target)
                        && tx.state().hand_count(*target) > 0
                })
                .collect(),
        ),
        CardKind::TrickYiyi => {
            // 以逸待劳（国战）：自动目标 = 己方全体（同势力或野心联盟，含自己）。
            let allies: Vec<_> = alive
                .iter()
                .copied()
                .filter(|candidate| {
                    let mine = tx.state().player_group(actor).map(str::to_string);
                    let theirs = tx.state().player_group(*candidate).map(str::to_string);
                    (mine.is_some() && mine == theirs)
                        || crate::mode::are_allies(tx.state(), actor, *candidate)
                })
                .collect();
            if allies.is_empty() {
                Err(EffectError::Illegal("以逸待劳 has no ally target".into()))
            } else {
                Ok(SkillCardTargetSpec::Immediate(allies))
            }
        }
        CardKind::TrickShuiyanqijunx => select_one(
            alive
                .iter()
                .copied()
                .filter(|target| *target != actor)
                .filter(|target| target_allowed(*target))
                .filter(|target| {
                    tx.state()
                        .zones
                        .find(crate::state::ZoneKind::Equip, *target)
                        .and_then(|zone| tx.state().zones.get(zone))
                        .is_some_and(|zone| !zone.cards.is_empty())
                })
                .collect(),
        ),
        CardKind::TrickLulitongxin => select_one(
            alive
                .iter()
                .copied()
                .filter(|target| target_allowed(*target))
                .collect(),
        ),
        CardKind::TrickKefuzhongyuan => {
            let options: Vec<_> = alive
                .iter()
                .copied()
                .filter(|target| target_allowed(*target))
                .collect();
            if options.is_empty() {
                Err(EffectError::Illegal("克复中原 has no target".into()))
            } else {
                Ok(SkillCardTargetSpec::Select {
                    min: 1,
                    max: options.len().min(u8::MAX as usize) as u8,
                    options,
                })
            }
        }
        CardKind::TrickGuguoanbang => {
            validate_explicit_target_policy(tx, actor, card_state, kind, &[actor])?;
            Ok(SkillCardTargetSpec::Immediate(vec![actor]))
        }
        CardKind::TrickHuoshaolianying => {
            // 火烧连营（国战）：目标自动 = 下家及其队列，无需选择。
            Ok(SkillCardTargetSpec::Immediate(Vec::new()))
        }
        CardKind::TrickLianjunshengyan => select_one(
            alive
                .iter()
                .copied()
                .filter(|target| *target != actor)
                .filter(|target| target_allowed(*target))
                .filter(|target| !crate::mode::same_side(tx.state(), actor, *target))
                .collect(),
        ),
        CardKind::TrickWenheluanwu => {
            // 文和乱武（国战）：目标自动 = 除自己外全部有手牌角色，无需选择。
            Ok(SkillCardTargetSpec::Immediate(Vec::new()))
        }
        CardKind::TrickHaolingtianxia => {
            // 号令天下（国战）：选一名体力值非全场最少的其他角色。
            let min_hp = alive
                .iter()
                .filter_map(|candidate| tx.state().players.get(*candidate))
                .map(|player| player.hp)
                .min()
                .unwrap_or(i32::MAX);
            select_one(
                alive
                    .iter()
                    .copied()
                    .filter(|target| *target != actor)
                    .filter(|target| target_allowed(*target))
                    .filter(|target| {
                        tx.state()
                            .players
                            .get(*target)
                            .is_some_and(|player| player.hp > min_hp)
                    })
                    .collect(),
            )
        }
        CardKind::TrickChiling => {
            // 敕令（国战）：目标自动 = 所有有暗置武将的角色（含使用者），无需选择。
            Ok(SkillCardTargetSpec::Immediate(Vec::new()))
        }
        CardKind::TrickTiaojiyanmei => {
            let options: Vec<_> = alive
                .iter()
                .copied()
                .filter(|target| *target != actor)
                .filter(|target| target_allowed(*target))
                .collect();
            if options.iter().any(|left| {
                options.iter().any(|right| {
                    left != right && tx.state().hand_count(*left) != tx.state().hand_count(*right)
                })
            }) {
                Ok(SkillCardTargetSpec::Select {
                    min: 2,
                    max: 2,
                    options,
                })
            } else {
                Err(EffectError::Illegal(
                    "调剂盐梅 has no legal target pair".into(),
                ))
            }
        }
        CardKind::TrickMantianguohai => {
            let options: Vec<_> = alive
                .iter()
                .copied()
                .filter(|target| *target != actor)
                .filter(|target| target_allowed(*target))
                .filter(|target| {
                    !crate::query::gainable_target_cards(
                        tx.state(),
                        actor,
                        *target,
                        rule_api::TargetCardScope::HandEquipJudge,
                    )
                    .is_empty()
                })
                .collect();
            if options.is_empty() {
                Err(EffectError::Illegal("瞒天过海 has no legal target".into()))
            } else {
                Ok(SkillCardTargetSpec::Select {
                    min: 1,
                    max: 2,
                    options,
                })
            }
        }
        CardKind::TrickNanman | CardKind::TrickWanjian => {
            reject_other_targeting_card_if_self_only(tx, actor)?;
            let targets: Vec<_> = alive
                .into_iter()
                .filter(|target| *target != actor)
                .collect();
            validate_explicit_target_policy(tx, actor, card_state, kind, &targets)?;
            Ok(SkillCardTargetSpec::Immediate(targets))
        }
        CardKind::TrickTaoyuan | CardKind::TrickWugu | CardKind::TrickDiaobingqianjiang => {
            let targets = if crate::query::card_targets_self_only_for(tx.state(), actor) {
                vec![actor]
            } else {
                let mut targets = alive;
                if kind == CardKind::TrickDiaobingqianjiang {
                    if let Some(start) = targets.iter().position(|player| *player == actor) {
                        targets.rotate_left(start);
                    }
                    targets.retain(|target| *target == actor || tx.state().hand_count(*target) > 0);
                }
                targets
            };
            validate_explicit_target_policy(tx, actor, card_state, kind, &targets)?;
            Ok(SkillCardTargetSpec::Immediate(targets))
        }
        CardKind::TrickJiedao => {
            let has_weapon = |holder: PlayerId| {
                tx.state()
                    .zones
                    .find(ZoneKind::Equip, holder)
                    .and_then(|zone| tx.state().zones.get(zone))
                    .is_some_and(|zone| {
                        zone.cards.iter().any(|card| {
                            tx.state()
                                .cards
                                .get(*card)
                                .is_some_and(|card| card.kind == CardKind::Weapon)
                        })
                    })
            };
            let mut pairs = Vec::new();
            for holder in alive.iter().copied().filter(|holder| {
                validate_alive_other(tx, actor, *holder).is_ok() && has_weapon(*holder)
            }) {
                for victim in alive
                    .iter()
                    .copied()
                    .filter(|victim| *victim != actor && *victim != holder)
                {
                    if validate_explicit_target_policy(
                        tx,
                        actor,
                        card_state,
                        kind,
                        &[holder, victim],
                    )
                    .is_ok()
                    {
                        pairs.push((holder, victim));
                    }
                }
            }
            if pairs.is_empty() {
                Err(EffectError::Illegal("jiedao has no legal targets".into()))
            } else {
                Ok(SkillCardTargetSpec::Pairs(pairs))
            }
        }
        CardKind::TrickTiesuo => {
            let options = alive
                .into_iter()
                .filter(|target| target_allowed(*target))
                .collect::<Vec<_>>();
            Ok(SkillCardTargetSpec::Select {
                min: 0,
                max: options
                    .len()
                    .min(
                        crate::query::card_target_limit_for_card(tx.state(), actor, card, 2)
                            as usize,
                    )
                    .min(u8::MAX as usize) as u8,
                options,
            })
        }
        CardKind::TrickLe | CardKind::TrickBingliang => select_one(
            alive
                .iter()
                .copied()
                .filter(|target| validate_alive_other(tx, actor, *target).is_ok())
                .filter(|target| target_allowed(*target))
                .filter(|target| {
                    ignore_distance
                        || kind != CardKind::TrickBingliang
                        || crate::query::card_target_within_distance(
                            tx.state(),
                            actor,
                            card,
                            *target,
                            1,
                        )
                })
                .filter(|target| {
                    kind != CardKind::TrickLe
                        || !crate::query::target_immune(
                            tx.state(),
                            *target,
                            crate::query::ModifierTarget::ImmuneLe,
                        )
                })
                .filter(|target| !tx.state().disabled_judge_zones.contains(target))
                .filter(|target| !has_delayed_of_kind(tx, *target, kind))
                .collect(),
        ),
        CardKind::TrickShandian => {
            if tx.state().disabled_judge_zones.contains(&actor) {
                return Err(EffectError::Illegal("judgment zone is disabled".into()));
            }
            if has_delayed_of_kind(tx, actor, kind) {
                return Err(EffectError::Illegal("already has lightning".into()));
            }
            validate_explicit_target_policy(tx, actor, card_state, kind, &[actor])?;
            Ok(SkillCardTargetSpec::Immediate(vec![actor]))
        }
    }
}

/// Use a shared-processing card through the ordinary card-resolution pipeline.
pub(crate) fn handle_skill_card(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    handle_skill_card_with_options(tx, actor, card, targets, false)
}

/// Use a shared-processing card while optionally ignoring ordinary distance.
pub(crate) fn handle_skill_card_with_options(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
) -> Result<(), EffectError> {
    handle_skill_card_with_policies(tx, actor, card, targets, ignore_distance, false)
}

/// Use a shared-processing card with explicit distance and use-count policies.
pub(crate) fn handle_skill_card_with_policies(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
    ignore_use_count: bool,
) -> Result<(), EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad skill card".into()))?;
    if card_state.zone != processing || card_state.owner.is_some() {
        return Err(EffectError::Illegal(
            "skill card not in shared processing zone".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, actor)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    crate::effect::move_one_card(tx, card, processing, hand, actor, false)?;
    handle_card_from_hand_with_policies(
        tx,
        actor,
        card,
        targets,
        ignore_distance,
        ignore_use_count,
        false,
    )
}

/// Use a materialized virtual Sha with explicit use policies and a fixed base-damage bonus.
pub(crate) fn handle_virtual_sha_with_policies_and_bonus(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
    ignore_use_count: bool,
    damage_bonus: u32,
) -> Result<(), EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, actor)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    crate::effect::move_one_card(tx, card, processing, hand, actor, false)?;
    play_as_kind_with_extra_damage_flags(
        tx,
        actor,
        card,
        CardKind::Sha,
        targets,
        ignore_distance,
        false,
        &[],
        ignore_use_count,
        false,
        0,
        &[],
        damage_bonus,
    )
}

/// Play a hand card as a different kind (view-as). Consumes the source card.
pub(crate) fn play_as_kind(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    as_kind: CardKind,
    targets: &[PlayerId],
    ignore_distance: bool,
    force_unrespondable: bool,
    ignore_use_count: bool,
    source_was_hand: bool,
) -> Result<(), EffectError> {
    play_as_kind_with_extra_damage_flags(
        tx,
        actor,
        card,
        as_kind,
        targets,
        ignore_distance,
        force_unrespondable,
        &[],
        ignore_use_count,
        source_was_hand,
        0,
        &[],
        0,
    )
}

/// Use a physical equipment/judgment card as an independent trick without first gaining it.
pub(crate) fn play_field_card_as_kind(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    as_kind: CardKind,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    if !matches!(
        as_kind,
        CardKind::TrickGuohe
            | CardKind::TrickShunshou
            | CardKind::TrickHuogong
            | CardKind::TrickJuedou
    ) {
        return Err(EffectError::ContentFault(
            "field view-as only supports independent tricks".into(),
        ));
    }
    let source = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad field card".into()))?
        .clone();
    if crate::query::card_use_cancelled_for_logical_kind(tx.state(), actor, &source, as_kind) {
        return Err(EffectError::Illegal("card use disabled".into()));
    }
    if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
        card_state.kind = as_kind;
    }
    handle_card_with_policies_and_flags(tx, actor, card, targets, true, true, false, 0, true)
}
