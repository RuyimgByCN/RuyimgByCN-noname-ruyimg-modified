//! Golden for SP 求援 xinfu_qiai (ABI 735, tag 756, Cont 328):
//! 限定技濒死交牌循环状态机 — owner 濒死时，其他角色依座位序（owner 之后
//! 环绕）各强制交给 owner 一张手/装备牌；无牌者跳过。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const XINFU_QIAI: u32 = 2_677_335_078;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64, count: u8) -> (GameState, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: count,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let players = state.players.alive_by_seat();
    (state, players)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, tag: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(92_700 + tag),
        kind: CardKind::Tao,
        suit: engine_api::Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn clear_hands(tx: &mut Transaction, players: &[PlayerId]) {
    for player in players {
        let cards = tx.state().hand_cards(*player);
        if !cards.is_empty() {
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: *player,
                    cards,
                }],
            )
            .unwrap();
        }
    }
}

fn arm_dying_owner(tx: &mut Transaction, owner: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINFU_QIAI),
        }],
    )
    .unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    tx.state_mut().players.get_mut(owner).unwrap().dying = true;
}

#[test]
fn package_publishes_xinfu_qiai_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let entry = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINFU_QIAI)
        .expect("xinfu_qiai trigger");
    assert_eq!(entry.event.as_str(), "dying");
    assert_eq!(entry.filter.as_str(), "owner_dying_zhupeilan_once_per_game");
    assert!(!entry.optional);
}

#[test]
fn dying_owner_collects_one_card_from_each_other_player_in_seat_order() {
    let (state, players) = started(2_026_091_001, 3);
    let owner = players[0];
    let mut tx = Transaction::begin(state);
    clear_hands(&mut tx, &players);
    arm_dying_owner(&mut tx, owner);
    let c1 = inject_hand(&mut tx, players[1], 1);
    let _c2 = inject_hand(&mut tx, players[1], 2);
    let c3 = inject_hand(&mut tx, players[2], 3);
    let owner_before = tx.state().hand_count(owner);

    // Entry：首 giver = players[1]（座位序 owner 之后）。
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveXinfuQiai { player: owner }],
    );
    assert!(matches!(effects, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ResolveXinfuQiaiStage
    );

    // giver1 交 c1 → 移入 owner 手牌区 + 打开 giver2 的选牌。
    let (state, result) = resolve(tx.commit().state, players[1], vec![c1.raw()], false, 7);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let tx = Transaction::begin(state);
    assert!(tx.state().hand_cards(owner).contains(&c1));
    assert!(!tx.state().hand_cards(players[1]).contains(&c1));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ResolveXinfuQiaiStage
    );
    assert_ne!(tx.state().resolution.pending_choice_id, 0);

    // giver2 交 c3 → 无更多 giver，机器收尾。
    let (state, result) = resolve(tx.commit().state, players[2], vec![c3.raw()], false, 8);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let state_tx = state.clone();
    assert!(state_tx.hand_cards(owner).contains(&c3));
    assert_eq!(state_tx.hand_count(owner), owner_before + 2);
    assert!(!state_tx.has_pending_choice());
}

#[test]
fn entry_without_any_qualified_giver_is_a_quiet_noop() {
    let (state, players) = started(2_026_091_002, 2);
    let owner = players[0];
    let mut tx = Transaction::begin(state);
    clear_hands(&mut tx, &players);
    arm_dying_owner(&mut tx, owner);

    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveXinfuQiai { player: owner }],
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!tx.state().has_pending_choice());
}
