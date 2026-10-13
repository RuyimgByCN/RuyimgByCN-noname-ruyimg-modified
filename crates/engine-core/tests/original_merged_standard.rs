//! O2 全量合并批次 1（2026-10-14）：家族 standard 的 25 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_standard_fanjian {
//! Golden coverage for STANDARD「反间」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FANJIAN: u32 = 36;
const YINGZI: u32 = 20;
const ZHOUYU: u32 = 19;
const SOURCE: &str = include_str!("../../../content/standard/fanjian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(FANJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == FANJIAN)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn activate(setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        setup.state.clone(),
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(setup.state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![setup.target],
                card_targets: vec![],
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selection: u32,
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
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_current_abi_active_and_complete_zhouyu() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == FANJIAN)
        .expect("fanjian active");
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.hand_nonempty);
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|targets| (targets.min, targets.max, targets.allow_self)),
        Some((1, 1, false))
    );
    assert!(package
        .character_index
        .iter()
        .any(|character| { character.id == ZHOUYU && character.skills == [YINGZI, FANJIAN] }));
    let compiled = content_compiler::compile_source_named("fanjian", SOURCE).unwrap();
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));
}

#[test]
fn target_guesses_then_gains_owner_hand_card_and_mismatch_deals_owner_damage() {
    let mut setup = setup(2_026_072_501);
    let card = setup.state.hand_cards(setup.owner)[0];
    setup.state.cards.get_mut(card).unwrap().suit = Suit::Spade;
    let target_card = setup.state.hand_cards(setup.target)[0];
    let hp = setup.state.players.get(setup.target).unwrap().hp;

    let (state, activated) = activate(&setup, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::FanjianChooseSuitOrHand
    );
    assert_eq!(
        activated.choice.as_ref().map(|choice| {
            choice
                .options
                .iter()
                .map(|option| option.token)
                .collect::<Vec<_>>()
        }),
        Some(vec![2, 4, 3, 1])
    );

    let (state, guessed) = resolve(state, setup.target, 2, 2);
    assert!(matches!(guessed.result, CommandResult::Accepted { .. }));
    assert!(guessed.choice.as_ref().is_some_and(|choice| choice
        .options
        .iter()
        .any(|option| option.token == card.raw())));

    let (unchanged, forged) = resolve(state.clone(), setup.target, target_card.raw(), 3);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(unchanged, state);

    let (state, gained) = resolve(state, setup.target, card.raw(), 4);
    assert!(matches!(gained.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(setup.target).contains(&card));
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
    assert!(gained.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target,
            amount: 1,
            ..
        } if *source == setup.owner && *target == setup.target
    )));

    setup.state = state;
    let (_, repeated) = activate(&setup, 5);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}

#[test]
fn matching_owner_effective_suit_moves_card_without_damage() {
    let mut setup = setup(2_026_072_502);
    let card: CardId = setup.state.hand_cards(setup.owner)[0];
    setup.state.cards.get_mut(card).unwrap().suit = Suit::Diamond;
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, _) = activate(&setup, 6);
    let (state, _) = resolve(state, setup.target, 4, 7);
    let (state, result) = resolve(state, setup.target, card.raw(), 8);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(setup.target).contains(&card));
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp);
}
}

mod original_standard_fankui {
//! Golden coverage for standard 反馈's once-per-damage-event card-taking contract.

use content_compiler::compile_source_named;
use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FANKUI: u32 = 5;
const SOURCE: &str = include_str!("../../../content/standard/fankui.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = compile_source_named("standard-fankui-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_725, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let old_skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.id)
        .collect();
    let mut effects: Vec<_> = old_skills
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(FANKUI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, source)
}

fn damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    (tx.commit().state, result)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_and_two_damage_offer_exactly_one_card_take() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == FANKUI
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target_source_alive_gainable_card"
            && trigger.optional
    }));

    let (state, owner, source) = setup();
    let owner_hand = state.hand_count(owner);
    let source_hand = state.hand_count(source);
    let (state, result) = damage(state, source, owner, 2);
    assert_eq!(result, Err(EffectError::ChoiceRequested));

    let (state, accepted) = resolve(state, owner, vec![1], 1);
    assert_eq!(
        accepted.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let card = accepted.choice.as_ref().unwrap().options[0].token;
    let (state, taken) = resolve(state, owner, vec![card], 2);
    assert!(matches!(taken.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.hand_count(source), source_hand - 1);
}

#[test]
fn self_damage_requires_equipment_and_never_exposes_hand() {
    let (state, owner, _) = setup();
    let (state, result) = damage(state, owner, owner, 1);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());

    let card = state.hand_cards(owner)[0];
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
    let (state, result) = damage(tx.commit().state, owner, owner, 1);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, accepted) = resolve(state, owner, vec![1], 3);
    assert_eq!(
        accepted.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let options: Vec<_> = accepted
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(options, vec![card.raw()]);
    let (_, taken) = resolve(state, owner, vec![card.raw()], 4);
    assert!(matches!(taken.result, CommandResult::Accepted { .. }));
}
}

mod original_standard_ganglie {
//! Golden coverage for standard 刚烈 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{damage_nature, EffectRequest};

const GANGLIE: u32 = 7;
const SOURCE: &str = include_str!("../../../content/standard/ganglie.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = compile_source_named("standard-ganglie-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_725_7, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(GANGLIE),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = 20;
        participant.max_hp = 20;
    }
    (tx.commit().state, owner, source)
}

fn put_judgment(state: &mut GameState, suit: Suit, def: u32) -> CardId {
    let zone = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn damage(state: GameState, source: PlayerId, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
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
fn package_publishes_exact_optional_abi171_ganglie() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == GANGLIE
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target"
            && trigger.optional
    }));
    let compiled = compile_source_named("standard-ganglie", SOURCE).unwrap();
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));
}

#[test]
fn nonheart_source_discards_exactly_two_hand_cards_or_takes_damage() {
    let (mut state, owner, source) = setup();
    put_judgment(&mut state, Suit::Spade, 171_001);
    let source_hp = state.players.get(source).unwrap().hp;
    let hand = state.hand_cards(source);
    let hand_count = state.hand_count(source);
    assert!(hand.len() >= 2);
    let state = damage(state, source, owner);
    let (state, accepted) = resolve(state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GanglieSourceDiscardOrDamage
    );
    let (state, discarded) = resolve(state, source, vec![hand[0].raw(), hand[1].raw()], false, 2);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(source), hand_count - 2);
    assert_eq!(state.players.get(source).unwrap().hp, source_hp);

    let (mut state, owner, source) = setup();
    put_judgment(&mut state, Suit::Club, 171_002);
    let source_hp = state.players.get(source).unwrap().hp;
    let state = damage(state, source, owner);
    let (state, _) = resolve(state, owner, vec![1], false, 3);
    let (state, declined) = resolve(state, source, Vec::new(), true, 4);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(source).unwrap().hp, source_hp - 1);
}

#[test]
fn heart_missing_source_or_missing_two_card_cost_opens_no_source_choice() {
    let (mut state, owner, source) = setup();
    put_judgment(&mut state, Suit::Heart, 171_010);
    let source_hp = state.players.get(source).unwrap().hp;
    let state = damage(state, source, owner);
    let (state, heart) = resolve(state, owner, vec![1], false, 5);
    assert!(matches!(heart.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(source).unwrap().hp, source_hp);

    let (mut state, owner, _) = setup();
    let judgment = put_judgment(&mut state, Suit::Spade, 171_011);
    let state = damage(state, PlayerId::NONE, owner);
    let (state, missing_source) = resolve(state, owner, vec![1], false, 6);
    assert!(matches!(
        missing_source.result,
        CommandResult::Accepted { .. }
    ));
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(judgment).unwrap().zone, discard);

    let (mut state, owner, source) = setup();
    let hand_zone = state.zones.find(ZoneKind::Hand, source).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in state.hand_cards(source).into_iter().skip(1) {
        state
            .zones
            .get_mut(hand_zone)
            .unwrap()
            .cards
            .retain(|id| *id != card);
        state.zones.get_mut(discard).unwrap().cards.push(card);
        let card_state = state.cards.get_mut(card).unwrap();
        card_state.zone = discard;
        card_state.owner = PlayerId::NONE;
    }
    put_judgment(&mut state, Suit::Diamond, 171_012);
    let source_hp = state.players.get(source).unwrap().hp;
    let state = damage(state, source, owner);
    let (state, insufficient) = resolve(state, owner, vec![1], false, 7);
    assert!(matches!(
        insufficient.result,
        CommandResult::Accepted { .. }
    ));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(source).unwrap().hp, source_hp - 1);
}
}

mod original_standard_hujia {
//! Golden coverage for Standard 护驾 rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const HUJIA: u32 = 3_353_250_727;
const SOURCE: &str = include_str!("../../../content/standard/hujia.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first_wei: PlayerId,
    second_wei: PlayerId,
    attacker: PlayerId,
    first_shan: CardId,
    second_shan: CardId,
    sha: CardId,
    skill: SkillId,
}

fn setup() -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_725_184, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let [owner, first_wei, second_wei, attacker] = players.as_slice() else {
        panic!("expected four players");
    };
    let (owner, first_wei, second_wei, attacker) = (*owner, *first_wei, *second_wei, *attacker);
    for (player, character) in [(owner, 1), (first_wei, 2), (second_wei, 3), (attacker, 8)] {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .character_def = character;
    }
    tx.state_mut().players.get_mut(owner).unwrap().identity = Identity::Lord;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HUJIA),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, first_wei, second_wei, attacker] {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    let first_shan = tx.state().hand_cards(first_wei)[0];
    let second_shan = tx.state().hand_cards(second_wei)[0];
    let sha = tx.state().hand_cards(attacker)[0];
    tx.state_mut().cards.get_mut(first_shan).unwrap().kind = CardKind::Shan;
    tx.state_mut().cards.get_mut(second_shan).unwrap().kind = CardKind::Shan;
    tx.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::Sha;
    tx.state_mut().turn.current = attacker;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == HUJIA)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        first_wei,
        second_wei,
        attacker,
        first_shan,
        second_shan,
        sha,
        skill,
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

