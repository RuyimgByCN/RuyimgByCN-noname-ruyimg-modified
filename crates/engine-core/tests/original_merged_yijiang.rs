//! O2 全量合并批次 1（2026-10-14）：家族 yijiang 的 52 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_yijiang_anguo {
//! Golden for 旧朱治「安国」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ANGUO: u32 = 3_231_604_344;
const OLD_ZHUZHI: u32 = 4_018_190_437;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/anguo.skill");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];

    let remove: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    run_effects(
        &mut tx,
        players
            .iter()
            .map(|player| EffectRequest::DiscardFromHand {
                player: *player,
                count: u32::MAX,
            })
            .collect(),
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ANGUO),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == ANGUO)
        .unwrap()
        .id;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn inject_equipment(
    state: &mut GameState,
    target: PlayerId,
    kind: CardKind,
    weapon_range: u8,
    def: u32,
) -> CardId {
    assert!(kind.is_equip());
    let equip = state.zones.find(ZoneKind::Equip, target).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range,
        zone: equip,
        owner: target,
        face_up: true,
    });
    state.zones.get_mut(equip).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    submit(
        setup.state.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    pipeline: PipelineResult,
    card: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = pipeline.choice.expect("equipment choice");
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (1, 1, false)
    );
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == card.raw()));
    submit(
        state,
        choice.observer,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: vec![card.raw()],
            cancelled: false,
        },
        key,
    )
}

#[test]
fn packages_publish_abi97_active_contract_and_old_zhuzhi_closure() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert_eq!(yijiang.skill_index.len(), 281);
    assert_eq!(yijiang.character_index.len(), 72);
    assert_eq!(old.character_index.len(), 78);

    let active = yijiang
        .active_index
        .iter()
        .find(|active| active.skill_def_id == ANGUO)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (1, 1));
    assert!(!targets.allow_self);
    assert!(targets.has_equip);

    let compiled = content_compiler::compile_source_named("anguo", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 97);
    assert_eq!(
        compiled.modules[0].logic_hash,
        yijiang
            .modules
            .iter()
            .find(|module| module.name == compiled.modules[0].name)
            .unwrap()
            .logic_hash
    );

    let character = old
        .character_index
        .iter()
        .find(|character| character.id == OLD_ZHUZHI)
        .unwrap();
    assert_eq!(character.name_key, "old_zhuzhi");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [ANGUO]);
}

#[test]
fn reclaiming_a_range_two_weapon_draws_only_after_the_range_shrinks() {
    let mut setup = setup(2_026_072_301);
    let weapon = inject_equipment(&mut setup.state, setup.target, CardKind::Weapon, 2, 97_301);
    let (state, activated) = activate(&setup, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let (state, resolved) = resolve(state, activated, weapon, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));

    assert_eq!(state.hand_cards(setup.owner).len(), 1);
    assert_eq!(state.hand_cards(setup.target), [weapon]);
    assert_eq!(
        state.cards.get(weapon).unwrap().zone,
        state.zones.find(ZoneKind::Hand, setup.target).unwrap()
    );
    assert_eq!(state.skills.get(setup.skill).unwrap().use_count, 1);
}

#[test]
fn reclaiming_non_range_equipment_does_not_draw() {
    let mut setup = setup(2_026_072_302);
    let armor = inject_equipment(&mut setup.state, setup.target, CardKind::Armor, 0, 97_302);
    let (state, activated) = activate(&setup, 3);
    let (state, resolved) = resolve(state, activated, armor, 4);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(setup.owner).is_empty());
    assert_eq!(state.hand_cards(setup.target), [armor]);
}

#[test]
fn empty_equipment_target_and_second_use_are_atomically_rejected() {
    let empty = setup(2_026_072_303);
    let before = canonical_state_hash(&empty.state);
    let (same, rejected) = activate(&empty, 5);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
    assert_eq!(same.skills.get(empty.skill).unwrap().use_count, 0);

    let mut setup = setup(2_026_072_304);
    let armor = inject_equipment(&mut setup.state, setup.target, CardKind::Armor, 0, 97_303);
    let (state, activated) = activate(&setup, 6);
    let (mut state, resolved) = resolve(state, activated, armor, 7);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    inject_equipment(&mut state, setup.target, CardKind::Armor, 0, 97_304);
    let before = canonical_state_hash(&state);
    let (same, rejected) = submit(
        state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        8,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}

#[test]
fn equipment_choice_revalidates_the_card_at_resume() {
    let mut setup = setup(2_026_072_305);
    let armor = inject_equipment(&mut setup.state, setup.target, CardKind::Armor, 0, 97_305);
    let (mut state, activated) = activate(&setup, 9);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));

    let equip = state.zones.find(ZoneKind::Equip, setup.target).unwrap();
    let hand = state.zones.find(ZoneKind::Hand, setup.target).unwrap();
    state
        .zones
        .get_mut(equip)
        .unwrap()
        .cards
        .retain(|card| *card != armor);
    state.zones.get_mut(hand).unwrap().cards.push(armor);
    let card = state.cards.get_mut(armor).unwrap();
    card.zone = hand;
    card.face_up = false;

    let before = canonical_state_hash(&state);
    let (same, rejected) = resolve(state, activated, armor, 10);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
}

mod original_yijiang_anjian {
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

const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const ANJIAN: u32 = 3_402_374_234;

#[test]
fn anjian_uses_the_targets_attack_range_for_logical_sha_damage() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(ANJIAN as u64),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: ANJIAN as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let source = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let source_index = seats.iter().position(|player| *player == source).unwrap();
    let adjacent = seats[(source_index + 1) % seats.len()];
    let target = seats[(source_index + 2) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let mut isolate: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    isolate.push(EffectRequest::AddSkill {
        player: source,
        skill_def: SkillDefId::new(ANJIAN),
    });
    run_effects(&mut tx, isolate).unwrap();
    assert_eq!(
        engine_core::query::sha_damage_outside_target_range_for(tx.state(), source, adjacent, 1),
        1
    );
    assert_eq!(
        engine_core::query::sha_damage_outside_target_range_for(tx.state(), source, target, 1),
        2
    );

    let source_equip = tx.state().zones.find(ZoneKind::Equip, source).unwrap();
    let weapon = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(ANJIAN + 1),
        kind: CardKind::Weapon,
        suit: Suit::Club,
        rank: 5,
        weapon_range: 2,
        zone: source_equip,
        owner: source,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(source_equip)
        .unwrap()
        .cards
        .push(weapon);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: target,
            count: u32::MAX,
        }],
    )
    .unwrap();
    tx.state_mut().players.get_mut(target).unwrap().hp = 10;
    tx.state_mut().players.get_mut(target).unwrap().max_hp = 10;
    let source_hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(ANJIAN + 2),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: source_hand,
        owner: source,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(source_hand)
        .unwrap()
        .cards
        .push(sha);

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 8);
}
}

mod original_yijiang_benxi_old_wuyi {
//! Golden coverage for old 吴懿「奔袭」rewritten from legacy JavaScript semantics.

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, sha_target_limit_for, QueryContext};
use engine_core::state::{turn_flag, CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const BENXI: u32 = 2_283_450_478;
const RENWANG: u32 = 4_083_737_417;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/benxi.skill");
const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn distance(state: &GameState, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("distance query failed: {other:?}"),
    }
}

fn target_legal(
    state: &GameState,
    user: PlayerId,
    card: engine_api::CardId,
    target: PlayerId,
) -> bool {
    match query(
        state,
        &QueryRequest::TargetLegal { user, card, target },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Bool(value) => value,
        other => panic!("target query failed: {other:?}"),
    }
}

#[test]
fn current_turn_distance_one_enables_extra_sha_target_and_armor_bypass() {
    let package = content_compiler::compile_source_named("benxi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 281);
    let mut state = GameState::empty();
    for bytes in [SP, MOBILE] {
        std::sync::Arc::make_mut(&mut state.content)
            .load_package(&content_schema::ContentPackage::from_json_bytes(bytes).unwrap())
            .unwrap();
    }
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_072_903, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let far = players[2];
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
            skill_def: SkillDefId::new(BENXI),
        },
        EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(RENWANG),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let sha = tx.state().hand_cards(owner)[0];
    {
        let card = tx.state_mut().cards.get_mut(sha).unwrap();
        card.kind = CardKind::Sha;
        card.suit = engine_api::Suit::Spade;
    }

    assert_eq!(distance(tx.state(), owner, far), 2);
    assert_eq!(sha_target_limit_for(tx.state(), owner), 1);
    assert!(!target_legal(tx.state(), owner, sha, target));

    tx.state_mut()
        .turn
        .set_flag(turn_flag::card_used_by_player(owner), 1);
    assert_eq!(distance(tx.state(), owner, far), 1);
    assert_eq!(sha_target_limit_for(tx.state(), owner), 2);
    assert!(target_legal(tx.state(), owner, sha, target));

    tx.state_mut().turn.current = target;
    assert_eq!(distance(tx.state(), owner, far), 2);
    assert_eq!(sha_target_limit_for(tx.state(), owner), 1);
    assert!(!target_legal(tx.state(), owner, sha, target));
}
}

mod original_yijiang_caifuren {
//! Golden coverage for 蔡夫人「窃听」「献州」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, ChoiceRequest, Command, CommandKind, CommandResult,
    DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
    TurnPhaseTag,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const CAIFUREN: u32 = 2_163_495_034;
const QIETING: u32 = 4_166_951_057;
const XIANZHOU: u32 = 4_217_889_451;
const QIETING_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/qieting.skill");
const XIANZHOU_SOURCE: &str =
    include_str!("../../../content/original/ported/yijiang/xianzhou.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    recipient: PlayerId,
    others: [PlayerId; 2],
}

struct Step {
    state: GameState,
    choice: Option<ChoiceRequest>,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let recipient = players[1];
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
    effects.extend(
        [QIETING, XIANZHOU].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 8;
        participant.max_hp = 8;
    }
    tx.state_mut().players.get_mut(owner).unwrap().character_def = CAIFUREN;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        recipient,
        others: [players[2], players[3]],
    }
}

fn inject_equipment(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    assert!(kind.is_equip());
    let equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) + 1,
        zone: equip,
        owner,
        face_up: true,
    });
    state.zones.get_mut(equip).unwrap().cards.push(card);
    card
}

fn skill_instance(state: &GameState, owner: PlayerId, skill_def: u32) -> SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn activate_xianzhou(
    state: GameState,
    owner: PlayerId,
    recipient: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let skill = skill_instance(&state, owner, XIANZHOU);
    submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![recipient],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn trigger_qieting(state: GameState, ending: PlayerId, card_targeted_other: bool) -> Step {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: ending,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let result = run_effects(&mut tx, effects);
    assert!(matches!(result, Ok(_) | Err(EffectError::ChoiceRequested)));
    let committed = tx.commit();
    Step {
        state: committed.state,
        choice: committed.output.choice,
    }
}

#[test]
fn package_publishes_complete_caifuren_and_exact_abi58_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CAIFUREN)
        .unwrap();
    assert_eq!(character.name_key, "caifuren");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [QIETING, XIANZHOU]);

    for (name, source) in [("qieting", QIETING_SOURCE), ("xianzhou", XIANZHOU_SOURCE)] {
        let compiled = content_compiler::compile_source_named(name, source).unwrap();
        assert_eq!(compiled.manifest.rule_vm_abi_version, 58);
        for expected in compiled.modules {
            let actual = package
                .modules
                .iter()
                .find(|module| module.name == expected.name)
                .unwrap();
            assert_eq!(actual.logic_hash, expected.logic_hash);
        }
    }

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == QIETING)
        .unwrap();
    assert_eq!(trigger.event, "turn_ended");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_turn_ended_no_card_targeted_other"
    );
    assert!(!trigger.optional);

    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == XIANZHOU)
        .unwrap();
    assert!(active.equip_nonempty);
    assert!(active.once_per_game);
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
}

#[test]
fn qieting_draw_cancel_and_turn_target_history_gate_are_authoritative() {
    let setup = setup(2_026_073_801);
    let blocked = trigger_qieting(setup.state.clone(), setup.recipient, true);
    assert!(blocked.choice.is_none());
    assert!(!blocked.state.has_pending_choice());

    let step = trigger_qieting(setup.state.clone(), setup.recipient, false);
    let choice = step.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (1, 1, true)
    );
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1]
    );
    let hand = step.state.hand_count(setup.owner);
    let (state, drawn) = resolve(step.state, setup.owner, vec![1], false, 1);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), hand + 1);

    let step = trigger_qieting(setup.state, setup.recipient, false);
    let hand = step.state.hand_count(setup.owner);
    let (state, cancelled) = resolve(step.state, setup.owner, Vec::new(), true, 2);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), hand);
    assert!(!state.has_pending_choice());
}

#[test]
fn qieting_moves_compatible_equipment_replaces_slot_and_rejects_forgery_atomically() {
    let mut setup = setup(2_026_073_802);
    let old_weapon = inject_equipment(&mut setup.state, setup.owner, 98_201, CardKind::Weapon);
    let target_weapon =
        inject_equipment(&mut setup.state, setup.recipient, 98_202, CardKind::Weapon);
    let step = trigger_qieting(setup.state, setup.recipient, false);
    assert_eq!(
        step.choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    let (state, branch) = resolve(step.state, setup.owner, vec![0], false, 10);
    assert!(matches!(branch.result, CommandResult::Accepted { .. }));
    assert_eq!(
        branch.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseMoveTargetEquipmentOrDraw
    );

    let before = canonical_state_hash(&state);
    let (state, forged) = resolve(state, setup.owner, vec![old_weapon.raw()], false, 11);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    let (state, moved) = resolve(state, setup.owner, vec![target_weapon.raw()], false, 12);
    assert!(matches!(moved.result, CommandResult::Accepted { .. }));
    let owner_equip = state.zones.find(ZoneKind::Equip, setup.owner).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(old_weapon).unwrap().zone, discard);
    let moved_card = state.cards.get(target_weapon).unwrap();
    assert_eq!(moved_card.zone, owner_equip);
    assert_eq!(moved_card.owner, setup.owner);
}

#[test]
fn xianzhou_gives_all_equipment_cancel_recovers_and_lifetime_limit_persists() {
    let mut setup = setup(2_026_073_803);
    let skill = skill_instance(&setup.state, setup.owner, XIANZHOU);
    let before = canonical_state_hash(&setup.state);
    let (state, rejected) = activate_xianzhou(setup.state, setup.owner, setup.recipient, 20);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    assert_eq!(state.skills.get(skill).unwrap().lifetime_fire_count, 0);
    setup.state = state;

    let weapon = inject_equipment(&mut setup.state, setup.owner, 98_301, CardKind::Weapon);
    let armor = inject_equipment(&mut setup.state, setup.owner, 98_302, CardKind::Armor);
    setup.state.players.get_mut(setup.owner).unwrap().hp = 3;
    let recipient_hand = setup.state.hand_count(setup.recipient);
    let (state, offered) = activate_xianzhou(setup.state, setup.owner, setup.recipient, 21);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    let choice = offered.choice.as_ref().unwrap();
    assert_eq!(choice.observer, setup.recipient);
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (1, 2, true)
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseRecoverOwnerOrDamageByGivenEquipment
    );
    let recipient_hand_zone = state.zones.find(ZoneKind::Hand, setup.recipient).unwrap();
    assert_eq!(state.hand_count(setup.recipient), recipient_hand + 2);
    assert_eq!(state.cards.get(weapon).unwrap().zone, recipient_hand_zone);
    assert_eq!(state.cards.get(armor).unwrap().zone, recipient_hand_zone);
    assert_eq!(state.skills.get(skill).unwrap().lifetime_fire_count, 1);

    let (mut state, recovered) = resolve(state, setup.recipient, Vec::new(), true, 22);
    assert!(matches!(recovered.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 5);
    inject_equipment(&mut state, setup.owner, 98_303, CardKind::Armor);
    let before = canonical_state_hash(&state);
    let (state, repeated) = activate_xianzhou(state, setup.owner, setup.recipient, 23);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    assert_eq!(state.skills.get(skill).unwrap().lifetime_fire_count, 1);
}

#[test]
fn xianzhou_damage_targets_are_distinct_in_range_and_validated_atomically() {
    let mut setup = setup(2_026_073_804);
    inject_equipment(&mut setup.state, setup.owner, 98_401, CardKind::Weapon);
    inject_equipment(&mut setup.state, setup.owner, 98_402, CardKind::Armor);
    let (state, offered) = activate_xianzhou(setup.state, setup.owner, setup.recipient, 30);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    let options: Vec<_> = offered
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(options, [setup.owner.raw(), setup.others[0].raw()]);

    let before = canonical_state_hash(&state);
    let (state, self_target) = resolve(
        state,
        setup.recipient,
        vec![setup.recipient.raw()],
        false,
        31,
    );
    assert!(matches!(self_target.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    let (state, out_of_range) = resolve(
        state,
        setup.recipient,
        vec![setup.others[1].raw()],
        false,
        32,
    );
    assert!(matches!(
        out_of_range.result,
        CommandResult::Rejected { .. }
    ));
    assert_eq!(canonical_state_hash(&state), before);
    let (state, duplicate) = resolve(
        state,
        setup.recipient,
        vec![setup.owner.raw(), setup.owner.raw()],
        false,
        33,
    );
    assert!(matches!(duplicate.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    let owner_hp = state.players.get(setup.owner).unwrap().hp;
    let other_hp = state.players.get(setup.others[0]).unwrap().hp;
    let (state, damaged) = resolve(
        state,
        setup.recipient,
        vec![setup.others[0].raw(), setup.owner.raw()],
        false,
        34,
    );
    assert!(matches!(damaged.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(setup.others[0]).unwrap().hp, other_hp - 1);
    assert_eq!(
        damaged
            .events
            .iter()
            .filter_map(|event| match event {
                DomainEvent::DamageDealt { source, target, .. } if *source == setup.recipient =>
                    Some(*target),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [setup.others[0], setup.owner]
    );
}
}

mod original_yijiang_caoxiu {
//! Golden coverage for 曹休「千驹」「倾袭」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const CAOXIU: u32 = 3_428_883_420;
const QIANJU: u32 = 2_828_017_083;
const QINGXI: u32 = 3_641_969_972;
const QIANJU_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/qianju.skill");
const QINGXI_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/qingxi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let other = players[2];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(QINGXI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().players.get_mut(owner).unwrap().character_def = CAOXIU;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        other,
    }
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
    weapon_range: u8,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn deal_card_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    kind: CardKind,
    nature: u8,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature,
            card: CardId::NONE,
            card_kind: kind.tag(),
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

fn open_qingxi(state: GameState, owner: PlayerId, target: PlayerId) -> GameState {
    let (state, result) = deal_card_damage(
        state,
        owner,
        target,
        CardKind::Sha,
        rule_api::damage_nature::NORMAL,
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    state
}

#[test]
fn package_publishes_complete_caoxiu_and_exact_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CAOXIU)
        .unwrap();
    assert_eq!(character.name_key, "caoxiu");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [QIANJU, QINGXI]);
    for (name, source, abi) in [("qianju", QIANJU_SOURCE, 3), ("qingxi", QINGXI_SOURCE, 59)] {
        let compiled = content_compiler::compile_source_named(name, source).unwrap();
        assert_eq!(compiled.manifest.rule_vm_abi_version, abi);
        for expected in compiled.modules {
            let actual = package
                .modules
                .iter()
                .find(|module| module.name == expected.name)
                .unwrap();
            assert_eq!(actual.logic_hash, expected.logic_hash);
        }
    }
}

#[test]
fn qingxi_requires_direct_sha_damage_and_physical_weapon() {
    let no_weapon = setup(2_026_073_901);
    let (state, result) = deal_card_damage(
        no_weapon.state,
        no_weapon.owner,
        no_weapon.target,
        CardKind::Sha,
        rule_api::damage_nature::NORMAL,
    );
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(no_weapon.target).unwrap().hp, 9);

    let mut setup = setup(2_026_073_902);
    inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Equip,
        99_901,
        CardKind::Weapon,
        2,
    );
    let (state, result) = deal_card_damage(
        setup.state,
        setup.owner,
        setup.target,
        CardKind::TrickJuedou,
        rule_api::damage_nature::NORMAL,
    );
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, 9);
}

#[test]
fn qingxi_insufficient_hand_and_cancel_increase_damage_without_discarding_weapon() {
    let mut insufficient = setup(2_026_073_903);
    let weapon = inject(
        &mut insufficient.state,
        insufficient.owner,
        ZoneKind::Equip,
        99_911,
        CardKind::Weapon,
        3,
    );
    for def in [99_912, 99_913] {
        inject(
            &mut insufficient.state,
            insufficient.target,
            ZoneKind::Hand,
            def,
            CardKind::Other,
            0,
        );
    }
    let state = open_qingxi(insufficient.state, insufficient.owner, insufficient.target);
    let (state, accepted) = resolve(state, insufficient.owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(insufficient.target).unwrap().hp, 8);
    assert_eq!(state.cards.get(weapon).unwrap().owner, insufficient.owner);

    let mut cancelled = setup(2_026_073_904);
    let weapon = inject(
        &mut cancelled.state,
        cancelled.owner,
        ZoneKind::Equip,
        99_921,
        CardKind::Weapon,
        2,
    );
    for def in [99_922, 99_923] {
        inject(
            &mut cancelled.state,
            cancelled.target,
            ZoneKind::Hand,
            def,
            CardKind::Other,
            0,
        );
    }
    let state = open_qingxi(cancelled.state, cancelled.owner, cancelled.target);
    let (state, choice) = resolve(state, cancelled.owner, vec![1], false, 2);
    assert_eq!(
        choice.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert_eq!(choice.choice.as_ref().unwrap().observer, cancelled.target);
    assert_eq!(choice.choice.as_ref().unwrap().min_count, 2);
    assert_eq!(choice.choice.as_ref().unwrap().max_count, 2);
    let (state, result) = resolve(state, cancelled.target, Vec::new(), true, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(cancelled.target).unwrap().hp, 8);
    assert_eq!(state.hand_count(cancelled.target), 2);
    assert_eq!(state.cards.get(weapon).unwrap().owner, cancelled.owner);
}

#[test]
fn qingxi_paid_discard_is_ordered_and_rejects_forgery_atomically() {
    let mut setup = setup(2_026_073_905);
    let weapon = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Equip,
        99_931,
        CardKind::Weapon,
        2,
    );
    let cards: Vec<_> = [99_932, 99_933, 99_934]
        .into_iter()
        .map(|def| {
            inject(
                &mut setup.state,
                setup.target,
                ZoneKind::Hand,
                def,
                CardKind::Other,
                0,
            )
        })
        .collect();
    let state = open_qingxi(setup.state, setup.owner, setup.target);
    let (state, _) = resolve(state, setup.owner, vec![1], false, 10);
    let before = canonical_state_hash(&state);
    let (state, forged) = resolve(
        state,
        setup.target,
        vec![cards[0].raw(), cards[0].raw()],
        false,
        11,
    );
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    let (state, paid) = resolve(
        state,
        setup.target,
        vec![cards[0].raw(), cards[1].raw()],
        false,
        12,
    );
    assert!(matches!(paid.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.target).unwrap().hp, 9);
    assert_eq!(state.hand_count(setup.target), 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in [cards[0], cards[1], weapon] {
        assert_eq!(state.cards.get(card).unwrap().zone, discard);
    }
    assert_eq!(state.cards.get(cards[2]).unwrap().owner, setup.target);
}

#[test]
fn qingxi_increases_chain_base_without_retriggering_on_propagation() {
    let mut setup = setup(2_026_073_906);
    inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Equip,
        99_941,
        CardKind::Weapon,
        2,
    );
    for def in [99_942, 99_943] {
        inject(
            &mut setup.state,
            setup.target,
            ZoneKind::Hand,
            def,
            CardKind::Other,
            0,
        );
    }
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddMark {
                player: setup.target,
                mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
            EffectRequest::AddMark {
                player: setup.other,
                mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
        ],
    )
    .unwrap();
    let state = tx.commit().state;
    let (state, result) = deal_card_damage(
        state,
        setup.owner,
        setup.target,
        CardKind::Sha,
        rule_api::damage_nature::FIRE,
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, _) = resolve(state, setup.owner, vec![1], false, 20);
    let (state, result) = resolve(state, setup.target, Vec::new(), true, 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, 8);
    assert_eq!(state.players.get(setup.other).unwrap().hp, 8);
}
}

mod original_yijiang_chenqun {
//! Golden coverage for 陈群「品第」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const CHENQUN: u32 = 2_960_571_502;
const PINDI: u32 = 4_057_378_399;
const FAEN: u32 = 2_817_547_947;
const PINDI_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/pindi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: PlayerId,
    second: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PINDI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        first: players[1],
        second: players[2],
    }
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn pindi_skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == PINDI)
        .unwrap()
        .id
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    card: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let skill = pindi_skill(&state, owner);
    submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![card],
        },
        key,
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

#[test]
fn package_publishes_complete_chenqun_and_exact_abi52_module_in_abi53_pack() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CHENQUN)
        .unwrap();
    assert_eq!(character.skills, [PINDI, FAEN]);
    let compiled = content_compiler::compile_source_named("pindi", PINDI_SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 52);
    for expected in compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn first_use_draws_one_and_links_owner_when_target_is_wounded() {
    let mut setup = setup(2_026_073_201);
    let cost = inject(&mut setup.state, setup.owner, CardKind::Sha, 99_201);
    inject(&mut setup.state, setup.first, CardKind::Shan, 99_202);
    {
        let target = setup.state.players.get_mut(setup.first).unwrap();
        target.max_hp = 4;
        target.hp = 3;
    }
    let before = setup.state.hand_count(setup.first);
    let (state, opened) = activate(setup.state, setup.owner, setup.first, cost, 1);
    assert!(matches!(opened.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded
    );
    let (state, resolved) = resolve(state, setup.owner, vec![1], 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.first), before + 1);
    assert!(state
        .players
        .get(setup.owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count == 1));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(cost).unwrap().zone, discard);
    let skill = state.skills.get(pindi_skill(&state, setup.owner)).unwrap();
    assert_eq!(skill.use_count, 1);
    assert_eq!(skill.state_blob.get(..4), Some(b"AUC1".as_slice()));
}

#[test]
fn second_use_discards_two_target_cards_and_counts_from_the_whole_turn() {
    let mut setup = setup(2_026_073_202);
    let first_cost = inject(&mut setup.state, setup.owner, CardKind::Sha, 99_211);
    inject(&mut setup.state, setup.first, CardKind::Shan, 99_212);
    let (state, _) = activate(setup.state, setup.owner, setup.first, first_cost, 10);
    let (mut state, _) = resolve(state, setup.owner, vec![1], 11);

    let second_cost = inject(&mut state, setup.owner, CardKind::TrickWuzhong, 99_213);
    let target_card_a = inject(&mut state, setup.second, CardKind::Shan, 99_214);
    let target_card_b = inject(&mut state, setup.second, CardKind::Tao, 99_215);
    let target_card_c = inject(&mut state, setup.second, CardKind::Jiu, 99_216);
    let (state, opened) = activate(state, setup.owner, setup.second, second_cost, 12);
    assert!(matches!(opened.result, CommandResult::Accepted { .. }));
    let (state, branch) = resolve(state, setup.owner, vec![2], 13);
    assert!(matches!(branch.result, CommandResult::Accepted { .. }));
    assert_eq!(branch.choice.as_ref().unwrap().observer, setup.second);
    assert_eq!(branch.choice.as_ref().unwrap().min_count, 2);
    let (state, discarded) = resolve(
        state,
        setup.second,
        vec![target_card_a.raw(), target_card_b.raw()],
        14,
    );
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(target_card_a).unwrap().zone, discard);
    assert_eq!(state.cards.get(target_card_b).unwrap().zone, discard);
    assert_ne!(state.cards.get(target_card_c).unwrap().zone, discard);
    assert_eq!(
        state
            .skills
            .get(pindi_skill(&state, setup.owner))
            .unwrap()
            .use_count,
        2
    );
}

