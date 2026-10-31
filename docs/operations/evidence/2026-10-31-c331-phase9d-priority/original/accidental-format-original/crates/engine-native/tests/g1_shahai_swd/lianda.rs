use super::*;

/// T12 swd 武将面批㉔（敷衍连打/荣霜隳延+断行/慕云破浪/陈靖仇诱引；移花
/// /竹海/击空 deferred）：五行为行为面 + 反例。
#[test]
fn swd_generals_b64_lianda_suiyan_duanxing_polang_youyin() {
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
    let mut e = Engine::new_with_extensions(62032, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 连打（敷衍）：杀结算后弃一张牌 → 虚拟杀对原目标再结算
    e.debug_grant(0, "lianda");
    let sha1 = e.debug_place(0, "sha");
    let disc1 = e.debug_place(0, "tao");
    e.debug_set_hp(1, 4);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    let c = e.open_choice().expect("连打确认窗");
    assert_eq!((c.spec.prompt, c.observer), ("trigger_confirm", 0));
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("连打弃牌窗");
    assert_eq!(c.spec.prompt, "lianda_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(disc1)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 2, "连打：原杀 1 伤 + 虚拟杀再结算 1 伤");
    assert!(e.state_core().turn.lianda_used, "连打：每回合旗置位");
    assert!(!e.debug_hand(0).contains(&disc1), "连打：代价牌已弃");
    // 反例：非杀类用牌不弹连打窗（filter sha-like 门）
    let wuzhong = e.debug_place(0, "wuzhong");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wuzhong,
            targets: vec![],
        },
    );
    assert!(e.open_choice().is_none(), "连打反例：非杀无窗");
    settle_optional_confirm(&mut e);
    assert!(e.state_core().turn.lianda_used, "连打反例：旗未变");

    // 推进新回合辅助（各段共用）
    fn next_p0_play(e: &mut Engine) {
        for _ in 0..3 {
            let c = e.turn().current;
            let _ = submit(e, c, GameAction::NextPhase);
        }
        for _ in 0..16 {
            let c = e.turn().current;
            if c == 0 && e.state_core().turn.phase == engine_native::event::Phase::Play {
                break;
            }
            let _ = submit(e, c, GameAction::NextPhase);
        }
        advance_to_play(e, 0);
    }

    // ② 隳延（荣霜）：造成伤害后弃一张牌 → 弃置对方全部装备（新回合）
    e.debug_grant(0, "suiyan");
    next_p0_play(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 布 p2 两张装备（目录扫描取装备实体）
    let eqs: Vec<u32> = {
        // 每槽位取一实体（Weapon/Armor/HorsePlus 优先），保证双装可并存
        let mut slots_seen: Vec<engine_native::catalog::EquipSlot> = Vec::new();
        let mut placed = Vec::new();
        for id in 0..e.catalog().cards.len() as u32 {
            if placed.len() >= 2 {
                break;
            }
            let Some(slot) = e.catalog().card_kind(id).equip_slot() else {
                continue;
            };
            // 坐骑改距会扰动杀距离布阵——排除
            if matches!(
                slot,
                engine_native::catalog::EquipSlot::HorsePlus
                    | engine_native::catalog::EquipSlot::HorseMinus
            ) {
                continue;
            }
            if slots_seen.contains(&slot) {
                continue;
            }
            let id = {
                // 实体可能已被发牌消耗——目录序与实体一致（0..N 即实体）
                let core = e.state_core_mut();
                core.deck.retain(|x| *x != id);
                core.discard.retain(|x| *x != id);
                core.table.retain(|x| *x != id);
                for pl in core.players.iter_mut() {
                    pl.hand.retain(|x| *x != id);
                    pl.equips.retain(|x| *x != id);
                }
                core.player_mut(1).equips.push(id);
                id
            };
            slots_seen.push(slot);
            placed.push(id);
        }
        placed
    };
    assert_eq!(eqs.len(), 2, "p1 双装备布阵");
    for p in 0..4 {
        if p != 1 {
            e.debug_clear_hand(p);
        }
    }
    let sha2 = e.debug_place(0, "sha");
    let disc2 = e.debug_place(0, "tao");
    e.debug_set_hp(1, 8);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    // 连打（①授予，旗已随回合清）与隳延双确认窗并存——首窗拒，次窗按
    // 形状分流：确认窗=隳延→确认；弃牌窗=连打弃牌（拒）后到隳延
    {
        let c = e.open_choice().expect("首确认窗");
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![],
            },
        );
    }
    // 首窗若是连打：拒后其无弃牌窗（取消=放弃）；次窗=隳延确认
    let c = e.open_choice().expect("隳延窗");
    if c.spec.prompt == "trigger_confirm" {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Confirm],
            },
        );
    } else if c.spec.prompt == "lianda_discard" {
        // 首窗实为隳延（已拒）→ 次窗=连打确认 → 拒 → 隳延已废？
        // 此形状不出现在确定性次序下——防御性拒之
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![],
            },
        );
        let c = e.open_choice().expect("隳延确认窗 2");
        assert_eq!(c.spec.prompt, "trigger_confirm");
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Confirm],
            },
        );
    }
    let c = e.open_choice().expect("隳延弃牌窗");
    assert_eq!(c.spec.prompt, "suiyan_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(disc2)],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(
        e.debug_equips(1).is_empty(),
        "隳延：p1 装备全弃（{:?}）",
        eqs
    );

    // ③ 断行（荣霜）：装备牌入装后 → 视为对距离内目标使用虚拟杀
    e.debug_grant(0, "duanxing");
    next_p0_play(&mut e);
    for p in 0..4 {
        if p != 2 {
            e.debug_clear_hand(p);
        }
    }
    e.debug_set_hp(1, 4);
    let eq3 = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c).equip_slot()
                    == Some(engine_native::catalog::EquipSlot::Armor)
            })
            .expect("防具在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
            pl.equips.retain(|x| *x != id);
        }
        core.players[0].hand.push(id);
        id
    };
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: eq3,
            targets: vec![],
        },
    );
    let c = e.open_choice().expect("断行确认窗");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("断行目标窗");
    assert_eq!(c.spec.prompt, "duanxing_target");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    // 虚拟杀自身 after 窗会触发连打（he=防具非空）——拒之防级联
    if let Some(c) = e.open_choice() {
        if c.spec.prompt == "trigger_confirm" {
            let (cid, obs) = (c.id, c.observer);
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![],
                },
            );
        }
    }
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "断行：虚拟杀 1 伤");

    // ④ 破浪（慕云）：造成伤害后弃置对方一张装备（自选，另一张保留）
    e.debug_grant(0, "polang");
    next_p0_play(&mut e);
    // p3 布一张武器（p1 持防具会引入八卦判定；p2 对座越距）
    let eq4 = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c).equip_slot()
                    == Some(engine_native::catalog::EquipSlot::Weapon)
            })
            .expect("武器在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
        }
        core.player_mut(3).equips.push(id);
        id
    };
    for p in 0..4 {
        if p != 3 {
            e.debug_clear_hand(p);
        }
    }
    let sha4 = e.debug_place(0, "sha");
    e.debug_set_hp(3, 8);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha4,
            targets: vec![3],
        },
    );
    // 连打→隳延→破浪三确认窗按触发序排队（同形 trigger_confirm）——
    // 拒前二（连打/隳延），确认第三（破浪）
    for i in 0..3 {
        if e.open_choice()
            .is_some_and(|c| c.spec.prompt == "trigger_order")
        {
            let def = e
                .catalog()
                .skill_by_name(["lianda", "suiyan", "polang"][i])
                .unwrap();
            let c = e.open_choice().unwrap().clone();
            assert!(c.spec.options.contains(&ChoiceOption::Skill(def)));
            submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Skill(def)],
                },
            );
        }
        let c = e.open_choice().expect("确认窗序列");
        assert_eq!(c.spec.prompt, "trigger_confirm", "第 {i} 窗");
        let (cid, obs) = (c.id, c.observer);
        let sel = if i < 2 {
            vec![]
        } else {
            vec![ChoiceOption::Confirm]
        };
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
    let c = e.open_choice().expect("破浪选装备窗");
    assert_eq!(c.spec.prompt, "polang_discard_equip");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(eq4)],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(!e.debug_equips(3).contains(&eq4), "破浪：选中装备被弃");

    // ⑤ 诱引（陈靖仇）：其他角色弃非基本牌且手牌<5 → 自动摸 1
    e.debug_grant(0, "youyin");
    next_p0_play(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // p0 手牌留 1（<5）；p1 布手牌：1 trick + 压血强制弃牌阶段弃 trick
    let _keep = e.debug_place(0, "tao");
    let trick = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                let k = e.catalog().card_kind(*c);
                !k.is_basic_kind() && k.equip_slot().is_none()
            })
            .expect("锦囊在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
        }
        core.players[1].hand.push(id);
        id
    };
    let extra1 = e.debug_place(1, "tao");
    let extra2 = e.debug_place(1, "tao");
    e.debug_set_hp(1, 2); // 弃牌阶段需弃至 2：手牌 3 张 → 弃 1
    let h0 = e.debug_hand(0).len();
    // 推进 p1 回合：出牌（pass）→ 弃牌阶段弃 1（选 trick）
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..24 {
        if let Some(c) = e.open_choice() {
            // p1 弃牌窗：优先选 trick，按 min 补足（不足则拒无效）
            if c.spec.prompt.contains("discard") {
                let mut sel: Vec<ChoiceOption> = c
                    .spec
                    .options
                    .iter()
                    .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == trick))
                    .cloned()
                    .collect();
                for o in &c.spec.options {
                    if sel.len() >= c.spec.min {
                        break;
                    }
                    if !sel.contains(o) {
                        sel.push(o.clone());
                    }
                }
                let (cid, obs) = (c.id, c.observer);
                let s = if sel.is_empty() {
                    vec![c.spec.options[0].clone()]
                } else {
                    sel
                };
                submit(
                    &mut e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: s,
                    },
                );
                continue;
            }
            let (cid, obs) = (c.id, c.observer);
            let s = if !c.spec.options.is_empty() {
                vec![c.spec.options[0].clone()]
            } else {
                vec![]
            };
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: s,
                },
            );
            continue;
        }
        let cur = e.turn().current;
        if cur == 2 {
            break;
        }
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    assert!(
        !e.debug_hand(1).contains(&trick) && !e.debug_hand(1).contains(&extra1),
        "诱引：p1 弃牌阶段已弃牌"
    );
    assert_eq!(e.debug_hand(0).len(), h0 + 1, "诱引：p0 自动摸 1");
    let _ = extra2;
}
