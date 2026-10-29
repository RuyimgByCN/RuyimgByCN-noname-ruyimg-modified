//! VM 效果请求的宿主绑定（bind_effect）：pot、tw 族结算器。

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

        req => req,
    })
}
