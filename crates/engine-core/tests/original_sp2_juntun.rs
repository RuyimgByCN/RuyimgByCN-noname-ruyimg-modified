//! Golden for SP2「军屯」的准备阶段语义。

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{CardKind, Identity, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const JUNTUN: u32 = 3_811_889_239;
const ZHIHU: u32 = 3_057_288_085;
const CHENGGONG: u32 = 2_957_700_161;
const HUANGKONG: u32 = 2_905_812_698;
const QUANJIU: u32 = 2_358_780_202;
const STARZHONGGU: u32 = 2_431_837_100;
const RELIEWEI: u32 = 3_943_534_187;
const STARSUJUN: u32 = 4_007_048_185;
const YECHOU2: u32 = 3_899_060_351;
/// Concatenate every committed sp2 source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn sp2_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/sp2");
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

const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn started_engine(package: &content_schema::ContentPackage) -> Engine {
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
    engine.load_package(package).unwrap();
    engine
}

#[test]
fn yechou2_loses_one_hp_at_every_turn_end() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let ending_player = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(YECHOU2),
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(owner).unwrap().hp;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: ending_player,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

#[test]
fn chenggong_only_offers_to_draw_for_a_multi_target_card() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let user = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(CHENGGONG),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(user);
    let mut fires = TriggerFireRecord::new();
    let paused = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: user,
            card: CardId::NONE,
            targets: vec![owner, user],
        }],
        &mut fires,
    )
    .unwrap_err();
    assert_eq!(paused, engine_core::effect::EffectError::ChoiceRequested);

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(user), hand + 1);

    let mut single = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut single,
        &[DomainEvent::CardUsed {
            player: user,
            card: CardId::NONE,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
}

fn prepare(tx: &mut Transaction, owner: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
}

fn accept_optional(tx: Transaction, owner: PlayerId, key: u64) -> Transaction {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    Transaction::begin(state)
}

#[test]
fn committed_package_loses_max_hp_then_draws_original_max_hp() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut compiled = content_compiler::compile_source_named("sp2", &sp2_sources()).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JUNTUN));

    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.hp = 4;
        player.max_hp = 4;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JUNTUN),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    prepare(&mut tx, owner);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 3);
    assert_eq!(tx.state().hand_count(owner), hand + 4);

    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 1;
    let hand = tx.state().hand_count(owner);
    prepare(&mut tx, owner);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 1);
    assert_eq!(tx.state().hand_count(owner), hand);
}

#[test]
fn zhihu_draws_twice_per_turn_for_damage_to_others() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
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
            skill_def: SkillDefId::new(ZHIHU),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);

    for expected in [hand + 2, hand + 4, hand + 4] {
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
        assert_eq!(tx.state().hand_count(owner), expected);
    }
}

#[test]
fn huangkong_draws_only_when_targeted_outside_the_owners_turn_with_no_hand() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(HUANGKONG),
        },
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
    ]);
    run_effects(&mut tx, effects).unwrap();

    let cards = tx.state().hand_cards(source);
    assert!(cards.len() >= 3);
    tx.state_mut().cards.get_mut(cards[0]).unwrap().kind = CardKind::Sha;
    tx.state_mut().cards.get_mut(cards[1]).unwrap().kind = CardKind::TrickGuohe;
    tx.state_mut().cards.get_mut(cards[2]).unwrap().kind = CardKind::Sha;

    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: cards[0],
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 0);

    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: source,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: cards[1],
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 2);

    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: cards[2],
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 2);
}

#[test]
fn quanjiu_uses_physical_identity_for_the_converted_shas_use_limit() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let skills = tx.state().players.get(owner).unwrap().skills.clone();
    for skill in skills {
        run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(QUANJIU),
        }],
    )
    .unwrap();

    let cards = tx.state().hand_cards(owner);
    let jiu = cards[0];
    let sha = cards[1];
    tx.state_mut().cards.get_mut(jiu).unwrap().kind = CardKind::Jiu;
    tx.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::Sha;

    let physical = tx.state().cards.get(jiu).unwrap();
    assert_eq!(physical.kind, CardKind::Jiu);
    assert_eq!(
        engine_core::query::effective_card_kind(tx.state(), owner, physical),
        CardKind::Sha
    );
    assert!(engine_core::query::card_use_unlimited_for_card(
        tx.state(),
        owner,
        jiu
    ));
    assert!(!engine_core::query::card_use_unlimited_for_card(
        tx.state(),
        owner,
        sha
    ));
    assert_eq!(tx.state().cards.get(jiu).unwrap().kind, CardKind::Jiu);
}

