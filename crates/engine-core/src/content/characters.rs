//! Standard character ids plus the data-driven packaged roster.

use content_schema::{PackageCardEntry, PackageCharacterEntry};
use std::sync::LazyLock;

/// Skill definition ids shared with `content/standard/standard_m1.skill`.
pub mod skill_def {
    /// 奸雄
    pub const JIANXIONG: u32 = 1;
    /// 马术
    pub const MASHU: u32 = 2;
    /// 奇策/范围样例
    pub const QICAO: u32 = 3;
    /// 闭月
    pub const BIYUE: u32 = 4;
    /// 反馈
    pub const FANKUI: u32 = 5;
    /// 天妒
    pub const TIANDU: u32 = 6;
    /// 刚烈
    pub const GANGLIE: u32 = 7;
    /// 制衡
    pub const ZHIHENG: u32 = 10;
    /// 仁德
    pub const RENDE: u32 = 11;
    /// 苦肉
    pub const KUROU: u32 = 12;
    /// 青囊
    pub const QINGNANG: u32 = 13;
    /// 集智
    pub const JIZHI: u32 = 14;
    /// 突袭
    pub const TUXI: u32 = 15;
    /// 武圣
    pub const WUSHENG: u32 = 16;
    /// 龙胆
    pub const LONGDAN: u32 = 17;
    /// 无双
    pub const WUSHUANG: u32 = 18;
    /// 鬼才
    pub const GUICAI: u32 = 19;
    /// 英姿
    pub const YINGZI: u32 = 20;
    /// 连营
    pub const LIANYING: u32 = 21;
    /// 枭姬
    pub const XIAOJI: u32 = 22;
    /// 咆哮
    pub const PAOXIAO: u32 = 23;
    /// 倾国
    pub const QINGGUO: u32 = 24;
    /// 结姻
    pub const JIEYIN: u32 = 25;
    /// 遗计
    pub const YIJI: u32 = 26;
    /// 奇袭
    pub const QIXI: u32 = 27;
    /// 国色
    pub const GUOSE: u32 = 28;
    /// 裸衣
    pub const LUOYI: u32 = 29;
    /// 谦逊
    pub const QIANXUN: u32 = 30;
    /// 克己
    pub const KEJI: u32 = 31;
    /// 观星
    pub const GUANXING: u32 = 32;
    /// 空城
    pub const KONGCHENG: u32 = 3_681_607_931;
}

/// Character definition loaded from package data.
pub type CharacterDef = PackageCharacterEntry;

const STANDARD_CHARACTERS_JSON: &str = include_str!("../../../../content/standard/characters.json");

/// Compatibility view of the packaged standard roster.
///
/// The source of truth is `content/standard/characters.json`; gameplay reads the
/// same rows from `ContentRegistry` after package loading.
pub static STANDARD_CHARACTERS: LazyLock<Vec<CharacterDef>> = LazyLock::new(|| {
    serde_json::from_str(STANDARD_CHARACTERS_JSON).expect("standard characters.json must be valid")
});

/// Clone the packaged standard character rows for package construction.
pub fn standard_character_entries() -> Vec<CharacterDef> {
    STANDARD_CHARACTERS.clone()
}

const GUOZHAN_CHARACTERS_JSON: &str =
    include_str!("../../../../content/guozhan/guozhan_characters.json");

/// Packaged guozhan roster (256 gz_ characters, ADR 0604). Loaded into the content registry
/// by `ensure_guozhan_package` so `bootstrap_guozhan` deals from the guozhan pool rather
/// than the identity roster.
pub static GUOZHAN_CHARACTERS: LazyLock<Vec<CharacterDef>> = LazyLock::new(|| {
    serde_json::from_str(GUOZHAN_CHARACTERS_JSON)
        .expect("guozhan characters.json must be valid")
});

/// Clone the packaged guozhan character rows for package construction.
pub fn guozhan_character_entries() -> Vec<CharacterDef> {
    GUOZHAN_CHARACTERS.clone()
}

const GUOZHAN_CARDS_JSON: &str = include_str!("../../../../content/guozhan/guozhan_cards.json");

/// Packaged guozhan equipment card definitions (12 cards, ADR 0604). Loaded into the
/// content registry alongside the guozhan characters so guozhan decks can reference them.
pub static GUOZHAN_CARDS: LazyLock<Vec<PackageCardEntry>> = LazyLock::new(|| {
    serde_json::from_str(GUOZHAN_CARDS_JSON).expect("guozhan cards.json must be valid")
});

/// Clone the packaged guozhan card rows for package construction.
pub fn guozhan_card_entries() -> Vec<PackageCardEntry> {
    GUOZHAN_CARDS.clone()
}

/// Compatibility lookup in the embedded standard data.
pub fn character_by_id(id: u32) -> Option<&'static CharacterDef> {
    STANDARD_CHARACTERS
        .iter()
        .find(|character| character.id == id)
}
