use super::*;

/// 四象系朱雀首片（规则 v236）：焚天——①伤害恒视为火（毕方免疫反证）；
/// ②红色杀无次数限制+无距离限制（隔位目标可指定；黑杀仍受限）；③红色
/// 酒不受每回合一次限制；④红色杀无视防具（八卦阵判定跳过，黑杀照常判
/// 定）。朱雀条目（群目录近似/hp4/female/[神裔,焚天]；boss_chiyan2 赤
/// 炎未落随下批；wuxie 禁面登记差异）。
#[test]
fn boss_zhuque_fentian_red_cards() {
    // ①②③ 同局推进
    let mut e = Engine::new_boss_stage(56001, 4, 1, 3, 6, 4, 0);
    let zhuque_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 12) as u16; // BOSS_GENERALS[12] 朱雀
    let bifang_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 5) as u16; // BOSS_GENERALS[5] 毕方
    e.debug_set_general_def(1, zhuque_def);
    e.debug_set_general_def(2, bifang_def);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 25);
    assert_eq!(
        e.catalog().general_meta(zhuque_def).map(|g| g.0),
        Some("朱雀"),
        "朱雀入目录"
    );
    let fdef = e.catalog().skill_by_name("焚天").expect("焚天入目录");
    let sdef = e.catalog().skill_by_name("神裔").expect("神裔入目录");
    assert!(e.state_core().skills.contains_key(&(1, fdef)));
    assert!(e.state_core().skills.contains_key(&(1, sdef)), "神裔共用");

    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(0);
    // ① 普杀毕方：伤害恒视为火 → 免疫反证
    e.debug_set_hp(0, 4);
    e.debug_set_hp(3, 4);
    let sha1 = e.debug_place(1, "sha");
    let _out1 = play(&mut e, sha1, vec![2]);
    drain_choices(&mut e);
    assert_eq!(
        e.debug_hp(2),
        25,
        "焚天：伤害视为火，毕方免疫（反证 nature=Fire）"
    );

    // 分拣红/黑杀与红酒
    let mut red_shas = Vec::new();
    let mut black_sha = None;
    for _ in 0..8 {
        let c = e.debug_place(1, "sha");
        match e.catalog().cards[c as usize].suit {
            engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond => {
                red_shas.push(c)
            }
            _ => black_sha = Some(c),
        }
        if red_shas.len() >= 2 && black_sha.is_some() {
            break;
        }
    }
    assert!(red_shas.len() >= 2, "红杀分拣不足");
    let black_sha = black_sha.expect("黑杀分拣不足");

    // ② 红色杀无次数+无距离：连打两张（第二张隔位 3 号）
    let r1 = red_shas[0];
    play(&mut e, r1, vec![0]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(0), 3, "红杀 1 命中 0 号");
    let r2 = red_shas[1];
    let out = play(&mut e, r2, vec![3]);
    // 3 号持闪：询问挂起=杀已被受理（拒收不会开闪询问）
    assert!(
        !matches!(out, CommandOutcome::Rejected { .. }),
        "焚天：红杀无次数无距离（隔位可指定）"
    );
    if let CommandOutcome::Pending { .. } = out {
        answer(&mut e, ChoiceOption::Pass);
    }
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(3), 3, "红杀 2 隔位命中 3 号");

    // ③ 黑杀仍受限（同回合第二张）
    let out = play(&mut e, black_sha, vec![0]);
    assert!(
        !matches!(out, CommandOutcome::Accepted { .. }),
        "焚天只豁免红色：黑杀应受限"
    );

    // ④ 红酒免次数面：目录无实体红酒（酒=黑桃3/梅花9），仅经声明转化
    //    （红实体视作酒）可达——代码在位，登记不测（转化路径由 host 面
    //    既有回归覆盖其骨架）。
    assert_zone_conservation(&e);

    // ⑤ 红色杀无视防具（八卦阵跳过；黑杀照常判定）
    let mut e2 = Engine::new(56100, 4);
    advance_to_play(&mut e2, 1);
    // 1 号改为朱雀（标准局 debug 装配）
    e2.debug_set_general_def(1, zhuque_def);
    advance_to_play(&mut e2, 0);
    e2.debug_clear_hand(0);
    let armor = e2.debug_place(0, "armor");
    play(&mut e2, armor, vec![]);
    drain_choices(&mut e2);
    advance_to_play(&mut e2, 1);
    e2.debug_clear_hand(1);
    e2.debug_clear_hand(0);
    let mut red_sha2 = None;
    let mut black_sha2 = None;
    for _ in 0..8 {
        let c = e2.debug_place(1, "sha");
        match e2.catalog().cards[c as usize].suit {
            engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond => {
                red_sha2 = Some(c);
                break;
            }
            _ => {
                if black_sha2.is_none() {
                    black_sha2 = Some(c);
                }
            }
        }
    }
    let red_sha2 = red_sha2.expect("红杀不足");
    e2.debug_set_deck_top_by(|def| def.suit == engine_native::catalog::Suit::Heart);
    play(&mut e2, red_sha2, vec![0]);
    drain_choices(&mut e2);
    assert!(
        !log_has(&e2, |l| matches!(l, LE::ArmorJudged { .. })),
        "焚天：红色杀跳过八卦阵判定"
    );
    // 黑杀照常判定（新回合避开杀限）
    next_phase(&mut e2);
    drain_choices(&mut e2);
    next_phase(&mut e2);
    drain_choices(&mut e2);
    advance_to_play(&mut e2, 1);
    if let Some(_b) = black_sha2 {
        e2.debug_clear_hand(1);
        e2.debug_clear_hand(0);
        let b = e2.debug_place(1, "sha");
        e2.debug_set_deck_top_by(|def| def.suit == engine_native::catalog::Suit::Heart);
        play(&mut e2, b, vec![0]);
        drain_choices(&mut e2);
        assert!(
            log_has(&e2, |l| matches!(l, LE::ArmorJudged { .. })),
            "黑杀照常做八卦阵判定"
        );
    }
    let snap = e2.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(
        r.state_core().skills.contains_key(&(1, fdef)),
        "焚天实例往返"
    );
    assert_zone_conservation(&r);
    assert_zone_conservation(&e2);
}
