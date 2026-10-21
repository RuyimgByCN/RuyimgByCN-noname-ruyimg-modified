use super::*;

/// T12 swd deferred 消解批 106：蛇魂（山小小 `swd_shanxiaoxiao`）——弃 1–4 张
/// 手牌并选一名有牌角色：其弃等量 he 牌，若其中有与弃牌同花色者则其受 1 伤。
#[test]
fn swd_deferred_shehun_suit_match_damage() {
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

    // 花色匹配：0 号与 1 号弃同花色 → 1 号受 1 伤。
    let mut e = Engine::new_with_extensions(62065, 4, &[&pack]).expect("建局");
    let def = e.catalog().skill_by_name("shehun").expect("shehun 入目录");
    e.debug_grant(0, "shehun");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let own = e.debug_place(0, "tao");
    let own_suit = e.catalog().cards[own as usize].suit;
    let n = e.catalog().cards.len() as u32;
    let same = (0..n)
        .find(|c| e.catalog().cards[*c as usize].suit == own_suit && *c != own)
        .expect("同花色牌");
    inject_hand_at(&mut e, &pack, 1, &[same]);
    e.debug_set_hp(1, 4);

    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    for _ in 0..12 {
        let Some(c) = e.open_choice() else { break };
        let (cid, obs) = (c.id, c.observer);
        let sel = match c.spec.prompt {
            "shehun_discard" => vec![ChoiceOption::Card(own)],
            "shehun_target" => vec![ChoiceOption::Player(1)],
            "shehun_target_discard" => vec![ChoiceOption::Card(same)],
            other => panic!("未预期询问 {other}"),
        };
        let _ = submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
    assert_eq!(e.debug_hp(1), 3, "蛇魂：花色匹配 → 1 号受 1 伤（4→3）");
    assert!(!e.debug_hand(0).contains(&own), "蛇魂：0 号弃牌离手");
    assert!(!e.debug_hand(1).contains(&same), "蛇魂：1 号弃牌离手");

    // 花色不匹配：0 号弃红桃、1 号弃黑桃 → 无伤害。
    let mut e = Engine::new_with_extensions(62066, 4, &[&pack]).expect("建局2");
    e.debug_grant(0, "shehun");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let own = e.debug_place(0, "tao");
    let own_suit = e.catalog().cards[own as usize].suit;
    let other_suit = (0..n)
        .find(|c| e.catalog().cards[*c as usize].suit != own_suit)
        .expect("异花色牌");
    inject_hand_at(&mut e, &pack, 1, &[other_suit]);
    e.debug_set_hp(1, 4);
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    for _ in 0..12 {
        let Some(c) = e.open_choice() else { break };
        let (cid, obs) = (c.id, c.observer);
        let sel = match c.spec.prompt {
            "shehun_discard" => vec![ChoiceOption::Card(own)],
            "shehun_target" => vec![ChoiceOption::Player(1)],
            "shehun_target_discard" => vec![ChoiceOption::Card(other_suit)],
            other => panic!("未预期询问 {other}"),
        };
        let _ = submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
    assert_eq!(e.debug_hp(1), 4, "蛇魂：花色不匹配 → 无伤害");
}
