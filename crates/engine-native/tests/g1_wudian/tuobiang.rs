use super::*;

/// 炮台（U05 ow 批六，规则 v123）：托比昂——出牌阶段弃一张【杀】布置/
/// 升级炮台（≤3 级，跨回合持久）；结束阶段开始按 30%×层概率对随机敌
/// 人造成 1 点火焰伤害（RuleRng 概率判定）；受到 1 点伤害炮台降 1 级。
#[test]
fn tuobiang_paotai_stack_fire_and_downgrade() {
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

    let mut e = Engine::new_with_extensions(9601, 5, &[&pack]).expect("建局");
    // 注意：paotai 在 shaai-ow 包——本测试文件 setup() 为 wudian 包，
    // 故自建（同毒瘟批）。
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "paotai");
    e.debug_grant(0, "paotai2");
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 4);

    let def_pt = e.catalog().skill_by_name("paotai").expect("def");

    // ① 三次布置（3 张杀）→ lv=3
    for i in 0..3 {
        let sha = e.debug_place(0, "sha");
        let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_pt });
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "{out:?}"
        );
        let c = e.open_choice().expect("炮台弃杀挂起");
        assert_eq!(
            (c.spec.prompt, c.observer),
            (engine_native::content::host::WUDIAN_PAOTAI_CARD, 0)
        );
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Card(sha)],
            },
        );
        assert_eq!(e.state_core().player(0).paotai_lv, i + 1, "第 {} 层", i + 1);
    }
    // lv=3 时不再可发动
    let _sha = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_pt });
    assert!(
        matches!(
            out,
            CommandOutcome::Rejected(engine_native::RejectReason::InvalidChoice)
        ),
        "满级应拒: {out:?}"
    );

    // ② 结束阶段射击：lv=3 → 概率 100%（roll<100 恒真）→ 必中随机敌
    // 推进到 End 阶段开始（fire 在 End 阶段开始帧触发——自愈/割据同款
    // PhaseBegin 通道）；进入 End 后随机敌掉血
    let hp1 = e.debug_hp(1);
    let hp2 = e.debug_hp(2);
    for _ in 0..8 {
        if e.turn().phase == Phase::End {
            break;
        }
        if let Some(c) = e.open_choice() {
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
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    assert_eq!(e.turn().phase, Phase::End);
    assert!(
        e.debug_hp(1) == hp1 - 1 || e.debug_hp(2) == hp2 - 1,
        "lv=3 概率 100%：随机敌受 1 火伤"
    );

    // ③ 受伤降级：0 号受 2 伤 → lv 3→1
    e.debug_set_hp(0, 4);
    e.debug_deal_damage(Some(1), 0, 2, engine_native::event::DamageNature::Normal);
    assert_eq!(e.state_core().player(0).paotai_lv, 1, "受 2 伤降 2 级");

    // ④ 换回合后 lv 持久
    let round0 = e.turn().round;
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.turn().round > round0 {
            break;
        }
        if let Some(c) = e.open_choice() {
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
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    assert_eq!(e.state_core().player(0).paotai_lv, 1, "等级跨回合持久");
}
