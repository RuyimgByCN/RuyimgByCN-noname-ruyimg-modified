//! Validate active declarations before joining them to their trigger handlers.
use super::*;
use engine_native::rvm_bridge::{
    VmActiveLimits, VmFilterPolicy, VmHandlerSet, VmSkillProgram, VmTriggerProgram,
};
use std::sync::Arc;

pub(super) fn bridge_active_entries(
    pkg: &content_schema::ContentPackage,
    decls: &mut Vec<engine_native::content::extension::VmSkillDecl>,
) -> Result<(), BridgeError> {
    let mut seen = BTreeSet::new();
    for active in &pkg.active_index {
        if !seen.insert(active.skill_def_id) {
            return Err(invalid(
                "active_index",
                "multiple active entries for one skill are unsupported",
            ));
        }
        let skill = pkg
            .skill_index
            .iter()
            .find(|s| s.skill_def_id == active.skill_def_id)
            .ok_or_else(|| invalid("active_index.skill_def_id", "missing logical skill"))?;
        if skill.lord_only || skill.conversion || skill.non_gainable {
            return Err(invalid(
                "skill_index",
                "active skill metadata is unsupported",
            ));
        }
        if active.window != "play_phase"
            || active.selection.is_some()
            || active
                .targets
                .as_ref()
                .is_some_and(|t| t.min != 0 || t.max != 0)
            || active.uses_per_turn.is_some_and(|n| n != 1)
            || active.uses_per_play_phase.is_some_and(|n| n != 1)
            || active.uses_per_turn_source.is_some()
            || active.owner_missing_skill.is_some()
            || active.hand_limit_positive
            || active.hand_missing_gaintag.is_some()
            || active.once_per_game
        {
            return Err(invalid(
                "active_index",
                format!(
                    "unsupported active declaration for {}: {active:?}",
                    skill.name
                ),
            ));
        }
        // A zero-target contract must not silently discard additional predicates.
        if let Some(targets) = &active.targets {
            let mut plain = targets.clone();
            plain.min = 0;
            plain.max = 0;
            plain.allow_self = targets.allow_self;
            let expected: content_schema::PackageTargetSpec = serde_json::from_value(
                serde_json::json!({"min":0,"max":0,"allow_self":targets.allow_self}),
            )
            .map_err(|e| invalid("active_index.targets", e.to_string()))?;
            if plain != expected {
                return Err(invalid(
                    "active_index.targets",
                    "unsupported target predicates",
                ));
            }
        }
        let module = pkg
            .modules
            .iter()
            .find(|m| m.name == active.module)
            .ok_or_else(|| invalid("active_index.module", "missing module"))?;
        validate_entry(module, active.entry_pc, "active_index.entry_pc")?;
        let program = Arc::new(
            VmSkillProgram::new(module.clone(), skill.name.clone())
                .map_err(|e| invalid("active_index.module", e.to_string()))?,
        );
        let mut triggers = Vec::new();
        let mut remove = Vec::new();
        for trigger in pkg
            .trigger_index
            .iter()
            .filter(|t| t.skill_def_id == active.skill_def_id)
        {
            let index = decls
                .iter()
                .enumerate()
                .find(|(index, d)| {
                    !remove.contains(index)
                        && d.name == skill.name
                        && d.module.name == trigger.module
                })
                .map(|(index, _)| index)
                .ok_or_else(|| invalid("trigger_index", "missing bridged handler"))?;
            let decl = &decls[index];
            if decl.trigger.event == engine_native::event::EventKind::DamageBefore {
                return Err(invalid(
                    "trigger_index.event",
                    "active skill with damage_before handler is unsupported",
                ));
            }
            if triggers.first().is_some_and(|t: &VmTriggerProgram| {
                (
                    t.meta.optional,
                    t.meta.timing,
                    t.meta.priority,
                    t.meta.order_selectable,
                ) != (
                    decl.trigger.optional,
                    decl.trigger.timing,
                    decl.trigger.priority,
                    decl.trigger.order_selectable,
                )
            }) {
                return Err(invalid(
                    "trigger_index.optional",
                    "mixed trigger ordering/confirmation tuples are unsupported",
                ));
            }
            triggers.push(VmTriggerProgram {
                program: Arc::new(
                    VmSkillProgram::new(decl.module.clone(), skill.name.clone())
                        .map_err(|e| invalid("trigger_index.module", e.to_string()))?,
                ),
                meta: decl.trigger.clone(),
                filter: decl.filter.clone(),
            });
            remove.push(index);
        }
        remove.sort_unstable();
        for index in remove.into_iter().rev() {
            decls.remove(index);
        }
        let mut meta = triggers.first().map(|t| t.meta.clone()).unwrap_or(
            engine_native::catalog::TriggerMeta {
                event: engine_native::event::EventKind::SkillToggled,
                timing: engine_native::event::Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            },
        );
        for trigger in &triggers {
            if trigger.meta.event != meta.event && !meta.extra_events.contains(&trigger.meta.event)
            {
                meta.extra_events.push(trigger.meta.event);
            }
            for extra in &trigger.meta.extra_events {
                if *extra != meta.event && !meta.extra_events.contains(extra) {
                    meta.extra_events.push(*extra);
                }
            }
        }
        let limits = VmActiveLimits {
            once_per_turn: active.uses_per_turn == Some(1),
            once_per_play_phase: active.uses_per_play_phase == Some(1),
            hand_nonempty: active.hand_nonempty,
            equip_nonempty: active.equip_nonempty,
            hand_gt_hp: active.hand_gt_hp,
            hand_gt_max_hp: active.hand_gt_max_hp,
            face_up: active.face_up,
            owner_unlinked: active.owner_unlinked,
            mark_at_least: active.mark_at_least,
        };
        decls.push(engine_native::content::extension::VmSkillDecl {
            declaration_hash: None,
            presentation_binding: None,
            name: skill.name.clone(),
            module: module.clone(),
            trigger: meta,
            filter: VmFilterPolicy::Handlers(Arc::new(VmHandlerSet {
                skill_def_id: active.skill_def_id,
                active: Some(program),
                limits,
                triggers,
            })),
            r05_identity: None,
        });
    }
    Ok(())
}

