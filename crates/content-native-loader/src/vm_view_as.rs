//! PA12 Batch 15：Rule-VM 转化技（`PackageViewAsEntry` → `VmViewAsSpec`）桥接。

use super::*;
use engine_native::catalog::CardKind;
use engine_native::rvm_bridge::{
    VmActiveLimits, VmFilterPolicy, VmHandlerSet, VmSkillProgram, VmTriggerProgram, VmViewAsFilter,
    VmViewAsSpec, VmViewAsWindow, VmViewAsZone,
};
use std::sync::Arc;

fn map_view_as_window(window: &str) -> Option<VmViewAsWindow> {
    match window {
        "play_phase" => Some(VmViewAsWindow::PlayPhase),
        "response" => Some(VmViewAsWindow::Response),
        "any" => Some(VmViewAsWindow::Any),
        _ => None,
    }
}

fn map_view_as_zone(zone: Option<&str>) -> Option<VmViewAsZone> {
    match zone {
        None | Some("hand") => Some(VmViewAsZone::Hand),
        Some("equip") => Some(VmViewAsZone::Equip),
        Some("hand_equip" | "hand_or_equip") => Some(VmViewAsZone::HandEquip),
        _ => None,
    }
}

fn map_view_as_kind(as_kind: &str, nature: Option<&str>) -> Option<CardKind> {
    match (as_kind, nature) {
        ("sha", None) => Some(CardKind::Sha),
        ("sha", Some("fire")) | ("fire_sha", None) => Some(CardKind::FireSha),
        ("sha", Some("thunder")) | ("thunder_sha" | "lei_sha", None) => Some(CardKind::LeiSha),
        ("shan", None) => Some(CardKind::Shan),
        ("tao", None) => Some(CardKind::Tao),
        ("jiu", None) => Some(CardKind::Jiu),
        ("wuxie" | "nullify", None) => Some(CardKind::Nullify),
        ("guohe" | "dismantle", None) => Some(CardKind::Dismantle),
        ("shunshou" | "snatch", None) => Some(CardKind::Snatch),
        ("juedou" | "duel", None) => Some(CardKind::Duel),
        ("wuzhong", None) => Some(CardKind::Wuzhong),
        ("taoyuan", None) => Some(CardKind::Taoyuan),
        ("wugu", None) => Some(CardKind::Wugu),
        ("jiedao", None) => Some(CardKind::Jiedao),
        ("nanman" | "savage_attack", None) => Some(CardKind::SavageAttack),
        ("wanjian" | "archery_attack", None) => Some(CardKind::ArcheryAttack),
        ("tiesuo" | "chain", None) => Some(CardKind::Chain),
        ("le" | "lebu" | "lebusishu", None) => Some(CardKind::Le),
        ("bing" | "bingliang", None) => Some(CardKind::Bing),
        ("shandian", None) => Some(CardKind::ShanDian),
        _ => None,
    }
}

fn map_view_as_filter(filter: &str) -> Option<VmViewAsFilter> {
    Some(match filter {
        "any" => VmViewAsFilter::Any,
        "red" | "card_red" => VmViewAsFilter::Red,
        "black" | "card_black" => VmViewAsFilter::Black,
        "spade" | "card_spade" => VmViewAsFilter::Spade,
        "heart" | "card_heart" => VmViewAsFilter::Heart,
        "club" | "card_club" => VmViewAsFilter::Club,
        "diamond" | "card_diamond" => VmViewAsFilter::Diamond,
        "owner_dying_club" => VmViewAsFilter::OwnerDyingClub,
        "owner_outside_turn" => VmViewAsFilter::OwnerOutsideTurn,
        "owner_outside_turn_red" => VmViewAsFilter::OwnerOutsideTurnRed,
        "owner_single_hand" => VmViewAsFilter::OwnerSingleHand,
        "owner_single_red_hand" => VmViewAsFilter::OwnerSingleRedHand,
        "owner_hand_gt_hp" => VmViewAsFilter::OwnerHandGtHp,
        "owner_hp_le_two_black_trick" => VmViewAsFilter::OwnerHpLeTwoBlackTrick,
        "card_basic" | "basic" => VmViewAsFilter::Basic,
        "non_basic" | "card_nonbasic" | "nonbasic" => VmViewAsFilter::NonBasic,
        "card_equip" | "equip" => VmViewAsFilter::Equip,
        "card_non_equip" | "non_equip" => VmViewAsFilter::NonEquip,
        "card_trick" | "trick" => VmViewAsFilter::Trick,
        "damage_trick" => VmViewAsFilter::DamageTrick,
        "card_sha" | "sha" => VmViewAsFilter::Sha,
        "card_shan" | "shan" => VmViewAsFilter::Shan,
        "card_tao" | "tao" => VmViewAsFilter::Tao,
        "card_jiu" | "jiu" => VmViewAsFilter::Jiu,
        "weapon" | "card_weapon" => VmViewAsFilter::Weapon,
        "armor" | "card_armor" => VmViewAsFilter::Armor,
        "mount" | "card_mount" => VmViewAsFilter::Mount,
        "black_basic_equip" => VmViewAsFilter::BlackBasicEquip,
        "red_basic" => VmViewAsFilter::RedBasic,
        "black_basic" => VmViewAsFilter::BlackBasic,
        "red_trick" => VmViewAsFilter::RedTrick,
        "black_trick" => VmViewAsFilter::BlackTrick,
        "red_non_basic" => VmViewAsFilter::RedNonBasic,
        "black_non_basic" => VmViewAsFilter::BlackNonBasic,
        _ => return None,
    })
}