#[test]
fn package_closes_caocao_with_lord_only_group_response() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let caocao = package
        .character_index
        .iter()
        .find(|character| character.id == 1)
        .unwrap();
    assert_eq!(caocao.skills, [1, HUJIA]);
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == HUJIA)
        .unwrap();
    assert!(skill.lord_only);
    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == HUJIA)
        .unwrap();
    assert_eq!(view.source_zone.as_deref(), Some("group_wei"));
    let compiled = content_compiler::compile_source_named("standard-hujia", SOURCE).unwrap();
    assert_eq!(compiled.view_as_index, vec![view.clone()]);
}

#[test]
fn wei_players_are_asked_in_seat_order_until_one_provides_shan() {
    let setup = setup();
    let hp = setup.state.players.get(setup.owner).unwrap().hp;
    let (state, offered) = command(
        setup.state,
        setup.attacker,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.owner],
            card_targets: vec![],
        },
        1,
    );
    let hujia_token = offered.choice.as_ref().unwrap().options[0].token;
    assert_eq!(offered.choice.as_ref().unwrap().observer, setup.owner);
    let (state, first) = resolve(state, setup.owner, vec![hujia_token], false, 2);
    assert_eq!(first.choice.as_ref().unwrap().observer, setup.first_wei);
    let (state, second) = resolve(state, setup.first_wei, vec![], true, 3);
    assert_eq!(second.choice.as_ref().unwrap().observer, setup.second_wei);
    let (state, saved) = resolve(
        state,
        setup.second_wei,
        vec![setup.second_shan.raw()],
        false,
        4,
    );
    assert!(matches!(saved.result, CommandResult::Accepted { .. }));
    assert!(saved.choice.is_none());
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp);
    assert!(state
        .hand_cards(setup.first_wei)
        .contains(&setup.first_shan));
    assert_eq!(
        state.cards.get(setup.second_shan).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(saved.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardResponded {
            player,
            respond_to,
            card,
            ..
        } if *player == setup.second_wei
            && *respond_to == setup.owner
            && *card == setup.second_shan
    )));
}

#[test]
fn hujia_is_unavailable_when_the_owner_is_not_the_lord() {
    let mut setup = setup();
    setup.state.players.get_mut(setup.owner).unwrap().identity = Identity::Rebel;
    let hp = setup.state.players.get(setup.owner).unwrap().hp;
    let (state, result) = command(
        setup.state,
        setup.attacker,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.owner],
            card_targets: vec![],
        },
        5,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp - 1);
    assert_eq!(state.skills.get(setup.skill).unwrap().use_count, 0);
}
}

mod original_standard_jianxiong {
//! Golden coverage for standard 奸雄's physical damage-card gain contract.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIANXIONG: u32 = 1;
const SOURCE: &str = include_str!("../../../content/standard/jianxiong.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = compile_source_named("standard-jianxiong-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_725_1, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JIANXIONG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 20;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 20;
    (tx.commit().state, owner, source)
}

fn inject(state: &mut GameState, zone: ZoneKind, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(zone, PlayerId::NONE).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
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

fn deal_card_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    card: CardId,
    kind: CardKind,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card,
            card_kind: kind.tag(),
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

#[test]
fn package_and_runtime_gain_the_processing_damage_card() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JIANXIONG
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target_card_in_processing"
            && trigger.optional
    }));

    let (mut state, owner, source) = setup();
    let card = inject(&mut state, ZoneKind::Processing, CardKind::Sha, 170_001);
    let hand_before = state.hand_count(owner);
    let (state, result) = deal_card_damage(state, source, owner, card, CardKind::Sha);
    assert_eq!(result, Err(EffectError::ChoiceRequested));

    let (state, resolved) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert!(state.hand_cards(owner).contains(&card));
}

#[test]
fn cardless_or_already_discarded_damage_opens_no_choice() {
    let (state, owner, source) = setup();
    let hand_before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    let mut state = tx.commit().state;
    assert!(!state.has_pending_choice());

    let card = inject(&mut state, ZoneKind::DiscardPile, CardKind::Sha, 170_002);
    let (state, result) = deal_card_damage(state, source, owner, card, CardKind::Sha);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand_before);
}
}

mod original_standard_jijiang {
//! Golden coverage for Standard 激将 rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIJIANG: u32 = 2_377_283_906;
const SOURCE: &str = include_str!("../../../content/standard/jijiang.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first_shu: PlayerId,
    second_shu: PlayerId,
    target: PlayerId,
    first_sha: CardId,
    second_sha: CardId,
    unlock_card: CardId,
    skill: SkillId,
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(97_000 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_725_187, "identity-standard").unwrap();
    let [owner, first_shu, second_shu, target]: [PlayerId; 4] =
        tx.state().players.alive_by_seat().try_into().unwrap();
    for (player, character) in [(owner, 8), (first_shu, 9), (second_shu, 10), (target, 1)] {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .character_def = character;
    }
    tx.state_mut().players.get_mut(owner).unwrap().identity = Identity::Lord;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([owner, first_shu, second_shu, target].map(|player| {
        EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        }
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JIJIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, first_shu, second_shu, target] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    let first_sha = inject(&mut tx, first_shu, CardKind::Sha, 2);
    let second_sha = inject(&mut tx, second_shu, CardKind::Sha, 3);
    let unlock_card = inject(&mut tx, owner, CardKind::Jiu, 4);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == JIJIANG)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        first_shu,
        second_shu,
        target,
        first_sha,
        second_sha,
        unlock_card,
        skill,
    }
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

fn use_jijiang(setup: &Setup) -> CommandKind {
    CommandKind::UseSkillCard {
        skill: setup.skill,
        card_kind: CardKind::Sha.tag(),
        targets: vec![setup.target],
        equip_slot: 0,
    }
}

#[test]
fn package_closes_liubei_with_lord_only_group_sha() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let liubei = package
        .character_index
        .iter()
        .find(|character| character.id == 8)
        .unwrap();
    assert_eq!(liubei.skills, [11, JIJIANG]);
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == JIJIANG)
        .unwrap();
    assert!(skill.lord_only);
    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == JIJIANG)
        .unwrap();
    assert_eq!(view.as_kind, "sha");
    assert_eq!(view.window, "any");
    assert_eq!(view.source_zone.as_deref(), Some("group_shu"));
    let compiled = content_compiler::compile_source_named("standard-jijiang", SOURCE).unwrap();
    assert_eq!(compiled.view_as_index, vec![view.clone()]);
}

#[test]
fn shu_players_are_asked_in_seat_order_and_provider_card_becomes_lords_sha() {
    let setup = setup();
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, first) = submit(setup.state.clone(), setup.owner, use_jijiang(&setup), 1);
    assert_eq!(first.choice.as_ref().unwrap().observer, setup.first_shu);
    let (state, second) = resolve(state, setup.first_shu, vec![], true, 2);
    assert_eq!(second.choice.as_ref().unwrap().observer, setup.second_shu);
    let (state, used) = resolve(
        state,
        setup.second_shu,
        vec![setup.second_sha.raw()],
        false,
        3,
    );
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert!(used.choice.is_none());
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
    assert!(state.hand_cards(setup.first_shu).contains(&setup.first_sha));
    assert_eq!(
        state.cards.get(setup.second_sha).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(used.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardResponded {
            player,
            respond_to,
            card,
            ..
        } if *player == setup.second_shu
            && *respond_to == setup.owner
            && *card == setup.second_sha
    )));
    assert!(used.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardUsed {
            player,
            targets,
            ..
        } if *player == setup.owner && targets == &[setup.target]
    )));
}

#[test]
fn delegated_sha_can_answer_duel_for_the_lord() {
    let mut setup = setup();
    setup.state.turn.current = setup.target;
    let mut tx = Transaction::begin(setup.state);
    let duel = inject(&mut tx, setup.target, CardKind::TrickJuedou, 5);
    setup.state = tx.commit().state;
    let owner_hp = setup.state.players.get(setup.owner).unwrap().hp;
    let target_hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, duel_choice) = submit(
        setup.state,
        setup.target,
        CommandKind::PlayCard {
            card: duel,
            targets: vec![setup.owner],
            card_targets: vec![],
        },
        4,
    );
    let token = duel_choice
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .find(|option| option.token > u32::MAX - 1_000)
        .unwrap()
        .token;
    let (state, provider) = resolve(state, setup.owner, vec![token], false, 5);
    assert_eq!(provider.choice.as_ref().unwrap().observer, setup.first_shu);
    let (state, resolved) = resolve(
        state,
        setup.first_shu,
        vec![setup.first_sha.raw()],
        false,
        6,
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(resolved.choice.is_none());
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp);
    assert_eq!(state.players.get(setup.target).unwrap().hp, target_hp - 1);
}

#[test]
fn all_decline_blocks_only_an_immediate_retry() {
    let setup = setup();
    let use_skill = use_jijiang(&setup);
    let (state, first) = submit(setup.state, setup.owner, use_skill.clone(), 7);
    assert_eq!(first.choice.as_ref().unwrap().observer, setup.first_shu);
    let (state, second) = resolve(state, setup.first_shu, vec![], true, 8);
    assert_eq!(second.choice.as_ref().unwrap().observer, setup.second_shu);
    let (state, declined) = resolve(state, setup.second_shu, vec![], true, 9);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert!(declined.choice.is_none());

    let (state, blocked) = submit(state, setup.owner, use_skill.clone(), 10);
    assert!(matches!(blocked.result, CommandResult::Rejected { .. }));
    assert!(blocked.choice.is_none());

    let (state, unlocked) = submit(
        state,
        setup.owner,
        CommandKind::PlayCard {
            card: setup.unlock_card,
            targets: vec![],
            card_targets: vec![],
        },
        11,
    );
    assert!(matches!(unlocked.result, CommandResult::Accepted { .. }));
    let (_, retry) = submit(state, setup.owner, use_skill, 12);
    assert_eq!(retry.choice.as_ref().unwrap().observer, setup.first_shu);
}
}

