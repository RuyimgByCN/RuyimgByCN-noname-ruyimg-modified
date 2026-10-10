//! Golden for HUICUI「刚骨」的失去体力与每回合限一次语义。

use engine_api::{
    Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const GANGU: u32 = 3_823_814_559;
const SHENWEI: u32 = 2_271_917_703;
const YUXIANG: u32 = 3_515_326_729;
const RESHISHOU: u32 = 2_341_206_262;
const RESHISHOU2: u32 = 2_672_855_040;
const QINGMAN: u32 = 2_536_693_396;
const RECANGCHU: u32 = 2_416_555_605;
const RECANGCHU2: u32 = 2_551_663_610;
const RECANGCHU3: u32 = 3_636_591_775;
const DCCUNWEI: u32 = 4_022_570_059;
const XIXIU: u32 = 3_425_169_185;
const CHENJIAN: u32 = 3_452_524_123;
const TENGYIN: u32 = 4_177_903_929;
const RECANGCHU_MARK: u32 = 2_058_376_721;
const DISTANCE_FIXTURE: u32 = 4_294_967_000;
const SUIT_FIXTURE: u32 = 4_294_966_998;
/// Concatenate every committed huicui source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn huicui_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/huicui");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().map_or(false, |ext| ext == "skill")
        })
        .map(|entry| entry.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|path| {
            let mut source = std::fs::read_to_string(path).unwrap();
            source.push('\n');
            source
        })
        .collect()
}

const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/huicui/huicui.npack.json");

fn recangchu_marks(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == RECANGCHU_MARK)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn committed_package_distinguishes_hp_loss_and_fires_once() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut compiled = content_compiler::compile_source_named("huicui", &huicui_sources()).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == GANGU));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == CHENJIAN));
    let tengyin = committed
        .character_index
        .iter()
        .find(|character| character.id == TENGYIN)
        .expect("tengyin character");
    assert_eq!(tengyin.name_key, "tengyin");
    assert_eq!(tengyin.max_hp, 3);
    assert_eq!(tengyin.skills, vec![CHENJIAN, XIXIU]);

    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_713,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(GANGU),
    });
    run_effects(&mut tx, setup).unwrap();
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 8;
    tx.state_mut().players.get_mut(other).unwrap().hp = 8;
    let hand = tx.state().hand_count(owner);
    let hp = tx.state().players.get(owner).unwrap().hp;

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: other,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 3);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: other,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 3);
}

#[test]
fn yuxiang_uses_each_targets_final_distance_for_sha_response_lock() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_823),
        kind: CommandKind::StartGame {
            player_count: 8,
            rng_seed: 20_260_823,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let distance_fixture = content_compiler::compile_source_named(
        "distance-fixture",
        "skill distance_fixture(4294967000) { modifier distance_to add 5; }",
    )
    .unwrap();
    engine.load_package(&distance_fixture).unwrap();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(YUXIANG),
        }],
    )
    .unwrap();
    let far = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .max_by_key(|target| {
            match query(
                tx.state(),
                &QueryRequest::Distance {
                    from: owner,
                    to: *target,
                },
                &mut QueryContext::new(),
            ) {
                QueryResponse::Int(distance) => distance,
                _ => 0,
            }
        })
        .unwrap();
    let sha = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::Sha;
    assert!(engine_core::query::sha_unrespondable_for_card_target(
        tx.state(),
        owner,
        far,
        sha
    ));
    assert!(!engine_core::query::sha_unrespondable_for_card(
        tx.state(),
        owner,
        sha
    ));

    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: far,
            skill_def: SkillDefId::new(DISTANCE_FIXTURE),
        }],
    )
    .unwrap();
    let distance = match query(
        tx.state(),
        &QueryRequest::Distance {
            from: owner,
            to: far,
        },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(distance) => distance,
        other => panic!("unexpected distance response: {other:?}"),
    };
    assert!(distance > 2);
    assert!(!engine_core::query::sha_unrespondable_for_card_target(
        tx.state(),
        owner,
        far,
        sha
    ));
}

#[test]
fn shenwei_adds_two_draws_and_two_hand_limit() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let base_draw = engine_core::query::draw_count_for(tx.state(), owner);
    let base_hand = match engine_core::query::query(
        tx.state(),
        &rule_api::QueryRequest::HandLimit { player: owner },
        &mut engine_core::query::QueryContext::new(),
    ) {
        rule_api::QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    };
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHENWEI),
        }],
    )
    .unwrap();
    assert_eq!(
        engine_core::query::draw_count_for(tx.state(), owner),
        base_draw + 2
    );
    assert!(matches!(
        engine_core::query::query(
            tx.state(),
            &rule_api::QueryRequest::HandLimit { player: owner },
            &mut engine_core::query::QueryContext::new(),
        ),
        rule_api::QueryResponse::Int(value) if value == base_hand + 2
    ));
}

