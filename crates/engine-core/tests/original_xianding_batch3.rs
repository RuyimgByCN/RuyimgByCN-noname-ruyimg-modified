//! Golden for XIANDING（限定包）批次三：
//! 存嗣（dccunsi）、勇决（dcyongjue）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCCUNSI: u32 = 4_233_990_625;
const DCYONGJUE: u32 = 2_297_054_445;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCCUNSI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
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
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
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
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn package_exposes_batch3_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let cunsi = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCCUNSI)
        .expect("dccunsi active");
    assert_eq!(cunsi.window, "play_phase");
    assert!(cunsi.once_per_game);
    let targets = cunsi.targets.as_ref().expect("dccunsi targets");
    assert_eq!((targets.min, targets.max), (1, 1));
    assert!(targets.allow_self);

    let yongjue = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCYONGJUE)
        .expect("dcyongjue trigger");
    assert_eq!(yongjue.event, "card_used");
    assert_eq!(yongjue.filter, "owner_source_first_sha_this_play_phase");
    assert!(yongjue.optional);
}

#[test]
fn dccunsi_grants_yongjue_to_other_and_draws_two() {
    let (tx, owner, first, _second) = setup(20_260_805);
    let before = tx.state().hand_count(owner);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCCUNSI)
        .expect("dccunsi skill instance")
        .id;
    let (state, pipe) = activate(tx.state().clone(), owner, skill, vec![first], 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, first, DCYONGJUE));
    assert_eq!(state.hand_count(owner), before + 2);
}

#[test]
fn dccunsi_on_self_grants_yongjue_without_draw() {
    let (tx, owner, _first, _second) = setup(20_260_806);
    let before = tx.state().hand_count(owner);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCCUNSI)
        .expect("dccunsi skill instance")
        .id;
    let (state, pipe) = activate(tx.state().clone(), owner, skill, vec![owner], 2);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, DCYONGJUE));
    assert_eq!(state.hand_count(owner), before);
}

#[test]
fn dcyongjue_offers_refund_or_gain_choice_on_first_play_phase_sha() {
    let (mut tx, owner, first, _second) = setup(20_260_807);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCYONGJUE),
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);
    let err = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![first],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(
        matches!(err, Err(EffectError::ChoiceRequested)),
        "use err: {:?}",
        err
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 3);
    assert!(has_skill(&state, owner, DCYONGJUE), "yongjue should remain");
}
