//! PA12 Batch 14：Rule-VM 通用静态修饰器（`VmModifierRule`）定义与求值。
//!
//! 承载 `content-schema::PackageModifierEntry` 编译产出的声明式修饰规则，
//! 在 `hand_limit`、`attack_range`、`computed_distance`、`DrawPhaseMachine`、
//! `play_sha`、`play_rules` 与 `play_card` 中按宿主状态统一求值。

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

/// 修饰卡牌／角色选择器（对齐 `content-schema::PackageModifierEntry::selector`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmModifierSelector {
    Any,
    Sha,
    Tao,
    Bingliang,
    Lebu,
    Basic,
    NonBasic,
    Trick,
    Equip,
    Spade,
    Heart,
    Club,
    Diamond,
    Red,
    Black,
    Juedou,
    BlackTrick,
    DamageTrick,
    ShaFire,
    Huogong,
    Jiu,
    PhysicalJiu,
    ShaRed,
    Nanman,
    OrdinaryTrick,
    ShaBlack,
    ShaElemental,
    EquipNonMount,
    ArmorOrTreasure,
    Wanjian,
    ShaNormal,
    ShaHeart,
    DelayedTrick,
    Wuxie,
    Guohe,
    Shunshou,
    ShaThunder,
    HandRankLetter,
    SoleHandCard,
    ShaDiamond,
    SkillTracked,
    DamageCard,
    FaceUp,
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

fn is_delayed_trick_kind(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Le | CardKind::Bing | CardKind::ShanDian | CardKind::HongShui
    )
}

fn is_trick_kind(kind: CardKind) -> bool {
    kind.is_basic_trick_or_equip() && !kind.is_basic_kind() && !kind.is_equip()
}

fn is_damage_trick_kind(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Duel
            | CardKind::SavageAttack
            | CardKind::ArcheryAttack
            | CardKind::ShanDian
            | CardKind::HongShui
            | CardKind::Linghunzhihuo
            | CardKind::Shenhuofeiya
            | CardKind::Shatang
            | CardKind::Yuansuhuimie
            | CardKind::Aozuzhilei
            | CardKind::Jingleishan
            | CardKind::Chiyuxi
            | CardKind::Liuxinghuoyu
    )
}

