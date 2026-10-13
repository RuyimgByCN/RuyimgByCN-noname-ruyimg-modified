//! O2 全量合并批次 1（2026-10-14）：家族 onlyol 的 9 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_onlyol_oldangxian {
//! Golden for ONLYOL 廖化「当先」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OL_LIAOHUA: u32 = 2_195_044_091;
const OLDANGXIAN: u32 = 4_090_178_594;
const OLFULI: u32 = 3_137_508_676;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");

struct Setup {
    state: GameState,
    previous: PlayerId,
    owner: PlayerId,
    target: PlayerId,
    draw_sha: CardId,
    discard_sha: CardId,
    other_sha: CardId,
}

fn inject_card(
    state: &mut GameState,
    zone_kind: ZoneKind,
    owner: PlayerId,
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
        face_up: matches!(zone_kind, ZoneKind::DiscardPile),
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
    let previous = tx.state().turn.current;
    let previous_index = seats.iter().position(|player| *player == previous).unwrap();
    let owner = seats[(previous_index + 1) % seats.len()];
    let target = seats[(previous_index + 2) % seats.len()];
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLDANGXIAN),
        },
        EffectRequest::SetTurnState {
            current: previous,
            phase: Phase::End.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let draw_sha = inject_card(
        tx.state_mut(),
        ZoneKind::DrawPile,
        PlayerId::NONE,
        149_001,
        Suit::Spade,
    );
    let discard_sha = inject_card(
        tx.state_mut(),
        ZoneKind::DiscardPile,
        PlayerId::NONE,
        149_002,
        Suit::Heart,
    );
    let other_sha = inject_card(tx.state_mut(), ZoneKind::Hand, owner, 149_003, Suit::Club);
    Setup {
        state: tx.commit().state,
        previous,
        owner,
        target,
        draw_sha,
        discard_sha,
        other_sha,
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

fn end_phase(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
    command(state, actor, CommandKind::EndPhase, key)
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

fn open_gain_choice(setup: Setup, key: u64) -> (Setup, GameState, PipelineResult) {
    let (state, offered) = end_phase(setup.state.clone(), setup.previous, key);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, setup.owner);
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE), 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseGainShaForOpeningExtraPlay
    );
    (setup, state, offered)
}

#[test]
fn package_publishes_complete_ol_liaohua_at_abi293() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 688);
    assert_eq!(package.skill_index.len(), 188);
    assert_eq!(package.character_index.len(), 83);
    assert_eq!(package.trigger_index.len(), 27); // 批162 olrenxin +1
    assert_eq!(package.modifier_index.len(), 169); // 批162 olrenxin noop modifier -1
    assert_eq!(package.modules.len(), 30); // 批162 olrenxin +1
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_LIAOHUA)
        .unwrap();
    assert_eq!(character.name_key, "ol_liaohua");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "shu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [OLDANGXIAN, OLFULI]);
}

#[test]
fn gain_is_optional_and_cancelling_does_not_arm_the_penalty() {
    let setup = setup(2_026_072_501);
    let owner = setup.owner;
    let draw_sha = setup.draw_sha;
    let (setup, state, offered) = open_gain_choice(setup, 1);
    let choice = offered.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (0, 1, true)
    );
    assert_eq!(
        choice
            .options
            .into_iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1]
    );
    let hp = state.players.get(owner).unwrap().hp;
    let (state, result) = resolve(state, owner, Vec::new(), true, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.hand_cards(owner).contains(&draw_sha));
    let (state, result) = end_phase(state, owner, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Prepare);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert!(state
        .zones
        .get(
            state
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap()
        )
        .unwrap()
        .cards
        .contains(&setup.discard_sha));
}

#[test]
fn gained_draw_pile_sha_alone_ignores_distance_and_no_damage_causes_self_damage() {
    let setup = setup(2_026_072_502);
    let owner = setup.owner;
    let draw_sha = setup.draw_sha;
    let discard_sha = setup.discard_sha;
    let other_sha = setup.other_sha;
    let (_, state, _) = open_gain_choice(setup, 44);
    let hp = state.players.get(owner).unwrap().hp;
    let (state, result) = resolve(state, owner, vec![1], false, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).contains(&draw_sha));
    assert!(!state.hand_cards(owner).contains(&discard_sha));
    assert!(engine_core::query::card_distance_unlimited_for_card(
        &state, owner, draw_sha
    ));
    assert!(!engine_core::query::card_distance_unlimited_for_card(
        &state, owner, other_sha
    ));
    let (state, result) = end_phase(state, owner, 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Prepare);
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert!(!engine_core::query::card_distance_unlimited_for_card(
        &state, owner, draw_sha
    ));
}

