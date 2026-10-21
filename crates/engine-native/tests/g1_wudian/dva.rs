use super::*;

// ---------------------------------------------------------------------------
// U05 ow 纵切批二十四（规则 v141）：D.Va（ow_dva）机甲（jijia，标记技
// +commit_damage 拦截）——初始 4 点机甲血：受伤 1:1 抵扣（体力不损），
// 归零后照常扣体力；机甲血>0 时手牌上限 +4。tuijin/zihui/chongzhuang
// 登记暂缓。
// ---------------------------------------------------------------------------

/// 机甲：受伤由机甲血抵扣（体力不变）；机甲血耗尽后照常扣体力；机甲
/// 血>0 时手牌上限 +4（弃牌阶段免弃）。
#[test]
fn dva_jijia_absorbs_damage_and_hand_limit() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9997, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "jijia");
    e.debug_set_jijia(0, 4);
    e.debug_set_hp(0, 2);
    e.debug_set_hp(1, 4);

    // ① 杀 0 号：机甲血 4→3（体力不变）
    e.debug_place(1, "sha");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 收束→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    let atk = e.debug_place(1, "sha");
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: atk,
            targets: vec![0],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert_eq!(e.debug_hp(0), 2, "机甲血抵扣：体力不变");
    assert_eq!(e.save().core.players[0].jijia_hp, 3, "机甲血 4→3");

    // ② 手牌上限 +4：0 号 5 张牌（上限 1+4=5）免弃
    for _ in 0..5 {
        e.debug_place(0, "tao");
    }
    // 驱动：1 收束→2/3/4→0 弃牌阶段（0 手牌 5≤1+4 免弃→End）
    let _ = submit(&mut e, 1, GameAction::NextPhase); // 1 收束→2 Judge
    for _ in 0..96 {
        let t = e.turn();
        if t.current == 0 && t.phase == Phase::Discard && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let (cid, obs) = (c.id, c.observer);
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            drop(c);
            let out = submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            assert!(
                matches!(
                    out,
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
                ),
                "{out:?}"
            );
            continue;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "{out:?}"
        );
    }
    assert!(e.open_choice().is_none(), "机甲血>0：5 张≤上限 5 免弃");
}
