use super::*;

/// T12 swd 武将面批 26：雪煌（芝茵 tag 286）——limited 每局一次主动技：
/// 手牌全红且非空时发动，弃全部手牌并逐张视为对（至多）两名其他存活
/// 角色使用火杀（use_virtual_card 座位序确定性；登记差异上游随机）。
/// 字段批配套：PlayerState.xuehuang_used（存档 v69/规则 v281/golden 14
/// 场重录+280 探针零扰动）。
#[test]
fn swd_generals_b26_xuehuang_limited_virtual_firesha() {
    use engine_native::catalog::Suit;
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
    let mut e = Engine::new_with_extensions(62009, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "xuehuang");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let xuehuang_def = e.catalog().skill_by_name("xuehuang").expect("雪煌 def");

    // 反例 ①a：手牌含黑 → activate 被拒
    let _black = {
        let id = e.debug_set_deck_top_by(|d| {
            d.kind.is_sha_like() && matches!(d.suit, Suit::Spade | Suit::Club)
        });
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.players[0].hand.push(id);
        id
    };
    let _t = e.debug_place(0, "tao");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill {
            skill: xuehuang_def,
        },
    });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "雪煌反例：含黑牌拒 {out:?}"
    );

    // 正例：手 3 红（桃+桃+红杀）→ 弃 3 → 3 张×对 p1/p2 火杀
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _t2 = e.debug_place(0, "tao");
    let _t3 = e.debug_place(0, "tao");
    {
        let id = e.debug_set_deck_top_by(|d| {
            d.kind.is_sha_like() && matches!(d.suit, Suit::Heart | Suit::Diamond)
        });
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.players[0].hand.push(id);
    }
    assert_eq!(hand_len_w(&e, 0), 3);
    e.debug_set_hp(1, 8);
    e.debug_set_hp(2, 8);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill {
            skill: xuehuang_def,
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "雪煌受理 {out:?}"
    );
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), 0, "雪煌：手牌全弃");
    assert!(e.state_core().player(0).xuehuang_used, "雪煌：limited 置位");
    assert_eq!(e.debug_hp(1), 5, "雪煌：p1 吃 3 张火杀（8-3）");
    assert_eq!(e.debug_hp(2), 5, "雪煌：p2 吃 3 张火杀（8-3）");

    // 反例 ①b：limited——本回合再发动被拒
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill {
            skill: xuehuang_def,
        },
    });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "雪煌反例：每局一次 {out:?}"
    );

    // 反例 ①c：下一回合仍拒（limited 跨回合持久）
    for _ in 0..4 {
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
        settle_optional_confirm(&mut e);
    }
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill {
            skill: xuehuang_def,
        },
    });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "雪煌反例：跨回合仍拒 {out:?}"
    );
}