#[test]
fn reshishou2_loses_hp_at_prepare_only_without_recangchu_mark() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_714),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    for (marks, expected_hp) in [(0, 4), (1, 5)] {
        let mut tx = Transaction::begin(engine.state().clone());
        let remove_existing: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        run_effects(&mut tx, remove_existing).unwrap();
        {
            let player = tx.state_mut().players.get_mut(owner).unwrap();
            player.hp = 5;
            player.max_hp = 5;
        }
        let mut setup = vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RESHISHOU2),
        }];
        if marks > 0 {
            setup.push(EffectRequest::AddMark {
                player: owner,
                mark: engine_api::MarkDefId::new(RECANGCHU_MARK),
                delta: marks,
            });
        }
        run_effects(&mut tx, setup).unwrap();
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::Prepare.to_u8(),
            }],
        )
        .unwrap();
        assert_eq!(tx.state().players.get(owner).unwrap().hp, expected_hp);
    }
}

#[test]
fn reshishou_consumes_grain_for_logical_jiu_and_fire_damage() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let fixture = content_compiler::compile_source_named(
        "logical-jiu-fixture",
        "skill logical_jiu_fixture(4294966999) locked { modifier card_name(sha) override jiu; }",
    )
    .unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.load_package(&fixture).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_716),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_716,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RESHISHOU),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(4_294_966_999),
        },
        EffectRequest::AddMark {
            player: owner,
            mark: engine_api::MarkDefId::new(RECANGCHU_MARK),
            delta: 4,
        },
    ]);
    run_effects(&mut tx, setup).unwrap();
    for player in [owner, other] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }

    let owner_card = tx.state().hand_cards(owner)[0];
    let other_card = tx.state().hand_cards(other)[0];
    tx.state_mut().cards.get_mut(owner_card).unwrap().kind = CardKind::Sha;
    tx.state_mut().cards.get_mut(other_card).unwrap().kind = CardKind::Jiu;
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: other,
            card: other_card,
            targets: vec![],
        }],
    )
    .unwrap();
    assert_eq!(recangchu_marks(&tx, owner), 4);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: owner,
            card: owner_card,
            targets: vec![],
        }],
    )
    .unwrap();
    assert_eq!(recangchu_marks(&tx, owner), 3);

    for (nature, amount, expected_marks) in [(0, 1, 3), (1, 2, 1), (1, 2, 0)] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount,
                is_lose_hp: false,
                nature,
            }],
        )
        .unwrap();
        assert_eq!(recangchu_marks(&tx, owner), expected_marks);
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
}

#[test]
fn recangchu_rewrites_all_three_legacy_js_behaviors() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for skill_def in [RECANGCHU, RECANGCHU2, RECANGCHU3] {
        assert!(committed
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == skill_def));
    }

    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_717),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 20_260_717,
            ruleset: "identity-standard".into(),
        },
    });
    let players = engine.state().players.alive_by_seat();
    let [owner, other, third, fourth] = players.as_slice() else {
        panic!("expected four players");
    };
    let (owner, other, third, fourth) = (*owner, *other, *third, *fourth);
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    let base_hand_limit = match query(
        tx.state(),
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(limit) => limit,
        other => panic!("unexpected hand-limit response: {other:?}"),
    };
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RECANGCHU),
        }],
    )
    .unwrap();

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 4 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(recangchu_marks(&tx, owner), 3);
    assert_eq!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(base_hand_limit + 3)
    );

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: other,
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::DrawCards {
                player: owner,
                count: 1,
            },
        ],
    )
    .unwrap();
    assert_eq!(recangchu_marks(&tx, owner), 4);

    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let other_card = tx.state().hand_cards(other)[0];
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveMark {
                player: owner,
                mark: engine_api::MarkDefId::new(RECANGCHU_MARK),
                amount: 1,
            },
            EffectRequest::MoveCards {
                cards: vec![other_card],
                to: owner_hand,
                from: Some(other_hand),
            },
        ],
    )
    .unwrap();
    assert_eq!(recangchu_marks(&tx, owner), 3);

    // SetTurnState is a low-level fixture effect and does not run the command-layer turn reset.
    let recangchu_skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == RECANGCHU)
        .unwrap()
        .id;
    tx.state_mut()
        .skills
        .get_mut(recangchu_skill)
        .unwrap()
        .turn_fire_count = 0;
    let third_hand = tx.state().zones.find(ZoneKind::Hand, third).unwrap();
    let third_card = tx.state().hand_cards(third)[0];
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: third,
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::MoveCards {
                cards: vec![third_card],
                to: owner_hand,
                from: Some(third_hand),
            },
        ],
    )
    .unwrap();
    assert_eq!(recangchu_marks(&tx, owner), 4);

    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: fourth,
            killer: owner,
        }],
    )
    .unwrap();
    assert_eq!(recangchu_marks(&tx, owner), 3);
}

