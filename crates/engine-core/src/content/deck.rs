//! Standard deck composition for identity mode (M1).
//!
//! The deck can be built from either the hardcoded specs or from a JSON
//! config file (§48). The JSON config is embedded at compile time via
//! `include_str!` so the engine core has no filesystem dependency (§2.1).

use crate::state::CardKind;
use engine_api::Suit;
use serde::{Deserialize, Serialize};

/// Equipment slot kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipSlot {
    /// Weapon.
    Weapon,
    /// Armor.
    Armor,
    /// +1 horse (defensive): others' distance to you +1.
    HorseDef,
    /// -1 horse (offensive): your distance to others -1.
    HorseOff,
    /// Treasure.
    Treasure,
}

/// Spec for one physical card in the deck before instance allocation.
#[derive(Debug, Clone, Copy)]
pub struct DeckCardSpec {
    /// Definition id.
    pub def: u32,
    /// Kind.
    pub kind: CardKind,
    /// Suit.
    pub suit: Suit,
    /// Rank 1–13.
    pub rank: u8,
    /// Weapon attack range (0 if N/A).
    pub weapon_range: u8,
    /// Equip slot if equipment.
    pub equip: Option<EquipSlot>,
}

/// Build M1 standard-ish deck (basics + equips + common tricks).
pub fn build_standard_deck_specs() -> Vec<DeckCardSpec> {
    let mut v = Vec::new();
    // Basics by suit (classic-ish density)
    for rank in 1..=13u8 {
        v.push(basic(10 + rank as u32, CardKind::Sha, Suit::Spade, rank));
    }
    for rank in 1..=5u8 {
        v.push(basic(30 + rank as u32, CardKind::Tao, Suit::Heart, rank));
    }
    for rank in 6..=13u8 {
        v.push(basic(40 + rank as u32, CardKind::Sha, Suit::Heart, rank));
    }
    for rank in 1..=13u8 {
        v.push(basic(50 + rank as u32, CardKind::Shan, Suit::Club, rank));
    }
    for rank in 1..=9u8 {
        v.push(basic(70 + rank as u32, CardKind::Sha, Suit::Diamond, rank));
    }
    for rank in 10..=13u8 {
        v.push(basic(80 + rank as u32, CardKind::Jiu, Suit::Diamond, rank));
    }
    // 火杀 / 雷杀（属性杀，与藤甲/铁索互动）
    v.push(basic(90, CardKind::ShaFire, Suit::Heart, 4));
    v.push(basic(91, CardKind::ShaFire, Suit::Heart, 7));
    v.push(basic(92, CardKind::ShaFire, Suit::Diamond, 5));
    v.push(basic(93, CardKind::ShaThunder, Suit::Spade, 5));
    v.push(basic(94, CardKind::ShaThunder, Suit::Spade, 6));
    v.push(basic(95, CardKind::ShaThunder, Suit::Club, 5));
    v.push(basic(96, CardKind::ShaThunder, Suit::Club, 6));

    // Weapons
    v.push(weapon(100, Suit::Spade, 6, 2));
    v.push(weapon(101, Suit::Spade, 2, 2));
    v.push(weapon(102, Suit::Diamond, 5, 3));
    v.push(weapon(103, Suit::Spade, 12, 3));
    v.push(weapon(104, Suit::Diamond, 12, 4));
    v.push(weapon(105, Suit::Heart, 5, 5));

    // Armor: 八卦阵 (110)、藤甲 (111)
    v.push(equip(
        110,
        CardKind::Armor,
        Suit::Spade,
        2,
        0,
        EquipSlot::Armor,
    ));
    v.push(equip(
        111,
        CardKind::Armor,
        Suit::Club,
        2,
        0,
        EquipSlot::Armor,
    )); // 藤甲

    // Horses
    v.push(equip(
        120,
        CardKind::HorseOff,
        Suit::Heart,
        5,
        0,
        EquipSlot::HorseOff,
    ));
    v.push(equip(
        121,
        CardKind::HorseOff,
        Suit::Diamond,
        13,
        0,
        EquipSlot::HorseOff,
    ));
    v.push(equip(
        122,
        CardKind::HorseDef,
        Suit::Spade,
        5,
        0,
        EquipSlot::HorseDef,
    ));
    v.push(equip(
        123,
        CardKind::HorseDef,
        Suit::Club,
        5,
        0,
        EquipSlot::HorseDef,
    ));

    // 无中生有
    for (i, suit, rank) in [
        (130u32, Suit::Heart, 7u8),
        (131, Suit::Heart, 8),
        (132, Suit::Heart, 9),
        (133, Suit::Heart, 11),
    ] {
        v.push(trick(i, CardKind::TrickWuzhong, suit, rank));
    }
    // 过河拆桥
    for (i, suit, rank) in [
        (140u32, Suit::Spade, 3u8),
        (141, Suit::Spade, 4),
        (142, Suit::Spade, 12),
        (143, Suit::Club, 3),
        (144, Suit::Club, 4),
        (145, Suit::Heart, 12),
    ] {
        v.push(trick(i, CardKind::TrickGuohe, suit, rank));
    }
    // 顺手牵羊
    for (i, suit, rank) in [
        (150u32, Suit::Spade, 3u8),
        (151, Suit::Spade, 4),
        (152, Suit::Spade, 11),
        (153, Suit::Diamond, 3),
        (154, Suit::Diamond, 4),
    ] {
        v.push(trick(i, CardKind::TrickShunshou, suit, rank));
    }
    // 决斗
    for (i, suit, rank) in [
        (160u32, Suit::Spade, 1u8),
        (161, Suit::Club, 1),
        (162, Suit::Diamond, 1),
    ] {
        v.push(trick(i, CardKind::TrickJuedou, suit, rank));
    }
    // 南蛮 / 万箭
    v.push(trick(170, CardKind::TrickNanman, Suit::Spade, 7));
    v.push(trick(171, CardKind::TrickNanman, Suit::Spade, 13));
    v.push(trick(172, CardKind::TrickNanman, Suit::Club, 7));
    v.push(trick(180, CardKind::TrickWanjian, Suit::Heart, 1));
    // 桃园 / 五谷
    v.push(trick(190, CardKind::TrickTaoyuan, Suit::Heart, 1));
    v.push(trick(191, CardKind::TrickWugu, Suit::Heart, 3));
    v.push(trick(192, CardKind::TrickWugu, Suit::Heart, 4));

    // 延时锦囊
    v.push(trick(200, CardKind::TrickLe, Suit::Spade, 6));
    v.push(trick(201, CardKind::TrickLe, Suit::Heart, 6));
    v.push(trick(202, CardKind::TrickLe, Suit::Club, 6));
    v.push(trick(210, CardKind::TrickBingliang, Suit::Spade, 10));
    v.push(trick(211, CardKind::TrickBingliang, Suit::Club, 4));
    v.push(trick(220, CardKind::TrickShandian, Suit::Spade, 1));
    v.push(trick(221, CardKind::TrickShandian, Suit::Heart, 12));

    // 借刀 / 铁索
    v.push(trick(230, CardKind::TrickJiedao, Suit::Club, 12));
    v.push(trick(231, CardKind::TrickJiedao, Suit::Club, 13));
    for (i, suit, rank) in [
        (240u32, Suit::Spade, 11u8),
        (241, Suit::Spade, 12),
        (242, Suit::Club, 10),
        (243, Suit::Club, 11),
        (244, Suit::Club, 12),
        (245, Suit::Club, 13),
    ] {
        v.push(trick(i, CardKind::TrickTiesuo, suit, rank));
    }

    // 火攻
    for (i, suit, rank) in [
        (250u32, Suit::Heart, 2u8),
        (251, Suit::Heart, 3),
        (252, Suit::Heart, 4),
        (253, Suit::Diamond, 12),
    ] {
        v.push(trick(i, CardKind::TrickHuogong, suit, rank));
    }

    // 无懈可击
    for (i, suit, rank) in [
        (260u32, Suit::Spade, 11u8),
        (261, Suit::Club, 12),
        (262, Suit::Club, 13),
        (263, Suit::Diamond, 12),
    ] {
        v.push(trick(i, CardKind::TrickWuxie, suit, rank));
    }

    v
}