#[test]
fn repeated_target_or_logical_category_and_forged_branch_are_atomic() {
    let mut setup = setup(2_026_073_203);
    let first_cost = inject(&mut setup.state, setup.owner, CardKind::Sha, 99_221);
    inject(&mut setup.state, setup.first, CardKind::Shan, 99_222);
    let (state, _) = activate(setup.state, setup.owner, setup.first, first_cost, 20);
    let (mut state, _) = resolve(state, setup.owner, vec![1], 21);

    let repeated_target_cost = inject(&mut state, setup.owner, CardKind::TrickWuzhong, 99_223);
    let before_target = canonical_state_hash(&state);
    let (state, rejected_target) =
        activate(state, setup.owner, setup.first, repeated_target_cost, 22);
    assert!(!matches!(
        rejected_target.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(canonical_state_hash(&state), before_target);

    let mut state = state;
    let repeated_category_cost = inject(&mut state, setup.owner, CardKind::Shan, 99_224);
    let before_category = canonical_state_hash(&state);
    let (state, rejected_category) =
        activate(state, setup.owner, setup.second, repeated_category_cost, 23);
    assert!(!matches!(
        rejected_category.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(canonical_state_hash(&state), before_category);

    let mut state = state;
    let valid_cost = inject(&mut state, setup.owner, CardKind::Weapon, 99_226);
    inject(&mut state, setup.second, CardKind::Tao, 99_227);
    let (state, _) = activate(state, setup.owner, setup.second, valid_cost, 24);
    let before_forge = canonical_state_hash(&state);
    let (state, forged) = resolve(state, setup.owner, vec![3], 25);
    assert!(!matches!(forged.result, CommandResult::Accepted { .. }));
    assert_eq!(canonical_state_hash(&state), before_forge);
}
}

mod original_yijiang_chunlao {
//! Golden coverage for 程普「醇醪」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, CardZoneTag, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, ObserverKind, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::invariants::{check_invariants, InvariantError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::{canonical_state_hash, project_observer};
use rule_api::EffectRequest;

const CHENGPU: u32 = 2_944_307_433;
const LIHUO: u32 = 3_289_572_540;
const CHUNLAO: u32 = 3_165_720_039;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/chunlao.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    dying: PlayerId,
    sha: [CardId; 2],
    other: CardId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind, suit: Suit) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let dying = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &seats {
        effects.push(EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        });
    }
    // 先设结束阶段再挂技能，避免技能自身的 phase_end 可选触发在 setup 中开确认。
    effects.extend([
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::End.to_u8(),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(CHUNLAO),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let sha = [
        inject(tx.state_mut(), owner, 99_001, CardKind::Sha, Suit::Spade),
        inject(
            tx.state_mut(),
            owner,
            99_002,
            CardKind::ShaFire,
            Suit::Heart,
        ),
    ];
    let other = inject(
        tx.state_mut(),
        owner,
        99_003,
        CardKind::TrickWuzhong,
        Suit::Club,
    );
    Setup {
        state: tx.commit().state,
        owner,
        dying,
        sha,
        other,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
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
                cancelled: false,
            },
        },
    )
}

fn store(setup: &Setup, key: u64) -> GameState {
    let mut tx = Transaction::begin(setup.state.clone());
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: setup.owner,
                phase: TurnPhaseTag::End,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    // 结束阶段可选触发：先确认，再选牌存储。

    let (state, confirmed) = resolve(tx.state().clone(), setup.owner, vec![1], key);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseStoreHandShaInSkillPile
    );
    let offered: Vec<_> = confirmed
        .choice
        .as_ref()
        .map(|choice| {
            choice
                .options
                .iter()
                .map(|option| option.token)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    assert_eq!(offered, setup.sha.map(CardId::raw));
    let (state, stored) = resolve(
        state,
        setup.owner,
        setup.sha.into_iter().map(CardId::raw).collect(),
        key + 1,
    );
    assert!(matches!(stored.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    state
}

fn enter_dying(state: GameState, source: PlayerId, target: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(target).unwrap().hp = 1;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(tx.state().players.get(target).unwrap().hp, 0);
    assert!(tx.state().players.get(target).unwrap().dying);
    tx.state().clone()
}

#[test]
fn package_publishes_complete_chengpu_and_exact_abi53_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CHENGPU)
        .unwrap();
    assert_eq!(character.skills, [LIHUO, CHUNLAO]);
    let compiled = content_compiler::compile_source_named("chunlao", SOURCE).unwrap();
    assert_eq!(compiled.modules.len(), 2);
    for expected in &compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn end_phase_stores_only_hand_sha_in_one_public_canonical_skill_pile() {
    let setup = setup(2_026_072_002);
    let state = store(&setup, 1);

    assert_eq!(state.skill_pile_cards(setup.owner, CHUNLAO), setup.sha);
    assert_eq!(state.hand_cards(setup.owner), [setup.other]);
    assert_eq!(state.skill_piles.len(), 1);
    let pile = state.skill_piles[0];
    let zone = state.zones.get(pile.zone).unwrap();
    assert_eq!((pile.owner, pile.skill_def), (setup.owner, CHUNLAO));
    assert_eq!((zone.kind, zone.owner), (ZoneKind::Expansion, setup.owner));
    assert!(setup
        .sha
        .iter()
        .all(|card| state.cards.get(*card).unwrap().face_up));
    check_invariants(&state).unwrap();
    assert_eq!(
        canonical_state_hash(&state),
        canonical_state_hash(&state.clone())
    );

    let mut wrong_zone = state.clone();
    wrong_zone.skill_piles[0].zone = wrong_zone.zones.find(ZoneKind::Hand, setup.dying).unwrap();
    assert!(matches!(
        check_invariants(&wrong_zone),
        Err(InvariantError::InvalidSkillPile { .. })
    ));
    let mut duplicate = state.clone();
    duplicate.skill_piles.push(duplicate.skill_piles[0]);
    assert_eq!(
        check_invariants(&duplicate),
        Err(InvariantError::SkillPileOrder)
    );

    let view = project_observer(&state, ObserverKind::Player(setup.dying));
    for card in setup.sha {
        let public = view
            .known_cards
            .iter()
            .find(|known| known.id == card)
            .unwrap();
        assert_eq!(public.zone, CardZoneTag::Expansion);
        assert!(public.def.is_some());
    }

    let mut tx = Transaction::begin(state);
    // 同一回合内再次 dispatch phase_end 会再次触发可选存牌（游戏中 phase_end 每回合仅一次）；
    // 确认并存入剩余手牌后应无悬挂 choice。
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: setup.owner,
                phase: TurnPhaseTag::End,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let optional = tx.commit().state;
    assert_eq!(
        optional.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    // 确认后效果发现 skill pile 已非空（首次已存）而跳过，无后续选牌。
    let (state, confirmed) = resolve(optional, setup.owner, vec![1], 2);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
}

#[test]
fn each_stored_sha_can_save_a_dying_player_through_the_virtual_jiu_pipeline() {
    let setup = setup(2_026_072_003);
    let mut state = store(&setup, 10);

    for (index, spent) in setup.sha.into_iter().enumerate() {
        state = enter_dying(state, setup.owner, setup.dying);
        assert_eq!(
            state.resolution.continuation_kind,
            ContinuationKind::OptionalTrigger
        );

        let (next, accepted) = resolve(state, setup.owner, vec![1], 20 + index as u64 * 2);
        assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
        assert_eq!(
            next.resolution.continuation_kind,
            ContinuationKind::ChooseSpendSkillPileForDyingJiu
        );
        let offered: Vec<_> = accepted
            .choice
            .unwrap()
            .options
            .into_iter()
            .map(|option| option.token)
            .collect();
        assert!(offered.contains(&spent.raw()));

        let (next, saved) = resolve(next, setup.owner, vec![spent.raw()], 21 + index as u64 * 2);
        assert!(matches!(saved.result, CommandResult::Accepted { .. }));
        assert!(!next.has_pending_choice());
        let rescued = next.players.get(setup.dying).unwrap();
        assert_eq!(rescued.hp, 1);
        assert!(!rescued.dying);
        let discard = next
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        assert_eq!(next.cards.get(spent).unwrap().zone, discard);
        assert!(saved.events.iter().any(|event| matches!(
            event,
            DomainEvent::CardUsed {
                player,
                targets,
                ..
            } if *player == setup.dying && targets == &[setup.dying]
        )));
        assert!(saved.events.iter().any(|event| matches!(
            event,
            DomainEvent::HpRecovered { player, amount, .. }
                if *player == setup.dying && *amount == 1
        )));
        check_invariants(&next).unwrap();
        state = next;
    }

    assert!(state.skill_pile_cards(setup.owner, CHUNLAO).is_empty());
}
}

mod original_yijiang_dangxian {
//! Golden coverage for 廖化「当先」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DANGXIAN: u32 = 2_393_638_375;
const FULI: u32 = 3_748_306_630;
const LIAOHUA: u32 = 3_588_461_279;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/dangxian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    first_sha: CardId,
    second_sha: CardId,
}

fn inject_sha(state: &mut GameState, owner: PlayerId, def: u32, suit: Suit) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit,
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

    let seats = tx.state().players.alive_by_seat();
    let current = tx.state().turn.current;
    let current_index = seats.iter().position(|player| *player == current).unwrap();
    let owner = seats[(current_index + 1) % seats.len()];
    let target = current;

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in seats {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DANGXIAN),
        },
        EffectRequest::SetTurnState {
            current,
            phase: Phase::End.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();

    let first_sha = inject_sha(tx.state_mut(), owner, 94_001, Suit::Spade);
    let second_sha = inject_sha(tx.state_mut(), owner, 94_002, Suit::Heart);
    Setup {
        state: tx.commit().state,
        owner,
        target,
        first_sha,
        second_sha,
    }
}

fn end_phase(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
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

fn play_sha(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

#[test]
fn package_publishes_complete_liaohua_and_exact_abi53_package() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == LIAOHUA)
        .unwrap();
    assert_eq!(character.skills, [DANGXIAN, FULI]);
    let compiled = content_compiler::compile_source_named("dangxian", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn extra_play_precedes_prepare_and_normal_play_has_an_independent_sha_allowance() {
    let setup = setup(2_026_071_951);

    let (state, began) = end_phase(setup.state, setup.target, 1);
    assert!(matches!(began.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, setup.owner);
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE), 1);
    assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_PENDING), 0);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE), 0);

    let (state, first) = play_sha(state, setup.owner, setup.first_sha, setup.target, 2);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE), 1);

    let (state, prepare) = end_phase(state, setup.owner, 3);
    assert!(matches!(prepare.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Prepare);
    assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE), 0);

    let (state, judge) = end_phase(state, setup.owner, 4);
    assert!(matches!(judge.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Judge);

    let (state, draw) = end_phase(state, setup.owner, 5);
    assert!(matches!(draw.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Draw);

    let (state, play) = end_phase(state, setup.owner, 6);
    assert!(matches!(play.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE), 0);

    let (state, second) = play_sha(state, setup.owner, setup.second_sha, setup.target, 7);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 2);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE), 1);
}
}

mod original_yijiang_dingpin_old_chenqun {
//! Golden coverage for 旧陈群「定品」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DINGPIN: u32 = 3_356_547_985;
const OLDFAEN: u32 = 3_151_099_816;
const OLD_CHENQUN: u32 = 2_336_327_482;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/dingpin.skill");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("dingpin-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DINGPIN),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, target)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn judgment(state: &mut GameState, suit: Suit, def: u32) {
    inject(
        state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        suit,
        def,
    );
}

fn skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DINGPIN)
        .unwrap()
        .id
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    cost: CardId,
    key: u64,
) -> (GameState, CommandResult) {
    let runtime_skill = skill(&state, owner);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: runtime_skill,
                targets: vec![target],
                card_targets: vec![cost],
            },
        },
    );
    (state, result.result)
}

#[test]
fn dingpin_preserves_owner_history_judgment_target_and_old_character_semantics() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert!(
        yijiang
            .active_index
            .iter()
            .find(|active| active.skill_def_id == DINGPIN)
            .unwrap()
            .selection
            .as_ref()
            .unwrap()
            .unused_category_by_owner_history_this_turn
    );
    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    let character = old
        .character_index
        .iter()
        .find(|character| character.id == OLD_CHENQUN)
        .unwrap();
    assert_eq!(character.skills, [DINGPIN, OLDFAEN]);

    // A red result turns the owner over but does not lock the target; the discarded
    // basic category alone prevents another basic cost.
    let (mut red, owner, _target) = setup(20_260_729_01);
    let basic = inject(
        &mut red,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        98_001,
    );
    judgment(&mut red, Suit::Heart, 98_002);
    let (mut red, first) = activate(red, owner, owner, basic, 1);
    assert!(
        matches!(first, CommandResult::Accepted { .. }),
        "unexpected first Dingpin result: {first:?}"
    );
    assert!(red.players.get(owner).unwrap().turned_over);
    let repeated_basic = inject(
        &mut red,
        owner,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Diamond,
        98_003,
    );
    let (mut red, repeated) = activate(red, owner, owner, repeated_basic, 2);
    assert!(matches!(repeated, CommandResult::Rejected { .. }));
    let trick = inject(
        &mut red,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Club,
        98_004,
    );
    judgment(&mut red, Suit::Diamond, 98_005);
    let (_red, second) = activate(red, owner, owner, trick, 3);
    assert!(matches!(second, CommandResult::Accepted { .. }));

    // A black result draws current lost HP and only then locks that target.
    let (mut black, owner, target) = setup(20_260_729_02);
    {
        let target_state = black.players.get_mut(target).unwrap();
        target_state.max_hp = 4;
        target_state.hp = 2;
    }
    let before = black.hand_count(target);
    let equip = inject(
        &mut black,
        owner,
        ZoneKind::Hand,
        CardKind::Weapon,
        Suit::Heart,
        98_011,
    );
    judgment(&mut black, Suit::Spade, 98_012);
    let (mut black, result) = activate(black, owner, target, equip, 4);
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert_eq!(black.hand_count(target), before + 2);
    let basic = inject(
        &mut black,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        98_013,
    );
    let (_black, repeated_target) = activate(black, owner, target, basic, 5);
    assert!(matches!(repeated_target, CommandResult::Rejected { .. }));

    // Card-use history is independent from Dingpin's own costs.
    let (mut used, owner, target) = setup(20_260_729_03);
    let used_trick = inject(
        &mut used,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Heart,
        98_021,
    );
    let mut tx = Transaction::begin(used);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: owner,
            card: used_trick,
            targets: vec![],
        }],
    )
    .unwrap();
    let mut used = tx.commit().state;
    let repeated_trick = inject(
        &mut used,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Club,
        98_022,
    );
    let (_used, repeated) = activate(used, owner, target, repeated_trick, 6);
    assert!(matches!(repeated, CommandResult::Rejected { .. }));
}
}

mod original_yijiang_duodao {
//! Golden coverage for 潘璋马忠「夺刀」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DUODAO: u32 = 4_208_671_631;
const ANJIAN: u32 = 3_402_374_234;
const PANZHANGMAZHONG: u32 = 3_150_630_034;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/duodao.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    source: PlayerId,
    owner: PlayerId,
    cost: CardId,
    weapon: Option<CardId>,
    armor: CardId,
}

fn add_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 5,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn damage_with_sha_and_pause(seed: u64, include_weapon: bool) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let source = engine.state().turn.current;
    let owner = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != source)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 10;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 10;

    let mut setup: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .flat_map(|player| tx.state().players.get(player).unwrap().skills.clone())
        .map(|skill| EffectRequest::RemoveSkill { skill })
        .collect();
    setup.extend([
        EffectRequest::DiscardFromHand {
            player: source,
            count: u32::MAX,
        },
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DUODAO),
        },
    ]);
    run_effects(&mut tx, setup).unwrap();

    let cost = add_card(
        &mut tx,
        owner,
        ZoneKind::Hand,
        DUODAO + 1,
        CardKind::TrickWuzhong,
    );
    let sha = add_card(&mut tx, source, ZoneKind::Hand, DUODAO + 2, CardKind::Sha);
    let weapon = include_weapon.then(|| {
        add_card(
            &mut tx,
            source,
            ZoneKind::Equip,
            DUODAO + 3,
            CardKind::Weapon,
        )
    });
    let armor = add_card(
        &mut tx,
        source,
        ZoneKind::Equip,
        DUODAO + 4,
        CardKind::Armor,
    );

    let (state, pipeline) = apply_command(
        tx.commit().state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed + 1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![owner],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 9);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardOneThenGainTargetEquipment
    );
    assert_eq!(
        state.resolution.continuation_payload,
        vec![owner.raw(), source.raw(), 1]
    );
    Setup {
        state,
        source,
        owner,
        cost,
        weapon,
        armor,
    }
}

fn resolve(
    setup: Setup,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, CommandResult) {
    let choice_id = ChoiceId::new(setup.state.resolution.pending_choice_id);
    let (state, pipeline) = apply_command(
        setup.state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    (state, pipeline.result)
}

#[test]
fn package_publishes_complete_character_in_abi53_package() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == DUODAO && skill.name == "duodao"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == PANZHANGMAZHONG)
        .unwrap();
    assert_eq!(character.skills, vec![DUODAO, ANJIAN]);
    let compiled = content_compiler::compile_source_named("duodao", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn valid_cost_is_discarded_then_weapon_is_gained_but_armor_stays() {
    let setup = damage_with_sha_and_pause(2_026_071_921, true);
    let owner = setup.owner;
    let source = setup.source;
    let cost = setup.cost;
    let weapon = setup.weapon.unwrap();
    let armor = setup.armor;
    let (state, result) = resolve(setup, vec![cost.raw()], false, 1);
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let source_equip = state.zones.find(ZoneKind::Equip, source).unwrap();
    assert_eq!(state.cards.get(cost).unwrap().zone, discard);
    assert_eq!(state.cards.get(weapon).unwrap().zone, owner_hand);
    assert_eq!(state.cards.get(weapon).unwrap().owner, owner);
    assert_eq!(state.cards.get(armor).unwrap().zone, source_equip);
    assert!(!state.has_pending_choice());
}

#[test]
fn cancelling_keeps_cost_and_weapon_in_place() {
    let setup = damage_with_sha_and_pause(2_026_071_922, true);
    let owner_hand = setup.state.zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let source_equip = setup
        .state
        .zones
        .find(ZoneKind::Equip, setup.source)
        .unwrap();
    let cost = setup.cost;
    let weapon = setup.weapon.unwrap();
    let (state, result) = resolve(setup, Vec::new(), true, 2);
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(cost).unwrap().zone, owner_hand);
    assert_eq!(state.cards.get(weapon).unwrap().zone, source_equip);
    assert!(!state.has_pending_choice());
}

#[test]
fn paying_without_a_weapon_still_discards_the_cost() {
    let setup = damage_with_sha_and_pause(2_026_071_923, false);
    let cost = setup.cost;
    let armor = setup.armor;
    let source_equip = setup
        .state
        .zones
        .find(ZoneKind::Equip, setup.source)
        .unwrap();
    let (state, result) = resolve(setup, vec![cost.raw()], false, 3);
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(cost).unwrap().zone, discard);
    assert_eq!(state.cards.get(armor).unwrap().zone, source_equip);
}

#[test]
fn forged_foreign_cost_is_rejected_atomically() {
    let setup = damage_with_sha_and_pause(2_026_071_924, true);
    let forged = setup.weapon.unwrap();
    let before = canonical_state_hash(&setup.state);
    let owner = setup.owner;
    let choice_id = ChoiceId::new(setup.state.resolution.pending_choice_id);
    let (state, pipeline) = apply_command(
        setup.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![forged.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    assert!(state.has_pending_choice());
}
}

mod original_yijiang_enyuan2 {
//! Golden for 一将成名法正「恩怨」的回复来源语义。

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const ENYUAN2: u32 = 3_002_282_592;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ENYUAN2 && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(3_002_282_592),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 20_260_714,
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
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ENYUAN2),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, source)
}

#[test]
fn other_living_recovery_source_draws_actual_recovered_amount_only() {
    let (mut tx, owner, source) = isolated_state();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let source_hand = tx.state().hand_count(source);

    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source,
            player: owner,
            amount: 2,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(source), source_hand + 2);

    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let owner_hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: owner,
            player: owner,
            amount: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), owner_hand);

    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    tx.state_mut().players.get_mut(source).unwrap().alive = false;
    let source_hand = tx.state().hand_count(source);
    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source,
            player: owner,
            amount: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(source), source_hand);
}
}

mod original_yijiang_fuli {
//! Golden for 一将成名廖化「伏枥」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const FULI: u32 = 3_748_306_630;
const XINFULI: u32 = 3_062_292_376;
const WEI: u32 = 1;
const SHU: u32 = 8;
const WU: u32 = 15;
const QUN: u32 = 23;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn isolated(skill_def: u32, character_defs: [u32; 4]) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_716),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 20_260_716,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let mut players = vec![owner];
    players.extend(
        engine
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|player| *player != owner),
    );
    let mut tx = Transaction::begin(engine.state().clone());
    for (player, character_def) in players.iter().copied().zip(character_defs) {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .character_def = character_def;
    }
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    (tx, owner, players)
}

fn offer(tx: &mut Transaction, owner: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve(
    tx: &Transaction,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> engine_core::state::GameState {
    let (state, result) = apply_command(
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
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn has_skill(state: &engine_core::state::GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

#[test]
fn committed_package_exposes_optional_limited_dying_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 6);
    for skill_def in [FULI, XINFULI] {
        assert!(package
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == skill_def && !skill.locked));
        let trigger = package
            .trigger_index
            .iter()
            .find(|trigger| trigger.skill_def_id == skill_def)
            .unwrap();
        assert_eq!(
            (
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional
            ),
            ("dying", "owner_is_target", true)
        );
    }
}

#[test]
fn duplicate_groups_count_once_and_acceptance_recovers_turns_over_and_consumes() {
    let (mut tx, owner, _) = isolated(FULI, [WEI, WEI, SHU, WU]);
    assert_eq!(
        query(
            tx.state(),
            &QueryRequest::AliveGroupCount,
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(3)
    );
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 1);
    assert_eq!(accepted.players.get(owner).unwrap().hp, 3);
    assert!(accepted.players.get(owner).unwrap().turned_over);
    assert!(!has_skill(&accepted, owner, FULI));
}

#[test]
fn declining_preserves_state_and_dead_players_groups_do_not_count() {
    let (mut tx, owner, players) = isolated(FULI, [WEI, SHU, WU, QUN]);
    tx.state_mut().players.get_mut(players[3]).unwrap().alive = false;
    assert_eq!(
        query(
            tx.state(),
            &QueryRequest::AliveGroupCount,
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(3)
    );
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let declined = resolve(&tx, owner, false, 2);
    assert_eq!(declined.players.get(owner).unwrap().hp, 0);
    assert!(!declined.players.get(owner).unwrap().turned_over);
    assert!(has_skill(&declined, owner, FULI));
}

#[test]
fn xinfuli_recovers_and_draws_to_groups_then_turns_over_above_two() {
    let (mut tx, owner, _) = isolated(XINFULI, [WEI, WEI, SHU, WU]);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 3);
    assert_eq!(accepted.players.get(owner).unwrap().hp, 3);
    assert_eq!(accepted.hand_count(owner), 3);
    assert!(accepted.players.get(owner).unwrap().turned_over);
    assert!(!has_skill(&accepted, owner, XINFULI));
}

#[test]
fn xinfuli_does_not_reduce_hand_or_turn_over_at_two_groups() {
    let (mut tx, owner, _) = isolated(XINFULI, [WEI, WEI, SHU, SHU]);
    let hand = tx.state().hand_count(owner);
    assert!(hand > 2);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 4);
    assert_eq!(accepted.players.get(owner).unwrap().hp, 2);
    assert_eq!(accepted.hand_count(owner), hand);
    assert!(!accepted.players.get(owner).unwrap().turned_over);
    assert!(!has_skill(&accepted, owner, XINFULI));
}
}

mod original_yijiang_guyong {
//! Golden coverage for 顾雍「秉壹」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, ChoiceRequest, Command, CommandKind, CommandResult,
    DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const GUYONG: u32 = 3_099_069_622;
const SHENXING: u32 = 3_283_042_324;
const OLBINGYI: u32 = 2_703_324_677;
const OLBINGYI_SOURCE: &str =
    include_str!("../../../content/original/ported/yijiang/olbingyi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: [PlayerId; 3],
}

struct Step {
    state: GameState,
    events: Vec<DomainEvent>,
    choice: Option<ChoiceRequest>,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLBINGYI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 8;
        participant.max_hp = 8;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        others: [players[1], players[2], players[3]],
    }
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, suit: Suit) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn discard(state: GameState, player: PlayerId, card: CardId) -> Step {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player,
            cards: vec![card],
        }],
    );
    assert!(matches!(result, Ok(_) | Err(EffectError::ChoiceRequested)));
    let committed = tx.commit();
    Step {
        state: committed.state,
        events: committed.output.events,
        choice: committed.output.choice,
    }
}

fn olbingyi_instance(state: &GameState, owner: PlayerId) -> SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == OLBINGYI)
        .unwrap()
        .id
}

fn assert_optional(choice: &Option<ChoiceRequest>) {
    let choice = choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1]
    );
}

#[test]
fn package_publishes_complete_guyong_and_exact_abi56_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == GUYONG)
        .unwrap();
    assert_eq!(character.name_key, "guyong");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [SHENXING, OLBINGYI]);

    let compiled = content_compiler::compile_source_named("olbingyi", OLBINGYI_SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 56);
    for expected in compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn non_uniform_hand_is_publicly_revealed_without_drawing() {
    let mut setup = setup(2_026_073_601);
    let discarded = inject(&mut setup.state, setup.owner, 99_601, Suit::Club);
    let red = inject(&mut setup.state, setup.owner, 99_602, Suit::Heart);
    let black = inject(&mut setup.state, setup.owner, 99_603, Suit::Spade);
    let step = discard(setup.state, setup.owner, discarded);
    assert_optional(&step.choice);

    let before = step.state.hand_count(setup.owner);
    let (state, accepted) = resolve(step.state, setup.owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), before);
    assert!(accepted.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &vec![red, black] && to.is_empty()
    )));
    let skill = olbingyi_instance(&state, setup.owner);
    assert_eq!(state.skills.get(skill).unwrap().play_phase_fire_count, 1);
}

#[test]
fn uniform_hand_draws_selected_participants_by_seat_after_atomic_validation() {
    let mut setup = setup(2_026_073_602);
    let discarded = inject(&mut setup.state, setup.owner, 99_611, Suit::Club);
    let red_a = inject(&mut setup.state, setup.owner, 99_612, Suit::Heart);
    let red_b = inject(&mut setup.state, setup.owner, 99_613, Suit::Diamond);
    let step = discard(setup.state, setup.owner, discarded);
    let (state, accepted) = resolve(step.state, setup.owner, vec![1], false, 10);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(accepted.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &vec![red_a, red_b] && to.is_empty()
    )));
    let choice = accepted.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (1, 2, true)
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOthersAfterUniformHandReveal
    );

    let before_forged = canonical_state_hash(&state);
    let (unchanged, forged) = resolve(state, setup.owner, vec![setup.owner.raw()], false, 11);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&unchanged), before_forged);
    let (unchanged, duplicate) = resolve(
        unchanged,
        setup.owner,
        vec![setup.others[0].raw(), setup.others[0].raw()],
        false,
        12,
    );
    assert!(matches!(duplicate.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&unchanged), before_forged);

    let owner_before = unchanged.hand_count(setup.owner);
    let first_before = unchanged.hand_count(setup.others[0]);
    let middle_before = unchanged.hand_count(setup.others[1]);
    let last_before = unchanged.hand_count(setup.others[2]);
    let (state, drawn) = resolve(
        unchanged,
        setup.owner,
        vec![setup.others[2].raw(), setup.others[0].raw()],
        false,
        13,
    );
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), owner_before + 1);
    assert_eq!(state.hand_count(setup.others[0]), first_before + 1);
    assert_eq!(state.hand_count(setup.others[1]), middle_before);
    assert_eq!(state.hand_count(setup.others[2]), last_before + 1);
    assert_eq!(
        drawn
            .events
            .iter()
            .filter_map(|event| match event {
                DomainEvent::CardsDrawn { player, .. } => Some(*player),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [setup.owner, setup.others[0], setup.others[2]]
    );
}

#[test]
fn decline_does_not_consume_limit_and_acceptance_resets_on_next_phase() {
    let mut setup = setup(2_026_073_603);
    let first = inject(&mut setup.state, setup.owner, 99_621, Suit::Club);
    let second = inject(&mut setup.state, setup.owner, 99_622, Suit::Heart);
    let _red = inject(&mut setup.state, setup.owner, 99_623, Suit::Diamond);
    let _black = inject(&mut setup.state, setup.owner, 99_624, Suit::Spade);

    let step = discard(setup.state, setup.owner, first);
    assert_optional(&step.choice);
    let (state, declined) = resolve(step.state, setup.owner, vec![0], false, 20);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    let skill = olbingyi_instance(&state, setup.owner);
    assert_eq!(state.skills.get(skill).unwrap().play_phase_fire_count, 0);

    let step = discard(state, setup.owner, second);
    assert_optional(&step.choice);
    let (state, accepted) = resolve(step.state, setup.owner, vec![1], false, 21);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.skills.get(skill).unwrap().play_phase_fire_count, 1);

    let mut state = state;
    let third = inject(&mut state, setup.owner, 99_625, Suit::Club);
    let same_phase = discard(state, setup.owner, third);
    assert!(same_phase.choice.is_none());
    assert!(same_phase.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved { cards, .. } if cards == &vec![third]
    )));

    let mut tx = Transaction::begin(same_phase.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: setup.owner,
            phase: Phase::End.to_u8(),
        }],
    )
    .unwrap();
    let mut state = tx.commit().state;
    assert_eq!(state.skills.get(skill).unwrap().play_phase_fire_count, 0);
    let fourth = inject(&mut state, setup.owner, 99_626, Suit::Club);
    let next_phase = discard(state, setup.owner, fourth);
    assert_optional(&next_phase.choice);
}
}

