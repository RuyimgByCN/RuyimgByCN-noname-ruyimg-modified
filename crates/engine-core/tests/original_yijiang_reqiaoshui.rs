//! Golden coverage for 界简雍「巧说」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{card_ignored_for_hand_limit, discard_to_limit_tokens};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REQIAOSHUI: u32 = 3_083_818_598;
const QIAOSHUI3: u32 = 2_260_792_673;
const QIAOSHUI4: u32 = 3_521_041_512;
const TARGET_BOOST: u32 = 1_990_000_067;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const TARGET_BOOST_SOURCE: &str = r#"
skill qiaoshui_test_target_boost(1990000067) {
  modifier attack_range override 99;
  modifier sha_target_limit add 2;
}
"#;

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: PlayerId,
    second: PlayerId,
    third: PlayerId,
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    def: u32,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, skill_defs: &[u32]) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let boost =
        content_compiler::compile_source_named("qiaoshui_test_boost", TARGET_BOOST_SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&boost).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
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
    effects.extend(skill_defs.iter().map(|skill_def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(*skill_def),
    }));
    effects.push(EffectRequest::SetTurnState {
        current: owner,
        phase: Phase::Play.to_u8(),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    Setup {
        state: tx.commit().state,
        owner,
        first: seats[1],
        second: seats[2],
        third: seats[3],
    }
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
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
                .is_some_and(|skill| skill.def.raw() == def)
        })
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

fn compare(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    owner_card: CardId,
    target_card: CardId,
    key: u64,
) -> GameState {
    let active = skill_id(&state, owner, REQIAOSHUI);
    let (state, activated) = command(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        key,
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let (state, owner_selected) = resolve(state, owner, vec![owner_card.raw()], false, key + 1);
    assert!(matches!(
        owner_selected.result,
        CommandResult::Accepted { .. }
    ));
    let (state, compared) = resolve(state, target, vec![target_card.raw()], false, key + 2);
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_reqiaoshui_helpers_at_abi68() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    for def in [REQIAOSHUI, QIAOSHUI3, QIAOSHUI4] {
        assert!(package
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == def));
    }
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == REQIAOSHUI)
        .unwrap();
    assert_eq!(active.uses_per_turn, None);
}

#[test]
fn strict_win_grants_one_shot_helper_and_reqiaoshui_remains_repeatable() {
    let mut setup = setup(2_026_072_201, &[REQIAOSHUI]);
    let owner_card = inject(
        &mut setup.state,
        setup.owner,
        99_201,
        CardKind::Other,
        Suit::Spade,
        13,
    );
    let target_card = inject(
        &mut setup.state,
        setup.first,
        99_202,
        CardKind::Other,
        Suit::Heart,
        1,
    );
    let mut state = compare(
        setup.state,
        setup.owner,
        setup.first,
        owner_card,
        target_card,
        1,
    );
    assert!(has_skill(&state, setup.owner, QIAOSHUI3));
    let active = skill_id(&state, setup.owner, REQIAOSHUI);
    assert_eq!(state.skills.get(active).unwrap().use_count, 1);

    let second_owner = inject(
        &mut state,
        setup.owner,
        99_203,
        CardKind::Other,
        Suit::Club,
        12,
    );
    let _second_target = inject(
        &mut state,
        setup.first,
        99_204,
        CardKind::Other,
        Suit::Diamond,
        2,
    );
    let (state, activated_again) = command(
        state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.first],
            card_targets: Vec::new(),
        },
        4,
    );
    assert!(matches!(
        activated_again.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ActiveSkillSelection
    );
    assert!(state.hand_cards(setup.owner).contains(&second_owner));
}

