//! L11（太虚幻境）Batch L11-3 / L11-4 增益修饰器纯查表辅助（`d34_l11_buff_mods.rs`）。
//!
//! 承载需要内核与跨机器直接读取的状态规则：
//! - `shandian_reversed`：`txhj_huangtiannizhuan`（奇数个标记反转闪电判定）；
//! - `is_trick_unwuxie_for_nullify_window`：`buff_txhj_wenhezhenlue` 仅常规锦囊不可被无懈；
//! - `is_trick_like_kind`：`cebeibuyu` 覆盖普通锦囊与延时锦囊；
//! - `taoyuan_recover_bonus`、`shierqicejin_excludes`、`shierqicetui_excludes`；
//! - `has_duocejiyong`、`has_wolongmice`、`has_fenchengluetu`；
//! - `has_fuchendangmo`、`has_shenhanxueyin`、`has_shangjiangdelvli`；
//! - `huangtian_judge_override`：`huangtianyili`（黑桃）与 `huangtianzhizu`（黑桃 5）。

use super::d34_flags::d34_l11_mods::is_ordinary_trick_kind;
use super::d34_flags::{flag_val, has_ext_skill};
use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

/// L11 锁定技/模式祝福统一激活检查：未被退避/白板（`mbcmfangzhu_ban == 0`）且存活。
fn l11_buff_active(state: &GameStateCore, p: PlayerId) -> bool {
    state.player(p).alive && flag_val(state, p, "mbcmfangzhu_ban") == 0
}

/// 槽位 `slot` 是否有装备。
pub fn has_equip_slot(state: &GameStateCore, catalog: &Catalog, p: PlayerId, slot: EquipSlot) -> bool {
    state
        .player(p)
        .equips
        .iter()
        .any(|&c| catalog.card_slot_of(state, c) == Some(slot))
}

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

/// `get.type(card, 'trick') == 'trick' || get.type(card, 'delay') == 'delay'`：
/// 覆盖全部普通锦囊与延时锦囊。
pub fn is_trick_like_kind(kind: CardKind) -> bool {
    is_ordinary_trick_kind(kind)
        || matches!(
            kind,
            CardKind::Le | CardKind::Bing | CardKind::ShanDian | CardKind::HongShui | CardKind::Guiyoujie
        )
}

/// `buff_txhj_taoyuanyishi` / `_buff_txhj_taoyuanyishi` / `txhj_taoyuanyishi` / `taoyuanyishiX`
/// （`extension_buff.js:455`、`extension_skill.js:2563, 3425, 3435`）：使用者持祝福时【桃园结义】对己方回复量 +1。
pub fn taoyuan_recover_bonus(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    target: PlayerId,
) -> u32 {
    if !l11_buff_active(state, owner) || !state.is_friend_of(catalog, owner, target) {
        return 0;
    }
    let active = has_ext_skill(state, catalog, owner, "buff_txhj_taoyuanyishi")
        || has_ext_skill(state, catalog, owner, "_buff_txhj_taoyuanyishi")
        || has_ext_skill(state, catalog, owner, "txhj_taoyuanyishi")
        || has_ext_skill(state, catalog, target, "taoyuanyishiX");
    u32::from(active)
}

/// `buff_txhj_shierqicejin` / `_buff_txhj_shierqicejin` / `txhj_shierqicejin`
/// （`extension_skill.js:3129, 3543`）：玩家使用的【南蛮入侵】【万箭齐发】对己方（含带标记友方）无效，仍成为目标。
pub fn shierqicejin_excludes(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
) -> bool {
    if target == source || !l11_buff_active(state, source) || !state.is_friend_of(catalog, source, target) {
        return false;
    }
    let has_buff = has_ext_skill(state, catalog, source, "buff_txhj_shierqicejin")
        || has_ext_skill(state, catalog, source, "_buff_txhj_shierqicejin");
    if !has_buff {
        return false;
    }
    if flag_val(state, source, "l11_shierqicejin_applied") > 0 {
        has_ext_skill(state, catalog, target, "txhj_shierqicejin")
    } else {
        true
    }
}

