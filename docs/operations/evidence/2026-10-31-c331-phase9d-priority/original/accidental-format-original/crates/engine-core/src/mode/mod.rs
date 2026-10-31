//! Game modes (Phase 5: identity; ADR 0598: guozhan skeleton; M4: single).

mod guozhan;
mod identity;
mod single;

pub use guozhan::{
    apply_guozhan_victory_if_any, are_allies, check_guozhan_victory, compute_group_from_pair,
    faction_population, form_alliance, guozhan_max_hp, guozhan_player, is_major_faction_player,
    is_perfect_pair, resolve_lourenxin_choice, reveal_general, same_side, should_become_ambition,
    total_population, AMBITION, FACTIONS, LOUREN_XIN_CHOICE, ZHULIAN_BIHE_MARK,
};
pub use identity::{
    apply_victory_if_any, check_identity_victory, distribute_identities, IdentityVictory,
    STANDARD_IDENTITY_TABLE,
};
pub use single::{apply_single_victory_if_any, check_single_victory, SingleVictory};
