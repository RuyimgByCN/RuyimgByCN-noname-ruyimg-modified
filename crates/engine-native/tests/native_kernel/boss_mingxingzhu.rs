use super::*;

/// 星宿阶段2系明星片（规则 v245）：庆祝+夹击+急叮三技——①庆祝跳弃牌+
/// 杀禁用（获急叮解除）②夹击：相邻敌方坐骑全弃 ③急叮：友方受伤→对敌
/// 方来源虚拟雷杀。明星条目（群目录近似/hp3/female/[庆祝,夹击,急叮]；
/// 蓐收加成尾面登记差异）。
#[test]
fn boss_mingxingzhu_qingzhu_jiazu_jiding() {
    // 建局：1=明星（hp3）；fellow 4=蓐收（boss 方受伤者）
    let mut e = Engine::new_boss_stage(65001, 4, 1, 3, 6, 4, 0);
    let mx_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 22) as u16; // BOSS_GENERALS[22] 明星
    e.debug_set_general_def(1, mx_def);
    e.debug_set_hp(1, 3);
    assert_eq!(
        e.catalog().general_meta(mx_def).map(|g| g.0),
        Some("明星"),
        "明星入目录"
    );
    let qz = e.catalog().skill_by_name("庆祝").expect("庆祝入目录");
    let jz = e.catalog().skill_by_name("夹击").expect("夹击入目录");
    let jd = e.catalog().skill_by_name("急叮").expect("急叮入目录");
    assert!(e.state_core().skills.contains_key(&(1, qz)));
    assert!(e.state_core().skills.contains_key(&(1, jz)));
    assert!(
        !e.state_core().skills.contains_key(&(1, jd)),
        "急叮不随条目装配（星囚授予面）"
    );

    // ① 庆祝杀禁用：明星出牌阶段杀被拒
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    let sha = e.debug_place(1, "sha");
    let out = play(&mut e, sha, vec![2]);
    assert!(
        !matches!(out, CommandOutcome::Accepted { .. }),
        "庆祝：无急叮不能用杀"
    );

    // ③ 急叮：先授急叮（模拟星囚 addSkills 授予面）——fellow（boss 方座
    //    位 4）受 0 号伤害 → 明星对 0 虚拟雷杀
    {
        let core = e.state_core_mut();
        core.skills.insert(
            (1, jd),
            engine_native::state::SkillInstance {
                owner: 1,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
    }
    let fellow = e.debug_add_fellow(4).expect("fellow 入队");
    // fellow 不装配武将（无技伤害载体）
    e.debug_clear_hand(0);
    e.debug_set_hp(0, 4);
    let hp0 = e.debug_hp(0);
    e.debug_deal_damage(Some(0), fellow, 1, DamageNature::Normal);
    // 虚拟雷杀直发（机器直发不经询问）→ 0 号受 1 雷伤
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(0), hp0 - 1, "急叮：对敌方来源虚拟雷杀（1 雷）");

    // ② 夹击：2 号（明星相邻下家）持坐骑 → 被全弃
    let horse = e.debug_place(2, "horse_plus");
    {
        let core = e.state_core_mut();
        let pos = core.players[2]
            .hand
            .iter()
            .position(|c| *c == horse)
            .expect("horse in hand");
        let c = core.players[2].hand.remove(pos);
        core.players[2].equips.push(c);
    }
    e.debug_clear_hand(2);
    let h3_before = e.debug_hand(3).len();
    e.debug_clear_hand(3);
    // 3 号也持坐骑但非相邻（对座）→ 不触发
    let horse3 = e.debug_place(3, "horse_minus");
    {
        let core = e.state_core_mut();
        let pos = core.players[3]
            .hand
            .iter()
            .position(|c| *c == horse3)
            .expect("horse3 in hand");
        let c = core.players[3].hand.remove(pos);
        core.players[3].equips.push(c);
    }
    // 推进到明星下一回合（准备阶段触发夹击）
    next_phase(&mut e); // 1 Play→Discard（庆祝跳弃牌）
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End
    drain_choices(&mut e);
    advance_to_play(&mut e, 1);
    assert!(
        e.state_core().player(2).equips.is_empty(),
        "夹击：相邻敌方坐骑全弃"
    );
    assert!(
        !e.state_core().player(3).equips.is_empty(),
        "对座非相邻：坐骑保留"
    );

    // ④ 庆祝跳弃牌（手牌超上限保留）+ 杀禁用解除（急叮已授）
    let keep = e.debug_place(1, "tao");
    next_phase(&mut e); // Play→Discard（庆祝跳）
    drain_choices(&mut e);
    assert!(e.debug_hand(1).contains(&keep), "庆祝：跳弃牌手牌保留");
    next_phase(&mut e); // Discard→End
    drain_choices(&mut e);
    advance_to_play(&mut e, 1);
    let sha2 = e.debug_place(1, "sha");
    let out = play(&mut e, sha2, vec![2]);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "获急叮后杀解除（{out:?}）"
    );
    drain_choices(&mut e);
    let _ = h3_before;

    // ⑤ 存档往返
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(r.state_core().skills.contains_key(&(1, qz)), "庆祝实例往返");
    assert_zone_conservation(&r);
    assert_zone_conservation(&e);
}
