//! O2 全量合并批次 1（2026-10-14）：家族 collab 的 20 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_collab_dcfanzhuan {
//! Golden tests for COLLAB「反转」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const DCFANZHUAN: u32 = 2_353_467_697;
const DCNIYUN: u32 = 3_024_539_645;
const CHARACTER: u32 = 2_549_428_417;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

fn started_tx(seed: u64) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package).unwrap();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    // The standard roster includes choice-bearing characters (e.g. 夏侯惇 刚烈)
    // whose booted presence makes these tests roster-dependent; this test only
    // exercises DCFANZHUAN, so drop every booted skill.
    let booted_skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    if !booted_skills.is_empty() {
        run_effects(&mut tx, booted_skills).unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCFANZHUAN),
        }],
    )
    .unwrap();
    (tx, owner)
}

fn begin_turn(tx: &mut Transaction, current: PlayerId) -> Result<(), EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .map(|_| ())
}

fn resolve(tx: Transaction, accept: bool, key: u64) -> Transaction {
    let state = tx.state().clone();
    let observer = PlayerId::new(state.resolution.pending_observer);
    let (state, pipe) = apply_command(
        state,
        Command {
            actor: observer,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    Transaction::begin(state)
}

#[test]
fn package_contains_complete_initially_damaged_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CHARACTER)
        .expect("逆转之神 character entry");
    assert_eq!((character.initial_hp, character.max_hp), (4, 5));
    assert_eq!(character.skills, vec![DCFANZHUAN, DCNIYUN]);
}

#[test]
fn adjusts_both_directions_draws_delta_and_fires_once_per_round() {
    let (mut tx, owner) = started_tx(2_026_071_510);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.hp = 1;
        player.max_hp = 5;
    }
    let hand = tx.state().hand_count(owner);
    assert_eq!(
        begin_turn(&mut tx, other),
        Err(EffectError::ChoiceRequested)
    );
    let mut tx = resolve(tx, true, 1);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
    assert_eq!(tx.state().hand_count(owner), hand + 3);

    assert_eq!(begin_turn(&mut tx, owner), Ok(()));
    assert!(!tx.state().has_pending_choice());

    tx.state_mut().turn.round += 1;
    let hand = tx.state().hand_count(owner);
    assert_eq!(
        begin_turn(&mut tx, other),
        Err(EffectError::ChoiceRequested)
    );
    let tx = resolve(tx, true, 2);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 1);
    assert_eq!(tx.state().hand_count(owner), hand + 3);
}

#[test]
fn declining_does_not_consume_the_round_limit() {
    let (mut tx, owner) = started_tx(2_026_071_511);
    assert_eq!(
        begin_turn(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    let mut tx = resolve(tx, false, 1);
    assert_eq!(
        begin_turn(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
}
}

mod original_collab_dcjianxiong {
//! Golden for 联动经典曹操「奸雄」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DCJIANXIONG: u32 = 3_695_091_838;
const DC_CAOCAO: u32 = 4_106_722_512;
const SOURCE: &str = include_str!("../../../content/original/ported/collab/dcjianxiong.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

fn inject(state: &mut GameState, zone_kind: ZoneKind, def: u32) -> CardId {
    let zone = state.zones.find(zone_kind, PlayerId::NONE).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn mark_count(state: &GameState, owner: PlayerId) -> i32 {
    state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find_map(|(mark, count)| (*mark == DCJIANXIONG).then_some(*count))
        .unwrap_or(0)
}

fn damage_and_accept(
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    card: CardId,
    key: u64,
) -> GameState {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealCardDamage {
                source,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
                card,
                card_kind: CardKind::Sha.tag(),
                card_flags: 0,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, output) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_and_runtime_match_the_legacy_growth_and_card_gain_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let compiled = content_compiler::compile_source_named("dcjianxiong", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 694);
    assert_eq!(package.skill_index.len(), 156);
    assert_eq!(package.character_index.len(), 90);
    assert!(package
        .character_index
        .iter()
        .any(|character| character.id == DC_CAOCAO && character.skills == [DCJIANXIONG]));
    assert_eq!(
        package
            .modules
            .iter()
            .find(|module| module.name == compiled.modules[0].name)
            .unwrap()
            .logic_hash,
        compiled.modules[0].logic_hash
    );

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_722),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_722,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
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
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCJIANXIONG),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 100;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 100;
    let mut state = tx.commit().state;

    for index in 0..6 {
        let card = inject(&mut state, ZoneKind::Processing, 90_000 + index);
        state = damage_and_accept(state, owner, source, card, index as u64 + 1);
        assert!(state.hand_cards(owner).contains(&card));
        assert_eq!(mark_count(&state, owner), (index + 1).min(4) as i32);
    }
    assert_eq!(state.hand_count(owner), 26);

    let discarded = inject(&mut state, ZoneKind::DiscardPile, 91_000);
    state = damage_and_accept(state, owner, source, discarded, 7);
    assert_eq!(state.hand_count(owner), 31);
    assert!(!state.hand_cards(owner).contains(&discarded));
    assert_eq!(mark_count(&state, owner), 4);

    state = damage_and_accept(state, owner, source, CardId::NONE, 8);
    assert_eq!(state.hand_count(owner), 36);
}
}

mod original_collab_dclonggong {
//! Golden for 联动「龙宫」：每回合一次防伤并令存活来源获得装备。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCLONGGONG: u32 = 3_209_573_631;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

fn prepared(seed: u64) -> (Transaction, PlayerId, PlayerId, CardId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: tx.state().turn.current,
            skill_def: SkillDefId::new(DCLONGGONG),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    let owner = tx.state().turn.current;
    let source = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.hp = 10;
    player.max_hp = 10;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let equipment = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_165_001),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 1,
        weapon_range: 2,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .push(equipment);
    (tx, owner, source, equipment)
}

fn resolve(tx: Transaction, owner: PlayerId, accept: bool, key: u64) -> GameState {
    let committed = tx.commit();
    let choice_id = ChoiceId::new(committed.state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn damage(tx: &mut Transaction, source: PlayerId, owner: PlayerId) -> Result<(), EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .map(|_| ())
}

#[test]
fn accepted_use_prevents_damage_grants_equipment_and_consumes_turn_limit() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == DCLONGGONG
            && trigger.event == "damage_before"
            && trigger.filter == "owner_is_target_source_alive_once"
            && trigger.optional
    }));

    let (mut tx, owner, source, equipment) = prepared(2_026_071_651);
    let hp = tx.state().players.get(owner).unwrap().hp;
    let source_hand = tx.state().hand_count(source);
    assert_eq!(
        damage(&mut tx, source, owner),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx, owner, true, 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert_eq!(state.hand_count(source), source_hand + 1);
    let hand = state.zones.find(ZoneKind::Hand, source).unwrap();
    assert!(state.zones.get(hand).unwrap().cards.contains(&equipment));

    let mut tx = Transaction::begin(state);
    damage(&mut tx, source, owner).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

#[test]
fn decline_keeps_the_use_and_source_free_damage_never_prompts() {
    let (mut tx, owner, source, _) = prepared(2_026_071_652);
    assert_eq!(
        damage(&mut tx, source, owner),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx, owner, false, 2);
    let hp = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        damage(&mut tx, source, owner),
        Err(EffectError::ChoiceRequested)
    );

    let (mut tx, owner, _, _) = prepared(2_026_071_653);
    damage(&mut tx, PlayerId::NONE, owner).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);
}
}

