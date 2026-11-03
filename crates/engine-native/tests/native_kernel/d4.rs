use super::*;

// probe: bianchengwang death flow

// ---------------------------------------------------------------------------
// D4 Boss 关卡流程子系统 B0（ADR 0783 / T11-BOSS-STAGE-SUBSYSTEM）
// ---------------------------------------------------------------------------

/// B0-STAGE-STATE + B0-ARCHIVE-ROUNDTRIP：默认 kind=None；encode→decode 全等。
#[test]
fn d4_boss_b0_stage_state_default_none_roundtrip() {
    use engine_native::archive::{decode_archive, encode_archive};
    use engine_native::{BossStageKind, BossStageState};

    let e = Engine::new_boss(29101, 3, 0, 0);
    let stage = e
        .save()
        .core
        .boss_stage()
        .expect("Boss 模式应有 stage")
        .clone();
    assert_eq!(stage.kind, BossStageKind::None);
    assert_eq!(stage.level, 0);
    assert!(!stage.mengpo_spawned);
    assert!(!stage.mengpo_died);
    assert!(stage.banned_generals.is_empty());
    assert_eq!(stage.wing_armed, 0);
    assert!(stage.pending.is_none());
    assert_eq!(stage, BossStageState::default_none());

    let bytes = encode_archive(&e.save());
    let back = decode_archive(&bytes).expect("v79 解码");
    assert_eq!(back.core.mode, e.save().core.mode);
    assert_eq!(
        back.core.boss_stage(),
        Some(&BossStageState::default_none())
    );
}

/// B0-ARCHIVE-ROUNDTRIP：旧档格式被门禁拒绝（无迁移）。
#[test]
fn d4_boss_b0_archive_format_v79_rejects_v78_without_migration() {
    use engine_native::archive::{
        decode_archive, encode_archive, ArchiveError, ARCHIVE_FORMAT_VERSION, ARCHIVE_MAGIC,
    };

    // R05-G 批（2026-11-04）：106→107——detached 实体区入档，v106 拒收无迁移。
    assert_eq!(ARCHIVE_FORMAT_VERSION, 107);
    let e = Engine::new_boss(29102, 3, 0, 0);
    let mut bytes = encode_archive(&e.save());
    // 伪造头格式版本 84（载荷仍为 v86 布局）——门禁应拒收。
    bytes[4..6].copy_from_slice(&84u16.to_le_bytes());
    assert_eq!(&bytes[0..4], ARCHIVE_MAGIC);
    match decode_archive(&bytes) {
        Err(ArchiveError::UnsupportedFormatVersion(84)) => {}
        other => panic!("期望拒收 v84，得到 {other:?}"),
    }
}

/// B0-HASH-PROBE：仅改 stage 字段 → 摘要变化。
#[test]
fn d4_boss_b0_hash_includes_stage_fields() {
    let e = Engine::new_boss(29103, 3, 0, 0);
    let h0 = e.state_hash();
    // 经快照注入改 level（不经默认对局路径）
    let mut snap = e.save();
    match &mut snap.core.mode {
        GameMode::Boss { stage, .. } => stage.level = 2,
        _ => panic!("Boss"),
    }
    let e2 = Engine::restore(&snap).expect("恢复");
    assert_ne!(h0, e2.state_hash(), "level 变化应位移摘要");
    match &mut snap.core.mode {
        GameMode::Boss { stage, .. } => {
            stage.level = 0;
            stage.mengpo_spawned = true;
        }
        _ => panic!("Boss"),
    }
    let e3 = Engine::restore(&snap).expect("恢复");
    assert_ne!(h0, e3.state_hash(), "mengpo_spawned 变化应位移摘要");
}

/// B0-CHANGE-BOSS-API：换将更新装配；座位环长度不变。
#[test]
fn d4_boss_b0_change_boss_api_reinit_updates_pointer() {
    use engine_native::content::standard::BOSS_GENERALS;

    let mut e = Engine::new_boss(29104, 3, 0, 0);
    advance_to_play(&mut e, 0);
    let base = (e.catalog().general_count() - BOSS_GENERALS.len()) as u16;
    let zhuyin = base; // BOSS_GENERALS[0]
    let hundun = base + 1; // BOSS_GENERALS[1]
    e.debug_set_general_def(0, zhuyin);
    let ring_before = e.save().core.seat_ring.clone();
    let hp_before = e.debug_hp(0);
    assert!(e.change_boss(hundun, true), "change_boss 应成功");
    let (boss, _) = e.save().core.boss_mode().expect("Boss");
    assert_eq!(boss, 0, "boss 指针不变");
    assert_eq!(
        e.save().core.players[0].general_slots[0],
        Some(hundun),
        "主将槽更新"
    );
    // 混沌携带无脏等；烛阴仅凶兽——换将后应有无脏实例
    let wuzang = e.catalog().skill_by_name("无脏").expect("无脏");
    assert!(
        e.save().core.skills.contains_key(&(0, wuzang)),
        "新将技能已装配"
    );
    let xiongshou = e.catalog().skill_by_name("凶兽").expect("凶兽");
    // 凶兽为共有技：混沌条目亦携带，故仍在；断言主将槽已换即可
    let _ = xiongshou;
    assert_ne!(zhuyin, hundun);
    assert_eq!(e.save().core.seat_ring, ring_before, "座位环不破坏");
    assert_eq!(
        e.debug_hp(0),
        hp_before.min(e.save().core.players[0].max_hp as i32)
    );
}