#[test]
fn qingman_draws_to_the_ending_players_empty_equip_slots() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_715),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_715,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(QINGMAN),
        },
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
    ]);
    run_effects(&mut tx, setup).unwrap();

    let equip = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let equipment = tx.state().hand_cards(other)[..2].to_vec();
    tx.state_mut().cards.get_mut(equipment[0]).unwrap().kind = CardKind::Weapon;
    tx.state_mut().cards.get_mut(equipment[1]).unwrap().kind = CardKind::Armor;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: equipment,
                to: equip,
                from: Some(hand),
            },
            EffectRequest::SetTurnState {
                current: other,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 3);

    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    tx.state_mut().players.get_mut(other).unwrap().alive = false;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: other,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 0);
}

#[test]
fn dccunwei_draws_for_single_target_and_discards_only_for_payable_multi_target_tricks() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_716),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_716,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCCUNWEI),
    });
    run_effects(&mut tx, setup).unwrap();

    let source_cards = tx.state().hand_cards(source);
    let basic = source_cards[0];
    let single_trick = source_cards[1];
    let multi_trick = source_cards[2];
    tx.state_mut().cards.get_mut(basic).unwrap().kind = CardKind::Sha;
    tx.state_mut().cards.get_mut(single_trick).unwrap().kind = CardKind::TrickGuohe;
    tx.state_mut().cards.get_mut(multi_trick).unwrap().kind = CardKind::TrickNanman;
    let initial_hand = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: basic,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), initial_hand);

    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: single_trick,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), initial_hand + 1);

    let discarded = tx.state().hand_cards(owner)[0];
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: source,
                card: multi_trick,
                targets: vec![owner, source],
            }],
        )
        .unwrap_err(),
        engine_core::effect::EffectError::ChoiceRequested
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        engine_core::state::ContinuationKind::DiscardOneHandOrEquip
    );
    let state = tx.commit().state;
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![discarded.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(
        result.result,
        engine_api::CommandResult::Accepted { .. }
    ));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(discarded).unwrap().zone, discard);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 0);
    let source_hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![multi_trick, single_trick],
            to: source_hand,
            from: Some(discard),
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: multi_trick,
            targets: vec![owner, source],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());

    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: single_trick,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 1);
}

#[test]
fn xixiu_uses_raw_event_suit_effective_equip_suit_and_protects_only_the_sole_equip() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let suit_fixture = content_compiler::compile_source_named(
        "suit-fixture",
        "skill suit_fixture(4294966998) locked { modifier card_suit(spade) override heart; }",
    )
    .unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.load_package(&suit_fixture).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_717),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_717,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XIXIU),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SUIT_FIXTURE),
        },
        EffectRequest::AddSkill {
            player: source,
            skill_def: SkillDefId::new(SUIT_FIXTURE),
        },
    ]);
    run_effects(&mut tx, setup).unwrap();

    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let owner_equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let equipment = tx.state().hand_cards(owner)[..2].to_vec();
    tx.state_mut().cards.get_mut(equipment[0]).unwrap().kind = CardKind::Weapon;
    tx.state_mut().cards.get_mut(equipment[0]).unwrap().suit = engine_api::Suit::Spade;
    tx.state_mut().cards.get_mut(equipment[1]).unwrap().kind = CardKind::Armor;
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![equipment[0]],
            to: owner_equip,
            from: Some(owner_hand),
        }],
    )
    .unwrap();
    assert!(engine_core::query::card_discard_protected_from(
        tx.state(),
        source,
        owner,
        equipment[0]
    ));
    assert!(!engine_core::query::card_discard_protected_from(
        tx.state(),
        owner,
        owner,
        equipment[0]
    ));

    let source_cards = tx.state().hand_cards(source);
    let raw_spade = source_cards[0];
    let raw_heart = source_cards[1];
    tx.state_mut().cards.get_mut(raw_spade).unwrap().suit = engine_api::Suit::Spade;
    tx.state_mut().cards.get_mut(raw_heart).unwrap().suit = engine_api::Suit::Heart;
    let initial_hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: raw_spade,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), initial_hand);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: raw_heart,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), initial_hand + 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![equipment[1]],
            to: owner_equip,
            from: Some(owner_hand),
        }],
    )
    .unwrap();
    assert!(!engine_core::query::card_discard_protected_from(
        tx.state(),
        source,
        owner,
        equipment[0]
    ));
}