impl VmModifierSelector {
    /// 判断卡牌是否匹配该选择器。
    pub fn matches_card(
        self,
        state: &GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        card: CardId,
        kind: CardKind,
        nature: DamageNature,
    ) -> bool {
        let Some(def) = catalog.cards.get(card as usize) else {
            return matches!(self, Self::Any);
        };
        let suit = def.suit;
        let is_red = matches!(suit, Suit::Heart | Suit::Diamond);
        let is_black = matches!(suit, Suit::Spade | Suit::Club);
        let rank = def.rank;
        let is_sha = kind.is_sha_like();
        let eff_nature = match kind {
            CardKind::LeiSha => DamageNature::Thunder,
            CardKind::FireSha => DamageNature::Fire,
            _ => nature,
        };
        match self {
            Self::Any => true,
            Self::Sha => is_sha,
            Self::Tao => kind == CardKind::Tao,
            Self::Bingliang => kind == CardKind::Bing,
            Self::Lebu => kind == CardKind::Le,
            Self::Basic => kind.is_basic_kind(),
            Self::NonBasic => !kind.is_basic_kind(),
            Self::Trick => is_trick_kind(kind),
            Self::Equip => kind.is_equip(),
            Self::Spade => suit == Suit::Spade,
            Self::Heart => suit == Suit::Heart,
            Self::Club => suit == Suit::Club,
            Self::Diamond => suit == Suit::Diamond,
            Self::Red => is_red,
            Self::Black => is_black,
            Self::Juedou => kind == CardKind::Duel,
            Self::BlackTrick => is_trick_kind(kind) && is_black,
            Self::DamageTrick => is_damage_trick_kind(kind),
            Self::ShaFire => is_sha && eff_nature == DamageNature::Fire,
            Self::Huogong => def.name == "huogong",
            Self::Jiu | Self::PhysicalJiu => kind == CardKind::Jiu,
            Self::ShaRed => is_sha && is_red,
            Self::Nanman => kind == CardKind::SavageAttack,
            Self::OrdinaryTrick => is_trick_kind(kind) && !is_delayed_trick_kind(kind),
            Self::ShaBlack => is_sha && is_black,
            Self::ShaElemental => is_sha && eff_nature != DamageNature::Normal,
            Self::EquipNonMount => matches!(
                kind.equip_slot(),
                Some(EquipSlot::Weapon | EquipSlot::Armor | EquipSlot::Treasure)
            ),
            Self::ArmorOrTreasure => {
                matches!(kind.equip_slot(), Some(EquipSlot::Armor | EquipSlot::Treasure))
            }
            Self::Wanjian => kind == CardKind::ArcheryAttack,
            Self::ShaNormal => is_sha && eff_nature == DamageNature::Normal,
            Self::ShaHeart => is_sha && suit == Suit::Heart,
            Self::DelayedTrick => is_delayed_trick_kind(kind),
            Self::Wuxie => kind == CardKind::Nullify,
            Self::Guohe => kind == CardKind::Dismantle,
            Self::Shunshou => kind == CardKind::Snatch,
            Self::ShaThunder => is_sha && eff_nature == DamageNature::Thunder,
            Self::HandRankLetter => matches!(rank, 1 | 11 | 12 | 13),
            Self::SoleHandCard => {
                state.player(owner).hand.len() == 1 && state.player(owner).hand.contains(&card)
            }
            Self::ShaDiamond => is_sha && suit == Suit::Diamond,
            Self::SkillTracked => false,
            Self::DamageCard => is_sha || is_damage_trick_kind(kind),
            Self::FaceUp => !state.player(owner).face_down,
        }
    }

    /// 面向非卡牌上下文（如摸牌数、手牌上限）判断持有者状态是否匹配。
    pub fn matches_owner(self, state: &GameStateCore, owner: PlayerId) -> bool {
        match self {
            Self::Any => true,
            Self::FaceUp => !state.player(owner).face_down,
            _ => true,
        }
    }
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
                    if !k.is_sha_like() && !is_damage_trick_kind(k) {
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
        match self.value_source {
            Some(src) => {
                let raw = src.evaluate(state, catalog, owner, skill_def);
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

/// 迭代角色当前拥有的所有已启用技能（含标记临时技能）的 `VmModifierRule`。
pub fn for_each_player_vm_modifier<F>(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut f: F,
) where
    F: FnMut(SkillDefId, &VmModifierRule),
{
    for ((owner, def), inst) in &state.skills {
        if *owner == player && inst.enabled {
            let rules = catalog.skills[*def as usize].behavior.vm_modifiers();
            for rule in rules {
                f(*def, rule);
            }
        }
    }
}

/// 评估角色手牌上限修正（`HandLimit`；`HandLimitBaseMaxHp` 已在 `hand_limit_base` 中结算）。
pub fn eval_hand_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut limit: i32,
) -> usize {
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::HandLimit
            && rule
                .selector
                .map_or(true, |s| s.matches_owner(state, player))
        {
            let val = rule.effective_value(state, catalog, player, def);
            limit = rule.apply_to(limit, val);
        }
    });
    limit.max(0) as usize
}

/// 评估角色攻击范围修正（`AttackRange`）。
pub fn eval_attack_range(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    base: u32,
) -> u32 {
    let mut cur = base as i32;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::AttackRange
            && rule
                .selector
                .map_or(true, |s| s.matches_owner(state, player))
        {
            let val = rule.effective_value(state, catalog, player, def);
            cur = rule.apply_to(cur, val);
        }
    });
    cur.max(0) as u32
}

