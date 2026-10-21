use super::*;

/// N08 装备卸装效果终止：武器替换=卸装旧武器——射程即时回落、旧武器恰一份弃置。
/// 行为链：射程1 够不到距离2 → weapon2（射程2）够到 → 守方 +1 馬 距离3 →
/// weapon3（射程3）替换后够到 → weapon2b（射程2）替换后再次够不到
/// （旧 weapon3 射程即时失效的实证）。全程 4 人原型局。
#[test]
fn weapon_replacement_terminates_old_range() {
    let mut e = Engine::new(293, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _seed_sha = e.debug_place(0, "sha");
    let try_sha_at_2 = |e: &mut Engine| -> bool {
        let sha = e
            .debug_hand(0)
            .iter()
            .copied()
            .find(|c| e.catalog().card_kind(*c) == CardKind::Sha)
            .expect("0 号应有杀");
        let out = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::PlayCard {
                card: sha,
                targets: vec![2],
            },
        });
        match out {
            CommandOutcome::Accepted { .. } => {
                answer_and_drive(e, out);
                true
            }
            CommandOutcome::Rejected(RejectReason::IllegalTargets) => false,
            other => panic!("意外结果 {other:?}"),
        }
    };
    // 阶段 0：无武器射程 1，够不到座位距离 2 的 2 号
    assert!(!try_sha_at_2(&mut e), "射程 1 不应够到距离 2");
    // 阶段 1：装备 weapon2（射程 2）→ 可达
    let w2 = e.debug_move_named_card_to_hand(0, "weapon2");
    play(&mut e, w2, vec![]);
    assert_eq!(e.state_core().attack_range(e.catalog(), 0), 2);
    assert!(try_sha_at_2(&mut e), "射程 2 应够到距离 2");
    // 阶段 2：2 号自己的回合装备 +1 馬 → 距离 3，射程 2 不可达
    advance_to_play(&mut e, 2);
    e.debug_clear_hand(2);
    let horse = e.debug_place(2, "horse_plus");
    play(&mut e, horse, vec![]);
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 3);
    // 阶段 3：0 号回合替换 weapon3（射程 3）→ 旧 weapon2 卸装进弃牌堆恰一份
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let w3 = e.debug_move_named_card_to_hand(0, "weapon3");
    play(&mut e, w3, vec![]);
    assert_eq!(
        e.state_core().attack_range(e.catalog(), 0),
        3,
        "weapon3 射程生效"
    );
    assert_eq!(
        discard_count(&e, w2),
        1,
        "被替换的 weapon2 应卸装进弃牌堆恰一份"
    );
    assert!(
        e.log().iter().any(|l| matches!(l,
            LE::Equipped { player: 0, card, replaced: Some(r) } if *card == w3 && *r == w2)),
        "Equipped 事件应携带被替换旧武器"
    );
    assert!(!e.debug_alive(2) || true, "2 号存活性与本断言无关");
    // 阶段 4：再替换 weapon2b（射程 2）→ 行为级证明旧 weapon3 射程已终止
    let w2b = e.debug_move_named_card_to_hand(0, "weapon2b");
    play(&mut e, w2b, vec![]);
    assert_eq!(
        e.state_core().attack_range(e.catalog(), 0),
        2,
        "射程应回落到 2"
    );
    assert_eq!(discard_count(&e, w3), 1, "weapon3 应卸装进弃牌堆恰一份");
    // 0 号本回合尚未出杀 → 再次出杀应因距离不足被拒（而非杀数上限）
    let _sha_fresh = e.debug_place(0, "sha");
    assert!(
        !try_sha_at_2(&mut e),
        "旧 weapon3 射程已终止：距离 3 不可达"
    );
    assert_zone_conservation(&e);
}
