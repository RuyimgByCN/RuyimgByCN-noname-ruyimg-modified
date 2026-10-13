//! O2 全量合并批次 1（2026-10-14）：家族 extra 的 10 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_extra_qixing {
//! Golden semantics for EXTRA 神诸葛亮「七星」「狂风」「大雾」。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const QIXING: u32 = 2_771_819_420;
const KUANGFENG: u32 = 4_216_995_242;
const DAWU: u32 = 2_545_944_631;
const KUANGFENG2: u32 = 4_154_289_782;
const DAWU2: u32 = 3_348_547_572;
const SHEN_ZHUGELIANG: u32 = 4_215_109_649;
const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>, [SkillId; 3]) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(QIXING as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: QIXING as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let owner = players[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(QIXING),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(KUANGFENG),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(DAWU),
            },
        ],
    )
    .unwrap();
    let skill = |def| {
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .copied()
            .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
            .unwrap()
    };
    let skills = [skill(QIXING), skill(KUANGFENG), skill(DAWU)];
    (tx.state().clone(), players, skills)
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
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

fn initialize_qixing(state: GameState, owner: PlayerId, skill: SkillId) -> GameState {
    let mut tx = Transaction::begin(state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveQixing {
                player: owner,
                initialize: true,
                stage: 0,
                hand_cards: Vec::new(),
                pile_cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    tx.state().clone()
}

fn helper_skill(state: &GameState, target: PlayerId, def: u32) -> Option<SkillId> {
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
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn package_closes_shen_zhugeliang_on_current_extra_abi() {
    let package = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SHEN_ZHUGELIANG)
        .unwrap();
    assert_eq!(character.skills, [QIXING, KUANGFENG, DAWU]);
    content_runtime::check_engine_compat(
        &package.manifest,
        &content_runtime::tooling_host_versions(),
    )
    .unwrap();
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == DAWU2));
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == KUANGFENG2));
}

#[test]
fn qixing_initializes_seven_public_stars_and_exchanges_equal_counts() {
    let (state, players, skills) = setup();
    let owner = players[0];
    let hand_before = state.hand_cards(owner);
    let state = initialize_qixing(state, owner, skills[0]);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseQixingHandCards
    );
    let stars = state.skill_pile_cards(owner, QIXING);
    assert_eq!(stars.len(), 7);
    assert!(stars
        .iter()
        .all(|card| state.cards.get(*card).unwrap().face_up));

    let (state, pile_choice) = resolve(state, owner, vec![hand_before[0].raw()], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseQixingPileCards
    );
    assert_eq!(pile_choice.choice.unwrap().min_count, 1);
    let (state, result) = resolve(state, owner, vec![stars[0].raw()], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state
        .skill_pile_cards(owner, QIXING)
        .contains(&hand_before[0]));
    assert!(state.hand_cards(owner).contains(&stars[0]));
    assert_eq!(state.skill_pile_cards(owner, QIXING).len(), 7);
}

#[test]
fn kuangfeng_adds_fire_damage_and_dawu_only_allows_thunder_damage() {
    let (state, players, skills) = setup();
    let owner = players[0];
    let target = players[1];
    let state = initialize_qixing(state, owner, skills[0]);
    let (state, _) = resolve(state, owner, Vec::new(), true, 3);
    let stars = state.skill_pile_cards(owner, QIXING);

    let mut tx = Transaction::begin(state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveQixingWeather {
                player: owner,
                qixing_skill_def: SkillDefId::new(QIXING),
                helper_skill_def: SkillDefId::new(KUANGFENG2),
                max_targets: 1,
                stage: 0,
                cards: Vec::new(),
                targets: Vec::new(),
                skill: skills[1],
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let state = tx.state().clone();
    let (state, _) = resolve(state, owner, vec![stars[0].raw()], false, 4);
    let (state, result) = resolve(state, owner, vec![target.raw()], false, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let before = state.players.get(target).unwrap().hp;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::FIRE,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, before - 2);

    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveQixingWeather {
            player: owner,
            qixing_skill_def: SkillDefId::new(QIXING),
            helper_skill_def: SkillDefId::new(DAWU2),
            max_targets: u8::MAX,
            stage: 1,
            cards: vec![stars[1]],
            targets: vec![target],
            skill: skills[2],
        }],
    )
    .unwrap();
    let before = tx.state().players.get(target).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, before);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::THUNDER,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, before - 1);
}

#[test]
fn weather_sources_expire_independently_at_each_sources_next_turn() {
    let (state, players, _skills) = setup();
    let first = players[0];
    let second = players[1];
    let target = players[2];
    let mut tx = Transaction::begin(state);
    for source in [first, second] {
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveQixingWeather {
                player: source,
                qixing_skill_def: SkillDefId::new(QIXING),
                helper_skill_def: SkillDefId::new(KUANGFENG2),
                max_targets: 1,
                stage: 1,
                cards: vec![engine_api::CardId::new(source.raw())],
                targets: vec![target],
                skill: SkillId::NONE,
            }],
        )
        .unwrap();
    }
    let helper = helper_skill(tx.state(), target, KUANGFENG2).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: players[2],
            phase: Phase::End.to_u8(),
        }],
    )
    .unwrap();
    let state = tx.state().clone();
    let (state, result) = submit(state, PlayerId::NONE, CommandKind::EndPhase, 6);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.skills.get(helper).is_some());

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: players[0],
            phase: Phase::End.to_u8(),
        }],
    )
    .unwrap();
    let (state, result) = submit(tx.state().clone(), PlayerId::NONE, CommandKind::EndPhase, 7);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.skills.get(helper).is_none());
}
}

mod original_extra_relonghun {
//! Golden semantics for EXTRA 神赵云「龙魂」。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{CardKind, ContinuationKind, GameState, Phase, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const RELONGHUN: u32 = 2_722_411_221;
const LONGHUN: u32 = 3_422_266_115;
const SHEN_ZHAOYUN: u32 = 3_401_672_399;
const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn cancel_startup_choices(engine: &mut Engine) {
    let mut key = 90_000;
    while engine.state().resolution.pending_choice_id != 0 {
        let state = engine.state();
        let actor = PlayerId::new(state.resolution.pending_observer);
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        assert!(matches!(
            engine.submit(Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
        key += 1;
    }
}

fn setup(owner_is_current: bool) -> (GameState, PlayerId, PlayerId, SkillId) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(RELONGHUN as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: RELONGHUN as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    cancel_startup_choices(&mut engine);
    let players = engine.state().players.alive_by_seat();
    let current = engine.state().turn.current;
    let other = players
        .into_iter()
        .find(|player| *player != current)
        .unwrap();
    let (owner, attacker) = if owner_is_current {
        (current, other)
    } else {
        (other, current)
    };
    let mut tx = Transaction::begin(engine.state().clone());
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

    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: if owner_is_current { owner } else { attacker },
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(RELONGHUN),
            },
        ],
    )
    .unwrap();
    for player in [owner, attacker] {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.max_hp = 8;
        participant.hp = 8;
    }
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == RELONGHUN)
        .unwrap();
    (tx.state().clone(), owner, attacker, skill)
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

fn replace_with_longhun(
    state: GameState,
    owner: PlayerId,
    old_skill: SkillId,
) -> (GameState, SkillId) {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill { skill: old_skill },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(LONGHUN),
            },
        ],
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == LONGHUN)
        .unwrap();
    (tx.state().clone(), skill)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections,
            cancelled: false,
        },
        key,
    )
}

#[test]
fn package_closes_shen_zhaoyun_with_typed_multi_card_rules() {
    let package = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SHEN_ZHAOYUN)
        .unwrap();
    assert_eq!(character.skills, [2_679_563_700, RELONGHUN]);
    let rules: Vec<_> = package
        .view_as_index
        .iter()
        .filter(|rule| rule.skill_def_id == RELONGHUN)
        .collect();
    assert_eq!(rules.len(), 6);
    assert!(rules
        .iter()
        .all(|rule| rule.max_cards == 2 && rule.same_suit));
    assert!(rules.iter().any(|rule| {
        rule.as_kind == "sha"
            && rule.nature.as_deref() == Some("fire")
            && rule.multi_card_bonus.as_deref() == Some("damage_1")
    }));
    let legacy_rules: Vec<_> = package
        .view_as_index
        .iter()
        .filter(|rule| rule.skill_def_id == LONGHUN)
        .collect();
    assert_eq!(legacy_rules.len(), 6);
    assert!(legacy_rules.iter().all(|rule| {
        rule.owner_hp_min_one_cards
            && rule.source_zone.as_deref() == Some("hand_equip_expansion")
            && rule.multi_card_bonus.is_none()
    }));
}

#[test]
fn longhun_requires_current_hp_cards_and_has_no_relonghun_bonus() {
    let (state, owner, target, old_skill) = setup(true);
    let (mut state, skill) = replace_with_longhun(state, owner, old_skill);
    state.players.get_mut(owner).unwrap().hp = 2;
    let cards = state.hand_cards(owner);
    for card in cards.iter().take(2) {
        state.cards.get_mut(*card).unwrap().suit = Suit::Diamond;
    }
    for card in state.hand_cards(target) {
        let card = state.cards.get_mut(card).unwrap();
        card.kind = CardKind::Tao;
        card.suit = Suit::Spade;
    }
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let expansion = state.zones.alloc(Zone {
        id: engine_api::ZoneId::NONE,
        kind: ZoneKind::Expansion,
        owner,
        cards: Vec::new(),
    });
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![cards[1]],
            to: expansion,
            from: Some(hand),
        }],
    )
    .unwrap();
    let state = tx.state().clone();
    let (unchanged, rejected) = submit(
        state.clone(),
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![cards[0]],
        },
        20,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(unchanged, state);

    let hp = state.players.get(target).unwrap().hp;
    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: cards[..2].to_vec(),
        },
        21,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn longhun_response_choice_tracks_current_hp_exactly() {
    let (state, owner, attacker, old_skill) = setup(false);
    let (mut state, _skill) = replace_with_longhun(state, owner, old_skill);
    state.players.get_mut(owner).unwrap().hp = 3;
    let sources = state.hand_cards(owner);
    for source in &sources {
        let card = state.cards.get_mut(*source).unwrap();
        card.kind = CardKind::Tao;
        card.suit = Suit::Club;
    }
    let attack = state.hand_cards(attacker)[0];
    state.cards.get_mut(attack).unwrap().kind = CardKind::Sha;
    let (state, response) = submit(
        state,
        attacker,
        CommandKind::PlayCard {
            card: attack,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        22,
    );
    let source_tokens: std::collections::BTreeSet<_> =
        sources.iter().map(|card| card.raw()).collect();
    let conversion = response
        .choice
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .find(|token| !source_tokens.contains(token))
        .unwrap();
    let (state, source_choice) = resolve(state, owner, vec![conversion], 23);
    let choice = source_choice.choice.unwrap();
    assert_eq!((choice.min_count, choice.max_count), (3, 3));
    let selected = sources[..3]
        .iter()
        .map(|card| card.raw())
        .collect::<Vec<_>>();
    let (state, result) = resolve(state, owner, selected, 24);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
}

#[test]
fn two_diamonds_become_a_two_damage_fire_sha() {
    let (mut state, owner, target, skill) = setup(true);
    let cards = state.hand_cards(owner);
    for card in cards.iter().take(2) {
        state.cards.get_mut(*card).unwrap().suit = Suit::Diamond;
    }
    for card in state.hand_cards(target) {
        let card = state.cards.get_mut(card).unwrap();
        card.kind = CardKind::Tao;
        card.suit = Suit::Spade;
    }
    let before = state.players.get(target).unwrap().hp;
    let (state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: cards.iter().take(2).copied().collect(),
        },
        1,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, before - 2);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(cards
        .iter()
        .take(2)
        .all(|card| state.cards.get(*card).unwrap().zone == discard));
}

#[test]
fn two_hearts_become_a_tao_that_recovers_two() {
    let (mut state, owner, _other, skill) = setup(true);
    let cards = state.hand_cards(owner);
    for card in cards.iter().take(2) {
        state.cards.get_mut(*card).unwrap().suit = Suit::Heart;
    }
    state.players.get_mut(owner).unwrap().hp = 4;
    let (state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: Vec::new(),
            card_targets: cards.iter().take(2).copied().collect(),
        },
        2,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 6);
}

