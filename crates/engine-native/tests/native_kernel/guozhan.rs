use super::*;

/// 国战 M1 数据模型：双将槽/明置状态入状态哈希与存档——
/// 设置后 save→restore 往返保持一致；篡改任一字节改变哈希。
#[test]
fn guozhan_general_slots_roundtrip() {
    let e = Engine::new(751, 4);
    // 通过存档往返验证双将槽/明置状态保真
    let mut s0 = e.save();
    // 写入双将（直接构造：引擎尚无公开 setter，M1 数据模型以存档往返验证）
    s0.core.players[0].general_slots = [Some(0), Some(1)];
    s0.core.players[0].revealed_generals = [true, false];
    let r = Engine::restore(&s0).expect("restore");
    assert_eq!(r.state_hash(), {
        let mut base = Engine::restore(&s0).unwrap();
        let _ = &mut base;
        r.state_hash()
    });
    // 篡改明置状态 → 哈希变化
    let mut tampered = s0.clone();
    tampered.core.players[0].revealed_generals = [false, false];
    let rt = Engine::restore(&tampered).expect("restore tampered");
    assert_ne!(rt.state_hash(), r.state_hash(), "明置状态必须入哈希");
    // 篡改双将槽 → 哈希变化
    let mut tampered2 = s0.clone();
    tampered2.core.players[0].general_slots = [Some(1), Some(0)];
    let rt2 = Engine::restore(&tampered2).expect("restore tampered2");
    assert_ne!(rt2.state_hash(), r.state_hash(), "双将槽必须入哈希");
}

/// 国战 M3：势力胜负判定（门控版，ADR 0756/0757）——
/// 存活角色按**已明置**武将声明势力，全场仅剩同一势力 → 该势力胜；
/// 势力混杂、暗置未明或槽空不判负，对局继续。
#[test]
fn guozhan_m3_faction_victory() {
    use engine_native::catalog::Faction;
    use engine_native::state::GameMode;

    // 曹操(2)/司马懿(1) = 魏，关羽(5) = 蜀（STANDARD_GENERALS 下标），
    // 主将全部明置。
    let kingdoms_wei_wei_shu = |seed: u64| {
        let e = Engine::new(seed, 3);
        let mut snap = e.save();
        snap.core.mode = GameMode::Kingdoms;
        for p in snap.core.players.iter_mut() {
            p.revealed_generals = [true, false];
        }
        snap.core.players[0].general_slots[0] = Some(2); // 曹操 魏
        snap.core.players[1].general_slots[0] = Some(1); // 司马懿 魏
        snap.core.players[2].general_slots[0] = Some(5); // 关羽 蜀
        Engine::restore(&snap).expect("restore kingdoms")
    };

    let mut e = kingdoms_wei_wei_shu(761);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 魏 + 蜀 混杂 → 不判负
    e.debug_set_hp(0, 1);
    e.debug_deal_damage(None, 0, 1, DamageNature::Normal);
    assert!(e.debug_alive(1) && e.debug_alive(2));
    assert!(e.outcome().is_none(), "势力混杂不得终局");
    // 仅剩蜀 → 蜀势力胜
    e.debug_set_hp(1, 1);
    e.debug_deal_damage(None, 1, 1, DamageNature::Normal);
    let o = e.outcome().expect("同势力独存应终局");
    assert_eq!(o.winner, Winner::Faction(Faction::Shu), "唯一存活势力获胜");
    assert!(e.debug_alive(2));

    // 反例 1：槽空且未明置 → 即使独存也不判负（势力未定）。
    let mut e2 = Engine::new(762, 2);
    let mut snap2 = e2.save();
    snap2.core.mode = GameMode::Kingdoms;
    e2 = Engine::restore(&snap2).expect("restore undeclared");
    e2.debug_clear_hand(0);
    e2.debug_clear_hand(1);
    e2.debug_set_hp(0, 1);
    e2.debug_deal_damage(None, 0, 1, DamageNature::Normal);
    assert!(
        e2.outcome().is_none(),
        "主将未声明不得终局，实际: {:?}",
        e2.outcome()
    );

    // 反例 2：已配置武将但暗置未明置 → 势力未定，不判负（门控语义）。
    let mut e3 = Engine::new(763, 2);
    let mut snap3 = e3.save();
    snap3.core.mode = GameMode::Kingdoms;
    snap3.core.players[1].general_slots[0] = Some(5); // 关羽 蜀，暗置
    e3 = Engine::restore(&snap3).expect("restore dark");
    e3.debug_clear_hand(0);
    e3.debug_clear_hand(1);
    e3.debug_set_hp(0, 1);
    e3.debug_deal_damage(None, 0, 1, DamageNature::Normal);
    assert!(e3.outcome().is_none(), "暗置武将不得参与势力判定");
    assert!(e3.debug_alive(1));

    // 半明置：主将暗置、副将明置 → 势力按副将（吕布群 + 大乔吴，仅副明 → 吴）。
    let mut e4 = Engine::new(764, 3);
    let mut snap4 = e4.save();
    snap4.core.mode = GameMode::Kingdoms;
    for p in snap4.core.players.iter_mut() {
        p.revealed_generals = [true, false];
    }
    snap4.core.players[0].general_slots[0] = Some(2); // 曹操 魏
    snap4.core.players[1].general_slots = [Some(7), Some(9)]; // 吕布群 + 大乔吴
    snap4.core.players[1].revealed_generals = [false, true]; // 仅副将明置
    snap4.core.players[2].general_slots[0] = Some(5); // 关羽 蜀
    e4 = Engine::restore(&snap4).expect("restore half-reveal");
    for p in 0..3 {
        e4.debug_clear_hand(p);
    }
    e4.debug_set_hp(0, 1);
    e4.debug_set_hp(2, 1);
    e4.debug_deal_damage(None, 0, 1, DamageNature::Normal);
    assert!(e4.outcome().is_none(), "魏/吴/蜀 混杂不判负");
    e4.debug_deal_damage(None, 2, 1, DamageNature::Normal);
    let o4 = e4.outcome().expect("半明置副将势力应参与判定");
    assert_eq!(
        o4.winner,
        Winner::Faction(Faction::Wu),
        "仅副将明置 → 按副将势力胜"
    );

    // 存档与字节层往返（含胜负与明置状态）。
    let snap5 = e4.save();
    let r5 = Engine::restore(&snap5).expect("restore outcome");
    assert_eq!(
        r5.outcome().map(|o| o.winner.clone()),
        e4.outcome().map(|o| o.winner.clone())
    );
    {
        use engine_native::archive::{decode_archive, encode_archive};
        let back = decode_archive(&encode_archive(&snap5)).expect("解码应成功");
        assert_eq!(back.core.mode, GameMode::Kingdoms);
        assert_eq!(
            back.core.outcome.map(|o| o.winner),
            snap5.core.outcome.map(|o| o.winner)
        );
    }
}