mod original_collab_dczhiheng {
//! Golden coverage for 制衡（collab 经典孙权，ABI 467）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag, ZoneId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCZHIHENG: u32 = 2_730_395_750;
const SOURCE: &str = include_str!("../../../content/original/ported/collab/dczhiheng.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn fire_play(tx: Transaction, player: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

/// Deterministic hands: empty both, seed owner with the given cards. Returns their ids.
fn seed_owner_hand(
    tx: &mut Transaction,
    owner: PlayerId,
    target: PlayerId,
    specs: &[(u32, CardKind, Suit, u8)],
) -> Vec<CardId> {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for (zone_kind, player) in [(ZoneKind::Hand, owner), (ZoneKind::Hand, target)] {
        let zone_id = tx.state().zones.find(zone_kind, player).unwrap();
        let cards: Vec<CardId> = tx.state().zones.get(zone_id).unwrap().cards.clone();
        let moves = cards
            .into_iter()
            .map(|card| EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(zone_id),
            })
            .collect();
        run_effects(tx, moves).unwrap();
    }
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let mut ids = Vec::new();
    for (def, kind, suit, rank) in specs {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(*def),
            kind: *kind,
            suit: *suit,
            rank: *rank,
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
            .push(card);
        ids.push(card);
    }
    ids
}

fn setup(tx: &mut Transaction, owner: PlayerId, target: PlayerId) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCZHIHENG),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
}

#[test]
fn discards_and_draws_equal_with_bonus_when_all_hand_discarded() {
    let package = content_compiler::compile_source_named("dczhiheng-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 467);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let ids = seed_owner_hand(
        &mut tx,
        owner,
        target,
        &[
            (96_000, CardKind::Sha, Suit::Spade, 5),
            (96_001, CardKind::Sha, Suit::Heart, 6),
            (96_002, CardKind::Tao, Suit::Club, 7),
            (96_003, CardKind::Tao, Suit::Diamond, 8),
        ],
    );
    assert_eq!(tx.state().hand_count(owner), 4);

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "discard choice pending");
    // Discard all four hand cards -> draw 4 + 1 bonus.
    let all = ids.iter().map(|id| id.raw()).collect();
    state = drive_choice(state, owner, all, false);
    assert_eq!(state.hand_count(owner), 5, "drew 4 + 1 bonus");
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        state.players.get(owner).unwrap().max_hp as i32,
        "no hp change"
    );
}

#[test]
fn no_bonus_when_hand_kept() {
    let package = content_compiler::compile_source_named("dczhiheng-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let ids = seed_owner_hand(
        &mut tx,
        owner,
        target,
        &[
            (96_000, CardKind::Sha, Suit::Spade, 5),
            (96_001, CardKind::Sha, Suit::Heart, 6),
        ],
    );
    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    // Discard only one of the two hand cards -> draw exactly 1.
    state = drive_choice(state, owner, vec![ids[0].raw()], false);
    assert_eq!(state.hand_count(owner), 2, "1 kept + 1 drawn");
}

#[test]
fn equip_cards_are_discardable() {
    let package = content_compiler::compile_source_named("dczhiheng-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let ids = seed_owner_hand(
        &mut tx,
        owner,
        target,
        &[
            (96_000, CardKind::Sha, Suit::Spade, 5),
            (96_001, CardKind::Weapon, Suit::Club, 7),
        ],
    );
    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    // A weapon card in the hand zone is a discardable hand-equip card: discarding it
    // (while keeping one hand card) draws exactly one.
    state = drive_choice(state, owner, vec![ids[1].raw()], false);
    assert_eq!(state.hand_count(owner), 2, "1 kept + 1 drawn");
}
}

mod original_collab_fuzhao {
//! Golden for 联动「福照」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const FUZHAO: u32 = 3_351_225_946;
const HONGYAN_FIXTURE: u32 = 4_290_000_100;
const SOURCE: &str = include_str!("../../../content/original/ported/collab/fuzhao.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

fn start() -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_718),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 20_260_718,
            ruleset: "identity-standard".into(),
        },
    }
}

fn inject_judgment(tx: &mut Transaction, suit: Suit) -> CardId {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(100_000_000 + suit as u32),
        kind: CardKind::Other,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(draw).unwrap().cards.push(card);
    card
}

fn isolated(
    package: &content_schema::ContentPackage,
    suit: Suit,
    add_hongyan: bool,
) -> (Transaction, PlayerId, PlayerId, CardId) {
    let mut engine = Engine::new();
    engine.load_package(package).unwrap();
    assert!(matches!(
        engine.submit(start()),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
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
        skill_def: SkillDefId::new(FUZHAO),
    });
    if add_hongyan {
        effects.push(EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(HONGYAN_FIXTURE),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    let target_state = tx.state_mut().players.get_mut(target).unwrap();
    target_state.hp = 0;
    target_state.dying = true;
    let judgment = inject_judgment(&mut tx, suit);
    (tx, owner, target, judgment)
}

fn offer(tx: &mut Transaction, owner: PlayerId, target: PlayerId) {
    assert_eq!(
        dispatch_for_events(
            tx,
            &[DomainEvent::EnteredDying {
                player: target,
                source: PlayerId::NONE,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(tx.state().resolution.pending_observer, owner.raw());
}

fn resolve(
    tx: &Transaction,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> (engine_core::state::GameState, Vec<DomainEvent>) {
    let (state, output) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    (state, output.events)
}

fn in_discard(state: &engine_core::state::GameState, card: CardId) -> bool {
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    state.zones.get(discard).unwrap().cards.contains(&card)
}

#[test]
fn committed_package_contains_the_optional_rewrite() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let compiled = content_compiler::compile_source_named("fuzhao", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
    assert_eq!(committed.manifest.rule_vm_abi_version, 694);
    assert_eq!(
        (
            compiled.trigger_index[0].event.as_str(),
            compiled.trigger_index[0].filter.as_str(),
            compiled.trigger_index[0].optional,
        ),
        ("dying", "owner_alive", true)
    );
}

#[test]
fn heart_judgment_recovers_the_dying_player_with_the_owner_as_source() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let (mut tx, owner, target, judgment) = isolated(&package, Suit::Heart, false);
    offer(&mut tx, owner, target);
    let (state, events) = resolve(&tx, owner, true, 1);

    assert_eq!(state.players.get(target).unwrap().hp, 1);
    assert!(!state.players.get(target).unwrap().dying);
    assert!(in_discard(&state, judgment));
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::HpRecovered {
            source,
            player,
            amount: 1,
        } if *source == owner && *player == target
    )));
}

#[test]
fn non_heart_judgment_does_not_recover_and_declining_does_not_judge() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let (mut tx, owner, target, judgment) = isolated(&package, Suit::Club, false);
    offer(&mut tx, owner, target);
    let (state, _) = resolve(&tx, owner, true, 2);
    assert_eq!(state.players.get(target).unwrap().hp, 0);
    assert!(state.players.get(target).unwrap().dying);
    assert!(in_discard(&state, judgment));

    let (mut tx, owner, target, judgment) = isolated(&package, Suit::Heart, false);
    offer(&mut tx, owner, target);
    let (state, _) = resolve(&tx, owner, false, 3);
    assert_eq!(state.players.get(target).unwrap().hp, 0);
    assert!(state.players.get(target).unwrap().dying);
    assert!(!in_discard(&state, judgment));
}

#[test]
fn judgment_uses_the_judged_players_effective_suit() {
    let fixture = content_compiler::compile_source_named(
        "fuzhao-effective-suit",
        &format!(
            "{SOURCE}\nskill fixture_hongyan({HONGYAN_FIXTURE}) locked {{ modifier card_suit(spade) override heart; }}"
        ),
    )
    .unwrap();
    let (mut tx, owner, target, judgment) = isolated(&fixture, Suit::Spade, true);
    offer(&mut tx, owner, target);
    let (state, _) = resolve(&tx, owner, true, 4);

    assert_eq!(state.players.get(target).unwrap().hp, 1);
    assert!(!state.players.get(target).unwrap().dying);
    assert!(in_discard(&state, judgment));
}
}

mod original_collab_haoxue {
//! 第一百四十七批：collab 好学吕蒙「好学」（haoxue）零新原语真实语义重写。
//! 上游：出牌阶段内获得牌后回复 1 点体力（需已受伤）；出牌阶段外弃置牌后失去 1 点体力。
//! 实现：cards_moved 双臂 + owner_gained_cards_during_play_phase_wounded /
//! owner_discarded_outside_play_phase（本批注册过滤器）。

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const HAOXUE: u32 = 4_229_967_180;
const HAOXUE_SRC: &str = include_str!("../../../content/original/ported/collab/haoxue.skill");

fn bootstrap() -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("haoxue", HAOXUE_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(HAOXUE), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HAOXUE),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn hp_of(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state().players.get(player).unwrap().hp
}

fn max_hp_of(tx: &Transaction, player: PlayerId) -> u32 {
    tx.state().players.get(player).unwrap().max_hp
}

#[test]
fn haoxue_recovers_once_when_gaining_during_own_play_phase() {
    let (mut tx, owner, _other) = bootstrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let max_hp = max_hp_of(&tx, owner);
    tx.state_mut().players.get_mut(owner).unwrap().hp = max_hp as i32 - 2;
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 2,
        }],
    )
    .unwrap();
    assert_eq!(
        hp_of(&tx, owner),
        max_hp as i32 - 1,
        "one gain event must recover exactly 1 (not per card)"
    );
}

