//! PA12 Batch 14 / Batch 16：Rule-VM 通用静态修饰器（`VmModifierRule`）定义与求值。
//!
//! 承载 `content-schema::PackageModifierEntry` 编译产出的声明式修饰规则，
//! 在 `hand_limit`、`attack_range`、`computed_distance`、`DrawPhaseMachine`、
//! `play_sha`、`play_rules` 与 `play_card` 中按宿主状态统一求值。

mod selector;
pub use selector::VmModifierSelector;

use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::event::DamageNature;
use crate::ids::{CardId, PlayerId, SkillDefId};
use crate::state::GameStateCore;
use crate::vocabulary::generals::Gender;

/// 修饰目标（对齐 `content-schema::PackageModifierEntry::target`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmModifierTarget {
    HandLimit,
    HandLimitBaseMaxHp,
    ShaUseLimit,
    ShaExtraTargets,
    AttackRange,
    DistanceToOthers,
    DistanceFromOthers,
    CardDistanceUnlimited,
    CardUseUnlimited,
    DrawCount,
    CardTargetCancel,
    ShaUseCancel,
    CardUseCancel,
    HandCardUseResponseCancel,
    DyingPeachTargetCancel,
    GlobalHandLimit,
    GlobalDrawCount,
    GlobalShaUseLimit,
    GlobalAttackRange,
    GlobalDistanceTo,
    ShaIgnoredHandLimit,
    CardIgnoredHandLimit,
    CardTargetLimit,
    CardUseLimit,
    AttackRangeAbsent,
    AttackRangeFixed,
    DistanceFromIfHpGteTarget,
    DamageAmount,
    DamageTakenAmount,
    DamageTakenCancel,
    IgnoreArmor,
    CardUseUnrespondable,
    ShaUnrespondable,
    DyingSavePeachUsable,
    CardUsableOnSelf,
    CardSuit,
    CardName,
    CardNature,
    CardNumber,
    CardUsable,
    CardTargetEnabled,
    CardSavable,
    CardRespondable,
    CardUnrespondable,
    JudgmentRewriteHandEquipCostMark,
    CardDistanceUnlimitedTargetMark,
    CardDistanceUnlimitedSkillOnOwner,
    CardUseUnrespondableSkillOnOwner,
    ShaUseUnlimitedTargetMark,
    SkillConditionalModifier,
}

/// 修饰运算模式（对齐 `content-schema::PackageModifierEntry::mode`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmModifierMode {
    Add,
    Override,
    Max,
    Min,
    Multiply,
    Veto,
}