mod original_yijiang_hanhaoshihuan {
//! Golden coverage for 韩浩史涣「慎断」「勇略」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const HANHAOSHIHUAN: u32 = 4_039_637_603;
const SHENDUAN: u32 = 2_370_759_068;
const YONGLVE: u32 = 2_407_381_445;
const ZHICHI2: u32 = 3_197_261_303;
const SHENDUAN_SOURCE: &str =
    include_str!("../../../content/original/ported/yijiang/shenduan.skill");
const YONGLVE_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/yonglve.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    far: PlayerId,
}

fn setup(seed: u64, skill_def: u32) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let far = players[2];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        far,
    }
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
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

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn open_yonglve(mut state: GameState, target: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = engine_core::state::Phase::Judge;
    let events = [DomainEvent::TurnStateChanged {
        current: target,
        phase: TurnPhaseTag::Judge,
    }];
    let effects = dispatch_for_events(&mut tx, &events, &mut TriggerFireRecord::new()).unwrap();
    assert!(matches!(
        run_effects(&mut tx, effects),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    state = tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage
    );
    state
}

#[test]
fn package_publishes_complete_hanhaoshihuan_and_abi53_package() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == HANHAOSHIHUAN)
        .unwrap();
    assert_eq!(character.skills, [SHENDUAN, YONGLVE]);

    for (name, source) in [("shenduan", SHENDUAN_SOURCE), ("yonglve", YONGLVE_SOURCE)] {
        let compiled = content_compiler::compile_source_named(name, source).unwrap();
        assert_eq!(compiled.manifest.rule_vm_abi_version, 51);
        for expected in compiled.modules {
            let actual = package
                .modules
                .iter()
                .find(|module| module.name == expected.name)
                .unwrap();
            assert_eq!(actual.logic_hash, expected.logic_hash);
        }
    }
}

#[test]
fn shenduan_repeats_over_one_discard_batch_and_ignores_bingliang_distance() {
    let mut setup = setup(2_026_073_101, SHENDUAN);
    let first = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        98_101,
        CardKind::Sha,
        Suit::Spade,
    );
    let second = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        98_102,
        CardKind::Shan,
        Suit::Club,
    );
    let red = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        98_103,
        CardKind::Tao,
        Suit::Heart,
    );
    let mut tx = Transaction::begin(setup.state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: setup.owner,
                cards: vec![first, second, red],
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let state = tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardedBlackBasicAsBingliang
    );
    assert_eq!(state.resolution.continuation_payload.first(), Some(&0));
    let (state, picked) = resolve(state, setup.owner, vec![first.raw()], false, 1);
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    let options: Vec<_> = picked
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(options.contains(&setup.far.raw()));

    let (state, used) = resolve(state, setup.owner, vec![setup.far.raw()], false, 2);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    let judge = state.zones.find(ZoneKind::Judge, setup.far).unwrap();
    assert_eq!(state.cards.get(first).unwrap().zone, judge);
    assert_eq!(
        state.cards.get(first).unwrap().kind,
        CardKind::TrickBingliang
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardedBlackBasicAsBingliang
    );
    assert_eq!(state.resolution.continuation_payload.first(), Some(&0));

    let (state, declined) = resolve(state, setup.owner, Vec::new(), true, 3);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(second).unwrap().zone, discard);
    assert_eq!(state.cards.get(red).unwrap().zone, discard);
}

#[test]
fn shenduan_rejects_a_forged_noncandidate_without_mutation() {
    let mut setup = setup(2_026_073_104, SHENDUAN);
    let black = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        98_401,
        CardKind::Sha,
        Suit::Spade,
    );
    let red = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        98_402,
        CardKind::Tao,
        Suit::Diamond,
    );
    let mut tx = Transaction::begin(setup.state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: setup.owner,
                cards: vec![black, red],
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let state = tx.commit().state;
    let before = canonical_state_hash(&state);
    let (after, rejected) = resolve(state, setup.owner, vec![red.raw()], false, 4);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&after), before);
}

#[test]
fn yonglve_hit_does_not_draw() {
    let mut setup = setup(2_026_073_102, YONGLVE);
    let judge = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Judge,
        98_201,
        CardKind::TrickLe,
        Suit::Heart,
    );
    let state = open_yonglve(setup.state, setup.target);
    let owner_hand = state.hand_count(setup.owner);
    let target_hp = state.players.get(setup.target).unwrap().hp;
    let (state, resolved) = resolve(state, setup.owner, vec![judge.raw()], false, 10);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, target_hp - 1);
    assert_eq!(state.hand_count(setup.owner), owner_hand);
}

#[test]
fn yonglve_dodged_exact_sha_draws_once() {
    let mut setup = setup(2_026_073_103, YONGLVE);
    let judge = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Judge,
        98_301,
        CardKind::TrickLe,
        Suit::Diamond,
    );
    let shan = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        98_302,
        CardKind::Shan,
        Suit::Heart,
    );
    let state = open_yonglve(setup.state, setup.target);
    let owner_hand = state.hand_count(setup.owner);
    let target_hp = state.players.get(setup.target).unwrap().hp;
    let (state, discarded) = resolve(state, setup.owner, vec![judge.raw()], false, 20);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RespondToSha
    );
    let (state, dodged) = resolve(state, setup.target, vec![shan.raw()], false, 21);
    assert!(matches!(dodged.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, target_hp);
    assert_eq!(state.hand_count(setup.owner), owner_hand + 1);
    assert!(state.resolution.sha_no_damage_draw_skills.is_empty());
}

#[test]
fn yonglve_draws_when_the_virtual_sha_is_not_legal() {
    let mut setup = setup(2_026_073_105, YONGLVE);
    let judge = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Judge,
        98_501,
        CardKind::TrickLe,
        Suit::Club,
    );
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: setup.target,
            skill_def: SkillDefId::new(ZHICHI2),
        }],
    )
    .unwrap();
    let state = open_yonglve(tx.commit().state, setup.target);
    let owner_hand = state.hand_count(setup.owner);
    let target_hp = state.players.get(setup.target).unwrap().hp;
    let (state, resolved) = resolve(state, setup.owner, vec![judge.raw()], false, 30);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, target_hp);
    assert_eq!(state.hand_count(setup.owner), owner_hand + 1);
}
}

mod original_yijiang_jingce {
//! Golden for 一将成名「精策」用牌历史语义。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const JINGCE: u32 = 2_443_825_940;
const XINJINGCE: u32 = 4_267_063_918;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn at_phase_with_used_cards(skill: u32, used: usize) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|entry| entry.skill_def_id == skill && !entry.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(skill as u64 + used as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714 + used as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|instance| EffectRequest::RemoveSkill { skill: instance.id })
        .collect();
    setup.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill),
    });
    setup.push(EffectRequest::SetTurnState {
        current: owner,
        phase: Phase::Play.to_u8(),
    });
    run_effects(&mut tx, setup).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;

    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, owner)
        .and_then(|zone| tx.state().zones.get(zone))
        .unwrap()
        .cards
        .clone();
    assert!(hand.len() >= used);
    for card in hand.into_iter().take(used) {
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: owner,
                card,
                targets: vec![],
            }],
        )
        .unwrap();
    }
    (tx, owner)
}

fn assert_threshold(skill: u32, trigger_phase: Phase) {
    let (mut below, owner) = at_phase_with_used_cards(skill, 1);
    let hand = below.state().hand_count(owner);
    run_effects(
        &mut below,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: trigger_phase.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(below.state().hand_count(owner), hand);
    assert!(!below.state().has_pending_choice());

    let (mut enough, owner) = at_phase_with_used_cards(skill, 2);
    let hand = enough.state().hand_count(owner);
    let paused = run_effects(
        &mut enough,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: trigger_phase.to_u8(),
        }],
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);

    let (state, result) = apply_command(
        enough.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill as u64),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(enough.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 2);
}

#[test]
fn jingce_checks_total_cards_used_when_play_phase_ends() {
    assert_threshold(JINGCE, Phase::Discard);
}

#[test]
fn xinjingce_checks_the_same_turn_history_when_end_phase_begins() {
    assert_threshold(XINJINGCE, Phase::End);
}
}

mod original_yijiang_juece {
//! Golden for 一将成名「绝策」：拥有者回合内对失去最后手牌的其他角色造成伤害。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const JUECE: u32 = 2_465_882_550;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn accept(tx: &Transaction, owner: PlayerId) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(JUECE as u64),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn only_last_hand_loss_during_the_owners_turn_offers_damage() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JUECE
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_alive_current_turn_other_event_player_from_hand_empty"
            && trigger.optional
    }));
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_715),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let (owner, other, third) = (players[0], players[1], players[2]);
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.extend([
        EffectRequest::DiscardFromHand {
            player: other,
            count: u32::MAX,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JUECE),
        },
        EffectRequest::DrawCards {
            player: other,
            count: 1,
        },
    ]);
    run_effects(&mut tx, setup).unwrap();

    let hand_zone = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state().hand_cards(other)[0];
    let hp = tx.state().players.get(other).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(hand_zone),
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(&tx, owner);
    assert_eq!(state.players.get(other).unwrap().hp, hp - 1);

    let mut tx = Transaction::begin(state);
    let juece = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == JUECE)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill { skill: juece },
            EffectRequest::DrawCards {
                player: other,
                count: 1,
            },
        ],
    )
    .unwrap();
    let hand_zone = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    let equipment = tx.state().hand_cards(other)[0];
    tx.state_mut().cards.get_mut(equipment).unwrap().kind = CardKind::Weapon;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![equipment],
                to: equip_zone,
                from: Some(hand_zone),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JUECE),
            },
            EffectRequest::MoveCards {
                cards: vec![equipment],
                to: discard,
                from: Some(equip_zone),
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, hp - 1);

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: third,
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::DrawCards {
                player: other,
                count: 1,
            },
        ],
    )
    .unwrap();
    let hand_zone = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let card = tx.state().hand_cards(other)[0];
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: discard,
            from: Some(hand_zone),
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(other).unwrap().hp, hp - 1);
}
}

mod original_yijiang_jueqing {
//! Golden for 一将成名张春华「绝情」的伤害改失去体力语义。

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::effect::{apply_effect, run_effects};
use engine_core::state::turn_flag;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const JUEQING: u32 = 3_481_582_878;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn isolated_state(with_jueqing: bool) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JUEQING && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(JUEQING as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
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
    if with_jueqing {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JUEQING),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, target)
}

#[test]
fn damage_becomes_equal_hp_loss_without_damage_bookkeeping() {
    let (mut tx, owner, target) = isolated_state(true);
    let hp = tx.state().players.get(target).unwrap().hp;
    let events = apply_effect(
        &mut tx,
        EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 2,
            is_lose_hp: false,
            nature: 1,
        },
    )
    .unwrap();

    assert!(matches!(
        events.as_slice(),
        [DomainEvent::DamageDealt {
            source,
            target: event_target,
            amount: 2,
            is_lose_hp: true,
            nature: 1,
        }] if *source == owner && *event_target == target
    ));
    assert_eq!(tx.state().players.get(target).unwrap().hp, hp - 2);
    assert!(!tx.state().damage_history.contains(&(owner, target)));
    assert_eq!(tx.state().turn.flag(turn_flag::DAMAGE_DEALT), 0);
    assert_eq!(tx.state().turn.damaged_player_count(), 0);

    let (mut ordinary, owner, target) = isolated_state(false);
    apply_effect(
        &mut ordinary,
        EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        },
    )
    .unwrap();
    assert!(ordinary.state().damage_history.contains(&(owner, target)));
    assert_eq!(ordinary.state().turn.flag(turn_flag::DAMAGE_DEALT), 1);
    assert_eq!(ordinary.state().turn.damaged_player_count(), 1);
}
}

mod original_yijiang_jyzongshi {
//! Golden coverage for 简雍「纵适」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JYZONGSHI: u32 = 2_485_328_360;
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn inject_discard(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: discard,
        owner,
        face_up: true,
    });
    state.zones.get_mut(discard).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, owner_is_initiator: bool) -> (GameState, PlayerId, PlayerId, CardId, CardId) {
    let package = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (initiator, target) = (seats[0], seats[1]);
    let owner = if owner_is_initiator {
        initiator
    } else {
        target
    };
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JYZONGSHI),
    });
    run_effects(&mut tx, effects).unwrap();
    let initiator_card = inject_discard(tx.state_mut(), initiator, 99_001);
    let target_card = inject_discard(tx.state_mut(), target, 99_002);
    (
        tx.commit().state,
        initiator,
        target,
        initiator_card,
        target_card,
    )
}

fn open_finished(
    state: GameState,
    initiator: PlayerId,
    target: PlayerId,
    initiator_card: CardId,
    target_card: CardId,
    winner: PlayerId,
) -> GameState {
    let mut tx = Transaction::begin(state);
    let event = DomainEvent::HandCompareFinished {
        player: initiator,
        target,
        player_card: initiator_card,
        target_card,
        winner,
    };
    match dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new()) {
        Ok(effects) => assert_eq!(
            run_effects(&mut tx, effects).unwrap_err(),
            EffectError::ChoiceRequested
        ),
        Err(error) => assert_eq!(error, EffectError::ChoiceRequested),
    }
    let committed = tx.commit();
    let choice = committed.output.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    committed.state
}

fn accept(state: GameState, owner: PlayerId, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_jyzongshi_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| { skill.skill_def_id == JYZONGSHI && skill.name == "jyzongshi" }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JYZONGSHI
            && trigger.event == "hand_compare_finished"
            && trigger.filter == "owner_hand_compare_participant_result_card_in_discard"
            && trigger.optional
    }));
}

#[test]
fn result_card_matches_initiator_target_win_loss_and_tie_semantics() {
    for (index, owner_is_initiator, winner_kind) in [
        (0_u64, true, 0_u8),
        (1, true, 1),
        (2, true, 2),
        (3, false, 0),
        (4, false, 1),
        (5, false, 2),
    ] {
        let (state, initiator, target, initiator_card, target_card) =
            setup(2_026_072_300 + index, owner_is_initiator);
        let owner = if owner_is_initiator {
            initiator
        } else {
            target
        };
        let opponent = if owner_is_initiator {
            target
        } else {
            initiator
        };
        let winner = match winner_kind {
            0 => owner,
            1 => opponent,
            _ => PlayerId::NONE,
        };
        let expected = if owner_is_initiator {
            if winner == owner {
                target_card
            } else {
                initiator_card
            }
        } else if winner == owner {
            initiator_card
        } else {
            target_card
        };
        let other = if expected == initiator_card {
            target_card
        } else {
            initiator_card
        };
        let state = open_finished(
            state,
            initiator,
            target,
            initiator_card,
            target_card,
            winner,
        );
        let state = accept(state, owner, 9_000 + index);
        let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        assert_eq!(state.cards.get(expected).unwrap().zone, hand);
        assert_eq!(state.cards.get(other).unwrap().zone, discard);
    }
}

#[test]
fn unavailable_designated_card_does_not_prompt() {
    let (mut state, initiator, target, initiator_card, target_card) = setup(2_026_072_306, true);
    let hand = state.zones.find(ZoneKind::Hand, initiator).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    state
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .retain(|card| *card != initiator_card);
    state
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .push(initiator_card);
    state.cards.get_mut(initiator_card).unwrap().zone = hand;
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::HandCompareFinished {
            player: initiator,
            target,
            player_card: initiator_card,
            target_card,
            winner: target,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!tx.state().has_pending_choice());
}
}

mod original_yijiang_lihuo {
//! Golden coverage for 程普「疠火」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LIHUO: u32 = 3_289_572_540;
const OLDQIANXI: u32 = 3_282_621_718;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/lihuo.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    extra: PlayerId,
    sha: CardId,
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
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

fn setup(seed: u64, kind: CardKind, oldqianxi: bool) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let target = seats[1];
    let extra = seats[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &seats {
        effects.push(EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIHUO),
    });
    if oldqianxi {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLDQIANXI),
        });
    }
    effects.push(EffectRequest::SetTurnState {
        current: owner,
        phase: Phase::Play.to_u8(),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
    }
    let sha = inject(
        tx.state_mut(),
        owner,
        ZoneKind::Hand,
        98_001,
        kind,
        Suit::Spade,
    );
    Setup {
        state: tx.commit().state,
        owner,
        target,
        extra,
        sha,
    }
}

fn play(setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        setup.state.clone(),
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: setup.sha,
                targets: vec![setup.target],
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
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
fn package_publishes_exact_abi53_lihuo_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let compiled = content_compiler::compile_source_named("lihuo", SOURCE).unwrap();
    assert_eq!(compiled.modules.len(), 2);
    for expected in &compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn converted_sha_adds_one_target_then_loses_hp_once_after_both_hits() {
    let setup = setup(2_026_072_001, CardKind::Sha, false);
    let (state, offered) = play(&setup, 1);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    let (state, add_choice) = resolve(state, setup.owner, vec![1], false, 2);
    assert!(matches!(add_choice.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddCurrentShaTarget
    );

    let (state, resolved) = resolve(state, setup.owner, vec![setup.extra.raw()], false, 3);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardTargetAdded {
            player,
            card,
            target,
            targets,
        } if *player == setup.owner
            && *card == setup.sha
            && *target == setup.extra
            && targets == &[setup.target, setup.extra]
    )));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, 3);
    assert_eq!(state.players.get(setup.extra).unwrap().hp, 3);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 3);
}

#[test]
fn fully_dodged_converted_sha_does_not_cost_hp() {
    let mut setup = setup(2_026_072_002, CardKind::Sha, false);
    let shan = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        98_002,
        CardKind::Shan,
        Suit::Heart,
    );
    let (state, _) = play(&setup, 10);
    let (state, _) = resolve(state, setup.owner, vec![1], false, 11);
    let (state, add_cancelled) = resolve(state, setup.owner, Vec::new(), true, 12);
    assert!(matches!(
        add_cancelled.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RespondToSha
    );
    let (state, dodged) = resolve(state, setup.target, vec![shan.raw()], false, 13);
    assert!(matches!(dodged.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, 4);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
}

#[test]
fn prevented_converted_sha_does_not_cost_hp() {
    let mut setup = setup(2_026_072_003, CardKind::Sha, true);
    inject(
        &mut setup.state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        98_003,
        CardKind::Other,
        Suit::Spade,
    );
    let (state, _) = play(&setup, 20);
    let (state, _) = resolve(state, setup.owner, vec![1], false, 21);
    let (state, _) = resolve(state, setup.owner, Vec::new(), true, 22);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, prevented) = resolve(state, setup.owner, vec![1], false, 23);
    assert!(matches!(prevented.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    // 旧潜袭降低体力上限时会把满体力同步夹到新上限；这不是该【杀】造成的伤害。
    assert_eq!(state.players.get(setup.target).unwrap().hp, 3);
    assert_eq!(state.players.get(setup.target).unwrap().max_hp, 3);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
}

#[test]
fn natural_fire_sha_may_add_a_target_without_conversion_hp_loss() {
    let setup = setup(2_026_072_004, CardKind::ShaFire, false);
    let (state, offered) = play(&setup, 30);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddCurrentShaTarget
    );
    let (state, cancelled) = resolve(state, setup.owner, Vec::new(), true, 31);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, 3);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
}
}

mod original_yijiang_liubiao {
//! Golden coverage for 刘表「界自守」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LIUBIAO: u32 = 4_020_542_401;
const QUANCONG: u32 = 3_313_708_633;
const CAOXIU: u32 = 3_428_883_420;
const REZISHOU: u32 = 4_220_178_807;
const REZISHOU2: u32 = 2_249_965_594;
const ZISHOU: u32 = 3_804_877_958;
const ZISHOU2: u32 = 4_152_430_403;
const ZONGSHI: u32 = 2_639_072_445;
const REZONGSHI: u32 = 3_490_528_973;
const RE_LIUBIAO: u32 = 1_069_046_603;
const OLDRE_LIUBIAO: u32 = 2_422_267_398;
const REZISHOU_SOURCE: &str =
    include_str!("../../../content/original/ported/refresh/rezishou.skill");
const ZISHOU_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/zishou.skill");
const REFRESH_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const YIJIANG_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const MOBILE_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");
const OLD_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH_PACKAGE).unwrap();
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG_PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&refresh).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&yijiang).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REZISHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().players.get_mut(owner).unwrap().character_def = LIUBIAO;
    tx.state_mut()
        .players
        .get_mut(players[1])
        .unwrap()
        .character_def = QUANCONG;
    tx.state_mut()
        .players
        .get_mut(players[2])
        .unwrap()
        .character_def = CAOXIU;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Judge;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        other,
    }
}

fn setup_zishou(seed: u64) -> Setup {
    let setup = setup(seed);
    let mut tx = Transaction::begin(setup.state);
    let rezishou = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == setup.owner && skill.def.raw() == REZISHOU)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill { skill: rezishou },
            EffectRequest::AddSkill {
                player: setup.owner,
                skill_def: SkillDefId::new(ZISHOU),
            },
        ],
    )
    .unwrap();
    Setup {
        state: tx.commit().state,
        owner: setup.owner,
        other: setup.other,
    }
}

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn enter_draw(state: GameState, owner: PlayerId, accept: bool) -> (GameState, PipelineResult) {
    let (state, offered) = submit(state, owner, CommandKind::EndPhase, 1);
    assert_eq!(offered.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    resolve(state, owner, vec![u32::from(accept)], 2)
}

fn deal_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
) -> (GameState, Result<Vec<DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    (tx.commit().state, result)
}

#[test]
fn packages_publish_complete_liubiao_dependency_and_exact_abi61_module() {
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH_PACKAGE).unwrap();
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG_PACKAGE).unwrap();
    assert!(refresh.manifest.rule_vm_abi_version >= 251);
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert!(yijiang.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "refresh"
            && dependency.version == "0.1.0"
    }));
    // 上游 refresh 已移除 liubiao 角色（仅存 xin_liubiao），仅验证技能模块契约。
    let compiled = content_compiler::compile_source_named("rezishou", REZISHOU_SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 61);
    for expected in compiled.modules {
        let actual = refresh
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn declining_rezishou_keeps_the_ordinary_single_draw_batch() {
    let setup = setup(2_026_074_101);
    let before = setup.state.hand_count(setup.owner);
    let (state, result) = enter_draw(setup.state, setup.owner, false);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), before + 2);
    assert!(!has_skill(&state, setup.owner, REZISHOU2));
    let draws: Vec<_> = result
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::CardsDrawn { player, cards } if *player == setup.owner => {
                Some(cards.len())
            }
            _ => None,
        })
        .collect();
    assert_eq!(draws, [2]);
}

#[test]
fn accepting_rezishou_adds_alive_groups_to_one_draw_and_consumes_the_bonus() {
    let setup = setup(2_026_074_102);
    let groups = engine_core::query::alive_group_count(&setup.state);
    assert_eq!(groups, 3);
    let before = setup.state.hand_count(setup.owner);
    let (state, result) = enter_draw(setup.state, setup.owner, true);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let expected = 2 + groups as u32;
    assert_eq!(state.hand_count(setup.owner), before + expected);
    assert!(has_skill(&state, setup.owner, REZISHOU2));
    assert_eq!(state.turn.flag(turn_flag::CURRENT_DRAW_COUNT_BONUS), 0);
    let draws: Vec<_> = result
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::CardsDrawn { player, cards } if *player == setup.owner => {
                Some(cards.len() as u32)
            }
            _ => None,
        })
        .collect();
    assert_eq!(draws, [expected]);
}

#[test]
fn rezishou2_prevents_outgoing_damage_to_others_but_not_self_damage() {
    let setup = setup(2_026_074_103);
    let (state, _) = enter_draw(setup.state, setup.owner, true);
    let other_hp = state.players.get(setup.other).unwrap().hp;
    let (state, other_result) = deal_damage(state, setup.owner, setup.other);
    assert!(other_result.is_ok());
    assert_eq!(state.players.get(setup.other).unwrap().hp, other_hp);

    let owner_hp = state.players.get(setup.owner).unwrap().hp;
    let (state, self_result) = deal_damage(state, setup.owner, setup.owner);
    assert!(self_result.is_ok());
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp - 1);
}

#[test]
fn rezishou2_expires_when_the_owners_turn_ends() {
    let setup = setup(2_026_074_104);
    let (mut state, _) = enter_draw(setup.state, setup.owner, true);
    assert!(has_skill(&state, setup.owner, REZISHOU2));
    state.turn.phase = Phase::End;
    let (state, result) = submit(state, setup.owner, CommandKind::EndPhase, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!has_skill(&state, setup.owner, REZISHOU2));
    assert_ne!(state.turn.current, setup.owner);
}

#[test]
fn zishou_package_closes_both_legacy_liubiao_characters() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG_PACKAGE).unwrap();
    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE_PACKAGE).unwrap();
    let old = content_schema::ContentPackage::from_json_bytes(OLD_PACKAGE).unwrap();
    let compiled = content_compiler::compile_source_named("zishou", ZISHOU_SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 61);
    for expected in compiled.modules {
        let actual = yijiang
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
    assert!(mobile.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "yijiang"
            && dependency.version == "0.1.0"
    }));
    let re_liubiao = mobile
        .character_index
        .iter()
        .find(|character| character.id == RE_LIUBIAO)
        .unwrap();
    assert_eq!(re_liubiao.name_key, "re_liubiao");
    assert_eq!(re_liubiao.skills, [ZISHOU, REZONGSHI]);
    let oldre_liubiao = old
        .character_index
        .iter()
        .find(|character| character.id == OLDRE_LIUBIAO)
        .unwrap();
    assert_eq!(oldre_liubiao.name_key, "oldre_liubiao");
    assert_eq!(oldre_liubiao.skills, [ZISHOU, ZONGSHI]);
}

#[test]
fn zishou_draws_by_alive_groups_locks_targets_and_expires_at_turn_end() {
    let setup = setup_zishou(2_026_074_105);
    let groups = engine_core::query::alive_group_count(&setup.state);
    let before = setup.state.hand_count(setup.owner);
    let (mut state, result) = enter_draw(setup.state, setup.owner, true);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), before + 2 + groups as u32);
    assert!(has_skill(&state, setup.owner, ZISHOU2));
    assert!(engine_core::query::card_targets_self_only_for(
        &state,
        setup.owner
    ));

    state.turn.phase = Phase::End;
    let (state, result) = submit(state, setup.owner, CommandKind::EndPhase, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!has_skill(&state, setup.owner, ZISHOU2));
}
}