#[test]
fn damage_dealt_after_accepting_suppresses_the_self_damage() {
    let setup = setup(2_026_072_503);
    let owner = setup.owner;
    let target = setup.target;
    let (_, state, _) = open_gain_choice(setup, 100);
    let (state, _) = resolve(state, owner, vec![1], false, 21);
    let owner_hp = state.players.get(owner).unwrap().hp;
    let target_hp = state.players.get(target).unwrap().hp;
    let mut tx = Transaction::begin(state);
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
    let state = tx.commit().state;
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    let (state, result) = end_phase(state, owner, 119);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
}

#[test]
fn opening_phase_origin_prevents_oldangxian_from_firing_in_another_skills_segment() {
    let setup = setup(2_026_072_504);
    let owner = setup.owner;
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLFULI),
        }],
    )
    .unwrap();
    let other_skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == OLFULI)
        })
        .unwrap();
    run_effects(&mut tx, vec![EffectRequest::BeginTurn { player: owner }]).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::QueueOpeningExtraPlayPhase {
            player: owner,
            skill: other_skill,
        }],
    )
    .unwrap();
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::FinishTurnBegin { player: owner }]
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let state = tx.commit().state;
    let (state, _) = resolve(state, owner, Vec::new(), true, 30);
    let (state, result) = end_phase(state, owner, 31);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.opening_extra_play_active_skill, other_skill);
}
}

mod original_onlyol_olfuli {
//! Golden for ONLYOL 廖化「伏枥」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLFULI: u32 = 3_137_508_676;
const WU: u32 = 4_065_000_621;
const SHU: u32 = 3_601_555_604;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");
const CHARACTER_FIXTURE: &[u8] =
    include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn isolated(damage_dealt: u32) -> (Transaction, PlayerId) {
    let character_fixture =
        content_schema::ContentPackage::from_json_bytes(CHARACTER_FIXTURE).unwrap();
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&character_fixture).unwrap();
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
    let target = players[1];
    let mut tx = Transaction::begin(engine.state().clone());
    for (player, character_def) in players.iter().copied().zip([WU, WU, SHU, SHU]) {
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
    effects.push(EffectRequest::DiscardFromHand {
        player: owner,
        count: u32::MAX,
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLFULI),
    });
    run_effects(&mut tx, effects).unwrap();
    if damage_dealt > 0 {
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target,
                amount: damage_dealt,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .unwrap();
    }
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    (tx, owner)
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

fn has_olfuli(state: &engine_core::state::GameState, owner: PlayerId) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == OLFULI)
}

#[test]
fn committed_package_exposes_optional_limited_dying_contract_at_abi8() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 688);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLFULI)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        ("dying", "owner_is_target", true)
    );
    assert!(package.modules.iter().any(|module| {
        module.code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_QUERY,
                    rule_vm::bytecode::query_tag::LIFETIME_DAMAGE_DEALT,
                ]
        })
    }));
}

#[test]
fn low_lifetime_damage_recovers_refills_turns_over_and_consumes() {
    let (mut tx, owner) = isolated(0);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 1);
    assert_eq!(accepted.players.get(owner).unwrap().hp, 2);
    assert_eq!(accepted.hand_count(owner), 2);
    assert!(accepted.players.get(owner).unwrap().turned_over);
    assert!(!has_olfuli(&accepted, owner));
}

#[test]
fn enough_actual_damage_does_not_turn_over() {
    let (mut tx, owner) = isolated(2);
    assert_eq!(tx.state().damage_dealt_totals.get(&owner), Some(&2));
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 2);
    assert_eq!(accepted.players.get(owner).unwrap().hp, 2);
    assert_eq!(accepted.hand_count(owner), 2);
    assert!(!accepted.players.get(owner).unwrap().turned_over);
    assert!(!has_olfuli(&accepted, owner));
}

