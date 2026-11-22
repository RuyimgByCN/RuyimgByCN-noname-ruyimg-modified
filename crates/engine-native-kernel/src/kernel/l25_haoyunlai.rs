//! L25 `_haoyunlai`（`祖安武将/extension.js:47584-47660`）：
//! 仅当配置开启 `goodluck` 且摸牌者为 `game.me`（原生 0 号位）且牌堆余牌 > 1 时，
//! 于 `drawBegin` 在牌堆顶最多 20 张中选出价值最高的牌置顶（不消耗 RNG）。

use crate::catalog::{CardKind, Catalog};
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

fn haoyunlai_card_value(core: &GameStateCore, catalog: &Catalog, player: PlayerId, cid: CardId) -> i32 {
    let p = core.player(player);
    match catalog.card_kind(cid) {
        CardKind::Tao => {
            if p.hp < p.max_hp as i32 {
                100
            } else {
                75
            }
        }
        CardKind::Wuzhong => 95,
        CardKind::Snatch => 85,
        CardKind::Dismantle => 80,
        CardKind::Nullify => 72,
        CardKind::Jiu => 65,
        CardKind::Duel
        | CardKind::SavageAttack
        | CardKind::ArcheryAttack
        | CardKind::Taoyuan
        | CardKind::Jiedao => 62,
        CardKind::Le | CardKind::Bing => 58,
        k if k.equip_slot().is_some() => 55,
        CardKind::Wugu | CardKind::Chain => 48,
        CardKind::Shan => 45,
        k if k.is_sha_like() => 42,
        k if !k.is_basic_kind() && k.equip_slot().is_none() && k.is_basic_trick_or_equip() => 50,
        _ => 20,
    }
}

/// 在原生 `DrawMachine` 开始摸牌（`drawBegin`）时执行 `_haoyunlai` 置顶调序（零 RNG 扰动）。
pub fn l25_haoyunlai_on_draw_begin(
    core: &mut GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) {
    if player != 0 || core.deck.len() <= 1 {
        return;
    }
    let enabled = core
        .vm_aux
        .flags
        .get(&(0, "l25_goodluck_enabled".to_string()))
        .copied()
        .unwrap_or(0)
        > 0;
    if !enabled {
        return;
    }
    if !catalog
        .skill_by_name("_haoyunlai")
        .is_some_and(|d| core.has_skill(player, d))
    {
        return;
    }
    let len = core.deck.len();
    let start = len.saturating_sub(20);
    let mut best_idx = len - 1;
    let mut best_val = haoyunlai_card_value(core, catalog, player, core.deck[best_idx]);
    // 自牌堆顶向下扫描至多 19 张后续牌，严格更高价值者置顶。
    for idx in (start..len - 1).rev() {
        let v = haoyunlai_card_value(core, catalog, player, core.deck[idx]);
        if v > best_val {
            best_val = v;
            best_idx = idx;
        }
    }
    if best_idx != len - 1 {
        let chosen = core.deck.remove(best_idx);
        core.deck.push(chosen);
    }
}
