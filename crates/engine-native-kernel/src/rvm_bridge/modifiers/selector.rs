//! PA12 Batch 16：`VmModifierSelector` 卡牌／伤害属性选择器与动态数值来源扩展求值。

use super::VmModifierValueSource;
use crate::catalog::{CardKind, Catalog, EquipSlot, Faction, Suit};
use crate::event::DamageNature;
use crate::ids::{CardId, PlayerId, SkillDefId};
use crate::state::GameStateCore;

/// 修饰卡牌／角色／伤害属性选择器（对齐 `content-schema::PackageModifierEntry::selector`）。
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
    DamageNormal,
    DamageFire,
    DamageThunder,
}

pub(super) fn is_delayed_trick_kind(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Le | CardKind::Bing | CardKind::ShanDian | CardKind::HongShui
    )
}

pub(super) fn is_trick_kind(kind: CardKind) -> bool {
    kind.is_basic_trick_or_equip() && !kind.is_basic_kind() && !kind.is_equip()
}

pub(super) fn is_damage_trick_kind(kind: CardKind) -> bool {
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

/// 判断卡牌或伤害属性是否匹配 `VmModifierSelector`。
///
/// 当 `card <= 2` 且不属于角色手牌／装备区实体牌（或超出目录范围）时，
/// 兼容 `card == 0/1/2` 编码 `DamageNature`（`0 = Normal, 1 = Fire, 2 = Thunder`）。
pub fn matches_vm_modifier_selector(
    selector: VmModifierSelector,
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    card: CardId,
    kind: CardKind,
    nature: DamageNature,
) -> bool {
    let eff_nature = match kind {
        CardKind::LeiSha => DamageNature::Thunder,
        CardKind::FireSha => DamageNature::Fire,
        _ if nature != DamageNature::Normal => nature,
        _ if card == 1
            && !state.player(owner).hand.contains(&card)
            && !state.player(owner).equips.contains(&card) =>
        {
            DamageNature::Fire
        }
        _ if card == 2
            && !state.player(owner).hand.contains(&card)
            && !state.player(owner).equips.contains(&card) =>
        {
            DamageNature::Thunder
        }
        _ => nature,
    };
    let Some(def) = catalog.cards.get(card as usize) else {
        return match selector {
            VmModifierSelector::Any => true,
            VmModifierSelector::DamageNormal => eff_nature == DamageNature::Normal,
            VmModifierSelector::DamageFire => eff_nature == DamageNature::Fire,
            VmModifierSelector::DamageThunder => eff_nature == DamageNature::Thunder,
            _ => false,
        };
    };
    let suit = def.suit;
    let is_red = matches!(suit, Suit::Heart | Suit::Diamond);
    let is_black = matches!(suit, Suit::Spade | Suit::Club);
    let rank = def.rank;
    let is_sha = kind.is_sha_like();
    match selector {
        VmModifierSelector::Any => true,
        VmModifierSelector::Sha => is_sha,
        VmModifierSelector::Tao => kind == CardKind::Tao,
        VmModifierSelector::Bingliang => kind == CardKind::Bing,
        VmModifierSelector::Lebu => kind == CardKind::Le,
        VmModifierSelector::Basic => kind.is_basic_kind(),
        VmModifierSelector::NonBasic => !kind.is_basic_kind(),
        VmModifierSelector::Trick => is_trick_kind(kind),
        VmModifierSelector::Equip => kind.is_equip(),
        VmModifierSelector::Spade => suit == Suit::Spade,
        VmModifierSelector::Heart => suit == Suit::Heart,
        VmModifierSelector::Club => suit == Suit::Club,
        VmModifierSelector::Diamond => suit == Suit::Diamond,
        VmModifierSelector::Red => is_red,
        VmModifierSelector::Black => is_black,
        VmModifierSelector::Juedou => kind == CardKind::Duel,
        VmModifierSelector::BlackTrick => is_trick_kind(kind) && is_black,
        VmModifierSelector::DamageTrick => is_damage_trick_kind(kind),
        VmModifierSelector::ShaFire => is_sha && eff_nature == DamageNature::Fire,
        VmModifierSelector::Huogong => def.name == "huogong",
        VmModifierSelector::Jiu | VmModifierSelector::PhysicalJiu => kind == CardKind::Jiu,
        VmModifierSelector::ShaRed => is_sha && is_red,
        VmModifierSelector::Nanman => kind == CardKind::SavageAttack,
        VmModifierSelector::OrdinaryTrick => is_trick_kind(kind) && !is_delayed_trick_kind(kind),
        VmModifierSelector::ShaBlack => is_sha && is_black,
        VmModifierSelector::ShaElemental => is_sha && eff_nature != DamageNature::Normal,
        VmModifierSelector::EquipNonMount => matches!(
            kind.equip_slot(),
            Some(EquipSlot::Weapon | EquipSlot::Armor | EquipSlot::Treasure)
        ),
        VmModifierSelector::ArmorOrTreasure => {
            matches!(kind.equip_slot(), Some(EquipSlot::Armor | EquipSlot::Treasure))
        }
        VmModifierSelector::Wanjian => kind == CardKind::ArcheryAttack,
        VmModifierSelector::ShaNormal => is_sha && eff_nature == DamageNature::Normal,
        VmModifierSelector::ShaHeart => is_sha && suit == Suit::Heart,
        VmModifierSelector::DelayedTrick => is_delayed_trick_kind(kind),
        VmModifierSelector::Wuxie => kind == CardKind::Nullify,
        VmModifierSelector::Guohe => kind == CardKind::Dismantle,
        VmModifierSelector::Shunshou => kind == CardKind::Snatch,
        VmModifierSelector::ShaThunder => is_sha && eff_nature == DamageNature::Thunder,
        VmModifierSelector::HandRankLetter => matches!(rank, 1 | 11 | 12 | 13),
        VmModifierSelector::SoleHandCard => {
            state.player(owner).hand.len() == 1 && state.player(owner).hand.contains(&card)
        }
        VmModifierSelector::ShaDiamond => is_sha && suit == Suit::Diamond,
        VmModifierSelector::SkillTracked => false,
        VmModifierSelector::DamageCard => is_sha || is_damage_trick_kind(kind),
        VmModifierSelector::FaceUp => !state.player(owner).face_down,
        VmModifierSelector::DamageNormal => eff_nature == DamageNature::Normal,
        VmModifierSelector::DamageFire => {
            eff_nature == DamageNature::Fire
                || def.name == "huogong"
                || matches!(
                    kind,
                    CardKind::FireSha
                        | CardKind::Linghunzhihuo
                        | CardKind::Shenhuofeiya
                        | CardKind::Chiyuxi
                        | CardKind::Liuxinghuoyu
                )
        }
        VmModifierSelector::DamageThunder => {
            eff_nature == DamageNature::Thunder
                || matches!(
                    kind,
                    CardKind::LeiSha
                        | CardKind::ShanDian
                        | CardKind::Aozuzhilei
                        | CardKind::Jingleishan
                )
        }
    }
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
        matches_vm_modifier_selector(self, state, catalog, owner, card, kind, nature)
    }

    /// 面向非卡牌上下文（如摸牌数、手牌上限）判断持有者状态是否匹配。
    pub fn matches_owner(self, state: &GameStateCore, owner: PlayerId) -> bool {
        match self {
            Self::Any => true,
            Self::FaceUp => !state.player(owner).face_down,
            _ => true,
        }
    }

    /// 面向伤害结算上下文判断伤害属性是否匹配。
    pub fn matches_nature(self, nature: DamageNature) -> bool {
        match self {
            Self::Any => true,
            Self::DamageNormal | Self::ShaNormal => nature == DamageNature::Normal,
            Self::DamageFire | Self::ShaFire => nature == DamageNature::Fire,
            Self::DamageThunder | Self::ShaThunder => nature == DamageNature::Thunder,
            Self::ShaElemental => nature != DamageNature::Normal,
            _ => true,
        }
    }
}