pub(super) fn validate_entry(
    module: &engine_native::rvm_bridge::BytecodeModule,
    entry_pc: u32,
    field: &'static str,
) -> Result<(), BridgeError> {
    if entry_pc != module.entry_pc {
        return Err(invalid(field, "non-default module entries are unsupported"));
    }
    VmSkillProgram::new(module.clone(), module.name.clone())
        .map_err(|e| invalid(field, e.to_string()))?;
    Ok(())
}

/// Trigger-only skills share the same stable logical identity as active+trigger skills.
pub(super) fn join_trigger_only_entries(
    pkg: &content_schema::ContentPackage,
    decls: &mut Vec<engine_native::content::extension::VmSkillDecl>,
) -> Result<(), BridgeError> {
    for skill in &pkg.skill_index {
        if pkg
            .active_index
            .iter()
            .any(|a| a.skill_def_id == skill.skill_def_id)
        {
            continue;
        }
        let indices: Vec<_> = decls
            .iter()
            .enumerate()
            .filter(|(_, d)| d.name == skill.name)
            .map(|(i, _)| i)
            .collect();
        if indices.len() <= 1 {
            continue;
        }
        let mut triggers = Vec::new();
        for index in &indices {
            let decl = &decls[*index];
            if let Some(first) = triggers.first() {
                let first: &VmTriggerProgram = first;
                if (
                    first.meta.timing,
                    first.meta.priority,
                    first.meta.order_selectable,
                    first.meta.optional,
                ) != (
                    decl.trigger.timing,
                    decl.trigger.priority,
                    decl.trigger.order_selectable,
                    decl.trigger.optional,
                ) {
                    return Err(invalid(
                        "trigger_index",
                        "mixed trigger ordering/confirmation tuples are unsupported",
                    ));
                }
            }
            triggers.push(VmTriggerProgram {
                program: Arc::new(
                    VmSkillProgram::new(decl.module.clone(), skill.name.clone())
                        .map_err(|e| invalid("trigger_index.module", e.to_string()))?,
                ),
                meta: decl.trigger.clone(),
                filter: decl.filter.clone(),
            });
        }
        let mut decl = decls[indices[0]].clone();
        for t in &triggers {
            if t.meta.event != decl.trigger.event
                && !decl.trigger.extra_events.contains(&t.meta.event)
            {
                decl.trigger.extra_events.push(t.meta.event);
            }
            for extra in &t.meta.extra_events {
                if *extra != decl.trigger.event && !decl.trigger.extra_events.contains(extra) {
                    decl.trigger.extra_events.push(*extra);
                }
            }
        }
        decl.filter = VmFilterPolicy::Handlers(Arc::new(VmHandlerSet {
            skill_def_id: skill.skill_def_id,
            active: None,
            limits: VmActiveLimits {
                once_per_turn: false,
                once_per_play_phase: false,
                hand_nonempty: false,
                equip_nonempty: false,
                hand_gt_hp: false,
                hand_gt_max_hp: false,
                face_up: false,
                owner_unlinked: false,
                mark_at_least: None,
            },
            triggers,
        }));
        for index in indices.into_iter().rev() {
            decls.remove(index);
        }
        decls.push(decl);
    }
    Ok(())
}

