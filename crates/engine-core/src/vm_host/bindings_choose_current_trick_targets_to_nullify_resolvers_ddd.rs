//! VM 效果请求的宿主绑定（bind_effect）：ddd 族结算器。

use super::*;

pub(super) fn bind_effect(
    host: &EngineVmHost<'_>,
    req: EffectRequest,
) -> Result<EffectRequest, HostTrap> {
    Ok(match req {
        EffectRequest::ResolveDddXianxi {
            player,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddXianxi {
            player,
            card,
            action,
            event_targets: host
                .event_targets
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect(),
            damaged_targets: host
                .event_targets
                .iter()
                .enumerate()
                .filter(|(index, _)| {
                    1u32.checked_shl(*index as u32)
                        .is_some_and(|bit| u32::from(host.event_nature) & bit != 0)
                })
                .map(|(_, target)| PlayerId::new(*target))
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddXiaheng {
            player,
            discard_target,
            damage_target,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddXiaheng {
            player,
            discard_target,
            damage_target,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddLiuchong {
            player,
            actor,
            card,
            action,
            cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddLiuchong {
            player,
            actor,
            card,
            action,
            cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddDuanbing {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddDuanbing {
            player,
            target,
            card,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddWuzhi {
            player,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddWuzhi {
            player,
            card,
            action,
            cards: host
                .event_cards
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddXujing {
            player,
            source,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddXujing {
            player,
            source,
            card,
            action,
            targets: host
                .event_targets
                .iter()
                .copied()
                .map(engine_api::PlayerId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddCaomao {
            player,
            action,
            cards,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddCaomao {
            player,
            action,
            cards,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddXinxianying {
            player,
            actor,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddXinxianying {
            player,
            actor,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddXianglang {
            player,
            actor,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddXianglang {
            player,
            actor,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddYujin {
            player,
            actor,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddYujin {
            player,
            actor,
            target: host.event_move_target,
            move_mask: host.event_move_mask,
            action,
            candidates: Vec::new(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddLiuye {
            player,
            actor,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddLiuye {
            player,
            actor,
            card,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddBaosanniang {
            player,
            target,
            card,
            action,
            cards,
            targets,
            used_kind,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddBaosanniang {
            player,
            target,
            card,
            action,
            cards,
            targets,
            used_kind,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddZhenji {
            player,
            target,
            action,
            stage,
            piles,
            cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddZhenji {
            player,
            target,
            action,
            stage,
            piles,
            cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddZhaoang {
            player,
            action,
            stage,
            card,
            kind,
            count,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddZhaoang {
            player,
            action,
            stage,
            card,
            kind,
            count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddLiuba {
            player,
            actor,
            action,
            cards,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddLiuba {
            player,
            actor,
            action,
            cards: if action == 0 {
                host.selected_cards
                    .iter()
                    .copied()
                    .map(CardId::new)
                    .collect()
            } else {
                cards
            },
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },
        req => req,
    })
}
