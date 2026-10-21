use super::*;

/// T12 swd 武将面批㉝（胡月鸣凤/山小小小莫雨/独孤宁珂突袭 builtin；
/// hyunshen 计数存储/shehun defer/nlianji 拼点机 deferred）：三将行为面
/// + 反例。
#[test]
fn swd_generals_b73_fengming_xiaomoyu_touxi() {
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
    let mut e = Engine::new_with_extensions(62041, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 鸣凤（胡月）：弃装备牌 → 任一角色回 1 + 摸 1
    e.debug_grant(0, "fengming");
    let eq1 = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c).equip_slot()
                    == Some(engine_native::catalog::EquipSlot::HorsePlus)
            })
            .expect("坐骑在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
        }
        core.players[0].hand.push(id);
        id
    };
    e.debug_set_hp(1, 2);
    let h1 = e.debug_hand(1).len();
    let def_fm = e.catalog().skill_by_name("fengming").expect("鸣凤在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_fm });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "鸣凤发动 {out:?}"
    );
    let c = e.open_choice().expect("鸣凤弃牌窗");
    assert_eq!(c.spec.prompt, "fengming_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(eq1)],
        },
    );
    let c = e.open_choice().expect("鸣凤目标窗");
    assert_eq!(c.spec.prompt, "fengming_target");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "鸣凤：p1 回 1（2→3）");
    assert_eq!(e.debug_hand(1).len(), h1 + 1, "鸣凤：p1 摸 1");
    assert!(!e.debug_hand(0).contains(&eq1), "鸣凤：装备牌已弃");
    // 反例：每回合限一次
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_fm });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "鸣凤反例：再发动拒 {out:?}"
    );

    // ② 小莫雨（山小小）：造成伤害后回复 1（满血改摸 1），每回合一次
    e.debug_grant(0, "xiaomoyu");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 2); // 受伤→回 1 分支
    let sha1 = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 3, "小莫雨：受伤分支回 1（2→3）");
    // 反例：同回合第二次造成伤害不再触发
    let sha1b = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1b,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 3, "小莫雨反例：每回合一次（不再回）");
    // 满血分支：新回合满血造成伤害→摸 1
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 4); // 满血（maxHp 4）
    let sha1c = e.debug_place(0, "sha");
    let h0 = e.debug_hand(0).len();
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1c,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(
        e.debug_hand(0).len(),
        h0,
        "小莫雨：满血分支摸 1（出 1 摸 1 净不变）"
    );

    // ③ 突袭 builtin（独孤宁珂）：零代码白名单转正——目录技能存在
    e.debug_grant(0, "突袭");
    assert!(
        e.catalog()
            .skill_by_name("突袭")
            .map(|d| e.state_core().skills.contains_key(&(0, d)))
            .unwrap_or(false),
        "突袭：builtin 白名单授予"
    );
}
