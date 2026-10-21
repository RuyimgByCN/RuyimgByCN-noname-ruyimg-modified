use super::*;

/// T12 swd deferred 消解批 100：改天道（孤月 `swd_guyue`）——全局判定，hes
/// 任意牌改判并获得原判定牌（同 Boss 张角「天道」语义）。
#[test]
fn swd_deferred_gtiandao_rewrites_judge_with_any_he() {
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

    let mut e = Engine::new_with_extensions(62059, 4, &[&pack]).expect("建局");
    let gd = e
        .catalog()
        .skill_by_name("gtiandao")
        .expect("gtiandao 入目录");
    e.debug_grant(1, "gtiandao");
    assert!(
        e.state_core().skills.contains_key(&(1, gd)),
        "gtiandao 实例"
    );

    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号判定区置闪电（♠1，黑色）；1 号手牌放一张红桃桃（非黑）作改判牌。
    let sd = e.debug_place_judge(1, "shandian");
    let tao = e.debug_place(1, "tao");
    assert!(e.debug_judge(1).contains(&sd), "闪电在 1 号判定区");
    assert!(e.debug_hand(1).contains(&tao), "桃在 1 号手牌");

    let mut saw = false;
    for _ in 0..80 {
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == "gui_dao_rewrite" {
                saw = true;
                break;
            }
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
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("推进被拒: {other:?}"),
        }
    }
    assert!(saw, "改天道：改判询问挂起");
    let choice = e.open_choice().expect("choice").clone();
    // 改天道：选项含非黑手牌（红桃桃）——hes 全量扩展的直接证据。
    assert!(
        choice
            .spec
            .options
            .iter()
            .any(|o| matches!(o, ChoiceOption::Card(c) if *c == tao)),
        "改天道：改判选项含红桃手牌"
    );
    let hand_before = e.debug_hand(1).len();
    let (cid, obs) = (choice.id, choice.observer);
    let _ = submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(tao)],
        },
    );
    for _ in 0..16 {
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
    // 改判牌（桃）离开手牌；原判定牌（牌堆抽出的判定牌）归改判者，净持平。
    assert!(!e.debug_hand(1).contains(&tao), "改天道：改判牌离手");
    assert_eq!(
        e.debug_hand(1).len(),
        hand_before,
        "改天道：换一进一出（净持平）"
    );
    assert!(
        e.log().iter().any(|l| matches!(
            l,
            engine_native::machine::LogEvent::JudgementRewritten { player: 1, .. }
        )),
        "改天道：判定改写事件落账"
    );
    let _ = sd;
}