/// B0-ADD-FELLOW-API：正式 API 入队尾并同步 seat_ring；boss 侧判定含 fellow。
#[test]
fn d4_boss_b0_add_fellow_api_appends_and_marks_boss_side() {
    let mut e = Engine::new_boss(29105, 3, 0, 0);
    advance_to_play(&mut e, 0);
    let fellow = e.add_boss_fellow(4).expect("入队");
    assert_eq!(fellow, 3);
    assert_eq!(e.save().core.boss_fellows().unwrap(), &[3]);
    assert_eq!(e.save().core.seat_ring.len(), 4);
    assert_eq!(*e.save().core.seat_ring.last().unwrap(), 3);
    assert!(e.save().core.is_boss_side(3));
    assert!(!e.save().core.is_boss_side(1));
}

/// B0-BAN-QUERY-STUB：恒未禁、旗 getter 默认 false。
#[test]
fn d4_boss_b0_ban_query_stub_always_false() {
    let e = Engine::new_boss(29106, 3, 0, 0);
    for id in 0..64u16 {
        assert!(
            !e.save().core.is_boss_general_banned(id),
            "B0 禁将空表 id={id}"
        );
    }
    assert!(!e.save().core.boss_mengpo_spawned());
    assert!(!e.save().core.boss_mengpo_died());
    // 非 Boss 模式同样恒 false
    let p = Engine::new(29107, 2);
    assert!(!p.save().core.is_boss_general_banned(0));
    assert!(!p.save().core.boss_mengpo_spawned());
}

/// 青木链 1→2：青龙死亡 → 勾芒 + 两书精 fellow + 挑战者重整 + stage.
#[test]
fn d4_boss_b1_qingmu2_chain_to_goumang() {
    use engine_native::BossStageKind;

    let mut e = Engine::new_boss_stage(29201, 4, 0, 0, 4, 4, 0);
    let qinglong = boss_general_named(&e, "青龙");
    let goumang = boss_general_named(&e, "勾芒");
    let shujing = boss_general_named(&e, "书精");
    e.debug_set_general_def(0, qinglong);
    e.debug_set_hp(0, 4);
    advance_to_play(&mut e, 0);
    assert!(
        e.catalog().skill_by_name("青木").is_some()
            && e.state_core()
                .skills
                .contains_key(&(0, e.catalog().skill_by_name("青木").unwrap())),
        "青龙应装配青木"
    );

    // 击杀 Boss：清手防自救 → 过量伤害
    for s in 0..4u8 {
        e.debug_clear_hand(s);
    }
    e.debug_deal_damage(Some(1), 0, 8, DamageNature::Normal);
    drain_choices(&mut e);

    assert!(e.debug_alive(0), "转阶后 Boss 座位应存活");
    assert_eq!(
        e.state_core().players[0].general_slots[0],
        Some(goumang),
        "应换为勾芒"
    );
    let fellows = e.state_core().boss_fellows().expect("fellows");
    assert_eq!(fellows.len(), 2, "应追加两书精 fellow");
    for &f in fellows {
        assert_eq!(
            e.state_core().players[f as usize].general_slots[0],
            Some(shujing),
            "fellow 应为书精"
        );
        assert!(e.state_core().is_boss_side(f));
    }
    let stage = e.state_core().boss_stage().expect("stage");
    assert_eq!(stage.kind, BossStageKind::SixiangChain);
    assert_eq!(stage.level, 2);
    assert_eq!(e.state_core().turn.current, 0, "回合锚定 Boss");
    // 挑战者应回满
    for t in 1..4u8 {
        assert!(e.debug_alive(t));
        assert_eq!(
            e.debug_hp(t),
            e.state_core().players[t as usize].max_hp as i32
        );
    }
    assert_zone_conservation(&e);

    // 存档往返
    let snap = e.save();
    let e2 = Engine::restore(&snap).expect("restore");
    assert_eq!(e2.state_core().players[0].general_slots[0], Some(goumang));
    assert_eq!(e2.state_core().boss_stage().unwrap().level, 2);
}

