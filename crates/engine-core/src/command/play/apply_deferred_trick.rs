//! Command operations for `apply_deferred_trick` and related helpers.

use super::*;

/// Apply trick effect after 无懈 window clears without odd nullify.
pub(crate) fn apply_deferred_trick(
    tx: &mut Transaction,
    effect_tag: u32,
    actor: PlayerId,
    args: &[u32],
    source_card: CardId,
) -> Result<(), EffectError> {
    let source_card = if source_card.is_none() {
        effect_args_source_card(args)
    } else {
        source_card
    };
    match effect_tag {
        trick_effect::WUZHONG => run_effects(
            tx,
            vec![EffectRequest::DrawCards {
                player: actor,
                count: 2,
            }],
        )
        .map(|_| ()),
        trick_effect::YUANJIAO => {
            let target = PlayerId::new(*args.first().unwrap_or(&0));
            run_effects(
                tx,
                vec![
                    EffectRequest::DrawCards {
                        player: target,
                        count: 1,
                    },
                    EffectRequest::DrawCards {
                        player: actor,
                        count: 3,
                    },
                ],
            )
            .map(|_| ())
        }
        trick_effect::TIAOJIYANMEI => {
            let [left_raw, right_raw] = args else {
                return Err(EffectError::ContentFault(
                    "调剂盐梅 target payload is invalid".into(),
                ));
            };
            let left_excluded = left_raw & SAVED_TARGET_EXCLUDED_MASK != 0;
            let right_excluded = right_raw & SAVED_TARGET_EXCLUDED_MASK != 0;
            let left = PlayerId::new(left_raw & !SAVED_TARGET_EXCLUDED_MASK);
            let right = PlayerId::new(right_raw & !SAVED_TARGET_EXCLUDED_MASK);
            let left_hand = tx.state().hand_count(left);
            let right_hand = tx.state().hand_count(right);
            let (lower, higher) = if left_hand < right_hand {
                (left, right)
            } else if right_hand < left_hand {
                (right, left)
            } else {
                return Ok(());
            };
            let mut effects = Vec::new();
            let lower_excluded = if lower == left {
                left_excluded
            } else {
                right_excluded
            };
            let higher_excluded = if higher == left {
                left_excluded
            } else {
                right_excluded
            };
            if !lower_excluded {
                effects.push(EffectRequest::DrawCards {
                    player: lower,
                    count: 1,
                });
            }
            if !higher_excluded {
                effects.push(EffectRequest::BeginTiaojiyanmeiDiscard {
                    actor,
                    left,
                    right,
                    higher,
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        trick_effect::MANTIANGUOHAI => {
            let Some((queue, target)) = queued_independent_trick(args) else {
                return Err(EffectError::EngineFault(
                    "瞒天过海 missing independent target queue".into(),
                ));
            };
            let options: Vec<_> = crate::query::gainable_target_cards(
                tx.state(),
                actor,
                target,
                rule_api::TargetCardScope::HandEquipJudge,
            )
            .into_iter()
            .map(CardId::raw)
            .collect();
            if options.is_empty() {
                return finish_independent_trick_target(tx, args);
            }
            let mut payload = vec![
                MANTIANGUOHAI_CUSTOM_CHOICE,
                0,
                actor.raw(),
                target.raw(),
                queue.len() as u32,
            ];
            payload.extend_from_slice(queue);
            crate::effect::open_choice(
                tx,
                actor,
                1,
                1,
                false,
                options,
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::YOUDISHENRU => {
            let (protected, source, _, _, _) = parse_youdishenru_saved(args)?;
            begin_youdishenru_force_loop(tx, protected, source, args)
        }
        trick_effect::DIAOBINGQIANJIANG => {
            let Some((queue, target)) = queued_independent_trick(args) else {
                return Err(EffectError::EngineFault(
                    "调兵遣将 missing independent target queue".into(),
                ));
            };
            let len = *queue.get(6).unwrap_or(&0) as usize;
            let extension = 7usize.saturating_add(len);
            if queue.get(extension) != Some(&DIAOBING_POOL_MAGIC) {
                return Err(EffectError::EngineFault(
                    "调兵遣将 pool marker is missing".into(),
                ));
            }
            let count = *queue.get(extension + 1).unwrap_or(&u32::MAX) as usize;
            let revealed: Vec<_> = queue
                .get(extension + 2..extension + 2 + count)
                .ok_or_else(|| EffectError::EngineFault("调兵遣将 pool is truncated".into()))?
                .iter()
                .copied()
                .map(CardId::new)
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|state| {
                        tx.state()
                            .zones
                            .get(state.zone)
                            .is_some_and(|zone| zone.kind == ZoneKind::Processing)
                    })
                })
                .collect();
            if revealed.is_empty()
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || tx.state().hand_cards(target).is_empty()
            {
                return finish_independent_trick_target(tx, args);
            }
            let mut payload = vec![
                crate::effect::RESOLVE_WDB_YANGYI_CHOICE,
                2,
                queue.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend_from_slice(queue);
            crate::effect::open_choice(
                tx,
                target,
                0,
                1,
                true,
                revealed.iter().map(|card| card.raw()).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::WUGU => {
            let Some((queue, target)) = queued_independent_trick(args) else {
                return Err(EffectError::EngineFault(
                    "五谷丰登 missing independent target queue".into(),
                ));
            };
            let len = *queue.get(6).unwrap_or(&0) as usize;
            let extension = 7usize.saturating_add(len);
            if queue.get(extension) != Some(&WUGU_POOL_MAGIC) {
                return Err(EffectError::EngineFault(
                    "五谷丰登 pool marker is missing".into(),
                ));
            }
            let count = *queue.get(extension + 1).unwrap_or(&u32::MAX) as usize;
            let revealed: Vec<_> = queue
                .get(extension + 2..extension + 2 + count)
                .ok_or_else(|| EffectError::EngineFault("五谷丰登 pool is truncated".into()))?
                .iter()
                .copied()
                .map(CardId::new)
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|state| {
                        tx.state()
                            .zones
                            .get(state.zone)
                            .is_some_and(|zone| zone.kind == ZoneKind::Processing)
                    })
                })
                .collect();
            if revealed.is_empty()
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            let mut payload = vec![
                crate::effect::RESOLVE_WDB_ZAOZHIRENJUN_CHOICE,
                2,
                queue.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend_from_slice(queue);
            crate::effect::open_choice(
                tx,
                target,
                1,
                1,
                false,
                revealed.iter().map(|card| card.raw()).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::WANGMEIZHIKE => {
            let target = PlayerId::new(*args.first().unwrap_or(&0));
            run_effects(
                tx,
                vec![EffectRequest::ResolveWdbZaozhiRenjun {
                    player: actor,
                    target,
                    card: source_card,
                    cards: Vec::new(),
                    action: 2,
                    skill: engine_api::SkillId::NONE,
                }],
            )
            .map(|_| ())
        }
        trick_effect::CHUQIBUYI => {
            // 出其不意：展示目标一张手牌（确定性首张近似），与源牌花色
            // 不同则对其造成 1 点伤害。
            let target = PlayerId::new(*args.first().unwrap_or(&0));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            let shown = tx.state().hand_cards(target).first().copied();
            let source_suit = tx.state().cards.get(source_card).map(|card| card.suit);
            if let (Some(shown), Some(source_suit)) = (shown, source_suit) {
                let shown_suit = tx.state().cards.get(shown).map(|card| card.suit);
                if shown_suit.is_some_and(|suit| suit != source_suit) {
                    return run_effects_then_finish_independent_trick_target(
                        tx,
                        vec![EffectRequest::DealDamage {
                            source: actor,
                            target,
                            amount: 1,
                            is_lose_hp: false,
                            nature: rule_api::damage_nature::NORMAL,
                        }],
                        args,
                    );
                }
            }
            finish_independent_trick_target(tx, args)
        }
        trick_effect::YIYI => {
            // 以逸待劳（国战）：目标摸两张牌，然后弃置两张牌。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            // 弃牌 choice 挂起时，已保存的 IndependentTrickQueue resume 会在 choice
            // 完成后经 resume_interrupted_continuation 推进到下一目标。
            run_effects_then_finish_independent_trick_target(
                tx,
                vec![
                    EffectRequest::DrawCards {
                        player: target,
                        count: 2,
                    },
                    EffectRequest::DiscardHandEquipCount {
                        player: target,
                        count: 2,
                        skill: engine_api::SkillId::NONE,
                    },
                ],
                args,
            )
        }
        trick_effect::SHUIYANQIJUNX => {
            // 水淹七军（国战）：目标选择弃置全部装备牌或受 1 点雷电伤害。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            if crate::query::discardable_equipment_cards(tx.state(), target, target).is_empty() {
                // 无装备可弃：直接受 1 点雷电伤害。
                return run_effects_then_finish_independent_trick_target(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: actor,
                        target,
                        amount: 1,
                        is_lose_hp: false,
                        nature: rule_api::damage_nature::THUNDER,
                    }],
                    args,
                );
            }
            let mut payload = vec![
                crate::effect::SHUIYANQIJUNX_CUSTOM_CHOICE,
                actor.raw(),
                target.raw(),
            ];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                target,
                1,
                1,
                false,
                vec![1, 2],
                engine_api::ChoiceKind::Custom,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::LULITONGXIN => {
            // 戮力同心（国战）：目标未横置 → 横置；已横置 → 摸一张牌。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            let linked = player_has_mark(tx, target, crate::content::MARK_CHAIN);
            let effects = if linked {
                vec![EffectRequest::DrawCards {
                    player: target,
                    count: 1,
                }]
            } else {
                vec![EffectRequest::AddMark {
                    player: target,
                    mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
                    delta: 1,
                }]
            };
            run_effects_then_finish_independent_trick_target(tx, effects, args)
        }
        trick_effect::KEFUZHONGYUAN => {
            // 克复中原（国战）：目标选择 1=视为使用杀（蜀势力伤害基数+1）；2=摸牌
            // （蜀势力摸两张，否则一张）。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            let mut payload = vec![
                crate::effect::KEFUZHONGYUAN_CUSTOM_CHOICE,
                actor.raw(),
                target.raw(),
            ];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                target,
                1,
                1,
                false,
                vec![1, 2],
                engine_api::ChoiceKind::Custom,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::GUGUOANBANG => {
            // 固国安邦（国战）：摸八张牌，然后弃置至少六张手牌；吴势力可选将弃牌
            // 交给其他吴角色（每名至多两张，单轮近似）。三阶段 Custom 续体。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            run_effects(
                tx,
                vec![EffectRequest::DrawCards {
                    player: target,
                    count: 8,
                }],
            )?;
            let hand = tx.state().hand_cards(target);
            if hand.len() < 6 {
                return finish_independent_trick_target(tx, args);
            }
            let mut payload = vec![
                crate::effect::GUGUOANBANG_CUSTOM_CHOICE,
                1,
                actor.raw(),
                target.raw(),
            ];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                target,
                6,
                hand.len().min(u8::MAX as usize) as u8,
                false,
                hand.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::HUOSHAOLIANYING => {
            // 火烧连营（国战）：对目标造成 1 点火属性伤害。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            run_effects_then_finish_independent_trick_target(
                tx,
                vec![EffectRequest::DealDamage {
                    source: actor,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::FIRE,
                }],
                args,
            )
        }
        trick_effect::LIANJUNSHENGYAN => {
            // 联军盛宴（国战）：非己目标摸 1 并重置（解除横置）；自己按敌方势力人数
            // 分配「摸 Y 张 + 回 X−Y 点」（Y∈[0,X]，回复量以缺体为上限）。
            let (queue_payload, target) = queued_independent_trick(args)
                .map(|(payload, target)| (Some(payload), target))
                .unwrap_or((None, PlayerId::new(*args.first().unwrap_or(&0))));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            if target != actor {
                let mut effects = vec![EffectRequest::DrawCards {
                    player: target,
                    count: 1,
                }];
                if player_has_mark(tx, target, crate::content::MARK_CHAIN) {
                    effects.push(EffectRequest::RemoveMark {
                        player: target,
                        mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
                        amount: 0,
                    });
                }
                return run_effects_then_finish_independent_trick_target(tx, effects, args);
            }
            // 自己：X = 敌方势力人数 = 队列目标数 − 1（含自己）。
            let x = queue_payload
                .and_then(|payload| payload.get(6).copied())
                .map(|len| len.saturating_sub(1))
                .unwrap_or(0);
            if x == 0 {
                return finish_independent_trick_target(tx, args);
            }
            let participant = tx
                .state()
                .players
                .get(actor)
                .ok_or_else(|| EffectError::Illegal("user missing".into()))?;
            let damaged = participant
                .max_hp
                .saturating_sub(participant.hp.max(0) as u32);
            if damaged == 0 {
                return run_effects_then_finish_independent_trick_target(
                    tx,
                    vec![EffectRequest::DrawCards {
                        player: actor,
                        count: x,
                    }],
                    args,
                );
            }
            let cap = x.min(damaged);
            let mut payload = vec![crate::effect::LIANJUNSHENGYAN_CUSTOM_CHOICE, actor.raw(), x];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                actor,
                1,
                1,
                false,
                (0..=cap).collect(),
                engine_api::ChoiceKind::Custom,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::WENHELUANWU => {
            // 文和乱武（国战）：目标展示手牌；使用者二选一：①令其弃置两张类型不同的
            // 手牌（仅一种类型时退化为弃一张）②弃置其一张手牌；然后若目标为群势力
            // 且空手，则摸至当前体力（至多5）。选择权在使用者，故 choice 挂到 actor。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
                || tx.state().hand_cards(target).is_empty()
            {
                return finish_independent_trick_target(tx, args);
            }
            let mut payload = vec![
                crate::effect::WENHELUANWU_CUSTOM_CHOICE,
                1, // stage 1 = 使用者二选一
                actor.raw(),
                target.raw(),
            ];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                actor,
                1,
                1,
                false,
                vec![1, 2],
                engine_api::ChoiceKind::Custom,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::HAOLINGTIANXIA => {
            // 号令天下（国战）：对目标外所有角色（按座次）依次二选一。链状态以
            // 独立锦囊队列格式持久化（effect_tag = HAOLINGTIANXIA），每个步骤经
            // IndependentTrickQueue resume 续链（杀以直接伤害近似，ADR 0623）。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
            {
                return finish_independent_trick_target(tx, args);
            }
            let chain: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != target)
                .collect();
            if chain.is_empty() {
                return finish_independent_trick_target(tx, args);
            }
            let source_card_state = tx
                .state()
                .cards
                .get(source_card)
                .ok_or_else(|| EffectError::Illegal("bad trick card".into()))?;
            let kind = crate::query::effective_card_kind(tx.state(), actor, source_card_state);
            let suit = crate::query::effective_card_suit(tx.state(), actor, source_card_state);
            let mut queue = vec![
                source_card.raw(),
                actor.raw(),
                kind.tag() as u32,
                super::super::judge::suit_tag(suit),
                trick_effect::HAOLINGTIANXIA,
                0,
                chain.len() as u32,
            ];
            queue.extend(chain.iter().map(|candidate| candidate.raw()));
            queue.push(target.raw());
            queue.extend_from_slice(args);
            advance_haolingtianxia_player(tx, &queue)
        }
        trick_effect::CHILING => {
            // 敕令（国战）：目标选 ①明置一张武将牌并摸一张 ②失去1点体力 ③弃置一张
            // 装备牌（有装备牌时可选）。选择权在目标自身，choice 挂到 target。
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|player| player.alive)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|player| player.unseen[0] || player.unseen[1])
            {
                return finish_independent_trick_target(tx, args);
            }
            let mut choices = vec![1, 2];
            if !equip_typed_zone_cards(tx, target).is_empty() {
                choices.push(3);
            }
            let mut payload = vec![
                crate::effect::CHILING_CUSTOM_CHOICE,
                1, // stage 1 = 目标三选一
                target.raw(),
            ];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                target,
                1,
                1,
                false,
                choices,
                engine_api::ChoiceKind::Custom,
                crate::state::ContinuationKind::Custom,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::GUOHE => {
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            let tokens = discardable_target_zone_card_tokens(tx, actor, target);
            if tokens.is_empty() {
                return finish_independent_trick_target(tx, args);
            }
            let mut payload = vec![target.raw(), actor.raw(), 0];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                actor,
                1,
                1,
                false,
                tokens,
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::PickTargetCard,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::SHUNSHOU => {
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            let tokens = target_zone_card_tokens(tx, target, true, true, false);
            if tokens.is_empty() {
                return finish_independent_trick_target(tx, args);
            }
            let mut payload = vec![target.raw(), actor.raw(), 1];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                actor,
                1,
                1,
                false,
                tokens,
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::PickTargetCard,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::HUOGONG => {
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            let thand = tx.state().hand_cards(target);
            if thand.is_empty() {
                return finish_independent_trick_target(tx, args);
            }
            let tokens: Vec<u32> = thand.iter().map(|c| c.raw()).collect();
            let mut payload = vec![actor.raw(), target.raw()];
            if queued_independent_trick(args).is_some() {
                payload.extend_from_slice(args);
            }
            crate::effect::open_choice(
                tx,
                target,
                1,
                1,
                false,
                tokens,
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::HuogongShow,
                payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::JUEDOU => {
            let target = queued_independent_trick(args)
                .map(|(_, target)| target)
                .unwrap_or_else(|| PlayerId::new(*args.first().unwrap_or(&0)));
            if tx
                .state()
                .players
                .get(target)
                .map(|p| !p.alive)
                .unwrap_or(true)
            {
                return finish_independent_trick_target(tx, args);
            }
            let saved_tail;
            let tail: &[u32] = if queued_independent_trick(args).is_some() {
                args
            } else if source_card != CardId::NONE {
                saved_tail = vec![SAVED_WUXIE_SOURCE_CARD_MAGIC, source_card.raw()];
                &saved_tail
            } else {
                &[]
            };
            begin_juedou_with_tail(tx, actor, target, tail)
        }
        trick_effect::NANMAN => {
            let len = *args.first().unwrap_or(&0) as usize;
            let targets: Vec<PlayerId> = args
                .get(1..1 + len)
                .unwrap_or(&[])
                .iter()
                .map(|r| PlayerId::new(*r))
                .filter(|p| tx.state().players.get(*p).map(|x| x.alive).unwrap_or(false))
                .collect();
            if targets.is_empty() {
                return Ok(());
            }
            begin_aoe_queue(
                tx,
                1,
                actor,
                saved_aoe_base_damage(args, len),
                &targets,
                source_card,
            )
        }
        trick_effect::WANJIAN => {
            let len = *args.first().unwrap_or(&0) as usize;
            let targets: Vec<PlayerId> = args
                .get(1..1 + len)
                .unwrap_or(&[])
                .iter()
                .map(|r| PlayerId::new(*r))
                .filter(|p| tx.state().players.get(*p).map(|x| x.alive).unwrap_or(false))
                .collect();
            if targets.is_empty() {
                return Ok(());
            }
            begin_aoe_queue(
                tx,
                2,
                actor,
                saved_aoe_base_damage(args, len),
                &targets,
                source_card,
            )
        }
        trick_effect::TAOYUAN => {
            let alive = optional_trick_targets(tx, args)
                .unwrap_or_else(|| tx.state().players.alive_by_seat());
            let mut q = Vec::new();
            for p in alive {
                let pl = tx.state().players.get(p).unwrap();
                if pl.hp < pl.max_hp as i32 {
                    q.push(EffectRequest::RecoverHp {
                        source: actor,
                        player: p,
                        amount: 1,
                    });
                }
            }
            if q.is_empty() {
                return Ok(());
            }
            run_effects(tx, q).map(|_| ())
        }
        trick_effect::TIESUO => {
            let (len_index, targets_start) = if args.first() == Some(&TIESUO_SAVED_TARGETS_MAGIC) {
                (2, 3)
            } else {
                (0, 1)
            };
            let len = *args.get(len_index).unwrap_or(&0) as usize;
            let targets: Vec<PlayerId> = args
                .get(targets_start..targets_start + len)
                .unwrap_or(&[])
                .iter()
                .map(|r| PlayerId::new(*r))
                .collect();
            let mut q = Vec::new();
            for t in targets {
                if tx.state().players.get(t).map(|p| !p.alive).unwrap_or(true) {
                    continue;
                }
                if player_has_mark(tx, t, crate::content::MARK_CHAIN) {
                    q.push(EffectRequest::RemoveMark {
                        player: t,
                        mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
                        amount: 0,
                    });
                } else {
                    q.push(EffectRequest::AddMark {
                        player: t,
                        mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
                        delta: 1,
                    });
                }
            }
            if q.is_empty() {
                return Ok(());
            }
            run_effects(tx, q).map(|_| ())
        }
        trick_effect::JIEDAO => {
            let holder = PlayerId::new(*args.first().unwrap_or(&0));
            let victim = PlayerId::new(*args.get(1).unwrap_or(&0));
            if tx
                .state()
                .players
                .get(holder)
                .map(|p| !p.alive)
                .unwrap_or(true)
            {
                return Ok(());
            }
            let mut tokens = vec![0u32];
            if sha_in_range(tx, holder, victim) {
                tokens.push(1);
            }
            crate::effect::open_choice(
                tx,
                holder,
                1,
                1,
                false,
                tokens,
                engine_api::ChoiceKind::Custom,
                crate::state::ContinuationKind::JiedaoChoice,
                vec![actor.raw(), holder.raw(), victim.raw()],
            )?;
            Err(EffectError::ChoiceRequested)
        }
        trick_effect::DELAYED_PLACE => {
            // args: [card, target, kind_tag] — card currently in discard after use
            let cid = CardId::new(*args.first().unwrap_or(&0));
            let target = PlayerId::new(*args.get(1).unwrap_or(&0));
            let kind_tag = *args.get(2).unwrap_or(&0) as u8;
            let kind = match kind_tag {
                30 => CardKind::TrickLe,
                31 => CardKind::TrickBingliang,
                32 => CardKind::TrickShandian,
                _ => CardKind::Other,
            };
            if tx
                .state()
                .players
                .get(target)
                .map(|p| !p.alive)
                .unwrap_or(true)
            {
                return Ok(()); // target dead → stay discarded
            }
            if tx.state().disabled_judge_zones.contains(&target) {
                return Ok(());
            }
            // Re-check duplicate delayed (race with other effects)
            if kind != CardKind::Other && has_delayed_of_kind(tx, target, kind) {
                return Ok(());
            }
            let Some(card) = tx.state().cards.get(cid).cloned() else {
                return Ok(());
            };
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
            if card.zone != discard {
                return Ok(()); // already moved elsewhere
            }
            let judge = tx
                .state()
                .zones
                .find(ZoneKind::Judge, target)
                .ok_or_else(|| EffectError::Illegal("no judge".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![cid],
                    to: judge,
                    from: Some(discard),
                }],
            )?;
            // `UseCard` temporarily stages delayed tricks in the discard pile. The generic
            // use/response-discard counter observes that staging move; a successfully placed
            // delayed trick did not actually finish in discard and must therefore be removed
            // from the turn total. A cancelled/invalid delayed trick stays discarded and keeps
            // the count.
            if tx.state().content.tracks_turn_use_response_discard_count() {
                let count = tx
                    .state()
                    .turn
                    .flag(crate::state::turn_flag::USE_RESPONSE_DISCARDED);
                tx.state_mut().turn.set_flag(
                    crate::state::turn_flag::USE_RESPONSE_DISCARDED,
                    count.saturating_sub(1),
                );
            }
            if let Some(c) = tx.state_mut().cards.get_mut(cid) {
                c.face_up = true;
                c.owner = target;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
