use super::common::*;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::Phase;
use engine_native::machine::LogEvent;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction};

/// Round 3 #1: `aojian` 与 `shangxi` 均按攻击距离 `<= 1`（不受武器攻击范围影响），
/// 且 `shangxi` 阶段开始过滤必须存在满足 `distance <= 1 && player.hp <= target.hp` 的目标。
#[test]
fn r3_p1_aojian_and_shangxi_use_attack_distance_le_1() {
    // 1) 奥剑：装 5 射程麒麟弓，p2 与 p0 座位距离为 2（attack_range=5 但 computed_distance=2）
    let (_pack, mut e) = swd_engine(705_301);
    e.debug_grant(0, "aojian");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 2); // X = 2
    let qilin = weapon_with_range(&e, 5);
    equip_slot_card(&mut e, 0, qilin);
    assert!(
        e.state_core().attack_range(e.catalog(), 0) >= 5,
        "前置：麒麟弓使攻击范围 >= 5"
    );
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 2),
        2,
        "前置：p0 到 p2 攻击距离为 2"
    );
    let c1 = find_card(&e, |k, _| k == CardKind::Tao);
    let c2 = find_card(&e, |k, _| k == CardKind::Shan);
    move_card_to_hand(&mut e, 0, c1);
    move_card_to_hand(&mut e, 0, c2);

    let aj = e.catalog().skill_by_name("aojian").expect("aojian");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: aj });
    assert!(
        matches!(out, CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }),
        "dist<=1 仍有 p1/p3 时奥剑可发动: {out:?}"
    );
    let dch = e.open_choice().expect("aojian_discard");
    assert_eq!(dch.spec.prompt, "aojian_discard");
    let (cid, obs) = (dch.id, dch.observer);
    let _ = submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)],
        },
    );
    let tch = e.open_choice().expect("aojian_targets");
    assert_eq!(tch.spec.prompt, "aojian_targets");
    assert!(
        !tch.spec.options.contains(&ChoiceOption::Player(2)),
        "奥剑目标只能是攻击距离<=1角色，不得包含 dist=2 的 p2（即使持 5 射程武器）"
    );
    assert!(
        tch.spec.options.contains(&ChoiceOption::Player(1))
            && tch.spec.options.contains(&ChoiceOption::Player(3)),
        "奥剑应包含 dist=1 的 p1/p3"
    );
    // 取消，并给 p1、p3 分别装备 +1 马使所有他角距离均 >= 2 → 此时即使持麒麟弓也不可发动奥剑
    let (cid, obs) = (tch.id, tch.observer);
    let _ = submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: Vec::new(),
        },
    );
    let mut horses = (0..e.catalog().cards.len() as u32)
        .filter(|c| e.catalog().cards[*c as usize].kind == CardKind::HorsePlus);
    let h1 = horses.next().expect("horse+ 1");
    let h3 = horses.next().expect("horse+ 2");
    equip_slot_card(&mut e, 1, h1);
    equip_slot_card(&mut e, 3, h3);
    let out_no_target = submit(&mut e, 0, GameAction::ActivateSkill { skill: aj });
    assert!(
        matches!(out_no_target, CommandOutcome::Rejected(_)),
        "所有他角攻击距离 >1 时即使武器射程 >=5 也不可发动奥剑: {out_no_target:?}"
    );

    // 2) 袭西：仅 p2(dist=2) hp >= p0，p1/p3(dist=1) 均 hp < p0 → 进入 Play 阶段不得开 shangxi_discard
    let (_pack2, mut e2) = swd_engine(705_302);
    e2.debug_grant(0, "shangxi");
    advance_to_play(&mut e2, 0);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    let qilin2 = weapon_with_range(&e2, 5);
    equip_slot_card(&mut e2, 0, qilin2);
    let sx_c = find_card(&e2, |k, _| k == CardKind::Tao);
    move_card_to_hand(&mut e2, 0, sx_c);
    e2.debug_set_hp(0, 3);
    e2.debug_set_hp(1, 2); // dist=1 但 hp < p0
    e2.debug_set_hp(2, 4); // hp >= p0 但 dist=2
    e2.debug_set_hp(3, 2); // dist=1 但 hp < p0
    // 推进到 p0 下一回合 Draw → Play
    loop {
        let cur = e2.turn().current;
        if cur == 0 && e2.turn().phase == Phase::Draw {
            break;
        }
        let _ = submit(&mut e2, cur, GameAction::NextPhase);
        drain_pass_or_first(&mut e2);
    }
    let _ = submit(&mut e2, 0, GameAction::NextPhase);
    assert!(
        e2.open_choice()
            .is_none_or(|c| c.spec.prompt != "shangxi_discard"),
        "仅 dist=2 的 p2 体力>=自己且 dist<=1 无合规目标时，袭西不得触发询问"
    );

    // 3) 若 p1(dist=1) 体力 >= p0、p2(dist=2) 体力也 >= p0 → 袭西开启，且目标只含 p1 不含 p2
    e2.debug_set_hp(1, 4);
    e2.debug_set_hp(2, 4);
    e2.debug_set_hp(3, 1);
    move_card_to_hand(&mut e2, 0, sx_c);
    loop {
        let cur = e2.turn().current;
        if cur == 0 && e2.turn().phase == Phase::Draw {
            break;
        }
        let _ = submit(&mut e2, cur, GameAction::NextPhase);
        drain_pass_or_first(&mut e2);
    }
    let _ = submit(&mut e2, 0, GameAction::NextPhase);
    let sx_disc = e2.open_choice().expect("应开 shangxi_discard");
    assert_eq!(sx_disc.spec.prompt, "shangxi_discard");
    let (cid, obs) = (sx_disc.id, sx_disc.observer);
    let _ = submit(
        &mut e2,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(sx_c)],
        },
    );
    let sx_tgt = e2.open_choice().expect("应开 shangxi_target");
    assert_eq!(sx_tgt.spec.prompt, "shangxi_target");
    assert!(
        sx_tgt.spec.options.contains(&ChoiceOption::Player(1))
            && !sx_tgt.spec.options.contains(&ChoiceOption::Player(2)),
        "袭西目标窗只能包含攻击距离<=1且体力>=己方的 p1，不得包含 dist=2 的 p2"
    );
}

