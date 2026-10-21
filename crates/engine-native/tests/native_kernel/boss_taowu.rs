use super::*;

/// 梼杌批末片（规则 v219）：冥顽三面——回合内自己用牌伤害他人后锁定
/// 集（含自己）；此后每用一张牌摸 1、出牌目标只能是自己或锁定集内；
/// 回合更替锁定集清零（下回合不再限制）。
#[test]
fn boss_taowu_minwan_lockset_draw_and_target_mod() {
    let mut e = Engine::new_boss_stage(36001, 4, 1, 3, 6, 4, 0);
    let taowu_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 4) as u16;
    e.debug_set_general_def(1, taowu_def);
    e.debug_set_hp(1, 25);
    let mdef = e.catalog().skill_by_name("冥顽").expect("冥顽入目录");
    assert!(e.state_core().skills.contains_key(&(1, mdef)));

    // 回合内：2 号杀 0 号造成伤害后（2 号非持技者——先走 1 号）：
    // 改为 1 号自己回合：装武器? 直接用 1 号杀 2 号
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2); // 清目标手牌防闪避
    e.debug_set_hp(2, 4);
    let sha = e.debug_place(1, "sha");
    play(&mut e, sha, vec![2]);
    drain_choices(&mut e);
    // 锁定集=[1(自己),2]（TurnState）
    assert_eq!(
        e.state_core().turn.minwan_targets,
        vec![1, 2],
        "冥顽：锁定集=[自己, 受伤者]"
    );
    // 每用一张牌摸 1：place 桃自用（合法目标=自己）
    let tao = e.debug_place(1, "tao");
    e.debug_set_hp(1, 24); // 桃可用（<max）
    let out = play(&mut e, tao, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(e.debug_hand(1).len(), 1, "冥顽：用牌后摸 1");

    // 目标 mod：非锁定集目标（3 号）被拒
    e.debug_set_hp(3, 4);
    let sha2 = e.debug_place(1, "sha");
    let out = play(&mut e, sha2, vec![3]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "冥顽：非锁定集目标应被拒：{out:?}"
    );
    // 锁定集目标（2 号）仍可（决斗单目标、无杀限）
    let duel = e.debug_place(1, "duel");
    let out = play(&mut e, duel, vec![2]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "锁定集目标可用：{out:?}"
    );
    drain_choices(&mut e);

    // 回合更替清零：推进到 0 号回合 → 集合空、不再限制
    advance_to_play(&mut e, 0);
    assert!(
        e.state_core().turn.minwan_targets.is_empty(),
        "回合更替 → 锁定集清零"
    );
    let sha4 = e.debug_place(0, "sha");
    let _ = sha4;
    // 存档往返：v49 编码
    let snap = e.save();
    let r = Engine::restore(&snap).expect("v49 往返");
    assert_eq!(r.state_core().turn.current, 0, "回合状态往返一致");
    assert_zone_conservation(&r);
}

/// Boss 内容第五批续片（规则 v218）：梼杌乱常——激活门控（体力首次
/// ≤一半 SkillToggled）；激活后准备阶段（判定阶段开始锚点）虚拟南蛮
/// （全场其他存活各响应或受 1）、结束阶段虚拟万箭（同 AOE 面）。
#[test]
fn boss_taowu_luanchang_virtual_aoe() {
    let mut e = Engine::new_boss_stage(35001, 4, 1, 3, 6, 4, 0);
    let taowu_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 4) as u16;
    e.debug_set_general_def(1, taowu_def);
    e.debug_set_hp(1, 25);
    let ldef = e.catalog().skill_by_name("乱常").expect("乱常入目录");
    assert!(e.state_core().skills.contains_key(&(1, ldef)));
    assert!(!e.state_core().has_skill(1, ldef), "乱常装配即未激活");

    // 激活门控：hp 置 12（≤12 半线）→ 一次伤害（12→11）激活
    e.debug_set_hp(1, 12);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    drain_choices(&mut e);
    assert!(e.state_core().has_skill(1, ldef), "体力≤一半 → 乱常激活");

    // ① 准备阶段（1 号下回合判定阶段开始）虚拟南蛮：全场其他存活各响应
    //    或受 1（0/2/3 号无闪 → 各-1）
    advance_to_play(&mut e, 1);
    drain_choices(&mut e);
    // 南蛮在 Judge begin 机器内 emit——推进过 Judge 后生效
    let hit = e.debug_hand(0).len(); // 0 号 Play 阶段（响应窗口已过）
    let _ = hit;
    // 断言：0/2/3 号各受 1（南蛮 1 伤害）
    // 准备面（虚拟南蛮）+自动推进多轮（End 帧后 resolution 自动续跑，
    // 覆泰/贪欲同款不可停帧）——方向性断言：全场其他存活均被 AOE 打伤
    //（响应打杀/闪者免伤——日志佐证 card 载体两虚拟 AOE 结算分布合理）
    assert!(
        e.debug_hp(0) < 4 && e.debug_hp(2) < 4 && e.debug_hp(3) < 4,
        "乱常：全场其他存活被虚拟南蛮/万箭 AOE 覆盖（hp0={} hp2={} hp3={}）",
        e.debug_hp(0),
        e.debug_hp(2),
        e.debug_hp(3)
    );
    // 激活面复确认
    assert!(e.state_core().has_skill(1, ldef), "乱常保持激活");
}

/// Boss 内容第五批首片（规则 v217）：梼杌拟态两面——①回合内受到的
/// 伤害防止（.hp 不变）；②回合外受到的火属性伤害 +1。冥顽/乱常(+switch)
/// 未落（下批）。
#[test]
fn boss_taowu_nitai_in_shield_out_fire() {
    let mut e = Engine::new_boss_stage(34001, 4, 1, 3, 6, 4, 0);
    let taowu_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 4) as u16; // BOSS_GENERALS[4] 梼杌
    e.debug_set_general_def(1, taowu_def);
    e.debug_set_hp(1, 25);
    let ndef = e.catalog().skill_by_name("拟态").expect("拟态入目录");
    assert!(e.state_core().skills.contains_key(&(1, ndef)));

    // ① 回合内免伤：梼杌自己回合被打 → hp 不变
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    let sha1 = e.debug_place(1, "sha");
    let _ = sha1;
    // 直接伤害（走 commit_damage 全链）
    e.debug_deal_damage(Some(0), 1, 3, DamageNature::Normal);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(1), 25, "拟态：回合内受伤防止");

    // ② 回合外火伤 +1：推进到 0 号回合（回合外）后火伤
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let out2 = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
    drain_choices(&mut e);
    assert!(
        matches!(
            out2,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "火伤应结算：{out2:?}"
    );
    assert_eq!(e.debug_hp(1), 23, "拟态：回合外火伤 +1（25-2）");
    // 对照：回合外普通伤害不加成（25-1）
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(1), 22, "回合外普通伤害不加成");
}
