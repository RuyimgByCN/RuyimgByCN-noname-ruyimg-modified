//! Effect primitive implementations — refresh cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

/// 迂志 storage keys (per-skill keyed content storage).
const YUZHI_KEY_DRAWN_ROUND: u32 = 1;
/// Previous round's 迂志 draw count.
const YUZHI_KEY_DRAWN_LAST: u32 = 2;
/// This round's revealed-card name length mark.
const YUZHI_KEY_MARK: u32 = 3;
/// 保族 (clanbaozu) skill definition id — the skill 迂志 can remove.
const YUZHI_CLAN_BAOZU: u32 = 3_600_751_244;

/// 惬诣 per-suit first-use storage keys (suit tag offset).
const QIEYI_SUIT_KEY_BASE: u32 = 10;

/// 惬诣 (clanqieyi) skill definition id — 谏直 counts its fires this turn.
const JIANZHI_CLANQIEYI: u32 = 3_347_941_832;

/// Custom-continuation marker for ABI 455 `choose_target_draw_to_max_hp_cap_then_add_skill`.
pub(crate) const CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_ADD_SKILL_CUSTOM_CHOICE: u32 = 0xDCA5_4550;

mod refresh_skill;
pub(crate) use refresh_skill::*;
mod choose_targets_damage;
pub(crate) use choose_targets_damage::*;
mod choose_gain_target_card_alternate_zone_then_target_draw;
pub(crate) use choose_gain_target_card_alternate_zone_then_target_draw::*;