/// 青木3 链 2→3：勾芒死亡 → 太昊 + 侧座勾芒/书精。
#[test]
fn d4_boss_b1_qingmu3_chain_to_taihao_sides() {
    use engine_native::BossStageKind;

    let mut e = Engine::new_boss_stage(29202, 4, 0, 0, 4, 4, 0);
    let goumang = boss_general_named(&e, "勾芒");
    let taihao = boss_general_named(&e, "太昊");
    let shujing = boss_general_named(&e, "书精");
    e.debug_set_general_def(0, goumang);
    e.debug_set_hp(0, 5);
    // 侧座预置无关将，转阶后应被覆盖
    e.debug_set_general_def(1, boss_general_named(&e, "书精"));
    e.debug_set_general_def(3, boss_general_named(&e, "书精"));
    advance_to_play(&mut e, 0);
    assert!(
        e.state_core()
            .skills
            .contains_key(&(0, e.catalog().skill_by_name("青木3").unwrap())),
        "勾芒应装配青木3"
    );

    for s in 0..4u8 {
        e.debug_clear_hand(s);
    }
    e.debug_deal_damage(Some(1), 0, 10, DamageNature::Normal);
    drain_choices(&mut e);

    assert!(e.debug_alive(0));
    assert_eq!(e.state_core().players[0].general_slots[0], Some(taihao));
    let prev = e.state_core().ring_prev(0);
    let next = e.state_core().ring_next(0);
    assert_eq!(
        e.state_core().players[prev as usize].general_slots[0],
        Some(goumang),
        "prevSeat → 勾芒"
    );
    assert_eq!(
        e.state_core().players[next as usize].general_slots[0],
        Some(shujing),
        "nextSeat → 书精"
    );
    assert!(e.state_core().is_boss_side(prev));
    assert!(e.state_core().is_boss_side(next));
    let stage = e.state_core().boss_stage().expect("stage");
    assert_eq!(stage.kind, BossStageKind::SixiangChain);
    assert_eq!(stage.level, 3);
    assert_zone_conservation(&e);
}

/// HostBehavior tag 411–414 往返。
#[test]
fn d4_boss_b1_qingmu_host_tags() {
    use engine_native::content::host::HostBehavior;
    assert_eq!(HostBehavior::BossQingmu2.tag(), 411);
    assert_eq!(HostBehavior::BossQingmu2x.tag(), 412);
    assert_eq!(HostBehavior::BossQingmu3.tag(), 413);
    assert_eq!(HostBehavior::BossQingmu3x.tag(), 414);
    assert_eq!(HostBehavior::from_tag(411), Some(HostBehavior::BossQingmu2));
    assert_eq!(
        HostBehavior::from_tag(414),
        Some(HostBehavior::BossQingmu3x)
    );
}

// ---------------------------------------------------------------------------
// D4/B2 白芒/赤炎/玄麟隐藏链（tag 415–420）
// ---------------------------------------------------------------------------

/// 白芒链 1→2：白虎死亡 → 蓐收 + 两明星 fellow。
#[test]
fn d4_boss_b2_baimang2_chain_to_rushou() {
    use engine_native::BossStageKind;

    let mut e = Engine::new_boss_stage(29301, 4, 0, 0, 4, 4, 0);
    let baihu = boss_general_named(&e, "白虎");
    let rushou = boss_general_named(&e, "蓐收");
    let mingxing = boss_general_named(&e, "明星");
    e.debug_set_general_def(0, baihu);
    e.debug_set_hp(0, 4);
    advance_to_play(&mut e, 0);
    assert!(
        e.state_core()
            .skills
            .contains_key(&(0, e.catalog().skill_by_name("白芒").unwrap())),
        "白虎应装配白芒"
    );

    for s in 0..4u8 {
        e.debug_clear_hand(s);
    }
    e.debug_deal_damage(Some(1), 0, 8, DamageNature::Normal);
    drain_choices(&mut e);

    assert!(e.debug_alive(0), "转阶后 Boss 座位应存活");
    assert_eq!(
        e.state_core().players[0].general_slots[0],
        Some(rushou),
        "应换为蓐收"
    );
    let fellows = e.state_core().boss_fellows().expect("fellows");
    assert_eq!(fellows.len(), 2, "应追加两明星 fellow");
    for &f in fellows {
        assert_eq!(
            e.state_core().players[f as usize].general_slots[0],
            Some(mingxing),
            "fellow 应为明星"
        );
        assert!(e.state_core().is_boss_side(f));
    }
    let stage = e.state_core().boss_stage().expect("stage");
    assert_eq!(stage.kind, BossStageKind::SixiangChain);
    assert_eq!(stage.level, 2);
    assert_eq!(e.state_core().turn.current, 0);
    for t in 1..4u8 {
        assert!(e.debug_alive(t));
        assert_eq!(
            e.debug_hp(t),
            e.state_core().players[t as usize].max_hp as i32
        );
    }
    assert_zone_conservation(&e);

    let snap = e.save();
    let e2 = Engine::restore(&snap).expect("restore");
    assert_eq!(e2.state_core().players[0].general_slots[0], Some(rushou));
    assert_eq!(e2.state_core().boss_stage().unwrap().level, 2);
}

