//! Golden for ONLYOL「七弦」的手牌上限基数语义。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const QIXIAN: u32 = 3_046_538_003;
const OLQIANJU: u32 = 2_755_577_993;
const OLD_OLDANXIN: u32 = 4_026_350_658;
const OLD_OLDANXIN_MARK: u32 = 1_878_867_010;
const OLDANXIN: u32 = 2_442_439_931;
const OLJIAOZHAO_USED: u32 = 2_592_049_175;
const OLJIEYONG: u32 = 3_193_662_455;
const OL_GUANPING: u32 = 3_037_868_276;
const LONGYIN: u32 = 3_578_226_944;
/// Concatenate every committed onlyol source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn onlyol_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/onlyOL");
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

const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_500 + rank as u32),
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

fn distance(tx: &Transaction, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

#[test]
fn hand_limit_base_is_seven() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut compiled = content_compiler::compile_source_named("onlyOL", &onlyol_sources()).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);

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
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(QIXIAN),
        }],
    )
    .unwrap();
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(7)
    ));
}

#[test]
fn package_publishes_complete_ol_guanping_and_yijiang_dependency() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "yijiang"
            && dependency.version == "0.1.0"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_GUANPING)
        .unwrap();
    assert_eq!(character.name_key, "ol_guanping");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "shu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [LONGYIN, OLJIEYONG]);
}

#[test]
fn oljieyong_uses_exactly_one_red_hand_card_as_sha() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for window in ["play_phase", "response"] {
        assert!(package.view_as_index.iter().any(|view| {
            view.skill_def_id == OLJIEYONG
                && view.as_kind == "sha"
                && view.window == window
                && view.filter == "owner_single_red_hand"
                && view.source_zone.is_none()
        }));
    }
    let view_as_only = content_compiler::compile_source(include_str!(
        "../../../content/original/ported/onlyOL/oljieyong.skill"
    ))
    .unwrap();
    assert_eq!(view_as_only.manifest.rule_vm_abi_version, 5);

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_735),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_735,
            ruleset: "identity-standard".into(),
        },
    });
    let source = engine.state().turn.current;
    let responder = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    let setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(
            [source, responder]
                .into_iter()
                .map(|player| EffectRequest::DiscardFromHand {
                    player,
                    count: u32::MAX,
                }),
        )
        .chain([EffectRequest::AddSkill {
            player: responder,
            skill_def: SkillDefId::new(OLJIEYONG),
        }])
        .collect();
    run_effects(&mut tx, setup).unwrap();
    let first_duel = inject(&mut tx, source, CardKind::TrickJuedou, Suit::Spade, 2);
    let second_duel = inject(&mut tx, source, CardKind::TrickJuedou, Suit::Club, 3);
    let red = inject(&mut tx, responder, CardKind::Other, Suit::Heart, 4);
    let black = inject(&mut tx, responder, CardKind::Other, Suit::Spade, 5);
    let one_red = tx.state().clone();

    let (_, result) = apply_command(
        one_red.clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: first_duel,
                targets: vec![responder],
                card_targets: vec![],
            },
        },
    );
    let choice = result.choice.expect("竭勇应提供最后一张红牌作为响应杀");
    assert_eq!(choice.observer, responder);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == red.raw()));
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == black.raw()));

    let mut two_red = Transaction::begin(one_red);
    inject(&mut two_red, responder, CardKind::Other, Suit::Diamond, 6);
    let (_, result) = apply_command(
        two_red.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::PlayCard {
                card: second_duel,
                targets: vec![responder],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
}

#[test]
fn olqianju_uses_lost_hp_and_all_targets_in_range() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_794),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 20_260_794,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let far = seats[(owner_index + 2) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLQIANJU),
        }],
    )
    .unwrap();

    assert_eq!(distance(&tx, owner, far), 2);
    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 1);

    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    assert_eq!(distance(&tx, owner, far), 1);
    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 2);
}

#[test]
fn old_oldanxin_draws_each_time_and_caps_its_upgrade_mark_at_two() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_795),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_795,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
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
        player.hp = 10;
        player.max_hp = 10;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLD_OLDANXIN),
        }],
    )
    .unwrap();
    let initial_hand = tx.state().hand_count(owner);
    let mut state = tx.state().clone();

    for activation in 1..=3 {
        let mut damage = Transaction::begin(state);
        assert_eq!(
            run_effects(
                &mut damage,
                vec![EffectRequest::DealDamage {
                    source,
                    target: owner,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .unwrap_err(),
            EffectError::ChoiceRequested
        );
        let (next, result) = apply_command(
            damage.state().clone(),
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(activation),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(damage.state().resolution.pending_choice_id),
                    selections: vec![1],
                    cancelled: false,
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert_eq!(next.hand_count(owner), initial_hand + activation as u32);
        assert_eq!(
            next.players
                .get(owner)
                .unwrap()
                .marks
                .iter()
                .find(|(mark, _)| *mark == OLD_OLDANXIN_MARK)
                .map_or(0, |(_, count)| *count),
            activation.min(2) as i32
        );
        state = next;
    }
}

#[test]
fn oldanxin_draws_by_pre_upgrade_level_and_clears_jiaozhao_used_at_level_two() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLDANXIN && !skill.locked));
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLJIAOZHAO_USED));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_716_22),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_716_22,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
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
        player.hp = 10;
        player.max_hp = 10;
    }
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(OLDANXIN),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(OLJIAOZHAO_USED),
            },
        ],
    )
    .unwrap();
    let initial_hand = tx.state().hand_count(owner);
    let mut state = tx.state().clone();

    for activation in 1..=3 {
        let mut damage = Transaction::begin(state);
        assert_eq!(
            run_effects(
                &mut damage,
                vec![EffectRequest::DealDamage {
                    source,
                    target: owner,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .unwrap_err(),
            EffectError::ChoiceRequested
        );
        let (next, result) = apply_command(
            damage.state().clone(),
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(10 + activation),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(damage.state().resolution.pending_choice_id),
                    selections: vec![1],
                    cancelled: false,
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        let expected_drawn = match activation {
            1 => 0,
            2 => 1,
            _ => 3,
        };
        assert_eq!(next.hand_count(owner), initial_hand + expected_drawn);
        assert_eq!(
            next.players
                .get(owner)
                .unwrap()
                .marks
                .iter()
                .find(|(mark, _)| *mark == OLDANXIN)
                .map_or(0, |(_, count)| *count),
            activation.min(2) as i32
        );
        let has_used = next
            .skills
            .sorted_live()
            .into_iter()
            .any(|skill| skill.owner == owner && skill.def.raw() == OLJIAOZHAO_USED);
        assert_eq!(has_used, activation == 1);
        state = next;
    }
}
