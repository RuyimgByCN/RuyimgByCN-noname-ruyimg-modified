//! NC10: a suspended duel must not reveal the other player's hidden hand to planning.
use engine_native::ai::BoundedPlanner;
use engine_native::{
    CardKind, ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, MachineSnapshot,
};

fn full_pack() -> engine_native::content::extension::ExtensionPack {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json");
    let raw = std::fs::read_to_string(path).unwrap().replace("\r\n", "\n");
    content_native_loader::bridge_pack(&raw).unwrap().pack
}

/// `fixture()` 下方的 18 步回放使用 seed=9002 在 198 张牌堆（标准 108 + swd 90）
/// Fisher-Yates 洗牌下的硬编码 `CardId`（`1, 102, 73, 169, 105, 167, 97, 34, 57, 192, 17`）。
/// 裁决 33 将 `cangchizhibi` 补入 `shahai-swd` 后正式包为 91 张（见下方
/// `full_91_card_swd_pack_with_cangchizhibi_preserves_duel_privacy` 专项验证）。
fn pack() -> engine_native::content::extension::ExtensionPack {
    let mut pack = full_pack();
    pack.cards.truncate(90);
    pack
}

fn fixture(pack: &engine_native::content::extension::ExtensionPack) -> Engine {
    let mut e = Engine::new_with_extensions(9002, 4, &[pack]).unwrap();
    use ChoiceOption::{Card, Pass};
    let actions = vec![
        (0, GameAction::NextPhase),
        (0, GameAction::NextPhase),
        (
            0,
            GameAction::PlayCard {
                card: 1,
                targets: vec![1],
            },
        ),
        (
            0,
            GameAction::PlayCard {
                card: 102,
                targets: vec![],
            },
        ),
        (
            3,
            GameAction::Answer {
                choice_id: 1,
                selection: vec![Pass],
            },
        ),
        (
            3,
            GameAction::Answer {
                choice_id: 2,
                selection: vec![Pass],
            },
        ),
        (
            3,
            GameAction::Answer {
                choice_id: 3,
                selection: vec![Card(73)],
            },
        ),
        (
            0,
            GameAction::PlayCard {
                card: 169,
                targets: vec![1],
            },
        ),
        (
            0,
            GameAction::PlayCard {
                card: 105,
                targets: vec![],
            },
        ),
        (0, GameAction::NextPhase),
        (
            0,
            GameAction::Answer {
                choice_id: 4,
                selection: vec![Card(167)],
            },
        ),
        (0, GameAction::NextPhase),
        (0, GameAction::NextPhase),
        (1, GameAction::NextPhase),
        (1, GameAction::NextPhase),
        (
            1,
            GameAction::PlayCard {
                card: 97,
                targets: vec![0],
            },
        ),
        (
            0,
            GameAction::Answer {
                choice_id: 5,
                selection: vec![Card(34)],
            },
        ),
        (
            1,
            GameAction::PlayCard {
                card: 57,
                targets: vec![0],
            },
        ),
    ];
    for (step, (actor, action)) in actions.into_iter().enumerate() {
        let out = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: step as u64 + 100,
            action,
        });
        assert!(
            !matches!(out, CommandOutcome::Rejected(_)),
            "step {step}: {out:?}"
        );
    }
    assert_eq!(e.open_choice().unwrap().spec.prompt, "duel_sha");
    assert_eq!(e.open_choice().unwrap().observer, 0);
    let res = e.resolution();
    assert_eq!(res.stack.len(), 1);
    assert_eq!(res.parks.len(), 1);
    match res.parks[0].machine.snapshot() {
        MachineSnapshot::NullifyWindow {
            card: 57,
            effect: Some(inner),
            ..
        } => {
            assert!(matches!(
                *inner,
                MachineSnapshot::Duel {
                    card: 57,
                    source: 1,
                    target: 0,
                    asked: true,
                    ..
                }
            ));
        }
        other => panic!("unexpected machine: {other:?}"),
    }
    assert!(e.state_core().table.contains(&57));
    e
}

