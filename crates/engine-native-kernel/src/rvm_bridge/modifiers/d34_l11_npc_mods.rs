//! Batch L11-6a 太虚自有 NPC 武将技的只读规则修正（`txhj_guimei` / `txhj_renwang`）。

use crate::catalog::{CardKind, Catalog};
use crate::ids::PlayerId;
use crate::state::GameStateCore;

fn has_unsealed_npc_skill(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    skill_name: &str,
    locked: bool,
) -> bool {
    if !state.player(player).alive
        || catalog.is_chanyuan_sealed(state, player)
        || catalog
            .skill_by_name("txhj_feilong_skill_blank")
            .is_some_and(|b| state.has_skill(player, b))
    {
        return false;
    }
    if !locked
        && (catalog.is_tieji_sealed(state, player)
            || crate::kernel::mbcmfangzhu_baiban(state, player))
    {
        return false;
    }
    catalog
        .skill_by_name(skill_name)
        .is_some_and(|def| state.has_skill(player, def))
}

/// 仁王（`txhj_renwang` · `extension.js:1393`）：手牌上限 +“方”数（不计铁骑封锁外的失效）。
pub(super) fn apply_hand_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    limit: i32,
) -> i32 {
    if !has_unsealed_npc_skill(state, catalog, player, "txhj_renwang", false) {
        return limit;
    }
    let fang = state
        .vm_aux
        .piles
        .get(&(player, "txhj_renwang".to_string()))
        .map_or(0, |v| v.len() as i32);
    limit.saturating_add(fang)
}

/// 仁王（`txhj_renwang` · `extension.js:1393`）：出杀次数上限 + `ceil(方 / 5)`。
pub(super) fn apply_sha_use_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    cur: i32,
    unlimited: bool,
) -> (i32, bool) {
    if !has_unsealed_npc_skill(state, catalog, player, "txhj_renwang", false) {
        return (cur, unlimited);
    }
    let fang = state
        .vm_aux
        .piles
        .get(&(player, "txhj_renwang".to_string()))
        .map_or(0, |v| v.len());
    let bonus = fang.div_ceil(5) as i32;
    (cur.saturating_add(bonus), unlimited)
}

/// 鬼魅（`txhj_guimei` · `extension.js:658`）：锁定技，不能成为延时锦囊目标。
pub(super) fn apply_card_target_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    target: PlayerId,
    kind: CardKind,
    canceled: bool,
) -> bool {
    if !canceled
        && kind.is_delay_trick_kind()
        && has_unsealed_npc_skill(state, catalog, target, "txhj_guimei", true)
    {
        return true;
    }
    canceled
}
