//! Golden coverage for BINGSHI 势杨弘「间计」「远谟」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBJIANJI: u32 = 3_226_006_705;
const MBYUANMO: u32 = 4_112_615_395;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(skill_def: u32, player_count: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(u64::from(skill_def)),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: u64::from(skill_def),
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let others = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, others)
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let cards = tx.state().hand_cards(player);
    if !cards.is_empty() {
        run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards,
                to: discard,
                from: Some(hand),
            }],
        )
        .unwrap();
    }
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    rank: u8,
    serial: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(509_000 + serial),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: if kind == CardKind::Weapon { 3 } else { 0 },
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, selections: Vec<u32>, cancelled: bool, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_complete_yanghong_at_abi509() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    let character = package
        .character_index
        .iter()
        .find(|character| character.name_key == "mb_yanghong")
        .unwrap();
    assert_eq!(character.skills, [MBJIANJI, MBYUANMO]);
}

#[test]
fn jianji_shared_sha_compares_once_then_damages_its_chooser() {
    let (mut tx, owner, others) = setup(MBJIANJI, 3);
    let first = others[0];
    let second = others[1];
    for player in [owner, first, second] {
        clear_hand(&mut tx, player);
    }
    let shared = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 8, 1);
    let losing = inject(&mut tx, second, ZoneKind::Hand, CardKind::Shan, 5, 2);
    let first_hp = tx.state().players.get(first).unwrap().hp;
    let second_hp = tx.state().players.get(second).unwrap().hp;
    let skill = skill(tx.state(), owner, MBJIANJI);
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![first, second],
                card_targets: vec![shared],
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let state = resolve(state, vec![shared.raw()], false, 2);
    let state = resolve(state, vec![losing.raw()], false, 3);
    assert_eq!(state.players.get(first).unwrap().hp, first_hp - 1);
    assert_eq!(state.players.get(second).unwrap().hp, second_hp - 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(shared).unwrap().zone, discard);
}

#[test]
fn jianji_both_participants_can_share_the_same_physical_card() {
    let (mut tx, owner, others) = setup(MBJIANJI, 3);
    let first = others[0];
    let second = others[1];
    for player in [owner, first, second] {
        clear_hand(&mut tx, player);
    }
    let shared = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 9, 20);
    let first_hp = tx.state().players.get(first).unwrap().hp;
    let second_hp = tx.state().players.get(second).unwrap().hp;
    let skill = skill(tx.state(), owner, MBJIANJI);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbjianji {
                player: owner,
                source: first,
                target: second,
                stage: 0,
                card: shared,
                cards: Vec::new(),
                targets: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![shared.raw()], false, 20);
    let state = resolve(state, vec![shared.raw()], false, 21);
    assert_eq!(state.players.get(first).unwrap().hp, first_hp - 1);
    assert_eq!(state.players.get(second).unwrap().hp, second_hp - 1);
}

#[test]
fn yuanmo_draws_for_attack_range_population_lost_by_moving_weapon() {
    let (mut tx, owner, others) = setup(MBYUANMO, 4);
    let source = others[0];
    let destination = others[1];
    let weapon = inject(&mut tx, source, ZoneKind::Equip, CardKind::Weapon, 7, 3);
    let before = tx.state().hand_count(source);
    let skill = skill(tx.state(), owner, MBYUANMO);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbyuanmo {
                player: owner,
                target: PlayerId::NONE,
                stage: 0,
                card: CardId::NONE,
                value: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![weapon.raw()], false, 3);
    let state = resolve(state, vec![destination.raw()], false, 4);
    assert!(state.resolution.pending_choice_id != 0);
    let state = resolve(state, vec![1], false, 5);
    assert_eq!(state.hand_count(source), before + 1);
    let equip = state.zones.find(ZoneKind::Equip, destination).unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, equip);
}
