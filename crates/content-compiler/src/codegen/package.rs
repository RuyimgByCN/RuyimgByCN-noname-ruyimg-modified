use super::*;

fn compile_module_ir_inner(
    ir: &RuleModuleIr,
) -> Result<
    (
        Vec<BytecodeModule>,
        Vec<TriggerEmit>,
        Vec<ModifierEmit>,
        Vec<ActiveEmit>,
        Vec<ViewAsEmit>,
        Vec<SourceMap>,
        u32,
    ),
    CompileError,
> {
    let mut modules = Vec::new();
    let mut triggers = Vec::new();
    let mut modifiers = Vec::new();
    let mut actives = Vec::new();
    let mut view_as = Vec::new();
    let mut maps = Vec::new();
    let mut required_abi = 3;
    let mut trigger_def_seq = 1u32;

    for skill in &ir.skills {
        for m in &skill.modifiers {
            if m.value_source.as_deref() == Some("owner_skill_state_u32") {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_522);
            }
            if m.value_source.as_deref() == Some("owner_skill_state_u32_negative") {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_612);
            }
            if m.value_source.as_deref() == Some("owner_skill_pile_count") {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_566);
            }
            if m.value_source.as_deref() == Some("queried_player_qun_owner_lord_alive") {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_553);
            }
            if m.target == "judgment_rewrite_red_hand_equip_expansion" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_342);
            }
            if m.target == "card_number" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_301);
            }
            if m.target == "card_gain_protected" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_300);
            }
            if m.target == "sha_distance_unlimited_if_target_distance_lte_rank" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_223);
            }
            if m.value_source.as_deref().is_some_and(|source| {
                source.starts_with("owner_skill_play_phase_fire_count_below_")
            }) {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_220);
            }
            if m.value_source.as_deref() == Some("owner_skill_use_count") {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_143);
            }
            if m.target == "link_loss_cancel"
                || m.value_source.as_deref()
                    == Some("queried_player_linked_first_living_skill_owner")
                || m.selector.as_deref() == Some("sha_thunder")
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_213);
            }
            if matches!(
                m.target.as_str(),
                "global_draw_count" | "global_sha_use_limit"
            ) || m
                .value_source
                .as_deref()
                .is_some_and(|source| source.starts_with("queried_player_mark_positive_"))
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_215);
            }
            if m.target
                .starts_with("judgment_rewrite_hand_equip_cost_mark_")
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_218);
            }
            modifiers.push(ModifierEmit {
                skill_def_id: skill.def_id,
                target: m.target.clone(),
                selector: m.selector.clone(),
                mode: m.mode.clone(),
                value_source: m.value_source.clone(),
                value: m.value,
            });
            let _ = m as &ModifierIr;
        }
        for (trigger_index, tr) in skill.triggers.iter().enumerate() {
            if matches!(
                tr.filter.as_str(),
                "owner_alive_other_turn_discarded_recorded_potfuan_suit"
                    | "owner_source_sha_resolved_potfuan_dealt_damage"
            ) {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_525);
            }
            if tr.filter == "owner_discarded_sha_to_discard" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_522);
            }
            if tr.filter == "owner_source_or_targeted_club_linked_exists_once_per_turn" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_344);
            }
            if tr.filter
                == "owner_source_target_earlier_seat_hand_below_target_max_hp_once_per_turn"
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_340);
            }
            if tr.filter == "owner_is_target_event_card_sha" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_339);
            }
            if tr.filter == "owner_alive_gained_two_this_turn_can_compare_other" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_325);
            }
            if tr.filter == "owner_is_target_draw_not_fixed_hp_one_or_hand_empty" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_300);
            }
            if tr.filter == "owner_is_target_draw_not_replaced" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_225);
            }
            if tr.filter == "owner_is_target_judge_nonempty" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_224);
            }
            if tr.filter == "owner_source_in_turn_target_unharmed_card_actions_ge_target_hp" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_749);
            }
            if tr.filter == "owner_is_target_source_hp_gt_owner_or_source_damaged_owner"
                || tr.filter == "owner_is_target_owner_hp_gt_source_or_source_damaged_owner"
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_750);
            }
            if tr.filter == "owner_is_target_sanshou_type_tracked" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_751);
            }
            if tr.filter == "owner_source_sha_target_hand_lte_owner_or_hp_gte_owner" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_223);
            }
            let duplicate_event = skill
                .triggers
                .iter()
                .filter(|candidate| candidate.event == tr.event)
                .count()
                > 1;
            let mod_name = if duplicate_event {
                format!("{}_{}_{}", skill.name, tr.event, trigger_index)
            } else {
                format!("{}_{}", skill.name, tr.event)
            };
            let (module, smap, entry, module_abi) = compile_body(&mod_name, &tr.body)?;
            required_abi = required_abi.max(module_abi);
            triggers.push(TriggerEmit {
                trigger_def_id: trigger_def_seq,
                skill_def_id: skill.def_id,
                event: tr.event.clone(),
                filter: tr.filter.clone(),
                optional: tr.optional,
                choice_actor: tr.choice_actor,
                module: mod_name,
                entry_pc: entry,
            });
            trigger_def_seq += 1;
            modules.push(module);
            maps.push(smap);
            let _ = tr as &TriggerIr;
        }
        for (i, ac) in skill.actives.iter().enumerate() {
            let mod_name = format!("{}_active_{}_{}", skill.name, ac.window, i);
            let (module, smap, entry, module_abi) = compile_body(&mod_name, &ac.body)?;
            required_abi = required_abi.max(module_abi);
            if ac.uses_per_play_phase.is_some()
                || ac
                    .selection
                    .as_ref()
                    .is_some_and(|selection| selection.sum_effective_number_exact.is_some())
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_301);
            }
            if ac.uses_per_turn_source.as_deref() == Some("living_rebel_count") {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_302);
            }
            if ac.hand_missing_gaintag.is_some() {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_745);
            }
            if ac.hand_limit_positive {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_143);
            }
            if ac.mark_at_least.is_some() {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_207);
            }
            if ac.owner_missing_skill.is_some() {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_234);
            }
            if ac.owner_unlinked {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_611);
            }
            if ac
                .targets
                .as_ref()
                .is_some_and(|targets| targets.linked || targets.max_by_owner_mark.is_some())
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_208);
            }
            if ac
                .targets
                .as_ref()
                .is_some_and(|targets| targets.has_gainable_equip)
                || ac
                    .selection
                    .as_ref()
                    .is_some_and(|selection| selection.filter.as_deref() == Some("card_nonbasic"))
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_327);
            }
            if ac
                .selection
                .as_ref()
                .is_some_and(|selection| selection.filter.as_deref() == Some("card_non_equip"))
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_497);
            }
            if ac
                .targets
                .as_ref()
                .is_some_and(|targets| targets.has_gainable_hand_equip)
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_355);
            }
            actives.push(ActiveEmit {
                skill_def_id: skill.def_id,
                window: ac.window.clone(),
                module: mod_name,
                entry_pc: entry,
                targets: ac.targets.clone(),
                uses_per_turn: ac.uses_per_turn,
                uses_per_turn_source: ac.uses_per_turn_source.clone(),
                uses_per_play_phase: ac.uses_per_play_phase,
                mark_at_least: ac.mark_at_least,
                owner_missing_skill: ac.owner_missing_skill,
                hand_gt_hp: ac.hand_gt_hp,
                hand_gt_max_hp: ac.hand_gt_max_hp,
                hand_limit_positive: ac.hand_limit_positive,
                equip_nonempty: ac.equip_nonempty,
                hand_nonempty: ac.hand_nonempty,
                face_up: ac.face_up,
                hand_missing_gaintag: ac.hand_missing_gaintag,
                owner_unlinked: ac.owner_unlinked,
                once_per_game: ac.once_per_game,
                selection: ac.selection.clone(),
            });
            modules.push(module);
            maps.push(smap);
            let _ = ac as &ActiveIr;
        }
        for v in &skill.view_as {
            if v.no_use_count || v.target_lacks_source_suit_unrespondable {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_521);
            }
            if v.cost.as_deref() == Some("hand_equip_mark") || v.as_kind == "trick" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_502);
            }
            if v.filter == "owner_single_hand" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_341);
            }
            if v.owner_hp_min_one_cards {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_236);
            }
            if v.owner_basic_uses_plus_one_cards {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_741);
            }
            if v.max_cards == u8::MAX {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_742);
            }
            if v.once_per_turn_per_branch {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_744);
            }
            if v.min_cards > 1 || v.as_kind == "wanjian" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_221);
            }
            if v.filter == "card_trick" || v.nature.as_deref() == Some("thunder") {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_213);
            }
            view_as.push(ViewAsEmit {
                skill_def_id: skill.def_id,
                as_kind: v.as_kind.clone(),
                window: v.window.clone(),
                filter: v.filter.clone(),
                source_zone: v.source_zone.clone(),
                uses_per_turn: v.uses_per_turn,
                ignore_distance: v.ignore_distance,
                red_unrespondable: v.red_unrespondable,
                black_no_use_count: v.black_no_use_count,
                no_use_count: v.no_use_count,
                target_lacks_source_suit_unrespondable: v.target_lacks_source_suit_unrespondable,
                refund_use_count_if_no_damage: v.refund_use_count_if_no_damage,
                min_cards: v.min_cards,
                max_cards: v.max_cards,
                owner_hp_min_one_cards: v.owner_hp_min_one_cards,
                same_suit: v.same_suit,
                owner_basic_uses_plus_one_cards: v.owner_basic_uses_plus_one_cards,
                once_per_turn_per_branch: v.once_per_turn_per_branch,
                nature: v.nature.clone(),
                multi_card_bonus: v.multi_card_bonus.clone(),
                branch: v.branch,
                once_per_round: v.once_per_round,
                next_branch: v.next_branch,
                cost: v.cost.clone(),
                mark_cost: v.mark_cost,
                post_draw: v.post_draw,
            });
        }
    }

    Ok((
        modules,
        triggers,
        modifiers,
        actives,
        view_as,
        maps,
        required_abi,
    ))
}

