use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第八切片（规则 v149）：王蓬絮（pal_wangpengxu）废墟（f-
// eixu，他人使用/打出闪后确认）——使用或打出闪的角色摸一张牌。duxin-
// shu（读心）登记暂缓。
// ---------------------------------------------------------------------------

/// 废墟：他人打出闪（杀被闪响应）→使用者摸 1；他人使用闪（对自身的
/// 闪？此处以五谷/桃为反例——非闪牌不触发）。
#[test]
fn wangpengxu_feixu_shan_draw() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10006, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "feixu"); // 1 号持废墟
    e.debug_set_hp(1, 4);
    // 0 号杀 1 号（距离 1），1 号出闪 → 废墟持有者（0/1）各触发确认，
    // 出闪者=1 号摸牌（每持有者各一次）
    e.debug_grant(0, "feixu");
    e.debug_clear_hand(0);
    e.debug_place(0, "sha");
    e.debug_clear_hand(1);
    e.debug_place(1, "shan");
    let hand1_before = e.debug_hand(1).len();

    let sha = e
        .debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Sha)
        .expect("0 号应有杀");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    // 1 号出闪询问→选闪
    let shan1 = e.debug_hand(1)[0];
    let out = answer_open(&mut e, vec![ChoiceOption::Card(shan1)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 出闪后废墟触发（确认门 optional——observer=持有者）：0/1 号各持技
    // →两个确认门，依次 Confirm（1 号自己出闪自己摸）
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        assert_eq!(c.spec.prompt, "trigger_confirm");
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Confirm],
            },
        );
    }
    // 出闪 -1，废墟触发 2 次（0/1 各一次）出闪者=1 号各摸 1 → 净 +1
    assert_eq!(
        e.debug_hand(1).len(),
        hand1_before + 1,
        "废墟：出闪者被两位持有者各触发一次摸牌（-1 出闪+2=净 +1）"
    );
}