// ---- R05-E（裁决 28 合同 v3 §2）：三固定 compound 的四角色私有 helper 派生 ----

/// 调用包可用的三 compound 效果标签（rule-vm `effect_tag`，ABI 759）。
const R05_COMPOUND_TIQI: u16 = 781;
const R05_COMPOUND_BAOSHU: u16 = 782;
const R05_COMPOUND_DCBIHUO: u16 = 783;
/// 角色顺序固定（Limit/Combs/Next/Current；`vm_effects::R05_ROLES` 同序）。
const R05_ROLE_TAGS: [&str; 4] = ["limit", "combs", "next", "current"];

fn r05_roles_of(compound: u16) -> &'static [usize] {
    match compound {
        R05_COMPOUND_TIQI => &[0],
        R05_COMPOUND_BAOSHU => &[1],
        R05_COMPOUND_DCBIHUO => &[2, 3],
        _ => &[],
    }
}

fn r05_module_compounds(code: &[u8], found: &mut Vec<u16>) {
    let mut i = 0usize;
    while i < code.len() {
        let opc = code[i];
        i += 1;
        match opc {
            0x10 | 0x11 | 0x12 => i += 4,
            0x20 | 0x21 | 0x50 | 0x51 | 0x52 => i += 1,
            0x40 | 0x41 => i += 2,
            0x5A => {
                if i + 2 <= code.len() {
                    let tag = u16::from_le_bytes([code[i], code[i + 1]]);
                    if !r05_roles_of(tag).is_empty() && !found.contains(&tag) {
                        found.push(tag);
                    }
                }
                i += 2;
            }
            _ => {}
        }
    }
}

pub(super) fn derive_r05_helpers(
    pkg: &content_schema::ContentPackage,
    out: &mut Vec<engine_native::content::extension::VmSkillDecl>,
) -> Result<(), BridgeError> {
    use engine_native::rvm_bridge::BytecodeModule;
    for skill in &pkg.skill_index {
        let mut compounds: Vec<u16> = Vec::new();
        let module_names: Vec<&str> = pkg
            .trigger_index
            .iter()
            .filter(|t| t.skill_def_id == skill.skill_def_id)
            .map(|t| t.module.as_str())
            .collect();
        let active_module = pkg
            .active_index
            .iter()
            .find(|a| a.skill_def_id == skill.skill_def_id)
            .map(|a| a.module.as_str());
        for name in active_module.into_iter().chain(module_names) {
            if let Some(module) = pkg.modules.iter().find(|m| m.name == name) {
                r05_module_compounds(&module.code, &mut compounds);
            }
        }
        compounds.sort_unstable();
        for compound in compounds {
            for &role in r05_roles_of(compound) {
                let name = format!("{}#r05:{}", skill.name, R05_ROLE_TAGS[role]);
                if out.iter().any(|d| d.name == name) {
                    return Err(invalid(
                        "skill_index",
                        format!("重复身份/角色：派生 helper {name} 已存在"),
                    ));
                }
                let code = vec![0x01u8];
                let helper = BytecodeModule {
                    logic_hash: BytecodeModule::compute_logic_hash(&name, &code, 0, 0, 0),
                    name: name.clone(),
                    code,
                    local_count: 0,
                    max_stack: 1,
                    entry_pc: 0,
                    caps_mask: 0,
                };
                VmSkillProgram::new(helper.clone(), name.clone())
                    .map_err(|e| invalid("r05_helper", e.to_string()))?;
                out.push(engine_native::content::extension::VmSkillDecl {
                    declaration_hash: None,
                    presentation_binding: None,
                    name,
                    module: helper,
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
                        limits: VmActiveLimits {
                            once_per_turn: false,
                            once_per_play_phase: false,
                            hand_nonempty: false,
                            equip_nonempty: false,
                            hand_gt_hp: false,
                            hand_gt_max_hp: false,
                            face_up: false,
                            owner_unlinked: false,
                            mark_at_least: None,
                        },
                        triggers: Vec::new(),
                    })),
                    r05_identity: Some(engine_native::catalog::QualifiedSkillRef {
                        namespace: pkg.manifest.namespace.clone(),
                        package_id: pkg.manifest.package_id.clone(),
                        logical_skill_def_id: skill.skill_def_id,
                    }),
                });
            }
        }
    }
    Ok(())
}
