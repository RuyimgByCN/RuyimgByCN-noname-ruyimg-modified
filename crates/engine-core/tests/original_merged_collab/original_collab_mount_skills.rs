//! Golden for 联动包「骏魁」「骥冠」的全局坐骑清理与常驻修正。

use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, ZoneKind};
use engine_core::Engine;
use rule_api::{QueryRequest, QueryResponse};

const MBJUNKUI: u32 = 3_131_135_455;
const MBJIGUAN: u32 = 2_860_585_832;
const COLLAB: &[u8] =
    include_bytes!("../../../../content/original/ported/collab/collab.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

fn start_with_skill(skill: u32, seed: u64) -> Engine {
    let mut collab = content_schema::ContentPackage::from_json_bytes(COLLAB).unwrap();
    assert!(collab
        .skill_index
        .iter()
        .any(|entry| entry.skill_def_id == skill && entry.locked));
    collab.character_index.clear();
    collab.manifest.logic_hash = collab.compute_logic_hash();

    let mut roster = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    for character in &mut roster.character_index {
        character.skills = vec![skill];
    }
    roster.manifest.logic_hash = roster.compute_logic_hash();

    let mut engine = Engine::new();
    engine.load_package(&collab).unwrap();
    engine.load_package(&roster).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    engine
}

fn assert_all_mounts_removed(engine: &Engine) {
    let state = engine.state();
    let removed = state.zones.find(ZoneKind::Removed, PlayerId::NONE).unwrap();
    let mounts: Vec<_> = state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| matches!(card.kind, CardKind::HorseOff | CardKind::HorseDef))
        .collect();
    assert!(!mounts.is_empty(), "standard deck must contain mount cards");
    assert!(mounts
        .iter()
        .all(|card| card.zone == removed && card.owner.is_none() && card.face_up));
    assert!(state.cards.sorted_live().into_iter().any(|card| !matches!(
        card.kind,
        CardKind::HorseOff | CardKind::HorseDef
    ) && card.zone != removed));
}

fn hand_limit(engine: &Engine, player: PlayerId) -> i32 {
    match query(
        engine.state(),
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("expected hand-limit integer, got {other:?}"),
    }
}

#[test]
fn mbjunkui_removes_every_mount_and_adds_one_sha_use() {
    let engine = start_with_skill(MBJUNKUI, 20_260_714_01);
    assert_all_mounts_removed(&engine);
    for player in engine.state().players.alive_by_seat() {
        assert_eq!(
            engine_core::query::sha_use_limit_for(engine.state(), player),
            2
        );
    }
}

#[test]
fn mbjiguan_removes_every_mount_and_adds_two_hand_limit() {
    let engine = start_with_skill(MBJIGUAN, 20_260_714_02);
    assert_all_mounts_removed(&engine);
    for player in engine.state().players.alive_by_seat() {
        let hp = engine.state().players.get(player).unwrap().hp.max(0);
        assert_eq!(hand_limit(&engine, player), hp + 2);
    }
}