mod original_yijiang_longyin {
//! Golden coverage for 关平「龙吟」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LONGYIN: u32 = 3_578_226_944;
const GUANPING: u32 = 2_957_024_910;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/longyin.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    owner: PlayerId,
    sha: CardId,
    cost: CardId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind, suit: Suit) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, suit: Suit) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let source = tx.state().turn.current;
    let target = players
        .iter()
        .copied()
        .find(|player| *player != source)
        .unwrap();
    let owner = players
        .iter()
        .copied()
        .find(|player| *player != source && *player != target)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in players {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(LONGYIN),
        },
        EffectRequest::SetTurnState {
            current: source,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let sha = inject(tx.state_mut(), source, 93_001, CardKind::Sha, suit);
    let cost = inject(tx.state_mut(), owner, 93_002, CardKind::Jiu, Suit::Club);
    Setup {
        state: tx.commit().state,
        source,
        target,
        owner,
        sha,
        cost,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

fn use_sha_and_pay(setup: Setup, key: u64) -> GameState {
    let (state, offered) = apply_command(
        setup.state,
        Command {
            actor: setup.source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: setup.sha,
                targets: vec![setup.target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    assert_eq!(state.resolution.pending_observer, setup.owner.raw());

    let (state, discard) = resolve(state, setup.owner, vec![1], key + 1);
    assert!(matches!(discard.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardOneForShaRefundAndDrawIfRed
    );
    assert_eq!(state.resolution.pending_observer, setup.owner.raw());

    let (state, paid) = resolve(state, setup.owner, vec![setup.cost.raw()], key + 2);
    assert!(matches!(paid.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    state
}

#[test]
fn package_publishes_complete_guanping_and_exact_abi53_package() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == GUANPING)
        .unwrap();
    assert_eq!(character.skills, [LONGYIN]);
    let compiled = content_compiler::compile_source_named("longyin", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn black_sha_refunds_its_count_after_cost_without_drawing() {
    let setup = setup(2_026_071_931, Suit::Spade);
    let owner = setup.owner;
    let state = use_sha_and_pay(setup, 10);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 0);
    assert_eq!(state.hand_count(owner), 0);
}

#[test]
fn red_sha_refunds_its_count_after_cost_and_draws_one() {
    let setup = setup(2_026_071_932, Suit::Heart);
    let owner = setup.owner;
    let state = use_sha_and_pay(setup, 20);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 0);
    assert_eq!(state.hand_count(owner), 1);
}

#[test]
fn refund_is_idempotent_and_never_decrements_an_uncounted_sha() {
    let mut counted = setup(2_026_071_933, Suit::Spade);
    counted.state.turn.set_flag(turn_flag::SHA_PLAYED, 1);
    counted.state.resolution.resume_kind = ContinuationKind::ResumePlayedSha;
    // saved-Sha flags: bit 0 respondable, bit 1 counted, bit 2 refunded.
    counted.state.resolution.resume_payload = vec![
        counted.source.raw(),
        counted.target.raw(),
        1,
        0,
        1 | 2,
        counted.sha.raw(),
    ];
    let refund = EffectRequest::RefundCurrentShaUseAndDrawIfRed {
        player: counted.owner,
        sha_user: counted.source,
        card: counted.sha,
        skill: SkillId::NONE,
    };
    let mut tx = Transaction::begin(counted.state);
    run_effects(&mut tx, vec![refund.clone(), refund]).unwrap();
    assert_eq!(tx.state().turn.flag(turn_flag::SHA_PLAYED), 0);
    assert_eq!(tx.state().resolution.resume_payload[4] & 4, 4);

    let mut uncounted = setup(2_026_071_934, Suit::Spade);
    uncounted.state.turn.set_flag(turn_flag::SHA_PLAYED, 1);
    uncounted.state.resolution.resume_kind = ContinuationKind::ResumePlayedSha;
    uncounted.state.resolution.resume_payload = vec![
        uncounted.source.raw(),
        uncounted.target.raw(),
        1,
        0,
        1,
        uncounted.sha.raw(),
    ];
    let mut tx = Transaction::begin(uncounted.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RefundCurrentShaUseAndDrawIfRed {
            player: uncounted.owner,
            sha_user: uncounted.source,
            card: uncounted.sha,
            skill: SkillId::NONE,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().turn.flag(turn_flag::SHA_PLAYED), 1);
    assert_eq!(tx.state().resolution.resume_payload[4], 1);
}

#[test]
fn missing_cost_at_effect_time_cannot_refund_or_draw() {
    let setup = setup(2_026_071_935, Suit::Heart);
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: setup.owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 1);
    tx.state_mut().resolution.resume_kind = ContinuationKind::ResumePlayedSha;
    tx.state_mut().resolution.resume_payload = vec![
        setup.source.raw(),
        setup.target.raw(),
        1,
        0,
        1 | 2,
        setup.sha.raw(),
    ];
    run_effects(
        &mut tx,
        vec![EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed {
            player: setup.owner,
            sha_user: setup.source,
            card: setup.sha,
            skill: SkillId::NONE,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().turn.flag(turn_flag::SHA_PLAYED), 1);
    assert_eq!(tx.state().hand_count(setup.owner), 0);
}
}

mod original_yijiang_olbenxi {
//! Golden coverage for 吴懿「奔袭」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, sha_unrespondable_for_card, QueryContext};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const WUYI: u32 = 3_716_905_283;
const OLBENXI: u32 = 2_954_453_328;
const OLBENXI_EFFECT: u32 = 2_946_386_821;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/olbenxi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    far: PlayerId,
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
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

fn setup(seed: u64, players: u8) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, players, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let target = seats[1];
    let far = seats[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLBENXI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        far,
    }
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn distance(state: &GameState, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("distance query failed: {other:?}"),
    }
}

fn helper_id(state: &GameState, owner: PlayerId) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)?
        .skills
        .iter()
        .copied()
        .find(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == OLBENXI_EFFECT)
        })
}

fn play_card(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    command(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets,
            card_targets: Vec::new(),
        },
        key,
    )
}

#[test]
fn package_publishes_complete_wuyi_and_exact_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == WUYI)
        .unwrap();
    assert_eq!(character.skills, [OLBENXI]);
    let compiled = content_compiler::compile_source_named("olbenxi", SOURCE).unwrap();
    assert_eq!(compiled.modules.len(), 3);
    for expected in &compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn each_own_turn_card_use_reduces_distance_and_the_value_is_zero_off_turn() {
    let mut setup = setup(2_026_072_901, 4);
    assert_eq!(distance(&setup.state, setup.owner, setup.far), 2);
    let weapon = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_301,
        CardKind::Weapon,
        Suit::Spade,
    );
    let (mut state, played) = play_card(setup.state, setup.owner, weapon, Vec::new(), 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.turn.flag(turn_flag::card_used_by_player(setup.owner)),
        1
    );
    assert_eq!(distance(&state, setup.owner, setup.far), 1);
    state.turn.current = setup.target;
    assert_eq!(distance(&state, setup.owner, setup.far), 2);
}

#[test]
fn trigger_waits_until_every_other_player_is_within_one() {
    let mut setup = setup(2_026_072_902, 6);
    let sha = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_302,
        CardKind::Sha,
        Suit::Spade,
    );
    let (state, played) = play_card(setup.state, setup.owner, sha, vec![setup.target], 2);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(state
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != setup.owner)
        .any(|player| distance(&state, setup.owner, player) > 1));
    assert!(helper_id(&state, setup.owner).is_none());
}

#[test]
fn forced_options_reject_forgery_then_add_targets_to_sha_and_ordinary_trick() {
    let mut sha_setup = setup(2_026_072_903, 4);
    let sha = inject(
        &mut sha_setup.state,
        sha_setup.owner,
        ZoneKind::Hand,
        99_303,
        CardKind::Sha,
        Suit::Diamond,
    );
    let (state, offered) = play_card(
        sha_setup.state,
        sha_setup.owner,
        sha,
        vec![sha_setup.target],
        3,
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCurrentCardUseEnhancements
    );
    let options: Vec<_> = offered
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(options, [0, 1, 2, 3]);

    let (state, forged) = resolve(state, sha_setup.owner, vec![1, 1], false, 4);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert!(state.has_pending_choice());
    assert!(helper_id(&state, sha_setup.owner).is_none());

    let (state, picked) = resolve(state, sha_setup.owner, vec![0, 3], false, 5);
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddCurrentCardTarget
    );
    let target = picked.choice.unwrap().options[0].token;
    let added_target = PlayerId::new(target);
    let (state, added) = resolve(state, sha_setup.owner, vec![target], false, 6);
    assert!(matches!(added.result, CommandResult::Accepted { .. }));
    assert!(added.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardTargetAdded { card, target, targets, .. }
            if *card == sha && *target == added_target
                && targets == &[sha_setup.target, added_target]
    )));
    assert_eq!(state.players.get(sha_setup.target).unwrap().hp, 9);
    assert_eq!(state.players.get(added_target).unwrap().hp, 9);
    assert_eq!(state.hand_count(sha_setup.owner), 2);

    let mut trick_setup = setup(2_026_072_904, 4);
    let duel = inject(
        &mut trick_setup.state,
        trick_setup.owner,
        ZoneKind::Hand,
        99_304,
        CardKind::TrickJuedou,
        Suit::Club,
    );
    let (state, _) = play_card(
        trick_setup.state,
        trick_setup.owner,
        duel,
        vec![trick_setup.target],
        7,
    );
    let (state, picked) = resolve(state, trick_setup.owner, vec![0], false, 8);
    let target = picked.choice.unwrap().options[0].token;
    let added_target = PlayerId::new(target);
    let (state, added) = resolve(state, trick_setup.owner, vec![target], false, 9);
    assert!(matches!(added.result, CommandResult::Accepted { .. }));
    assert!(added.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardTargetAdded { card, target, targets, .. }
            if *card == duel && *target == added_target
                && targets == &[trick_setup.target, added_target]
    )));
    assert_eq!(state.players.get(trick_setup.target).unwrap().hp, 9);
    assert_eq!(state.players.get(added_target).unwrap().hp, 9);
}

#[test]
fn exact_card_can_ignore_armor_and_responses_then_helper_expires_at_turn_end() {
    let mut setup = setup(2_026_072_905, 4);
    inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Equip,
        engine_core::content::DEF_TENGJIA,
        CardKind::Armor,
        Suit::Club,
    );
    let shan = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        99_305,
        CardKind::Shan,
        Suit::Heart,
    );
    let fire_sha = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_306,
        CardKind::ShaFire,
        Suit::Diamond,
    );
    let (state, _) = play_card(setup.state, setup.owner, fire_sha, vec![setup.target], 10);
    let hp = state.players.get(setup.target).unwrap().hp;
    let (mut state, enhanced) = resolve(state, setup.owner, vec![1, 2], false, 11);
    assert!(matches!(enhanced.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
    assert_eq!(state.cards.get(shan).unwrap().owner, setup.target);
    assert!(sha_unrespondable_for_card(&state, setup.owner, fire_sha));

    let other_fire_sha = inject(
        &mut state,
        setup.owner,
        ZoneKind::Hand,
        99_307,
        CardKind::ShaFire,
        Suit::Heart,
    );
    assert!(!sha_unrespondable_for_card(
        &state,
        setup.owner,
        other_fire_sha
    ));
    let hp = state.players.get(setup.target).unwrap().hp;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: setup.owner,
            target: setup.target,
            card: other_fire_sha,
            amount: 1,
            nature: rule_api::damage_nature::FIRE,
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(setup.target).unwrap().hp, hp - 2);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: setup.owner,
            phase: TurnPhaseTag::End,
            damage_sources: 1,
            damaged_players: 1,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(helper_id(tx.state(), setup.owner).is_none());
}

#[test]
fn unrespondable_ordinary_trick_skips_wuxie_for_that_exact_use() {
    let mut setup = setup(2_026_072_906, 4);
    let wuxie = inject(
        &mut setup.state,
        setup.far,
        ZoneKind::Hand,
        99_308,
        CardKind::TrickWuxie,
        Suit::Spade,
    );
    let duel = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_309,
        CardKind::TrickJuedou,
        Suit::Diamond,
    );
    let (state, _) = play_card(setup.state, setup.owner, duel, vec![setup.target], 12);
    let hp = state.players.get(setup.target).unwrap().hp;
    let (state, enhanced) = resolve(state, setup.owner, vec![2], false, 13);
    assert!(matches!(enhanced.result, CommandResult::Accepted { .. }));
    assert_ne!(
        state.resolution.continuation_kind,
        ContinuationKind::WuxieWindow,
        "the original Wuxie window must be skipped"
    );
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
    assert_eq!(state.cards.get(wuxie).unwrap().owner, setup.far);
}
}

mod original_yijiang_old_fuhun {
//! Golden coverage for 旧关兴张苞「父魂」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLD_FUHUN: u32 = 4_126_064_689;
const OLD_GUANZHANG: u32 = 3_002_051_488;
const WUSHENG: u32 = 16;
const PAOXIAO: u32 = 23;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/old_fuhun.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    top: [CardId; 2],
}

fn setup(first: Suit, second: Suit, permanent_wusheng: bool) -> Setup {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let fuhun = content_compiler::compile_source_named("old-fuhun-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut state.content).load_package(&fuhun).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_072_702, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLD_FUHUN),
    });
    if permanent_wusheng {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(WUSHENG),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    tx.state_mut()
        .turn
        .set_flag(turn_flag::DRAW_PHASE_DRAW_ZEROED, 0);

    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let cards = tx.state().zones.get(draw).unwrap().cards.clone();
    let top = [cards[cards.len() - 2], cards[cards.len() - 1]];
    tx.state_mut().cards.get_mut(top[0]).unwrap().suit = first;
    tx.state_mut().cards.get_mut(top[1]).unwrap().suit = second;
    Setup { tx, owner, top }
}

fn fire_draw(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    let event = DomainEvent::TurnStateChanged {
        current: owner,
        phase: TurnPhaseTag::Draw,
    };
    let effects = dispatch_for_events(tx, &[event], &mut TriggerFireRecord::new())?;
    run_effects(tx, effects).map(|_| ())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == def)
}

#[test]
fn packages_publish_complete_character_and_skill_without_legacy_js() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert!(yijiang.manifest.rule_vm_abi_version >= 287);
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLD_FUHUN && skill.name == "old_fuhun"));
    assert!(yijiang
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "standard"));

    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    let character = old
        .character_index
        .iter()
        .find(|character| character.id == OLD_GUANZHANG)
        .unwrap();
    assert_eq!(character.skills, [OLD_FUHUN]);
    assert!(old
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "yijiang"));
    assert!(!serde_json::to_string(&(yijiang, old))
        .unwrap()
        .contains(".js"));
}

#[test]
fn replaced_draw_suppresses_prompt_and_decline_preserves_normal_draw() {
    let Setup { mut tx, owner, top } = setup(Suit::Heart, Suit::Spade, false);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::DRAW_PHASE_DRAW_ZEROED, 1);
    fire_draw(&mut tx, owner).unwrap();
    assert!(!tx.state().has_pending_choice());
    assert!(top
        .iter()
        .all(|card| tx.state().cards.get(*card).unwrap().zone
            == tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .unwrap()));

    tx.state_mut()
        .turn
        .set_flag(turn_flag::DRAW_PHASE_DRAW_ZEROED, 0);
    assert_eq!(fire_draw(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let state = tx.commit().state;
    let (state, result) = resolve(state, owner, Vec::new(), true, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.flag(turn_flag::DRAW_PHASE_DRAW_ZEROED), 0);
    assert!(top.iter().all(|card| {
        state.cards.get(*card).unwrap().zone
            == state
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .unwrap()
    }));
}

#[test]
fn same_color_cards_are_publicly_revealed_and_gained_without_skills() {
    let Setup { mut tx, owner, top } = setup(Suit::Heart, Suit::Diamond, false);
    let hand_before = tx.state().hand_count(owner);
    assert_eq!(fire_draw(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let (state, result) = resolve(tx.commit().state, owner, vec![1], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.flag(turn_flag::DRAW_PHASE_DRAW_ZEROED), 1);
    assert_eq!(state.hand_count(owner), hand_before + 2);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert!(top
        .iter()
        .all(|card| state.cards.get(*card).unwrap().zone == hand));
    assert!(result.events.iter().any(
        |event| matches!(event, DomainEvent::CardsRevealed { cards, to } if cards.len() == 2 && top.iter().all(|card| cards.contains(card)) && to.is_empty())
    ));
    assert!(!has_skill(&state, owner, WUSHENG));
    assert!(!has_skill(&state, owner, PAOXIAO));
}

#[test]
fn different_colors_grant_both_skills_after_gain_and_expire_at_turn_end() {
    let Setup { mut tx, owner, top } = setup(Suit::Heart, Suit::Spade, false);
    assert_eq!(fire_draw(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let (mut state, result) = resolve(tx.commit().state, owner, vec![1], false, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, WUSHENG));
    assert!(has_skill(&state, owner, PAOXIAO));
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let gain_index = result
        .events
        .iter()
        .position(|event| {
            matches!(event, DomainEvent::CardsMoved { cards, to, .. } if *to == hand && top.iter().all(|card| cards.contains(card)))
        })
        .unwrap();
    let skill_index = result
        .events
        .iter()
        .position(
            |event| matches!(event, DomainEvent::SkillAdded { player, .. } if *player == owner),
        )
        .unwrap();
    assert!(gain_index < skill_index);

    state.turn.phase = Phase::End;
    let revision = state.revision;
    let (state, ended) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert!(!has_skill(&state, owner, WUSHENG));
    assert!(!has_skill(&state, owner, PAOXIAO));
}

#[test]
fn permanent_wusheng_is_not_made_temporary() {
    let Setup {
        mut tx,
        owner,
        top: _,
    } = setup(Suit::Diamond, Suit::Club, true);
    assert_eq!(fire_draw(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let (mut state, result) = resolve(tx.commit().state, owner, vec![1], false, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, WUSHENG));
    assert!(has_skill(&state, owner, PAOXIAO));
    state.turn.phase = Phase::End;
    let revision = state.revision;
    let (state, ended) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(6),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, WUSHENG));
    assert!(!has_skill(&state, owner, PAOXIAO));
}
}

mod original_yijiang_olddanshou_old_zhuran {
//! Golden coverage for 旧朱然「胆守」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDDANSHOU: u32 = 3_778_601_240;
const OLD_ZHURAN: u32 = 4_091_953_259;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/olddanshou.skill");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId, CardId) {
    let package = content_compiler::compile_source_named("olddanshou-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 286);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let current = players[2];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLDDANSHOU),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = current;
    tx.state_mut().turn.phase = Phase::Play;

    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_601),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: processing,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(processing)
        .unwrap()
        .cards
        .push(card);
    (tx, owner, target, current, card)
}

fn damage(tx: &mut Transaction, owner: PlayerId, target: PlayerId) {
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::RecoverHp {
            source: PlayerId::NONE,
            player: target,
            amount: 1,
        });
    assert_eq!(
        run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}

fn resolve_optional(tx: Transaction, owner: PlayerId, accepted: bool, key: u64) -> GameState {
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: accepted.then_some(1).into_iter().collect(),
                cancelled: !accepted,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

#[test]
fn olddanshou_aborts_the_resolution_and_phase_only_when_accepted() {
    let (mut declined, owner, target, current, processing_card) = setup(20_260_729_11);
    let hand = declined.state().hand_count(owner);
    let hp = declined.state().players.get(target).unwrap().hp;
    damage(&mut declined, owner, target);
    let declined = resolve_optional(declined, owner, false, 1);
    assert_eq!(declined.hand_count(owner), hand);
    assert_eq!(declined.players.get(target).unwrap().hp, hp);
    assert_eq!(declined.turn.current, current);
    assert_eq!(declined.turn.phase, Phase::Play);
    assert_eq!(
        declined.cards.get(processing_card).unwrap().zone,
        declined
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .unwrap()
    );

    let (mut accepted, owner, target, current, processing_card) = setup(20_260_729_12);
    let hand = accepted.state().hand_count(owner);
    let hp = accepted.state().players.get(target).unwrap().hp;
    damage(&mut accepted, owner, target);
    let accepted = resolve_optional(accepted, owner, true, 2);
    assert_eq!(accepted.hand_count(owner), hand + 1);
    assert_eq!(accepted.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(accepted.turn.current, current);
    assert_eq!(accepted.turn.phase, Phase::Discard);
    assert_eq!(accepted.turn.flag(turn_flag::SKIPPED_PHASE_COUNT), 0);
    assert_eq!(
        accepted.cards.get(processing_card).unwrap().zone,
        accepted
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}

#[test]
fn packages_publish_old_zhuran_with_exact_skill() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLDDANSHOU && skill.name == "olddanshou"));
    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    assert_eq!(
        old.character_index
            .iter()
            .find(|character| character.id == OLD_ZHURAN)
            .unwrap()
            .skills,
        [OLDDANSHOU]
    );
}
}

mod original_yijiang_oldqianxi {
//! Golden coverage for YIJIANG「旧潜袭」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDQIANXI: u32 = 3_282_621_718;
const MADAI: u32 = 2_878_015_596;
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/oldqianxi.skill");

fn setup(player_count: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("oldqianxi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, player_count, 20_260_718, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDQIANXI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let state = tx.state_mut().players.get_mut(*player).unwrap();
        state.hp = 4;
        state.max_hp = 4;
    }
    (tx, owner, seats)
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn open_sha_damage(tx: &mut Transaction, source: PlayerId, target: PlayerId, card: CardId) {
    assert_eq!(
        run_effects(
            tx,
            vec![EffectRequest::DealShaDamage {
                source,
                target,
                card,
                amount: 1,
                nature: 0,
                card_flags: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}

fn resolve_optional(tx: Transaction, actor: PlayerId, accepted: bool, key: u64) -> GameState {
    let committed = tx.commit();
    let choice_id = ChoiceId::new(committed.state.resolution.pending_choice_id);
    let (state, pipeline) = apply_command(
        committed.state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(committed.state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: accepted.then_some(1).into_iter().collect(),
                cancelled: !accepted,
            },
        },
    );
    assert!(
        matches!(pipeline.result, CommandResult::Accepted { .. }),
        "unexpected optional-trigger result: {:?}",
        pipeline.result
    );
    assert!(pipeline.choice.is_none());
    state
}

#[test]
fn packages_publish_exact_skill_character_and_dependencies() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    let compiled = content_compiler::compile_source_named("oldqianxi", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = yijiang
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
    let trigger = yijiang
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLDQIANXI)
        .unwrap();
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(
        trigger.filter,
        "owner_source_sha_target_alive_distance_le_one"
    );
    assert!(trigger.optional);

    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    for dependency in ["standard", "yijiang"] {
        assert!(old.manifest.dependencies.iter().any(|entry| {
            entry.namespace == "official"
                && entry.package_id == dependency
                && entry.version == "0.1.0"
        }));
    }
    let madai = old
        .character_index
        .iter()
        .find(|character| character.id == MADAI)
        .unwrap();
    assert_eq!(madai.name_key, "madai");
    assert_eq!(madai.max_hp, 4);
    assert_eq!(madai.skills, [2, OLDQIANXI]);
}

#[test]
fn nonheart_judgment_prevents_damage_and_reduces_max_hp() {
    let (mut tx, owner, seats) = setup(2);
    let target = seats[1];
    tx.state_mut().players.get_mut(target).unwrap().hp = 3;
    let judgment = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        Suit::Spade,
        97_000,
    );
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        97_001,
    );
    open_sha_damage(&mut tx, owner, target, sha);
    let state = resolve_optional(tx, owner, true, 1);
    let target_state = state.players.get(target).unwrap();
    assert_eq!((target_state.hp, target_state.max_hp), (3, 3));
    assert!(state.resolution.prevented_damage_ids.is_empty());
    assert_eq!(state.resolution.current_damage_id, 0);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&judgment));
}

#[test]
fn heart_judgment_keeps_damage_and_max_hp_unchanged() {
    let (mut tx, owner, seats) = setup(2);
    let target = seats[1];
    tx.state_mut().players.get_mut(target).unwrap().hp = 3;
    inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        Suit::Heart,
        97_010,
    );
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        97_011,
    );
    open_sha_damage(&mut tx, owner, target, sha);
    let state = resolve_optional(tx, owner, true, 2);
    let target_state = state.players.get(target).unwrap();
    assert_eq!((target_state.hp, target_state.max_hp), (2, 4));
}

#[test]
fn declining_the_optional_trigger_keeps_normal_sha_damage() {
    let (mut tx, owner, seats) = setup(2);
    let target = seats[1];
    tx.state_mut().players.get_mut(target).unwrap().hp = 3;
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        97_020,
    );
    open_sha_damage(&mut tx, owner, target, sha);
    let state = resolve_optional(tx, owner, false, 3);
    let target_state = state.players.get(target).unwrap();
    assert_eq!((target_state.hp, target_state.max_hp), (2, 4));
}

#[test]
fn far_sha_and_non_sha_damage_do_not_open_the_trigger() {
    let (mut tx, owner, seats) = setup(4);
    let far = seats[2];
    tx.state_mut().players.get_mut(far).unwrap().hp = 3;
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        97_030,
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: owner,
            target: far,
            card: sha,
            amount: 1,
            nature: 0,
            card_flags: 0,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(far).unwrap().hp, 2);

    let near = seats[1];
    tx.state_mut().players.get_mut(near).unwrap().hp = 3;
    let duel = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        Suit::Club,
        97_031,
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: owner,
            target: near,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            card: duel,
            card_kind: CardKind::TrickJuedou.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(near).unwrap().hp, 2);
}
}

mod original_yijiang_pojun {
//! Golden for 一将成名徐盛「破军」rewritten from the legacy JavaScript semantics.

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

const POJUN: u32 = 2_171_911_670;
const OLD_XUSHENG: u32 = 3_544_872_927;
const YIJIANG_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const OLD_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

fn isolated(seed: u64) -> (Transaction, PlayerId, PlayerId, CardId) {
    let package = content_schema::ContentPackage::from_json_bytes(YIJIANG_PACKAGE).unwrap();
    let mut engine = Engine::new();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    engine.load_package(&package).unwrap();
    let source = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != source)
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
            player: target,
            count: u32::MAX,
        },
        EffectRequest::AddSkill {
            player: source,
            skill_def: SkillDefId::new(POJUN),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();

    let hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(82_171),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: source,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    (tx, source, target, card)
}

fn accept(state: &GameState, source: PlayerId, key: u64) -> GameState {
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn committed_packages_contain_pojun_and_old_xusheng_cross_package_closure() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG_PACKAGE).unwrap();
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == POJUN && !skill.locked));
    let trigger = yijiang
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == POJUN)
        .unwrap();
    assert_eq!(
        (trigger.event.as_str(), trigger.filter.as_str()),
        ("damage", "owner_source_sha")
    );
    assert!(trigger.optional);

    let old = content_schema::ContentPackage::from_json_bytes(OLD_PACKAGE).unwrap();
    let xusheng = old
        .character_index
        .iter()
        .find(|character| character.id == OLD_XUSHENG)
        .unwrap();
    assert_eq!(xusheng.name_key, "old_xusheng");
    assert_eq!(
        (xusheng.sex.as_str(), xusheng.group.as_str()),
        ("male", "wu")
    );
    assert_eq!(xusheng.max_hp, 4);
    assert_eq!(xusheng.skills, [POJUN]);
    assert!(old
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "yijiang"));
}

#[test]
fn sha_damage_draws_up_to_five_from_current_hp_then_turns_target_over() {
    let (mut tx, source, target, card) = isolated(20_260_716_03);
    let player = tx.state_mut().players.get_mut(target).unwrap();
    player.max_hp = 6;
    player.hp = 6;

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealShaDamage {
                source,
                target,
                card,
                amount: 1,
                nature: rule_api::damage_nature::NORMAL,
                card_flags: 0,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(tx.state().players.get(target).unwrap().hp, 5);
    let state = accept(tx.state(), source, 1);
    assert_eq!(state.hand_count(target), 5);
    assert!(state.players.get(target).unwrap().turned_over);
}

#[test]
fn current_hp_controls_draw_count_and_turn_over_toggles() {
    let (mut tx, source, target, card) = isolated(20_260_716_04);
    let player = tx.state_mut().players.get_mut(target).unwrap();
    player.max_hp = 4;
    player.hp = 3;
    player.turned_over = true;

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealShaDamage {
                source,
                target,
                card,
                amount: 1,
                nature: rule_api::damage_nature::NORMAL,
                card_flags: 0,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx.state(), source, 2);
    assert_eq!(state.hand_count(target), 2);
    assert!(!state.players.get(target).unwrap().turned_over);
}

#[test]
fn non_sha_damage_does_not_trigger_pojun() {
    let (mut tx, source, target, _) = isolated(20_260_716_05);
    let hand = tx.state().hand_count(target);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(target), hand);
    assert!(!tx.state().players.get(target).unwrap().turned_over);
    assert!(!tx.state().has_pending_choice());
}
}