/// 评估角色 `a -> b` 的距离修正（`DistanceToOthers` 与 `DistanceFromOthers`）。
pub fn eval_distance_total(
    state: &GameStateCore,
    catalog: &Catalog,
    a: PlayerId,
    b: PlayerId,
    base_total: i32,
) -> u32 {
    let mut cur = base_total;
    for_each_player_vm_modifier(state, catalog, a, |def, rule| {
        if rule.target == VmModifierTarget::DistanceToOthers
            && rule.selector.map_or(true, |s| s.matches_owner(state, a))
        {
            let val = rule.effective_value(state, catalog, a, def);
            cur = rule.apply_to(cur, val);
        }
    });
    for_each_player_vm_modifier(state, catalog, b, |def, rule| {
        if rule.target == VmModifierTarget::DistanceFromOthers
            && rule.selector.map_or(true, |s| s.matches_owner(state, b))
        {
            let val = rule.effective_value(state, catalog, b, def);
            cur = rule.apply_to(cur, val);
        }
    });
    cur.max(1) as u32
}

/// 评估摸牌阶段摸牌数修正（`DrawCount`）。
pub fn eval_draw_count(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    base_count: i32,
) -> i32 {
    let mut cur = base_count;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::DrawCount
            && rule
                .selector
                .map_or(true, |s| s.matches_owner(state, player))
        {
            let val = rule.effective_value(state, catalog, player, def);
            cur = rule.apply_to(cur, val);
        }
    });
    cur
}

/// 评估出牌阶段【杀】使用次数上限修正与是否无次数限制。
pub fn eval_sha_use_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: Option<CardId>,
    kind: CardKind,
    nature: DamageNature,
    base_limit: u32,
) -> (u32, bool) {
    let mut cur = base_limit as i32;
    let mut unlimited = false;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        let sel_ok = match (rule.selector, card) {
            (None, _) => true,
            (Some(s), Some(c)) => s.matches_card(state, catalog, player, c, kind, nature),
            (Some(s), None) => matches!(
                s,
                VmModifierSelector::Any
                    | VmModifierSelector::Sha
                    | VmModifierSelector::Basic
                    | VmModifierSelector::DamageCard
            ),
        };
        if !sel_ok {
            return;
        }
        match rule.target {
            VmModifierTarget::ShaUseLimit => {
                let val = rule.effective_value(state, catalog, player, def);
                if val >= 99 {
                    unlimited = true;
                } else {
                    cur = rule.apply_to(cur, val);
                }
            }
            VmModifierTarget::CardUseUnlimited => {
                if rule.effective_value(state, catalog, player, def) > 0 {
                    unlimited = true;
                }
            }
            _ => {}
        }
    });
    (cur.max(0) as u32, unlimited)
}

/// 评估【杀】额外目标上限加成（`ShaExtraTargets`）。
pub fn eval_sha_extra_targets(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
    kind: CardKind,
    nature: DamageNature,
) -> usize {
    let mut extra = 0i32;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::ShaExtraTargets
            && rule
                .selector
                .map_or(true, |s| s.matches_card(state, catalog, player, card, kind, nature))
        {
            let val = rule.effective_value(state, catalog, player, def);
            extra = rule.apply_to(extra, val);
        }
    });
    extra.max(0) as usize
}

/// 评估使用卡牌是否忽略距离限制（`CardDistanceUnlimited`）。
pub fn eval_card_distance_unlimited(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
    kind: CardKind,
    nature: DamageNature,
) -> bool {
    let mut unlimited = false;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::CardDistanceUnlimited
            && rule
                .selector
                .map_or(true, |s| s.matches_card(state, catalog, player, card, kind, nature))
            && rule.effective_value(state, catalog, player, def) > 0
        {
            unlimited = true;
        }
    });
    unlimited
}

/// 评估目标是否免疫某张牌的指向（`CardTargetCancel`）。
pub fn eval_card_target_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    target: PlayerId,
    card: CardId,
    kind: CardKind,
    nature: DamageNature,
) -> bool {
    let mut canceled = false;
    for_each_player_vm_modifier(state, catalog, target, |def, rule| {
        if rule.target == VmModifierTarget::CardTargetCancel
            && rule
                .selector
                .map_or(true, |s| s.matches_card(state, catalog, actor, card, kind, nature))
            && rule.effective_value(state, catalog, target, def) > 0
        {
            canceled = true;
        }
    });
    canceled
}

