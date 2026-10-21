use super::*;

/// G1 三尖武器技（两段选择）：装备三尖 → 杀伤害 → 弃一张手牌 →
/// 对距受损者距离 ≤1 的另一名角色造成 1 点伤害；两段均字节恢复。
#[test]
fn sanjian_weapon_skill_two_stage_choice_with_restore() {
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
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(3503, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 装备三尖（equip_skills 生命周期授予 wd_sanjian）
    let sj = {
        let catalog = e.catalog();
        catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("wd_sanjian"))
            .expect("三尖在目录") as u32
    };
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != sj);
    snap.core.players[0].hand.push(sj);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sj,
            targets: vec![],
        },
    );
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    let sj_def = e.catalog().skill_by_name("wd_sanjian").unwrap();
    assert!(
        e.save().core.skills.contains_key(&(0, sj_def)),
        "装备三尖应授予 wd_sanjian"
    );
    // 杀 → 1 号（距离 1）受 1 伤 → 三尖触发：弃牌选择挂起
    let sha = e.debug_place(0, "sha");
    let discard_card = e.debug_place(0, "tao");
    e.debug_set_hp(1, 4);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    let c = e.open_choice().expect("三尖弃牌选择挂起");
    assert_eq!(c.spec.prompt, "wd_sanjian_discard");
    let cid = c.id;
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(discard_card)],
        },
    );
    // 第二段：目标选择（字节恢复后应答）
    let c2 = e.open_choice().expect("三尖目标选择挂起");
    assert_eq!(c2.spec.prompt, "wd_sanjian_target");
    let bytes = encode_archive(&e.save());
    let mut restored =
        Engine::restore_with_extensions(&decode_archive(&bytes).expect("decode"), &[&pack])
            .expect("三尖目标选择字节恢复");
    let cid2 = restored.open_choice().unwrap().id;
    submit(
        &mut restored,
        0,
        GameAction::Answer {
            choice_id: cid2,
            selection: vec![ChoiceOption::Player(2)],
        },
    );
    assert_eq!(
        restored.debug_hp(2),
        3,
        "三尖对距离 ≤1 的另一角色造成 1 点伤害"
    );
    assert_eq!(restored.debug_hp(1), 3, "原伤害不变");
}
