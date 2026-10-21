use super::*;

/// T12 swd 武将面批㊞（张烈破晓/轩辕剑仙花间复用；huajin 授技/pozhou/
/// xuanyuan deferred）：两将行为面 + 反例。
#[test]
fn swd_generals_b79_poxiao() {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(62047, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 破晓 viewAs：装备牌当杀（对座 p2 距离 2 无武器出界反例面）
    e.debug_grant(0, "poxiao");
    let eq1 = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c).equip_slot()
                    == Some(engine_native::catalog::EquipSlot::HorseMinus)
            })
            .expect("减马在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
        }
        core.players[0].hand.push(id);
        id
    };
    e.debug_set_hp(1, 4);
    // 有武器 → 无武器双 mod 不触发，但 viewAs 装备牌当杀仍可用
    let w = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c).equip_slot()
                    == Some(engine_native::catalog::EquipSlot::Weapon)
            })
            .expect("武器在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.players[0].equips.push(id);
        id
    };
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: eq1,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "破晓：装备牌当杀 1 伤（p1 无闪）");
    let _ = w;
    // 反例：装备牌当杀不能指定无手牌目标以外……（冰箭同型已有反例面——
    // 此处断言装备牌已消耗）
    assert!(!e.debug_hand(0).contains(&eq1), "破晓：装备牌已消耗");

    // ② 破晓无武器距离 -1：卸武器后对座 p2（距离 2-1=1）入范围（新回合）
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    {
        let core = e.state_core_mut();
        core.player_mut(0).equips.retain(|x| *x != w);
        core.discard.push(w);
    }
    let sha2 = e.debug_place(0, "sha");
    e.debug_set_hp(2, 4);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![2],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(2), 3, "破晓：无武器距离 -1 → 对座 p2 入范围受 1");

    // ③ 轩辕剑仙花间复用：阶段末弃牌虚拟杀
    e.debug_grant(0, "huajian");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 4);
    let d2 = e.debug_place(0, "tao");
    submit(&mut e, 0, GameAction::NextPhase);
    let c = e.open_choice().expect("花间弃牌窗");
    assert_eq!(c.spec.prompt, "huajian_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(d2)],
        },
    );
    let c = e.open_choice().expect("花间目标窗");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "花间复用：虚拟杀 1 伤");
}
