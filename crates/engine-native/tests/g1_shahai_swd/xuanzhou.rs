use super::*;

/// T12 swd deferred 消解批 97（玄咒 xuanzhou/因果 yinguo）：两技行为面
/// + 反例。
#[test]
fn swd_deferred_b97_xuanzhou_yinguo() {
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
    let mut e = Engine::new_with_extensions(62054, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 玄咒（陈辅）：置出普通锦囊 → 目标判定区得延时牌（确定性 kind）
    e.debug_grant(0, "xuanzhou");
    let trick_c = {
        let t = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                let k = e.catalog().card_kind(*c);
                !k.is_basic_kind()
                    && k.equip_slot().is_none()
                    && k != engine_native::catalog::CardKind::Jiguan
                    && k != engine_native::catalog::CardKind::Dismantle
                    && e.state_core().deck.contains(c)
            })
            .expect("普通锦囊在牌库");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != t);
        core.players[0].hand.push(t);
        t
    };
    // p1 手牌 2（>1）→ kind=Le
    e.debug_place(1, "tao");
    e.debug_place(1, "shan");
    let def_xz = e.catalog().skill_by_name("xuanzhou").expect("玄咒在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_xz });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "玄咒发动 {out:?}"
    );
    let c = e.open_choice().expect("玄咒锦囊窗");
    assert_eq!(c.spec.prompt, "xuanzhou_trick");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(trick_c)],
        },
    );
    let c = e.open_choice().expect("玄咒目标窗");
    assert_eq!(c.spec.prompt, "xuanzhou_target");
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
    let judge = e.state_core().player(1).judge.clone();
    assert_eq!(judge.len(), 1, "玄咒：p1 判定区 +1 延时");
    assert!(
        e.catalog().card_kind(judge[0]) == engine_native::catalog::CardKind::Le,
        "玄咒：确定性 kind=乐（p1 手牌>1）"
    );
    assert!(!e.debug_hand(0).contains(&trick_c), "玄咒：素材锦囊已置出");
    // 反例：每回合限一次
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_xz });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "玄咒反例：再发动拒 {out:?}"
    );

    // ② 因果（康那里士）：p1 杀 p2 的伤害前置——p3（持因果）弃一张牌
    // 对调 source/target → p1（原来源）受伤
    e.debug_grant(3, "yinguo");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // p1 回合
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..16 {
        let c = e.turn().current;
        if c == 1 && e.state_core().turn.phase == engine_native::event::Phase::Play {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        if p != 1 && p != 3 {
            e.debug_clear_hand(p);
        }
    }
    let cost3 = e.debug_place(3, "tao");
    let sha2 = e.debug_place(1, "sha");
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![2],
        },
    );
    // p2 空手无闪窗——直接进 DamageBefore 前置窗
    // 因果确认窗（p3）→ 确认 → 弃牌窗
    let c = e.open_choice().expect("因果确认窗");
    assert_eq!((c.spec.prompt, c.observer), ("trigger_confirm", 3));
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("因果弃牌窗");
    assert_eq!(c.spec.prompt, "yinguo_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(cost3)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "因果：对调后原来源 p1 受伤（4→3）");
    assert_eq!(e.debug_hp(2), 4, "因果：原目标 p2 无伤");
    let yg_def = e.catalog().skill_by_name("yinguo").expect("因果在目录");
    assert!(
        !e.state_core().skills.contains_key(&(3, yg_def)),
        "因果：unique 实例已移除"
    );
    let _ = cost3;
}