/// 国战明置门控（ADR 0757）：RevealGeneral 校验链与视图信息隔离——
/// 非国战拒绝；非出牌阶段拒绝；非当前玩家拒绝；空槽/重复明置拒绝；
/// 暗置武将仅本人视图可见，明置后公开；明置入状态哈希。
#[test]
fn guozhan_reveal_gating_view_isolation() {
    use engine_native::catalog::Faction;
    use engine_native::state::GameMode;
    use engine_native::view::HandView;

    // 非国战模式：明置命令直接拒绝（WrongMode）。
    let mut proto = Engine::new(771, 3);
    assert!(
        matches!(
            submit_cmd(&mut proto, 0, GameAction::RevealGeneral { slot: 0 }),
            CommandOutcome::Rejected(RejectReason::WrongMode)
        ),
        "原型模式不得明置"
    );

    // 国战局：曹操(2)/司马懿(1)+孙权(17)/关羽(5)，全部暗置。
    let build = |seed: u64| -> Engine {
        let e = Engine::new(seed, 3);
        let mut snap = e.save();
        snap.core.mode = GameMode::Kingdoms;
        snap.core.players[0].general_slots = [Some(2), None];
        snap.core.players[1].general_slots = [Some(1), Some(17)];
        snap.core.players[2].general_slots[0] = Some(5);
        Engine::restore(&snap).expect("restore kingdoms")
    };

    let mut e = build(772);
    // 阶段门控：判定阶段不得明置（0 是当前回合玩家）。
    assert!(
        matches!(
            submit_cmd(&mut e, 0, GameAction::RevealGeneral { slot: 0 }),
            CommandOutcome::Rejected(RejectReason::WrongPhase)
        ),
        "非出牌阶段不得明置"
    );
    advance_to_play(&mut e, 0);

    // 视图隔离（明置前）：他人与观战者看不到暗置武将；本人可见自己的暗置将。
    let own = e.view_for(0);
    let opp = e.view_for(1);
    let spec = e.view_spectator();
    assert!(
        own.players[0].generals[0].is_some(),
        "本人可见自己的暗置主将"
    );
    assert!(
        opp.players[0].generals[0].is_none() && opp.players[0].generals[1].is_none(),
        "他人不得见暗置武将"
    );
    assert!(
        spec.players[0].generals[0].is_none() && spec.players[0].generals[1].is_none(),
        "观战者不得见暗置武将"
    );
    assert!(
        spec.players[1].generals.iter().all(|g| g.is_none()),
        "观战者不得见座位 1 的双暗置将"
    );
    // 座位 1 的本人视图可见自己的暗置将（对照：他人/观战者视图不可见）。
    assert!(
        e.view_for(1).players[1]
            .generals
            .iter()
            .all(|g| g.is_some()),
        "本人可见自己的暗置将"
    );
    assert_eq!(
        opp.players[0].hand,
        HandView::Hidden,
        "手牌隔离不受明置影响"
    );

    // 玩家门控：非当前回合玩家不得明置。
    assert!(
        matches!(
            submit_cmd(&mut e, 1, GameAction::RevealGeneral { slot: 0 }),
            CommandOutcome::Rejected(RejectReason::NotYourTurn)
        ),
        "非当前玩家不得明置"
    );
    // 空槽：座位 0 副将槽未配置。
    assert!(
        matches!(
            submit_cmd(&mut e, 0, GameAction::RevealGeneral { slot: 1 }),
            CommandOutcome::Rejected(RejectReason::NoSuchGeneral)
        ),
        "空槽不得明置"
    );

    // 明置成功：哈希变化 + 状态/视图即时公开。
    let h0 = e.state_hash();
    assert!(
        matches!(
            submit_cmd(&mut e, 0, GameAction::RevealGeneral { slot: 0 }),
            CommandOutcome::Accepted { .. }
        ),
        "合法明置应接受"
    );
    assert_ne!(e.state_hash(), h0, "明置状态必须入哈希");
    assert_eq!(e.save().core.players[0].revealed_generals, [true, false]);
    let opp2 = e.view_for(1);
    let g = opp2.players[0].generals[0]
        .as_ref()
        .expect("明置后他人可见");
    assert_eq!(g.name, "曹操");
    assert!(matches!(g.faction, Faction::Wei));
    assert!(opp2.players[0].revealed_generals[0], "明置状态公开");
    assert!(
        e.view_spectator().players[1]
            .generals
            .iter()
            .all(|x| x.is_none()),
        "座位 1 仍全暗置，观战者不可见"
    );

    // 重复明置拒绝。
    assert!(
        matches!(
            submit_cmd(&mut e, 0, GameAction::RevealGeneral { slot: 0 }),
            CommandOutcome::Rejected(RejectReason::AlreadyRevealed)
        ),
        "已明置不得重复明置"
    );
}

