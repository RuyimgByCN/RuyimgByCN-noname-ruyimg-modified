use super::common::*;
use content_native_loader::restore_locked_engine;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::CardKind;
use engine_native::{ChoiceOption, GameAction};

#[test]
fn u05_b7b_r3_n04e_sadengjinhuan_judge_confirm_and_extra_shan_snapshot_roundtrip() {
    let (v, mut e) = start_shahai_swd_only(9601, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    grant_skill(&mut e, &v, 0, "sadengjinhuan");
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    let sha = find_kind_nth(&e, CardKind::Sha, 0);
    let shan1 = find_kind_nth(&e, CardKind::Shan, 0);
    let shan2 = find_kind_nth(&e, CardKind::Shan, 1);
    force_move_card_to_hand(&mut e, &v, 0, sha);
    force_move_card_to_hand(&mut e, &v, 1, shan1);
    force_move_card_to_hand(&mut e, &v, 1, shan2);
    e.debug_set_hp(1, 4);
    e.debug_set_deck_top_by(|d| d.suit.is_heart());

    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    // Seat 1 plays first Shan -> Sadengjinhuan judge confirmation opens.
    answer_open(&mut e, vec![ChoiceOption::Card(shan1)]);
    let c_confirm = e.open_choice().expect("sadengjinhuan_judge_confirm").clone();
    assert_eq!(c_confirm.spec.prompt, "sadengjinhuan_judge_confirm");

    // N04-e save/restore roundtrip while parked on sadengjinhuan_judge_confirm.
    let bytes1 = encode_archive(&e.save());
    let dec1 = decode_archive(&bytes1).expect("decode archive on sadengjinhuan_judge_confirm");
    e = restore_locked_engine(&dec1, &v).expect("restore on sadengjinhuan_judge_confirm");
    let c_confirm_restored = e.open_choice().expect("restored confirm").clone();
    assert_eq!(c_confirm_restored.spec.prompt, "sadengjinhuan_judge_confirm");
    assert_eq!(c_confirm_restored.observer, c_confirm.observer);

    // Confirm -> red judge -> sadengjinhuan_extra_shan opens on seat 1.
    answer_open(&mut e, vec![ChoiceOption::Confirm]);
    let c_extra = e.open_choice().expect("sadengjinhuan_extra_shan").clone();
    assert_eq!(c_extra.spec.prompt, "sadengjinhuan_extra_shan");
    assert!(c_extra.spec.options.contains(&ChoiceOption::Card(shan2)));

    // N04-e save/restore roundtrip while parked on sadengjinhuan_extra_shan.
    let bytes2 = encode_archive(&e.save());
    let dec2 = decode_archive(&bytes2).expect("decode archive on sadengjinhuan_extra_shan");
    e = restore_locked_engine(&dec2, &v).expect("restore on sadengjinhuan_extra_shan");
    let c_extra_restored = e.open_choice().expect("restored extra shan").clone();
    assert_eq!(c_extra_restored.spec.prompt, "sadengjinhuan_extra_shan");
    assert_eq!(c_extra_restored.observer, c_extra.observer);
    assert_eq!(c_extra_restored.spec.options, c_extra.spec.options);

    // Decline extra Shan -> Sha deals 1 damage (4 -> 3).
    answer_open(&mut e, Vec::new());
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(1).hp, 3);
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7b_r3_n04e_dying_save_tianxianjiu_and_yuchan_snapshot_roundtrip() {
    let (v, mut e) = start_shahai_swd_only(9602, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);

    let sha = find_kind_nth(&e, CardKind::Sha, 0);
    let tao = find_kind_nth(&e, CardKind::Tao, 0);
    let txj0 = find_kind_nth(&e, CardKind::Tianxianjiu, 0);
    let ycx0 = find_kind(&e, CardKind::Yuchanxun);
    let ycz0 = find_kind(&e, CardKind::Yuchanzhen);
    let ycl0 = find_kind(&e, CardKind::Yuchanli);
    let txj1 = find_kind_nth(&e, CardKind::Tianxianjiu, 1);

    force_move_card_to_hand(&mut e, &v, 0, sha);
    force_move_card_to_hand(&mut e, &v, 0, tao);
    force_move_card_to_hand(&mut e, &v, 0, txj0);
    force_move_card_to_hand(&mut e, &v, 0, ycx0);
    force_move_card_to_hand(&mut e, &v, 0, ycz0);
    force_move_card_to_hand(&mut e, &v, 0, ycl0);
    force_move_card_to_hand(&mut e, &v, 1, txj1);

    e.debug_set_hp(1, 1);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );

    // DyingSave iterates seat_order_from(turn.current): seat 0 (turn owner, non-dying) is asked FIRST,
    // holding [Tao, Tianxianjiu, Yuchanxun, Yuchanzhen, Yuchanli]!
    let c0 = e.open_choice().expect("seat 0 non-dying dying_save").clone();
    assert_eq!((c0.observer, c0.spec.prompt), (0, "dying_save"));
    assert!(c0.spec.options.contains(&ChoiceOption::Card(tao)));
    assert!(
        c0.spec.options.contains(&ChoiceOption::Card(ycx0)),
        "Yuchanxun acts as Tao for non-dying responder: {:?}",
        c0.spec.options
    );
    assert!(
        !c0.spec.options.contains(&ChoiceOption::Card(txj0)),
        "Tianxianjiu is self-only and must NOT appear for non-dying seat 0: {:?}",
        c0.spec.options
    );
    assert!(
        !c0.spec.options.contains(&ChoiceOption::Card(ycz0)),
        "Yuchanzhen is self-only and must NOT appear for non-dying seat 0: {:?}",
        c0.spec.options
    );
    assert!(
        !c0.spec.options.contains(&ChoiceOption::Card(ycl0)),
        "Yuchanli is self-only and must NOT appear for non-dying seat 0: {:?}",
        c0.spec.options
    );

    // N04-e roundtrip on non-dying responder holding [Tao, Tianxianjiu, Yuchanxun, Yuchanzhen, Yuchanli]!
    let b0 = encode_archive(&e.save());
    let d0 = decode_archive(&b0).expect("decode non-dying save");
    e = restore_locked_engine(&d0, &v).expect("restore non-dying save");
    let c0_restored = e.open_choice().expect("restored seat 0 dying_save").clone();
    assert_eq!(c0_restored.spec.options, c0.spec.options);

    // Reject forged/stale answer attempting to use non-dying Tianxianjiu, then pass to ask dying seat 1!
    let bad = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c0_restored.id,
            selection: vec![ChoiceOption::Card(txj0)],
        },
    );
    assert!(matches!(bad, engine_native::CommandOutcome::Rejected(_)));
    answer_open(&mut e, vec![ChoiceOption::Pass]);

    // Dying seat 1 holds Tianxianjiu (plus test self Yuchanxun/Yuchanzhen/Yuchanli visibility):
    let c1 = e.open_choice().expect("seat 1 dying dying_save").clone();
    assert_eq!((c1.observer, c1.spec.prompt), (1, "dying_save"));
    assert!(
        c1.spec.options.contains(&ChoiceOption::Card(txj1)),
        "dying seat 1 must see own Tianxianjiu in dying_save: {:?}",
        c1.spec.options
    );
    let b1 = encode_archive(&e.save());
    let d1 = decode_archive(&b1).expect("decode dying self save");
    e = restore_locked_engine(&d1, &v).expect("restore dying self save");
    assert_eq!(e.open_choice().unwrap().spec.options, c1.spec.options);
    answer_open(&mut e, vec![ChoiceOption::Card(txj1)]);
    drain_pass_all(&mut e);
    assert!(e.state_core().player(1).alive);
    assert_eq!(e.state_core().player(1).hp, 1);
    assert_eq!(total_card_set(&e), init_set);
}
