//! Effect primitives for `resolve_ddd_liuba` and related helpers.

use super::*;

pub(crate) fn resolve_ddd_liuba(
    tx: &mut Transaction,
    player: PlayerId,
    _actor: PlayerId,
    action: u8,
    mut cards: Vec<CardId>,
    _targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if action <= 4 && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        // 丰政：点数和为 13 的任意张手牌作为完整【无中生有】使用。
        0 => {
            if !crate::command::skill::ddd_fengzheng_provider_exists(tx.state())
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || cards.is_empty()
            {
                return Err(EffectError::Illegal("丰政转化时机无效".into()));
            }
            cards.sort_by_key(|card| card.raw());
            cards.dedup();
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let total = cards.iter().try_fold(0u16, |sum, card| {
                tx.state().cards.get(*card).and_then(|card| {
                    (Some(card.zone) == hand).then(|| {
                        sum.saturating_add(u16::from(crate::query::effective_card_number(
                            tx.state(),
                            player,
                            card,
                        )))
                    })
                })
            });
            if total != Some(13) {
                return Err(EffectError::Illegal("丰政手牌点数和必须为 13".into()));
            }
            let round = tx.state().turn.round;
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.round_fire_round = round;
            }
            let carrier = cards[0];
            let extras = &cards[1..];
            tx.state_mut()
                .turn
                .set_flag(turn_flag::VIEW_AS_CARD, carrier.raw() as i32);
            tx.state_mut()
                .turn
                .set_flag(turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
            tx.state_mut().turn.set_flag(
                turn_flag::VIEW_AS_LOGICAL_KIND,
                i32::from(CardKind::TrickWuzhong.tag()),
            );
            let result = crate::command::play::play_as_kind_with_extra_damage_flags(
                tx,
                player,
                carrier,
                CardKind::TrickWuzhong,
                &[player],
                false,
                false,
                &[],
                false,
                true,
                0,
                extras,
                0,
            );
            tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_CARD, 0);
            tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_SKILL, 0);
            tx.state_mut()
                .turn
                .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, 0);
            result.map(|_| Vec::new())
        }
        // 丰政轮首：本轮只提示一次，选择是否开始分配。
        1 => {
            let round = tx.state().turn.round;
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.round_fire_round = round;
            }
            let count = tx
                .state()
                .skills
                .sorted_live()
                .iter()
                .filter(|instance| {
                    instance.def.raw() == crate::command::skill::DDD_FENGZHENG_GLOBAL_DEF
                        && instance.round_fire_round != 0
                        && instance.round_fire_round.saturating_add(1) == round
                })
                .count();
            if count == 0 {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                vec![1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_LIUBA_CHOICE,
                    0,
                    player.raw(),
                    skill.raw(),
                    count as u32,
                ],
            )
        }
        // 安装每名角色自己的 non-gainable 全局技实例。
        2 => {
            let mut events = Vec::new();
            for target in tx.state().players.alive_by_seat() {
                let exists = tx.state().players.get(target).is_some_and(|participant| {
                    participant.skills.iter().any(|owned| {
                        tx.state().skills.get(*owned).is_some_and(|instance| {
                            instance.def.raw() == crate::command::skill::DDD_FENGZHENG_GLOBAL_DEF
                        })
                    })
                });
                if !exists {
                    events.extend(
                        grant_skill_instance(
                            tx,
                            target,
                            engine_api::SkillDefId::new(
                                crate::command::skill::DDD_FENGZHENG_GLOBAL_DEF,
                            ),
                            Vec::new(),
                        )?
                        .1,
                    );
                }
            }
            Ok(events)
        }
        // 玉律起始：先摸一张，嵌套事件结束后再置牌。
        3 => {
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveDddLiuba {
                    player,
                    actor: player,
                    action: 11,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    skill,
                });
            run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }])
        }
        // 玉律②：记录本回合序号后由拥有者选择摸牌或弃牌。
        4 => {
            let actor = tx.state().turn.current;
            let ordinal = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .map(|instance| {
                    instance.turn_fire_count = instance.turn_fire_count.saturating_add(1);
                    instance.turn_fire_count
                })
                .unwrap_or(0);
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_LIUBA_CHOICE,
                    3,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                    ordinal,
                ],
            )
        }
        // 丰政顶牌进入处理区后的第一次分配。
        10 => ddd_liuba_open_distribution_subset(tx, player, skill, cards, Vec::new()),
        // 玉律摸牌后选择一张手牌置于公开技能牌堆。
        11 => {
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![RESOLVE_DDD_LIUBA_CHOICE, 4, player.raw(), skill.raw()],
            )
        }
        // 第二次玉律后的交换入口。
        12 => ddd_liuba_open_yulv_exchange(tx, player, skill),
        // 无手牌先摸一张后的交换入口。
        13 => {
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() {
                Ok(Vec::new())
            } else {
                open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    hand.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    vec![RESOLVE_DDD_LIUBA_CHOICE, 6, player.raw(), skill.raw()],
                )
            }
        }
        // 分配选择全部完成后，在一个效果段内统一移动全部牌。
        14 => {
            if cards.len() != _targets.len() {
                return Err(EffectError::EngineFault("丰政最终分配长度不一致".into()));
            }
            ddd_liuba_finish_distribution(
                tx,
                player,
                skill,
                cards.into_iter().zip(_targets).collect(),
            )
        }
        // 交换“玉律”，两次移动属于同一效果段。
        15 => {
            let [new_card] = cards.as_slice() else {
                return Err(EffectError::EngineFault("玉律交换牌数量无效".into()));
            };
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("玉律拥有者手牌区不存在".into()))?;
            if !tx
                .state()
                .cards
                .get(*new_card)
                .is_some_and(|card| card.zone == hand)
            {
                return Err(EffectError::Illegal("玉律交换手牌已失效".into()));
            }
            let old = tx
                .state()
                .skill_pile_cards(player, DDD_YULV_DEF)
                .first()
                .copied()
                .ok_or_else(|| EffectError::Illegal("玉律牌不存在".into()))?;
            let pile = ensure_skill_pile_zone(tx, player, DDD_YULV_DEF)?;
            let mut events = move_cards(tx, vec![old], hand, Some(pile))?;
            events.extend(move_cards(tx, vec![*new_card], pile, Some(hand))?);
            Ok(events)
        }
        _ => Err(EffectError::EngineFault("unknown 3D精选刘巴 action".into())),
    }
}
