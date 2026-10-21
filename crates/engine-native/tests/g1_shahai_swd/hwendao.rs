use super::*;

/// T12 swd 武将面批㊲（慧彦横刀+凌锋/秦壬天火；hxunzhi/huanyin/xuanzhou
/// deferred）：两将行为面 + 反例。
#[test]
fn swd_generals_b77_hwendao_lingfeng_tianhuo() {
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
    let mut e = Engine::new_with_extensions(62045, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 横刀（慧彦）：回合外出牌结算后——当前回合角色弃同花色或被获得
    e.debug_grant(0, "hwendao");
    let tao0 = e.debug_place(0, "tao"); // 红桃——同花色候选锚
    let cur_card = e.debug_place(1, "tao"); // p1（当前回合）同花色牌
    let cur_other = e.debug_place(1, "shan");
    // 推进到 p1 回合出桃（回合外=慧彦 owner p0 出牌，需 p1 出牌？
    // 横刀触发条件：owner 回合外出牌。owner=p0，故需 p0 在 p1 回合出牌——
    // 原生无回合外出牌面，改测响应路径：p1 杀 p0、p0 闪响应（红桃闪）
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
        e.debug_clear_hand(p);
    }
    // p0 手牌：红桃闪（响应锚）+ 任意
    let shan0 = e.debug_place(0, "shan");
    // p1 同花色候选（锚=闪的花色）
    let shan_suit = e.catalog().cards[shan0 as usize].suit;
    let p1_match = {
        let m = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().cards[*c as usize].suit == shan_suit
                    && e.catalog().card_kind(*c) != engine_native::catalog::CardKind::Shan
                    && e.state_core().deck.contains(c)
            })
            .expect("同花色非闪牌在牌库");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != m);
        core.players[1].hand.push(m);
        m
    };
    let p1_keep = e.debug_place(1, "shan");
    let sha1 = e.debug_place(1, "sha");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![0],
        },
    );
    let c = e.open_choice().expect("p0 出闪窗");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(shan0)],
        },
    );
    // 横刀确认门 → 确认 → 当前回合角色（p1）同花色弃牌窗
    let c = e.open_choice().expect("横刀确认窗");
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
    let c = e.open_choice().expect("横刀同花色窗");
    assert_eq!(c.spec.prompt, "hwendao_discard_suit");
    assert_eq!(c.observer, 1);
    let has_match = c.spec.options.contains(&ChoiceOption::Card(p1_match));
    assert!(has_match, "横刀：同花色候选含 p1 匹配牌");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(p1_match)],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(!e.debug_hand(1).contains(&p1_match), "横刀：p1 弃同花色牌");
    assert!(e.debug_hand(1).contains(&p1_keep), "横刀：其余保留");
    let _ = (tao0, cur_card, cur_other);

    // ② 凌锋（慧彦）：阶段结束出牌数达标——攻击距离内目标受 1 或自获护甲
    e.debug_grant(0, "lingfeng");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // p0 出牌阶段出 1 张杀（cards_used_play_phase=1 ≥ min(3, hp)）
    // hp=4 → min(3,4)=3 → 需出 3 张……直接压 hp=1 → min(3,1)=1
    e.debug_set_hp(0, 1);
    let sha2 = e.debug_place(0, "sha");
    e.debug_set_hp(1, 4);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    // 出牌阶段结束（Play→Discard）触发凌锋：目标窗
    submit(&mut e, 0, GameAction::NextPhase);
    let c = e.open_choice().expect("凌锋目标窗");
    assert_eq!(c.spec.prompt, "lingfeng_target");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    // 二选一窗：取消=自获 1 护甲
    let c = e.open_choice().expect("凌锋二选一窗");
    assert_eq!(c.spec.prompt, "lingfeng_damage_or_hujia");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(
        e.state_core().player(0).hujia,
        1,
        "凌锋：取消分支自获 1 护甲"
    );
    assert_eq!(e.debug_hp(1), 3, "凌锋：取消分支仅杀伤（4→3），凌锋无伤害");

    // ③ 天火（秦壬）：判定区有牌的其他角色各弃判定牌+受等量火伤
    e.debug_grant(0, "tianhuo");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 4);
    let le1 = e.debug_place_judge(1, "le");
    let le2 = e.debug_place_judge(1, "le");
    let def_th = e.catalog().skill_by_name("tianhuo").expect("天火在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_th });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "天火发动 {out:?}"
    );
    settle_optional_confirm(&mut e);
    assert!(
        !e.state_core().player(1).judge.contains(&le1),
        "天火：判定牌弃"
    );
    assert!(
        !e.state_core().player(1).judge.contains(&le2),
        "天火：判定牌弃 2"
    );
    assert_eq!(e.debug_hp(1), 2, "天火：受等量火伤 2（4→2）");
}
