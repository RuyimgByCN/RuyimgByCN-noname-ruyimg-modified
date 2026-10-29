//! VM 效果请求解码链：选择（CHOOSE_*）与判定（JUDGE_*）类效果。

use super::*;

pub(in super::super) fn decode_request(
    host: &mut dyn VmHost,
    frame: &mut Frame,
    tag: u16,
) -> Result<EffectRequest, VmError> {
    Ok(match tag {
        effect_tag::CHOOSE_LINK_OTHER => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseLinkOther {
                player: PlayerId::new(player),
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
        effect_tag::CHOOSE_REPOLU_TARGETS_DRAW => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseRepoluTargetsDraw {
                player: PlayerId::new(player),
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
        effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_REPLACE_DRAW => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
                player: PlayerId::new(player),
                initialized: false,
                remaining_targets: Vec::new(),
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
        effect_tag::CHOOSE_TARGET_DISCARD_EXCESS_THEN_SELF_DISCARD_OR_TARGET_DRAW => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
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
        effect_tag::CHOOSE_OTHER_RECOVER => {
            let amount = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOtherRecover {
                player: PlayerId::new(player),
                amount,
            }
        }
        effect_tag::JUDGE_GAIN_EQUIPMENT_BY_COLOR => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::JudgeGainEquipmentByColor {
                player: PlayerId::new(player),
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
        effect_tag::CHOOSE_OTHER_HAND_STORE_SKILL_PILE => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOtherHandStoreSkillPile {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
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
        effect_tag::CHOOSE_TARGET_GAIN_MAX_HP => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseTargetGainMaxHp {
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
        _ => return super::resolve_starcuxia_weiming::decode_request(host, frame, tag),
    })
}
