//! Golden tests for SP/SP2「伏骑」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const SP: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");
const SP2: &[u8] = include_bytes!("../../../../content/original/ported/sp2/sp2.npack.json");
const FUQI_SOURCE: &str = include_str!("../../../../content/original/ported/sp/fuqi.skill");
const REFUQI_SOURCE: &str = include_str!("../../../../content/original/ported/sp2/refuqi.skill");
const FUQI: u32 = 3_157_963_287;
const REFUQI: u32 = 4_088_557_780;
const JIAOZI: u32 = 2_415_897_133;
const QUYI: u32 = 2_408_538_527;
const RE_QUYI: u32 = 2_820_866_421;
const DEFENSIVE_DISTANCE: u32 = 4_294_000_001;
const MULTI_SHA: u32 = 4_294_000_002;
const FIXTURE_SOURCE: &str = r#"
skill test_defensive_distance(4294000001) {
  modifier distance_to add 1;
}
skill test_multi_sha(4294000002) {
  modifier sha_target_limit add 1;
  modifier card_distance_unlimited(sha) override 1;
}
"#;

fn start(seed: u64) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

fn loaded(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let sp = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    let sp2 = content_schema::ContentPackage::from_json_bytes(SP2).unwrap();
    let fixture = content_compiler::compile_source_named("fuqi-fixture", FIXTURE_SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&sp).unwrap();
    engine.load_package(&sp2).unwrap();
    engine.load_package(&fixture).unwrap();
    engine.submit(start(seed));

    let source = engine.state().turn.current;
    let order = engine.state().players.alive_by_seat();
    let pos = order.iter().position(|player| *player == source).unwrap();
    let near = order[(pos + 1) % order.len()];
    let far = order[(pos + 2) % order.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    let remove: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    (tx, source, near, far)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, skill_def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
}

fn remove_skill(tx: &mut Transaction, skill_def: u32) {
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == skill_def)
        .unwrap()
        .id;
    run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
}

fn clear_hands(tx: &mut Transaction) {
    let players = tx.state().players.alive_by_seat();
    for player in players {
        run_effects(
            tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::new(9_700 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    };
    let id = tx.state_mut().cards.alloc(card);
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(id);
    id
}

fn distance(tx: &Transaction, from: PlayerId, to: PlayerId) -> i32 {
    match engine_core::query::query(
        tx.state(),
        &QueryRequest::Distance { from, to },
        &mut engine_core::query::QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected distance response: {other:?}"),
    }
}

fn play(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
            },
        },
    )
}

#[test]
fn committed_packages_match_sources_and_close_both_characters() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    let sp2 = content_schema::ContentPackage::from_json_bytes(SP2).unwrap();
    let compiled_fuqi = content_compiler::compile_source_named("fuqi", FUQI_SOURCE).unwrap();
    let compiled_refuqi = content_compiler::compile_source_named("refuqi", REFUQI_SOURCE).unwrap();

    assert!(compiled_fuqi.skill_index[0].locked);
    assert_eq!(
        compiled_fuqi.modifier_index[0].target,
        "card_use_unrespondable_within_responder_to_owner_distance"
    );
    assert!(compiled_refuqi.skill_index[0].locked);
    assert_eq!(
        compiled_refuqi.modifier_index[0].target,
        "card_use_unrespondable_within_owner_to_responder_distance"
    );
    assert!(sp
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == FUQI && skill.locked));
    assert!(sp2
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == REFUQI && skill.locked));
    assert!(sp2.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "sp"
            && dependency.version == "0.1.0"
    }));
    assert!(sp
        .character_index
        .iter()
        .any(|character| { character.id == QUYI && character.skills == vec![FUQI, JIAOZI] }));
    assert!(sp2
        .character_index
        .iter()
        .any(|character| { character.id == RE_QUYI && character.skills == vec![REFUQI, JIAOZI] }));
}

#[test]
fn legacy_distance_direction_is_preserved() {
    let (mut tx, source, near, far) = loaded(20_260_715);
    assert_eq!(distance(&tx, source, near), 1);
    assert_eq!(distance(&tx, near, source), 1);
    assert!(distance(&tx, source, far) > 1);

    add_skill(&mut tx, source, FUQI);
    assert!(engine_core::query::card_use_unrespondable_within_distance(
        tx.state(),
        source,
        near
    ));
    assert!(!engine_core::query::card_use_unrespondable_within_distance(
        tx.state(),
        source,
        far
    ));
    assert!(!engine_core::query::card_use_unrespondable_within_distance(
        tx.state(),
        source,
        source
    ));

    add_skill(&mut tx, source, DEFENSIVE_DISTANCE);
    assert_eq!(distance(&tx, source, near), 1);
    assert_eq!(distance(&tx, near, source), 2);
    assert!(!engine_core::query::card_use_unrespondable_within_distance(
        tx.state(),
        source,
        near
    ));

    remove_skill(&mut tx, FUQI);
    add_skill(&mut tx, source, REFUQI);
    assert!(engine_core::query::card_use_unrespondable_within_distance(
        tx.state(),
        source,
        near
    ));
}