fn basic(def: u32, kind: CardKind, suit: Suit, rank: u8) -> DeckCardSpec {
    DeckCardSpec {
        def,
        kind,
        suit,
        rank,
        weapon_range: 0,
        equip: None,
    }
}

fn weapon(def: u32, suit: Suit, rank: u8, range: u8) -> DeckCardSpec {
    DeckCardSpec {
        def,
        kind: CardKind::Weapon,
        suit,
        rank,
        weapon_range: range,
        equip: Some(EquipSlot::Weapon),
    }
}

fn equip(
    def: u32,
    kind: CardKind,
    suit: Suit,
    rank: u8,
    weapon_range: u8,
    slot: EquipSlot,
) -> DeckCardSpec {
    DeckCardSpec {
        def,
        kind,
        suit,
        rank,
        weapon_range,
        equip: Some(slot),
    }
}

fn trick(def: u32, kind: CardKind, suit: Suit, rank: u8) -> DeckCardSpec {
    DeckCardSpec {
        def,
        kind,
        suit,
        rank,
        weapon_range: 0,
        equip: None,
    }
}

// ---- §48 Config-driven deck ----

/// The embedded base-deck JSON (compile-time, no filesystem dependency).
const BASE_DECK_JSON: &str = include_str!("../../../../content/core-rules/base-deck.json");