#[test]
fn haoxue_no_recover_when_full_hp() {
    let (mut tx, owner, _other) = bootstrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let before = hp_of(&tx, owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(hp_of(&tx, owner), before, "full-HP gains must not recover");
}

#[test]
fn haoxue_loses_hp_when_discarding_outside_own_play_phase() {
    let (mut tx, owner, other) = bootstrap();
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 2,
        }],
    )
    .unwrap();
    let before = hp_of(&tx, owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(
        hp_of(&tx, owner),
        before - 1,
        "discard outside own play phase must cost 1 HP"
    );
}

#[test]
fn haoxue_no_loss_when_discarding_during_own_play_phase() {
    let (mut tx, owner, _other) = bootstrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 2,
        }],
    )
    .unwrap();
    let before = hp_of(&tx, owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(
        hp_of(&tx, owner),
        before,
        "discard during own play phase must not cost HP"
    );
}
}

mod original_collab_mount_skills {
//! Golden for 联动包「骏魁」「骥冠」的全局坐骑清理与常驻修正。

use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, ZoneKind};
use engine_core::Engine;
use rule_api::{QueryRequest, QueryResponse};

const MBJUNKUI: u32 = 3_131_135_455;
const MBJIGUAN: u32 = 2_860_585_832;
const COLLAB: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

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
}

mod original_collab_natuxing_gate {
//! §8 S1（第一百一十八批）：`owner_source_hand_gaintag_absent_<tag>` 参数化
//! 过滤器——natuxing 兴门：手中无锁定兴 tag 牌时用牌不可被响应。
//! 兴 tag = blake2b("natuxing") = 297116519。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NATUXING: u32 = 4_091_854_877;
const XING_TAG: u32 = 297_116_519;

const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/collab"
);

fn sources() -> String {
    let dir = std::path::Path::new(SOURCES_DIR);
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "skill"))
        .map(|e| e.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|p| {
            let mut s = std::fs::read_to_string(p).unwrap();
            s.push('\n');
            s
        })
        .collect()
}


fn drive(
    tx: &mut Transaction,
    events: &[engine_api::DomainEvent],
) -> Result<(), EffectError> {
    match engine_core::trigger::dispatch_for_events(
        tx,
        events,
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Err(e) => Err(e),
        Ok(effects) => run_effects(tx, effects).map(|_| ()),
    }
}

fn started() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("collab-natuxing", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_838, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NATUXING),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

fn inject_card(
    tx: &mut Transaction,
    owner: PlayerId,
    tag: Option<u32>,
) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: engine_core::state::CardKind::Tao,
        suit: engine_api::Suit::Heart,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    if let Some(tag) = tag {
        tx.state_mut()
            .content_card_tags
            .entry(card)
            .or_default()
            .insert(tag);
    }
    card
}

fn drive_use_card(tx: &mut Transaction, holder: PlayerId, card: CardId) -> Result<(), EffectError> {
    match engine_core::trigger::dispatch_for_events(
        tx,
        &[engine_api::DomainEvent::CardUsed {
            player: holder,
            card,
            targets: vec![],
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Err(e) => Err(e),
        Ok(effects) => run_effects(tx, effects).map(|_| ()),
    }
}

#[test]
fn natuxing_unrespondable_gated_by_gaintag_absent() {
    let (state, holder) = started();
    let mut tx = Transaction::begin(state);

    // 手牌含兴 tag 牌 → 门关闭（无 unrespondable 效果）。
    let tagged = inject_card(&mut tx, holder, Some(XING_TAG));
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: holder,
            card: tagged,
            targets: vec![],
        }],
    );
    opened.unwrap();
    assert!(!tx.state().has_pending_choice(), "gate closed while tagged");
    let mut committed = tx.commit();

    // 手牌无兴 tag 牌 → 门开（不可被响应效果）。
    // 先移除场景一的兴 tag 牌（模拟其已被使用消耗）。
    {
        let hand = committed.state.zones.find(ZoneKind::Hand, holder).unwrap();
        committed
            .state
            .zones
            .get_mut(hand)
            .unwrap()
            .cards
            .retain(|&c| c != tagged);
    }
    let mut tx = Transaction::begin(committed.state);
    let plain = inject_card(&mut tx, holder, None);
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: holder,
            card: plain,
            targets: vec![],
        }],
    );
    opened.unwrap();
    let committed2 = tx.commit();
    let sid2 = committed2
        .state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == NATUXING)
        .unwrap()
        .id;
    let policy_armed = committed2
        .state
        .skills
        .sorted_live()
        .iter()
        .filter(|s| s.def.raw() == NATUXING)
        .any(|s| s.card_use_policy(plain) & 2 != 0);
    assert!(policy_armed, "gate open: unrespondable armed");
}
}

mod original_collab_olchenlong {
//! Golden coverage for COLLAB「辰龙」and the complete ol_chenlong closure.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLCHENLONG: u32 = 2_779_773_800;
const OL_CHENLONG: u32 = 3_917_415_586;
const SOURCE: &str = include_str!("../../../content/original/ported/collab/olchenlong.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

fn inject_tao(state: &mut GameState, owner: PlayerId) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(108_001),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank: 3,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn neutralize_existing_save_cards(state: &mut GameState) {
    let players = state.players.alive_by_seat();
    for player in players {
        let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
        let cards = state.zones.get(hand).unwrap().cards.clone();
        for card in cards {
            let physical = state.cards.get_mut(card).unwrap();
            if matches!(physical.kind, CardKind::Tao | CardKind::Jiu) {
                physical.kind = CardKind::Sha;
            }
        }
    }
}

fn setup() -> (GameState, PlayerId, PlayerId, engine_api::SkillId) {
    let package = content_compiler::compile_source_named("olchenlong-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, OLCHENLONG as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
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
            skill_def: SkillDefId::new(OLCHENLONG),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = *tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .last()
        .unwrap();
    (tx.commit().state, owner, target, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    choice: engine_api::ChoiceId,
    token: u32,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![token],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_numeric_active_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 694);
    assert!(package.manifest.capabilities.request_choice);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLCHENLONG && skill.name == "olchenlong"));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == OLCHENLONG)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_CHENLONG
            && character.skills == vec![OLCHENLONG]
            && character.max_hp == 4
            && character.sex == "male"
            && character.group == "qun"
    }));
}

