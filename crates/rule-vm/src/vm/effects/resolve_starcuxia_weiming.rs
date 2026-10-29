//! VM 效果请求解码链：具名技能结算器（无包前缀者）与效果原语。

use super::*;

pub(in super::super) fn decode_request(
    host: &mut dyn VmHost,
    frame: &mut Frame,
    tag: u16,
) -> Result<EffectRequest, VmError> {
    Ok(match tag {
        effect_tag::RESOLVE_ZHONGJIE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveZhongjie {
                player: PlayerId::new(player),
            }
        }
        effect_tag::RESOLVE_DINGCE => {
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDingce {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
            }
        }
        effect_tag::RESOLVE_QIANGYONG => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveQiangyong {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::new(host.context().skill_id),
            }
        }
        effect_tag::RESOLVE_MEIYONG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMeiyong {
                player: PlayerId::new(player),
            }
        }
        effect_tag::RESOLVE_FRENDBIWEI => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveFriendbiwei {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: rule_api::CardId::new(host.selected_cards().first().copied().unwrap_or(0)),
                skill: engine_api::SkillId::new(host.context().skill_id),
            }
        }
        effect_tag::RESOLVE_CHULAO => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveChulao {
                player: PlayerId::new(player),
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
            }
        }
        effect_tag::RESOLVE_DUODUAN => {
            let card = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDuoduan {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: rule_api::CardId::new(card),
                skill: engine_api::SkillId::new(host.context().skill_id),
            }
        }
        effect_tag::RESOLVE_GAOYUAN => {
            let card = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveGaoyuan {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: rule_api::CardId::new(card),
            }
        }
        effect_tag::RESET_LINKED_SELECTED_TARGETS => {
            let source = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResetLinkedSelectedTargets {
                source: PlayerId::new(source),
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
            }
        }
        effect_tag::RESOLVE_RELIANGYIN => {
            let mode = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveReliangyin {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                mode,
            }
        }
        effect_tag::RESOLVE_ZHENGJIAN => {
            let event_player = pop(frame)?.0.max(0) as u32;
            let stage = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveZhengjian {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                stage,
                event_player: PlayerId::new(event_player),
            }
        }
        effect_tag::RESOLVE_QINMIAN => {
            let amount = pop(frame)?.0.max(0) as u32;
            let mode = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveQinmian {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                mode,
                amount,
            }
        }
        effect_tag::RESOLVE_CHUANYUN => {
            let event_card = pop(frame)?.0.max(0) as u32;
            let event_player = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveChuanyun {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                event_player: PlayerId::new(event_player),
                event_card,
            }
        }
        effect_tag::RESOLVE_DUWANG => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDuwang {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::RESOLVE_QUANJIA => {
            let target = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveQuanjia {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                source: PlayerId::new(source),
                target: PlayerId::new(target),
            }
        }
        effect_tag::RESOLVE_PXQIXIAN => {
            let kind = pop(frame)?.0.max(0) as u32;
            let stage = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePxqixian {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                stage,
                kind,
            }
        }
        effect_tag::CHOOSE_HAND_CARDS_TO_SKILL_PILE => {
            let max_count = pop(frame)?.0 as u32;
            let min_count = pop(frame)?.0 as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseHandCardsToSkillPile {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                min_count,
                max_count,
            }
        }
        effect_tag::PLACE_HAND_CARDS_TO_DECK_TOP => {
            let count = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::PlaceHandCardsToDeckTop {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                count,
            }
        }
        effect_tag::GAIN_WHOLE_SKILL_PILE => {
            let pile_def = pop(frame)?.0 as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GainWholeSkillPile {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                pile_def,
            }
        }
        effect_tag::DISCARD_BASIC_DRAW_EQUAL => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DiscardBasicDrawEqual {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::START_DEBATE_ALL_OTHERS => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::StartDebateAllOthers {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::START_DEBATE_TARGETS => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::StartDebateTargets {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::PIN_DEBATE_OPINIONS => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::PinDebateOpinions {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::ALIGN_DEBATE_OPINIONS => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::AlignDebateOpinions {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::GAIN_SHADOW_CARDS => {
            let skill = pop(frame)?.0.max(0) as u32;
            let count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GainShadowCards {
                player: PlayerId::new(player),
                count,
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::CHANGE_GROUP => {
            let skill = pop(frame)?.0.max(0) as u32;
            let group_code = pop(frame)?.0.max(0) as usize;
            let player = pop(frame)?.0.max(0) as u32;
            let group = match group_code {
                0 => "wei".to_string(),
                1 => "shu".to_string(),
                2 => "wu".to_string(),
                3 => "qun".to_string(),
                4 => "jin".to_string(),
                _ => String::new(),
            };
            EffectRequest::ChangeGroup {
                player: PlayerId::new(player),
                group,
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::DISTRIBUTE_SHADOW_CARDS => {
            let skill = pop(frame)?.0.max(0) as u32;
            let count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DistributeShadowCards {
                player: PlayerId::new(player),
                count,
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::REGISTER_TURN_END_DEBATE => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::RegisterTurnEndDebate {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::RESOLVE_TURN_END_DEBATES => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTurnEndDebates {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::CHOOSE_GROUP => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseGroup {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::CHOOSE_GAIN_OTHER_SKILL => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseGainOtherSkill {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::LOSE_GAINED_SKILL => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::LoseGainedSkill {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::RESOLVE_POTHEYUN => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotheyun {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::PLACE_XUMOU => {
            let skill = pop(frame)?.0.max(0) as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::PlaceXumou {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::PLACE_DECK_TOP_XUMOU => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::PlaceDeckTopXumou {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::CHOOSE_PLACE_HAND_XUMOU => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChoosePlaceHandXumou {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::DISCARD_ONE_OWN_XUMOU => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DiscardOneOwnXumou {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::PLACE_DECK_TOP_EXPANSION => {
            let skill = pop(frame)?.0.max(0) as u32;
            let count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::PlaceDeckTopExpansion {
                player: PlayerId::new(player),
                count,
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::CITY_CONSUME => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::CityConsume {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::QUEUE_EXTRA_TURN => {
            let skill = pop(frame)?.0.max(0) as u32;
            let check = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::QueueExtraTurn {
                player: PlayerId::new(player),
                check,
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::CHOOSE_OTHERS_ADD_MARK => {
            let skill = pop(frame)?.0.max(0) as u32;
            let max_count = pop(frame)?.0.max(0) as u32;
            let delta = pop(frame)?.0 as i32;
            let mark = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOthersAddMark {
                player: PlayerId::new(player),
                mark,
                delta,
                max_count,
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::CHOOSE_OTHERS_INSERT_EXTRA_TURN => {
            let skill = pop(frame)?.0.max(0) as u32;
            let check = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOthersInsertExtraTurn {
                player: PlayerId::new(player),
                check,
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::QUEUE_EXTRA_PHASES => {
            let skill = pop(frame)?.0.max(0) as u32;
            let mask = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::QueueExtraPhases {
                player: PlayerId::new(player),
                phases_mask: mask,
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::SKIP_REMAINING_PHASES => {
            let skill = pop(frame)?.0.max(0) as u32;
            let mask = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::SkipRemainingPhases {
                player: PlayerId::new(player),
                phases_mask: mask,
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::CHOOSE_OTHER_ADD_MARK => {
            let delta = pop(frame)?.0 as i32;
            let mark = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOtherAddMark {
                player: PlayerId::new(player),
                mark,
                delta,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::PLACE_HAND_CARD_TO_EXPANSION => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::PlaceHandCardToExpansion {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::EXTRA_RESOLVE_CURRENT_TRICK => {
            let skill = pop(frame)?.0.max(0) as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ExtraResolveCurrentTrick {
                player: PlayerId::new(player),
                card: rule_api::CardId::new(card),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::INCREASE_CURRENT_SHA_BASE_DAMAGE => {
            let amount = pop(frame)?.0.max(0) as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::IncreaseCurrentShaBaseDamage {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                amount,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::SKIP_NEXT_PLAY_PHASE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::SkipNextPlayPhase {
                player: PlayerId::new(player),
            }
        }
        effect_tag::SKIP_NEXT_JUDGE_PHASE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::SkipNextJudgePhase {
                player: PlayerId::new(player),
            }
        }
        effect_tag::CHOOSE_DISCARD_DAMAGE_ENDPOINT_CARD_INCREASE_IF_SUIT => {
            let amount = pop(frame)?.0.max(0) as u32;
            let suit = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardDamageEndpointCardIncreaseIfSuit {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                damage_id: 0,
                suit,
                amount,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_OTHER_DRAW_OR_RECOVER => {
            let times = pop(frame)?.0.max(0) as u32;
            let recover_count = pop(frame)?.0.max(0) as u32;
            let draw_count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOtherDrawOrRecover {
                player: PlayerId::new(player),
                draw_count,
                recover_count,
                times,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_DISCARD_HAND_EQUIP_THEN_REMOVE_SELF_SKILL => {
            let count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill {
                player: PlayerId::new(player),
                count,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::DRAW_SELECTED_TARGETS => {
            let count = pop(frame)?.0.max(0) as u32;
            EffectRequest::DrawSelectedTargets {
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                count,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RECOVER_IF_MIN_CURRENT_HP => {
            let amount = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::RecoverIfMinCurrentHp {
                player: PlayerId::new(player),
                amount,
            }
        }
        effect_tag::INCREASE_CURRENT_CARD_BASE_DAMAGE => {
            let amount = pop(frame)?.0.max(0) as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::IncreaseCurrentCardBaseDamage {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                amount,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::LOSE_HP_THEN_DAMAGE_OTHER_LOSE_MAX_HP_IF_DYING => {
            let skill = pop(frame)?.0.max(0) as u32;
            let amount = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                amount,
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::RECOVER_SELECTED_TARGETS => {
            let amount = pop(frame)?.0.max(0) as u32;
            EffectRequest::RecoverSelectedTargets {
                source: PlayerId::new(host.context().self_id),
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                amount,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RECORD_SELECTED_PLAY_PHASE_CATEGORY => {
            let skill = pop(frame)?.0.max(0) as u32;
            let card = host.selected_cards().first().copied().unwrap_or(0);
            EffectRequest::RecordSelectedPlayPhaseCategory {
                card: engine_api::CardId::new(card),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::TAKE_TARGET_HAND_THEN_REPEAT_UNTIL_MAX => {
            let skill = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::TakeTargetHandThenRepeatUntilMax {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::CHOOSE_DRAW_OR_INCREASE_CURRENT_CARD_DAMAGE_OR_BOTH_MARK => {
            let mark_count = pop(frame)?.0.max(0) as u32;
            // mark_def 是 u32 def id 字面量：位型保留（禁 max(0) 钳位）。
            let mark_def = pop(frame)?.0 as u32;
            let damage_amount = pop(frame)?.0.max(0) as u32;
            let draw_count = pop(frame)?.0.max(0) as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                draw_count,
                damage_amount,
                mark_def: engine_api::MarkDefId::new(mark_def),
                mark_count,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::DRAW_THEN_CHOOSE_GIVE_OR_DISCARD_HAND => {
            let payment_count = pop(frame)?.0.max(0) as u32;
            let draw_count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DrawThenChooseGiveOrDiscardHand {
                player: PlayerId::new(player),
                draw_count,
                payment_count,
                skill: engine_api::SkillId::NONE,
                ready_to_choose: false,
            }
        }
        effect_tag::GAIN_TARGET_EQUIPMENT => {
            let slot = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GainTargetEquipment {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                slot,
            }
        }
        effect_tag::DISCARD_ALL_TARGET_EQUIPMENT => {
            let target = pop(frame)?.0.max(0) as u32;
            let actor = pop(frame)?.0.max(0) as u32;
            EffectRequest::DiscardAllTargetEquipment {
                actor: PlayerId::new(actor),
                target: PlayerId::new(target),
            }
        }
        effect_tag::INCREASE_CURRENT_SHA_TARGET_DAMAGE => {
            let amount = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            EffectRequest::IncreaseCurrentShaTargetDamage {
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                target: PlayerId::new(target),
                amount,
            }
        }
        effect_tag::CHOOSE_TARGET_DRAW_THEN_OWNER_IF_BELOW_MAX_HP => {
            let count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                count,
                skill: engine_api::SkillId::NONE,
                ready_to_check: false,
            }
        }
        effect_tag::CHOOSE_BOUND_SOURCE_ATTACK_RANGE_DAMAGE => {
            let amount = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseBoundSourceAttackRangeDamage {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                amount,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_DISCARD_EXCESS => {
            let cap = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                cap,
                skill: engine_api::SkillId::NONE,
                ready_to_discard: false,
            }
        }
        effect_tag::CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_ADD_SKILL => {
            let add_def = pop(frame)?.0 as u32;
            let cap = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseTargetDrawMaxHpCapThenAddSkill {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                cap,
                skill: engine_api::SkillId::NONE,
                add_def: engine_api::SkillDefId::new(add_def),
            }
        }
        effect_tag::GIVE_SELECTED_THEN_TARGET_CHOOSE_DISCARD_SAME_SUIT_OR_LOSE_HP => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            let card = host.selected_cards().first().copied().unwrap_or(0);
            EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                suit: engine_api::Suit::None,
                skill: engine_api::SkillId::NONE,
                ready_to_choose: false,
            }
        }
        effect_tag::GAIN_EQUIPMENT_FROM_DRAW_PILE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GainEquipmentFromDrawPile {
                player: PlayerId::new(player),
            }
        }
        effect_tag::TARGET_CHOOSE_SUIT_GAIN_OWNER_HAND_DAMAGE_IF_MISMATCH => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::TargetChooseSuitGainOwnerHandDamageIfMismatch {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::JUDGE_SOURCE_DISCARD_TWO_HAND_OR_DAMAGE => {
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::JudgeSourceDiscardTwoHandOrDamage {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
            }
        }
        effect_tag::GIVE_SELECTED_RECOVER_AFTER_PHASE_TOTAL_TWO => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                cards: host
                    .selected_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::NONE,
                ready_to_recover: false,
            }
        }
        effect_tag::CHOOSE_OTHER_HAND_TARGETS_GAIN_ONE_REPLACE_DRAW => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
                player: PlayerId::new(player),
                targets: Vec::new(),
                index: 0,
                selected: engine_api::CardId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_OTHER_HAND_TARGETS_GAIN_ONE_REDUCE_DRAW => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
                player: PlayerId::new(player),
                targets: Vec::new(),
                index: 0,
                selected: engine_api::CardId::NONE,
                draw_count: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::DISTRIBUTE_TOP_TWO_CARDS => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DistributeTopTwoCards {
                player: PlayerId::new(player),
                cards: Vec::new(),
                selected: Vec::new(),
                target: PlayerId::NONE,
                skill: engine_api::SkillId::NONE,
                ready_to_choose: false,
            }
        }
        effect_tag::GIVE_ALL_HAND_RECOVER_TO_ONE => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GiveAllHandRecoverToOne {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                ready_to_recover: false,
            }
        }
        effect_tag::CHOOSE_TARGET_SELF_DISCARD_THEN_OWNER_LOSE_HP_IF_HAND_NE_HP => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                skill: engine_api::SkillId::NONE,
                ready_to_check: false,
            }
        }
        effect_tag::CHOOSE_GIVE_ONE_HAND_EQUIP_OR_RECOVER => {
            let target = pop(frame)?.0.max(0) as u32;
            let giver = pop(frame)?.0.max(0) as u32;
            let owner = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseGiveOneHandEquipOrRecover {
                owner: PlayerId::new(owner),
                giver: PlayerId::new(giver),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_GIVE_TWO_DIFFERENT_COLORS_OR_RECOVER => {
            let target = pop(frame)?.0.max(0) as u32;
            let giver = pop(frame)?.0.max(0) as u32;
            let owner = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseGiveTwoDifferentColorsOrRecover {
                owner: PlayerId::new(owner),
                giver: PlayerId::new(giver),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::TAKE_ONE_SCOPED_CARD => {
            let scope = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let to = pop(frame)?.0.max(0) as u32;
            let from = pop(frame)?.0.max(0) as u32;
            EffectRequest::TakeOneScopedCard {
                from: PlayerId::new(from),
                to: PlayerId::new(to),
                scope: rule_api::TargetCardScope::from_tag(scope)
                    .ok_or_else(|| VmError::Host("invalid target-card scope".into()))?,
            }
        }
        effect_tag::REVEAL_HAND => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::RevealHand {
                player: PlayerId::new(player),
            }
        }
        effect_tag::DRAW_LINKED_PLAYERS => {
            let count = pop(frame)?.0.max(0) as u32;
            EffectRequest::DrawLinkedPlayers { count }
        }
        effect_tag::REVEAL_TOP_GAIN_IF_COLOR => {
            let color_mask = pop(frame)?.0.max(0) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::RevealTopGainIfColor {
                player: PlayerId::new(player),
                color_mask,
            }
        }
        effect_tag::SPJUNGONG_DISCARD_OR_LOSE_THEN_SHA => {
            let skill = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::SpjungongDiscardOrLoseThenSha {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::BAN_ACTIVE_UNTIL_TURN_END => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::BanActiveUntilTurnEnd {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::SPWANWEI_ACTIVE => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::SpwanweiActive {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
            }
        }
        effect_tag::SPWANWEI_DYING => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::SpwanweiDying {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
            }
        }
        effect_tag::SKIP_JUDGE_AND_DRAW_PHASES => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::SkipJudgeAndDrawPhases {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::EQUIP_WEAPON_FROM_DRAW_PILE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::EquipWeaponFromDrawPile {
                player: PlayerId::new(player),
            }
        }
        _ => return super::resolve_starcuxia_weiming_packs::decode_request(host, frame, tag),
    })
}
