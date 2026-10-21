use super::*;

/// T12 swd 武将面批 82（福愚钝/肯迪占星）：两将行为面 + 反例。
#[test]
fn swd_generals_b82_yudun_zhanxing() {
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
    assert!(
        pack.generals.len() >= 71,
        "71 将入册（现 {}）",
        pack.generals.len()
    );
    let mut e = Engine::new_with_extensions(62049, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 愚钝：锁锦囊（无法使用锦囊）——反例：直接打无中生有被拒
    e.debug_grant(0, "yudun");
    let wz = e.debug_place(0, "wuzhong");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wz,
            targets: vec![0],
        },
    );
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "愚钝：锦囊禁用（无中生有拒）{out:?}"
    );
    // ② 愚钝：两张锦囊当杀（不计出杀次数——先普通出杀，再转化杀仍可用）
    e.debug_clear_hand(0);
    let t1 = e.debug_place(0, "wuzhong");
    let t2 = e.debug_place(0, "snatch");
    // 先出一张普通杀（占掉限一次）
    let s0 = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: s0,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "普通杀 {out:?}"
    );
    settle_optional_confirm(&mut e);
    pass_nullify(&mut e);
    let def_yd = e.catalog().skill_by_name("yudun").expect("愚钝在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_yd });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "愚钝转化杀发动 {out:?}"
    );
    let c = e.open_choice().expect("愚钝选牌窗");
    assert_eq!(c.spec.prompt, "yudun_pick_two");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(t1), ChoiceOption::Card(t2)],
        },
    );
    let c = e.open_choice().expect("愚钝目标窗");
    assert_eq!(c.spec.prompt, "yudun_sha_target");
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
    pass_nullify(&mut e);
    assert_eq!(e.debug_hp(1), 2, "愚钝：双锦囊当杀命中");
    let _ = (s0, t1, t2);

    // ③ 占星：弃 1 张手牌亮 1 张牌堆顶（黑桃→选目标受 1 雷伤）
    // 布牌：手牌 1 张，牌堆顶黑桃
    e.debug_clear_hand(0);
    let h1 = e.debug_place(0, "tao");
    e.debug_set_hp(1, 4);
    // 压黑桃入堆顶
    let spade = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Spade
                ) && e.catalog().card_kind(*c) == CardKind::Sha
            })
            .expect("黑桃杀");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
        }
        core.deck.push(id);
        id
    };
    let _ = spade;
    e.debug_grant(0, "zhanxing");
    let def_zx = e.catalog().skill_by_name("zhanxing").expect("占星在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_zx });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "占星发动 {out:?}"
    );
    let c = e.open_choice().expect("占星弃牌窗");
    assert_eq!(c.spec.prompt, "zhanxing_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(h1)],
        },
    );
    let c = e.open_choice().expect("占星目标窗（黑桃）");
    assert_eq!(c.spec.prompt, "zhanxing_target");
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
    pass_nullify(&mut e);
    assert_eq!(e.debug_hp(1), 3, "占星：黑桃令 p1 受 1 雷伤（4→3）");
    // 反例：无手牌发动被拒
    e.debug_clear_hand(0);
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_zx });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "占星反例：无手牌拒 {out:?}"
    );
}
