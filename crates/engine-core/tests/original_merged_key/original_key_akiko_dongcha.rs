//! Golden for KEY「洞察」rewritten from the legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, ObserverKind, PlayerId,
    ReplayPrivacy, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::projection::{project_observer, project_player};
use engine_core::query::{observer_foreign_hand_faces_visible, observer_hidden_identities_visible};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DONGCHA: u32 = 2_799_956_315;
const AKIKO: u32 = 3_018_824_106;
const SOURCE: &str = include_str!("../../../../content/original/ported/key/akiko_dongcha.skill");
const KEY: &[u8] = include_bytes!("../../../../content/original/ported/key/key.npack.json");

fn compiled_package() -> content_schema::ContentPackage {
    content_compiler::compile_source_named("key-akiko-dongcha", SOURCE).unwrap()
}

fn foreign_hand_faces_are_visible(
    state: &engine_core::state::GameState,
    view: &engine_api::ObserverSnapshot,
    observer: PlayerId,
) -> bool {
    state
        .players
        .iter_live()
        .filter(|player| player.id != observer)
        .flat_map(|player| state.hand_cards(player.id))
        .all(|card| {
            view.known_cards
                .iter()
                .any(|known| known.id == card && known.def.is_some())
        })
}

#[test]
fn package_declares_typed_observer_visibility_and_character() {
    let compiled = compiled_package();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 20);
    assert_eq!(compiled.modifier_index.len(), 2);
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == DONGCHA
            && modifier.target == "observer_foreign_hand_faces_visible"
            && modifier.mode == "override"
            && modifier.value == 1
    }));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == DONGCHA
            && modifier.target == "observer_hidden_identities_visible"
            && modifier.mode == "override"
            && modifier.value == 1
    }));

    let committed = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 691);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == DONGCHA && skill.locked));
    let character = committed
        .character_index
        .iter()
        .find(|character| character.id == AKIKO)
        .unwrap();
    assert_eq!(character.max_hp, 3);
    assert_eq!(
        if character.initial_hp == 0 {
            character.max_hp
        } else {
            character.initial_hp
        },
        3
    );
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "key");
    assert_eq!(character.skills, vec![DONGCHA]);
}

#[test]
fn only_the_skill_owner_and_private_replay_receive_hidden_information() {
    let package = compiled_package();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_718),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 20_260_718,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let observer = PlayerId::new(2);
    let other = PlayerId::new(3);
    let still_hidden = PlayerId::new(4);
    let before = project_player(engine.state(), observer);
    assert!(!foreign_hand_faces_are_visible(
        engine.state(),
        &before,
        observer
    ));
    assert_eq!(
        before
            .seats
            .iter()
            .find(|seat| seat.player == still_hidden)
            .unwrap()
            .identity,
        None
    );

    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: observer,
        skill_def: SkillDefId::new(DONGCHA),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;

    assert!(observer_foreign_hand_faces_visible(&state, observer));
    assert!(observer_hidden_identities_visible(&state, observer));
    assert!(!observer_foreign_hand_faces_visible(&state, other));
    assert!(!observer_hidden_identities_visible(&state, other));

    let owner_view = project_player(&state, observer);
    assert!(foreign_hand_faces_are_visible(
        &state,
        &owner_view,
        observer
    ));
    assert!(owner_view.seats.iter().all(|seat| seat.identity.is_some()));

    let private_replay = project_observer(
        &state,
        ObserverKind::ReplayExporter {
            privacy: ReplayPrivacy::Player(observer),
        },
    );
    assert!(foreign_hand_faces_are_visible(
        &state,
        &private_replay,
        observer
    ));
    assert!(private_replay
        .seats
        .iter()
        .all(|seat| seat.identity.is_some()));

    let other_view = project_player(&state, other);
    assert!(!foreign_hand_faces_are_visible(&state, &other_view, other));
    assert_eq!(
        other_view
            .seats
            .iter()
            .find(|seat| seat.player == still_hidden)
            .unwrap()
            .identity,
        None
    );

    let spectator = project_observer(&state, ObserverKind::Spectator);
    assert!(state
        .players
        .iter_live()
        .flat_map(|player| state.hand_cards(player.id))
        .all(|card| !spectator
            .known_cards
            .iter()
            .any(|known| known.id == card && known.def.is_some())));
    assert_eq!(
        spectator
            .seats
            .iter()
            .find(|seat| seat.player == still_hidden)
            .unwrap()
            .identity,
        None
    );

    let referee = project_observer(&state, ObserverKind::Referee);
    assert!(state
        .players
        .iter_live()
        .flat_map(|player| state.hand_cards(player.id))
        .all(|card| referee
            .known_cards
            .iter()
            .any(|known| known.id == card && known.def.is_some())));
    assert!(referee.seats.iter().all(|seat| seat.identity.is_some()));
}