#[test]
fn starzhonggu_is_lord_only_and_compares_round_to_alive_qun_count() {
    let mut package = content_compiler::compile_source_named(
        "sp2-starzhonggu",
        include_str!("../../../content/original/ported/sp2/starzhonggu.skill"),
    )
    .unwrap();
    package.character_index = [9_211_001, 9_211_002]
        .into_iter()
        .map(|id| PackageCharacterEntry {
            id,
            name_key: format!("qun_{id}"),
            max_hp: 4,
            initial_hp: 0,
            skills: vec![],
            sex: "male".into(),
            group: "qun".into(),
            is_lord: false,
            clans: vec![],
            tags: vec![],
            sort_key: String::new(),
            portrait_asset: None,
            skin_assets: vec![],
        })
        .collect();
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_711),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_711,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let lord = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.identity == Identity::Lord)
        .unwrap()
        .id;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != lord)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().players.get_mut(lord).unwrap().character_def = 9_211_001;
    tx.state_mut().players.get_mut(other).unwrap().character_def = 9_211_002;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: lord,
                skill_def: SkillDefId::new(STARZHONGGU),
            },
            EffectRequest::AddSkill {
                player: other,
                skill_def: SkillDefId::new(STARZHONGGU),
            },
        ],
    )
    .unwrap();

    tx.state_mut().turn.round = 1;
    assert_eq!(engine_core::query::draw_count_for(tx.state(), lord), 1);
    assert_eq!(engine_core::query::draw_count_for(tx.state(), other), 2);

    tx.state_mut().turn.round = 2;
    assert_eq!(engine_core::query::draw_count_for(tx.state(), lord), 4);

    tx.state_mut().turn.round = 1;
    tx.state_mut().players.get_mut(other).unwrap().alive = false;
    assert_eq!(engine_core::query::draw_count_for(tx.state(), lord), 4);
}

#[test]
fn reliewei_tracks_the_hp_limit_but_is_unlimited_during_the_owners_turn() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
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
        skill_def: SkillDefId::new(RELIEWEI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    tx.state_mut().turn.current = other;
    let hand = tx.state().hand_count(owner);

    for key in 10..12 {
        let paused = dispatch_for_events(
            &mut tx,
            &[DomainEvent::EnteredDying {
                player: other,
                source: owner,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err();
        assert_eq!(paused, engine_core::effect::EffectError::ChoiceRequested);
        tx = accept_optional(tx, owner, key);
    }
    assert_eq!(tx.state().hand_count(owner), hand + 2);
    dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: other,
            source: owner,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());

    tx.state_mut().turn.current = owner;
    let paused = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: other,
            source: owner,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_err();
    assert_eq!(paused, engine_core::effect::EffectError::ChoiceRequested);
}

#[test]
fn starsujun_checks_the_remaining_hand_after_card_use() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
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
        skill_def: SkillDefId::new(STARSUJUN),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().hand_cards(owner);
    assert!(hand.len() >= 4 && hand.len() % 2 == 0);
    let half = hand.len() / 2;
    for card in &hand[..half] {
        tx.state_mut().cards.get_mut(*card).unwrap().kind = CardKind::Sha;
    }
    for card in &hand[half..] {
        tx.state_mut().cards.get_mut(*card).unwrap().kind = CardKind::TrickGuohe;
    }

    let paused = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: CardId::NONE,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_err();
    assert_eq!(paused, engine_core::effect::EffectError::ChoiceRequested);
    tx = accept_optional(tx, owner, 20);
    assert_eq!(tx.state().hand_count(owner), hand.len() as u32 + 2);

    let hand = tx.state().hand_cards(owner);
    for card in &hand {
        tx.state_mut().cards.get_mut(*card).unwrap().kind = CardKind::Sha;
    }
    tx.state_mut().cards.get_mut(hand[0]).unwrap().kind = CardKind::TrickGuohe;

    dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: CardId::NONE,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
}