/// 动态数值来源（对齐 `content-schema::PackageModifierEntry::value_source`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmModifierValueSource {
    OwnerHp,
    OwnerHpMinusOne,
    OwnerHpGtTwo,
    OwnerHpLeTwo,
    OwnerMaxHp,
    OwnerLostHp,
    OwnerLostHpMinOne,
    OwnerLostHpMinusOne,
    OwnerLostHpAtLeastTwo,
    OwnerLostHpNegated,
    OwnerHandCount,
    OwnerHandEmpty,
    OwnerEquipmentCount,
    OwnerEquipCountFloor(u32),
    OwnerEquipCountAtLeast(u32),
    OwnerEquipCountExactlyOne,
    OwnerEquipCountHalfFloor,
    OwnerEquipCountOdd,
    OwnerEquipCountEven,
    FieldMountCardCountCeilHalf,
    OwnerSkillPlayPhaseFireCountBelow(u32),
    OwnerShield,
    LivingPlayerCount,
    AliveOtherCountCappedFour,
    AliveMountedPlayerCountMinOne,
    AliveFieldRankThirteenPlayerCount,
    RoundNumberCappedFive,
    DeadPlayerCount,
    WoundedPlayerCount,
    WoundedPlayerCountMinOne,
    UnwoundedPlayerCount,
    OwnerHealthy,
    GroupCount,
    SameGroupLivingCount,
    DifferentGroupLivingCount,
    MaxGroupLivingCount,
    OtherFemaleLivingCount,
    AliveFemaleCount,
    SameGroupCountNegated,
    DifferentGroupCount,
    DifferentGroupOthersCountNegated,
    OwnerJudgeCount,
    OwnerEmptyEquipSlotCount,
    OwnerNoMount,
    OwnerNoWeapon,
    OwnerArmorEmpty,
    OwnerDisabledEquipSlotCount,
    OwnerEnabledEquipSlotCount,
    OwnerHasEquipment,
    OwnerHasWeapon,
    OwnerHasArmorEquip,
    OwnerHasTreasure,
    OwnerHasHeartEquip,
    OwnerLinked,
    OwnerAllOthersInAttackRange,
    OwnerCurrentTurnAllOthersDistanceAtMostOne,
    OwnerShaUsedZero,
    OwnerShaUsedNonzero,
    OwnerDamagedNotTurnPlayer,
    OwnerHandCountGtHp,
    OwnerCompareWinThisTurn,
    OwnerUsedDistinctCardTypeCountThisTurn,
    OwnerShaHandCount,
    OwnerNonDamageHandTypeCount,
    OwnerUsedCardCountThisTurnNegated,
    OwnerCurrentTurnCardUseCount,
    OwnerPlayPhaseCardUseCountZero,
    OwnerSkillPileCount(u32),
    OwnerMarkCount(u32),
    OwnerSkillFlagValue(u32),
    OwnerRenPileCount,
    TaoSaveCancelBeyondDistance2,
    TaoSaveCancelNonDyingDuringOwnerTurn,
    AliveWoundedCountAtLeast(u8),
    QueriedPlayerMarkPositive(u32),
    AliveWoundedCountZeroOrAtLeastFour,
    QueriedPlayerLinkedFirstLivingSkillOwner,
    OwnerShaUsedExactlyOne,
    OwnerShaUsedExactlyTwo,
    OwnerQunOtherShu,
    OwnerUniqueMaxHand,
    OwnerTurnedOverDiffOther,
    QueriedPlayerLinkedOtherThanOwner,
    QueriedPlayerWoundedInOwnerAttackRangeOther,
    QueriedPlayerOtherThanOwner,
    QueriedPlayerJudgeNonemptyOtherThanOwner,
    QueriedSourceEnemyOwnerAllyTargetOtherThanOwner,
    OwnerHandGtOther,
    OwnerHandLtOther,
    OwnerHandEquipGteOther,
    OwnerShaDamageDealtZeroThisTurn,
    OwnerMaxHandBeforeSkillFiresThisTurn,
    OwnerLinkedDiffOther,
    OwnerTurnDamageDealt,
}

/// 单条已编译 VM 静态修饰规则。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmModifierRule {
    pub target: VmModifierTarget,
    pub mode: VmModifierMode,
    pub selector: Option<VmModifierSelector>,
    pub value: i32,
    pub value_source: Option<VmModifierValueSource>,
}

impl VmModifierValueSource {
    /// 根据当前状态计算动态修正值。
    pub fn evaluate(
        self,
        state: &GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        skill_def: SkillDefId,
    ) -> i32 {
        self.evaluate_for(state, catalog, owner, None, skill_def)
    }

