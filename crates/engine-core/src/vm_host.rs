//! Bridge Rule VM host calls into EffectRequest queues for the engine.

use crate::query::{query, QueryContext};
use crate::state::GameState;
use engine_api::{CardId, PlayerId};
use rule_api::{CapabilitySet, EffectRequest, HostTrap, QueryRequest, QueryResponse};
use rule_vm::{HostContext, VmHost};

/// Host that records effects and answers queries against live `GameState`.
pub struct EngineVmHost<'a> {
    /// Borrowed state (read-only queries).
    pub state: &'a GameState,
    /// Capability set.
    pub caps: CapabilitySet,
    /// Skill / event context.
    pub ctx: HostContext,
    /// Effects collected during execution.
    pub effects: Vec<EffectRequest>,
    /// Active invocation card inputs.
    pub selected_cards: Vec<u32>,
    /// Active invocation player targets.
    pub selected_targets: Vec<u32>,
    /// Physical cards captured from the triggering domain event.
    pub event_cards: Vec<u32>,
    /// Ordered player targets captured from the triggering domain event.
    pub event_targets: Vec<u32>,
    /// Damage nature for the current trigger.
    pub event_nature: u8,
    /// Logical damage-card kind for the current trigger.
    pub event_card_kind: u8,
    /// Damage-card provenance flags for the current trigger.
    pub event_card_flags: u8,
    /// Destination owner for the current CardsMoved trigger.
    pub event_move_target: PlayerId,
    /// From/to hand/equipment bits for the current CardsMoved trigger.
    pub event_move_mask: u8,
}

impl<'a> EngineVmHost<'a> {
    /// Create host for a skill firing on an event.
    pub fn new(
        state: &'a GameState,
        skill_id: u32,
        self_id: u32,
        event_player: u32,
        event_source: u32,
        event_card: u32,
        event_amount: u32,
    ) -> Self {
        Self {
            state,
            caps: CapabilitySet::standard_skill(),
            ctx: HostContext {
                skill_id,
                self_id,
                event_player,
                event_source,
                event_card,
                event_amount,
            },
            effects: Vec::new(),
            selected_cards: Vec::new(),
            selected_targets: Vec::new(),
            event_cards: Vec::new(),
            event_targets: Vec::new(),
            event_nature: 0,
            event_card_kind: 0,
            event_card_flags: 0,
            event_move_target: PlayerId::NONE,
            event_move_mask: 0,
        }
    }
}

