//! Effect primitive implementations — sp 2 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

/// Custom-choice marker for the 择一加技能 primitive.
pub(crate) const CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS_CUSTOM_CHOICE: u32 = 0x4353_4453;

/// Custom-choice marker for the 水淹七军 (guozhan) target choice.
pub(crate) const SHUIYANQIJUNX_CUSTOM_CHOICE: u32 = 0x5359_514A;

/// Custom-choice marker for the 克复中原 (guozhan) target choice.
pub(crate) const KEFUZHONGYUAN_CUSTOM_CHOICE: u32 = 0x4B46_5A59;

/// Custom-choice marker for the 固国安邦 (guozhan) staged settlement.
pub(crate) const GUGUOANBANG_CUSTOM_CHOICE: u32 = 0x4747_4142;

/// Custom-choice marker for the 联军盛宴 (guozhan) draw/recover split.
pub(crate) const LIANJUNSHENGYAN_CUSTOM_CHOICE: u32 = 0x4C4A_5359;

/// Custom-choice marker for the 文和乱武 (guozhan) staged settlement.
pub(crate) const WENHELUANWU_CUSTOM_CHOICE: u32 = 0x5745_4C57;

/// Custom-choice marker for the 号令天下 (guozhan) per-player option chain.
pub(crate) const HAOLINGTIANXIA_CUSTOM_CHOICE: u32 = 0x484C_5458;

/// Custom-choice marker for the 敕令 (guozhan) per-target option choice.
pub(crate) const CHILING_CUSTOM_CHOICE: u32 = 0x4348_4C4E;

pub(crate) const CATEGORY_GIFT_DAMAGE_PLUS_ONE_MODE: u32 = 0xDCD9_4980;

mod discard_all_judge;
pub(crate) use discard_all_judge::*;

mod choose_discard_basic_then_target_discard_equip_or_damage;
pub(crate) use choose_discard_basic_then_target_discard_equip_or_damage::*;
mod resolve_shoulijian;
pub(crate) use resolve_shoulijian::*;
mod damage_then_draw_selected_targets;
pub(crate) use damage_then_draw_selected_targets::*;
mod choose_category_then_target_give_matching_or_sha_unrespondable;
pub(crate) use choose_category_then_target_give_matching_or_sha_unrespondable::*;
