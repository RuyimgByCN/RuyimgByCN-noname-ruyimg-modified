//! 第一百七十二批：offline「仁望」（pe_jun_liubei jun_renwang）——
//! 「交牌回自己族」首原语 give_selected_then_recover_self（ABI 679）。
//! 上游：出牌阶段限一次，将至少两张手牌/装备区牌交给一名其他角色，然后你回复
//! 1 点体力；本回合出【杀】次数上限 +发动次数（effect 段本轮登记债省略）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const JUN_RENWANG: u32 = 3_013_147_713;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(start, CommandResult::Accepted { .. }),
        "start rejected: {start:?}"
    );
    let state = engine.state().clone();
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    engine_core::effect::run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JUN_RENWANG),
        }],
    )
    .unwrap();
    // owner 先受 1 伤，使回复可观察。
    let hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp - 1;
    (tx.commit().state, owner, other)
}

fn inject_two_cards(tx: &mut Transaction, owner: PlayerId) -> (CardId, CardId) {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let c1 = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_101),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 6,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    let c2 = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_102),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank: 4,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(c1);
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(c2);
    (c1, c2)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: u32,
    cards: Vec<CardId>,
    target: PlayerId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let skill_id = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def == SkillDefId::new(skill))
        .unwrap()
        .id;
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![target],
                card_targets: cards,
            },
        },
    )
}

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

#[test]
fn package_publishes_jun_renwang_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == JUN_RENWANG));
    let active = package
        .active_index
        .iter()
        .find(|a| a.skill_def_id == JUN_RENWANG)
        .expect("jun_renwang must publish an active entry");
    let selection = active.selection.as_ref().expect("selection block");
    assert_eq!(selection.zone, "hand_equip");
    assert_eq!((selection.min, selection.max), (2, 255));
}

#[test]
fn jun_renwang_gives_two_cards_then_self_recovers() {
    let (state, owner, other) = started(2_026_091_701);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let (c1, c2) = inject_two_cards(&mut tx, owner);
    let state = tx.commit().state;

    let hp_before = hp_of(&state, owner);
    let owner_hand = hand_count(&state, owner);
    let other_hand = hand_count(&state, other);

    let (state, result) = activate(state, owner, JUN_RENWANG, vec![c1, c2], other, 1);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        result.result
    );

    assert_eq!(hp_of(&state, owner), hp_before + 1, "chooser recovers one");
    assert_eq!(
        hand_count(&state, owner),
        owner_hand - 2,
        "gift leaves hand"
    );
    assert_eq!(
        hand_count(&state, other),
        other_hand + 2,
        "target receives gift"
    );
}

#[test]
fn jun_renwang_rejects_single_card_selection() {
    let (state, owner, other) = started(2_026_091_702);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let (c1, _c2) = inject_two_cards(&mut tx, owner);
    let state = tx.commit().state;

    let (_state, result) = activate(state, owner, JUN_RENWANG, vec![c1], other, 1);
    assert!(
        !matches!(result.result, CommandResult::Accepted { .. }),
        "single-card selection must be rejected (min 2)"
    );
}
