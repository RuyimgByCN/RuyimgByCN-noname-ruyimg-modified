//! P3-E1 Track `L11VmFixA`: stripped `taixu-upstream-liuli` VM skills migrated to host behaviors.

/// The 31 `taixu` cross/suspect skills migrated to `HostBehavior::L11VmFixA`.
///
/// (`xiangle` reuses `HostBehavior::U01DddBatchD`; `jsrgbahu` stays on the `taixu` VM.)
pub const L11_TAIXU_VM_FIX_A_SKILLS: [&str; 31] = [
    "miji",
    "fuji",
    "wurong",
    "fulin",
    "refankui",
    "yeyan",
    "fenwei",
    "kuangfu",
    "langmie",
    "wushen",
    "yingjian",
    "qiaomeng",
    "shenxian",
    "jiezi",
    "qiangwu",
    "lianpo",
    "bihuo",
    "chunlao",
    "drlt_xiongluan",
    "fenming",
    "fenyin",
    "guzheng",
    "gzyinghun",
    "jianying",
    "kaikang",
    "kunfen",
    "lihuo",
    "manji",
    "mingce",
    "nsyangwu",
    "nzry_longnu",
];

/// Returns `true` if a `taixu-upstream-liuli` VM skill should be stripped in favor of host bindings.
pub fn is_stripped_vm_skill(name: &str) -> bool {
    matches!(name, "xiangle" | "nzry_longnu_1" | "nzry_longnu_2")
        || L11_TAIXU_VM_FIX_A_SKILLS.contains(&name)
}

pub(crate) fn make_vm_skill_decl(
    name_str: &str,
    dispatch_id: u32,
    idx: usize,
) -> Option<engine_native::content::extension::VmSkillDecl> {
    let (event, inner_filter) = crate::classify_skill_trigger(name_str)?;
    let mod_name = format!("{name_str}#d34p9");
    let mut code = Vec::with_capacity(14);
    code.push(0x57); // HOST_SKILL
    code.push(0x10); // CONST_I32
    code.extend_from_slice(&(crate::DECISION34_STORAGE_DISPATCH_KEY as i32).to_le_bytes());
    code.push(0x10); // CONST_I32
    code.extend_from_slice(&(dispatch_id as i32).to_le_bytes());
    code.push(0x5A); // HOST_EFFECT_EXT
    code.extend_from_slice(&649u16.to_le_bytes()); // STORAGE_SET = 649
    code.push(0x01); // HALT
    let module = engine_native::rvm_bridge::BytecodeModule {
        logic_hash: engine_native::rvm_bridge::BytecodeModule::compute_logic_hash(
            &mod_name, &code, 0, 1, 0,
        ),
        name: mod_name,
        code,
        local_count: 0,
        max_stack: 4,
        entry_pc: 0,
        caps_mask: 1,
    };
    let skill_def_id = dispatch_id.wrapping_add(idx as u32);
    let extra_events = if name_str == "dddxiaxing" {
        vec![engine_native::event::EventKind::Dying]
    } else {
        Vec::new()
    };
    Some(engine_native::content::extension::VmSkillDecl {
        declaration_hash: None,
        presentation_binding: None,
        name: name_str.to_string(),
        module,
        trigger: engine_native::catalog::TriggerMeta {
            event,
            timing: engine_native::event::Timing::Normal,
            priority: 15,
            order_selectable: false,
            optional: false,
            extra_events,
        },
        filter: engine_native::rvm_bridge::VmFilterPolicy::Declared {
            skill_def_id,
            hand_limit_base_max_hp: false,
            choice_actor: engine_native::rvm_bridge::VmChoiceActor::Owner,
            modifiers: Vec::new(),
            inner: Box::new(inner_filter),
        },
        r05_identity: None,
    })
}