fn alternate_hidden(mut e: Engine) -> Engine {
    assert_eq!(e.state_core().players[1].hand[0], 192);
    assert_eq!(e.state_core().deck[0], 17);
    assert_eq!(e.catalog().cards[192].kind, CardKind::Sifeizhenmian);
    assert_eq!(e.catalog().cards[17].kind, CardKind::Sha);
    let core = e.state_core_mut();
    core.players[1].hand[0] = 17;
    core.deck[0] = 192;
    e
}

#[test]
fn duel_planning_does_not_depend_on_opponent_hidden_card() {
    let pack = pack();
    let e = fixture(&pack);
    let other = alternate_hidden(e.clone());
    let left = e.ai_session(0).input();
    let right = other.ai_session(0).input();
    assert_eq!(left.view, right.view);
    assert_eq!(left.choice, right.choice);
    assert_eq!(left.own_hand_kinds, right.own_hand_kinds);
    assert_eq!(e.resolution().stack, other.resolution().stack);
    assert_eq!(
        e.resolution().parks[0].machine.snapshot(),
        other.resolution().parks[0].machine.snapshot()
    );
    for variant in 0..=2 {
        for budget in [0, 1, 48] {
            let planner = BoundedPlanner::new(budget)
                .with_policy_variant(variant)
                .with_lookahead(2);
            let first = e.plan_action(0, &planner);
            let second = other.plan_action(0, &planner);
            assert_eq!(first.action, second.action, "variant={variant} budget={budget}: hidden-card substitution changed the pending duel decision: {first:?} vs {second:?}");
            assert_eq!(first.nodes_used, second.nodes_used);
            assert_eq!(first.sim_faults, 0);
            assert_eq!(second.sim_faults, 0);
            let mut original = e.clone();
            let mut alternate = other.clone();
            assert!(!matches!(
                original.apply_plan(&first).unwrap(),
                CommandOutcome::Rejected(_)
            ));
            assert!(!matches!(
                alternate.apply_plan(&second).unwrap(),
                CommandOutcome::Rejected(_)
            ));
        }
    }
}

#[test]
fn redaction_preserves_duel_machine_and_referenced_cards() {
    let pack = pack();
    let e = fixture(&pack);
    let alternate = alternate_hidden(e.clone());
    let mut left = e.clone();
    let mut right = alternate.clone();
    engine_native::ai::redact_opponent_hands(&mut left, 0);
    engine_native::ai::redact_opponent_hands(&mut right, 0);
    assert_eq!(
        left.resolution().parks[0].machine.snapshot(),
        e.resolution().parks[0].machine.snapshot()
    );
    assert_eq!(left.resolution().stack, e.resolution().stack);
    assert_eq!(left.open_choice(), e.open_choice());
    assert_eq!(left.state_core().table, e.state_core().table);
    assert_eq!(
        left.state_core().players[0].hand,
        e.state_core().players[0].hand
    );
    assert_eq!(
        left.state_core().players[1].hand,
        right.state_core().players[1].hand
    );
    assert_eq!(left.state_core().deck, right.state_core().deck);
    Engine::restore_with_extensions(&left.save(), &[&pack])
        .expect("redacted state must preserve all references");
}

#[test]
fn public_special_zones_are_preserved_during_redaction() {
    let pack = pack();
    let mut e = fixture(&pack);
    let card = e.state_core_mut().deck.pop().unwrap();
    e.state_core_mut().players[2].zhucheng.push(card);
    let original = e.state_core().players[1].hand.clone();
    let machine = e.resolution().parks[0].machine.snapshot();
    engine_native::ai::redact_opponent_hands(&mut e, 0);
    assert_ne!(e.state_core().players[1].hand, original);
    assert_eq!(e.state_core().players[2].zhucheng, vec![card]);
    assert_eq!(e.resolution().parks[0].machine.snapshot(), machine);
}