mod original_standard_jijiu {
//! Golden coverage for Standard 急救 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{CardDefId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::dying_save_card_for;
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIJIU: u32 = 2_961_041_949;
const SOURCE: &str = include_str!("../../../content/standard/jijiu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> engine_api::CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(9_600 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn package_closes_huatuo_with_response_only_jijiu() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == JIJIU)
        .unwrap();
    assert_eq!(view.as_kind, "tao");
    assert_eq!(view.window, "response");
    assert_eq!(view.filter, "owner_outside_turn_red");
    assert_eq!(view.source_zone.as_deref(), Some("hand_equip"));
    let huatuo = package
        .character_index
        .iter()
        .find(|character| character.name_key == "huatuo")
        .unwrap();
    assert!(huatuo.skills.contains(&JIJIU));
}

#[test]
fn red_hand_or_equipment_card_is_tao_only_outside_owner_turn() {
    let package = compile_source_named("standard-jijiu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_725_44, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    for skill in tx.state().players.get(owner).unwrap().skills.clone() {
        run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let old_hand = tx.state().hand_cards(owner);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: old_hand,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JIJIU),
        }],
    )
    .unwrap();
    let black = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        7,
    );
    let red = inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Heart,
        8,
    );

    tx.state_mut().turn.current = other;
    assert_eq!(dying_save_card_for(tx.state(), owner, other), Some(red));
    tx.state_mut().turn.current = owner;
    assert_eq!(dying_save_card_for(tx.state(), owner, other), None);
    assert_ne!(black, red);
}
}

mod original_standard_jiuyuan {
//! Golden coverage for Standard 救援 rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{CardKind, GameState, Identity, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIUYUAN: u32 = 3_165_045_964;
const SOURCE: &str = include_str!("../../../content/standard/jiuyuan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    responder: PlayerId,
    tao: engine_api::CardId,
}

fn setup(owner_identity: Identity, responder_character: u32, hp: i32) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_725_185, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let [owner, responder, other] = players.as_slice() else {
        panic!("expected three players");
    };
    let (owner, responder, other) = (*owner, *responder, *other);
    for (player, character) in [(owner, 15), (responder, responder_character), (other, 8)] {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .character_def = character;
    }
    tx.state_mut().players.get_mut(owner).unwrap().identity = owner_identity;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JIUYUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, responder, other] {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    let tao = tx.state().hand_cards(responder)[0];
    tx.state_mut().cards.get_mut(tao).unwrap().kind = CardKind::Tao;
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.hp = hp;
        player.dying = true;
    }
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ContinueDyingSave {
                player: owner,
                source: other,
                responders: vec![responder],
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    Setup {
        state: tx.commit().state,
        owner,
        responder,
        tao,
    }
}

fn save(setup: Setup) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(setup.state.resolution.pending_choice_id);
    apply_command(
        setup.state,
        Command {
            actor: setup.responder,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_closes_sunquan_with_locked_lord_skill() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let sunquan = package
        .character_index
        .iter()
        .find(|character| character.id == 15)
        .unwrap();
    assert_eq!(sunquan.skills, [10, JIUYUAN]);
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == JIUYUAN)
        .unwrap();
    assert!(skill.locked);
    assert!(skill.lord_only);
    let compiled = content_compiler::compile_source_named("standard-jiuyuan", SOURCE).unwrap();
    assert_eq!(
        compiled.trigger_index[0].filter,
        "owner_dying_other_tao_responder_group_wu"
    );
}

#[test]
fn other_wu_tao_recovers_dying_lord_twice_with_responder_as_source() {
    let setup = setup(Identity::Lord, 16, -1);
    let owner = setup.owner;
    let responder = setup.responder;
    let tao = setup.tao;
    let (state, result) = save(setup);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?} events={:?}",
        result.result,
        result.events
    );
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(owner).unwrap().hp, 1);
    assert!(!state.players.get(owner).unwrap().dying);
    assert_eq!(
        state.cards.get(tao).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert_eq!(
        result
            .events
            .iter()
            .filter(|event| matches!(
                event,
                DomainEvent::HpRecovered {
                    source,
                    player,
                    amount: 1,
                } if *source == responder && *player == owner
            ))
            .count(),
        2
    );
}

#[test]
fn non_wu_or_non_lord_tao_only_performs_the_base_recovery() {
    for (identity, responder_character) in [(Identity::Lord, 8), (Identity::Rebel, 16)] {
        let setup = setup(identity, responder_character, 0);
        let owner = setup.owner;
        let (state, result) = save(setup);
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert_eq!(state.players.get(owner).unwrap().hp, 1);
        assert!(!state.players.get(owner).unwrap().dying);
        assert_eq!(
            result
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    DomainEvent::HpRecovered {
                        player,
                        amount: 1,
                        ..
                    } if *player == owner
                ))
                .count(),
            1
        );
    }
}
}

mod original_standard_lijian {
//! Golden coverage for Standard 离间 rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LIJIAN: u32 = 2_292_478_631;
const DIAOCHAN: u32 = 25;
const JUE_DOU_LOCK: u32 = 2_000_000_183;
const SOURCE: &str = include_str!("../../../content/standard/lijian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    user: PlayerId,
    observer: PlayerId,
    skill: SkillId,
    cost: CardId,
    wuxie: CardId,
}

fn setup(lock_user: bool) -> Setup {
    let standard = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_725_183, "identity-standard").unwrap();
    if lock_user {
        let lock = content_compiler::compile_source_named(
            "standard-lijian-lock-test",
            &format!(
                "skill juedou_lock({JUE_DOU_LOCK}) locked {{
                    modifier card_use_cancel(juedou) override 1;
                }}"
            ),
        )
        .unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&lock).unwrap();
    }
    let players = tx.state().players.alive_by_seat();
    let [owner, target, user, observer] = players.as_slice() else {
        panic!("expected four players");
    };
    let (owner, target, user, observer) = (*owner, *target, *user, *observer);
    tx.state_mut().players.get_mut(owner).unwrap().character_def = DIAOCHAN;
    tx.state_mut()
        .players
        .get_mut(target)
        .unwrap()
        .character_def = 1;
    tx.state_mut().players.get_mut(user).unwrap().character_def = 2;
    tx.state_mut()
        .players
        .get_mut(observer)
        .unwrap()
        .character_def = 7;

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIJIAN),
    });
    if lock_user {
        effects.push(EffectRequest::AddSkill {
            player: user,
            skill_def: SkillDefId::new(JUE_DOU_LOCK),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    for player in [owner, target, user, observer] {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    let cost = tx.state().hand_cards(owner)[0];
    let wuxie = tx.state().hand_cards(observer)[0];
    tx.state_mut().cards.get_mut(wuxie).unwrap().kind = CardKind::TrickWuxie;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == LIJIAN)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        user,
        observer,
        skill,
        cost,
        wuxie,
    }
}

fn activate(setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        setup.state.clone(),
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(setup.state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![setup.target, setup.user],
                card_targets: vec![setup.cost],
            },
        },
    )
}

#[test]
fn package_closes_diaochan_and_declares_the_ordered_duel_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let diaochan = package
        .character_index
        .iter()
        .find(|character| character.id == DIAOCHAN)
        .unwrap();
    assert_eq!(diaochan.skills, [LIJIAN, 4]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == LIJIAN)
        .unwrap();
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (2, 2));
    assert!(!targets.allow_self);
    assert!(targets.male);
    assert!(targets.second_can_use_juedou_on_first);
    let compiled = content_compiler::compile_source_named("standard-lijian", SOURCE).unwrap();
    assert!(compiled
        .modules
        .iter()
        .all(|expected| package.modules.iter().any(
            |actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash
        )));
}

#[test]
fn second_target_uses_unwuxieable_duel_on_first_after_the_cost_is_discarded() {
    let setup = setup(false);
    let target_hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, result) = activate(&setup, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(setup.target).unwrap().hp, target_hp - 1);
    assert_eq!(
        state.cards.get(setup.cost).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(state.hand_cards(setup.observer).contains(&setup.wuxie));
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardUsed {
            player,
            targets,
            ..
        } if *player == setup.user && targets == &[setup.target]
    )));
}

#[test]
fn an_illegal_second_user_is_rejected_before_the_discard_cost() {
    let setup = setup(true);
    let (same, result) = activate(&setup, 2);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(same, setup.state);
    assert!(same.hand_cards(setup.owner).contains(&setup.cost));
    assert_eq!(same.skills.get(setup.skill).unwrap().use_count, 0);
}
}

