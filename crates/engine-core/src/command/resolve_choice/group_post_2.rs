//! Choice-continuation handlers (post 2).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

use super::*;

mod resolve_choose_card_user_or_add_target;
pub(crate) use resolve_choose_card_user_or_add_target::*;

mod resolve_choose_target_discard_weapon_range_hand_or_increase_damage;
pub(crate) use resolve_choose_target_discard_weapon_range_hand_or_increase_damage::*;
mod resolve_tuxi_gain_other_hands_replace_draw;
pub(crate) use resolve_tuxi_gain_other_hands_replace_draw::*;
mod resolve_choose_dingpan_branch;
pub(crate) use resolve_choose_dingpan_branch::*;
mod draw_give_or_discard_hand_cards;
pub(crate) use draw_give_or_discard_hand_cards::*;