#[test]
fn declining_preserves_state_and_limited_skill() {
    let (mut tx, owner) = isolated(0);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let declined = resolve(&tx, owner, false, 3);
    assert_eq!(declined.players.get(owner).unwrap().hp, 0);
    assert_eq!(declined.hand_count(owner), 0);
    assert!(!declined.players.get(owner).unwrap().turned_over);
    assert!(has_olfuli(&declined, owner));
}
}

mod original_onlyol_oljiangchi {
//! Golden for ONLYOL 曹彰「将驰」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OL_CAOZHANG: u32 = 2_831_667_463;
const OLJIANGCHI: u32 = 3_424_528_555;
const OLJIANGCHI_LESS: u32 = 2_381_340_383;
const OLJIANGCHI_MORE: u32 = 4_245_625_871;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::DiscardFromHand {
        player: owner,
        count: u32::MAX,
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLJIANGCHI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
    }
}

fn inject_equip(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Armor,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: true,
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

fn end_phase(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    command(state, owner, CommandKind::EndPhase, key)
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn open_choice(setup: Setup, key: u64) -> (GameState, PipelineResult) {
    let (state, offered) = end_phase(setup.state, setup.owner, key);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrawOrRecastAddSkill
    );
    (state, offered)
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
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn package_publishes_complete_ol_caozhang_at_abi293() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 688);
    assert_eq!(package.skill_index.len(), 188);
    assert_eq!(package.character_index.len(), 83);
    assert_eq!(package.trigger_index.len(), 27); // 批162 olrenxin +1
    assert_eq!(package.modules.len(), 30); // 批162 olrenxin +1
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_CAOZHANG)
        .unwrap();
    assert_eq!(character.name_key, "ol_caozhang");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [OLJIANGCHI]);
    assert_eq!(
        package
            .modifier_index
            .iter()
            .filter(|modifier| {
                modifier.skill_def_id == OLJIANGCHI_LESS || modifier.skill_def_id == OLJIANGCHI_MORE
            })
            .count(),
        4
    );
}

#[test]
fn choice_can_cancel_or_draw_and_the_draw_helper_expires_at_turn_end() {
    let cancelled = setup(2_026_072_601);
    let owner = cancelled.owner;
    let (state, offered) = open_choice(cancelled, 1);
    let choice = offered.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (0, 1, true)
    );
    assert_eq!(
        choice
            .options
            .into_iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1]
    );
    let (state, result) = resolve(state, owner, Vec::new(), true, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!has_skill(&state, owner, OLJIANGCHI_LESS));

    let drawn = setup(2_026_072_602);
    let owner = drawn.owner;
    let hand = drawn.state.hand_count(owner);
    let (state, _) = open_choice(drawn, 3);
    let (state, result) = resolve(state, owner, vec![1], false, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 1);
    assert!(has_skill(&state, owner, OLJIANGCHI_LESS));
    assert_eq!(engine_core::query::sha_use_limit_for(&state, owner), 0);
    assert!(engine_core::query::sha_ignored_for_hand_limit(
        &state, owner
    ));
    let (state, _) = end_phase(state, owner, 5);
    let (state, _) = end_phase(state, owner, 6);
    let (state, _) = end_phase(state, owner, 7);
    assert!(!has_skill(&state, owner, OLJIANGCHI_LESS));
}

#[test]
fn recast_branch_recasts_one_card_and_grants_sha_range_and_limit() {
    let mut setup = setup(2_026_072_603);
    let owner = setup.owner;
    let cost = inject_equip(&mut setup.state, owner, 99_603);
    let (state, offered) = open_choice(setup, 10);
    assert_eq!(
        offered
            .choice
            .unwrap()
            .options
            .into_iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let (state, select) = resolve(state, owner, vec![2], false, 11);
    assert_eq!(select.choice.unwrap().kind, ChoiceKind::SelectCards);
    let (state, result) = resolve(state, owner, vec![cost.raw()], false, 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(cost).unwrap().zone, discard);
    assert_eq!(state.hand_count(owner), 1);
    assert!(has_skill(&state, owner, OLJIANGCHI_MORE));
    assert_eq!(engine_core::query::sha_use_limit_for(&state, owner), 2);
    assert!(engine_core::query::sha_distance_unlimited_for(
        &state, owner
    ));
}
}