#[test]
fn nonlethal_choice_loses_and_deals_the_same_amount_without_max_hp_cost() {
    let (mut state, owner, target, skill) = setup();
    state.players.get_mut(owner).unwrap().hp = 4;
    state.players.get_mut(target).unwrap().hp = 4;
    let owner_max = state.players.get(owner).unwrap().max_hp;

    let (waiting, activated) = activate(state, owner, target, skill, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.expect("1/2 amount choice");
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );

    let (state, resolved) = resolve(waiting, owner, choice.choice_id, 2, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
    assert_eq!(state.players.get(owner).unwrap().max_hp, owner_max);
    assert_eq!(state.players.get(target).unwrap().hp, 2);
    assert!(resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target: damaged,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        } if *source == owner && *damaged == target
    )));
}

#[test]
fn exact_self_loss_dying_reduces_max_hp_and_defers_target_damage_until_saved() {
    let (mut state, owner, target, skill) = setup();
    state.players.get_mut(owner).unwrap().hp = 1;
    state.players.get_mut(target).unwrap().hp = 4;
    let owner_max = state.players.get(owner).unwrap().max_hp;
    neutralize_existing_save_cards(&mut state);
    let tao = inject_tao(&mut state, target);

    let (waiting, activated) = activate(state, owner, target, skill, 10);
    let amount_choice = activated.choice.expect("amount choice");
    let (dying, amount_resolved) = resolve(waiting, owner, amount_choice.choice_id, 1, 11);
    assert!(matches!(
        amount_resolved.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(dying.players.get(owner).unwrap().hp, 0);
    assert!(dying.players.get(owner).unwrap().dying);
    assert_eq!(dying.players.get(owner).unwrap().max_hp, owner_max - 1);
    assert_eq!(
        dying.players.get(target).unwrap().hp,
        4,
        "target damage must wait behind the dying barrier"
    );
    assert!(amount_resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::MaxHpChanged { player, new_max }
            if *player == owner && *new_max == owner_max - 1
    )));

    let save = amount_resolved.choice.expect("dying save choice");
    assert_eq!(save.observer, target);
    let (state, saved) = resolve(dying, target, save.choice_id, 1, 12);
    assert!(matches!(saved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, 1);
    assert!(!state.players.get(owner).unwrap().dying);
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert_eq!(
        state.cards.get(tao).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}
}

mod original_collab_olchouniu {
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
        "647cdf8372e43d26f75d06f97c9a1909a081ba1e25e4a49d6847e85d09c7b117"
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
}

mod original_collab_olhaizhu {
//! Golden coverage for COLLAB「亥猪」rewritten from legacy JavaScript semantics.

use engine_api::{CardDefId, CardId, DomainEvent, PlayerId, SkillDefId, Suit, TurnPhaseTag};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLHAIZHU: u32 = 2_513_379_804;
const OL_HAIZHU: u32 = 3_508_419_110;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: Vec<PlayerId>,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().turn.current;
    let others = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLHAIZHU),
    });
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        others,
    }
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    suit: Suit,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn dispatch(tx: &mut Transaction, events: &[DomainEvent]) {
    let effects = dispatch_for_events(tx, events, &mut TriggerFireRecord::new()).unwrap();
    run_effects(tx, effects).unwrap();
}

#[test]
fn package_retains_abi112_skill_and_character_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 694);
    assert_eq!(package.skill_index.len(), 156);
    assert_eq!(package.character_index.len(), 90);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLHAIZHU)
        .collect();
    assert!(triggers.iter().any(|trigger| {
        trigger.event == "cards_moved"
            && trigger.filter == "owner_alive_other_discarded_black_to_discard"
    }));
    assert!(triggers.iter().any(|trigger| {
        trigger.event == "phase_prepare" && trigger.filter == "owner_is_target_tied_max_hand"
    }));
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_HAIZHU
            && character.name_key == "ol_haizhu"
            && character.sex == "male"
            && character.group == "qun"
            && character.max_hp == 5
            && character.skills == [OLHAIZHU]
    }));
}

#[test]
fn gains_only_other_players_black_cards_from_the_exact_discard_batch() {
    let mut setup = setup(2_026_072_312);
    let other = setup.others[0];
    let black = inject(&mut setup.state, other, ZoneKind::Hand, 99_201, Suit::Spade);
    let red = inject(&mut setup.state, other, ZoneKind::Hand, 99_202, Suit::Heart);
    let mut tx = Transaction::begin(setup.state);
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: other,
            cards: vec![black, red],
        }],
    )
    .unwrap();
    dispatch(&mut tx, &events);

    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(tx.state().cards.get(black).unwrap().zone, owner_hand);
    assert_eq!(tx.state().cards.get(black).unwrap().owner, setup.owner);
    assert_eq!(tx.state().cards.get(red).unwrap().zone, discard);
}

#[test]
fn own_discards_and_non_discard_movements_are_not_reclaimed() {
    let mut setup = setup(2_026_072_313);
    let own_black = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_211,
        Suit::Club,
    );
    let other_black = inject(
        &mut setup.state,
        setup.others[0],
        ZoneKind::Hand,
        99_212,
        Suit::Spade,
    );
    let mut tx = Transaction::begin(setup.state);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: setup.owner,
            cards: vec![own_black],
        }],
    )
    .unwrap();
    dispatch(&mut tx, &events);
    assert_eq!(tx.state().cards.get(own_black).unwrap().zone, discard);

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![other_black],
            to: discard,
            from: None,
        }],
    )
    .unwrap();
    dispatch(&mut tx, &events);
    assert_eq!(tx.state().cards.get(other_black).unwrap().zone, discard);
}

#[test]
fn preparation_hp_loss_counts_ties_but_not_a_smaller_hand() {
    let tied = setup(2_026_072_314);
    let before = tied.state.players.get(tied.owner).unwrap().hp;
    let mut tx = Transaction::begin(tied.state);
    dispatch(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: tied.owner,
            phase: TurnPhaseTag::Prepare,
        }],
    );
    assert_eq!(tx.state().players.get(tied.owner).unwrap().hp, before - 1);

    let mut setup = setup(2_026_072_315);
    let other = setup.others[0];
    let needed = setup
        .state
        .hand_count(setup.owner)
        .saturating_sub(setup.state.hand_count(other))
        + 1;
    for offset in 0..needed {
        inject(
            &mut setup.state,
            other,
            ZoneKind::Hand,
            99_221 + offset,
            Suit::Heart,
        );
    }
    let before = setup.state.players.get(setup.owner).unwrap().hp;
    let mut tx = Transaction::begin(setup.state);
    dispatch(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: TurnPhaseTag::Prepare,
        }],
    );
    assert_eq!(tx.state().players.get(setup.owner).unwrap().hp, before);
}
}

mod original_collab_olmaotu {
//! Golden coverage for COLLAB「卯兔」and the complete ol_maotu closure.

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const OLMAOTU: u32 = 2_889_501_326;
const OLMAOTU_EFFECT: u32 = 3_563_166_419;
const OL_MAOTU: u32 = 4_111_734_089;
const SOURCE: &str = include_str!("../../../content/original/ported/collab/olmaotu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

fn inject_juedou(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::TrickJuedou,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn target_legal(tx: &Transaction, user: PlayerId, card: CardId, target: PlayerId) -> bool {
    match query(
        tx.state(),
        &QueryRequest::TargetLegal { user, card, target },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Bool(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId, CardId, CardId) {
    let package = content_compiler::compile_source_named("olmaotu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, OLMAOTU as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let higher = players[1];
    let lower = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLMAOTU),
    });
    effects.push(EffectRequest::SetTurnState {
        current: higher,
        phase: Phase::Play.to_u8(),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    tx.state_mut().players.get_mut(higher).unwrap().hp = 3;
    tx.state_mut().players.get_mut(lower).unwrap().hp = 1;
    let high_card = inject_juedou(&mut tx, higher, 107_001);
    let low_card = inject_juedou(&mut tx, lower, 107_002);
    (tx, owner, higher, lower, high_card, low_card)
}

#[test]
fn package_publishes_abi107_helper_modifier_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 694);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLMAOTU && skill.name == "olmaotu"));
    assert!(package
        .skill_index
        .iter()
        .any(|skill| { skill.skill_def_id == OLMAOTU_EFFECT && skill.name == "olmaotu_effect" }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == OLMAOTU_EFFECT
            && modifier.target == "card_target_cancel_if_other_user_hp_gte_target"
            && modifier.value == 1
    }));
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_MAOTU
            && character.skills == vec![OLMAOTU]
            && character.max_hp == 3
            && character.sex == "female"
            && character.group == "qun"
    }));
}

