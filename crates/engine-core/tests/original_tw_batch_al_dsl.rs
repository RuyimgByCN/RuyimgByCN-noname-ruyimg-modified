//! Golden for batch-AL rewrites:
//! 浮沉 (twfucheng, forced): at prepare phase, an all-red hand grants 璧月
//!   (sb/sbbiyue 4152380059) and an all-black hand grants 武神
//!   (sb/sbwushuang 4242421006); mixed or empty hands grant nothing.
//! Also verifies the 13 tw *_sxrm_caocao audio-only noops are finalized
//! (honest noops, no stale TODO headers — upstream `{ audio: 1 }`).

use engine_api::{
    CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const TWFUCHENG: u32 = 3_898_925_066;
const SBBIYUE: u32 = 4_152_380_059;
const SBWUSHUANG: u32 = 4_242_421_006;
const TW_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");
const SB_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sb/sb.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let tw = content_schema::ContentPackage::from_json_bytes(TW_PACKAGE).unwrap();
    let sb = content_schema::ContentPackage::from_json_bytes(SB_PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&tw).unwrap();
    engine.load_package(&sb).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn inject(state: &mut GameState, player: PlayerId, suit: engine_api::Suit, tag: u32) {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(91_900 + tag),
        kind: CardKind::Tao,
        suit,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
}

fn with_clean_owner(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
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

fn run_prepare(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_batch_al_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(TW_PACKAGE).unwrap();
    let entry = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TWFUCHENG)
        .expect("twfucheng trigger");
    assert_eq!(entry.event.as_str(), "phase_prepare");
    assert_eq!(entry.filter.as_str(), "owner_is_target");
    assert!(!entry.optional);
}

#[test]
fn twfucheng_all_red_hand_grants_sbbiyue() {
    let (state, owner) = started(2_026_081_540);
    let mut state = with_clean_owner(state, owner);
    // Empty the hand, then refill with red suits only.
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let mut state = tx.commit().state;
    inject(&mut state, owner, engine_api::Suit::Heart, 1);
    inject(&mut state, owner, engine_api::Suit::Diamond, 2);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWFUCHENG),
        }],
    )
    .unwrap();
    let state = tx.commit().state;

    let state = run_prepare(state, owner);
    assert!(has_skill(&state, owner, SBBIYUE));
    assert!(!has_skill(&state, owner, SBWUSHUANG));
}

#[test]
fn twfucheng_all_black_hand_grants_sbwushuang() {
    let (state, owner) = started(2_026_081_541);
    let mut state = with_clean_owner(state, owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let mut state = tx.commit().state;
    inject(&mut state, owner, engine_api::Suit::Spade, 3);
    inject(&mut state, owner, engine_api::Suit::Club, 4);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWFUCHENG),
        }],
    )
    .unwrap();
    let state = tx.commit().state;

    let state = run_prepare(state, owner);
    assert!(has_skill(&state, owner, SBWUSHUANG));
    assert!(!has_skill(&state, owner, SBBIYUE));
}

#[test]
fn twfucheng_mixed_hand_grants_nothing() {
    let (state, owner) = started(2_026_081_542);
    let state = with_clean_owner(state, owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let mut state = tx.commit().state;
    inject(&mut state, owner, engine_api::Suit::Heart, 5);
    inject(&mut state, owner, engine_api::Suit::Spade, 6);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWFUCHENG),
        }],
    )
    .unwrap();
    let state = tx.commit().state;

    let state = run_prepare(state, owner);
    assert!(!has_skill(&state, owner, SBBIYUE));
    assert!(!has_skill(&state, owner, SBWUSHUANG));
}

#[test]
fn sxrm_caocao_audio_only_noops_are_finalized() {
    // The 13 *_sxrm_caocao skills are upstream `{ audio: 1 }` aliases with no rule
    // surface; their .skill files must be honest noops without stale TODO headers.
    let package = content_schema::ContentPackage::from_json_bytes(TW_PACKAGE).unwrap();
    let names = [
        "zhichi_sxrm_caocao",
        "reganglie_sxrm_caocao",
        "refankui_sxrm_caocao",
        "reyiji_sxrm_caocao",
        "oljieming_sxrm_caocao",
        "fangzhu_sxrm_caocao",
        "shibei_sxrm_caocao",
        "rechengxiang_sxrm_caocao",
        "zhiyu_sxrm_caocao",
        "jilei_sxrm_caocao",
        "benyu_sxrm_caocao",
        "chouce_sxrm_caocao",
        "wuhun_sxrm_caocao",
    ];
    for name in names {
        let path = format!("{}/../../content/original/ported/tw/{name}.skill", env!("CARGO_MANIFEST_DIR"));
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(!source.contains("// TODO"), "{name} still has a TODO header");
        assert!(source.contains("INTENTIONAL_EMPTY_NOOP"), "{name} lost its noop marker");
        assert!(
            source.contains("audio-only"),
            "{name} lost its upstream audio-only citation"
        );
    }
    // Their defs still resolve in the committed package.
    for def in [
        4_274_763_640u32, 3_210_720_173, 4_249_767_960, 3_253_555_636, 4_093_162_705,
        2_307_818_525, 3_631_705_171, 2_285_536_647, 4_195_097_884, 3_398_468_226,
        3_695_755_888, 3_218_932_758, 2_867_527_212,
    ] {
        assert!(
            package.skill_index.iter().any(|skill| skill.skill_def_id == def),
            "def {def} missing from tw package"
        );
    }
}