mod original_onlyol_oljingce {
//! Golden for ONLYOL 郭淮「精策」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLJINGCE: u32 = 3_351_317_413;
const OL_GUOHUAI: u32 = 2_919_059_025;
const REJINGCE: u32 = 2_158_663_152;
const SOURCE: &str = include_str!("../../../content/original/ported/onlyOL/oljingce.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    current: PlayerId,
    basic: CardId,
    equipment: CardId,
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

fn setup(skill: u32, seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&yijiang).unwrap();
    if skill == OLJINGCE {
        std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    }
    let owner = tx.state().turn.current;
    let current = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    {
        let character = tx.state_mut().players.get_mut(owner).unwrap();
        character.character_def = OL_GUOHUAI;
        character.max_hp = 4;
        character.hp = 4;
    }
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|instance| EffectRequest::RemoveSkill { skill: instance.id })
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
            skill_def: SkillDefId::new(skill),
        },
        EffectRequest::SetTurnState {
            current,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let basic = inject(tx.state_mut(), owner, 293_001, CardKind::Jiu, Suit::Spade);
    let equipment = inject(
        tx.state_mut(),
        owner,
        293_002,
        CardKind::Weapon,
        Suit::Heart,
    );
    Setup {
        state: tx.commit().state,
        owner,
        current,
        basic,
        equipment,
    }
}

fn use_card(state: GameState, player: PlayerId, card: CardId) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player,
            card,
            targets: Vec::new(),
        }],
    )
    .unwrap();
    tx.commit().state
}

fn end_play(state: GameState, current: PlayerId) -> (Transaction, Result<(), EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current,
            phase: Phase::Discard.to_u8(),
        }],
    )
    .map(|_| ());
    (tx, result)
}

#[test]
fn package_publishes_complete_ol_guohuai_at_abi293() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 688);
    assert_eq!(package.skill_index.len(), 188);
    assert_eq!(package.character_index.len(), 83);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_GUOHUAI)
        .unwrap();
    assert_eq!(character.name_key, "ol_guohuai");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [OLJINGCE]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLJINGCE)
        .unwrap();
    assert_eq!(trigger.event, "phase_play_end");
    assert_eq!(
        trigger.filter,
        "owner_alive_play_phase_card_category_count_positive"
    );
    assert!(trigger.optional);
    let modifier = package
        .modifier_index
        .iter()
        .find(|modifier| modifier.skill_def_id == OLJINGCE)
        .unwrap();
    assert_eq!(modifier.target, "hand_limit");
    assert_eq!(
        modifier.value_source.as_deref(),
        Some("owner_distinct_hand_card_suits_used_this_turn")
    );
    let compiled = content_compiler::compile_source_named("oljingce", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == trigger.module)
        .unwrap();
    let expected = compiled
        .modules
        .iter()
        .find(|module| module.name == trigger.module)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn owner_use_during_another_players_play_phase_draws_distinct_categories() {
    let setup = setup(OLJINGCE, 2_026_073_001);
    let state = use_card(setup.state, setup.owner, setup.basic);
    let state = use_card(state, setup.owner, setup.equipment);
    assert_eq!(state.turn.play_card_category_count(setup.owner), 2);
    let hand_before = state.hand_count(setup.owner);

    let (tx, phase_end) = end_play(state, setup.current);
    assert_eq!(phase_end.unwrap_err(), EffectError::ChoiceRequested);
    assert!(tx.state().has_pending_choice());
    let (state, accepted) = apply_command(
        tx.state().clone(),
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(293_003),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), hand_before + 2);
}

#[test]
fn owner_without_card_history_is_not_prompted() {
    let setup = setup(OLJINGCE, 2_026_073_002);
    let (tx, phase_end) = end_play(setup.state, setup.current);
    phase_end.unwrap();
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn rejingce_remains_limited_to_the_owners_own_play_phase_end() {
    let setup = setup(REJINGCE, 2_026_073_003);
    let state = use_card(setup.state, setup.owner, setup.basic);
    assert_eq!(state.turn.play_card_category_count(setup.owner), 0);
    let (tx, phase_end) = end_play(state, setup.current);
    phase_end.unwrap();
    assert!(!tx.state().has_pending_choice());
}
}