/// One card entry in the JSON deck config.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonCardEntry {
    def: u32,
    kind: String,
    suit: String,
    rank: u8,
    #[serde(default)]
    weapon_range: u8,
}

/// The JSON deck config root.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeckConfig {
    module_id: String,
    #[serde(default)]
    description: String,
    cards: Vec<JsonCardEntry>,
}

/// Parse a card kind string into CardKind.
fn parse_kind(s: &str) -> Option<CardKind> {
    match s {
        "sha" => Some(CardKind::Sha),
        "shan" => Some(CardKind::Shan),
        "tao" => Some(CardKind::Tao),
        "jiu" => Some(CardKind::Jiu),
        "sha_fire" => Some(CardKind::ShaFire),
        "sha_thunder" => Some(CardKind::ShaThunder),
        "weapon" => Some(CardKind::Weapon),
        "armor" => Some(CardKind::Armor),
        "horse_off" => Some(CardKind::HorseOff),
        "horse_def" => Some(CardKind::HorseDef),
        "treasure" => Some(CardKind::Treasure),
        "trick_wuzhong" => Some(CardKind::TrickWuzhong),
        "trick_guohe" => Some(CardKind::TrickGuohe),
        "trick_shunshou" => Some(CardKind::TrickShunshou),
        "trick_juedou" => Some(CardKind::TrickJuedou),
        "trick_nanman" => Some(CardKind::TrickNanman),
        "trick_wanjian" => Some(CardKind::TrickWanjian),
        "trick_taoyuan" => Some(CardKind::TrickTaoyuan),
        "trick_wugu" => Some(CardKind::TrickWugu),
        "trick_le" => Some(CardKind::TrickLe),
        "trick_bingliang" => Some(CardKind::TrickBingliang),
        "trick_shandian" => Some(CardKind::TrickShandian),
        "trick_jiedao" => Some(CardKind::TrickJiedao),
        "trick_tiesuo" => Some(CardKind::TrickTiesuo),
        "trick_huogong" => Some(CardKind::TrickHuogong),
        "trick_wuxie" => Some(CardKind::TrickWuxie),
        _ => None,
    }
}

