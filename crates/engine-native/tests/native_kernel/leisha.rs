use super::*;

/// 雷杀：按杀结算（消耗出牌阶段杀次数）且伤害为雷属性
///（铁索传导按雷属性，见覆盖清单；此处验证结算与次数共享）。
#[test]
fn leisha_resolves_as_sha_and_shares_limit() {
    let mut e = Engine::new(3005, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let leisha = e.debug_place(0, "leisha");
    e.debug_set_hp(1, 4);
    play(&mut e, leisha, vec![1]);
    assert_eq!(e.debug_hp(1), 3, "雷杀命中 1 点伤害");

    // 雷杀占用杀次数：同阶段第二张杀被拒
    let sha = e.debug_place(0, "sha");
    let out = play(&mut e, sha, vec![1]);
    assert!(matches!(
        out,
        CommandOutcome::Rejected(RejectReason::ShaLimit)
    ));
    assert_zone_conservation(&e);
}

/// 雷杀挂起出闪询问的字节往返（LeiSha nature 入 CardUsed/快照）。
#[test]
fn leisha_pending_ask_byte_roundtrip() {
    use engine_native::archive::{decode_archive, encode_archive};
    let mut e = Engine::new(3006, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let leisha = e.debug_place(0, "leisha");
    let shan = e.debug_place(1, "shan");
    e.debug_set_hp(1, 4);
    play(&mut e, leisha, vec![1]);
    let c = e.open_choice().expect("出闪询问");
    assert_eq!((c.spec.prompt, c.observer), ("respond_shan", 1));
    let bytes = encode_archive(&e.save());
    let mut restored = Engine::restore(&decode_archive(&bytes).expect("decode")).expect("restore");
    let cid = restored.open_choice().unwrap().id;
    submit_cmd(
        &mut restored,
        1,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(shan)],
        },
    );
    assert_eq!(restored.debug_hp(1), 4, "出闪免伤");
}
