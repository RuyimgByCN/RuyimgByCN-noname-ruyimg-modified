//! Golden coverage for DIY old-border Liu Bei's rewritten 仁德.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JUNKRENDE: u32 = 4_276_125_264;
const JUNK_LIUBEI: u32 = 3_715_728_088;
const SOURCE: &str = include_str!("../../../../content/original/ported/diy/junkrende.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/diy/diy.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: PlayerId,
    second: PlayerId,
    skill: SkillId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("diy-junkrende-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_726_204, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let first = players[1];
    let second = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JUNKRENDE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.hp = 3;
    participant.max_hp = 4;
    let skill = participant.skills[0];
    Setup {
        state: tx.commit().state,
        owner,
        first,
        second,
        skill,
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn tokens(pipe: &PipelineResult) -> Vec<u32> {
    pipe.choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect()
}

#[test]
fn package_publishes_abi204_complete_junk_liubei() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 684);
    assert_eq!(package.skill_index.len(), 147);
    assert_eq!(package.character_index.len(), 66);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == JUNK_LIUBEI)
        .unwrap();
    assert!(character.is_lord);
    assert_eq!(character.skills, vec![JUNKRENDE, 2_377_283_906]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == JUNKRENDE)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
}

#[test]
fn gives_to_distinct_targets_and_recovers_once_at_two_cards() {
    let setup = setup();
    let hand = setup.state.hand_cards(setup.owner);
    let first_card = hand[0];
    let second_card = hand[1];
    let owner_before = setup.state.hand_count(setup.owner);
    let first_before = setup.state.hand_count(setup.first);
    let second_before = setup.state.hand_count(setup.second);

    let (state, activated) = submit(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.first],
            card_targets: vec![first_card],
        },
        1,
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(
        activated.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert!(!tokens(&activated).contains(&first_card.raw()));

    let (state, selected) = resolve(state, setup.owner, vec![second_card.raw()], false, 2);
    assert_eq!(
        selected.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    assert_eq!(tokens(&selected), vec![setup.second.raw()]);

    let (state, completed) = resolve(state, setup.owner, vec![setup.second.raw()], false, 3);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
    assert_eq!(state.hand_count(setup.owner), owner_before - 2);
    assert_eq!(state.hand_count(setup.first), first_before + 1);
    assert_eq!(state.hand_count(setup.second), second_before + 1);
}