#[test]
fn two_clubs_answer_sha_then_discard_the_current_players_card() {
    let (mut state, owner, attacker, _skill) = setup(false);
    let sources = state.hand_cards(owner);
    for card in &sources {
        let card = state.cards.get_mut(*card).unwrap();
        card.kind = CardKind::Tao;
        card.suit = Suit::Club;
    }
    let attack = state.hand_cards(attacker)[0];
    state.cards.get_mut(attack).unwrap().kind = CardKind::Sha;
    let before = state.players.get(owner).unwrap().hp;
    let (state, response) = submit(
        state,
        attacker,
        CommandKind::PlayCard {
            card: attack,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        3,
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RespondToSha
    );
    let source_tokens: std::collections::BTreeSet<_> =
        sources.iter().map(|card| card.raw()).collect();
    let conversion = response
        .choice
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .find(|token| !source_tokens.contains(token))
        .unwrap();
    let (state, source_choice) = resolve(state, owner, vec![conversion], 4);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::MultiCardViewAsResponse
    );
    assert_eq!(source_choice.choice.unwrap().max_count, 2);
    let selected: Vec<_> = sources.iter().take(2).map(|card| card.raw()).collect();
    let (state, discard_choice) = resolve(state, owner, selected.clone(), 5);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCard
    );
    let choice = discard_choice.choice.unwrap();
    assert_eq!(choice.observer, owner);
    let discarded = choice.options[0].token;
    let (state, result) = resolve(state, owner, vec![discarded], 6);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, before);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(selected.iter().all(|card| {
        state
            .cards
            .get(engine_api::CardId::new(*card))
            .unwrap()
            .zone
            == discard
    }));
    assert_eq!(
        state
            .cards
            .get(engine_api::CardId::new(discarded))
            .unwrap()
            .zone,
        discard
    );
}

#[test]
#[ignore]
fn two_hearts_can_save_a_dying_player_for_two_hp() {
    let (mut state, owner, attacker, _skill) = setup(false);
    // The dying window must offer the 急救 red-card→桃 conversion; grant it
    // explicitly so the test is independent of the booted roster.
    {
        let mut tx = Transaction::begin(state);
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: engine_api::SkillDefId::new(2_961_041_949), // jijiu
            }],
        )
        .unwrap();
        state = tx.commit().state;
    }
    let sources = state.hand_cards(owner);
    for card in &sources {
        let card = state.cards.get_mut(*card).unwrap();
        card.kind = CardKind::Tao;
        card.suit = Suit::Heart;
    }
    state.players.get_mut(owner).unwrap().hp = 1;
    let attack = state.hand_cards(attacker)[0];
    state.cards.get_mut(attack).unwrap().kind = CardKind::Sha;
    let (state, save) = submit(
        state,
        attacker,
        CommandKind::PlayCard {
            card: attack,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        7,
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DyingSave
    );
    let conversion = save
        .choice
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .find(|token| !matches!(*token, 0 | 1))
        .unwrap();
    let (state, _) = resolve(state, owner, vec![conversion], 8);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::MultiCardViewAsResponse
    );
    let selected = sources.iter().take(2).map(|card| card.raw()).collect();
    let (state, result) = resolve(state, owner, selected, 9);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
    assert!(!state.players.get(owner).unwrap().dying);
}
}

mod original_extra_shen_ganning {
//! Golden semantics for EXTRA 神甘宁「魄袭」「劫营」。

use engine_api::{
    CardDefId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    ObserverKind, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::projection::project_observer;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const POXI: u32 = 3_008_932_273;
const POXI_LIMIT: u32 = 2_484_371_675;
const JIEYING: u32 = 2_400_922_959;
const JIEYING_MARK: u32 = 2_172_383_355;
const SHEN_GANNING: u32 = 3_481_630_433;
const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup(player_count: u8) -> (GameState, Vec<PlayerId>) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(POXI as u64),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: POXI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let mut key = 1;
    while engine.state().has_pending_choice() {
        let choice = engine.last_choice().unwrap().clone();
        assert!(matches!(
            engine.submit(Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(engine.state().revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
        key += 1;
    }
    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    (tx.state().clone(), players)
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> SkillId {
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

fn inject(tx: &mut Transaction, owner: PlayerId, suit: Suit, rank: u8) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(99_000 + u32::from(rank)),
        kind: CardKind::Other,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve_choice(state: GameState, actor: PlayerId, cards: Vec<u32>, key: u64) -> GameState {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: cards,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_closes_shen_ganning_with_initial_hp_and_abi() {
    let package = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SHEN_GANNING)
        .unwrap();
    assert_eq!((character.initial_hp, character.max_hp), (3, 6));
    assert_eq!(character.skills, [POXI, JIEYING]);
    content_runtime::check_engine_compat(
        &package.manifest,
        &content_runtime::tooling_host_versions(),
    )
    .unwrap();
    assert!(
        package
            .modifier_index
            .iter()
            .any(|modifier| modifier.skill_def_id == JIEYING
                && modifier.target == "global_draw_count")
    );
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == JIEYING
            && modifier.target == "global_sha_use_limit"));
}

#[test]
fn poxi_reveals_target_hand_and_zero_own_cards_loses_max_hp() {
    let (state, players) = setup(2);
    let owner = players[0];
    let target = players[1];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 6;
    let cards = [
        inject(&mut tx, target, Suit::Spade, 1),
        inject(&mut tx, target, Suit::Heart, 2),
        inject(&mut tx, target, Suit::Club, 3),
        inject(&mut tx, target, Suit::Diamond, 4),
    ];
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(POXI),
        }],
    )
    .unwrap();
    let poxi = skill(tx.state(), owner, POXI);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDrltPoxi {
                player: owner,
                target,
                hand_limit_skill_def: SkillDefId::new(POXI_LIMIT),
                stage: 0,
                own_count: 0,
                skill: poxi,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseDrltPoxiCards
    );
    let snapshot = project_observer(tx.state(), ObserverKind::Player(owner));
    assert!(cards
        .iter()
        .all(|card| snapshot.known_cards.iter().any(|known| known.id == *card)));
    let state = resolve_choice(
        tx.state().clone(),
        owner,
        cards.iter().map(|card| card.raw()).collect(),
        20,
    );
    assert_eq!(state.players.get(owner).unwrap().max_hp, 5);
    assert_eq!(state.hand_count(target), 0);
}

#[test]
fn poxi_post_discard_branches_keep_the_exact_legacy_results() {
    let (state, players) = setup(2);
    let owner = players[0];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDrltPoxi {
            player: owner,
            target: players[1],
            hand_limit_skill_def: SkillDefId::new(POXI_LIMIT),
            stage: 1,
            own_count: 1,
            skill: SkillId::NONE,
        }],
    )
    .unwrap();
    assert_ne!(tx.state().turn.phase, Phase::Play);
    assert_eq!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(tx.state().players.get(owner).unwrap().hp.max(0) - 1)
    );

    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 4;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDrltPoxi {
            player: owner,
            target: players[1],
            hand_limit_skill_def: SkillDefId::new(POXI_LIMIT),
            stage: 1,
            own_count: 3,
            skill: SkillId::NONE,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDrltPoxi {
            player: owner,
            target: players[1],
            hand_limit_skill_def: SkillDefId::new(POXI_LIMIT),
            stage: 1,
            own_count: 4,
            skill: SkillId::NONE,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 4);
}

#[test]
fn jieying_stacks_per_owner_transfers_and_collects_the_marked_hand() {
    let (state, players) = setup(3);
    let owner = players[0];
    let second_owner = players[1];
    let target = players[2];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JIEYING),
            },
            EffectRequest::AddSkill {
                player: second_owner,
                skill_def: SkillDefId::new(JIEYING),
            },
        ],
    )
    .unwrap();
    let jieying = skill(tx.state(), owner, JIEYING);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDrltJieying {
            player: owner,
            event_player: owner,
            mark: engine_api::MarkDefId::new(JIEYING_MARK),
            mode: 0,
            skill: jieying,
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::End;
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDrltJieying {
                player: owner,
                event_player: owner,
                mark: engine_api::MarkDefId::new(JIEYING_MARK),
                mode: 1,
                skill: jieying,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let state = resolve_choice(tx.state().clone(), owner, vec![target.raw()], 30);
    assert_eq!(engine_core::query::draw_count_for(&state, target), 4);
    assert_eq!(engine_core::query::sha_use_limit_for(&state, target), 3);
    assert_eq!(
        query(
            &state,
            &QueryRequest::HandLimit { player: target },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(state.players.get(target).unwrap().hp.max(0) + 2)
    );

    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    inject(&mut tx, target, Suit::Heart, 7);
    inject(&mut tx, target, Suit::Club, 8);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDrltJieying {
            player: owner,
            event_player: target,
            mark: engine_api::MarkDefId::new(JIEYING_MARK),
            mode: 2,
            skill: jieying,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(target), 0);
    assert_eq!(tx.state().hand_count(owner), 2);
    assert_eq!(engine_core::query::draw_count_for(tx.state(), target), 2);
}
}

mod original_extra_shen_liubei {
//! Golden semantics for EXTRA 神刘备「龙怒」「结营」。

use engine_api::{
    CardDefId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const LONGNU: u32 = 2_666_801_006;
const LONGNU_FIRE: u32 = 3_653_578_858;
const LONGNU_THUNDER: u32 = 2_167_979_660;
const JIEYING: u32 = 3_363_786_286;
const SHEN_LIUBEI: u32 = 3_081_060_085;
const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup(player_count: u8) -> (GameState, Vec<PlayerId>) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(LONGNU as u64),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: LONGNU as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let mut key = 1;
    while engine.state().has_pending_choice() {
        let choice = engine.last_choice().unwrap().clone();
        assert!(matches!(
            engine.submit(Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(engine.state().revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
        key += 1;
    }
    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    (tx.state().clone(), players)
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> SkillId {
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

fn inject_trick(tx: &mut Transaction, owner: PlayerId, rank: u8) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind: CardKind::TrickWuzhong,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
    target: PlayerId,
    card: engine_api::CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
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

#[test]
fn package_closes_shen_liubei_with_source_free_contracts() {
    let package = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SHEN_LIUBEI)
        .unwrap();
    assert_eq!(character.skills, [LONGNU, JIEYING]);
    content_runtime::check_engine_compat(
        &package.manifest,
        &content_runtime::tooling_host_versions(),
    )
    .unwrap();
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == LONGNU_FIRE
            && view.filter == "red"
            && view.nature.as_deref() == Some("fire")
            && view.ignore_distance
    }));
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == LONGNU_THUNDER
            && view.filter == "card_trick"
            && view.nature.as_deref() == Some("thunder")
    }));
}

#[test]
fn longnu_starts_on_fire_then_alternates_to_thunder() {
    let (state, players) = setup(2);
    let owner = players[0];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.hp = 6;
    participant.max_hp = 6;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(LONGNU),
        }],
    )
    .unwrap();
    let longnu = skill(tx.state(), owner, LONGNU);
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveNzryLongnu {
            player: owner,
            fire_skill_def: SkillDefId::new(LONGNU_FIRE),
            thunder_skill_def: SkillDefId::new(LONGNU_THUNDER),
            stage: 0,
            skill: longnu,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 5);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 6);
    assert_eq!(tx.state().hand_count(owner), hand + 1);
    let fire = skill(tx.state(), owner, LONGNU_FIRE);

    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill { skill: fire },
            EffectRequest::ResolveNzryLongnu {
                player: owner,
                fire_skill_def: SkillDefId::new(LONGNU_FIRE),
                thunder_skill_def: SkillDefId::new(LONGNU_THUNDER),
                stage: 0,
                skill: longnu,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 5);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 5);
    assert_eq!(tx.state().hand_count(owner), hand + 2);
    assert_eq!(
        tx.state()
            .skills
            .get(skill(tx.state(), owner, LONGNU_THUNDER))
            .unwrap()
            .def
            .raw(),
        LONGNU_THUNDER
    );
}