    /// 带可选被查询角色（`queried`）的动态修正值求值。
    pub fn evaluate_for(
        self,
        state: &GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        queried: Option<PlayerId>,
        skill_def: SkillDefId,
    ) -> i32 {
        if let Some(v) =
            selector::evaluate_batch16_value_source(self, state, catalog, owner, queried, skill_def)
        {
            return v;
        }
        let p = state.player(owner);
        let lost_hp = (p.max_hp as i32 - p.hp).max(0);
        let disabled_mask = state
            .vm_aux
            .flags
            .get(&(owner, "disabled_equip_slots".to_string()))
            .copied()
            .unwrap_or(0);
        match self {
            Self::OwnerHp => p.hp.max(0),
            Self::OwnerHpMinusOne => (p.hp - 1).max(0),
            Self::OwnerHpGtTwo => i32::from(p.hp > 2),
            Self::OwnerHpLeTwo => i32::from(p.hp <= 2),
            Self::OwnerMaxHp => p.max_hp as i32,
            Self::OwnerLostHp => lost_hp,
            Self::OwnerLostHpMinOne => lost_hp.max(1),
            Self::OwnerLostHpMinusOne => (lost_hp - 1).max(0),
            Self::OwnerLostHpAtLeastTwo => i32::from(lost_hp >= 2),
            Self::OwnerLostHpNegated => -lost_hp,
            Self::OwnerHandCount => p.hand.len() as i32,
            Self::OwnerHandEmpty => i32::from(p.hand.is_empty()),
            Self::OwnerEquipmentCount => p.equips.len() as i32,
            Self::OwnerEquipCountFloor(n) => (p.equips.len() as i32).max(n as i32),
            Self::OwnerEquipCountAtLeast(n) => i32::from(p.equips.len() >= n as usize),
            Self::OwnerEquipCountExactlyOne => i32::from(p.equips.len() == 1),
            Self::OwnerEquipCountHalfFloor => (p.equips.len() / 2) as i32,
            Self::OwnerEquipCountOdd => i32::from(p.equips.len() % 2 == 1),
            Self::OwnerEquipCountEven => i32::from(p.equips.len() % 2 == 0),
            Self::FieldMountCardCountCeilHalf => {
                let mounts: usize = state
                    .alive_players()
                    .map(|q| {
                        state
                            .player(q)
                            .equips
                            .iter()
                            .filter(|&&c| {
                                matches!(
                                    catalog.card_kind(c),
                                    CardKind::HorsePlus | CardKind::HorseMinus
                                )
                            })
                            .count()
                    })
                    .sum();
                mounts.div_ceil(2) as i32
            }
            Self::OwnerSkillPlayPhaseFireCountBelow(n) => {
                let name = catalog
                    .skills
                    .get(skill_def as usize)
                    .map(|d| d.name)
                    .unwrap_or("");
                i32::from(state.vm_aux.once_count(owner, 1, name) < n as usize)
            }
            Self::OwnerShield => i32::from(p.hujia),
            Self::LivingPlayerCount => state.alive_players().count() as i32,
            Self::AliveOtherCountCappedFour => {
                state.alive_players().filter(|&q| q != owner).count().min(4) as i32
            }
            Self::AliveMountedPlayerCountMinOne => {
                state
                    .alive_players()
                    .filter(|&q| {
                        state.player(q).equips.iter().any(|&c| {
                            matches!(
                                catalog.card_kind(c),
                                CardKind::HorsePlus | CardKind::HorseMinus
                            )
                        })
                    })
                    .count()
                    .max(1) as i32
            }
            Self::AliveFieldRankThirteenPlayerCount => {
                state
                    .alive_players()
                    .filter(|&q| {
                        state
                            .player(q)
                            .equips
                            .iter()
                            .chain(state.player(q).judge.iter())
                            .any(|&c| catalog.cards.get(c as usize).is_some_and(|cd| cd.rank == 13))
                    })
                    .count() as i32
            }
            Self::RoundNumberCappedFive => state.turn.round.clamp(1, 5) as i32,
            Self::DeadPlayerCount => state.players.iter().filter(|pl| !pl.alive).count() as i32,
            Self::WoundedPlayerCount => state
                .alive_players()
                .filter(|&q| state.player(q).hp < state.player(q).max_hp as i32)
                .count() as i32,
            Self::WoundedPlayerCountMinOne => {
                state
                    .alive_players()
                    .filter(|&q| state.player(q).hp < state.player(q).max_hp as i32)
                    .count()
                    .max(1) as i32
            }
            Self::UnwoundedPlayerCount => state
                .alive_players()
                .filter(|&q| state.player(q).hp >= state.player(q).max_hp as i32)
                .count() as i32,
            Self::OwnerHealthy => i32::from(p.hp >= p.max_hp as i32),
            Self::GroupCount => {
                let mut mask = 0u32;
                for q in state.alive_players() {
                    if let Some(f) = state.effective_faction(catalog, q) {
                        mask |= 1u32 << (f as u8);
                    }
                }
                mask.count_ones() as i32
            }
            Self::SameGroupLivingCount => {
                let Some(my_f) = state.effective_faction(catalog, owner) else {
                    return 0;
                };
                state
                    .alive_players()
                    .filter(|&q| state.effective_faction(catalog, q) == Some(my_f))
                    .count() as i32
            }
            Self::DifferentGroupLivingCount | Self::DifferentGroupCount => {
                let my_f = state.effective_faction(catalog, owner);
                state
                    .alive_players()
                    .filter(|&q| q != owner && state.effective_faction(catalog, q) != my_f)
                    .count() as i32
            }
            Self::MaxGroupLivingCount => {
                let mut counts = [0i32; 8];
                for q in state.alive_players() {
                    if let Some(f) = state.effective_faction(catalog, q) {
                        let idx = (f as usize).min(7);
                        counts[idx] += 1;
                    }
                }
                counts.into_iter().max().unwrap_or(0)
            }
            Self::OtherFemaleLivingCount => state
                .alive_players()
                .filter(|&q| q != owner && state.effective_gender(q) == Gender::Female)
                .count() as i32,
            Self::AliveFemaleCount => state
                .alive_players()
                .filter(|&q| state.effective_gender(q) == Gender::Female)
                .count() as i32,
            Self::SameGroupCountNegated => {
                let Some(my_f) = state.effective_faction(catalog, owner) else {
                    return 0;
                };
                -(state
                    .alive_players()
                    .filter(|&q| state.effective_faction(catalog, q) == Some(my_f))
                    .count() as i32)
            }
            Self::DifferentGroupOthersCountNegated => {
                let my_f = state.effective_faction(catalog, owner);
                -(state
                    .alive_players()
                    .filter(|&q| q != owner && state.effective_faction(catalog, q) != my_f)
                    .count() as i32)
            }
            Self::OwnerJudgeCount => p.judge.len() as i32,
            Self::OwnerEmptyEquipSlotCount => (5usize.saturating_sub(p.equips.len())) as i32,
            Self::OwnerNoMount => i32::from(!p.equips.iter().any(|&c| {
                matches!(
                    catalog.card_kind(c),
                    CardKind::HorsePlus | CardKind::HorseMinus
                )
            })),
            Self::OwnerNoWeapon => i32::from(
                !p.equips
                    .iter()
                    .any(|&c| catalog.card_kind(c).equip_slot() == Some(EquipSlot::Weapon)),
            ),
            Self::OwnerArmorEmpty => i32::from(
                !p.equips
                    .iter()
                    .any(|&c| catalog.card_kind(c).equip_slot() == Some(EquipSlot::Armor)),
            ),
            Self::OwnerDisabledEquipSlotCount => disabled_mask.count_ones() as i32,
            Self::OwnerEnabledEquipSlotCount => {
                5i32.saturating_sub(disabled_mask.count_ones() as i32)
            }
            Self::OwnerHasEquipment => i32::from(!p.equips.is_empty()),
            Self::OwnerHasWeapon => i32::from(
                p.equips
                    .iter()
                    .any(|&c| catalog.card_kind(c).equip_slot() == Some(EquipSlot::Weapon)),
            ),
            Self::OwnerHasArmorEquip => i32::from(
                p.equips
                    .iter()
                    .any(|&c| catalog.card_kind(c).equip_slot() == Some(EquipSlot::Armor)),
            ),
            Self::OwnerHasTreasure => i32::from(
                p.equips
                    .iter()
                    .any(|&c| catalog.card_kind(c).equip_slot() == Some(EquipSlot::Treasure)),
            ),
            Self::OwnerHasHeartEquip => i32::from(
                p.equips
                    .iter()
                    .any(|&c| catalog.cards.get(c as usize).is_some_and(|cd| cd.suit == Suit::Heart)),
            ),
            Self::OwnerLinked => i32::from(p.linked),
            Self::OwnerAllOthersInAttackRange => {
                let range = state.attack_range(catalog, owner);
                let mut has_other = false;
                let all_in = state.alive_players().filter(|&q| q != owner).all(|q| {
                    has_other = true;
                    state.computed_distance(catalog, owner, q) <= range
                });
                i32::from(has_other && all_in)
            }
            Self::OwnerCurrentTurnAllOthersDistanceAtMostOne => {
                if state.turn.current != owner {
                    0
                } else {
                    let mut has_other = false;
                    let all_le_one = state.alive_players().filter(|&q| q != owner).all(|q| {
                        has_other = true;
                        state.computed_distance(catalog, owner, q) <= 1
                    });
                    i32::from(has_other && all_le_one)
                }
            }
            Self::OwnerShaUsedZero => {
                i32::from(state.turn.current != owner || !state.turn.sha_used_play_phase)
            }
            Self::OwnerShaUsedNonzero => {
                i32::from(state.turn.current == owner && state.turn.sha_used_play_phase)
            }
            Self::OwnerDamagedNotTurnPlayer => {
                i32::from(state.turn.current != owner && p.hp < p.max_hp as i32)
            }
            Self::OwnerHandCountGtHp => i32::from((p.hand.len() as i32) > p.hp.max(0)),
            Self::OwnerCompareWinThisTurn => state
                .vm_aux
                .flags
                .get(&(owner, "compare_win_turn".to_string()))
                .copied()
                .unwrap_or(0) as i32,
            Self::OwnerUsedDistinctCardTypeCountThisTurn => {
                let mask = state
                    .vm_aux
                    .flags
                    .get(&(owner, "play_cat_mask".to_string()))
                    .copied()
                    .unwrap_or(0);
                mask.count_ones() as i32
            }
            Self::OwnerShaHandCount => p
                .hand
                .iter()
                .filter(|&&c| catalog.card_kind(c).is_sha_like())
                .count() as i32,
            Self::OwnerNonDamageHandTypeCount => {
                let mut mask = 0u8;
                for &c in &p.hand {
                    let k = catalog.card_kind(c);
                    if !k.is_sha_like() && !selector::is_damage_trick_kind(k) {
                        if k.is_basic_kind() {
                            mask |= 1;
                        } else if k.is_equip() {
                            mask |= 4;
                        } else {
                            mask |= 2;
                        }
                    }
                }
                mask.count_ones() as i32
            }
            Self::OwnerUsedCardCountThisTurnNegated => {
                if state.turn.current == owner {
                    -(state.turn.cards_used_play_phase as i32)
                } else {
                    0
                }
            }
            Self::OwnerCurrentTurnCardUseCount => {
                if state.turn.current == owner {
                    state.turn.cards_used_play_phase as i32
                } else {
                    0
                }
            }
            Self::OwnerPlayPhaseCardUseCountZero => {
                i32::from(state.turn.current != owner || state.turn.cards_used_play_phase == 0)
            }
            Self::OwnerSkillPileCount(def_override) => {
                let key_name = if def_override != 0 {
                    catalog
                        .skills
                        .get(def_override as usize)
                        .map(|d| d.name.to_string())
                        .unwrap_or_else(|| def_override.to_string())
                } else {
                    catalog
                        .skills
                        .get(skill_def as usize)
                        .map(|d| d.name.to_string())
                        .unwrap_or_else(|| u32::from(skill_def).to_string())
                };
                state
                    .vm_aux
                    .piles
                    .get(&(owner, key_name))
                    .map(|v| v.len() as i32)
                    .unwrap_or(0)
            }
            Self::OwnerMarkCount(mark_override) => {
                let key_mark = if mark_override != 0 {
                    mark_override
                } else {
                    catalog
                        .skills
                        .get(skill_def as usize)
                        .and_then(|d| d.behavior.package_skill_id())
                        .unwrap_or(u32::from(skill_def))
                };
                state
                    .vm_aux
                    .marks
                    .get(&(owner, key_mark))
                    .copied()
                    .or_else(|| state.vm_aux.marks.get(&(owner, u32::from(skill_def))).copied())
                    .unwrap_or(0)
            }
            Self::OwnerSkillFlagValue(def_override) => {
                let key_def = if def_override != 0 {
                    def_override
                } else {
                    catalog
                        .skills
                        .get(skill_def as usize)
                        .and_then(|d| d.behavior.package_skill_id())
                        .unwrap_or(u32::from(skill_def))
                };
                state
                    .vm_aux
                    .flags
                    .get(&(owner, format!("skill:{key_def}")))
                    .copied()
                    .unwrap_or(0) as i32
            }
            Self::OwnerRenPileCount => state
                .vm_aux
                .piles
                .get(&(0, "__ren_pile__".to_string()))
                .map(|v| v.len() as i32)
                .unwrap_or(0),
            Self::TaoSaveCancelBeyondDistance2 | Self::TaoSaveCancelNonDyingDuringOwnerTurn => 1,
            _ => 0,
        }
    }
}

