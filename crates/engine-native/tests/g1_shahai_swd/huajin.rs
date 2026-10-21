use super::*;

/// T12 swd deferred 消解批 99（化金 huajin/明镜 minjing）：两技行为面
/// + 反例。
#[test]
fn swd_deferred_b99_huajin_minjing() {
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
    let mut e = Engine::new_with_extensions(62056, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 化金（张烈）：弃一张 he → 造成伤害 +1；下一回合准备阶段自清
    e.debug_grant(0, "huajin");
    let cost = e.debug_place(0, "tao");
    let def_hj = e.catalog().skill_by_name("huajin").expect("化金在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_hj });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "化金发动 {out:?}"
    );
    let c = e.open_choice().expect("化金弃牌窗");
    assert_eq!(c.spec.prompt, "huajin_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(cost)],
        },
    );
    settle_optional_confirm(&mut e);
    let hj2 = e
        .catalog()
        .skill_by_name("swd_huajin2")
        .expect("化金触发面");
    assert!(
        e.state_core().skills.contains_key(&(0, hj2)),
        "化金：触发面已授予"
    );
    assert!(!e.debug_hand(0).contains(&cost), "化金：代价已弃");

    // 造成伤害：杀 p1（空手无闪）→ 伤害应为 2（1+化金）
    e.debug_set_hp(1, 4);
    let sha = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    pass_nullify(&mut e);
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 2, "化金：伤害 +1（4→2）");

    // 反例：每回合限一次
    let cost2 = e.debug_place(0, "shan");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_hj });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "化金反例：再发动拒 {out:?}"
    );
    let _ = cost2;

    // 推进到 p0 下一回合准备阶段 → 触发面移除
    next_p0_play_fn(&mut e);
    // next_p0_play_fn 已越过准备阶段；验证实例已无
    assert!(
        !e.state_core().skills.contains_key(&(0, hj2)),
        "化金：下一回合准备阶段触发面已移除"
    );

    // ② 明镜（妮可）：无防具时 1/3 将本次伤害 -1（减至 0 则 hp 不变）
    let mut e2 = Engine::new_with_extensions(62058, 4, &[&pack]).expect("建局2");
    advance_to_play(&mut e2, 0);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    e2.debug_grant(1, "minjing");
    let mut reduced = false;
    for attempt in 0..60 {
        let before = 4i32;
        e2.debug_set_hp(1, before);
        for p in 0..4 {
            e2.debug_clear_hand(p);
        }
        let sha = e2.debug_place(0, "sha");
        let _ = submit(
            &mut e2,
            0,
            GameAction::PlayCard {
                card: sha,
                targets: vec![1],
            },
        );
        for _ in 0..8 {
            if let Some(c) = e2.open_choice() {
                let sel = if c.spec.min == 0 || c.spec.allow_cancel {
                    vec![ChoiceOption::Pass]
                } else if !c.spec.options.is_empty() {
                    vec![c.spec.options[0]]
                } else {
                    vec![]
                };
                let (cid, obs) = (c.id, c.observer);
                submit(
                    &mut e2,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: sel,
                    },
                );
            } else {
                break;
            }
        }
        let after = e2.debug_hp(1);
        if after == before {
            reduced = true;
            break;
        }
        assert_eq!(after, before - 1, "明镜：非减伤应为 1 伤 attempt={attempt}");
        next_p0_play_fn(&mut e2);
    }
    assert!(reduced, "明镜：60 次内应至少一次 1/3 减伤至 0");
}