/// 赤炎链 1→2：朱雀死亡 → 祝融 + 两焰灵 fellow。
#[test]
fn d4_boss_b2_chiyan2_chain_to_zhurong() {
    use engine_native::BossStageKind;

    let mut e = Engine::new_boss_stage(29302, 4, 0, 0, 4, 4, 0);
    let zhuque = boss_general_named(&e, "朱雀");
    let zhurong = boss_general_named(&e, "祝融");
    let yanling = boss_general_named(&e, "焰灵");
    e.debug_set_general_def(0, zhuque);
    e.debug_set_hp(0, 4);
    advance_to_play(&mut e, 0);
    assert!(
        e.state_core()
            .skills
            .contains_key(&(0, e.catalog().skill_by_name("赤炎").unwrap())),
        "朱雀应装配赤炎"
    );

    for s in 0..4u8 {
        e.debug_clear_hand(s);
    }
    e.debug_deal_damage(Some(1), 0, 8, DamageNature::Normal);
    drain_choices(&mut e);

    assert!(e.debug_alive(0));
    assert_eq!(
        e.state_core().players[0].general_slots[0],
        Some(zhurong),
        "应换为祝融"
    );
    let fellows = e.state_core().boss_fellows().expect("fellows");
    assert_eq!(fellows.len(), 2, "应追加两焰灵 fellow");
    for &f in fellows {
        assert_eq!(
            e.state_core().players[f as usize].general_slots[0],
            Some(yanling),
            "fellow 应为焰灵"
        );
    }
    let stage = e.state_core().boss_stage().expect("stage");
    assert_eq!(stage.kind, BossStageKind::SixiangChain);
    assert_eq!(stage.level, 2);
    assert_zone_conservation(&e);
}

/// 玄麟链 1→2：玄武死亡 → 共工 + 一玄冥 fellow。
/// 注：玄武灵渠将 amount>1 归零，故以 1 体 + 1 伤击杀。
#[test]
fn d4_boss_b2_xuanlin2_chain_to_gonggong() {
    use engine_native::BossStageKind;

    let mut e = Engine::new_boss_stage(29303, 4, 0, 0, 4, 4, 0);
    let xuanwu = boss_general_named(&e, "玄武");
    let gonggong = boss_general_named(&e, "共工");
    let xuanming = boss_general_named(&e, "玄冥");
    e.debug_set_general_def(0, xuanwu);
    e.debug_set_hp(0, 1);
    advance_to_play(&mut e, 0);
    assert!(
        e.state_core()
            .skills
            .contains_key(&(0, e.catalog().skill_by_name("玄麟").unwrap())),
        "玄武应装配玄麟"
    );

    for s in 0..4u8 {
        e.debug_clear_hand(s);
    }
    // 灵渠：受伤>1 归零；1 伤照常，且手牌已清空防桃救
    e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    drain_choices(&mut e);

    assert!(e.debug_alive(0));
    assert_eq!(
        e.state_core().players[0].general_slots[0],
        Some(gonggong),
        "应换为共工"
    );
    let fellows = e.state_core().boss_fellows().expect("fellows");
    assert_eq!(fellows.len(), 1, "应追加一玄冥 fellow");
    assert_eq!(
        e.state_core().players[fellows[0] as usize].general_slots[0],
        Some(xuanming),
        "fellow 应为玄冥"
    );
    let stage = e.state_core().boss_stage().expect("stage");
    assert_eq!(stage.kind, BossStageKind::SixiangChain);
    assert_eq!(stage.level, 2);
    assert_zone_conservation(&e);
}

/// HostBehavior tag 415–420 往返。
#[test]
fn d4_boss_b2_sixiang_host_tags() {
    use engine_native::content::host::HostBehavior;
    assert_eq!(HostBehavior::BossBaimang2.tag(), 415);
    assert_eq!(HostBehavior::BossBaimang2x.tag(), 416);
    assert_eq!(HostBehavior::BossChiyan2.tag(), 417);
    assert_eq!(HostBehavior::BossChiyan2x.tag(), 418);
    assert_eq!(HostBehavior::BossXuanlin2.tag(), 419);
    assert_eq!(HostBehavior::BossXuanlin2x.tag(), 420);
    assert_eq!(
        HostBehavior::from_tag(415),
        Some(HostBehavior::BossBaimang2)
    );
    assert_eq!(
        HostBehavior::from_tag(420),
        Some(HostBehavior::BossXuanlin2x)
    );
}

// ---------------------------------------------------------------------------
// D4/B3 白芒/赤炎/玄麟 3/3x（tag 421–426；侧座 reinit 复用青木3）
// ---------------------------------------------------------------------------

