//! Golden for 联动包丑牛：初始 1/5 体力与全场最低体力回复。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::{draw_count_for, optional_draw_bonuses_for};
use engine_core::state::{turn_flag, Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::{canonical_state_hash, Engine};
use rule_api::EffectRequest;

const OL_CHOUNIU: u32 = 2_881_593_425;
const OLCHOUNIU: u32 = 3_858_737_350;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

/// Concatenate every committed collab source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn collab_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/collab");
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

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn started_with_character(package: &content_schema::ContentPackage, character: u32) -> Engine {
    (1..=256)
        .find_map(|seed| {
            let mut engine = Engine::new();
            engine.load_package(package).unwrap();
            engine.submit(start(seed));
            let found = engine
                .state()
                .players
                .iter_live()
                .any(|player| player.character_def == character);
            found.then_some(engine)
        })
        .expect("character must be dealable")
}

#[test]
fn complete_character_starts_damaged_and_recovers_only_at_minimum_hp() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let source = collab_sources();
    let mut compiled = content_compiler::compile_source_named("collab", &source).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
    let skill = committed
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == OLCHOUNIU)
        .unwrap();
    assert!(skill.locked);
    let character = committed
        .character_index
        .iter()
        .find(|character| character.id == OL_CHOUNIU)
        .unwrap();
    assert_eq!(character.initial_hp, 1);
    assert_eq!(character.max_hp, 5);
    assert_eq!(character.skills, vec![OLCHOUNIU]);

    let engine = started_with_character(&committed, OL_CHOUNIU);
    let owner = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.character_def == OL_CHOUNIU)
        .map(|player| player.id)
        .unwrap();
    let player = engine.state().players.get(owner).unwrap();
    let lord_bonus = u32::from(matches!(
        player.identity,
        engine_core::state::Identity::Lord
    ));
    assert_eq!(player.hp, (1 + lord_bonus) as i32);
    assert_eq!(player.max_hp, 5 + lord_bonus);

    let mut tx = Transaction::begin(engine.state().clone());
    let current = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let before = tx.state().players.get(owner).unwrap().hp;
    let mut fires = TriggerFireRecord::new();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::End,
        }],
        &mut fires,
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, before + 1);
    // 冻结哈希随批 237 quanjia 内容变更更新（新增触发模块改变同包装载态）。
    assert_eq!(
        hex(canonical_state_hash(tx.state())),
        "7b8a73b37e6a9e98b9acf03a05a81f7923906a8766ec5637deba97c4e05ada84"
    );

    tx.state_mut().players.get_mut(current).unwrap().hp = 1;
    let before = tx.state().players.get(owner).unwrap().hp;
    let mut fires = TriggerFireRecord::new();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::End,
        }],
        &mut fires,
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, before);
}

#[test]
fn bachiqionggouyu_recovers_after_play_and_draws_two_extra_only_at_full_hp() {
    const BACHIQIONGGOUYU: u32 = 2_812_865_529;

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    assert!(matches!(
        engine.submit(start(31)),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    let base_draw = draw_count_for(tx.state(), owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(BACHIQIONGGOUYU),
        }],
    )
    .unwrap();
    let max_hp = tx.state().players.get(owner).unwrap().max_hp as i32;
    tx.state_mut().players.get_mut(owner).unwrap().hp = max_hp - 1;
    assert_eq!(draw_count_for(tx.state(), owner), base_draw);

    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Discard.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, max_hp);
    assert_eq!(draw_count_for(tx.state(), owner), base_draw + 2);

    tx.state_mut().players.get_mut(owner).unwrap().hp = max_hp - 1;
    assert_eq!(draw_count_for(tx.state(), owner), base_draw);
}

#[test]
fn clbshuijian_draw_bonus_uses_half_the_equip_count_rounded_down_plus_one() {
    const CLBSHUIJIAN: u32 = 2_631_259_916;

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    assert!(matches!(
        engine.submit(start(2_631_259_916)),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, existing).unwrap();
    let base = draw_count_for(tx.state(), owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(CLBSHUIJIAN),
        }],
    )
    .unwrap();
    assert_eq!(draw_count_for(tx.state(), owner), base);
    assert_eq!(optional_draw_bonuses_for(tx.state(), owner), vec![1]);

    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    for (index, expected) in [1, 2, 2].into_iter().enumerate() {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(20_000 + index as u32),
            kind: CardKind::Treasure,
            suit: Suit::Club,
            rank: 1,
            weapon_range: 0,
            zone: equip,
            owner,
            face_up: true,
        });
        tx.state_mut()
            .zones
            .get_mut(equip)
            .unwrap()
            .cards
            .push(card);
        assert_eq!(optional_draw_bonuses_for(tx.state(), owner), vec![expected]);
    }

    tx.state_mut().turn.phase = engine_core::state::Phase::Judge;
    let before_draw = tx.state().hand_count(owner);
    let draw_revision = tx.state().revision;
    let draw_start = tx.state().clone();
    let (waiting, result) = apply_command(
        draw_start.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(draw_revision),
            idempotency_key: IdempotencyKey(2_631_259_917),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(waiting.has_pending_choice());
    assert_eq!(waiting.hand_count(owner), before_draw);
    let choice_id = ChoiceId::new(waiting.resolution.pending_choice_id);
    let revision = waiting.revision;
    let (accepted, result) = apply_command(
        waiting,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2_631_259_918),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(accepted.hand_count(owner), before_draw + base + 2);

    let (waiting, result) = apply_command(
        draw_start,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(draw_revision),
            idempotency_key: IdempotencyKey(2_631_259_919),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let choice_id = ChoiceId::new(waiting.resolution.pending_choice_id);
    let revision = waiting.revision;
    let (declined, result) = apply_command(
        waiting,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2_631_259_920),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![0],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(declined.hand_count(owner), before_draw + base);
}

#[test]
fn dcxurui_draws_attack_range_capped_five_only_without_turn_damage() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    assert!(matches!(
        engine.submit(start(20_260_821)),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(3_352_309_555),
        }],
    )
    .unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let weapon = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_997),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 8,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(equip)
        .unwrap()
        .cards
        .push(weapon);

    let before = tx.state().hand_count(owner);
    let mut fires = TriggerFireRecord::new();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut fires,
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 5);

    tx.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, 1);
    let before = tx.state().hand_count(owner);
    let mut fires = TriggerFireRecord::new();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut fires,
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before);
}
