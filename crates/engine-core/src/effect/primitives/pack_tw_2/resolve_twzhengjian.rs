//! Effect primitives for `resolve_twzhengjian` and related helpers.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twzhengjian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    helper_skill_def: u32,
    action: u8,
    stage: u8,
    card: CardId,
    branch: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::ResolveTwzhengjian,
            vec![0, player.raw(), helper_skill_def, skill.raw()],
        ),
        (1, 0) => {
            let (_, awakened, _) = tx
                .state()
                .skills
                .get(skill)
                .map_or((None, false, Vec::new()), |skill| skill.twzhengjian_state());
            if awakened {
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    vec![0, 1, 2, 3],
                    ChoiceKind::Custom,
                    ContinuationKind::ResolveTwzhengjian,
                    vec![2, player.raw(), target.raw(), helper_skill_def, skill.raw()],
                );
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![1, player.raw(), target.raw(), helper_skill_def, skill.raw()];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwzhengjian,
                payload,
            )
        }
        (1, 1) => {
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("TW Zhengjian card missing".into()))?;
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("TW Zhengjian hand missing".into()))?;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveTwzhengjian {
                    player,
                    target,
                    helper_skill_def,
                    action: 1,
                    stage: 2,
                    card: CardId::NONE,
                    branch: 0,
                    skill,
                },
            );
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: vec![card],
                    to: hand,
                    from,
                }],
            )
        }
        (1, 2) => {
            let player_count = tx.state().players.sorted_live().len();
            let threshold = player_count.div_ceil(2);
            let (selected, awakened, mut contributors) = tx
                .state()
                .skills
                .get(skill)
                .map_or((None, false, Vec::new()), |skill| skill.twzhengjian_state());
            if !contributors.contains(&target) {
                contributors.push(target);
            }
            let awaken_now = !awakened && contributors.len() >= threshold;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("TW Zhengjian skill disappeared".into()))?
                .set_twzhengjian_state(selected.unwrap_or(0), awakened || awaken_now, contributors);
            if awaken_now {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveTwzhengjian {
                        player,
                        target,
                        helper_skill_def,
                        action: 1,
                        stage: 4,
                        card: CardId::NONE,
                        branch: 0,
                        skill,
                    },
                );
                return run_effects(
                    tx,
                    vec![
                        EffectRequest::RecoverHp {
                            source: player,
                            player,
                            amount: 2,
                        },
                        EffectRequest::AddSkill {
                            player,
                            skill_def: engine_api::SkillDefId::new(helper_skill_def),
                        },
                    ],
                );
            }
            resolve_twzhengjian(
                tx,
                player,
                target,
                helper_skill_def,
                1,
                4,
                CardId::NONE,
                0,
                skill,
            )
        }
        (1, 3) => {
            let (selected, awakened, contributors) = tx
                .state()
                .skills
                .get(skill)
                .map_or((None, false, Vec::new()), |skill| skill.twzhengjian_state());
            let mut events = Vec::new();
            if branch & 0b10 != 0 {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .expect("validated TW Zhengjian skill")
                    .set_twzhengjian_state(1 - selected.unwrap_or(0), awakened, contributors);
            }
            if branch & 0b01 != 0 {
                events.extend(run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: player,
                        target,
                        amount: 1,
                        is_lose_hp: false,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                )?);
            }
            Ok(events)
        }
        (1, 4) => open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::ResolveTwzhengjian,
            vec![3, player.raw(), skill.raw()],
        ),
        _ => Err(EffectError::ContentFault(
            "unknown TW Zhengjian resolver stage".into(),
        )),
    }
}
