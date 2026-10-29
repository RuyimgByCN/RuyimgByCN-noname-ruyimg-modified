//! Query operations for `matches_kind_and_suit` and related helpers.

use super::*;

impl CardSelector {
    pub(crate) fn matches_kind_and_suit(self, kind: CardKind, suit: Suit) -> bool {
        let basic = kind.is_basic();
        match self {
            Self::Any => true,
            Self::Sha => kind.is_sha(),
            Self::Tao => kind == CardKind::Tao,
            Self::Bingliang => kind == CardKind::TrickBingliang,
            Self::Basic => basic,
            Self::NonBasic => !basic,
            Self::Trick => kind.is_trick(),
            Self::Equip => kind.is_equip(),
            Self::Spade => suit == Suit::Spade,
            Self::Heart => suit == Suit::Heart,
            Self::Club => suit == Suit::Club,
            Self::Diamond => suit == Suit::Diamond,
            Self::Red => matches!(suit, Suit::Heart | Suit::Diamond),
            Self::Black => matches!(suit, Suit::Spade | Suit::Club),
            Self::Juedou => kind == CardKind::TrickJuedou,
            Self::BlackTrick => kind.is_trick() && matches!(suit, Suit::Spade | Suit::Club),
            Self::DamageTrick => kind.is_damage_trick(),
            Self::ShaFire => kind == CardKind::ShaFire,
            Self::Huogong => kind == CardKind::TrickHuogong,
            Self::Jiu => kind == CardKind::Jiu,
            Self::PhysicalJiu => kind == CardKind::Jiu,
            Self::ShaRed => kind.is_sha() && matches!(suit, Suit::Heart | Suit::Diamond),
            Self::Nanman => kind == CardKind::TrickNanman,
            Self::OrdinaryTrick => kind.is_trick() && !kind.is_delayed_trick(),
            Self::ShaBlack => kind.is_sha() && matches!(suit, Suit::Spade | Suit::Club),
            Self::ShaElemental => matches!(kind, CardKind::ShaFire | CardKind::ShaThunder),
            Self::EquipNonMount => {
                matches!(
                    kind,
                    CardKind::Weapon | CardKind::Armor | CardKind::Treasure
                )
            }
            Self::ArmorOrTreasure => matches!(kind, CardKind::Armor | CardKind::Treasure),
            Self::Wanjian => kind == CardKind::TrickWanjian,
            Self::ShaNormal => kind == CardKind::Sha,
            Self::ShaHeart => kind.is_sha() && suit == Suit::Heart,
            Self::DelayedTrick => kind.is_delayed_trick(),
            Self::Wuxie => kind == CardKind::TrickWuxie,
            Self::Guohe => kind == CardKind::TrickGuohe,
            Self::Shunshou => kind == CardKind::TrickShunshou,
            Self::ShaThunder => kind == CardKind::ShaThunder,
            Self::ShaDiamond => kind.is_sha() && suit == Suit::Diamond,
            Self::DamageCard => kind.is_sha() || kind.is_damage_trick(),
            Self::Lebu => kind == CardKind::TrickLe,
            Self::HandRankLetter | Self::SoleHandCard | Self::SkillTracked | Self::FaceUp => false,
        }
    }

    pub(crate) fn matches_physical_card(
        self,
        state: &GameState,
        player: PlayerId,
        card: &Card,
        suit: Suit,
    ) -> bool {
        match self {
            Self::SkillTracked => false,
            Self::HandRankLetter => {
                card.owner == player
                    && state
                        .zones
                        .get(card.zone)
                        .is_some_and(|zone| zone.kind == ZoneKind::Hand)
                    && matches!(card.rank, 1 | 11 | 12 | 13)
            }
            Self::SoleHandCard => {
                card.owner == player
                    && state.hand_count(player) == 1
                    && state
                        .zones
                        .get(card.zone)
                        .is_some_and(|zone| zone.kind == ZoneKind::Hand)
            }
            Self::FaceUp => card.face_up,
            _ => self.matches_kind_and_suit(card.kind, suit),
        }
    }

    pub(crate) fn matches_card(
        self,
        physical_kind: CardKind,
        effective_kind: CardKind,
        effective_suit: Suit,
    ) -> bool {
        match self {
            Self::PhysicalJiu => physical_kind == CardKind::Jiu,
            _ => self.matches_kind_and_suit(effective_kind, effective_suit),
        }
    }

    pub(crate) fn tag(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Sha => "sha",
            Self::Tao => "tao",
            Self::Bingliang => "bingliang",
            Self::Basic => "basic",
            Self::NonBasic => "non_basic",
            Self::Trick => "trick",
            Self::Equip => "equip",
            Self::Spade => "spade",
            Self::Heart => "heart",
            Self::Club => "club",
            Self::Diamond => "diamond",
            Self::Red => "red",
            Self::Black => "black",
            Self::Juedou => "juedou",
            Self::BlackTrick => "black_trick",
            Self::DamageTrick => "damage_trick",
            Self::ShaFire => "sha_fire",
            Self::Huogong => "huogong",
            Self::Jiu => "jiu",
            Self::PhysicalJiu => "physical_jiu",
            Self::ShaRed => "sha_red",
            Self::Nanman => "nanman",
            Self::OrdinaryTrick => "ordinary_trick",
            Self::ShaBlack => "sha_black",
            Self::ShaElemental => "sha_elemental",
            Self::EquipNonMount => "equip_non_mount",
            Self::ArmorOrTreasure => "armor_or_treasure",
            Self::Wanjian => "wanjian",
            Self::ShaNormal => "sha_normal",
            Self::ShaHeart => "sha_heart",
            Self::DelayedTrick => "delayed_trick",
            Self::Wuxie => "wuxie",
            Self::Guohe => "guohe",
            Self::Shunshou => "shunshou",
            Self::ShaThunder => "sha_thunder",
            Self::HandRankLetter => "hand_rank_letter",
            Self::SoleHandCard => "sole_hand_card",
            Self::ShaDiamond => "sha_diamond",
            Self::SkillTracked => "skill_tracked",
            Self::DamageCard => "damage_card",
            Self::Lebu => "lebu",
            Self::FaceUp => "face_up",
        }
    }
}
