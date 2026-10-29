use super::*;

pub(super) fn required_abi(ir: &RuleModuleIr, module_required_abi: u32) -> u32 {
    if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_513 {
        ADDITIVE_RULE_VM_ABI_VERSION_513
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_512 {
        ADDITIVE_RULE_VM_ABI_VERSION_512
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_511 {
        ADDITIVE_RULE_VM_ABI_VERSION_511
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_510
        || ir.skills.iter().any(|skill| {
            skill.view_as.iter().any(|view| {
                matches!(view.filter.as_str(), "damage_trick" | "color_ne_shuangxiong")
            }) || skill.triggers.iter().any(|trigger| {
                matches!(
                    trigger.filter.as_str(),
                    "owner_source_first_single_target_sha_mbfeijing_twice_per_turn"
                        | "owner_source_damage_before_mbfeijing_extra"
                        | "owner_source_sha_resolved_original_single_mbxiaoge"
                        | "owner_alive_other_discarded_in_discard_phase"
                )
            })
        }) {
            ADDITIVE_RULE_VM_ABI_VERSION_510
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_509
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter
                        == "owner_is_target_can_move_field_card_mbyuanmo_twice_per_turn"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_509
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_508 {
            ADDITIVE_RULE_VM_ABI_VERSION_508
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_507
            || ir.skills.iter().any(|skill| {
                skill.modifiers.iter().any(|modifier| {
                    modifier.target == "first_hand_card_suit_use_unlimited"
                }) || skill.triggers.iter().any(|trigger| {
                    matches!(
                        trigger.filter.as_str(),
                        "owner_source_damage_mbxiongtu_armed"
                            | "owner_phase_play_end_mbxiongtu_state"
                    )
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_507
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_506 {
            ADDITIVE_RULE_VM_ABI_VERSION_506
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_505 {
            ADDITIVE_RULE_VM_ABI_VERSION_505
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_504 {
            ADDITIVE_RULE_VM_ABI_VERSION_504
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_503 {
            ADDITIVE_RULE_VM_ABI_VERSION_503
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_502 {
            ADDITIVE_RULE_VM_ABI_VERSION_502
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_501 {
            ADDITIVE_RULE_VM_ABI_VERSION_501
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_500 {
            ADDITIVE_RULE_VM_ABI_VERSION_500
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_499 {
            ADDITIVE_RULE_VM_ABI_VERSION_499
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_498 {
            ADDITIVE_RULE_VM_ABI_VERSION_498
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_497 {
            ADDITIVE_RULE_VM_ABI_VERSION_497
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_496 {
            ADDITIVE_RULE_VM_ABI_VERSION_496
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_495 {
            ADDITIVE_RULE_VM_ABI_VERSION_495
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_494 {
            ADDITIVE_RULE_VM_ABI_VERSION_494
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_493 {
            ADDITIVE_RULE_VM_ABI_VERSION_493
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_492 {
            ADDITIVE_RULE_VM_ABI_VERSION_492
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_491 {
            ADDITIVE_RULE_VM_ABI_VERSION_491
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_490
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter
                        == "owner_source_sha_or_juedou_single_target_hand_nonempty"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_490
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_487
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter
                        == "owner_alive_neighbor_gained_cards_outside_own_draw_once_per_turn"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_487
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_486 {
            ADDITIVE_RULE_VM_ABI_VERSION_486
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_485 {
            ADDITIVE_RULE_VM_ABI_VERSION_485
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_484 {
            ADDITIVE_RULE_VM_ABI_VERSION_484
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_483 {
            ADDITIVE_RULE_VM_ABI_VERSION_483
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_482 {
            ADDITIVE_RULE_VM_ABI_VERSION_482
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_481 {
            ADDITIVE_RULE_VM_ABI_VERSION_481
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_480 {
            ADDITIVE_RULE_VM_ABI_VERSION_480
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_479 {
            ADDITIVE_RULE_VM_ABI_VERSION_479
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_478 {
            ADDITIVE_RULE_VM_ABI_VERSION_478
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_477 {
            ADDITIVE_RULE_VM_ABI_VERSION_477
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_476 {
            ADDITIVE_RULE_VM_ABI_VERSION_476
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_475 {
            ADDITIVE_RULE_VM_ABI_VERSION_475
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_474 {
            ADDITIVE_RULE_VM_ABI_VERSION_474
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_473 {
            ADDITIVE_RULE_VM_ABI_VERSION_473
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_472 {
            ADDITIVE_RULE_VM_ABI_VERSION_472
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_471 {
            ADDITIVE_RULE_VM_ABI_VERSION_471
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_470 {
            ADDITIVE_RULE_VM_ABI_VERSION_470
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_469 {
            ADDITIVE_RULE_VM_ABI_VERSION_469
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_468 {
            ADDITIVE_RULE_VM_ABI_VERSION_468
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_467 {
            ADDITIVE_RULE_VM_ABI_VERSION_467
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_466 {
            ADDITIVE_RULE_VM_ABI_VERSION_466
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_465 {
            ADDITIVE_RULE_VM_ABI_VERSION_465
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_464
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_source_sha_once_per_turn"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_464
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_463 {
            ADDITIVE_RULE_VM_ABI_VERSION_463
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_462
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_source_wuxie_used"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_462
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_461 {
            ADDITIVE_RULE_VM_ABI_VERSION_461
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_460 {
            ADDITIVE_RULE_VM_ABI_VERSION_460
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_459 {
            ADDITIVE_RULE_VM_ABI_VERSION_459
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_458 {
            ADDITIVE_RULE_VM_ABI_VERSION_458
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_457 {
            ADDITIVE_RULE_VM_ABI_VERSION_457
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_456 {
            ADDITIVE_RULE_VM_ABI_VERSION_456
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_455 {
            ADDITIVE_RULE_VM_ABI_VERSION_455
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_454
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_source_or_within_distance_one_sha_or_juedou"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_454
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_453 {
            ADDITIVE_RULE_VM_ABI_VERSION_453
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_452
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_is_target_sha_or_trick"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_452
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_451 {
            ADDITIVE_RULE_VM_ABI_VERSION_451
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_450 {
            ADDITIVE_RULE_VM_ABI_VERSION_450
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_427
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_alive_other_single_target_juedou_or_red_sha"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_427
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_426 {
            ADDITIVE_RULE_VM_ABI_VERSION_426
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_425
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter.starts_with("owner_alive_card_targeted_marked_peer_")
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_425
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_424 {
            ADDITIVE_RULE_VM_ABI_VERSION_424
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_423 {
            ADDITIVE_RULE_VM_ABI_VERSION_423
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_422 {
            ADDITIVE_RULE_VM_ABI_VERSION_422
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_421 {
            ADDITIVE_RULE_VM_ABI_VERSION_421
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_420
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_source_sha_or_juedou_single_target"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_420
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_419 {
            ADDITIVE_RULE_VM_ABI_VERSION_419
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_418 {
            ADDITIVE_RULE_VM_ABI_VERSION_418
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_417 {
            ADDITIVE_RULE_VM_ABI_VERSION_417
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_416 {
            ADDITIVE_RULE_VM_ABI_VERSION_416
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_415 {
            ADDITIVE_RULE_VM_ABI_VERSION_415
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_414 {
            ADDITIVE_RULE_VM_ABI_VERSION_414
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_413 {
            ADDITIVE_RULE_VM_ABI_VERSION_413
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_412 {
            ADDITIVE_RULE_VM_ABI_VERSION_412
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_411 {
            ADDITIVE_RULE_VM_ABI_VERSION_411
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_410
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_source_sha_first_target"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_410
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_409 {
            ADDITIVE_RULE_VM_ABI_VERSION_409
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_408
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger
                        .filter
                        .starts_with("owner_source_target_mark_positive_")
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_408
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_407
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_source_first_sha_this_play_phase"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_407
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_406 {
            ADDITIVE_RULE_VM_ABI_VERSION_406
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_405
            || ir.skills.iter().any(|skill| {
                skill
                    .triggers
                    .iter()
                    .any(|trigger| trigger.filter == "owner_alive_lost_hand_this_turn")
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_405
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_404 {
            ADDITIVE_RULE_VM_ABI_VERSION_404
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_403 {
            ADDITIVE_RULE_VM_ABI_VERSION_403
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_402
            || ir.skills.iter().any(|skill| {
                skill.actives.iter().any(|active| {
                    active
                        .targets
                        .as_ref()
                        .is_some_and(|targets| targets.hp_lte_owner)
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_402
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_401 {
            ADDITIVE_RULE_VM_ABI_VERSION_401
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_400 {
            ADDITIVE_RULE_VM_ABI_VERSION_400
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_399 {
            ADDITIVE_RULE_VM_ABI_VERSION_399
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_398 {
            ADDITIVE_RULE_VM_ABI_VERSION_398
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_397 {
            ADDITIVE_RULE_VM_ABI_VERSION_397
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_396 {
            ADDITIVE_RULE_VM_ABI_VERSION_396
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_395 {
            ADDITIVE_RULE_VM_ABI_VERSION_395
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_394 {
            ADDITIVE_RULE_VM_ABI_VERSION_394
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_393 {
            ADDITIVE_RULE_VM_ABI_VERSION_393
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_392
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    matches!(
                        trigger.filter.as_str(),
                        "owner_alive_other_source_single_sha_target_other_owner_in_attack_range"
                            | "owner_alive_other_source_sha_resolved_twxiaolian_redirect_has_hand_equip"
                    )
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_392
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_391 {
            ADDITIVE_RULE_VM_ABI_VERSION_391
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_390 {
            ADDITIVE_RULE_VM_ABI_VERSION_390
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_389 {
            ADDITIVE_RULE_VM_ABI_VERSION_389
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_388 {
            ADDITIVE_RULE_VM_ABI_VERSION_388
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_387
            || ir.skills.iter().any(|skill| skill.conversion) {
            ADDITIVE_RULE_VM_ABI_VERSION_387
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_386
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_targeted_by_other_card_once_per_turn"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_386
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_385
            || ir.skills.iter().any(|skill| {
                skill.view_as.iter().any(|view_as| {
                    view_as.cost.as_deref() == Some("draw_one_outside_turn_empty_hand")
                }) || skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_dying_other_hand_equip_nonempty_once_per_game"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_385
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_384
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    matches!(
                        trigger.filter.as_str(),
                        "owner_source_first_target_damage_card_once_per_turn"
                            | "owner_outside_turn_discarded_red_cards_in_discard"
                    )
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_384
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_383
            || ir.skills.iter().any(|skill| {
                skill.view_as.iter().any(|view_as| {
                    view_as.cost.as_deref() == Some("swap_lower_hand")
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_383
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_382
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter
                        == "owner_source_sha_target_twzhenxi_available_once_per_turn"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_382
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_381
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_alive_other_phase_play_end_twzhengjian_condition"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_381
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_380
            || ir.skills.iter().any(|skill| {
                skill.modifiers.iter().any(|modifier| {
                    modifier.target == "hand_card_gained_this_turn_only"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_380
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_379
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    matches!(
                        trigger.filter.as_str(),
                        "owner_source_damage_target_distance_at_most_one_empty_equip_slot_unused"
                            | "owner_source_first_sha_target_this_turn"
                    )
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_379
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_378 {
            ADDITIVE_RULE_VM_ABI_VERSION_378
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_377 {
            ADDITIVE_RULE_VM_ABI_VERSION_377
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_376 {
            ADDITIVE_RULE_VM_ABI_VERSION_376
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_375 {
            ADDITIVE_RULE_VM_ABI_VERSION_375
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_374 {
            ADDITIVE_RULE_VM_ABI_VERSION_374
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_373 {
            ADDITIVE_RULE_VM_ABI_VERSION_373
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_372 {
            ADDITIVE_RULE_VM_ABI_VERSION_372
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_371 {
            ADDITIVE_RULE_VM_ABI_VERSION_371
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_370
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_target_physical_sha_source_hp_gt_owner"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_370
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_369 {
            ADDITIVE_RULE_VM_ABI_VERSION_369
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_368 {
            ADDITIVE_RULE_VM_ABI_VERSION_368
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_367
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_alive_other_phase_prepare_can_compare_owner"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_367
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_366
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_alive_lost_weapon_outside_turn"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_366
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_365
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    matches!(
                        trigger.filter.as_str(),
                        "owner_alive_twruilian_round_unprompted"
                            | "owner_alive_twruilian_selected_target_discarded"
                            | "owner_alive_twruilian_selected_target_turn_ended"
                    )
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_365
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_364
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    matches!(
                        trigger.filter.as_str(),
                        "owner_alive_other_twkujian_used_or_responded"
                            | "owner_alive_other_twkujian_lost"
                    )
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_364
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_363
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter
                        == "owner_alive_draw_player_owner_or_min_hp_twjiaohua_available"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_363
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_362 {
            ADDITIVE_RULE_VM_ABI_VERSION_362
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_361 {
            ADDITIVE_RULE_VM_ABI_VERSION_361
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_360 {
            ADDITIVE_RULE_VM_ABI_VERSION_360
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_359 {
            ADDITIVE_RULE_VM_ABI_VERSION_359
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_358
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_target_hand_empty_elemental_or_trick_damage_before"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_358
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_357
            || ir.skills.iter().any(|skill| {
                skill.triggers.iter().any(|trigger| {
                    trigger.filter == "owner_first_transformed_sha_used_this_turn"
                })
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_357
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_356 {
            ADDITIVE_RULE_VM_ABI_VERSION_356
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_355
            || ir.skills.iter().any(|skill| {
                skill
                    .actives
                    .iter()
                    .any(|active| active.targets.as_ref().is_some_and(|targets| targets.has_gainable_hand_equip))
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_355
        } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_354
            || ir.skills.iter().any(|skill| {
                skill
                    .view_as
                    .iter()
                    .any(|view| view.refund_use_count_if_no_damage)
            }) {
            ADDITIVE_RULE_VM_ABI_VERSION_354
        } else {
        super::since_abi_353::required_abi(ir, module_required_abi)
    }
}
