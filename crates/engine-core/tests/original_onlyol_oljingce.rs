//! Golden for ONLYOL 郭淮「精策」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLJINGCE: u32 = 3_351_317_413;
const OL_GUOHUAI: u32 = 2_919_059_025;
const REJINGCE: u32 = 2_158_663_152;
const SOURCE: &str = include_str!("../../../content/original/ported/onlyOL/oljingce.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    current: PlayerId,
    basic: CardId,
    equipment: CardId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind, suit: Suit) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(skill: u32, seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&yijiang).unwrap();
    if skill == OLJINGCE {
        std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    }
    let owner = tx.state().turn.current;
    let current = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    {
        let character = tx.state_mut().players.get_mut(owner).unwrap();
        character.character_def = OL_GUOHUAI;
        character.max_hp = 4;
        character.hp = 4;
    }
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|instance| EffectRequest::RemoveSkill { skill: instance.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill),
        },
        EffectRequest::SetTurnState {
            current,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let basic = inject(tx.state_mut(), owner, 293_001, CardKind::Jiu, Suit::Spade);
    let equipment = inject(
        tx.state_mut(),
        owner,
        293_002,
        CardKind::Weapon,
        Suit::Heart,
    );
    Setup {
        state: tx.commit().state,
        owner,
        current,
        basic,
        equipment,
    }
}

fn use_card(state: GameState, player: PlayerId, card: CardId) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player,
            card,
            targets: Vec::new(),
        }],
    )
    .unwrap();
    tx.commit().state
}

fn end_play(state: GameState, current: PlayerId) -> (Transaction, Result<(), EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current,
            phase: Phase::Discard.to_u8(),
        }],
    )
    .map(|_| ());
    (tx, result)
}

#[test]
fn package_publishes_complete_ol_guohuai_at_abi293() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 688);
    assert_eq!(package.skill_index.len(), 188);
    assert_eq!(package.character_index.len(), 83);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_GUOHUAI)
        .unwrap();
    assert_eq!(character.name_key, "ol_guohuai");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [OLJINGCE]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLJINGCE)
        .unwrap();
    assert_eq!(trigger.event, "phase_play_end");
    assert_eq!(
        trigger.filter,
        "owner_alive_play_phase_card_category_count_positive"
    );
    assert!(trigger.optional);
    let modifier = package
        .modifier_index
        .iter()
        .find(|modifier| modifier.skill_def_id == OLJINGCE)
        .unwrap();
    assert_eq!(modifier.target, "hand_limit");
    assert_eq!(
        modifier.value_source.as_deref(),
        Some("owner_distinct_hand_card_suits_used_this_turn")
    );
    let compiled = content_compiler::compile_source_named("oljingce", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == trigger.module)
        .unwrap();
    let expected = compiled
        .modules
        .iter()
        .find(|module| module.name == trigger.module)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn owner_use_during_another_players_play_phase_draws_distinct_categories() {
    let setup = setup(OLJINGCE, 2_026_073_001);
    let state = use_card(setup.state, setup.owner, setup.basic);
    let state = use_card(state, setup.owner, setup.equipment);
    assert_eq!(state.turn.play_card_category_count(setup.owner), 2);
    let hand_before = state.hand_count(setup.owner);

    let (tx, phase_end) = end_play(state, setup.current);
    assert_eq!(phase_end.unwrap_err(), EffectError::ChoiceRequested);
    assert!(tx.state().has_pending_choice());
    let (state, accepted) = apply_command(
        tx.state().clone(),
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(293_003),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), hand_before + 2);
}

#[test]
fn owner_without_card_history_is_not_prompted() {
    let setup = setup(OLJINGCE, 2_026_073_002);
    let (tx, phase_end) = end_play(setup.state, setup.current);
    phase_end.unwrap();
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn rejingce_remains_limited_to_the_owners_own_play_phase_end() {
    let setup = setup(REJINGCE, 2_026_073_003);
    let state = use_card(setup.state, setup.owner, setup.basic);
    assert_eq!(state.turn.play_card_category_count(setup.owner), 0);
    let (tx, phase_end) = end_play(state, setup.current);
    phase_end.unwrap();
    assert!(!tx.state().has_pending_choice());
}