#[test]
fn thunder_longnu_can_use_multiple_converted_sha() {
    let (state, players) = setup(2);
    let owner = players[0];
    let target = players[1];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(LONGNU_THUNDER),
        }],
    )
    .unwrap();
    let helper = skill(tx.state(), owner, LONGNU_THUNDER);
    let first = inject_trick(&mut tx, owner, 7);
    let second = inject_trick(&mut tx, owner, 8);
    let hp = tx.state().players.get(target).unwrap().hp;

    let (state, first_result) = activate(tx.state().clone(), owner, helper, target, first, 10);
    assert!(matches!(
        first_result.result,
        CommandResult::Accepted { .. }
    ));
    let (state, second_result) = activate(state, owner, helper, target, second, 11);
    assert!(matches!(
        second_result.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
}

#[test]
fn jieying_prevents_unlink_adds_two_once_and_forces_an_unlinked_target() {
    let (state, players) = setup(3);
    let owner = players[0];
    let linked = players[1];
    let target = players[2];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    for player in &players {
        tx.state_mut()
            .players
            .get_mut(*player)
            .unwrap()
            .marks
            .retain(|(mark, _)| *mark != engine_core::content::MARK_CHAIN);
    }
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JIEYING),
            },
            EffectRequest::AddSkill {
                player: target,
                skill_def: SkillDefId::new(JIEYING),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: engine_api::MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
            EffectRequest::AddMark {
                player: linked,
                mark: engine_api::MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
        ],
    )
    .unwrap();
    let base = tx.state().players.get(linked).unwrap().hp.max(0);
    assert_eq!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: linked },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(base + 2)
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::RemoveMark {
            player: owner,
            mark: engine_api::MarkDefId::new(engine_core::content::MARK_CHAIN),
            amount: 0,
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0));

    let jieying = skill(tx.state(), owner, JIEYING);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ChooseLinkOther {
                player: owner,
                skill: jieying,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseLinkOther
    );
    let state = tx.state().clone();
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(12),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![target.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state
        .players
        .get(target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0));
}
}

mod original_extra_shen_luxun {
//! Golden semantics for EXTRA 神陆逊「摧克」「绽火」。

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    MarkDefId, PlayerId, SkillDefId, SkillId,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const CUIKE: u32 = 2_516_554_182;
const DINGHUO: u32 = 3_907_113_145;
const JUNLVE_MARK: u32 = 1_681_567_279;
const CHAIN_MARK: u32 = 2;
const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn cancel_startup_choices(engine: &mut Engine) {
    let mut key = 90_000;
    while engine.state().resolution.pending_choice_id != 0 {
        let state = engine.state();
        let actor = PlayerId::new(state.resolution.pending_observer);
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        assert!(matches!(
            engine.submit(Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
        key += 1;
    }
}

fn skill_state(
    skill_def: u32,
    marks: u32,
    player_count: u8,
) -> (engine_core::state::GameState, PlayerId, SkillId) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    cancel_startup_choices(&mut engine);
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let old: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(skill_def),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(JUNLVE_MARK),
                delta: marks as i32,
            },
        ],
    )
    .unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id;
    (tx.state().clone(), owner, skill)
}

fn choose(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    let revision = state.revision;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    engine_core::command::apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections,
                cancelled: false,
            },
        },
    )
}

fn offer_cuike(
    state: engine_core::state::GameState,
    owner: PlayerId,
    skill: SkillId,
) -> (engine_core::state::GameState, engine_api::ChoiceRequest) {
    let mut tx = Transaction::begin(state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMarkParityControl {
                player: owner,
                mark: MarkDefId::new(JUNLVE_MARK),
                mass_threshold: 7,
                damage_amount: 1,
                stage: 0,
                index: 0,
                target: PlayerId::NONE,
                targets: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let committed = tx.commit();
    (committed.state, committed.output.choice.unwrap())
}

fn mark_count(state: &engine_core::state::GameState, player: PlayerId, mark: u32) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(definition, _)| *definition == mark)
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn cuike_is_offered_at_play_phase_begin() {
    let (state, owner, _) = skill_state(CUIKE, 1, 2);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Draw;
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let committed = tx.commit();
    let choice = committed.output.choice.unwrap();
    assert_eq!(choice.observer, owner);
    assert_eq!(choice.kind, ChoiceKind::Confirm);
}

#[test]
fn cuike_odd_damages_even_links_and_discards_hej() {
    let (state, owner, skill) = skill_state(CUIKE, 1, 2);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let hp = state.players.get(target).unwrap().hp;
    let (state, target_choice) = offer_cuike(state, owner, skill);
    assert_eq!(target_choice.kind, ChoiceKind::SelectTargets);
    let (state, result) = choose(state, owner, vec![target.raw()], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);

    let (mut state, owner, skill) = skill_state(CUIKE, 2, 2);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let card = state.hand_cards(target)[0];
    state.cards.get_mut(card).unwrap().kind = CardKind::Other;
    let (state, _) = offer_cuike(state, owner, skill);
    let (state, result) = choose(state, owner, vec![target.raw()], 2);
    let card_choice = result.choice.unwrap();
    assert_eq!(card_choice.kind, ChoiceKind::SelectCards);
    assert!(card_choice
        .options
        .iter()
        .any(|option| option.token == card.raw()));
    let (state, result) = choose(state, owner, vec![card.raw()], 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(mark_count(&state, target, CHAIN_MARK), 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(card).unwrap().zone, discard);
}

#[test]
fn cuike_above_seven_can_clear_marks_and_damage_all_other_players() {
    let (state, owner, skill) = skill_state(CUIKE, 9, 3);
    let others: Vec<_> = state
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let hp: Vec<_> = others
        .iter()
        .map(|player| state.players.get(*player).unwrap().hp)
        .collect();
    let (state, _) = offer_cuike(state, owner, skill);
    let (state, result) = choose(state, owner, vec![others[0].raw()], 4);
    let confirm = result.choice.unwrap();
    assert_eq!(confirm.kind, ChoiceKind::Custom);
    assert_eq!(
        confirm
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    let (state, result) = choose(state, owner, vec![1], 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(mark_count(&state, owner, JUNLVE_MARK), 0);
    assert_eq!(state.players.get(others[0]).unwrap().hp, hp[0] - 2);
    assert_eq!(state.players.get(others[1]).unwrap().hp, hp[1] - 1);
}

#[test]
fn dinghuo_enforces_linked_mark_capped_targets_and_resolves_in_order() {
    let (mut state, owner, skill) = skill_state(DINGHUO, 2, 3);
    let others: Vec<_> = state
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(CHAIN_MARK),
                delta: 1,
            },
            EffectRequest::AddMark {
                player: others[0],
                mark: MarkDefId::new(CHAIN_MARK),
                delta: 1,
            },
            EffectRequest::AddMark {
                player: others[1],
                mark: MarkDefId::new(CHAIN_MARK),
                delta: 1,
            },
        ],
    )
    .unwrap();
    state = tx.state().clone();

    let revision = state.revision;
    let (_, rejected) = engine_core::command::apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![owner, others[0], others[1]],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RemoveMark {
            player: owner,
            mark: MarkDefId::new(CHAIN_MARK),
            amount: 0,
        }],
    )
    .unwrap();
    state = tx.state().clone();
    let revision = state.revision;
    let (_, rejected) = engine_core::command::apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(11),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![owner],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let mut tx = Transaction::begin(state);
    let mut equipment = Vec::new();
    for target in &others {
        let card = tx.state().hand_cards(*target)[0];
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Weapon;
        let hand = tx.state().zones.find(ZoneKind::Hand, *target).unwrap();
        let equip = tx.state().zones.find(ZoneKind::Equip, *target).unwrap();
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![card],
                to: equip,
                from: Some(hand),
            }],
        )
        .unwrap();
        equipment.push(card);
    }
    state = tx.state().clone();
    let hp = state.players.get(others[1]).unwrap().hp;
    let revision = state.revision;
    let (state, activated) = engine_core::command::apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(12),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: others.clone(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(mark_count(&state, owner, JUNLVE_MARK), 0);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(equipment
        .iter()
        .all(|card| state.cards.get(*card).unwrap().zone == discard));

    let (state, resolved) = choose(state, owner, vec![others[1].raw()], 13);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(state.players.get(others[1]).unwrap().hp < hp);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(JUNLVE_MARK),
                delta: 2,
            },
            EffectRequest::AddMark {
                player: others[0],
                mark: MarkDefId::new(CHAIN_MARK),
                delta: 1,
            },
            EffectRequest::AddMark {
                player: others[1],
                mark: MarkDefId::new(CHAIN_MARK),
                delta: 1,
            },
        ],
    )
    .unwrap();
    let state = tx.state().clone();
    let revision = state.revision;
    let (_, repeated) = engine_core::command::apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(14),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: others,
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}
}

mod original_extra_shen_lvmeng {
//! Golden semantics for EXTRA 神吕蒙「涉猎」「攻心」。

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SHELIE: u32 = 4_211_343_786;
const GONGXIN: u32 = 3_100_453_257;
const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn cancel_startup_choices(engine: &mut Engine) {
    let mut key = 90_000;
    while engine.state().resolution.pending_choice_id != 0 {
        let state = engine.state();
        let actor = PlayerId::new(state.resolution.pending_observer);
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        assert!(matches!(
            engine.submit(Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
        key += 1;
    }
}

fn skill_state(
    skill_def: u32,
    phase: Phase,
) -> (engine_core::state::GameState, PlayerId, PlayerId, SkillId) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    cancel_startup_choices(&mut engine);
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let old: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: phase.to_u8(),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(skill_def),
            },
        ],
    )
    .unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id;
    (tx.state().clone(), owner, target, skill)
}

fn choose(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    let revision = state.revision;
    engine_core::command::apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn shelie_replaces_draw_and_requires_exactly_one_card_per_distinct_suit() {
    let (mut state, owner, _, skill) = skill_state(SHELIE, Phase::Draw);
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top: Vec<_> = {
        let zone = state.zones.get(draw).unwrap();
        zone.cards[zone.cards.len() - 5..].to_vec()
    };
    for (card, suit) in top.iter().zip([
        Suit::Heart,
        Suit::Heart,
        Suit::Diamond,
        Suit::Club,
        Suit::Spade,
    ]) {
        state.cards.get_mut(*card).unwrap().suit = suit;
    }
    let hand_before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ReplaceDrawWithDistinctSuitSelection {
                player: owner,
                count: 5,
                stage: 0,
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(tx.state().turn.flag(turn_flag::DRAW_PHASE_DRAW_ZEROED), 1);
    let committed = tx.commit();
    assert!(committed.output.events.iter().any(|event| {
        matches!(event, DomainEvent::CardsRevealed { cards, to } if cards.len() == 5 && to.is_empty())
    }));
    let choice = committed.output.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.min_count, 4);
    assert_eq!(choice.max_count, 4);

    let (same, rejected) = choose(
        committed.state.clone(),
        owner,
        vec![top[0].raw(), top[1].raw(), top[2].raw(), top[3].raw()],
        1,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(same, committed.state);

    let selected = vec![top[0], top[2], top[3], top[4]];
    let (state, accepted) = choose(
        committed.state,
        owner,
        selected.iter().map(|card| card.raw()).collect(),
        2,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 4);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert!(selected
        .iter()
        .all(|card| state.cards.get(*card).unwrap().zone == hand));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(top[1]).unwrap().zone, discard);
}

fn activate_gongxin(
    state: engine_core::state::GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    let revision = state.revision;
    engine_core::command::apply_command(
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

#[test]
fn gongxin_can_put_one_inspected_heart_on_draw_pile_top() {
    let (mut state, owner, target, skill) = skill_state(GONGXIN, Phase::Play);
    let cards = state.hand_cards(target);
    let heart = cards[0];
    state.cards.get_mut(heart).unwrap().suit = Suit::Heart;
    for card in &cards[1..] {
        state.cards.get_mut(*card).unwrap().suit = Suit::Spade;
    }
    let (state, activated) = activate_gongxin(state, owner, target, skill, 3);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [heart.raw()]
    );
    assert!(activated.events.iter().any(|event| {
        matches!(event, DomainEvent::CardsRevealed { cards, to } if cards.contains(&heart) && to == &[owner])
    }));

    let (state, destination) = choose(state, owner, vec![heart.raw()], 4);
    assert_eq!(destination.choice.unwrap().kind, ChoiceKind::Custom);
    let (state, resolved) = choose(state, owner, vec![1], 5);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(heart).unwrap().zone, draw);
    assert_eq!(
        state.zones.get(draw).unwrap().cards.last().copied(),
        Some(heart)
    );

    let (_, repeated) = activate_gongxin(state, owner, target, skill, 6);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}

#[test]
fn gongxin_can_discard_the_selected_heart_and_no_heart_only_reveals() {
    let (mut state, owner, target, skill) = skill_state(GONGXIN, Phase::Play);
    let heart = state.hand_cards(target)[0];
    state.cards.get_mut(heart).unwrap().suit = Suit::Heart;
    let (state, _) = activate_gongxin(state, owner, target, skill, 7);
    let (state, _) = choose(state, owner, vec![heart.raw()], 8);
    let (state, resolved) = choose(state, owner, vec![0], 9);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(heart).unwrap().zone, discard);

    let (mut state, owner, target, skill) = skill_state(GONGXIN, Phase::Play);
    for card in state.hand_cards(target) {
        state.cards.get_mut(card).unwrap().suit = Suit::Club;
    }
    let hand = state.hand_cards(target);
    let (state, activated) = activate_gongxin(state, owner, target, skill, 10);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert!(activated.choice.is_none());
    assert_eq!(state.hand_cards(target), hand);
}
}

