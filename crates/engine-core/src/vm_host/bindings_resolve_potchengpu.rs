//! VM 效果请求的宿主绑定（bind_effect）：通用效果原语。

use super::*;

pub(super) fn bind_effect(
    host: &EngineVmHost<'_>,
    req: EffectRequest,
) -> Result<EffectRequest, HostTrap> {
    Ok(match req {
        EffectRequest::SetSkillContentFlag {
            player, bit, value, ..
        } if host.ctx.skill_id != 0 => EffectRequest::SetSkillContentFlag {
            player,
            bit,
            value,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::DistributeTopTwoCards {
            player,
            cards,
            selected,
            target,
            ready_to_choose,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::DistributeTopTwoCards {
            player,
            cards,
            selected,
            target,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_choose,
        },

        EffectRequest::ChooseDiscardBasicThenTargetDiscardEquipOrDamage {
            player, target, ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseDiscardBasicThenTargetDiscardEquipOrDamage {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner { player, target, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseTargetLoseHpDiscardHandShaIfBasic {
            player,
            target,
            stage,
            discarded_basic,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseTargetLoseHpDiscardHandShaIfBasic {
            player,
            target,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            stage,
            discarded_basic,
        },

        EffectRequest::ChooseCurrentShaTargetDiscardRedirectToOwner {
            owner,
            source,
            card,
            target,
            ready_to_redirect,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseCurrentShaTargetDiscardRedirectToOwner {
                owner,
                source,
                card,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
                ready_to_redirect,
            }
        }

        EffectRequest::ChooseDiscardRedirectCurrentSha {
            player,
            source,
            card,
            target,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseDiscardRedirectCurrentSha {
            player,
            source,
            card,
            target,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseTargetGainBottomCard {
            player,
            card,
            target,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseTargetGainBottomCard {
            player,
            card,
            target,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::JudgeBlackGainRepeat {
            player,
            card,
            ready_to_choose,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::JudgeBlackGainRepeat {
            player,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_choose,
        },

        EffectRequest::JudgeBlackGainRepeatProtectHandLimit {
            player,
            card,
            ready_to_choose,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::JudgeBlackGainRepeatProtectHandLimit {
            player,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_choose,
        },

        EffectRequest::JudgeSameColorGainRepeat {
            player,
            card,
            ready_to_choose,
            color,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::JudgeSameColorGainRepeat {
            player,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_choose,
            color,
        },
        req => req,
    })
}