/// Parse a suit string into Suit.
fn parse_suit(s: &str) -> Option<Suit> {
    match s {
        "spade" => Some(Suit::Spade),
        "heart" => Some(Suit::Heart),
        "club" => Some(Suit::Club),
        "diamond" => Some(Suit::Diamond),
        _ => None,
    }
}

/// Determine equip slot from card kind.
fn equip_slot_for(kind: CardKind) -> Option<EquipSlot> {
    match kind {
        CardKind::Weapon => Some(EquipSlot::Weapon),
        CardKind::Armor => Some(EquipSlot::Armor),
        CardKind::HorseOff => Some(EquipSlot::HorseOff),
        CardKind::HorseDef => Some(EquipSlot::HorseDef),
        CardKind::Treasure => Some(EquipSlot::Treasure),
        _ => None,
    }
}

/// Build deck specs from the embedded JSON config (§48: config-driven deck).
///
/// Returns `None` if the config is missing or malformed (caller falls back
/// to the hardcoded `build_standard_deck_specs`).
pub fn build_deck_from_config() -> Option<Vec<DeckCardSpec>> {
    let config: DeckConfig = serde_json::from_str(BASE_DECK_JSON).ok()?;
    let mut specs = Vec::with_capacity(config.cards.len());
    for entry in &config.cards {
        let kind = parse_kind(&entry.kind)?;
        let suit = parse_suit(&entry.suit)?;
        let equip = equip_slot_for(kind);
        specs.push(DeckCardSpec {
            def: entry.def,
            kind,
            suit,
            rank: entry.rank,
            weapon_range: entry.weapon_range,
            equip,
        });
    }
    if specs.is_empty() {
        None
    } else {
        Some(specs)
    }
}

/// Build deck specs, preferring JSON config, falling back to hardcoded.
///
/// This is the primary entry point for `bootstrap_match`. The config-driven
/// deck must produce identical card specs (same defs, types, suits, ranks) in
/// the same order as the hardcoded deck to preserve golden vector stability.
pub fn build_deck_specs() -> Vec<DeckCardSpec> {
    build_deck_from_config().unwrap_or_else(build_standard_deck_specs)
}

