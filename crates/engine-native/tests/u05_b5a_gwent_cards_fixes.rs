//! U05-5a (`杀海拾遗 · gwent` 卡牌与卡牌技 80 项) 上游逐行还原 TDD 验证集。
//!
//! 覆盖：
//! - `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js`（43 张）
//! - `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/skill.js`（11 技）
//! - `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js`（26 张）
//! - `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/skill.js`（关联技与 `_gainspell`）

#[path = "u05_b5a_gwent_cards/common.rs"]
pub mod common;

#[path = "u05_b5a_gwent_cards/bronze_fixes.rs"]
mod bronze_fixes;

#[path = "u05_b5a_gwent_cards/gold_spells.rs"]
mod gold_spells;

#[path = "u05_b5a_gwent_cards/silver_spells.rs"]
mod silver_spells;

#[path = "u05_b5a_gwent_cards/adventure_cards.rs"]
mod adventure_cards;

#[path = "u05_b5a_gwent_cards/card_skills.rs"]
mod card_skills;

#[path = "u05_b5a_gwent_cards/ai_and_snapshot.rs"]
mod ai_and_snapshot;

#[path = "u05_b5a_gwent_cards/round2_fixes.rs"]
mod round2_fixes;
