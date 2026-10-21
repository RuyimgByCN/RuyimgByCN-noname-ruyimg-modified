use super::*;

/// T12 swd 武将面批㉜（恒爱凌波/Miles 血裔/康那里士虚影复用/赵云龙胆 builtin；
/// funiao/ningxian/aojian/mohua2/busi/yinguo/pozhen/tanlin deferred）：四将
/// 行为面 + 反例。
#[test]
fn swd_generals_b72_hlingbo_miles_xueyi() {
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
    let mut e = Engine::new_with_extensions(62040, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 凌波·恒爱面：闪响应后 frequent 摸 2
    e.debug_grant(0, "hlingbo");
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
    let shan0 = e.debug_place(0, "shan");
    let sha1 = e.debug_place(1, "sha");
    let h0 = e.debug_hand(0).len();
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![0],
        },
    );
    let c = e.open_choice().expect("p0 出闪窗");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(shan0)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(
        e.debug_hand(0).len(),
        h0 - 1 + 2,
        "凌波：闪响应后摸 2（出 1 摸 2）"
    );

    // ② 血裔·Miles 面：伤害取消+失去 1（无条件）
    e.debug_grant(0, "miles_xueyi");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 3);
    // 推进 p1 回合出杀（p0 空手也失去 1——无条件面）
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
    e.debug_set_hp(0, 3);
    let sha2 = e.debug_place(1, "sha");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![0],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 2, "血裔：伤害取消→失去 1（3→2，空手同）");

    // ③ 虚影 def 复用（康那里士）：有手=取消+失去 1（与 Miles 无条件相
    // 对照：虚影空手=完全免伤）
    e.debug_grant(0, "xuying");
    {
        let dm = e
            .catalog()
            .skill_by_name("miles_xueyi")
            .expect("血裔在目录");
        e.state_core_mut().skills.remove(&(0, dm));
    }
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 3);
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
    e.debug_set_hp(0, 3);
    let sha3 = e.debug_place(1, "sha");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![0],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 3, "虚影复用：空手=完全免伤（与血裔对照）");

    // ④ 龙胆 builtin（赵云）：黑杀当闪打出
    e.debug_grant(0, "龙胆");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 4);
    // p0 回合直接响应面不可测——改用 p1 回合出杀、p0 龙胆黑杀响应
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
    let black_sha = {
        let b = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c) == engine_native::catalog::CardKind::Sha
                    && matches!(
                        e.catalog().cards[*c as usize].suit,
                        engine_native::catalog::Suit::Spade
                    )
            })
            .expect("黑杀在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != b);
        core.discard.retain(|x| *x != b);
        core.table.retain(|x| *x != b);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != b);
        }
        core.players[0].hand.push(b);
        b
    };
    let sha4 = e.debug_place(1, "sha");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha4,
            targets: vec![0],
        },
    );
    let c = e.open_choice().expect("p0 响应窗（龙胆黑杀候选）");
    assert_eq!(c.observer, 0);
    let has_black = c
        .spec
        .options
        .iter()
        .any(|o| matches!(o, ChoiceOption::Card(x) if *x == black_sha));
    assert!(has_black, "龙胆：黑杀入响应候选");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(black_sha)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 4, "龙胆：黑杀当闪闪避");
}