/// `buff_txhj_shierqicetui` / `_buff_txhj_shierqicetui` / `txhj_shierqicetui`
/// （`extension_skill.js:3159, 3567`）：玩家使用的【桃园结义】【五谷丰登】对敌方（含带标记敌方）无效，仍成为目标。
pub fn shierqicetui_excludes(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
) -> bool {
    if target == source || !l11_buff_active(state, source) || state.is_friend_of(catalog, source, target) {
        return false;
    }
    let has_buff = has_ext_skill(state, catalog, source, "buff_txhj_shierqicetui")
        || has_ext_skill(state, catalog, source, "_buff_txhj_shierqicetui");
    if !has_buff {
        return false;
    }
    if flag_val(state, source, "l11_shierqicetui_applied") > 0 {
        has_ext_skill(state, catalog, target, "txhj_shierqicetui")
    } else {
        true
    }
}

/// `_buff_txhj_duocejiyong`（`extension_skill.js:2609`）：持有者使用【无懈可击】生效时获得被抵消的牌。
pub fn has_duocejiyong(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> bool {
    l11_buff_active(state, player)
        && (has_ext_skill(state, catalog, player, "buff_txhj_duocejiyong")
            || has_ext_skill(state, catalog, player, "_buff_txhj_duocejiyong"))
}

/// `buff_txhj_wolongmice`（`extension_skill.js:487`）：【火攻】仅需弃置同色手牌。
pub fn has_wolongmice(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> bool {
    l11_buff_active(state, player) && has_ext_skill(state, catalog, player, "buff_txhj_wolongmice")
}

/// `buff_txhj_fenchengluetu`（`extension_skill.js:511`）：【火攻】成功弃牌后先获得目标展示的牌。
pub fn has_fenchengluetu(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> bool {
    l11_buff_active(state, player) && has_ext_skill(state, catalog, player, "buff_txhj_fenchengluetu")
}

/// `buff_txhj_fuchendangmo`（`extension_skill.js:638..872`）：武器效果令他方弃牌时弃牌数 +1。
pub fn has_fuchendangmo(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> bool {
    l11_buff_active(state, player) && has_ext_skill(state, catalog, player, "buff_txhj_fuchendangmo")
}

/// `buff_txhj_shenhanxueyin`（`extension_buff.js:630`）：装备武器时发动武器效果弃置他方牌改为获得之。
pub fn has_shenhanxueyin(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> bool {
    l11_buff_active(state, player)
        && has_equip_slot(state, catalog, player, EquipSlot::Weapon)
        && has_ext_skill(state, catalog, player, "buff_txhj_shenhanxueyin")
}

/// `buff_txhj_shangjiangdelvli`（`extension_buff.js:619`）：装备武器时发动武器效果须弃置自身牌数 -1。
pub fn has_shangjiangdelvli(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> bool {
    l11_buff_active(state, player)
        && has_equip_slot(state, catalog, player, EquipSlot::Weapon)
        && has_ext_skill(state, catalog, player, "buff_txhj_shangjiangdelvli")
}

/// `_buff_txhj_huangtianyili`（`3211`：`priority 7` 花色→黑桃）与 `_buff_txhj_huangtianzhizu`
/// （`3234`：`priority 5` 花色→黑桃且点数→5）的 `judge` 改写覆盖。
pub fn huangtian_judge_override(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Option<(Suit, Option<u8>)> {
    if !l11_buff_active(state, owner) {
        return None;
    }
    let has_zhizu = has_ext_skill(state, catalog, owner, "buff_txhj_huangtianzhizu")
        || has_ext_skill(state, catalog, owner, "_buff_txhj_huangtianzhizu");
    if has_zhizu {
        return Some((Suit::Spade, Some(5)));
    }
    let has_yili = has_ext_skill(state, catalog, owner, "buff_txhj_huangtianyili")
        || has_ext_skill(state, catalog, owner, "_buff_txhj_huangtianyili");
    if has_yili {
        return Some((Suit::Spade, None));
    }
    None
}

/// 套用黄天系列判定覆盖后的最终 `(Suit, u8)`（供 `JudgePhaseMachine` 与内联判定共用）。
pub fn apply_huangtian_suit_rank(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    suit: Suit,
    rank: u8,
) -> (Suit, u8) {
    match huangtian_judge_override(state, catalog, owner) {
        Some((s, Some(n))) => (s, n),
        Some((s, None)) => (s, rank),
        None => (suit, rank),
    }
}