/// 白芒3 链 2→3：蓐收死亡 → 少昊 + 侧座蓐收/明星 + 一明星 fellow。
#[test]
fn d4_boss_b3_baimang3_chain_to_shaohao_sides() {
    use engine_native::BossStageKind;

    let mut e = Engine::new_boss_stage(29401, 4, 0, 0, 4, 4, 0);
    let rushou = boss_general_named(&e, "蓐收");
    let shaohao = boss_general_named(&e, "少昊");
    let mingxing = boss_general_named(&e, "明星");
    e.debug_set_general_def(0, rushou);
    e.debug_set_hp(0, 5);
    e.debug_set_general_def(1, boss_general_named(&e, "明星"));
    e.debug_set_general_def(3, boss_general_named(&e, "明星"));
    advance_to_play(&mut e, 0);
    assert!(
        e.state_core()
            .skills
            .contains_key(&(0, e.catalog().skill_by_name("白芒3").unwrap())),
        "蓐收应装配白芒3"
    );

    for s in 0..4u8 {
        e.debug_clear_hand(s);
    }
    e.debug_deal_damage(Some(1), 0, 10, DamageNature::Normal);
    drain_choices(&mut e);

    assert!(e.debug_alive(0));
    assert_eq!(e.state_core().players[0].general_slots[0], Some(shaohao));
    // 4 人环侧座固定 3=prev / 1=next；add_fellow 队尾后 ring_prev 会漂到新
    // fellow，故按座位号断言（与建局预置一致）。
    assert_eq!(
        e.state_core().players[3].general_slots[0],
        Some(rushou),
        "prevSeat(3) → 蓐收"
    );
    assert_eq!(
        e.state_core().players[1].general_slots[0],
        Some(mingxing),
        "nextSeat(1) → 明星"
    );
    assert!(e.state_core().is_boss_side(3));
    assert!(e.state_core().is_boss_side(1));
    let fellows = e.state_core().boss_fellows().expect("fellows");
    assert!(
        fellows.iter().any(|&f| {
            f >= 4 && e.state_core().players[f as usize].general_slots[0] == Some(mingxing)
        }),
        "应追加一明星 fellow（队尾）"
    );
    let stage = e.state_core().boss_stage().expect("stage");
    assert_eq!(stage.kind, BossStageKind::SixiangChain);
    assert_eq!(stage.level, 3);
    assert_zone_conservation(&e);
}

/// 赤炎3 链 2→3：祝融死亡 → 炎帝 + 侧座祝融/焰灵。
#[test]
fn d4_boss_b3_chiyan3_chain_to_yandi_sides() {
    use engine_native::BossStageKind;

    let mut e = Engine::new_boss_stage(29402, 4, 0, 0, 4, 4, 0);
    let zhurong = boss_general_named(&e, "祝融");
    let yandi = boss_general_named(&e, "炎帝");
    let yanling = boss_general_named(&e, "焰灵");
    e.debug_set_general_def(0, zhurong);
    e.debug_set_hp(0, 5);
    e.debug_set_general_def(1, boss_general_named(&e, "焰灵"));
    e.debug_set_general_def(3, boss_general_named(&e, "焰灵"));
    advance_to_play(&mut e, 0);
    assert!(
        e.state_core()
            .skills
            .contains_key(&(0, e.catalog().skill_by_name("赤炎3").unwrap())),
        "祝融应装配赤炎3"
    );

    for s in 0..4u8 {
        e.debug_clear_hand(s);
    }
    e.debug_deal_damage(Some(1), 0, 10, DamageNature::Normal);
    drain_choices(&mut e);

    assert!(e.debug_alive(0));
    assert_eq!(e.state_core().players[0].general_slots[0], Some(yandi));
    let prev = e.state_core().ring_prev(0);
    let next = e.state_core().ring_next(0);
    assert_eq!(
        e.state_core().players[prev as usize].general_slots[0],
        Some(zhurong),
        "prevSeat → 祝融"
    );
    assert_eq!(
        e.state_core().players[next as usize].general_slots[0],
        Some(yanling),
        "nextSeat → 焰灵"
    );
    assert!(e.state_core().is_boss_side(prev));
    assert!(e.state_core().is_boss_side(next));
    let stage = e.state_core().boss_stage().expect("stage");
    assert_eq!(stage.kind, BossStageKind::SixiangChain);
    assert_eq!(stage.level, 3);
    assert_zone_conservation(&e);
}

