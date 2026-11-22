//! L11 太虚幻境（`taixuhuanjing`）完整战役模式数据与原生会话运行时（206 `L11:modes:*` 全部覆盖）。
//!
//! 模块划分：
//! - [`catalog`]：`moveEffect` 效果枚举、`effectPack`、`buffPack`、`cardRank`、侍灵档案与升级经验表；
//! - [`skill_rank`]：625 项 `txhjPack.skillRank` 稀有度与定价表；
//! - [`universal`]：`gainPack`（22 项）与 `changePack`（5 商店 + 19 奇遇）定义；
//! - [`exams_a`] / [`exams_b`]：`exam1..4` 共 230 道完整题库与真值选项；
//! - [`seasons`] / [`season_events_a`] / [`season_events_b`] / [`chapter_refs_a`] / [`chapter_refs_b`]：
//!   8 个赛季 × 52 章节 `procedure` 与 313 项去重 DLC 战斗/抉择/事件定义；
//! - [`session_types`] / [`session`]：`TaixuModeSession` 状态机、商店/答题/奇遇/抉择交互、
//!   `Engine::new_taixu_versus` 原生实局桥接、战后结算与完整快照恢复。

pub mod catalog;
pub mod chapter_refs_a;
pub mod chapter_refs_b;
pub mod exams_a;
pub mod exams_b;
pub mod season_events_a;
pub mod season_events_b;
pub mod seasons;
pub mod session;
pub mod session_battle;
pub mod session_shop;
pub mod session_types;
pub mod skill_rank;
pub mod universal;

pub use catalog::{
    taixu_max_exp_for_level, TaixuBuffStoreSpec, TaixuCardCategory, TaixuCardRankSpec,
    TaixuEffectKind, TaixuEffectPackSpec, TaixuEffectSpec, TaixuServantActionSpec,
    TaixuServantFullSpec, TaixuServantSkillDescSpec, L11_TAIXU_BUFF_STORE, L11_TAIXU_CARD_RANK,
    L11_TAIXU_EFFECT_PACK, L11_TAIXU_LEVEL_XP_THRESHOLDS, L11_TAIXU_SERVANT_SPECS,
};
pub use exams_a::{TaixuExamQuestionSpec, L11_TAIXU_EXAMS_1_2};
pub use exams_b::L11_TAIXU_EXAMS_3_4;
pub use seasons::{
    get_dlc_event_by_idx, L11_TAIXU_CHAPTER_EVENT_REFS_A, L11_TAIXU_CHAPTER_EVENT_REFS_B,
    TaixuBattleTarget, TaixuChapterEventRef, TaixuChapterSpec, TaixuChooseBtnSpec,
    TaixuDlcEventSpec, TaixuNpcSpec, TaixuSeasonSpec, TaixuSpoilSpec, L11_TAIXU_SEASONS,
};
pub use session::{
    find_exam_by_id, find_universal_event, taixu_majun_upgrade_name, TaixuModeSession,
};
pub use session_types::{
    TaixuButtonOptionView, TaixuChooseOptionView, TaixuLevelUpChoice, TaixuOwnedEffect,
    TaixuOwnedSubButton, TaixuPendingBattleSpec, TaixuPendingInteraction, TaixuRunScore,
    TaixuShopCategory, TaixuShopStock, TaixuSlotNode, TaixuSplitMix64,
};
pub use skill_rank::{TaixuSkillRankSpec, L11_TAIXU_SKILL_RANK};
pub use universal::{
    TaixuBtnResult, TaixuButtonSpec, TaixuUniversalEventSpec, L11_TAIXU_CHANGE_PACK,
    L11_TAIXU_GAIN_PACK,
};