mod original_standard_liuli {
//! Golden coverage for Standard 流离 rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LIULI: u32 = 3_733_125_857;
const SOURCE: &str = include_str!("../../../content/standard/liuli.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    source: PlayerId,
    owner: PlayerId,
    candidate: PlayerId,
    far: PlayerId,
    sha: CardId,
    cost: CardId,
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind,
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

fn setup() -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_725_186, "identity-standard").unwrap();
    let [source, owner, candidate, far]: [PlayerId; 4] =
        tx.state().players.alive_by_seat().try_into().unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([source, owner, candidate, far].into_iter().map(|player| {
        EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        }
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIULI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in [source, owner, candidate, far] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    tx.state_mut().turn.current = source;
    tx.state_mut().turn.phase = Phase::Play;
    let cost = inject(&mut tx, owner, CardKind::Other, Suit::Club, 2);
    let sha = inject(&mut tx, source, CardKind::Sha, Suit::Spade, 3);
    Setup {
        state: tx.commit().state,
        source,
        owner,
        candidate,
        far,
        sha,
        cost,
    }
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

#[test]
fn package_closes_daqiao_with_guose_and_exact_liuli() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let daqiao = package
        .character_index
        .iter()
        .find(|character| character.id == 20)
        .unwrap();
    assert_eq!(daqiao.skills, [28, LIULI]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == LIULI)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_sha_target_can_discard_redirect_to_other_in_attack_range"
    );
    let compiled = content_compiler::compile_source_named("standard-liuli", SOURCE).unwrap();
    assert_eq!(compiled.trigger_index[0].skill_def_id, trigger.skill_def_id);
    assert_eq!(compiled.trigger_index[0].filter, trigger.filter);
    assert_eq!(compiled.trigger_index[0].module, trigger.module);
}

#[test]
fn discard_then_target_redirects_saved_sha_and_preserves_original_target() {
    let setup = setup();
    assert!(engine_core::query::player_in_attack_range(
        &setup.state,
        setup.owner,
        setup.candidate
    ));
    assert!(!engine_core::query::player_in_attack_range(
        &setup.state,
        setup.source,
        setup.candidate
    ));
    let owner_hp = setup.state.players.get(setup.owner).unwrap().hp;
    let candidate_hp = setup.state.players.get(setup.candidate).unwrap().hp;
    let (state, cost_choice) = submit(
        setup.state,
        setup.source,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.owner],
            card_targets: vec![],
        },
        1,
    );
    let cost_choice = cost_choice.choice.unwrap();
    assert_eq!(cost_choice.observer, setup.owner);
    assert_eq!(cost_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        cost_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [setup.cost.raw()]
    );
    let (state, target_choice) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: cost_choice.choice_id,
            selections: vec![setup.cost.raw()],
            cancelled: false,
        },
        2,
    );
    let target_choice = target_choice.choice.unwrap();
    assert_eq!(target_choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(
        target_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [setup.candidate.raw()]
    );
    assert!(!target_choice
        .options
        .iter()
        .any(|option| option.token == setup.source.raw()));
    assert!(!target_choice
        .options
        .iter()
        .any(|option| option.token == setup.far.raw()));
    let (state, redirected) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: target_choice.choice_id,
            selections: vec![setup.candidate.raw()],
            cancelled: false,
        },
        3,
    );
    assert!(matches!(redirected.result, CommandResult::Accepted { .. }));
    assert!(redirected.choice.is_none());
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp);
    assert_eq!(
        state.players.get(setup.candidate).unwrap().hp,
        candidate_hp - 1
    );
    assert_eq!(
        state.cards.get(setup.cost).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(redirected.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardTargetAdded {
            player,
            card,
            target,
            targets,
        } if *player == setup.source
            && *card == setup.sha
            && *target == setup.candidate
            && targets == &[setup.candidate]
    )));
}

#[test]
fn declining_the_discard_keeps_the_original_sha_target() {
    let setup = setup();
    let owner_hp = setup.state.players.get(setup.owner).unwrap().hp;
    let candidate_hp = setup.state.players.get(setup.candidate).unwrap().hp;
    let (state, offered) = submit(
        setup.state,
        setup.source,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.owner],
            card_targets: vec![],
        },
        4,
    );
    let choice = offered.choice.unwrap();
    let (state, result) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: vec![],
            cancelled: true,
        },
        5,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(setup.candidate).unwrap().hp, candidate_hp);
    assert!(state.hand_cards(setup.owner).contains(&setup.cost));
}
}

mod original_standard_luoshen {
//! Golden coverage for Standard 洛神 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LUOSHEN: u32 = 3_972_660_146;
const SOURCE: &str = include_str!("../../../content/standard/luoshen.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first_black: engine_api::CardId,
    second_black: engine_api::CardId,
    red: engine_api::CardId,
}

fn setup() -> Setup {
    let package = compile_source_named("standard-luoshen-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_725_182, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(LUOSHEN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let cards = tx.state().zones.get(draw).unwrap().cards.clone();
    let first_black = cards[cards.len() - 1];
    let second_black = cards[cards.len() - 2];
    let red = cards[cards.len() - 3];
    tx.state_mut().cards.get_mut(first_black).unwrap().suit = Suit::Spade;
    tx.state_mut().cards.get_mut(second_black).unwrap().suit = Suit::Club;
    tx.state_mut().cards.get_mut(red).unwrap().suit = Suit::Heart;
    Setup {
        state: tx.commit().state,
        owner,
        first_black,
        second_black,
        red,
    }
}

fn offer(
    setup: Setup,
) -> (
    GameState,
    PlayerId,
    engine_api::CardId,
    engine_api::CardId,
    engine_api::CardId,
) {
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: setup.owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = tx.commit();
    assert_eq!(
        committed.output.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );
    (
        committed.state,
        setup.owner,
        setup.first_black,
        setup.second_black,
        setup.red,
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
fn package_closes_zhenji_with_luoshen_and_qingguo() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == 7)
        .unwrap();
    assert_eq!(character.skills, [LUOSHEN, 24]);
}

#[test]
fn black_results_are_gained_and_repeat_until_the_first_red_result() {
    let setup = setup();
    let hand_before = setup.state.hand_count(setup.owner);
    let (state, owner, first_black, second_black, red) = offer(setup);
    let (state, first) = resolve(state, owner, vec![1], false, 1);
    assert_eq!(first.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    assert!(state.hand_cards(owner).contains(&first_black));
    let (state, second) = resolve(state, owner, vec![1], false, 2);
    assert_eq!(second.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    assert!(state.hand_cards(owner).contains(&second_black));
    let (state, stopped) = resolve(state, owner, vec![1], false, 3);
    assert!(matches!(stopped.result, CommandResult::Accepted { .. }));
    assert!(stopped.choice.is_none());
    assert_eq!(state.hand_count(owner), hand_before + 2);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(red).unwrap().zone, discard);
}

#[test]
fn owner_may_stop_after_a_black_result() {
    let setup = setup();
    let hand_before = setup.state.hand_count(setup.owner);
    let (state, owner, first_black, second_black, _) = offer(setup);
    let (state, first) = resolve(state, owner, vec![1], false, 4);
    assert_eq!(first.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let (state, stopped) = resolve(state, owner, Vec::new(), true, 5);
    assert!(matches!(stopped.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert!(state.hand_cards(owner).contains(&first_black));
    assert!(!state.hand_cards(owner).contains(&second_black));
}
}

mod original_standard_new_jiangchi {
//! Golden coverage for 曹彰「将驰」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{
    query, sha_ignored_for_hand_limit, sha_response_cancelled_for, sha_use_limit_for, QueryContext,
};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const CAOZHANG: u32 = 1_812_908_078;
const NEW_JIANGCHI: u32 = 2_751_572_143;
const NEW_JIANGCHI3: u32 = 3_349_403_784;
const JIANGCHI2: u32 = 4_168_410_091;
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const STANDARD_SOURCE: &str = include_str!("../../../content/standard/standard_m1.skill");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(NEW_JIANGCHI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    Setup {
        state: tx.commit().state,
        owner,
        other,
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

fn open_branch_choice(setup: Setup, key: u64) -> (GameState, PipelineResult) {
    let (state, offered) = end_phase(setup.state, setup.owner, key);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, accepted) = resolve(state, setup.owner, vec![1], false, key + 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrawOrDiscardAddSkill
    );
    (state, accepted)
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

fn attack_range(state: &GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::AttackRange { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("attack-range query failed: {other:?}"),
    }
}

#[test]
fn packages_publish_caozhang_cross_package_skill_and_abi_contracts() {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(standard.manifest.rule_vm_abi_version, 261);
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert!(yijiang.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "standard"
            && dependency.version == "0.1.0"
    }));
    let character = yijiang
        .character_index
        .iter()
        .find(|character| character.id == CAOZHANG)
        .unwrap();
    assert_eq!(character.skills, [NEW_JIANGCHI]);
    assert!(standard
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NEW_JIANGCHI));
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JIANGCHI2));

    let compiled = content_compiler::compile_source_named("standard_m1", STANDARD_SOURCE).unwrap();
    let expected = compiled
        .modules
        .iter()
        .find(|module| module.name == "new_jiangchi_phase_draw_end")
        .unwrap();
    let actual = standard
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn optional_trigger_can_be_declined_without_opening_the_branch_choice() {
    let setup = setup(2_026_073_001);
    let hand = setup.state.hand_count(setup.owner);
    let (state, offered) = end_phase(setup.state, setup.owner, 1);
    assert!(offered.choice.is_some());
    let (state, declined) = resolve(state, setup.owner, Vec::new(), true, 2);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.hand_count(setup.owner), hand);
    assert!(!has_skill(&state, setup.owner, NEW_JIANGCHI3));
    assert!(!has_skill(&state, setup.owner, JIANGCHI2));
}

#[test]
fn draw_branch_grants_all_restrictions_then_cleans_up_at_turn_end() {
    let setup = setup(2_026_073_002);
    let hand = setup.state.hand_count(setup.owner);
    let (state, branch) = open_branch_choice(setup, 10);
    assert_eq!(
        branch
            .choice
            .unwrap()
            .options
            .into_iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![1]
    );
    let owner = state.turn.current;
    let (state, drawn) = resolve(state, owner, vec![1], false, 12);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 1);
    assert!(has_skill(&state, owner, NEW_JIANGCHI3));
    assert_eq!(sha_use_limit_for(&state, owner), 0);
    assert!(sha_response_cancelled_for(&state, owner));
    assert!(sha_ignored_for_hand_limit(&state, owner));

    let (state, _) = end_phase(state, owner, 13);
    assert_eq!(state.turn.phase, Phase::Discard);
    assert!(has_skill(&state, owner, NEW_JIANGCHI3));
    let (state, _) = end_phase(state, owner, 14);
    assert_eq!(state.turn.phase, Phase::End);
    assert!(has_skill(&state, owner, NEW_JIANGCHI3));
    let (state, ended) = end_phase(state, owner, 15);
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert!(!has_skill(&state, owner, NEW_JIANGCHI3));
}

#[test]
fn discard_branch_revalidates_cost_then_grants_and_cleans_play_helper() {
    let mut setup = setup(2_026_073_003);
    let cost = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Equip,
        99_301,
        CardKind::Weapon,
    );
    let forged = inject(
        &mut setup.state,
        setup.other,
        ZoneKind::Hand,
        99_302,
        CardKind::Other,
    );
    let owner = setup.owner;
    let (state, branch) = open_branch_choice(setup, 20);
    assert_eq!(
        branch
            .choice
            .unwrap()
            .options
            .into_iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    let (state, cost_choice) = resolve(state, owner, vec![2], false, 22);
    assert!(matches!(cost_choice.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrawOrDiscardAddSkill
    );
    assert_eq!(
        cost_choice.choice.as_ref().unwrap().options[0].token,
        cost.raw()
    );

    let before = canonical_state_hash(&state);
    let (same, rejected) = resolve(state, owner, vec![forged.raw()], false, 23);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
    let (state, paid) = resolve(same, owner, vec![cost.raw()], false, 24);
    assert!(matches!(paid.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, JIANGCHI2));
    assert_eq!(attack_range(&state, owner), i32::MAX);
    assert_eq!(sha_use_limit_for(&state, owner), 2);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(cost).unwrap().zone, discard);

    let (state, ended) = end_phase(state, owner, 25);
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Discard);
    assert!(!has_skill(&state, owner, JIANGCHI2));
}

