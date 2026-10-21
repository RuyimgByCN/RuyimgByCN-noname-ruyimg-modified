use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第十九切片（规则 v160）：明绣（pal_mingxiu，shu 3 女）灵
// 火（linghuo，global phaseEnd 确认每 2 轮限一次）+归烬（guijin，出牌
// 阶段主动每 3 轮限一次）——本回合造成过伤害的其他角色阶段结束对其 1
// 火伤；亮 4 张逐张交付任意角色（获得者跳下次摸牌阶段）。chengxin 濒
// 死技能型救援通道缺口登记暂缓。
// ---------------------------------------------------------------------------

/// 灵火：伤害者阶段结束确认→1 火伤+2 轮冷却；冷却内不再触发。
#[test]
fn mingxiu_linghuo_round_cooldown_and_fire() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10023, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "linghuo");
    // 回合 1（初值 round=1）：0 号打 1 号一伤（自己阶段末不问=owner 排除）
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge（End 末不问）
    assert!(e.open_choice().is_none(), "自己阶段结束不触发");
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_deal_damage(Some(1), 2, 1, engine_native::event::DamageNature::Normal);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→2 Judge：灵火确认
    let c = e.open_choice().expect("灵火确认挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert_eq!(e.debug_hp(1), 4, "1 号吃 1 火伤（5→4）");
    assert_eq!(e.debug_hp(2), 5, "2 号仅吃 1 的伤害（6→5）");
    assert_eq!(
        e.state_core().player(0).linghuo_ready_round,
        3,
        "2 轮冷却置位"
    );
    assert_eq!(e.state_core().player(1).turn_damage, 1, "本回合伤害统计");

    // 回合 2：冷却内一律不触发（各阶段末均无询问）
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    let _ = submit(&mut e, 2, GameAction::NextPhase); // 2 Play（round→2 于 2→0 收束）
    e.debug_deal_damage(Some(2), 0, 1, engine_native::event::DamageNature::Normal);
    e.debug_clear_hand(2);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    let _ = submit(&mut e, 2, GameAction::NextPhase); // End→0 Judge：round→2
    assert!(e.open_choice().is_none(), "round 2 < 3：不触发");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 Play
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // 1 Play
    e.debug_deal_damage(Some(1), 2, 1, engine_native::event::DamageNature::Normal);
    e.debug_clear_hand(1);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→2 Judge
    assert!(e.open_choice().is_none(), "冷却内第二次不触发");

    // 回合 3：冷却结束——round 3 于 2→0 收束置位；2 号本回合已造成伤
    // 害 → 其 End 末（round 3）触发确认 → 2 号吃火伤
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    let _ = submit(&mut e, 2, GameAction::NextPhase); // 2 Play
    e.debug_deal_damage(Some(2), 0, 1, engine_native::event::DamageNature::Normal);
    e.debug_clear_hand(2);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    let _ = submit(&mut e, 2, GameAction::NextPhase); // End→0 Judge：round→3 + 灵火确认
    let c = e.open_choice().expect("灵火 round3 确认挂起");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert_eq!(
        e.debug_hp(2),
        3,
        "2 号（伤害者）吃 1 火伤（4→3；此前 r1/r2 各吃 1 伤害）"
    );
    assert_eq!(e.state_core().player(0).linghuo_ready_round, 5, "冷却续置");

    // round 3 后续：冷却已续置，0/1 号回合不再触发
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 Play
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    assert!(e.open_choice().is_none(), "续置冷却：不触发");
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // 1 Play
    e.debug_deal_damage(Some(1), 2, 1, engine_native::event::DamageNature::Normal);
    e.debug_clear_hand(1);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→2 Judge
    assert!(e.open_choice().is_none(), "续置冷却：仍不触发");
    // 终态血量核对：1 号 6-1(r1 0 伤)-1(火)-1(r2 0 伤)-1(r3 0 伤)=2；
    // 2 号 6-1(r1 1 伤)-1(r2 1 伤)-1(火)-1(r3 1 伤)=2
    assert_eq!(e.debug_hp(1), 2, "1 号终态血量");
    assert_eq!(e.debug_hp(2), 2, "2 号终态血量");
}

