//! Golden tests for SP「奇门八卦」与「鸟翔」rewritten from legacy JavaScript.

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

const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const QIMEN_SOURCE: &str = include_str!("../../../content/original/ported/sp/qimenbagua.skill");
const NIAOXIANG_SOURCE: &str = include_str!("../../../content/original/ported/sp/zniaoxiang.skill");
const QIMENBAGUA: u32 = 2_757_055_338;
const ZNIAOXIANG: u32 = 3_916_851_836;
const IGNORE_ARMOR: u32 = 4_290_000_067;

fn start(seed: u64) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

fn loaded(seed: u64, with_fixture: bool) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    let fixture = content_compiler::compile_source_named(
        "qimen-fixture",
        &format!("skill ignore_armor({IGNORE_ARMOR}) {{ modifier ignore_armor override 1; }}"),
    )
    .unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    if with_fixture {
        engine.load_package(&fixture).unwrap();
    }
    assert!(matches!(
        engine.submit(start(seed)),
        CommandResult::Accepted { .. }
    ));
    // 2026-08-15：sp 包出现 optional game_started 技能（olpingyuan/xiaoxi 等），
    // 抽到相关武将时 StartGame 会留下挂起的可选触发确认，击穿后续 setup 批次；
    // 此处以「取消」排空开局选择，对任何 optional 开局技保持鲁棒。
    let mut drained = engine.state().clone();
    let mut drain_key = 1u64;
    while drained.has_pending_choice() {
        let choice = engine_api::ChoiceId::new(drained.resolution.pending_choice_id);
        let revision = drained.revision;
        let (next, output) = apply_command(
            drained,
            engine_api::Command {
                actor: engine_api::PlayerId::NONE,
                expected_revision: engine_api::ExpectedRevision(revision),
                idempotency_key: engine_api::IdempotencyKey(seed ^ drain_key),
                kind: engine_api::CommandKind::ResolveChoice {
                    choice_id: choice,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(
            matches!(output.result, engine_api::CommandResult::Accepted { .. }),
            "drain rejected: {:?}",
            output.result
        );
        drained = next;
        drain_key += 1;
    }
    let source = drained.turn.current;
    let target = drained
        .players
        .iter_live()
        .find(|player| player.id != source)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(drained);
    let remove = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    for player in [source, target] {
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
    (tx, source, target)
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

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(67_000 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn play(source: PlayerId, card: CardId, target: PlayerId, key: u64) -> Command {
    Command {
        actor: source,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::PlayCard {
            card,
            targets: vec![target],
            card_targets: vec![],
        },
    }
}

#[test]
fn committed_package_contains_both_exact_modifiers() {
    let committed = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    let qimen = content_compiler::compile_source_named("qimenbagua", QIMEN_SOURCE).unwrap();
    let niaoxiang = content_compiler::compile_source_named("zniaoxiang", NIAOXIANG_SOURCE).unwrap();

    assert_eq!(qimen.modifier_index[0].target, "armor_card_target_cancel");
    assert_eq!(qimen.modifier_index[0].selector.as_deref(), Some("sha"));
    assert_eq!(niaoxiang.modifier_index[0].target, "response_count");
    for skill in [QIMENBAGUA, ZNIAOXIANG] {
        assert!(committed
            .skill_index
            .iter()
            .any(|entry| entry.skill_def_id == skill && entry.locked));
    }
}

#[test]
fn qimenbagua_blocks_sha_only_while_armor_effects_apply() {
    let (mut blocked, source, target) = loaded(20_260_718, false);
    add_skill(&mut blocked, target, QIMENBAGUA);
    let sha = inject(&mut blocked, source, CardKind::Sha, 7);
    let hp = blocked.state().players.get(target).unwrap().hp;
    let (state, pipeline) = apply_command(blocked.state().clone(), play(source, sha, target, 1));
    assert!(matches!(pipeline.result, CommandResult::Rejected { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    assert!(state.hand_cards(source).contains(&sha));

    let (mut bypassed, source, target) = loaded(20_260_719, true);
    add_skill(&mut bypassed, target, QIMENBAGUA);
    add_skill(&mut bypassed, source, IGNORE_ARMOR);
    let sha = inject(&mut bypassed, source, CardKind::Sha, 8);
    let hp = bypassed.state().players.get(target).unwrap().hp;
    let (state, pipeline) = apply_command(bypassed.state().clone(), play(source, sha, target, 2));
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn zniaoxiang_requires_two_shan_for_each_sha_target() {
    let (mut tx, source, target) = loaded(20_260_720, false);
    add_skill(&mut tx, source, ZNIAOXIANG);
    assert_eq!(
        engine_core::query::response_count_for(tx.state(), source, 1),
        2
    );
    let sha = inject(&mut tx, source, CardKind::Sha, 9);
    let first = inject(&mut tx, target, CardKind::Shan, 10);
    let second = inject(&mut tx, target, CardKind::Shan, 11);
    let hp = tx.state().players.get(target).unwrap().hp;

    let (state, pipeline) = apply_command(tx.state().clone(), play(source, sha, target, 3));
    let first_choice = pipeline.choice.unwrap();
    let (state, pipeline) = apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::ResolveChoice {
                choice_id: first_choice.choice_id,
                selections: vec![first.raw()],
                cancelled: false,
            },
        },
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RespondToSha
    );
    let second_choice = pipeline.choice.unwrap();
    let (state, pipeline) = apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(5),
            kind: CommandKind::ResolveChoice {
                choice_id: second_choice.choice_id,
                selections: vec![second.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, hp);
}