/// 玄麟3 链 2→3：共工死亡 → 颛顼 + 共工 fellow + 次座玄冥。
#[test]
fn d4_boss_b3_xuanlin3_chain_to_zhuanxu() {
    use engine_native::BossStageKind;

    let mut e = Engine::new_boss_stage(29403, 4, 0, 0, 4, 4, 0);
    let gonggong = boss_general_named(&e, "共工");
    let zhuanxu = boss_general_named(&e, "颛顼");
    let xuanming = boss_general_named(&e, "玄冥");
    e.debug_set_general_def(0, gonggong);
    e.debug_set_hp(0, 5);
    e.debug_set_general_def(1, boss_general_named(&e, "玄冥"));
    e.debug_set_general_def(3, boss_general_named(&e, "玄冥"));
    advance_to_play(&mut e, 0);
    assert!(
        e.state_core()
            .skills
            .contains_key(&(0, e.catalog().skill_by_name("玄麟3").unwrap())),
        "共工应装配玄麟3"
    );

    for s in 0..4u8 {
        e.debug_clear_hand(s);
    }
    e.debug_deal_damage(Some(1), 0, 10, DamageNature::Normal);
    drain_choices(&mut e);

    assert!(e.debug_alive(0));
    assert_eq!(
        e.state_core().players[0].general_slots[0],
        Some(zhuanxu),
        "应换为颛顼"
    );
    let next = e.state_core().ring_next(0);
    assert_eq!(
        e.state_core().players[next as usize].general_slots[0],
        Some(xuanming),
        "nextSeat → 玄冥"
    );
    assert!(e.state_core().is_boss_side(next));
    let fellows = e.state_core().boss_fellows().expect("fellows");
    assert!(
        fellows
            .iter()
            .any(|&f| { e.state_core().players[f as usize].general_slots[0] == Some(gonggong) }),
        "应追加一共工 fellow"
    );
    let stage = e.state_core().boss_stage().expect("stage");
    assert_eq!(stage.kind, BossStageKind::SixiangChain);
    assert_eq!(stage.level, 3);
    assert_zone_conservation(&e);
}

/// HostBehavior tag 421–426 往返。
#[test]
fn d4_boss_b3_sixiang3_host_tags() {
    use engine_native::content::host::HostBehavior;
    assert_eq!(HostBehavior::BossBaimang3.tag(), 421);
    assert_eq!(HostBehavior::BossBaimang3x.tag(), 422);
    assert_eq!(HostBehavior::BossChiyan3.tag(), 423);
    assert_eq!(HostBehavior::BossChiyan3x.tag(), 424);
    assert_eq!(HostBehavior::BossXuanlin3.tag(), 425);
    assert_eq!(HostBehavior::BossXuanlin3x.tag(), 426);
    assert_eq!(
        HostBehavior::from_tag(421),
        Some(HostBehavior::BossBaimang3)
    );
    assert_eq!(
        HostBehavior::from_tag(426),
        Some(HostBehavior::BossXuanlin3x)
    );
}

#[test]
fn d4_boss_b4_host_tags() {
    use engine_native::content::host::HostBehavior;
    assert_eq!(HostBehavior::BossBianshen3.tag(), 427);
    assert_eq!(HostBehavior::BossBianshen3x.tag(), 428);
    assert_eq!(HostBehavior::BossBianshen4.tag(), 429);
    assert_eq!(HostBehavior::BossBianshen4x.tag(), 430);
    assert_eq!(HostBehavior::BossHuanren.tag(), 431);
    assert_eq!(HostBehavior::BossHuanrenDie.tag(), 432);
    assert_eq!(HostBehavior::BossQingyi.tag(), 433);
    assert_eq!(HostBehavior::BossChiyi.tag(), 434);
    assert_eq!(HostBehavior::BossChiyiMark.tag(), 435);
    assert_eq!(HostBehavior::BossBaiyi.tag(), 436);
    assert_eq!(HostBehavior::BossBaiyiThunder.tag(), 437);
    assert_eq!(
        HostBehavior::from_tag(427),
        Some(HostBehavior::BossBianshen3)
    );
    assert_eq!(
        HostBehavior::from_tag(437),
        Some(HostBehavior::BossBaiyiThunder)
    );
}

#[test]
fn d4_boss_b4_bianshen3_changes_boss_seeded() {
    let a = bianshen_outcome(29501, "牛头");
    let b = bianshen_outcome(29501, "牛头");
    assert_eq!(a, b, "同一种子换将结果相同");
    assert!(
        a == "白无常" || a == "黑无常",
        "后援3 应换白/黑无常，实得 {a}"
    );
    let c = bianshen_outcome(29502, "马面");
    assert!(c == "白无常" || c == "黑无常", "马面后援3 实得 {c}");
}

#[test]
fn d4_boss_b4_bianshen4_changes_boss_seeded() {
    let a = bianshen_outcome(29503, "黑无常");
    let b = bianshen_outcome(29503, "黑无常");
    assert_eq!(a, b, "同一种子换将结果相同");
    assert!(a == "夜叉" || a == "罗刹", "后援4 应换夜叉/罗刹，实得 {a}");
    let c = bianshen_outcome(29504, "白无常");
    assert!(c == "夜叉" || c == "罗刹", "白无常后援4 实得 {c}");
}

