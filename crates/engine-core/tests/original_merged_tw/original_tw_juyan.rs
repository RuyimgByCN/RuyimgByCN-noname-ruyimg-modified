//! Golden for TW「炬湮」的火焰伤害语义。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const JUYAN: u32 = 2_964_355_463;
const MANJI: u32 = 3_959_991_082;
const LINGBAO_XIANHU: u32 = 2_267_146_098;
const JINTAO: u32 = 3_887_207_405;
const TWSAOTAO: u32 = 4_248_892_988;
/// Concatenate every committed tw source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn tw_sources() -> String {
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/original/ported/tw");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "skill"))
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

const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/tw/tw.npack.json");

#[test]
fn committed_package_only_reacts_to_fire_damage_from_owner() {
    std::thread::Builder::new().stack_size(16 << 20).spawn(|| {
        let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
        let mut compiled = content_compiler::compile_source_named("tw", &tw_sources()).unwrap();
        compiled.character_index = committed.character_index.clone();
        compiled.manifest.dependencies = committed.manifest.dependencies.clone();
        compiled.manifest.logic_hash = compiled.compute_logic_hash();
        assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
        assert!(committed
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == JUYAN));

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
        let target = tx.state_mut().players.get_mut(other).unwrap();
        target.hp = 10;
        target.max_hp = 10;
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JUYAN),
            }],
        )
        .unwrap();
        let hand = tx.state().hand_count(owner);

        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target: other,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), hand);
        assert_eq!(tx.state().players.get(other).unwrap().max_hp, 10);

        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target: other,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::FIRE,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), hand + 1);
        assert_eq!(tx.state().players.get(other).unwrap().max_hp, 9);
    });
}

#[test]
fn manji_executes_both_effects_when_hp_becomes_equal() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
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
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.hp = 2;
    owner_state.max_hp = 4;
    tx.state_mut().players.get_mut(other).unwrap().hp = 3;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(MANJI),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: other,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, 2);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
    assert_eq!(tx.state().hand_count(owner), hand + 1);
}

#[test]
fn lingbao_xianhu_requires_large_source_damage_but_accepts_any_death() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 20_260_715,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let mut others = engine
        .state()
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id);
    let target = others.next().unwrap();
    let victim = others.next().unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.hp = 2;
    owner_state.max_hp = 4;
    let target_state = tx.state_mut().players.get_mut(target).unwrap();
    target_state.hp = 10;
    target_state.max_hp = 10;
    for player in [owner, target, victim] {
        for skill in tx.state().players.get(player).unwrap().skills.clone() {
            run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
        }
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(LINGBAO_XIANHU),
        }],
    )
    .unwrap();
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp + 1);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);

    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: target,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp + 2);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
}

#[test]
fn jintao_rewrites_the_first_two_sha_uses_only() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(JINTAO as u64),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: JINTAO as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let target = seats[(owner_index + 2) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    for player in [owner, target] {
        for skill in tx.state().players.get(player).unwrap().skills.clone() {
            run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
        }
    }
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JINTAO),
            },
            EffectRequest::DiscardFromHand {
                player: target,
                count: u32::MAX,
            },
        ],
    )
    .unwrap();
    let target_state = tx.state_mut().players.get_mut(target).unwrap();
    target_state.hp = 10;
    target_state.max_hp = 10;
    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 2);

    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let first_sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(JINTAO + 1),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: owner_hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(owner_hand)
        .unwrap()
        .cards
        .push(first_sha);
    assert!(engine_core::query::card_distance_unlimited_for_target(
        tx.state(),
        owner,
        first_sha,
        target
    ));

    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 1);
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, first_sha, 1),
        2
    );
    assert!(!engine_core::query::sha_unrespondable_for_card_target(
        tx.state(),
        owner,
        target,
        first_sha
    ));
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 2);
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, first_sha, 1),
        1
    );
    assert!(engine_core::query::sha_unrespondable_for_card_target(
        tx.state(),
        owner,
        target,
        first_sha
    ));
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 3);
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, first_sha, 1),
        1
    );
    assert!(!engine_core::query::sha_unrespondable_for_card_target(
        tx.state(),
        owner,
        target,
        first_sha
    ));
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 0);

    let (state, first_result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: first_sha,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(
        first_result.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.players.get(target).unwrap().hp, 8);
    assert_eq!(
        state.turn.flag(engine_core::state::turn_flag::SHA_PLAYED),
        1
    );

    let mut tx = Transaction::begin(state);
    let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let shan = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(JINTAO + 2),
        kind: CardKind::Shan,
        suit: Suit::Heart,
        rank: 2,
        weapon_range: 0,
        zone: target_hand,
        owner: target,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(target_hand)
        .unwrap()
        .cards
        .push(shan);
    let second_sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(JINTAO + 3),
        kind: CardKind::Sha,
        suit: Suit::Club,
        rank: 8,
        weapon_range: 0,
        zone: owner_hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(owner_hand)
        .unwrap()
        .cards
        .push(second_sha);
    let (state, second_result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::PlayCard {
                card: second_sha,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(
        second_result.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.players.get(target).unwrap().hp, 7);
    assert!(state.hand_cards(target).contains(&shan));
    assert!(!state.has_pending_choice());
}

#[test]
fn twsaotao_makes_sha_and_ordinary_tricks_unrespondable() {
    let compiled = content_compiler::compile_source_named(
        "twsaotao",
        include_str!("../../../../content/original/ported/tw/twsaotao.skill"),
    )
    .unwrap();
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TWSAOTAO && skill.locked));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == TWSAOTAO
            && modifier.target == "sha_unrespondable"
            && modifier.selector.as_deref() == Some("any")
    }));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == TWSAOTAO
            && modifier.target == "card_use_unrespondable"
            && modifier.selector.as_deref() == Some("ordinary_trick")
    }));
}