mod original_yijiang_qiaoshi {
//! Golden for 一将成名夏侯氏「樵拾」的等手牌结束阶段语义。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::CardKind;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const QIAOSHI: u32 = 2_607_312_044;
const JYZONGSHI_OLD: u32 = 2_968_553_132;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn qiaoshi_event(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn isolated_state() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == QIAOSHI && !skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
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
        skill_def: SkillDefId::new(QIAOSHI),
    });
    run_effects(&mut tx, effects).unwrap();
    let owner_hand = tx.state().hand_count(owner);
    let other_hand = tx.state().hand_count(other);
    if owner_hand > other_hand {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player: owner,
                count: owner_hand - other_hand,
            }],
        )
        .unwrap();
    } else if other_hand > owner_hand {
        run_effects(
            &mut tx,
            vec![EffectRequest::DrawCards {
                player: owner,
                count: other_hand - owner_hand,
            }],
        )
        .unwrap();
    }
    (tx, owner, other)
}

#[test]
fn equal_hands_offer_owner_choice_and_accepting_draws_for_both() {
    let (mut tx, owner, other) = isolated_state();
    assert_eq!(tx.state().hand_count(owner), tx.state().hand_count(other));
    let owner_hand = tx.state().hand_count(owner);
    let other_hand = tx.state().hand_count(other);

    let paused = dispatch_for_events(
        &mut tx,
        &[qiaoshi_event(other)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);

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
    assert_eq!(state.hand_count(other), other_hand + 1);
    assert_eq!(state.hand_count(owner), owner_hand + 1);
}

#[test]
fn unequal_hands_or_owners_own_end_phase_do_not_offer_a_choice() {
    let (mut tx, owner, other) = isolated_state();
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert_ne!(tx.state().hand_count(owner), tx.state().hand_count(other));

    let effects = dispatch_for_events(
        &mut tx,
        &[qiaoshi_event(other)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!tx.state().has_pending_choice());

    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), tx.state().hand_count(other));
    let effects = dispatch_for_events(
        &mut tx,
        &[qiaoshi_event(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn old_zongshi_only_draws_for_another_players_single_target_trick() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(30),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714,
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
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JYZONGSHI_OLD),
    });
    run_effects(&mut tx, effects).unwrap();
    let card = tx.state().hand_cards(other)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::TrickGuohe;
    let hand = tx.state().hand_count(owner);

    let paused = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: other,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(31),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 1);

    let mut tx = Transaction::begin(state);
    for event in [
        DomainEvent::CardUsed {
            player: other,
            card,
            targets: vec![owner, other],
        },
        DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![owner],
        },
    ] {
        dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new()).unwrap();
        assert!(!tx.state().has_pending_choice());
    }
}
}

mod original_yijiang_quancong {
//! Golden coverage for 全琮「邀名」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const QUANCONG: u32 = 3_313_708_633;
const YAOMING: u32 = 2_608_199_273;
const YAOMING_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/yaoming.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let other = players[2];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(YAOMING),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().players.get_mut(owner).unwrap().character_def = QUANCONG;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        other,
    }
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
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

fn deal_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    (tx.commit().state, result)
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn open_yaoming(state: GameState, owner: PlayerId, target: PlayerId) -> GameState {
    let (state, result) = deal_damage(state, owner, target);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    state
}

#[test]
fn package_publishes_complete_quancong_and_exact_abi60_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == QUANCONG)
        .unwrap();
    assert_eq!(character.name_key, "quancong");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [YAOMING]);
    let compiled = content_compiler::compile_source_named("yaoming", YAOMING_SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 60);
    assert!(package.manifest.rule_vm_abi_version >= compiled.manifest.rule_vm_abi_version);
    for expected in compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn yaoming_requires_owner_damage_involvement_and_a_different_hand_count() {
    let equal = setup(2_026_074_001);
    let (state, result) = deal_damage(equal.state, equal.owner, equal.target);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());

    let mut uninvolved = setup(2_026_074_002);
    inject_hand(&mut uninvolved.state, uninvolved.target, 99_001);
    let (state, result) = deal_damage(uninvolved.state, uninvolved.target, uninvolved.other);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());

    let mut source = setup(2_026_074_003);
    inject_hand(&mut source.state, source.target, 99_002);
    let (state, result) = deal_damage(source.state, source.owner, source.target);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    let mut damaged = setup(2_026_074_004);
    inject_hand(&mut damaged.state, damaged.target, 99_003);
    let (state, result) = deal_damage(damaged.state, damaged.target, damaged.owner);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}

#[test]
fn yaoming_lower_hand_target_draws_one() {
    let mut setup = setup(2_026_074_005);
    inject_hand(&mut setup.state, setup.owner, 99_011);
    inject_hand(&mut setup.state, setup.owner, 99_012);
    let state = open_yaoming(setup.state, setup.owner, setup.target);
    let (state, choice) = resolve(state, setup.owner, vec![1], false, 1);
    assert_eq!(
        choice.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseHandCountDifferentTargetDrawOrRandomDiscard
    );
    let before = state.hand_count(setup.target);
    let (state, result) = resolve(state, setup.owner, vec![setup.target.raw()], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), before + 1);
}

#[test]
fn yaoming_higher_hand_target_discards_one_with_authoritative_rng() {
    let mut setup = setup(2_026_074_006);
    let cards: Vec<_> = [99_021, 99_022, 99_023]
        .into_iter()
        .map(|def| inject_hand(&mut setup.state, setup.target, def))
        .collect();
    let state = open_yaoming(setup.state, setup.owner, setup.target);
    let (state, _) = resolve(state, setup.owner, vec![1], false, 10);
    let calls = state.rng.call_count;
    let (state, result) = resolve(state, setup.owner, vec![setup.target.raw()], false, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), 2);
    assert_eq!(state.rng.call_count, calls + 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        cards
            .iter()
            .filter(|card| state.cards.get(**card).unwrap().zone == discard)
            .count(),
        1
    );
}

#[test]
fn yaoming_rejects_forged_target_atomically_and_fires_once_per_turn() {
    let mut setup = setup(2_026_074_007);
    inject_hand(&mut setup.state, setup.owner, 99_031);
    inject_hand(&mut setup.state, setup.other, 99_032);
    let state = open_yaoming(setup.state, setup.owner, setup.target);
    let (state, _) = resolve(state, setup.owner, vec![1], false, 20);
    let before = canonical_state_hash(&state);
    let (state, forged) = resolve(state, setup.owner, vec![setup.other.raw()], false, 21);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    let (mut state, result) = resolve(state, setup.owner, vec![setup.target.raw()], false, 22);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    inject_hand(&mut state, setup.target, 99_033);
    let (state, second) = deal_damage(state, setup.owner, setup.target);
    assert!(second.is_ok());
    assert!(!state.has_pending_choice());
}
}

mod original_yijiang_rejingce {
//! Golden coverage for 郭淮「精策」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const REJINGCE: u32 = 2_158_663_152;
const GUOHUAI: u32 = 3_179_439_469;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/rejingce.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    jiu: [CardId; 2],
    trick: CardId,
    weapon: CardId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind, suit: Suit) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
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
    tx.state_mut().revision = 1;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    {
        let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
        owner_state.character_def = GUOHUAI;
        owner_state.max_hp = 3;
        owner_state.hp = 3;
    }

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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REJINGCE),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let jiu = [
        inject(tx.state_mut(), owner, 92_001, CardKind::Jiu, Suit::Spade),
        inject(tx.state_mut(), owner, 92_002, CardKind::Jiu, Suit::Spade),
    ];
    let trick = inject(
        tx.state_mut(),
        owner,
        92_003,
        CardKind::TrickWuzhong,
        Suit::Heart,
    );
    let weapon = inject(tx.state_mut(), owner, 92_004, CardKind::Weapon, Suit::Club);
    Setup {
        state: tx.commit().state,
        owner,
        other,
        jiu,
        trick,
        weapon,
    }
}

fn play(state: GameState, actor: PlayerId, card: CardId, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    )
}

fn end_play(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
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

fn accept_optional(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    )
}

fn hand_limit(state: &GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected hand-limit response: {other:?}"),
    }
}

#[test]
fn package_publishes_complete_guohuai_in_abi53_package() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == GUOHUAI)
        .unwrap();
    assert_eq!(character.skills, [REJINGCE]);
    let compiled = content_compiler::compile_source_named("rejingce", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn repeated_suit_is_bounded_while_logical_categories_accumulate() {
    let setup = setup(2_026_071_901);
    assert_eq!(hand_limit(&setup.state, setup.owner), 3);

    let (state, first) = play(setup.state, setup.owner, setup.jiu[0], 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(hand_limit(&state, setup.owner), 4);
    assert_eq!(state.turn.play_card_category_count(setup.owner), 1);

    let (state, repeated) = play(state, setup.owner, setup.jiu[1], 2);
    assert!(matches!(repeated.result, CommandResult::Accepted { .. }));
    assert_eq!(hand_limit(&state, setup.owner), 4);
    assert_eq!(state.turn.play_card_category_count(setup.owner), 1);

    let (state, trick) = play(state, setup.owner, setup.trick, 3);
    assert!(matches!(trick.result, CommandResult::Accepted { .. }));
    assert_eq!(hand_limit(&state, setup.owner), 5);
    assert_eq!(state.turn.play_card_category_count(setup.owner), 2);

    let (state, weapon) = play(state, setup.owner, setup.weapon, 4);
    assert!(matches!(weapon.result, CommandResult::Accepted { .. }));
    assert_eq!(hand_limit(&state, setup.owner), 6);
    assert_eq!(state.turn.play_card_category_count(setup.owner), 3);
}

#[test]
fn phase_end_draws_category_count_and_resets_histories_at_the_right_boundaries() {
    let setup = setup(2_026_071_902);
    let (state, _) = play(setup.state, setup.owner, setup.jiu[0], 10);
    let (state, _) = play(state, setup.owner, setup.trick, 11);
    let (state, _) = play(state, setup.owner, setup.weapon, 12);
    let hand_before = state.hand_count(setup.owner);

    let (state, optional) = end_play(state, setup.owner, 13);
    assert!(matches!(optional.result, CommandResult::Accepted { .. }));
    assert!(optional.choice.is_some());
    let (state, accepted) = accept_optional(state, setup.owner, 14);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), hand_before + 3);
    assert!(!state.has_pending_choice());

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: setup.owner,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().turn.play_card_category_count(setup.owner), 0);
    assert_eq!(
        tx.state().turn.distinct_hand_card_suits_used(setup.owner),
        3
    );
    assert_eq!(hand_limit(tx.state(), setup.owner), 6);

    let (state, empty_end) = end_play(tx.commit().state, setup.owner, 15);
    assert!(matches!(empty_end.result, CommandResult::Accepted { .. }));
    assert!(empty_end.choice.is_none());

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: setup.other,
            phase: Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().turn.distinct_hand_card_suits_used(setup.owner),
        0
    );
    assert_eq!(hand_limit(tx.state(), setup.owner), 3);
}
}

mod original_yijiang_reqiaoshui {
//! Golden coverage for 界简雍「巧说」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{card_ignored_for_hand_limit, discard_to_limit_tokens};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REQIAOSHUI: u32 = 3_083_818_598;
const QIAOSHUI3: u32 = 2_260_792_673;
const QIAOSHUI4: u32 = 3_521_041_512;
const TARGET_BOOST: u32 = 1_990_000_067;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const TARGET_BOOST_SOURCE: &str = r#"
skill qiaoshui_test_target_boost(1990000067) {
  modifier attack_range override 99;
  modifier sha_target_limit add 2;
}
"#;

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: PlayerId,
    second: PlayerId,
    third: PlayerId,
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    def: u32,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, skill_defs: &[u32]) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let boost =
        content_compiler::compile_source_named("qiaoshui_test_boost", TARGET_BOOST_SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&boost).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend(skill_defs.iter().map(|skill_def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(*skill_def),
    }));
    effects.push(EffectRequest::SetTurnState {
        current: owner,
        phase: Phase::Play.to_u8(),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    Setup {
        state: tx.commit().state,
        owner,
        first: seats[1],
        second: seats[2],
        third: seats[3],
    }
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == def)
        })
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn compare(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    owner_card: CardId,
    target_card: CardId,
    key: u64,
) -> GameState {
    let active = skill_id(&state, owner, REQIAOSHUI);
    let (state, activated) = command(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        key,
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let (state, owner_selected) = resolve(state, owner, vec![owner_card.raw()], false, key + 1);
    assert!(matches!(
        owner_selected.result,
        CommandResult::Accepted { .. }
    ));
    let (state, compared) = resolve(state, target, vec![target_card.raw()], false, key + 2);
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_reqiaoshui_helpers_at_abi68() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    for def in [REQIAOSHUI, QIAOSHUI3, QIAOSHUI4] {
        assert!(package
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == def));
    }
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == REQIAOSHUI)
        .unwrap();
    assert_eq!(active.uses_per_turn, None);
}

#[test]
fn strict_win_grants_one_shot_helper_and_reqiaoshui_remains_repeatable() {
    let mut setup = setup(2_026_072_201, &[REQIAOSHUI]);
    let owner_card = inject(
        &mut setup.state,
        setup.owner,
        99_201,
        CardKind::Other,
        Suit::Spade,
        13,
    );
    let target_card = inject(
        &mut setup.state,
        setup.first,
        99_202,
        CardKind::Other,
        Suit::Heart,
        1,
    );
    let mut state = compare(
        setup.state,
        setup.owner,
        setup.first,
        owner_card,
        target_card,
        1,
    );
    assert!(has_skill(&state, setup.owner, QIAOSHUI3));
    let active = skill_id(&state, setup.owner, REQIAOSHUI);
    assert_eq!(state.skills.get(active).unwrap().use_count, 1);

    let second_owner = inject(
        &mut state,
        setup.owner,
        99_203,
        CardKind::Other,
        Suit::Club,
        12,
    );
    let _second_target = inject(
        &mut state,
        setup.first,
        99_204,
        CardKind::Other,
        Suit::Diamond,
        2,
    );
    let (state, activated_again) = command(
        state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.first],
            card_targets: Vec::new(),
        },
        4,
    );
    assert!(matches!(
        activated_again.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ActiveSkillSelection
    );
    assert!(state.hand_cards(setup.owner).contains(&second_owner));
}

#[test]
fn failed_compare_skips_play_and_keeps_tricks_out_of_discard_limit_until_turn_end() {
    let mut setup = setup(2_026_072_202, &[REQIAOSHUI]);
    let owner_card = inject(
        &mut setup.state,
        setup.owner,
        99_205,
        CardKind::Other,
        Suit::Spade,
        1,
    );
    let target_card = inject(
        &mut setup.state,
        setup.first,
        99_206,
        CardKind::Other,
        Suit::Heart,
        13,
    );
    let mut state = compare(
        setup.state,
        setup.owner,
        setup.first,
        owner_card,
        target_card,
        10,
    );
    assert_eq!(state.turn.phase, Phase::Discard);
    assert!(has_skill(&state, setup.owner, QIAOSHUI4));
    let trick = inject(
        &mut state,
        setup.owner,
        99_207,
        CardKind::TrickGuohe,
        Suit::Club,
        7,
    );
    assert!(card_ignored_for_hand_limit(&state, setup.owner, trick));
    assert!(!discard_to_limit_tokens(&state, setup.owner).contains(&trick));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: setup.owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: setup.owner,
                phase: Phase::Idle.to_u8(),
            },
        ],
    )
    .unwrap();
    assert!(!has_skill(tx.state(), setup.owner, QIAOSHUI4));
}

#[test]
fn one_shot_helper_adds_or_removes_a_saved_sha_target_and_consumes_itself() {
    let mut add_setup = setup(2_026_072_203, &[QIAOSHUI3, TARGET_BOOST]);
    let sha = inject(
        &mut add_setup.state,
        add_setup.owner,
        99_208,
        CardKind::Sha,
        Suit::Diamond,
        8,
    );
    let (state, played) = command(
        add_setup.state,
        add_setup.owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![add_setup.first],
            card_targets: Vec::new(),
        },
        20,
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddOrRemoveCurrentCardTarget
    );
    let (state, added) = resolve(
        state,
        add_setup.owner,
        vec![add_setup.second.raw()],
        false,
        21,
    );
    assert!(matches!(added.result, CommandResult::Accepted { .. }));
    assert!(added.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardTargetAdded { target, targets, .. }
            if *target == add_setup.second
                && targets == &[add_setup.first, add_setup.second]
    )));
    assert_eq!(state.players.get(add_setup.first).unwrap().hp, 9);
    assert_eq!(state.players.get(add_setup.second).unwrap().hp, 9);
    assert!(!has_skill(&state, add_setup.owner, QIAOSHUI3));

    let mut remove_setup = setup(2_026_072_204, &[QIAOSHUI3, TARGET_BOOST]);
    let sha = inject(
        &mut remove_setup.state,
        remove_setup.owner,
        99_209,
        CardKind::Sha,
        Suit::Club,
        9,
    );
    let (state, played) = command(
        remove_setup.state,
        remove_setup.owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![remove_setup.first, remove_setup.second],
            card_targets: Vec::new(),
        },
        30,
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let (state, mode) = resolve(state, remove_setup.owner, vec![1], false, 31);
    assert!(matches!(mode.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddOrRemoveCurrentCardTarget
    );
    let (state, removed) = resolve(
        state,
        remove_setup.owner,
        vec![remove_setup.second.raw()],
        false,
        32,
    );
    assert!(matches!(removed.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(remove_setup.first).unwrap().hp, 9);
    assert_eq!(state.players.get(remove_setup.second).unwrap().hp, 10);
    assert_eq!(state.players.get(remove_setup.third).unwrap().hp, 10);
    assert!(!has_skill(&state, remove_setup.owner, QIAOSHUI3));
}

#[test]
fn cancelling_the_optional_adjustment_still_consumes_qiaoshui3() {
    let mut setup = setup(2_026_072_205, &[QIAOSHUI3, TARGET_BOOST]);
    let sha = inject(
        &mut setup.state,
        setup.owner,
        99_210,
        CardKind::Sha,
        Suit::Spade,
        10,
    );
    let (state, played) = command(
        setup.state,
        setup.owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![setup.first],
            card_targets: Vec::new(),
        },
        40,
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let (state, cancelled) = resolve(state, setup.owner, Vec::new(), true, 41);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.first).unwrap().hp, 9);
    assert_eq!(state.players.get(setup.second).unwrap().hp, 10);
    assert!(!has_skill(&state, setup.owner, QIAOSHUI3));
}
}

mod original_yijiang_rezhiman {
//! Golden for YIJIANG「制蛮」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REZHIMAN: u32 = 3_521_391_741;
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
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
fn rezhiman_gains_judgment_card_then_prevents_damage() {
    let package = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    assert_eq!(package.skill_index.len(), 281);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == REZHIMAN
            && trigger.event == "damage_before"
            && trigger.filter == "owner_source_other_damage_before"
            && trigger.optional
    }));

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_731, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let target = seats[1];
    let remove: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REZHIMAN),
        }],
    )
    .unwrap();

    let judge = tx.state().zones.find(ZoneKind::Judge, target).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(97_334),
        kind: CardKind::TrickLe,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: judge,
        owner: target,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(judge)
        .unwrap()
        .cards
        .push(card);
    let hp = tx.state().players.get(target).unwrap().hp;

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    let (state, accepted) = resolve(tx.commit().state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let choice = accepted.choice.expect("制蛮应请求选择一张目标牌");
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.observer, owner);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == card.raw()));

    let (state, selected) = resolve(state, owner, vec![card.raw()], false, 2);
    assert!(matches!(selected.result, CommandResult::Accepted { .. }));
    assert!(selected.choice.is_none());
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(state.cards.get(card).unwrap().zone, hand);
    assert_eq!(state.cards.get(card).unwrap().owner, owner);
    assert_eq!(state.resolution.current_damage_id, 0);
}
}

mod original_yijiang_sanyao {
//! Golden for Yijiang `sanyao`, rewritten from legacy JavaScript.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId,
};
use engine_core::canonical_state_hash;
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SANYAO: u32 = 3_737_839_931;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn started() -> (GameState, PlayerId, PlayerId, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_071_717, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let players = tx.state().players.alive_by_seat();
    let owner = tx.state().turn.current;
    let mut others = players.into_iter().filter(|player| *player != owner);
    let highest = others.next().unwrap();
    let lower = others.next().unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    tx.state_mut().players.get_mut(highest).unwrap().hp = 4;
    tx.state_mut().players.get_mut(lower).unwrap().hp = 2;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SANYAO),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SANYAO)
        .unwrap();
    (tx.state().clone(), owner, highest, lower, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    card: engine_api::CardId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![card],
            },
        },
    )
}

#[test]
fn package_exposes_highest_hp_target_and_hand_equip_cost() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SANYAO)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (1, 1));
    assert!(targets.allow_self);
    assert!(targets.highest_hp);
    assert!(!targets.lowest_hp);
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(
        (selection.zone.as_str(), selection.min, selection.max),
        ("hand_equip", 1, 1)
    );
}

#[test]
fn only_a_highest_hp_target_can_be_damaged_and_the_cost_is_discarded() {
    let (state, owner, highest, lower, skill) = started();
    let cost = state.hand_cards(owner)[0];
    let before = canonical_state_hash(&state);
    let (same, rejected) = activate(state.clone(), owner, lower, skill, cost, 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let hp = state.players.get(highest).unwrap().hp;
    let (state, accepted) = activate(state, owner, highest, skill, cost, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(highest).unwrap().hp, hp - 1);
    assert_eq!(
        state.cards.get(cost).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);
}
}

mod original_yijiang_shangshi {
//! Golden for 一将成名「伤逝」的相关变化过滤与补至损失体力值。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SHANGSHI: u32 = 3_386_412_611;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, zone_kind: ZoneKind, rank: u8) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_000_000 + u32::from(rank)),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.character_index.iter().any(|character| {
        character.name_key == "zhangchunhua" && character.skills == [3_481_582_878, SHANGSHI]
    }));
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_717),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_717,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
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
    run_effects(&mut tx, std::mem::take(&mut effects)).unwrap();
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, other);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHANGSHI),
        }],
    )
    .unwrap();
    (tx, owner, other)
}

fn confirm(
    state: engine_core::state::GameState,
    owner: PlayerId,
    key: u64,
) -> engine_core::state::GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn owner_gain_refills_to_lost_hp_but_unrelated_moves_do_not_trigger() {
    let (mut tx, owner, other) = setup();
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.max_hp = 4;
    owner_state.hp = 1;
    let other_card = inject(&mut tx, other, ZoneKind::Hand, 1);
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![other_card],
                to: owner_hand,
                from: Some(other_hand),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = confirm(tx.commit().state, owner, 1);
    assert_eq!(state.hand_count(owner), 3);

    let mut unrelated = Transaction::begin(state);
    let a = inject(&mut unrelated, other, ZoneKind::Hand, 2);
    let discard = unrelated
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(run_effects(
        &mut unrelated,
        vec![EffectRequest::MoveCards {
            cards: vec![a],
            to: discard,
            from: Some(other_hand),
        }],
    )
    .is_ok());
    assert!(!unrelated.state().has_pending_choice());
}

#[test]
fn damage_refills_using_post_damage_lost_hp() {
    let (mut tx, owner, other) = setup();
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.max_hp = 4;
    owner_state.hp = 3;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = confirm(tx.commit().state, owner, 2);
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
    assert_eq!(state.hand_count(owner), 2);
}
}

mod original_yijiang_shenxing {
//! Golden for Yijiang `shenxing`, rewritten from legacy JavaScript.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SHENXING: u32 = 3_283_042_324;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn started() -> (GameState, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_071_522, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHENXING),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SHENXING)
        .unwrap();
    (tx.state().clone(), owner, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let cards = state.hand_cards(owner)[..2].to_vec();
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: cards,
            },
        },
    )
}

#[test]
fn shenxing_discards_two_draws_one_and_has_no_implicit_use_limit() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SHENXING)
        .unwrap();
    assert_eq!(active.uses_per_turn, None);
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(
        (selection.zone.as_str(), selection.min, selection.max),
        ("hand_equip", 2, 2)
    );

    let (state, owner, skill) = started();
    let hand = state.hand_count(owner);
    let (state, first) = activate(state, owner, skill, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand - 1);

    let (state, second) = activate(state, owner, skill, 2);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand - 2);
    assert_eq!(state.skills.get(skill).unwrap().use_count, 2);
}
}

mod original_yijiang_shibei {
//! Golden for 一将成名「矢北」rewritten from the legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SHIBEI: u32 = 2_648_030_702;
const JIANYING: u32 = 3_679_198_172;
const YJ_JUSHOU: u32 = 3_182_131_819;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

#[test]
fn package_contains_locked_first_and_later_damage_branches() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == YJ_JUSHOU)
        .expect("committed 沮授 character");
    assert_eq!(character.name_key, "yj_jushou");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.initial_hp, 0, "omitted means start at max_hp");
    assert_eq!(character.skills, [JIANYING, SHIBEI]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == SHIBEI && skill.locked));
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == SHIBEI)
            .map(|trigger| trigger.filter.as_str())
            .collect::<Vec<_>>(),
        [
            "owner_is_target_first_damage",
            "owner_is_target_after_first_damage"
        ]
    );
}

#[test]
fn first_damage_recovers_and_later_damage_loses_hp_in_the_same_turn() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_716_20),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_716_20,
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SHIBEI),
    });
    run_effects(&mut tx, effects).unwrap();
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 6;
        player.hp = 5;
    }

    let damage = || EffectRequest::DealDamage {
        source,
        target: owner,
        amount: 1,
        is_lose_hp: false,
        nature: 0,
    };
    run_effects(&mut tx, vec![damage()]).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 5);

    run_effects(&mut tx, vec![damage()]).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == SHIBEI)
        .unwrap();
    assert_eq!(skill.turn_fire_count, 2);
}
}

mod original_yijiang_shiyong {
//! Golden tests for yijiang 将华雄「恃勇」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/shiyong.skill");
const OLD_HUAXIONG: u32 = 3_259_572_192;
const SHIYONG: u32 = 2_676_469_853;

fn loaded(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let source = engine.state().turn.current;
    let owner = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source)
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
    for player in [source, owner] {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SHIYONG),
    });
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.max_hp = 6;
    player.hp = 6;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, source, owner)
}

fn inject_sha(tx: &mut Transaction, source: PlayerId, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + suit as u32),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: source,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn deal_sha(
    tx: &mut Transaction,
    source: PlayerId,
    owner: PlayerId,
    card: CardId,
    amount: u32,
    card_flags: u8,
) {
    run_effects(
        tx,
        vec![EffectRequest::DealShaDamage {
            source,
            target: owner,
            card,
            amount,
            nature: rule_api::damage_nature::NORMAL,
            card_flags,
        }],
    )
    .unwrap();
}

#[test]
fn committed_yijiang_package_contains_old_huaxiong_and_locked_shiyong() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OLD_HUAXIONG)
        .unwrap();
    assert_eq!((character.initial_hp, character.max_hp), (0, 6));
    assert_eq!(character.skills, [SHIYONG]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == SHIYONG && skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SHIYONG)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target_red_or_jiu_sha");
}

#[test]
fn red_sha_damage_reduces_max_hp_after_damage() {
    let (mut tx, source, owner) = loaded(20_260_716_6);
    let card = inject_sha(&mut tx, source, Suit::Heart);
    deal_sha(&mut tx, source, owner, card, 1, 0);
    let player = tx.state().players.get(owner).unwrap();
    assert_eq!((player.hp, player.max_hp), (5, 5));
}

