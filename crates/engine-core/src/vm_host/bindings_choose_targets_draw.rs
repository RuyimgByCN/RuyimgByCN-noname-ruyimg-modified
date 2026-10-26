//! Effect bindings for `ChooseTargetsDraw` and related requests.

use super::*;

pub(super) fn bind_effect(
    host: &EngineVmHost<'_>,
    req: EffectRequest,
) -> Result<EffectRequest, HostTrap> {
    Ok(match req {
        EffectRequest::ChooseTargetsDraw {
            player,
            count,
            target_count,
            scope,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseTargetsDraw {
            player,
            count,
            target_count,
            scope,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseTargetsDamage {
            player,
            min_count,
            max_count,
            scope,
            amount,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseTargetsDamage {
            player,
            min_count,
            max_count,
            scope,
            amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseOptionalTargetsDraw {
            player,
            count,
            min_count,
            max_count,
            scope,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseOptionalTargetsDraw {
            player,
            count,
            min_count,
            max_count,
            scope,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::RevealTargetHandShaOrDiscardBind {
            player,
            target,
            helper_skill_def,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::RevealTargetHandShaOrDiscardBind {
            player,
            target,
            helper_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            stage,
        },

        EffectRequest::DrawRememberedHandCards { player, count, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::DrawRememberedHandCards {
                player,
                count,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::DiscardRememberedHandCards { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::DiscardRememberedHandCards {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::DiscardTargetEquipmentDrawWeaponRange { player, target, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::DiscardTargetEquipmentDrawWeaponRange {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseTargetRevealHandDamageIfSha { player, amount, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseTargetRevealHandDamageIfSha {
                player,
                amount,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseOtherGiveOneHandEquip { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseOtherGiveOneHandEquip {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }
        EffectRequest::ChooseGiveOrRecastOneHand {
            player,
            require_skill_def,
            grant_skill_def,
            grant_mark_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseGiveOrRecastOneHand {
            player,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            require_skill_def,
            grant_skill_def,
            grant_mark_def,
        },

        EffectRequest::ChooseRevealHandCardBindCategoryTarget { player, target, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseRevealHandCardBindCategoryTarget {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::RecoverToMaxThenOptionalExchangeSkill {
            player,
            removable_skill_def,
            restore_skill_def,
            ready_to_choose,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::RecoverToMaxThenOptionalExchangeSkill {
            player,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            removable_skill_def,
            restore_skill_def,
            ready_to_choose,
        },

        EffectRequest::ChooseTargetDrawToMaxHpCap { player, cap, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseTargetDrawToMaxHpCap {
                player,
                cap,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { player, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::DrawByPlayPhaseCardCategoryCount { player, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::DrawByPlayPhaseCardCategoryCount {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed {
            player,
            sha_user,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed {
            player,
            sha_user,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveOllinjie { player, source, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveOllinjie {
                player,
                source,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
            player,
            sha_user,
            card,
            restore_skill_def,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
                player,
                sha_user,
                card,
                restore_skill_def,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::QueueOpeningExtraPlayPhase { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::QueueOpeningExtraPlayPhase {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage { player, card, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage {
                player,
                card,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::IncreaseCurrentShaBaseDamage {
            player,
            card,
            amount,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::IncreaseCurrentShaBaseDamage {
            player,
            card,
            amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::IncreaseCurrentCardBaseDamage {
            player,
            card,
            amount,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::IncreaseCurrentCardBaseDamage {
            player,
            card,
            amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
            player,
            card,
            draw_count,
            damage_amount,
            mark_def,
            mark_count,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
                player,
                card,
                draw_count,
                damage_amount,
                mark_def,
                mark_count,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::DrawThenChooseGiveOrDiscardHand {
            player,
            draw_count,
            payment_count,
            ready_to_choose,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::DrawThenChooseGiveOrDiscardHand {
            player,
            draw_count,
            payment_count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_choose,
        },

        EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp {
            player,
            target,
            count,
            ready_to_check,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp {
            player,
            target,
            count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_check,
        },

        EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess {
            player,
            target,
            cap,
            ready_to_discard,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess {
            player,
            target,
            cap,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_discard,
        },

        EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
            player,
            target,
            card,
            suit,
            ready_to_choose,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
                player,
                target,
                card,
                suit,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
                ready_to_choose,
            }
        }

        EffectRequest::TargetChooseSuitGainOwnerHandDamageIfMismatch { player, target, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::TargetChooseSuitGainOwnerHandDamageIfMismatch {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo {
            player,
            target,
            cards,
            ready_to_recover,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo {
            player,
            target,
            cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_recover,
        },

        EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
            player,
            targets,
            index,
            selected,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
            player,
            targets,
            index,
            selected,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
            player,
            targets,
            index,
            selected,
            draw_count,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
            player,
            targets,
            index,
            selected,
            draw_count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDcqiaomeng {
            player,
            card,
            target,
            selected,
            helper_skill_def,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcqiaomeng {
            player,
            card,
            targets: host
                .event_targets
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect(),
            target,
            selected,
            helper_skill_def,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveNewReluoyi {
            player,
            buff_skill_def,
            stage,
            cards,
            accepted,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveNewReluoyi {
            player,
            buff_skill_def,
            stage,
            cards,
            accepted,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveRemoshi {
            player,
            target,
            card,
            helper_skill_def,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveRemoshi {
            player,
            target,
            card,
            helper_skill_def,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveXinyaoming {
            player,
            target,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveXinyaoming {
            player,
            target,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveRehuaiyi {
            player,
            stage,
            color,
            discarded_count,
            targets,
            index,
            gained_count,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveRehuaiyi {
            player,
            stage,
            color,
            discarded_count,
            targets,
            index,
            gained_count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveReanguo {
            player,
            target,
            stage,
            index,
            remaining,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveReanguo {
            player,
            target,
            stage,
            index,
            remaining,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveBeizhu {
            player,
            target,
            stage,
            cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveBeizhu {
            player,
            target,
            stage,
            cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMbshanjia {
            player,
            lost_equipment_count,
            distance_skill_def,
            stage,
            reward_mask,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbshanjia {
            player,
            lost_equipment_count,
            distance_skill_def,
            stage,
            reward_mask,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDaoji {
            player,
            target,
            card,
            stage,
            tw,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDaoji {
            player,
            target,
            card,
            stage,
            tw,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::GiveAllHandThenChooseUseOneOrDamage {
            player,
            target,
            cards,
            targets,
            stage,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::GiveAllHandThenChooseUseOneOrDamage {
            player,
            target,
            cards,
            targets,
            stage,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
            player,
            card,
            target,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
                player,
                card,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants {
            player,
            card,
            targets,
            stage,
            damage_bonus,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants {
            player,
            card,
            targets,
            stage,
            damage_bonus,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwfupan {
            player,
            damage_amount,
            card,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwfupan {
            player,
            damage_amount,
            card,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwjiaohua {
            player,
            target,
            event_cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwjiaohua {
            player,
            target,
            event_cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwkujian {
            player,
            target,
            cards,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwkujian {
            player,
            target,
            cards,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwruilian {
            player,
            target,
            event_cards,
            action,
            category,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwruilian {
            player,
            target,
            event_cards,
            action,
            category,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwjianwei {
            player,
            actor,
            target,
            card,
            winner,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwjianwei {
            player,
            actor,
            target,
            card,
            winner,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwtanfeng {
            player,
            target,
            card,
            stage,
            phase,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwtanfeng {
            player,
            target,
            card,
            stage,
            phase,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwzhibian {
            player,
            target,
            winner,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwzhibian {
            player,
            target,
            winner,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwyuyan {
            player,
            source,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwyuyan {
            player,
            source,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwguoyi {
            player,
            target,
            card,
            hand_lock_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwguoyi {
            player,
            target,
            card,
            hand_lock_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwchuhai {
            player,
            action,
            stage,
            index,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwchuhai {
            player,
            action,
            stage,
            index,
            event_cards: host
                .event_cards
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwyizhu {
            player,
            source,
            card,
            action,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwyizhu {
            player,
            source,
            card,
            action,
            stage,
            event_cards: host
                .event_cards
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTwluanchou {
            player,
            helper_skill_def,
            action,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwluanchou {
            player,
            helper_skill_def,
            action,
            targets,
            source: engine_api::PlayerId::new(host.ctx.event_source),
            target: engine_api::PlayerId::new(host.ctx.event_player),
            amount: host.ctx.event_amount,
            nature: host.event_nature,
            card: engine_api::CardId::new(host.ctx.event_card),
            card_kind: host.event_card_kind,
            card_flags: host.event_card_flags,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveYuli { player, mode, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveYuli {
                player,
                mode,
                source: engine_api::PlayerId::new(host.ctx.event_source),
                target: engine_api::PlayerId::new(host.ctx.event_player),
                amount: host.ctx.event_amount,
                nature: host.event_nature,
                card: engine_api::CardId::new(host.ctx.event_card),
                card_kind: host.event_card_kind,
                card_flags: host.event_card_flags,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveTingwei { player, mode, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveTingwei {
                player,
                mode,
                card: engine_api::CardId::new(host.ctx.event_card),
                targets: host
                    .event_targets
                    .iter()
                    .map(|raw| engine_api::PlayerId::new(*raw))
                    .collect(),
                target: engine_api::PlayerId::NONE,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveXinmingshi { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveXinmingshi {
                player,
                source: engine_api::PlayerId::new(host.ctx.event_source),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveXunyi { player, mode, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveXunyi {
                player,
                mode,
                source: engine_api::PlayerId::new(host.ctx.event_source),
                target: engine_api::PlayerId::new(host.ctx.event_player),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveBoming { player, target, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveBoming {
                player,
                target,
                card: host
                    .selected_cards
                    .first()
                    .copied()
                    .map(engine_api::CardId::new)
                    .unwrap_or(engine_api::CardId::NONE),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveSpqiai { player, target, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveSpqiai {
                player,
                target,
                card: host
                    .selected_cards
                    .first()
                    .copied()
                    .map(engine_api::CardId::new)
                    .unwrap_or(engine_api::CardId::NONE),
                stage: 0,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveEjian { player, target, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveEjian {
                player,
                target,
                card: host
                    .event_cards
                    .first()
                    .copied()
                    .map(engine_api::CardId::new)
                    .unwrap_or(engine_api::CardId::NONE),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveYiyong { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveYiyong {
                player,
                card: engine_api::CardId::new(host.ctx.event_card),
                source: engine_api::PlayerId::new(host.ctx.event_source),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveShanxie { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveShanxie {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveTuntian { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveTuntian {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveZaoxian {
            player,
            jixi_def,
            tuntian_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveZaoxian {
            player,
            jixi_def,
            tuntian_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTianxiang {
            player,
            draw_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTianxiang {
            player,
            target: PlayerId::NONE,
            discard_card: engine_api::CardId::NONE,
            source: engine_api::PlayerId::new(host.ctx.event_source),
            amount: host.ctx.event_amount,
            nature: host.event_nature,
            card: engine_api::CardId::new(host.ctx.event_card),
            card_kind: host.event_card_kind,
            card_flags: host.event_card_flags,
            draw_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveRetianxiang { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveRetianxiang {
                player,
                target: PlayerId::NONE,
                discard_card: engine_api::CardId::NONE,
                branch: 0,
                source: engine_api::PlayerId::new(host.ctx.event_source),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseOtherAddMark {
            player,
            mark,
            delta,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseOtherAddMark {
            player,
            mark,
            delta,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveStddechong { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveStddechong {
                player,
                target: engine_api::PlayerId::new(host.ctx.event_player),
                cards: Vec::new(),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveStdjujian { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveStdjujian {
                player,
                target: PlayerId::NONE,
                cards: (host.ctx.event_card != 0)
                    .then(|| vec![engine_api::CardId::new(host.ctx.event_card)])
                    .unwrap_or_default(),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveDctingxian { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveDctingxian {
                player,
                card: engine_api::CardId::new(host.ctx.event_card),
                targets: host
                    .event_targets
                    .iter()
                    .copied()
                    .map(engine_api::PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveXinfuQinguo { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveXinfuQinguo {
                player,
                card: engine_api::CardId::new(host.ctx.event_card),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveXinxingluan { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveXinxingluan {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveDckuizhen { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveDckuizhen {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveDckuizhenResolve { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveDckuizhenResolve {
                player,
                winner: PlayerId::new(host.ctx.event_player),
                loser: PlayerId::new(host.ctx.event_source),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveNsdingzhou { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveNsdingzhou {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveSxrmlanjiao { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveSxrmlanjiao {
                player,
                target: PlayerId::NONE,
                stage: 0,
                view: Vec::new(),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveRekuanshi { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveRekuanshi {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveRekuanshiResolve { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveRekuanshiResolve {
                player,
                damaged: PlayerId::new(host.ctx.event_player),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveDcmoyu { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveDcmoyu {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveXizhen { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveXizhen {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveXuxie { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveXuxie {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveXuxieAdd { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveXuxieAdd {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveDcchiying {
            player,
            target,
            stage,
            accum,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcchiying {
            player,
            target,
            stage,
            accum,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDcmanzhi { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveDcmanzhi {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveXinkuangfu { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveXinkuangfu {
                player,
                target: PlayerId::NONE,
                sha_target: PlayerId::NONE,
                stage: 1,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveDcxiangshu { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveDcxiangshu {
                player,
                target: PlayerId::new(host.ctx.event_player),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveDcxiangshuLottery { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveDcxiangshuLottery {
                player,
                target: PlayerId::new(host.ctx.event_player),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveGongjian { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveGongjian {
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

        EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss {
            player,
            reveal_count,
            recipient_cap,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss {
                player,
                reveal_count,
                recipient_cap,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp { player, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveRefuhai { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveRefuhai {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseGainTargetHandCeilHalfDifference {
            player,
            target,
            card,
            ready_to_choose,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseGainTargetHandCeilHalfDifference {
            player,
            target,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_choose,
        },

        EffectRequest::ResolveGzjinfa {
            player,
            target,
            cost_card,
            given_card,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveGzjinfa {
            player,
            target,
            cost_card,
            given_card,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWnQifeng { player, target, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveWnQifeng {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
            player,
            target,
            card,
            targets,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
            player,
            target,
            card,
            targets,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDcyingtu { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveDcyingtu {
                player,
                target: PlayerId::NONE,
                stage: 0,
                cards: host
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
            player,
            target,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
                player,
                target,
                damage_id: host.state.resolution.current_damage_id,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
            player,
            source,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
                player,
                source,
                card: engine_api::CardId::NONE,
                stage: 0,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveMobiledanshou { player, target, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveMobiledanshou {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveJsrgchiying { player, target, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveJsrgchiying {
                player,
                target,
                started: false,
                remaining: Vec::new(),
                accum: Vec::new(),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveMbqingdao {
            player,
            source,
            card,
            damaged,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbqingdao {
            player,
            source,
            card,
            damaged,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMbxiugeng {
            player,
            helper_def,
            hand_limit_def,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbxiugeng {
            player,
            helper_def,
            hand_limit_def,
            stage,
            targets: Vec::new(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMbchenshe {
            player,
            dying,
            source,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbchenshe {
            player,
            dying,
            source,
            stage: 0,
            roles: Vec::new(),
            suits: Vec::new(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMbquanchong {
            player, helper_def, ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbquanchong {
            player,
            helper_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMbrenxing { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveMbrenxing {
                player,
                stage: 0,
                target: PlayerId::NONE,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveMbchizhang { player, card, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveMbchizhang {
                player,
                card,
                stage: 0,
                cards: Vec::new(),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveMbduanyang {
            player,
            target,
            card,
            mode,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbduanyang {
            player,
            target,
            card,
            mode,
            stage: 0,
            cards: host
                .event_cards
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMbrunwei {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbrunwei {
            player,
            target,
            stage,
            value: 0,
            cards: host
                .event_cards
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMbshuanghuai { player, target, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveMbshuanghuai {
                player,
                target,
                stage: 0,
                choice: 0,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveMbxiongtu {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbxiongtu {
            player,
            target,
            stage,
            card: engine_api::CardId::NONE,
            cards: Vec::new(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMbganggeng {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbganggeng {
            player,
            target,
            stage,
            cards: host
                .selected_cards
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMbsijian {
            player, helper_def, ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbsijian {
            player,
            target: engine_api::PlayerId::NONE,
            stage: 0,
            choice: 0,
            helper_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMbjianji { player, .. } if host.ctx.skill_id != 0 => {
            let source = host
                .selected_targets
                .first()
                .copied()
                .map(engine_api::PlayerId::new)
                .unwrap_or(engine_api::PlayerId::NONE);
            let target = host
                .selected_targets
                .get(1)
                .copied()
                .map(engine_api::PlayerId::new)
                .unwrap_or(engine_api::PlayerId::NONE);
            EffectRequest::ResolveMbjianji {
                player,
                source,
                target,
                stage: 0,
                card: host
                    .selected_cards
                    .first()
                    .copied()
                    .map(engine_api::CardId::new)
                    .unwrap_or(engine_api::CardId::NONE),
                cards: Vec::new(),
                targets: Vec::new(),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveMbyuanmo { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveMbyuanmo {
                player,
                target: engine_api::PlayerId::NONE,
                stage: 0,
                card: engine_api::CardId::NONE,
                value: 0,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveMbfeijing {
            player,
            target,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbfeijing {
            player,
            target,
            card,
            stage: 0,
            targets: Vec::new(),
            cards: Vec::new(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMbxiaoge {
            player,
            target,
            card,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbxiaoge {
            player,
            target,
            card,
            stage,
            targets: host
                .event_targets
                .iter()
                .copied()
                .map(engine_api::PlayerId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMbxuye { player, target, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveMbxuye {
                player,
                target,
                stage: 0,
                was_max: false,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveMbkuangxiang {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbkuangxiang {
            player,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },
        req => req,
    })
}