#[test]
fn discard_branch_is_absent_without_a_legal_cost_and_forgery_is_atomic() {
    let setup = setup(2_026_073_004);
    let owner = setup.owner;
    let (state, branch) = open_branch_choice(setup, 30);
    assert_eq!(
        branch
            .choice
            .unwrap()
            .options
            .into_iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![1]
    );
    let before = canonical_state_hash(&state);
    let (same, rejected) = resolve(state, owner, vec![2], false, 32);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
}

mod original_standard_remaining {
//! Closure coverage for the last four legacy `standard` skills.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const GANGLIE_THREE: u32 = 3_884_116_323;
const ZHONGYI: u32 = 2_818_590_263;
const ZHONGYI2: u32 = 3_789_127_080;
const ZHANSHEN: u32 = 3_834_202_888;
const MASHU: u32 = 2;
const SHENJI: u32 = 3_515_984_232;
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const EXTENSION: &[u8] = include_bytes!("../../../extensions/standard/package.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let ally = players[1];
    let enemy = players[2];
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for (player, identity) in [
        (owner, Identity::Lord),
        (ally, Identity::Loyalist),
        (enemy, Identity::Rebel),
    ] {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.identity = identity;
        participant.hp = 10;
        participant.max_hp = 10;
    }
    (tx.commit().state, owner, ally, enemy)
}

fn add_skill(state: GameState, player: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    tx.commit().state
}

fn skill_id(state: &GameState, player: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == def)
        })
        .unwrap()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let (state, result) = apply_command(
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
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn put_judgment(state: &mut GameState, suit: Suit) {
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(990_001),
        kind: CardKind::Other,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(draw).unwrap().cards.push(card);
}

fn equip_first_hand_as_weapon(state: &mut GameState, player: PlayerId) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let equip = state.zones.find(ZoneKind::Equip, player).unwrap();
    let card = state.zones.get(hand).unwrap().cards[0];
    state
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .retain(|found| *found != card);
    state.zones.get_mut(equip).unwrap().cards.push(card);
    let card_state = state.cards.get_mut(card).unwrap();
    card_state.zone = equip;
    card_state.kind = CardKind::Weapon;
    card_state.weapon_range = 3;
    card
}

#[test]
fn package_and_drop_in_extension_publish_the_four_rewrites() {
    assert_eq!(PACKAGE, EXTENSION);
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    assert_eq!(package.skill_index.len(), 65);
    for def in [GANGLIE_THREE, ZHONGYI, ZHONGYI2, ZHANSHEN] {
        assert!(package
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == def));
    }
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == ZHONGYI2
            && trigger.filter == "owner_alive_source_ally_sha_damage_before"
    }));
}

#[test]
fn ganglie_three_targets_only_an_enemy_and_accepts_equipment_as_discard() {
    let (mut state, owner, ally, enemy) = setup(261_001);
    state = add_skill(state, owner, GANGLIE_THREE);
    let weapon = equip_first_hand_as_weapon(&mut state, enemy);
    let hand = state.hand_cards(enemy);
    put_judgment(&mut state, Suit::Spade);
    let enemy_hp = state.players.get(enemy).unwrap().hp;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: ally,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveGanglieThree
    );
    let state = resolve(state, owner, vec![enemy.raw()], false, 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GanglieSourceDiscardOrDamage
    );
    let state = resolve(state, enemy, vec![weapon.raw(), hand[0].raw()], false, 3);
    assert_eq!(state.players.get(enemy).unwrap().hp, enemy_hp);
    assert!(!state
        .zones
        .get(state.zones.find(ZoneKind::Equip, enemy).unwrap())
        .unwrap()
        .cards
        .contains(&weapon));
}

#[test]
fn zhongyi_round_lifecycle_and_zhanshen_awakening_resolve() {
    let (mut state, owner, ally, enemy) = setup(261_002);
    state = add_skill(state, owner, ZHONGYI);
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let card = state.hand_cards(owner)[0];
    let active = skill_id(&state, owner, ZHONGYI);
    let (state, activated) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::ActivateSkill {
                skill: active,
                targets: Vec::new(),
                card_targets: vec![card],
            },
        },
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let helper = skill_id(&state, owner, ZHONGYI2);
    assert_eq!(state.skill_pile_cards(owner, ZHONGYI2), [card]);

    let enemy_hp = state.players.get(enemy).unwrap().hp;
    let mut tx = Transaction::begin(state);
    tx.state_mut().resolution.current_damage_id = 900;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::CommitDamage {
            damage_id: 900,
            source: ally,
            target: enemy,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        });
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhongyi {
            player: owner,
            action: 1,
            helper_skill_def: SkillDefId::new(ZHONGYI2),
            card: CardId::NONE,
            skill: helper,
        }],
    )
    .unwrap();
    let tail = std::mem::take(&mut tx.state_mut().resolution.deferred_tail_effects);
    tx.state_mut().resolution.current_damage_id = 0;
    run_effects(&mut tx, tail).unwrap();
    assert_eq!(tx.state().players.get(enemy).unwrap().hp, enemy_hp - 2);
    tx.state_mut().turn.round += 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhongyi {
            player: owner,
            action: 2,
            helper_skill_def: SkillDefId::new(ZHONGYI2),
            card: CardId::NONE,
            skill: helper,
        }],
    )
    .unwrap();
    assert!(tx.state().skill_pile_cards(owner, ZHONGYI2).is_empty());
    assert!(tx.state().skills.get(helper).is_none());

    let mut state = tx.commit().state;
    state.players.get_mut(ally).unwrap().alive = false;
    let weapon = equip_first_hand_as_weapon(&mut state, owner);
    state.players.get_mut(owner).unwrap().hp = 8;
    state = add_skill(state, owner, ZHANSHEN);
    let max_hp = state.players.get(owner).unwrap().max_hp;
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
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp - 1);
    assert!(tx
        .state()
        .skills
        .get(skill_id(tx.state(), owner, MASHU))
        .is_some());
    assert!(tx
        .state()
        .skills
        .get(skill_id(tx.state(), owner, SHENJI))
        .is_some());
    assert!(!tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| tx
            .state()
            .skills
            .get(*skill)
            .is_some_and(|skill| skill.def.raw() == ZHANSHEN)));
    assert!(!tx
        .state()
        .zones
        .get(tx.state().zones.find(ZoneKind::Equip, owner).unwrap())
        .unwrap()
        .cards
        .contains(&weapon));
}
}

mod original_standard_rende {
//! Golden coverage for standard 仁德 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RENDE: u32 = 11;
const SOURCE: &str = include_str!("../../../content/standard/rende.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
}

fn setup() -> Setup {
    let package = compile_source_named("standard-rende-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_725_11, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(RENDE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.hp = 3;
    participant.max_hp = 4;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == RENDE)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    target: PlayerId,
    cards: Vec<CardId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: cards,
            },
        },
    )
}

#[test]
fn package_publishes_abi172_repeatable_hand_gift() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == RENDE)
        .unwrap();
    assert_eq!(active.uses_per_turn, None);
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|target| (target.min, target.max, target.allow_self)),
        Some((1, 1, false))
    );
    assert_eq!(
        active
            .selection
            .as_ref()
            .map(|selection| (&*selection.zone, selection.min, selection.max)),
        Some(("hand", 1, u8::MAX))
    );
    let compiled = compile_source_named("standard-rende", SOURCE).unwrap();
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));
}

#[test]
fn cumulative_second_card_recovers_once_after_gifts() {
    let setup = setup();
    let cards = setup.state.hand_cards(setup.owner);
    assert!(cards.len() >= 3);
    let (state, first) = activate(
        setup.state,
        setup.owner,
        setup.skill,
        setup.target,
        vec![cards[0]],
        1,
    );
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(setup.target).contains(&cards[0]));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 3);
    assert_eq!(
        state.skills.get(setup.skill).unwrap().play_phase_fire_count,
        1
    );

    let (mut state, second) = activate(
        state,
        setup.owner,
        setup.skill,
        setup.target,
        vec![cards[1]],
        2,
    );
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(setup.target).contains(&cards[1]));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
    assert_eq!(
        state.skills.get(setup.skill).unwrap().play_phase_fire_count,
        2
    );

    state.players.get_mut(setup.owner).unwrap().hp = 3;
    let (state, third) = activate(
        state,
        setup.owner,
        setup.skill,
        setup.target,
        vec![cards[2]],
        3,
    );
    assert!(matches!(third.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 3);
    assert_eq!(
        state.skills.get(setup.skill).unwrap().play_phase_fire_count,
        3
    );
}

#[test]
fn two_cards_at_once_recover_and_forged_inputs_are_rejected() {
    let setup = setup();
    let cards = setup.state.hand_cards(setup.owner);
    let foreign = setup.state.hand_cards(setup.target)[0];
    let (unchanged, forged) = activate(
        setup.state.clone(),
        setup.owner,
        setup.skill,
        setup.target,
        vec![foreign],
        4,
    );
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(unchanged, setup.state);

    let (state, accepted) = activate(
        setup.state,
        setup.owner,
        setup.skill,
        setup.target,
        vec![cards[0], cards[1]],
        5,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
    assert!(cards[..2]
        .iter()
        .all(|card| state.hand_cards(setup.target).contains(card)));
}
}