#[test]
fn black_sha_without_jiu_does_not_trigger_even_at_two_damage() {
    let (mut tx, source, owner) = loaded(20_260_716_7);
    let card = inject_sha(&mut tx, source, Suit::Club);
    deal_sha(&mut tx, source, owner, card, 2, 0);
    let player = tx.state().players.get(owner).unwrap();
    assert_eq!((player.hp, player.max_hp), (4, 6));
}

#[test]
fn black_sha_boosted_by_jiu_reduces_max_hp() {
    let (mut tx, source, owner) = loaded(20_260_716_8);
    let card = inject_sha(&mut tx, source, Suit::Spade);
    deal_sha(
        &mut tx,
        source,
        owner,
        card,
        2,
        rule_api::damage_card_flag::JIU_BOOSTED,
    );
    let player = tx.state().players.get(owner).unwrap();
    assert_eq!((player.hp, player.max_hp), (4, 5));
}

#[test]
fn playing_black_sha_with_jiu_mark_propagates_the_provenance_flag() {
    let (mut tx, source, owner) = loaded(20_260_716_9);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: source,
            mark: MarkDefId::new(engine_core::content::MARK_JIU),
            delta: 1,
        }],
    )
    .unwrap();
    let card = inject_sha(&mut tx, source, Suit::Club);
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![owner],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(pipeline.choice.is_none());
    let player = state.players.get(owner).unwrap();
    assert_eq!((player.hp, player.max_hp), (4, 5));
    assert_eq!(
        state
            .players
            .get(source)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == engine_core::content::MARK_JIU)
            .map(|(_, count)| *count)
            .unwrap_or(0),
        0
    );
}
}

mod original_yijiang_sidi_old_caozhen {
//! Golden coverage for 旧曹真「司敌」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::query::sha_use_limit_for;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLD_CAOZHEN: u32 = 2_623_285_537;
const SIDI: u32 = 3_456_311_904;
const SIDI3: u32 = 3_364_392_152;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/yijiang/sidi.skill"),
    "\n",
    include_str!("../../../content/original/ported/yijiang/sidi3.skill")
);
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, CardId) {
    let package = content_compiler::compile_source_named("sidi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 287);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_729_21, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SIDI),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let shan = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_701),
        kind: CardKind::Shan,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(shan);
    (tx.commit().state, owner, target, shan)
}

fn dispatch(state: GameState, events: &[DomainEvent]) -> GameState {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        dispatch_for_events(&mut tx, events, &mut TriggerFireRecord::new()),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

#[test]
fn shan_builds_the_public_pile_and_one_chosen_card_reduces_sha_until_play_ends() {
    let (mut state, owner, target, shan) = setup();
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let first = *state.zones.get(draw).unwrap().cards.last().unwrap();
    for key in 1..=2 {
        state = dispatch(
            state,
            &[DomainEvent::CardUsed {
                player: owner,
                card: shan,
                targets: Vec::new(),
            }],
        );
        state = resolve(state, owner, vec![1], key);
    }
    let pile = state.skill_pile_cards(owner, SIDI);
    assert_eq!(pile.len(), 2);
    assert!(pile.contains(&first));

    let base = sha_use_limit_for(&state, target);
    state.turn.current = target;
    state.turn.phase = Phase::Play;
    state = dispatch(
        state,
        &[DomainEvent::TurnStateChanged {
            current: target,
            phase: TurnPhaseTag::Play,
        }],
    );
    state = resolve(state, owner, vec![1], 3);
    assert!(state.has_pending_choice());
    state = resolve(state, owner, vec![pile[0].raw()], 4);

    assert_eq!(state.skill_pile_cards(owner, SIDI).len(), 1);
    assert!(state
        .players
        .get(target)
        .unwrap()
        .marks
        .contains(&(SIDI3, 1)));
    assert_eq!(sha_use_limit_for(&state, target), base.saturating_sub(1));

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: target,
            phase: TurnPhaseTag::Play,
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
    assert_eq!(sha_use_limit_for(tx.state(), target), base);
    assert!(!tx
        .state()
        .players
        .get(target)
        .unwrap()
        .marks
        .contains(&(SIDI3, 1)));
}

#[test]
fn packages_publish_sidi_and_old_caozhen() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == SIDI && skill.name == "sidi"));
    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    assert_eq!(
        old.character_index
            .iter()
            .find(|character| character.id == OLD_CAOZHEN)
            .unwrap()
            .skills,
        [SIDI]
    );
}
}

mod original_yijiang_sunluban {
//! Golden coverage for 孙鲁班「谮毁」「骄矜」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SUNLUBAN: u32 = 3_957_343_056;
const GUYONG: u32 = 3_099_069_622;
const CHANHUI: u32 = 2_955_673_837;
const JIAOJIN: u32 = 2_809_119_976;
const CHANHUI_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/chanhui.skill");
const JIAOJIN_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/jiaojin.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    female: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let alternate = players[2];
    let female = players[3];
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
    effects.extend([CHANHUI, JIAOJIN].map(|skill_def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().players.get_mut(owner).unwrap().character_def = SUNLUBAN;
    tx.state_mut()
        .players
        .get_mut(target)
        .unwrap()
        .character_def = GUYONG;
    tx.state_mut()
        .players
        .get_mut(alternate)
        .unwrap()
        .character_def = GUYONG;
    tx.state_mut()
        .players
        .get_mut(female)
        .unwrap()
        .character_def = SUNLUBAN;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        female,
    }
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
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

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn play_sha(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    submit(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn open_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
) -> (GameState, Vec<DomainEvent>, engine_api::ChoiceRequest) {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target,
                amount,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    (
        committed.state,
        committed.output.events,
        committed.output.choice.unwrap(),
    )
}

fn skill_phase_count(state: &GameState, owner: PlayerId, skill_def: u32) -> u32 {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .play_phase_fire_count
}

#[test]
fn package_publishes_complete_sunluban_and_exact_abi57_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SUNLUBAN)
        .unwrap();
    assert_eq!(character.name_key, "sunluban");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [CHANHUI, JIAOJIN]);
    for (name, source) in [("chanhui", CHANHUI_SOURCE), ("jiaojin", JIAOJIN_SOURCE)] {
        let compiled = content_compiler::compile_source_named(name, source).unwrap();
        assert_eq!(compiled.manifest.rule_vm_abi_version, 57);
        for expected in compiled.modules {
            let actual = package
                .modules
                .iter()
                .find(|module| module.name == expected.name)
                .unwrap();
            assert_eq!(actual.logic_hash, expected.logic_hash);
        }
    }
}

#[test]
fn chanhui_cancel_branch_adds_target_and_rejects_forged_gift_atomically() {
    let mut setup = setup(2_026_073_701);
    let sha = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_701,
        CardKind::Sha,
        Suit::Spade,
    );
    let gift = inject(
        &mut setup.state,
        setup.female,
        ZoneKind::Hand,
        99_702,
        CardKind::Other,
        Suit::Heart,
    );
    let (state, optional) = play_sha(setup.state, setup.owner, sha, setup.target, 1);
    assert_eq!(optional.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let (state, targets) = resolve(state, setup.owner, vec![1], false, 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCardUserOrAddTarget
    );
    assert!(targets
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == setup.female.raw()));
    let (state, branch) = resolve(state, setup.owner, vec![setup.female.raw()], false, 3);
    assert_eq!(branch.choice.as_ref().unwrap().observer, setup.female);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseGiveCardOrAddTarget
    );
    assert_eq!(skill_phase_count(&state, setup.owner, CHANHUI), 1);

    let before = canonical_state_hash(&state);
    let (state, forged) = resolve(state, setup.female, vec![sha.raw()], false, 4);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    let (state, added) = resolve(state, setup.female, Vec::new(), true, 5);
    assert!(matches!(added.result, CommandResult::Accepted { .. }));
    assert!(added.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardTargetAdded { player, card, target, targets }
            if *player == setup.owner && *card == sha && *target == setup.female
                && targets == &[setup.target, setup.female]
    )));
    assert_eq!(state.players.get(setup.target).unwrap().hp, 9);
    assert_eq!(state.players.get(setup.female).unwrap().hp, 9);
    assert_eq!(state.cards.get(gift).unwrap().owner, setup.female);
}

#[test]
fn chanhui_gift_branch_transfers_card_user_before_damage() {
    let mut setup = setup(2_026_073_702);
    let sha = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_711,
        CardKind::Sha,
        Suit::Club,
    );
    let gift = inject(
        &mut setup.state,
        setup.female,
        ZoneKind::Hand,
        99_712,
        CardKind::Other,
        Suit::Diamond,
    );
    let (state, _) = play_sha(setup.state, setup.owner, sha, setup.target, 10);
    let (state, _) = resolve(state, setup.owner, vec![1], false, 11);
    let (state, _) = resolve(state, setup.owner, vec![setup.female.raw()], false, 12);
    let owner_hand = state.hand_count(setup.owner);
    let (state, transferred) = resolve(state, setup.female, vec![gift.raw()], false, 13);
    assert!(
        matches!(transferred.result, CommandResult::Accepted { .. }),
        "unexpected transfer result: {:?}",
        transferred.result
    );
    assert!(transferred.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardUserChanged { previous, player, card }
            if *previous == setup.owner && *player == setup.female && *card == sha
    )));
    assert!(transferred.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt { source, target, amount: 1, .. }
            if *source == setup.female && *target == setup.target
    )));
    assert_eq!(state.hand_count(setup.owner), owner_hand + 1);
    assert_eq!(state.cards.get(gift).unwrap().owner, setup.owner);
    assert_eq!(state.players.get(setup.target).unwrap().hp, 9);
}

#[test]
fn chanhui_accepts_black_but_not_red_single_target_ordinary_tricks() {
    let mut red_setup = setup(2_026_073_706);
    let red_duel = inject(
        &mut red_setup.state,
        red_setup.owner,
        ZoneKind::Hand,
        99_713,
        CardKind::TrickJuedou,
        Suit::Heart,
    );
    let (state, red) = submit(
        red_setup.state,
        red_setup.owner,
        CommandKind::PlayCard {
            card: red_duel,
            targets: vec![red_setup.target],
            card_targets: Vec::new(),
        },
        14,
    );
    assert!(matches!(red.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(red_setup.target).unwrap().hp, 9);

    let mut black_setup = setup(2_026_073_707);
    let black_duel = inject(
        &mut black_setup.state,
        black_setup.owner,
        ZoneKind::Hand,
        99_714,
        CardKind::TrickJuedou,
        Suit::Club,
    );
    let (state, black) = submit(
        black_setup.state,
        black_setup.owner,
        CommandKind::PlayCard {
            card: black_duel,
            targets: vec![black_setup.target],
            card_targets: Vec::new(),
        },
        15,
    );
    assert_eq!(black.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let (state, _) = resolve(state, black_setup.owner, vec![1], false, 16);
    let (state, added) = resolve(
        state,
        black_setup.owner,
        vec![black_setup.female.raw()],
        false,
        17,
    );
    assert!(matches!(added.result, CommandResult::Accepted { .. }));
    assert!(added.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardTargetAdded { card, target, targets, .. }
            if *card == black_duel && *target == black_setup.female
                && targets == &[black_setup.target, black_setup.female]
    )));
    assert_eq!(state.players.get(black_setup.target).unwrap().hp, 9);
    assert_eq!(state.players.get(black_setup.female).unwrap().hp, 9);
}

#[test]
fn jiaojin_discards_effective_equipment_and_reduces_or_prevents_male_damage() {
    let mut first_setup = setup(2_026_073_703);
    let equipment = inject(
        &mut first_setup.state,
        first_setup.owner,
        ZoneKind::Hand,
        99_721,
        CardKind::Weapon,
        Suit::Spade,
    );
    let basic = inject(
        &mut first_setup.state,
        first_setup.owner,
        ZoneKind::Hand,
        99_722,
        CardKind::Tao,
        Suit::Heart,
    );
    let (state, _, optional) =
        open_damage(first_setup.state, first_setup.target, first_setup.owner, 2);
    assert_eq!(optional.kind, ChoiceKind::Confirm);
    let (state, cost) = resolve(state, first_setup.owner, vec![1], false, 20);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardEquipmentReduceCurrentDamage
    );
    assert_eq!(
        cost.choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [equipment.raw()]
    );
    let before = canonical_state_hash(&state);
    let (state, forged) = resolve(state, first_setup.owner, vec![basic.raw()], false, 21);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    let (state, reduced) = resolve(state, first_setup.owner, vec![equipment.raw()], false, 22);
    assert!(matches!(reduced.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(first_setup.owner).unwrap().hp, 9);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(equipment).unwrap().zone, discard);

    let mut setup = setup(2_026_073_704);
    let equipment = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Equip,
        99_723,
        CardKind::Armor,
        Suit::Club,
    );
    let (state, _, _) = open_damage(setup.state, setup.target, setup.owner, 1);
    let (state, _) = resolve(state, setup.owner, vec![1], false, 23);
    let (state, prevented) = resolve(state, setup.owner, vec![equipment.raw()], false, 24);
    assert!(matches!(prevented.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 10);
    assert_eq!(state.resolution.current_damage_id, 0);
    assert!(state.resolution.prevented_damage_ids.is_empty());
}

#[test]
fn jiaojin_does_not_trigger_for_female_damage_source() {
    let mut setup = setup(2_026_073_705);
    inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_731,
        CardKind::Treasure,
        Suit::Diamond,
    );
    let mut tx = Transaction::begin(setup.state);
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: setup.female,
            target: setup.owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt { source, target, amount: 1, .. }
            if *source == setup.female && *target == setup.owner
    )));
    let state = tx.commit().state;
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 9);
    assert!(!state.has_pending_choice());
}
}

mod original_yijiang_xinbenxi {
//! Golden for 界吴懿「奔袭」rewritten from legacy JavaScript semantics.

const RE_WUYI: u32 = 3_817_550_013;
const XINBENXI: u32 = 3_506_100_401;
const XINBENXI_EFFECT: u32 = 4_250_961_579;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/xinbenxi.skill");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

#[test]
fn packages_publish_xinbenxi_policy_and_complete_re_wuyi() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert_eq!(yijiang.skill_index.len(), 281);
    assert_eq!(yijiang.character_index.len(), 72);
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINBENXI && skill.locked));
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINBENXI_EFFECT));
    assert!(yijiang.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == XINBENXI
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_single_target_sha_or_ordinary_trick_current_turn"
    }));
    assert!(yijiang.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == XINBENXI
            && modifier.target == "distance_from"
            && modifier.value_source.as_deref() == Some("owner_current_turn_card_use_count")
            && modifier.value == -1
    }));
    assert!(yijiang.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == XINBENXI_EFFECT
            && modifier.target == "ignore_armor_for_skill_card_policy"
    }));
    assert!(yijiang.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == XINBENXI_EFFECT
            && modifier.target == "card_use_unrespondable_for_skill_card_policy"
    }));
    assert!(yijiang.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == XINBENXI_EFFECT
            && trigger.event == "damage"
            && trigger.filter == "owner_source_event_card_policy_draw"
    }));

    let compiled = content_compiler::compile_source_named("xinbenxi", SOURCE).unwrap();
    assert_eq!(compiled.modules.len(), 3);
    for expected in &compiled.modules {
        let actual = yijiang
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }

    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(refresh.manifest.rule_vm_abi_version >= 251);
    assert!(refresh.skill_index.len() >= 76);
    let character = refresh
        .character_index
        .iter()
        .find(|character| character.id == RE_WUYI)
        .unwrap();
    assert_eq!(character.skills, [XINBENXI]);
}
}

mod original_yijiang_xindangxian {
//! Golden coverage for 界廖化「当先」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINDANGXIAN: u32 = 3_456_939_183;
const XINFULI: u32 = 3_062_292_376;
const RE_LIAOHUA: u32 = 1_916_827_798;
const YIJIANG_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const REFRESH_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/yijiang/xindangxian.skill"),
    "\n",
    include_str!("../../../content/original/ported/yijiang/xinfuli.skill"),
);

struct Setup {
    state: GameState,
    owner: PlayerId,
    previous: PlayerId,
    sha: CardId,
}

fn setup(seed: u64, owns_xinfuli: bool) -> Setup {
    let package = content_compiler::compile_source_named("xindangxian", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 200);
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();

    let seats = tx.state().players.alive_by_seat();
    let previous = tx.state().turn.current;
    let previous_index = seats.iter().position(|player| *player == previous).unwrap();
    let owner = seats[(previous_index + 1) % seats.len()];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINDANGXIAN),
    });
    if owns_xinfuli {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINFULI),
        });
    }
    effects.push(EffectRequest::SetTurnState {
        current: previous,
        phase: Phase::End.to_u8(),
    });
    run_effects(&mut tx, effects).unwrap();

    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(95_001),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
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
        .push(sha);
    Setup {
        state: tx.commit().state,
        owner,
        previous,
        sha,
    }
}

fn end_phase(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::EndPhase,
        },
    )
}

#[test]
fn committed_packages_publish_xindangxian_and_complete_re_liaohua() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG_PACKAGE).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert_eq!(yijiang.skill_index.len(), 281);
    assert_eq!(yijiang.trigger_index.len(), 215);
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINDANGXIAN && skill.locked));

    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH_PACKAGE).unwrap();
    assert!(refresh.manifest.rule_vm_abi_version >= 251);
    let character = refresh
        .character_index
        .iter()
        .find(|character| character.id == RE_LIAOHUA)
        .unwrap();
    assert_eq!(character.skills, [XINDANGXIAN, XINFULI]);
}

#[test]
fn unused_xinfuli_forces_hp_loss_then_gains_discard_sha() {
    let setup = setup(2_026_072_601, true);
    let before_hp = setup.state.players.get(setup.owner).unwrap().hp;
    let hand = setup.state.zones.find(ZoneKind::Hand, setup.owner).unwrap();

    let (state, result) = end_phase(setup.state, setup.previous, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.current, setup.owner);
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE), 1);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, before_hp - 1);
    assert_eq!(state.cards.get(setup.sha).unwrap().zone, hand);
}

#[test]
fn consumed_xinfuli_makes_cost_optional_and_cancel_preserves_state() {
    let setup = setup(2_026_072_602, false);
    let before_hp = setup.state.players.get(setup.owner).unwrap().hp;
    let discard = setup
        .state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();

    let (state, offered) = end_phase(setup.state, setup.previous, 2);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, cancelled) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: Vec::new(),
                cancelled: true,
            },
        },
    );
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, before_hp);
    assert_eq!(state.cards.get(setup.sha).unwrap().zone, discard);
}

#[test]
fn hp_loss_gain_is_inert_outside_its_originating_extra_phase() {
    let setup = setup(2_026_072_603, true);
    let skill = setup
        .state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == setup.owner && skill.def.raw() == XINDANGXIAN)
        .unwrap()
        .id;
    let before_hp = setup.state.players.get(setup.owner).unwrap().hp;
    let sha_zone = setup.state.cards.get(setup.sha).unwrap().zone;
    let mut tx = Transaction::begin(setup.state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay {
                player: setup.owner,
                kind: CardKind::Sha.tag(),
                mandatory_skill_def: SkillDefId::new(XINFULI),
                skill,
            }]
        ),
        Ok(Vec::new())
    );
    assert_eq!(tx.state().players.get(setup.owner).unwrap().hp, before_hp);
    assert_eq!(tx.state().cards.get(setup.sha).unwrap().zone, sha_zone);
}
}

mod original_yijiang_xinjuece {
//! Golden coverage for 界李儒「绝策」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const XINJUECE: u32 = 3_145_104_894;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/xinjuece.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_501, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, empty, other) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINJUECE),
    });
    effects.push(EffectRequest::DiscardFromHand {
        player: empty,
        count: u32::MAX,
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, empty, other)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
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
                cancelled: false,
            },
        },
    )
}

fn offer(mut tx: Transaction, owner: PlayerId) -> (GameState, PipelineResult) {
    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new(),),
        Err(EffectError::ChoiceRequested)
    );
    let (state, output) = resolve(tx.state().clone(), owner, vec![1], 1);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetsDamage
    );
    (state, output)
}

#[test]
fn package_contract_and_hand_empty_scope_require_abi162() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    assert_eq!(package.skill_index.len(), 281);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == XINJUECE && skill.name == "xinjuece" && !skill.locked
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == XINJUECE
            && trigger.event == "phase_end"
            && trigger.filter == "owner_is_target_any_alive_hand_empty"
            && trigger.optional
    }));
    let compiled = content_compiler::compile_source_named("xinjuece", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 162);
}

#[test]
fn only_empty_handed_players_are_offered_and_the_selected_player_takes_damage() {
    let (tx, owner, empty, other) = setup();
    let hp = tx.state().players.get(empty).unwrap().hp;
    let (state, choice_output) = offer(tx, owner);
    let choice = choice_output.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    let options: Vec<_> = choice.options.iter().map(|option| option.token).collect();
    assert_eq!(options, [empty.raw()]);
    assert!(!options.contains(&owner.raw()));
    assert!(!options.contains(&other.raw()));

    let (state, output) = resolve(state, owner, vec![empty.raw()], 2);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(empty).unwrap().hp, hp - 1);
}

#[test]
fn owner_can_target_self_when_empty_and_stale_nonempty_target_is_rejected() {
    let (mut tx, owner, empty, _) = setup();
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let (state, choice_output) = offer(tx, owner);
    let options: Vec<_> = choice_output
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert!(options.contains(&owner.raw()));
    assert!(options.contains(&empty.raw()));

    let mut stale = state;
    let draw = stale
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let hand = stale.zones.find(ZoneKind::Hand, empty).unwrap();
    let card = stale.zones.get(draw).unwrap().cards[0];
    assert!(stale.zones.remove_card(draw, card));
    assert!(stale.zones.push_card(hand, card));
    let hp = stale.players.get(empty).unwrap().hp;
    let (stale, output) = resolve(stale, owner, vec![empty.raw()], 3);
    assert!(matches!(output.result, CommandResult::Rejected { .. }));
    assert_eq!(stale.players.get(empty).unwrap().hp, hp);
}

#[test]
fn no_empty_handed_player_suppresses_the_trigger() {
    let (mut tx, owner, empty, _) = setup();
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: empty,
            count: 1,
        }],
    )
    .unwrap();
    let effects =
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new()).unwrap();
    assert!(effects.is_empty());
    assert!(!tx.state().has_pending_choice());
}
}

mod original_yijiang_xinsidi {
//! Golden coverage for 曹真「司敌」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::query::{
    card_target_within_distance, card_use_cancelled_for_card,
    discardable_non_basic_matching_equip_color_cards, dying_save_card,
};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const CAOZHEN: u32 = 2_266_553_256;
const XINSIDI: u32 = 2_768_966_746;
const XINSIDI2: u32 = 3_172_356_775;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/xinsidi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    attacker: PlayerId,
    target: PlayerId,
    red_equip: CardId,
    red_cost: CardId,
    red_basic: CardId,
    black_non_basic: CardId,
    black_save: CardId,
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
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

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 5, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let attacker = seats[1];
    let target = seats[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINSIDI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    let red_equip = inject(
        tx.state_mut(),
        owner,
        ZoneKind::Equip,
        99_201,
        CardKind::Weapon,
        Suit::Heart,
    );
    let red_cost = inject(
        tx.state_mut(),
        owner,
        ZoneKind::Hand,
        99_202,
        CardKind::TrickWuxie,
        Suit::Diamond,
    );
    let red_basic = inject(
        tx.state_mut(),
        owner,
        ZoneKind::Hand,
        99_203,
        CardKind::Tao,
        Suit::Heart,
    );
    let black_non_basic = inject(
        tx.state_mut(),
        owner,
        ZoneKind::Hand,
        99_204,
        CardKind::TrickWuxie,
        Suit::Club,
    );
    let black_save = inject(
        tx.state_mut(),
        target,
        ZoneKind::Hand,
        99_205,
        CardKind::Jiu,
        Suit::Club,
    );
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut()
        .turn
        .set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    Setup {
        state: tx.commit().state,
        owner,
        attacker,
        target,
        red_equip,
        red_cost,
        red_basic,
        black_non_basic,
        black_save,
    }
}

fn helper_id(state: &GameState, target: PlayerId) -> Option<engine_api::SkillId> {
    state
        .players
        .get(target)?
        .skills
        .iter()
        .copied()
        .find(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == XINSIDI2)
        })
}

fn open_xinsidi(state: GameState, target: PlayerId) -> (GameState, Vec<u32>) {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: target,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let paused = run_effects(&mut tx, effects).unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    let committed = tx.commit();
    let options = committed
        .output
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    (committed.state, options)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn locked(seed: u64) -> Setup {
    let mut setup = setup(seed);
    let (state, _) = open_xinsidi(setup.state, setup.target);
    let (state, result) = resolve(state, setup.owner, vec![setup.red_cost.raw()], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    setup.state = state;
    setup
}

fn end_play(state: GameState, target: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: target,
            phase: TurnPhaseTag::Play,
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
    tx.commit().state
}

#[test]
fn package_publishes_complete_caozhen_and_exact_abi53_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CAOZHEN)
        .unwrap();
    assert_eq!(character.skills, [XINSIDI]);
    let compiled = content_compiler::compile_source_named("xinsidi", SOURCE).unwrap();
    assert_eq!(compiled.modules.len(), 2);
    for expected in &compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn phase_begin_offers_only_matching_color_non_basic_hand_and_equipment_cards() {
    let setup = setup(2_026_072_201);
    assert_eq!(
        discardable_non_basic_matching_equip_color_cards(&setup.state, setup.owner),
        [setup.red_cost, setup.red_equip]
    );
    let owner = setup.owner;
    let target = setup.target;
    let (state, options) = open_xinsidi(setup.state, target);
    assert_eq!(options, [setup.red_cost.raw(), setup.red_equip.raw()]);
    assert!(!options.contains(&setup.red_basic.raw()));
    assert!(!options.contains(&setup.black_non_basic.raw()));

    let (state, cancelled) = resolve(state, owner, Vec::new(), true, 2);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert!(helper_id(&state, target).is_none());
    assert_eq!(state.cards.get(setup.red_cost).unwrap().owner, owner);
}

#[test]
fn stale_or_forged_cost_is_rejected_without_discard_or_skill_grant() {
    let setup = setup(2_026_072_202);
    let owner = setup.owner;
    let target = setup.target;
    let (state, _) = open_xinsidi(setup.state, target);
    let (state, forged) = resolve(state, owner, vec![setup.black_non_basic.raw()], false, 3);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert!(state.has_pending_choice());
    assert!(helper_id(&state, target).is_none());
    assert_eq!(state.cards.get(setup.black_non_basic).unwrap().owner, owner);
}

#[test]
fn chosen_color_blocks_physical_use_response_and_save_but_not_the_other_color() {
    let mut setup = locked(2_026_072_203);
    let discard = setup
        .state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(setup.state.cards.get(setup.red_cost).unwrap().zone, discard);
    let helper = helper_id(&setup.state, setup.target).unwrap();
    assert_eq!(setup.state.skills.get(helper).unwrap().state_blob[8], 0b01);

    let red_tao = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        99_206,
        CardKind::Tao,
        Suit::Diamond,
    );
    let black_trick = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        99_207,
        CardKind::TrickWuxie,
        Suit::Spade,
    );
    assert!(card_use_cancelled_for_card(
        &setup.state,
        setup.target,
        red_tao
    ));
    assert!(!card_use_cancelled_for_card(
        &setup.state,
        setup.target,
        black_trick
    ));
    assert_eq!(
        dying_save_card(&setup.state, setup.target),
        Some(setup.black_save)
    );

    let red_shan = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        99_208,
        CardKind::Shan,
        Suit::Heart,
    );
    let black_shan = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        99_209,
        CardKind::Shan,
        Suit::Club,
    );
    let sha = inject(
        &mut setup.state,
        setup.attacker,
        ZoneKind::Hand,
        99_210,
        CardKind::Sha,
        Suit::Spade,
    );
    setup.state.turn.current = setup.attacker;
    setup.state.turn.phase = Phase::Play;
    let (state, played) = apply_command(
        setup.state,
        Command {
            actor: setup.attacker,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![setup.target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let options: Vec<_> = played
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(!options.contains(&red_shan.raw()));
    assert!(options.contains(&black_shan.raw()));
    assert!(state.has_pending_choice());
}

#[test]
fn no_sha_phase_end_retaliates_from_recorded_source_ignoring_distance_then_cleans_up() {
    let mut setup = locked(2_026_072_204);
    let probe_sha = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_211,
        CardKind::Sha,
        Suit::Club,
    );
    assert!(!card_target_within_distance(
        &setup.state,
        setup.owner,
        probe_sha,
        setup.target,
        1
    ));
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let state = end_play(setup.state, setup.target);
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
    assert!(helper_id(&state, setup.target).is_none());
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE), 0);
}