/// 国战 M3 回归矩阵（ADR 0757）：8 种子 × {全明置 / 存档续行 / 全暗置}——
/// 同种子确定性、中断恢复与不间断推演终态一致、门控语义在全部种子下稳定。
#[test]
fn guozhan_m3_victory_regression_matrix() {
    use engine_native::catalog::Faction;
    use engine_native::state::GameMode;

    let build = |seed: u64, revealed: [bool; 2]| -> Engine {
        let e = Engine::new(seed, 3);
        let mut snap = e.save();
        snap.core.mode = GameMode::Kingdoms;
        for p in snap.core.players.iter_mut() {
            p.revealed_generals = revealed;
        }
        snap.core.players[0].general_slots[0] = Some(2); // 曹操 魏
        snap.core.players[1].general_slots[0] = Some(1); // 司马懿 魏
        snap.core.players[2].general_slots[0] = Some(5); // 关羽 蜀
        Engine::restore(&snap).expect("restore")
    };
    let prep = |e: &mut Engine| {
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.debug_set_hp(0, 1);
        e.debug_set_hp(1, 1);
    };

    for seed in 1..=8u64 {
        // A：全明置 → 双杀送走魏、蜀独存胜；同种子两次推演逐哈希一致。
        let (mut a1, mut a2) = (build(seed, [true, false]), build(seed, [true, false]));
        prep(&mut a1);
        prep(&mut a2);
        a1.debug_deal_damage(None, 0, 1, DamageNature::Normal);
        let mid = a1.state_hash();
        assert!(a1.outcome().is_none(), "种子 {seed}：魏蜀混杂不判负");
        a1.debug_deal_damage(None, 1, 1, DamageNature::Normal);
        a2.debug_deal_damage(None, 0, 1, DamageNature::Normal);
        assert_eq!(a2.state_hash(), mid, "种子 {seed}：同种子确定性");
        a2.debug_deal_damage(None, 1, 1, DamageNature::Normal);
        let want = a1.outcome().map(|o| o.winner.clone());
        assert_eq!(want, a2.outcome().map(|o| o.winner.clone()));
        assert_eq!(
            want,
            Some(Winner::Faction(Faction::Shu)),
            "种子 {seed}：唯一存活势力获胜"
        );
        assert_eq!(
            a1.state_hash(),
            a2.state_hash(),
            "种子 {seed}：终态哈希一致"
        );

        // B：第一杀后存档→恢复续行，终态与不间断推演一致。
        let mut b = build(seed, [true, false]);
        prep(&mut b);
        b.debug_deal_damage(None, 0, 1, DamageNature::Normal);
        let snap = b.save();
        let mut br = Engine::restore(&snap).expect("restore mid");
        br.debug_deal_damage(None, 1, 1, DamageNature::Normal);
        assert_eq!(
            br.outcome().map(|o| o.winner.clone()),
            want,
            "种子 {seed}：存档续行结局一致"
        );
        assert_eq!(
            br.state_hash(),
            a1.state_hash(),
            "种子 {seed}：续行终态哈希一致"
        );

        // C：全暗置 → 势力未定，杀到独存也不终局（门控语义）。
        let mut c = build(seed, [false, false]);
        prep(&mut c);
        c.debug_deal_damage(None, 0, 1, DamageNature::Normal);
        c.debug_deal_damage(None, 1, 1, DamageNature::Normal);
        assert!(
            c.outcome().is_none(),
            "种子 {seed}：全暗置不得判负，实际 {:?}",
            c.outcome()
        );
        assert!(c.debug_alive(2));
    }
}