mod original_onlyol_oljuece {
//! Golden coverage for OL界李儒「绝策」rewritten from legacy JavaScript.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLJUECE: u32 = 2_978_424_418;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    eligible: PlayerId,
    ineligible: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let eligible = players[1];
    let ineligible = players[2];
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
        EffectRequest::DiscardFromHand {
            player: eligible,
            count: u32::MAX,
        },
        EffectRequest::DiscardFromHand {
            player: ineligible,
            count: u32::MAX,
        },
        EffectRequest::DrawCards {
            player: owner,
            count: 2,
        },
        EffectRequest::DrawCards {
            player: eligible,
            count: 1,
        },
        EffectRequest::DrawCards {
            player: ineligible,
            count: 3,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLJUECE),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    Setup {
        state: tx.commit().state,
        owner,
        eligible,
        ineligible,
    }
}

fn trigger(state: GameState, owner: PlayerId) -> (GameState, engine_api::ChoiceRequest) {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::End,
            }],
            &mut TriggerFireRecord::new(),
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

fn accept_trigger(
    state: GameState,
    owner: PlayerId,
    key: u64,
) -> (GameState, engine_api::ChoiceRequest) {
    let (state, result) = resolve(state, owner, vec![1], key);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    (state, result.choice.unwrap())
}

#[test]
fn package_publishes_optional_hand_lte_trigger_at_abi338() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 688);
    assert_eq!(package.skill_index.len(), 188);
    assert_eq!(package.trigger_index.len(), 27); // 批162 olrenxin +1
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLJUECE)
        .unwrap();
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target_other_hand_lte_owner");
    assert!(trigger.optional);
}

#[test]
fn only_other_players_with_no_more_hand_cards_are_selectable_and_damaged() {
    let setup = setup(2_026_073_102);
    assert_eq!(setup.state.hand_count(setup.owner), 2);
    assert_eq!(setup.state.hand_count(setup.eligible), 1);
    assert_eq!(setup.state.hand_count(setup.ineligible), 3);
    let (state, optional) = trigger(setup.state, setup.owner);
    assert_eq!(optional.kind, ChoiceKind::Confirm);
    let (state, choice) = accept_trigger(state, setup.owner, 1);
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [setup.eligible.raw()]
    );
    let hp_before = state.players.get(setup.eligible).unwrap().hp;
    let (state, result) = resolve(state, setup.owner, vec![setup.eligible.raw()], 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.eligible).unwrap().hp, hp_before - 1);
}

#[test]
fn target_hand_growth_is_revalidated_before_damage() {
    let setup = setup(2_026_073_103);
    let (state, _) = trigger(setup.state, setup.owner);
    let (state, _) = accept_trigger(state, setup.owner, 3);
    let hp_before = state.players.get(setup.eligible).unwrap().hp;
    let mut state = state;
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let hand = state.zones.find(ZoneKind::Hand, setup.eligible).unwrap();
    for _ in 0..2 {
        let card = state.zones.get(draw).unwrap().cards[0];
        assert!(state.zones.remove_card(draw, card));
        assert!(state.zones.push_card(hand, card));
        let card_state = state.cards.get_mut(card).unwrap();
        card_state.zone = hand;
        card_state.owner = setup.eligible;
    }
    assert!(state.hand_count(setup.eligible) > state.hand_count(setup.owner));
    let (state, result) = resolve(state, setup.owner, vec![setup.eligible.raw()], 4);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(state.players.get(setup.eligible).unwrap().hp, hp_before);
    assert!(state.has_pending_choice());
}
}

mod original_onlyol_olqieting {
//! Golden for ONLYOL 蔡夫人「OL窃听」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, ChoiceRequest, Command, CommandKind, CommandResult,
    DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OL_CAIFUREN: u32 = 2_284_410_050;
const OLQIETING: u32 = 4_244_184_524;
const XIANZHOU: u32 = 4_217_889_451;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    ending: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let ending = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::DiscardFromHand {
        player: owner,
        count: u32::MAX,
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLQIETING),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = ending;
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        ending,
    }
}

fn inject_weapon(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 2,
        zone: equip,
        owner,
        face_up: true,
    });
    state.zones.get_mut(equip).unwrap().cards.push(card);
    card
}