#[test]
fn repeated_grants_merge_colors_replace_source_and_sha_use_suppresses_retaliation() {
    let mut setup = locked(2_026_072_205);
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddCardColorLockSkill {
            player: setup.target,
            source: setup.attacker,
            color_mask: 0b10,
            skill_def: SkillDefId::new(XINSIDI2),
        }],
    )
    .unwrap();
    setup.state = tx.commit().state;
    let helpers: Vec<_> = setup
        .state
        .players
        .get(setup.target)
        .unwrap()
        .skills
        .iter()
        .copied()
        .filter(|skill| setup.state.skills.get(*skill).unwrap().def.raw() == XINSIDI2)
        .collect();
    assert_eq!(helpers.len(), 1);
    let blob = &setup.state.skills.get(helpers[0]).unwrap().state_blob;
    assert_eq!(blob[8], 0b11);
    assert_eq!(
        u32::from_le_bytes(blob[4..8].try_into().unwrap()),
        setup.attacker.raw()
    );
    assert!(card_use_cancelled_for_card(
        &setup.state,
        setup.target,
        setup.black_save
    ));

    setup
        .state
        .turn
        .set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 1);
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let state = end_play(setup.state, setup.target);
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp);
    assert!(helper_id(&state, setup.target).is_none());
}
}

mod original_yijiang_xinxianzhen {
//! Golden coverage for 高顺「陷阵」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{
    card_target_within_distance, sha_ignored_for_hand_limit, sha_use_limit_for,
    sha_use_unlimited_for_target,
};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const GAOSHUN: u32 = 2_843_282_379;
const XINXIANZHEN: u32 = 3_226_686_922;
const XINXIANZHEN_WIN: u32 = 2_960_051_687;
const XINXIANZHEN_LOSE: u32 = 3_077_071_581;
const JINJIU: u32 = 2_885_443_165;
const TEST_ARMOR: u32 = 1_990_000_001;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/xinxianzhen.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const ARMOR_SOURCE: &str = r#"
skill xinxianzhen_test_armor(1990000001) {
  modifier armor_damage_taken_amount(fire) add 1;
}
"#;

struct Setup {
    state: GameState,
    owner: PlayerId,
    near: PlayerId,
    bound: PlayerId,
    far: PlayerId,
    owner_compare: CardId,
    target_compare: CardId,
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    def: u32,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, owner_rank: u8, target_rank: Option<u8>) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let armor = content_compiler::compile_source_named("xinxianzhen_armor", ARMOR_SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 5, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&armor).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let near = seats[1];
    let bound = seats[2];
    let far = seats[3];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINXIANZHEN),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    let owner_compare = inject(
        tx.state_mut(),
        owner,
        99_101,
        CardKind::Other,
        Suit::Spade,
        owner_rank,
    );
    let target_compare = target_rank
        .map(|rank| {
            inject(
                tx.state_mut(),
                bound,
                99_102,
                CardKind::Other,
                Suit::Heart,
                rank,
            )
        })
        .unwrap_or(CardId::NONE);
    Setup {
        state: tx.commit().state,
        owner,
        near,
        bound,
        far,
        owner_compare,
        target_compare,
    }
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn compare(setup: &Setup, key: u64) -> GameState {
    let active = skill_id(&setup.state, setup.owner, XINXIANZHEN);
    let (state, choose_owner_card) = command(
        setup.state.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.bound],
            card_targets: Vec::new(),
        },
        key,
    );
    assert!(matches!(
        choose_owner_card.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ActiveSkillSelection
    );
    let (state, choose_target_card) = resolve(
        state,
        setup.owner,
        vec![setup.owner_compare.raw()],
        false,
        key + 1,
    );
    assert!(matches!(
        choose_target_card.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCompareTargetHandCard
    );
    let (state, compared) = resolve(
        state,
        setup.bound,
        vec![setup.target_compare.raw()],
        false,
        key + 2,
    );
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    state
}

#[test]
fn package_publishes_complete_gaoshun_and_exact_abi53_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == GAOSHUN)
        .unwrap();
    assert_eq!(character.skills, [XINXIANZHEN, JINJIU]);
    let compiled = content_compiler::compile_source_named("xinxianzhen", SOURCE).unwrap();
    assert_eq!(compiled.modules.len(), 1);
    for expected in &compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn strict_win_binds_only_the_compared_target_for_distance_use_count_and_armor() {
    let setup = setup(2_026_072_101, 13, Some(1));
    let mut state = compare(&setup, 1);
    let win = skill_id(&state, setup.owner, XINXIANZHEN_WIN);
    assert_eq!(state.skills.get(win).unwrap().def.raw(), XINXIANZHEN_WIN);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(setup.owner_compare).unwrap().zone, discard);
    assert_eq!(state.cards.get(setup.target_compare).unwrap().zone, discard);

    let sha = inject(
        &mut state,
        setup.owner,
        99_103,
        CardKind::Sha,
        Suit::Club,
        7,
    );
    assert!(card_target_within_distance(
        &state,
        setup.owner,
        sha,
        setup.bound,
        1
    ));
    assert!(!card_target_within_distance(
        &state,
        setup.owner,
        sha,
        setup.far,
        1
    ));
    assert!(sha_use_unlimited_for_target(
        &state,
        setup.owner,
        setup.bound
    ));
    assert!(!sha_use_unlimited_for_target(
        &state,
        setup.owner,
        setup.near
    ));

    state.turn.set_flag(turn_flag::SHA_PLAYED, 1);
    state.turn.set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 1);
    let (bound_state, bound_use) = command(
        state.clone(),
        setup.owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![setup.bound],
            card_targets: Vec::new(),
        },
        10,
    );
    assert!(
        matches!(bound_use.result, CommandResult::Accepted { .. }),
        "bound use result: {:?}",
        bound_use.result
    );
    assert_eq!(bound_state.players.get(setup.bound).unwrap().hp, 9);
    let (_, ordinary_target_use) = command(
        state.clone(),
        setup.owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![setup.near],
            card_targets: Vec::new(),
        },
        11,
    );
    assert!(matches!(
        ordinary_target_use.result,
        CommandResult::Rejected { .. }
    ));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: setup.bound,
                skill_def: SkillDefId::new(TEST_ARMOR),
            },
            EffectRequest::AddSkill {
                player: setup.far,
                skill_def: SkillDefId::new(TEST_ARMOR),
            },
            EffectRequest::DealDamage {
                source: setup.owner,
                target: setup.bound,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::FIRE,
            },
            EffectRequest::DealDamage {
                source: setup.owner,
                target: setup.far,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::FIRE,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(setup.bound).unwrap().hp, 9);
    assert_eq!(tx.state().players.get(setup.far).unwrap().hp, 8);
}

#[test]
fn success_helper_appends_bound_target_to_sha_and_independent_trick() {
    let setup = setup(2_026_072_102, 12, Some(2));
    let mut state = compare(&setup, 20);
    let sha = inject(
        &mut state,
        setup.owner,
        99_104,
        CardKind::Sha,
        Suit::Diamond,
        8,
    );
    let (state_after_play, add_sha) = command(
        state.clone(),
        setup.owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![setup.near],
            card_targets: Vec::new(),
        },
        23,
    );
    assert!(matches!(add_sha.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state_after_play.resolution.continuation_kind,
        ContinuationKind::ChooseAddSkillBoundCardTarget,
        "play result={:?} events={:?}",
        add_sha.result,
        add_sha.events
    );
    let (sha_done, added) = resolve(
        state_after_play,
        setup.owner,
        vec![setup.bound.raw()],
        false,
        24,
    );
    assert!(matches!(added.result, CommandResult::Accepted { .. }));
    assert!(added.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardTargetAdded { target, targets, .. }
            if *target == setup.bound && targets == &[setup.near, setup.bound]
    )));
    assert_eq!(sha_done.players.get(setup.near).unwrap().hp, 9);
    assert_eq!(sha_done.players.get(setup.bound).unwrap().hp, 9);

    let duel = inject(
        &mut state,
        setup.owner,
        99_105,
        CardKind::TrickJuedou,
        Suit::Spade,
        9,
    );
    let (duel_state, add_duel) = command(
        state,
        setup.owner,
        CommandKind::PlayCard {
            card: duel,
            targets: vec![setup.near],
            card_targets: Vec::new(),
        },
        25,
    );
    assert!(matches!(add_duel.result, CommandResult::Accepted { .. }));
    assert_eq!(
        duel_state.resolution.continuation_kind,
        ContinuationKind::ChooseAddSkillBoundCardTarget
    );
    let (duel_done, added) = resolve(duel_state, setup.owner, vec![setup.bound.raw()], false, 26);
    assert!(matches!(added.result, CommandResult::Accepted { .. }));
    assert!(added.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardTargetAdded { target, targets, .. }
            if *target == setup.bound && targets == &[setup.near, setup.bound]
    )));
    assert_eq!(duel_done.players.get(setup.near).unwrap().hp, 9);
    assert_eq!(duel_done.players.get(setup.bound).unwrap().hp, 9);
}

#[test]
fn tie_is_failure_sha_is_disabled_and_temporary_helpers_expire() {
    let setup = setup(2_026_072_103, 7, Some(7));
    let state = compare(&setup, 30);
    assert!(state
        .players
        .get(setup.owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| state.skills.get(*skill).unwrap().def.raw() == XINXIANZHEN_LOSE));
    assert_eq!(sha_use_limit_for(&state, setup.owner), 0);
    assert!(sha_ignored_for_hand_limit(&state, setup.owner));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: setup.owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: setup.owner,
                phase: Phase::Idle.to_u8(),
            },
        ],
    )
    .unwrap();
    assert!(tx
        .state()
        .players
        .get(setup.owner)
        .unwrap()
        .skills
        .iter()
        .all(|skill| {
            !matches!(
                tx.state().skills.get(*skill).unwrap().def.raw(),
                XINXIANZHEN_WIN | XINXIANZHEN_LOSE
            )
        }));
}

#[test]
fn success_helper_appends_bound_target_to_single_target_tiesuo() {
    let setup = setup(2_026_072_105, 11, Some(3));
    let mut state = compare(&setup, 50);
    let tiesuo = inject(
        &mut state,
        setup.owner,
        99_106,
        CardKind::TrickTiesuo,
        Suit::Club,
        6,
    );
    let (state, add_target) = command(
        state,
        setup.owner,
        CommandKind::PlayCard {
            card: tiesuo,
            targets: vec![setup.near],
            card_targets: Vec::new(),
        },
        53,
    );
    assert!(matches!(add_target.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddSkillBoundCardTarget
    );
    let (state, added) = resolve(state, setup.owner, vec![setup.bound.raw()], false, 54);
    assert!(matches!(added.result, CommandResult::Accepted { .. }));
    assert!(added.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardTargetAdded { target, targets, .. }
            if *target == setup.bound && targets == &[setup.near, setup.bound]
    )));
    for target in [setup.near, setup.bound] {
        assert!(state
            .players
            .get(target)
            .unwrap()
            .marks
            .iter()
            .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count == 1));
    }
}

#[test]
fn comparison_target_without_hand_is_rejected_before_owner_card_choice() {
    let setup = setup(2_026_072_104, 11, None);
    let active = skill_id(&setup.state, setup.owner, XINXIANZHEN);
    let (state, result) = command(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.bound],
            card_targets: Vec::new(),
        },
        40,
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert!(!state.has_pending_choice());
}
}

mod original_yijiang_xinyaoming_re_quancong {
//! Golden coverage for 界全琮「邀名」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_QUANCONG: u32 = 387_181_137;
const XINYAOMING: u32 = 3_897_075_796;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/xinyaoming.skill");
const YIJIANG_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const REFRESH_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("xinyaoming-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_730, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let other = players[2];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(players.iter().map(|player| EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        }))
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINYAOMING),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().players.get_mut(owner).unwrap().character_def = RE_QUANCONG;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        other,
    }
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
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

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn open_trigger(state: GameState, owner: PlayerId, target: PlayerId, key: u64) -> GameState {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let state = tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, result) = resolve(state, owner, vec![1], key);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveXinyaoming
    );
    state
}

#[test]
fn packages_publish_abi319_skill_and_complete_refresh_character() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG_PACKAGE).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINYAOMING));
    assert!(yijiang.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == XINYAOMING
            && trigger.event == "damage"
            && trigger.filter == "owner_is_source_or_target_xinyaoming_available"
            && trigger.optional
    }));
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH_PACKAGE).unwrap();
    let character = refresh
        .character_index
        .iter()
        .find(|character| character.id == RE_QUANCONG)
        .unwrap();
    assert_eq!(character.name_key, "re_quancong");
    assert_eq!(character.skills, [XINYAOMING]);
}

#[test]
fn three_actions_are_each_once_per_turn_and_reset_next_turn() {
    let mut setup = setup();
    for def in [99_401, 99_402, 99_403] {
        inject_hand(&mut setup.state, setup.target, def);
    }

    let state = open_trigger(setup.state, setup.owner, setup.target, 1);
    let (state, target_choice) = resolve(state, setup.owner, vec![setup.target.raw()], 2);
    assert_eq!(
        target_choice.choice.as_ref().unwrap().kind,
        ChoiceKind::Custom
    );
    assert_eq!(
        target_choice
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    let before_draw = state.hand_count(setup.target);
    let (state, result) = resolve(state, setup.owner, vec![1], 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), before_draw + 1);

    let state = open_trigger(state, setup.owner, setup.target, 4);
    let (state, action_choice) = resolve(state, setup.owner, vec![setup.target.raw()], 5);
    assert_eq!(
        action_choice
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [2, 3]
    );
    let (state, discard_choice) = resolve(state, setup.owner, vec![2], 6);
    assert_eq!(
        discard_choice.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let discard_card = discard_choice.choice.as_ref().unwrap().options[0].token;
    let before_discard = state.hand_count(setup.target);
    let (state, result) = resolve(state, setup.owner, vec![discard_card], 7);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), before_discard - 1);

    let state = open_trigger(state, setup.owner, setup.target, 8);
    let (state, discard_draw_choice) = resolve(state, setup.owner, vec![setup.target.raw()], 9);
    assert_eq!(
        discard_draw_choice.choice.as_ref().unwrap().observer,
        setup.target
    );
    let cards: Vec<_> = discard_draw_choice
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .take(2)
        .map(|option| option.token)
        .collect();
    let before_exchange = state.hand_count(setup.target);
    let (state, result) = resolve(state, setup.target, cards, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), before_exchange);

    let mut tx = Transaction::begin(state);
    let fourth = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: setup.owner,
            target: setup.target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    assert!(fourth.is_ok());
    assert!(!tx.state().has_pending_choice());
    tx.state_mut().turn.phase = Phase::End;
    run_effects(&mut tx, vec![EffectRequest::AdvanceTurn]).unwrap();
    let state = tx.commit().state;
    let state = open_trigger(state, setup.owner, setup.target, 11);
    assert!(state.has_pending_choice());
}

#[test]
fn uninvolved_damage_does_not_offer_xinyaoming() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: setup.target,
            target: setup.other,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    assert!(result.is_ok());
    assert!(!tx.state().has_pending_choice());
}
}

mod original_yijiang_xinzhan {
//! Golden coverage for YIJIANG「心战」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    ObserverKind, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{canonical_state_hash, project_observer};
use rule_api::EffectRequest;

const XINZHAN: u32 = 3_275_354_148;
const HUILEI: u32 = 2_889_506_960;
const MASU: u32 = 3_141_577_158;
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/xinzhan.skill");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let players = tx.state().players.alive_by_seat();
    let owner = tx.state().turn.current;
    let other = players.into_iter().find(|player| *player != owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINZHAN),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == XINZHAN)
        .unwrap();
    (tx.state().clone(), owner, other, skill)
}

fn inject_top(state: &mut GameState, cards: &[(Suit, u32)]) -> Vec<CardId> {
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    cards
        .iter()
        .map(|(suit, def)| {
            let card = state.cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::new(*def),
                kind: CardKind::Other,
                suit: *suit,
                rank: 7,
                weapon_range: 0,
                zone: draw,
                owner: PlayerId::NONE,
                face_up: false,
            });
            state.zones.get_mut(draw).unwrap().cards.push(card);
            card
        })
        .collect()
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
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
fn packages_publish_exact_skill_character_and_dependency() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    let compiled = content_compiler::compile_source_named("xinzhan", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = yijiang
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
    let active = yijiang
        .active_index
        .iter()
        .find(|active| active.skill_def_id == XINZHAN)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.hand_gt_max_hp);
    assert!(!active.hand_gt_hp);

    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    assert!(old.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "yijiang"
            && dependency.version == "0.1.0"
    }));
    let masu = old
        .character_index
        .iter()
        .find(|character| character.id == MASU)
        .unwrap();
    assert_eq!((masu.name_key.as_str(), masu.max_hp), ("masu", 3));
    assert_eq!(masu.skills, [XINZHAN, HUILEI]);
}

#[test]
fn activation_requires_hand_above_max_hp_and_is_limited_once() {
    let (mut state, owner, _, skill) = started(2_026_071_801);
    let hand = state.hand_count(owner);
    state.players.get_mut(owner).unwrap().max_hp = hand;
    let before = canonical_state_hash(&state);
    let (same, rejected) = activate(state, owner, skill, 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut state = same;
    state.players.get_mut(owner).unwrap().max_hp = hand - 1;
    inject_top(&mut state, &[(Suit::Heart, 98_000)]);
    let (state, accepted) = activate(state, owner, skill, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(accepted.choice.is_some());
    let (state, cancelled) = resolve(state, owner, Vec::new(), true, 3);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);
    let (same, rejected) = activate(state.clone(), owner, skill, 4);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), canonical_state_hash(&state));
}

#[test]
fn private_peek_offers_only_hearts_and_selected_subset_is_publicly_gained() {
    let (mut state, owner, other, skill) = started(2_026_071_802);
    state.players.get_mut(owner).unwrap().max_hp = 3;
    let top = inject_top(
        &mut state,
        &[
            (Suit::Heart, 98_010),
            (Suit::Spade, 98_011),
            (Suit::Heart, 98_012),
        ],
    );
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let (state, pipeline) = activate(state, owner, skill, 1);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseGainTopCardsBySuit
    );
    assert!(pipeline.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardsRevealed { cards, to }
            if cards == &top && to == &vec![owner]
    )));
    let choice = pipeline.choice.unwrap();
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![top[0].raw(), top[2].raw()]
    );
    let owner_view = project_observer(&state, ObserverKind::Player(owner));
    let other_view = project_observer(&state, ObserverKind::Player(other));
    assert!(top.iter().all(|card| owner_view
        .known_cards
        .iter()
        .any(|known| known.id == *card && known.def.is_some())));
    assert!(top
        .iter()
        .all(|card| other_view.known_cards.iter().all(|known| known.id != *card)));

    let (state, pipeline) = resolve(state, owner, vec![top[0].raw()], false, 2);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(state.cards.get(top[0]).unwrap().zone, hand);
    assert_eq!(state.cards.get(top[0]).unwrap().owner, owner);
    let draw_cards = &state.zones.get(draw).unwrap().cards;
    assert_eq!(&draw_cards[draw_cards.len() - 2..], &[top[1], top[2]]);
    assert!(pipeline.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardsRevealed { cards, to }
            if cards == &vec![top[0]] && to.is_empty()
    )));
}

#[test]
fn cancellation_and_no_matching_suit_leave_the_draw_top_unchanged() {
    let (mut state, owner, _, skill) = started(2_026_071_803);
    state.players.get_mut(owner).unwrap().max_hp = 3;
    let top = inject_top(
        &mut state,
        &[
            (Suit::Heart, 98_020),
            (Suit::Club, 98_021),
            (Suit::Diamond, 98_022),
        ],
    );
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let (state, pipeline) = activate(state, owner, skill, 1);
    assert!(pipeline.choice.is_some());
    let (state, pipeline) = resolve(state, owner, Vec::new(), true, 2);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    let cards = &state.zones.get(draw).unwrap().cards;
    assert_eq!(&cards[cards.len() - 3..], top.as_slice());

    let (mut state, owner, _, skill) = started(2_026_071_804);
    state.players.get_mut(owner).unwrap().max_hp = 3;
    let top = inject_top(
        &mut state,
        &[
            (Suit::Spade, 98_030),
            (Suit::Club, 98_031),
            (Suit::Diamond, 98_032),
        ],
    );
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let (state, pipeline) = activate(state, owner, skill, 3);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(pipeline.choice.is_none());
    let cards = &state.zones.get(draw).unwrap().cards;
    assert_eq!(&cards[cards.len() - 3..], top.as_slice());
}

#[test]
fn forged_nonheart_selection_is_rejected_without_mutation() {
    let (mut state, owner, _, skill) = started(2_026_071_805);
    state.players.get_mut(owner).unwrap().max_hp = 3;
    let top = inject_top(
        &mut state,
        &[
            (Suit::Heart, 98_040),
            (Suit::Spade, 98_041),
            (Suit::Heart, 98_042),
        ],
    );
    let (state, pipeline) = activate(state, owner, skill, 1);
    assert!(pipeline.choice.is_some());
    let before = canonical_state_hash(&state);
    let (same, rejected) = resolve(state, owner, vec![top[1].raw()], false, 2);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
}

mod original_yijiang_xinzhongyong {
//! Golden coverage for 周仓「新忠勇」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZHOUCANG: u32 = 2_590_799_493;
const XINZHONGYONG: u32 = 3_382_820_136;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/xinzhongyong.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    recipient: PlayerId,
    other: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINZHONGYONG),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target: players[1],
        recipient: players[2],
        other: players[3],
    }
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, suit: Suit, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn confirm_trigger(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    resolve(state, owner, vec![1], false, key)
}

fn choice_tokens(result: &PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect()
}

#[test]
fn package_publishes_complete_zhoucang_and_exact_abi54_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == ZHOUCANG)
        .unwrap();
    assert_eq!(character.skills, [XINZHONGYONG]);
    let compiled = content_compiler::compile_source_named("xinzhongyong", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 54);
    assert_eq!(compiled.modules.len(), 1);
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn black_resolved_sha_is_gifted_only_to_a_non_owner_non_original_target() {
    let mut setup = setup(2_026_073_401);
    let sha = inject(
        &mut setup.state,
        setup.owner,
        CardKind::Sha,
        Suit::Spade,
        99_401,
    );
    let (state, played) = submit(
        setup.state,
        setup.owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let (state, recipient_choice) = confirm_trigger(state, setup.owner, 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseGainResolvedShaOrShanThenOptionalUseSha
    );
    let options = choice_tokens(&recipient_choice);
    assert!(!options.contains(&setup.owner.raw()));
    assert!(!options.contains(&setup.target.raw()));
    assert!(options.contains(&setup.recipient.raw()));
    assert!(options.contains(&setup.other.raw()));

    let before = canonical_state_hash(&state);
    let (same, forged) = resolve(state, setup.owner, vec![setup.target.raw()], false, 3);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let (state, gifted) = resolve(same, setup.owner, vec![setup.recipient.raw()], false, 4);
    assert!(matches!(gifted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(sha).unwrap().owner, setup.recipient);
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::None);
    assert!(state.resolution.sha_resolution_records.is_empty());
}

#[test]
fn red_shan_group_enables_count_free_sha_with_owner_range_and_owner_exclusion() {
    let mut setup = setup(2_026_073_402);
    let sha = inject(
        &mut setup.state,
        setup.owner,
        CardKind::Sha,
        Suit::Club,
        99_411,
    );
    let shan = inject(
        &mut setup.state,
        setup.target,
        CardKind::Shan,
        Suit::Heart,
        99_412,
    );
    let followup = inject(
        &mut setup.state,
        setup.recipient,
        CardKind::Sha,
        Suit::Spade,
        99_413,
    );
    let (state, response) = submit(
        setup.state,
        setup.owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        10,
    );
    assert!(matches!(response.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RespondToSha
    );
    let (state, responded) = resolve(state, setup.target, vec![shan.raw()], false, 11);
    assert!(matches!(responded.result, CommandResult::Accepted { .. }));
    let (state, recipient_choice) = confirm_trigger(state, setup.owner, 12);
    assert!(choice_tokens(&recipient_choice).contains(&setup.recipient.raw()));
    let (state, group_choice) = resolve(state, setup.owner, vec![setup.recipient.raw()], false, 13);
    assert_eq!(choice_tokens(&group_choice), [1, 2]);
    let (state, sha_source_choice) = resolve(state, setup.owner, vec![2], false, 14);
    assert_eq!(state.cards.get(shan).unwrap().owner, setup.recipient);
    assert_eq!(state.resolution.continuation_payload.first(), Some(&2));
    assert!(choice_tokens(&sha_source_choice).contains(&followup.raw()));

    let counted_before = state.turn.flag(turn_flag::SHA_PLAYED);
    let (state, target_choice) = resolve(state, setup.recipient, vec![followup.raw()], false, 15);
    let targets = choice_tokens(&target_choice);
    assert!(!targets.contains(&setup.owner.raw()));
    assert!(targets.contains(&setup.target.raw()));
    assert!(targets.contains(&setup.other.raw()));
    assert!(!targets.contains(&setup.recipient.raw()));

    let hp = state.players.get(setup.other).unwrap().hp;
    let (state, used) = resolve(state, setup.recipient, vec![setup.other.raw()], false, 16);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.other).unwrap().hp, hp - 1);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), counted_before);
    assert_eq!(
        state.cards.get(followup).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}
}

mod original_yijiang_yanyu {
//! Golden coverage for 夏侯氏「燕语」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YANYU: u32 = 2_263_835_600;
const QIAOSHI: u32 = 2_607_312_044;
const XIAHOUSHI: u32 = 4_278_886_536;
const PANZHANGMAZHONG: u32 = 3_150_630_034;
const ZHANGCHUNHUA: u32 = 2_950_326_494;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/yanyu.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    male: PlayerId,
    female: PlayerId,
    skill: SkillId,
    sha: [CardId; 3],
    non_sha: CardId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
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
    tx.state_mut().revision = 1;
    let owner = tx.state().turn.current;
    let mut others = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner);
    let male = others.next().unwrap();
    let female = others.next().unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().character_def = XIAHOUSHI;
    tx.state_mut().players.get_mut(male).unwrap().character_def = PANZHANGMAZHONG;
    tx.state_mut()
        .players
        .get_mut(female)
        .unwrap()
        .character_def = ZHANGCHUNHUA;

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
            skill_def: SkillDefId::new(YANYU),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == YANYU)
        .unwrap();
    let sha = [
        inject(tx.state_mut(), owner, 91_001, CardKind::Sha),
        inject(tx.state_mut(), owner, 91_002, CardKind::ShaFire),
        inject(tx.state_mut(), owner, 91_003, CardKind::ShaThunder),
    ];
    let non_sha = inject(tx.state_mut(), owner, 91_004, CardKind::TrickWuzhong);
    Setup {
        state: tx.commit().state,
        owner,
        male,
        female,
        skill,
        sha,
        non_sha,
    }
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    card: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: vec![card],
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn end_play(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::EndPhase,
        },
    )
}

