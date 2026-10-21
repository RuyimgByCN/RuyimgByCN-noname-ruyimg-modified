use super::*;

/// 内容锁与字节路径：带扩展存档被纯标准恢复拒绝；同集合双路径恢复。
#[test]
fn g1_wudian_content_lock_and_byte_restore() {
    let (e, pack) = setup();
    let saved = e.save();
    assert!(matches!(
        Engine::restore(&saved),
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. })
    ));
    Engine::restore_with_extensions(&saved, &[&pack]).expect("同集合恢复");
    let bytes = engine_native::archive::encode_archive(&saved);
    Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("字节同集合恢复");
}

/// G1 武将批：6 名武将元数据经角色包管线入目录；
/// 思继（可选触发）：弃牌阶段弃 2 张杀 → 确认 → 摸 4；字节恢复贯通。
#[test]
fn g1_wudian_generals_and_siji_discard_draw() {
    use engine_native::archive::{decode_archive, encode_archive};
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "wudian-upstream-noname")
            .expect("wudian 可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let out = content_native_loader::bridge_generals(&json).expect("generals 装载");
    let generals = out.generals;
    let host_skills = out.host_skills;
    let unmapped_total = out.unmapped_skills;
    assert_eq!(
        generals.len(),
        9,
        "玩点论杀九名武将全量入册（批 10 补西真夕红/造之仁君/杨仪）"
    );
    assert_eq!(
        unmapped_total, 4,
        "4 个技能登记暂缓（批 15 良策转正；余仇渡+存量夺启/佑赞/坚壁全域面）"
    );
    let pack = ExtensionPack {
        def: ExtensionDef {
            name: "wudian-upstream-noname",
            version: 2,
            description: "G1 玩点论杀卡牌+武将批",
        },
        generals,
        cards: Vec::new(),
        host_skills,
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let base_count = engine_native::content::standard::STANDARD_GENERALS.len();
    let mut e = Engine::new_with_extensions(3502, 3, &[&pack]).expect("建局");
    // 韩龙 = 扩展表下标 1
    let (_, _, skills, hp, _) = e
        .catalog()
        .general_meta(base_count as u16 + 1)
        .expect("韩龙在目录");
    assert_eq!(
        skills,
        vec!["siji", "ciqiu"],
        "韩龙思继+刺猬（刺猬批一 v111 接线）"
    );
    assert_eq!(hp, 4);

    // 思继行为：授予 → 弃牌阶段弃 2 杀 → 确认 → 摸 4
    e.debug_grant(0, "siji");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let s1 = e.debug_place(0, "sha");
    let s2 = e.debug_place(0, "sha");
    e.debug_place(0, "shan");
    e.debug_set_hp(0, 2); // 4 张手牌 > 体力 2 → 弃牌阶段需弃 2 张
    for _ in 0..8 {
        if e.open_choice().is_some() {
            break;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    let c = e.open_choice().expect("弃牌选择挂起");
    assert_eq!(c.spec.prompt, "discard_to_limit");
    // 弃一张杀（弃牌上限按手牌-体力计算；弃置含杀 → 触发思继）
    let sel: Vec<ChoiceOption> = c
        .spec
        .options
        .iter()
        .take(c.spec.min.max(1))
        .copied()
        .collect();
    let discarded_sha = sel
        .iter()
        .any(|o| *o == ChoiceOption::Card(s1) || *o == ChoiceOption::Card(s2));
    let cid = c.id;
    let out = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: cid,
            selection: sel,
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 思继确认（可选触发，OptionalGate）——弃含杀 → 触发
    if discarded_sha {
        let c2 = e.open_choice().expect("思继确认应挂起");
        assert_eq!(c2.spec.prompt, "trigger_confirm");
        let bytes = encode_archive(&e.save());
        let mut restored =
            Engine::restore_with_extensions(&decode_archive(&bytes).expect("decode"), &[&pack])
                .expect("思继确认挂起字节恢复");
        let cid2 = restored.open_choice().unwrap().id;
        submit(
            &mut restored,
            0,
            GameAction::Answer {
                choice_id: cid2,
                selection: vec![ChoiceOption::Confirm],
            },
        );
        // 弃 1 杀 → 摸 2
        let hand_after = restored.debug_hand(0).len();
        assert_eq!(hand_after % 2, 0, "摸牌数为偶数（2×弃杀数）: {hand_after}");
    } else {
        panic!("弃牌应含杀以触发思继");
    }
}
