//! Golden coverage for REFRESH 界郭嘉's rewritten 遗计.

use engine_api::{
    CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NEW_REYIJI: u32 = 3_740_567_047;
const RE_GUOJIA: u32 = 3_968_003_419;
const SOURCE: &str = include_str!("../../../../content/original/ported/refresh/new_reyiji.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let package =
        content_compiler::compile_source_named("refresh-new-reyiji-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_726_203, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let other = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NEW_REYIJI),
    });
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        source,
        other,
    }
}

fn damage(state: GameState, source: PlayerId, target: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
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
fn package_publishes_abi232_complete_re_guojia() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    assert!(package.skill_index.len() >= 76);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_GUOJIA)
        .unwrap();
    assert_eq!(character.skills, vec![6, NEW_REYIJI]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NEW_REYIJI)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target_per_damage_point");
    assert!(trigger.optional);
}

#[test]
fn plans_current_hand_for_multiple_recipients_then_moves_all_at_finish() {
    let setup = setup();
    let owner_hand = setup.state.hand_cards(setup.owner);
    let original = owner_hand[0];
    let owner_before = setup.state.hand_count(setup.owner);
    let source_before = setup.state.hand_count(setup.source);
    let other_before = setup.state.hand_count(setup.other);

    let state = damage(setup.state, setup.source, setup.owner);
    let (state, confirmed) = resolve(state, setup.owner, vec![1], false, 1);
    assert_eq!(
        confirmed.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert!(tokens(&confirmed).contains(&original.raw()));
    assert_eq!(state.hand_count(setup.owner), owner_before + 2);

    let (state, first_cards) = resolve(state, setup.owner, vec![original.raw()], false, 2);
    assert_eq!(
        first_cards.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    let (state, first_target) = resolve(state, setup.owner, vec![setup.source.raw()], false, 3);
    assert_eq!(
        first_target.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let owner_hand_zone = state.zones.find(ZoneKind::Hand, setup.owner).unwrap();
    assert_eq!(state.cards.get(original).unwrap().zone, owner_hand_zone);

    let second = CardId::new(tokens(&first_target)[0]);
    let (state, second_cards) = resolve(state, setup.owner, vec![second.raw()], false, 4);
    assert_eq!(
        second_cards.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    let (state, completed) = resolve(state, setup.owner, vec![setup.other.raw()], false, 5);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), owner_before);
    assert_eq!(state.hand_count(setup.source), source_before + 1);
    assert_eq!(state.hand_count(setup.other), other_before + 1);
}

#[test]
fn distribution_may_be_declined_after_drawing() {
    let setup = setup();
    let owner_before = setup.state.hand_count(setup.owner);
    let source_before = setup.state.hand_count(setup.source);
    let state = damage(setup.state, setup.source, setup.owner);
    let (state, confirmed) = resolve(state, setup.owner, vec![1], false, 11);
    assert_eq!(
        confirmed.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let (state, declined) = resolve(state, setup.owner, Vec::new(), true, 12);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), owner_before + 2);
    assert_eq!(state.hand_count(setup.source), source_before);
}
