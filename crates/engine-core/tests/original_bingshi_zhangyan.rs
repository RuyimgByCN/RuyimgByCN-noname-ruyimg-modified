//! Golden coverage for BINGSHI 势张燕「飞径」「骁戈」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBFEIJING: u32 = 2_916_762_074;
const MBXIAOGE: u32 = 3_431_894_905;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup() -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(510),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 510,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let order = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([MBFEIJING, MBXIAOGE].map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, order)
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
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
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

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(510_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, selections: Vec<u32>, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_zhangyan_and_damage_trick_view_as_at_abi510() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "mb_zhangyan")
            .unwrap()
            .skills,
        [MBFEIJING, MBXIAOGE]
    );
    assert_eq!(
        package
            .view_as_index
            .iter()
            .filter(|view| view.skill_def_id == MBFEIJING && view.filter == "damage_trick")
            .count(),
        2
    );
}

#[test]
fn feijing_discards_on_chosen_path_and_adds_matching_color_target() {
    let (mut tx, owner, order) = setup();
    for player in &order {
        clear_hand(&mut tx, *player);
    }
    let owner_index = order.iter().position(|player| *player == owner).unwrap();
    let middle = order[(owner_index + 1) % 4];
    let target = order[(owner_index + 2) % 4];
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        1,
    );
    let red = inject(
        &mut tx,
        middle,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        2,
    );
    let middle_hp = tx.state().players.get(middle).unwrap().hp;
    tx.state_mut().resolution.resume_kind = ContinuationKind::ResumePlayedSha;
    tx.state_mut().resolution.resume_payload = vec![owner.raw(), target.raw(), 1, 0, 1, sha.raw()];
    let feijing = skill(tx.state(), owner, MBFEIJING);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbfeijing {
                player: owner,
                target,
                card: sha,
                stage: 0,
                targets: Vec::new(),
                cards: Vec::new(),
                skill: feijing,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![1], 1);
    let state = choose(state, vec![red.raw()], 2);
    let state = choose(state, vec![1], 3);
    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(state.cards.get(red).unwrap().zone, owner_hand);
    assert_eq!(state.players.get(middle).unwrap().hp, middle_hp);
}

#[test]
fn xiaoge_prevents_bound_damage_recovers_and_reclaims_discard() {
    let (mut tx, owner, order) = setup();
    let target = order.into_iter().find(|player| *player != owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        3,
    );
    let selected = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        CardKind::Shan,
        Suit::Heart,
        4,
    );
    let feijing = skill(tx.state(), owner, MBFEIJING);
    let xiaoge = skill(tx.state(), owner, MBXIAOGE);
    let mut blob = b"FJG1".to_vec();
    blob.extend_from_slice(&sha.raw().to_le_bytes());
    blob.extend_from_slice(&1u32.to_le_bytes());
    blob.extend_from_slice(&target.raw().to_le_bytes());
    blob.extend_from_slice(&selected.raw().to_le_bytes());
    tx.state_mut().skills.get_mut(feijing).unwrap().state_blob = blob;
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    let hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut().resolution.current_damage_id = 77;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbxiaoge {
            player: owner,
            target,
            card: sha,
            stage: 0,
            targets: Vec::new(),
            skill: xiaoge,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp + 1);
    assert!(tx.state().resolution.prevented_damage_ids.contains(&77));
    assert_ne!(tx.state().cards.get(selected).unwrap().zone, discard);
    assert_eq!(tx.state().skills.get(feijing).unwrap().state_blob.len(), 12);
}

#[test]
fn xiaoge_follows_the_original_single_target_sha_with_duel() {
    let (mut tx, owner, order) = setup();
    let target = order
        .iter()
        .copied()
        .find(|player| *player != owner)
        .unwrap();
    for player in order {
        clear_hand(&mut tx, player);
    }
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        5,
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let xiaoge = skill(tx.state(), owner, MBXIAOGE);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbxiaoge {
            player: owner,
            target: owner,
            card: sha,
            stage: 1,
            targets: vec![target],
            skill: xiaoge,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, hp - 1);
}
