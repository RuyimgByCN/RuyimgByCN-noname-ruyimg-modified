//! Command operations for `handle_card_with_policies_and_flags` and related helpers.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_card_with_policies_and_flags(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
    ignore_use_count: bool,
    source_was_hand: bool,
    extra_damage_flags: u8,
    field_source: bool,
) -> Result<(), EffectError> {
    let player = tx
        .state()
        .players
        .get(actor)
        .ok_or_else(|| EffectError::Illegal("bad actor".into()))?
        .clone();
    if !player.alive || player.dying {
        return Err(EffectError::Illegal("actor not able to play".into()));
    }

    let c = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad card".into()))?
        .clone();
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, actor)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    let valid_source = if field_source {
        tx.state()
            .zones
            .get(c.zone)
            .is_some_and(|zone| matches!(zone.kind, ZoneKind::Equip | ZoneKind::Judge))
    } else {
        tx.state().playable_hand_cards(actor).contains(&card)
            || (tx.state().turn.flag(crate::state::turn_flag::VIEW_AS_CARD) == card.raw() as i32
                && tx
                    .state()
                    .zones
                    .get(c.zone)
                    .is_some_and(|zone| zone.kind == ZoneKind::Equip && zone.owner == actor))
    };
    if !valid_source {
        return Err(EffectError::Illegal("card not in hand".into()));
    }
    if crate::query::card_use_cancelled_for_card(tx.state(), actor, card) {
        return Err(EffectError::Illegal("card use disabled".into()));
    }
    let kind = crate::query::effective_card_kind(tx.state(), actor, &c);
    validate_explicit_target_policy(tx, actor, &c, kind, targets)?;
    let source_counts_as_hand =
        c.zone == hand || crate::query::borrowed_hand_source(tx.state(), actor, card).is_some();
    remember_hand_card_suit_use(tx, actor, card, source_was_hand && source_counts_as_hand);

    let mut queue: Vec<EffectRequest> = Vec::new();

    match kind {
        // 影不可响应任何窗口（无队列效果）。
        CardKind::Ying => {}
        CardKind::Shan => {
            return Err(EffectError::Illegal("cannot play shan proactively".into()));
        }
        CardKind::Sha | CardKind::ShaFire | CardKind::ShaThunder => {
            let ignores_use_count = ignore_use_count
                || crate::query::sha_use_does_not_count_for_card(tx.state(), actor, card);
            let respondable = !crate::query::sha_unrespondable_for_card(tx.state(), actor, card);
            if !ignores_use_count && !sha_use_count_allows_targets(tx.state(), actor, card, targets)
            {
                return Err(EffectError::Illegal("sha use limit reached".into()));
            }
            validate_sha_targets_with_options(tx, actor, card, targets, ignore_distance)?;
            let nature = crate::query::effective_card_nature(tx.state(), actor, &c, kind);
            let mut amount = 1u32;
            let jiu = player
                .marks
                .iter()
                .any(|(m, c)| *m == crate::content::MARK_JIU && *c > 0);
            let card_flags = (if jiu {
                rule_api::damage_card_flag::JIU_BOOSTED
            } else {
                0
            }) | extra_damage_flags;
            if jiu {
                amount = 2;
                queue.push(EffectRequest::RemoveMark {
                    player: actor,
                    mark: engine_api::MarkDefId::new(crate::content::MARK_JIU),
                    amount: 1,
                });
            }
            // Apply jiu mark remove + use card first
            queue.push(EffectRequest::UseCard {
                player: actor,
                card,
                targets: targets.to_vec(),
            });
            // Record that the current player played a 杀 this turn (克己).
            if !ignores_use_count {
                record_counted_sha_use(tx);
            }
            let resume_flags = saved_sha_flags(tx, respondable, !ignores_use_count);
            let resume = if targets.len() == 1 {
                (
                    crate::state::ContinuationKind::ResumePlayedSha,
                    vec![
                        actor.raw(),
                        targets[0].raw(),
                        amount,
                        pack_damage_card_context(nature, card_flags),
                        resume_flags,
                        card.raw(),
                    ],
                )
            } else {
                let mut payload = vec![
                    actor.raw(),
                    amount,
                    pack_damage_card_context(nature, card_flags),
                    resume_flags,
                    card.raw(),
                    targets.len() as u32,
                ];
                payload.extend(targets.iter().map(|target| target.raw()));
                payload.push(u32::MAX);
                payload.extend(std::iter::repeat(1).take(targets.len()));
                (
                    crate::state::ContinuationKind::ResumePlayedShaQueue,
                    payload,
                )
            };
            let (kind, payload) = run_effects_with_saved_resume(tx, queue, resume.0, resume.1)?;
            return resume_played_sha_from_payload(tx, kind, &payload);
        }
        CardKind::Tao => {
            if crate::query::card_target_cancelled_for_logical_kind(
                tx.state(),
                actor,
                actor,
                &c,
                CardKind::Tao,
            ) {
                return Err(EffectError::Illegal("tao target disabled".into()));
            }
            let hp = player.hp;
            let max = player.max_hp as i32;
            if hp >= max {
                return Err(EffectError::Illegal("hp full".into()));
            }
            if !targets.is_empty() && targets != [actor] {
                return Err(EffectError::Illegal("tao self only in play phase".into()));
            }
            queue.push(EffectRequest::UseCard {
                player: actor,
                card,
                targets: vec![actor],
            });
            queue.push(EffectRequest::RecoverHp {
                source: actor,
                player: actor,
                amount: 1,
            });
        }
        CardKind::Jiu => {
            // M1: if wounded → recover 1; else gain JIU mark (next Sha +1 damage).
            queue.push(EffectRequest::UseCard {
                player: actor,
                card,
                targets: vec![actor],
            });
            if player.hp < player.max_hp as i32 {
                queue.push(EffectRequest::RecoverHp {
                    source: actor,
                    player: actor,
                    amount: 1,
                });
            } else {
                queue.push(EffectRequest::AddMark {
                    player: actor,
                    mark: engine_api::MarkDefId::new(crate::content::MARK_JIU),
                    delta: 1,
                });
            }
        }
        CardKind::Weapon
        | CardKind::Armor
        | CardKind::HorseOff
        | CardKind::HorseDef
        | CardKind::Treasure => {
            return equip_card(tx, actor, card, kind);
        }
        CardKind::TrickWuxie | CardKind::TrickYoudishenru => {
            return Err(EffectError::Illegal("card is response only".into()));
        }
        CardKind::TrickWuzhong => {
            return use_card_then_wuxie(tx, actor, card, &[], trick_effect::WUZHONG, &[]);
        }
        CardKind::TrickGuohe => {
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "guohe")?;
            for &target in targets {
                let t = tx
                    .state()
                    .players
                    .get(target)
                    .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
                if !t.alive {
                    return Err(EffectError::Illegal("target dead".into()));
                }
                if discardable_target_zone_card_tokens(tx, actor, target).is_empty() {
                    return Err(EffectError::Illegal("target has nothing to discard".into()));
                }
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                targets,
                trick_effect::GUOHE,
                field_source,
            );
        }
        CardKind::TrickShunshou => {
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "shunshou")?;
            for &target in targets {
                validate_alive_other(tx, actor, target)?;
                if !ignore_distance
                    && !crate::query::card_target_within_distance(
                        tx.state(),
                        actor,
                        card,
                        target,
                        1,
                    )
                {
                    return Err(EffectError::Illegal("target out of 顺手牵羊 range".into()));
                }
                if crate::query::target_immune(
                    tx.state(),
                    target,
                    crate::query::ModifierTarget::ImmuneShunshou,
                ) {
                    return Err(EffectError::Illegal(
                        "target immune to 顺手牵羊 (谦逊)".into(),
                    ));
                }
                if target_zone_card_tokens(tx, target, true, true, false).is_empty() {
                    return Err(EffectError::Illegal("target has nothing to take".into()));
                }
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                targets,
                trick_effect::SHUNSHOU,
                field_source,
            );
        }
        CardKind::TrickHuogong => {
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "huogong")?;
            for &target in targets {
                let t = tx
                    .state()
                    .players
                    .get(target)
                    .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
                if !t.alive {
                    return Err(EffectError::Illegal("target dead".into()));
                }
                if tx.state().hand_cards(target).is_empty() {
                    return Err(EffectError::Illegal("target has no hand".into()));
                }
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                targets,
                trick_effect::HUOGONG,
                field_source,
            );
        }
        CardKind::TrickJuedou => {
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "juedou")?;
            for &target in targets {
                validate_alive_other(tx, actor, target)?;
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                targets,
                trick_effect::JUEDOU,
                field_source,
            );
        }
        CardKind::TrickYuanjiao => {
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "yuanjiao")?;
            let target = targets[0];
            validate_alive_other(tx, actor, target)?;
            if tx.state().disabled_judge_zones.contains(&target) {
                return Err(EffectError::Illegal(
                    "target judgment zone is disabled".into(),
                ));
            }
            let actor_group = tx.state().player_group(actor);
            let target_group = tx.state().player_group(target);
            if actor_group.is_some() && actor_group == target_group {
                return Err(EffectError::Illegal(
                    "远交近攻 requires a different-group target".into(),
                ));
            }
            return use_card_then_wuxie(
                tx,
                actor,
                card,
                targets,
                trick_effect::YUANJIAO,
                &[target.raw()],
            );
        }
        CardKind::TrickWangmeizhike => {
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "wangmeizhike")?;
            let target = targets[0];
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return Err(EffectError::Illegal("望梅止渴 target must be alive".into()));
            }
            return use_card_then_wuxie(
                tx,
                actor,
                card,
                targets,
                trick_effect::WANGMEIZHIKE,
                &[target.raw()],
            );
        }
        CardKind::TrickChuqibuyi => {
            // 出其不意：对一名有手牌的其他角色使用。
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "chuqibuyi")?;
            let target = targets[0];
            if target == actor
                || !tx.state().players.get(target).is_some_and(|p| p.alive)
                || tx.state().hand_count(target) == 0
            {
                return Err(EffectError::Illegal(
                    "chuqibuyi target must be another alive player with hand cards".into(),
                ));
            }
            return use_card_then_wuxie(
                tx,
                actor,
                card,
                targets,
                trick_effect::CHUQIBUYI,
                &[target.raw()],
            );
        }
        CardKind::TrickYiyi => {
            // 以逸待劳（国战）：对己方全体角色使用，各摸两张再弃两张。
            // 己方 = 同势力 或 野心联盟（are_allies）。
            let allies: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    let mine = tx.state().player_group(actor).map(str::to_string);
                    let theirs = tx.state().player_group(*candidate).map(str::to_string);
                    (mine.is_some() && mine == theirs)
                        || crate::mode::are_allies(tx.state(), actor, *candidate)
                })
                .collect();
            if allies.is_empty() {
                return Err(EffectError::Illegal("以逸待劳 has no ally target".into()));
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &allies,
                trick_effect::YIYI,
                field_source,
            );
        }
        CardKind::TrickShuiyanqijunx => {
            // 水淹七军（国战）：对一名装备区里有牌的其他角色使用。
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "shuiyanqijunx")?;
            let target = targets[0];
            let has_equip = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::Equip, target)
                .and_then(|zone| tx.state().zones.get(zone))
                .is_some_and(|zone| !zone.cards.is_empty());
            if target == actor
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|player| player.alive)
                || !has_equip
            {
                return Err(EffectError::Illegal(
                    "水淹七军 target must be another living player with equipment".into(),
                ));
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &[target],
                trick_effect::SHUIYANQIJUNX,
                field_source,
            );
        }
        CardKind::TrickLulitongxin => {
            // 戮力同心（国战）：选一名角色为代表，对与其同为大势力或同为小势力的
            // 全部角色使用。
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "lulitongxin")?;
            let picked = targets[0];
            if !tx
                .state()
                .players
                .get(picked)
                .is_some_and(|player| player.alive)
            {
                return Err(EffectError::Illegal("戮力同心 target must be alive".into()));
            }
            let major = crate::mode::is_major_faction_player(tx.state(), picked);
            let group: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    crate::mode::is_major_faction_player(tx.state(), *candidate) == major
                })
                .collect();
            if group.is_empty() {
                return Err(EffectError::Illegal("戮力同心 has no target".into()));
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &group,
                trick_effect::LULITONGXIN,
                field_source,
            );
        }
        CardKind::TrickKefuzhongyuan => {
            // 克复中原（国战）：对任意名其他角色使用，各目标视为使用杀或摸牌。
            let max_targets = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|player| *player != actor)
                .count() as u32;
            validate_independent_trick_target_count(
                tx,
                actor,
                card,
                targets,
                max_targets,
                "kefuzhongyuan",
            )?;
            for &target in targets {
                validate_alive_other(tx, actor, target)?;
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                targets,
                trick_effect::KEFUZHONGYUAN,
                field_source,
            );
        }
        CardKind::TrickGuguoanbang => {
            // 固国安邦（国战）：对你自己使用，摸八弃≥六；吴势力可将弃牌交给其他吴角色。
            if targets != [actor] {
                return Err(EffectError::Illegal(
                    "固国安邦 targets only its user".into(),
                ));
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &[actor],
                trick_effect::GUGUOANBANG,
                field_source,
            );
        }
        CardKind::TrickHuoshaolianying => {
            // 火烧连营（国战）：对下家及其「队列」（以之为中心的连续同势力座次块）
            // 中所有角色造成 1 点火属性伤害。目标自动计算，无需选择。
            let alive = tx.state().players.alive_by_seat();
            let len = alive.len();
            if len < 2 {
                return Err(EffectError::Illegal(
                    "火烧连营 needs at least two players".into(),
                ));
            }
            let next_index = alive
                .iter()
                .position(|player| *player == actor)
                .map(|index| (index + 1) % len)
                .unwrap_or(0);
            let next = alive[next_index];
            // 队列：从 next 向两翼扩展连续同势力角色。
            let mut queue = vec![next];
            let mut index = (next_index + 1) % len;
            while index != next_index && crate::mode::same_side(tx.state(), next, alive[index]) {
                queue.push(alive[index]);
                index = (index + 1) % len;
            }
            let mut index = (next_index + len - 1) % len;
            while index != next_index && crate::mode::same_side(tx.state(), next, alive[index]) {
                queue.insert(0, alive[index]);
                index = (index + len - 1) % len;
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &queue,
                trick_effect::HUOSHAOLIANYING,
                field_source,
            );
        }
        CardKind::TrickLianjunshengyan => {
            // 联军盛宴（国战）：选一名敌方势力角色为代表，目标 = 使用者自己 + 该
            // 势力全体（同势力/野心联盟）。
            validate_independent_trick_target_count(
                tx,
                actor,
                card,
                targets,
                1,
                "lianjunshengyan",
            )?;
            let picked = targets[0];
            if picked == actor
                || !tx
                    .state()
                    .players
                    .get(picked)
                    .is_some_and(|player| player.alive)
                || crate::mode::same_side(tx.state(), actor, picked)
            {
                return Err(EffectError::Illegal(
                    "联军盛宴 requires a living enemy-faction representative".into(),
                ));
            }
            let mut group = vec![actor, picked];
            for candidate in tx.state().players.alive_by_seat() {
                if candidate != picked
                    && candidate != actor
                    && crate::mode::same_side(tx.state(), picked, candidate)
                {
                    group.push(candidate);
                }
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &group,
                trick_effect::LIANJUNSHENGYAN,
                field_source,
            );
        }
        CardKind::TrickWenheluanwu => {
            // 文和乱武（国战）：对所有其他角色使用，无手牌者自动跳过
            // （上游 ignoreTarget）。目标自动计算，无需选择。
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != actor)
                .filter(|candidate| tx.state().hand_count(*candidate) > 0)
                .collect();
            if targets.is_empty() {
                return Err(EffectError::Illegal("文和乱武 has no target".into()));
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &targets,
                trick_effect::WENHELUANWU,
                field_source,
            );
        }
        CardKind::TrickHaolingtianxia => {
            // 号令天下（国战）：对一名体力值非全场最少（上游 isMinHp）的其他角色使用。
            validate_independent_trick_target_count(tx, actor, card, targets, 1, "haolingtianxia")?;
            let target = targets[0];
            let min_hp = tx
                .state()
                .players
                .alive_by_seat()
                .iter()
                .filter_map(|candidate| tx.state().players.get(*candidate))
                .map(|player| player.hp)
                .min()
                .unwrap_or(i32::MAX);
            if target == actor
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|player| player.alive)
                || tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|player| player.hp <= min_hp)
            {
                return Err(EffectError::Illegal(
                    "号令天下 target must be another living player whose HP exceeds the minimum"
                        .into(),
                ));
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &[target],
                trick_effect::HAOLINGTIANXIA,
                field_source,
            );
        }
        CardKind::TrickChiling => {
            // 敕令（国战）：对所有有暗置武将（unseen 任一槽）的角色使用——含使用者
            // （上游 filterTarget 无自身排除）。目标自动计算，无需选择。
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    tx.state()
                        .players
                        .get(*candidate)
                        .is_some_and(|player| player.unseen[0] || player.unseen[1])
                })
                .collect();
            if targets.is_empty() {
                return Err(EffectError::Illegal("敕令 has no unseen target".into()));
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                &targets,
                trick_effect::CHILING,
                field_source,
            );
        }
        CardKind::TrickTiaojiyanmei => {
            if targets.len() != 2
                || targets[0] == targets[1]
                || targets.contains(&actor)
                || targets.iter().any(|target| {
                    !tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|player| player.alive)
                })
                || tx.state().hand_count(targets[0]) == tx.state().hand_count(targets[1])
            {
                return Err(EffectError::Illegal(
                    "调剂盐梅 requires two living targets with different hand counts".into(),
                ));
            }
            return use_card_then_wuxie(
                tx,
                actor,
                card,
                targets,
                trick_effect::TIAOJIYANMEI,
                &[targets[0].raw(), targets[1].raw()],
            );
        }
        CardKind::TrickMantianguohai => {
            if targets.is_empty()
                || targets.len() > 2
                || targets.iter().any(|target| {
                    *target == actor
                        || !tx
                            .state()
                            .players
                            .get(*target)
                            .is_some_and(|player| player.alive)
                        || crate::query::gainable_target_cards(
                            tx.state(),
                            actor,
                            *target,
                            rule_api::TargetCardScope::HandEquipJudge,
                        )
                        .is_empty()
                })
                || targets.len() == 2 && targets[0] == targets[1]
            {
                return Err(EffectError::Illegal(
                    "瞒天过海 requires one or two distinct living other targets with gainable cards"
                        .into(),
                ));
            }
            return use_independent_trick_targets(
                tx,
                actor,
                card,
                targets,
                trick_effect::MANTIANGUOHAI,
                field_source,
            );
        }
        CardKind::TrickNanman => {
            reject_other_targeting_card_if_self_only(tx, actor)?;
            let others: Vec<PlayerId> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|p| *p != actor)
                .collect();
            let mut args = vec![others.len() as u32];
            args.extend(others.iter().map(|p| p.raw()));
            return use_card_then_wuxie(tx, actor, card, &others, trick_effect::NANMAN, &args);
        }
        CardKind::TrickWanjian => {
            reject_other_targeting_card_if_self_only(tx, actor)?;
            let others: Vec<PlayerId> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|p| *p != actor)
                .collect();
            let mut args = vec![others.len() as u32];
            args.extend(others.iter().map(|p| p.raw()));
            return use_card_then_wuxie(tx, actor, card, &others, trick_effect::WANJIAN, &args);
        }
        CardKind::TrickTaoyuan => {
            let self_only = crate::query::card_targets_self_only_for(tx.state(), actor);
            let alive = if self_only {
                vec![actor]
            } else {
                tx.state().players.alive_by_seat()
            };
            let args = self_only.then(|| vec![1, actor.raw()]).unwrap_or_default();
            return use_card_then_wuxie(tx, actor, card, &alive, trick_effect::TAOYUAN, &args);
        }
        CardKind::TrickWugu => {
            let self_only = crate::query::card_targets_self_only_for(tx.state(), actor);
            let alive = if self_only {
                vec![actor]
            } else {
                tx.state().players.alive_by_seat()
            };
            return run_effects(
                tx,
                vec![
                    EffectRequest::UseCard {
                        player: actor,
                        card,
                        targets: alive.clone(),
                    },
                    EffectRequest::ContinueWugu {
                        action: 0,
                        actor,
                        source_card: card,
                        targets: alive,
                        excluded: Vec::new(),
                        revealed: Vec::new(),
                    },
                ],
            )
            .map(|_| ());
        }
        CardKind::TrickDiaobingqianjiang => {
            let mut alive = tx.state().players.alive_by_seat();
            if let Some(start) = alive.iter().position(|player| *player == actor) {
                alive.rotate_left(start);
            }
            alive.retain(|target| *target == actor || tx.state().hand_count(*target) > 0);
            return run_effects(
                tx,
                vec![
                    EffectRequest::UseCard {
                        player: actor,
                        card,
                        targets: alive.clone(),
                    },
                    EffectRequest::ResolveWdbYangyi {
                        player: actor,
                        target: PlayerId::NONE,
                        card,
                        action: 2,
                        skill: engine_api::SkillId::NONE,
                    },
                    EffectRequest::ContinueDiaobingqianjiang {
                        action: 0,
                        actor,
                        source_card: card,
                        targets: alive,
                        revealed: Vec::new(),
                    },
                ],
            )
            .map(|_| ());
        }
        CardKind::TrickJiedao => {
            // 借刀: targets [weapon_holder, victim]
            if targets.len() != 2 {
                return Err(EffectError::Illegal(
                    "jiedao needs holder and victim".into(),
                ));
            }
            let holder = targets[0];
            let victim = targets[1];
            validate_alive_other(tx, actor, holder)?;
            if victim == holder || victim == actor {
                return Err(EffectError::Illegal("bad jiedao victim".into()));
            }
            let v = tx
                .state()
                .players
                .get(victim)
                .ok_or_else(|| EffectError::Illegal("bad victim".into()))?;
            if !v.alive {
                return Err(EffectError::Illegal("victim dead".into()));
            }
            let equip = tx
                .state()
                .zones
                .find(ZoneKind::Equip, holder)
                .ok_or_else(|| EffectError::Illegal("no equip".into()))?;
            let has_weapon = tx
                .state()
                .zones
                .get(equip)
                .map(|z| {
                    z.cards.iter().any(|cid| {
                        tx.state()
                            .cards
                            .get(*cid)
                            .map(|c| c.kind == CardKind::Weapon)
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);
            if !has_weapon {
                return Err(EffectError::Illegal("holder has no weapon".into()));
            }
            return use_card_then_wuxie(
                tx,
                actor,
                card,
                &[holder, victim],
                trick_effect::JIEDAO,
                &[holder.raw(), victim.raw()],
            );
        }
        CardKind::TrickTiesuo => {
            // 0 targets: 重铸 — discard and draw 1 (not nullifiable as "use effect")
            // 1–2 targets: toggle chain — goes through 无懈
            let limit = crate::query::card_target_limit_for_card(tx.state(), actor, card, 2);
            if targets.len() > limit as usize {
                return Err(EffectError::Illegal(format!(
                    "tiesuo at most {limit} targets"
                )));
            }
            ensure_distinct_targets(targets)?;
            if targets.is_empty() {
                queue.push(EffectRequest::UseCard {
                    player: actor,
                    card,
                    targets: vec![],
                });
                queue.push(EffectRequest::DrawCards {
                    player: actor,
                    count: 1,
                });
            } else {
                for &t in targets {
                    if tx.state().players.get(t).map(|p| !p.alive).unwrap_or(true) {
                        return Err(EffectError::Illegal("tiesuo target dead".into()));
                    }
                }
                let mut args = vec![TIESUO_SAVED_TARGETS_MAGIC, card.raw(), targets.len() as u32];
                args.extend(targets.iter().map(|t| t.raw()));
                return run_effects_then_wuxie(
                    tx,
                    vec![
                        EffectRequest::UseCard {
                            player: actor,
                            card,
                            targets: targets.to_vec(),
                        },
                        EffectRequest::EmitRuleEvent {
                            code: 52,
                            payload: {
                                let mut v = vec![actor.raw(), card.raw()];
                                v.extend(targets.iter().map(|t| t.raw()));
                                v
                            },
                        },
                    ],
                    actor,
                    trick_effect::TIESUO,
                    actor,
                    &args,
                );
            }
        }
        CardKind::TrickLe | CardKind::TrickBingliang => {
            if targets.len() != 1 {
                return Err(EffectError::Illegal(
                    "delayed trick needs one target".into(),
                ));
            }
            let target = targets[0];
            validate_alive_other(tx, actor, target)?;
            if kind == CardKind::TrickBingliang
                && !ignore_distance
                && !crate::query::card_target_within_distance(tx.state(), actor, card, target, 1)
            {
                return Err(EffectError::Illegal("target out of 兵粮寸断 range".into()));
            }
            if kind == CardKind::TrickLe
                && crate::query::target_immune(
                    tx.state(),
                    target,
                    crate::query::ModifierTarget::ImmuneLe,
                )
            {
                return Err(EffectError::Illegal(
                    "target immune to 乐不思蜀 (谦逊)".into(),
                ));
            }
            if has_delayed_of_kind(tx, target, kind) {
                return Err(EffectError::Illegal(
                    "target already has this delayed".into(),
                ));
            }
            // Leave hand into discard first; after 无懈, move into judge (or stay discarded if nullified).
            return run_effects_then_wuxie(
                tx,
                vec![
                    EffectRequest::UseCard {
                        player: actor,
                        card,
                        targets: vec![target],
                    },
                    EffectRequest::EmitRuleEvent {
                        code: 51, // delayed used
                        payload: vec![actor.raw(), card.raw(), target.raw(), kind.tag() as u32],
                    },
                ],
                actor,
                trick_effect::DELAYED_PLACE,
                actor,
                &[card.raw(), target.raw(), kind.tag() as u32],
            );
        }
        CardKind::TrickShandian => {
            if tx.state().disabled_judge_zones.contains(&actor) {
                return Err(EffectError::Illegal("judgment zone is disabled".into()));
            }
            if has_delayed_of_kind(tx, actor, CardKind::TrickShandian) {
                return Err(EffectError::Illegal("already has lightning".into()));
            }
            return run_effects_then_wuxie(
                tx,
                vec![
                    EffectRequest::UseCard {
                        player: actor,
                        card,
                        targets: vec![actor],
                    },
                    EffectRequest::EmitRuleEvent {
                        code: 51,
                        payload: vec![actor.raw(), card.raw(), actor.raw(), kind.tag() as u32],
                    },
                ],
                actor,
                trick_effect::DELAYED_PLACE,
                actor,
                &[card.raw(), actor.raw(), kind.tag() as u32],
            );
        }
        CardKind::Other => {
            queue.push(EffectRequest::UseCard {
                player: actor,
                card,
                targets: targets.to_vec(),
            });
        }
    }

    run_effects(tx, queue).map(|_| ())
}

/// Proactive ordinary tricks that can currently be used source-free by `actor`.
/// `used_mask` stores permanent CardKind-tag bits for skills such as 3D精选【奋计】.
pub(crate) fn source_free_ordinary_trick_kinds(
    state: &crate::state::GameState,
    actor: PlayerId,
    used_mask: u64,
) -> Vec<CardKind> {
    let mut probe = Transaction::begin(state.clone());
    let Some(processing) = probe
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
    else {
        return Vec::new();
    };
    let mut candidates: Vec<_> = state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| card.def.is_some())
        .map(|card| card.kind)
        .filter(|kind| {
            kind.is_trick()
                && !kind.is_delayed_trick()
                && !matches!(kind, CardKind::TrickWuxie | CardKind::TrickYoudishenru)
                && used_mask & (1u64 << kind.tag()) == 0
        })
        .collect();
    candidates.sort_by_key(|kind| kind.tag());
    candidates.dedup();
    candidates
        .into_iter()
        .filter(|kind| {
            let card = probe.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::NONE,
                kind: *kind,
                suit: engine_api::Suit::None,
                rank: 0,
                weapon_range: 0,
                zone: processing,
                owner: PlayerId::NONE,
                face_up: true,
            });
            probe
                .state_mut()
                .zones
                .get_mut(processing)
                .expect("validated processing zone")
                .cards
                .push(card);
            skill_card_target_spec(&probe, actor, card).is_ok()
        })
        .collect()
}

/// Determine whether a revealed card has a legal use and which target choice it needs.
pub(crate) fn skill_card_target_spec(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
) -> Result<SkillCardTargetSpec, EffectError> {
    skill_card_target_spec_with_options(tx, actor, card, false, false)
}
