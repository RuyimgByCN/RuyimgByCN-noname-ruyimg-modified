//! Golden coverage for old「归心」rewritten from legacy JavaScript semantics.

use content_schema::PackageCharacterEntry;
use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::hash::canonical_state_hash;
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JUNKGUIXIN: u32 = 4_267_439_357;
const LORD_SKILL: u32 = 90_272;
const SOURCE: &str = include_str!("../../../../content/original/ported/old/junkguixin.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let mut package = content_compiler::compile_source_named(
        "junkguixin-test",
        &format!("{SOURCE}\nskill test_lord({LORD_SKILL}) locked lord_only {{}}"),
    )
    .unwrap();
    package.character_index.push(PackageCharacterEntry {
        id: 90_272,
        name_key: "unused_lord".into(),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![LORD_SKILL],
        sex: "male".into(),
        group: "wei".into(),
        is_lord: true,
        clans: vec![],
        tags: Vec::new(),
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: Vec::new(),
    });
    package.manifest.logic_hash = package.compute_logic_hash();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_930, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JUNKGUIXIN),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        tx.state_mut()
            .players
            .get_mut(*player)
            .unwrap()
            .character_def = 0;
    }
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveJunkguixin {
                player: owner,
                target: PlayerId::NONE,
                stage: 0,
                group: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    (tx.commit().state, owner, target)
}

fn choose(state: GameState, owner: PlayerId, token: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![token],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn gains_an_unused_lord_skill_or_changes_another_players_faction() {
    let (state, owner, _) = setup();
    let state = choose(state, owner, 1, 1);
    let state = choose(state, owner, LORD_SKILL, 2);
    assert!(state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == LORD_SKILL)
        }));

    let (state, owner, target) = setup();
    let before = canonical_state_hash(&state);
    let state = choose(state, owner, 2, 3);
    let state = choose(state, owner, target.raw(), 4);
    let state = choose(state, owner, 4, 5);
    assert_eq!(state.player_group(target), Some("qun"));
    assert_ne!(canonical_state_hash(&state), before);
}

#[test]
fn package_publishes_junkguixin_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JUNKGUIXIN && skill.name == "junkguixin"));
    assert!(package
        .character_index
        .iter()
        .any(|character| character.id == 2_847_507_392 && character.name_key == "old_caocao"));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
