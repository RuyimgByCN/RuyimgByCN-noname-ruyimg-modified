//! Golden coverage for SP「牺牲」and KEY 零 rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::query::dying_save_card_for;
use engine_core::state::{CardKind, ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const XISHENG: u32 = 4_243_619_960;
const XIANDENG: u32 = 2_439_633_691;
const SHULV: u32 = 2_586_462_603;
const KEY_REI: u32 = 3_471_626_821;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp/xisheng.skill");
const SP: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");
const KEY: &[u8] = include_bytes!("../../../../content/original/ported/key/key.npack.json");

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("xisheng-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 289);
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_729_32,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let attacker = engine.state().turn.current;
    let owner = players
        .into_iter()
        .find(|player| *player != attacker)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: attacker,
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(XISHENG),
            },
        ],
    )
    .unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    for card in tx.state().hand_cards(owner) {
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
    }
    for card in tx.state().hand_cards(attacker) {
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
    }
    let attack = tx.state().hand_cards(attacker)[0];
    tx.state_mut().cards.get_mut(attack).unwrap().kind = CardKind::Sha;
    (tx.commit().state, owner, attacker)
}

#[test]
fn packages_publish_exact_cross_package_character_contract() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    let key = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    let view = sp
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == XISHENG)
        .unwrap();
    assert_eq!(
        (
            view.as_kind.as_str(),
            view.window.as_str(),
            view.filter.as_str(),
            view.source_zone.as_deref(),
            view.uses_per_turn,
            view.min_cards,
            view.max_cards,
        ),
        (
            "tao",
            "response",
            "owner_outside_turn",
            Some("hand_equip_expansion"),
            Some(1),
            2,
            2,
        )
    );
    assert!(key
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "sp"));
    assert_eq!(
        key.character_index
            .iter()
            .find(|character| character.id == KEY_REI)
            .unwrap()
            .skills,
        [XIANDENG, SHULV, XISHENG]
    );
}

#[test]
fn exactly_two_cards_save_once_only_outside_the_owners_turn() {
    let (state, owner, attacker) = setup();
    let sources = state.hand_cards(owner);
    assert!(sources.len() >= 2);
    assert_eq!(dying_save_card_for(&state, owner, owner), None);
    let attack = state.hand_cards(attacker)[0];
    let (state, save) = submit(
        state,
        attacker,
        CommandKind::PlayCard {
            card: attack,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        2,
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DyingSave
    );
    let source_tokens: std::collections::BTreeSet<_> =
        sources.iter().map(|card| card.raw()).collect();
    let options: Vec<_> = save
        .choice
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    let conversion = options
        .iter()
        .copied()
        .find(|token| !matches!(*token, 0 | 1) && !source_tokens.contains(token))
        .unwrap_or_else(|| panic!("missing conversion: options={options:?} sources={sources:?}"));
    let (state, choice) = resolve(state, owner, vec![conversion], 3);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::MultiCardViewAsResponse
    );
    let choice = choice.choice.unwrap();
    assert_eq!((choice.min_count, choice.max_count), (2, 2));
    let selected: Vec<_> = sources.iter().take(2).map(|card| card.raw()).collect();
    let (mut state, result) = resolve(state, owner, selected, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 1);
    assert!(!state.players.get(owner).unwrap().dying);

    state.players.get_mut(owner).unwrap().hp = 0;
    state.players.get_mut(owner).unwrap().dying = true;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RequestDyingSave {
            player: owner,
            source: attacker,
        }],
    )
    .unwrap();
    assert!(!tx.state().players.get(owner).unwrap().alive);

    let (mut state, owner, attacker) = setup();
    state.turn.current = owner;
    state.players.get_mut(owner).unwrap().hp = 0;
    state.players.get_mut(owner).unwrap().dying = true;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RequestDyingSave {
            player: owner,
            source: attacker,
        }],
    )
    .unwrap();
    assert!(!tx.state().players.get(owner).unwrap().alive);
}