#[test]
fn leaving_dying_grants_dynamic_hp_based_target_protection() {
    let (mut tx, owner, higher, lower, high_card, low_card) = isolated();
    assert!(target_legal(&tx, higher, high_card, owner));
    assert!(target_legal(&tx, lower, low_card, owner));

    {
        let player = tx.state_mut().players.get_mut(lower).unwrap();
        player.hp = 0;
        player.dying = true;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: lower,
            player: lower,
            amount: 1,
        }],
    )
    .unwrap();

    assert!(has_skill(tx.state(), owner, OLMAOTU_EFFECT));
    assert!(!target_legal(&tx, higher, high_card, owner));
    assert!(target_legal(&tx, lower, low_card, owner));

    tx.state_mut().players.get_mut(higher).unwrap().hp = 1;
    assert!(target_legal(&tx, higher, high_card, owner));
    tx.state_mut().players.get_mut(lower).unwrap().hp = 2;
    assert!(!target_legal(&tx, lower, low_card, owner));
}

#[test]
fn protection_survives_other_turns_and_expires_at_the_owners_next_turn_start() {
    let (mut tx, owner, higher, lower, high_card, _) = isolated();
    {
        let player = tx.state_mut().players.get_mut(lower).unwrap();
        player.hp = 0;
        player.dying = true;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: lower,
            player: lower,
            amount: 1,
        }],
    )
    .unwrap();
    assert!(!target_legal(&tx, higher, high_card, owner));

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: higher,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();

    assert!(!has_skill(tx.state(), owner, OLMAOTU_EFFECT));
    assert!(target_legal(&tx, higher, high_card, owner));
}
}

mod original_collab_olshenhou {
//! Golden tests for 联动包「申猴」rewritten from the legacy JavaScript semantics.

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

const COLLAB: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");
const SOURCE: &str = include_str!("../../../content/original/ported/collab/olshenhou.skill");
const OL_SHENHOU: u32 = 2_433_370_010;
const OLSHENHOU: u32 = 2_393_323_694;
const MULTI_SHA: u32 = 4_290_000_066;

fn start(seed: u64, player_count: u8) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

fn loaded(package: &content_schema::ContentPackage, seed: u64) -> (Transaction, [PlayerId; 3]) {
    let mut engine = Engine::new();
    engine.load_package(package).unwrap();
    assert!(matches!(
        engine.submit(start(seed, 3)),
        CommandResult::Accepted { .. }
    ));
    let source = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let source_index = seats.iter().position(|player| *player == source).unwrap();
    let players = [
        source,
        seats[(source_index + 1) % seats.len()],
        seats[(source_index + 2) % seats.len()],
    ];
    let mut tx = Transaction::begin(engine.state().clone());
    let remove = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    for player in players {
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
    (tx, players)
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

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(66_000 + kind.tag() as u32 * 20 + rank as u32),
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

fn inject_judgment(tx: &mut Transaction, suit: Suit) -> CardId {
    inject(
        tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        suit,
        7,
    )
}

fn play_and_confirm(
    tx: &Transaction,
    source: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    owner: PlayerId,
) -> engine_core::state::GameState {
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let choice = pipeline.choice.unwrap();
    assert_eq!(choice.observer, owner);
    let (state, pipeline) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipeline.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipeline.result
    );
    state
}

#[test]
fn committed_package_contains_the_complete_rewritten_character() {
    let committed = content_schema::ContentPackage::from_json_bytes(COLLAB).unwrap();
    let compiled = content_compiler::compile_source_named("olshenhou", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
    assert_eq!(
        compiled.trigger_index[0].filter,
        "owner_targeted_by_other_sha"
    );
    assert!(compiled.trigger_index[0].optional);

    let character = committed
        .character_index
        .iter()
        .find(|character| character.id == OL_SHENHOU)
        .unwrap();
    assert_eq!((character.initial_hp, character.max_hp), (0, 3));
    assert_eq!(character.skills, vec![OLSHENHOU]);
}

fn assert_single_target_judgment(suit: Suit, damage: i32, seed: u64) {
    let package = content_schema::ContentPackage::from_json_bytes(COLLAB).unwrap();
    let (mut tx, [source, owner, _]) = loaded(&package, seed);
    add_skill(&mut tx, owner, OLSHENHOU);
    let judgment = inject_judgment(&mut tx, suit);
    let sha = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        8,
    );
    let before = tx.state().players.get(owner).unwrap().hp;

    let state = play_and_confirm(&tx, source, sha, vec![owner], owner);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, before - damage);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&judgment));
}

#[test]
fn red_judgment_excludes_only_the_current_sha_target() {
    assert_single_target_judgment(Suit::Heart, 0, 20_260_715);

    let fixture = content_compiler::compile_source_named(
        "olshenhou-multi",
        &format!("{SOURCE}\nskill multi_sha({MULTI_SHA}) {{ modifier sha_target_limit add 1; }}"),
    )
    .unwrap();
    let (mut tx, [source, owner, other]) = loaded(&fixture, 20_260_716);
    add_skill(&mut tx, owner, OLSHENHOU);
    add_skill(&mut tx, source, MULTI_SHA);
    inject_judgment(&mut tx, Suit::Diamond);
    let sha = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        9,
    );
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let other_hp = tx.state().players.get(other).unwrap().hp;

    let state = play_and_confirm(&tx, source, sha, vec![owner, other], owner);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp - 1);
}

#[test]
fn black_judgment_keeps_the_sha_effective() {
    assert_single_target_judgment(Suit::Spade, 1, 20_260_717);
}
}

mod original_collab_olsishe {
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
}

mod original_collab_olweiyang {
//! Golden coverage for COLLAB「未羊」and the complete ol_weiyang closure.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLWEIYANG: u32 = 4_199_352_428;
const OL_WEIYANG: u32 = 2_376_598_041;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, OLWEIYANG as u64, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let first = players[1];
    let second = players[2];
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
            skill_def: SkillDefId::new(OLWEIYANG),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = *tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .last()
        .unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, first, second, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
    cards: Vec<engine_api::CardId>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: cards,
            },
        },
    )
}

#[test]
fn package_publishes_abi109_matched_selection_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 694);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLWEIYANG && skill.name == "olweiyang"));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == OLWEIYANG)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (1, 3));
    assert!(targets.allow_self);
    assert!(targets.wounded);
    assert!(targets.match_selected_card_count);
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(
        (selection.zone.as_str(), selection.min, selection.max),
        ("hand_equip", 1, 3)
    );
    assert!(selection.distinct_categories);
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_WEIYANG
            && character.skills == vec![OLWEIYANG]
            && character.max_hp == 3
            && character.sex == "female"
            && character.group == "qun"
    }));
}

#[test]
fn two_wounded_targets_require_two_distinct_categories_then_both_recover() {
    let (mut state, owner, first, _second, skill) = setup();
    state.players.get_mut(owner).unwrap().hp = 2;
    state.players.get_mut(first).unwrap().hp = 2;
    let cards = state.hand_cards(owner)[..2].to_vec();
    state.cards.get_mut(cards[0]).unwrap().kind = CardKind::Sha;
    state.cards.get_mut(cards[1]).unwrap().kind = CardKind::TrickWuzhong;

    let (waiting, activated) = activate(state, owner, skill, vec![owner, first], Vec::new(), 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.expect("two-card cost choice");
    assert_eq!((choice.min_count, choice.max_count), (2, 2));
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == cards[0].raw()));
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == cards[1].raw()));

    let revision = waiting.revision;
    let (state, resolved) = apply_command(
        waiting,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: cards.iter().map(|card| card.raw()).collect(),
                cancelled: false,
            },
        },
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert_eq!(state.players.get(first).unwrap().hp, 3);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(cards
        .iter()
        .all(|card| state.cards.get(*card).unwrap().zone == discard));
}

