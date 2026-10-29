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
    } else {
        super::since_abi_513::required_abi(ir, module_required_abi)
    }
}