/// Round 3 #2: `linyun` 须受出牌阶段杀次数限制（除非诸葛连弩等），且混色素材 `[黑, 红]` / `[红, 黑]`
/// 产生无色（`"none"`）杀，不被仁王盾阻挡，且不触发破月/红杀效果；双黑仍被仁王盾阻挡。
#[test]
fn r3_p2_linyun_sha_limit_and_mixed_color_none() {
    // 1) 出过一次杀后未配连弩不可再用林韵；配连弩后可用
    let (_pack, mut e) = swd_engine(705_303);
    e.debug_grant(0, "linyun");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha1 = find_card(&e, |k, _| k == CardKind::Sha);
    let m1 = find_card(&e, |k, s| k == CardKind::Tao && matches!(s, Suit::Heart));
    let m2 = find_card(&e, |k, s| k == CardKind::Shan && matches!(s, Suit::Diamond));
    move_card_to_hand(&mut e, 0, sha1);
    move_card_to_hand(&mut e, 0, m1);
    move_card_to_hand(&mut e, 0, m2);
    let ly = e.catalog().skill_by_name("linyun").expect("linyun");
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    drain_pass_or_first(&mut e);
    assert!(
        !e.legal_actions(0).contains(&GameAction::ActivateSkill { skill: ly }),
        "本阶段已出过杀且无连弩，legal_actions 不得再提供林韵"
    );
    let out_blocked = submit(&mut e, 0, GameAction::ActivateSkill { skill: ly });
    assert!(
        matches!(out_blocked, CommandOutcome::Rejected(_)),
        "本阶段已出过杀不可再发动林韵: {out_blocked:?}"
    );
    let zhuge = find_card(&e, |k, _| k == CardKind::Zhuge);
    move_card_to_hand(&mut e, 0, zhuge);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zhuge,
            targets: vec![],
        },
    );
    assert!(
        e.legal_actions(0).contains(&GameAction::ActivateSkill { skill: ly }),
        "装备诸葛连弩后，legal_actions 允许继续林韵"
    );

    // 2) [黑, 红] 混色杀为无色 "none"，不被 p1 仁王盾阻挡；对比 [黑, 黑] 被仁王盾阻挡
    let (_pack2, mut e2) = swd_engine(705_304);
    e2.debug_grant(0, "linyun");
    advance_to_play(&mut e2, 0);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    let renwang = find_card(&e2, |k, _| k == CardKind::Renwang);
    equip_slot_card(&mut e2, 1, renwang);
    let blk1 = find_card(&e2, |k, s| k == CardKind::Dismantle && s == Suit::Spade);
    let red1 = find_card(&e2, |k, s| k == CardKind::Tao && s == Suit::Heart);
    move_card_to_hand(&mut e2, 0, blk1);
    move_card_to_hand(&mut e2, 0, red1);
    let ly2 = e2.catalog().skill_by_name("linyun").expect("linyun");
    let hp1_before = e2.debug_hp(1);
    let _ = submit(&mut e2, 0, GameAction::ActivateSkill { skill: ly2 });
    let ch = e2.open_choice().expect("linyun_pick_two");
    let (cid, obs) = (ch.id, ch.observer);
    // 先选黑牌作 c1，再选红牌作 c2：混色必须为 "none"，不得因 c1 为黑被仁王盾挡下
    let _ = submit(
        &mut e2,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(blk1), ChoiceOption::Card(red1)],
        },
    );
    let tgt = e2.open_choice().expect("linyun_sha_target");
    let (cid, obs) = (tgt.id, tgt.observer);
    let _ = submit(
        &mut e2,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    drain_pass_or_first(&mut e2);
    assert_eq!(
        e2.debug_hp(1),
        hp1_before - 1,
        "[黑, 红] 林韵杀为无色，不得被仁王盾阻挡"
    );

    // 同局下一步（重置出杀次）：[黑, 黑] 林韵杀颜色仍为黑，应被仁王盾阻挡
    let mut snap = e2.save();
    snap.core.turn.sha_used_play_phase = false;
    let mut e3 = Engine::restore_with_extensions(&snap, &[&_pack2]).unwrap();
    let blk_s = find_card(&e3, |k, s| k == CardKind::Sha && s == Suit::Spade);
    let blk_c = find_card(&e3, |k, s| k == CardKind::Jiu && s == Suit::Club);
    move_card_to_hand(&mut e3, 0, blk_s);
    move_card_to_hand(&mut e3, 0, blk_c);
    let hp1_mid = e3.debug_hp(1);
    let _ = submit(&mut e3, 0, GameAction::ActivateSkill { skill: ly2 });
    let ch = e3.open_choice().expect("linyun_pick_two 2");
    let (cid, obs) = (ch.id, ch.observer);
    let _ = submit(
        &mut e3,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(blk_s), ChoiceOption::Card(blk_c)],
        },
    );
    let tgt = e3.open_choice().expect("linyun_sha_target 2");
    let (cid, obs) = (tgt.id, tgt.observer);
    let _ = submit(
        &mut e3,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    drain_pass_or_first(&mut e3);
    assert_eq!(
        e3.debug_hp(1),
        hp1_mid,
        "[黑, 黑] 林韵杀为黑色，应被仁王盾阻挡"
    );

    // 3) [红, 黑] 混色林韵在持破月时为无色 "none"，不得因 c1 为红而免出杀次数
    let (_pack4, mut e4) = swd_engine(705_305);
    e4.debug_grant(0, "linyun");
    e4.debug_grant(0, "poyue");
    advance_to_play(&mut e4, 0);
    for p in 0..4 {
        e4.debug_clear_hand(p);
    }
    let r_c1 = find_card(&e4, |k, s| k == CardKind::Shan && s == Suit::Heart);
    let b_c2 = find_card(&e4, |k, s| k == CardKind::Dismantle && s == Suit::Club);
    move_card_to_hand(&mut e4, 0, r_c1);
    move_card_to_hand(&mut e4, 0, b_c2);
    let ly4 = e4.catalog().skill_by_name("linyun").expect("linyun");
    let _ = submit(&mut e4, 0, GameAction::ActivateSkill { skill: ly4 });
    let ch = e4.open_choice().expect("pick");
    let (cid, obs) = (ch.id, ch.observer);
    let _ = submit(
        &mut e4,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(r_c1), ChoiceOption::Card(b_c2)],
        },
    );
    let tgt = e4.open_choice().expect("tgt");
    let (cid, obs) = (tgt.id, tgt.observer);
    let _ = submit(
        &mut e4,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    drain_pass_or_first(&mut e4);
    assert!(
        e4.state_core().turn.sha_used_play_phase,
        "[红, 黑] 混色林韵为无色 \"none\"，破月不得视其为红杀免次数"
    );
}