mod original_standard_retongji {
//! Golden coverage for Standard 同疾 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RETONGJI: u32 = 2_773_091_493;
const RE_YUANSHU: u32 = 3_366_235_136;
const SOURCE: &str = include_str!("../../../content/standard/retongji.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_000 + u32::from(rank)),
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

fn setup(with_owner_shan: bool) -> (GameState, PlayerId, PlayerId, PlayerId, CardId, CardId) {
    let package = compile_source_named("standard-retongji-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_725_62, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let source = players[0];
    let target = players[1];
    let owner = players[2];
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
        skill_def: SkillDefId::new(RETONGJI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().turn.current = source;
    tx.state_mut().turn.phase = Phase::Play;
    let cost = inject(&mut tx, target, CardKind::Other, Suit::Club, 2);
    if with_owner_shan {
        inject(&mut tx, owner, CardKind::Shan, Suit::Heart, 3);
    }
    let sha = inject(&mut tx, source, CardKind::Sha, Suit::Spade, 4);
    (tx.commit().state, source, target, owner, sha, cost)
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn play(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    sha: CardId,
) -> (GameState, PipelineResult) {
    submit(
        state,
        source,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        1,
    )
}

#[test]
fn package_closes_re_yuanshu_with_both_exact_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == RETONGJI)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_sha_target_can_redirect_to_owner"
    );
    assert!(!trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_YUANSHU)
        .unwrap();
    assert_eq!(character.skills, [3_817_957_374, RETONGJI]);
}

#[test]
fn target_may_discard_to_redirect_sha_to_owner_outside_source_range() {
    let (state, source, target, owner, sha, cost) = setup(true);
    assert!(!engine_core::query::player_in_attack_range(
        &state, source, owner
    ));
    assert!(engine_core::query::player_in_attack_range(
        &state, target, owner
    ));
    let target_hp = state.players.get(target).unwrap().hp;
    let owner_hp = state.players.get(owner).unwrap().hp;
    let (state, offered) = play(state, source, target, sha);
    let choice = offered.choice.unwrap();
    assert_eq!(choice.observer, target);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == cost.raw()));
    let (state, redirected) = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: vec![cost.raw()],
            cancelled: false,
        },
        2,
    );
    let response = redirected.choice.unwrap();
    assert_eq!(response.observer, owner);
    let shan = state
        .hand_cards(owner)
        .into_iter()
        .find(|card| state.cards.get(*card).unwrap().kind == CardKind::Shan)
        .unwrap();
    let (state, result) = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: response.choice_id,
            selections: vec![shan.raw()],
            cancelled: false,
        },
        3,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
    assert!(!state.hand_cards(target).contains(&cost));
}

#[test]
fn declining_cost_keeps_the_original_sha_target() {
    let (state, source, target, owner, sha, cost) = setup(false);
    let target_hp = state.players.get(target).unwrap().hp;
    let owner_hp = state.players.get(owner).unwrap().hp;
    let (state, offered) = play(state, source, target, sha);
    let choice = offered.choice.unwrap();
    let (state, result) = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: Vec::new(),
            cancelled: true,
        },
        2,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
    assert!(state.hand_cards(target).contains(&cost));
}
}

mod original_standard_rewangzun {
//! Golden coverage for Standard 妄尊 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{DomainEvent, PlayerId, SkillDefId, TurnPhaseTag};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, QueryContext};
use engine_core::state::{GameState, Identity};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const REWANGZUN: u32 = 3_817_957_374;
const REWANGZUN2: u32 = 2_737_456_606;
const SOURCE: &str = include_str!("../../../content/standard/rewangzun.skill");

fn setup() -> (GameState, PlayerId, PlayerId, PlayerId) {
    let package = compile_source_named("standard-rewangzun-test", SOURCE).unwrap();
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == REWANGZUN)
            .count(),
        2
    );
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    assert_eq!(
        state
            .content
            .defs_for_skill(SkillDefId::new(REWANGZUN))
            .len(),
        2
    );
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_725_61, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let lord = players[0];
    let first = players[1];
    let second = players[2];
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
    effects.extend([first, second].map(|player| EffectRequest::AddSkill {
        player,
        skill_def: SkillDefId::new(REWANGZUN),
    }));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(
        tx.state()
            .content
            .defs_for_skill(SkillDefId::new(REWANGZUN))
            .len(),
        2
    );
    assert_eq!(
        tx.state()
            .triggers
            .iter()
            .filter(|trigger| trigger.skill_def.raw() == REWANGZUN)
            .count(),
        4
    );
    tx.state_mut().players.get_mut(lord).unwrap().identity = Identity::Lord;
    tx.state_mut().players.get_mut(lord).unwrap().hp = 4;
    for owner in [first, second] {
        tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    }
    tx.state_mut().turn.current = lord;
    (tx.commit().state, lord, first, second)
}

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new()).unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
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

fn mark_count(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == REWANGZUN2)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn higher_hp_lord_grants_two_draws_per_owner_and_stacks_penalty() {
    let (state, lord, first, second) = setup();
    let first_before = state.hand_count(first);
    let second_before = state.hand_count(second);
    let base_limit = hand_limit(&state, lord);
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: lord,
            phase: TurnPhaseTag::Prepare,
        },
    );
    assert_eq!(state.hand_count(first), first_before + 2);
    assert_eq!(state.hand_count(second), second_before + 2);
    assert_eq!(mark_count(&state, lord), 2);
    assert_eq!(hand_limit(&state, lord), base_limit - 2);
    assert_eq!(
        state
            .players
            .get(lord)
            .unwrap()
            .skills
            .iter()
            .filter(|skill| {
                state
                    .skills
                    .get(**skill)
                    .is_some_and(|skill| skill.def.raw() == REWANGZUN2)
            })
            .count(),
        1
    );

    let state = dispatch(
        state,
        DomainEvent::PhaseEnded {
            current: lord,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        },
    );
    assert_eq!(mark_count(&state, lord), 0);
    assert_eq!(hand_limit(&state, lord), base_limit);
}

#[test]
fn non_lord_gets_no_penalty_and_equal_hp_does_not_trigger() {
    let (mut state, target, first, second) = setup();
    state.players.get_mut(target).unwrap().identity = Identity::Rebel;
    let first_before = state.hand_count(first);
    let second_before = state.hand_count(second);
    state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: target,
            phase: TurnPhaseTag::Prepare,
        },
    );
    assert_eq!(state.hand_count(first), first_before + 1);
    assert_eq!(state.hand_count(second), second_before + 1);
    assert_eq!(mark_count(&state, target), 0);

    state.players.get_mut(target).unwrap().hp = 3;
    let first_before = state.hand_count(first);
    let second_before = state.hand_count(second);
    state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: target,
            phase: TurnPhaseTag::Prepare,
        },
    );
    assert_eq!(state.hand_count(first), first_before);
    assert_eq!(state.hand_count(second), second_before);
}
}

mod original_standard_stdkuangfu {
//! Golden for standard 潘凤「狂斧」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const SOURCE: &str = include_str!("../../../content/standard/standard_m1.skill");
const STDKUANGFU: u32 = 4_213_747_779;

fn setup(seed: u64, owner_hp: i32, target_hp: i32) -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
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
    for player in [owner, target] {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDKUANGFU),
    });
    run_effects(&mut tx, effects).unwrap();
    for (player, hp) in [(owner, owner_hp), (target, target_hp)] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.max_hp = state.max_hp.max(hp.max(1) as u32);
        state.hp = hp;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.state().clone(), owner, target)
}

fn play_sha(state: GameState, owner: PlayerId, target: PlayerId, key: u64) -> GameState {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + key as u32),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipeline.result, CommandResult::Accepted { .. }),
        "play Sha rejected: {:?}",
        pipeline.result
    );
    assert!(pipeline.choice.is_none());
    state
}

fn allow_another_sha(state: GameState) -> GameState {
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    tx.state().clone()
}

fn enter_another_play_phase(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Discard.to_u8(),
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 0);
    tx.state().clone()
}

#[test]
fn committed_package_contains_locked_stdkuangfu_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let panfeng = package
        .character_index
        .iter()
        .find(|character| character.name_key == "std_panfeng")
        .expect("standard package must publish legacy std_panfeng");
    assert_eq!(panfeng.id, 4_123_573_742);
    assert_eq!(panfeng.sex, "male");
    assert_eq!(panfeng.group, "qun");
    assert_eq!(panfeng.max_hp, 4);
    assert_eq!(panfeng.skills, [STDKUANGFU]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDKUANGFU && skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDKUANGFU)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(
        trigger.filter,
        "owner_source_play_sha_other_alive_once_per_play_phase"
    );
    assert!(!trigger.optional);
}

#[test]
fn lower_hp_target_draws_two_only_once_in_each_play_phase() {
    let (state, owner, target) = setup(20_260_717_1, 4, 4);
    let state = play_sha(state, owner, target, 1);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.players.get(owner).unwrap().hp, 4);

    let state = play_sha(allow_another_sha(state), owner, target, 2);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.players.get(owner).unwrap().hp, 4);

    let state = play_sha(enter_another_play_phase(state, owner), owner, target, 3);
    assert_eq!(state.hand_count(owner), 4);
    assert_eq!(state.players.get(owner).unwrap().hp, 4);
}

#[test]
fn target_hp_not_lower_after_damage_makes_owner_lose_hp() {
    let (state, owner, target) = setup(20_260_717_2, 2, 4);
    let state = play_sha(state, owner, target, 4);
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert_eq!(state.players.get(owner).unwrap().hp, 1);
    assert_eq!(state.hand_count(owner), 0);
}
}

mod original_standard_stdshushen {
//! Golden coverage for standard 甘夫人「淑慎」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::canonical_state_hash;
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDSHUSHEN: u32 = 3_504_783_479;
const SOURCE: &str = include_str!("../../../content/standard/stdshushen.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let owner = tx.state().turn.current;
    let mut setup: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .flat_map(|player| tx.state().players.get(player).unwrap().skills.clone())
        .map(|skill| EffectRequest::RemoveSkill { skill })
        .collect();
    setup.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDSHUSHEN),
    });
    run_effects(&mut tx, setup).unwrap();
    let others = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let committed = tx.commit();
    (committed.state, owner, others)
}

