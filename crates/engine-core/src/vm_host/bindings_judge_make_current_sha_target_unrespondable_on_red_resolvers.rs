//! VM 效果请求的宿主绑定（bind_effect）：具名技能结算器（Resolve*）。

use super::*;

pub(super) fn bind_effect(
    host: &EngineVmHost<'_>,
    req: EffectRequest,
) -> Result<EffectRequest, HostTrap> {
    Ok(match req {
        EffectRequest::ResolveDingpan {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDingpan {
            player,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTanlu {
            player,
            target,
            amount,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTanlu {
            player,
            target,
            amount,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveHiddenSkillPile {
            player,
            pile_skill_def,
            action,
            card,
            stage,
            hand_cards,
            pile_cards,
            event_cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveHiddenSkillPile {
            player,
            pile_skill_def,
            action,
            card,
            stage,
            hand_cards,
            pile_cards,
            event_cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDiliYuheng {
            player,
            action,
            card,
            event_cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDiliYuheng {
            player,
            action,
            card,
            event_cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveGuanchaoRank { player, card, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveGuanchaoRank {
                player,
                card,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveGuanwei {
            player,
            target,
            stage,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveGuanwei {
            player,
            target,
            stage,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveLanjiang {
            player,
            stage,
            index,
            target,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveLanjiang {
            player,
            stage,
            index,
            target,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveRebingyi { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveRebingyi {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveRebotu { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveRebotu {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveOldimeng {
            player,
            stage,
            first,
            second,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveOldimeng {
            player,
            stage,
            first,
            second,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveRejieyin {
            player,
            target,
            stage,
            card,
            equip,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveRejieyin {
            player,
            target,
            stage,
            card,
            equip,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveReanxu {
            player,
            stage,
            first,
            second,
            card,
            from_hand,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveReanxu {
            player,
            stage,
            first,
            second,
            card,
            from_hand,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveOldAnxu {
            player,
            stage,
            giver,
            receiver,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveOldAnxu {
            player,
            stage,
            giver,
            receiver,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDcbingji {
            player,
            stage,
            kind,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcbingji {
            player,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            stage,
            kind,
        },

        EffectRequest::ResolveQuanjia {
            player,
            source,
            target,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveQuanjia {
            player,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            source,
            target,
        },

        EffectRequest::ResolveDuwang { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveDuwang {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveOldQinqing {
            player,
            target,
            lord,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveOldQinqing {
            player,
            target,
            lord,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveOldMingjian {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveOldMingjian {
            player,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveOldXuanfeng {
            player,
            target,
            card,
            stage,
            branch,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveOldXuanfeng {
            player,
            target,
            card,
            stage,
            branch,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveJianrou {
            player,
            damage_id: 0,
            ..
        } => EffectRequest::ResolveJianrou {
            player,
            damage_id: host.state.resolution.current_damage_id,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveOldHuisheng {
            player,
            source,
            damage_id: 0,
            stage,
            cards,
            ..
        } => EffectRequest::ResolveOldHuisheng {
            player,
            source: if source.is_some() {
                source
            } else {
                PlayerId::new(host.ctx.event_source)
            },
            damage_id: host.state.resolution.current_damage_id,
            stage,
            cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveOldJiefan {
            player,
            dying,
            current,
            card,
            damage_id,
            stage,
            ..
        } => EffectRequest::ResolveOldJiefan {
            player,
            dying: if dying.is_some() {
                dying
            } else {
                PlayerId::new(host.ctx.event_player)
            },
            current: if current.is_some() {
                current
            } else {
                host.state.turn.current
            },
            card: if card.is_some() {
                card
            } else {
                engine_api::CardId::new(host.ctx.event_card)
            },
            damage_id: if damage_id != 0 {
                damage_id
            } else {
                host.state.resolution.current_damage_id
            },
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveChegu {
            player,
            card,
            targets,
            cards,
            branch,
            index,
            stage,
            ..
        } => EffectRequest::ResolveChegu {
            player,
            card: if card.is_some() {
                card
            } else {
                engine_api::CardId::new(host.ctx.event_card)
            },
            targets: if targets.is_empty() {
                host.event_targets
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect()
            } else {
                targets
            },
            cards,
            branch,
            index,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveJunkguixin {
            player,
            target,
            stage,
            group,
            ..
        } => EffectRequest::ResolveJunkguixin {
            player,
            target,
            stage,
            group,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveYaoChenwei {
            player,
            target,
            card,
            mode,
            stage,
            ..
        } => EffectRequest::ResolveYaoChenwei {
            player,
            target,
            card,
            mode,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDrltJueyan {
            player,
            stage,
            slot,
            first,
            sha_skill_def,
            weapon_skill_def,
            mount_skill_def,
            armor_skill_def,
            treasure_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDrltJueyan {
            player,
            stage,
            slot,
            first,
            sha_skill_def,
            weapon_skill_def,
            mount_skill_def,
            armor_skill_def,
            treasure_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveRejizhi {
            player,
            stage,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveRejizhi {
            player,
            stage,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveJiexun {
            player,
            stage,
            target,
            draw_count,
            discard_count,
            funan_helper_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveJiexun {
            player,
            stage,
            target,
            draw_count,
            discard_count,
            funan_helper_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWenguaSetup {
            player,
            helper_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWenguaSetup {
            player,
            helper_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWengua {
            player,
            stage,
            target,
            card,
            branch,
            main_skill_def,
            marker_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWengua {
            player,
            stage,
            target,
            card,
            branch,
            main_skill_def,
            marker_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveXinanguo {
            player,
            target,
            stage,
            index,
            remaining,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveXinanguo {
            player,
            target,
            stage,
            index,
            remaining,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveGanglieThree { player, target, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveGanglieThree {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveZhongyi {
            player,
            action,
            helper_skill_def,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveZhongyi {
            player,
            action,
            helper_skill_def,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMassDamageDiscardTurnOver {
            player,
            mark,
            mark_count,
            damage_amount,
            hand_count,
            stage,
            index,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMassDamageDiscardTurnOver {
            player,
            mark,
            mark_count,
            damage_amount,
            hand_count,
            stage,
            index,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMassDamageDiscardHandEquip {
            player,
            mark,
            mark_count,
            damage_amount,
            discard_count,
            stage,
            index,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMassDamageDiscardHandEquip {
            player,
            mark,
            mark_count,
            damage_amount,
            discard_count,
            stage,
            index,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMarkParityControl {
            player,
            mark,
            mass_threshold,
            damage_amount,
            stage,
            index,
            target,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMarkParityControl {
            player,
            mark,
            mass_threshold,
            damage_amount,
            stage,
            index,
            target,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage {
            player,
            damage_amount,
            nature,
            stage,
            index,
            targets,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage {
                player,
                damage_amount,
                nature,
                stage,
                index,
                targets,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveYeyan {
            player,
            mode,
            stage,
            index,
            targets,
            data,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveYeyan {
            player,
            mode,
            stage,
            index,
            targets,
            data,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveQixing {
            player,
            initialize,
            stage,
            hand_cards,
            pile_cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveQixing {
            player,
            initialize,
            stage,
            hand_cards,
            pile_cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveQixingWeather {
            player,
            qixing_skill_def,
            helper_skill_def,
            max_targets,
            stage,
            cards,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveQixingWeather {
            player,
            qixing_skill_def,
            helper_skill_def,
            max_targets,
            stage,
            cards,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveNzryLongnu {
            player,
            fire_skill_def,
            thunder_skill_def,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveNzryLongnu {
            player,
            fire_skill_def,
            thunder_skill_def,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDrltPoxi {
            player,
            target,
            hand_limit_skill_def,
            stage,
            own_count,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDrltPoxi {
            player,
            target,
            hand_limit_skill_def,
            stage,
            own_count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDrltJieying {
            player,
            event_player,
            mark,
            mode,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDrltJieying {
            player,
            event_player,
            mark,
            mode,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDrltDuorui {
            player,
            target,
            helper_skill_def,
            stage,
            selected_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDrltDuorui {
            player,
            target,
            helper_skill_def,
            stage,
            selected_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDrltZhiti {
            player,
            event_player,
            event_source,
            event_amount,
            mode,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDrltZhiti {
            player,
            event_player,
            event_source,
            event_amount,
            mode,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveShenSimayi {
            player,
            event_player,
            event_amount,
            mark,
            mode,
            cards,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveShenSimayi {
            player,
            event_player,
            event_amount,
            mark,
            mode,
            cards,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveBeige {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveBeige {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveBeigeConfigured {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            heart_recover_amount,
            diamond_draw_count,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveBeigeConfigured {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            heart_recover_amount,
            diamond_draw_count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveOlwenren {
            player, targets, ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveOlwenren {
            player,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        req => req,
    })
}