mod original_extra_shen_simayi {
//! Golden semantics for EXTRA 神司马懿「忍戒」「拜印」「极略」「连破」。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    MarkDefId, PlayerId, SkillDefId, SkillId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{
    can_rewrite_judgment, judgment_rewrite_includes_equip, judgment_rewrite_mark_cost, query,
    QueryContext,
};
use engine_core::state::{turn_flag, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const SHEN_SIMAYI: u32 = 4_212_877_947;
const RENJIE: u32 = 3_085_167_840;
const RENJIE2: u32 = 2_678_110_361;
const SBAIYIN: u32 = 2_725_045_502;
const LIANPO: u32 = 2_556_576_493;
const JILUE: u32 = 4_138_291_948;
const JILUE_GUICAI: u32 = 3_991_027_258;
const JILUE_FANGZHU: u32 = 3_045_250_601;
const JILUE_JIZHI: u32 = 2_747_240_883;
const JILUE_ZHIHENG: u32 = 3_079_429_948;
const JILUE_WANSHA: u32 = 3_522_766_701;
const DUORUI: u32 = 4_266_461_274;
const ZHITI: u32 = 3_298_844_877;
const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(SHEN_SIMAYI as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: SHEN_SIMAYI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let mut key = 1;
    while engine.state().has_pending_choice() {
        let choice = engine.last_choice().unwrap().clone();
        assert!(matches!(
            engine.submit(Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(engine.state().revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
        key += 1;
    }
    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let skills = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, skills).unwrap();
    (tx.state().clone(), players)
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> SkillId {
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

fn mark_count(state: &GameState, player: PlayerId, mark: u32) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(definition, _)| *definition == mark)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_closes_shen_simayi_skill_family() {
    let package = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SHEN_SIMAYI)
        .unwrap();
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [RENJIE, SBAIYIN, LIANPO]);
    for definition in [
        RENJIE,
        RENJIE2,
        SBAIYIN,
        LIANPO,
        JILUE,
        JILUE_GUICAI,
        JILUE_FANGZHU,
        JILUE_JIZHI,
        JILUE_ZHIHENG,
        JILUE_WANSHA,
    ] {
        assert!(package
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == definition));
    }
    content_runtime::check_engine_compat(
        &package.manifest,
        &content_runtime::tooling_host_versions(),
    )
    .unwrap();
}

#[test]
fn renjie_discard_and_sbaiyin_awaken_resolve_in_engine() {
    let (state, players) = setup();
    let owner = players[0];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Discard;
    tx.state_mut()
        .turn
        .set_flag(turn_flag::DISCARD_PHASE_DISCARDED, 4);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RENJIE),
        }],
    )
    .unwrap();
    let renjie = skill(tx.state(), owner, RENJIE);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveShenSimayi {
            player: owner,
            event_player: owner,
            event_amount: 0,
            mark: MarkDefId::new(RENJIE),
            mode: 0,
            cards: Vec::new(),
            targets: Vec::new(),
            skill: renjie,
        }],
    )
    .unwrap();
    assert_eq!(mark_count(tx.state(), owner, RENJIE), 4);

    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SBAIYIN),
        }],
    )
    .unwrap();
    assert!(has_skill(tx.state(), owner, SBAIYIN));
    assert_eq!(mark_count(tx.state(), owner, RENJIE), 4);
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(!effects.is_empty());
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp - 1);
    assert!(!has_skill(tx.state(), owner, SBAIYIN));
    for definition in [
        JILUE,
        JILUE_GUICAI,
        JILUE_FANGZHU,
        JILUE_JIZHI,
        JILUE_ZHIHENG,
        JILUE_WANSHA,
    ] {
        assert!(has_skill(tx.state(), owner, definition));
    }
}

#[test]
fn jilue_zhiheng_and_wansha_pay_ren_and_apply_their_turn_state() {
    let (state, players) = setup();
    let owner = players[0];
    let other = players[1];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JILUE_ZHIHENG),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JILUE_WANSHA),
            },
            EffectRequest::AddSkill {
                player: other,
                skill_def: SkillDefId::new(DUORUI),
            },
            EffectRequest::AddSkill {
                player: other,
                skill_def: SkillDefId::new(ZHITI),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(RENJIE),
                delta: 2,
            },
        ],
    )
    .unwrap();
    let hand = tx.state().hand_cards(owner);
    let zhiheng = skill(tx.state(), owner, JILUE_ZHIHENG);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveShenSimayi {
            player: owner,
            event_player: owner,
            event_amount: 0,
            mark: MarkDefId::new(RENJIE),
            mode: 3,
            cards: hand.clone(),
            targets: Vec::new(),
            skill: zhiheng,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_cards(owner).len(), hand.len() + 1);
    assert_eq!(mark_count(tx.state(), owner, RENJIE), 1);

    let wansha = skill(tx.state(), owner, JILUE_WANSHA);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveShenSimayi {
            player: owner,
            event_player: owner,
            event_amount: 0,
            mark: MarkDefId::new(RENJIE),
            mode: 2,
            cards: Vec::new(),
            targets: Vec::new(),
            skill: wansha,
        }],
    )
    .unwrap();
    assert_eq!(mark_count(tx.state(), owner, RENJIE), 0);
    assert_eq!(mark_count(tx.state(), owner, JILUE_WANSHA), 1);
    let duorui = skill(tx.state(), other, DUORUI);
    let zhiti = skill(tx.state(), other, ZHITI);
    assert!(!tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(duorui).unwrap()));
    assert!(tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(zhiti).unwrap()));
    tx.state_mut().players.get_mut(other).unwrap().dying = true;
    assert!(tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(duorui).unwrap()));
}

#[test]
fn jilue_jizhi_discards_the_basic_draw_for_one_turn_hand_limit() {
    let (state, players) = setup();
    let owner = players[0];
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JILUE_JIZHI),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(RENJIE),
                delta: 1,
            },
        ],
    )
    .unwrap();
    let before = query(
        tx.state(),
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    );
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let drawn = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    tx.state_mut().cards.get_mut(drawn).unwrap().kind = CardKind::Sha;
    let jizhi = skill(tx.state(), owner, JILUE_JIZHI);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveShenSimayi {
                player: owner,
                event_player: owner,
                event_amount: 0,
                mark: MarkDefId::new(RENJIE),
                mode: 4,
                cards: Vec::new(),
                targets: Vec::new(),
                skill: jizhi,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseJilueJizhiDiscard
    );
    let revision = tx.state().revision;
    let pending = tx.state().resolution.pending_choice_id;
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(80),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(pending),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.hand_cards(owner).contains(&drawn));
    assert_eq!(mark_count(&state, owner, RENJIE), 0);
    assert_eq!(mark_count(&state, owner, JILUE_JIZHI), 1);
    let after = query(
        &state,
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    );
    let (QueryResponse::Int(before), QueryResponse::Int(after)) = (before, after) else {
        panic!("hand-limit query must return integers");
    };
    assert_eq!(after, before + 1);
}

#[test]
fn jilue_guicai_requires_one_ren_and_accepts_equipment() {
    let (state, players) = setup();
    let owner = players[0];
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JILUE_GUICAI),
        }],
    )
    .unwrap();
    assert!(!can_rewrite_judgment(tx.state(), owner));
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: MarkDefId::new(RENJIE),
            delta: 1,
        }],
    )
    .unwrap();
    assert!(can_rewrite_judgment(tx.state(), owner));
    assert!(judgment_rewrite_includes_equip(tx.state(), owner));
    assert_eq!(judgment_rewrite_mark_cost(tx.state(), owner), Some(RENJIE));
}

#[test]
fn lianpo_extra_turn_returns_to_the_original_turn_anchor() {
    let (state, players) = setup();
    let owner = players[0];
    let current = players[1];
    let normal_next = players[2];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = current;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(LIANPO),
        }],
    )
    .unwrap();
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: current,
                killer: owner,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let pending = tx.state().resolution.pending_choice_id;
    let revision = tx.state().revision;
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(90),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(pending),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, owner);
    assert_eq!(state.turn.phase, Phase::Prepare);

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    run_effects(&mut tx, vec![EffectRequest::AdvanceTurn]).unwrap();
    assert_eq!(tx.state().turn.current, normal_next);
    assert_eq!(tx.state().turn.phase, Phase::Prepare);
}
}

mod original_extra_shen_zhangliao {
//! Golden semantics for EXTRA 神张辽「夺锐」「止啼」。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const DUORUI: u32 = 4_266_461_274;
const DUORUI_HELPER: u32 = 4_193_490_776;
const ZHITI: u32 = 3_298_844_877;
const POXI: u32 = 3_008_932_273;
const SHEN_ZHANGLIAO: u32 = 3_439_287_180;
const SHEN_GANNING: u32 = 3_481_630_433;
const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(DUORUI as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: DUORUI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let mut key = 1;
    while engine.state().has_pending_choice() {
        let choice = engine.last_choice().unwrap().clone();
        assert!(matches!(
            engine.submit(Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(engine.state().revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
        key += 1;
    }
    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    tx.state_mut()
        .players
        .get_mut(players[0])
        .unwrap()
        .character_def = SHEN_ZHANGLIAO;
    tx.state_mut()
        .players
        .get_mut(players[1])
        .unwrap()
        .character_def = SHEN_GANNING;
    (tx.state().clone(), players[0], players[1])
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> SkillId {
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

fn resolve_choice(state: GameState, actor: PlayerId, selection: u32, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_closes_shen_zhangliao_and_marks_helper_non_gainable() {
    let package = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SHEN_ZHANGLIAO)
        .unwrap();
    assert_eq!(
        (character.max_hp, character.skills.as_slice()),
        (4, [DUORUI, ZHITI].as_slice())
    );
    content_runtime::check_engine_compat(
        &package.manifest,
        &content_runtime::tooling_host_versions(),
    )
    .unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == DUORUI_HELPER && skill.non_gainable));
}

#[test]
fn duorui_disables_slot_borrows_skill_and_cleans_up_by_reason() {
    let (state, owner, target) = setup();
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(DUORUI),
            },
            EffectRequest::AddSkill {
                player: target,
                skill_def: SkillDefId::new(POXI),
            },
        ],
    )
    .unwrap();
    let duorui = skill(tx.state(), owner, DUORUI);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDrltDuorui {
                player: owner,
                target,
                helper_skill_def: SkillDefId::new(DUORUI_HELPER),
                stage: 0,
                selected_skill_def: SkillDefId::NONE,
                skill: duorui,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseDrltDuoruiDisableSlot
    );
    let state = resolve_choice(tx.state().clone(), owner, 1, 20);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrltDuoruiSkill
    );
    let state = resolve_choice(state, owner, POXI, 21);
    assert_eq!(state.players.get(owner).unwrap().disabled_equip_slots[0], 1);
    let target_skill = skill(&state, target, POXI);
    assert!(!state.skill_effectively_enabled(state.skills.get(target_skill).unwrap()));
    let borrowed = skill(&state, owner, POXI);
    let helper = skill(&state, target, DUORUI_HELPER);

    let mut tx = Transaction::begin(state);
    tx.state_mut()
        .skills
        .get_mut(target_skill)
        .unwrap()
        .disabled_by_skills
        .push(SkillId::new(999));
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDrltDuorui {
            player: target,
            target,
            helper_skill_def: SkillDefId::new(DUORUI_HELPER),
            stage: 3,
            selected_skill_def: SkillDefId::NONE,
            skill: helper,
        }],
    )
    .unwrap();
    assert!(tx.state().skills.get(borrowed).is_none());
    assert!(!tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(target_skill).unwrap()));
    assert_eq!(
        tx.state()
            .skills
            .get(target_skill)
            .unwrap()
            .disabled_by_skills,
        [SkillId::new(999)]
    );
    assert!(tx
        .state_mut()
        .skills
        .remove_disable_reason(SkillId::new(999)));
    tx.state_mut().rebuild_derived_indexes();
    assert!(tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(target_skill).unwrap()));
    assert!(tx.state().skills.get(helper).is_none());
}

