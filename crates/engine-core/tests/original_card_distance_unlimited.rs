//! Golden coverage for the shared all-card distance-unlimited modifier.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const DRLT_JUEYAN2: u32 = 3_629_461_497;
const QINGYI3: u32 = 3_610_775_756;
const ZHANYI_TRICK: u32 = 2_374_118_291;
const FEIYING: u32 = 2_669_566_023;
const QICAI: u32 = 2_286_640_837;
const DUANLIANG: u32 = 2_757_928_964;
const DUANLIANG2: u32 = 2_158_464_053;
const DUANLIANG3: u32 = 4_145_070_079;
const GZDUANLIANG: u32 = 2_148_140_249;

fn strip_skills(tx: &mut Transaction, player: PlayerId) {
    let skills = tx.state().players.get(player).unwrap().skills.clone();
    for skill in skills {
        run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
}

fn inject_hand_card(tx: &mut Transaction, owner: PlayerId, kind: CardKind) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(9_700 + kind.tag() as u32),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    };
    let card = tx.state_mut().cards.alloc(card);
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn transaction(package: &[u8], seed: u64) -> (Transaction, PlayerId, PlayerId) {
    transaction_players(package, seed, 4)
}

fn transaction_players(
    package: &[u8],
    seed: u64,
    player_count: u8,
) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let actor = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let actor_index = seats.iter().position(|player| *player == actor).unwrap();
    let far = seats[(actor_index + seats.len() / 2) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    strip_skills(&mut tx, actor);
    strip_skills(&mut tx, far);
    (tx, actor, far)
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

fn distance(tx: &Transaction, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        response => panic!("unexpected distance response: {response:?}"),
    }
}

#[test]
fn legacy_skills_unlock_sha_shunshou_and_bingliang_at_distance_two() {
    for (case, (package, skill_def)) in [(SHENHUA, DRLT_JUEYAN2), (SP, QINGYI3), (SP, ZHANYI_TRICK)]
        .into_iter()
        .enumerate()
    {
        for (kind_index, kind) in [
            CardKind::Sha,
            CardKind::TrickShunshou,
            CardKind::TrickBingliang,
        ]
        .into_iter()
        .enumerate()
        {
            let (mut tx, actor, far) =
                transaction(package, 20_260_760 + (case * 10 + kind_index) as u64);
            let card = inject_hand_card(&mut tx, actor, kind);
            let command = Command {
                actor,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(1),
                kind: CommandKind::PlayCard {
                    card,
                    targets: vec![far],
                    card_targets: vec![],
                },
            };

            let (_, rejected) = apply_command(tx.state().clone(), command.clone());
            assert!(
                matches!(rejected.result, CommandResult::Rejected { .. }),
                "{kind:?} should keep its ordinary distance limit"
            );

            run_effects(
                &mut tx,
                vec![EffectRequest::AddSkill {
                    player: actor,
                    skill_def: SkillDefId::new(skill_def),
                }],
            )
            .unwrap();
            assert!(engine_core::query::card_distance_unlimited_for(
                tx.state(),
                actor
            ));
            let (_, accepted) = apply_command(tx.state().clone(), command);
            assert!(
                matches!(accepted.result, CommandResult::Accepted { .. }),
                "{kind:?} should ignore distance with skill {skill_def}: {:?}",
                accepted.result
            );
        }
    }
}

#[test]
fn qicai_only_unlocks_trick_distance() {
    for (index, kind) in [CardKind::TrickShunshou, CardKind::TrickBingliang]
        .into_iter()
        .enumerate()
    {
        let (mut tx, actor, far) = transaction(STANDARD, 20_260_800 + index as u64);
        let card = inject_hand_card(&mut tx, actor, kind);
        assert!(!engine_core::query::card_target_within_distance(
            tx.state(),
            actor,
            card,
            far,
            1,
        ));
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: actor,
                skill_def: SkillDefId::new(QICAI),
            }],
        )
        .unwrap();
        assert!(engine_core::query::card_target_within_distance(
            tx.state(),
            actor,
            card,
            far,
            1,
        ));
        let (_, accepted) = apply_command(
            tx.state().clone(),
            Command {
                actor,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(2),
                kind: CommandKind::PlayCard {
                    card,
                    targets: vec![far],
                    card_targets: vec![],
                },
            },
        );
        assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    }

    let (mut tx, actor, far) = transaction(STANDARD, 20_260_802);
    let sha = inject_hand_card(&mut tx, actor, CardKind::Sha);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: actor,
            skill_def: SkillDefId::new(QICAI),
        }],
    )
    .unwrap();
    assert!(!engine_core::query::card_target_within_distance(
        tx.state(),
        actor,
        sha,
        far,
        1,
    ));
}

