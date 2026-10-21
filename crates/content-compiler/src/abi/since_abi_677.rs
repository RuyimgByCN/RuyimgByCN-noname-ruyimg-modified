use super::*;

pub(super) fn required_abi(ir: &RuleModuleIr, module_required_abi: u32) -> u32 {
    if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_677 {
        ADDITIVE_RULE_VM_ABI_VERSION_677
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_676 {
        ADDITIVE_RULE_VM_ABI_VERSION_676
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_675 {
        ADDITIVE_RULE_VM_ABI_VERSION_675
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_674 {
        ADDITIVE_RULE_VM_ABI_VERSION_674
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_673 {
        ADDITIVE_RULE_VM_ABI_VERSION_673
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_672 {
        ADDITIVE_RULE_VM_ABI_VERSION_672
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_671 {
        ADDITIVE_RULE_VM_ABI_VERSION_671
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_670 {
        ADDITIVE_RULE_VM_ABI_VERSION_670
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_669 {
        ADDITIVE_RULE_VM_ABI_VERSION_669
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_668 {
        ADDITIVE_RULE_VM_ABI_VERSION_668
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_667 {
        ADDITIVE_RULE_VM_ABI_VERSION_667
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_666 {
        ADDITIVE_RULE_VM_ABI_VERSION_666
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_665 {
        ADDITIVE_RULE_VM_ABI_VERSION_665
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_664 {
        ADDITIVE_RULE_VM_ABI_VERSION_664
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_663 {
        ADDITIVE_RULE_VM_ABI_VERSION_663
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_662 {
        ADDITIVE_RULE_VM_ABI_VERSION_662
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_661 {
        ADDITIVE_RULE_VM_ABI_VERSION_661
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_660 {
        ADDITIVE_RULE_VM_ABI_VERSION_660
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_659 {
        ADDITIVE_RULE_VM_ABI_VERSION_659
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_658 {
        ADDITIVE_RULE_VM_ABI_VERSION_658
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_657 {
        ADDITIVE_RULE_VM_ABI_VERSION_657
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_656 {
        ADDITIVE_RULE_VM_ABI_VERSION_656
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_655 {
        ADDITIVE_RULE_VM_ABI_VERSION_655
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_654 {
        ADDITIVE_RULE_VM_ABI_VERSION_654
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_653 {
        ADDITIVE_RULE_VM_ABI_VERSION_653
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_652 {
        ADDITIVE_RULE_VM_ABI_VERSION_652
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_651 {
        ADDITIVE_RULE_VM_ABI_VERSION_651
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_650 {
        ADDITIVE_RULE_VM_ABI_VERSION_650
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_649 {
        ADDITIVE_RULE_VM_ABI_VERSION_649
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_648 {
        ADDITIVE_RULE_VM_ABI_VERSION_648
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_647 {
        ADDITIVE_RULE_VM_ABI_VERSION_647
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_646 {
        ADDITIVE_RULE_VM_ABI_VERSION_646
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_645 {
        ADDITIVE_RULE_VM_ABI_VERSION_645
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_644 {
        ADDITIVE_RULE_VM_ABI_VERSION_644
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_643 {
        ADDITIVE_RULE_VM_ABI_VERSION_643
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_642 {
        ADDITIVE_RULE_VM_ABI_VERSION_642
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_641 {
        ADDITIVE_RULE_VM_ABI_VERSION_641
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_640 {
        ADDITIVE_RULE_VM_ABI_VERSION_640
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_639 {
        ADDITIVE_RULE_VM_ABI_VERSION_639
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_638 {
        ADDITIVE_RULE_VM_ABI_VERSION_638
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_637
        || ir.skills.iter().any(|skill| {
            skill
                .view_as
                .iter()
                .any(|view| view.cost.as_deref() == Some("turn_over"))
                || skill
                    .triggers
                    .iter()
                    .any(|trigger| trigger.filter == "owner_is_target_face_down")
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_637
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_636 {
        ADDITIVE_RULE_VM_ABI_VERSION_636
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_635 {
        ADDITIVE_RULE_VM_ABI_VERSION_635
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_634 {
        ADDITIVE_RULE_VM_ABI_VERSION_634
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_633 {
        ADDITIVE_RULE_VM_ABI_VERSION_633
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_632 {
        ADDITIVE_RULE_VM_ABI_VERSION_632
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_631 {
        ADDITIVE_RULE_VM_ABI_VERSION_631
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_630 {
        ADDITIVE_RULE_VM_ABI_VERSION_630
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_629 {
        ADDITIVE_RULE_VM_ABI_VERSION_629
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_628 {
        ADDITIVE_RULE_VM_ABI_VERSION_628
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_627 {
        ADDITIVE_RULE_VM_ABI_VERSION_627
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_626
        || ir.skills.iter().any(|skill| {
            skill.view_as.iter().any(|view| view.cost.as_deref() == Some("dddfusi"))
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_626
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_625
        || ir.skills.iter().any(|skill| {
            skill.view_as.iter().any(|view| view.cost.as_deref() == Some("dddzhijie"))
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_625
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_624 {
        ADDITIVE_RULE_VM_ABI_VERSION_624
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_623 {
        ADDITIVE_RULE_VM_ABI_VERSION_623
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_622 {
        ADDITIVE_RULE_VM_ABI_VERSION_622
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_621 {
        ADDITIVE_RULE_VM_ABI_VERSION_621
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_620
        || ir.skills.iter().any(|skill| {
            skill
                .view_as
                .iter()
                .any(|view| view.cost.as_deref() == Some("dddshilie"))
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_620
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_619 {
        ADDITIVE_RULE_VM_ABI_VERSION_619
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_618 {
        ADDITIVE_RULE_VM_ABI_VERSION_618
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_617 {
        ADDITIVE_RULE_VM_ABI_VERSION_617
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_616 {
        ADDITIVE_RULE_VM_ABI_VERSION_616
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_615 {
        ADDITIVE_RULE_VM_ABI_VERSION_615
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_614
        || ir.skills.iter().any(|skill| {
            skill.modifiers.iter().any(|modifier| {
                modifier.target == "card_response_cancel" && modifier.selector.is_some()
            }) || skill
                .view_as
                .iter()
                .any(|view| view.filter == "owner_hp_le_two_black_trick")
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_614
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_613 {
        ADDITIVE_RULE_VM_ABI_VERSION_613
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_612 {
        ADDITIVE_RULE_VM_ABI_VERSION_612
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_611 {
        ADDITIVE_RULE_VM_ABI_VERSION_611
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_610 {
        ADDITIVE_RULE_VM_ABI_VERSION_610
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_609 {
        ADDITIVE_RULE_VM_ABI_VERSION_609
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_608 {
        ADDITIVE_RULE_VM_ABI_VERSION_608
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_607 {
        ADDITIVE_RULE_VM_ABI_VERSION_607
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_606 {
        ADDITIVE_RULE_VM_ABI_VERSION_606
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_605 {
        ADDITIVE_RULE_VM_ABI_VERSION_605
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_604 {
        ADDITIVE_RULE_VM_ABI_VERSION_604
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_603 {
        ADDITIVE_RULE_VM_ABI_VERSION_603
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_602 {
        ADDITIVE_RULE_VM_ABI_VERSION_602
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_601 {
        ADDITIVE_RULE_VM_ABI_VERSION_601
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_600 {
        ADDITIVE_RULE_VM_ABI_VERSION_600
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_599 {
        ADDITIVE_RULE_VM_ABI_VERSION_599
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_598 {
        ADDITIVE_RULE_VM_ABI_VERSION_598
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_597 {
        ADDITIVE_RULE_VM_ABI_VERSION_597
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_596 {
        ADDITIVE_RULE_VM_ABI_VERSION_596
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_595 {
        ADDITIVE_RULE_VM_ABI_VERSION_595
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_594 {
        ADDITIVE_RULE_VM_ABI_VERSION_594
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_593 {
        ADDITIVE_RULE_VM_ABI_VERSION_593
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_591 {
        ADDITIVE_RULE_VM_ABI_VERSION_591
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_590 {
        ADDITIVE_RULE_VM_ABI_VERSION_590
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_589 {
        ADDITIVE_RULE_VM_ABI_VERSION_589
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_588 {
        ADDITIVE_RULE_VM_ABI_VERSION_588
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_587 {
        ADDITIVE_RULE_VM_ABI_VERSION_587
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_586 {
        ADDITIVE_RULE_VM_ABI_VERSION_586
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_585 {
        ADDITIVE_RULE_VM_ABI_VERSION_585
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_584
        || ir.skills.iter().any(|skill| {
            skill
                .view_as
                .iter()
                .any(|view| view.cost.as_deref() == Some("dczhaowen"))
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_584
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_583 {
        ADDITIVE_RULE_VM_ABI_VERSION_583
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_582 {
        ADDITIVE_RULE_VM_ABI_VERSION_582
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_581 {
        ADDITIVE_RULE_VM_ABI_VERSION_581
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_580
        || ir.skills.iter().any(|skill| {
            skill
                .triggers
                .iter()
                .any(|trigger| trigger.filter == "owner_source_heart_sha")
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_580
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_579 {
        ADDITIVE_RULE_VM_ABI_VERSION_579
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_578 {
        ADDITIVE_RULE_VM_ABI_VERSION_578
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_577 {
        ADDITIVE_RULE_VM_ABI_VERSION_577
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_576 {
        ADDITIVE_RULE_VM_ABI_VERSION_576
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_575 {
        ADDITIVE_RULE_VM_ABI_VERSION_575
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_574 {
        ADDITIVE_RULE_VM_ABI_VERSION_574
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_573 {
        ADDITIVE_RULE_VM_ABI_VERSION_573
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_572 {
        ADDITIVE_RULE_VM_ABI_VERSION_572
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_571 {
        ADDITIVE_RULE_VM_ABI_VERSION_571
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_570 {
        ADDITIVE_RULE_VM_ABI_VERSION_570
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_569 {
        ADDITIVE_RULE_VM_ABI_VERSION_569
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_568 {
        ADDITIVE_RULE_VM_ABI_VERSION_568
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_567 {
        ADDITIVE_RULE_VM_ABI_VERSION_567
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_566 {
        ADDITIVE_RULE_VM_ABI_VERSION_566
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_565 {
        ADDITIVE_RULE_VM_ABI_VERSION_565
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_564 {
        ADDITIVE_RULE_VM_ABI_VERSION_564
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_563 {
        ADDITIVE_RULE_VM_ABI_VERSION_563
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_562 {
        ADDITIVE_RULE_VM_ABI_VERSION_562
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_561 {
        ADDITIVE_RULE_VM_ABI_VERSION_561
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_560 {
        ADDITIVE_RULE_VM_ABI_VERSION_560
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_559 {
        ADDITIVE_RULE_VM_ABI_VERSION_559
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_558 {
        ADDITIVE_RULE_VM_ABI_VERSION_558
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_447
        || ir.skills.iter().any(|skill| {
            skill.triggers.iter().any(|trigger| {
                trigger.filter == "owner_lost_one_hand_equip_judge_other_current_phase_once"
            })
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_447
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_446 {
        ADDITIVE_RULE_VM_ABI_VERSION_446
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_445 {
        ADDITIVE_RULE_VM_ABI_VERSION_445
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_444 {
        ADDITIVE_RULE_VM_ABI_VERSION_444
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_443 {
        ADDITIVE_RULE_VM_ABI_VERSION_443
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_442
        || ir.skills.iter().any(|skill| {
            skill.triggers.iter().any(|trigger| {
                trigger.filter == "owner_is_target_single_target_other_source_once_per_turn"
            })
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_442
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_441
        || ir.skills.iter().any(|skill| {
            skill.triggers.iter().any(|trigger| {
                trigger.filter == "owner_alive_other_marked_xi"
            })
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_441
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_440 {
        ADDITIVE_RULE_VM_ABI_VERSION_440
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_439 {
        ADDITIVE_RULE_VM_ABI_VERSION_439
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_438
        || ir.skills.iter().any(|skill| {
            skill.triggers.iter().any(|trigger| {
                trigger.filter == "owner_alive_other_trick_first_target_source_max_hand"
            })
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_438
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_437 {
        ADDITIVE_RULE_VM_ABI_VERSION_437
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_436
        || ir.skills.iter().any(|skill| {
            skill.triggers.iter().any(|trigger| {
                trigger.filter == "owner_alive_other_play_phase_discardable_gte_mark"
            })
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_436
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_435
        || ir.skills.iter().any(|skill| {
            skill.triggers.iter().any(|trigger| trigger.filter == "owner_source_club")
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_435
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_434 {
        ADDITIVE_RULE_VM_ABI_VERSION_434
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_433 {
        ADDITIVE_RULE_VM_ABI_VERSION_433
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_432
        || ir.skills.iter().any(|skill| {
            skill.triggers.iter().any(|trigger| {
                matches!(
                    trigger.filter.as_str(),
                    "owner_alive_other_play_phase_zhengsu_missing_in_range"
                        | "owner_is_target_missing_zhengsu"
                )
            })
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_432
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_431 {
        ADDITIVE_RULE_VM_ABI_VERSION_431
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_430 {
        ADDITIVE_RULE_VM_ABI_VERSION_430
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_429 {
        ADDITIVE_RULE_VM_ABI_VERSION_429
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_428
        || ir.skills.iter().any(|skill| {
            skill.triggers.iter().any(|trigger| {
                trigger.filter == "owner_alive_hand_gained_this_turn_can_compare_other"
            })
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_428
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_556 {
        ADDITIVE_RULE_VM_ABI_VERSION_556
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_555 {
        ADDITIVE_RULE_VM_ABI_VERSION_555
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_554 {
        ADDITIVE_RULE_VM_ABI_VERSION_554
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_553 {
        ADDITIVE_RULE_VM_ABI_VERSION_553
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_552
        || ir.skills.iter().any(|skill| {
            skill
                .modifiers
                .iter()
                .any(|m| m.target == "skills_disabled_when_hp_one")
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_552
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_551 {
        ADDITIVE_RULE_VM_ABI_VERSION_551
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_549 {
        ADDITIVE_RULE_VM_ABI_VERSION_549
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_548
        || ir.skills.iter().any(|skill| {
            skill.triggers.iter().any(|trigger| {
                matches!(
                    trigger.filter.as_str(),
                    "owner_alive_sha_entered_discard_potzhanlie_available"
                        | "owner_source_sha_resolved_tracked_potzhanlie_draw"
                )
            })
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_548
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_547 {
        ADDITIVE_RULE_VM_ABI_VERSION_547
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_546 {
        ADDITIVE_RULE_VM_ABI_VERSION_546
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_545 {
        ADDITIVE_RULE_VM_ABI_VERSION_545
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_544
        || ir.skills.iter().any(|skill| {
            skill
                .actives
                .iter()
                .any(|active| active.targets.as_ref().is_some_and(|targets| targets.owner_in_target_attack_range))
                || skill.view_as.iter().any(|view| view.cost.as_deref() == Some("potqingyan"))
                || skill.triggers.iter().any(|trigger| trigger.filter == "owner_source_sha_resolved_tracked_potceduan_dealt_damage")
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_544
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_543 {
        ADDITIVE_RULE_VM_ABI_VERSION_543
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_542 {
        ADDITIVE_RULE_VM_ABI_VERSION_542
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_540 {
        ADDITIVE_RULE_VM_ABI_VERSION_540
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_539 {
        ADDITIVE_RULE_VM_ABI_VERSION_539
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_538 {
        ADDITIVE_RULE_VM_ABI_VERSION_538
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_537 {
        ADDITIVE_RULE_VM_ABI_VERSION_537
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_536 {
        ADDITIVE_RULE_VM_ABI_VERSION_536
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_535
        || ir.skills.iter().any(|skill| {
            skill.actives.iter().any(|active| {
                active
                    .targets
                    .as_ref()
                    .is_some_and(|targets| targets.pair_hand_difference_lte_three)
            })
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_535
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_534 {
        ADDITIVE_RULE_VM_ABI_VERSION_534
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_533 {
        ADDITIVE_RULE_VM_ABI_VERSION_533
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_532
        || ir.skills.iter().any(|skill| {
            skill
                .actives
                .iter()
                .any(|active| active.selection.as_ref().is_some_and(|s| s.distinct_suits))
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_532
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_531 {
        ADDITIVE_RULE_VM_ABI_VERSION_531
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_530
        || ir.skills.iter().any(|skill| {
            skill.triggers.iter().any(|trigger| {
                matches!(
                    trigger.filter.as_str(),
                    "owner_source_sha_damage_target_alive_both_hand"
                        | "owner_alive_other_nanman_cards_discard"
                        | "owner_alive_skill_bound_target_is_event_source"
                )
            })
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_530
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_529
        || ir.skills.iter().any(|skill| {
            skill
                .view_as
                .iter()
                .any(|view| view.cost.as_deref() == Some("all_hand"))
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_529
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_528 {
        ADDITIVE_RULE_VM_ABI_VERSION_528
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_527 {
        ADDITIVE_RULE_VM_ABI_VERSION_527
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_526 {
        ADDITIVE_RULE_VM_ABI_VERSION_526
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_525 {
        ADDITIVE_RULE_VM_ABI_VERSION_525
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_524 {
        ADDITIVE_RULE_VM_ABI_VERSION_524
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_523 {
        ADDITIVE_RULE_VM_ABI_VERSION_523
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_522 {
        ADDITIVE_RULE_VM_ABI_VERSION_522
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_521 {
        ADDITIVE_RULE_VM_ABI_VERSION_521
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_520 {
            ADDITIVE_RULE_VM_ABI_VERSION_520
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_519 {
        ADDITIVE_RULE_VM_ABI_VERSION_519
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_518 {
        ADDITIVE_RULE_VM_ABI_VERSION_518
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_517 {
        ADDITIVE_RULE_VM_ABI_VERSION_517
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_516
        || ir.skills.iter().any(|skill| {
            skill.triggers.iter().any(|trigger| {
                trigger.filter == "owner_alive_card_targeted_healthy"
            }) || skill.view_as.iter().any(|view| {
                view.cost.as_deref() == Some("lose_max_hp_1")
            })
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_516
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_515 {
        ADDITIVE_RULE_VM_ABI_VERSION_515
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_514
        || ir.skills.iter().any(|skill| {
            skill.actives.iter().any(|active| {
                active
                    .targets
                    .as_ref()
                    .is_some_and(|targets| targets.hand_lte_owner)
            }) || skill.triggers.iter().any(|trigger| {
                matches!(
                    trigger.filter.as_str(),
                    "owner_alive_damage_target_min_hand_mbxuye_once_per_turn"
                        | "owner_mbkuangxiang_received_cards_exhausted"
                        | "owner_phase_play_mbkuangxiang_active"
                )
            })
        }) {
        ADDITIVE_RULE_VM_ABI_VERSION_514
    } else if module_required_abi >= ADDITIVE_RULE_VM_ABI_VERSION_513 {
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