#[test]
fn fuqi_locks_only_near_targets_in_a_multi_target_sha() {
    let (mut tx, source, near, far) = loaded(20_260_716);
    clear_hands(&mut tx);
    add_skill(&mut tx, source, FUQI);
    add_skill(&mut tx, source, MULTI_SHA);
    let sha = inject(&mut tx, source, CardKind::Sha, Suit::Spade, 7);
    inject(&mut tx, near, CardKind::Shan, Suit::Heart, 8);
    inject(&mut tx, far, CardKind::Shan, Suit::Diamond, 9);
    tx.state_mut().players.get_mut(near).unwrap().hp = 10;
    tx.state_mut().players.get_mut(near).unwrap().max_hp = 10;
    tx.state_mut().players.get_mut(far).unwrap().hp = 10;
    tx.state_mut().players.get_mut(far).unwrap().max_hp = 10;

    let (state, pipeline) = play(&tx, source, sha, vec![near, far]);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(near).unwrap().hp, 9);
    assert_eq!(state.players.get(far).unwrap().hp, 10);
    assert_eq!(pipeline.choice.unwrap().observer, far);
}

#[test]
fn fuqi_applies_to_aoe_duel_and_wuxie_windows() {
    let (mut nanman_tx, source, near, far) = loaded(20_260_717);
    clear_hands(&mut nanman_tx);
    add_skill(&mut nanman_tx, source, FUQI);
    let nanman = inject(
        &mut nanman_tx,
        source,
        CardKind::TrickNanman,
        Suit::Spade,
        7,
    );
    inject(&mut nanman_tx, near, CardKind::Sha, Suit::Heart, 8);
    inject(&mut nanman_tx, far, CardKind::Sha, Suit::Diamond, 9);
    nanman_tx.state_mut().players.get_mut(near).unwrap().hp = 10;
    nanman_tx.state_mut().players.get_mut(near).unwrap().max_hp = 10;
    let (nanman_state, nanman_pipeline) = play(&nanman_tx, source, nanman, vec![]);
    assert!(matches!(
        nanman_pipeline.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(nanman_state.players.get(near).unwrap().hp, 9);
    assert_eq!(nanman_pipeline.choice.unwrap().observer, far);

    let (mut duel_tx, source, near, _) = loaded(20_260_718);
    clear_hands(&mut duel_tx);
    add_skill(&mut duel_tx, source, FUQI);
    let duel = inject(&mut duel_tx, source, CardKind::TrickJuedou, Suit::Club, 1);
    inject(&mut duel_tx, near, CardKind::Sha, Suit::Spade, 2);
    duel_tx.state_mut().players.get_mut(near).unwrap().hp = 10;
    duel_tx.state_mut().players.get_mut(near).unwrap().max_hp = 10;
    let (duel_state, duel_pipeline) = play(&duel_tx, source, duel, vec![near]);
    assert!(matches!(
        duel_pipeline.result,
        CommandResult::Accepted { .. }
    ));
    assert!(!duel_state.has_pending_choice());
    assert_eq!(duel_state.players.get(near).unwrap().hp, 9);

    let (mut wuxie_tx, source, near, far) = loaded(20_260_719);
    clear_hands(&mut wuxie_tx);
    add_skill(&mut wuxie_tx, source, FUQI);
    let wuzhong = inject(
        &mut wuxie_tx,
        source,
        CardKind::TrickWuzhong,
        Suit::Heart,
        7,
    );
    inject(&mut wuxie_tx, near, CardKind::TrickWuxie, Suit::Club, 11);
    let far_wuxie = inject(&mut wuxie_tx, far, CardKind::TrickWuxie, Suit::Spade, 12);
    let (_, wuxie_pipeline) = play(&wuxie_tx, source, wuzhong, vec![]);
    assert!(matches!(
        wuxie_pipeline.result,
        CommandResult::Accepted { .. }
    ));
    let choice = wuxie_pipeline.choice.unwrap();
    assert_eq!(choice.observer, far);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == far_wuxie.raw()));
}