#[test]
fn feiying_only_increases_distance_to_its_owner() {
    let (mut tx, actor, far) = transaction(STANDARD, 20_260_803);
    assert_eq!(distance(&tx, actor, far), 2);
    assert_eq!(distance(&tx, far, actor), 2);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: far,
            skill_def: SkillDefId::new(FEIYING),
        }],
    )
    .unwrap();
    assert_eq!(distance(&tx, actor, far), 3);
    assert_eq!(distance(&tx, far, actor), 2);
}

#[test]
fn duanliang2_extends_only_bingliang_to_distance_two() {
    let (mut tx, actor, far) = transaction(SHENHUA, 20_260_804);
    assert_eq!(distance(&tx, actor, far), 2);
    let bingliang = inject_hand_card(&mut tx, actor, CardKind::TrickBingliang);
    assert!(!engine_core::query::card_target_within_distance(
        tx.state(),
        actor,
        bingliang,
        far,
        1,
    ));
    add_skill(&mut tx, actor, GZDUANLIANG);
    assert!(engine_core::query::card_target_within_distance(
        tx.state(),
        actor,
        bingliang,
        far,
        1,
    ));
    let (_, accepted) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::PlayCard {
                card: bingliang,
                targets: vec![far],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));

    let shunshou = inject_hand_card(&mut tx, actor, CardKind::TrickShunshou);
    assert!(!engine_core::query::card_target_within_distance(
        tx.state(),
        actor,
        shunshou,
        far,
        1,
    ));

    let (mut tx, actor, far) = transaction_players(SHENHUA, 20_260_805, 6);
    assert_eq!(distance(&tx, actor, far), 3);
    let bingliang = inject_hand_card(&mut tx, actor, CardKind::TrickBingliang);
    add_skill(&mut tx, actor, DUANLIANG2);
    assert!(!engine_core::query::card_target_within_distance(
        tx.state(),
        actor,
        bingliang,
        far,
        1,
    ));
    let (_, rejected) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::PlayCard {
                card: bingliang,
                targets: vec![far],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}

#[test]
fn duanliang3_unlocks_only_bingliang_for_targets_with_at_least_as_many_hand_cards() {
    for (seed, skill_def) in [(20_260_806, DUANLIANG3), (20_260_807, DUANLIANG)] {
        let (mut tx, actor, far) = transaction(SHENHUA, seed);
        assert_eq!(distance(&tx, actor, far), 2);
        let bingliang = inject_hand_card(&mut tx, actor, CardKind::TrickBingliang);
        add_skill(&mut tx, actor, skill_def);

        while tx.state().hand_cards(far).len() >= tx.state().hand_cards(actor).len() {
            inject_hand_card(&mut tx, actor, CardKind::Other);
        }
        assert!(!engine_core::query::card_target_within_distance(
            tx.state(),
            actor,
            bingliang,
            far,
            1,
        ));

        while tx.state().hand_cards(far).len() < tx.state().hand_cards(actor).len() {
            inject_hand_card(&mut tx, far, CardKind::Other);
        }
        assert!(engine_core::query::card_target_within_distance(
            tx.state(),
            actor,
            bingliang,
            far,
            1,
        ));
        let (_, accepted) = apply_command(
            tx.state().clone(),
            Command {
                actor,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(5),
                kind: CommandKind::PlayCard {
                    card: bingliang,
                    targets: vec![far],
                    card_targets: vec![],
                },
            },
        );
        assert!(matches!(accepted.result, CommandResult::Accepted { .. }));

        let shunshou = inject_hand_card(&mut tx, actor, CardKind::TrickShunshou);
        assert!(!engine_core::query::card_target_within_distance(
            tx.state(),
            actor,
            shunshou,
            far,
            1,
        ));
    }
}
