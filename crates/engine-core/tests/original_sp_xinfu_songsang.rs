//! Golden for SP「送丧」rewritten from legacy JavaScript.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_JSP_PANGTONG: u32 = 2_348_829_214;
const XINFU_SONGSANG: u32 = 2_272_181_771;
const XINFU_ZHANJI: u32 = 4_089_086_847;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: Vec<PlayerId>,
}

fn has_skill(state: &GameState, player: PlayerId, skill_def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == skill_def)
        })
}

fn setup(wounded: bool) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, XINFU_SONGSANG as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINFU_SONGSANG),
    });
    run_effects(&mut tx, effects).unwrap();
    if wounded {
        tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    }
    Setup {
        state: tx.commit().state,
        owner,
        others: players
            .into_iter()
            .filter(|player| *player != owner)
            .collect(),
    }
}

fn kill(state: GameState, player: PlayerId, killer: PlayerId) -> (GameState, bool) {
    let mut tx = Transaction::begin(state);
    let requested = run_effects(&mut tx, vec![EffectRequest::KillPlayer { player, killer }])
        == Err(EffectError::ChoiceRequested);
    (tx.commit().state, requested)
}

fn resolve(state: GameState, owner: PlayerId, cancelled: bool, key: u64) -> GameState {
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: if cancelled { Vec::new() } else { vec![1] },
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_exposes_optional_once_only_death_skill_without_incomplete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert_eq!(package.manifest.logic_hash, package.compute_logic_hash());
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINFU_SONGSANG)
        .unwrap();
    assert_eq!(trigger.event, "death");
    assert_eq!(trigger.filter, "owner_alive");
    assert!(trigger.optional);
    assert!(package.character_index.iter().any(|character| {
        character.id == RE_JSP_PANGTONG && character.skills == [2_723_622_416, XINFU_SONGSANG]
    }));
}

#[test]
fn cancellation_preserves_songsang_and_self_death_does_not_trigger() {
    let wounded = setup(true);
    let hp = wounded.state.players.get(wounded.owner).unwrap().hp;
    let max_hp = wounded.state.players.get(wounded.owner).unwrap().max_hp;
    let (state, requested) = kill(wounded.state, wounded.others[0], wounded.owner);
    assert!(requested);
    let state = resolve(state, wounded.owner, true, 1);
    assert!(has_skill(&state, wounded.owner, XINFU_SONGSANG));
    assert!(!has_skill(&state, wounded.owner, XINFU_ZHANJI));
    assert_eq!(state.players.get(wounded.owner).unwrap().hp, hp);
    assert_eq!(state.players.get(wounded.owner).unwrap().max_hp, max_hp);
    let (_, requested_again) = kill(state, wounded.others[1], wounded.owner);
    assert!(requested_again);

    let full = setup(false);
    let (state, requested) = kill(full.state, full.owner, full.others[0]);
    assert!(!requested);
    assert!(!state.has_pending_choice());
}

#[test]
fn accepted_songsang_recovers_if_wounded_otherwise_only_raises_max_hp() {
    let wounded = setup(true);
    let hp = wounded.state.players.get(wounded.owner).unwrap().hp;
    let max_hp = wounded.state.players.get(wounded.owner).unwrap().max_hp;
    let (state, requested) = kill(wounded.state, wounded.others[0], wounded.owner);
    assert!(requested);
    let state = resolve(state, wounded.owner, false, 2);
    assert_eq!(state.players.get(wounded.owner).unwrap().hp, hp + 1);
    assert_eq!(state.players.get(wounded.owner).unwrap().max_hp, max_hp);
    assert!(!has_skill(&state, wounded.owner, XINFU_SONGSANG));
    assert!(has_skill(&state, wounded.owner, XINFU_ZHANJI));

    let full = setup(false);
    let hp = full.state.players.get(full.owner).unwrap().hp;
    let max_hp = full.state.players.get(full.owner).unwrap().max_hp;
    let (state, requested) = kill(full.state, full.others[0], full.owner);
    assert!(requested);
    let state = resolve(state, full.owner, false, 3);
    assert_eq!(state.players.get(full.owner).unwrap().hp, hp);
    assert_eq!(state.players.get(full.owner).unwrap().max_hp, max_hp + 1);
    assert!(!has_skill(&state, full.owner, XINFU_SONGSANG));
    assert!(has_skill(&state, full.owner, XINFU_ZHANJI));
}