fn trigger(
    state: GameState,
    ending: PlayerId,
    card_targeted_other: bool,
    damage_sources: u32,
    current_dealt_damage_other: bool,
) -> (GameState, Option<ChoiceRequest>) {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: ending,
            phase: TurnPhaseTag::End,
            damage_sources,
            damaged_players: 0,
            card_targeted_other,
            current_dealt_damage_other,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let result = run_effects(&mut tx, effects);
    assert!(matches!(result, Ok(_) | Err(EffectError::ChoiceRequested)));
    let committed = tx.commit();
    (committed.state, committed.output.choice)
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
fn package_publishes_complete_ol_caifuren_at_abi293() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 688);
    assert_eq!(package.skill_index.len(), 188);
    assert_eq!(package.character_index.len(), 83);
    assert_eq!(package.trigger_index.len(), 27); // 批162 olrenxin +1
    assert_eq!(package.modules.len(), 30); // 批162 olrenxin +1
    assert!(package.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "yijiang"
            && dependency.version == "0.1.0"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_CAIFUREN)
        .unwrap();
    assert_eq!(character.name_key, "ol_caifuren");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [OLQIETING, XIANZHOU]);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLQIETING)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert_eq!(
        triggers[0].filter,
        "owner_alive_other_turn_ended_no_card_targeted_other"
    );
    assert_eq!(
        triggers[1].filter,
        "owner_alive_other_turn_ended_card_targeted_other_no_damage_other"
    );
}

#[test]
fn no_other_target_allows_move_then_draw_while_damage_history_keeps_single_choice() {
    let mut first = setup(2_026_072_401);
    let weapon = inject_weapon(&mut first.state, first.ending, 99_401);
    let hand = first.state.hand_count(first.owner);
    let (state, choice) = trigger(first.state, first.ending, false, 0, false);
    let choice = choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (1, 2, true)
    );
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    let (state, branch) = resolve(state, first.owner, vec![0, 1], false, 1);
    assert!(matches!(branch.result, CommandResult::Accepted { .. }));
    assert_eq!(
        branch.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let (state, finished) = resolve(state, first.owner, vec![weapon.raw()], false, 2);
    assert!(matches!(finished.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(first.owner), hand + 1);
    let owner_equip = state.zones.find(ZoneKind::Equip, first.owner).unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, owner_equip);

    let mut second = setup(2_026_072_402);
    inject_weapon(&mut second.state, second.ending, 99_402);
    let ending_bit = 1_u32 << (second.ending.raw() - 1);
    let (_, choice) = trigger(second.state, second.ending, true, ending_bit, false);
    let choice = choice.unwrap();
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    let blocked_setup = setup(2_026_072_405);
    let (_, blocked) = trigger(blocked_setup.state, blocked_setup.ending, true, 0, true);
    assert!(blocked.is_none());
}

#[test]
fn both_turn_history_branches_remain_cancellable() {
    for (seed, card_targeted_other) in [(2_026_072_403, false), (2_026_072_404, true)] {
        let setup = setup(seed);
        let hand = setup.state.hand_count(setup.owner);
        let (state, choice) = trigger(setup.state, setup.ending, card_targeted_other, 0, false);
        assert!(choice.unwrap().allow_cancel);
        let (state, cancelled) = resolve(state, setup.owner, Vec::new(), true, seed);
        assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
        assert_eq!(state.hand_count(setup.owner), hand);
        assert!(!state.has_pending_choice());
    }
}
}

mod original_onlyol_olqingxi {
//! Golden for ONLYOL 曹休「倾袭」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OL_CAOXIU: u32 = 3_425_947_453;
const OLQIANJU: u32 = 2_755_577_993;
const OLQINGXI: u32 = 3_468_570_040;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");

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
        skill_def: SkillDefId::new(OLQINGXI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().players.get_mut(owner).unwrap().character_def = OL_CAOXIU;
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

fn deal_sha(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
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
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

fn accept_olqingxi(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let (state, result) = deal_sha(state, owner, target, rule_api::damage_nature::NORMAL);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, accepted) = resolve(state, owner, vec![1], key);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    (state, accepted)
}

#[test]
fn package_publishes_complete_ol_caoxiu_at_abi293() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 688);
    assert_eq!(package.skill_index.len(), 188);
    assert_eq!(package.character_index.len(), 83);
    assert_eq!(package.trigger_index.len(), 27); // 批162 olrenxin +1
    assert_eq!(package.modules.len(), 30); // 批162 olrenxin +1
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_CAOXIU)
        .unwrap();
    assert_eq!(character.name_key, "ol_caoxiu");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [OLQIANJU, OLQINGXI]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLQINGXI)
        .unwrap();
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(trigger.filter, "owner_source_sha_direct_damage_before");
    assert!(trigger.optional);
}