fn empty_hand(state: GameState, player: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        }],
    )
    .unwrap();
    tx.commit().state
}

fn recover_and_pause(state: GameState, owner: PlayerId, amount: u32) -> GameState {
    let mut state = state;
    let max_hp = state.players.get(owner).unwrap().max_hp as i32;
    state.players.get_mut(owner).unwrap().hp = max_hp - amount as i32;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::RecoverHp {
                source: owner,
                player: owner,
                amount,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = tx.commit();
    assert_eq!(
        committed.state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawByHandCount
    );
    committed.state
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
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
fn standard_package_publishes_exact_abi50_module() {
    let package = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDSHUSHEN && skill.name == "stdshushen"));
    let compiled = content_compiler::compile_source_named("stdshushen", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn nonempty_target_draws_one_and_empty_target_draws_two() {
    let (state, owner, others) = started(2_026_071_901);
    let target = others[0];
    let before = state.hand_count(target);
    let state = recover_and_pause(state, owner, 1);
    let (state, pipeline) = resolve(state, owner, vec![target.raw()], false, 1);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(target), before + 1);
    assert!(!state.has_pending_choice());

    let (state, owner, others) = started(2_026_071_902);
    let target = others[0];
    let state = empty_hand(state, target);
    let state = recover_and_pause(state, owner, 1);
    let (state, pipeline) = resolve(state, owner, vec![target.raw()], false, 2);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(target), 2);
    assert!(!state.has_pending_choice());
}

#[test]
fn every_recovered_point_has_an_independent_cancellable_choice() {
    let (state, owner, others) = started(2_026_071_903);
    let empty = others[0];
    let nonempty = others[1];
    let state = empty_hand(state, empty);
    let nonempty_before = state.hand_count(nonempty);
    let state = recover_and_pause(state, owner, 2);
    assert_eq!(state.resolution.continuation_payload[3], 2);

    let (state, first) = resolve(state, owner, vec![empty.raw()], false, 3);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(empty), 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawByHandCount
    );
    assert_eq!(state.resolution.continuation_payload[3], 1);

    let (state, second) = resolve(state, owner, Vec::new(), true, 4);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(nonempty), nonempty_before);
    assert!(!state.has_pending_choice());
}

#[test]
fn cancelling_one_point_does_not_cancel_the_next_point() {
    let (state, owner, others) = started(2_026_071_904);
    let target = others[0];
    let before = state.hand_count(target);
    let state = recover_and_pause(state, owner, 2);
    let (state, first) = resolve(state, owner, Vec::new(), true, 5);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let (state, second) = resolve(state, owner, vec![target.raw()], false, 6);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(target), before + 1);
    assert!(!state.has_pending_choice());
}

#[test]
fn forged_self_target_is_rejected_atomically() {
    let (state, owner, _) = started(2_026_071_905);
    let state = recover_and_pause(state, owner, 1);
    let before = canonical_state_hash(&state);
    let (same, pipeline) = resolve(state, owner, vec![owner.raw()], false, 7);
    assert!(matches!(pipeline.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
    assert!(same.has_pending_choice());
}
}

mod original_standard_stdxiaoguo {
//! Golden coverage for Standard 骁果 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDXIAOGUO: u32 = 3_757_481_350;
const STD_YUEJIN: u32 = 3_041_549_837;
const SOURCE: &str = include_str!("../../../content/standard/stdxiaoguo.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(with_equipment: bool) -> (GameState, PlayerId, PlayerId, CardId, Option<CardId>) {
    let package = compile_source_named("standard-stdxiaoguo-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_725_53, "identity-standard").unwrap();
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDXIAOGUO),
    });
    run_effects(&mut tx, effects).unwrap();
    let basic = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 2);
    inject(&mut tx, owner, ZoneKind::Hand, CardKind::Other, 3);
    let equipment =
        with_equipment.then(|| inject(&mut tx, target, ZoneKind::Hand, CardKind::Armor, 4));
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = Phase::End;
    (tx.commit().state, owner, target, basic, equipment)
}

fn dispatch_end(state: GameState, target: PlayerId) -> (GameState, engine_api::ChoiceRequest) {
    let mut tx = Transaction::begin(state);
    let mut fires = TriggerFireRecord::new();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: target,
            phase: TurnPhaseTag::End,
        }],
        &mut fires,
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    (committed.state, committed.output.choice.unwrap())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    choice: engine_api::ChoiceRequest,
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
                choice_id: choice.choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_contains_complete_standard_yuejin() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDXIAOGUO)
        .unwrap();
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_phase_end_has_discardable_basic_hand"
    );
    assert!(!trigger.optional);
    let yuejin = package
        .character_index
        .iter()
        .find(|character| character.id == STD_YUEJIN)
        .unwrap();
    assert_eq!(yuejin.skills, [STDXIAOGUO]);
}

#[test]
fn basic_cost_then_target_discards_equipment_or_takes_damage() {
    let (state, owner, target, basic, equipment) = setup(true);
    let equipment = equipment.unwrap();
    let hp = state.players.get(target).unwrap().hp;
    let (state, cost) = dispatch_end(state, target);
    assert_eq!(cost.observer, owner);
    assert_eq!(cost.kind, ChoiceKind::SelectCards);
    assert!(cost
        .options
        .iter()
        .any(|option| option.token == basic.raw()));
    assert_eq!(cost.options.len(), 1);
    let (state, result) = resolve(state, owner, cost, vec![basic.raw()], false, 1);
    let consequence = result.choice.unwrap();
    assert_eq!(consequence.observer, target);
    assert!(consequence
        .options
        .iter()
        .any(|option| option.token == equipment.raw()));
    let (discarded, result) = resolve(
        state.clone(),
        target,
        consequence.clone(),
        vec![equipment.raw()],
        false,
        2,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(discarded.players.get(target).unwrap().hp, hp);

    let (damaged, result) = resolve(state, target, consequence, Vec::new(), true, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(damaged.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn declining_cost_or_missing_equipment_resolves_without_extra_choices() {
    let (state, owner, target, basic, _) = setup(false);
    let hp = state.players.get(target).unwrap().hp;
    let (state, cost) = dispatch_end(state, target);
    let (declined, result) = resolve(state.clone(), owner, cost.clone(), Vec::new(), true, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(declined.players.get(target).unwrap().hp, hp);
    assert!(declined.hand_cards(owner).contains(&basic));

    let (damaged, result) = resolve(state, owner, cost, vec![basic.raw()], false, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(damaged.players.get(target).unwrap().hp, hp - 1);
}
}

mod original_standard_tiandu {
//! Golden coverage for STANDARD「天妒」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TIANDU: u32 = 6;
const SOURCE: &str = include_str!("../../../content/standard/tiandu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn inject_discard(tx: &mut Transaction) -> CardId {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(24_101),
        kind: CardKind::Other,
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
        .push(card);
    card
}

#[test]
fn owner_may_gain_the_final_physical_judgment_card() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TIANDU)
        .unwrap();
    assert_eq!(trigger.event, "judgment_resolved");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let compiled = content_compiler::compile_source_named("tiandu-test", SOURCE).unwrap();
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_725, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TIANDU),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;

    let result_card = inject_discard(&mut tx);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::EmitJudgmentResolved {
                player: owner,
                result_card,
                suit: Suit::Spade,
                rank: 7,
                cause_card: CardId::NONE,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );

    let state = tx.state().clone();
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, accepted) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(state.cards.get(result_card).unwrap().zone, hand);
}
}

mod original_standard_tieji {
//! Golden coverage for Standard 铁骑 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TIEJI: u32 = 3_561_072_419;
const SOURCE: &str = include_str!("../../../content/standard/tieji.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

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
        def: CardDefId::new(97_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, judgment_suit: Suit) -> (GameState, PlayerId, PlayerId, CardId, CardId) {
    let package = compile_source_named("standard-tieji-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TIEJI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let shan = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Club,
        2,
    );
    inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        judgment_suit,
        7,
    );
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        8,
    );
    (tx.commit().state, owner, target, sha, shan)
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn play_and_accept(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    sha: CardId,
) -> (GameState, PipelineResult) {
    let (state, offered) = submit(
        state,
        owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        1,
    );
    let choice = offered.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    assert_eq!(choice.observer, owner);
    submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: vec![1],
            cancelled: false,
        },
        2,
    )
}

#[test]
fn package_closes_machao_with_exact_optional_sha_target_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TIEJI)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(trigger.filter, "owner_source_sha");
    assert!(trigger.optional);
    let machao = package
        .character_index
        .iter()
        .find(|character| character.name_key == "machao")
        .unwrap();
    assert!(machao.skills.contains(&TIEJI));
}

#[test]
fn red_judgment_blocks_shan_but_black_judgment_keeps_the_response_window() {
    let (state, owner, target, sha, shan) = setup(20_260_725_51, Suit::Heart);
    let hp = state.players.get(target).unwrap().hp;
    let (red, result) = play_and_accept(state, owner, target, sha);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(red.players.get(target).unwrap().hp, hp - 1);
    assert!(red.hand_cards(target).contains(&shan));

    let (state, owner, target, sha, shan) = setup(20_260_725_52, Suit::Spade);
    let hp = state.players.get(target).unwrap().hp;
    let (state, result) = play_and_accept(state, owner, target, sha);
    let response = result.choice.unwrap();
    assert_eq!(response.observer, target);
    assert!(response
        .options
        .iter()
        .any(|option| option.token == shan.raw()));
    let (state, result) = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: response.choice_id,
            selections: vec![shan.raw()],
            cancelled: false,
        },
        3,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp);
}
}