#[test]
fn duplicate_categories_healthy_targets_and_mismatched_counts_are_atomic_rejections() {
    let (mut state, owner, first, second, skill) = setup();
    state.players.get_mut(owner).unwrap().hp = 2;
    state.players.get_mut(first).unwrap().hp = 2;
    state.players.get_mut(second).unwrap().hp = 2;
    let cards = state.hand_cards(owner)[..3].to_vec();
    state.cards.get_mut(cards[0]).unwrap().kind = CardKind::Sha;
    state.cards.get_mut(cards[1]).unwrap().kind = CardKind::Tao;
    state.cards.get_mut(cards[2]).unwrap().kind = CardKind::Weapon;

    let before = canonical_state_hash(&state);
    let (mut state, result) = activate(
        state,
        owner,
        skill,
        vec![owner, first],
        cards[..2].to_vec(),
        10,
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    let first_max_hp = state.players.get(first).unwrap().max_hp as i32;
    state.players.get_mut(first).unwrap().hp = first_max_hp;
    let before = canonical_state_hash(&state);
    let (mut state, result) = activate(
        state,
        owner,
        skill,
        vec![owner, first],
        vec![cards[0], cards[2]],
        11,
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    state.players.get_mut(first).unwrap().hp = 2;
    let before = canonical_state_hash(&state);
    let (state, result) = activate(
        state,
        owner,
        skill,
        vec![owner, first, second],
        vec![cards[0], cards[2]],
        12,
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
}
}

mod original_collab_olwuma {
//! Golden coverage for COLLAB「午马」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLWUMA: u32 = 3_608_420_255;
const OL_WUMA: u32 = 3_167_375_132;
const FORCE_SKIP: u32 = 4_294_967_000;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: Vec<PlayerId>,
}

fn setup(seed: u64, force_skip: bool) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    if force_skip {
        let helper = content_compiler::compile_source_named(
            "olwuma-test-force-skip",
            r#"
skill force_skip(4294967000) locked {
  modifier skip_judge_phase override 1;
  modifier skip_discard_phase override 1;
}
"#,
        )
        .unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&helper).unwrap();
    }
    let owner = tx.state().turn.current;
    let others = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLWUMA),
    });
    if force_skip {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FORCE_SKIP),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        others,
    }
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::EndPhase,
        },
    )
}

fn dispatch_card_used(tx: &mut Transaction, user: PlayerId, card: CardId, targets: Vec<PlayerId>) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}

#[test]
fn package_retains_olwuma_skill_and_character_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 694);
    assert_eq!(package.skill_index.len(), 156);
    assert_eq!(package.character_index.len(), 90);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLWUMA)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(trigger.filter, "owner_alive_targeted_by_other_trick");
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == OLWUMA && modifier.target == "turn_face_down_cancel"
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == OLWUMA && modifier.target == "restore_skipped_phase"
    }));
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_WUMA
            && character.name_key == "ol_wuma"
            && character.sex == "male"
            && character.group == "qun"
            && character.max_hp == 4
            && character.skills == [OLWUMA]
    }));
}

#[test]
fn another_players_trick_draws_once_even_when_it_has_multiple_targets() {
    let mut setup = setup(2_026_072_306, false);
    let source = setup.others[0];
    let other = setup.others[1];
    let trick = inject(
        &mut setup.state,
        source,
        ZoneKind::Hand,
        99_101,
        CardKind::TrickWuzhong,
    );
    let before = setup.state.hand_count(setup.owner);
    let mut tx = Transaction::begin(setup.state);
    dispatch_card_used(&mut tx, source, trick, vec![setup.owner, source, other]);
    assert_eq!(tx.state().hand_count(setup.owner), before + 1);

    let sha = inject(
        tx.state_mut(),
        source,
        ZoneKind::Hand,
        99_102,
        CardKind::Sha,
    );
    dispatch_card_used(&mut tx, source, sha, vec![setup.owner]);
    assert_eq!(tx.state().hand_count(setup.owner), before + 1);

    let own_trick = inject(
        tx.state_mut(),
        setup.owner,
        ZoneKind::Hand,
        99_103,
        CardKind::TrickJuedou,
    );
    let before_own_use = tx.state().hand_count(setup.owner);
    dispatch_card_used(&mut tx, setup.owner, own_trick, vec![other]);
    assert_eq!(tx.state().hand_count(setup.owner), before_own_use);
}

#[test]
fn face_up_cannot_turn_face_down_but_existing_face_down_state_can_clear() {
    let setup = setup(2_026_072_307, false);
    let mut tx = Transaction::begin(setup.state);
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::TurnOver {
            player: setup.owner,
        }],
    )
    .unwrap();
    assert!(events.is_empty());
    assert!(!tx.state().players.get(setup.owner).unwrap().turned_over);

    tx.state_mut()
        .players
        .get_mut(setup.owner)
        .unwrap()
        .turned_over = true;
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::TurnOver {
            player: setup.owner,
        }],
    )
    .unwrap();
    assert!(!tx.state().players.get(setup.owner).unwrap().turned_over);
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::PlayerTurnedOver {
            player,
            turned_over: false,
        } if *player == setup.owner
    )));
}

#[test]
fn draw_and_play_skip_flags_record_events_but_restore_the_exact_phases() {
    let mut setup = setup(2_026_072_308, false);
    setup.state.turn.current = setup.owner;
    setup.state.turn.phase = Phase::Judge;
    setup.state.turn.set_flag(turn_flag::SKIP_DRAW, 1);
    let before = setup.state.hand_count(setup.owner);
    let (state, draw) = submit(setup.state, setup.owner, 1);
    assert!(matches!(draw.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(state.hand_count(setup.owner), before + 2);
    assert!(draw.events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Draw,
        } if *current == setup.owner
    )));

    let mut state = state;
    state.turn.set_flag(turn_flag::SKIP_PLAY, 1);
    let (state, play) = submit(state, setup.owner, 2);
    assert!(matches!(play.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Play);
    assert!(play.events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Play,
        } if *current == setup.owner
    )));
}

#[test]
fn judgment_and_discard_skip_modifiers_still_execute_the_owner_phases() {
    let mut judge = setup(2_026_072_309, true);
    let delayed = inject(
        &mut judge.state,
        judge.owner,
        ZoneKind::Judge,
        99_111,
        CardKind::TrickBingliang,
    );
    judge.state.turn.current = judge.owner;
    judge.state.turn.phase = Phase::Judge;
    let (state, result) = submit(judge.state, judge.owner, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let judge_zone = state.zones.find(ZoneKind::Judge, judge.owner).unwrap();
    assert!(!state
        .zones
        .get(judge_zone)
        .unwrap()
        .cards
        .contains(&delayed));
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Judge,
        } if *current == judge.owner
    )));

    let mut discard = setup(2_026_072_310, true);
    inject(
        &mut discard.state,
        discard.owner,
        ZoneKind::Hand,
        99_112,
        CardKind::Sha,
    );
    discard.state.turn.current = discard.owner;
    discard.state.turn.phase = Phase::Discard;
    let (state, result) = submit(discard.state, discard.owner, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Discard);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::DiscardToLimit
    );
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Discard,
        } if *current == discard.owner
    )));
}

