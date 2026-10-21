//! Golden coverage for BINGSHI 势孙峻「凶图」「险帅」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, schedule_damage, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBXIONGTU: u32 = 2_558_604_057;
const MBXIANSHUAI: u32 = 3_336_756_615;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(u64::from(skill_def)),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: u64::from(skill_def),
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
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
    (tx, owner, target)
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

fn inject(tx: &mut Transaction, owner: PlayerId, suit: Suit) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(507_001 + suit as u32),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
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
fn xianshuai_only_unlimits_the_first_hand_card_of_each_suit() {
    let (mut tx, owner, _) = setup(MBXIANSHUAI);
    let first = inject(&mut tx, owner, Suit::Spade);
    let second = inject(&mut tx, owner, Suit::Spade);
    assert!(engine_core::query::card_use_unlimited_for_card(
        tx.state(),
        owner,
        first
    ));
    tx.state_mut()
        .turn
        .remember_hand_card_suit_used(owner, Suit::Spade);
    assert!(!engine_core::query::card_use_unlimited_for_card(
        tx.state(),
        owner,
        second
    ));
}

#[test]
fn xiongtu_damage_arms_only_after_itself_then_external_damage_enables_second_use() {
    let (mut tx, owner, target) = setup(MBXIONGTU);
    let skill = skill(tx.state(), owner, MBXIONGTU);
    tx.state_mut()
        .skills
        .get_mut(skill)
        .unwrap()
        .play_phase_use_count = 1;
    tx.state_mut()
        .turn
        .set_flag(turn_flag::DISCARD_PILE_SUIT_MASK, 0b1111);
    let revealed = inject(&mut tx, target, Suit::Heart);
    let hp = tx.state().players.get(target).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbxiongtu {
                player: owner,
                target,
                stage: 0,
                card: CardId::NONE,
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![revealed.raw()], false, 1);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    let flags = u32::from_le_bytes(
        state.skills.get(skill).unwrap().state_blob[4..8]
            .try_into()
            .unwrap(),
    );
    assert_eq!(flags, 0b001);

    let hand_before = state.hand_cards(owner).len();
    let mut tx = Transaction::begin(state);
    schedule_damage(&mut tx, owner, target, 1).unwrap();
    let flags = u32::from_le_bytes(
        tx.state().skills.get(skill).unwrap().state_blob[4..8]
            .try_into()
            .unwrap(),
    );
    assert_eq!(flags, 0b010);
    assert_eq!(tx.state().hand_cards(owner).len(), hand_before + 1);

    tx.state_mut()
        .skills
        .get_mut(skill)
        .unwrap()
        .play_phase_use_count = 2;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbxiongtu {
                player: owner,
                target,
                stage: 0,
                card: CardId::NONE,
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
}

#[test]
fn xiongtu_cancelled_cost_discards_the_revealed_hand_card_and_arms() {
    let (mut tx, owner, target) = setup(MBXIONGTU);
    let skill = skill(tx.state(), owner, MBXIONGTU);
    tx.state_mut()
        .skills
        .get_mut(skill)
        .unwrap()
        .play_phase_use_count = 1;
    let revealed = inject(&mut tx, target, Suit::Diamond);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbxiongtu {
                player: owner,
                target,
                stage: 0,
                card: CardId::NONE,
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![revealed.raw()], false, 2);
    let state = resolve(state, Vec::new(), true, 3);
    assert!(!state.hand_cards(target).contains(&revealed));
    assert_eq!(&state.skills.get(skill).unwrap().state_blob[..4], b"XTG1");
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbxiongtu {
            player: owner,
            target,
            stage: 6,
            card: CardId::NONE,
            cards: Vec::new(),
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        u32::from_le_bytes(
            tx.state().skills.get(skill).unwrap().state_blob[4..8]
                .try_into()
                .unwrap()
        ),
        0b001
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbxiongtu {
            player: owner,
            target,
            stage: 7,
            card: CardId::NONE,
            cards: Vec::new(),
            skill,
        }],
    )
    .unwrap();
    assert!(tx.state().skills.get(skill).unwrap().state_blob.is_empty());
}