mod original_standard_tuxi {
//! Golden coverage for standard 突袭 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TUXI: u32 = 15;
const SOURCE: &str = include_str!("../../../content/standard/tuxi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: PlayerId,
    second: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = compile_source_named("standard-tuxi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TUXI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Judge;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == TUXI)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        first,
        second,
        skill,
    }
}

fn command(
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

fn open_optional(setup: &Setup) -> GameState {
    let (state, result) = command(setup.state.clone(), setup.owner, CommandKind::EndPhase, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    state
}

#[test]
fn package_publishes_abi173_draw_trigger_instead_of_active_placeholder() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    assert!(package
        .active_index
        .iter()
        .all(|active| active.skill_def_id != TUXI));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TUXI)
        .unwrap();
    assert_eq!(trigger.event, "phase_draw");
    assert_eq!(trigger.filter, "owner_is_target_other_hand_nonempty");
    assert!(trigger.optional);
    let compiled = compile_source_named("standard-tuxi", SOURCE).unwrap();
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));
}

#[test]
fn declining_tuxi_keeps_the_normal_two_card_draw() {
    let setup = setup(20_260_725_15);
    let before = setup.state.hand_count(setup.owner);
    let state = open_optional(&setup);
    let (state, result) = resolve(state, setup.owner, Vec::new(), true, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(state.hand_count(setup.owner), before + 2);
}

#[test]
fn two_targets_each_lose_one_hand_card_and_normal_draw_is_zero() {
    let setup = setup(20_260_725_16);
    let owner_before = setup.state.hand_count(setup.owner);
    let first_before = setup.state.hand_count(setup.first);
    let second_before = setup.state.hand_count(setup.second);
    let first_card = setup.state.hand_cards(setup.first)[0];
    let second_card = setup.state.hand_cards(setup.second)[0];

    let state = open_optional(&setup);
    let (state, accepted) = resolve(state, setup.owner, vec![1], false, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::TuxiGainOtherHandsReplaceDraw
    );

    let (state, targeted) = resolve(
        state,
        setup.owner,
        vec![setup.second.raw(), setup.first.raw()],
        false,
        3,
    );
    assert!(matches!(targeted.result, CommandResult::Accepted { .. }));
    let (state, first_gain) = resolve(state, setup.owner, vec![first_card.raw()], false, 4);
    assert!(matches!(first_gain.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::TuxiGainOtherHandsReplaceDraw
    );
    let (state, second_gain) = resolve(state, setup.owner, vec![second_card.raw()], false, 5);
    assert!(matches!(second_gain.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(state.hand_count(setup.owner), owner_before + 2);
    assert_eq!(state.hand_count(setup.first), first_before - 1);
    assert_eq!(state.hand_count(setup.second), second_before - 1);
    assert!(state.hand_cards(setup.owner).contains(&first_card));
    assert!(state.hand_cards(setup.owner).contains(&second_card));
    assert_eq!(
        state.skills.get(setup.skill).unwrap().def.raw(),
        TUXI,
        "the originating skill remains bound through both choices"
    );
}
}

mod original_standard_xf_yiji {
//! Golden coverage for Standard 伊籍 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const XF_YIJI: u32 = 4_192_370_611;
const JIJIE: u32 = 4_028_496_128;
const JIYUAN: u32 = 2_449_404_379;
const SOURCE: &str = include_str!("../../../content/standard/xinfu_jijie.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    jijie: SkillId,
}

fn setup() -> Setup {
    let package = compile_source_named("standard-xf-yiji-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_725_181, "identity-standard").unwrap();
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JIJIE),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JIYUAN),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let jijie = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == JIJIE)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        other,
        jijie,
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
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

#[test]
fn package_closes_xf_yiji_with_both_exact_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == XF_YIJI)
        .unwrap();
    assert_eq!(character.skills, [JIJIE, JIYUAN]);
    let jiyuan: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == JIYUAN)
        .collect();
    assert_eq!(jiyuan.len(), 2);
    assert!(jiyuan.iter().any(|trigger| {
        trigger.event == "cards_given"
            && trigger.filter == "owner_alive_gave_cards_to_other_alive"
            && trigger.optional
    }));
}

#[test]
fn jijie_reveals_exact_bottom_card_and_jiyuan_draws_for_recipient() {
    let setup = setup();
    let draw = setup
        .state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let bottom = setup.state.zones.get(draw).unwrap().cards[0];
    let before = setup.state.hand_count(setup.other);
    let (state, offered) = submit(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.jijie,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(offered.choice.is_some(), "{offered:?}");
    let choice = offered.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert!(state.cards.get(bottom).is_some_and(|card| card.zone
        == state
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .unwrap()));

    let (state, optional) = resolve(state, setup.owner, vec![setup.other.raw()], 2);
    assert_eq!(optional.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    assert!(state.hand_cards(setup.other).contains(&bottom));
    let (state, done) = resolve(state, setup.owner, vec![1], 3);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.other), before + 2);
}

#[test]
fn jiyuan_may_draw_for_any_other_dying_player() {
    let setup = setup();
    let before = setup.state.hand_count(setup.other);
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::EnteredDying {
                player: setup.other,
                source: PlayerId::NONE,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = tx.commit().state;
    let (state, done) = resolve(state, setup.owner, vec![1], 4);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.other), before + 1);
}
}

mod original_standard_yaowu {
//! Golden tests for standard 华雄「耀武」rewritten from legacy JavaScript.

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

const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const SOURCE: &str = include_str!("../../../content/standard/standard_m1.skill");
const HUAXIONG: u32 = 4_212_795_236;
const YAOWU: u32 = 3_872_761_084;

fn loaded(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let source = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let remove = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    for player in [source, target] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(YAOWU),
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    (tx, source, target)
}

fn inject_sha(tx: &mut Transaction, source: PlayerId, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(70_000 + suit as u32),
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

fn play(source: PlayerId, target: PlayerId, card: CardId, key: u64) -> Command {
    Command {
        actor: source,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::PlayCard {
            card,
            targets: vec![target],
            card_targets: vec![],
        },
    }
}

#[test]
fn committed_standard_package_contains_huaxiong_and_locked_yaowu() {
    let package = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == HUAXIONG)
        .unwrap();
    assert_eq!((character.initial_hp, character.max_hp), (0, 6));
    assert_eq!(character.skills, [YAOWU]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YAOWU && skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YAOWU)
        .unwrap();
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(trigger.filter, "owner_is_target_red_sha_source_alive");
}

#[test]
fn red_sha_forces_the_living_source_to_choose_before_damage() {
    let (mut tx, source, target) = loaded(20_260_716_4);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: source,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        }],
    )
    .unwrap();
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let sha = inject_sha(&mut tx, source, Suit::Diamond);
    let (state, pipeline) = apply_command(tx.state().clone(), play(source, target, sha, 1));
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    let choice = pipeline.choice.unwrap();
    assert_eq!(choice.observer, source);
    assert!(!choice.allow_cancel);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp);
}

#[test]
fn nonred_sha_does_not_trigger_standard_yaowu() {
    let (mut tx, source, target) = loaded(20_260_716_5);
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let target_hand = tx.state().hand_count(target);
    let sha = inject_sha(&mut tx, source, Suit::Club);
    let (state, pipeline) = apply_command(tx.state().clone(), play(source, target, sha, 2));
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(pipeline.choice.is_none());
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.hand_count(target), target_hand);
}
}

mod original_standard_yiji {
//! Golden coverage for standard 遗计 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YIJI: u32 = 26;
const SOURCE: &str = include_str!("../../../content/standard/yiji.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let package = compile_source_named("standard-yiji-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_725_26, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let other = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(YIJI),
    });
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        source,
        other,
    }
}

fn damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    (tx.commit().state, result)
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

fn choice_tokens(result: &PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .expect("pending choice snapshot")
        .options
        .iter()
        .map(|option| option.token)
        .collect()
}

#[test]
fn package_publishes_abi174_per_damage_point_distribution() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YIJI)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target_per_damage_point");
    assert!(trigger.optional);
    let compiled = compile_source_named("standard-yiji", SOURCE).unwrap();
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));
}

#[test]
fn two_damage_points_each_distribute_two_public_processing_cards() {
    let setup = setup();
    let draw = setup
        .state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_before = setup.state.zones.get(draw).unwrap().cards.len();
    let owner_before = setup.state.hand_count(setup.owner);
    let source_before = setup.state.hand_count(setup.source);
    let other_before = setup.state.hand_count(setup.other);
    let (state, damage_result) = damage(setup.state, setup.source, setup.owner, 2);
    assert_eq!(damage_result, Err(EffectError::ChoiceRequested));

    let (state, first_confirm) = resolve(state, setup.owner, vec![1], 1);
    assert_eq!(
        first_confirm.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );
    let (state, second_confirm) = resolve(state, setup.owner, vec![1], 2);
    assert_eq!(
        second_confirm.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let first_pair = choice_tokens(&second_confirm);
    assert_eq!(first_pair.len(), 2);
    let processing = state
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert!(first_pair.iter().all(|raw| {
        state
            .cards
            .get(engine_api::CardId::new(*raw))
            .is_some_and(|card| card.zone == processing && card.face_up)
    }));

    let (state, first_subset) = resolve(state, setup.owner, vec![first_pair[0]], 3);
    assert_eq!(
        first_subset.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    let (state, first_recipient) = resolve(state, setup.owner, vec![setup.source.raw()], 4);
    assert_eq!(
        first_recipient.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets,
        "the lone remaining card skips a redundant subset choice"
    );
    let (state, second_recipient) = resolve(state, setup.owner, vec![setup.other.raw()], 5);
    assert_eq!(
        second_recipient.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards,
        "the second damage point opens a fresh two-card distribution"
    );
    let second_pair = choice_tokens(&second_recipient);
    assert_eq!(second_pair.len(), 2);

    let (state, second_subset) = resolve(state, setup.owner, second_pair, 6);
    assert_eq!(
        second_subset.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    let (state, completed) = resolve(state, setup.owner, vec![setup.owner.raw()], 7);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.zones.get(draw).unwrap().cards.len(), draw_before - 4);
    assert_eq!(state.hand_count(setup.owner), owner_before + 2);
    assert_eq!(state.hand_count(setup.source), source_before + 1);
    assert_eq!(state.hand_count(setup.other), other_before + 1);
}
}
