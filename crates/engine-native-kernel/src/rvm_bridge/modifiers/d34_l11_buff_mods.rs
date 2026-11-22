//! L11（太虚幻境）Batch L11-3 增益修饰器纯查表辅助（`d34_l11_buff_mods.rs`）。
//!
//! 承载需要内核跨机器直接读取的 Batch L11-3 状态规则：
//! - `shandian_reversed`：`txhj_huangtiannizhuan`（奇数个标记反转闪电判定）；
//! - `is_trick_unwuxie_for_nullify_window`：`buff_txhj_wenhezhenlue` 仅常规锦囊不可被无懈。

use super::d34_flags::d34_l11_mods::is_ordinary_trick_kind;
use super::d34_flags::{flag_val, has_ext_skill};
use crate::catalog::Catalog;
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

/// `txhj_huangtiannizhuan`（`extension_skill.js:3413`）：持有者且标记数为奇数时，
/// 闪电判定结果反转（`♥/♣/♦/♠非2-9` 命中，`♠2-9` 未命中）。
pub fn shandian_reversed(state: &GameStateCore, catalog: &Catalog, target: PlayerId) -> bool {
    has_ext_skill(state, catalog, target, "txhj_huangtiannizhuan")
        && flag_val(state, target, "l11_huangtiannizhuan_marks") % 2 == 1
}

/// `_buff_txhj_wenhezhenlue`（`extension_skill.js:1800`）：常规锦囊（`get.type == 'trick'`，
/// 不含延时锦囊）不可被无懈可击响应。
pub fn is_trick_unwuxie_for_nullify_window(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    card: CardId,
) -> bool {
    if flag_val(state, source, &format!("card_nowuxie:{card}")) > 0 {
        return true;
    }
    let kind = state
        .declared_kinds
        .get(&card)
        .copied()
        .unwrap_or_else(|| catalog.card_kind(card));
    is_ordinary_trick_kind(kind)
        && (flag_val(state, source, "l11_buff_trick_unwuxie") > 0
            || has_ext_skill(state, catalog, source, "buff_txhj_wenhezhenlue")
            || has_ext_skill(state, catalog, source, "_buff_txhj_wenhezhenlue"))
}
