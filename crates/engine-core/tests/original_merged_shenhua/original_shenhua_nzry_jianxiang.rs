//! Golden coverage for 神话「荐降」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NZRY_JIANXIANG: u32 = 3_232_965_394;
const SOURCE: &str =
    include_str!("../../../../content/original/ported/shenhua/nzry_jianxiang.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    tied_min: PlayerId,
    other: PlayerId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("nzry-jianxiang-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_722_83, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let source = seats[1];
    let tied_min = seats[2];
    let other = seats[3];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NZRY_JIANXIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    inject(tx.state_mut(), source, 9_830_001);
    inject(tx.state_mut(), other, 9_830_002);
    inject(tx.state_mut(), other, 9_830_003);
    Setup {
        state: tx.commit().state,
        owner,
        source,
        tied_min,
        other,
    }
}

fn target_owner(mut setup: Setup) -> (GameState, PipelineResult) {
    let used = inject(&mut setup.state, setup.source, 9_830_004);
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: setup.source,
                card: used,
                targets: vec![setup.owner],
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.state().clone();
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state.clone(),
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    )
}

fn resolve_target(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![target.raw()],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_exposes_exact_optional_targeted_contract() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert!(committed.skill_index.iter().any(|skill| {
        skill.skill_def_id == NZRY_JIANXIANG && skill.name == "nzry_jianxiang" && !skill.locked
    }));
    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NZRY_JIANXIANG)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(trigger.filter, "owner_alive_targeted_by_other");
    assert!(trigger.optional);

    let compiled = content_compiler::compile_source_named("nzry_jianxiang", SOURCE).unwrap();
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn accepting_chooses_any_player_tied_for_minimum_hand_and_draws_one() {
    let setup = setup();
    let owner = setup.owner;
    let tied_min = setup.tied_min;
    let (state, accepted) = target_owner(setup);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let options: Vec<_> = accepted
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(options, vec![owner.raw(), tied_min.raw()]);

    let before = state.hand_count(tied_min);
    let (state, drawn) = resolve_target(state, owner, tied_min, 2);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(tied_min), before + 1);
}

#[test]
fn self_use_does_not_trigger_and_stale_minimum_target_is_rejected() {
    let mut self_use = setup();
    let card = inject(&mut self_use.state, self_use.owner, 9_830_005);
    let mut tx = Transaction::begin(self_use.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: self_use.owner,
            card,
            targets: vec![self_use.owner],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());

    let setup = setup();
    let owner = setup.owner;
    let tied_min = setup.tied_min;
    let other = setup.other;
    let (mut state, accepted) = target_owner(setup);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    inject(&mut state, tied_min, 9_830_006);
    assert!(state.hand_count(tied_min) > state.hand_count(owner));
    assert!(state.hand_count(other) > state.hand_count(owner));
    let (_, stale) = resolve_target(state, owner, tied_min, 3);
    assert!(matches!(stale.result, CommandResult::Rejected { .. }));
}
