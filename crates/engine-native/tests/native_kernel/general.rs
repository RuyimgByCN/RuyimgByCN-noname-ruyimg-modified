use super::*;

// ---------------------------------------------------------------------------
// N08 内容扩面：完整武将授予 / 动态技能资格 / 装备卸装效果终止（ADR 0724）
// ---------------------------------------------------------------------------

/// N08 完整武将授予：按武将名（样例·张角组合 = 鬼道 + 天妒）一次性授予全部技能，
/// 单人流完成改判→终定→天妒获得全链路，事件与守恒逐项核对。
#[test]
fn general_bundle_grant_full_skill_set() {
    let (mut e, _le) = build_guidao_pending();
    e.debug_grant_general(1, "样例·张角组合");
    assert!(e.debug_has_skill(1, "鬼道") && e.debug_has_skill(1, "天妒"));
    let black = place_black_sha(&mut e, 1);
    loop {
        if e.open_choice().is_some() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    // 第一挂起点：改判询问（自拥有者 1 起）——出黑牌改判
    assert_eq!(prompt_of(&e), "gui_dao_rewrite");
    let old_judge = judged_card_of(&e);
    let cid = e.open_choice().unwrap().id;
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(black)],
        },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    // 第二挂起点：天妒询问（武将另一技能）——确认获得
    assert_eq!(prompt_of(&e), "tian_du_gain");
    let cid = e.open_choice().unwrap().id;
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    // 事件与守恒
    assert!(log_has(&e, |l| matches!(l,
        LE::JudgementRewritten { player: 1, card, replaced } if *card == black && *replaced == old_judge)));
    assert!(log_has(&e, |l| matches!(l,
        LE::JudgementGained { player: 1, card } if *card == black)));
    assert_eq!(
        log_count(&e, |l| matches!(l, LE::JudgementFinalized { .. })),
        1,
        "Finalized 恰一次"
    );
    assert!(e.debug_hand(1).contains(&black), "天妒获得最终判定牌");
    assert_zone_conservation(&e);
}

/// N08 完整武将授予（扩表）：STANDARD_GENERALS 多武将——每名武将全部技能
/// 一次授予到位（张角=机制样例组合；样例条目=机制样例组合，ADR 0712/0768）。
#[test]
fn general_table_grants_all_bundles() {
    let catalog_general = |name: &str| -> Vec<&'static str> {
        engine_native::content::standard::STANDARD_GENERALS
            .iter()
            .find(|g| g.name == name)
            .map(|g| g.skills.to_vec())
            .unwrap_or_else(|| panic!("unknown general {name}"))
    };
    // 表完整性：武将名唯一、每个技能在目录中存在
    let mut names: Vec<&str> = engine_native::content::standard::STANDARD_GENERALS
        .iter()
        .map(|g| g.name)
        .collect();
    let total = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), total, "武将名不得重复");
    for g in engine_native::content::standard::STANDARD_GENERALS {
        for sk in g.skills.iter() {
            assert!(
                engine_native::Engine::new(1, 4)
                    .catalog()
                    .skill_by_name(sk)
                    .is_some(),
                "技能 {sk} 应在目录中"
            );
        }
    }
    // 授予行为：逐武将授予 → 全部技能 enabled
    for name in ["样例·张角组合", "样例·策援", "样例·守财"] {
        let mut e = Engine::new(1, 4);
        e.debug_grant_general(0, name);
        for sk in catalog_general(name) {
            assert!(e.debug_has_skill(0, sk), "{name} 授予后应含 {sk}");
        }
    }
}

// ---------------------------------------------------------------------------
// N11 国战 M2：Faction 势力模型（ADR 0755）
// ---------------------------------------------------------------------------