#[test]
fn no_weapon_target_chooses_discard_or_increased_damage() {
    let mut discard = setup(2_026_072_501);
    let card = inject(
        &mut discard.state,
        discard.target,
        ZoneKind::Hand,
        99_501,
        CardKind::Other,
        0,
    );
    let (state, accepted) = accept_olqingxi(discard.state, discard.owner, discard.target, 1);
    let choice = accepted.choice.as_ref().unwrap();
    assert_eq!(choice.observer, discard.target);
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let (state, branch) = resolve(state, discard.target, vec![1], 2);
    assert_eq!(
        branch.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let (state, result) = resolve(state, discard.target, vec![card.raw()], 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(discard.target).unwrap().hp, 9);
    assert_eq!(
        state.cards.get(card).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );

    let mut increase = setup(2_026_072_502);
    inject(
        &mut increase.state,
        increase.target,
        ZoneKind::Hand,
        99_502,
        CardKind::Other,
        0,
    );
    let (state, _) = accept_olqingxi(increase.state, increase.owner, increase.target, 4);
    let (state, result) = resolve(state, increase.target, vec![2], 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(increase.target).unwrap().hp, 8);
    assert_eq!(state.hand_count(increase.target), 1);
}

#[test]
fn weapon_forces_discard_damage_and_weapon_loss_even_when_cards_are_short() {
    let mut exact = setup(2_026_072_503);
    let weapon = inject(
        &mut exact.state,
        exact.owner,
        ZoneKind::Equip,
        99_511,
        CardKind::Weapon,
        2,
    );
    let first = inject(
        &mut exact.state,
        exact.target,
        ZoneKind::Hand,
        99_512,
        CardKind::Other,
        0,
    );
    let second = inject(
        &mut exact.state,
        exact.target,
        ZoneKind::Equip,
        99_513,
        CardKind::Armor,
        0,
    );
    let (state, accepted) = accept_olqingxi(exact.state, exact.owner, exact.target, 10);
    let choice = accepted.choice.as_ref().unwrap();
    assert_eq!(choice.observer, exact.target);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!((choice.min_count, choice.max_count), (2, 2));
    let (state, result) = resolve(state, exact.target, vec![first.raw(), second.raw()], 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(exact.target).unwrap().hp, 8);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in [first, second, weapon] {
        assert_eq!(state.cards.get(card).unwrap().zone, discard);
    }

    let mut short = setup(2_026_072_504);
    let weapon = inject(
        &mut short.state,
        short.owner,
        ZoneKind::Equip,
        99_521,
        CardKind::Weapon,
        3,
    );
    let only = inject(
        &mut short.state,
        short.target,
        ZoneKind::Hand,
        99_522,
        CardKind::Other,
        0,
    );
    let (state, accepted) = accept_olqingxi(short.state, short.owner, short.target, 12);
    assert_eq!(
        (
            accepted.choice.as_ref().unwrap().min_count,
            accepted.choice.as_ref().unwrap().max_count,
        ),
        (1, 1)
    );
    let (state, result) = resolve(state, short.target, vec![only.raw()], 13);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(short.target).unwrap().hp, 8);
    assert_eq!(
        state.cards.get(weapon).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}

#[test]
fn direct_filter_does_not_retrigger_for_chain_propagation() {
    let mut setup = setup(2_026_072_505);
    inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Equip,
        99_531,
        CardKind::Weapon,
        1,
    );
    let card = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        99_532,
        CardKind::Other,
        0,
    );
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
    let (state, result) = deal_sha(
        tx.commit().state,
        setup.owner,
        setup.target,
        rule_api::damage_nature::FIRE,
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, _) = resolve(state, setup.owner, vec![1], 100);
    let (state, result) = resolve(state, setup.target, vec![card.raw()], 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, 8);
    assert_eq!(state.players.get(setup.other).unwrap().hp, 8);
}
}

