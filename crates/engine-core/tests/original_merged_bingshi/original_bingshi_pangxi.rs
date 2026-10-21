//! Golden coverage for BINGSHI 势庞羲「蓄业」「狂想」.

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

const MBXUYE: u32 = 4_127_729_199;
const MBKUANGXIANG: u32 = 3_758_195_005;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup() -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(514),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 514,
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
    effects.extend([MBXUYE, MBKUANGXIANG].map(|def| EffectRequest::AddSkill {
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
    let from = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let to = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to,
            from: Some(from),
        }],
    )
    .unwrap();
}

fn inject(tx: &mut Transaction, owner: PlayerId, zone_kind: ZoneKind, serial: u32) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(514_000 + serial),
        kind: CardKind::Shan,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, selection: u32) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_pangxi_and_hand_lte_target_contract_at_abi514() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pangxi")
            .unwrap()
            .skills,
        [MBXUYE, MBKUANGXIANG]
    );
    assert!(
        package
            .active_index
            .iter()
            .find(|active| active.skill_def_id == MBKUANGXIANG)
            .unwrap()
            .targets
            .as_ref()
            .unwrap()
            .hand_lte_owner
    );
}

#[test]
fn xuye_draws_then_places_target_hej_card_on_draw_pile_top() {
    let (mut tx, owner, order) = setup();
    let target = order.iter().copied().find(|role| *role != owner).unwrap();
    for role in order.iter().copied() {
        clear_hand(&mut tx, role);
    }
    for role in order.iter().copied().filter(|role| *role != target) {
        inject(&mut tx, role, ZoneKind::Hand, 10 + role.raw());
    }
    let field = inject(&mut tx, target, ZoneKind::Equip, 1);
    let xuye = skill(tx.state(), owner, MBXUYE);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbxuye {
                player: owner,
                target,
                stage: 0,
                was_max: false,
                skill: xuye,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, field.raw());
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.zones.get(draw).unwrap().cards.last(), Some(&field));
}

#[test]
fn kuangxiang_swaps_whole_hands_and_each_received_set_fires_once() {
    let (mut tx, owner, order) = setup();
    let target = order.into_iter().find(|role| *role != owner).unwrap();
    let third = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|role| *role != owner && *role != target)
        .unwrap();
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    clear_hand(&mut tx, third);
    for serial in 20..25 {
        inject(&mut tx, third, ZoneKind::Hand, serial);
    }
    let owner_card = inject(&mut tx, owner, ZoneKind::Hand, 2);
    let target_card = inject(&mut tx, target, ZoneKind::Hand, 3);
    let kuangxiang = skill(tx.state(), owner, MBKUANGXIANG);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbkuangxiang {
            player: owner,
            target,
            stage: 0,
            skill: kuangxiang,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_cards(owner), [target_card]);
    assert_eq!(tx.state().hand_cards(target), [owner_card]);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let before = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![owner_card],
                to: discard,
                from: Some(target_hand),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, 1);
    assert_eq!(state.hand_count(owner), before + 2);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbkuangxiang {
            player: owner,
            target,
            stage: 1,
            skill: kuangxiang,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 2);
}