/// Guozhan deck: the identity deck plus the 12 guozhan equipment cards (ADR 0604).
///
/// The upstream guozhan pile (`mode/guozhan/src/info/pile.js`, 389 cards) also carries 19
/// guozhan-only tricks (gz_haolingtianxia, zhibi, …) that have no `CardKind` yet; migrating
/// them needs new card kinds (an ABI-level change) and is deferred. The equipment subset is
/// mapped to existing kinds so guozhan matches can draw the guozhan gear.
pub fn guozhan_deck_specs() -> Vec<DeckCardSpec> {
    let mut specs = build_deck_specs();
    // [name, suit, rank, kind, equip_slot] — from upstream pile.js.
    let gz_equip: &[(&str, Suit, u8, CardKind, EquipSlot)] = &[
        ("feilongduofeng", Suit::Spade, 2, CardKind::Weapon, EquipSlot::Weapon),
        ("feilongduofeng", Suit::Spade, 6, CardKind::Weapon, EquipSlot::Weapon),
        ("wuliu", Suit::Diamond, 6, CardKind::Weapon, EquipSlot::Weapon),
        ("sanjian", Suit::Diamond, 12, CardKind::Weapon, EquipSlot::Weapon),
        ("jilinqianyi", Suit::Spade, 6, CardKind::Weapon, EquipSlot::Weapon),
        ("minguangkai", Suit::Spade, 2, CardKind::Armor, EquipSlot::Armor),
        ("taipingyaoshu", Suit::Heart, 3, CardKind::Armor, EquipSlot::Armor),
        ("huxinjing", Suit::Club, 1, CardKind::Armor, EquipSlot::Armor),
        ("jingfanma", Suit::Heart, 3, CardKind::HorseOff, EquipSlot::HorseOff),
        ("yuxi", Suit::Club, 1, CardKind::Weapon, EquipSlot::Weapon),
        ("dinglanyemingzhu", Suit::Diamond, 4, CardKind::Weapon, EquipSlot::Weapon),
        ("liulongcanjia", Suit::Heart, 5, CardKind::Weapon, EquipSlot::Weapon),
        ("zhaoshu", Suit::Heart, 6, CardKind::Treasure, EquipSlot::Treasure),
    ];
    for (name, suit, rank, kind, slot) in gz_equip {
        let def = crate::content::guozhan_card_entries()
            .iter()
            .find(|c| c.name == *name)
            .map(|c| c.card_def_id)
            .unwrap_or(0);
        if def != 0 {
            specs.push(DeckCardSpec {
                def,
                kind: *kind,
                suit: *suit,
                rank: *rank,
                weapon_range: if *slot == EquipSlot::Weapon { 2 } else { 0 },
                equip: Some(*slot),
            });
        }
    }
    // Guozhan-only tricks (ADR 0604). Their settlements need new CardKinds (an ABI 642+
    // change, tracked in ABI-COORDINATION); for now they enter the deck as `Other` so they
    // are drawable, with the actual effect pending content work.
    let gz_tricks: &[(&str, Suit, u8)] = &[
        ("gz_haolingtianxia", Suit::Spade, 12),
        ("gz_kefuzhongyuan", Suit::Diamond, 1),
        ("gz_guguoanbang", Suit::Heart, 1),
        ("gz_wenheluanwu", Suit::Club, 12),
        ("zhibi", Suit::Club, 3),
        ("zhibi", Suit::Club, 4),
        ("chiling", Suit::Club, 3),
        ("huoshaolianying", Suit::Spade, 3),
        ("huoshaolianying", Suit::Club, 11),
        ("huoshaolianying", Suit::Heart, 12),
        ("lulitongxin", Suit::Spade, 12),
        ("lulitongxin", Suit::Club, 10),
        ("lianjunshengyan", Suit::Heart, 1),
        ("shuiyanqijunx", Suit::Heart, 13),
        ("shuiyanqijunx", Suit::Club, 12),
        ("yiyi", Suit::Diamond, 4),
        ("yiyi", Suit::Heart, 11),
        ("yuanjiao", Suit::Heart, 9),
    ];
    for (name, suit, rank) in gz_tricks {
        let def = crate::content::guozhan_card_entries()
            .iter()
            .find(|c| c.name == *name)
            .map(|c| c.card_def_id)
            .unwrap_or_else(|| {
                // Cards not in card_index (tricks) use a stable crc32 def.
                crc32_def(name)
            });
        // 以逸待劳/水淹七军/戮力同心/克复中原/固国安邦/火烧连营/联军盛宴 已有真实
        // CardKind 结算（ADR 0614-0620）；远交近攻（yuanjiao）直接映射既有
        // CardKind::TrickYuanjiao（目标摸1、使用者摸3，目标策略已过滤异势力）；
        // 文和乱武（gz_wenheluanwu）已有真实 CardKind 结算（ADR 0622）；
        // 号令天下（gz_haolingtianxia）已有真实 CardKind 结算（ADR 0623）；
        // 敕令（chiling）已有真实 CardKind 结算（ADR 0624）；
        // 其余国战锦囊暂以 CardKind::Other 占位。
        let kind = match *name {
            "yiyi" => CardKind::TrickYiyi,
            "shuiyanqijunx" => CardKind::TrickShuiyanqijunx,
            "lulitongxin" => CardKind::TrickLulitongxin,
            "gz_kefuzhongyuan" => CardKind::TrickKefuzhongyuan,
            "gz_guguoanbang" => CardKind::TrickGuguoanbang,
            "huoshaolianying" => CardKind::TrickHuoshaolianying,
            "lianjunshengyan" => CardKind::TrickLianjunshengyan,
            "gz_wenheluanwu" => CardKind::TrickWenheluanwu,
            "gz_haolingtianxia" => CardKind::TrickHaolingtianxia,
            "chiling" => CardKind::TrickChiling,
            "yuanjiao" => CardKind::TrickYuanjiao,
            _ => CardKind::Other,
        };
        specs.push(DeckCardSpec {
            def,
            kind,
            suit: *suit,
            rank: *rank,
            weapon_range: 0,
            equip: None,
        });
    }
    specs
}

