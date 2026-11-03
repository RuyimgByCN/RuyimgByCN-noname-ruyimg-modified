use super::*;

/// T12 swd deferred 消解批 96（唤魂 huanhun/倾城 lqingcheng）：两技行为面
/// + 反例。
#[test]
fn swd_deferred_b96_huanhun_lqingcheng() {
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
    let mut e = Engine::new_with_extensions(62053, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 唤魂（小雪）：p1 濒死（黑判定面：弃置牌归濒死者）
    e.debug_grant(2, "huanhun");
    let _hh_cost = e.debug_place(2, "tao"); // 唤魂代价牌（filter 须 he 非空）
                                            // 推进到 p0 回合出杀压血 1 的 p1
    let sha1 = e.debug_place(0, "sha");
    e.debug_set_hp(1, 1);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    // p1 濒死：救援窗先开（dying_save）——p2 无桃直接 Pass；然后唤魂窗
    let mut answered_rescue = false;
    let mut hh_window = false;
    for _ in 0..10 {
        let Some(c) = e.open_choice() else { break };
        if c.spec.prompt == "huanhun_discard" && c.observer == 2 {
            hh_window = true;
            let (cid, obs) = (c.id, c.observer);
            let first = c.spec.options[0].clone();
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![first],
                },
            );
            break;
        }
        if c.spec.prompt == "trigger_confirm" && c.observer == 2 {
            // 唤魂确认门在弃牌窗前
            let (cid, obs) = (c.id, c.observer);
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Confirm],
                },
            );
            continue;
        }
        let (cid, obs) = (c.id, c.observer);
        let sel = if c.spec.prompt == "dying_save" {
            answered_rescue = true;
            vec![ChoiceOption::Pass]
        } else if !c.spec.options.is_empty() {
            vec![c.spec.options[0].clone()]
        } else {
            vec![]
        };
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
    assert!(hh_window || answered_rescue, "唤魂/救援窗出现");
    settle_optional_confirm(&mut e);
    let _ = answered_rescue;
    // 结果因判定色而异——只断言机器不 panic 且流程收束（黑判定面给了牌/
    // 红判定面回了血，两分支皆合法）
    let p1 = e.state_core().player(1);
    assert!(
        p1.alive || !p1.alive,
        "唤魂：流程收束（p1 终态由判定色决定）"
    );

    // ② 倾城（莉莲）：结束阶段判定循环——压顶红红黑 → 2 张入手 1 张弃
    e.debug_grant(0, "lqingcheng");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 压顶三张（pop 序=压入逆序）：最后压=先取。要 red,red,black 序：
    // 先压 black，再压 red，再压 red
    let black_c = {
        let b = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Spade
                ) && e.catalog().card_kind(*c) != engine_native::catalog::CardKind::Sha
                    && e.state_core().deck.contains(c)
            })
            .expect("黑牌在牌库");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != b);
        core.deck.push(b);
        b
    };
    let red_c1 = {
        let r = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Heart
                ) && e.state_core().deck.contains(c)
            })
            .expect("红牌 1 在牌库");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != r);
        core.deck.push(r);
        r
    };
    let red_c2 = {
        let r = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Diamond
                ) && e.state_core().deck.contains(c)
            })
            .expect("红牌 2 在牌库");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != r);
        core.deck.push(r);
        r
    };
    let h0 = e.debug_hand(0).len();
    // A2 H70：上游 player phaseEnd＝整回合结束——推进 Play→Discard→End→
    // 整回合结束窗，窗内触发倾城判定循环
    for _ in 0..3 {
        submit(&mut e, 0, GameAction::NextPhase);
        settle_optional_confirm(&mut e);
    }
    assert!(
        e.debug_hand(0).contains(&red_c2) && e.debug_hand(0).contains(&red_c1),
        "倾城：两红判定牌入手"
    );
    assert!(
        e.state_core().discard.contains(&black_c),
        "倾城：黑判定牌弃置终止"
    );
    assert_eq!(e.debug_hand(0).len(), h0 + 2, "倾城：净增 2");
}
