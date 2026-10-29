//! VM 效果请求的宿主绑定（bind_effect）：具名技能结算器（Resolve*）。

use super::*;

pub(super) fn bind_effect(
    host: &EngineVmHost<'_>,
    req: EffectRequest,
) -> Result<EffectRequest, HostTrap> {
    Ok(match req {
        EffectRequest::ResolveMbzhuguo {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbzhuguo {
            player,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMbanxian {
            player,
            cards,
            card,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbanxian {
            player,
            cards,
            card,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMbjinzu {
            player,
            target,
            cards,
            card,
            helper_skill_def,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbjinzu {
            player,
            target,
            cards,
            card,
            helper_skill_def,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveYoudi {
            player,
            target,
            card,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveYoudi {
            player,
            target,
            card,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveKuangfu {
            player,
            target,
            card,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveKuangfu {
            player,
            target,
            card,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveShanjia {
            player,
            count,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveShanjia {
            player,
            count,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveNewYajiao {
            player,
            source_card,
            revealed,
            target,
            source_category,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveNewYajiao {
            player,
            source_card,
            revealed,
            target,
            source_category,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDingpin { player, target, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveDingpin {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveZhuiyi { player, killer, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveZhuiyi {
                player,
                killer,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveDecadezhenjun {
            player,
            target,
            stage,
            count,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDecadezhenjun {
            player,
            target,
            stage,
            count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveSpXiaoguo {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveSpXiaoguo {
            player,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveXinfuGuolun {
            player,
            target,
            player_card,
            target_card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveXinfuGuolun {
            player,
            target,
            player_card,
            target_card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDccunsi {
            player,
            target,
            guixiu_def,
            yongjue_def,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDccunsi {
            player,
            target,
            guixiu_def,
            yongjue_def,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDcjue {
            player,
            target,
            stage,
            remaining,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcjue {
            player,
            target,
            stage,
            remaining,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveRuanji {
            player,
            source,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveRuanji {
            player,
            source,
            card,
            action,
            cards: if host.event_cards.is_empty() {
                card.is_some().then_some(card).into_iter().collect()
            } else {
                host.event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect()
            },
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveZhengbi {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveZhengbi {
            player,
            target,
            card,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveFengying {
            player,
            helper_def,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveFengying {
            player,
            helper_def,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveQiaojian {
            player,
            target,
            helper_def,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveQiaojian {
            player,
            target,
            helper_def,
            action,
            cards: host
                .event_cards
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect(),
            split: 0,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveXicha {
            player,
            source,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveXicha {
            player,
            source,
            card,
            action,
            cards: if host.event_cards.is_empty() {
                card.is_some().then_some(card).into_iter().collect()
            } else {
                host.event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect()
            },
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveHuzun {
            player,
            source,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveHuzun {
            player,
            source,
            card,
            action,
            cards: if host.event_cards.is_empty() {
                card.is_some().then_some(card).into_iter().collect()
            } else {
                host.event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect()
            },
            targets: Vec::new(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveChenglve { player, stage, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveChenglve {
                player,
                stage,
                suits: 0,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveShicai {
            player,
            card,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveShicai {
            player,
            card,
            placed: Vec::new(),
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveXiongluan {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveXiongluan {
            player,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveGzbuqu {
            player,
            delta,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveGzbuqu {
            player,
            delta,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveSbbiyue { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveSbbiyue {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveSxrmqiewang {
            player,
            event_player,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveSxrmqiewang {
            player,
            event_player,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveSxrmqianliu {
            player,
            source,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveSxrmqianliu {
            player,
            source,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveGzbuquRecover { player, stage, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveGzbuquRecover {
                player,
                stage,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveTwjuxiang {
            player,
            event_player,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwjuxiang {
            player,
            event_player,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveZhugemengxue {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveZhugemengxue {
            player,
            target,
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

        EffectRequest::ResolveBailingyun {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveBailingyun {
            player,
            target,
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

        EffectRequest::ResolveDcshaowei { player, action, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveDcshaowei {
                player,
                action,
                cards: host
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveDcdichou {
            player,
            target,
            card,
            action,
            helper_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcdichou {
            player,
            target,
            card,
            action,
            helper_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDcdufeng {
            player,
            action,
            helper_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcdufeng {
            player,
            action,
            helper_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDcctjiuxian {
            player,
            event_player,
            target,
            card,
            action,
            cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcctjiuxian {
            player,
            event_player,
            target,
            card,
            action,
            cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveBianyue {
            player,
            target,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveBianyue {
            player,
            target,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveZhupeilan {
            player,
            target,
            card,
            helper_skill_def,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveZhupeilan {
            player,
            target,
            card,
            helper_skill_def,
            action,
            event_cards: host
                .event_cards
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDcSbZhangxiu {
            player,
            target,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcSbZhangxiu {
            player,
            target,
            action,
            cards: Vec::new(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDcSbZhouyu {
            player,
            source,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcSbZhouyu {
            player,
            source,
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

        EffectRequest::ResolveDcSbLusu {
            player,
            source,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcSbLusu {
            player,
            source,
            card,
            action,
            cards: host.event_cards.iter().copied().map(CardId::new).collect(),
            amount: host.ctx.event_amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveZhangjian {
            player,
            card,
            grant_skill_def,
            action,
            target,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveZhangjian {
            player,
            card,
            grant_skill_def,
            action,
            target,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveZhugeruoxue {
            player,
            target,
            card,
            action,
            remaining,
            used_mask,
            repeated,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveZhugeruoxue {
            player,
            target,
            card,
            action,
            remaining,
            used_mask,
            repeated,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveCaoyi {
            player,
            card,
            helper_skill_def,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveCaoyi {
            player,
            card,
            helper_skill_def,
            action,
            event_targets: host
                .event_targets
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMalingli {
            player,
            card,
            helper_skill_def,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMalingli {
            player,
            card,
            helper_skill_def,
            action,
            source: PlayerId::new(host.ctx.event_source),
            target: PlayerId::new(host.ctx.event_player),
            nature: host.event_nature,
            damage_id: 0,
            event_cards: host.event_cards.iter().copied().map(CardId::new).collect(),
            event_targets: host
                .event_targets
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveNsshuaiyan {
            player,
            recovered_player,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveNsshuaiyan {
            player,
            recovered_player,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWdbHanlong {
            player,
            target,
            card,
            helper_skill_def,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWdbHanlong {
            player,
            target,
            card,
            helper_skill_def,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWdbLiufu {
            player,
            target,
            card,
            zhucheng_skill_def,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWdbLiufu {
            player,
            target,
            card,
            zhucheng_skill_def,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWdbLiuyan {
            player,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWdbLiuyan {
            player,
            card: host
                .selected_cards
                .first()
                .copied()
                .map(engine_api::CardId::new)
                .unwrap_or(card),
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWdbLiuzan {
            player,
            target,
            helper_skill_def,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWdbLiuzan {
            player,
            target: host
                .selected_targets
                .first()
                .copied()
                .map(engine_api::PlayerId::new)
                .unwrap_or(target),
            helper_skill_def,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWdbTianyu {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWdbTianyu {
            player,
            target,
            card,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWdbXizhenxihong {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWdbXizhenxihong {
            player,
            target,
            card,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWdbYangyi {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => {
            let event_card = host
                .event_cards
                .first()
                .copied()
                .map(engine_api::CardId::new);
            let gained_target = event_card
                .and_then(|card| host.state.cards.get(card))
                .map(|card| card.owner);
            EffectRequest::ResolveWdbYangyi {
                player,
                target: if action == 1 {
                    gained_target.unwrap_or(target)
                } else {
                    target
                },
                card: if action == 1 {
                    event_card.unwrap_or(card)
                } else {
                    host.selected_cards
                        .first()
                        .copied()
                        .map(engine_api::CardId::new)
                        .unwrap_or(card)
                },
                action,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveWdbZaozhiRenjun {
            player,
            target,
            card,
            cards,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWdbZaozhiRenjun {
            player,
            target,
            card,
            cards,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        req => req,
    })
}