pub(super) fn compile_view_as_entry(
    skill_name: &str,
    v: &content_schema::PackageViewAsEntry,
) -> Option<VmViewAsSpec> {
    if v.min_cards != 1
        || v.max_cards != 1
        || v.owner_hp_min_one_cards
        || v.owner_basic_uses_plus_one_cards
        || v.once_per_turn_per_branch
        || v.same_suit
        || v.multi_card_bonus.is_some()
        || v.branch.is_some()
        || v.once_per_round
        || v.next_branch.is_some()
        || v.cost.is_some()
        || v.mark_cost.is_some()
        || v.uses_per_turn == Some(0)
    {
        return None;
    }
    let window = map_view_as_window(v.window.as_str())?;
    let zone = map_view_as_zone(v.source_zone.as_deref())?;
    let as_kind = map_view_as_kind(v.as_kind.as_str(), v.nature.as_deref())?;
    let filter = map_view_as_filter(v.filter.as_str())?;
    Some(VmViewAsSpec {
        skill_def_id: v.skill_def_id,
        skill_name: skill_name.to_string(),
        as_kind,
        window,
        zone,
        filter,
        uses_per_turn: v.uses_per_turn,
        ignore_distance: v.ignore_distance,
        red_unrespondable: v.red_unrespondable,
        black_no_use_count: v.black_no_use_count,
        no_use_count: v.no_use_count,
        target_lacks_source_suit_unrespondable: v.target_lacks_source_suit_unrespondable,
        refund_use_count_if_no_damage: v.refund_use_count_if_no_damage,
        post_draw: v.post_draw,
    })
}

pub(super) fn bridge_view_as_entries(
    pkg: &content_schema::ContentPackage,
    decls: &mut Vec<engine_native::content::extension::VmSkillDecl>,
) -> Result<(), BridgeError> {
    if pkg.view_as_index.is_empty() {
        return Ok(());
    }
    for v in &pkg.view_as_index {
        if !pkg
            .skill_index
            .iter()
            .any(|s| s.skill_def_id == v.skill_def_id)
        {
            return Err(invalid("view_as_index.skill_def_id", "missing logical skill"));
        }
    }
    for skill in &pkg.skill_index {
        let entries: Vec<_> = pkg
            .view_as_index
            .iter()
            .filter(|v| v.skill_def_id == skill.skill_def_id)
            .collect();
        if entries.is_empty() {
            continue;
        }
        if skill.lord_only || skill.non_gainable {
            return Err(invalid(
                "skill_index",
                "view-as skill metadata is unsupported",
            ));
        }
        let mut specs = Vec::with_capacity(entries.len());
        for v in entries {
            let spec = compile_view_as_entry(&skill.name, v).ok_or_else(|| {
                invalid(
                    "view_as_index",
                    "native VM view-as declarations are unsupported",
                )
            })?;
            specs.push(spec);
        }
        if let Some(existing) = decls.iter_mut().find(|d| d.name == skill.name) {
            match &existing.filter {
                VmFilterPolicy::Handlers(set) => {
                    let mut updated = (**set).clone();
                    updated.view_as = specs;
                    existing.filter = VmFilterPolicy::Handlers(Arc::new(updated));
                }
                VmFilterPolicy::Declared {
                    skill_def_id,
                    choice_actor,
                    modifiers,
                    ..
                } => {
                    let program = Arc::new(
                        VmSkillProgram::new(existing.module.clone(), skill.name.clone())
                            .map_err(|e| invalid("trigger_index.module", e.to_string()))?,
                    );
                    let trigger = VmTriggerProgram {
                        program,
                        meta: existing.trigger.clone(),
                        filter: existing.filter.clone(),
                        choice_actor: *choice_actor,
                    };
                    existing.filter = VmFilterPolicy::Handlers(Arc::new(VmHandlerSet {
                        skill_def_id: *skill_def_id,
                        active: None,
                        limits: VmActiveLimits::default(),
                        triggers: vec![trigger],
                        modifiers: modifiers.clone(),
                        view_as: specs,
                    }));
                }
                _ => {
                    return Err(invalid(
                        "view_as_index",
                        "unexpected filter policy on view-as skill",
                    ));
                }
            }
        } else {
            let mod_name = format!("{}#view_as", skill.name);
            let code = vec![0x01u8];
            let module = engine_native::rvm_bridge::BytecodeModule {
                logic_hash: engine_native::rvm_bridge::BytecodeModule::compute_logic_hash(
                    &mod_name, &code, 0, 0, 0,
                ),
                name: mod_name,
                code,
                local_count: 0,
                max_stack: 1,
                entry_pc: 0,
                caps_mask: 0,
            };
            decls.push(engine_native::content::extension::VmSkillDecl {
                declaration_hash: None,
                presentation_binding: None,
                name: skill.name.clone(),
                module,
                trigger: engine_native::catalog::TriggerMeta {
                    event: engine_native::event::EventKind::SkillToggled,
                    timing: engine_native::event::Timing::Normal,
                    priority: 0,
                    order_selectable: true,
                    optional: false,
                    extra_events: Vec::new(),
                },
                filter: VmFilterPolicy::Handlers(Arc::new(VmHandlerSet {
                    skill_def_id: skill.skill_def_id,
                    active: None,
                    limits: VmActiveLimits::default(),
                    triggers: Vec::new(),
                    modifiers: Vec::new(),
                    view_as: specs,
                })),
                r05_identity: None,
            });
        }
    }
    Ok(())
}