#[test]
fn explicit_skip_current_phase_is_audited_without_advancing() {
    let mut setup = setup(2_026_072_311, false);
    for phase in [Phase::Draw, Phase::Play, Phase::Discard] {
        setup.state.turn.current = setup.owner;
        setup.state.turn.phase = phase;
        let mut tx = Transaction::begin(setup.state);
        let events = run_effects(
            &mut tx,
            vec![EffectRequest::SkipCurrentPhase {
                player: setup.owner,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().turn.phase, phase);
        assert!(events.iter().any(|event| matches!(
            event,
            DomainEvent::PhaseSkipped { current, phase: skipped }
                if *current == setup.owner && *skipped == phase.to_tag()
        )));
        setup.state = tx.commit().state;
    }
}
}

mod original_collab_olxugou {
//! Golden for 联动「虚构」：红色【杀】免疫、无距离限制且伤害 +1。

use engine_api::{
    Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const OLXUGOU: u32 = 2_803_220_890;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

fn target_legal(
    tx: &Transaction,
    user: PlayerId,
    card: engine_api::CardId,
    target: PlayerId,
) -> bool {
    match query(
        tx.state(),
        &QueryRequest::TargetLegal { user, card, target },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Bool(value) => value,
        other => panic!("unexpected target query: {other:?}"),
    }
}

#[test]
fn red_sha_is_blocked_incoming_and_boosted_outgoing() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == OLXUGOU)
        .expect("olxugou must be packaged");
    assert!(skill.locked);

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(2_803_220_890),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 2_803_220_890,
            ruleset: "identity-standard".into(),
        },
    });

    let seats = engine.state().players.alive_by_seat();
    let owner = seats[0];
    let attacker = seats[1];
    let distant = seats[2];
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLXUGOU),
        }],
    )
    .unwrap();

    let outgoing = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(outgoing).unwrap().kind = CardKind::Sha;
    tx.state_mut().cards.get_mut(outgoing).unwrap().suit = Suit::Heart;
    assert!(engine_core::query::card_target_within_distance(
        tx.state(),
        owner,
        outgoing,
        distant,
        1,
    ));
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, outgoing, 1),
        2
    );
    tx.state_mut().cards.get_mut(outgoing).unwrap().suit = Suit::Spade;
    assert!(!engine_core::query::card_target_within_distance(
        tx.state(),
        owner,
        outgoing,
        distant,
        1,
    ));
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, outgoing, 1),
        1
    );

    tx.state_mut().turn.current = attacker;
    tx.state_mut().turn.phase = Phase::Play;
    let incoming = tx.state().hand_cards(attacker)[0];
    tx.state_mut().cards.get_mut(incoming).unwrap().kind = CardKind::Sha;
    tx.state_mut().cards.get_mut(incoming).unwrap().suit = Suit::Diamond;
    assert!(!target_legal(&tx, attacker, incoming, owner));
    tx.state_mut().cards.get_mut(incoming).unwrap().suit = Suit::Club;
    assert!(target_legal(&tx, attacker, incoming, owner));
}
}

mod original_collab_olyinhu {
//! Golden coverage for COLLAB「寅虎」and the complete ol_yinhu closure.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLYINHU: u32 = 4_066_124_366;
const OL_YINHU: u32 = 3_117_465_792;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    targets: Vec<PlayerId>,
    skill: SkillId,
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    // Roster-independent: the standard roster gained choice-bearing characters
    // (gongsunzan 义从 / old_re_lidian 询询+忘隙) whose booted presence changes
    // range/hand-limit/trigger answers; this test grants its own skills.
    let booted: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    if !booted.is_empty() {
        run_effects(&mut tx, booted).unwrap();
    }

    let owner = tx.state().turn.current;
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(|player| EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }),
    );
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLYINHU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLYINHU)
        .unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        targets,
        skill,
    }
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    card: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![card],
            },
        },
    )
}

fn enter_extra_play_phase(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_abi110_play_phase_category_contract_and_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 694);
    assert_eq!(package.skill_index.len(), 156);
    assert_eq!(package.character_index.len(), 90);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLYINHU && skill.name == "olyinhu"));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == OLYINHU)
        .unwrap();
    assert_eq!(active.uses_per_turn, None);
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(
        (selection.zone.as_str(), selection.min, selection.max),
        ("hand_equip", 1, 1)
    );
    assert!(selection.unused_category_by_skill_this_play_phase);
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_YINHU
            && character.skills == vec![OLYINHU]
            && character.max_hp == 4
            && character.sex == "male"
            && character.group == "qun"
    }));
}

#[test]
fn categories_are_unique_per_play_phase_but_targets_may_repeat() {
    let mut setup = setup(2_026_072_304);
    let basic = inject_hand(&mut setup.state, setup.owner, 99_001, CardKind::Sha);
    let repeated_basic = inject_hand(&mut setup.state, setup.owner, 99_002, CardKind::Shan);
    let trick = inject_hand(
        &mut setup.state,
        setup.owner,
        99_003,
        CardKind::TrickWuzhong,
    );
    let equip = inject_hand(&mut setup.state, setup.owner, 99_004, CardKind::Weapon);
    let target = setup.targets[0];
    // Fixed HP so three one-damage hits never enter dying and ban the skill.
    setup.state.players.get_mut(target).unwrap().max_hp = 5;
    setup.state.players.get_mut(target).unwrap().hp = 5;
    let hp = 5;

    let (state, first) = activate(setup.state, setup.owner, target, setup.skill, basic, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(
        state
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_category_mask,
        0b001
    );

    let before = canonical_state_hash(&state);
    let (state, repeated) = activate(state, setup.owner, target, setup.skill, repeated_basic, 2);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    let (state, second) = activate(state, setup.owner, target, setup.skill, trick, 3);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    let (mut state, third) = activate(state, setup.owner, target, setup.skill, equip, 4);
    assert!(matches!(third.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 3);
    assert_eq!(
        state
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_category_mask,
        0b111
    );
    assert!(!state.skills.get(setup.skill).unwrap().active_turn_banned);

    state.players.get_mut(target).unwrap().hp = hp;
    let state = enter_extra_play_phase(state, setup.owner);
    assert_eq!(
        state
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_category_mask,
        0
    );
    let (state, next_phase) = activate(state, setup.owner, target, setup.skill, repeated_basic, 5);
    assert!(matches!(next_phase.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_category_mask,
        0b001
    );
}

#[test]
fn exact_dying_bans_for_the_turn_even_across_an_extra_play_phase() {
    let mut setup = setup(2_026_072_305);
    let lethal = inject_hand(&mut setup.state, setup.owner, 99_011, CardKind::Sha);
    let retry = inject_hand(
        &mut setup.state,
        setup.owner,
        99_012,
        CardKind::TrickWuzhong,
    );
    let dying_target = setup.targets[0];
    let surviving_target = setup.targets[1];
    let _tao = inject_hand(&mut setup.state, surviving_target, 99_013, CardKind::Tao);
    setup.state.players.get_mut(dying_target).unwrap().hp = 1;

    let (dying, accepted) = activate(
        setup.state,
        setup.owner,
        dying_target,
        setup.skill,
        lethal,
        10,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(accepted.events.iter().any(|event| matches!(
        event,
        DomainEvent::EnteredDying { player, source }
            if *player == dying_target && *source == setup.owner
    )));
    assert!(dying.skills.get(setup.skill).unwrap().active_turn_banned);
    assert_eq!(
        dying
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_category_mask,
        0b001
    );

    let choice = accepted.choice.expect("dying save choice");
    let (state, saved) = apply_command(
        dying,
        Command {
            actor: choice.observer,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(11),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(saved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());

    let state = enter_extra_play_phase(state, setup.owner);
    assert_eq!(
        state
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_category_mask,
        0
    );
    assert!(state.skills.get(setup.skill).unwrap().active_turn_banned);
    let before = canonical_state_hash(&state);
    let (mut state, rejected) =
        activate(state, setup.owner, surviving_target, setup.skill, retry, 12);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    state.turn.phase = Phase::End;
    let (next_turn, advanced) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(13),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(advanced.result, CommandResult::Accepted { .. }));
    assert_ne!(next_turn.turn.current, setup.owner);
    assert!(
        !next_turn
            .skills
            .get(setup.skill)
            .unwrap()
            .active_turn_banned
    );
}
}

mod original_collab_olzishu {
//! Golden coverage for COLLAB「子鼠」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLZISHU: u32 = 2_483_133_612;
const OL_ZISHU: u32 = 3_877_208_473;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: Vec<PlayerId>,
    skill: SkillId,
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().turn.current;
    let others = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect::<Vec<_>>();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(|player| EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }),
    );
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLZISHU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLZISHU)
        .unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        others,
        skill,
    }
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    choice_id: ChoiceId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_larger_hand_contract_and_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 694);
    assert_eq!(package.skill_index.len(), 156);
    assert_eq!(package.character_index.len(), 90);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == OLZISHU)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    assert!(targets.hand_gt_owner);
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_ZISHU
            && character.name_key == "ol_zishu"
            && character.sex == "male"
            && character.group == "qun"
            && character.max_hp == 3
            && character.skills == [OLZISHU]
    }));
}

