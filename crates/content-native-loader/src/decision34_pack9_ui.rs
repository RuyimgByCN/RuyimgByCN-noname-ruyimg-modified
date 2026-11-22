//! UI, standee, skin-transform, DIY draft, and dual-general pick helpers for Batch 26 sources
//! (`U01`, `L24`, `L25`, `L27`).

use super::U01_DDD_GENERALS;

/// `U01` (`3D精选`) 3D perspective standee gallery state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct U01GalleryState {
    pub selected_general: String,
    pub yaw_deg: i16,
    pub pitch_deg: i16,
    pub zoom_pct: u16,
}

/// Resolve and validate `U01` 3D select gallery camera/standee parameters.
pub fn resolve_u01_3d_gallery(
    general_id: &str,
    yaw_deg: i16,
    pitch_deg: i16,
    zoom_pct: u16,
) -> Result<U01GalleryState, String> {
    if !U01_DDD_GENERALS.iter().any(|&(g, _, _, _, _)| g == general_id) {
        return Err(format!("UNKNOWN_U01_DDD_GENERAL:{general_id}"));
    }
    Ok(U01GalleryState {
        selected_general: general_id.to_string(),
        yaw_deg: yaw_deg.clamp(-45, 45),
        pitch_deg: pitch_deg.clamp(-30, 30),
        zoom_pct: zoom_pct.clamp(80, 150),
    })
}

/// `L24` (`皮肤切换`) dynamic skin transform & Spine animation clip state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct L24SkinTransform {
    pub general_id: String,
    pub skin_slot: u8,
    pub stand_x: i32,
    pub stand_y: i32,
    pub scale_pct: u16,
    pub active_clip: String,
}

/// Resolve and validate `L24` dynamic skin coordinate offsets and clip triggers
/// (`daiji`, `gongji`, `teshu`, `chuchang`, `shan`).
pub fn resolve_l24_skin_transform(
    general_id: &str,
    skin_slot: u8,
    clip: &str,
) -> Result<L24SkinTransform, String> {
    if general_id.trim().is_empty() {
        return Err("EMPTY_GENERAL_ID".to_string());
    }
    if !(1..=4).contains(&skin_slot) {
        return Err(format!("INVALID_SKIN_SLOT:{skin_slot}"));
    }
    if !matches!(clip, "daiji" | "gongji" | "teshu" | "chuchang" | "shan") {
        return Err(format!("INVALID_SPINE_CLIP:{clip}"));
    }
    let slot_i = i32::from(skin_slot);
    let (dx, dy, scale_pct) = match clip {
        "gongji" => (slot_i * 12, -8, 108),
        "teshu" => (slot_i * 8, -14, 112),
        "chuchang" => (0, -20, 115),
        "shan" => (-slot_i * 10, 4, 96),
        _ => (slot_i * 4, 0, 100),
    };
    Ok(L24SkinTransform {
        general_id: general_id.to_string(),
        skin_slot,
        stand_x: dx,
        stand_y: dy,
        scale_pct,
        active_clip: clip.to_string(),
    })
}

/// `L25` (`祖安扩展`) offline-gated sandboxed DIY general/skill editor draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct L25DiyEditorDraft {
    pub general_id: String,
    pub faction: String,
    pub max_hp: u8,
    pub skills: Vec<String>,
    pub online_room_active: bool,
}

/// Validate and apply an `L25` DIY character draft; strictly rejects online multiplayer rooms.
pub fn validate_and_apply_l25_diy(draft: &L25DiyEditorDraft) -> Result<String, String> {
    if draft.online_room_active {
        return Err("ONLINE_MULTIPLAYER_DIY_FORBIDDEN".to_string());
    }
    if draft.general_id.trim().is_empty() {
        return Err("EMPTY_DIY_GENERAL_ID".to_string());
    }
    if !(1..=15).contains(&draft.max_hp) {
        return Err(format!("INVALID_DIY_MAX_HP:{}", draft.max_hp));
    }
    if draft.skills.is_empty() {
        return Err("EMPTY_DIY_SKILLS".to_string());
    }
    Ok(format!(
        "diy_ok:{}|faction:{}|hp:{}|skills:{}",
        draft.general_id,
        draft.faction,
        draft.max_hp,
        draft.skills.join(",")
    ))
}

/// `L27` (`选将美化`) dual-general selection (`chooseCharacterTwo`) state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct L27DualGeneralPick {
    pub candidates: Vec<String>,
    pub selected: Vec<String>,
}

/// Resolve and validate `L27` `chooseCharacterTwo` dual-general pick.
pub fn resolve_l27_choose_character_two(
    pick: &L27DualGeneralPick,
) -> Result<(String, String, u8), String> {
    if pick.selected.len() != 2 {
        return Err(format!("DUAL_PICK_REQUIRES_2_GOT_{}", pick.selected.len()));
    }
    let primary = &pick.selected[0];
    let secondary = &pick.selected[1];
    if primary == secondary {
        return Err("DUAL_PICK_DUPLICATE_GENERAL".to_string());
    }
    if !pick.candidates.contains(primary) || !pick.candidates.contains(secondary) {
        return Err("DUAL_PICK_NOT_IN_CANDIDATES".to_string());
    }
    Ok((primary.clone(), secondary.clone(), 4))
}
