use super::*;

/// 墨守装配接线（2026-10-08）：费诗 generals 声明 mapping null →
/// host:moshou_immunity——桥接后目录元数据 skills 应含 moshou
/// （装配/明置授予路径据此自动挂接；宿主行为本体已由
/// `moshou_blocks_delayed_trick_target` 以授予驱动验证）。
#[test]
fn feishi_moshou_declaration_wired_to_host_behavior() {
    let (e, _pack) = setup();
    let base_count = engine_native::content::standard::STANDARD_GENERALS.len();
    // 扩展武将接在基础表后：按名查找费诗并断言其目录技能含 moshou
    let mut found = None;
    for idx in (base_count as u16)..(base_count as u16 + 8) {
        if let Some((name, _, skills, _, _)) = e.catalog().general_meta(idx) {
            if name == "费诗" {
                found = Some(skills.to_vec());
                break;
            }
        }
    }
    assert_eq!(
        found.map(|v| v.to_vec()),
        Some(vec!["nsshuaiyan", "moshou"]),
        "费诗目录技能=率言+墨守（率言批二 v112 接线）"
    );
}

/// 率言（U07 技能批二，规则 v112）：费诗——他人回复体力后（回复者非
/// 你且你非当前回合玩家），回复者二选一：令你摸一张 / 令你弃置其一张
/// 牌（上游态度 AI 推导不迁移）。自己回血/自己回合不触发。
#[test]
fn feishi_shuaiyan_recover_trigger_two_choice() {
    let (mut e, _pack) = setup();
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "nsshuaiyan");

    // 轮到 1 号回合：1 号压 1 血后吃桃回血 → 触发率言（回复者=1，当前
    // 回合玩家=1≠owner 0）
    advance_loop(&mut e, 1);
    let tao = e.debug_place(1, "tao");
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert_eq!(e.debug_hp(1), 3, "压 1 血（4→3）");
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: tao,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 回复者二选一挂起（observer=1 号）
    let c = e.open_choice().expect("率言二选一挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (engine_native::content::host::WUDIAN_SHUAIYAN_CHOICE, 1)
    );
    let (cid, obs) = (c.id, c.observer);
    // ① 选项一：费诗（0 号）摸 1
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    assert_eq!(e.debug_hp(1), 4, "吃桃回满");
    assert_eq!(e.debug_hand(0).len(), 1, "选项一：费诗摸 1");

    // ② 再次驱动：1 号再压血吃桃 → 选二 → 1 号弃一张
    let tao = e.debug_place(1, "tao");
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let t2 = e.debug_place(1, "shan"); // 被弃候选
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: tao,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().expect("率言二选一挂起②");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_SHUAIYAN_CHOICE
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    let c = e.open_choice().expect("被弃者选牌挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (engine_native::content::host::WUDIAN_SHUAIYAN_DISCARD, 1)
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(t2)],
        },
    );
    assert!(!e.debug_hand(1).contains(&t2), "选项二：回复者弃 1（被动）");
    assert!(e.debug_hand(0).len() == 1, "选项二费诗不摸");

    // ③ 负例：0 号自己回合自己回血不触发——0 号压血吃桃
    for _ in 0..48 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.turn().round > 0 {
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
    e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    let hp0 = e.debug_hp(0);
    let tao = e.debug_place(0, "tao");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: tao,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(e.open_choice().is_none(), "自己回合自己回血不触发");
    assert_eq!(e.debug_hp(0), hp0 + 1, "正常回血");
}