/// Round 3 #3: `yudun` 发动前要求本阶段仍可出杀（已出过普通杀后不可发动），
/// 且 `yudun_free` 仅限 `yudun` 自身转化之杀，不得误豁免还侠/妖童/破晓转化杀。
#[test]
fn r3_p3_yudun_preuse_sha_gate_and_scoped_refund() {
    // 1) 已使用普通杀后，不可再发动 yudun
    let (_pack, mut e) = swd_engine(705_306);
    e.debug_grant(0, "yudun");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = find_card(&e, |k, _| k == CardKind::Sha);
    let t1 = find_card(&e, |k, _| k == CardKind::Wuzhong);
    let t2 = find_card(&e, |k, _| k == CardKind::Snatch);
    move_card_to_hand(&mut e, 0, sha);
    move_card_to_hand(&mut e, 0, t1);
    move_card_to_hand(&mut e, 0, t2);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    drain_pass_or_first(&mut e);
    let yd = e.catalog().skill_by_name("yudun").expect("yudun");
    assert!(
        !e.legal_actions(0).contains(&GameAction::ActivateSkill { skill: yd }),
        "用过常规杀后 legal_actions 不应含 yudun"
    );
    let out_after_sha = submit(&mut e, 0, GameAction::ActivateSkill { skill: yd });
    assert!(
        matches!(out_after_sha, CommandOutcome::Rejected(_)),
        "已出普通杀后不得再发动 yudun: {out_after_sha:?}"
    );

    // 2) 同持 yudun + huanxia 时，使用还侠红色牌当杀仍必须消耗出杀次数
    let (_pack2, mut e2) = swd_engine(705_307);
    e2.debug_grant(0, "yudun");
    e2.debug_grant(0, "huanxia");
    advance_to_play(&mut e2, 0);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    let red_tao = find_card(&e2, |k, s| k == CardKind::Tao && matches!(s, Suit::Heart | Suit::Diamond));
    let norm_sha = find_card(&e2, |k, _| k == CardKind::Sha);
    move_card_to_hand(&mut e2, 0, red_tao);
    move_card_to_hand(&mut e2, 0, norm_sha);
    let out_hx = submit(
        &mut e2,
        0,
        GameAction::PlayCard {
            card: red_tao,
            targets: vec![1],
        },
    );
    assert!(
        matches!(out_hx, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "还侠红桃当杀 {out_hx:?}"
    );
    drain_pass_or_first(&mut e2);
    assert!(
        e2.state_core().turn.sha_used_play_phase,
        "yudun 不得误豁免还侠转化杀的出杀次数"
    );
    let out_sha2 = submit(
        &mut e2,
        0,
        GameAction::PlayCard {
            card: norm_sha,
            targets: vec![1],
        },
    );
    assert!(
        matches!(out_sha2, CommandOutcome::Rejected(_)),
        "还侠用杀后未持连弩不得再出普通杀: {out_sha2:?}"
    );

    // 3) 若先发动 yudun 再出普通杀，则仅 yudun 自身不计次
    let (_pack3, mut e3) = swd_engine(705_308);
    e3.debug_grant(0, "yudun");
    advance_to_play(&mut e3, 0);
    for p in 0..4 {
        e3.debug_clear_hand(p);
    }
    let u1 = find_card(&e3, |k, _| k == CardKind::Wuzhong);
    let u2 = find_card(&e3, |k, _| k == CardKind::Snatch);
    let u_sha = find_card(&e3, |k, _| k == CardKind::Sha);
    move_card_to_hand(&mut e3, 0, u1);
    move_card_to_hand(&mut e3, 0, u2);
    move_card_to_hand(&mut e3, 0, u_sha);
    let yd3 = e3.catalog().skill_by_name("yudun").expect("yudun");
    let _ = submit(&mut e3, 0, GameAction::ActivateSkill { skill: yd3 });
    let ch = e3.open_choice().expect("yudun_pick_two");
    let (cid, obs) = (ch.id, ch.observer);
    let _ = submit(
        &mut e3,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(u1), ChoiceOption::Card(u2)],
        },
    );
    let tgt = e3.open_choice().expect("yudun_sha_target");
    let (cid, obs) = (tgt.id, tgt.observer);
    let _ = submit(
        &mut e3,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    drain_pass_or_first(&mut e3);
    assert!(
        !e3.state_core().turn.sha_used_play_phase,
        "yudun 自身转化杀不计入出杀次数"
    );
    let out_follow_sha = submit(
        &mut e3,
        0,
        GameAction::PlayCard {
            card: u_sha,
            targets: vec![1],
        },
    );
    assert!(
        matches!(out_follow_sha, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "先 yudun 后仍可出普通杀: {out_follow_sha:?}"
    );
}

/// Round 3 #4: `kbolan` 为 `frequent: true` 可选触发——拒绝则保持原摸牌数且不挂 `kbolan2`/不置顶；
/// 确认则 +1 摸牌并开启 `kbolan2_top`，且挂起确认与置顶均支持存读档恢复与 AI 公开信息决策。
#[test]
fn r3_p4_kbolan_optional_confirm_and_decline() {
    let (pack, mut e) = swd_engine(705_309);
    e.debug_grant(0, "kbolan");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let wz1 = find_card(&e, |k, _| k == CardKind::Wuzhong);
    let wz2 = find_card(&e, |k, _| k == CardKind::Dismantle);
    let keep = find_card(&e, |k, _| k == CardKind::Tao);
    move_card_to_hand(&mut e, 0, wz1);
    move_card_to_hand(&mut e, 0, keep);
    let deck_top_before = e.state_core().deck.last().copied();

    // 1) 无中生有摸 2 → 先弹出可选确认（allow_cancel: true）→ 取消拒绝：仅摸 2 张且无 kbolan2
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wz1,
            targets: vec![],
        },
    );
    let ch = e.open_choice().expect("应先弹出 kbolan 可选确认窗");
    assert_ne!(ch.spec.prompt, "kbolan2_top", "不得跳过确认直接进入置顶");
    assert!(ch.spec.allow_cancel, "frequent kbolan 必须允许取消拒绝");
    let snap = decode_archive(&encode_archive(&e.save())).unwrap();
    let mut restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复 kbolan 确认窗");
    let ch_r = restored.open_choice().expect("恢复后确认窗仍在");
    assert_eq!(ch_r.spec.prompt, ch.spec.prompt);
    let (cid, obs) = (ch_r.id, ch_r.observer);
    let _ = submit(
        &mut restored,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: Vec::new(),
        },
    );
    assert!(
        restored.open_choice().is_none(),
        "拒绝 kbolan 后不得进入 kbolan2_top"
    );
    assert_eq!(
        restored.debug_hand(0).len(),
        3,
        "拒绝 kbolan 后仅摸 2 张（1 keep + 2 = 3）"
    );
    let kb2 = restored.catalog().skill_by_name("kbolan2");
    if let Some(def2) = kb2 {
        assert!(
            !restored.state_core().skills.contains_key(&(0, def2)),
            "拒绝 kbolan 后不得授予 kbolan2"
        );
    }
    assert_ne!(
        restored.state_core().deck.last().copied(),
        Some(keep),
        "拒绝 kbolan 后未置回 keep"
    );
    let _ = deck_top_before;

    // 2) AI 确认发动 → 摸 3 → kbolan2_top 置顶 1 张
    let _ = wz2;
    let wz_next = find_card(&e, |k, s| k == CardKind::Wuzhong && matches!(s, Suit::Heart));
    move_card_to_hand(&mut e, 0, wz_next);
    // 重置一局干净测 AI 确认
    let (_p2, mut e2) = swd_engine(705_310);
    e2.debug_grant(0, "kbolan");
    advance_to_play(&mut e2, 0);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    let wz_ai = find_card(&e2, |k, _| k == CardKind::Wuzhong);
    move_card_to_hand(&mut e2, 0, wz_ai);
    let _ = submit(
        &mut e2,
        0,
        GameAction::PlayCard {
            card: wz_ai,
            targets: vec![],
        },
    );
    let first_prompt = ai_answer_once(&mut e2).expect("AI 应答 kbolan 确认窗");
    assert_ne!(first_prompt, "kbolan2_top");
    let top_ch = e2.open_choice().expect("确认后应开启 kbolan2_top");
    assert_eq!(top_ch.spec.prompt, "kbolan2_top");
    assert_eq!(e2.debug_hand(0).len(), 3, "确认 kbolan 后无中生有摸 3 张");
    assert_eq!(ai_answer_once(&mut e2).as_deref(), Some("kbolan2_top"));
    assert_eq!(e2.debug_hand(0).len(), 2, "置顶 1 张后手牌余 2");
}

