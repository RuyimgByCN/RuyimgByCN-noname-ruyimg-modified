//! Goldens for HUICUI「从势」与四象「姝勇」的旧 JS 语义重写。

use engine_api::{
    CardDefId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DCCONGSHI: u32 = 3_668_194_449;
const STDSHUYONG: u32 = 4_071_785_718;
const HUICUI: &[u8] = include_bytes!("../../../content/original/ported/huicui/huicui.npack.json");
const SIXIANG: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup(package: &[u8], skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let user = engine
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        },
        EffectRequest::SetTurnState {
            current: user,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, user)
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    def: u32,
    kind: CardKind,
    suit: Suit,
) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn play(
    state: engine_core::state::GameState,
    actor: PlayerId,
    card: engine_api::CardId,
    key: u64,
) -> engine_core::state::GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn packages_expose_the_rewritten_trigger_contracts() {
    let huicui = content_schema::ContentPackage::from_json_bytes(HUICUI).unwrap();
    assert!(huicui.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == DCCONGSHI
            && trigger.event == "card_used"
            && trigger.filter == "owner_alive_equip_source_max_equip"
            && !trigger.optional
    }));
    let sixiang = content_schema::ContentPackage::from_json_bytes(SIXIANG).unwrap();
    assert!(sixiang.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDSHUYONG
            && trigger.event == "card_used"
            && trigger.filter == "owner_alive_other_current_source_repeats_card_name"
            && trigger.optional
    }));
}

#[test]
fn congshi_observes_equipment_after_it_enters_the_users_equip_zone() {
    let (mut tx, owner, user) = setup(HUICUI, DCCONGSHI);
    let initial_hand = tx.state().hand_count(owner);
    let weapon = inject(&mut tx, user, 98_001, CardKind::Weapon, Suit::Spade);
    let state = play(tx.state().clone(), user, weapon, 1);

    let equip = state.zones.find(ZoneKind::Equip, user).unwrap();
    assert!(state.zones.get(equip).unwrap().cards.contains(&weapon));
    assert_eq!(state.hand_count(owner), initial_hand + 1);
}

#[test]
fn shuyong_treats_normal_and_elemental_sha_as_the_same_card_name() {
    let (mut tx, owner, user) = setup(SIXIANG, STDSHUYONG);
    let initial_hand = tx.state().hand_count(owner);
    let normal = inject(&mut tx, user, 98_101, CardKind::Sha, Suit::Spade);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: user,
            card: normal,
            targets: vec![],
        }],
    )
    .unwrap();
    let fire = inject(&mut tx, user, 98_102, CardKind::ShaFire, Suit::Heart);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: user,
                card: fire,
                targets: vec![],
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );

    let state = tx.state().clone();
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), initial_hand + 1);
}

#[test]
fn shuyong_does_not_merge_different_equipment_names() {
    let (mut tx, owner, user) = setup(SIXIANG, STDSHUYONG);
    let initial_hand = tx.state().hand_count(owner);
    let first = inject(&mut tx, user, 98_201, CardKind::Weapon, Suit::Spade);
    let state = play(tx.state().clone(), user, first, 3);
    let mut tx = Transaction::begin(state);
    let second = inject(&mut tx, user, 98_202, CardKind::Weapon, Suit::Club);
    let state = play(tx.state().clone(), user, second, 4);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), initial_hand);
}
