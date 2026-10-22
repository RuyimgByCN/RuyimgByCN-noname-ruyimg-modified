use super::*;

/// T12 swd 武将面批㉑：三将部分转正——疾影（赛特 tag 312：锁定 sha 无
/// 视距离，sha_range_free 链查询）；沐风（桓远之 tag 313：结束阶段本回
/// 合用过基本牌则摸 1，TurnState.mufeng_used_basic）；望尘（古月圣 tag
/// 314：弃牌阶段弃置过基本牌则 End 入场令一名角色翻面，
/// TurnState.wangchen_discard_basic）。字段批：存档 v73→v74/规则
/// v286→v287/golden 重录+286 探针零扰动。
#[test]
fn swd_generals_b61_jiying_mufeng_wangchen() {
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
    let mut e = Engine::new_with_extensions(62025, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 疾影：赛特 sha 无视距离打 4 人局对座（距离 2）
    e.debug_grant(0, "jiying");
    e.debug_set_hp(2, 8);
    let sha_j = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_j,
            targets: vec![2],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(2), 7, "疾影：sha 无视距离命中对座");

    // ② 沐风：本回合用基本牌置旗 → 自己阶段结束确认摸 1
    e.debug_grant(0, "mufeng");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha_m = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_m,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(e.state_core().turn.mufeng_used_basic, "沐风旗置位");
    // Play 末（Phase{Play,begin:false}）沐风确认窗
    submit(&mut e, 0, GameAction::NextPhase);
    let h0 = hand_len_w(&e, 0);
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("沐风确认窗");
    assert_eq!(obs, 0);
    let conf: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Confirm))
            .cloned()
            .collect()
    };
    assert_eq!(conf.len(), 1);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: conf,
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), h0 + 1, "沐风：摸 1");

    // ③ 望尘：出牌阶段布 3 张基本牌、压血 3 → 弃牌阶段弃 1 张基本牌 →
    // End 入场望尘窗选 p1 翻面（须在进入弃牌阶段前布阵）
    e.debug_grant(0, "wangchen");
    e.debug_set_hp(0, 3);
    for _ in 0..3 {
        e.debug_place(0, "tao");
    }
    // Play→Discard：弃牌窗弃 1 张 tao（basic）→ 旗置位
    submit(&mut e, 0, GameAction::NextPhase);
    let mut saw_wangchen = false;
    let mut saw_flip = false;
    for _ in 0..48 {
        if let Some(c) = e.open_choice() {
            let prompt = c.spec.prompt;
            let (cid, obs) = (c.id, c.observer);
            if prompt == "wangchen_target" {
                saw_wangchen = true;
                submit(
                    &mut e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![ChoiceOption::Player(1)],
                    },
                );
                saw_flip = true;
                continue;
            }
            let sel: Vec<ChoiceOption> = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else if !c.spec.options.is_empty() {
                vec![c.spec.options[0]]
            } else {
                vec![]
            };
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            continue;
        }
        if saw_wangchen && saw_flip {
            break;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        if let CommandOutcome::Rejected(_) = out {
            continue;
        }
    }
    assert!(saw_wangchen, "望尘：确认窗出现");
    assert!(saw_flip, "望尘：目标选定");
    assert!(e.state_core().player(1).face_down, "望尘：p1 翻面");
}

/// 沐风：确认门后是**发现**（上游 `player.discoverCard()`）——
/// 亮出 3 张、选 1 张入手、其余入弃堆（第 973 轮实现，此前是「摸 1」）。
#[test]
fn swd_mufeng_uses_discover_not_draw() {
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
    let mut e = Engine::new_with_extensions(90210, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "mufeng");
    // 本回合用过基本牌（置旗）
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    // 推进到阶段末触发沐风
    let mut saw_discover = false;
    for _ in 0..12 {
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
        if let Some(ch) = e.open_choice().cloned() {
            if ch.spec.prompt == "mufeng_discover" {
                assert_eq!(ch.spec.min, 1, "发现选 1");
                assert_eq!(ch.spec.max, 1, "发现选 1");
                assert_eq!(ch.spec.options.len(), 3, "亮出 3 张");
                saw_discover = true;
                break;
            }
            let sel = vec![ch.spec.options[0].clone()];
            submit(
                &mut e,
                ch.observer,
                GameAction::Answer {
                    choice_id: ch.id,
                    selection: sel,
                },
            );
        }
    }
    assert!(saw_discover, "沐风应进入 discover（亮 3 选 1），而不是直接摸 1");
}