#[test]
fn d4_boss_b4_huanren_boot_bans_and_may_spawn_mengpo() {
    use engine_native::BossStageKind;

    let e = boot_huanren(29510);
    let stage = e.state_core().boss_stage().expect("stage");
    assert_eq!(stage.kind, BossStageKind::ShidianYanluo);
    assert_eq!(stage.level, 0);
    assert!(
        !stage.banned_generals.is_empty(),
        "禁将表应写入目录中存在的名字"
    );
    assert!(
        stage
            .banned_generals
            .contains(&general_named_first(&e, "陆逊")),
        "陆逊应在禁将表"
    );
    assert_eq!(
        e.state_core().players[1].general_slots[0],
        Some(general_named_first(&e, "刘备")),
        "被禁非 Boss 座应换成目录中第一名刘备"
    );
    assert_ne!(general_slot_name(&e, 1), Some("映姬"));

    let mut spawned = false;
    let mut quiet = false;
    for seed in 1..81u64 {
        let e = boot_huanren(seed);
        let flag = e.state_core().boss_mengpo_spawned();
        let living = living_named(&e, "孟婆");
        if flag {
            spawned = true;
            assert_eq!(living, 1, "seed {seed} 乱入后应有一名存活孟婆");
            let seat = (0..e.state_core().players.len() as u8)
                .find(|&s| general_slot_name(&e, s) == Some("孟婆"))
                .unwrap();
            assert!(
                e.debug_hand(seat).is_empty(),
                "level 0 孟婆不应保留摸到的 4 张牌"
            );
        } else {
            quiet = true;
            assert_eq!(living, 0, "seed {seed} 未乱入时不应有孟婆");
        }
    }
    assert!(spawned, "1..80 应出现孟婆乱入（40% 门）");
    assert!(quiet, "1..80 应出现不乱入");
}

#[test]
fn d4_boss_b4_huanren_die_picks_successor_and_removes_mengpo() {
    let mut spawned_seed = None;
    for seed in 1..81u64 {
        let e = boot_huanren(seed);
        if e.state_core().boss_mengpo_spawned() {
            spawned_seed = Some(seed);
            break;
        }
    }
    let seed = spawned_seed.expect("应有乱入种子");
    let mut e = boot_huanren(seed);
    assert_eq!(living_named(&e, "孟婆"), 1);
    kill_boss(&mut e);
    assert!(e.debug_alive(0));
    assert_eq!(general_slot_name(&e, 0), Some("楚江王"), "候选首项");
    assert_eq!(e.state_core().boss_stage().unwrap().level, 1);
    assert!(has_named_skill(&e, 0, "还人"));
    assert!(has_named_skill(&e, 0, "还人·转关"), "换将后应重授转关");
    assert_eq!(living_named(&e, "孟婆"), 0, "已乱入后再转关应移出孟婆");
    assert!(e.state_core().boss_mengpo_spawned(), "乱入旗保持");
    assert_ne!(general_slot_name(&e, 0), Some("映姬"));

    kill_boss(&mut e);
    assert_eq!(living_named(&e, "孟婆"), 0, "第二次转关不再投放孟婆");
    assert!(e.state_core().boss_mengpo_spawned());
    assert_eq!(e.state_core().boss_stage().unwrap().level, 2);
    assert_ne!(general_slot_name(&e, 0), Some("映姬"));
    assert_zone_conservation(&e);
}

#[test]
fn d4_boss_b4_huanren_dizang_and_zhuanlun_not_yingji() {
    let mut e = Engine::new_boss_stage(29530, 4, 0, 0, 4, 4, 0);
    e.debug_set_general_def(0, boss_general_named(&e, "秦广王"));
    e.debug_set_boss_progress(2, true, true);
    assert_eq!(e.state_core().turn.phase_number, 0);
    kill_boss(&mut e);
    assert_eq!(general_slot_name(&e, 0), Some("地藏王"));
    assert_eq!(e.state_core().boss_stage().unwrap().level, 3);
    assert_eq!(e.debug_hand(0).len(), 7, "弃置区域后摸 4，地藏王再摸 3");
    assert_ne!(general_slot_name(&e, 0), Some("映姬"));
    assert!(e.debug_alive(0));

    let mut e = Engine::new_boss_stage(29531, 4, 0, 0, 4, 4, 0);
    e.debug_set_general_def(0, boss_general_named(&e, "秦广王"));
    e.debug_set_boss_progress(2, true, false);
    kill_boss(&mut e);
    assert_eq!(general_slot_name(&e, 0), Some("转轮王"));
    assert_eq!(e.state_core().boss_stage().unwrap().level, 3);
    assert_ne!(general_slot_name(&e, 0), Some("映姬"));
    assert_ne!(general_slot_name(&e, 0), Some("地藏王"));
}