/// Round 3 #5: `jilve` 在牌堆为空或少于 3 张且弃牌堆非空时可发动并自动洗弃牌堆入牌堆（保留既有堆顶）。
#[test]
fn r3_p5_jilve_reshuffles_discard_when_deck_lt_3() {
    // 1) deck=0, discard 有牌时可激活极略并洗牌展示候选
    let (_pack, mut e) = swd_engine(705_311);
    e.debug_grant(0, "jilve");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha1 = find_card(&e, |k, _| k == CardKind::Sha);
    let tao1 = find_card(&e, |k, _| k == CardKind::Tao);
    let wz1 = find_card(&e, |k, _| k == CardKind::Wuzhong);
    {
        let core = e.state_core_mut();
        let all_deck: Vec<u32> = core.deck.drain(..).collect();
        core.discard.extend(all_deck);
        for c in [sha1, tao1, wz1] {
            if !core.discard.contains(&c) {
                for pl in &mut core.players {
                    pl.hand.retain(|x| *x != c);
                    pl.equips.retain(|x| *x != c);
                }
                core.discard.push(c);
            }
        }
    }
    let jl = e.catalog().skill_by_name("jilve").expect("jilve");
    assert!(
        e.legal_actions(0).contains(&GameAction::ActivateSkill { skill: jl }),
        "deck 空但 discard 非空时，legal_actions 必须包含 jilve"
    );
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: jl });
    assert!(
        matches!(out, CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }),
        "deck=0 discard>0 应可发动极略: {out:?}"
    );
    let ch = e.open_choice().expect("洗牌后应开启 jilve_card");
    assert_eq!(ch.spec.prompt, "jilve_card");
    assert!(
        e.log().iter().any(|ev| matches!(ev, LogEvent::DeckReshuffled)),
        "极略应触发 DeckReshuffled"
    );

    // 2) deck 仅余 1 张已知牌（比如 wuzhong）+ discard 有大量牌（如 >=10），极略必须保留原牌堆顶 1 张并在其下洗入 discard，提供共 3 张顶牌
    let (_pack2, mut e2) = swd_engine(705_312);
    e2.debug_grant(0, "jilve");
    advance_to_play(&mut e2, 0);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    let top_wz = find_card(&e2, |k, _| k == CardKind::Wuzhong);
    let keep_in_discard: std::collections::BTreeSet<u32> = (0..e2.catalog().cards.len() as u32)
        .filter(|&c| {
            matches!(
                e2.catalog().card_kind(c),
                CardKind::Sha | CardKind::Wuzhong | CardKind::Dismantle
            )
        })
        .collect();
    {
        let core = e2.state_core_mut();
        let drained: Vec<u32> = core.deck.drain(..).collect();
        core.discard.extend(drained);
        core.discard.retain(|x| *x != top_wz);
        for pl in &mut core.players {
            pl.hand.retain(|x| *x != top_wz);
            pl.equips.retain(|x| *x != top_wz);
        }
        // 只保留 1 张可用牌 top_wz 在 deck，其余（仅可出锦囊/基本如 Sha/Wuzhong/Dismantle）放进 discard
        core.discard.retain(|c| keep_in_discard.contains(c));
        core.deck.push(top_wz);
    }
    let discard_before = e2.state_core().discard.len();
    assert_eq!(e2.state_core().deck.len(), 1);
    assert!(discard_before >= 5);
    let jl2 = e2.catalog().skill_by_name("jilve").expect("jilve");
    let out2 = submit(&mut e2, 0, GameAction::ActivateSkill { skill: jl2 });
    assert!(
        matches!(out2, CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }),
        "deck=1 discard>=5 应可发动极略: {out2:?}"
    );
    let ch2 = e2.open_choice().expect("应开启 jilve_card");
    assert_eq!(ch2.spec.prompt, "jilve_card");
    assert_eq!(
        ch2.spec.options.len(),
        3,
        "deck=1 + washed discard 应查看 3 张牌并全为可出候选"
    );
    assert_eq!(
        ch2.spec.options.first().copied(),
        Some(ChoiceOption::Card(top_wz)),
        "原牌堆顶牌 top_wz 必须留在最顶位（首选项）"
    );
    assert_eq!(
        e2.state_core().deck.len(),
        1 + discard_before,
        "洗牌后牌堆数应为 原1 + discard_before"
    );
    assert!(e2.state_core().discard.is_empty());
}

