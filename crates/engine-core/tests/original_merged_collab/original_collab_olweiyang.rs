//! Golden coverage for COLLAB「未羊」and the complete ol_weiyang closure.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLWEIYANG: u32 = 4_199_352_428;
const OL_WEIYANG: u32 = 2_376_598_041;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/collab/collab.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, OLWEIYANG as u64, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLWEIYANG),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = *tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .last()
        .unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, first, second, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
    cards: Vec<engine_api::CardId>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: cards,
            },
        },
    )
}

#[test]
fn package_publishes_abi109_matched_selection_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 694);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLWEIYANG && skill.name == "olweiyang"));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == OLWEIYANG)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (1, 3));
    assert!(targets.allow_self);
    assert!(targets.wounded);
    assert!(targets.match_selected_card_count);
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(
        (selection.zone.as_str(), selection.min, selection.max),
        ("hand_equip", 1, 3)
    );
    assert!(selection.distinct_categories);
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_WEIYANG
            && character.skills == vec![OLWEIYANG]
            && character.max_hp == 3
            && character.sex == "female"
            && character.group == "qun"
    }));
}

#[test]
fn two_wounded_targets_require_two_distinct_categories_then_both_recover() {
    let (mut state, owner, first, _second, skill) = setup();
    state.players.get_mut(owner).unwrap().hp = 2;
    state.players.get_mut(first).unwrap().hp = 2;
    let cards = state.hand_cards(owner)[..2].to_vec();
    state.cards.get_mut(cards[0]).unwrap().kind = CardKind::Sha;
    state.cards.get_mut(cards[1]).unwrap().kind = CardKind::TrickWuzhong;

    let (waiting, activated) = activate(state, owner, skill, vec![owner, first], Vec::new(), 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.expect("two-card cost choice");
    assert_eq!((choice.min_count, choice.max_count), (2, 2));
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == cards[0].raw()));
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == cards[1].raw()));

    let revision = waiting.revision;
    let (state, resolved) = apply_command(
        waiting,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: cards.iter().map(|card| card.raw()).collect(),
                cancelled: false,
            },
        },
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert_eq!(state.players.get(first).unwrap().hp, 3);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(cards
        .iter()
        .all(|card| state.cards.get(*card).unwrap().zone == discard));
}

#[test]
fn duplicate_categories_healthy_targets_and_mismatched_counts_are_atomic_rejections() {
    let (mut state, owner, first, second, skill) = setup();
    state.players.get_mut(owner).unwrap().hp = 2;
    state.players.get_mut(first).unwrap().hp = 2;
    state.players.get_mut(second).unwrap().hp = 2;
    let cards = state.hand_cards(owner)[..3].to_vec();
    state.cards.get_mut(cards[0]).unwrap().kind = CardKind::Sha;
    state.cards.get_mut(cards[1]).unwrap().kind = CardKind::Tao;
    state.cards.get_mut(cards[2]).unwrap().kind = CardKind::Weapon;

    let before = canonical_state_hash(&state);
    let (mut state, result) = activate(
        state,
        owner,
        skill,
        vec![owner, first],
        cards[..2].to_vec(),
        10,
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    let first_max_hp = state.players.get(first).unwrap().max_hp as i32;
    state.players.get_mut(first).unwrap().hp = first_max_hp;
    let before = canonical_state_hash(&state);
    let (mut state, result) = activate(
        state,
        owner,
        skill,
        vec![owner, first],
        vec![cards[0], cards[2]],
        11,
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    state.players.get_mut(first).unwrap().hp = 2;
    let before = canonical_state_hash(&state);
    let (state, result) = activate(
        state,
        owner,
        skill,
        vec![owner, first, second],
        vec![cards[0], cards[2]],
        12,
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
}
