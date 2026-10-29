//! VM 效果请求解码链：通用效果原语与结算器（选择类见下一环 `draw_cards_choices`）。

use super::*;

pub(in super::super) fn decode_request(
    host: &mut dyn VmHost,
    frame: &mut Frame,
    tag: u16,
) -> Result<EffectRequest, VmError> {
    Ok(match tag {
            effect_tag::DRAW_CARDS => {
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawCards {
                    player: PlayerId::new(player),
                    count,
                }
            }
            effect_tag::MODIFY_CURRENT_DAMAGE => {
                let delta = pop(frame)?.0;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ModifyCurrentDamage {
                    player: PlayerId::new(player),
                    delta,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DEAL_DAMAGE => {
                let amount = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                EffectRequest::DealDamage {
                    source: PlayerId::new(source),
                    target: PlayerId::new(target),
                    amount,
                    is_lose_hp: false,
                    nature: 0,
                }
            }
            effect_tag::DEAL_NATURE_DAMAGE => {
                let nature = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let amount = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                EffectRequest::DealDamage {
                    source: PlayerId::new(source),
                    target: PlayerId::new(target),
                    amount,
                    is_lose_hp: false,
                    nature,
                }
            }
            effect_tag::RECOVER_HP => {
                let amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RecoverHp {
                    source: PlayerId::new(host.context().self_id),
                    player: PlayerId::new(player),
                    amount,
                }
            }
            effect_tag::RECOVER_HP_AS => {
                let amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                EffectRequest::RecoverHp {
                    source: PlayerId::new(source),
                    player: PlayerId::new(player),
                    amount,
                }
            }
            effect_tag::JUDGE_SET_SKILL_MARK_TO_RANK => {
                let skill = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeSetSkillMarkToRank {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::new(skill),
                }
            }
            effect_tag::DISABLE_SKILL_BY_DEF_UNTIL_TURN_END => {
                let skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DisableSkillByDefUntilTurnEnd {
                    player: PlayerId::new(player),
                    skill_def: rule_api::SkillDefId::new(skill_def),
                }
            }
            effect_tag::TARGET_DISCARD_THEN_DUEL_IF_NON_SHA_AND_HP_GTE_OWNER => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::EMIT_NOTE => {
                let code = pop(frame)?.0.max(0) as u32;
                EffectRequest::EmitRuleEvent {
                    code,
                    payload: vec![],
                }
            }
            effect_tag::CHANGE_MAX_HP => {
                let delta = pop(frame)?.0;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ChangeMaxHp {
                    player: PlayerId::new(player),
                    delta,
                }
            }
            effect_tag::ADD_MARK => {
                let delta = pop(frame)?.0;
                let mark = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::AddMark {
                    player: PlayerId::new(player),
                    mark: rule_api::MarkDefId::new(mark),
                    delta,
                }
            }
            effect_tag::REMOVE_MARK => {
                let amount = pop(frame)?.0.max(0) as u32;
                let mark = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RemoveMark {
                    player: PlayerId::new(player),
                    mark: rule_api::MarkDefId::new(mark),
                    amount,
                }
            }
            effect_tag::DISCARD_HAND => {
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardFromHand {
                    player: PlayerId::new(player),
                    count,
                }
            }
            effect_tag::DISCARD_HAND_EQUIP => {
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardFromHandEquip {
                    player: PlayerId::new(player),
                    count,
                }
            }
            effect_tag::TAKE_HAND => {
                let to = pop(frame)?.0.max(0) as u32;
                let from = pop(frame)?.0.max(0) as u32;
                EffectRequest::TakeFromHand {
                    from: PlayerId::new(from),
                    to: PlayerId::new(to),
                }
            }
            effect_tag::LOSE_HP => {
                let amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: PlayerId::new(player),
                    amount,
                    is_lose_hp: true,
                    nature: 0,
                }
            }
            effect_tag::ADD_SKILL => {
                let skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::AddSkill {
                    player: PlayerId::new(player),
                    skill_def: rule_api::SkillDefId::new(skill_def),
                }
            }
            effect_tag::ENTER_DYING => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::EnterDying {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::REVEAL_CARD => {
                let card = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealCards {
                    cards: vec![rule_api::CardId::new(card)],
                    to: vec![],
                }
            }
            effect_tag::USE_CARD => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::UseCard {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::new(card),
                    targets: vec![],
                }
            }
            effect_tag::REMOVE_SKILL => {
                let skill = pop(frame)?.0.max(0) as u32;
                EffectRequest::RemoveSkill {
                    skill: rule_api::SkillId::new(skill),
                }
            }
            effect_tag::MOVE_CARD => {
                let from = pop(frame)?.0.max(0) as u32;
                let to = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                EffectRequest::MoveCards {
                    cards: vec![rule_api::CardId::new(card)],
                    to: rule_api::ZoneId::new(to),
                    from: (from != 0).then(|| rule_api::ZoneId::new(from)),
                }
            }
            effect_tag::SET_TURN_STATE => {
                let phase = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let current = pop(frame)?.0.max(0) as u32;
                EffectRequest::SetTurnState {
                    current: PlayerId::new(current),
                    phase,
                }
            }
            effect_tag::USE_CARD_ON => {
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::UseCard {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::new(card),
                    targets: vec![PlayerId::new(target)],
                }
            }
            effect_tag::DISCARD_SELECTED => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardCards {
                    player: PlayerId::new(player),
                    cards: host
                        .selected_cards()
                        .iter()
                        .copied()
                        .map(rule_api::CardId::new)
                        .collect(),
                }
            }
            effect_tag::RECAST_SELECTED => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RecastCards {
                    player: PlayerId::new(player),
                    cards: host
                        .selected_cards()
                        .iter()
                        .copied()
                        .map(rule_api::CardId::new)
                        .collect(),
                }
            }
            effect_tag::DISABLE_ALL_EQUIP_SLOTS => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DisableEquipSlots {
                    player: PlayerId::new(player),
                    slots: vec![1, 2, 3, 4, 5],
                }
            }
            effect_tag::MOVE_EQUIPMENT_THEN_DRAW => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::MoveEquipmentThenDraw {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::NONE,
                }
            }
            effect_tag::DISCARD_EQUIPMENT_PAIR_WITH_ARMOR => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardEquipmentPairWithArmor {
                    player: PlayerId::new(player),
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    skill: rule_api::SkillId::NONE,
                }
            }
            effect_tag::DISCARD_TARGET_EQUIPMENT_GAIN_MOUNT => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardTargetEquipmentGainMount {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                }
            }
            effect_tag::DISCARD_TARGET_EQUIPMENT_DRAW_WEAPON_RANGE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DiscardTargetEquipmentDrawWeaponRange {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RECOVER_TO_MAX_THEN_OPTIONAL_EXCHANGE_SKILL => {
                let removable_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RecoverToMaxThenOptionalExchangeSkill {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                    removable_skill_def: engine_api::SkillDefId::new(removable_skill_def),
                    restore_skill_def: engine_api::SkillDefId::NONE,
                    ready_to_choose: false,
                }
            }
            effect_tag::REVEAL_HAND_DISCARD_KIND => {
                let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealHandDiscardKind {
                    player: PlayerId::new(player),
                    kind,
                }
            }
            effect_tag::FINISH_OPENING_EXTRA_PLAY_SHA_GAIN => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::FinishOpeningExtraPlayShaGain {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::REVEAL_TARGET_HAND_GAIN_SHAN_ARMOR_OR_DRAW_TWO => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealTargetHandGainShanArmorOrDrawTwo {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GIVE_ALL_HAND => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GiveAllHand {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                }
            }
            effect_tag::GAIN_CARD_IF_DISCARD => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainCardIfDiscard {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                }
            }
            effect_tag::REVEAL_HAND_TO_PLAYER => {
                let observer = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealHandToPlayer {
                    player: PlayerId::new(player),
                    observer: PlayerId::new(observer),
                }
            }
            effect_tag::REVEAL_HAND_CHOOSE_DISCARD_COLOR => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealHandChooseDiscardColor {
                    player: PlayerId::new(player),
                }
            }
            effect_tag::DISCARD_TARGET_CARD_GAIN_MOUNT => {
                let scope = pop(frame)?.0;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                let scope = u8::try_from(scope)
                    .ok()
                    .and_then(rule_api::TargetCardScope::from_tag)
                    .ok_or_else(|| VmError::Host("invalid target-card scope".into()))?;
                EffectRequest::DiscardTargetCardGainMount {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    scope,
                }
            }
            effect_tag::MAKE_CURRENT_SHA_TARGET_UNRESPONDABLE => {
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                EffectRequest::MakeCurrentShaTargetUnrespondable {
                    source: PlayerId::new(source),
                    card: rule_api::CardId::new(card),
                    target: PlayerId::new(target),
                }
            }
            effect_tag::JUDGE_MAKE_CURRENT_SHA_TARGET_UNRESPONDABLE_ON_RED => {
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::new(card),
                    target: PlayerId::new(target),
                    retieji: false,
                    skill: engine_api::SkillId::NONE,
                    ready_to_choose: false,
                    judgment_suit: 0,
                }
            }
            effect_tag::RESOLVE_RETIEJI => {
                let target = pop(frame)?.0.max(0) as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::new(card),
                    target: PlayerId::new(target),
                    retieji: true,
                    skill: engine_api::SkillId::NONE,
                    ready_to_choose: false,
                    judgment_suit: 0,
                }
            }
            effect_tag::RESOLVE_NEW_RELUOYI => {
                let buff_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveNewReluoyi {
                    player: PlayerId::new(player),
                    buff_skill_def: engine_api::SkillDefId::new(buff_skill_def),
                    stage: 0,
                    cards: Vec::new(),
                    accepted: false,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_REMOSHI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_skill_def = pop(frame)?.0 as u32;
                let card = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveRemoshi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: rule_api::CardId::new(card),
                    helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_XINYAOMING => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXinyaoming {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    action: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_REHUAIYI => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveRehuaiyi {
                    player: PlayerId::new(player),
                    stage: 0,
                    color: 0,
                    discarded_count: 0,
                    targets: Vec::new(),
                    index: 0,
                    gained_count: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_REANGUO => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveReanguo {
                    player: PlayerId::new(player),
                    target: host
                        .selected_targets()
                        .first()
                        .copied()
                        .map(PlayerId::new)
                        .unwrap_or(PlayerId::NONE),
                    stage: 0,
                    index: 0,
                    remaining: 0b111,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_BEIZHU => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveBeizhu {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage: 0,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_MBSHANJIA => {
                let distance_skill_def = pop(frame)?.0 as u32;
                let lost_equipment_count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveMbshanjia {
                    player: PlayerId::new(player),
                    lost_equipment_count,
                    distance_skill_def: engine_api::SkillDefId::new(distance_skill_def),
                    stage: 0,
                    reward_mask: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DAOJI => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDaoji {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: rule_api::CardId::NONE,
                    stage: 0,
                    tw: false,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_TW_DAOJI => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDaoji {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: rule_api::CardId::NONE,
                    stage: 0,
                    tw: true,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_JIAOHUA => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveJiaohua {
                    player: PlayerId::new(player),
                    category: 0,
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_LUANQUN => {
                let direct_hit_skill_def = pop(frame)?.0 as u32;
                let effect_skill_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveLuanqun {
                    player: PlayerId::new(player),
                    effect_skill_def: engine_api::SkillDefId::new(effect_skill_def),
                    direct_hit_skill_def: engine_api::SkillDefId::new(direct_hit_skill_def),
                    stage: 0,
                    participants: Vec::new(),
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::JUDGE_BLACK_GAIN_REPEAT => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeBlackGainRepeat {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                    ready_to_choose: false,
                }
            }
            effect_tag::JUDGE_BLACK_GAIN_REPEAT_PROTECT_HAND_LIMIT => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeBlackGainRepeatProtectHandLimit {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                    ready_to_choose: false,
                }
            }
            effect_tag::JUDGE_SAME_COLOR_GAIN_REPEAT => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgeSameColorGainRepeat {
                    player: PlayerId::new(player),
                    card: rule_api::CardId::NONE,
                    skill: engine_api::SkillId::NONE,
                    ready_to_choose: false,
                    color: 0,
                }
            }
            effect_tag::USE_VIRTUAL_JUEDOU_SECOND_ON_FIRST => {
                EffectRequest::UseVirtualJuedouSecondOnFirst {
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    card: rule_api::CardId::NONE,
                }
            }
            effect_tag::JUDGE_PREVENT_DAMAGE_LOSE_MAX_HP_ON_NONHEART => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::JudgePreventDamageLoseMaxHpOnNonHeart {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    damage_id: 0,
                }
            }
            effect_tag::DRAW_BY_PLAY_PHASE_CARD_CATEGORY_COUNT => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawByPlayPhaseCardCategoryCount {
                    player: PlayerId::new(player),
                    skill: rule_api::SkillId::NONE,
                }
            }
            effect_tag::QUEUE_OPENING_EXTRA_PLAY_PHASE => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::QueueOpeningExtraPlayPhase {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::CONVERT_CURRENT_SHA_TO_FIRE_LOSE_HP_IF_DAMAGE => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::REFUND_CURRENT_SHA_USE => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RefundCurrentShaUse {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DAMAGE_THEN_DRAW_SELECTED_TARGETS => {
                let draw = pop(frame)?.0.max(0) as u32;
                let damage = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                EffectRequest::DamageThenDrawSelectedTargets {
                    source: PlayerId::new(source),
                    targets: host
                        .selected_targets()
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect(),
                    damage,
                    draw,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_YOUDI => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveYoudi {
                    player: PlayerId::new(player),
                    target: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_KUANGFU => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveKuangfu {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::NONE,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_SHANJIA => {
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveShanjia {
                    player: PlayerId::new(player),
                    count,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_NEW_YAJIAO => {
                let source_card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveNewYajiao {
                    player: PlayerId::new(player),
                    source_card: engine_api::CardId::new(source_card),
                    revealed: engine_api::CardId::NONE,
                    target: PlayerId::NONE,
                    source_category: 0,
                    stage: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DINGPIN => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDingpin {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RANDOMIZE_CURRENT_HAND_COMPARE_OPPONENT => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RandomizeCurrentHandCompareOpponent {
                    player: PlayerId::new(player),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GAIN_HIGHEST_RANK_SHA_FROM_EVENT_CARDS => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainHighestRankShaFromEventCards {
                    player: PlayerId::new(player),
                    cards: host
                        .event_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GAIN_EVENT_CARDS => {
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainEventCards {
                    player: PlayerId::new(player),
                    cards: host
                        .event_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                }
            }
            effect_tag::GAIN_HAND_COMPARE_RESULT_CARD => {
                let player = pop(frame)?.0.max(0) as u32;
                let context = host.context();
                EffectRequest::GainHandCompareResultCard {
                    player: PlayerId::new(player),
                    initiator: PlayerId::new(context.event_player),
                    target: PlayerId::new(context.event_source),
                    winner: PlayerId::new(context.event_amount.saturating_sub(1)),
                    cards: host
                        .event_cards()
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GIVE_SELECTED_THEN_RECOVER_SELF => {
                let amount = pop(frame)?.0.max(0) as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GiveSelectedThenRecoverSelf {
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
                }
            }
            effect_tag::GUANXING_TOP => {
                let count = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GuanxingTop {
                    player: PlayerId::new(player),
                    count,
                    round: 0,
                    flipped: Vec::new(),
                    top: Vec::new(),
                    bottom: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GUANXING_TOP_BY_FALU_MARKS => {
                let diamond = pop(frame)?.0 as u32;
                let club = pop(frame)?.0 as u32;
                let heart = pop(frame)?.0 as u32;
                let spade = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GuanxingTopByFaluMarks {
                    player: PlayerId::new(player),
                    marks: [
                        engine_api::SkillDefId::new(spade),
                        engine_api::SkillDefId::new(heart),
                        engine_api::SkillDefId::new(club),
                        engine_api::SkillDefId::new(diamond),
                    ],
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::GAIN_DISCARD_CARD_BY_KIND => {
                let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::GainDiscardCardByKind {
                    player: PlayerId::new(player),
                    kind,
                }
            }
            effect_tag::REVEAL_UNIFORM_HAND_CHOOSE_DAMAGE => {
                let amount = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RevealUniformHandChooseDamage {
                    player: PlayerId::new(player),
                    amount,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::DRAW_FOR_LIFETIME_CARD_ACTION_DIVISORS => {
                let third = pop(frame)?.0.max(0) as u32;
                let second = pop(frame)?.0.max(0) as u32;
                let first = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::DrawForLifetimeCardActionDivisors {
                    player: PlayerId::new(player),
                    divisors: [first, second, third],
                }
            }
            effect_tag::ADD_FIRST_MISSING_SKILL => {
                let third = pop(frame)?.0 as u32;
                let second = pop(frame)?.0 as u32;
                let first = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::AddFirstMissingSkill {
                    player: PlayerId::new(player),
                    skill_defs: [
                        engine_api::SkillDefId::new(first),
                        engine_api::SkillDefId::new(second),
                        engine_api::SkillDefId::new(third),
                    ],
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCCUNSI => {
                let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let yongjue_def = pop(frame)?.0 as u32;
                let guixiu_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDccunsi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    guixiu_def: engine_api::SkillDefId::new(guixiu_def),
                    yongjue_def: engine_api::SkillDefId::new(yongjue_def),
                    stage,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RECORD_CURRENT_SHA_ALL_TARGETS => {
                let card = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::RecordCurrentShaAllTargets {
                    player: PlayerId::new(player),
                    card: engine_api::CardId::new(card),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_DCJUE => {
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveDcjue {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    stage: 0,
                    remaining: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_RUANJI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveRuanji {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    action,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_ZHENGBI => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveZhengbi {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    card: engine_api::CardId::NONE,
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_FENGYING => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_def = pop(frame)?.0 as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveFengying {
                    player: PlayerId::new(player),
                    helper_def: engine_api::SkillDefId::new(helper_def),
                    action,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_QIAOJIAN => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let helper_def = pop(frame)?.0 as u32;
                let target = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveQiaojian {
                    player: PlayerId::new(player),
                    target: PlayerId::new(target),
                    helper_def: engine_api::SkillDefId::new(helper_def),
                    action,
                    cards: Vec::new(),
                    split: 0,
                    skill: engine_api::SkillId::NONE,
                }
            }
            effect_tag::RESOLVE_XICHA => {
                let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
                let card = pop(frame)?.0.max(0) as u32;
                let source = pop(frame)?.0.max(0) as u32;
                let player = pop(frame)?.0.max(0) as u32;
                EffectRequest::ResolveXicha {
                    player: PlayerId::new(player),
                    source: PlayerId::new(source),
                    card: engine_api::CardId::new(card),
                    action,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::NONE,
                }
            }
            _ => return super::draw_cards_choices::decode_request(host, frame, tag),
    })
}
