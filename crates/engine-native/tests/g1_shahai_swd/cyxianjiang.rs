use super::*;

/// T12 swd 武将面批 89（车芸仙匠）：partial 转正（cyqiaoxie/shengong defer）。
#[test]
fn swd_generals_b89_cyxianjiang() {
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
    assert!(
        pack.generals.len() >= 78,
        "78 将入册（现 {}）",
        pack.generals.len()
    );
    let mut e = Engine::new_with_extensions(62056, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // 布阵：p0 持仙匠；p1 装备一把武器（p0 未持有）
    e.debug_grant(0, "cyxianjiang");
    let weapon = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c).equip_slot()
                    == Some(engine_native::catalog::EquipSlot::Weapon)
                    && e.catalog().card_kind(*c) != CardKind::Zhangba
            })
            .expect("武器牌");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
            pl.equips.retain(|x| *x != id);
        }
        core.players[1].equips.push(id);
        id
    };
    let kind_w = e.catalog().card_kind(weapon);
    // 另发现一把同 kind 的实体供复制（放入牌堆）
    let kinds: Vec<CardKind> = e.catalog().cards.iter().map(|c| c.kind).collect();
    {
        let core = e.state_core_mut();
        let found = core.deck.iter().position(|c| kinds[*c as usize] == kind_w);
        if found.is_none() {
            if let Some(pos) = core
                .discard
                .iter()
                .position(|c| kinds[*c as usize] == kind_w)
            {
                let c = core.discard.remove(pos);
                core.deck.push(c);
            }
        }
    }
    // p0 对 p1 出一张杀（唯一目标触发仙匠）
    e.debug_set_hp(1, 8);
    let sha = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "p0 杀 p1 {out:?}"
    );
    // 推进至仙匠确认窗（observer=0）
    let mut guard = 0;
    let confirm = loop {
        guard += 1;
        assert!(guard < 32, "仙匠确认窗未出现");
        let step = match e.open_choice() {
            None => None,
            Some(ch) => Some((
                ch.id,
                ch.observer,
                ch.spec.prompt.to_string(),
                ch.spec.min,
                ch.spec.options.clone(),
            )),
        };
        let Some((id, observer, prompt, min, options)) = step else {
            panic!("无挂起——仙匠窗缺失");
        };
        if prompt == "trigger_confirm" && observer == 0 {
            break (id, observer);
        }
        let sel: Vec<ChoiceOption> = if min == 0
            || prompt == "sha_ask"
            || prompt == "shan_ask"
            || prompt == "nullify_window"
        {
            vec![ChoiceOption::Pass]
        } else {
            options.iter().take(min.max(1)).copied().collect()
        };
        submit(
            &mut e,
            observer,
            GameAction::Answer {
                choice_id: id,
                selection: sel,
            },
        );
    };
    let (cid, obs) = confirm;
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("仙匠选牌窗");
    assert_eq!(c.spec.prompt, "cyxianjiang_pick");
    assert!(
        c.spec
            .options
            .iter()
            .any(|o| matches!(o, ChoiceOption::Card(x) if *x == weapon)),
        "仙匠：p1 武器在候选"
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(weapon)],
        },
    );
    settle_optional_confirm(&mut e);
    pass_nullify(&mut e);
    // 复制装备入场：p0 装备区出现同牌种武器（且不是 p1 那把本体）
    let p0_has = e.state_core().players[0]
        .equips
        .iter()
        .any(|c| e.catalog().card_kind(*c) == kind_w && *c != weapon);
    assert!(p0_has, "仙匠：p0 复制获得同牌种武器");
    assert!(
        e.state_core().players[1].equips.contains(&weapon),
        "仙匠：本体仍在 p1"
    );
}