#[test]
fn zhiti_reduces_wounded_in_range_hand_limit_and_restores_a_slot() {
    let (state, owner, target) = setup();
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(target).unwrap().hp = 3;
    tx.state_mut().players.get_mut(target).unwrap().max_hp = 4;
    tx.state_mut()
        .players
        .get_mut(owner)
        .unwrap()
        .disabled_equip_slots[0] = 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZHITI),
        }],
    )
    .unwrap();
    assert_eq!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: target },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(2)
    );
    let zhiti = skill(tx.state(), owner, ZHITI);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDrltZhiti {
                player: owner,
                event_player: owner,
                event_source: target,
                event_amount: 1,
                mode: 2,
                skill: zhiti,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let state = resolve_choice(tx.state().clone(), owner, 1, 30);
    assert_eq!(state.players.get(owner).unwrap().disabled_equip_slots[0], 0);
}
}

mod original_extra_shen_zhouyu {
//! Golden semantics for EXTRA 神周瑜「业炎」「琴音」。

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const YEYAN: u32 = 3_293_607_680;
const QINYIN: u32 = 3_841_403_199;
const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn cancel_startup_choices(engine: &mut Engine) {
    let mut key = 90_000;
    while engine.state().resolution.pending_choice_id != 0 {
        let state = engine.state();
        let actor = PlayerId::new(state.resolution.pending_observer);
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        assert!(matches!(
            engine.submit(Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
        key += 1;
    }
}

fn skill_state(
    skill_def: u32,
    phase: Phase,
) -> (engine_core::state::GameState, Vec<PlayerId>, SkillId) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    cancel_startup_choices(&mut engine);
    let players = engine.state().players.alive_by_seat();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let old: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: phase.to_u8(),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(skill_def),
            },
        ],
    )
    .unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id;
    (tx.state().clone(), players, skill)
}

fn resolve_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    engine_core::command::apply_command(
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

fn activate_yeyan(
    state: engine_core::state::GameState,
    owner: PlayerId,
    skill: SkillId,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    let revision = state.revision;
    engine_core::command::apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    )
}

#[test]
fn small_yeyan_damages_one_to_three_selected_players_with_fire() {
    let (mut state, players, skill) = skill_state(YEYAN, Phase::Play);
    let owner = state.turn.current;
    for card in state.hand_cards(owner) {
        state.cards.get_mut(card).unwrap().suit = Suit::Spade;
    }
    let targets: Vec<_> = players
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let before: Vec<_> = targets
        .iter()
        .map(|target| state.players.get(*target).unwrap().hp)
        .collect();

    let (state, activated) = activate_yeyan(state, owner, skill, 1);
    assert_eq!(activated.choice.unwrap().kind, ChoiceKind::Custom);
    let (state, targets_choice) = resolve_choice(state, owner, vec![0], 2);
    assert_eq!(
        targets_choice.choice.unwrap().kind,
        ChoiceKind::SelectTargets
    );
    let (state, resolved) = resolve_choice(
        state,
        owner,
        targets.iter().map(|target| target.raw()).collect(),
        3,
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    for (target, hp) in targets.iter().zip(before) {
        assert_eq!(state.players.get(*target).unwrap().hp, hp - 1);
    }
    let (_, repeated) = activate_yeyan(state, owner, skill, 4);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}

#[test]
fn big_yeyan_discards_four_suits_loses_three_hp_and_allocates_three_fire_damage() {
    let (mut state, players, skill) = skill_state(YEYAN, Phase::Play);
    let owner = state.turn.current;
    let cards = state.hand_cards(owner);
    assert!(cards.len() >= 4);
    for (card, suit) in
        cards
            .iter()
            .take(4)
            .zip([Suit::Spade, Suit::Heart, Suit::Club, Suit::Diamond])
    {
        state.cards.get_mut(*card).unwrap().suit = suit;
    }
    let owner_state = state.players.get_mut(owner).unwrap();
    owner_state.max_hp = 7;
    owner_state.hp = 7;
    let targets: Vec<_> = players
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let first_hp = state.players.get(targets[0]).unwrap().hp;
    let second_hp = state.players.get(targets[1]).unwrap().hp;

    let (state, mode) = activate_yeyan(state, owner, skill, 5);
    assert_eq!(
        mode.choice
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    let (state, cards_choice) = resolve_choice(state, owner, vec![1], 6);
    assert_eq!(cards_choice.choice.unwrap().kind, ChoiceKind::SelectCards);
    let cost: Vec<_> = cards.iter().take(4).map(|card| card.raw()).collect();
    let (same, rejected) = resolve_choice(
        state.clone(),
        owner,
        vec![cost[0], cost[0], cost[2], cost[3]],
        7,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(same, state);
    let (state, targets_choice) = resolve_choice(state, owner, cost.clone(), 8);
    assert_eq!(
        targets_choice.choice.unwrap().kind,
        ChoiceKind::SelectTargets
    );
    let (state, allocation) = resolve_choice(
        state,
        owner,
        targets.iter().map(|target| target.raw()).collect(),
        9,
    );
    assert_eq!(state.players.get(owner).unwrap().hp, 4);
    assert_eq!(allocation.choice.unwrap().kind, ChoiceKind::SelectTargets);
    let (state, resolved) = resolve_choice(state, owner, vec![targets[0].raw()], 10);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(targets[0]).unwrap().hp, first_hp - 2);
    assert_eq!(state.players.get(targets[1]).unwrap().hp, second_hp - 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(cost.iter().all(|card| state
        .cards
        .get(engine_api::CardId::new(*card))
        .unwrap()
        .zone
        == discard));
}

#[test]
fn single_target_big_yeyan_can_choose_three_fire_damage() {
    let (state, players, skill) = skill_state(YEYAN, Phase::Play);
    let owner = state.turn.current;
    let target = players.into_iter().find(|player| *player != owner).unwrap();
    let hp = state.players.get(target).unwrap().hp;
    let mut tx = Transaction::begin(state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveYeyan {
                player: owner,
                mode: 1,
                stage: 3,
                index: 0,
                targets: vec![target],
                data: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let committed = tx.commit();
    assert_eq!(
        committed
            .output
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [2, 3]
    );
    let (state, resolved) = resolve_choice(committed.state, owner, vec![3], 11);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 3);
}

fn end_discard_after(
    state: engine_core::state::GameState,
    owner: PlayerId,
    count: usize,
) -> (
    engine_core::state::GameState,
    Option<engine_api::ChoiceRequest>,
) {
    let cards: Vec<_> = state.hand_cards(owner).into_iter().take(count).collect();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: owner,
            cards,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().turn.flag(turn_flag::DISCARD_PHASE_DISCARDED),
        count as i32
    );
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::End.to_u8(),
        }],
    );
    if count >= 2 {
        assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    } else {
        result.unwrap();
    }
    let committed = tx.commit();
    (committed.state, committed.output.choice)
}

#[test]
fn qinyin_requires_two_discards_and_supports_recover_or_lose_hp() {
    let (mut state, players, _) = skill_state(QINYIN, Phase::Discard);
    let owner = state.turn.current;
    for player in &players {
        let state = state.players.get_mut(*player).unwrap();
        state.hp -= 1;
    }
    let before: Vec<_> = players
        .iter()
        .map(|player| state.players.get(*player).unwrap().hp)
        .collect();
    let (state, prompt) = end_discard_after(state, owner, 2);
    assert_eq!(prompt.unwrap().kind, ChoiceKind::Custom);
    let (state, recovered) = resolve_choice(state, owner, vec![1], 12);
    assert!(matches!(recovered.result, CommandResult::Accepted { .. }));
    for (player, hp) in players.iter().zip(before) {
        assert_eq!(state.players.get(*player).unwrap().hp, hp + 1);
    }

    let (state, players, _) = skill_state(QINYIN, Phase::Discard);
    let owner = state.turn.current;
    let before: Vec<_> = players
        .iter()
        .map(|player| state.players.get(*player).unwrap().hp)
        .collect();
    let (state, _) = end_discard_after(state, owner, 2);
    let (state, lost) = resolve_choice(state, owner, vec![0], 13);
    assert!(matches!(lost.result, CommandResult::Accepted { .. }));
    for (player, hp) in players.iter().zip(before) {
        assert_eq!(state.players.get(*player).unwrap().hp, hp - 1);
    }

    let (state, _, _) = skill_state(QINYIN, Phase::Discard);
    let owner = state.turn.current;
    let (_, no_prompt) = end_discard_after(state, owner, 1);
    assert!(no_prompt.is_none());
}
}