/// Compile IR to modules + indexes.
pub fn compile_module_ir(
    ir: &RuleModuleIr,
) -> Result<
    (
        Vec<BytecodeModule>,
        Vec<TriggerEmit>,
        Vec<ModifierEmit>,
        Vec<ActiveEmit>,
        Vec<ViewAsEmit>,
        Vec<SourceMap>,
    ),
    CompileError,
> {
    let (modules, triggers, modifiers, actives, view_as, maps, _) = compile_module_ir_inner(ir)?;
    Ok((modules, triggers, modifiers, actives, view_as, maps))
}

pub(crate) fn compile_module_ir_with_required_abi(
    ir: &RuleModuleIr,
) -> Result<
    (
        Vec<BytecodeModule>,
        Vec<TriggerEmit>,
        Vec<ModifierEmit>,
        Vec<ActiveEmit>,
        Vec<ViewAsEmit>,
        Vec<SourceMap>,
        u32,
    ),
    CompileError,
> {
    compile_module_ir_inner(ir)
}

fn compile_body(
    name: &str,
    body: &[StmtIr],
) -> Result<(BytecodeModule, SourceMap, u32, u32), CompileError> {
    let mut asm = Assembler::new();
    asm.emit_stmts(body)?;
    asm.ops.push(Op::Halt);

    let caps_mask = required_caps(&asm.ops);
    let mut code = assemble(&asm.ops);
    asm.patch_jumps_in_bytes(&mut code);
    let logic_hash = BytecodeModule::compute_logic_hash(name, &code, asm.local_count, caps_mask, 0);
    let module = BytecodeModule {
        name: name.into(),
        code,
        local_count: asm.local_count,
        max_stack: asm.max_stack.max(8),
        entry_pc: 0,
        logic_hash,
        caps_mask,
    };
    let required_abi = asm.required_abi;
    let smap = SourceMap {
        module: name.into(),
        entries: asm.smap,
    };
    Ok((module, smap, 0, required_abi))
}
