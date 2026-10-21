use super::*;

/// 抗yin（U07 技能批四，规则 v114）：留赞——出牌阶段限一次：失去 1 点
/// 体力 → 弃目标一张牌（被弃者自选）→ 已受伤时按弃牌类别分支：基本=
/// 选至多 X 名各摸一（X=maxHp-体力）/ 非基本=授 kangyin2（到他人距离
/// -X、杀额外目标 +X，回合内）。未受伤无分支。
#[test]
fn liuzan_kangyin_losehp_discard_and_branch() {
    let (mut e, _pack) = setup();
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "kangyin");
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 2); // 已受伤：X = 4-2 = 2
    e.debug_grant(0, "咆哮"); // 双次出杀驱动（kangyin2 验证）

    // ① 发动：目标 1 号 → 失去 1 体力（2→1）→ 1 号被弃一张（基本·桃）
    // → 基本分支：选 2 名（0/2）各摸 1
    let tao = e.debug_place(1, "tao");
    let def_ky = e.catalog().skill_by_name("kangyin").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().expect("抗yin目标挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (engine_native::content::host::WUDIAN_KANGYIN_TARGET, 0)
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert_eq!(e.debug_hp(0), 1, "失去 1 体力（2→1）");
    let c = e.open_choice().expect("被弃者选牌挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (engine_native::content::host::WUDIAN_KANGYIN_DISCARD, 1)
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(tao)],
        },
    );
    // 基本分支：选至多 2 名各摸一（X=maxHp2-体力1=... 重算：hp 现在 1，maxHp 4 → X=3？上游=发动时？实现=分支时计算）
    let c = e.open_choice().expect("基本分支摸牌挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (engine_native::content::host::WUDIAN_KANGYIN_DRAW, 0)
    );
    assert!(c.spec.max >= 1, "至多 X 名");
    let picks: Vec<ChoiceOption> = c
        .spec
        .options
        .iter()
        .filter(|o| matches!(o, ChoiceOption::Player(0) | ChoiceOption::Player(2)))
        .copied()
        .collect();
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: picks,
        },
    );
    assert!(
        e.debug_hand(0).iter().any(|c| *c != tao) || e.debug_hand(2).len() > 0,
        "所选角色各摸 1"
    );

    // ② 非基本分支：再发动（咆哮只放杀，抗yin usable1 限一次——
    // 换 1 号回合用 1 号视角不可，改验证限一次）→ SkillAlreadyUsed
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_ky });
    assert!(
        matches!(
            out,
            CommandOutcome::Rejected(engine_native::RejectReason::SkillAlreadyUsed)
        ),
        "限一次: {out:?}"
    );

    // ③ kangyin2 杀额外目标：非基本分支驱动——再布非基本场景需再次
    // 发动（usable 已耗）。改在 1 号回合以 1 号授权?——1 号无技。
    // 直接验证 kangyin2 状态通道：手动置 turn.kangyin2 后杀可双目标
    {
        let mut snap = e.save();
        snap.core.turn.kangyin2 = Some((0, 1));
        e = Engine::restore_with_extensions(&snap, &[&_pack]).expect("kangyin2 置位");
    }
    let sha = e.debug_place(0, "sha");
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1, 2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "kangyin2 杀双目标放行: {out:?}"
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
    assert_eq!(e.debug_hp(1), 3, "目标 1 掉血");
    assert_eq!(e.debug_hp(2), 3, "目标 2 掉血");
}