#[test]
fn d4_boss_b4_qingyi_rounds_change_hp() {
    use engine_native::BossStageKind;

    let mut e = Engine::new_boss_stage(29540, 4, 0, 0, 4, 4, 0);
    e.debug_set_general_def(0, boss_general_named(&e, "太昊"));
    assert!(has_named_skill(&e, 0, "青翼"));
    let boss_hp = e.debug_hp(0);
    let foe_hp = e.debug_hp(1);
    assert!(boss_hp < e.state_core().players[0].max_hp as i32);
    end_turn_into_round(&mut e, 3);
    assert_eq!(e.debug_hp(0), boss_hp + 1, "第 3 轮友方回复 1");
    assert_eq!(e.debug_hp(1), foe_hp, "第 3 轮不改敌方体力");
    let stage = e.state_core().boss_stage().unwrap();
    assert_eq!(stage.kind, BossStageKind::WingGate);
    assert_eq!(stage.wing_armed & 1, 1, "青翼第 3 轮 bit0");

    let mut e = Engine::new_boss_stage(29541, 4, 0, 0, 4, 4, 0);
    e.debug_set_general_def(0, boss_general_named(&e, "太昊"));
    e.debug_set_hp(1, 3);
    let boss_hp = e.debug_hp(0);
    end_turn_into_round(&mut e, 5);
    assert_eq!(e.debug_hp(1), 2, "第 5 轮敌方失去 1 体力");
    assert_eq!(e.debug_hp(0), boss_hp, "第 5 轮不改友方体力");
    assert_eq!(e.state_core().boss_stage().unwrap().wing_armed & 2, 2);

    let mut e = Engine::new_boss_stage(29542, 4, 0, 0, 4, 4, 0);
    e.debug_set_general_def(0, boss_general_named(&e, "太昊"));
    e.debug_set_general_def(2, boss_general_named(&e, "书精"));
    e.state_core_mut().player_mut(2).alive = false;
    e.state_core_mut().player_mut(2).hp = 0;
    let hand = e.debug_hand(2).len();
    let max_hp = e.state_core().players[2].max_hp;
    end_turn_into_round(&mut e, 7);
    assert!(e.debug_alive(2), "第 7 轮应复活书精");
    assert_eq!(e.debug_hp(2), 3);
    assert_eq!(e.state_core().players[2].max_hp, max_hp + 1);
    assert_eq!(e.debug_hand(2).len(), hand + 3);
    assert_eq!(e.state_core().boss_stage().unwrap().wing_armed & 4, 4);
}

#[test]
fn d4_boss_b4_chiyi_mark_then_real_damage() {
    use engine_native::BossStageKind;

    let mut e = Engine::new_boss_stage(29550, 4, 0, 0, 4, 4, 0);
    e.debug_set_general_def(0, boss_general_named(&e, "炎帝"));
    assert!(has_named_skill(&e, 0, "赤翼"));
    end_turn_into_round(&mut e, 3);
    assert!(has_named_skill(&e, 1, "赤仪2"), "第 3 轮敌方获得赤仪2");
    assert_eq!(
        e.state_core().boss_stage().unwrap().kind,
        BossStageKind::WingGate
    );
    let hp = e.debug_hp(1);
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    finish_cmd(&mut e, out);
    assert_eq!(e.debug_hp(1), hp - 2, "赤仪2 使 1 点伤害变为 2");
}

#[test]
fn d4_boss_b4_baiyi_discard_thunder_and_draw() {
    let mut e = Engine::new_boss_stage(29560, 4, 0, 0, 4, 4, 0);
    e.debug_set_general_def(0, boss_general_named(&e, "少昊"));
    assert!(has_named_skill(&e, 0, "白翼"));
    assert!(has_named_skill(&e, 0, "白翼·雷"));
    let hand = e.debug_hand(1).len();
    assert!(hand >= 2);
    end_turn_into_round(&mut e, 5);
    assert_eq!(e.debug_hand(1).len(), hand - 2, "第 5 轮敌方弃 2");
    assert_eq!(
        e.state_core().boss_stage().unwrap().wing_armed & (1 << 6),
        1 << 6
    );

    let mut e = Engine::new_boss_stage(29561, 4, 0, 0, 4, 4, 0);
    e.debug_set_general_def(0, boss_general_named(&e, "少昊"));
    e.debug_set_turn(0, Phase::Play, 6);
    let hp = e.debug_hp(0);
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Thunder);
    finish_cmd(&mut e, out);
    assert_eq!(e.debug_hp(0), hp, "第 7 轮前雷伤被防止");
    e.debug_set_turn(0, Phase::Play, 7);
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Thunder);
    finish_cmd(&mut e, out);
    assert_eq!(e.debug_hp(0), hp - 1, "第 7 轮雷伤生效");

    let mut e = Engine::new_boss_stage(29562, 4, 0, 0, 4, 4, 0);
    e.debug_set_general_def(0, boss_general_named(&e, "少昊"));
    // 神恩光环按「场上存在该技能」计数，不看 enabled。摘掉实例后
    // 白翼把基础 2 削成 1 才能单独看见。
    let shenen = e.catalog().skill_by_name("神恩").expect("神恩");
    e.state_core_mut().skills.remove(&(0, shenen));
    e.debug_set_turn(1, Phase::Judge, 1);
    let before = e.debug_hand(1).len();
    let out = next_phase(&mut e);
    finish_cmd(&mut e, out);
    assert_eq!(e.debug_hand(1).len(), before + 1, "第 3 轮前敌方摸牌 -1");
    e.debug_set_turn(2, Phase::Judge, 3);
    let before = e.debug_hand(2).len();
    let out = next_phase(&mut e);
    finish_cmd(&mut e, out);
    assert_eq!(e.debug_hand(2).len(), before + 2, "第 3 轮起不再削减");
}
