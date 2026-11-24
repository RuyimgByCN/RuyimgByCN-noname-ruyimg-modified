//! 弹栈清理辅助（`pop_cleanup.rs`）。

use crate::event::EventData;
use crate::ids::{EventId, PlayerId};
use crate::state::GameStateCore;

pub(super) fn clear_jifeng_more_flags(core: &mut GameStateCore, player: PlayerId, frame_id: EventId) {
    let len = core
        .vm_aux
        .flags
        .remove(&(player, format!("jifeng_more_len:{frame_id}")))
        .unwrap_or(0);
    core.vm_aux
        .flags
        .remove(&(player, format!("jifeng_more_idx:{frame_id}")));
    core.vm_aux
        .flags
        .remove(&(player, format!("jifeng_vh:{frame_id}")));
    core.vm_aux
        .flags
        .remove(&(player, format!("jifeng_vtc:{frame_id}")));
    for i in 0..len {
        core.vm_aux
            .flags
            .remove(&(player, format!("jifeng_more:{frame_id}:{i}")));
    }
}

/// 疾风/游击多目标普通锦囊：`Slot::Intrinsic` 首目标前拆分余下目标至 `jifeng_more_*`。
pub(crate) fn setup_jifeng_more_targets(
    core: &mut GameStateCore,
    res: &mut super::Resolution,
    frame_id: EventId,
    u: &mut crate::event::CardUseData,
) {
    let tail: Vec<PlayerId> = u.targets[1..].to_vec();
    core.vm_aux
        .flags
        .insert((u.player, format!("jifeng_more_len:{frame_id}")), tail.len() as u32);
    core.vm_aux
        .flags
        .insert((u.player, format!("jifeng_more_idx:{frame_id}")), 0);
    if u.card.is_none() {
        let vh = core
            .vm_aux
            .flags
            .get(&(u.player, "l11_virtual_huogong".to_string()))
            .copied()
            .unwrap_or(0);
        if vh > 0 {
            core.vm_aux
                .flags
                .insert((u.player, format!("jifeng_vh:{frame_id}")), vh);
        }
        let vtc = core
            .vm_aux
            .flags
            .get(&(u.player, "l11_virtual_trick_cid".to_string()))
            .copied()
            .unwrap_or(0);
        if vtc > 0 {
            core.vm_aux
                .flags
                .insert((u.player, format!("jifeng_vtc:{frame_id}")), vtc);
        }
    }
    for (i, p) in tail.into_iter().enumerate() {
        core.vm_aux
            .flags
            .insert((u.player, format!("jifeng_more:{frame_id}:{i}")), u32::from(p));
    }
    u.targets.truncate(1);
    if let Some(fr) = res.frame_mut(frame_id) {
        fr.card_use_mut().targets.truncate(1);
    }
}

/// 疾风多目标普通锦囊：`Slot::Intrinsic` 收束后弹出下一个存活目标并回放至 `table`；返回 `true` 表示继续下一目标。
pub(super) fn advance_jifeng_slot(
    core: &mut GameStateCore,
    res: &mut super::Resolution,
    frame_id: EventId,
) -> bool {
    let Some((player, card)) = res.frame(frame_id).and_then(|f| match &f.data {
        EventData::CardUsed(u) => Some((u.player, u.card)),
        _ => None,
    }) else {
        return false;
    };
    let Some(&len) = core
        .vm_aux
        .flags
        .get(&(player, format!("jifeng_more_len:{frame_id}")))
    else {
        return false;
    };
    let mut idx = core
        .vm_aux
        .flags
        .get(&(player, format!("jifeng_more_idx:{frame_id}")))
        .copied()
        .unwrap_or(0);
    while idx < len {
        let next_t = core
            .vm_aux
            .flags
            .get(&(player, format!("jifeng_more:{frame_id}:{idx}")))
            .copied()
            .unwrap_or(0) as PlayerId;
        idx += 1;
        if !core.player(next_t).alive {
            continue;
        }
        core.vm_aux
            .flags
            .insert((player, format!("jifeng_more_idx:{frame_id}")), idx);
        if let Some(vh) = core
            .vm_aux
            .flags
            .get(&(player, format!("jifeng_vh:{frame_id}")))
            .copied()
        {
            core.vm_aux
                .flags
                .insert((player, "l11_virtual_huogong".to_string()), vh);
        }
        if let Some(vtc) = core
            .vm_aux
            .flags
            .get(&(player, format!("jifeng_vtc:{frame_id}")))
            .copied()
        {
            core.vm_aux
                .flags
                .insert((player, "l11_virtual_trick_cid".to_string()), vtc);
        }
        if let Some(c) = card {
            if !core.table.contains(&c) && core.discard.contains(&c) {
                core.discard.retain(|&x| x != c);
                core.table.push(c);
            }
        }
        if let Some(f) = res.frame_mut(frame_id) {
            f.card_use_mut().targets = vec![next_t];
        }
        return true;
    }
    clear_jifeng_more_flags(core, player, frame_id);
    false
}

pub(super) fn on_damage_pop_cleanup(
    core: &mut GameStateCore,
    catalog: &crate::catalog::Catalog,
    popped_id: EventId,
    target: PlayerId,
) {
    core.vm_aux
        .flags
        .remove(&(target, "l28_vdmg_kind".to_string()));
    core.vm_aux
        .flags
        .remove(&(target, format!("damage_hujia_absorbed:{popped_id}")));
    let y2_def = catalog.skill_by_behavior(crate::vocabulary::host::HostBehavior::SwdYinguo2);
    for p in 0..core.players.len() as PlayerId {
        core.vm_aux
            .flags
            .remove(&(p, format!("mbcmjiushi_dmg:{popped_id}")));
        let prefix = "yinguo2_frame:";
        let this_key = format!("{prefix}{popped_id}");
        if core.vm_aux.flags.remove(&(p, this_key)).is_some() {
            let has_other = core
                .vm_aux
                .flags
                .keys()
                .any(|(seat, k)| *seat == p && k.starts_with(prefix));
            if !has_other {
                if let Some(def) = y2_def {
                    core.skills.remove(&(p, def));
                }
            }
        }
    }
}
