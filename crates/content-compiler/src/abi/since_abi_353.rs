use super::*;

pub(super) fn required_abi(ir: &RuleModuleIr, module_required_abi: u32) -> u32 {
    if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_353
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter
                        == "owner_source_target_distance_le_one_first_damage_to_target_this_turn"
                }) || skill.modifiers.iter().any(|modifier| {
                    modifier.target
                        == "sha_response_count_if_source_owner_target_distance_le_one"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_353
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_352
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.body.iter().any(|stmt| {
                        matches!(stmt, StmtIr::EffectRevealTopGainIfColor { .. })
                    })
                }) || skill.actives.iter().any(|active| {
                    active.body.iter().any(|stmt| {
                        matches!(stmt, StmtIr::EffectRevealTopGainIfColor { .. })
                    })
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_352
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_351
            || ir.skills.iter().any(|skill| {
                skill.triggers
                    .iter()
                    .any(|trigger| trigger.filter == "owner_is_target_current_turn")
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_351
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_350
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.body.iter().any(|stmt| {
                        matches!(stmt, StmtIr::EffectDiscardHandEquip { .. })
                    })
                }) || skill.actives.iter().any(|active| {
                    active.body.iter().any(|stmt| {
                        matches!(stmt, StmtIr::EffectDiscardHandEquip { .. })
                    })
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_350
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_349
            || ir.skills.iter().any(|skill| {
                skill.actives.iter().any(|active| {
                    active.body.iter().any(|stmt| {
                        matches!(stmt, StmtIr::EffectChoosePskuijiDiscard { .. })
                    })
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_349
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_348
            || ir.skills.iter().any(|skill| {
                skill.actives.iter().any(|active| {
                    active.body.iter().any(|stmt| {
                        matches!(stmt, StmtIr::EffectChooseScskuijiDiscard { .. })
                    })
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_348
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_347
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    matches!(
                        trigger.filter.as_str(),
                        "owner_source_or_target_second_damage_this_turn"
                            | "owner_source_or_target_first_damage_this_turn"
                    )
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_347
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_346
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_alive_other_dying_first_per_target"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_346
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_345
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    matches!(
                        trigger.filter.as_str(),
                        "owner_source_sha_single_trick_source_card"
                            | "owner_source_sha_single_equip_source_card"
                            | "owner_is_target_hand_above_hp_no_liubei_lord"
                    )
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_345
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_344
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter
                        == "owner_source_or_targeted_club_linked_exists_once_per_turn"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_344
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_343 {
            ADDITIVE_RULE_VM_ABI_VERSION_343
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_342
            || ir.skills.iter().any(|skill| {
                skill.modifiers.iter().any(|modifier| {
                    modifier.target == "judgment_rewrite_red_hand_equip_expansion"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_342
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_341
            || ir.skills.iter().any(|skill| {
                skill
                    .view_as
                    .iter()
                    .any(|view| view.filter == "owner_single_hand")
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_341
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_340
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter
                        == "owner_source_target_earlier_seat_hand_below_target_max_hp_once_per_turn"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_340
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_339
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_is_target_event_card_sha"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_339
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_338
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_is_target_other_hand_lte_owner"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_338
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_337
            || ir.skills.iter().any(|skill| {
                skill
                    .triggers
                    .iter()
                    .any(|trigger| trigger.event == "damage_prevented")
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_337
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_336
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_is_target_first_hp_change_turn_unique_trick_majority"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_336
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_335
            || ir.skills.iter().any(|skill| {
                skill
                    .modifiers
                    .iter()
                    .any(|modifier| modifier.selector.as_deref() == Some("sha_diamond"))
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_335
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_334 {
            ADDITIVE_RULE_VM_ABI_VERSION_334
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_333 {
            ADDITIVE_RULE_VM_ABI_VERSION_333
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_332
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_first_hand_discard_has_sha_in_discard"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_332
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_331 {
            ADDITIVE_RULE_VM_ABI_VERSION_331
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_330 {
            ADDITIVE_RULE_VM_ABI_VERSION_330
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_329
            || ir.skills.iter().any(|skill| {
                skill
                    .triggers
                    .iter()
                    .any(|trigger| trigger.filter == "owner_source_sha_current_play_phase")
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_329
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_328 {
            ADDITIVE_RULE_VM_ABI_VERSION_328
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_327 {
            ADDITIVE_RULE_VM_ABI_VERSION_327
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_326 {
            ADDITIVE_RULE_VM_ABI_VERSION_326
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_325
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_alive_gained_two_this_turn_can_compare_other"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_325
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_324 {
            ADDITIVE_RULE_VM_ABI_VERSION_324
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_323
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_alive_other_turn_ended_no_damage_other"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_323
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_322
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_source_other_damage_before_once_per_turn"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_322
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_321 {
            ADDITIVE_RULE_VM_ABI_VERSION_321
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_320 {
            ADDITIVE_RULE_VM_ABI_VERSION_320
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_319 {
            ADDITIVE_RULE_VM_ABI_VERSION_319
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_318 {
            ADDITIVE_RULE_VM_ABI_VERSION_318
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_317 {
            ADDITIVE_RULE_VM_ABI_VERSION_317
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_316 {
            ADDITIVE_RULE_VM_ABI_VERSION_316
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_315 {
            ADDITIVE_RULE_VM_ABI_VERSION_315
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_314 {
            ADDITIVE_RULE_VM_ABI_VERSION_314
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_313
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter
                        == "owner_source_first_target_black_card_any_target_other_discardable_hand_equip"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_313
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_312 {
            ADDITIVE_RULE_VM_ABI_VERSION_312
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_311
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_is_target_positive_draw_other_hand_nonempty"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_311
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_310 {
            ADDITIVE_RULE_VM_ABI_VERSION_310
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_309 {
            ADDITIVE_RULE_VM_ABI_VERSION_309
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_308
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_alive_non_multitarget_trick_targets_gt_one"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_308
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_307 {
            ADDITIVE_RULE_VM_ABI_VERSION_307
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_306
            || ir.skills.iter().any(|skill| {
                skill
                    .triggers
                    .iter()
                    .any(|trigger| trigger.filter == "owner_alive_source_target_was_linked")
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_306
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_305
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter
                        == "owner_source_tiesuo_target_alive_unlinked_hand_equip_nonempty"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_305
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_304
            || ir.skills.iter().any(|skill| {
                skill.actives.iter().any(|active| {
                    active
                        .selection
                        .as_ref()
                        .is_some_and(|selection| selection.same_suit)
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_304
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_303 {
            ADDITIVE_RULE_VM_ABI_VERSION_303
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_302
            || ir.skills.iter().any(|skill| {
                skill.actives.iter().any(|active| {
                    active.uses_per_turn_source.as_deref() == Some("living_rebel_count")
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_302
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_301
            || ir.skills.iter().any(|skill| {
                skill
                    .modifiers
                    .iter()
                    .any(|modifier| modifier.target == "card_number")
                    || skill.actives.iter().any(|active| {
                        active.uses_per_play_phase.is_some()
                            || active.selection.as_ref().is_some_and(|selection| {
                                selection.sum_effective_number_exact.is_some()
                            })
                    })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_301
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_300
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_is_target_draw_not_fixed_hp_one_or_hand_empty"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_300
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_299
            || ir.skills.iter().any(|skill| {
                skill
                    .triggers
                    .iter()
                    .any(|trigger| trigger.filter == "owner_alive_other_phase_end_hand_nonempty")
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_299
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_298
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_is_target_any_alive_discardable_by_owner_hand_equip"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_298
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_297 {
            ADDITIVE_RULE_VM_ABI_VERSION_297
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_296 {
            ADDITIVE_RULE_VM_ABI_VERSION_296
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_295 {
            ADDITIVE_RULE_VM_ABI_VERSION_295
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_294 {
            ADDITIVE_RULE_VM_ABI_VERSION_294
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_293
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_alive_play_phase_card_category_count_positive"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_293
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_292
            || ir.skills.iter().any(|skill| {
                skill.actives.iter().any(|active| {
                    active
                        .targets
                        .as_ref()
                        .is_some_and(|targets| targets.unused_by_skill_this_play_phase)
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_292
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_291
            || ir.skills.iter().any(|skill| {
                skill
                    .triggers
                    .iter()
                    .any(|trigger| trigger.filter == "owner_alive_is_source")
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_291
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_290
            || ir.skills.iter().any(|skill| {
                skill
                    .view_as
                    .iter()
                    .any(|view| view.source_zone.as_deref() == Some("hand_expansion"))
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_290
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_289
            || ir.skills.iter().any(|skill| {
                skill
                    .view_as
                    .iter()
                    .any(|view| view.filter == "owner_outside_turn")
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_289
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_288
            || ir.skills.iter().any(|skill| {
                skill.actives.iter().any(|active| {
                    active
                        .targets
                        .as_ref()
                        .is_some_and(|targets| targets.max_by_owner_hp)
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_288
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_287 {
            ADDITIVE_RULE_VM_ABI_VERSION_287
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_286 {
            ADDITIVE_RULE_VM_ABI_VERSION_286
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_285
            || ir.skills.iter().any(|skill| {
                skill.actives.iter().any(|active| {
                    active.selection.as_ref().is_some_and(|selection| {
                        selection.unused_category_by_owner_history_this_turn
                    })
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_285
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_284
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_source_outside_turn_card_used_or_responded"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_284
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_283
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_source_equip_used_skill_mark_below_seven"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_283
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_282 {
            ADDITIVE_RULE_VM_ABI_VERSION_282
        } else if ir.skills.iter().any(|skill| {
            skill.modifiers.iter().any(|modifier| {
                modifier.value_source.as_deref()
                    == Some("owner_current_turn_all_others_distance_at_most_one")
            })
        }) {
            ADDITIVE_RULE_VM_ABI_VERSION_281
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_280 {
            ADDITIVE_RULE_VM_ABI_VERSION_280
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_279
            || ir.skills.iter().any(|skill| {
                skill
                    .triggers
                    .iter()
                    .any(|trigger| trigger.filter == "owner_is_target_turn_damage_at_least_three")
                    || skill.actives.iter().any(|active| {
                        active
                            .targets
                            .as_ref()
                            .is_some_and(|targets| targets.max_by_owner_lost_hp)
                            || active
                                .selection
                                .as_ref()
                                .and_then(|selection| selection.filter.as_deref())
                                == Some("card_red")
                    })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_279
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_278 {
            ADDITIVE_RULE_VM_ABI_VERSION_278
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_277 {
            ADDITIVE_RULE_VM_ABI_VERSION_277
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_276
            || ir.skills.iter().any(|skill| {
                skill
                    .triggers
                    .iter()
                    .any(|trigger| trigger.filter == "owner_alive_other_responded_sha_gainable")
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_276
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_275 {
            ADDITIVE_RULE_VM_ABI_VERSION_275
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_274
            || ir.skills.iter().any(|skill| {
                skill
                    .view_as
                    .iter()
                    .any(|view| view.cost.as_deref() == Some("face_down_hand"))
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_274
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_273
            || ir.skills.iter().any(|skill| {
                skill
                    .view_as
                    .iter()
                    .any(|view| view.cost.as_deref() == Some("guhuo"))
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_273
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_272 {
            ADDITIVE_RULE_VM_ABI_VERSION_272
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_271 {
            ADDITIVE_RULE_VM_ABI_VERSION_271
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_270 {
            ADDITIVE_RULE_VM_ABI_VERSION_270
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_269 {
            ADDITIVE_RULE_VM_ABI_VERSION_269
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_268 {
            ADDITIVE_RULE_VM_ABI_VERSION_268
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_267 {
            ADDITIVE_RULE_VM_ABI_VERSION_267
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_266 {
            ADDITIVE_RULE_VM_ABI_VERSION_266
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_265 {
            ADDITIVE_RULE_VM_ABI_VERSION_265
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_264 {
            ADDITIVE_RULE_VM_ABI_VERSION_264
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_263 {
            ADDITIVE_RULE_VM_ABI_VERSION_263
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_262 {
            ADDITIVE_RULE_VM_ABI_VERSION_262
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_261 {
            ADDITIVE_RULE_VM_ABI_VERSION_261
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_260 {
            ADDITIVE_RULE_VM_ABI_VERSION_260
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_259 {
            ADDITIVE_RULE_VM_ABI_VERSION_259
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_258 {
            ADDITIVE_RULE_VM_ABI_VERSION_258
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_257 {
            ADDITIVE_RULE_VM_ABI_VERSION_257
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_256 {
            ADDITIVE_RULE_VM_ABI_VERSION_256
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_255 {
            ADDITIVE_RULE_VM_ABI_VERSION_255
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_254 {
            ADDITIVE_RULE_VM_ABI_VERSION_254
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_253 {
            ADDITIVE_RULE_VM_ABI_VERSION_253
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_252 {
            ADDITIVE_RULE_VM_ABI_VERSION_252
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_251 {
            ADDITIVE_RULE_VM_ABI_VERSION_251
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_250 {
            ADDITIVE_RULE_VM_ABI_VERSION_250
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_249
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter
                        == "owner_is_event_player_discard_pile_four_suits_below_alive_cap"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_249
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_248 {
            ADDITIVE_RULE_VM_ABI_VERSION_248
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_247 {
            ADDITIVE_RULE_VM_ABI_VERSION_247
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_246
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter
                    == "owner_alive_event_player_play_phase_same_suit_at_least_two_once_per_turn"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_246
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_245
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger
                        .filter
                        .starts_with("owner_source_play_skill_mark_mode_")
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_245
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_244 {
            ADDITIVE_RULE_VM_ABI_VERSION_244
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_243 {
            ADDITIVE_RULE_VM_ABI_VERSION_243
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_242 {
            ADDITIVE_RULE_VM_ABI_VERSION_242
        } else {
            super::since_abi_241::required_abi(ir, module_required_abi)
        }
}