mod original_extra_wuhun {
//! Golden for EXTRA「武魂」的死亡反噬语义。

use engine_api::{
    CardDefId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, MarkDefId, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::projection::project_player;
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, ContinuationKind, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const WUHUN: u32 = 4_260_309_017;
const XINJUEJING: u32 = 2_679_563_700;
const CHANGANDAJIAN_EQUIP5: u32 = 3_727_522_753;
const JUEJING: u32 = 3_977_263_126;
const WUSHEN: u32 = 2_400_906_129;

fn cancel_startup_choices(engine: &mut Engine) {
    let mut key = 90_000;
    while engine.state().resolution.pending_choice_id != 0 {
        let state = engine.state();
        let actor = PlayerId::new(state.resolution.pending_observer);
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        assert!(matches!(
            engine.submit(Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
        key += 1;
    }
}
const NEW_WUHUN: u32 = 3_278_077_669;
const NEW_GUIXIN: u32 = 3_519_522_126;
const GUIXIN: u32 = 3_519_219_618;
const GUIXIN_SOURCE: &str = include_str!("../../../content/original/ported/extra/guixin.skill");
const GUIXIN_LOGIC_HASH: [u8; 32] = [
    131, 229, 90, 4, 186, 198, 104, 70, 86, 232, 54, 203, 123, 216, 108, 106, 162, 185, 21, 128,
    76, 230, 51, 135, 225, 1, 120, 29, 62, 138, 32, 77,
];
const JILIN: u32 = 3_993_974_385;
const YINGYOU: u32 = 2_877_848_391;
const YINGTIAN: u32 = 4_275_778_563;
const YINGTIAN_EFFECT: u32 = 3_066_580_555;
const YINGTIAN_GUICAI: u32 = 2_431_815_904;
const YINGTIAN_WANSHA: u32 = 2_204_719_707;
const YINGTIAN_LIANPO: u32 = 2_579_407_392;
const DILI: u32 = 3_732_080_996;
const YUHENG: u32 = 3_006_389_674;
const DILI_LINES: [u32; 6] = [
    3_511_235_243,
    3_520_220_242,
    4_057_699_019,
    2_682_708_493,
    3_616_039_425,
    2_521_447_965,
];
const YUHENG_POOL: [u32; 18] = [
    2_790_748_306,
    3_138_095_197,
    2_238_473_770,
    3_285_976_176,
    3_859_813_014,
    3_075_552_814,
    4_211_343_786,
    4_068_014_439,
    2_231_312_450,
    2_455_708_599,
    2_662_349_862,
    2_398_589_569,
    2_873_874_515,
    3_295_441_821,
    2_509_094_453,
    3_040_931_439,
    3_834_102_895,
    3_473_961_755,
];
const CHANGANDAJIAN_EQUIP5_CARD: u32 = 4_174_045_670;
const NZRY_JUNLVE: u32 = 3_829_050_927;
const NZRY_JUNLVE_MARK: u32 = 1_681_567_279;
const NZRY_CUIKE: u32 = 2_516_554_182;
const NZRY_DINGHUO: u32 = 3_907_113_145;
const SHELIE: u32 = 4_211_343_786;
const GONGXIN: u32 = 3_100_453_257;
const YEYAN: u32 = 3_293_607_680;
const QINYIN: u32 = 3_841_403_199;
const RELONGHUN: u32 = 2_722_411_221;
const BAONU: u32 = 3_399_710_515;
const BAONU_MARK: u32 = 1_252_226_867;
const WUMOU: u32 = 4_039_474_555;
const OL_WUQIAN: u32 = 4_021_373_467;
const OL_SHENFEN: u32 = 2_674_950_952;
const WUQIAN: u32 = 2_841_820_068;
const SHENFEN: u32 = 2_409_921_791;
/// Concatenate every committed extra source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn extra_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/extra");
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

const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const STANDARD_PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

#[test]
fn guixin_external_abi_host_continuation_and_module_are_frozen() {
    let compiled =
        content_compiler::compile_source_named("guixin-contract", GUIXIN_SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 237);
    let module = compiled
        .modules
        .iter()
        .find(|module| module.name == "guixin_damage")
        .unwrap();
    assert_eq!(module.code, [83, 80, 231, 1]);
    assert_eq!(module.logic_hash, GUIXIN_LOGIC_HASH);
    assert_eq!(
        rule_vm::bytecode::effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_THEN_TURN_OVER,
        231
    );
    assert_eq!(
        ContinuationKind::ChooseGainCardFromEachOtherThenTurnOver.tag(),
        164
    );
    assert_eq!(
        ContinuationKind::from_tag(164),
        ContinuationKind::ChooseGainCardFromEachOtherThenTurnOver
    );

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let published = committed
        .modules
        .iter()
        .find(|module| module.name == "guixin_damage")
        .unwrap();
    assert_eq!(published.code, module.code);
    assert_eq!(published.logic_hash, module.logic_hash);
}

#[test]
fn committed_package_makes_the_living_killer_enter_dying() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut compiled = content_compiler::compile_source_named("extra", &extra_sources()).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.card_index = committed.card_index.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == WUHUN));
    assert!(committed.character_index.iter().any(|character| {
        character.id == 2_268_373_161
            && character.max_hp == 5
            && character.skills == [WUSHEN, NEW_WUHUN]
    }));
    assert!(committed.character_index.iter().any(|character| {
        character.id == 3_839_224_428
            && character.max_hp == 3
            && character.skills == [NEW_GUIXIN, 2_669_566_023]
    }));
    assert!(committed.character_index.iter().any(|character| {
        character.id == 2_412_883_799
            && character.max_hp == 5
            && character.skills == [BAONU, WUMOU, OL_WUQIAN, OL_SHENFEN]
    }));
    assert!(committed.character_index.iter().any(|character| {
        character.id == 3_606_714_722
            && character.max_hp == 4
            && character.skills == [NZRY_JUNLVE, NZRY_CUIKE, NZRY_DINGHUO]
    }));
    assert!(committed.character_index.iter().any(|character| {
        character.id == 2_338_672_614
            && character.max_hp == 3
            && character.skills == [SHELIE, GONGXIN]
    }));
    assert!(committed.character_index.iter().any(|character| {
        character.id == 2_166_903_732
            && character.max_hp == 4
            && character.skills == [YEYAN, QINYIN]
    }));
    assert!(committed.character_index.iter().any(|character| {
        character.id == 3_401_672_399
            && character.max_hp == 2
            && character.skills == [XINJUEJING, RELONGHUN]
    }));
    assert!(committed.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == NEW_GUIXIN
            && trigger.filter == "owner_is_target_per_damage_point_other_hand_equip_judge_nonempty"
            && trigger.optional
    }));

    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 20_260_713,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let killer = engine.state().turn.current;
    let victim = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != killer)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let players = tx.state().players.alive_by_seat();
    for (player, identity) in players.into_iter().zip([
        Identity::Lord,
        Identity::Loyalist,
        Identity::Rebel,
        Identity::Renegade,
    ]) {
        tx.state_mut().players.get_mut(player).unwrap().identity = identity;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: victim,
            skill_def: SkillDefId::new(WUHUN),
        }],
    )
    .unwrap();
    let saver = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != killer && *player != victim)
        .unwrap();
    let tao = tx.state().hand_cards(saver)[0];
    tx.state_mut().cards.get_mut(tao).unwrap().kind = CardKind::Tao;

    let result = run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer,
        }],
    );
    assert!(
        matches!(result, Err(EffectError::ChoiceRequested)),
        "{result:?}"
    );
    let killer = tx.state().players.get(killer).unwrap();
    assert_eq!(killer.hp, 0);
    assert!(killer.alive && killer.dying);
}

#[test]
fn dili_and_yuheng_use_rule_rng_and_persist_the_fate_pool() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(DILI as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: DILI as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let mut tx = Transaction::begin(engine.state().clone());
    let owner = tx.state().turn.current;
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(DILI),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(YUHENG),
            },
        ],
    )
    .unwrap();
    let skill_by_def = |tx: &Transaction, def: u32| {
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .find(|skill| {
                tx.state()
                    .skills
                    .get(**skill)
                    .is_some_and(|instance| instance.def.raw() == def)
            })
            .copied()
    };
    let dili = skill_by_def(&tx, DILI).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDiliYuheng {
            player: owner,
            action: 0,
            card: engine_api::CardId::NONE,
            event_cards: Vec::new(),
            skill: dili,
        }],
    )
    .unwrap();
    let lines: Vec<_> = DILI_LINES
        .iter()
        .filter_map(|def| skill_by_def(&tx, *def))
        .collect();
    assert_eq!(lines.len(), 1);
    assert!(!tx.state().skills.get(lines[0]).unwrap().enabled);

    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let yuheng = skill_by_def(&tx, YUHENG).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDiliYuheng {
            player: owner,
            action: 1,
            card: engine_api::CardId::NONE,
            event_cards: Vec::new(),
            skill: yuheng,
        }],
    )
    .unwrap();
    assert_eq!(
        YUHENG_POOL
            .iter()
            .filter(|def| skill_by_def(&tx, **def).is_some())
            .count(),
        2
    );
    let hp = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDiliYuheng {
            player: owner,
            action: 2,
            card: engine_api::CardId::NONE,
            event_cards: Vec::new(),
            skill: yuheng,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    tx.state_mut()
        .skills
        .get_mut(yuheng)
        .unwrap()
        .play_phase_use_count = 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDiliYuheng {
            player: owner,
            action: 2,
            card: engine_api::CardId::NONE,
            event_cards: Vec::new(),
            skill: yuheng,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

#[test]
fn package_card_metadata_installs_and_destroys_changandajian() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.card_index.len(), 6);
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(CHANGANDAJIAN_EQUIP5_CARD as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: CHANGANDAJIAN_EQUIP5_CARD as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let mut tx = Transaction::begin(engine.state().clone());
    let owner = tx.state().turn.current;
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    let card = tx.state().hand_cards(owner)[0];
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    {
        let card = tx.state_mut().cards.get_mut(card).unwrap();
        card.def = CardDefId::new(CHANGANDAJIAN_EQUIP5_CARD);
        card.kind = CardKind::Treasure;
    }
    let base_limit = match query(
        tx.state(),
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected hand-limit query: {other:?}"),
    };
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == CHANGANDAJIAN_EQUIP5));
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(value) if value == base_limit + 2
    ));
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: discard,
            from: Some(equip),
        }],
    )
    .unwrap();
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .unwrap();
    assert_eq!(tx.state().cards.get(card).unwrap().zone, removed);
    assert!(!tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == CHANGANDAJIAN_EQUIP5));
}

#[test]
fn tiaojiyanmei_resumes_after_discard_and_can_give_that_card() {
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(38),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 38,
            ruleset: "identity-standard".into(),
        },
    });
    let mut tx = Transaction::begin(engine.state().clone());
    let players = tx.state().players.alive_by_seat();
    let (actor, lower, higher) = (players[0], players[1], players[2]);
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    let lower_hand = tx.state().zones.find(ZoneKind::Hand, lower).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let reduce_lower = tx.state().hand_cards(lower)[..2].to_vec();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: reduce_lower,
            to: discard,
            from: Some(lower_hand),
        }],
    )
    .unwrap();
    assert_eq!(
        (tx.state().hand_count(lower), tx.state().hand_count(higher)),
        (2, 4)
    );
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![
                EffectRequest::DrawCards {
                    player: lower,
                    count: 1,
                },
                EffectRequest::BeginTiaojiyanmeiDiscard {
                    actor,
                    left: lower,
                    right: higher,
                    higher,
                },
            ],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let committed = tx.commit();
    let discard_choice = committed.output.choice.unwrap();
    let discarded = committed.state.hand_cards(higher)[0];
    let (state, output) = engine_core::command::apply_command(
        committed.state,
        Command {
            actor: higher,
            expected_revision: ExpectedRevision(discard_choice.revision),
            idempotency_key: IdempotencyKey(39),
            kind: CommandKind::ResolveChoice {
                choice_id: discard_choice.choice_id,
                selections: vec![discarded.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let gain_choice = output
        .choice
        .expect("equal hands should offer discarded card");
    let (state, output) = engine_core::command::apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(gain_choice.revision),
            idempotency_key: IdempotencyKey(40),
            kind: CommandKind::ResolveChoice {
                choice_id: gain_choice.choice_id,
                selections: vec![lower.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let gained = state.cards.get(discarded).unwrap();
    assert_eq!(gained.owner, lower);
    assert_eq!(
        state.zones.get(gained.zone).map(|zone| zone.kind),
        Some(ZoneKind::Hand)
    );
}

#[test]
fn xinjuejing_draws_on_both_dying_edges_and_adds_two_to_hand_limit() {
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
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINJUEJING),
        }],
    )
    .unwrap();

    let base_limit = tx.state().players.get(owner).unwrap().hp;
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(value) if value == base_limit + 2
    ));
    let hand = tx.state().hand_count(owner);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;

    run_effects(&mut tx, vec![EffectRequest::EnterDying { player: owner }]).unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: owner,
            player: owner,
            amount: 1,
        }],
    )
    .unwrap();
    assert!(!tx.state().players.get(owner).unwrap().dying);
    assert_eq!(tx.state().hand_count(owner), hand + 2);
}

#[test]
fn juejing_adds_lost_hp_to_draw_count_and_two_to_hand_limit() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_817,
            ruleset: "identity-standard".into(),
        },
    });
    cancel_startup_choices(&mut engine);
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JUEJING),
        }],
    )
    .unwrap();

    let hp = tx.state().players.get(owner).unwrap().hp;
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(value) if value == hp + 2
    ));
    assert_eq!(engine_core::query::draw_count_for(tx.state(), owner), 2);

    tx.state_mut().players.get_mut(owner).unwrap().hp -= 2;
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(value) if value == hp
    ));
    assert_eq!(engine_core::query::draw_count_for(tx.state(), owner), 4);
}

#[test]
fn changandajian_equip5_adds_two_to_hand_limit() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_715,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(CHANGANDAJIAN_EQUIP5),
        }],
    )
    .unwrap();

    let base_limit = tx.state().players.get(owner).unwrap().hp;
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(value) if value == base_limit + 2
    ));
}

