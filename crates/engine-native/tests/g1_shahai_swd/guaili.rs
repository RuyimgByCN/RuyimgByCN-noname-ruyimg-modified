use super::*;

/// T12 swd 武将面批㉛（子巧乖戾+敷衍护甲/莉莲无邪/吉挺观虎；青城/灵视
/// deferred）：四行为行为面 + 反例。
#[test]
fn swd_generals_b71_guaili_fuyan_wuxie_guanhu() {
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
    let mut e = Engine::new_with_extensions(62039, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 乖戾（子巧）：杀伤害 +1（非铁索）+ 造成伤害后弃两张手牌
    e.debug_grant(0, "guaili");
    let sha1 = e.debug_place(0, "sha");
    let c1 = e.debug_place(0, "tao");
    let c2 = e.debug_place(0, "shan");
    e.debug_set_hp(1, 4);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 2, "乖戾：杀伤害 +1（4→2）");
    assert!(
        !e.debug_hand(0).contains(&c1) && !e.debug_hand(0).contains(&c2),
        "乖戾：伤害后弃两张手牌"
    );

    // ② 敷衍·护甲面（子巧）：受伤后令无护甲角色 +1 护甲
    e.debug_grant(0, "fuyan");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // p0 受伤：自残面——出杀给 p1 反伤不存在；用 p1 回合杀 p0
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
        if p != 1 {
            e.debug_clear_hand(p);
        }
    }
    e.debug_set_hp(0, 4);
    let sha2 = e.debug_place(1, "sha");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![0],
        },
    );
    // 敷衍确认门（optional）→ 确认 → 目标窗
    let c = e.open_choice().expect("敷衍确认窗");
    assert_eq!((c.spec.prompt, c.observer), ("trigger_confirm", 0));
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("敷衍目标窗");
    assert_eq!(c.spec.prompt, "fuyan_target");
    assert!(
        c.spec.options.contains(&ChoiceOption::Player(0)),
        "敷衍：p0（无护甲）在候选"
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(0)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.state_core().player(0).hujia, 1, "敷衍：p0 获 1 护甲");

    // ③ 无邪（莉莲）：延时锦囊只能以自己为目标
    e.debug_grant(0, "swd_wuxie");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let le = e.debug_place(0, "le");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: le,
            targets: vec![1],
        },
    );
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "无邪反例：延时对他人拒 {out:?}"
    );
    // 基础路由本就拒绝自置（乐/兵=置他人判定区）——无邪+基础=乐/兵整体
    // 不可用（闪电自置不受限，登记同口径）
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: le,
            targets: vec![0],
        },
    );
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "无邪：自置亦拒（基础路由+无邪=乐/兵整体不可用）{out:?}"
    );

    // ④ 观虎（吉挺）：杀伤害后弃目标一张手牌+一张装备
    // 撤①乖戾（伤害 +1 持续技——杀将致 2 伤）
    {
        let dg = e.catalog().skill_by_name("guaili").expect("乖戾在目录");
        e.state_core_mut().skills.remove(&(0, dg));
    }
    e.debug_grant(0, "guanhu");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let vic_hand = e.debug_place(1, "tao");
    let vic_eq = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c).equip_slot()
                    == Some(engine_native::catalog::EquipSlot::Weapon)
            })
            .expect("武器在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
        }
        core.player_mut(1).equips.push(id);
        id
    };
    let sha3 = e.debug_place(0, "sha");
    e.debug_set_hp(1, 4);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![1],
        },
    );
    // 观虎确认门（optional）→ 确认 → 弃牌窗
    let c = e.open_choice().expect("观虎确认窗");
    assert_eq!((c.spec.prompt, c.observer), ("trigger_confirm", 0));
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("观虎弃牌窗");
    assert_eq!(c.spec.prompt, "guanhu_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(vic_hand), ChoiceOption::Card(vic_eq)],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(
        !e.debug_hand(1).contains(&vic_hand) && !e.debug_equips(1).contains(&vic_eq),
        "观虎：手牌+装备各弃一张"
    );
    assert_eq!(e.debug_hp(1), 3, "观虎：杀照常结算");
}