/// 国战 M2：Faction 势力枚举 + 武将→势力映射完整性——
/// 每名武将的势力为四势力之一，同名不重复，映射表完整。
#[test]
fn general_faction_mapping_complete() {
    use engine_native::catalog::Faction;
    let generals = engine_native::content::standard::STANDARD_GENERALS;
    // 表非空且每名武将势力合法
    assert!(!generals.is_empty());
    let mut names: Vec<&str> = Vec::new();
    for g in generals {
        let (name, faction, skills, hp) = (g.name, g.faction, g.skills, g.max_hp);
        // 上游华雄 6 体力（standard/character.js），其余 3–4。
        assert!(hp >= 3 && hp <= 6, "{name} 体力配置合法");
        assert!(
            matches!(
                faction,
                Faction::Wei | Faction::Shu | Faction::Wu | Faction::Qun
            ),
            "{name} 势力合法"
        );
        assert!(!skills.is_empty(), "{name} 至少一个技能");
        names.push(name);
        // 技能都在目录中
        let e = Engine::new(1, 4);
        for sk in skills.iter() {
            assert!(
                e.catalog().skill_by_name(sk).is_some(),
                "{name} 技能 {sk} 应在目录"
            );
        }
    }
    // 武将名唯一
    names.sort_unstable();
    let total = names.len();
    names.dedup();
    assert_eq!(names.len(), total, "武将名不得重复");
    // 抽查官方势力归属
    let faction_of = |name: &str| {
        generals
            .iter()
            .find(|def| def.name == name)
            .map(|def| def.faction)
            .expect("武将存在")
    };
    assert!(matches!(faction_of("曹操"), Faction::Wei));
    assert!(matches!(faction_of("关羽"), Faction::Shu));
    assert!(matches!(faction_of("孙权"), Faction::Wu));
    assert!(matches!(faction_of("吕布"), Faction::Qun));
}

/// P2/N08 装配授予原语：装配武将即按武将表授予技能集与体力上限
/// （与国战 RevealGeneral 共用 apply_general_assembly，授予语义不分叉）；
/// 替换装配为并集授予（不自动撤销旧将技能）；摘除只撤销当前槽武将的
/// 技能集并清槽；`debug_set_general` 保持仅配置槽位（势力/性别资格
/// 测试依赖「不授予」的既有语义）。
#[test]
fn general_assembly_grants_and_remove_revokes_slot_skills() {
    let mut e = Engine::new(55, 2);

    // set_general 仅装槽：不授予技能、不改体力（既有语义回归）。
    e.debug_set_general(0, "夏侯惇");
    assert!(!e.debug_has_skill(0, "刚烈"), "set_general 不授予技能");
    assert_eq!(
        e.save().core.players[0].max_hp,
        4,
        "set_general 不改体力上限"
    );

    // 装配授予：技能集 + 体力上限随武将表。
    e.debug_assemble_general(1, "曹仁");
    assert!(e.debug_has_skill(1, "据守"), "装配曹仁即授予据守");
    assert_eq!(e.save().core.players[1].max_hp, 4, "曹仁体力上限 4");

    // 替换装配：授予新技能集并改体力上限（并集语义，不撤销旧将技能）。
    e.debug_set_hp(1, 5);
    e.debug_assemble_general(1, "司马懿");
    assert!(
        e.debug_has_skill(1, "鬼才") && e.debug_has_skill(1, "反馈"),
        "替换装配授予新技能集"
    );
    assert_eq!(e.save().core.players[1].max_hp, 3, "替换装配改体力上限");
    assert_eq!(e.save().core.players[1].hp, 3, "装配把体力夹到新上限");
    assert!(
        e.debug_has_skill(1, "据守"),
        "替换装配不自动撤销旧将技能（并集）"
    );

    // 摘除：只撤销当前槽武将（司马懿）的技能集并清槽。
    e.debug_remove_general(1);
    assert!(
        !e.debug_has_skill(1, "鬼才") && !e.debug_has_skill(1, "反馈"),
        "摘除撤销当前槽技能实例"
    );
    assert!(e.debug_has_skill(1, "据守"), "旧将技能不受后续摘除影响");
    assert!(
        e.save().core.players[1].general_slots[0].is_none(),
        "摘除清空主将槽"
    );
    e.debug_remove_general(1); // 空槽摘除幂等
}