#[test]
fn wushen_turns_hearts_into_unlimited_unanswerable_normal_sha() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_715_1,
            ruleset: "identity-standard".into(),
        },
    });
    cancel_startup_choices(&mut engine);
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    for skill in tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.id)
        .collect::<Vec<_>>()
    {
        run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(WUSHEN),
        }],
    )
    .unwrap();

    let cards = tx.state().hand_cards(owner);
    let heart = cards[0];
    let ordinary = cards[1];
    {
        let card = tx.state_mut().cards.get_mut(heart).unwrap();
        card.kind = CardKind::Tao;
        card.suit = Suit::Heart;
    }
    {
        let card = tx.state_mut().cards.get_mut(ordinary).unwrap();
        card.kind = CardKind::Sha;
        card.suit = Suit::Spade;
    }
    let shan = tx.state().hand_cards(target)[0];
    tx.state_mut().cards.get_mut(shan).unwrap().kind = CardKind::Shan;
    tx.state_mut().turn.phase = Phase::Play;

    let physical = tx.state().cards.get(heart).unwrap();
    assert_eq!(physical.kind, CardKind::Tao);
    assert_eq!(
        engine_core::query::effective_card_kind(tx.state(), owner, physical),
        CardKind::Sha
    );
    assert!(engine_core::query::card_use_unlimited_for_card(
        tx.state(),
        owner,
        heart
    ));
    assert!(engine_core::query::card_distance_unlimited_for_card(
        tx.state(),
        owner,
        heart
    ));
    assert!(engine_core::query::sha_unrespondable_for_card(
        tx.state(),
        owner,
        heart
    ));
    assert!(!engine_core::query::sha_unrespondable_for_card(
        tx.state(),
        owner,
        ordinary
    ));

    let hp = tx.state().players.get(target).unwrap().hp;
    let before = tx.state().clone();
    let (after, result) = engine_core::command::apply_command(
        before.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(before.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: heart,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(!after.has_pending_choice());
    assert_eq!(after.players.get(target).unwrap().hp, hp - 1);
    assert!(after.hand_cards(target).contains(&shan));
    assert_eq!(after.cards.get(heart).unwrap().kind, CardKind::Tao);
}

fn new_wuhun_death_result(judgment_kind: CardKind) -> bool {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(judgment_kind.tag() as u64),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 20_260_726,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package).unwrap();
    let players = engine.state().players.alive_by_seat();
    let (owner, source, tied) = (players[1], players[2], players[3]);
    let mut tx = Transaction::begin(engine.state().clone());
    for (player, identity) in players.into_iter().zip([
        Identity::Lord,
        Identity::Loyalist,
        Identity::Rebel,
        Identity::Renegade,
    ]) {
        tx.state_mut().players.get_mut(player).unwrap().identity = identity;
    }
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
            skill_def: SkillDefId::new(NEW_WUHUN),
        }],
    )
    .unwrap();

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let source_marks = tx
        .state()
        .players
        .get(source)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == NEW_WUHUN)
        .map_or(0, |(_, count)| *count);
    assert_eq!(source_marks, 2);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: tied,
            mark: MarkDefId::new(NEW_WUHUN),
            delta: 2,
        }],
    )
    .unwrap();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    tx.state_mut().cards.get_mut(top).unwrap().kind = judgment_kind;

    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: owner,
                killer: PlayerId::NONE,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan
    );
    let committed = tx.commit();
    let choice = committed.output.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [source.raw(), tied.raw()]
    );
    let (state, result) = engine_core::command::apply_command(
        committed.state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(committed.state.revision),
            idempotency_key: IdempotencyKey(NEW_WUHUN as u64 + judgment_kind.tag() as u64),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![source.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state.players.get(source).unwrap().alive
}

#[test]
fn new_wuhun_marks_damage_source_and_only_tao_or_taoyuan_prevents_death() {
    assert!(new_wuhun_death_result(CardKind::Tao));
    assert!(new_wuhun_death_result(CardKind::TrickTaoyuan));
    assert!(!new_wuhun_death_result(CardKind::Other));
}

#[test]
fn new_guixin_obeys_zone_priority_then_turns_over() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(NEW_GUIXIN as u64),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: NEW_GUIXIN as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package).unwrap();
    let players = engine.state().players.alive_by_seat();
    let (owner, equip_target, judge_target) = (players[1], players[2], players[0]);
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
            skill_def: SkillDefId::new(NEW_GUIXIN),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == NEW_GUIXIN)
        .unwrap()
        .id;

    let equip_card = tx.state().hand_cards(equip_target)[0];
    let equip_zone = tx
        .state()
        .zones
        .find(ZoneKind::Equip, equip_target)
        .unwrap();
    let equip_hand = tx.state().zones.find(ZoneKind::Hand, equip_target).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![equip_card],
            to: equip_zone,
            from: Some(equip_hand),
        }],
    )
    .unwrap();

    let judge_cards = tx.state().hand_cards(judge_target);
    let judge_card = judge_cards[0];
    let judge_zone = tx
        .state()
        .zones
        .find(ZoneKind::Judge, judge_target)
        .unwrap();
    let judge_hand = tx.state().zones.find(ZoneKind::Hand, judge_target).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![judge_card],
            to: judge_zone,
            from: Some(judge_hand),
        }],
    )
    .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: judge_cards[1..].to_vec(),
            to: discard,
            from: Some(judge_hand),
        }],
    )
    .unwrap();

    let hand_before = tx.state().hand_count(owner);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![
                EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
                    player: owner,
                    priority: u8::MAX,
                    remaining_targets: Vec::new(),
                    skill,
                },
            ],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver
    );
    let committed = tx.commit();
    let choice = committed.output.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    let (state, result) = engine_core::command::apply_command(
        committed.state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(committed.state.revision),
            idempotency_key: IdempotencyKey(NEW_GUIXIN as u64 + 1),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(!state.has_pending_choice());
    assert!(state.players.get(owner).unwrap().turned_over);
    assert_eq!(state.hand_count(owner), hand_before + 3);
    let owner_hand = state.hand_cards(owner);
    assert!(owner_hand.contains(&equip_card));
    assert!(owner_hand.contains(&judge_card));
}

#[test]
fn guixin_chooses_one_hej_card_from_each_other_then_turns_over() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(GUIXIN as u64),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: GUIXIN as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package).unwrap();
    let players = engine.state().players.alive_by_seat();
    let (owner, equip_target, judge_target) = (players[0], players[1], players[2]);
    let mut tx = Transaction::begin(engine.state().clone());
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(GUIXIN),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == GUIXIN)
        .unwrap()
        .id;
    let equip_card = tx.state().hand_cards(equip_target)[0];
    let equip_hand = tx.state().zones.find(ZoneKind::Hand, equip_target).unwrap();
    let equip_zone = tx
        .state()
        .zones
        .find(ZoneKind::Equip, equip_target)
        .unwrap();
    let judge_card = tx.state().hand_cards(judge_target)[0];
    let judge_hand = tx.state().zones.find(ZoneKind::Hand, judge_target).unwrap();
    let judge_zone = tx
        .state()
        .zones
        .find(ZoneKind::Judge, judge_target)
        .unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![equip_card],
                to: equip_zone,
                from: Some(equip_hand),
            },
            EffectRequest::MoveCards {
                cards: vec![judge_card],
                to: judge_zone,
                from: Some(judge_hand),
            },
        ],
    )
    .unwrap();
    let hand_before = tx.state().hand_count(owner);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
                player: owner,
                initialized: false,
                remaining_targets: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseGainCardFromEachOtherThenTurnOver
    );
    let payload = &tx.state().resolution.continuation_payload;
    assert_eq!(
        &payload[..3],
        [owner.raw(), equip_target.raw(), skill.raw()]
    );
    assert_eq!(payload.len(), 4 + payload[3] as usize);
    assert!(payload[4..].contains(&equip_card.raw()));
    let committed = tx.commit();
    let first = committed.output.choice.unwrap();
    assert!(first
        .options
        .iter()
        .any(|option| option.token == equip_card.raw()));
    let (state, second_result) = engine_core::command::apply_command(
        committed.state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(committed.state.revision),
            idempotency_key: IdempotencyKey(GUIXIN as u64 + 1),
            kind: CommandKind::ResolveChoice {
                choice_id: first.choice_id,
                selections: vec![equip_card.raw()],
                cancelled: false,
            },
        },
    );
    let second = second_result.choice.unwrap();
    assert!(second
        .options
        .iter()
        .any(|option| option.token == judge_card.raw()));
    let (state, result) = engine_core::command::apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(GUIXIN as u64 + 2),
            kind: CommandKind::ResolveChoice {
                choice_id: second.choice_id,
                selections: vec![judge_card.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(state.players.get(owner).unwrap().turned_over);
    assert_eq!(state.hand_count(owner), hand_before + 2);
    let hand = state.hand_cards(owner);
    assert!(hand.contains(&equip_card));
    assert!(hand.contains(&judge_card));
}

#[test]
fn jilin_hidden_pile_reveals_exchanges_and_drives_yingyou_draw() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(JILIN as u64),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: JILIN as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package).unwrap();
    let players = engine.state().players.alive_by_seat();
    let (owner, observer) = (players[0], players[1]);
    let mut tx = Transaction::begin(engine.state().clone());
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JILIN),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(YINGYOU),
            },
        ],
    )
    .unwrap();
    let skill = |tx: &Transaction, def| {
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .copied()
            .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
            .unwrap()
    };
    let jilin = skill(&tx, JILIN);
    let yingyou = skill(&tx, YINGYOU);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveHiddenSkillPile {
            player: owner,
            pile_skill_def: SkillDefId::new(JILIN),
            action: 0,
            card: engine_api::CardId::NONE,
            stage: 0,
            hand_cards: Vec::new(),
            pile_cards: Vec::new(),
            event_cards: Vec::new(),
            skill: jilin,
        }],
    )
    .unwrap();
    let hidden = tx.state().hidden_skill_pile_cards(owner, JILIN);
    assert_eq!(hidden.len(), 2);
    let owner_view = project_player(tx.state(), owner);
    let other_view = project_player(tx.state(), observer);
    for card in &hidden {
        assert!(owner_view
            .known_cards
            .iter()
            .any(|view| view.id == *card && view.def.is_some()));
        assert!(other_view
            .known_cards
            .iter()
            .any(|view| view.id == *card && view.def.is_none()));
    }

    let mut cancel_tx = Transaction::begin(tx.state().clone());
    let attacker = observer;
    let sha = cancel_tx.state().hand_cards(attacker)[0];
    cancel_tx.state_mut().resolution.resume_kind = ContinuationKind::ResumePlayedSha;
    cancel_tx.state_mut().resolution.resume_payload =
        vec![attacker.raw(), owner.raw(), 1, 0, 1, sha.raw()];
    let hp_before_cancel = cancel_tx.state().players.get(owner).unwrap().hp;
    assert!(matches!(
        run_effects(
            &mut cancel_tx,
            vec![EffectRequest::ResolveHiddenSkillPile {
                player: owner,
                pile_skill_def: SkillDefId::new(JILIN),
                action: 1,
                card: sha,
                stage: 0,
                hand_cards: Vec::new(),
                pile_cards: Vec::new(),
                event_cards: Vec::new(),
                skill: jilin,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let committed = cancel_tx.commit();
    let choice = committed.output.choice.unwrap();
    let (cancelled_state, cancel_result) = engine_core::command::apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(choice.revision),
            idempotency_key: IdempotencyKey(JILIN as u64 + 10),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![hidden[0].raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(
        cancel_result.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        cancelled_state.players.get(owner).unwrap().hp,
        hp_before_cancel
    );
    assert!(cancelled_state.cards.get(hidden[0]).unwrap().face_up);

    let hand_before_reveal = tx.state().hand_count(owner);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveHiddenSkillPile {
                player: owner,
                pile_skill_def: SkillDefId::new(JILIN),
                action: 3,
                card: engine_api::CardId::NONE,
                stage: 0,
                hand_cards: Vec::new(),
                pile_cards: Vec::new(),
                event_cards: Vec::new(),
                skill: yingyou,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let committed = tx.commit();
    let choice = committed.output.choice.unwrap();
    let (state, reveal_result) = engine_core::command::apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(choice.revision),
            idempotency_key: IdempotencyKey(JILIN as u64 + 1),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![hidden[0].raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(
        reveal_result.result,
        CommandResult::Accepted { .. }
    ));
    assert!(state.cards.get(hidden[0]).unwrap().face_up);
    assert_eq!(state.hand_count(owner), hand_before_reveal + 1);

    let mut tx = Transaction::begin(state);
    let outgoing = tx.state().hand_cards(owner)[0];
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveHiddenSkillPile {
                player: owner,
                pile_skill_def: SkillDefId::new(JILIN),
                action: 2,
                card: engine_api::CardId::NONE,
                stage: 0,
                hand_cards: Vec::new(),
                pile_cards: Vec::new(),
                event_cards: Vec::new(),
                skill: jilin,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let committed = tx.commit();
    let hand_choice = committed.output.choice.unwrap();
    let (state, pile_result) = engine_core::command::apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(hand_choice.revision),
            idempotency_key: IdempotencyKey(JILIN as u64 + 2),
            kind: CommandKind::ResolveChoice {
                choice_id: hand_choice.choice_id,
                selections: vec![outgoing.raw()],
                cancelled: false,
            },
        },
    );
    let pile_choice = pile_result.choice.unwrap();
    let (state, exchange_result) = engine_core::command::apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(pile_choice.revision),
            idempotency_key: IdempotencyKey(JILIN as u64 + 3),
            kind: CommandKind::ResolveChoice {
                choice_id: pile_choice.choice_id,
                selections: vec![hidden[1].raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(
        exchange_result.result,
        CommandResult::Accepted { .. }
    ));
    assert!(state.hand_cards(owner).contains(&hidden[1]));
    assert!(state
        .hidden_skill_pile_cards(owner, JILIN)
        .contains(&outgoing));

    let mut tx = Transaction::begin(state);
    let matching = tx.state().hand_cards(owner)[0];
    let revealed_suit = tx.state().cards.get(hidden[0]).unwrap().suit;
    tx.state_mut().cards.get_mut(matching).unwrap().suit = revealed_suit;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand_before_loss = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![matching],
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand_before_loss);
}

#[test]
fn yingtian_helpers_apply_distance_and_only_suppress_eligible_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for skill_def in [
        YINGTIAN,
        YINGTIAN_EFFECT,
        YINGTIAN_GUICAI,
        YINGTIAN_WANSHA,
        YINGTIAN_LIANPO,
    ] {
        assert!(
            package
                .skill_index
                .iter()
                .any(|skill| skill.skill_def_id == skill_def),
            "missing skill {skill_def}"
        );
    }

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(YINGTIAN as u64),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: YINGTIAN as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let mut tx = Transaction::begin(engine.state().clone());
    let owner = tx.state().turn.current;
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(YINGTIAN_WANSHA),
            },
            EffectRequest::AddSkill {
                player: others[0],
                skill_def: SkillDefId::new(YINGTIAN_EFFECT),
            },
            EffectRequest::AddSkill {
                player: others[0],
                skill_def: SkillDefId::new(YINGTIAN_GUICAI),
            },
            EffectRequest::AddSkill {
                player: others[1],
                skill_def: SkillDefId::new(YINGTIAN_GUICAI),
            },
        ],
    )
    .unwrap();
    let skill = |tx: &Transaction, player, def| {
        tx.state()
            .players
            .get(player)
            .unwrap()
            .skills
            .iter()
            .copied()
            .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
            .unwrap()
    };
    let source = skill(&tx, owner, YINGTIAN_WANSHA);
    let locked = skill(&tx, others[0], YINGTIAN_EFFECT);
    let non_locked = skill(&tx, others[0], YINGTIAN_GUICAI);
    let dying_non_locked = skill(&tx, others[1], YINGTIAN_GUICAI);
    assert!(engine_core::query::card_distance_unlimited_for(
        tx.state(),
        others[0]
    ));

    tx.state_mut().players.get_mut(others[1]).unwrap().dying = true;
    run_effects(
        &mut tx,
        vec![EffectRequest::SetNonLockedSkillSuppression {
            player: owner,
            enabled: true,
            skill: source,
        }],
    )
    .unwrap();
    assert!(!tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(non_locked).unwrap()));
    assert!(tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(locked).unwrap()));
    assert!(tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(dying_non_locked).unwrap()));

    tx.state_mut().players.get_mut(others[1]).unwrap().dying = false;
    run_effects(
        &mut tx,
        vec![EffectRequest::SetNonLockedSkillSuppression {
            player: owner,
            enabled: false,
            skill: source,
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(non_locked).unwrap()));
}

