//! 内容包装载前的校验与按 ABI 启用的历史追踪（`load_package_inner` 的前两段）。

use super::*;

impl ContentRegistry {
    /// 装载前校验：逻辑哈希、重复装载、VM ABI 门槛与依赖闭包（只读）。
    /// 返回 `Ok(false)` 表示同一逻辑哈希的包已装载，调用方应直接成功返回（幂等）。
    pub(super) fn validate_package_for_load(&self, pkg: &ContentPackage) -> Result<bool, ContentLoadError> {
        pkg.validate_vm_effect_abi().map_err(ContentLoadError::Invalid)?;
        if pkg.manifest.logic_hash != pkg.compute_logic_hash() {
            return Err(ContentLoadError::Invalid(format!(
                "logic hash mismatch for {}/{}@{}",
                pkg.manifest.namespace, pkg.manifest.package_id, pkg.manifest.version
            )));
        }
        pkg.modules
            .iter()
            .try_for_each(content_schema::validate_vm_module_hash)
            .map_err(ContentLoadError::Invalid)?;
        if self
            .packages
            .iter()
            .any(|p| p.logic_hash == pkg.manifest.logic_hash)
        {
            return Ok(false);
        }
        if self.packages.iter().any(|loaded| {
            loaded.namespace == pkg.manifest.namespace
                && loaded.package_id == pkg.manifest.package_id
        }) {
            return Err(ContentLoadError::Conflict(format!(
                "package slot {}/{}",
                pkg.manifest.namespace, pkg.manifest.package_id
            )));
        }
        let dingpan_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(module, rule_vm::bytecode::effect_tag::RESOLVE_DINGPAN)
            })
            .map(|module| module.name.as_str())
            .collect();
        let tanlu_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(module, rule_vm::bytecode::effect_tag::RESOLVE_TANLU)
            })
            .map(|module| module.name.as_str())
            .collect();
        let same_color_judgment_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(
                    module,
                    rule_vm::bytecode::effect_tag::JUDGE_SAME_COLOR_GAIN_REPEAT,
                )
            })
            .map(|module| module.name.as_str())
            .collect();
        let current_trick_nullification_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(
                    module,
                    rule_vm::bytecode::effect_tag::CHOOSE_CURRENT_TRICK_TARGETS_TO_NULLIFY,
                )
            })
            .map(|module| module.name.as_str())
            .collect();
        let protected_black_judgment_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(
                    module,
                    rule_vm::bytecode::effect_tag::JUDGE_BLACK_GAIN_REPEAT_PROTECT_HAND_LIMIT,
                )
            })
            .map(|module| module.name.as_str())
            .collect();
        let all_resolved_sha_gift_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(
                    module,
                    rule_vm::bytecode::effect_tag::CHOOSE_GAIN_ALL_RESOLVED_SHA_SHAN_THEN_COLOR_REWARDS,
                )
            })
            .map(|module| module.name.as_str())
            .collect();
        let reduced_tuxi_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(
                    module,
                    rule_vm::bytecode::effect_tag::CHOOSE_OTHER_HAND_TARGETS_GAIN_ONE_REDUCE_DRAW,
                )
            })
            .map(|module| module.name.as_str())
            .collect();
        let rejianchu_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(
                    module,
                    rule_vm::bytecode::effect_tag::CHOOSE_DISCARD_TARGET_CARD_RESOLVE_CURRENT_SHA_NONBASIC_BONUS,
                )
            })
            .map(|module| module.name.as_str())
            .collect();
        let dcqiaomeng_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(module, rule_vm::bytecode::effect_tag::RESOLVE_DCQIAOMENG)
            })
            .map(|module| module.name.as_str())
            .collect();
        let repindi_modules: BTreeSet<_> = pkg
            .modules
            .iter()
            .filter(|module| {
                module_uses_effect_tag(
                    module,
                    rule_vm::bytecode::effect_tag::CHOOSE_TARGET_DRAW_OR_DISCARD_BY_SKILL_USE_COUNT_TOGGLE_LINK_IF_WOUNDED,
                )
            })
            .map(|module| module.name.as_str())
            .collect();
        if pkg.manifest.rule_vm_abi_version < 301
            && (pkg
                .modifier_index
                .iter()
                .any(|modifier| modifier.target == "card_number")
                || pkg.active_index.iter().any(|active| {
                    active.uses_per_play_phase.is_some()
                        || active
                            .selection
                            .as_ref()
                            .is_some_and(|selection| selection.sum_effective_number_exact.is_some())
                }))
        {
            return Err(ContentLoadError::Invalid(
                "ABI 301 content requires rule_vm_abi_version >= 301".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 302
            && (!dingpan_modules.is_empty()
                || pkg.active_index.iter().any(|active| {
                    active.uses_per_turn_source.as_deref() == Some("living_rebel_count")
                }))
        {
            return Err(ContentLoadError::Invalid(
                "ABI 302 content requires rule_vm_abi_version >= 302".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 303 && !tanlu_modules.is_empty() {
            return Err(ContentLoadError::Invalid(
                "ABI 303 content requires rule_vm_abi_version >= 303".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 304
            && pkg.active_index.iter().any(|active| {
                active
                    .selection
                    .as_ref()
                    .is_some_and(|selection| selection.same_suit)
            })
        {
            return Err(ContentLoadError::Invalid(
                "ABI 304 content requires rule_vm_abi_version >= 304".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 305
            && pkg.trigger_index.iter().any(|trigger| {
                trigger.filter == "owner_source_tiesuo_target_alive_unlinked_hand_equip_nonempty"
            })
        {
            return Err(ContentLoadError::Invalid(
                "ABI 305 content requires rule_vm_abi_version >= 305".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 306
            && pkg
                .trigger_index
                .iter()
                .any(|trigger| trigger.filter == "owner_alive_source_target_was_linked")
        {
            return Err(ContentLoadError::Invalid(
                "ABI 306 content requires rule_vm_abi_version >= 306".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 307 && !same_color_judgment_modules.is_empty() {
            return Err(ContentLoadError::Invalid(
                "ABI 307 content requires rule_vm_abi_version >= 307".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 308
            && (!current_trick_nullification_modules.is_empty()
                || pkg.trigger_index.iter().any(|trigger| {
                    trigger.filter == "owner_alive_non_multitarget_trick_targets_gt_one"
                }))
        {
            return Err(ContentLoadError::Invalid(
                "ABI 308 content requires rule_vm_abi_version >= 308".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 309 && !protected_black_judgment_modules.is_empty() {
            return Err(ContentLoadError::Invalid(
                "ABI 309 content requires rule_vm_abi_version >= 309".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 310 && !all_resolved_sha_gift_modules.is_empty() {
            return Err(ContentLoadError::Invalid(
                "ABI 310 content requires rule_vm_abi_version >= 310".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 311 && !reduced_tuxi_modules.is_empty() {
            return Err(ContentLoadError::Invalid(
                "ABI 311 content requires rule_vm_abi_version >= 311".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 312 && !rejianchu_modules.is_empty() {
            return Err(ContentLoadError::Invalid(
                "ABI 312 content requires rule_vm_abi_version >= 312".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 313 && !dcqiaomeng_modules.is_empty() {
            return Err(ContentLoadError::Invalid(
                "ABI 313 content requires rule_vm_abi_version >= 313".into(),
            ));
        }
        if pkg.manifest.rule_vm_abi_version < 314 && !repindi_modules.is_empty() {
            return Err(ContentLoadError::Invalid(
                "ABI 314 content requires rule_vm_abi_version >= 314".into(),
            ));
        }
        if pkg
            .trigger_index
            .iter()
            .any(|trigger| dingpan_modules.contains(trigger.module.as_str()))
            || dingpan_modules.iter().any(|module| {
                let mut found = false;
                let invalid = pkg
                    .active_index
                    .iter()
                    .filter(|active| active.module == **module)
                    .any(|active| {
                        found = true;
                        active.window != "play_phase"
                            || active.uses_per_turn_source.as_deref() != Some("living_rebel_count")
                            || !active.targets.as_ref().is_some_and(|targets| {
                                targets.min == 1
                                    && targets.max == 1
                                    && targets.allow_self
                                    && targets.has_equip
                            })
                    });
                !found || invalid
            })
        {
            return Err(ContentLoadError::Invalid(
                "Dingpan bytecode requires `active play_phase uses_per_turn living_rebel_count targets 1 1 has_equip`".into(),
            ));
        }
        if pkg
            .active_index
            .iter()
            .any(|active| tanlu_modules.contains(active.module.as_str()))
            || tanlu_modules.iter().any(|module| {
                let mut found = false;
                let invalid = pkg
                    .trigger_index
                    .iter()
                    .filter(|trigger| trigger.module == **module)
                    .any(|trigger| {
                        found = true;
                        trigger.event != "phase_begin"
                            || trigger.filter != "owner_alive_not_target"
                            || !trigger.optional
                            || !trigger.choice_actor.is_owner()
                    });
                !found || invalid
            })
        {
            return Err(ContentLoadError::Invalid(
                "Tanlu bytecode requires `trigger phase_begin owner_alive_not_target optional`"
                    .into(),
            ));
        }
        if pkg
            .active_index
            .iter()
            .any(|active| current_trick_nullification_modules.contains(active.module.as_str()))
            || current_trick_nullification_modules.iter().any(|module| {
                let mut found = false;
                let invalid = pkg
                    .trigger_index
                    .iter()
                    .filter(|trigger| trigger.module == **module)
                    .any(|trigger| {
                        found = true;
                        trigger.event != "card_used"
                            || trigger.filter != "owner_alive_non_multitarget_trick_targets_gt_one"
                            || trigger.optional
                    });
                !found || invalid
            })
        {
            return Err(ContentLoadError::Invalid(
                "current-trick target nullification bytecode requires `trigger card_used owner_alive_non_multitarget_trick_targets_gt_one`".into(),
            ));
        }
        if pkg
            .active_index
            .iter()
            .any(|active| protected_black_judgment_modules.contains(active.module.as_str()))
            || protected_black_judgment_modules.iter().any(|module| {
                let mut found = false;
                let invalid = pkg
                    .trigger_index
                    .iter()
                    .filter(|trigger| trigger.module == **module)
                    .any(|trigger| {
                        found = true;
                        trigger.event != "phase_prepare"
                            || trigger.filter != "owner_is_target"
                            || !trigger.optional
                    });
                !found || invalid
            })
        {
            return Err(ContentLoadError::Invalid(
                "protected black-judgment bytecode requires `trigger phase_prepare owner_is_target optional`".into(),
            ));
        }
        if pkg
            .active_index
            .iter()
            .any(|active| reduced_tuxi_modules.contains(active.module.as_str()))
            || reduced_tuxi_modules.iter().any(|module| {
                let mut found = false;
                let invalid = pkg
                    .trigger_index
                    .iter()
                    .filter(|trigger| trigger.module == **module)
                    .any(|trigger| {
                        found = true;
                        trigger.event != "phase_draw"
                            || trigger.filter != "owner_is_target_positive_draw_other_hand_nonempty"
                            || !trigger.optional
                            || !trigger.choice_actor.is_owner()
                    });
                !found || invalid
            })
        {
            return Err(ContentLoadError::Invalid(
                "reduced Tuxi bytecode requires `trigger phase_draw owner_is_target_positive_draw_other_hand_nonempty optional`".into(),
            ));
        }
        if pkg
            .active_index
            .iter()
            .any(|active| rejianchu_modules.contains(active.module.as_str()))
            || rejianchu_modules.iter().any(|module| {
                let mut found = false;
                let invalid = pkg
                    .trigger_index
                    .iter()
                    .filter(|trigger| trigger.module == **module)
                    .any(|trigger| {
                        found = true;
                        trigger.event != "card_targeted"
                            || trigger.filter
                                != "owner_source_sha_target_alive_has_discardable_hand_equip"
                            || !trigger.optional
                            || !trigger.choice_actor.is_owner()
                    });
                !found || invalid
            })
        {
            return Err(ContentLoadError::Invalid(
                "Rejianchu bytecode requires `trigger card_targeted owner_source_sha_target_alive_has_discardable_hand_equip optional`".into(),
            ));
        }
        if pkg
            .active_index
            .iter()
            .any(|active| dcqiaomeng_modules.contains(active.module.as_str()))
            || dcqiaomeng_modules.iter().any(|module| {
                let mut found = false;
                let invalid = pkg
                    .trigger_index
                    .iter()
                    .filter(|trigger| trigger.module == **module)
                    .any(|trigger| {
                        found = true;
                        trigger.event != "card_targeted"
                            || trigger.filter
                                != "owner_source_first_target_black_card_any_target_other_discardable_hand_equip"
                            || !trigger.optional
                            || !trigger.choice_actor.is_owner()
                    });
                !found || invalid
            })
        {
            return Err(ContentLoadError::Invalid(
                "Dcqiaomeng bytecode requires `trigger card_targeted owner_source_first_target_black_card_any_target_other_discardable_hand_equip optional`".into(),
            ));
        }
        if pkg
            .trigger_index
            .iter()
            .any(|trigger| repindi_modules.contains(trigger.module.as_str()))
            || repindi_modules.iter().any(|module| {
                let mut found = false;
                let invalid = pkg
                    .active_index
                    .iter()
                    .filter(|active| active.module == **module)
                    .any(|active| {
                        found = true;
                        active.window != "play_phase"
                            || !active.targets.as_ref().is_some_and(|targets| {
                                targets.min == 1
                                    && targets.max == 1
                                    && targets.allow_self
                                    && targets.unused_by_skill_this_play_phase
                            })
                            || !active.selection.as_ref().is_some_and(|selection| {
                                selection.zone == "hand_equip"
                                    && selection.min == 1
                                    && selection.max == 1
                                    && !selection.allow_cancel
                                    && selection.unused_category_by_skill_this_play_phase
                            })
                    });
                !found || invalid
            })
        {
            return Err(ContentLoadError::Invalid(
                "Repindi bytecode requires `active play_phase targets 1 1 unused_by_skill_this_play_phase select_he 1 1 unused_category_by_skill_this_play_phase`".into(),
            ));
        }
        if pkg.manifest.dependencies.is_empty() {
            let available_skills: BTreeSet<_> = self
                .package_skills
                .keys()
                .copied()
                .chain(pkg.skill_index.iter().map(|skill| skill.skill_def_id))
                .collect();
            validate_character_skills(&pkg.character_index, &available_skills)?;
        }
        Ok(true)
    }

    /// 按包声明的 VM ABI 版本启用对应的回合/事件历史追踪。
    pub(super) fn enable_package_tracking(&mut self, pkg: &ContentPackage) {
        if pkg.manifest.rule_vm_abi_version >= 42 {
            self.track_turn_hand_card_suit_history = true;
            self.track_play_card_category_history = true;
        }
        if pkg
            .trigger_index
            .iter()
            .any(|trigger| trigger.filter == "owner_alive_play_phase_card_category_count_positive")
        {
            self.track_play_card_category_history_all_users = true;
        }
        if pkg.manifest.rule_vm_abi_version >= 43 {
            self.track_sha_use_refund_provenance = true;
        }
        if pkg.manifest.rule_vm_abi_version >= 44 {
            self.track_opening_extra_play_lifecycle = true;
        }
        if pkg.manifest.rule_vm_abi_version >= 232 {
            self.track_turn_used_card_suit_history = true;
        }
        if pkg.manifest.rule_vm_abi_version >= 285 {
            self.track_owner_card_category_history = true;
        }
        if pkg.manifest.rule_vm_abi_version >= 347 {
            self.track_damage_taken_count = true;
        }
        if pkg.manifest.rule_vm_abi_version >= 353 {
            self.track_turn_damage_pair_counts = true;
        }
        if pkg
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == 2_581_852_991)
        {
            self.track_entered_dying_history = true;
        }
        if pkg.modules.iter().any(|module| {
            module_uses_effect_tag(
                module,
                rule_vm::bytecode::effect_tag::CHOOSE_LOSE_HP_USE_VIRTUAL_SHA_BY_BASIC_VARIANTS,
            )
        }) {
            self.track_turn_basic_card_variant_history = true;
        }
        if pkg.modules.iter().any(|module| {
            module_uses_effect_tag(module, rule_vm::bytecode::effect_tag::ALIGN_DEBATE_OPINIONS)
        }) {
            self.track_damaged_by_player_bitset = true;
        }
    }
}
