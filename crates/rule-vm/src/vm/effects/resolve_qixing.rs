use super::*;

pub(in super::super) fn decode_request(
    host: &mut dyn VmHost,
    frame: &mut Frame,
    tag: u16,
) -> Result<EffectRequest, VmError> {
    Ok(match tag {
        effect_tag::RESOLVE_QIXING => {
            let initialize = pop(frame)?.0 != 0;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveQixing {
                player: PlayerId::new(player),
                initialize,
                stage: 0,
                hand_cards: Vec::new(),
                pile_cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_QIXING_WEATHER => {
            let max_targets = pop(frame)?.0.max(0) as u8;
            let helper_skill_def = pop(frame)?.0 as u32;
            let qixing_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveQixingWeather {
                player: PlayerId::new(player),
                qixing_skill_def: engine_api::SkillDefId::new(qixing_skill_def),
                helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                max_targets,
                stage: 0,
                cards: Vec::new(),
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_NZRY_LONGNU => {
            let thunder_skill_def = pop(frame)?.0 as u32;
            let fire_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveNzryLongnu {
                player: PlayerId::new(player),
                fire_skill_def: engine_api::SkillDefId::new(fire_skill_def),
                thunder_skill_def: engine_api::SkillDefId::new(thunder_skill_def),
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_LINK_OTHER => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseLinkOther {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DRLT_POXI => {
            let hand_limit_skill_def = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDrltPoxi {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                hand_limit_skill_def: engine_api::SkillDefId::new(hand_limit_skill_def),
                stage: 0,
                own_count: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DRLT_JIEYING => {
            let mode = pop(frame)?.0.max(0) as u8;
            let mark = pop(frame)?.0 as u32;
            let event_player = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDrltJieying {
                player: PlayerId::new(player),
                event_player: PlayerId::new(event_player),
                mark: engine_api::MarkDefId::new(mark),
                mode,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DRLT_DUORUI => {
            let stage = pop(frame)?.0.max(0) as u8;
            let helper_skill_def = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDrltDuorui {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                stage,
                selected_skill_def: engine_api::SkillDefId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DRLT_ZHITI => {
            let mode = pop(frame)?.0.max(0) as u8;
            let event_amount = pop(frame)?.0.max(0) as u32;
            let event_source = pop(frame)?.0.max(0) as u32;
            let event_player = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDrltZhiti {
                player: PlayerId::new(player),
                event_player: PlayerId::new(event_player),
                event_source: PlayerId::new(event_source),
                event_amount,
                mode,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SHEN_SIMAYI => {
            let mode = pop(frame)?.0.max(0) as u8;
            let mark = pop(frame)?.0 as u32;
            let event_amount = pop(frame)?.0.max(0) as u32;
            let event_player = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveShenSimayi {
                player: PlayerId::new(player),
                event_player: PlayerId::new(event_player),
                event_amount,
                mark: engine_api::MarkDefId::new(mark),
                mode,
                cards: host
                    .selected_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(engine_api::PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_BEIGE => {
            let source = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveBeige {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                source: PlayerId::new(source),
                stage: 0,
                result_card: engine_api::CardId::NONE,
                suit: engine_api::Suit::None,
                rank: 0,
                rewriters: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_ADD_SKILL_BOUND_CARD_TARGET => {
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseAddSkillBoundCardTarget {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_DISCARD_MATCHING_EQUIP_COLOR_ADD_SKILL => {
            let skill_def = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardMatchingEquipColorAddSkill {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill_def: engine_api::SkillDefId::new(skill_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::USE_VIRTUAL_SHA_FROM_CARD_COLOR_LOCK_SOURCE_IF_NO_SHA => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_CURRENT_CARD_USE_ENHANCEMENTS => {
            let helper_skill_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseCurrentCardUseEnhancements {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::MAKE_CURRENT_CARD_UNRESPONDABLE => {
            let helper_skill_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::AddCardUsePolicySkill {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                policy_mask: 1 << 1,
                skill_def: engine_api::SkillDefId::new(helper_skill_def),
            }
        }
        effect_tag::CHOOSE_DRAW_OR_DISCARD_ADD_SKILL => {
            let discard_skill_def = pop(frame)?.0 as u32;
            let draw_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDrawOrDiscardAddSkill {
                player: PlayerId::new(player),
                draw_skill_def: engine_api::SkillDefId::new(draw_skill_def),
                discard_skill_def: engine_api::SkillDefId::new(discard_skill_def),
                skill: engine_api::SkillId::NONE,
                three_way: false,
            }
        }
        effect_tag::CHOOSE_THREE_WAY_DRAW_OR_DISCARD_ADD_SKILL => {
            let discard_skill_def = pop(frame)?.0 as u32;
            let draw_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDrawOrDiscardAddSkill {
                player: PlayerId::new(player),
                draw_skill_def: engine_api::SkillDefId::new(draw_skill_def),
                discard_skill_def: engine_api::SkillDefId::new(discard_skill_def),
                skill: engine_api::SkillId::NONE,
                three_way: true,
            }
        }
        effect_tag::CHOOSE_DISCARDED_BLACK_BASIC_AS_BINGLIANG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardedBlackBasicAsBingliang {
                player: PlayerId::new(player),
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_DISCARD_JUDGE_USE_VIRTUAL_SHA_DRAW_IF_NO_DAMAGE => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::DISCARD_SELECTED_RECORD_UNIQUE_CATEGORY_TARGET => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            let card = host.selected_cards().first().copied().unwrap_or(0);
            EffectRequest::DiscardSelectedRecordUniqueCategoryTarget {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_TARGET_DRAW_OR_DISCARD_BY_SKILL_USE_COUNT_LINK_IF_WOUNDED => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                toggle_link: false,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_TARGET_DRAW_OR_DISCARD_BY_SKILL_USE_COUNT_TOGGLE_LINK_IF_WOUNDED => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                toggle_link: true,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_REVEAL_OTHER_HAND_CARD_REMEMBER_CATEGORY => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseRevealOtherHandCardRememberCategory {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::DRAW_THEN_GIVE_EXACT_RECORD_PHASE_TARGET => {
            let count = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DrawThenGiveExactRecordPhaseTarget {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                count,
                skill: engine_api::SkillId::NONE,
                ready_to_give: false,
            }
        }
        effect_tag::CLEAR_PHASE_GIFT_TARGET => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ClearPhaseGiftTarget {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_GAIN_RESOLVED_SHA_OR_SHAN_THEN_OPTIONAL_USE_SHA => {
            let sha_card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseGainResolvedShaOrShanThenOptionalUseSha {
                player: PlayerId::new(player),
                sha_card: engine_api::CardId::new(sha_card),
                shan_cards: Vec::new(),
                excluded_recipients: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_GAIN_ALL_RESOLVED_SHA_SHAN_THEN_COLOR_REWARDS => {
            let sha_card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseGainAllResolvedShaShanThenColorRewards {
                player: PlayerId::new(player),
                sha_card: engine_api::CardId::new(sha_card),
                shan_cards: Vec::new(),
                excluded_recipients: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::SKIP_CURRENT_PHASE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::SkipCurrentPhase {
                player: PlayerId::new(player),
            }
        }
        effect_tag::FINISH_CURRENT_PHASE_ABORT_RESOLUTION => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::FinishCurrentPhaseAbortResolution {
                player: PlayerId::new(player),
                stage: 0,
            }
        }
        effect_tag::GAIN_TOP_CARD_TO_SKILL_PILE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GainTopCardToSkillPile {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_DISCARD_SKILL_PILE_CARD_ADD_MARKED_SKILL => {
            let helper_skill_def = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                selected: engine_api::CardId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::GAIN_ONE_HAND_FROM_SELECTED_TARGETS => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GainOneHandFromSelectedTargets {
                player: PlayerId::new(player),
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                index: 0,
                selected: engine_api::CardId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_REPOLU_TARGETS_DRAW => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseRepoluTargetsDraw {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_XINFU_GUOLUN => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveXinfuGuolun {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                player_card: engine_api::CardId::NONE,
                target_card: engine_api::CardId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_DISCARD_ONE_FOR_SHA_REFUND_AND_DRAW_IF_RED_RESTORE_IF_EQUAL_RANK => {
            let restore_skill_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let sha_user = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
                player: PlayerId::new(player),
                sha_user: PlayerId::new(sha_user),
                card: engine_api::CardId::new(card),
                restore_skill_def: rule_api::SkillDefId::new(restore_skill_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_BEIGE_CONFIGURED => {
            let diamond_draw_count = pop(frame)?.0.max(0) as u32;
            let heart_recover_amount = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveBeigeConfigured {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                source: PlayerId::new(source),
                stage: 0,
                result_card: engine_api::CardId::NONE,
                suit: engine_api::Suit::None,
                rank: 0,
                rewriters: Vec::new(),
                heart_recover_amount,
                diamond_draw_count,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZHUIYI => {
            let killer = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveZhuiyi {
                player: PlayerId::new(player),
                killer: PlayerId::new(killer),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DECADEZHENJUN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDecadezhenjun {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                stage: 0,
                count: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SP_XIAOGUO => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpXiaoguo {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_REPLACE_DRAW => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
                player: PlayerId::new(player),
                initialized: false,
                remaining_targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::GIVE_SELECTED_THEN_RECOVER_TARGET => {
            let amount = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GiveSelectedThenRecoverTarget {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                cards: host
                    .selected_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                amount,
                skill: engine_api::SkillId::NONE,
                ready_to_recover: false,
            }
        }
        effect_tag::RESOLVE_DINGPAN => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDingpan {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TANLU => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTanlu {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                amount: 0,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_OTHER_TARGETS_BY_SKIPPED_PHASE_COUNT_DAMAGE => {
            let amount = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage {
                player: PlayerId::new(player),
                amount,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_OTHER_TARGETS_BY_SKIPPED_PHASE_COUNT_DAMAGE_DRAW => {
            let amount = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw {
                player: PlayerId::new(player),
                amount,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::REVEAL_UNIFORM_HAND_DRAW_SELF_CHOOSE_OTHERS => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::RevealUniformHandDrawSelfChooseOthers {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_CARD_USER_OR_ADD_TARGET => {
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseCardUserOrAddTarget {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_DISCARD_EQUIPMENT_REDUCE_CURRENT_DAMAGE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardEquipmentReduceCurrentDamage {
                player: PlayerId::new(player),
                damage_id: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_DISCARD_EQUIPMENT_PREVENT_CURRENT_DAMAGE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage {
                player: PlayerId::new(player),
                damage_id: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_MOVE_TARGET_EQUIPMENT_OR_DRAW => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseMoveTargetEquipmentOrDraw {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::GIVE_ALL_EQUIPMENT_CHOOSE_RECOVER_OR_DAMAGE => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GiveAllEquipmentChooseRecoverOrDamage {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_TARGET_DISCARD_WEAPON_RANGE_HAND_OR_INCREASE_DAMAGE => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                damage_id: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_HAND_COUNT_DIFFERENT_TARGET_DRAW_OR_RANDOM_DISCARD => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseHandCountDifferentTargetDrawOrRandomDiscard {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::INCREASE_CURRENT_DRAW_COUNT => {
            let count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::IncreaseCurrentDrawCount {
                player: PlayerId::new(player),
                count,
            }
        }
        effect_tag::CHOOSE_TARGET_DISCARD_EXCESS_THEN_SELF_DISCARD_OR_TARGET_DRAW => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::DRAW_THEN_CHOOSE_GAIN_ONE_FROM_DAMAGE_TARGET_AND_NEIGHBORS => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
                ready_to_choose: false,
            }
        }
        effect_tag::CHOOSE_PREVENT_DAMAGE_OR_LOSE_MAX_HP_INCREASE_DAMAGE => {
            let amount = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChoosePreventDamageOrLoseMaxHpIncreaseDamage {
                player: PlayerId::new(player),
                damage_id: 0,
                amount,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::DISCARD_ONE_HAND_OR_EQUIP => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardOneHandOrEquip {
                player: PlayerId::new(player),
            }
        }
        effect_tag::DISCARD_ALL_HAND_AND_EQUIP => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DiscardAllHandAndEquip {
                player: PlayerId::new(player),
            }
        }
        effect_tag::REMOVE_ALL_SKILLS => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::RemoveAllSkills {
                player: PlayerId::new(player),
            }
        }
        effect_tag::CHOOSE_OTHER_RECOVER => {
            let amount = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOtherRecover {
                player: PlayerId::new(player),
                amount,
            }
        }
        effect_tag::ENABLE_EQUIP_SLOT => {
            let source = pop(frame)?.0.max(0) as u32;
            let slot = pop(frame)?.0.max(0) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::EnableEquipSlot {
                player: PlayerId::new(player),
                slot,
                source: PlayerId::new(source),
            }
        }
        effect_tag::JUDGE_GAIN_EQUIPMENT_BY_COLOR => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::JudgeGainEquipmentByColor {
                player: PlayerId::new(player),
            }
        }
        effect_tag::REVEAL_DISCARD_SUIT_GAIN_OR_USE => {
            let max_mark = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let base_count = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let mark = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::RevealDiscardSuitGainOrUse {
                player: PlayerId::new(player),
                mark: rule_api::MarkDefId::new(mark),
                base_count,
                max_mark,
                revealed: Vec::new(),
                completed: 0,
                step: 0,
                data: 0,
            }
        }
        effect_tag::CHOOSE_DISCARD_OR_FIRE_DAMAGE_SKIP_PHASES => {
            let damage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let max_discard = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardOrFireDamageSkipPhases {
                player: PlayerId::new(player),
                max_discard,
                damage,
                selected: 0,
                step: 0,
                target: PlayerId::NONE,
            }
        }
        effect_tag::DRAW_SELECTED => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DrawCards {
                player: PlayerId::new(player),
                count: host.selected_cards().len() as u32,
            }
        }
        effect_tag::DRAW_EACH_BY_SEAT => {
            let count = pop(frame)?.0.max(0) as u32;
            let second = pop(frame)?.0.max(0) as u32;
            let first = pop(frame)?.0.max(0) as u32;
            EffectRequest::DrawEachBySeat {
                first: PlayerId::new(first),
                second: PlayerId::new(second),
                count,
                skill: rule_api::SkillId::NONE,
            }
        }
        effect_tag::DRAW_THEN_GIVE_ONE => {
            let mode = pop(frame)?.0.max(0) as u32;
            let count = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DrawThenGiveOne {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                count,
                drawn_only: mode & 1 != 0,
                use_given_equipment: mode & 2 != 0,
                skill: rule_api::SkillId::NONE,
                drawn: Vec::new(),
                ready_to_give: false,
            }
        }
        effect_tag::DRAW_THEN_TAKE_ONE => {
            let count = pop(frame)?.0.max(0) as u32;
            let taker = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DrawThenTakeOne {
                player: PlayerId::new(player),
                taker: PlayerId::new(taker),
                count,
                skill: rule_api::SkillId::NONE,
                ready_to_take: false,
            }
        }
        effect_tag::TAKE_ONE_CARD => {
            let equip_only = pop(frame)?.0 != 0;
            let to = pop(frame)?.0.max(0) as u32;
            let from = pop(frame)?.0.max(0) as u32;
            EffectRequest::TakeOneCard {
                from: PlayerId::new(from),
                to: PlayerId::new(to),
                equip_only,
            }
        }
        effect_tag::TAKE_ONE_EVENT_TARGET_HAND => {
            let to = pop(frame)?.0.max(0) as u32;
            EffectRequest::TakeOneHandCard {
                from: PlayerId::NONE,
                to: PlayerId::new(to),
            }
        }
        effect_tag::CHOOSE_OTHER_HAND_STORE_SKILL_PILE => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOtherHandStoreSkillPile {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::GAIN_ALL_SKILL_PILE_CARDS => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GainAllSkillPileCards {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::GAIN_OTHER_SKILL_PILE_CARDS => {
            let skill_def = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GainOtherSkillPileCards {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill_def,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_DISCARD_ONE_HE_OR_LOSE_HP => {
            let lose_amount = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardOneHeOrLoseHp {
                player: PlayerId::new(player),
                lose_amount,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::JUDGE_SPADE_TWO_TO_NINE_DRAW_TWO_OR_THUNDER_SHA_TWICE => {
            let target = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            EffectRequest::JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice {
                source: PlayerId::new(source),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_SOURCE_CONFIRM_OWNER_LOSE_HP => {
            let amount = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let owner = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseSourceConfirmOwnerLoseHp {
                owner: PlayerId::new(owner),
                source: PlayerId::new(source),
                amount,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_END_EXTRA_DRAW_OR_PLAY_BY_USED_SUITS => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::LOSE_HP_DOUBLE_CURRENT_DAMAGE_THEN_REPLACE => {
            let replacement_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::LoseHpDoubleCurrentDamageThenReplace {
                player: PlayerId::new(player),
                damage_id: 0,
                amount: 0,
                skill: engine_api::SkillId::NONE,
                replacement_skill_def: engine_api::SkillDefId::new(replacement_skill_def),
                ready_to_replace: false,
            }
        }
        effect_tag::TURN_OVER => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::TurnOver {
                player: PlayerId::new(player),
            }
        }
        effect_tag::RESTORE_CHARACTER_STATE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::RestoreCharacterState {
                player: PlayerId::new(player),
            }
        }
        effect_tag::DISCARD_ALL_HAND_EQUIP_JUDGE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DiscardAllHandEquipJudge {
                player: PlayerId::new(player),
            }
        }
        effect_tag::DAMAGE_THEN_DISCARD_ALL_JUDGE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DamageThenDiscardAllJudge {
                player: PlayerId::new(player),
                ready_to_discard: false,
            }
        }
        effect_tag::REPLACE_DRAW_GAIN_TOP_TWO_ADD_SKILLS_IF_DIFFERENT_COLORS => {
            let second_skill_def = pop(frame)?.0 as u32;
            let first_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
                player: PlayerId::new(player),
                first_skill_def: rule_api::SkillDefId::new(first_skill_def),
                second_skill_def: rule_api::SkillDefId::new(second_skill_def),
                stage: 0,
                cards: Vec::new(),
                skill: rule_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_GAIN_TARGET_CARD_ALTERNATE_ZONE_THEN_TARGET_DRAW => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                selected: rule_api::CardId::NONE,
                skill: rule_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_DISCARD_TARGET_CARD_RESOLVE_CURRENT_SHA => {
            let target = pop(frame)?.0.max(0) as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardTargetCardResolveCurrentSha {
                source: PlayerId::new(source),
                card: rule_api::CardId::new(card),
                target: PlayerId::new(target),
            }
        }
        effect_tag::CHOOSE_DISCARD_TARGET_CARD_RESOLVE_CURRENT_SHA_NONBASIC_BONUS => {
            let bonus_skill_def = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
                source: PlayerId::new(source),
                card: rule_api::CardId::new(card),
                target: PlayerId::new(target),
                bonus_skill_def: rule_api::SkillDefId::new(bonus_skill_def),
            }
        }
        effect_tag::RESOLVE_DCQIAOMENG => {
            let helper_skill_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcqiaomeng {
                player: PlayerId::new(player),
                card: rule_api::CardId::new(card),
                targets: Vec::new(),
                target: PlayerId::NONE,
                selected: rule_api::CardId::NONE,
                helper_skill_def: rule_api::SkillDefId::new(helper_skill_def),
                stage: 0,
                skill: rule_api::SkillId::NONE,
            }
        }
        effect_tag::REMOVE_SKILL_BY_DEF => {
            let skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::RemoveSkillByDef {
                player: PlayerId::new(player),
                skill_def: rule_api::SkillDefId::new(skill_def),
            }
        }
        effect_tag::CHOOSE_DISCARD_HAND_ADD_SKILL_BY_COLOR => {
            let black_skill_def = pop(frame)?.0 as u32;
            let red_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardHandAddSkillByColor {
                player: PlayerId::new(player),
                red_skill_def: rule_api::SkillDefId::new(red_skill_def),
                black_skill_def: rule_api::SkillDefId::new(black_skill_def),
            }
        }
        effect_tag::REVEAL_TOP_GAIN_NON_BASIC_OR_SWAP_BASIC => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::RevealTopGainNonBasicOrSwapBasic {
                player: PlayerId::new(player),
            }
        }
        effect_tag::PREVENT_CURRENT_DAMAGE => EffectRequest::PreventDamage { damage_id: 0 },
        effect_tag::JUDGE_CANCEL_CURRENT_SHA_TARGET_ON_RED => {
            let target = pop(frame)?.0.max(0) as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::JudgeCancelCurrentShaTargetOnRed {
                player: PlayerId::new(player),
                card: rule_api::CardId::new(card),
                target: PlayerId::new(target),
            }
        }
        effect_tag::CHOOSE_DRAW_OR_RECOVER => {
            let allow_cancel = pop(frame)?.0 != 0;
            let recover_count = pop(frame)?.0.max(0) as u32;
            let draw_count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDrawOrRecover {
                player: PlayerId::new(player),
                draw_count,
                recover_count,
                allow_cancel,
            }
        }
        effect_tag::CHOOSE_USE_VIRTUAL_SHA => {
            let ignore_distance = pop(frame)?.0 != 0;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseUseVirtualSha {
                player: PlayerId::new(player),
                ignore_distance,
            }
        }
        effect_tag::ADJUST_HAND_TO => {
            let count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::AdjustHandTo {
                player: PlayerId::new(player),
                count,
            }
        }
        effect_tag::REMOVE_CARDS_BY_KIND => {
            let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            EffectRequest::RemoveCardsByKind { kind }
        }
        effect_tag::GAIN_CARD_BY_KIND => {
            let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GainCardByKind {
                player: PlayerId::new(player),
                kind,
            }
        }
        effect_tag::JUDGE_RECOVER_ON_SUIT => {
            let amount = pop(frame)?.0.max(0) as u32;
            let suit = match pop(frame)?.0 {
                1 => engine_api::Suit::Spade,
                2 => engine_api::Suit::Heart,
                3 => engine_api::Suit::Club,
                4 => engine_api::Suit::Diamond,
                tag => {
                    return Err(VmError::Host(format!("invalid judgment suit tag {tag}")));
                }
            };
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::JudgeRecoverOnSuit {
                source: PlayerId::new(host.context().self_id),
                player: PlayerId::new(player),
                suit,
                amount,
            }
        }
        effect_tag::JUDGE_RECOVER_OTHER_ON_SUIT => {
            let amount = pop(frame)?.0.max(0) as u32;
            let suit = match pop(frame)?.0 {
                1 => engine_api::Suit::Spade,
                2 => engine_api::Suit::Heart,
                3 => engine_api::Suit::Club,
                4 => engine_api::Suit::Diamond,
                tag => {
                    return Err(VmError::Host(format!("invalid judgment suit tag {tag}")));
                }
            };
            let recover_player = pop(frame)?.0.max(0) as u32;
            let judge_player = pop(frame)?.0.max(0) as u32;
            EffectRequest::JudgeRecoverOtherOnSuit {
                source: PlayerId::new(host.context().self_id),
                judge_player: PlayerId::new(judge_player),
                recover_player: PlayerId::new(recover_player),
                suit,
                amount,
            }
        }
        effect_tag::FORCE_OTHERS_USE_SHA_ON_NEAREST_OR_LOSE_HP => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ForceOthersUseShaOnNearestOrLoseHp {
                player: PlayerId::new(player),
                remaining: Vec::new(),
                started: false,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::USE_VIRTUAL_CARD_ON => {
            let target = pop(frame)?.0.max(0) as u32;
            let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::UseVirtualCardOn {
                player: PlayerId::new(player),
                kind,
                target: PlayerId::new(target),
            }
        }
        effect_tag::JUDGE_DAMAGE_OTHER_ON_SUIT => {
            let nature = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let amount = pop(frame)?.0.max(0) as u32;
            let suit = match pop(frame)?.0 {
                1 => engine_api::Suit::Spade,
                2 => engine_api::Suit::Heart,
                3 => engine_api::Suit::Club,
                4 => engine_api::Suit::Diamond,
                tag => {
                    return Err(VmError::Host(format!("invalid judgment suit tag {tag}")));
                }
            };
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::JudgeDamageOtherOnSuit {
                player: PlayerId::new(player),
                suit,
                amount,
                nature,
            }
        }
        effect_tag::KILL_PLAYER => {
            let killer = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::KillPlayer {
                player: PlayerId::new(player),
                killer: PlayerId::new(killer),
            }
        }
        effect_tag::USE_VIRTUAL_CARD_ON_SELECTED_TARGETS => {
            let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::UseVirtualCardOnSelectedTargets {
                player: PlayerId::new(player),
                kind,
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
            }
        }
        effect_tag::DEAL_DAMAGE_BAN_ACTIVE_IF_DYING => {
            let skill = pop(frame)?.0.max(0) as u32;
            let amount = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            EffectRequest::DealDamageBanActiveIfDying {
                source: PlayerId::new(source),
                target: PlayerId::new(target),
                amount,
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::CHOOSE_TARGET_RECLAIM_EQUIPMENT_DRAW_IF_RANGE_REDUCED => {
            let skill = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseTargetReclaimEquipmentDrawIfRangeReduced {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::CHOOSE_USE_VIRTUAL_CARD => {
            let skill = pop(frame)?.0.max(0) as u32;
            let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseUseVirtualCard {
                player: PlayerId::new(player),
                kind,
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::CHOOSE_USE_VIRTUAL_NANMAN => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseUseVirtualNanman {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::RESOLVE_QILUAN => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveQiluan {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::DISCARD_ALL_OWN_JUDGE => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DiscardAllOwnJudge {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::STORE_EVENT_CARD_IN_SKILL_PILE => {
            let skill = pop(frame)?.0.max(0) as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::StoreEventCardInSkillPile {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::RESOLVE_OLLINJIE => {
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOllinjie {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                skill: rule_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_TARGET_GAIN_MAX_HP => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseTargetGainMaxHp {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::GAIN_TARGET_HAND_CARD => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GainTargetHandCard {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::CHOOSE_CLAN_NOT_MAX_HAND_LIMIT_GAIN => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseClanNotMaxHandLimitGain {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::GAIN_FROM_SKILL_PILE => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GainFromSkillPile {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::DISCARD_SKILL_PILE => {
            // pile_def 是 u32 def id 字面量：经 i32 栈必须按位型还原（禁 max(0) 钳位，
            // 否则 > i32::MAX 的 def id 归零——批 193 定位的生产管线丢堆根因）。
            let pile_def = pop(frame)?.0 as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DiscardSkillPile {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                pile_def,
            }
        }
        effect_tag::PLACE_TOP_DECK_TO_EXPANSION => {
            let count = pop(frame)?.0.max(0) as u32;
            let pile_def = pop(frame)?.0 as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::PlaceTopDeckToExpansion {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                pile_def,
                count,
            }
        }
        effect_tag::CHOOSE_ONE_FROM_SKILL_PILE_DISCARD => {
            let pile_def = pop(frame)?.0 as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOneFromSkillPileDiscard {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                pile_def,
            }
        }
        effect_tag::CHOOSE_TARGET_HAND_CARD_TO_PILE => {
            let pile_def = pop(frame)?.0 as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseTargetHandCardToPile {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                pile_def,
            }
        }
        effect_tag::REARRANGE_PILE_TO_DECK_TOP => {
            let count = pop(frame)?.0.max(0) as u32;
            let pile_def = pop(frame)?.0 as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::RearrangePileToDeckTop {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                pile_def,
                count,
            }
        }
        effect_tag::RESOLVE_QIANLONG => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveQianlong {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::RESOLVE_JSRGZUOZHAN => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveJsrgzuozhan {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::RESOLVE_JSRGZUOZHAN_GAIN => {
            let event_player = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveJsrgzuozhanGain {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                event_player: PlayerId::new(event_player),
            }
        }
        effect_tag::RESOLVE_TWMOUZHI => {
            let event_card = pop(frame)?.0.max(0) as u32;
            let mode = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwmouzhi {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                mode,
                event_card,
            }
        }
        effect_tag::RESOLVE_TWDAIGUI => {
            let count = pop(frame)?.0.max(0) as u32;
            let stage = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwdaigui {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                stage,
                count,
            }
        }
        effect_tag::RESOLVE_TWMITU => {
            let count = pop(frame)?.0.max(0) as u32;
            let stage = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwmitu {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                stage,
                count,
            }
        }
        effect_tag::RESOLVE_QIANTUN => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveQiantun {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::RESOLVE_YANGMING => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveYangming {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::RESOLVE_CANYUN => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveCanyun {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::RESOLVE_HENGWU => {
            let event_card = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveHengwu {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                event_card,
            }
        }
        effect_tag::RESOLVE_BIAOZHAO3 => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveBiaozhao3 {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::RESOLVE_BIAOZHAO2 => {
            let event_player = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveBiaozhao2 {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                event_player: PlayerId::new(event_player),
            }
        }
        effect_tag::RESOLVE_DCNIJI => {
            let event_card = pop(frame)?.0.max(0) as u32;
            let mode = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcniji {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                mode,
                event_card,
            }
        }
        effect_tag::RESOLVE_DCMOUKUI => {
            let target = pop(frame)?.0.max(0) as u32;
            let stage = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcmoukui {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                stage,
                target: PlayerId::new(target),
            }
        }
        effect_tag::RESOLVE_ZHONGYAN => {
            let stage = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveZhongyan {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                stage,
            }
        }
        effect_tag::RESOLVE_CHUMEI => {
            let stage = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveChumei {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                stage,
            }
        }
        effect_tag::RESOLVE_SHUAIYAN => {
            let stage = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveShuaiyan {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                stage,
            }
        }
        effect_tag::RESOLVE_TW_XIONGJUN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwxiongjun {
                player: PlayerId::new(player),
            }
        }
        effect_tag::RESOLVE_YIXIANG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveYixiang {
                player: PlayerId::new(player),
            }
        }
        effect_tag::RESOLVE_XINFU_QIAI => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveXinfuQiai {
                player: PlayerId::new(player),
            }
        }
        effect_tag::RESOLVE_XIANGSHU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveXiangshu {
                player: PlayerId::new(player),
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
            }
        }
        effect_tag::RESOLVE_NHLUNDAO => {
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveNhlundao {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
            }
        }
        effect_tag::RESOLVE_SANSHOU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSanshou {
                player: PlayerId::new(player),
            }
        }
        _ => return super::resolve_starcuxia_weiming::decode_request(host, frame, tag),
    })
}
