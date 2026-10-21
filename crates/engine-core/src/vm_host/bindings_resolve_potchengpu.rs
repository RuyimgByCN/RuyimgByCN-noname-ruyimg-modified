//! Effect bindings for `ResolvePotchengpu` and related requests.

use super::*;

pub(super) fn bind_effect(
    host: &EngineVmHost<'_>,
    req: EffectRequest,
) -> Result<EffectRequest, HostTrap> {
    Ok(match req {
        EffectRequest::ResolvePotchengpu {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotchengpu {
            player,
            target,
            stage,
            damage_id: 0,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePotjugu { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolvePotjugu {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolvePotsifeng {
            player,
            target,
            card,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotsifeng {
            player,
            target,
            card,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePotchenqun {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotchenqun {
            player,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePotwanglie {
            player,
            card,
            helper_skill_def,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotwanglie {
            player,
            card,
            helper_skill_def,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePothongyi { player, stage, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolvePothongyi {
                player,
                stage,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolvePotguansha {
            player,
            helper_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotguansha {
            player,
            helper_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePotjiyu {
            player,
            card,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotjiyu {
            player,
            card,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

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

        EffectRequest::ResolvePotlibing {
            player,
            card,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotlibing {
            player,
            card,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePotpoxi { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolvePotpoxi {
                player,
                cards: host
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolvePotfuan {
            player,
            card,
            stage,
            range_helper_def,
            yinxian_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotfuan {
            player,
            card,
            stage,
            range_helper_def,
            yinxian_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePotyinxian {
            player,
            card,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotyinxian {
            player,
            card,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePothuanjie {
            player,
            target,
            stage,
            qice_def,
            kanpo_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePothuanjie {
            player,
            target,
            stage,
            qice_def,
            kanpo_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePotnigu {
            player,
            target,
            card,
            targets,
            index,
            declined,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotnigu {
            player,
            target,
            card,
            targets,
            index,
            declined,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePotlulian {
            player,
            card,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotlulian {
            player,
            card,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePotdimeng {
            player, targets, ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotdimeng {
            player,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePothaoshi {
            player,
            target,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePothaoshi {
            player,
            target,
            action,
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

        EffectRequest::ResolveTwqiushou { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveTwqiushou {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolvePottuntian {
            player,
            action,
            amount,
            targets,
            zaoxian_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePottuntian {
            player,
            action,
            amount,
            targets,
            zaoxian_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePotjixi {
            player,
            target,
            action,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotjixi {
            player,
            target,
            action,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePotceduan {
            player,
            target,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotceduan {
            player,
            target,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePottaishici {
            player,
            target,
            action,
            cards,
            hanzhan_def,
            zhanlie_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePottaishici {
            player,
            target,
            action,
            cards,
            hanzhan_def,
            zhanlie_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

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

        EffectRequest::ResolvePotjiejie {
            player,
            actor,
            used_def,
            suit_defs,
            qingshi_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotjiejie {
            player,
            actor,
            used_def,
            suit_defs,
            qingshi_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePotqingshi { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolvePotqingshi {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolvePotweiyan {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotweiyan {
            player,
            target,
            card,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePotzhouyu {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotzhouyu {
            player,
            target,
            card,
            action,
            card_flags: host.event_card_flags,
            targets: host
                .event_targets
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePotlvyi {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotlvyi {
            player,
            target,
            card,
            action,
            event_cards: host
                .event_cards
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect(),
            targets: host
                .event_targets
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePotcaoshuang {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotcaoshuang {
            player,
            target,
            card,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolvePotyuji {
            player,
            target,
            card,
            action,
            cards,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolvePotyuji {
            player,
            target,
            card,
            action,
            cards: if action == 0 {
                host.selected_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect()
            } else if cards.is_empty() {
                host.event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect()
            } else {
                cards
            },
            targets: if action == 0 {
                host.selected_targets
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect()
            } else if targets.is_empty() {
                host.event_targets
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect()
            } else {
                targets
            },
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

        EffectRequest::ResolveTwxiawei {
            player,
            wangxing_skill_def,
            action,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwxiawei {
            player,
            wangxing_skill_def,
            action,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwbianfuren {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwbianfuren {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwfeiyi {
            player,
            action,
            stage,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwfeiyi {
            player,
            action,
            stage,
            card,
            event_cards: host
                .event_cards
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwchenzhen {
            player,
            target,
            helper_skill_def,
            record_skill_def,
            action,
            stage,
            card_kind,
            card,
            branch,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwchenzhen {
            player,
            target,
            helper_skill_def,
            record_skill_def,
            action,
            stage,
            card_kind,
            card,
            branch,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwqianju { player, target, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveTwqianju {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveTwqingxi {
            player,
            target,
            card,
            stage,
            count,
            cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwqingxi {
            player,
            target,
            card,
            stage,
            count,
            cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwzaoli {
            player,
            stage,
            cards,
            slots,
            count,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwzaoli {
            player,
            stage,
            cards,
            slots,
            count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwzhengjian {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwzhengjian {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwtianyu {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwtianyu {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwwujing {
            player,
            target_count,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwwujing {
            player,
            target_count,
            action,
            event_cards: host
                .event_cards
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwshanghe {
            player,
            stage,
            remaining,
            got_jiu,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwshanghe {
            player,
            stage,
            remaining,
            got_jiu,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwwangchang {
            player,
            target,
            card,
            selected_card,
            action,
            stage,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwwangchang {
            player,
            target,
            card,
            selected_card,
            action,
            stage,
            targets,
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

        EffectRequest::ResolveTwguohuai {
            player,
            target,
            source,
            card,
            mark_def,
            helper_skill_def,
            action,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwguohuai {
            player,
            target,
            source,
            card,
            mark_def,
            helper_skill_def,
            action,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwchengpu {
            player,
            target,
            source,
            card,
            chunlao_skill_def,
            action,
            stage,
            selected_card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwchengpu {
            player,
            target,
            source,
            card,
            chunlao_skill_def,
            action,
            stage,
            selected_card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwcaoang {
            player,
            target,
            source,
            card,
            helper_skill_def,
            action,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwcaoang {
            player,
            target,
            source,
            card,
            helper_skill_def,
            action,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwniujin {
            player,
            limited_skill_def,
            action,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwniujin {
            player,
            limited_skill_def,
            action,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwmoukui {
            player,
            target,
            card,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwmoukui {
            player,
            target,
            card,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwfuman {
            player,
            target,
            cards,
            action,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwfuman {
            player,
            target,
            cards,
            action,
            stage,
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

        EffectRequest::ResolveTwqingkou {
            player,
            winner,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwqingkou {
            player,
            winner,
            card,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwchenxun {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwchenxun {
            player,
            target,
            card,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwfenxian {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwfenxian {
            player,
            target,
            card,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::SetSkillContentFlag {
            player, bit, value, ..
        } if host.ctx.skill_id != 0 => EffectRequest::SetSkillContentFlag {
            player,
            bit,
            value,
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