/// 归烬：亮 4 张逐张交付（获得者跳下次摸牌阶段）；取消弃置剩余。
#[test]
fn mingxiu_guijin_deal_and_skip_draw() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10024, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "guijin");
    e.debug_set_hp(1, 4);

    // 激活：亮 4 张 → 选第一张交 1 号 → 剩余 3 张取消弃置
    let def = e.catalog().skill_by_name("guijin").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("归烬亮牌选择挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (engine_native::content::host::WUDIAN_XJ_GUIJIN_CARDS, 0)
    );
    assert_eq!(c.spec.options.len(), 4, "亮 4 张");
    let given = match c.spec.options[0] {
        ChoiceOption::Card(x) => x,
        _ => panic!("选项应为牌"),
    };
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(given)],
        },
    );
    let c = e.open_choice().expect("归烬目标挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_GUIJIN_TARGET
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert_eq!(e.debug_hand(1), vec![given], "1 号获得首张");
    assert!(e.state_core().player(1).skip_next_draw, "获得者跳摸标记");
    assert_eq!(
        e.state_core().player(0).guijin_ready_round,
        4,
        "3 轮冷却置位（round 初值 1）"
    );

    // 第二轮询问：取消 → 剩余 3 张弃置
    let c = e.open_choice().expect("归烬第二轮挂起");
    assert_eq!(c.spec.options.len(), 3);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    );
    assert!(e.open_choice().is_none(), "取消后收束");
    assert_eq!(e.debug_hand(0).len(), 0, "亮牌不归手");
    assert_eq!(e.state_core().discard.len() >= 3, true, "剩余亮牌入弃堆");
    assert!(!e.state_core().discard.contains(&given), "已交付牌不在弃堆");

    // 0 号过回合 → 1 号 Judge→Draw：跳摸（手牌不变）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw（跳）
    assert_eq!(e.debug_hand(1).len(), 1, "跳过摸牌阶段（手牌不变）");
    assert!(!e.state_core().player(1).skip_next_draw, "跳摸标记已消费");
}

// ---------------------------------------------------------------------------
// U05 xianjian 第二十三切片（规则 v164）：明绣诚信（chengxin，global
// Dying 确认每 4 轮冷却）——濒死者回复至 1 并获免伤至其 End 阶段结束
// （濒死救援通道第二例；v160 暂缓消账）。
// ---------------------------------------------------------------------------

/// 诚信：他人濒死确认→回复至 1+免伤；冷却内置位；免伤 End 末清除。
#[test]
fn mingxiu_chengxin_rescue_and_immunity() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10032, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 6);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "chengxin");

    // 1 号压到 -2 濒死 → 确认 → 回复至 1（量=1-(-2)=3）+免伤置位
    e.debug_set_hp(1, -2);
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().expect("诚信确认挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert_eq!(e.debug_hp(1), 1, "回复至 1（-3→1）");
    assert!(e.state_core().player(1).chengxin_active, "免伤置位");
    assert_eq!(
        e.state_core().player(0).chengxin_ready_round,
        5,
        "4 轮冷却置位（round 初值 1）"
    );

    // 免伤：1 号再受伤无效果
    e.debug_deal_damage(Some(2), 1, 2, engine_native::event::DamageNature::Normal);
    assert_eq!(e.debug_hp(1), 1, "免伤：体力不变");

    // 0 号回合收束（3 NP）→ 1 号回合走完（End 末清免伤）→ 再受伤照常
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→2 Judge：免伤清
    assert!(!e.state_core().player(1).chengxin_active, "End 末免伤清除");
    e.debug_deal_damage(Some(2), 1, 1, engine_native::event::DamageNature::Normal);
    assert_eq!(e.debug_hp(1), 0, "免伤清除后照常受伤（1→0 濒死）");
    // 1 号濒死：诚信冷却中（round 1 <5）→ 无诚信询问 → 求桃链（无桃死亡）
    // 2 号有桃可救——给 2 号桃以稳态
    let _tao2 = e.debug_place(2, "tao");
    // 无人响应则 1 号死亡：此处仅断言无诚信确认门
    // （当前 open_choice 若有=求桃询问，observer=响应者；跳过应答致死）
    while let Some(c) = e.open_choice() {
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
    assert!(!e.debug_alive(1), "冷却中不救：1 号死亡");
}