impl VmHost for EngineVmHost<'_> {
    fn capabilities(&self) -> &CapabilitySet {
        &self.caps
    }

    fn context(&self) -> HostContext {
        self.ctx
    }

    fn selected_cards(&self) -> &[u32] {
        &self.selected_cards
    }

    fn selected_targets(&self) -> &[u32] {
        &self.selected_targets
    }

    fn event_cards(&self) -> &[u32] {
        &self.event_cards
    }

    fn event_targets(&self) -> &[u32] {
        &self.event_targets
    }

    fn emit_effect(&mut self, req: EffectRequest) -> Result<(), HostTrap> {
        if !self.caps.emit_effects {
            return Err(HostTrap::CapabilityDenied {
                name: "emit_effects".into(),
            });
        }
        self.effects.push(match req {
            EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
                player,
                card,
                target,
                retieji: true,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
                    player,
                    card,
                    target,
                    retieji: true,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                    ready_to_choose: false,
                    judgment_suit: 0,
                }
            }
            EffectRequest::TakeOneHandCard { from, to } if from.is_none() => {
                EffectRequest::TakeOneHandCard {
                    from: PlayerId::new(*self.event_targets.first().ok_or_else(|| {
                        HostTrap::InvalidRequest {
                            message: "event has no counterpart".into(),
                        }
                    })?),
                    to,
                }
            }
            EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::LoseHpDoubleCurrentDamageThenReplace {
                player,
                amount,
                replacement_skill_def,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::LoseHpDoubleCurrentDamageThenReplace {
                    player,
                    damage_id: self.state.resolution.current_damage_id,
                    amount,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                    replacement_skill_def,
                    ready_to_replace: false,
                }
            }
            EffectRequest::PreventDamage { damage_id: 0 } => EffectRequest::PreventDamage {
                damage_id: self.state.resolution.current_damage_id,
            },
            EffectRequest::PreventRecovery { recovery_id: 0 } => EffectRequest::PreventRecovery {
                recovery_id: self.state.resolution.current_recovery_id,
            },
            EffectRequest::JudgePreventDamageLoseMaxHpOnNonHeart {
                player,
                target,
                damage_id: 0,
            } => EffectRequest::JudgePreventDamageLoseMaxHpOnNonHeart {
                player,
                target,
                damage_id: self.state.resolution.current_damage_id,
            },
            EffectRequest::ChooseDiscardEquipmentReduceCurrentDamage {
                player,
                damage_id: 0,
                ..
            } => EffectRequest::ChooseDiscardEquipmentReduceCurrentDamage {
                player,
                damage_id: self.state.resolution.current_damage_id,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage {
                player,
                damage_id: 0,
                ..
            } => EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage {
                player,
                damage_id: self.state.resolution.current_damage_id,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseDiscardDamageEndpointCardIncreaseIfSuit {
                player,
                target,
                damage_id: 0,
                suit,
                amount,
                ..
            } => EffectRequest::ChooseDiscardDamageEndpointCardIncreaseIfSuit {
                player,
                target,
                damage_id: self.state.resolution.current_damage_id,
                suit,
                amount,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
                player,
                target,
                damage_id: 0,
                ..
            } => EffectRequest::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
                player,
                target,
                damage_id: self.state.resolution.current_damage_id,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseTargetDiscardDistanceHandOrIncreaseDamage {
                player,
                target,
                damage_id: 0,
                ..
            } => EffectRequest::ChooseTargetDiscardDistanceHandOrIncreaseDamage {
                player,
                target,
                damage_id: self.state.resolution.current_damage_id,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
                player,
                target,
                damage_id: 0,
                ..
            } => {
                EffectRequest::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
                    player,
                    target,
                    damage_id: self.state.resolution.current_damage_id,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseDrawOrRecastAddSkill {
                player,
                draw_skill_def,
                recast_skill_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseDrawOrRecastAddSkill {
                player,
                draw_skill_def,
                recast_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha {
                player,
                card,
                target,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha {
                    player,
                    card,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseGainShaForOpeningExtraPlay { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseGainShaForOpeningExtraPlay {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::FinishOpeningExtraPlayShaGain { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::FinishOpeningExtraPlayShaGain {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay {
                player,
                kind,
                mandatory_skill_def,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay {
                    player,
                    kind,
                    mandatory_skill_def,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseGainOwnLostDiscardCardThisTurn { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseGainOwnLostDiscardCardThisTurn {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseMoveFieldCardTo { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseMoveFieldCardTo {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::DrawThenOptionallyDistributeHandCards {
                player,
                draw_count,
                give_cap,
                cards,
                targets,
                ready_to_choose,
                finish,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::DrawThenOptionallyDistributeHandCards {
                player,
                draw_count,
                give_cap,
                cards,
                targets,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
                ready_to_choose,
                finish,
            },
            EffectRequest::GiveSelectedThenContinueDistinctRecoverAtTwo {
                player,
                target,
                cards,
                used_targets,
                total_given,
                continue_only,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::GiveSelectedThenContinueDistinctRecoverAtTwo {
                    player,
                    target,
                    cards,
                    used_targets,
                    total_given,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                    continue_only,
                }
            }
            EffectRequest::GiveSelectedThenRecoverTarget {
                player,
                target,
                cards,
                amount,
                ready_to_recover,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::GiveSelectedThenRecoverTarget {
                player,
                target,
                cards,
                amount,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
                ready_to_recover,
            },
            EffectRequest::GiveSelectedThenRecoverSelf {
                player,
                target,
                cards,
                amount,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::GiveSelectedThenRecoverSelf {
                player,
                target,
                cards,
                amount,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDingpan {
                player,
                target,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDingpan {
                player,
                target,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTanlu {
                player,
                target,
                amount,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTanlu {
                player,
                target,
                amount,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
                player, mark, ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
                    player,
                    mark,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
                player,
                priority,
                remaining_targets,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
                    player,
                    priority,
                    remaining_targets,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
                player,
                initialized,
                remaining_targets,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
                    player,
                    initialized,
                    remaining_targets,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveHiddenSkillPile {
                player,
                pile_skill_def,
                action,
                card,
                stage,
                hand_cards,
                pile_cards,
                event_cards,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::SetNonLockedSkillSuppression {
                player, enabled, ..
            } if self.ctx.skill_id != 0 => EffectRequest::SetNonLockedSkillSuppression {
                player,
                enabled,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDiliYuheng {
                player,
                action,
                card,
                event_cards,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDiliYuheng {
                player,
                action,
                card,
                event_cards,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::DiscardSelectedDrawAllHandBonus { player, cards, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::DiscardSelectedDrawAllHandBonus {
                    player,
                    cards,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseOtherYinghun {
                player,
                target,
                count,
                branch,
                ready_to_discard,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseOtherYinghun {
                player,
                target,
                count,
                branch,
                ready_to_discard,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseOtherBizheng {
                player,
                target,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseOtherBizheng {
                player,
                target,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseOtherXiashu {
                player,
                target,
                stage,
                cards,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseOtherXiashu {
                player,
                target,
                stage,
                cards,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveGuanchaoRank { player, card, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveGuanchaoRank {
                    player,
                    card,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveGuanwei {
                player,
                target,
                stage,
                card,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveGuanwei {
                player,
                target,
                stage,
                card,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveLanjiang {
                player,
                stage,
                index,
                target,
                targets,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveLanjiang {
                player,
                stage,
                index,
                target,
                targets,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveRebingyi { player, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::ResolveRebingyi {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveRebotu { player, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::ResolveRebotu {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveOldimeng {
                player,
                stage,
                first,
                second,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveOldimeng {
                player,
                stage,
                first,
                second,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveRejieyin {
                player,
                target,
                stage,
                card,
                equip,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveRejieyin {
                player,
                target,
                stage,
                card,
                equip,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveReanxu {
                player,
                stage,
                first,
                second,
                card,
                from_hand,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveReanxu {
                player,
                stage,
                first,
                second,
                card,
                from_hand,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveOldAnxu {
                player,
                stage,
                giver,
                receiver,
                card,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveOldAnxu {
                player,
                stage,
                giver,
                receiver,
                card,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::StoreSelectedInSkillPile { player, cards, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::StoreSelectedInSkillPile {
                    player,
                    cards,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveDcbingji {
                player,
                stage,
                kind,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDcbingji {
                player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
                stage,
                kind,
            },
            EffectRequest::ResolveQuanjia {
                player,
                source,
                target,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveQuanjia {
                player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
                source,
                target,
            },
            EffectRequest::ResolveDuwang { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveDuwang {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveOldQinqing {
                player,
                target,
                lord,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveOldQinqing {
                player,
                target,
                lord,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveOldMingjian {
                player,
                target,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveOldMingjian {
                player,
                target,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveOldXuanfeng {
                player,
                target,
                card,
                stage,
                branch,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveOldXuanfeng {
                player,
                target,
                card,
                stage,
                branch,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveJianrou {
                player,
                damage_id: 0,
                ..
            } => EffectRequest::ResolveJianrou {
                player,
                damage_id: self.state.resolution.current_damage_id,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
                    PlayerId::new(self.ctx.event_source)
                },
                damage_id: self.state.resolution.current_damage_id,
                stage,
                cards,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
                    PlayerId::new(self.ctx.event_player)
                },
                current: if current.is_some() {
                    current
                } else {
                    self.state.turn.current
                },
                card: if card.is_some() {
                    card
                } else {
                    engine_api::CardId::new(self.ctx.event_card)
                },
                damage_id: if damage_id != 0 {
                    damage_id
                } else {
                    self.state.resolution.current_damage_id
                },
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
                    engine_api::CardId::new(self.ctx.event_card)
                },
                targets: if targets.is_empty() {
                    self.event_targets
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
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDrltJueyan {
                player,
                stage,
                slot,
                first,
                sha_skill_def,
                weapon_skill_def,
                mount_skill_def,
                armor_skill_def,
                treasure_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveRejizhi {
                player,
                stage,
                card,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveRejizhi {
                player,
                stage,
                card,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveJiexun {
                player,
                stage,
                target,
                draw_count,
                discard_count,
                funan_helper_skill_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveJiexun {
                player,
                stage,
                target,
                draw_count,
                discard_count,
                funan_helper_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveWenguaSetup {
                player,
                helper_skill_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveWenguaSetup {
                player,
                helper_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveWengua {
                player,
                stage,
                target,
                card,
                branch,
                main_skill_def,
                marker_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveXinanguo {
                player,
                target,
                stage,
                index,
                remaining,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveXinanguo {
                player,
                target,
                stage,
                index,
                remaining,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveGanglieThree { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveGanglieThree {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveZhongyi {
                player,
                action,
                helper_skill_def,
                card,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveZhongyi {
                player,
                action,
                helper_skill_def,
                card,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMassDamageDiscardTurnOver {
                player,
                mark,
                mark_count,
                damage_amount,
                hand_count,
                stage,
                index,
                targets,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMassDamageDiscardHandEquip {
                player,
                mark,
                mark_count,
                damage_amount,
                discard_count,
                stage,
                index,
                targets,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::RecordActiveTarget { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::RecordActiveTarget {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseDiscardWeaponOrLoseHp { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseDiscardWeaponOrLoseHp {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseDiscardOneHeOrLoseHp {
                player,
                lose_amount,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseDiscardOneHeOrLoseHp {
                player,
                lose_amount,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice {
                source,
                target,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice {
                source,
                target,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseSourceConfirmOwnerLoseHp {
                owner,
                source,
                amount,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseSourceConfirmOwnerLoseHp {
                owner,
                source,
                amount,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMarkParityControl {
                player,
                mark,
                mass_threshold,
                damage_amount,
                stage,
                index,
                target,
                targets,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage {
                player,
                damage_amount,
                nature,
                stage,
                index,
                targets,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage {
                    player,
                    damage_amount,
                    nature,
                    stage,
                    index,
                    targets,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ReplaceDrawWithDistinctSuitSelection {
                player,
                count,
                stage,
                cards,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ReplaceDrawWithDistinctSuitSelection {
                    player,
                    count,
                    stage,
                    cards,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
                player,
                first_skill_def,
                second_skill_def,
                stage,
                cards,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
                    player,
                    first_skill_def,
                    second_skill_def,
                    stage,
                    cards,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw {
                player,
                target,
                selected,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw {
                    player,
                    target,
                    selected,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::InspectTargetHandChooseHeartDiscardOrTop {
                player,
                target,
                stage,
                card,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::InspectTargetHandChooseHeartDiscardOrTop {
                    player,
                    target,
                    stage,
                    card,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveYeyan {
                player,
                mode,
                stage,
                index,
                targets,
                data,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseAllLoseHpOrRecover { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseAllLoseHpOrRecover {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveQixing {
                player,
                initialize,
                stage,
                hand_cards,
                pile_cards,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveQixing {
                player,
                initialize,
                stage,
                hand_cards,
                pile_cards,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveQixingWeather {
                player,
                qixing_skill_def,
                helper_skill_def,
                max_targets,
                stage,
                cards,
                targets,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveNzryLongnu {
                player,
                fire_skill_def,
                thunder_skill_def,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveNzryLongnu {
                player,
                fire_skill_def,
                thunder_skill_def,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseLinkOther { player, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseLinkOther {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveDrltPoxi {
                player,
                target,
                hand_limit_skill_def,
                stage,
                own_count,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDrltPoxi {
                player,
                target,
                hand_limit_skill_def,
                stage,
                own_count,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDrltJieying {
                player,
                event_player,
                mark,
                mode,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDrltJieying {
                player,
                event_player,
                mark,
                mode,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDrltDuorui {
                player,
                target,
                helper_skill_def,
                stage,
                selected_skill_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDrltDuorui {
                player,
                target,
                helper_skill_def,
                stage,
                selected_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDrltZhiti {
                player,
                event_player,
                event_source,
                event_amount,
                mode,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDrltZhiti {
                player,
                event_player,
                event_source,
                event_amount,
                mode,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveShenSimayi {
                player,
                event_player,
                event_amount,
                mark,
                mode,
                cards,
                targets,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveBeige {
                player,
                target,
                source,
                stage,
                result_card,
                suit,
                rank,
                rewriters,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveBeigeConfigured {
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
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::RevealTargetHandGainShanArmorOrDrawTwo {
                player, target, ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::RevealTargetHandGainShanArmorOrDrawTwo {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::DrawCards { player, count } if self.ctx.skill_id != 0 => {
                EffectRequest::DrawCardsFromSkill {
                    player,
                    count,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::DrawEachBySeat {
                first,
                second,
                count,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::DrawEachBySeat {
                first,
                second,
                count,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::DrawThenGiveOne {
                player,
                target,
                count,
                drawn_only,
                use_given_equipment,
                drawn,
                ready_to_give,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::DrawThenGiveOne {
                player,
                target,
                count,
                drawn_only,
                use_given_equipment,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
                drawn,
                ready_to_give,
            },
            EffectRequest::DrawThenTakeOne {
                player,
                taker,
                count,
                ready_to_take,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::DrawThenTakeOne {
                player,
                taker,
                count,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
                ready_to_take,
            },
            EffectRequest::MoveEquipmentThenDraw { player, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::MoveEquipmentThenDraw {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseUseVirtualShaOrDraw { player, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseUseVirtualShaOrDraw {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::DiscardEquipmentPairWithArmor {
                player, targets, ..
            } if self.ctx.skill_id != 0 => EffectRequest::DiscardEquipmentPairWithArmor {
                player,
                targets,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseOtherDrawByHandCount {
                player,
                normal_count,
                empty_count,
                times,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseOtherDrawByHandCount {
                player,
                normal_count,
                empty_count,
                times,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseOtherDrawOrRecover {
                player,
                draw_count,
                recover_count,
                times,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseOtherDrawOrRecover {
                player,
                draw_count,
                recover_count,
                times,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill { player, count, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill {
                    player,
                    count,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseDiscardHandEquipThenRecover {
                player,
                count,
                amount,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseDiscardHandEquipThenRecover {
                player,
                count,
                amount,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory {
                player,
                target,
                count,
                amount,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory {
                    player,
                    target,
                    count,
                    amount,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseCompareSelectedHandCardSecondFailureSeatDamage {
                player,
                target,
                card,
                draw_count,
                failure_count,
                lose_hp,
                damage,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseCompareSelectedHandCardSecondFailureSeatDamage {
                    player,
                    target,
                    card,
                    draw_count,
                    failure_count,
                    lose_hp,
                    damage,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::DrawSelectedTargets { targets, count, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::DrawSelectedTargets {
                    targets,
                    count,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveOlwenren { player, targets, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::ResolveOlwenren {
                    player,
                    targets,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::RecoverSelectedTargets {
                source,
                targets,
                amount,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::RecoverSelectedTargets {
                source,
                targets,
                amount,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::RecordSelectedPlayPhaseCategory { card, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::RecordSelectedPlayPhaseCategory {
                    card,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseTargetDraw {
                player,
                count,
                scope,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseTargetDraw {
                player,
                count,
                scope,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseTargetsDraw {
                player,
                count,
                target_count,
                scope,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseTargetsDraw {
                player,
                count,
                target_count,
                scope,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseTargetsDamage {
                player,
                min_count,
                max_count,
                scope,
                amount,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseTargetsDamage {
                player,
                min_count,
                max_count,
                scope,
                amount,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseOptionalTargetsDraw {
                player,
                count,
                min_count,
                max_count,
                scope,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseOptionalTargetsDraw {
                player,
                count,
                min_count,
                max_count,
                scope,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::RevealTargetHandShaOrDiscardBind {
                player,
                target,
                helper_skill_def,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::RevealTargetHandShaOrDiscardBind {
                player,
                target,
                helper_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
                stage,
            },
            EffectRequest::DrawRememberedHandCards {
                player,
                count,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::DrawRememberedHandCards {
                player,
                count,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::DiscardRememberedHandCards { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::DiscardRememberedHandCards {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::DiscardTargetEquipmentDrawWeaponRange { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::DiscardTargetEquipmentDrawWeaponRange {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseTargetRevealHandDamageIfSha { player, amount, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseTargetRevealHandDamageIfSha {
                    player,
                    amount,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseOtherGiveOneHandEquip { player, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseOtherGiveOneHandEquip {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseRevealHandCardBindCategoryTarget { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseRevealHandCardBindCategoryTarget {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::RecoverToMaxThenOptionalExchangeSkill {
                player,
                removable_skill_def,
                restore_skill_def,
                ready_to_choose,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::RecoverToMaxThenOptionalExchangeSkill {
                player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
                removable_skill_def,
                restore_skill_def,
                ready_to_choose,
            },
            EffectRequest::ChooseTargetDrawToMaxHpCap { player, cap, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseTargetDrawToMaxHpCap {
                    player,
                    cap,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::DrawByPlayPhaseCardCategoryCount { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::DrawByPlayPhaseCardCategoryCount {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed {
                player,
                sha_user,
                card,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed {
                    player,
                    sha_user,
                    card,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveOllinjie {
                player,
                source,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveOllinjie {
                player,
                source,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
                player,
                sha_user,
                card,
                restore_skill_def,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
                    player,
                    sha_user,
                    card,
                    restore_skill_def,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::QueueOpeningExtraPlayPhase { player, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::QueueOpeningExtraPlayPhase {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage { player, card, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage {
                    player,
                    card,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::IncreaseCurrentShaBaseDamage {
                player,
                card,
                amount,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::IncreaseCurrentShaBaseDamage {
                player,
                card,
                amount,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::IncreaseCurrentCardBaseDamage {
                player,
                card,
                amount,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::IncreaseCurrentCardBaseDamage {
                player,
                card,
                amount,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
                player,
                card,
                draw_count,
                damage_amount,
                mark_def,
                mark_count,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
                    player,
                    card,
                    draw_count,
                    damage_amount,
                    mark_def,
                    mark_count,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::DrawThenChooseGiveOrDiscardHand {
                player,
                draw_count,
                payment_count,
                ready_to_choose,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::DrawThenChooseGiveOrDiscardHand {
                player,
                draw_count,
                payment_count,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
                ready_to_choose,
            },
            EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp {
                player,
                target,
                count,
                ready_to_check,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp {
                player,
                target,
                count,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
                ready_to_check,
            },
            EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess {
                player,
                target,
                cap,
                ready_to_discard,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess {
                    player,
                    target,
                    cap,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                    ready_to_discard,
                }
            }
            EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
                player,
                target,
                card,
                suit,
                ready_to_choose,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
                    player,
                    target,
                    card,
                    suit,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                    ready_to_choose,
                }
            }
            EffectRequest::TargetChooseSuitGainOwnerHandDamageIfMismatch {
                player,
                target,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::TargetChooseSuitGainOwnerHandDamageIfMismatch {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo {
                player,
                target,
                cards,
                ready_to_recover,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo {
                player,
                target,
                cards,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
                ready_to_recover,
            },
            EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
                player,
                targets,
                index,
                selected,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
                    player,
                    targets,
                    index,
                    selected,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
                player,
                targets,
                index,
                selected,
                draw_count,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
                    player,
                    targets,
                    index,
                    selected,
                    draw_count,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveDcqiaomeng {
                player,
                card,
                target,
                selected,
                helper_skill_def,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDcqiaomeng {
                player,
                card,
                targets: self
                    .event_targets
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                target,
                selected,
                helper_skill_def,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveNewReluoyi {
                player,
                buff_skill_def,
                stage,
                cards,
                accepted,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveNewReluoyi {
                player,
                buff_skill_def,
                stage,
                cards,
                accepted,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveRemoshi {
                player,
                target,
                card,
                helper_skill_def,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveRemoshi {
                player,
                target,
                card,
                helper_skill_def,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveXinyaoming {
                player,
                target,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveXinyaoming {
                player,
                target,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveRehuaiyi {
                player,
                stage,
                color,
                discarded_count,
                targets,
                index,
                gained_count,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveReanguo {
                player,
                target,
                stage,
                index,
                remaining,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveReanguo {
                player,
                target,
                stage,
                index,
                remaining,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveBeizhu {
                player,
                target,
                stage,
                cards,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveBeizhu {
                player,
                target,
                stage,
                cards,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMbshanjia {
                player,
                lost_equipment_count,
                distance_skill_def,
                stage,
                reward_mask,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbshanjia {
                player,
                lost_equipment_count,
                distance_skill_def,
                stage,
                reward_mask,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDaoji {
                player,
                target,
                card,
                stage,
                tw,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDaoji {
                player,
                target,
                card,
                stage,
                tw,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::GiveAllHandThenChooseUseOneOrDamage {
                player,
                target,
                cards,
                targets,
                stage,
                card,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::GiveAllHandThenChooseUseOneOrDamage {
                player,
                target,
                cards,
                targets,
                stage,
                card,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
                player,
                card,
                target,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
                    player,
                    card,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants {
                player,
                card,
                targets,
                stage,
                damage_bonus,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants {
                    player,
                    card,
                    targets,
                    stage,
                    damage_bonus,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveTwfupan {
                player,
                damage_amount,
                card,
                target,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwfupan {
                player,
                damage_amount,
                card,
                target,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwjiaohua {
                player,
                target,
                event_cards,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwjiaohua {
                player,
                target,
                event_cards,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwkujian {
                player,
                target,
                cards,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwkujian {
                player,
                target,
                cards,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwruilian {
                player,
                target,
                event_cards,
                action,
                category,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwruilian {
                player,
                target,
                event_cards,
                action,
                category,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwjianwei {
                player,
                actor,
                target,
                card,
                winner,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwjianwei {
                player,
                actor,
                target,
                card,
                winner,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwtanfeng {
                player,
                target,
                card,
                stage,
                phase,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwtanfeng {
                player,
                target,
                card,
                stage,
                phase,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwzhibian {
                player,
                target,
                winner,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwzhibian {
                player,
                target,
                winner,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwyuyan {
                player,
                source,
                card,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwyuyan {
                player,
                source,
                card,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwguoyi {
                player,
                target,
                card,
                hand_lock_skill_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwguoyi {
                player,
                target,
                card,
                hand_lock_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwchuhai {
                player,
                action,
                stage,
                index,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwchuhai {
                player,
                action,
                stage,
                index,
                event_cards: self
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwyizhu {
                player,
                source,
                card,
                action,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwyizhu {
                player,
                source,
                card,
                action,
                stage,
                event_cards: self
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwluanchou {
                player,
                helper_skill_def,
                action,
                targets,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwluanchou {
                player,
                helper_skill_def,
                action,
                targets,
                source: engine_api::PlayerId::new(self.ctx.event_source),
                target: engine_api::PlayerId::new(self.ctx.event_player),
                amount: self.ctx.event_amount,
                nature: self.event_nature,
                card: engine_api::CardId::new(self.ctx.event_card),
                card_kind: self.event_card_kind,
                card_flags: self.event_card_flags,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveYuli { player, mode, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveYuli {
                    player,
                    mode,
                    source: engine_api::PlayerId::new(self.ctx.event_source),
                    target: engine_api::PlayerId::new(self.ctx.event_player),
                    amount: self.ctx.event_amount,
                    nature: self.event_nature,
                    card: engine_api::CardId::new(self.ctx.event_card),
                    card_kind: self.event_card_kind,
                    card_flags: self.event_card_flags,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveTingwei { player, mode, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveTingwei {
                    player,
                    mode,
                    card: engine_api::CardId::new(self.ctx.event_card),
                    targets: self
                        .event_targets
                        .iter()
                        .map(|raw| engine_api::PlayerId::new(*raw))
                        .collect(),
                    target: engine_api::PlayerId::NONE,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveXinmingshi { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveXinmingshi {
                    player,
                    source: engine_api::PlayerId::new(self.ctx.event_source),
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveXunyi { player, mode, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveXunyi {
                    player,
                    mode,
                    source: engine_api::PlayerId::new(self.ctx.event_source),
                    target: engine_api::PlayerId::new(self.ctx.event_player),
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveBoming { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveBoming {
                    player,
                    target,
                    card: self.selected_cards.first().copied().map(engine_api::CardId::new).unwrap_or(engine_api::CardId::NONE),
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveSpqiai { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveSpqiai {
                    player,
                    target,
                    card: self.selected_cards.first().copied().map(engine_api::CardId::new).unwrap_or(engine_api::CardId::NONE),
                    stage: 0,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveEjian { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveEjian {
                    player,
                    target,
                    card: self.event_cards.first().copied().map(engine_api::CardId::new).unwrap_or(engine_api::CardId::NONE),
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveYiyong { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveYiyong {
                    player,
                    card: engine_api::CardId::new(self.ctx.event_card),
                    source: engine_api::PlayerId::new(self.ctx.event_source),
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveShanxie { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveShanxie {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveTuntian { player, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::ResolveTuntian {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveZaoxian {
                player,
                jixi_def,
                tuntian_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveZaoxian {
                player,
                jixi_def,
                tuntian_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTianxiang {
                player,
                draw_skill_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTianxiang {
                player,
                target: PlayerId::NONE,
                discard_card: engine_api::CardId::NONE,
                source: engine_api::PlayerId::new(self.ctx.event_source),
                amount: self.ctx.event_amount,
                nature: self.event_nature,
                card: engine_api::CardId::new(self.ctx.event_card),
                card_kind: self.event_card_kind,
                card_flags: self.event_card_flags,
                draw_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveRetianxiang {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveRetianxiang {
                player,
                target: PlayerId::NONE,
                discard_card: engine_api::CardId::NONE,
                branch: 0,
                source: engine_api::PlayerId::new(self.ctx.event_source),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseOtherAddMark {
                player,
                mark,
                delta,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseOtherAddMark {
                player,
                mark,
                delta,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveStddechong {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveStddechong {
                player,
                target: engine_api::PlayerId::new(self.ctx.event_player),
                cards: Vec::new(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveStdjujian {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveStdjujian {
                player,
                target: PlayerId::NONE,
                cards: (self.ctx.event_card != 0)
                    .then(|| vec![engine_api::CardId::new(self.ctx.event_card)])
                    .unwrap_or_default(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDctingxian {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDctingxian {
                player,
                card: engine_api::CardId::new(self.ctx.event_card),
                targets: self
                    .event_targets
                    .iter()
                    .copied()
                    .map(engine_api::PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveXinfuQinguo {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveXinfuQinguo {
                player,
                card: engine_api::CardId::new(self.ctx.event_card),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveXinxingluan {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveXinxingluan {
                player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDckuizhen {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDckuizhen {
                player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDckuizhenResolve {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDckuizhenResolve {
                player,
                winner: PlayerId::new(self.ctx.event_player),
                loser: PlayerId::new(self.ctx.event_source),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveNsdingzhou {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveNsdingzhou {
                player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveSxrmlanjiao {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveSxrmlanjiao {
                player,
                target: PlayerId::NONE,
                stage: 0,
                view: Vec::new(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveRekuanshi {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveRekuanshi {
                player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveRekuanshiResolve {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveRekuanshiResolve {
                player,
                damaged: PlayerId::new(self.ctx.event_player),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDcmoyu {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDcmoyu {
                player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveXizhen {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveXizhen {
                player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveXuxie {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveXuxie {
                player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveXuxieAdd {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveXuxieAdd {
                player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDcchiying {
                player,
                target,
                stage,
                accum,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDcchiying {
                player,
                target,
                stage,
                accum,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDcmanzhi {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDcmanzhi {
                player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveXinkuangfu {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveXinkuangfu {
                player,
                target: PlayerId::NONE,
                sha_target: PlayerId::NONE,
                stage: 1,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDcxiangshu {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDcxiangshu {
                player,
                target: PlayerId::new(self.ctx.event_player),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDcxiangshuLottery {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDcxiangshuLottery {
                player,
                target: PlayerId::new(self.ctx.event_player),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveGongjian {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveGongjian {
                player,
                targets: self
                    .event_targets
                    .iter()
                    .copied()
                    .map(engine_api::PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss {
                player,
                reveal_count,
                recipient_cap,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss {
                    player,
                    reveal_count,
                    recipient_cap,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp {
                player, ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveRefuhai { player, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::ResolveRefuhai {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseGainTargetHandCeilHalfDifference {
                player,
                target,
                card,
                ready_to_choose,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseGainTargetHandCeilHalfDifference {
                    player,
                    target,
                    card,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                    ready_to_choose,
                }
            }
            EffectRequest::ResolveGzjinfa {
                player,
                target,
                cost_card,
                given_card,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveGzjinfa {
                player,
                target,
                cost_card,
                given_card,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveWnQifeng { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveWnQifeng {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
                player,
                target,
                card,
                targets,
                stage,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
                    player,
                    target,
                    card,
                    targets,
                    stage,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveDcyingtu { player, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::ResolveDcyingtu {
                    player,
                    target: PlayerId::NONE,
                    stage: 0,
                    cards: self
                        .event_cards
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
                player,
                target,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
                    player,
                    target,
                    damage_id: self.state.resolution.current_damage_id,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
                player,
                source,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
                    player,
                    source,
                    card: engine_api::CardId::NONE,
                    stage: 0,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveMobiledanshou { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveMobiledanshou {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveJsrgchiying { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveJsrgchiying {
                    player,
                    target,
                    started: false,
                    remaining: Vec::new(),
                    accum: Vec::new(),
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveMbqingdao {
                player,
                source,
                card,
                damaged,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbqingdao {
                player,
                source,
                card,
                damaged,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMbxiugeng {
                player,
                helper_def,
                hand_limit_def,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbxiugeng {
                player,
                helper_def,
                hand_limit_def,
                stage,
                targets: Vec::new(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMbchenshe {
                player,
                dying,
                source,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbchenshe {
                player,
                dying,
                source,
                stage: 0,
                roles: Vec::new(),
                suits: Vec::new(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMbquanchong {
                player, helper_def, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbquanchong {
                player,
                helper_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMbrenxing { player, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::ResolveMbrenxing {
                    player,
                    stage: 0,
                    target: PlayerId::NONE,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveMbchizhang { player, card, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveMbchizhang {
                    player,
                    card,
                    stage: 0,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveMbduanyang {
                player,
                target,
                card,
                mode,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbduanyang {
                player,
                target,
                card,
                mode,
                stage: 0,
                cards: self
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMbrunwei {
                player,
                target,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbrunwei {
                player,
                target,
                stage,
                value: 0,
                cards: self.event_cards.iter().copied().map(engine_api::CardId::new).collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMbshuanghuai { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveMbshuanghuai {
                    player,
                    target,
                    stage: 0,
                    choice: 0,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveMbxiongtu {
                player,
                target,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbxiongtu {
                player,
                target,
                stage,
                card: engine_api::CardId::NONE,
                cards: Vec::new(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMbganggeng {
                player,
                target,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbganggeng {
                player,
                target,
                stage,
                cards: self
                    .selected_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMbsijian {
                player,
                helper_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbsijian {
                player,
                target: engine_api::PlayerId::NONE,
                stage: 0,
                choice: 0,
                helper_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMbjianji { player, .. } if self.ctx.skill_id != 0 => {
                let source = self
                    .selected_targets
                    .first()
                    .copied()
                    .map(engine_api::PlayerId::new)
                    .unwrap_or(engine_api::PlayerId::NONE);
                let target = self
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
                    card: self
                        .selected_cards
                        .first()
                        .copied()
                        .map(engine_api::CardId::new)
                        .unwrap_or(engine_api::CardId::NONE),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveMbyuanmo { player, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::ResolveMbyuanmo {
                    player,
                    target: engine_api::PlayerId::NONE,
                    stage: 0,
                    card: engine_api::CardId::NONE,
                    value: 0,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveMbfeijing {
                player, target, card, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbfeijing {
                player,
                target,
                card,
                stage: 0,
                targets: Vec::new(),
                cards: Vec::new(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMbxiaoge {
                player,
                target,
                card,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbxiaoge {
                player,
                target,
                card,
                stage,
                targets: self
                    .event_targets
                    .iter()
                    .copied()
                    .map(engine_api::PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMbxuye { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveMbxuye {
                    player,
                    target,
                    stage: 0,
                    was_max: false,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveMbkuangxiang {
                player, target, stage, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbkuangxiang {
                player,
                target,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotchengpu {
                player,
                target,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotchengpu {
                player,
                target,
                stage,
                damage_id: 0,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotjugu { player, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::ResolvePotjugu {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolvePotsifeng {
                player,
                target,
                card,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotsifeng {
                player,
                target,
                card,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotchenqun {
                player,
                target,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotchenqun {
                player,
                target,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotwanglie {
                player,
                card,
                helper_skill_def,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotwanglie {
                player,
                card,
                helper_skill_def,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePothongyi { player, stage, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolvePothongyi {
                    player,
                    stage,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolvePotguansha {
                player,
                helper_skill_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotguansha {
                player,
                helper_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotjiyu {
                player,
                card,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotjiyu {
                player,
                card,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMbzhuji {
                player,
                choice,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbzhuji {
                player,
                choice,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotlibing {
                player,
                card,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotlibing {
                player,
                card,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotpoxi { player, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::ResolvePotpoxi {
                    player,
                    cards: self
                        .event_cards
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolvePotfuan {
                player,
                card,
                stage,
                range_helper_def,
                yinxian_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotfuan {
                player,
                card,
                stage,
                range_helper_def,
                yinxian_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotyinxian {
                player, card, stage, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotyinxian {
                player,
                card,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePothuanjie {
                player,
                target,
                stage,
                qice_def,
                kanpo_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePothuanjie {
                player,
                target,
                stage,
                qice_def,
                kanpo_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotnigu {
                player,
                target,
                card,
                targets,
                index,
                declined,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotlulian {
                player,
                card,
                target,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotlulian {
                player,
                card,
                target,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotdimeng {
                player, targets, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotdimeng {
                player,
                targets,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePothaoshi {
                player,
                target,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePothaoshi {
                player,
                target,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveSpmiaolve { player, action, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveSpmiaolve {
                    player,
                    action,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveSpyingjia {
                player,
                action,
                helper_def,
                target,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveSpyingjia {
                player,
                action,
                helper_def,
                card: engine_api::CardId::new(self.ctx.event_card),
                card_kind: self.event_card_kind,
                target,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTiaoxin {
                player,
                target,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTiaoxin {
                player,
                target,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveZhiji {
                player,
                guanxing_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveZhiji {
                player,
                guanxing_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveZaiqi {
                player,
                extra,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveZaiqi {
                player,
                extra,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwqiushou { player, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::ResolveTwqiushou {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolvePottuntian {
                player,
                action,
                amount,
                targets,
                zaoxian_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePottuntian {
                player,
                action,
                amount,
                targets,
                zaoxian_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotjixi {
                player,
                target,
                action,
                targets,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotjixi {
                player,
                target,
                action,
                targets,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotceduan {
                player,
                target,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotceduan {
                player,
                target,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePottaishici {
                player,
                target,
                action,
                cards,
                hanzhan_def,
                zhanlie_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePottaishici {
                player,
                target,
                action,
                cards,
                hanzhan_def,
                zhanlie_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMbzhonghui {
                player,
                target,
                action,
                amount,
                mbsizi_def,
                mbkechang_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbzhonghui {
                player,
                target,
                action,
                amount,
                mbsizi_def,
                mbkechang_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotjiejie {
                player,
                actor,
                used_def,
                suit_defs,
                qingshi_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotjiejie {
                player,
                actor,
                used_def,
                suit_defs,
                qingshi_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotqingshi { player, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::ResolvePotqingshi {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolvePotweiyan {
                player,
                target,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotweiyan {
                player,
                target,
                card,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotzhouyu {
                player,
                target,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotzhouyu {
                player,
                target,
                card,
                action,
                card_flags: self.event_card_flags,
                targets: self
                    .event_targets
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotlvyi {
                player,
                target,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotlvyi {
                player,
                target,
                card,
                action,
                event_cards: self
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                targets: self
                    .event_targets
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotcaoshuang {
                player,
                target,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotcaoshuang {
                player,
                target,
                card,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolvePotyuji {
                player,
                target,
                card,
                action,
                cards,
                targets,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolvePotyuji {
                player,
                target,
                card,
                action,
                cards: if action == 0 {
                    self.selected_cards
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect()
                } else if cards.is_empty() {
                    self.event_cards
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect()
                } else {
                    cards
                },
                targets: if action == 0 {
                    self.selected_targets
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect()
                } else if targets.is_empty() {
                    self.event_targets
                        .iter()
                        .copied()
                        .map(PlayerId::new)
                        .collect()
                } else {
                    targets
                },
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveHaoshi {
                player,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveHaoshi {
                player,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDimeng {
                player,
                targets,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDimeng {
                player,
                targets,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveQiaobian {
                player,
                phase,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveQiaobian {
                player,
                phase,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveXiangle {
                player,
                source,
                card,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveXiangle {
                player,
                source,
                card,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveFangquanSkip {
                player,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveFangquanSkip {
                player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveFangquanGrant {
                player,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveFangquanGrant {
                player,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveRuoyu {
                player,
                jijiang_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveRuoyu {
                player,
                jijiang_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveZuilun {
                player,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveZuilun {
                player,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveFuyin {
                player,
                source,
                card,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveFuyin {
                player,
                source,
                card,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveJuzhan {
                player,
                source,
                card,
                targets,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveJuzhan {
                player,
                source,
                card,
                targets,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveQizhi {
                player,
                targets,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveQizhi {
                player,
                targets,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveJinqu {
                player,
                qizhi_def,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveJinqu {
                player,
                qizhi_def,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveJianxiang {
                player,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveJianxiang {
                player,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveShenshi {
                player,
                source,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveShenshi {
                player,
                source,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMingren {
                player,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMingren {
                player,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveZhenliang {
                player,
                source,
                card,
                mingren_def,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveZhenliang {
                player,
                source,
                card,
                mingren_def,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveKuizhu {
                player,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveKuizhu {
                player,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveZhizheng {
                player,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveZhizheng {
                player,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveLijun {
                player,
                source,
                card,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveLijun {
                player,
                source,
                card,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveLiangyin {
                player,
                kongsheng_def,
                stage,
                target,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveLiangyin {
                player,
                kongsheng_def,
                stage,
                target,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveKongsheng {
                player,
                stage,
                target,
                card,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveKongsheng {
                player,
                stage,
                target,
                card,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveReleiji {
                player,
                target,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveReleiji {
                player,
                target,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveHuangtian {
                player,
                actor,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveHuangtian {
                player,
                actor,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveZhengrong {
                player,
                candidates,
                target,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveZhengrong {
                player,
                candidates,
                target,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveHongju {
                player,
                qingjian_def,
                rong_def,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveHongju {
                player,
                qingjian_def,
                rong_def,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveQingjian {
                player,
                rong_def,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveQingjian {
                player,
                rong_def,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveXinfuLianpian {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveXinfuLianpian {
                player,
                targets: self
                    .event_targets
                    .iter()
                    .copied()
                    .map(engine_api::PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveJieliangGain {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveJieliangGain {
                player,
                mover: engine_api::PlayerId::new(self.ctx.event_source),
                cards: self
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveShuangxiong {
                player, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveShuangxiong {
                player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveZhijian {
                player,
                card,
                stage,
                target,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveZhijian {
                player,
                card,
                stage,
                target,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveGuzheng {
                player,
                event_player,
                stage,
                cards,
                return_card,
                take_rest,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveGuzheng {
                player,
                event_player,
                stage,
                cards,
                return_card,
                take_rest,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveZhiba {
                player,
                actor,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveZhiba {
                player,
                actor,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDrltCongjian {
                player,
                targets: _,
                card,
                target,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDrltCongjian {
                player,
                targets: self
                    .event_targets
                    .iter()
                    .copied()
                    .map(engine_api::PlayerId::new)
                    .collect(),
                card,
                target,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveNzryFeijun {
                player,
                target,
                card,
                branch,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveNzryFeijun {
                player,
                target,
                card,
                branch,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDrltYongsi {
                player,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDrltYongsi {
                player,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDrltWeidi {
                player,
                cards,
                targets,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDrltWeidi {
                player,
                cards,
                targets,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveLieren {
                player,
                target,
                win_skill_def,
                lose_skill_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveLieren {
                player,
                target,
                win_skill_def,
                lose_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwxiawei {
                player,
                wangxing_skill_def,
                action,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwxiawei {
                player,
                wangxing_skill_def,
                action,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwbianfuren {
                player,
                target,
                helper_skill_def,
                action,
                stage,
                cards,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwbianfuren {
                player,
                target,
                helper_skill_def,
                action,
                stage,
                cards,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwfeiyi {
                player,
                action,
                stage,
                card,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwfeiyi {
                player,
                action,
                stage,
                card,
                event_cards: self
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwchenzhen {
                player,
                target,
                helper_skill_def,
                record_skill_def,
                action,
                stage,
                card_kind,
                card,
                branch,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwqianju { player, target, .. }
                if self.ctx.skill_id != 0 => EffectRequest::ResolveTwqianju {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                },
            EffectRequest::ResolveTwqingxi {
                player,
                target,
                card,
                stage,
                count,
                cards,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwqingxi {
                player,
                target,
                card,
                stage,
                count,
                cards,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwzaoli {
                player,
                stage,
                cards,
                slots,
                count,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwzaoli {
                player,
                stage,
                cards,
                slots,
                count,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwzhengjian {
                player,
                target,
                helper_skill_def,
                action,
                stage,
                card,
                branch,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwtianyu {
                player,
                target,
                helper_skill_def,
                action,
                stage,
                card,
                branch,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwwujing {
                player,
                target_count,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwwujing {
                player,
                target_count,
                action,
                event_cards: self
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwshanghe {
                player,
                stage,
                remaining,
                got_jiu,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwshanghe {
                player,
                stage,
                remaining,
                got_jiu,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwwangchang {
                player,
                target,
                card,
                selected_card,
                action,
                stage,
                targets,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveRehengjiang {
                player,
                actor,
                action,
                hand_limit_def,
                draw_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveRehengjiang {
                player,
                actor,
                action,
                hand_limit_def,
                draw_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwguohuai {
                player,
                target,
                source,
                card,
                mark_def,
                helper_skill_def,
                action,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwchengpu {
                player,
                target,
                source,
                card,
                chunlao_skill_def,
                action,
                stage,
                selected_card,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwcaoang {
                player,
                target,
                source,
                card,
                helper_skill_def,
                action,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwniujin {
                player,
                limited_skill_def,
                action,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwniujin {
                player,
                limited_skill_def,
                action,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwmoukui {
                player,
                target,
                card,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwmoukui {
                player,
                target,
                card,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwfuman {
                player,
                target,
                cards,
                action,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwfuman {
                player,
                target,
                cards,
                action,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveCuijin {
                player,
                source,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveCuijin {
                player,
                source,
                card,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwqingkou {
                player,
                winner,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwqingkou {
                player,
                winner,
                card,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwchenxun {
                player,
                target,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwchenxun {
                player,
                target,
                card,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwfenxian {
                player,
                target,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwfenxian {
                player,
                target,
                card,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::SetSkillContentFlag {
                player,
                bit,
                value,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::SetSkillContentFlag {
                player,
                bit,
                value,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveJiaohua {
                player,
                category,
                target,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveJiaohua {
                player,
                category,
                target,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveLuanqun {
                player,
                effect_skill_def,
                direct_hit_skill_def,
                stage,
                participants,
                cards,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveLuanqun {
                player,
                effect_skill_def,
                direct_hit_skill_def,
                stage,
                participants,
                cards,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::DistributeTopTwoCards {
                player,
                cards,
                selected,
                target,
                ready_to_choose,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::DistributeTopTwoCards {
                player,
                cards,
                selected,
                target,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
                ready_to_choose,
            },
            EffectRequest::ChooseDiscardBasicThenTargetDiscardEquipOrDamage {
                player, target, ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseDiscardBasicThenTargetDiscardEquipOrDamage {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner {
                player, target, ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseTargetLoseHpDiscardHandShaIfBasic {
                player,
                target,
                stage,
                discarded_basic,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseTargetLoseHpDiscardHandShaIfBasic {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                    stage,
                    discarded_basic,
                }
            }
            EffectRequest::ChooseCurrentShaTargetDiscardRedirectToOwner {
                owner,
                source,
                card,
                target,
                ready_to_redirect,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseCurrentShaTargetDiscardRedirectToOwner {
                    owner,
                    source,
                    card,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                    ready_to_redirect,
                }
            }
            EffectRequest::ChooseDiscardRedirectCurrentSha {
                player,
                source,
                card,
                target,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseDiscardRedirectCurrentSha {
                player,
                source,
                card,
                target,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseTargetGainBottomCard {
                player,
                card,
                target,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseTargetGainBottomCard {
                player,
                card,
                target,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::JudgeBlackGainRepeat {
                player,
                card,
                ready_to_choose,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::JudgeBlackGainRepeat {
                player,
                card,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
                ready_to_choose,
            },
            EffectRequest::JudgeBlackGainRepeatProtectHandLimit {
                player,
                card,
                ready_to_choose,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::JudgeBlackGainRepeatProtectHandLimit {
                    player,
                    card,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                    ready_to_choose,
                }
            }
            EffectRequest::JudgeSameColorGainRepeat {
                player,
                card,
                ready_to_choose,
                color,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::JudgeSameColorGainRepeat {
                player,
                card,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
                ready_to_choose,
                color,
            },
            EffectRequest::ChooseCurrentTrickTargetsToNullify {
                player,
                source,
                card,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseCurrentTrickTargetsToNullify {
                player,
                source,
                card,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
                player,
                target,
                ready_to_check,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                    ready_to_check,
                }
            }
            EffectRequest::ChooseGiveOneHandEquipOrRecover {
                owner,
                giver,
                target,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseGiveOneHandEquipOrRecover {
                owner,
                giver,
                target,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseGiveTwoDifferentColorsOrRecover {
                owner,
                giver,
                target,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseGiveTwoDifferentColorsOrRecover {
                owner,
                giver,
                target,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMbzhuguo {
                player,
                target,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbzhuguo {
                player,
                target,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMbanxian {
                player,
                cards,
                card,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbanxian {
                player,
                cards,
                card,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMbjinzu {
                player,
                target,
                cards,
                card,
                helper_skill_def,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMbjinzu {
                player,
                target,
                cards,
                card,
                helper_skill_def,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
                player,
                zone,
                discarded_count,
                ready_to_draw,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                    zone,
                    discarded_count,
                    ready_to_draw,
                }
            }
            EffectRequest::ChooseBoundSourceAttackRangeDamage {
                player,
                source,
                amount,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseBoundSourceAttackRangeDamage {
                player,
                source,
                amount,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseAddCurrentShaTarget { player, card, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseAddCurrentShaTarget {
                    player,
                    card,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
                player,
                card,
                max_distance,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
                    player,
                    card,
                    max_distance,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::RefundCurrentShaUse { player, card, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::RefundCurrentShaUse {
                    player,
                    card,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::DamageThenDrawSelectedTargets {
                source,
                targets,
                damage,
                draw,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::DamageThenDrawSelectedTargets {
                source,
                targets,
                damage,
                draw,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveYoudi {
                player,
                target,
                card,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveYoudi {
                player,
                target,
                card,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveKuangfu {
                player,
                target,
                card,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveKuangfu {
                player,
                target,
                card,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveShanjia {
                player,
                count,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveShanjia {
                player,
                count,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveNewYajiao {
                player,
                source_card,
                revealed,
                target,
                source_category,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveNewYajiao {
                player,
                source_card,
                revealed,
                target,
                source_category,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDingpin { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveDingpin {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::GainTopCardToSkillPile { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::GainTopCardToSkillPile {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
                player,
                target,
                helper_skill_def,
                selected,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
                    player,
                    target,
                    helper_skill_def,
                    selected,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::GainOneHandFromSelectedTargets {
                player,
                targets,
                index,
                selected,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::GainOneHandFromSelectedTargets {
                player,
                targets,
                index,
                selected,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseRepoluTargetsDraw { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseRepoluTargetsDraw {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveZhuiyi { player, killer, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveZhuiyi {
                    player,
                    killer,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveDecadezhenjun {
                player,
                target,
                stage,
                count,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDecadezhenjun {
                player,
                target,
                stage,
                count,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveSpXiaoguo {
                player,
                target,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveSpXiaoguo {
                player,
                target,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
                player,
                initialized,
                remaining_targets,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
                player,
                initialized,
                remaining_targets,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveXinfuGuolun {
                player,
                target,
                player_card,
                target_card,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveXinfuGuolun {
                player,
                target,
                player_card,
                target_card,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseStoreHandShaInSkillPile { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseStoreHandShaInSkillPile {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseSpendSkillPileForDyingJiu { player, dying, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseSpendSkillPileForDyingJiu {
                    player,
                    dying,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseCompareSelectedHandCard {
                player,
                target,
                card,
                win_skill_def,
                lose_skill_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseCompareSelectedHandCard {
                player,
                target,
                card,
                win_skill_def,
                lose_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::RandomizeCurrentHandCompareOpponent { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::RandomizeCurrentHandCompareOpponent {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::GainHighestRankShaFromEventCards { player, cards, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::GainHighestRankShaFromEventCards {
                    player,
                    cards,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::GainHandCompareResultCard {
                player,
                initiator,
                target,
                winner,
                cards,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::GainHandCompareResultCard {
                player,
                initiator,
                target,
                winner,
                cards,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseAddOrRemoveCurrentCardTarget { player, card, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseAddOrRemoveCurrentCardTarget {
                    player,
                    card,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseHandCompare {
                player,
                win_skill_def,
                lose_skill_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseHandCompare {
                player,
                win_skill_def,
                lose_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseHandCompareGrantTargetOnWin {
                player,
                win_grant_def,
                lose_skill_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseHandCompareGrantTargetOnWin {
                player,
                win_grant_def,
                lose_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseHandCompareBetween {
                player,
                first,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseHandCompareBetween {
                player,
                first,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::GuanxingTop {
                player,
                count,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::GuanxingTop {
                player,
                count,
                round: 0,
                flipped: Vec::new(),
                top: Vec::new(),
                bottom: Vec::new(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseOtherGiveThenRecover {
                player,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseOtherGiveThenRecover {
                player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::GuanxingTopByFaluMarks {
                player,
                marks,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::GuanxingTopByFaluMarks {
                player,
                marks,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseOtherDamage { player, amount, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseOtherDamage {
                    player,
                    amount,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::RevealUniformHandChooseDamage { player, amount, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::RevealUniformHandChooseDamage {
                    player,
                    amount,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseGainDiscardCardByKind { player, kind, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseGainDiscardCardByKind {
                    player,
                    kind,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseAddMissingSkill {
                player, skill_defs, ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseAddMissingSkill {
                player,
                skill_defs,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::AddFirstMissingSkill {
                player, skill_defs, ..
            } if self.ctx.skill_id != 0 => EffectRequest::AddFirstMissingSkill {
                player,
                skill_defs,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDccunsi {
                player,
                target,
                guixiu_def,
                yongjue_def,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDccunsi {
                player,
                target,
                guixiu_def,
                yongjue_def,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::RecordCurrentShaAllTargets { player, card, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::RecordCurrentShaAllTargets {
                    player,
                    card,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseGiveOrDiscardHand {
                player,
                payment_count,
                scope,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseGiveOrDiscardHand {
                player,
                payment_count,
                scope,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseDiscardHandEquipThenRecoverTarget {
                chooser,
                target,
                count,
                amount,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseDiscardHandEquipThenRecoverTarget {
                    chooser,
                    target,
                    count,
                    amount,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveDcjue {
                player,
                target,
                stage,
                remaining,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDcjue {
                player,
                target,
                stage,
                remaining,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveRuanji {
                player,
                source,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveRuanji {
                player,
                source,
                card,
                action,
                cards: if self.event_cards.is_empty() {
                    card.is_some().then_some(card).into_iter().collect()
                } else {
                    self.event_cards
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect()
                },
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveZhengbi {
                player,
                target,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveZhengbi {
                player,
                target,
                card,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveFengying {
                player,
                helper_def,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveFengying {
                player,
                helper_def,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveQiaojian {
                player,
                target,
                helper_def,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveQiaojian {
                player,
                target,
                helper_def,
                action,
                cards: self.event_cards.iter().copied().map(engine_api::CardId::new).collect(),
                split: 0,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveXicha {
                player,
                source,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveXicha {
                player,
                source,
                card,
                action,
                cards: if self.event_cards.is_empty() {
                    card.is_some().then_some(card).into_iter().collect()
                } else {
                    self.event_cards.iter().copied().map(engine_api::CardId::new).collect()
                },
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveHuzun {
                player,
                source,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveHuzun {
                player,
                source,
                card,
                action,
                cards: if self.event_cards.is_empty() {
                    card.is_some().then_some(card).into_iter().collect()
                } else {
                    self.event_cards.iter().copied().map(engine_api::CardId::new).collect()
                },
                targets: Vec::new(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveChenglve {
                player,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveChenglve {
                player,
                stage,
                suits: 0,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveShicai {
                player,
                card,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveShicai {
                player,
                card,
                placed: Vec::new(),
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveXiongluan {
                player,
                target,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveXiongluan {
                player,
                target,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveGzbuqu {
                player,
                delta,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveGzbuqu {
                player,
                delta,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveSbbiyue {
                player,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveSbbiyue {
                player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveSxrmqiewang {
                player,
                event_player,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveSxrmqiewang {
                player,
                event_player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveSxrmqianliu {
                player,
                source,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveSxrmqianliu {
                player,
                source,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseOtherHandStoreSkillPile {
                player,
                target,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseOtherHandStoreSkillPile {
                player,
                target,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::GainAllSkillPileCards {
                player,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::GainAllSkillPileCards {
                player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::GainOtherSkillPileCards {
                player,
                target,
                skill_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::GainOtherSkillPileCards {
                player,
                target,
                skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveGzbuquRecover {
                player,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveGzbuquRecover {
                player,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveTwjuxiang {
                player,
                event_player,
                stage,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveTwjuxiang {
                player,
                event_player,
                stage,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveZhugemengxue {
                player,
                target,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveZhugemengxue {
                player,
                target,
                card,
                action,
                cards: self
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveBailingyun {
                player,
                target,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveBailingyun {
                player,
                target,
                card,
                action,
                cards: self
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDcshaowei {
                player,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDcshaowei {
                player,
                action,
                cards: self
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDcdichou {
                player,
                target,
                card,
                action,
                helper_skill_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDcdichou {
                player,
                target,
                card,
                action,
                helper_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDcdufeng {
                player,
                action,
                helper_skill_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDcdufeng {
                player,
                action,
                helper_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseDiscardHandRange {
                player,
                min,
                max,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseDiscardHandRange {
                player,
                min,
                max,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDcctjiuxian {
                player,
                event_player,
                target,
                card,
                action,
                cards,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDcctjiuxian {
                player,
                event_player,
                target,
                card,
                action,
                cards,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveBianyue {
                player,
                target,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveBianyue {
                player,
                target,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveZhupeilan {
                player,
                target,
                card,
                helper_skill_def,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveZhupeilan {
                player,
                target,
                card,
                helper_skill_def,
                action,
                event_cards: self
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDcSbZhangxiu {
                player,
                target,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDcSbZhangxiu {
                player,
                target,
                action,
                cards: Vec::new(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDcSbZhouyu {
                player,
                source,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDcSbZhouyu {
                player,
                source,
                card,
                action,
                event_targets: self
                    .event_targets
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                damaged_targets: self
                    .event_targets
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| {
                        1u32.checked_shl(*index as u32)
                            .is_some_and(|bit| u32::from(self.event_nature) & bit != 0)
                    })
                    .map(|(_, target)| PlayerId::new(*target))
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDcSbLusu {
                player,
                source,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDcSbLusu {
                player,
                source,
                card,
                action,
                cards: self.event_cards.iter().copied().map(CardId::new).collect(),
                amount: self.ctx.event_amount,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveZhangjian {
                player,
                card,
                grant_skill_def,
                action,
                target,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveZhangjian {
                player,
                card,
                grant_skill_def,
                action,
                target,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
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
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveZhugeruoxue {
                player,
                target,
                card,
                action,
                remaining,
                used_mask,
                repeated,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveCaoyi {
                player,
                card,
                helper_skill_def,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveCaoyi {
                player,
                card,
                helper_skill_def,
                action,
                event_targets: self
                    .event_targets
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveMalingli {
                player,
                card,
                helper_skill_def,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveMalingli {
                player,
                card,
                helper_skill_def,
                action,
                source: PlayerId::new(self.ctx.event_source),
                target: PlayerId::new(self.ctx.event_player),
                nature: self.event_nature,
                damage_id: 0,
                event_cards: self.event_cards.iter().copied().map(CardId::new).collect(),
                event_targets: self.event_targets.iter().copied().map(PlayerId::new).collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveNsshuaiyan {
                player,
                recovered_player,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveNsshuaiyan {
                player,
                recovered_player,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveWdbHanlong {
                player,
                target,
                card,
                helper_skill_def,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveWdbHanlong {
                player,
                target,
                card,
                helper_skill_def,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveWdbLiufu {
                player,
                target,
                card,
                zhucheng_skill_def,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveWdbLiufu {
                player,
                target,
                card,
                zhucheng_skill_def,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveWdbLiuyan {
                player,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveWdbLiuyan {
                player,
                card: self
                    .selected_cards
                    .first()
                    .copied()
                    .map(engine_api::CardId::new)
                    .unwrap_or(card),
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveWdbLiuzan {
                player,
                target,
                helper_skill_def,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveWdbLiuzan {
                player,
                target: self
                    .selected_targets
                    .first()
                    .copied()
                    .map(engine_api::PlayerId::new)
                    .unwrap_or(target),
                helper_skill_def,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveWdbTianyu {
                player,
                target,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveWdbTianyu {
                player,
                target,
                card,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveWdbXizhenxihong {
                player,
                target,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveWdbXizhenxihong {
                player,
                target,
                card,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveWdbYangyi {
                player,
                target,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => {
                let event_card = self.event_cards.first().copied().map(engine_api::CardId::new);
                let gained_target = event_card
                    .and_then(|card| self.state.cards.get(card))
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
                        self.selected_cards
                            .first()
                            .copied()
                            .map(engine_api::CardId::new)
                            .unwrap_or(card)
                    },
                    action,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveWdbZaozhiRenjun {
                player,
                target,
                card,
                cards,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveWdbZaozhiRenjun {
                player,
                target,
                card,
                cards,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddXianxi {
                player,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddXianxi {
                player,
                card,
                action,
                event_targets: self
                    .event_targets
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                damaged_targets: self
                    .event_targets
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| {
                        1u32.checked_shl(*index as u32)
                            .is_some_and(|bit| u32::from(self.event_nature) & bit != 0)
                    })
                    .map(|(_, target)| PlayerId::new(*target))
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddXiaheng {
                player,
                discard_target,
                damage_target,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddXiaheng {
                player,
                discard_target,
                damage_target,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddLiuchong {
                player,
                actor,
                card,
                action,
                cards,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddLiuchong {
                player,
                actor,
                card,
                action,
                cards,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddDuanbing {
                player,
                target,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddDuanbing {
                player,
                target,
                card,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddWuzhi {
                player,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddWuzhi {
                player,
                card,
                action,
                cards: self
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddXujing {
                player,
                source,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddXujing {
                player,
                source,
                card,
                action,
                targets: self
                    .event_targets
                    .iter()
                    .copied()
                    .map(engine_api::PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddCaomao {
                player,
                action,
                cards,
                targets,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddCaomao {
                player,
                action,
                cards,
                targets,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddXinxianying {
                player,
                actor,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddXinxianying {
                player,
                actor,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddXianglang {
                player,
                actor,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddXianglang {
                player,
                actor,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddYujin {
                player,
                actor,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddYujin {
                player,
                actor,
                target: self.event_move_target,
                move_mask: self.event_move_mask,
                action,
                candidates: Vec::new(),
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddLiuye {
                player,
                actor,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddLiuye {
                player,
                actor,
                card,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddBaosanniang {
                player,
                target,
                card,
                action,
                cards,
                targets,
                used_kind,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddBaosanniang {
                player,
                target,
                card,
                action,
                cards,
                targets,
                used_kind,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddZhenji {
                player,
                target,
                action,
                stage,
                piles,
                cards,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddZhenji {
                player,
                target,
                action,
                stage,
                piles,
                cards,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddZhaoang {
                player,
                action,
                stage,
                card,
                kind,
                count,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddZhaoang {
                player,
                action,
                stage,
                card,
                kind,
                count,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddLiuba {
                player,
                actor,
                action,
                cards,
                targets,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddLiuba {
                player,
                actor,
                action,
                cards: if action == 0 {
                    self.selected_cards.iter().copied().map(CardId::new).collect()
                } else {
                    cards
                },
                targets,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddJianshuo {
                player,
                target,
                action,
                cards,
                targets,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddJianshuo {
                player,
                target: if action == 0 {
                    self.selected_targets
                        .first()
                        .copied()
                        .map(PlayerId::new)
                        .unwrap_or(target)
                } else {
                    target
                },
                action,
                cards,
                targets,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddGuanning {
                player,
                actor,
                action,
                card,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddGuanning {
                player,
                actor,
                action,
                card: if action == 2 {
                    self.selected_cards.first().copied().map(CardId::new).unwrap_or(card)
                } else {
                    card
                },
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddCaoshuang {
                player,
                actor,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddCaoshuang {
                player,
                actor,
                card,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddXuelingyun {
                player,
                actor,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddXuelingyun {
                player,
                actor,
                card,
                cards: self.event_cards.iter().copied().map(CardId::new).collect(),
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddLiuhong {
                player,
                actor,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddLiuhong {
                player,
                actor,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddXiahouxuan { player, action, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ResolveDddXiahouxuan {
                    player,
                    action,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ResolveDddZhangkai {
                player,
                actor,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddZhangkai {
                player,
                actor,
                card,
                cards: self.event_cards.iter().copied().map(CardId::new).collect(),
                amount: self.ctx.event_amount,
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ResolveDddLiangxi {
                player,
                actor,
                card,
                action,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ResolveDddLiangxi {
                player,
                actor,
                card,
                cards: self.event_cards.iter().copied().map(CardId::new).collect(),
                action,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseOtherDiscardTargetCard { player, scope, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseOtherDiscardTargetCard {
                    player,
                    scope,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseAddSkillBoundCardTarget { player, card, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseAddSkillBoundCardTarget {
                    player,
                    card,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseDiscardMatchingEquipColorAddSkill {
                player,
                target,
                skill_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseDiscardMatchingEquipColorAddSkill {
                player,
                target,
                skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseCurrentCardUseEnhancements {
                player,
                card,
                helper_skill_def,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseCurrentCardUseEnhancements {
                player,
                card,
                helper_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ChooseDrawOrDiscardAddSkill {
                player,
                draw_skill_def,
                discard_skill_def,
                three_way,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::ChooseDrawOrDiscardAddSkill {
                player,
                draw_skill_def,
                discard_skill_def,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
                three_way,
            },
            EffectRequest::ChooseDiscardedBlackBasicAsBingliang { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseDiscardedBlackBasicAsBingliang {
                    player,
                    cards: self
                        .event_cards
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage {
                player, target, ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::DiscardSelectedRecordUniqueCategoryTarget {
                player,
                target,
                card,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::DiscardSelectedRecordUniqueCategoryTarget {
                    player,
                    target,
                    card,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
                player,
                target,
                toggle_link,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
                    player,
                    target,
                    toggle_link,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseRevealOtherHandCardRememberCategory { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseRevealOtherHandCardRememberCategory {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::DrawThenGiveExactRecordPhaseTarget {
                player,
                target,
                count,
                ready_to_give,
                ..
            } if self.ctx.skill_id != 0 => EffectRequest::DrawThenGiveExactRecordPhaseTarget {
                player,
                target,
                count,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
                ready_to_give,
            },
            EffectRequest::ClearPhaseGiftTarget { player, .. } if self.ctx.skill_id != 0 => {
                EffectRequest::ClearPhaseGiftTarget {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseGainResolvedShaOrShanThenOptionalUseSha {
                player,
                sha_card,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseGainResolvedShaOrShanThenOptionalUseSha {
                    player,
                    sha_card,
                    shan_cards: self
                        .event_cards
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    excluded_recipients: self
                        .event_targets
                        .iter()
                        .copied()
                        .map(engine_api::PlayerId::new)
                        .collect(),
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseGainAllResolvedShaShanThenColorRewards {
                player,
                sha_card,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseGainAllResolvedShaShanThenColorRewards {
                    player,
                    sha_card,
                    shan_cards: self
                        .event_cards
                        .iter()
                        .copied()
                        .map(engine_api::CardId::new)
                        .collect(),
                    excluded_recipients: self
                        .event_targets
                        .iter()
                        .copied()
                        .map(engine_api::PlayerId::new)
                        .collect(),
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage {
                player, amount, ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage {
                    player,
                    amount,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::RevealUniformHandDrawSelfChooseOthers { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::RevealUniformHandDrawSelfChooseOthers {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseCardUserOrAddTarget { player, card, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseCardUserOrAddTarget {
                    player,
                    card,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseMoveTargetEquipmentOrDraw { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseMoveTargetEquipmentOrDraw {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseMoveTargetEquipmentAndOrDraw { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseMoveTargetEquipmentAndOrDraw {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseDrawViewTwoGainOrMoveEquipment { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseDrawViewTwoGainOrMoveEquipment {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::GiveAllEquipmentChooseRecoverOrDamage { player, target, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::GiveAllEquipmentChooseRecoverOrDamage {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseHandCountDifferentTargetDrawOrRandomDiscard { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ChooseHandCountDifferentTargetDrawOrRandomDiscard {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw {
                player, ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw {
                    player,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors {
                player,
                target,
                ready_to_choose,
                ..
            } if self.ctx.skill_id != 0 => {
                EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors {
                    player,
                    target,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                    ready_to_choose,
                }
            }
            EffectRequest::ChoosePreventDamageOrLoseMaxHpIncreaseDamage {
                player,
                damage_id: 0,
                amount,
                ..
            } => EffectRequest::ChoosePreventDamageOrLoseMaxHpIncreaseDamage {
                player,
                damage_id: self.state.resolution.current_damage_id,
                amount,
                skill: engine_api::SkillId::new(self.ctx.skill_id),
            },
            EffectRequest::ForceOthersUseShaOnNearestOrLoseHp { player, .. }
                if self.ctx.skill_id != 0 =>
            {
                EffectRequest::ForceOthersUseShaOnNearestOrLoseHp {
                    player,
                    remaining: Vec::new(),
                    started: false,
                    skill: engine_api::SkillId::new(self.ctx.skill_id),
                }
            }
            req => req,
        });
        Ok(())
    }

    fn run_query(&mut self, req: QueryRequest) -> Result<QueryResponse, HostTrap> {
        if !self.caps.run_queries {
            return Err(HostTrap::CapabilityDenied {
                name: "run_queries".into(),
            });
        }
        let mut ctx = QueryContext::new();
        Ok(query(self.state, &req, &mut ctx))
    }
}

/// Execute a verified module and return collected effects.
///
/// If the module pauses on `choose{}` (HOST_CHOICE), the effects emitted before
/// the pause are returned but the pause is dropped — callers that support DSL
/// choice must use [`run_module`] instead.
pub fn run_module_for_effects(
    state: &GameState,
    module: &rule_vm::BytecodeModule,
    skill_id: u32,
    self_id: u32,
    event_player: u32,
    event_source: u32,
    event_card: u32,
    event_amount: u32,
    fuel: u32,
) -> Result<Vec<EffectRequest>, rule_vm::VmError> {
    Ok(run_module(
        state,
        module,
        skill_id,
        [self_id, event_player, event_source, event_card],
        event_amount,
        fuel,
    )?
    .effects)
}

/// Outcome of running a VM module: emitted effects plus an optional pause.
pub struct VmRunOutcome {
    /// Effects emitted before completion or pause.
    pub effects: Vec<EffectRequest>,
    /// Set when the module paused on `choose{}`.
    pub paused: Option<VmPause>,
}

/// A paused VM run awaiting a choice resolution.
pub struct VmPause {
    /// Serialized continuation frame pack.
    pub frames: Vec<u8>,
    /// Resume program counter.
    pub pc: u32,
    /// Fuel remaining for the resumed segment.
    pub fuel_remaining: u32,
    /// Option tokens offered.
    pub options: Vec<u32>,
}

/// Execute a module, surfacing a pause if it hits `choose{}`.
///
/// `ctx` = `[self_id, event_player, event_source, event_card]`.
pub fn run_module(
    state: &GameState,
    module: &rule_vm::BytecodeModule,
    skill_id: u32,
    ctx: [u32; 4],
    event_amount: u32,
    fuel: u32,
) -> Result<VmRunOutcome, rule_vm::VmError> {
    run_module_with_selected_cards(state, module, skill_id, ctx, event_amount, &[], fuel)
}

/// Execute a trigger module while preserving every physical card from the domain event.
pub fn run_module_with_event_cards(
    state: &GameState,
    module: &rule_vm::BytecodeModule,
    skill_id: u32,
    ctx: [u32; 4],
    event_amount: u32,
    event_cards: &[u32],
    event_targets: &[u32],
    event_nature: u8,
    event_card_kind: u8,
    event_card_flags: u8,
    event_move_target: PlayerId,
    event_move_mask: u8,
    fuel: u32,
) -> Result<VmRunOutcome, rule_vm::VmError> {
    let mut vm = rule_vm::RuleVm::new();
    let mut host = EngineVmHost::new(
        state,
        skill_id,
        ctx[0],
        ctx[1],
        ctx[2],
        ctx[3],
        event_amount,
    );
    host.event_cards = event_cards.to_vec();
    host.event_targets = event_targets.to_vec();
    host.event_nature = event_nature;
    host.event_card_kind = event_card_kind;
    host.event_card_flags = event_card_flags;
    host.event_move_target = event_move_target;
    host.event_move_mask = event_move_mask;
    match vm.execute(module, &mut host, fuel)? {
        rule_vm::ExecResult::Halted { .. } => Ok(VmRunOutcome {
            effects: host.effects,
            paused: None,
        }),
        rule_vm::ExecResult::Paused {
            continuation,
            options,
        } => Ok(VmRunOutcome {
            effects: host.effects,
            paused: Some(VmPause {
                frames: continuation.frames,
                pc: continuation.pc,
                fuel_remaining: continuation.fuel_remaining,
                options,
            }),
        }),
    }
}

/// Execute a module with declaratively selected active-skill card inputs.
pub fn run_module_with_selected_cards(
    state: &GameState,
    module: &rule_vm::BytecodeModule,
    skill_id: u32,
    ctx: [u32; 4],
    event_amount: u32,
    selected_cards: &[u32],
    fuel: u32,
) -> Result<VmRunOutcome, rule_vm::VmError> {
    run_module_with_selected_inputs(
        state,
        module,
        skill_id,
        ctx,
        event_amount,
        selected_cards,
        &[],
        fuel,
    )
}

/// Execute a module with declaratively selected active-skill card and player inputs.
pub fn run_module_with_selected_inputs(
    state: &GameState,
    module: &rule_vm::BytecodeModule,
    skill_id: u32,
    ctx: [u32; 4],
    event_amount: u32,
    selected_cards: &[u32],
    selected_targets: &[u32],
    fuel: u32,
) -> Result<VmRunOutcome, rule_vm::VmError> {
    let mut vm = rule_vm::RuleVm::new();
    let mut host = EngineVmHost::new(
        state,
        skill_id,
        ctx[0],
        ctx[1],
        ctx[2],
        ctx[3],
        event_amount,
    );
    host.selected_cards = selected_cards.to_vec();
    host.selected_targets = selected_targets.to_vec();
    match vm.execute(module, &mut host, fuel)? {
        rule_vm::ExecResult::Halted { .. } => Ok(VmRunOutcome {
            effects: host.effects,
            paused: None,
        }),
        rule_vm::ExecResult::Paused {
            continuation,
            options,
        } => Ok(VmRunOutcome {
            effects: host.effects,
            paused: Some(VmPause {
                frames: continuation.frames,
                pc: continuation.pc,
                fuel_remaining: continuation.fuel_remaining,
                options,
            }),
        }),
    }
}

/// Resume a paused VM module with the chosen token, returning further effects
/// and any subsequent pause.
pub fn resume_module(
    state: &GameState,
    module: &rule_vm::BytecodeModule,
    skill_id: u32,
    ctx: [u32; 4],
    frames: Vec<u8>,
    pc: u32,
    fuel: u32,
    chosen_token: u32,
) -> Result<VmRunOutcome, rule_vm::VmError> {
    let mut vm = rule_vm::RuleVm::new();
    let mut host = EngineVmHost::new(state, skill_id, ctx[0], ctx[1], ctx[2], ctx[3], 0);
    let cont = rule_vm::Continuation {
        id: 0,
        frames,
        pc,
        fuel_remaining: fuel,
        module_hash: module.logic_hash,
    };
    match vm.resume(module, &mut host, cont, chosen_token)? {
        rule_vm::ExecResult::Halted { .. } => Ok(VmRunOutcome {
            effects: host.effects,
            paused: None,
        }),
        rule_vm::ExecResult::Paused {
            continuation,
            options,
        } => Ok(VmRunOutcome {
            effects: host.effects,
            paused: Some(VmPause {
                frames: continuation.frames,
                pc: continuation.pc,
                fuel_remaining: continuation.fuel_remaining,
                options,
            }),
        }),
    }
}