impl VmModifierRule {
    /// 计算本条规则在当前状态下的有效数值。
    pub fn effective_value(
        &self,
        state: &GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        skill_def: SkillDefId,
    ) -> i32 {
        self.effective_value_for(state, catalog, owner, None, skill_def)
    }

    /// 带可选被查询角色（`queried`）的有效数值求值。
    pub fn effective_value_for(
        &self,
        state: &GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        queried: Option<PlayerId>,
        skill_def: SkillDefId,
    ) -> i32 {
        match self.value_source {
            Some(src) => {
                let raw = src.evaluate_for(state, catalog, owner, queried, skill_def);
                if self.value < 0
                    && matches!(
                        src,
                        VmModifierValueSource::OwnerLostHpNegated
                            | VmModifierValueSource::SameGroupCountNegated
                            | VmModifierValueSource::DifferentGroupOthersCountNegated
                            | VmModifierValueSource::OwnerUsedCardCountThisTurnNegated
                    )
                {
                    raw
                } else {
                    raw.saturating_mul(self.value)
                }
            }
            None => self.value,
        }
    }

    /// 将规则应用到给定累加值上。
    pub fn apply_to(&self, current: i32, val: i32) -> i32 {
        match self.mode {
            VmModifierMode::Add => current.saturating_add(val),
            VmModifierMode::Override | VmModifierMode::Veto => val,
            VmModifierMode::Max => current.max(val),
            VmModifierMode::Min => current.min(val),
            VmModifierMode::Multiply => current.saturating_mul(val),
        }
    }
}

mod eval;
pub use eval::*;