#[test]
fn failed_compare_skips_play_and_keeps_tricks_out_of_discard_limit_until_turn_end() {
    let mut setup = setup(2_026_072_202, &[REQIAOSHUI]);
    let owner_card = inject(
        &mut setup.state,
        setup.owner,
        99_205,
        CardKind::Other,
        Suit::Spade,
        1,
    );
    let target_card = inject(
        &mut setup.state,
        setup.first,
        99_206,
        CardKind::Other,
        Suit::Heart,
        13,
    );
    let mut state = compare(
        setup.state,
        setup.owner,
        setup.first,
        owner_card,
        target_card,
        10,
    );
    assert_eq!(state.turn.phase, Phase::Discard);
    assert!(has_skill(&state, setup.owner, QIAOSHUI4));
    let trick = inject(
        &mut state,
        setup.owner,
        99_207,
        CardKind::TrickGuohe,
        Suit::Club,
        7,
    );
    assert!(card_ignored_for_hand_limit(&state, setup.owner, trick));
    assert!(!discard_to_limit_tokens(&state, setup.owner).contains(&trick));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: setup.owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: setup.owner,
                phase: Phase::Idle.to_u8(),
            },
        ],
    )
    .unwrap();
    assert!(!has_skill(tx.state(), setup.owner, QIAOSHUI4));
}

#[test]
fn one_shot_helper_adds_or_removes_a_saved_sha_target_and_consumes_itself() {
    let mut add_setup = setup(2_026_072_203, &[QIAOSHUI3, TARGET_BOOST]);
    let sha = inject(
        &mut add_setup.state,
        add_setup.owner,
        99_208,
        CardKind::Sha,
        Suit::Diamond,
        8,
    );
    let (state, played) = command(
        add_setup.state,
        add_setup.owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![add_setup.first],
            card_targets: Vec::new(),
        },
        20,
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddOrRemoveCurrentCardTarget
    );
    let (state, added) = resolve(
        state,
        add_setup.owner,
        vec![add_setup.second.raw()],
        false,
        21,
    );
    assert!(matches!(added.result, CommandResult::Accepted { .. }));
    assert!(added.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardTargetAdded { target, targets, .. }
            if *target == add_setup.second
                && targets == &[add_setup.first, add_setup.second]
    )));
    assert_eq!(state.players.get(add_setup.first).unwrap().hp, 9);
    assert_eq!(state.players.get(add_setup.second).unwrap().hp, 9);
    assert!(!has_skill(&state, add_setup.owner, QIAOSHUI3));

    let mut remove_setup = setup(2_026_072_204, &[QIAOSHUI3, TARGET_BOOST]);
    let sha = inject(
        &mut remove_setup.state,
        remove_setup.owner,
        99_209,
        CardKind::Sha,
        Suit::Club,
        9,
    );
    let (state, played) = command(
        remove_setup.state,
        remove_setup.owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![remove_setup.first, remove_setup.second],
            card_targets: Vec::new(),
        },
        30,
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let (state, mode) = resolve(state, remove_setup.owner, vec![1], false, 31);
    assert!(matches!(mode.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddOrRemoveCurrentCardTarget
    );
    let (state, removed) = resolve(
        state,
        remove_setup.owner,
        vec![remove_setup.second.raw()],
        false,
        32,
    );
    assert!(matches!(removed.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(remove_setup.first).unwrap().hp, 9);
    assert_eq!(state.players.get(remove_setup.second).unwrap().hp, 10);
    assert_eq!(state.players.get(remove_setup.third).unwrap().hp, 10);
    assert!(!has_skill(&state, remove_setup.owner, QIAOSHUI3));
}

#[test]
fn cancelling_the_optional_adjustment_still_consumes_qiaoshui3() {
    let mut setup = setup(2_026_072_205, &[QIAOSHUI3, TARGET_BOOST]);
    let sha = inject(
        &mut setup.state,
        setup.owner,
        99_210,
        CardKind::Sha,
        Suit::Spade,
        10,
    );
    let (state, played) = command(
        setup.state,
        setup.owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![setup.first],
            card_targets: Vec::new(),
        },
        40,
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let (state, cancelled) = resolve(state, setup.owner, Vec::new(), true, 41);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.first).unwrap().hp, 9);
    assert_eq!(state.players.get(setup.second).unwrap().hp, 10);
    assert!(!has_skill(&state, setup.owner, QIAOSHUI3));
}
