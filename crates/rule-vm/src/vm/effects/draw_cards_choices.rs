//! VM 效果请求解码链：选择类效果（CHOOSE_*）。

use super::*;

pub(in super::super) fn decode_request(
    host: &mut dyn VmHost,
    frame: &mut Frame,
    tag: u16,
) -> Result<EffectRequest, VmError> {
    Ok(match tag {
            effect_tag::CHOOSE_DISCARD_REDIRECT_CURRENT_SHA => {
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardRedirectCurrentSha {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: rule_api::CardId::new(card),
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_ONE_FROM_EACH_LINKED => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardOneFromEachLinked {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::CHOOSE_TARGET_LOSE_HP_DISCARD_HAND_SHA_IF_BASIC => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetLoseHpDiscardHandShaIfBasic {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                    stage: 0,
                    discarded_basic: false,
                }
            }
            effect_tag::CHOOSE_USE_VIRTUAL_SHA_OR_DRAW => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseUseVirtualShaOrDraw {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_TARGET_REVEAL_HAND_DAMAGE_IF_SHA => {
                let amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetRevealHandDamageIfSha {
                    player: PlayerId::new(player),
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_GIVE_OR_RECAST_ONE_HAND => {
            let grant_mark_def = pop(frame)?.0 as u32;
            let grant_skill_def = pop(frame)?.0 as u32;
            let require_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseGiveOrRecastOneHand {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
                require_skill_def,
                grant_skill_def,
                grant_mark_def,
            }
        }
        effect_tag::CHOOSE_OTHER_GIVE_ONE_HAND_EQUIP => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherGiveOneHandEquip {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_REVEAL_HAND_CARD_BIND_CATEGORY_TARGET => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseRevealHandCardBindCategoryTarget {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_MOVE_TARGET_EQUIPMENT_AND_OR_DRAW => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseMoveTargetEquipmentAndOrDraw {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_TARGET_DISCARD_ATTACK_RANGE_CARDS_OR_INCREASE_DAMAGE_WEAPON_FORCES_BOTH => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    damage_id: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DRAW_OR_RECAST_ADD_SKILL => {
                let recast_skill_def = pop(frame)?.0 as u32;
                let draw_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDrawOrRecastAddSkill {
                    player: PlayerId::new(player),
                    draw_skill_def: engine_api::SkillDefId::new(draw_skill_def),
                    recast_skill_def: engine_api::SkillDefId::new(recast_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_GAIN_SHA_FOR_OPENING_EXTRA_PLAY => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGainShaForOpeningExtraPlay {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DRAW_OR_DISCARD_TARGET_CARD_RECORD_SHA => {
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OPTIONAL_DISCARD_TARGET_CARD => {
                let scope = pop(frame)?.0;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let scope = u8::try_from(scope)
                    .ok()
                    .and_then(rule_api::TargetCardScope::from_tag)
                    .ok_or_else(|| VmError::Host("invalid target-card scope".into()))?;
                EffectRequest::ChooseOptionalDiscardTargetCard {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    scope,
                }
            }
            effect_tag::CHOOSE_DISCARD_TARGET_CARDS_RECOVER_SAME_CATEGORY => {
                let amount = pop(frame)?.0.max(0) as u32;
                let count = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    count,
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_SCSKUIJI_DISCARD => {
                let count = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseScskuijiDiscard {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    count,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_PSKKUIJI_DISCARD => {
                let count = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChoosePskuijiDiscard {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    count,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_COMPARE_SELECTED_HAND_CARD_SECOND_FAILURE_SEAT_DAMAGE => {
                let damage = pop(frame)?.0.max(0) as u32;
                let lose_hp = pop(frame)?.0.max(0) as u32;
                let failure_count = pop(frame)?.0.max(0) as u32;
                let draw_count = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let card = host.selected_cards().first().copied().unwrap_or(0);
                EffectRequest::ChooseCompareSelectedHandCardSecondFailureSeatDamage {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    draw_count,
                    failure_count,
                    lose_hp,
                    damage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_TARGET_DISCARD_DISTANCE_HAND_OR_INCREASE_DAMAGE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetDiscardDistanceHandOrIncreaseDamage {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    damage_id: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DRAW_VIEW_TWO_GAIN_OR_MOVE_EQUIPMENT => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDrawViewTwoGainOrMoveEquipment {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_BASIC_THEN_TARGET_DISCARD_EQUIP_OR_DAMAGE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardBasicThenTargetDiscardEquipOrDamage {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_CURRENT_SHA_TARGET_DISCARD_REDIRECT_TO_OWNER => {
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let owner = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseCurrentShaTargetDiscardRedirectToOwner {
                    owner: PlayerId::new(owner),
                    source: PlayerId::new(source),
                    card: rule_api::CardId::new(card),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                    ready_to_redirect: false,
                }
            }
            effect_tag::CHOOSE_TARGET_GAIN_BOTTOM_CARD => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetGainBottomCard {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::NONE,
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_CURRENT_TRICK_TARGETS_TO_NULLIFY => {
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseCurrentTrickTargetsToNullify {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: rule_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_TARGET_CARD => {
                let scope = pop(frame)?.0;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let scope = u8::try_from(scope)
                    .ok()
                    .and_then(rule_api::TargetCardScope::from_tag)
                    .ok_or_else(|| VmError::Host("invalid target-card scope".into()))?;
                EffectRequest::ChooseDiscardTargetCard {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    scope,
                }
            }
            effect_tag::CHOOSE_GAIN_TOP_CARDS_BY_SUIT => {
                let suit = match pop(frame)?.0 {
                    1 => engine_api::Suit::Spade,
                    2 => engine_api::Suit::Heart,
                    3 => engine_api::Suit::Club,
                    4 => engine_api::Suit::Diamond,
                    _ => return Err(VmError::Host("invalid top-card selection suit".into())),
                };
                let count = u8::try_from(pop(frame)?.0)
                    .ok()
                    .filter(|count| *count > 0)
                    .ok_or_else(|| VmError::Host("invalid top-card view count".into()))?;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGainTopCardsBySuit {
                    player: PlayerId::new(player),
                    count,
                    suit,
                }
            }
            effect_tag::CHOOSE_OTHER_DRAW_BY_HAND_COUNT => {
                let times = pop(frame)?.0.max(0) as u32;
                let empty_count = pop(frame)?.0.max(0) as u32;
                let normal_count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherDrawByHandCount {
                    player: PlayerId::new(player),
                    normal_count,
                    empty_count,
                    times,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_ONE_THEN_GAIN_TARGET_EQUIPMENT => {
                let slot = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardOneThenGainTargetEquipment {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    slot,
                }
            }
            effect_tag::CHOOSE_TARGET_DRAW => {
                let scope = u8::try_from(pop(frame)?.0)
                    .ok()
                    .and_then(DrawTargetScope::from_tag)
                    .ok_or_else(|| VmError::Host("invalid draw target scope".into()))?;
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetDraw {
                    player: PlayerId::new(player),
                    count,
                    scope,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_ONE_FOR_SHA_REFUND_AND_DRAW_IF_RED => {
                let card = pop(frame)?.0.max(0) as u32;
                let sha_user = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed {
                    player: PlayerId::new(player),
                    sha_user: PlayerId::new(sha_user),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_ADD_CURRENT_SHA_TARGET => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseAddCurrentShaTarget {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_ADD_CURRENT_SHA_TARGET_WITHIN_DISTANCE => {
                let max_distance = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    max_distance,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_STORE_HAND_SHA_IN_SKILL_PILE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseStoreHandShaInSkillPile {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_SPEND_SKILL_PILE_FOR_DYING_JIU => {
                let dying = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseSpendSkillPileForDyingJiu {
                    player: PlayerId::new(player),
                    dying: PlayerId::new(dying),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_COMPARE_SELECTED_HAND_CARD => {
                let lose_skill_def = pop(frame)?.0 as u32;
                let win_skill_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let card = host.selected_cards().first().copied().unwrap_or(0);
                EffectRequest::ChooseCompareSelectedHandCard {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::new(card),
                    win_skill_def: engine_api::SkillDefId::new(win_skill_def),
                    lose_skill_def: engine_api::SkillDefId::new(lose_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_TARGET_DRAW_TO_MAX_HP_CAP => {
                let cap = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseTargetDrawToMaxHpCap {
                    player: PlayerId::new(player),
                    cap,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OTHER_DRAW_OWNER_LOST_HP_TURN_OVER => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_ADD_OR_REMOVE_CURRENT_CARD_TARGET => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseAddOrRemoveCurrentCardTarget {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_HAND_COMPARE => {
                let lose_skill_def = pop(frame)?.0 as u32;
                let win_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseHandCompare {
                    player: PlayerId::new(player),
                    win_skill_def: engine_api::SkillDefId::new(win_skill_def),
                    lose_skill_def: engine_api::SkillDefId::new(lose_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_HAND_COMPARE_GRANT_TARGET_ON_WIN => {
                let lose_skill_def = pop(frame)?.0 as u32;
                let win_grant_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseHandCompareGrantTargetOnWin {
                    player: PlayerId::new(player),
                    win_grant_def: engine_api::SkillDefId::new(win_grant_def),
                    lose_skill_def: engine_api::SkillDefId::new(lose_skill_def),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_HAND_COMPARE_BETWEEN => {
                let first = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseHandCompareBetween {
                    player: PlayerId::new(player),
                    first: PlayerId::new(first),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OTHER_GIVE_THEN_RECOVER => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherGiveThenRecover {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OTHER_DAMAGE => {
                let amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseOtherDamage {
                    player: PlayerId::new(player),
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_OTHER_DISCARD_TARGET_CARD => {
                let scope = pop(frame)?.0;
                let player = pop(frame)?.0.max(0) as u32;
                let scope = u8::try_from(scope)
                    .ok()
                    .and_then(rule_api::TargetCardScope::from_tag)
                    .ok_or_else(|| VmError::Host("invalid target-card scope".into()))?;
                EffectRequest::ChooseOtherDiscardTargetCard {
                    player: PlayerId::new(player),
                    scope,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_GAIN_DISCARD_CARD_BY_KIND => {
                let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGainDiscardCardByKind {
                    player: PlayerId::new(player),
                    kind,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_ADD_MISSING_SKILL => {
                let third = pop(frame)?.0 as u32;
                let second = pop(frame)?.0 as u32;
                let first = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseAddMissingSkill {
                    player: PlayerId::new(player),
                    skill_defs: [
                        engine_api::SkillDefId::new(first),
                        engine_api::SkillDefId::new(second),
                        engine_api::SkillDefId::new(third),
                    ],
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_GIVE_OR_DISCARD_HAND => {
                let scope = u8::try_from(pop(frame)?.0)
                    .ok()
                    .and_then(DrawTargetScope::from_tag)
                    .ok_or_else(|| VmError::Host("invalid give recipient scope".into()))?;
                let payment_count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseGiveOrDiscardHand {
                    player: PlayerId::new(player),
                    payment_count,
                    scope,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CHOOSE_DISCARD_HAND_EQUIP_THEN_RECOVER_TARGET => {
                let amount = pop(frame)?.0.max(0) as u32;
                let count = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let chooser = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChooseDiscardHandEquipThenRecoverTarget {
                    chooser: PlayerId::new(chooser),
                    target: PlayerId::new(target),
                    count,
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            _ => return super::resolve_huzun::decode_request(host, frame, tag),
    })
}
