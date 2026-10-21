//! Golden coverage for 曹彰「将驰」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{
    query, sha_ignored_for_hand_limit, sha_response_cancelled_for, sha_use_limit_for, QueryContext,
};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const CAOZHANG: u32 = 1_812_908_078;
const NEW_JIANGCHI: u32 = 2_751_572_143;
const NEW_JIANGCHI3: u32 = 3_349_403_784;
const JIANGCHI2: u32 = 4_168_410_091;
const STANDARD: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");
const STANDARD_SOURCE: &str = include_str!("../../../../content/standard/standard_m1.skill");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&yijiang)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
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
        skill_def: SkillDefId::new(NEW_JIANGCHI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    Setup {
        state: tx.commit().state,
        owner,
        other,
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
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
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

fn end_phase(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    command(state, owner, CommandKind::EndPhase, key)
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

fn open_branch_choice(setup: Setup, key: u64) -> (GameState, PipelineResult) {
    let (state, offered) = end_phase(setup.state, setup.owner, key);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, accepted) = resolve(state, setup.owner, vec![1], false, key + 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrawOrDiscardAddSkill
    );
    (state, accepted)
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

fn attack_range(state: &GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::AttackRange { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("attack-range query failed: {other:?}"),
    }
}

#[test]
fn packages_publish_caozhang_cross_package_skill_and_abi_contracts() {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(standard.manifest.rule_vm_abi_version, 261);
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert!(yijiang.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "standard"
            && dependency.version == "0.1.0"
    }));
    let character = yijiang
        .character_index
        .iter()
        .find(|character| character.id == CAOZHANG)
        .unwrap();
    assert_eq!(character.skills, [NEW_JIANGCHI]);
    assert!(standard
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NEW_JIANGCHI));
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JIANGCHI2));

    let compiled = content_compiler::compile_source_named("standard_m1", STANDARD_SOURCE).unwrap();
    let expected = compiled
        .modules
        .iter()
        .find(|module| module.name == "new_jiangchi_phase_draw_end")
        .unwrap();
    let actual = standard
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn optional_trigger_can_be_declined_without_opening_the_branch_choice() {
    let setup = setup(2_026_073_001);
    let hand = setup.state.hand_count(setup.owner);
    let (state, offered) = end_phase(setup.state, setup.owner, 1);
    assert!(offered.choice.is_some());
    let (state, declined) = resolve(state, setup.owner, Vec::new(), true, 2);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.hand_count(setup.owner), hand);
    assert!(!has_skill(&state, setup.owner, NEW_JIANGCHI3));
    assert!(!has_skill(&state, setup.owner, JIANGCHI2));
}

#[test]
fn draw_branch_grants_all_restrictions_then_cleans_up_at_turn_end() {
    let setup = setup(2_026_073_002);
    let hand = setup.state.hand_count(setup.owner);
    let (state, branch) = open_branch_choice(setup, 10);
    assert_eq!(
        branch
            .choice
            .unwrap()
            .options
            .into_iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![1]
    );
    let owner = state.turn.current;
    let (state, drawn) = resolve(state, owner, vec![1], false, 12);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 1);
    assert!(has_skill(&state, owner, NEW_JIANGCHI3));
    assert_eq!(sha_use_limit_for(&state, owner), 0);
    assert!(sha_response_cancelled_for(&state, owner));
    assert!(sha_ignored_for_hand_limit(&state, owner));

    let (state, _) = end_phase(state, owner, 13);
    assert_eq!(state.turn.phase, Phase::Discard);
    assert!(has_skill(&state, owner, NEW_JIANGCHI3));
    let (state, _) = end_phase(state, owner, 14);
    assert_eq!(state.turn.phase, Phase::End);
    assert!(has_skill(&state, owner, NEW_JIANGCHI3));
    let (state, ended) = end_phase(state, owner, 15);
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert!(!has_skill(&state, owner, NEW_JIANGCHI3));
}

#[test]
fn discard_branch_revalidates_cost_then_grants_and_cleans_play_helper() {
    let mut setup = setup(2_026_073_003);
    let cost = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Equip,
        99_301,
        CardKind::Weapon,
    );
    let forged = inject(
        &mut setup.state,
        setup.other,
        ZoneKind::Hand,
        99_302,
        CardKind::Other,
    );
    let owner = setup.owner;
    let (state, branch) = open_branch_choice(setup, 20);
    assert_eq!(
        branch
            .choice
            .unwrap()
            .options
            .into_iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    let (state, cost_choice) = resolve(state, owner, vec![2], false, 22);
    assert!(matches!(cost_choice.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrawOrDiscardAddSkill
    );
    assert_eq!(
        cost_choice.choice.as_ref().unwrap().options[0].token,
        cost.raw()
    );

    let before = canonical_state_hash(&state);
    let (same, rejected) = resolve(state, owner, vec![forged.raw()], false, 23);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
    let (state, paid) = resolve(same, owner, vec![cost.raw()], false, 24);
    assert!(matches!(paid.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, JIANGCHI2));
    assert_eq!(attack_range(&state, owner), i32::MAX);
    assert_eq!(sha_use_limit_for(&state, owner), 2);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(cost).unwrap().zone, discard);

    let (state, ended) = end_phase(state, owner, 25);
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Discard);
    assert!(!has_skill(&state, owner, JIANGCHI2));
}

#[test]
fn discard_branch_is_absent_without_a_legal_cost_and_forgery_is_atomic() {
    let setup = setup(2_026_073_004);
    let owner = setup.owner;
    let (state, branch) = open_branch_choice(setup, 30);
    assert_eq!(
        branch
            .choice
            .unwrap()
            .options
            .into_iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![1]
    );
    let before = canonical_state_hash(&state);
    let (same, rejected) = resolve(state, owner, vec![2], false, 32);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