fn owner_sha_used_count(state: &GameStateCore, owner: PlayerId) -> u32 {
    if let Some(&c) = state.vm_aux.flags.get(&(owner, "sha_used_count".to_string())) {
        c
    } else if state.turn.current == owner && state.turn.sha_used_play_phase {
        1 + state
            .vm_aux
            .flags
            .get(&(owner, "vm_sha_extra_used".to_string()))
            .copied()
            .unwrap_or(0)
            + u32::from(state.turn.land_sha_used)
    } else {
        0
    }
}

/// PA12 Batch 16 新增 21 项 `VmModifierValueSource` 变体求值。
pub(super) fn evaluate_batch16_value_source(
    src: VmModifierValueSource,
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    queried: Option<PlayerId>,
    skill_def: SkillDefId,
) -> Option<i32> {
    let p = state.player(owner);
    let wounded_count = || {
        state
            .alive_players()
            .filter(|&q| state.player(q).hp < state.player(q).max_hp as i32)
            .count()
    };
    let resolve_other = || -> Option<PlayerId> {
        queried
            .filter(|&q| q != owner && state.player(q).alive)
            .or_else(|| state.alive_players().find(|&q| q != owner))
    };
    let val = match src {
        VmModifierValueSource::AliveWoundedCountAtLeast(n) => {
            i32::from(wounded_count() >= n as usize)
        }
        VmModifierValueSource::QueriedPlayerMarkPositive(mark) => match queried {
            Some(q) => i32::from(
                state
                    .vm_aux
                    .marks
                    .get(&(q, mark))
                    .copied()
                    .unwrap_or(0)
                    > 0,
            ),
            None => i32::from(state.alive_players().any(|q| {
                state
                    .vm_aux
                    .marks
                    .get(&(q, mark))
                    .copied()
                    .unwrap_or(0)
                    > 0
            })),
        },
        VmModifierValueSource::AliveWoundedCountZeroOrAtLeastFour => {
            let w = wounded_count();
            i32::from(w == 0 || w >= 4)
        }
        VmModifierValueSource::QueriedPlayerLinkedFirstLivingSkillOwner => {
            let first_owner = state
                .alive_players()
                .find(|&q| state.has_skill(q, skill_def))
                .unwrap_or(owner);
            let q_linked = match queried {
                Some(q) => state.player(q).linked,
                None => state.alive_players().any(|q| state.player(q).linked),
            };
            i32::from(first_owner == owner && q_linked)
        }
        VmModifierValueSource::OwnerShaUsedExactlyOne => {
            i32::from(owner_sha_used_count(state, owner) == 1)
        }
        VmModifierValueSource::OwnerShaUsedExactlyTwo => {
            i32::from(owner_sha_used_count(state, owner) == 2)
        }
        VmModifierValueSource::OwnerQunOtherShu => {
            let owner_is_qun = state.effective_faction(catalog, owner) == Some(Faction::Qun);
            let other_is_shu = match queried.filter(|&q| q != owner) {
                Some(q) => state.effective_faction(catalog, q) == Some(Faction::Shu),
                None => state
                    .alive_players()
                    .any(|q| q != owner && state.effective_faction(catalog, q) == Some(Faction::Shu)),
            };
            i32::from(owner_is_qun && other_is_shu)
        }
        VmModifierValueSource::OwnerUniqueMaxHand => {
            let my_hand = p.hand.len();
            let unique_max = state
                .alive_players()
                .filter(|&q| q != owner)
                .all(|q| my_hand > state.player(q).hand.len());
            i32::from(unique_max)
        }
        VmModifierValueSource::OwnerTurnedOverDiffOther => {
            i32::from(resolve_other().is_some_and(|q| p.face_down != state.player(q).face_down))
        }
        VmModifierValueSource::QueriedPlayerLinkedOtherThanOwner => match queried {
            Some(q) => i32::from(q != owner && state.player(q).linked),
            None => i32::from(
                state
                    .alive_players()
                    .any(|q| q != owner && state.player(q).linked),
            ),
        },
        VmModifierValueSource::QueriedPlayerWoundedInOwnerAttackRangeOther => {
            let range = state.attack_range(catalog, owner);
            let check = |q: PlayerId| {
                q != owner
                    && state.player(q).hp < state.player(q).max_hp as i32
                    && state.computed_distance(catalog, owner, q) <= range
            };
            match queried {
                Some(q) => i32::from(check(q)),
                None => i32::from(state.alive_players().any(check)),
            }
        }
        VmModifierValueSource::QueriedPlayerOtherThanOwner => match queried {
            Some(q) => i32::from(q != owner),
            None => i32::from(state.alive_players().any(|q| q != owner)),
        },
        VmModifierValueSource::QueriedPlayerJudgeNonemptyOtherThanOwner => match queried {
            Some(q) => i32::from(q != owner && !state.player(q).judge.is_empty()),
            None => i32::from(
                state
                    .alive_players()
                    .any(|q| q != owner && !state.player(q).judge.is_empty()),
            ),
        },
        VmModifierValueSource::QueriedSourceEnemyOwnerAllyTargetOtherThanOwner => match queried {
            Some(q) => i32::from(q != owner),
            None => i32::from(state.alive_players().any(|q| q != owner)),
        },
        VmModifierValueSource::OwnerHandGtOther => {
            i32::from(resolve_other().is_some_and(|q| p.hand.len() > state.player(q).hand.len()))
        }
        VmModifierValueSource::OwnerHandLtOther => {
            i32::from(resolve_other().is_some_and(|q| p.hand.len() < state.player(q).hand.len()))
        }
        VmModifierValueSource::OwnerHandEquipGteOther => i32::from(resolve_other().is_some_and(|q| {
            let my_he = p.hand.len() + p.equips.len();
            let other_he = state.player(q).hand.len() + state.player(q).equips.len();
            my_he >= other_he
        })),
        VmModifierValueSource::OwnerShaDamageDealtZeroThisTurn => i32::from(
            p.turn_damage == 0
                && state
                    .vm_aux
                    .flags
                    .get(&(owner, "sha_damage_dealt_turn".to_string()))
                    .copied()
                    .unwrap_or(0)
                    == 0,
        ),
        VmModifierValueSource::OwnerMaxHandBeforeSkillFiresThisTurn => {
            let my_hand = p.hand.len();
            let max_hand = state
                .alive_players()
                .map(|q| state.player(q).hand.len())
                .max()
                .unwrap_or(0);
            let name = catalog
                .skills
                .get(skill_def as usize)
                .map(|d| d.name)
                .unwrap_or("");
            i32::from(my_hand == max_hand && state.vm_aux.once_count(owner, 1, name) == 0)
        }
        VmModifierValueSource::OwnerLinkedDiffOther => {
            i32::from(resolve_other().is_some_and(|q| p.linked != state.player(q).linked))
        }
        VmModifierValueSource::OwnerTurnDamageDealt => p.turn_damage as i32,
        _ => return None,
    };
    Some(val)
}
