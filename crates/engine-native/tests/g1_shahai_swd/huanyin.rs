use super::*;

/// T12 swd deferred 消解批 103：幻音（琴儿 `swd_qiner`）——他人对你使用牌时
/// 判定：黑桃取消该牌、红桃摸 1、其余无事。
#[test]
fn swd_deferred_huanyin_judge_cancel_or_draw() {
    use engine_native::catalog::Suit;
    let root = repo_extensions_root();
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

    // 场景一：判定黑桃 → 取消 0 号对 1 号的铁索连环（1 号不横置）。
    let mut e = Engine::new_with_extensions(62061, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "huanyin");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let wz = e.debug_place(0, "chain");
    e.debug_set_deck_top_by(|d| d.suit == Suit::Spade);
    let out = play(&mut e, wz, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { break };
        let sel: Vec<ChoiceOption> = if c.spec.min == 0 || c.spec.allow_cancel {
            vec![ChoiceOption::Pass]
        } else {
            c.spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(c.spec.min.max(1))
                .collect()
        };
        let (cid, obs) = (c.id, c.observer);
        let _ = submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
    assert!(
        !e.debug_linked(1),
        "幻音：黑桃判定取消铁索连环（1 号不横置）"
    );

    // 场景二：判定红桃 → 1 号摸 1（幻音），铁索照常结算（1 号横置）。
    let mut e = Engine::new_with_extensions(62062, 4, &[&pack]).expect("建局2");
    e.debug_grant(1, "huanyin");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let wz = e.debug_place(0, "chain");
    e.debug_set_deck_top_by(|d| d.suit == Suit::Heart);
    let out = play(&mut e, wz, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { break };
        let sel: Vec<ChoiceOption> = if c.spec.min == 0 || c.spec.allow_cancel {
            vec![ChoiceOption::Pass]
        } else {
            c.spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(c.spec.min.max(1))
                .collect()
        };
        let (cid, obs) = (c.id, c.observer);
        let _ = submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
    assert_eq!(e.debug_hand(1).len(), 1, "幻音：红桃摸 1");
    assert!(
        e.debug_linked(1),
        "幻音：红桃不取消，铁索照常结算（1 号横置）"
    );
}
