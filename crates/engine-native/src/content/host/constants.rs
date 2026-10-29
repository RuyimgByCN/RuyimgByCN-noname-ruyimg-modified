pub(super) const MOYAN_ROUND: &str = "d3_moyan_round";

pub(super) const MOYAN_TARGET: &str = "d3_moyan_target";

pub(super) const MOYAN_COUNT: &str = "d3_moyan_count";

/// 蛊惑可声明的牌种表（基本牌 + 锦囊；顺序固定，Tag=下标）。
pub(super) const REGUHUO_KINDS: [crate::catalog::CardKind; 20] = [
    crate::catalog::CardKind::Sha,
    crate::catalog::CardKind::FireSha,
    crate::catalog::CardKind::LeiSha,
    crate::catalog::CardKind::Shan,
    crate::catalog::CardKind::Tao,
    crate::catalog::CardKind::Jiu,
    crate::catalog::CardKind::Wuzhong,
    crate::catalog::CardKind::Wugu,
    crate::catalog::CardKind::Taoyuan,
    crate::catalog::CardKind::SavageAttack,
    crate::catalog::CardKind::ArcheryAttack,
    crate::catalog::CardKind::Duel,
    crate::catalog::CardKind::Snatch,
    crate::catalog::CardKind::Dismantle,
    crate::catalog::CardKind::Chain,
    crate::catalog::CardKind::Le,
    crate::catalog::CardKind::Bing,
    crate::catalog::CardKind::ShanDian,
    crate::catalog::CardKind::Jiedao,
    crate::catalog::CardKind::Nullify,
];

pub(super) const HUANREN_PICK: &str = "huanren_pick";

/// 上游禁将九名；目录里能对上的只有这两名。孙策替换池不在目录，
/// 命中后改挂标准刘备（登记差异）。
pub(super) const HUANREN_BANNED: &[&str] = &["陆逊", "张春华"];

pub(super) const HUANREN_REPLACEMENT: &str = "刘备";

/// 白翼第 5 轮弃牌门。青翼用 bit 0..2，赤翼用 bit 3..5。
pub(super) const WING_BAI_5: u8 = 1 << 6;