mod original_onlyol_olrenxin {
//! 第一百六十二批：onlyOL「仁心」（ol_caochong olrenxin）近似重写。
//! 上游：其他角色进入濒死时，若你有装备牌（手牌或装备区），可弃一张装备牌并翻面，
//! 令其回复至 1 点体力。实现：dying + owner_alive_other_dying_has_equip_card（本批
//! 六点注册）→ turn_over(self) + recover_hp(event_player, 1 - hp(event_player))。
//! 近似：弃装备牌代价未建模（optional 确认保留选择权）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const OLRENXIN: u32 = 4_255_762_868;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId) {
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    let dying = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    (state, owner, dying)
}

fn strip_and_arm(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(
            tx.state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|player| EffectRequest::DiscardFromHand {
                    player,
                    count: u32::MAX,
                }),
        )
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLRENXIN),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.commit().state
}

fn inject_equip_hand(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 9,
        weapon_range: 1,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn push_to_dying(state: GameState, dying: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    // hp 直接归零后派发 EnteredDying 事件（olfuli 同款）；ChoiceRequested 属预期
    // 控制流（濒死求桃窗或仁心确认）。
    tx.state_mut().players.get_mut(dying).unwrap().hp = 0;
    let _ = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::EnteredDying {
            player: dying,
            source: PlayerId::NONE,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    );
    tx.commit().state
}

fn cancel_pending(state: GameState, actor: PlayerId, key: u64) -> GameState {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = engine_core::command::apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: Vec::new(),
                cancelled: true,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "cancel rejected: {:?}",
        result.result
    );
    state
}

fn confirm(state: GameState, owner: PlayerId, key: u64) -> GameState {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = engine_core::command::apply_command(
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "confirm rejected: {:?}",
        result.result
    );
    state
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

#[test]
fn package_publishes_olrenxin_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == OLRENXIN
            && trigger.event == "dying"
            && trigger.filter == "owner_alive_other_dying_has_equip_card"
            && trigger.optional
    }));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == OLRENXIN && modifier.target == "hand_limit"));
}

#[test]
fn olrenxin_rescues_dying_other_to_one_hp_at_turnover_cost() {
    let (state, owner, dying) = started(2_026_090_901);
    let state = strip_and_arm(state, owner);

    // owner 手牌含一张装备牌（过滤器门槛），濒死者被打入濒死。
    let mut tx = Transaction::begin(state);
    inject_equip_hand(&mut tx, owner, 98_801);
    let state = tx.commit().state;

    let mut state = push_to_dying(state, dying);
    assert!(hp_of(&state, dying) <= 0, "target must be dying");
    // 逐步跳过濒死求桃窗口，直到仁心确认出现（或窗口耗尽判失败）。
    for key in 2..12 {
        if state.resolution.continuation_kind
            == engine_core::state::ContinuationKind::OptionalTrigger
        {
            break;
        }
        if !state.has_pending_choice() {
            break;
        }
        state = cancel_pending(state, dying, key);
    }
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::OptionalTrigger,
        "other dying must open the renxin confirm"
    );
    let state = confirm(state, owner, 12);

    // 濒死者回复至 1 点体力（近似口径：无弃装备牌代价）。
    assert_eq!(hp_of(&state, dying), 1, "dying player recovers to exactly 1");
}

#[test]
fn olrenxin_skipped_without_any_equip_card() {
    let (state, owner, dying) = started(2_026_090_902);
    let state = strip_and_arm(state, owner);

    // owner 无装备牌（手牌与装备区皆空）：濒死事件不触发仁心。
    let mut state = push_to_dying(state, dying);
    assert!(hp_of(&state, dying) <= 0, "target must be dying");
    for key in 2..12 {
        if state.resolution.continuation_kind
            == engine_core::state::ContinuationKind::OptionalTrigger
        {
            break;
        }
        if !state.has_pending_choice() {
            break;
        }
        state = cancel_pending(state, dying, key);
    }
    assert_ne!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::OptionalTrigger,
        "no equip card in hand or equip zone must gate renxin"
    );
}
}

mod original_onlyol_qixian {
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
}