/// Round 3 #6: `huanxia` & `yaotong` 可将红/奇数手牌中的锦囊（决斗/拆/顺/乐/兵/铁索/草药/借刀等）当作【杀】使用，
/// 同时不劫持 `PlayCard` 正常打出这些锦囊。
#[test]
fn r3_p6_huanxia_and_yaotong_convert_red_and_odd_tricks_without_hijack() {
    // 1) huanxia：手牌红色过河拆桥 & 红桃乐不思蜀（p1 有手牌所以拆自然合法，但持技者可选择转化为杀）
    let (_pack, mut e) = swd_engine(705_313);
    e.debug_grant(0, "huanxia");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let red_guohe = find_card(&e, |k, s| k == CardKind::Dismantle && matches!(s, Suit::Heart | Suit::Diamond));
    let red_snatch = find_card(&e, |k, s| k == CardKind::Snatch && matches!(s, Suit::Heart | Suit::Diamond));
    let p1_keep = find_card(&e, |k, _| k == CardKind::Shan);
    move_card_to_hand(&mut e, 0, red_guohe);
    move_card_to_hand(&mut e, 0, red_snatch);
    move_card_to_hand(&mut e, 1, p1_keep);

    let hx = e.catalog().skill_by_name("huanxia").expect("huanxia");
    assert!(
        e.legal_actions(0).contains(&GameAction::ActivateSkill { skill: hx }),
        "有红锦囊且可出杀时 legal_actions 须包含 ActivateSkill(huanxia)"
    );
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: hx });
    assert!(
        matches!(out, CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }),
        "ActivateSkill(huanxia) 应接受: {out:?}"
    );
    let pick = e.open_choice().expect("huanxia_pick");
    assert_eq!(pick.spec.prompt, "huanxia_pick");
    assert!(
        pick.spec.options.contains(&ChoiceOption::Card(red_guohe))
            && pick.spec.options.contains(&ChoiceOption::Card(red_snatch)),
        "huanxia_pick 须允许选用红色拆/顺当杀"
    );
    let (cid, obs) = (pick.id, pick.observer);
    let _ = submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(red_guohe)],
        },
    );
    let tgt = e.open_choice().expect("huanxia_target");
    assert_eq!(tgt.spec.prompt, "huanxia_target");
    let (cid, obs) = (tgt.id, tgt.observer);
    let _ = submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    // p1 选不出闪被命中（p1 仍持有 p1_keep）
    drain_pass_or_first(&mut e);
    assert_eq!(e.debug_hp(1), 3, "红过河拆桥通过还侠转化杀命中 p1 减 1 血");
    assert!(e.debug_hand(1).contains(&p1_keep), "转化杀不拆牌");
    assert!(e.state_core().turn.sha_used_play_phase, "消耗本阶段杀次数");
    assert!(
        !e.legal_actions(0).contains(&GameAction::ActivateSkill { skill: hx }),
        "用杀次数耗尽后不可再发动还侠"
    );
    // 而此时 PlayCard 打出红顺手牵羊，依然按顺手牵羊正常偷取 p1_keep
    let out_nat = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: red_snatch,
            targets: vec![1],
        },
    );
    assert!(
        matches!(out_nat, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "PlayCard 打红顺手牵羊仍走自然顺手牵羊不被劫持: {out_nat:?}"
    );
    drain_pass_or_first(&mut e);
    assert!(e.debug_hand(0).contains(&p1_keep), "顺手牵羊成功偷到 p1_keep");

    // 2) 还侠未造成伤害（被闪避）经 ActivateSkill 转化也在回合阶段末收回
    let (_pack_hx2, mut e_hx2) = swd_engine(705_314);
    e_hx2.debug_grant(0, "huanxia");
    advance_to_play(&mut e_hx2, 0);
    for p in 0..4 {
        e_hx2.debug_clear_hand(p);
    }
    let red_le = find_card(&e_hx2, |k, s| k == CardKind::Le && matches!(s, Suit::Heart | Suit::Diamond));
    let p1_shan = find_card(&e_hx2, |k, _| k == CardKind::Shan);
    move_card_to_hand(&mut e_hx2, 0, red_le);
    move_card_to_hand(&mut e_hx2, 1, p1_shan);
    let hx2 = e_hx2.catalog().skill_by_name("huanxia").expect("huanxia");
    let _ = submit(&mut e_hx2, 0, GameAction::ActivateSkill { skill: hx2 });
    let ch = e_hx2.open_choice().unwrap();
    let (cid, obs) = (ch.id, ch.observer);
    let _ = submit(
        &mut e_hx2,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(red_le)],
        },
    );
    let tgt = e_hx2.open_choice().unwrap();
    let (cid, obs) = (tgt.id, tgt.observer);
    let _ = submit(
        &mut e_hx2,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    let shan_ch = e_hx2.open_choice().expect("respond_shan");
    let (cid, obs) = (shan_ch.id, shan_ch.observer);
    let _ = submit(
        &mut e_hx2,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(p1_shan)],
        },
    );
    drain_pass_or_first(&mut e_hx2);
    assert!(
        !e_hx2.debug_hand(0).contains(&red_le) && e_hx2.state_core().discard.contains(&red_le),
        "闪避后红乐应先落入弃牌堆并记入 huanxia_track"
    );
    while e_hx2.turn().current == 0 {
        let _ = submit(&mut e_hx2, 0, GameAction::NextPhase);
        drain_pass_or_first(&mut e_hx2);
    }
    assert!(
        e_hx2.debug_hand(0).contains(&red_le),
        "还侠 ActivateSkill 转红乐当杀被闪避后，阶段末应收回"
    );

    // 3) yaotong：奇数手牌（1 张决斗）可通过 ActivateSkill(yaotong) 当杀使用，且偶数手牌时不可用，且不劫持 PlayCard 顺手牵羊
    let (_pack3, mut e3) = swd_engine(705_315);
    e3.debug_grant(0, "yaotong");
    advance_to_play(&mut e3, 0);
    for p in 0..4 {
        e3.debug_clear_hand(p);
    }
    let b_duel = find_card(&e3, |k, _| k == CardKind::Duel);
    move_card_to_hand(&mut e3, 0, b_duel);
    let yt = e3.catalog().skill_by_name("yaotong").expect("yaotong");
    assert_eq!(e3.debug_hand(0).len(), 1);
    let out_yt = submit(&mut e3, 0, GameAction::ActivateSkill { skill: yt });
    assert!(
        matches!(out_yt, CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }),
        "奇数手牌(1张决斗)须可发动 yaotong 当杀: {out_yt:?}"
    );
    let pick_yt = e3.open_choice().expect("yaotong_pick");
    assert_eq!(pick_yt.spec.prompt, "yaotong_pick");
    assert!(pick_yt.spec.options.contains(&ChoiceOption::Card(b_duel)));
    let (cid, obs) = (pick_yt.id, pick_yt.observer);
    let _ = submit(
        &mut e3,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(b_duel)],
        },
    );
    let tgt_yt = e3.open_choice().expect("yaotong_target");
    assert_eq!(tgt_yt.spec.prompt, "yaotong_target");
    let (cid, obs) = (tgt_yt.id, tgt_yt.observer);
    let _ = submit(
        &mut e3,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    drain_pass_or_first(&mut e3);
    assert_eq!(e3.debug_hp(1), 3, "妖童转化决斗为杀命中 p1 扣 1 血");

    // 偶数手牌（2 张）不可发动 yaotong
    let mut snap3 = e3.save();
    snap3.core.turn.sha_used_play_phase = false;
    let mut e4 = Engine::restore_with_extensions(&snap3, &[&_pack3]).unwrap();
    let j1 = find_card(&e4, |k, _| k == CardKind::Jiedao);
    let j2 = find_card(&e4, |k, _| k == CardKind::Chain);
    move_card_to_hand(&mut e4, 0, j1);
    move_card_to_hand(&mut e4, 0, j2);
    let out_even = submit(&mut e4, 0, GameAction::ActivateSkill { skill: yt });
    assert!(
        matches!(out_even, CommandOutcome::Rejected(_)),
        "偶数手牌不得发动 yaotong: {out_even:?}"
    );
}
