//! Golden coverage for 界甘宁「奋威」rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_GANNING: u32 = 2_643_035_493;
const QIXI: u32 = 27;
const FENWEI: u32 = 3_203_648_912;
const SOURCE: &str = include_str!("../../../../content/original/ported/refresh/fenwei.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    actor: PlayerId,
    lower: PlayerId,
    higher: PlayerId,
    owner: PlayerId,
    trick: CardId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = compile_source_named("refresh-fenwei-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (actor, lower, higher, owner) = (seats[0], seats[1], seats[2], seats[3]);
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FENWEI),
        },
        EffectRequest::SetTurnState {
            current: actor,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    inject(tx.state_mut(), lower, 99_301, CardKind::Other);
    for def in 99_302..99_305 {
        inject(tx.state_mut(), higher, def, CardKind::Other);
    }
    let trick = inject(tx.state_mut(), actor, 99_305, CardKind::TrickTiaojiyanmei);
    Setup {
        state: tx.commit().state,
        actor,
        lower,
        higher,
        owner,
        trick,
    }
}

fn command(
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

fn play(setup: Setup, key: u64) -> (Setup, PipelineResult) {
    let (state, output) = command(
        setup.state,
        setup.actor,
        CommandKind::PlayCard {
            card: setup.trick,
            targets: vec![setup.lower, setup.higher],
            card_targets: Vec::new(),
        },
        key,
    );
    (Setup { state, ..setup }, output)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
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

fn has_fenwei(state: &GameState, owner: PlayerId) -> bool {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == FENWEI)
        })
}

#[test]
fn package_closes_re_ganning_without_legacy_javascript() {
    let compiled = compile_source_named("refresh-fenwei", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 308);
    assert_eq!(
        rule_vm::bytecode::effect_tag::CHOOSE_CURRENT_TRICK_TARGETS_TO_NULLIFY,
        292
    );

    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_GANNING)
        .unwrap();
    assert_eq!(character.skills, [QIXI, FENWEI]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == FENWEI));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn selected_targets_are_nullified_and_the_limited_skill_is_consumed() {
    let (setup, played) = play(setup(20_260_730_308), 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let choice = played.choice.unwrap();
    assert_eq!(choice.observer, setup.owner);
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!((choice.min_count, choice.max_count), (1, 2));
    assert!(choice.allow_cancel);

    let (state, resolved) = resolve(setup.state, setup.owner, vec![setup.higher.raw()], false, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(resolved.choice.is_none());
    assert_eq!(state.hand_count(setup.lower), 2);
    assert_eq!(state.hand_count(setup.higher), 3);
    assert!(!has_fenwei(&state, setup.owner));
}

#[test]
fn cancellation_keeps_fenwei_and_allows_the_original_trick_to_resolve() {
    let (setup, played) = play(setup(20_260_730_309), 3);
    assert!(played.choice.is_some());
    let (state, resolved) = resolve(setup.state, setup.owner, Vec::new(), true, 4);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.lower), 2);
    assert!(has_fenwei(&state, setup.owner));
    assert_eq!(resolved.choice.unwrap().observer, setup.higher);
}
