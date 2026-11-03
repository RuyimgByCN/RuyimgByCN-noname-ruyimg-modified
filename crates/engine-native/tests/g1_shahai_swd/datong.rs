use super::*;

/// 大同独立测：全场手牌极差 ≤1 → 摸 2。
#[test]
fn swd_generals_b18_datong_draw() {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(62001, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "datong");
    advance_to_play(&mut e, 0);
    // 布阵：全清手后极差 1（p0 手 3、p1 手 2、p2 手 3、p3 手 2 → max 3 min 2）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let c1 = e.debug_place(0, "tao");
    let c2 = e.debug_place(0, "sha");
    let b1 = e.debug_place(1, "tao");
    let b2 = e.debug_place(1, "tao");
    let s1 = e.debug_place(2, "sha");
    let s2 = e.debug_place(2, "sha");
    let s3 = e.debug_place(3, "sha");
    let s4 = e.debug_place(3, "sha");
    {
        let core = e.state_core_mut();
        core.players[0].hand.push(c1);
        core.players[0].hand.push(c2);

        core.players[1].hand.push(b1);
        core.players[1].hand.push(b2);
        core.players[2].hand.push(s1);
        core.players[2].hand.push(s2);
        core.players[3].hand.push(s3);
        core.players[3].hand.push(s4);
    }
    let h0 = hand_len_w(&e, 0);
    // 任意玩家整回合结束触发（A2 H62：global phaseEnd＝任意玩家整回合
    // 结束）——推进 p0 到 End 再收束整回合
    for _ in 0..4 {
        if e.turn().phase == engine_native::event::Phase::End {
            break;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
        settle_optional_confirm(&mut e);
    }
    submit(&mut e, 0, GameAction::NextPhase); // End 收束 → 整回合结束窗
    settle_optional_confirm(&mut e);
    // 整回合结束窗 phaseEnd 段触发后大同摸 2
    assert_eq!(hand_len_w(&e, 0), h0 + 2, "大同：极差 ≤1 摸 2");
}
