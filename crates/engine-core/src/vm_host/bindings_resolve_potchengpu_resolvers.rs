//! VM 效果请求的宿主绑定（bind_effect）：具名技能结算器（Resolve*）。

use super::*;

pub(super) fn bind_effect(
    host: &EngineVmHost<'_>,
    req: EffectRequest,
) -> Result<EffectRequest, HostTrap> {
    Ok(match req {
        EffectRequest::ResolveMbzhuji {
            player,
            choice,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbzhuji {
            player,
            choice,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveSpmiaolve { player, action, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveSpmiaolve {
                player,
                action,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveSpyingjia {
            player,
            action,
            helper_def,
            target,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveSpyingjia {
            player,
            action,
            helper_def,
            card: engine_api::CardId::new(host.ctx.event_card),
            card_kind: host.event_card_kind,
            target,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTiaoxin {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTiaoxin {
            player,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveZhiji {
            player,
            guanxing_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveZhiji {
            player,
            guanxing_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveZaiqi { player, extra, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveZaiqi {
                player,
                extra,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveMbzhonghui {
            player,
            target,
            action,
            amount,
            mbsizi_def,
            mbkechang_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbzhonghui {
            player,
            target,
            action,
            amount,
            mbsizi_def,
            mbkechang_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveHaoshi { player, stage, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveHaoshi {
                player,
                stage,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveDimeng {
            player,
            targets,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDimeng {
            player,
            targets,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveQiaobian {
            player,
            phase,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveQiaobian {
            player,
            phase,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveXiangle {
            player,
            source,
            card,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveXiangle {
            player,
            source,
            card,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveFangquanSkip { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveFangquanSkip {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveFangquanGrant { player, stage, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveFangquanGrant {
                player,
                stage,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveRuoyu {
            player,
            jijiang_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveRuoyu {
            player,
            jijiang_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveZuilun { player, stage, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveZuilun {
                player,
                stage,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveFuyin {
            player,
            source,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveFuyin {
            player,
            source,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveJuzhan {
            player,
            source,
            card,
            targets,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveJuzhan {
            player,
            source,
            card,
            targets,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveQizhi {
            player,
            targets,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveQizhi {
            player,
            targets,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveJinqu {
            player,
            qizhi_def,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveJinqu {
            player,
            qizhi_def,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveJianxiang { player, stage, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveJianxiang {
                player,
                stage,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveShenshi {
            player,
            source,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveShenshi {
            player,
            source,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMingren { player, stage, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveMingren {
                player,
                stage,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveZhenliang {
            player,
            source,
            card,
            mingren_def,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveZhenliang {
            player,
            source,
            card,
            mingren_def,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveKuizhu { player, stage, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveKuizhu {
                player,
                stage,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveZhizheng { player, stage, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveZhizheng {
                player,
                stage,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveLijun {
            player,
            source,
            card,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveLijun {
            player,
            source,
            card,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveLiangyin {
            player,
            kongsheng_def,
            stage,
            target,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveLiangyin {
            player,
            kongsheng_def,
            stage,
            target,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveKongsheng {
            player,
            stage,
            target,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveKongsheng {
            player,
            stage,
            target,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveReleiji {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveReleiji {
            player,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveHuangtian {
            player,
            actor,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveHuangtian {
            player,
            actor,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveZhengrong {
            player,
            candidates,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveZhengrong {
            player,
            candidates,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveHongju {
            player,
            qingjian_def,
            rong_def,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveHongju {
            player,
            qingjian_def,
            rong_def,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveQingjian {
            player,
            rong_def,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveQingjian {
            player,
            rong_def,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveXinfuLianpian { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveXinfuLianpian {
                player,
                targets: host
                    .event_targets
                    .iter()
                    .copied()
                    .map(engine_api::PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveJieliangGain { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveJieliangGain {
                player,
                mover: engine_api::PlayerId::new(host.ctx.event_source),
                cards: host
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveShuangxiong { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveShuangxiong {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveZhijian {
            player,
            card,
            stage,
            target,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveZhijian {
            player,
            card,
            stage,
            target,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveGuzheng {
            player,
            event_player,
            stage,
            cards,
            return_card,
            take_rest,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveGuzheng {
            player,
            event_player,
            stage,
            cards,
            return_card,
            take_rest,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveZhiba { player, actor, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveZhiba {
                player,
                actor,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveDrltCongjian {
            player,
            targets: _,
            card,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDrltCongjian {
            player,
            targets: host
                .event_targets
                .iter()
                .copied()
                .map(engine_api::PlayerId::new)
                .collect(),
            card,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveNzryFeijun {
            player,
            target,
            card,
            branch,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveNzryFeijun {
            player,
            target,
            card,
            branch,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDrltYongsi { player, stage, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveDrltYongsi {
                player,
                stage,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveDrltWeidi {
            player,
            cards,
            targets,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDrltWeidi {
            player,
            cards,
            targets,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveLieren {
            player,
            target,
            win_skill_def,
            lose_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveLieren {
            player,
            target,
            win_skill_def,
            lose_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveRehengjiang {
            player,
            actor,
            action,
            hand_limit_def,
            draw_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveRehengjiang {
            player,
            actor,
            action,
            hand_limit_def,
            draw_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveCuijin {
            player,
            source,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveCuijin {
            player,
            source,
            card,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveJiaohua {
            player,
            category,
            target,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveJiaohua {
            player,
            category,
            target,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveLuanqun {
            player,
            effect_skill_def,
            direct_hit_skill_def,
            stage,
            participants,
            cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveLuanqun {
            player,
            effect_skill_def,
            direct_hit_skill_def,
            stage,
            participants,
            cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        req => req,
    })
}
