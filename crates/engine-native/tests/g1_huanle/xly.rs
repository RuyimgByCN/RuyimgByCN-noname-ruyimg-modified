use super::*;

/// 诊断探针（ignore）：写轮眼窃取后推进多回合，逐步打印
/// resolution.stack / parks / skills 的规模，定位状态膨胀源。
#[test]
#[ignore = "诊断探针：手动运行观察规模曲线"]
fn xly_state_growth_probe() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "huanle-upstream-noname")
            .expect("欢乐包可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");

    let mut e = Engine::new_with_extensions(3205, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let xl = e.debug_place(0, "hnl_xuelunyang_4");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: xl,
            targets: vec![],
        },
    );
    e.debug_grant(1, "马术");

    // 窃取流程：确认门 → 选目标(1) → PickTag(0)。
    advance_to_play(&mut e, 1);
    for _ in 0..64 {
        match e.open_choice() {
            Some(g) if g.spec.prompt == "trigger_confirm" && g.observer == 0 => {
                let (id2, obs2) = (g.id, g.observer);
                submit(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: vec![ChoiceOption::Confirm],
                    },
                );
            }
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                submit(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: sel,
                    },
                );
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } => {}
                    CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
        if e.catalog()
            .skill_by_name("马术")
            .is_some_and(|d| e.save().core.skills.contains_key(&(0, d)))
        {
            break; // 窃取完成
        }
    }

    // 规模观测：推进 120 步（约多回合），每 10 步打印。
    let mut steps = 0;
    for _ in 0..120 {
        steps += 1;
        let snap = e.save();
        if steps % 10 == 0 {
            eprintln!(
                "PROBE step={steps} cur={} phase={:?} stack={} parks={} skills={} loglen={}",
                snap.core.turn.current,
                snap.core.turn.phase,
                snap.resolution.stack.len(),
                snap.resolution.parks.len(),
                snap.core.skills.len(),
                e.log().len()
            );
        }
        if e.outcome().is_some() {
            eprintln!("PROBE over at step={steps}");
            break;
        }
        match e.open_choice() {
            Some(g) if g.spec.prompt == "trigger_confirm" => {
                // OptionalGate 门 options=[Confirm]：放弃 = 空选择取消。
                let (id2, obs2) = (g.id, g.observer);
                submit(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: vec![],
                    },
                );
            }
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                submit(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: sel,
                    },
                );
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } => {}
                    CommandOutcome::Pending { .. } => {}
                    CommandOutcome::Rejected(_) => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
}