#[test]
#[ignore = "manual same-machine latency sampling"]
fn pending_duel_timing() {
    let pack = pack();
    let e = fixture(&pack);
    let planner = BoundedPlanner::new(48)
        .with_policy_variant(1)
        .with_lookahead(2);
    for _ in 0..32 {
        std::hint::black_box(e.plan_action(0, &planner));
    }
    let mut samples = Vec::new();
    for _ in 0..500 {
        let start = std::time::Instant::now();
        std::hint::black_box(e.plan_action(0, &planner));
        samples.push(start.elapsed().as_micros());
    }
    samples.sort_unstable();
    eprintln!(
        "samples={} p50={}us p95={}us p99={}us",
        samples.len(),
        samples[249],
        samples[474],
        samples[494]
    );
}

#[test]
fn full_91_card_swd_pack_with_cangchizhibi_preserves_duel_privacy() {
    let pack = full_pack();
    assert_eq!(pack.cards.len(), 91, "裁决 33 补齐苍螭之璧后 swd 正式包应为 91 张");
    let mut e = Engine::new_with_extensions(9002, 4, &[&pack]).unwrap();
    let juedou_id = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.kind == CardKind::Duel)
        .map(|(i, _)| i as u32)
        .unwrap();
    let sha_ids: Vec<u32> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.kind == CardKind::Sha)
        .take(2)
        .map(|(i, _)| i as u32)
        .collect();
    let cangchi_id = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.kind == CardKind::Cangchizhibi)
        .map(|(i, _)| i as u32)
        .unwrap();

    // 推进 0 号至出牌阶段（若准备阶段弹出选择窗口则取消/过）
    let mut dedup = 500u64;
    for _ in 0..10 {
        while let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.allow_cancel {
                Vec::new()
            } else if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else {
                c.spec.options.iter().take(1).copied().collect()
            };
            dedup += 1;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: dedup,
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            });
        }
        if e.turn().current == 0 && e.turn().phase == engine_native::Phase::Play {
            break;
        }
        dedup += 1;
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: dedup,
            action: GameAction::NextPhase,
        });
    }

    // 保持 199 张全集分区不变量：先把 0/1 号手牌退回牌堆，再精确放置 [juedou_id, cangchi_id]、[sha_ids[0]] 与牌堆顶 sha_ids[1]
    {
        let core = e.state_core_mut();
        for p in 0..4 {
            let hand = std::mem::take(&mut core.players[p].hand);
            core.deck.extend(hand);
        }
        for id in [juedou_id, cangchi_id, sha_ids[0], sha_ids[1]] {
            core.deck.retain(|c| *c != id);
            core.discard.retain(|c| *c != id);
        }
        core.players[0].hand = vec![juedou_id, cangchi_id];
        core.players[1].hand = vec![sha_ids[0]];
        core.deck.insert(0, sha_ids[1]);
    }

    dedup += 1;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: dedup,
        action: GameAction::PlayCard {
            card: juedou_id,
            targets: vec![1],
        },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "play juedou: {out:?}");
    assert_eq!(e.open_choice().unwrap().spec.prompt, "duel_sha");
    assert_eq!(e.open_choice().unwrap().observer, 1);

    // 替换对手（0 号）暗手牌：cangchizhibi <-> sha_ids[1]
    let mut other = e.clone();
    assert_eq!(other.state_core().players[0].hand, vec![cangchi_id]);
    assert_eq!(other.state_core().deck[0], sha_ids[1]);
    other.state_core_mut().players[0].hand[0] = sha_ids[1];
    other.state_core_mut().deck[0] = cangchi_id;

    let planner = BoundedPlanner::new(24).with_policy_variant(1).with_lookahead(2);
    let first = e.plan_action(1, &planner);
    let second = other.plan_action(1, &planner);
    assert_eq!(first.action, second.action);
    assert_eq!(first.sim_faults, 0);
    assert_eq!(second.sim_faults, 0);

    let mut redacted = e.clone();
    engine_native::ai::redact_opponent_hands(&mut redacted, 1);
    Engine::restore_with_extensions(&redacted.save(), &[&pack])
        .expect("91 张完整 swd 包下脱敏存档恢复须保持全部引用合法");
}