/// Stable def id for a card name (used for guozhan tricks absent from card_index).
pub fn card_def_id_for_name(name: &str) -> u32 {
    crc32_def(name)
}

/// Whether a legacy/standard card definition is recastable (currently 铁索连环).
pub fn is_recastable_card_def(def: u32) -> bool {
    def == crc32_def("tiesuo")
}

fn crc32_def(name: &str) -> u32 {
    let mut hash = 0u32;
    for byte in name.bytes() {
        hash ^= u32::from(byte);
        for _ in 0..8 {
            hash = (hash >> 1) ^ (0xEDB8_8320 & (hash & 1).wrapping_neg());
        }
    }
    hash
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn config_deck_loads_successfully() {
        let specs = build_deck_from_config();
        assert!(specs.is_some(), "embedded base-deck.json must parse");
        let specs = specs.unwrap();
        assert!(!specs.is_empty(), "deck must have cards");
        // Must include basic card types.
        assert!(
            specs.iter().any(|s| s.kind == CardKind::Sha),
            "deck must have Sha"
        );
        assert!(
            specs.iter().any(|s| s.kind == CardKind::Shan),
            "deck must have Shan"
        );
        assert!(
            specs.iter().any(|s| s.kind == CardKind::Tao),
            "deck must have Tao"
        );
        // Must include weapons.
        assert!(
            specs.iter().any(|s| s.kind == CardKind::Weapon),
            "deck must have weapons"
        );
        // Must include tricks.
        assert!(
            specs.iter().any(|s| s.kind == CardKind::TrickWuxie),
            "deck must have wuxie"
        );
    }

    #[test]
    fn config_deck_matches_hardcoded_count() {
        // The config-driven deck must have the same card count as hardcoded.
        let config = build_deck_from_config().unwrap();
        let hardcoded = build_standard_deck_specs();
        assert_eq!(
            config.len(),
            hardcoded.len(),
            "config deck ({}) must match hardcoded deck ({}) card count",
            config.len(),
            hardcoded.len()
        );
    }

    #[test]
    fn config_deck_type_distribution_matches_hardcoded() {
        // The config and hardcoded deck must have the same count of each card kind.
        let config = build_deck_from_config().unwrap();
        let hardcoded = build_standard_deck_specs();
        let count_by_kind = |specs: &[DeckCardSpec]| -> Vec<(u8, usize)> {
            let mut m: std::collections::BTreeMap<u8, usize> = std::collections::BTreeMap::new();
            for s in specs {
                *m.entry(s.kind.tag()).or_insert(0) += 1;
            }
            m.into_iter().collect()
        };
        let config_dist = count_by_kind(&config);
        let hardcoded_dist = count_by_kind(&hardcoded);
        assert_eq!(
            config_dist, hardcoded_dist,
            "config and hardcoded deck must have identical type distribution"
        );
    }

    #[test]
    fn build_deck_specs_prefers_config() {
        // build_deck_specs() should use config when available.
        let specs = build_deck_specs();
        assert!(!specs.is_empty());
        // If config loaded, count matches config.
        if let Some(config) = build_deck_from_config() {
            assert_eq!(specs.len(), config.len());
        }
    }
}
