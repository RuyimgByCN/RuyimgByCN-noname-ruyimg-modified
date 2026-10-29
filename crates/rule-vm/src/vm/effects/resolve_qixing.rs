//! VM 效果请求解码链：结算器与效果原语（选择与判定类见下一环）。

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
        effect_tag::USE_VIRTUAL_SHA_FROM_CARD_COLOR_LOCK_SOURCE_IF_NO_SHA => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha {
                player: PlayerId::new(player),
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
        effect_tag::REVEAL_UNIFORM_HAND_DRAW_SELF_CHOOSE_OTHERS => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::RevealUniformHandDrawSelfChooseOthers {
                player: PlayerId::new(player),
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
        effect_tag::INCREASE_CURRENT_DRAW_COUNT => {
            let count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::IncreaseCurrentDrawCount {
                player: PlayerId::new(player),
                count,
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
        effect_tag::REVEAL_TOP_GAIN_NON_BASIC_OR_SWAP_BASIC => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::RevealTopGainNonBasicOrSwapBasic {
                player: PlayerId::new(player),
            }
        }
        effect_tag::PREVENT_CURRENT_DAMAGE => EffectRequest::PreventDamage { damage_id: 0 },
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
        effect_tag::GAIN_TARGET_HAND_CARD => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GainTargetHandCard {
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
        _ => return super::resolve_qixing_choices::decode_request(host, frame, tag),
    })
}
