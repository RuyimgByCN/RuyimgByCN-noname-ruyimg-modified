//! Effect primitives for `resolve_nzry_longnu` and related helpers.

use super::*;

pub(crate) fn resolve_nzry_longnu(
    tx: &mut Transaction,
    player: PlayerId,
    fire_skill_def: engine_api::SkillDefId,
    thunder_skill_def: engine_api::SkillDefId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if fire_skill_def.is_none() || thunder_skill_def.is_none() || stage > 1 {
        return Err(EffectError::ContentFault(
            "Longnu requires two helper skills and a valid stage".into(),
        ));
    }
    if stage == 0 {
        let fire = {
            let state = tx.state();
            let instance = state
                .skills
                .get(skill)
                .ok_or_else(|| EffectError::ContentFault("Longnu skill missing".into()))?;
            if instance.owner != player
                || !state.skill_effectively_enabled(instance)
                || state.turn.current != player
                || state.turn.phase != Phase::Play
                || !state
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            !instance.nzry_longnu_fire_branch()
        };
        tx.state_mut()
            .skills
            .get_mut(skill)
            .unwrap()
            .set_nzry_longnu_fire_branch(fire);
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ResolveNzryLongnu {
                player,
                fire_skill_def,
                thunder_skill_def,
                stage: 1,
                skill,
            });
        if fire {
            return super::super::run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .map(|_| Vec::new());
        }
        return change_max_hp(tx, player, -1);
    }

    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let fire = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| instance.owner == player)
        .is_some_and(|instance| instance.nzry_longnu_fire_branch());
    super::super::run_effects(
        tx,
        vec![
            EffectRequest::DrawCards { player, count: 1 },
            EffectRequest::AddSkill {
                player,
                skill_def: if fire {
                    fire_skill_def
                } else {
                    thunder_skill_def
                },
            },
        ],
    )
    .map(|_| Vec::new())
}
