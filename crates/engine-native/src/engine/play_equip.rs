//! 装备牌出牌分派与相关出牌钩子旗标记录（自 `play_card` 拆出）。

use super::*;

pub(super) fn play_equip(
    state: &mut GameState,
    catalog: &SharedCatalog,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), RejectReason> {
    if !targets.is_empty() {
        return Err(RejectReason::IllegalTargets);
    }
    let player = state.core.turn.current;
    if !state.core.player(player).hand.contains(&card) {
        return Err(RejectReason::CardNotInHand);
    }
    if catalog
        .card_kind(card)
        .equip_slot()
        .is_some_and(|slot| !catalog.is_equip_slot_enabled(&state.core, player, slot))
    {
        return Err(RejectReason::CardLimit);
    }
    crate::kernel::record_dddanzhi_card(&mut state.core, catalog, Some(card));
    let frame = crate::resolution::push_frame(
        &mut state.resolution,
        EventData::Equipped {
            player,
            card,
            replaced: None,
        },
    );
    if catalog
        .skill_by_name("dddjinggou")
        .is_some_and(|def| state.core.has_skill(player, def))
    {
        state
            .core
            .vm_aux
            .flags
            .insert((player, format!("play_equip_use:{frame}")), 1);
    }
    if catalog
        .skill_by_name("dddzhilian")
        .is_some_and(|def| state.core.alive_players().any(|p| state.core.has_skill(p, def)))
    {
        state
            .core
            .vm_aux
            .flags
            .insert((player, format!("zhilian_equip_use:{frame}")), 1);
    }
    if catalog.skill_by_name("dddanzhi").is_some() {
        state
            .core
            .vm_aux
            .flags
            .insert((player, format!("dddanzhi_equip_use:{frame}")), 1);
    }
    if catalog.skill_by_name("dddjiexing").is_some() {
        state
            .core
            .vm_aux
            .flags
            .insert((player, format!("u01_equip_played:{frame}")), 1);
    }
    if catalog.skill_by_name("dddchashi").is_some() {
        state
            .core
            .vm_aux
            .flags
            .insert((player, format!("chashi_equip_use:{frame}")), 1);
    }
    if catalog.skill_by_name("txmouduan").is_some()
        || catalog.skill_by_name("txhj_mouduan").is_some()
        || catalog.skill_by_name("txhj_ditingSkill3").is_some()
    {
        state
            .core
            .vm_aux
            .flags
            .insert((player, format!("l11_equip_use:{frame}")), 1);
    }
    if state.core.card_natures.contains_key(&card) {
        state
            .core
            .vm_aux
            .flags
            .insert((player, format!("wuxing_equip_use:{frame}")), 1);
    }
    Ok(())
}
