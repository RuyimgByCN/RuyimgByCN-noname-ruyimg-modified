use super::*;

/// 毒瘟（U07 ow 批五，规则 v122）：黑百合——锁定技：造成伤害时若你手
/// 牌数与受伤害角色相等，伤害 +1（铁索传导为独立事件照常判定）。
#[test]
fn heibaihe_duwen_equal_hand_damage_bonus() {
    // 毒瘟在 shaai-ow 包——自建 ow 局
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
    let _pack = bridge(&json);
    let mut e = Engine::new_with_extensions(9501, 3, &[&_pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "duwen");
    e.debug_grant(0, "咆哮"); // 双次出杀驱动
    e.debug_set_max_hp(1, 4);
    e.debug_set_hp(1, 4);

    // ① 双方手牌各 1 张（出杀结算时 0 号 sha 已离手：2-1=1=1 号）→
    // 伤害 +1（4→2）
    let _ = e.debug_place(0, "tao");
    let sha = e.debug_place(0, "sha");
    let _ = e.debug_place(1, "tao");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
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
    assert_eq!(e.debug_hp(1), 2, "手牌相等：伤害 +1（1+1=2，4→2）");

    // ② 手牌不等（0 号 1 张、1 号 0 张）→ 无加成（2→1）
    let _ = e.debug_place(0, "tao");
    let sha = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
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
    assert_eq!(e.debug_hp(1), 1, "手牌不等：无加成（2→1）");
}