#[test]
fn mandatory_first_take_repeats_and_stops_on_a_maximum_hand_tie() {
    let mut setup = setup(2_026_072_316);
    let first = setup.others[0];
    let second = setup.others[1];
    inject_hand(&mut setup.state, setup.owner, 99_301);
    let first_cards = [
        inject_hand(&mut setup.state, first, 99_311),
        inject_hand(&mut setup.state, first, 99_312),
        inject_hand(&mut setup.state, first, 99_313),
    ];
    let second_cards = [
        inject_hand(&mut setup.state, second, 99_321),
        inject_hand(&mut setup.state, second, 99_322),
        inject_hand(&mut setup.state, second, 99_323),
        inject_hand(&mut setup.state, second, 99_324),
    ];

    let (state, activated) = activate(setup.state, setup.owner, first, setup.skill, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let first_card_choice = activated.choice.expect("mandatory first hand-card choice");
    assert!(!first_card_choice.allow_cancel);
    assert_eq!(
        first_card_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        first_cards.map(CardId::raw)
    );

    let (state, first_resolved) = resolve(
        state,
        setup.owner,
        first_card_choice.choice_id,
        vec![first_cards[0].raw()],
        false,
        2,
    );
    assert!(matches!(
        first_resolved.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.hand_count(setup.owner), 2);
    assert_eq!(state.hand_count(first), 2);
    let target_choice = first_resolved.choice.expect("optional repeated target");
    assert!(target_choice.allow_cancel);
    assert_eq!(
        target_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [second.raw()]
    );

    let (state, target_resolved) = resolve(
        state,
        setup.owner,
        target_choice.choice_id,
        vec![second.raw()],
        false,
        3,
    );
    let second_card_choice = target_resolved.choice.expect("second target hand card");
    assert!(!second_card_choice.allow_cancel);
    assert_eq!(
        second_card_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        second_cards.map(CardId::raw)
    );

    let (state, second_resolved) = resolve(
        state,
        setup.owner,
        second_card_choice.choice_id,
        vec![second_cards[0].raw()],
        false,
        4,
    );
    assert!(matches!(
        second_resolved.result,
        CommandResult::Accepted { .. }
    ));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), 3);
    assert_eq!(state.hand_count(second), 3);
}

#[test]
fn repeated_choice_may_be_cancelled_and_initial_non_larger_target_is_rejected() {
    let mut setup = setup(2_026_072_317);
    let equal = setup.others[0];
    let larger = setup.others[1];
    inject_hand(&mut setup.state, setup.owner, 99_331);
    inject_hand(&mut setup.state, equal, 99_332);
    let larger_cards = [
        inject_hand(&mut setup.state, larger, 99_341),
        inject_hand(&mut setup.state, larger, 99_342),
        inject_hand(&mut setup.state, larger, 99_343),
        inject_hand(&mut setup.state, larger, 99_344),
    ];

    let before = canonical_state_hash(&setup.state);
    let (state, rejected) = activate(setup.state, setup.owner, equal, setup.skill, 10);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    let (state, activated) = activate(state, setup.owner, larger, setup.skill, 11);
    let card_choice = activated.choice.expect("first take");
    let (state, taken) = resolve(
        state,
        setup.owner,
        card_choice.choice_id,
        vec![larger_cards[0].raw()],
        false,
        12,
    );
    let repeat = taken.choice.expect("repeat choice");
    let (state, cancelled) = resolve(state, setup.owner, repeat.choice_id, Vec::new(), true, 13);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), 2);
    assert_eq!(state.hand_count(larger), 3);
}
}

mod original_collab_spfengyin {
//! Golden coverage for 封印（collab SP伏完，ABI 471）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag, ZoneId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SPFENGYIN: u32 = 3_552_373_198;
const SOURCE: &str = include_str!("../../../content/original/ported/collab/spfengyin.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn fire_prepare(tx: Transaction, current: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn empty_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        cards
            .into_iter()
            .map(|card| EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(hand),
            })
            .collect(),
    )
    .unwrap();
}

/// Seed a Sha into the player's hand; returns its id.
fn seed_sha(tx: &mut Transaction, player: PlayerId) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_000),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn gives_sha_and_skips_play_and_discard_phases() {
    let package = content_compiler::compile_source_named("spfengyin-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 471);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPFENGYIN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    tx.state_mut().players.get_mut(target).unwrap().hp = 4;
    empty_hand(&mut tx, owner);
    empty_hand(&mut tx, target);
    // Owner holds one Sha; the target's hp (4) >= owner's (4).
    let sha = seed_sha(&mut tx, owner);

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = target;
    let tx = fire_prepare(tx, target);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "Sha give choice pending");
    state = drive_choice(state, owner, vec![sha.raw()], false);

    assert_eq!(state.hand_count(owner), 0, "Sha given away");
    assert_eq!(state.hand_count(target), 1, "target received the Sha");
    assert_eq!(
        state.turn.flag(engine_core::state::turn_flag::SKIP_PLAY),
        1,
        "play phase skipped"
    );
    assert_eq!(
        state.turn.flag(engine_core::state::turn_flag::SKIP_DISCARD),
        1,
        "discard phase skipped"
    );
}

#[test]
fn cancel_keeps_phases() {
    let package = content_compiler::compile_source_named("spfengyin-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPFENGYIN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    tx.state_mut().players.get_mut(target).unwrap().hp = 4;
    empty_hand(&mut tx, owner);
    empty_hand(&mut tx, target);
    let sha = seed_sha(&mut tx, owner);

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = target;
    let tx = fire_prepare(tx, target);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice());
    state = drive_choice(state, owner, vec![], true);
    assert_eq!(state.hand_count(owner), 1, "Sha kept");
    assert_eq!(state.hand_count(target), 0);
    assert_eq!(
        state.turn.flag(engine_core::state::turn_flag::SKIP_PLAY),
        0,
        "play phase not skipped"
    );
    assert_eq!(
        state.turn.flag(engine_core::state::turn_flag::SKIP_DISCARD),
        0,
        "discard phase not skipped"
    );
}

#[test]
fn no_choice_when_target_hp_below_owner() {
    let package = content_compiler::compile_source_named("spfengyin-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPFENGYIN),
    });
    run_effects(&mut tx, effects).unwrap();
    empty_hand(&mut tx, owner);
    empty_hand(&mut tx, target);
    seed_sha(&mut tx, owner);
    // Make the owner tankier: target hp < owner hp blocks the seal.
    let owner_max = tx.state().players.get(owner).unwrap().max_hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = owner_max as i32;
    tx.state_mut().players.get_mut(target).unwrap().hp = 1;

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = target;
    let tx = fire_prepare(tx, target);
    let state = tx.commit().state;
    assert!(
        !state.has_pending_choice(),
        "no choice when the target's hp is below the owner's"
    );
}
}
