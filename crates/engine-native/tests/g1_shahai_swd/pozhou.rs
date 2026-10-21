use super::*;

/// T12 swd deferred 消解批 110：破舟（刘承宣 `swd_liuchengxuan`）——受伤累计
/// 计数；出牌阶段消耗 1 计数封印一名其他角色（全部技能禁用至其下一阶段开始）。
#[test]
fn swd_deferred_pozhou_seal_cycle() {
    use engine_native::event::DamageNature;
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

    let mut e = Engine::new_with_extensions(62071, 4, &[&pack]).expect("建局");
    let def = e
        .catalog()
        .skill_by_name("pozhou2")
        .expect("pozhou2 入目录");
    e.debug_grant(0, "pozhou");
    e.debug_grant(0, "pozhou2");
    e.debug_grant(1, "lanzhi");
    let lanzhi = e.catalog().skill_by_name("lanzhi").expect("lanzhi");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号受 2 伤 → 计数 2。
    let _ = e.debug_deal_damage(None, 0, 2, DamageNature::Normal);
    assert_eq!(
        e.state_core().player(0).pozhou_count,
        2,
        "破舟：受伤累计计数 2"
    );
    // 发动破舟：封印 1 号。
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { break };
        let (cid, obs) = (c.id, c.observer);
        let sel = match c.spec.prompt {
            "pozhou_target" => vec![ChoiceOption::Player(1)],
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
    assert_eq!(
        e.state_core().player(0).pozhou_count,
        1,
        "破舟：消耗 1 计数"
    );
    assert!(
        !e.state_core()
            .skills
            .get(&(1, lanzhi))
            .map(|i| i.enabled)
            .unwrap_or(false),
        "破舟：1 号技能被封印"
    );

    // 推进到 1 号阶段开始 → 封印解除（技能恢复）。
    for _ in 0..20 {
        if e.turn().current == 1 {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
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
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    assert!(
        e.state_core()
            .skills
            .get(&(1, lanzhi))
            .map(|i| i.enabled)
            .unwrap_or(false),
        "破舟：1 号阶段开始解封（技能恢复）"
    );
}
