//! Golden for SP/OLD 张星彩「枪舞」rewritten from legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const QIANGWU: u32 = 2_563_176_834;
const SHENXIAN: u32 = 3_316_671_408;
const OLD_SHENXIAN: u32 = 2_584_430_535;
const ZHANGXINGCAI: u32 = 3_565_568_608;
const OLD_ZHANGXINGCAI: u32 = 2_854_497_058;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/qiangwu.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const OLD_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

fn inject(
    state: &mut GameState,
    zone: ZoneKind,
    owner: PlayerId,
    kind: CardKind,
    rank: u8,
    def: u32,
) -> CardId {
    let zone_id = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: zone_id,
        owner,
        face_up: zone != ZoneKind::Hand,
    });
    state.zones.get_mut(zone_id).unwrap().cards.push(card);
    card
}

fn setup() -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = compile_source_named("sp-qiangwu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_072_501, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let far_target = players[2];
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(QIANGWU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == QIANGWU)
        .unwrap()
        .id;
    (tx.commit().state, owner, far_target, skill)
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn play(state: GameState, owner: PlayerId, card: CardId, target: PlayerId, key: u64) -> GameState {
    let (state, pipeline) = submit(
        state,
        owner,
        CommandKind::PlayCard {
            card,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        key,
    );
    assert!(
        matches!(pipeline.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipeline.result
    );
    state
}

#[test]
fn packages_publish_both_zhangxingcai_closures() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    assert!(sp
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == QIANGWU));
    assert_eq!(
        sp.character_index
            .iter()
            .find(|character| character.id == ZHANGXINGCAI)
            .unwrap()
            .skills,
        [SHENXIAN, QIANGWU]
    );

    let old = content_schema::ContentPackage::from_json_bytes(OLD_PACKAGE).unwrap();
    assert_eq!(
        old.character_index
            .iter()
            .find(|character| character.id == OLD_ZHANGXINGCAI)
            .unwrap()
            .skills,
        [OLD_SHENXIAN, QIANGWU]
    );
    assert!(old
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "sp"));
}

#[test]
fn judgment_rank_drives_strict_lower_distance_and_higher_no_count() {
    let (mut state, owner, far_target, skill) = setup();
    inject(
        &mut state,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Other,
        7,
        188_001,
    );
    let low = inject(&mut state, ZoneKind::Hand, owner, CardKind::Sha, 3, 188_002);
    let equal = inject(&mut state, ZoneKind::Hand, owner, CardKind::Sha, 7, 188_003);
    let high = inject(&mut state, ZoneKind::Hand, owner, CardKind::Sha, 9, 188_004);
    let higher = inject(
        &mut state,
        ZoneKind::Hand,
        owner,
        CardKind::Sha,
        10,
        188_005,
    );
    let rankless = inject(&mut state, ZoneKind::Hand, owner, CardKind::Sha, 0, 188_006);

    let (state, activated) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert!(activated.events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::JudgmentResolved {
                player,
                rank: 7,
                ..
            } if *player == owner
        )
    }));
    assert_eq!(
        state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == QIANGWU)
            .map(|(_, count)| *count),
        Some(7)
    );
    assert!(engine_core::query::card_distance_unlimited_for_target(
        &state, owner, low, far_target
    ));
    assert!(!engine_core::query::card_distance_unlimited_for_target(
        &state, owner, equal, far_target
    ));
    assert!(!engine_core::query::sha_use_does_not_count_for_card(
        &state, owner, equal
    ));
    assert!(engine_core::query::sha_use_does_not_count_for_card(
        &state, owner, high
    ));
    assert!(engine_core::query::card_distance_unlimited_for_target(
        &state, owner, rankless, far_target
    ));
    assert!(engine_core::query::sha_use_does_not_count_for_card(
        &state, owner, rankless
    ));

    let adjacent = state.players.alive_by_seat()[1];
    let state = play(state, owner, low, far_target, 2);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    let state = play(state, owner, high, adjacent, 3);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    let state = play(state, owner, higher, adjacent, 4);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);

    let (_, rejected) = submit(
        state,
        owner,
        CommandKind::PlayCard {
            card: equal,
            targets: vec![adjacent],
            card_targets: Vec::new(),
        },
        5,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}

#[test]
fn play_phase_end_clears_rank_policy() {
    let (mut state, owner, far_target, skill) = setup();
    inject(
        &mut state,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Other,
        8,
        188_010,
    );
    let low = inject(&mut state, ZoneKind::Hand, owner, CardKind::Sha, 2, 188_011);
    let (state, activated) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        10,
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert!(engine_core::query::card_distance_unlimited_for_target(
        &state, owner, low, far_target
    ));

    let (state, ended) = submit(state, owner, CommandKind::EndPhase, 11);
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert!(!state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, _)| *mark == QIANGWU));
    assert!(!engine_core::query::card_distance_unlimited_for_target(
        &state, owner, low, far_target
    ));
}
