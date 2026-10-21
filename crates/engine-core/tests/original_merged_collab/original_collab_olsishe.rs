//! Golden for 联动包巳蛇：受到伤害后可对存活来源造成等量伤害。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::{canonical_state_hash, Engine};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const OL_SISHE: u32 = 2_716_794_819;
const OLSISHE: u32 = 2_380_847_221;
const SPCHIZHONG: u32 = 3_386_846_056;
const DCQIUSUO: u32 = 4_105_095_171;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/collab/collab.npack.json");

/// Concatenate every committed collab source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn collab_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/collab");
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
fn optional_reflection_uses_damage_amount_and_requires_a_living_source() {
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
        .find(|skill| skill.skill_def_id == OLSISHE)
        .unwrap();
    assert!(!skill.locked);
    let character = committed
        .character_index
        .iter()
        .find(|character| character.id == OL_SISHE)
        .unwrap();
    assert_eq!((character.initial_hp, character.max_hp), (0, 3));
    assert_eq!(character.skills, vec![OLSISHE]);

    let engine = started_with_character(&committed, OL_SISHE);
    let owner = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.character_def == OL_SISHE)
        .map(|player| player.id)
        .unwrap();
    let source = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(engine.state().clone());
    let skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.id)
        .collect();
    let mut effects: Vec<_> = skills
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLSISHE),
    });
    run_effects(&mut tx, effects).unwrap();

    let source_hp = tx.state().players.get(source).unwrap().hp;
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let paused = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, owner_hp - 2);

    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(source).unwrap().hp, source_hp - 2);
    assert_eq!(
        hex(canonical_state_hash(&state)),
        "33278f37ce2336eddd4fe3f15b45d6e0627e5fc00cc963319e1834b6506b2cd6"
    );

    let mut dead_source = Transaction::begin(state);
    dead_source
        .state_mut()
        .players
        .get_mut(source)
        .unwrap()
        .alive = false;
    let mut fires = TriggerFireRecord::new();
    let effects = dispatch_for_events(
        &mut dead_source,
        &[DomainEvent::DamageDealt {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut fires,
    )
    .unwrap();
    assert!(effects.is_empty());
}

#[test]
fn qiusuo_gains_from_draw_pile_before_discard_pile() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    assert!(matches!(
        engine.submit(start(20_260_716)),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCQIUSUO),
    });
    run_effects(&mut tx, effects).unwrap();

    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .unwrap();
    for zone in [draw, discard] {
        let existing: Vec<_> = tx
            .state()
            .zones
            .get(zone)
            .unwrap()
            .cards
            .iter()
            .copied()
            .filter(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.kind == CardKind::TrickTiesuo)
            })
            .collect();
        if !existing.is_empty() {
            run_effects(
                &mut tx,
                vec![EffectRequest::MoveCards {
                    cards: existing,
                    to: removed,
                    from: Some(zone),
                }],
            )
            .unwrap();
        }
    }
    let draw_tiesuo = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(DCQIUSUO + 1),
        kind: CardKind::TrickTiesuo,
        suit: Suit::Club,
        rank: 11,
        weapon_range: 0,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .push(draw_tiesuo);
    let discard_tiesuo = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(DCQIUSUO + 2),
        kind: CardKind::TrickTiesuo,
        suit: Suit::Spade,
        rank: 12,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(discard_tiesuo);

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(DCQIUSUO as u64),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).contains(&draw_tiesuo));
    assert!(!state.hand_cards(owner).contains(&discard_tiesuo));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::GainCardByKind {
            player: owner,
            kind: CardKind::TrickTiesuo.tag(),
        }],
    )
    .unwrap();
    assert!(tx.state().hand_cards(owner).contains(&discard_tiesuo));

    let hand = tx.state().hand_cards(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::GainCardByKind {
            player: owner,
            kind: CardKind::TrickTiesuo.tag(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_cards(owner), hand);
}

#[test]
fn spchizhong_uses_max_hp_for_hand_limit_and_gains_max_hp_after_another_death() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    assert!(matches!(
        engine.submit(start(20_260_816)),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let victim = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let skills = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id });
    let mut effects: Vec<_> = skills.collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPCHIZHONG),
    });
    run_effects(&mut tx, effects).unwrap();

    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = max_hp as i32 - 2;
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new()
        ),
        QueryResponse::Int(value) if value == max_hp as i32
    ));

    let hp = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: owner,
        }],
    )
    .unwrap();
    let owner_state = tx.state().players.get(owner).unwrap();
    assert_eq!(owner_state.max_hp, max_hp + 1);
    assert_eq!(owner_state.hp, hp);
}
