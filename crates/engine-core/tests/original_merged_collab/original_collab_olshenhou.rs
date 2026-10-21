//! Golden tests for 联动包「申猴」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const COLLAB: &[u8] =
    include_bytes!("../../../../content/original/ported/collab/collab.npack.json");
const SOURCE: &str = include_str!("../../../../content/original/ported/collab/olshenhou.skill");
const OL_SHENHOU: u32 = 2_433_370_010;
const OLSHENHOU: u32 = 2_393_323_694;
const MULTI_SHA: u32 = 4_290_000_066;

fn start(seed: u64, player_count: u8) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

fn loaded(package: &content_schema::ContentPackage, seed: u64) -> (Transaction, [PlayerId; 3]) {
    let mut engine = Engine::new();
    engine.load_package(package).unwrap();
    assert!(matches!(
        engine.submit(start(seed, 3)),
        CommandResult::Accepted { .. }
    ));
    let source = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let source_index = seats.iter().position(|player| *player == source).unwrap();
    let players = [
        source,
        seats[(source_index + 1) % seats.len()],
        seats[(source_index + 2) % seats.len()],
    ];
    let mut tx = Transaction::begin(engine.state().clone());
    let remove = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    for player in players {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    tx.state_mut().turn.phase = Phase::Play;
    (tx, players)
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

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(66_000 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn inject_judgment(tx: &mut Transaction, suit: Suit) -> CardId {
    inject(
        tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        suit,
        7,
    )
}

fn play_and_confirm(
    tx: &Transaction,
    source: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    owner: PlayerId,
) -> engine_core::state::GameState {
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let choice = pipeline.choice.unwrap();
    assert_eq!(choice.observer, owner);
    let (state, pipeline) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipeline.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipeline.result
    );
    state
}

#[test]
fn committed_package_contains_the_complete_rewritten_character() {
    let committed = content_schema::ContentPackage::from_json_bytes(COLLAB).unwrap();
    let compiled = content_compiler::compile_source_named("olshenhou", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
    assert_eq!(
        compiled.trigger_index[0].filter,
        "owner_targeted_by_other_sha"
    );
    assert!(compiled.trigger_index[0].optional);

    let character = committed
        .character_index
        .iter()
        .find(|character| character.id == OL_SHENHOU)
        .unwrap();
    assert_eq!((character.initial_hp, character.max_hp), (0, 3));
    assert_eq!(character.skills, vec![OLSHENHOU]);
}

fn assert_single_target_judgment(suit: Suit, damage: i32, seed: u64) {
    let package = content_schema::ContentPackage::from_json_bytes(COLLAB).unwrap();
    let (mut tx, [source, owner, _]) = loaded(&package, seed);
    add_skill(&mut tx, owner, OLSHENHOU);
    let judgment = inject_judgment(&mut tx, suit);
    let sha = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        8,
    );
    let before = tx.state().players.get(owner).unwrap().hp;

    let state = play_and_confirm(&tx, source, sha, vec![owner], owner);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, before - damage);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&judgment));
}

#[test]
fn red_judgment_excludes_only_the_current_sha_target() {
    assert_single_target_judgment(Suit::Heart, 0, 20_260_715);

    let fixture = content_compiler::compile_source_named(
        "olshenhou-multi",
        &format!("{SOURCE}\nskill multi_sha({MULTI_SHA}) {{ modifier sha_target_limit add 1; }}"),
    )
    .unwrap();
    let (mut tx, [source, owner, other]) = loaded(&fixture, 20_260_716);
    add_skill(&mut tx, owner, OLSHENHOU);
    add_skill(&mut tx, source, MULTI_SHA);
    inject_judgment(&mut tx, Suit::Diamond);
    let sha = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        9,
    );
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let other_hp = tx.state().players.get(other).unwrap().hp;

    let state = play_and_confirm(&tx, source, sha, vec![owner, other], owner);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp - 1);
}

#[test]
fn black_judgment_keeps_the_sha_effective() {
    assert_single_target_judgment(Suit::Spade, 1, 20_260_717);
}
