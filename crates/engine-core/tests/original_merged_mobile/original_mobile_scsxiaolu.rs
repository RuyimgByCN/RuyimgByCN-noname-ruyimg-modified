//! Golden coverage for MOBILE 韩悝组件「宵赂」rewritten from legacy JavaScript.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SCSXIAOLU: u32 = 3_320_236_272;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
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
            skill_def: SkillDefId::new(SCSXIAOLU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SCSXIAOLU)
        .unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        other,
        skill,
    }
}

fn activate(setup: Setup, key: u64) -> (GameState, PipelineResult, u32, u32) {
    let owner_hand = setup.state.hand_count(setup.owner);
    let other_hand = setup.state.hand_count(setup.other);
    let revision = setup.state.revision;
    let (state, result) = apply_command(
        setup.state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    (state, result, owner_hand, other_hand)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    choice_id: ChoiceId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
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
                cancelled: false,
            },
        },
    )
}

fn option_tokens(result: &PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect()
}

#[test]
fn package_contains_active_without_publishing_hidden_hankui() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert!(package.manifest.capabilities.request_choice);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == SCSXIAOLU && skill.name == "scsxiaolu"));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SCSXIAOLU)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.targets.is_none());
    // 十常侍 roster (08-04 round): hidden forms publish as minimal legal
    // characters; scs_hankui carries scsxiaolu.
    assert!(package.character_index.iter().any(
        |character| character.name_key == "scs_hankui" && character.skills.contains(&SCSXIAOLU)
    ));
}

#[test]
fn give_branch_draws_then_gives_exactly_two_current_hand_cards() {
    let setup = setup(2_026_072_401);
    let owner = setup.owner;
    let other = setup.other;
    let skill = setup.skill;
    let (state, activated, owner_before, other_before) = activate(setup, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(option_tokens(&activated), [1, 2]);
    assert_eq!(state.hand_count(owner), owner_before + 2);

    let branch = activated.choice.unwrap();
    let (state, target_choice) = resolve(state, owner, branch.choice_id, vec![1], 2);
    assert!(matches!(
        target_choice.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(option_tokens(&target_choice), [other.raw()]);

    let target = target_choice.choice.unwrap();
    let (state, card_choice) = resolve(state, owner, target.choice_id, vec![other.raw()], 3);
    assert!(matches!(card_choice.result, CommandResult::Accepted { .. }));
    let cards = option_tokens(&card_choice);
    assert!(cards.len() >= 2);

    let card_choice = card_choice.choice.unwrap();
    let (state, given) = resolve(state, owner, card_choice.choice_id, cards[..2].to_vec(), 4);
    assert!(matches!(given.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_before);
    assert_eq!(state.hand_count(other), other_before + 2);

    let before = canonical_state_hash(&state);
    let revision = state.revision;
    let (same, second) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(5),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(second.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}

#[test]
fn discard_branch_discards_exactly_two_after_the_draw() {
    let setup = setup(2_026_072_402);
    let owner = setup.owner;
    let (state, activated, owner_before, _) = activate(setup, 10);
    let branch = activated.choice.unwrap();
    let (state, card_choice) = resolve(state, owner, branch.choice_id, vec![2], 11);
    assert!(matches!(card_choice.result, CommandResult::Accepted { .. }));
    let cards = option_tokens(&card_choice);
    assert!(cards.len() >= 2);

    let choice = card_choice.choice.unwrap();
    let (state, discarded) = resolve(state, owner, choice.choice_id, cards[..2].to_vec(), 12);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_before);
}

#[test]
fn insufficient_hand_fallback_discards_every_current_discardable_hand_card() {
    let setup = setup(2_026_072_403);
    let keep = setup.state.hand_cards(setup.owner)[0];
    let discard: Vec<CardId> = setup
        .state
        .hand_cards(setup.owner)
        .into_iter()
        .filter(|card| *card != keep)
        .collect();
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: setup.owner,
            cards: discard,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_cards(setup.owner), [keep]);

    run_effects(
        &mut tx,
        vec![EffectRequest::DrawThenChooseGiveOrDiscardHand {
            player: setup.owner,
            draw_count: 0,
            payment_count: 2,
            skill: setup.skill,
            ready_to_choose: true,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(setup.owner), 0);
    assert!(!tx.state().has_pending_choice());
}