#[test]
fn nzry_junlve_adds_one_mark_for_each_damage_point_dealt_or_taken() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(NZRY_JUNLVE as u64),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: NZRY_JUNLVE as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
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
            skill_def: SkillDefId::new(NZRY_JUNLVE),
        }],
    )
    .unwrap();

    let marks = |tx: &Transaction| {
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == NZRY_JUNLVE_MARK)
            .map(|(_, count)| *count)
            .unwrap_or(0)
    };
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(marks(&tx), 2);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(marks(&tx), 3);
}

#[test]
fn baonu_starts_at_two_before_the_first_turn_and_tracks_damage_amount() {
    let mut skill_package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut roster = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
    for character in &mut skill_package.character_index {
        character.skills = vec![BAONU];
    }
    skill_package.manifest.logic_hash = skill_package.compute_logic_hash();
    for character in &mut roster.character_index {
        character.skills = vec![BAONU];
    }
    roster.manifest.logic_hash = roster.compute_logic_hash();

    let mut engine = Engine::new();
    engine.load_package(&skill_package).unwrap();
    engine.load_package(&roster).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(BAONU as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: BAONU as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let mark_count = |state: &engine_core::state::GameState, player: PlayerId| {
        state
            .players
            .get(player)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == BAONU_MARK)
            .map(|(_, count)| *count)
            .unwrap_or(0)
    };
    let players = engine.state().players.alive_by_seat();
    for player in &players {
        assert_eq!(mark_count(engine.state(), *player), 2);
    }
    let mark_event = engine
        .last_events()
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::MarkChanged { mark_def, count: 2, .. } if *mark_def == BAONU_MARK
            )
        })
        .expect("game-start mark event");
    let first_turn = engine
        .last_events()
        .iter()
        .position(|event| matches!(event, DomainEvent::TurnStateChanged { .. }))
        .expect("first turn event");
    assert!(mark_event < first_turn);

    let source = players[0];
    let target = players[1];
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(mark_count(tx.state(), source), 4);
    assert_eq!(mark_count(tx.state(), target), 4);
}

fn extra_active_state(
    skill_def: u32,
    marks: u32,
    player_count: u8,
) -> (engine_core::state::GameState, PlayerId, SkillId) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    cancel_startup_choices(&mut engine);
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
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(skill_def),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(BAONU_MARK),
                delta: marks as i32,
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == skill_def)
        .unwrap();
    (tx.state().clone(), owner, skill)
}

fn activate_extra(
    state: engine_core::state::GameState,
    owner: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    let revision = state.revision;
    engine_core::command::apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

#[test]
fn ol_wuqian_requires_two_marks_and_wushuang_expires_without_removing_permanent_skill() {
    let (state, owner, skill) = extra_active_state(OL_WUQIAN, 1, 2);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let (_, rejected) = activate_extra(state, owner, skill, vec![target], 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let (state, owner, skill) = extra_active_state(OL_WUQIAN, 4, 2);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let (mut state, accepted) = activate_extra(state, owner, skill, vec![target], 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let (same_state, repeated) = activate_extra(state.clone(), owner, skill, vec![target], 20);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
    assert_eq!(same_state, state);
    let temporary = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == 18)
        .unwrap();
    assert_eq!(temporary.temporary_until_turn_end_of, owner);
    state.turn.phase = Phase::End;
    let revision = state.revision;
    let (state, ended) = engine_core::command::apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert!(!state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == 18));

    let (state, owner, skill) = extra_active_state(OL_WUQIAN, 2, 2);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(18),
        }],
    )
    .unwrap();
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let (mut state, accepted) = activate_extra(tx.state().clone(), owner, skill, vec![target], 4);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == 18)
            .unwrap()
            .temporary_until_turn_end_of,
        PlayerId::NONE
    );
    state.turn.phase = Phase::End;
    let revision = state.revision;
    let (state, _) = engine_core::command::apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(5),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == 18));
}

#[test]
fn wuqian_requires_missing_wushuang_then_spends_two_marks_for_temporary_wushuang() {
    let (state, owner, wuqian) = extra_active_state(WUQIAN, 2, 2);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(18),
        }],
    )
    .unwrap();
    let state = tx.state().clone();
    let (unchanged, rejected) = activate_extra(state.clone(), owner, wuqian, Vec::new(), 21);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(unchanged, state);

    let wushuang = state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == 18)
        .unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RemoveSkill { skill: wushuang }],
    )
    .unwrap();
    let (state, accepted) = activate_extra(tx.state().clone(), owner, wuqian, Vec::new(), 22);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == BAONU_MARK)
            .map_or(0, |(_, count)| *count),
        0
    );
    let temporary = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == 18)
        .unwrap();
    assert_eq!(temporary.temporary_until_turn_end_of, owner);
}

#[test]
fn wumou_loses_hp_without_baonu_and_offers_the_two_forced_branches_with_baonu() {
    let (state, owner, _) = extra_active_state(WUMOU, 0, 2);
    let mut tx = Transaction::begin(state);
    let card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::TrickWuzhong;
    let hp = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: owner,
            card,
            targets: Vec::new(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);

    let (state, owner, _) = extra_active_state(WUMOU, 1, 2);
    let mut tx = Transaction::begin(state);
    let card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::TrickWuzhong;
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: owner,
                card,
                targets: Vec::new(),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::VmResume
    );
}

#[test]
fn ol_shenfen_resolves_damage_equipment_chosen_hand_discard_and_turn_over_in_order() {
    let (state, owner, skill) = extra_active_state(OL_SHENFEN, 6, 2);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: target,
            count: 2,
        }],
    )
    .unwrap();
    let equip = tx.state().hand_cards(target)[0];
    let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, target).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![equip],
            to: equip_zone,
            from: Some(hand),
        }],
    )
    .unwrap();
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let (state, activated) = activate_extra(tx.state().clone(), owner, skill, Vec::new(), 10);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert!(state.zones.get(equip_zone).unwrap().cards.is_empty());
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DiscardHandForMassDamage
    );
    let offered = state
        .resolution
        .continuation_payload
        .get(3..)
        .unwrap()
        .to_vec();
    let selected = offered.iter().rev().take(4).copied().collect::<Vec<_>>();
    let choice_id = state.resolution.pending_choice_id;
    let revision = state.revision;
    let (state, resolved) = engine_core::command::apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(11),
            kind: CommandKind::ResolveChoice {
                choice_id: engine_api::ChoiceId::new(choice_id),
                selections: selected,
                cancelled: false,
            },
        },
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(target), 1);
    assert!(state.players.get(owner).unwrap().turned_over);
}

#[test]
fn shenfen_once_discards_four_combined_hand_equip_without_turning_over() {
    let (state, owner, skill) = extra_active_state(SHENFEN, 6, 2);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: target,
            count: 2,
        }],
    )
    .unwrap();
    let equip = tx.state().hand_cards(target)[0];
    let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, target).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![equip],
            to: equip_zone,
            from: Some(hand),
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, activated) = activate_extra(tx.state().clone(), owner, skill, Vec::new(), 30);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(!state.players.get(owner).unwrap().turned_over);
    let offered = state.resolution.continuation_payload[4..].to_vec();
    assert!(offered.contains(&equip.raw()));
    let mut selected = vec![equip.raw()];
    selected.extend(
        offered
            .into_iter()
            .filter(|card| *card != equip.raw())
            .take(3),
    );
    let choice_id = state.resolution.pending_choice_id;
    let revision = state.revision;
    let (state, resolved) = engine_core::command::apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(31),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice_id),
                selections: selected,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(resolved.result, CommandResult::Accepted { .. }),
        "{resolved:?}"
    );
    assert!(!state.has_pending_choice());
    assert!(state.zones.get(equip_zone).unwrap().cards.is_empty());
    assert_eq!(state.hand_count(target), 2);
    assert!(!state.players.get(owner).unwrap().turned_over);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: MarkDefId::new(BAONU_MARK),
            delta: 6,
        }],
    )
    .unwrap();
    let (_, repeated) = activate_extra(tx.state().clone(), owner, skill, Vec::new(), 32);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}
}