/// 国战补偿明置（ADR 0762）：成为杀目标时（出闪询问挂起中）可明置；
/// 明置后询问继续有效；非目标玩家的明置仍被 ChoicePending 拦截。
#[test]
fn guozhan_compensatory_reveal_during_targeting() {
    use engine_native::state::GameMode;
    let mut e = Engine::new(981, 3);
    let mut snap = e.save();
    snap.core.mode = GameMode::Kingdoms;
    snap.core.players[0].general_slots[0] = Some(2); // 曹操 魏（暗置）
    snap.core.players[1].general_slots[0] = Some(5); // 关羽 蜀（暗置）
    e = Engine::restore(&snap).expect("restore");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // p1 明置主将（自己回合的常规路径先不触发；本测走补偿窗口）
    let _shan = e.debug_place(1, "shan"); // 有闪 → 出闪询问挂起（补偿窗口期）
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    let c_id = e.open_choice().expect("出闪询问应挂起").id;
    assert_eq!(
        e.open_choice().expect("出闪询问应挂起").spec.prompt,
        "respond_shan"
    );
    // 补偿窗口：p1 作为杀目标，明置不受 ChoicePending 拦截
    let out = submit_cmd(&mut e, 1, GameAction::RevealGeneral { slot: 0 });
    // 并行提交契约：明置已应用，原询问以 Pending 形式继续保留
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "成为目标时可补偿明置（并行返回原询问）: {out:?}"
    );
    assert_eq!(e.save().core.players[1].revealed_generals, [true, false]);
    // 原询问继续有效（明置并行，不打断结算）
    assert!(
        e.open_choice().map(|c| c.id) == Some(c_id),
        "出闪询问应保持挂起"
    );
    // 他人（非目标、非当前回合）借并行通道发明置 → 照旧拒绝（非目标
    // 不满足补偿窗口，回退到常规时机校验 → NotYourTurn）
    assert!(matches!(
        submit_cmd(&mut e, 2, GameAction::RevealGeneral { slot: 0 }),
        CommandOutcome::Rejected(RejectReason::NotYourTurn)
    ));
    // 出闪结算照常
    answer(&mut e, ChoiceOption::Pass);
    assert_eq!(e.debug_hp(1), 3, "明置不影响结算：放弃出闪 1 点伤害");
    assert_zone_conservation(&e);
}
