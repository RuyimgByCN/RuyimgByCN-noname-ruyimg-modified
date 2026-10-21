//! Golden coverage for 界曹彰「将驰」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{
    card_distance_unlimited_for_card, sha_response_cancelled_for, sha_use_limit_for,
};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_CAOZHANG: u32 = 2_557_216_502;
const XINJIANGCHI: u32 = 3_041_318_040;
const XINJIANGCHI_LESS: u32 = 2_327_944_393;
const XINJIANGCHI_MORE: u32 = 3_884_080_518;
const SOURCE: &str = include_str!("../../../../content/original/ported/refresh/xinjiangchi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
        skill_def: SkillDefId::new(XINJIANGCHI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    Setup {
        state: tx.commit().state,
        owner,
        other: players[1],
    }
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(
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

fn end_phase(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    submit(state, owner, CommandKind::EndPhase, key)
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

fn open_branch_choice(setup: Setup, key: u64) -> (GameState, PipelineResult) {
    let (state, opened) = end_phase(setup.state, setup.owner, key);
    assert!(matches!(opened.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrawOrDiscardAddSkill
    );
    (state, opened)
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
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

fn finish_turn(mut state: GameState, owner: PlayerId, key: u64) -> GameState {
    for offset in 0..3 {
        let (next, result) = end_phase(state, owner, key + offset);
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        state = next;
    }
    state
}

#[test]
fn package_publishes_complete_re_caozhang_and_exact_abi315_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 315);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_CAOZHANG)
        .unwrap();
    assert_eq!(character.skills, [XINJIANGCHI]);
    for skill in [XINJIANGCHI, XINJIANGCHI_LESS, XINJIANGCHI_MORE] {
        assert!(package
            .skill_index
            .iter()
            .any(|entry| entry.skill_def_id == skill));
    }
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINJIANGCHI)
        .unwrap();
    assert_eq!(trigger.event, "phase_play");
    assert!(!trigger.optional);

    let compiled = content_compiler::compile_source_named("xinjiangchi", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 315);
    for expected in compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn cancel_and_plain_draw_are_available_without_a_discard_cost() {
    let cancellable = setup(2_026_073_101);
    let owner = cancellable.owner;
    let hand = cancellable.state.hand_count(owner);
    let (state, branch) = open_branch_choice(cancellable, 1);
    let choice = branch.choice.as_ref().unwrap();
    assert!(choice.allow_cancel);
    assert_eq!((choice.min_count, choice.max_count), (0, 1));
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    let (state, cancelled) = resolve(state, owner, Vec::new(), true, 2);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand);
    assert!(!has_skill(&state, owner, XINJIANGCHI_LESS));
    assert!(!has_skill(&state, owner, XINJIANGCHI_MORE));

    let plain = setup(2_026_073_102);
    let owner = plain.owner;
    let hand = plain.state.hand_count(owner);
    let (state, _) = open_branch_choice(plain, 3);
    let (state, drawn) = resolve(state, owner, vec![1], false, 4);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 1);
    assert!(!has_skill(&state, owner, XINJIANGCHI_LESS));
    assert!(!has_skill(&state, owner, XINJIANGCHI_MORE));
}

#[test]
fn draw_two_branch_blocks_sha_use_and_response_until_turn_end() {
    let setup = setup(2_026_073_103);
    let owner = setup.owner;
    let hand = setup.state.hand_count(owner);
    let (state, _) = open_branch_choice(setup, 10);
    let (state, drawn) = resolve(state, owner, vec![2], false, 11);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 2);
    assert!(has_skill(&state, owner, XINJIANGCHI_LESS));
    assert_eq!(sha_use_limit_for(&state, owner), 0);
    assert!(sha_response_cancelled_for(&state, owner));

    let state = finish_turn(state, owner, 12);
    assert!(!has_skill(&state, owner, XINJIANGCHI_LESS));
}

#[test]
fn discard_branch_revalidates_cost_and_boosts_sha_until_turn_end() {
    let mut setup = setup(2_026_073_104);
    let owner = setup.owner;
    let cost = inject(
        &mut setup.state,
        owner,
        ZoneKind::Equip,
        99_321,
        CardKind::Weapon,
    );
    let sha = inject(
        &mut setup.state,
        owner,
        ZoneKind::Hand,
        99_322,
        CardKind::Sha,
    );
    let forged = inject(
        &mut setup.state,
        setup.other,
        ZoneKind::Hand,
        99_323,
        CardKind::Tao,
    );
    let (state, branch) = open_branch_choice(setup, 20);
    assert_eq!(
        branch
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    let (state, cost_choice) = resolve(state, owner, vec![3], false, 21);
    assert!(matches!(cost_choice.result, CommandResult::Accepted { .. }));

    let before = canonical_state_hash(&state);
    let (same, rejected) = resolve(state, owner, vec![forged.raw()], false, 22);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let (state, paid) = resolve(same, owner, vec![cost.raw()], false, 23);
    assert!(matches!(paid.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, XINJIANGCHI_MORE));
    assert_eq!(sha_use_limit_for(&state, owner), 2);
    assert!(card_distance_unlimited_for_card(&state, owner, sha));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(cost).unwrap().zone, discard);

    let state = finish_turn(state, owner, 24);
    assert!(!has_skill(&state, owner, XINJIANGCHI_MORE));
}