#[test]
fn package_publishes_complete_xiahoushi_in_abi53_package() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == XIAHOUSHI)
        .unwrap();
    assert_eq!(character.skills, [QIAOSHI, YANYU]);
    let compiled = content_compiler::compile_source_named("yanyu", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn selection_only_offers_logical_sha_and_recast_counts_the_current_play_phase() {
    let setup = setup(2_026_071_801);
    let (state, offered) = apply_command(
        setup.state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    let offered_tokens: Vec<_> = offered
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(offered_tokens, setup.sha.map(CardId::raw));
    assert!(!offered_tokens.contains(&setup.non_sha.raw()));
    assert_eq!(
        state.skills.get(setup.skill).unwrap().play_phase_use_count,
        0
    );

    let hand = state.hand_count(setup.owner);
    let (state, recast) = resolve(state, setup.owner, vec![setup.sha[0].raw()], false, 2);
    assert!(matches!(recast.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), hand);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(setup.sha[0]).unwrap().zone, discard);
    let instance = state.skills.get(setup.skill).unwrap();
    assert_eq!((instance.use_count, instance.play_phase_use_count), (1, 1));
}

#[test]
fn two_recasts_offer_only_a_male_other_player_then_draw_two() {
    let setup = setup(2_026_071_802);
    let (state, first) = activate(setup.state, setup.owner, setup.skill, setup.sha[0], 10);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    let (state, second) = activate(state, setup.owner, setup.skill, setup.sha[1], 11);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.skills.get(setup.skill).unwrap().play_phase_use_count,
        2
    );

    let (state, optional) = end_play(state, setup.owner, 12);
    assert!(matches!(optional.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, target_choice) = resolve(state, setup.owner, vec![1], false, 13);
    assert!(matches!(
        target_choice.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let target_tokens: Vec<_> = target_choice
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(target_tokens, vec![setup.male.raw()]);

    let before = canonical_state_hash(&state);
    let (same, forged) = resolve(state, setup.owner, vec![setup.female.raw()], false, 14);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
    assert!(same.has_pending_choice());

    let hand = same.hand_count(setup.male);
    let (state, accepted) = resolve(same, setup.owner, vec![setup.male.raw()], false, 15);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.male), hand + 2);
    assert!(!state.has_pending_choice());
}

#[test]
fn one_recast_is_insufficient_and_entering_a_new_play_phase_resets_the_counter() {
    let setup = setup(2_026_071_803);
    let (state, used) = activate(setup.state, setup.owner, setup.skill, setup.sha[0], 20);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    let (state, ended) = end_play(state, setup.owner, 21);
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert!(ended.choice.is_none());
    assert!(!state.has_pending_choice());

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: setup.owner,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state()
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_use_count,
        0
    );
    let (state, used) = activate(
        tx.commit().state,
        setup.owner,
        setup.skill,
        setup.sha[1],
        22,
    );
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    let (state, ended) = end_play(state, setup.owner, 23);
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert!(ended.choice.is_none());
    assert!(!state.has_pending_choice());
}
}

mod original_yijiang_youdi_old_zhuhuan {
//! Golden coverage for 游敌 and old 朱桓 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YOUDI: u32 = 3_686_673_279;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/youdi.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
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
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(owner_kind: CardKind, seed: u64) -> (GameState, PlayerId, PlayerId, CardId, CardId) {
    let package = content_compiler::compile_source_named("youdi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 280);
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_phase_end_hand_equip_nonempty_other_alive"
    );
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &players {
        effects.push(EffectRequest::MoveCards {
            cards: tx.state().hand_cards(*player),
            to: discard,
            from: tx.state().zones.find(ZoneKind::Hand, *player),
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(YOUDI),
    });
    run_effects(&mut tx, effects).unwrap();
    let owner_card = inject(&mut tx, owner, owner_kind, 98_001);
    let target_card = inject(&mut tx, target, CardKind::Shan, 98_002);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def == SkillDefId::new(YOUDI))
        .unwrap()
        .id;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveYoudi {
                player: owner,
                target: PlayerId::NONE,
                card: CardId::NONE,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    (tx.commit().state, owner, target, owner_card, target_card)
}

fn resolve(state: GameState, actor: PlayerId, selection: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

#[test]
fn non_sha_grants_counter_gain_but_sha_does_not() {
    let (state, owner, target, owner_card, target_card) = setup(CardKind::Shan, 2_026_072_901);
    assert_eq!(state.resolution.pending_observer, owner.raw());
    let state = resolve(state, owner, target.raw(), 1);
    assert_eq!(state.resolution.pending_observer, target.raw());
    let state = resolve(state, target, owner_card.raw(), 2);
    assert_eq!(state.resolution.pending_observer, owner.raw());
    let state = resolve(state, owner, target_card.raw(), 3);
    assert_eq!(state.resolution.pending_choice_id, 0);
    assert!(state.hand_cards(owner).contains(&target_card));
    assert!(!state.hand_cards(target).contains(&target_card));

    let (state, owner, target, owner_sha, target_card) = setup(CardKind::Sha, 2_026_072_902);
    let state = resolve(state, owner, target.raw(), 4);
    let state = resolve(state, target, owner_sha.raw(), 5);
    assert_eq!(state.resolution.pending_choice_id, 0);
    assert!(state.hand_cards(target).contains(&target_card));
}
}

mod original_yijiang_yujin {
//! Golden coverage for 于禁「镇军」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const YUJIN: u32 = 2_365_095_387;
const REZHENJUN: u32 = 4_131_625_144;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/rezhenjun.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
    skill: engine_api::SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let other = players[2];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REZHENJUN),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
    }
    tx.state_mut().players.get_mut(owner).unwrap().character_def = YUJIN;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    tx.state_mut().revision = 1;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|instance| instance.owner == owner && instance.def.raw() == REZHENJUN)
        .unwrap()
        .id;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        other,
        skill,
    }
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn open_effect(setup: &Setup) -> GameState {
    let mut tx = Transaction::begin(setup.state.clone());
    assert_eq!(
        run_effects(
            &mut tx,
            vec![
                EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw {
                    player: setup.owner,
                    skill: setup.skill,
                },
            ],
        ),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

fn select_target(state: GameState, setup: &Setup, key: u64) -> GameState {
    let (state, result) = resolve(state, setup.owner, vec![setup.target.raw()], false, key);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    state
}

#[test]
fn package_publishes_complete_yujin_and_exact_abi62_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == YUJIN)
        .unwrap();
    assert_eq!(character.name_key, "yujin");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [REZHENJUN]);
    let compiled = content_compiler::compile_source_named("rezhenjun", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 62);
    for expected in compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn prepare_filter_requires_a_hand_card_and_target_choice_includes_self() {
    let mut empty = setup(2_026_074_301);
    let effects = dispatch_for_events(
        &mut Transaction::begin(empty.state.clone()),
        &[DomainEvent::TurnStateChanged {
            current: empty.owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());

    inject_hand(&mut empty.state, empty.owner, 99_301, CardKind::Other);
    let mut tx = Transaction::begin(empty.state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: empty.owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, result) = resolve(state, empty.owner, vec![1], false, 1);
    assert!(result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == empty.owner.raw()));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw
    );
}

#[test]
fn discarding_only_equipment_finishes_without_a_second_branch() {
    let mut setup = setup(2_026_074_302);
    let equipment = inject_hand(&mut setup.state, setup.target, 99_311, CardKind::Weapon);
    setup.state.players.get_mut(setup.target).unwrap().hp = 4;
    let state = select_target(open_effect(&setup), &setup, 10);
    let (state, result) = resolve(state, setup.owner, vec![equipment.raw()], false, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.target), 0);
}

#[test]
fn non_equipment_count_becomes_the_exact_optional_self_discard_cost() {
    let mut setup = setup(2_026_074_303);
    setup.state.players.get_mut(setup.target).unwrap().hp = 1;
    let ordinary = inject_hand(&mut setup.state, setup.target, 99_321, CardKind::Other);
    let equipment = inject_hand(&mut setup.state, setup.target, 99_322, CardKind::Armor);
    inject_hand(&mut setup.state, setup.target, 99_323, CardKind::Other);
    let cost = inject_hand(&mut setup.state, setup.owner, 99_324, CardKind::Other);
    let state = select_target(open_effect(&setup), &setup, 20);
    let (state, result) = resolve(
        state,
        setup.owner,
        vec![ordinary.raw(), equipment.raw()],
        false,
        21,
    );
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert_eq!(state.resolution.continuation_payload[0], 2);
    assert_eq!(state.resolution.continuation_payload[4], 1);
    let target_before = state.hand_count(setup.target);
    let (state, result) = resolve(state, setup.owner, vec![cost.raw()], false, 22);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), target_before);
    assert_eq!(state.hand_count(setup.owner), 0);
}

#[test]
fn cancel_or_insufficient_cost_draws_and_forged_cost_is_atomic() {
    let mut cancel = setup(2_026_074_304);
    cancel.state.players.get_mut(cancel.target).unwrap().hp = 4;
    let target_card = inject_hand(&mut cancel.state, cancel.target, 99_331, CardKind::Other);
    inject_hand(&mut cancel.state, cancel.owner, 99_332, CardKind::Other);
    let state = select_target(open_effect(&cancel), &cancel, 30);
    let (state, _) = resolve(state, cancel.owner, vec![target_card.raw()], false, 31);
    let before = canonical_state_hash(&state);
    let (state, forged) = resolve(state, cancel.owner, vec![9_999_999], false, 32);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    let target_before = state.hand_count(cancel.target);
    let (state, result) = resolve(state, cancel.owner, Vec::new(), true, 33);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(cancel.target), target_before + 1);

    let mut insufficient = setup(2_026_074_305);
    insufficient
        .state
        .players
        .get_mut(insufficient.target)
        .unwrap()
        .hp = 1;
    let first = inject_hand(
        &mut insufficient.state,
        insufficient.target,
        99_341,
        CardKind::Other,
    );
    let second = inject_hand(
        &mut insufficient.state,
        insufficient.target,
        99_342,
        CardKind::Other,
    );
    inject_hand(
        &mut insufficient.state,
        insufficient.target,
        99_343,
        CardKind::Other,
    );
    inject_hand(
        &mut insufficient.state,
        insufficient.owner,
        99_344,
        CardKind::Other,
    );
    let state = select_target(open_effect(&insufficient), &insufficient, 40);
    let before = state.hand_count(insufficient.target);
    let (state, result) = resolve(
        state,
        insufficient.owner,
        vec![first.raw(), second.raw()],
        false,
        41,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(insufficient.target), before);
    assert_ne!(insufficient.other, insufficient.target);
}
}

mod original_yijiang_zhangsong {
//! Golden coverage for 张松「强识」「献图」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHANGSONG: u32 = 2_743_357_358;
const QIANGZHI: u32 = 2_499_367_015;
const XIANTU: u32 = 2_670_783_458;
const QIANGZHI_SOURCE: &str =
    include_str!("../../../content/original/ported/yijiang/qiangzhi.skill");
const XIANTU_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/xiantu.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
}

fn setup(seed: u64, skill_def: u32, current: usize) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[current];
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target: players[1],
        other: players[2],
    }
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn open_event(state: GameState, event: DomainEvent) -> (GameState, Vec<u32>) {
    let mut tx = Transaction::begin(state);
    match dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new()) {
        Ok(effects) => {
            let paused = run_effects(&mut tx, effects).unwrap_err();
            assert_eq!(paused, EffectError::ChoiceRequested);
        }
        Err(error) => assert_eq!(error, EffectError::ChoiceRequested),
    }
    let committed = tx.commit();
    let options = committed
        .output
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    (committed.state, options)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

fn begin_xiantu(mut setup: Setup, key: u64) -> Setup {
    let owner_card_a = inject(&mut setup.state, setup.owner, CardKind::Sha, 99_301);
    let owner_card_b = inject(
        &mut setup.state,
        setup.owner,
        CardKind::TrickWuzhong,
        99_302,
    );
    let (state, confirm) = open_event(
        setup.state,
        DomainEvent::TurnStateChanged {
            current: setup.target,
            phase: TurnPhaseTag::Play,
        },
    );
    assert_eq!(confirm, [0, 1]);
    let (state, confirmed) = resolve(state, setup.owner, vec![1], key);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GiveExactRecordPhaseTarget
    );
    let options: Vec<_> = confirmed
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(options.contains(&owner_card_a.raw()));
    assert!(options.contains(&owner_card_b.raw()));
    let (state, given) = resolve(
        state,
        setup.owner,
        vec![owner_card_a.raw(), owner_card_b.raw()],
        key + 1,
    );
    assert!(matches!(given.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(owner_card_a).unwrap().owner, setup.target);
    assert_eq!(state.cards.get(owner_card_b).unwrap().owner, setup.target);
    assert_eq!(
        state
            .skills
            .get(skill_id(&state, setup.owner, XIANTU))
            .unwrap()
            .state_blob
            .get(..4),
        Some(b"PGT1".as_slice())
    );
    setup.state = state;
    setup
}

#[test]
fn package_publishes_complete_zhangsong_and_exact_abi53_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == ZHANGSONG)
        .unwrap();
    assert_eq!(character.skills, [QIANGZHI, XIANTU]);
    for (name, source) in [("qiangzhi", QIANGZHI_SOURCE), ("xiantu", XIANTU_SOURCE)] {
        let compiled = content_compiler::compile_source_named(name, source).unwrap();
        assert_eq!(compiled.manifest.rule_vm_abi_version, 53);
        for expected in compiled.modules {
            let actual = package
                .modules
                .iter()
                .find(|module| module.name == expected.name)
                .unwrap();
            assert_eq!(actual.logic_hash, expected.logic_hash);
        }
    }
}

#[test]
fn qiangzhi_reveals_and_draws_only_for_the_remembered_play_phase_category() {
    let mut setup = setup(2_026_073_301, QIANGZHI, 0);
    let revealed = inject(
        &mut setup.state,
        setup.target,
        CardKind::TrickWuzhong,
        99_311,
    );
    let used = inject(&mut setup.state, setup.owner, CardKind::TrickWuxie, 99_312);
    let (state, _) = open_event(
        setup.state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: TurnPhaseTag::Play,
        },
    );
    let (state, target_choice) = resolve(state, setup.owner, vec![1], 10);
    assert_eq!(
        target_choice.choice.unwrap().kind,
        engine_api::ChoiceKind::SelectTargets
    );
    let (state, card_choice) = resolve(state, setup.owner, vec![setup.target.raw()], 11);
    assert_eq!(
        card_choice.choice.unwrap().kind,
        engine_api::ChoiceKind::SelectCards
    );
    let (state, revealed_result) = resolve(state, setup.owner, vec![revealed.raw()], 12);
    assert!(matches!(
        revealed_result.result,
        CommandResult::Accepted { .. }
    ));
    let instance = state
        .skills
        .get(skill_id(&state, setup.owner, QIANGZHI))
        .unwrap();
    assert_eq!(instance.state_blob, [b"RCG1".as_slice(), &[2]].concat());

    let before = state.hand_count(setup.owner);
    let (state, _) = open_event(
        state,
        DomainEvent::CardUsed {
            player: setup.owner,
            card: used,
            targets: vec![setup.other],
        },
    );
    let (state, drawn) = resolve(state, setup.owner, vec![1], 13);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), before + 1);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: setup.owner,
            phase: Phase::Discard.to_u8(),
        }],
    )
    .unwrap();
    let paused = run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: setup.owner,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    assert!(tx
        .state()
        .skills
        .get(skill_id(tx.state(), setup.owner, QIANGZHI))
        .unwrap()
        .state_blob
        .is_empty());
}

#[test]
fn xiantu_gift_without_a_kill_loses_one_hp_at_that_play_phase_end() {
    let setup = begin_xiantu(setup(2_026_073_302, XIANTU, 1), 20);
    let hp = setup.state.players.get(setup.owner).unwrap().hp;
    let mut tx = Transaction::begin(setup.state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: setup.target,
            phase: TurnPhaseTag::Play,
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
    let state = tx.commit().state;
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp - 1);
    assert!(state
        .skills
        .get(skill_id(&state, setup.owner, XIANTU))
        .unwrap()
        .state_blob
        .is_empty());
}

#[test]
fn xiantu_target_kill_clears_the_delayed_penalty_and_forged_gift_is_atomic() {
    let mut setup = setup(2_026_073_303, XIANTU, 1);
    inject(&mut setup.state, setup.owner, CardKind::Sha, 99_321);
    inject(&mut setup.state, setup.owner, CardKind::Shan, 99_322);
    let forged = inject(&mut setup.state, setup.other, CardKind::Tao, 99_323);
    let (state, _) = open_event(
        setup.state,
        DomainEvent::TurnStateChanged {
            current: setup.target,
            phase: TurnPhaseTag::Play,
        },
    );
    let (state, confirmed) = resolve(state, setup.owner, vec![1], 30);
    assert!(confirmed.choice.is_some());
    let offered = confirmed.choice.unwrap().options;
    let valid = offered[0].token;
    let before = canonical_state_hash(&state);
    let (state, rejected) = resolve(state, setup.owner, vec![valid, forged.raw()], 31);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    let second = offered
        .iter()
        .map(|option| option.token)
        .find(|token| *token != valid)
        .unwrap();
    let (state, _) = resolve(state, setup.owner, vec![valid, second], 32);
    let hp = state.players.get(setup.owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    let cleared = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: setup.other,
            killer: setup.target,
            hand_equip_cards: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, cleared).unwrap();
    let end = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: setup.target,
            phase: TurnPhaseTag::Play,
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
    assert!(end.is_empty());
    let state = tx.commit().state;
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp);
}
}

mod original_yijiang_zhuhuan {
//! Golden coverage for 朱桓「奋励」「平寇」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHUHUAN: u32 = 3_434_288_625;
const FENLI: u32 = 2_848_697_859;
const PINGKOU: u32 = 3_823_456_872;
const FENLI_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/fenli.skill");
const PINGKOU_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/pingkou.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: [PlayerId; 3],
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
    effects.extend([FENLI, PINGKOU].map(|skill_def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 8;
        participant.max_hp = 8;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Judge;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        others: [players[1], players[2], players[3]],
    }
}

fn inject_equip(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 2,
        zone,
        owner,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn open_event(state: GameState, event: DomainEvent) -> (GameState, PipelineResult) {
    let mut tx = Transaction::begin(state);
    match dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new()) {
        Ok(effects) => assert_eq!(
            run_effects(&mut tx, effects).unwrap_err(),
            EffectError::ChoiceRequested
        ),
        Err(error) => assert_eq!(error, EffectError::ChoiceRequested),
    }
    let committed = tx.commit();
    let new_revision = committed.state.revision;
    let state_hash = committed.state_hash;
    let events = committed.output.events;
    let choice = committed.output.choice;
    (
        committed.state,
        PipelineResult {
            result: CommandResult::Accepted {
                new_revision,
                state_hash,
            },
            events,
            choice,
        },
    )
}

fn assert_optional(result: &PipelineResult) {
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1]
    );
}

#[test]
fn package_publishes_complete_zhuhuan_and_exact_abi55_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == ZHUHUAN)
        .unwrap();
    assert_eq!(character.skills, [FENLI, PINGKOU]);
    for (name, source) in [("fenli", FENLI_SOURCE), ("pingkou", PINGKOU_SOURCE)] {
        let compiled = content_compiler::compile_source_named(name, source).unwrap();
        assert_eq!(compiled.manifest.rule_vm_abi_version, 55);
        for expected in compiled.modules {
            let actual = package
                .modules
                .iter()
                .find(|module| module.name == expected.name)
                .unwrap();
            assert_eq!(actual.logic_hash, expected.logic_hash);
        }
    }
}

#[test]
fn declining_draw_fenli_resumes_the_original_draw_phase() {
    let setup = setup(2_026_073_501);
    let before = setup.state.hand_count(setup.owner);
    let (state, offered) = submit(setup.state, setup.owner, CommandKind::EndPhase, 1);
    assert_optional(&offered);
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(
        state.resolution.resume_kind,
        ContinuationKind::ContinueEnteredDrawPhase
    );

    let (state, declined) = resolve(state, setup.owner, vec![0], 2);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(state.hand_count(setup.owner), before + 2);
    assert_eq!(state.turn.skipped_phase_count(), 0);
}

#[test]
fn draw_fenli_skip_does_not_draw_and_records_the_phase() {
    let mut setup = setup(2_026_073_502);
    setup.state.players.get_mut(setup.others[0]).unwrap().hp = 9;
    setup.state.players.get_mut(setup.others[0]).unwrap().max_hp = 9;
    let before = setup.state.hand_count(setup.owner);
    let (state, offered) = submit(setup.state, setup.owner, CommandKind::EndPhase, 10);
    assert_optional(&offered);
    let (state, accepted) = resolve(state, setup.owner, vec![1], 11);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.hand_count(setup.owner), before);
    assert_eq!(state.turn.skipped_phase_count(), 1);
    assert!(accepted.events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Draw,
        } if *current == setup.owner
    )));
}

#[test]
fn play_and_discard_fenli_feed_ordered_pingkou_with_atomic_target_validation() {
    let mut setup = setup(2_026_073_503);
    setup.state.turn.phase = Phase::Play;
    inject_equip(&mut setup.state, setup.owner, 99_501);
    let (state, play_offer) = open_event(
        setup.state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: TurnPhaseTag::Play,
        },
    );
    assert_optional(&play_offer);

    let (state, discard_offer) = resolve(state, setup.owner, vec![1], 20);
    assert_optional(&discard_offer);
    assert_eq!(state.turn.phase, Phase::Discard);
    assert_eq!(state.turn.skipped_phase_count(), 1);

    let (state, pingkou_offer) = resolve(state, setup.owner, vec![1], 21);
    assert_optional(&pingkou_offer);
    assert_eq!(state.turn.phase, Phase::End);
    assert_eq!(state.turn.skipped_phase_count(), 2);

    let (state, targets_offer) = resolve(state, setup.owner, vec![1], 22);
    let targets = targets_offer.choice.as_ref().unwrap();
    assert_eq!(targets.kind, ChoiceKind::SelectTargets);
    assert_eq!(targets.min_count, 1);
    assert_eq!(targets.max_count, 2);
    assert!(!targets
        .options
        .iter()
        .any(|option| option.token == setup.owner.raw()));
    for target in setup.others {
        assert!(targets
            .options
            .iter()
            .any(|option| option.token == target.raw()));
    }

    let before_hash = canonical_state_hash(&state);
    let (same, forged) = resolve(state, setup.owner, vec![setup.owner.raw()], 23);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before_hash);

    let first = setup.others[1];
    let second = setup.others[0];
    let first_hp = same.players.get(first).unwrap().hp;
    let second_hp = same.players.get(second).unwrap().hp;
    let (state, damaged) = resolve(same, setup.owner, vec![first.raw(), second.raw()], 24);
    assert!(matches!(damaged.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(first).unwrap().hp, first_hp - 1);
    assert_eq!(state.players.get(second).unwrap().hp, second_hp - 1);
    let order: Vec<_> = damaged
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::DamageDealt { source, target, .. } if *source == setup.owner => {
                Some(*target)
            }
            _ => None,
        })
        .collect();
    assert_eq!(order, [first, second]);
}
}

mod original_yijiang_zhuiyi_mobile_re_bulianshi {
//! Golden coverage for YIJIANG「追忆」and MOBILE 界步练师.

use engine_api::{
    ChoiceId, ChoiceKind, ChoiceRequest, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REANXU: u32 = 2_398_589_569;
const ZHUIYI: u32 = 3_427_217_715;
const RE_BULIANSHI: u32 = 3_361_488_418;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/zhuiyi.skill");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    killer: PlayerId,
    recipients: Vec<PlayerId>,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("zhuiyi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 297);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 5, ZHUIYI as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[1];
    let killer = players[0];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHUIYI),
    });
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        killer,
        recipients: players
            .into_iter()
            .filter(|player| *player != owner && *player != killer)
            .collect(),
    }
}

fn kill(state: GameState, owner: PlayerId, killer: PlayerId) -> (GameState, ChoiceRequest) {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: owner,
                killer,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    (committed.state, committed.output.choice.unwrap())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn compiled_contract_and_mobile_cross_package_character_are_closed() {
    let package = content_compiler::compile_source_named("zhuiyi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 297);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == ZHUIYI)
        .unwrap();
    assert_eq!(trigger.event, "death");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let module = package
        .modules
        .iter()
        .find(|module| module.name == trigger.module)
        .unwrap();
    assert_eq!(module.code, [83, 85, 90, 28, 1, 1]);

    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZHUIYI));
    let bulianshi = mobile
        .character_index
        .iter()
        .find(|character| character.id == RE_BULIANSHI)
        .unwrap();
    assert_eq!(bulianshi.name_key, "re_bulianshi");
    assert_eq!(bulianshi.sex, "female");
    assert_eq!(bulianshi.group, "wu");
    assert_eq!(bulianshi.max_hp, 3);
    assert_eq!(bulianshi.skills, [REANXU, ZHUIYI]);
    assert!(mobile
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "yijiang"));
    assert!(mobile
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == REANXU));
    assert!(!mobile
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZHUIYI));
    assert!(bulianshi.skills.iter().all(|wanted| mobile
        .skill_index
        .iter()
        .chain(yijiang.skill_index.iter())
        .any(|skill| skill.skill_def_id == *wanted)));
}

#[test]
fn death_opens_one_cancellable_target_choice_for_owner_and_excludes_killer() {
    let scenario = setup();
    let (state, choice) = kill(scenario.state, scenario.owner, scenario.killer);
    assert!(!state.players.get(scenario.owner).unwrap().alive);
    assert_eq!(choice.observer, scenario.owner);
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    assert!(choice.allow_cancel);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveZhuiyi
    );
    assert_eq!(
        state.resolution.continuation_payload,
        [
            scenario.owner.raw(),
            scenario.killer.raw(),
            state
                .skills
                .sorted_live()
                .into_iter()
                .find(|skill| skill.owner == scenario.owner && skill.def.raw() == ZHUIYI)
                .unwrap()
                .id
                .raw(),
        ]
    );
    let offered: Vec<_> = choice.options.iter().map(|option| option.token).collect();
    assert_eq!(
        offered,
        scenario
            .recipients
            .iter()
            .copied()
            .map(PlayerId::raw)
            .collect::<Vec<_>>()
    );
    assert!(!offered.contains(&scenario.owner.raw()));
    assert!(!offered.contains(&scenario.killer.raw()));

    let before: Vec<_> = scenario
        .recipients
        .iter()
        .map(|player| {
            (
                *player,
                state.players.get(*player).unwrap().hp,
                state.hand_count(*player),
            )
        })
        .collect();
    let (state, cancelled) = resolve(state, scenario.owner, Vec::new(), true, 1);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    for (player, hp, hand) in before {
        assert_eq!(state.players.get(player).unwrap().hp, hp);
        assert_eq!(state.hand_count(player), hand);
    }
    assert!(!state.has_pending_choice());

    let no_source = setup();
    let (state, choice) = kill(no_source.state, no_source.owner, PlayerId::NONE);
    assert_eq!(choice.observer, no_source.owner);
    let offered: Vec<_> = choice.options.iter().map(|option| option.token).collect();
    assert_eq!(offered.len(), 4);
    assert!(!offered.contains(&no_source.owner.raw()));
    assert!(offered.contains(&no_source.killer.raw()));
    let (state, cancelled) = resolve(state, no_source.owner, Vec::new(), true, 2);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
}

#[test]
fn injured_target_recovers_before_drawing_and_full_target_still_draws_three() {
    let mut injured = setup();
    let target = injured.recipients[0];
    injured.state.players.get_mut(target).unwrap().hp = 2;
    let (state, _) = kill(injured.state, injured.owner, injured.killer);
    let (state, pipeline) = resolve(state, injured.owner, vec![target.raw()], false, 3);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert_eq!(state.hand_count(target), 3);
    let recovered = pipeline
        .events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::HpRecovered {
                    source,
                    player,
                    amount: 1,
                } if *source == PlayerId::NONE && *player == target
            )
        })
        .unwrap();
    let drawn = pipeline
        .events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::CardsDrawn { player, cards }
                    if *player == target && cards.len() == 3
            )
        })
        .unwrap();
    assert!(recovered < drawn);

    let full = setup();
    let target = full.recipients[0];
    let max_hp = full.state.players.get(target).unwrap().max_hp;
    assert_eq!(full.state.players.get(target).unwrap().hp, max_hp as i32);
    let (state, _) = kill(full.state, full.owner, full.killer);
    let (state, pipeline) = resolve(state, full.owner, vec![target.raw()], false, 4);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, max_hp as i32);
    assert_eq!(state.hand_count(target), 3);
    assert!(!pipeline.events.iter().any(
        |event| matches!(event, DomainEvent::HpRecovered { player, .. } if *player == target)
    ));
    assert!(pipeline.events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::CardsDrawn { player, cards }
                if *player == target && cards.len() == 3
        )
    }));
}
}