/// 评估角色是否被禁止使用某张牌（`ShaUseCancel` / `CardUseCancel` / `HandCardUseResponseCancel`）。
pub fn eval_card_use_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    card: CardId,
    kind: CardKind,
    nature: DamageNature,
) -> bool {
    let is_sha = kind.is_sha_like();
    let in_hand = state.player(actor).hand.contains(&card);
    let mut canceled = false;
    for_each_player_vm_modifier(state, catalog, actor, |def, rule| {
        let sel_ok = rule
            .selector
            .map_or(true, |s| s.matches_card(state, catalog, actor, card, kind, nature));
        if !sel_ok || rule.effective_value(state, catalog, actor, def) <= 0 {
            return;
        }
        match rule.target {
            VmModifierTarget::ShaUseCancel if is_sha => canceled = true,
            VmModifierTarget::CardUseCancel => canceled = true,
            VmModifierTarget::HandCardUseResponseCancel if in_hand => canceled = true,
            _ => {}
        }
    });
    if kind == CardKind::Tao && actor != state.turn.current && state.player(state.turn.current).alive
    {
        let cur = state.turn.current;
        for_each_player_vm_modifier(state, catalog, cur, |def, rule| {
            if rule.target == VmModifierTarget::DyingPeachTargetCancel
                && rule.value_source
                    == Some(VmModifierValueSource::TaoSaveCancelNonDyingDuringOwnerTurn)
                && rule.effective_value(state, catalog, cur, def) > 0
            {
                canceled = true;
            }
        });
    }
    canceled
}

/// 评估角色是否被禁止对濒死角色使用【桃】（`DyingPeachTargetCancel`）。
pub fn eval_dying_peach_target_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    dying_target: PlayerId,
    card: CardId,
) -> bool {
    let kind = catalog.card_kind(card);
    let mut canceled = false;
    for_each_player_vm_modifier(state, catalog, actor, |def, rule| {
        if rule.target == VmModifierTarget::DyingPeachTargetCancel
            && rule.selector.map_or(true, |s| {
                s.matches_card(state, catalog, actor, card, kind, DamageNature::Normal)
            })
            && match rule.value_source {
                Some(VmModifierValueSource::TaoSaveCancelBeyondDistance2) => {
                    kind == CardKind::Tao
                        && actor != dying_target
                        && state.computed_distance(catalog, actor, dying_target) > 2
                }
                Some(VmModifierValueSource::TaoSaveCancelNonDyingDuringOwnerTurn) => false,
                _ => rule.effective_value(state, catalog, actor, def) > 0,
            }
        {
            canceled = true;
        }
    });
    if actor != dying_target && kind == CardKind::Tao {
        for_each_player_vm_modifier(state, catalog, dying_target, |def, rule| {
            if rule.target == VmModifierTarget::DyingPeachTargetCancel
                && rule.value_source
                    != Some(VmModifierValueSource::TaoSaveCancelNonDyingDuringOwnerTurn)
                && rule.value_source != Some(VmModifierValueSource::TaoSaveCancelBeyondDistance2)
                && rule.effective_value(state, catalog, dying_target, def) > 0
            {
                canceled = true;
            }
        });
        let cur = state.turn.current;
        if actor != cur && state.player(cur).alive {
            for_each_player_vm_modifier(state, catalog, cur, |def, rule| {
                if rule.target == VmModifierTarget::DyingPeachTargetCancel
                    && rule.value_source
                        == Some(VmModifierValueSource::TaoSaveCancelNonDyingDuringOwnerTurn)
                    && rule.effective_value(state, catalog, cur, def) > 0
                {
                    canceled = true;
                }
            });
        }
    }
    canceled
}
