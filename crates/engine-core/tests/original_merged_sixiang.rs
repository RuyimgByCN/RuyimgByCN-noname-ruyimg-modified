//! O2 全量合并批次 1（2026-10-14）：家族 sixiang 的 49 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_sixiang_oldlonghun {
//! Golden semantics for SIXIANG 牢神赵云「旧龙魂」。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, ContinuationKind, GameState, Phase, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDLONGHUN: u32 = 4_175_093_300;
const SUIT_OVERRIDE: u32 = 4_200_003_050;
const OLD_SHEN_ZHAOYUN: u32 = 3_575_824_543;
const OLDJUEJING: u32 = 2_511_966_880;
const OLDLONGHUN_SOURCE: &str =
    include_str!("../../../content/original/ported/sixiang/oldlonghun.skill");
const TEST_SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/sixiang/oldlonghun.skill"),
    "\nskill oldlonghun_test_suit(4200003050) locked {\n",
    "  modifier card_suit(club) override diamond;\n",
    "}\n"
);
const CHARACTERS: &str = include_str!("../../../content/original/ported/sixiang/characters.json");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    skill: SkillId,
    cards: [CardId; 4],
}

fn setup(owner_is_current: bool, hp: i32, suit_override: bool) -> Setup {
    let package = content_compiler::compile_source_named("oldlonghun-test", TEST_SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, OLDLONGHUN as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let current = tx.state().turn.current;
    let other = players
        .into_iter()
        .find(|player| *player != current)
        .unwrap();
    let (owner, other) = if owner_is_current {
        (current, other)
    } else {
        (other, current)
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
        skill_def: SkillDefId::new(OLDLONGHUN),
    });
    if suit_override {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SUIT_OVERRIDE),
        });
    }
    run_effects(&mut tx, effects).unwrap();

    tx.state_mut().turn.current = if owner_is_current { owner } else { other };
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = u32::try_from(hp.max(4)).unwrap();
    tx.state_mut().players.get_mut(other).unwrap().hp = 8;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 8;

    let cards: [CardId; 4] = tx.state().hand_cards(owner)[..4].try_into().unwrap();
    for card_id in cards {
        let card = tx.state_mut().cards.get_mut(card_id).unwrap();
        card.kind = CardKind::Tao;
        card.suit = Suit::Club;
    }
    for card_id in tx.state().hand_cards(other) {
        let card = tx.state_mut().cards.get_mut(card_id).unwrap();
        card.kind = CardKind::Tao;
        card.suit = Suit::Spade;
    }
    tx.state_mut().cards.get_mut(cards[2]).unwrap().kind = CardKind::Armor;

    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let expansion = tx.state_mut().zones.alloc(Zone {
        id: engine_api::ZoneId::NONE,
        kind: ZoneKind::Expansion,
        owner,
        cards: Vec::new(),
    });
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![cards[1]],
                to: expansion,
                from: Some(hand),
            },
            EffectRequest::MoveCards {
                cards: vec![cards[2]],
                to: equip,
                from: Some(hand),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLDLONGHUN)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        other,
        skill,
        cards,
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
fn source_contract_and_character_closure_are_exact() {
    let package =
        content_compiler::compile_source_named("oldlonghun-contract", OLDLONGHUN_SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 290);
    let rules: Vec<_> = package
        .view_as_index
        .iter()
        .filter(|rule| rule.skill_def_id == OLDLONGHUN)
        .collect();
    assert_eq!(rules.len(), 6);
    assert!(rules.iter().all(|rule| {
        rule.source_zone.as_deref() == Some("hand_expansion")
            && rule.owner_hp_min_one_cards
            && rule.multi_card_bonus.is_none()
    }));
    let signatures: Vec<_> = rules
        .iter()
        .map(|rule| {
            (
                rule.as_kind.as_str(),
                rule.window.as_str(),
                rule.filter.as_str(),
                rule.nature.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        signatures,
        [
            ("tao", "play_phase", "heart", None),
            ("tao", "response", "heart", None),
            ("sha", "play_phase", "diamond", Some("fire")),
            ("sha", "response", "diamond", Some("fire")),
            ("wuxie", "response", "spade", None),
            ("shan", "response", "club", None),
        ]
    );

    let characters: serde_json::Value = serde_json::from_str(CHARACTERS).unwrap();
    let character = characters
        .as_array()
        .unwrap()
        .iter()
        .find(|character| character["id"] == OLD_SHEN_ZHAOYUN)
        .unwrap();
    assert_eq!(character["name_key"], "old_shen_zhaoyun");
    assert_eq!(character["sex"], "male");
    assert_eq!(character["group"], "shen");
    assert_eq!(character["max_hp"], 2);
    assert_eq!(
        character["skills"],
        serde_json::json!([OLDJUEJING, OLDLONGHUN])
    );

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 562);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLDLONGHUN && skill.name == "oldlonghun"));
    let committed_rules: Vec<_> = committed
        .view_as_index
        .iter()
        .filter(|rule| rule.skill_def_id == OLDLONGHUN)
        .collect();
    assert_eq!(committed_rules, rules);
    let committed_character = committed
        .character_index
        .iter()
        .find(|character| character.id == OLD_SHEN_ZHAOYUN)
        .unwrap();
    assert_eq!(committed_character.max_hp, 2);
    assert_eq!(committed_character.skills, [OLDJUEJING, OLDLONGHUN]);
    assert!(!serde_json::to_string(&committed).unwrap().contains(".js"));
}

#[test]
fn play_uses_exact_hp_effective_suit_hand_and_expansion_without_bonus() {
    let Setup {
        state,
        owner,
        other: target,
        skill,
        cards,
    } = setup(true, 2, true);

    let (_, one_card) = submit(
        state.clone(),
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![cards[0]],
        },
        1,
    );
    assert!(matches!(one_card.result, CommandResult::Rejected { .. }));

    let (_, includes_equip) = submit(
        state.clone(),
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![cards[0], cards[2]],
        },
        2,
    );
    assert!(matches!(
        includes_equip.result,
        CommandResult::Rejected { .. }
    ));

    let hp = state.players.get(target).unwrap().hp;
    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![cards[0], cards[1]],
        },
        3,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(cards[0]).unwrap().zone, discard);
    assert_eq!(state.cards.get(cards[1]).unwrap().zone, discard);
    assert_eq!(
        state
            .zones
            .get(state.cards.get(cards[2]).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::Equip
    );
}

#[test]
fn response_requires_current_hp_sources_and_does_not_offer_equipment() {
    let Setup {
        mut state,
        owner,
        other: attacker,
        cards,
        ..
    } = setup(false, 3, false);
    let attack = state.hand_cards(attacker)[0];
    state.cards.get_mut(attack).unwrap().kind = CardKind::Sha;
    let hp = state.players.get(owner).unwrap().hp;
    let (state, response) = submit(
        state,
        attacker,
        CommandKind::PlayCard {
            card: attack,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        10,
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RespondToSha
    );
    let physical_tokens: std::collections::BTreeSet<_> =
        cards.iter().map(|card| card.raw()).collect();
    let conversion = response
        .choice
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .find(|token| !physical_tokens.contains(token))
        .unwrap();

    let (state, source_choice) = resolve(state, owner, vec![conversion], 11);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::MultiCardViewAsResponse
    );
    let choice = source_choice.choice.unwrap();
    assert_eq!((choice.min_count, choice.max_count), (3, 3));
    let offered: std::collections::BTreeSet<_> =
        choice.options.iter().map(|option| option.token).collect();
    let allowed = [cards[0].raw(), cards[1].raw(), cards[3].raw()]
        .into_iter()
        .collect();
    assert_eq!(offered, allowed);
    assert!(!offered.contains(&cards[2].raw()));

    let selected = vec![cards[0].raw(), cards[1].raw(), cards[3].raw()];
    let (state, completed) = resolve(state, owner, selected, 12);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
}
}

mod original_sixiang_stdanda {
//! Golden coverage for 标吴珂「谙达」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const STD_WUKE: u32 = 4_182_138_442;
const STDANDA: u32 = 3_646_320_211;
const STDZHUGUO: u32 = 3_147_233_152;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(with_source_card: bool) -> (Transaction, PlayerId, PlayerId, PlayerId, Option<CardId>) {
    let mut engine = Engine::new();
    engine.load_package(&package()).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 2_026_072_406,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let others: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let source = others[0];
    let target = others[1];
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
        skill_def: SkillDefId::new(STDANDA),
    });
    run_effects(&mut tx, effects).unwrap();
    clear_hand_and_equip(&mut tx, source);
    let card = with_source_card.then(|| inject(&mut tx, source));
    {
        let participant = tx.state_mut().players.get_mut(target).unwrap();
        participant.hp = 0;
        participant.dying = true;
    }
    (tx, owner, source, target, card)
}

fn clear_hand_and_equip(tx: &mut Transaction, player: PlayerId) {
    let mut cards = tx.state().hand_cards(player);
    if let Some(equip) = tx.state().zones.find(ZoneKind::Equip, player) {
        cards.extend(tx.state().zones.get(equip).unwrap().cards.iter().copied());
    }
    if !cards.is_empty() {
        run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).unwrap();
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(129_001),
        kind: CardKind::Sha,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn offer(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player: target,
            source,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, Vec<DomainEvent>) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, output) = apply_command(
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
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    (state, output.events)
}

fn stdanda_instance(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == STDANDA)
        .unwrap()
        .id
}

#[test]
fn package_publishes_abi131_skill_and_complete_std_wuke() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDANDA && skill.name == "stdanda" && !skill.locked));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_WUKE)
        .unwrap();
    assert_eq!(character.name_key, "std_wuke");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [STDANDA, STDZHUGUO]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDANDA)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        (
            "dying",
            "owner_alive_source_alive_event_player_dying_once_per_round",
            true
        )
    );
}

#[test]
fn accepted_source_gift_is_observed_by_source_and_does_not_recover() {
    let (mut tx, owner, source, target, card) = setup(true);
    assert_eq!(
        offer(&mut tx, source, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (offered, _) = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert!(offered.has_pending_choice());
    assert_eq!(offered.resolution.pending_observer, source.raw());
    let card = card.unwrap();
    let (given, events) = resolve(offered, source, vec![card.raw()], false, 2);
    assert_eq!(given.cards.get(card).unwrap().owner, target);
    assert_eq!(given.players.get(target).unwrap().hp, 0);
    assert!(given.players.get(target).unwrap().dying);
    assert!(events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsMoved { .. })));
    assert!(!events
        .iter()
        .any(|event| matches!(event, DomainEvent::HpRecovered { .. })));
    let skill = given.skills.get(stdanda_instance(&given, owner)).unwrap();
    assert_eq!(skill.round_fire_round, given.turn.round);

    let mut spent = Transaction::begin(given);
    assert!(offer(&mut spent, source, target).unwrap().is_empty());
    assert!(!spent.state().has_pending_choice());
}

#[test]
fn source_decline_recovers_dying_target_by_one() {
    let (mut tx, owner, source, target, _) = setup(true);
    assert_eq!(
        offer(&mut tx, source, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (offered, _) = resolve(tx.state().clone(), owner, vec![1], false, 3);
    let (recovered, events) = resolve(offered, source, Vec::new(), true, 4);
    assert_eq!(recovered.players.get(target).unwrap().hp, 1);
    assert!(!recovered.players.get(target).unwrap().dying);
    assert!(events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::HpRecovered {
                player,
                amount: 1,
                ..
            } if *player == target
        )
    }));
}

#[test]
fn source_without_cards_recovers_immediately_after_owner_accepts() {
    let (mut tx, owner, source, target, card) = setup(false);
    assert!(card.is_none());
    assert_eq!(
        offer(&mut tx, source, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (recovered, events) = resolve(tx.state().clone(), owner, vec![1], false, 5);
    assert!(!recovered.has_pending_choice());
    assert_eq!(recovered.players.get(target).unwrap().hp, 1);
    assert!(events
        .iter()
        .any(|event| matches!(event, DomainEvent::HpRecovered { player, amount: 1, .. } if *player == target)));
}

#[test]
fn missing_living_source_or_non_dying_target_does_not_offer() {
    let (mut tx, _, source, target, _) = setup(true);
    assert!(offer(&mut tx, PlayerId::NONE, target).unwrap().is_empty());
    tx.state_mut().players.get_mut(target).unwrap().hp = 1;
    tx.state_mut().players.get_mut(target).unwrap().dying = false;
    assert!(offer(&mut tx, source, target).unwrap().is_empty());
    assert!(!tx.state().has_pending_choice());
}
}

mod original_sixiang_stdbaobian {
//! Golden coverage for 标夏侯霸「豹变」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDBAOBIAN: u32 = 3_406_574_661;
const STD_XIAHOUBA: u32 = 2_845_782_138;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdbaobian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup(card_kind: Option<CardKind>) -> (Transaction, PlayerId, PlayerId, Option<CardId>) {
    let package = content_compiler::compile_source_named("stdbaobian-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_726, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        players
            .iter()
            .map(|player| EffectRequest::DiscardAllHandEquipJudge { player: *player }),
    );
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDBAOBIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let card = card_kind.map(|kind| {
        let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(194_001),
            kind,
            suit: Suit::Heart,
            rank: 7,
            weapon_range: 0,
            zone: hand,
            owner: target,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        card
    });
    (tx, owner, target, card)
}

fn phase_play(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::Play,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
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
        dispatch_for_events(&mut tx, &[phase_play(owner)], &mut TriggerFireRecord::new(),),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    assert_eq!(committed.output.choice.unwrap().kind, ChoiceKind::Confirm);
    resolve(committed.state, owner, vec![1], 1)
}

#[test]
fn package_publishes_abi194_skill_and_complete_std_xiahouba() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDBAOBIAN)
        .unwrap();
    assert_eq!(trigger.event, "phase_play");
    assert_eq!(
        trigger.filter,
        "owner_is_target_any_alive_self_discardable_hand"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_XIAHOUBA)
        .unwrap();
    assert_eq!(character.name_key, "std_xiahouba");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [STDBAOBIAN]);
}

#[test]
fn basic_discard_follows_hp_loss_with_distance_ignoring_virtual_sha() {
    let (tx, owner, target, card) = setup(Some(CardKind::Shan));
    let card = card.unwrap();
    let (state, target_choice) = offer(tx, owner);
    assert!(matches!(
        target_choice.result,
        CommandResult::Accepted { .. }
    ));
    let choice = target_choice.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(choice.observer, owner);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [target.raw()]
    );

    let (state, discard_choice) = resolve(state, owner, vec![target.raw()], 2);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    let choice = discard_choice.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.observer, target);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [card.raw()]
    );

    let (state, resolved) = resolve(state, target, vec![card.raw()], 3);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert_eq!(
        state.cards.get(card).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target: damaged,
            amount: 1,
            ..
        } if *source == owner && *damaged == target
    )));
}

#[test]
fn non_basic_discard_suppresses_sha_and_empty_table_suppresses_trigger() {
    let (tx, owner, target, card) = setup(Some(CardKind::Other));
    let card = card.unwrap();
    let (state, _) = offer(tx, owner);
    let (state, _) = resolve(state, owner, vec![target.raw()], 2);
    let (state, resolved) = resolve(state, target, vec![card.raw()], 3);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert_eq!(state.players.get(target).unwrap().hp, 4);
    assert!(!resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target: damaged,
            ..
        } if *source == owner && *damaged == target
    )));

    let (mut empty, owner, _, _) = setup(None);
    assert!(dispatch_for_events(
        &mut empty,
        &[phase_play(owner)],
        &mut TriggerFireRecord::new(),
    )
    .is_ok());
    assert_eq!(empty.state().resolution.pending_choice_id, 0);
}
}

mod original_sixiang_stdbingzheng {
//! Golden coverage for 标董允「秉正」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDBINGZHENG: u32 = 3_798_362_737;
const STD_DONGYUN: u32 = 2_711_327_184;
const STDDULIANG: u32 = 2_231_518_236;
const MOVE_PROBE: u32 = 4_200_000_127;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdbingzheng.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn setup(with_move_probe: bool) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdbingzheng-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    if with_move_probe {
        let probe = content_compiler::compile_source_named(
            "stdbingzheng-move-probe",
            r#"
skill move_probe(4200000127) locked {
  trigger cards_moved owner_from_field_empty {
    effect draw_cards(self, 1);
  }
}
"#,
        )
        .unwrap();
        std::sync::Arc::make_mut(&mut state.content).load_package(&probe).unwrap();
    }
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_724, "identity-standard").unwrap();
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
    effects.extend(
        players
            .iter()
            .map(|player| EffectRequest::DiscardAllHandEquipJudge { player: *player }),
    );
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDBINGZHENG),
    });
    if with_move_probe {
        effects.push(EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(MOVE_PROBE),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
    }
    tx.state_mut().turn.current = owner;
    (tx, owner, target, other)
}

fn inject(tx: &mut Transaction, owner: PlayerId, zone_kind: ZoneKind, def: u32) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: if zone_kind == ZoneKind::Equip {
            CardKind::Armor
        } else {
            CardKind::Other
        },
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
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
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn offer(mut tx: Transaction, owner: PlayerId) -> GameState {
    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new(),),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

#[test]
fn package_publishes_abi127_skill_and_complete_std_dongyun() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDBINGZHENG)
        .unwrap();
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(
        trigger.filter,
        "owner_is_target_any_alive_self_discardable_hand_equip"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_DONGYUN)
        .unwrap();
    assert_eq!(character.name_key, "std_dongyun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [STDBINGZHENG, STDDULIANG]);
}

#[test]
fn accepted_target_self_discards_then_owner_loses_hp_when_hand_differs() {
    let (mut tx, owner, target, _) = setup(false);
    let card = inject(&mut tx, target, ZoneKind::Hand, 127_001);
    tx.state_mut().players.get_mut(target).unwrap().hp = 2;
    let offered = offer(tx, owner);

    let (state, result) = resolve(offered, owner, vec![1], false, 1);
    let target_choice = result.choice.unwrap();
    assert_eq!(target_choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(target_choice.observer, owner);
    assert_eq!(
        target_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [target.raw()]
    );

    let (state, result) = resolve(state, owner, vec![target.raw()], false, 2);
    let card_choice = result.choice.unwrap();
    assert_eq!(card_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(card_choice.observer, target);
    assert_eq!(
        card_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [card.raw()]
    );

    let (resolved, result) = resolve(state, target, vec![card.raw()], false, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(resolved.players.get(owner).unwrap().hp, 3);
    assert_eq!(resolved.hand_count(target), 0);
    let discard = resolved
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(resolved.cards.get(card).unwrap().zone, discard);
}

#[test]
fn post_discard_check_waits_for_nested_movement_triggers() {
    let (mut tx, owner, target, _) = setup(true);
    let equipment = inject(&mut tx, target, ZoneKind::Equip, 127_010);
    tx.state_mut().players.get_mut(target).unwrap().hp = 1;
    let offered = offer(tx, owner);
    let (state, _) = resolve(offered, owner, vec![1], false, 10);
    let (state, _) = resolve(state, owner, vec![target.raw()], false, 11);
    let (resolved, result) = resolve(state, target, vec![equipment.raw()], false, 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(resolved.hand_count(target), 1);
    assert_eq!(resolved.players.get(owner).unwrap().hp, 4);
}

#[test]
fn decline_and_no_eligible_target_preserve_state_without_extra_choices() {
    let (mut tx, owner, target, _) = setup(false);
    let card = inject(&mut tx, target, ZoneKind::Hand, 127_020);
    let offered = offer(tx, owner);
    let (declined, result) = resolve(offered, owner, vec![0], false, 20);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(declined.cards.get(card).unwrap().owner, target);
    assert_eq!(declined.players.get(owner).unwrap().hp, 4);

    let (mut empty, empty_owner, _, _) = setup(false);
    assert_eq!(
        dispatch_for_events(
            &mut empty,
            &[phase_end(empty_owner)],
            &mut TriggerFireRecord::new(),
        )
        .unwrap(),
        Vec::new()
    );
    assert!(!empty.state().has_pending_choice());
}
}

mod original_sixiang_stdcanshi {
//! Golden coverage for 标孙皓「残蚀」rewritten from legacy JavaScript semantics.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Identity, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const STDCANSHI: u32 = 2_290_683_272;
const STDCANSHI_EFFECT: u32 = 2_335_115_700;
const STD_SUNHAO: u32 = 4_237_199_211;
const GUIMING: u32 = 3_828_399_214;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/sixiang/stdcanshi.skill"),
    "\nskill guiming(3828399214) locked lord_only {}\n"
);
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn character(id: u32, group: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("stdcanshi_{id}"),
        max_hp: 5,
        initial_hp: 0,
        skills: vec![],
        sex: "male".into(),
        group: group.into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

fn setup() -> (Transaction, PlayerId, Vec<PlayerId>) {
    let mut package = content_compiler::compile_source_named("stdcanshi-test", SOURCE).unwrap();
    package.character_index = vec![
        character(9_222_001, "wu"),
        character(9_222_002, "wu"),
        character(9_222_003, "wu"),
        character(9_222_004, "qun"),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_724),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 20_260_724,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let mut tx = Transaction::begin(engine.state().clone());
    let seats = tx.state().players.alive_by_seat();
    let owner = seats
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let others: Vec<_> = seats
        .iter()
        .copied()
        .filter(|player| *player != owner)
        .collect();
    for (player, character) in [owner]
        .into_iter()
        .chain(others.iter().copied())
        .zip(9_222_001..=9_222_004)
    {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .character_def = character;
    }

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
            skill_def: SkillDefId::new(STDCANSHI),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(GUIMING),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, others)
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_abi145_and_complete_std_sunhao() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package
        .manifest
        .dependencies
        .iter()
        .any(|dependency| { dependency.namespace == "official" && dependency.package_id == "sp" }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == STDCANSHI_EFFECT
            && modifier.target == "draw_count"
            && modifier.value_source.as_deref() == Some("alive_wounded_or_guiming_wu_count_min_one")
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDCANSHI_EFFECT
            && trigger.event == "card_targeted"
            && trigger.filter
                == "owner_source_sha_or_ordinary_trick_target_wounded_has_discardable_hand_equip"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_SUNHAO)
        .unwrap();
    assert_eq!(character.name_key, "std_sunhao");
    assert_eq!(character.max_hp, 5);
    assert_eq!(character.skills, [STDCANSHI, 2_265_775_915, GUIMING]);
}

#[test]
fn draw_phase_counts_wounded_and_guiming_wu_players_then_expires() {
    let (mut tx, owner, others) = setup();
    assert!(has_skill(tx.state(), owner, STDCANSHI_EFFECT));
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    tx.state_mut().players.get_mut(others[2]).unwrap().hp -= 1;
    assert_eq!(engine_core::query::draw_count_for(tx.state(), owner), 4);

    tx.state_mut().players.get_mut(owner).unwrap().identity = Identity::Rebel;
    assert_eq!(engine_core::query::draw_count_for(tx.state(), owner), 2);
    tx.state_mut().players.get_mut(owner).unwrap().identity = Identity::Lord;

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
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
    assert!(!has_skill(tx.state(), owner, STDCANSHI_EFFECT));
}

#[test]
fn targeting_a_wounded_player_with_sha_forces_one_owner_discard() {
    let (mut tx, owner, others) = setup();
    let target = others[0];
    let used = inject_hand(&mut tx, owner, CardKind::Sha, 145_001);
    let discarded = inject_hand(&mut tx, owner, CardKind::Other, 145_002);
    tx.state_mut().players.get_mut(target).unwrap().hp -= 1;

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: used,
            targets: vec![target],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let offered = tx.commit();
    let choice = offered.output.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == discarded.raw()));
    let choice_id = ChoiceId::new(offered.state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        offered.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![discarded.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.cards.get(discarded).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}
}

mod original_sixiang_stdcuorui {
//! Golden for 四象标牛金「挫锐」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const STDCUORUI: u32 = 2_683_321_015;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn isolated(player_count: u8, seed: u64) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDCUORUI && !skill.locked));

    let mut engine = Engine::new();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    engine.load_package(&package).unwrap();
    let players = engine.state().players.alive_by_seat();
    let owner = players[0];
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
        skill_def: SkillDefId::new(STDCUORUI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, players)
}

fn set_hand(tx: &mut Transaction, player: PlayerId, count: u32) {
    run_effects(
        tx,
        vec![EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        }],
    )
    .unwrap();
    if count > 0 {
        run_effects(tx, vec![EffectRequest::DrawCards { player, count }]).unwrap();
    }
    assert_eq!(tx.state().hand_count(player), count);
}

fn accept(state: &GameState, owner: PlayerId, key: u64) -> GameState {
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
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
fn game_start_refills_to_seven_and_does_not_offer_a_noop_choice() {
    let (mut tx, owner, _) = isolated(8, 20_260_716_01);
    set_hand(&mut tx, owner, 2);

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::GameStarted { player_count: 8 }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx.state(), owner, 1);
    assert_eq!(state.hand_count(owner), 7);

    let mut filled = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut filled,
        &[DomainEvent::GameStarted { player_count: 8 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!filled.state().has_pending_choice());
}

#[test]
fn only_the_killer_refills_to_the_post_death_alive_count() {
    let (mut tx, owner, players) = isolated(4, 20_260_716_02);
    set_hand(&mut tx, owner, 1);

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: players[1],
                killer: owner,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx.state(), owner, 2);
    assert_eq!(state.hand_count(owner), 3);

    let mut not_killer = Transaction::begin(state);
    set_hand(&mut not_killer, owner, 1);
    run_effects(
        &mut not_killer,
        vec![EffectRequest::KillPlayer {
            player: players[2],
            killer: players[3],
        }],
    )
    .unwrap();
    assert_eq!(not_killer.state().hand_count(owner), 1);
    assert!(!not_killer.state().has_pending_choice());
}
}

mod original_sixiang_stddechong {
//! Golden coverage for 得宠（sixiang 标毛皇后）赠牌 + 后续伤害（ABI 459）.
//!
//! 其他角色准备阶段：选任意张可弃置手牌/装备交给该角色；其下回合弃牌阶段开始时
//! 若手牌不小于体力值，拥有者可对其造成 1 点伤害。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDDECHONG: u32 = 2_987_704_809;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stddechong.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selection: u32,
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
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted"
    );
    next
}

fn fire_phase(
    mut tx: Transaction,
    player: PlayerId,
    phase: TurnPhaseTag,
) -> Result<Transaction, EffectError> {
    let fired = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: player,
            phase,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => Ok(tx),
            Err(EffectError::ChoiceRequested) => Ok(tx),
            Err(e) => Err(e),
        },
        Err(EffectError::ChoiceRequested) => Ok(tx),
        Err(e) => Err(e),
    }
}

#[test]
fn gives_cards_then_damages_at_target_discard_phase() {
    let package = content_compiler::compile_source_named("stddechong-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 459);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, _third) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDDECHONG),
    });
    run_effects(&mut tx, effects).unwrap();
    // Give the owner one discardable hand card.
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, owner)
        .expect("owner hand zone");
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_700),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Heart,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .expect("hand zone")
        .cards
        .push(card);
    let target_hand_before = tx.state().hand_count(target);
    let target_hp_before = tx.state().players.get(target).unwrap().hp;

    // Target's prepare phase: card gift choice.
    let tx = fire_phase(tx, target, TurnPhaseTag::Prepare).unwrap();
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "gift card choice must be pending"
    );
    state = drive_choice(state, owner, card.raw());
    assert_eq!(
        state.hand_count(target),
        target_hand_before + 1,
        "target gains the gifted card"
    );
    assert!(!state.hand_cards(owner).contains(&card));

    // Target's discard phase: owner optionally deals 1 damage.
    let mut tx = Transaction::begin(state);
    let tx = fire_phase(tx, target, TurnPhaseTag::Discard).unwrap();
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "optional damage confirm must be pending"
    );
    state = drive_choice(state, owner, 1);
    assert_eq!(
        state.players.get(target).unwrap().hp,
        target_hp_before - 1,
        "target takes 1 damage at its discard phase"
    );
    assert!(!state.has_pending_choice());
}
}

mod original_sixiang_stddianfeng {
//! Golden coverage for 标吕范「典封」and the typed last-equipment-loss filter.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{move_one_card, run_effects, EffectError};
use engine_core::state::{CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const STDDIANFENG: u32 = 3_414_143_685;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup(equipment_count: usize) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(STDDIANFENG as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: STDDIANFENG as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDDIANFENG),
    });
    run_effects(&mut tx, setup).unwrap();

    let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, target).unwrap();
    for card in tx
        .state()
        .hand_cards(target)
        .into_iter()
        .take(equipment_count)
        .collect::<Vec<_>>()
    {
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Weapon;
        move_one_card(&mut tx, card, hand, equip, target, true).unwrap();
    }
    (tx, owner, target)
}

fn move_equipment_to_discard(
    tx: &mut Transaction,
    target: PlayerId,
    count: usize,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    let equip = tx.state().zones.find(ZoneKind::Equip, target).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let cards = tx
        .state()
        .zones
        .get(equip)
        .unwrap()
        .cards
        .iter()
        .copied()
        .take(count)
        .collect();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(equip),
        }],
    )
}

#[test]
fn committed_package_declares_precise_optional_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDDIANFENG)
        .unwrap();
    assert_eq!(trigger.event, "cards_moved");
    assert_eq!(trigger.filter, "owner_alive_event_player_from_equip_empty");
    assert!(trigger.optional);
}

#[test]
fn losing_the_last_equipment_offers_owner_draw() {
    let (mut tx, owner, target) = setup(1);
    let hand_before = tx.state().hand_count(owner);
    assert_eq!(
        move_equipment_to_discard(&mut tx, target, 1).unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(tx.state().resolution.pending_observer, owner.raw());

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 1);
}

#[test]
fn remaining_equipment_and_unrelated_moves_do_not_trigger() {
    let (mut tx, owner, target) = setup(2);
    let hand_before = tx.state().hand_count(owner);
    move_equipment_to_discard(&mut tx, target, 1).unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(owner), hand_before);

    let (mut tx, owner, target) = setup(0);
    let hand_before = tx.state().hand_count(owner);
    let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state().hand_cards(target)[0];
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(owner), hand_before);
}
}

mod original_sixiang_stddingyi {
//! Golden coverage for 标孙邵「定仪」event-player-owned optional trigger semantics.

use content_schema::TriggerChoiceActor;
use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, move_one_card, run_effects, EffectError};
use engine_core::state::{CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDDINGYI: u32 = 2_968_274_563;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stddingyi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stddingyi-test", SOURCE).unwrap();
    let mut state = engine_core::state::GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_717, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, ending_player) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDDINGYI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, ending_player)
}

fn resolve(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selection: u32,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
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
    )
}

#[test]
fn committed_package_exposes_event_player_choice_ownership() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDDINGYI
            && trigger.event == "phase_end"
            && trigger.filter == "owner_alive_event_player_equip_empty"
            && trigger.optional
            && trigger.choice_actor == TriggerChoiceActor::EventPlayer
    }));
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDDINGYI && !skill.locked));
}

#[test]
fn ending_player_exclusively_confirms_and_draws() {
    let (mut tx, owner, ending_player) = setup();
    let owner_hand = tx.state().hand_count(owner);
    let ending_hand = tx.state().hand_count(ending_player);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[phase_end(ending_player)],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(tx.state().resolution.pending_observer, ending_player.raw());

    let (state, wrong_actor) = resolve(tx.state().clone(), owner, 1, 1);
    assert!(matches!(wrong_actor.result, CommandResult::Rejected { .. }));
    assert!(state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_hand);
    assert_eq!(state.hand_count(ending_player), ending_hand);

    let (state, accepted) = resolve(state, ending_player, 1, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_hand);
    assert_eq!(state.hand_count(ending_player), ending_hand + 1);
}

#[test]
fn equipment_blocks_the_offer_and_declining_draws_nothing() {
    let (mut equipped, _owner, ending_player) = setup();
    let hand = equipped
        .state()
        .zones
        .find(ZoneKind::Hand, ending_player)
        .unwrap();
    let equip = equipped
        .state()
        .zones
        .find(ZoneKind::Equip, ending_player)
        .unwrap();
    let card = equipped.state().hand_cards(ending_player)[0];
    equipped.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Weapon;
    move_one_card(&mut equipped, card, hand, equip, ending_player, true).unwrap();
    let effects = dispatch_for_events(
        &mut equipped,
        &[phase_end(ending_player)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!equipped.state().has_pending_choice());

    let (mut declined, _owner, ending_player) = setup();
    let hand_before = declined.state().hand_count(ending_player);
    assert_eq!(
        dispatch_for_events(
            &mut declined,
            &[phase_end(ending_player)],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = resolve(declined.state().clone(), ending_player, 0, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(ending_player), hand_before);
}
}

mod original_sixiang_stdduanbi {
//! Golden coverage for 标刘巴「锻币」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_LIUBA: u32 = 2_185_226_076;
const STDDUANBI: u32 = 3_811_377_323;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdduanbi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn setup(player_count: u8) -> (Transaction, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("stdduanbi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, player_count, 20_260_722, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(STDDUANBI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players)
}

fn inject(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
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

#[test]
fn package_publishes_exact_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDDUANBI && skill.name == "stdduanbi" && !skill.locked
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_LIUBA)
        .unwrap();
    assert_eq!(character.name_key, "std_liuba");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "shu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [STDDUANBI]);
}

#[test]
fn discards_all_hand_then_forces_two_distinct_targets_and_draws_in_seat_order() {
    let (mut tx, players) = setup(3);
    let [owner, second, third] = players.as_slice() else {
        panic!("expected three players");
    };
    let owner = *owner;
    let second = *second;
    let third = *third;
    let discarded = [
        inject(&mut tx, owner, 97_001),
        inject(&mut tx, owner, 97_002),
        inject(&mut tx, owner, 97_003),
    ];
    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new(),),
        Err(EffectError::ChoiceRequested)
    );

    let (state, target_choice) = resolve(tx.commit().state, owner, vec![1], 1);
    assert!(matches!(
        target_choice.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.hand_count(owner), 0);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(discarded
        .iter()
        .all(|card| state.cards.get(*card).unwrap().zone == discard));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let choice = target_choice.choice.unwrap();
    assert!(!choice.allow_cancel);
    assert_eq!((choice.min_count, choice.max_count), (2, 2));
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![owner.raw(), second.raw(), third.raw()]
    );

    let before = canonical_state_hash(&state);
    let (same, duplicate) = resolve(state, owner, vec![second.raw(), second.raw()], 2);
    assert!(matches!(duplicate.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let draw_pile = same.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let top_four: Vec<_> = same
        .zones
        .get(draw_pile)
        .unwrap()
        .cards
        .iter()
        .rev()
        .take(4)
        .copied()
        .collect();
    let third_hand = same.hand_count(third);
    let (state, drawn) = resolve(same, owner, vec![second.raw(), owner.raw()], 3);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(second), 2);
    assert_eq!(state.hand_count(third), third_hand);
    assert_eq!(state.cards.get(top_four[0]).unwrap().owner, owner);
    assert_eq!(state.cards.get(top_four[1]).unwrap().owner, owner);
    assert_eq!(state.cards.get(top_four[2]).unwrap().owner, second);
    assert_eq!(state.cards.get(top_four[3]).unwrap().owner, second);
}

#[test]
fn with_only_one_survivor_the_cost_is_paid_without_opening_target_choice() {
    let (mut tx, players) = setup(2);
    let owner = players[0];
    let other = players[1];
    tx.state_mut().players.get_mut(other).unwrap().alive = false;
    inject(&mut tx, owner, 97_011);
    inject(&mut tx, owner, 97_012);
    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new(),),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = resolve(tx.commit().state, owner, vec![1], 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 0);
    assert!(!state.has_pending_choice());
}
}

mod original_sixiang_stdduanbing {
//! Golden for 四象「短兵」固定攻击范围与每回合首次【杀】伤害语义。

use engine_api::{
    CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::intent::{resolve_damage_intent_with_kind, DamageIntent};
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const STDDUANBING: u32 = 2_170_694_873;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDDUANBING && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(STDDUANBING as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: STDDUANBING as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(STDDUANBING),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let state = tx.state_mut().players.get_mut(*player).unwrap();
        state.max_hp = 10;
        state.hp = 10;
    }
    (tx, players[0], players[1], players[2])
}

fn attack_range(tx: &Transaction, owner: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::AttackRange { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

fn resolved_sha_amount(tx: &Transaction, source: PlayerId, target: PlayerId) -> u32 {
    resolve_damage_intent_with_kind(
        tx.state(),
        DamageIntent {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            cancelled: false,
        },
        true,
    )
    .amount
}

fn deal_sha(tx: &mut Transaction, source: PlayerId, target: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::DealShaDamage {
            source,
            target,
            card: CardId::NONE,
            amount: 1,
            nature: 0,
            card_flags: 0,
        }],
    )
    .unwrap();
}

#[test]
fn attack_range_remains_one_even_with_a_long_weapon() {
    let (mut tx, owner, _, _) = isolated();
    let weapon = tx.state().hand_cards(owner)[0];
    {
        let card = tx.state_mut().cards.get_mut(weapon).unwrap();
        card.kind = CardKind::Weapon;
        card.weapon_range = 5;
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();

    assert_eq!(attack_range(&tx, owner), 1);
}

#[test]
fn only_the_owners_first_sha_damage_each_turn_gets_the_bonus() {
    let (mut tx, owner, target, other) = isolated();

    deal_sha(&mut tx, other, target);
    assert_eq!(resolved_sha_amount(&tx, owner, target), 2);

    deal_sha(&mut tx, owner, target);
    assert_eq!(resolved_sha_amount(&tx, owner, target), 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: other,
            phase: 1,
        }],
    )
    .unwrap();
    assert_eq!(resolved_sha_amount(&tx, owner, target), 2);
}

#[test]
fn non_sha_damage_neither_gains_nor_consumes_the_bonus() {
    let (mut tx, owner, target, _) = isolated();
    let ordinary = resolve_damage_intent_with_kind(
        tx.state(),
        DamageIntent {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            cancelled: false,
        },
        false,
    );
    assert_eq!(ordinary.amount, 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(resolved_sha_amount(&tx, owner, target), 2);
}
}

mod original_sixiang_stdfengpo_mayunlu {
//! Golden for SIXIANG Ma Yunlu's 标凤魄 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDFENGPO: u32 = 2_400_535_495;
const MASHU: u32 = 2;
const STD_MAYUNLU: u32 = 3_313_632_024;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdfengpo.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
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

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdfengpo-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDFENGPO),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.max_hp = 20;
        participant.hp = 15;
    }
    (tx.commit().state, owner, target, other)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
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

fn begin_sha_damage(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    flags: u8,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
            card: CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: flags,
        }],
    );
    (tx.commit().state, result)
}

fn accept(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    resolve(state, owner, vec![1], false, key)
}

#[test]
fn package_publishes_stdfengpo_and_std_mayunlu_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert_eq!(package.trigger_index.len(), 124);
    assert_eq!(package.modules.len(), 147);
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == STDFENGPO)
        .unwrap();
    assert_eq!(skill.name, "stdfengpo");
    assert!(!skill.locked);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDFENGPO)
        .unwrap();
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(
        trigger.filter,
        "owner_source_sha_damage_before_endpoint_discardable_hand_equip"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_MAYUNLU)
        .unwrap();
    assert_eq!(character.skills, [STDFENGPO, MASHU]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn diamond_from_either_endpoint_adds_one_after_discard() {
    for (seed, from_owner) in [(2_026_072_710, false), (2_026_072_711, true)] {
        let (mut state, owner, target, _) = setup(seed);
        let card_owner = if from_owner { owner } else { target };
        let card = inject(
            &mut state,
            card_owner,
            if from_owner {
                ZoneKind::Equip
            } else {
                ZoneKind::Hand
            },
            Suit::Diamond,
            230_000 + u32::from(from_owner),
        );
        let hp_before = state.players.get(target).unwrap().hp;
        let (state, result) = begin_sha_damage(state, owner, target, 0);
        assert_eq!(result, Err(EffectError::ChoiceRequested));
        let (state, accepted) = accept(state, owner, 1);
        let choice = accepted.choice.unwrap();
        assert_eq!(choice.kind, ChoiceKind::SelectCards);
        assert_eq!(choice.observer, owner);
        assert!(choice
            .options
            .iter()
            .any(|option| option.token == card.raw()));
        let (state, result) = resolve(state, owner, vec![card.raw()], false, 2);
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "{result:?}"
        );
        assert_eq!(state.players.get(target).unwrap().hp, hp_before - 2);
        assert_eq!(
            state.cards.get(card).unwrap().zone,
            state
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap()
        );
    }
}

#[test]
fn nonmatching_suit_discards_without_increasing_damage() {
    let (mut state, owner, target, _) = setup(2_026_072_712);
    let card = inject(&mut state, target, ZoneKind::Hand, Suit::Club, 230_010);
    let hp_before = state.players.get(target).unwrap().hp;
    let (state, result) = begin_sha_damage(state, owner, target, 0);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, _) = accept(state, owner, 3);
    let (state, result) = resolve(state, owner, vec![card.raw()], false, 4);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(target).unwrap().hp, hp_before - 1);
}

#[test]
fn chain_sha_is_included_and_empty_endpoints_do_not_prompt() {
    let (mut state, owner, target, _) = setup(2_026_072_713);
    let card = inject(&mut state, owner, ZoneKind::Hand, Suit::Diamond, 230_020);
    let hp_before = state.players.get(target).unwrap().hp;
    let (state, result) = begin_sha_damage(
        state,
        owner,
        target,
        rule_api::damage_card_flag::CHAIN_PROPAGATED,
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, _) = accept(state, owner, 5);
    let (state, result) = resolve(state, owner, vec![card.raw()], false, 6);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(target).unwrap().hp, hp_before - 2);

    let (state, owner, target, _) = setup(2_026_072_714);
    let hp_before = state.players.get(target).unwrap().hp;
    let (state, result) = begin_sha_damage(state, owner, target, 0);
    assert!(result.is_ok());
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::None);
    assert_eq!(state.players.get(target).unwrap().hp, hp_before - 1);
}
}

mod original_sixiang_stdfenxun {
//! Golden coverage for 标丁奉「奋迅」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, RejectReason, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::query::card_target_within_distance;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::TriggerFireRecord;
use rule_api::EffectRequest;

const STD_DINGFENG: u32 = 3_454_509_574;
const STDDUANBING: u32 = 2_170_694_873;
const STDFENXUN: u32 = 2_579_637_455;
const STDFENXUN_EFFECT: u32 = 3_372_226_216;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdfenxun.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    bound: PlayerId,
    far: PlayerId,
    skill: engine_api::SkillId,
    armor: CardId,
    weapon: CardId,
    sha: CardId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
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

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
}

fn setup(include_armor: bool) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 5, 2_026_072_407, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let owner = tx.state().turn.current;
    let players = tx.state().players.alive_by_seat();
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDFENXUN),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let armor = if include_armor {
        inject(
            tx.state_mut(),
            owner,
            130_001,
            CardKind::Armor,
            Suit::Heart,
            2,
        )
    } else {
        CardId::NONE
    };
    let weapon = inject(
        tx.state_mut(),
        owner,
        130_002,
        CardKind::Weapon,
        Suit::Club,
        5,
    );
    let sha = inject(
        tx.state_mut(),
        owner,
        130_003,
        CardKind::Sha,
        Suit::Spade,
        7,
    );
    let distant: Vec<_> = players
        .into_iter()
        .filter(|player| {
            *player != owner && !card_target_within_distance(tx.state(), owner, sha, *player, 1)
        })
        .collect();
    assert!(distant.len() >= 2);
    let skill = skill_id(tx.state(), owner, STDFENXUN).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        bound: distant[0],
        far: distant[1],
        skill,
        armor,
        weapon,
        sha,
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
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
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

fn end_owner_turn(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
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
    tx.commit().state
}

#[test]
fn package_publishes_abi131_skills_and_complete_std_dingfeng() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == STDFENXUN)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        (
            active.targets.as_ref().unwrap().min,
            active.targets.as_ref().unwrap().max
        ),
        (1, 1)
    );
    assert!(!active.targets.as_ref().unwrap().allow_self);
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(selection.zone, "hand_equip");
    assert_eq!((selection.min, selection.max), (1, 1));
    assert_eq!(selection.filter.as_deref(), Some("card_armor"));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == STDFENXUN_EFFECT
            && modifier.target == "card_distance_unlimited_for_skill_bound_target"
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDFENXUN_EFFECT
            && trigger.event == "turn_ended"
            && trigger.filter == "owner_is_target"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_DINGFENG)
        .unwrap();
    assert_eq!(character.name_key, "std_dingfeng");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [STDDUANBING, STDFENXUN]);

    let compiled = content_compiler::compile_source_named("stdfenxun", SOURCE).unwrap();
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
fn armor_cost_binds_only_the_selected_target_and_expires_at_turn_end() {
    let setup = setup(true);
    let (offered, activation) = command(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.bound],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(activation.result, CommandResult::Accepted { .. }));
    assert_eq!(
        offered.resolution.continuation_kind,
        ContinuationKind::ActiveSkillSelection
    );
    let options: Vec<_> = activation
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(options, [setup.armor.raw()]);

    let before_forgery = canonical_state_hash(&offered);
    let (unchanged, forged) = resolve(offered.clone(), setup.owner, vec![setup.weapon.raw()], 2);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&unchanged), before_forgery);

    let (resolved, result) = resolve(offered, setup.owner, vec![setup.armor.raw()], 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!resolved.has_pending_choice());
    let discard = resolved
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(resolved.cards.get(setup.armor).unwrap().zone, discard);
    let helper = skill_id(&resolved, setup.owner, STDFENXUN_EFFECT).unwrap();
    let mut bound_state = b"BND1".to_vec();
    bound_state.extend_from_slice(&setup.bound.raw().to_le_bytes());
    assert_eq!(resolved.skills.get(helper).unwrap().state_blob, bound_state);
    assert!(card_target_within_distance(
        &resolved,
        setup.owner,
        setup.sha,
        setup.bound,
        1
    ));
    assert!(!card_target_within_distance(
        &resolved,
        setup.owner,
        setup.sha,
        setup.far,
        1
    ));

    let (_, repeated) = command(
        resolved.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.far],
            card_targets: vec![setup.weapon],
        },
        4,
    );
    assert!(matches!(
        repeated.result,
        CommandResult::Rejected {
            reason: RejectReason::IllegalAction { ref code }
        } if code == "active skill use limit reached"
    ));

    let expired = end_owner_turn(resolved, setup.owner);
    assert!(skill_id(&expired, setup.owner, STDFENXUN_EFFECT).is_none());
    assert!(!card_target_within_distance(
        &expired,
        setup.owner,
        setup.sha,
        setup.bound,
        1
    ));
}

#[test]
fn self_target_and_activation_without_armor_are_rejected_atomically() {
    let with_armor = setup(true);
    let before_self = canonical_state_hash(&with_armor.state);
    let (same, self_target) = command(
        with_armor.state,
        with_armor.owner,
        CommandKind::ActivateSkill {
            skill: with_armor.skill,
            targets: vec![with_armor.owner],
            card_targets: vec![with_armor.armor],
        },
        10,
    );
    assert!(matches!(self_target.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before_self);

    let without_armor = setup(false);
    let before_cost = canonical_state_hash(&without_armor.state);
    let (same, no_cost) = command(
        without_armor.state,
        without_armor.owner,
        CommandKind::ActivateSkill {
            skill: without_armor.skill,
            targets: vec![without_armor.bound],
            card_targets: Vec::new(),
        },
        11,
    );
    assert!(matches!(no_cost.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before_cost);
}
}

mod original_sixiang_stdfenyin {
//! Golden coverage for 标留赞「奋音」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_LIUZAN: u32 = 4_100_887_402;
const STDFENYIN: u32 = 2_225_023_788;
const STDFENYIN_DISCARD: u32 = 3_965_200_877;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdfenyin.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
}

fn setup(seed: u64) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
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
        skill_def: SkillDefId::new(STDFENYIN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Judge;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
    }
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
    owner: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        owner,
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
    let choice = offered.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    resolve(state, owner, vec![u32::from(accept)], 2)
}

fn inject_event_card(state: &mut GameState, def: u32, suit: Suit, rank: u8) -> CardId {
    let zone = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn dispatch_use(
    state: GameState,
    player: PlayerId,
    card: CardId,
) -> (GameState, Option<engine_api::ChoiceRequest>) {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player,
            card,
            targets: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let result = run_effects(&mut tx, effects);
    assert!(result.is_ok() || result == Err(EffectError::ChoiceRequested));
    let committed = tx.commit();
    (committed.state, committed.output.choice)
}

fn end_owner_turn(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
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
    tx.commit().state
}

#[test]
fn package_publishes_abi131_skills_and_complete_std_liuzan() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let draw = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDFENYIN && trigger.event == "phase_draw")
        .unwrap();
    assert!(draw.optional);
    let discard = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDFENYIN_DISCARD && trigger.event == "card_used")
        .unwrap();
    assert_eq!(
        discard.filter,
        "owner_source_repeats_color_has_discardable_hand_equip"
    );
    assert!(!discard.optional);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDFENYIN_DISCARD
            && trigger.event == "turn_ended"
            && trigger.filter == "owner_is_target"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_LIUZAN)
        .unwrap();
    assert_eq!(character.name_key, "std_liuzan");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [STDFENYIN]);

    let compiled = content_compiler::compile_source_named("stdfenyin", SOURCE).unwrap();
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
fn accepting_draws_two_extra_then_repeated_color_forces_one_discard() {
    let setup = setup(2_026_072_408);
    let before = setup.state.hand_count(setup.owner);
    let (mut state, accepted) = enter_draw(setup.state, setup.owner, true);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), before + 4);
    assert!(skill_id(&state, setup.owner, STDFENYIN_DISCARD).is_some());
    state.turn.phase = Phase::Play;

    let red_one = inject_event_card(&mut state, 131_001, Suit::Heart, 2);
    let red_two = inject_event_card(&mut state, 131_002, Suit::Diamond, 3);
    let black_one = inject_event_card(&mut state, 131_003, Suit::Spade, 4);
    let black_two = inject_event_card(&mut state, 131_004, Suit::Club, 5);
    let colorless_one = inject_event_card(&mut state, 131_005, Suit::None, 6);
    let colorless_two = inject_event_card(&mut state, 131_006, Suit::None, 7);

    let (state_after_first, choice) = dispatch_use(state, setup.owner, red_one);
    assert!(choice.is_none());
    let (pending, choice) = dispatch_use(state_after_first, setup.owner, red_two);
    assert_eq!(pending.resolution.pending_observer, setup.owner.raw());
    let choice = choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.min_count, 1);
    assert_eq!(choice.max_count, 1);
    let cost = CardId::new(choice.options[0].token);
    let hand_before = pending.hand_count(setup.owner);
    let (resolved, result) = resolve(pending, setup.owner, vec![cost.raw()], 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(resolved.hand_count(setup.owner), hand_before - 1);
    assert!(!resolved.has_pending_choice());

    let (state_after_black, choice) = dispatch_use(resolved, setup.owner, black_one);
    assert!(choice.is_none());
    let (pending, choice) = dispatch_use(state_after_black, setup.owner, black_two);
    let cost = choice.unwrap().options[0].token;
    let (resolved, result) = resolve(pending, setup.owner, vec![cost], 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    let (state_after_colorless, choice) = dispatch_use(resolved, setup.owner, colorless_one);
    assert!(choice.is_none());
    let (state, choice) = dispatch_use(state_after_colorless, setup.owner, colorless_two);
    assert!(choice.is_none());

    let expired = end_owner_turn(state, setup.owner);
    assert!(skill_id(&expired, setup.owner, STDFENYIN_DISCARD).is_none());
}

#[test]
fn declining_draws_normally_and_outside_owner_turn_never_discards() {
    let declined = setup(2_026_072_409);
    let before = declined.state.hand_count(declined.owner);
    let (state, result) = enter_draw(declined.state, declined.owner, false);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(declined.owner), before + 2);
    assert!(skill_id(&state, declined.owner, STDFENYIN_DISCARD).is_none());

    let accepted = setup(2_026_072_410);
    let (mut state, result) = enter_draw(accepted.state, accepted.owner, true);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != accepted.owner)
        .unwrap();
    state.turn.current = other;
    state.turn.phase = Phase::Play;
    let red_one = inject_event_card(&mut state, 131_011, Suit::Heart, 8);
    let red_two = inject_event_card(&mut state, 131_012, Suit::Diamond, 9);
    let (state, choice) = dispatch_use(state, accepted.owner, red_one);
    assert!(choice.is_none());
    let (_, choice) = dispatch_use(state, accepted.owner, red_two);
    assert!(choice.is_none());
}
}

mod original_sixiang_stdgaobian {
//! Golden coverage for 高变（sixiang 标王沈，ABI 468）.

use engine_api::{
    CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDGAOBIAN: u32 = 4_024_217_730;
const STDANRAN: u32 = 2_536_614_283;
const ANRAN_MARK: u32 = 221_181_016;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdgaobian.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
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
                cancelled: false,
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

fn fire_end(tx: Transaction, current: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::End,
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

fn setup(tx: &mut Transaction, owner: PlayerId) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDGAOBIAN),
    });
    run_effects(tx, effects).unwrap();
}

#[test]
fn sha_branch_uses_virtual_sha() {
    let package = content_compiler::compile_source_named("stdgaobian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 468);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, chooser, victim) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner);
    // Empty the victim's hand so the virtual Sha cannot be dodged.
    let victim_hand = tx.state().zones.find(ZoneKind::Hand, victim).unwrap();
    let victim_cards: Vec<CardId> = tx.state().zones.get(victim_hand).unwrap().cards.clone();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        victim_cards
            .into_iter()
            .map(|card| EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(victim_hand),
            })
            .collect(),
    )
    .unwrap();
    // The chooser takes 1 damage this turn (exactly one damaged player).
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: chooser,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert!(
        tx.state().turn.player_received_damage(chooser),
        "chooser recorded as damaged this turn"
    );
    assert_eq!(tx.state().turn.damaged_player_count(), 1);
    let victim_hp_before = tx.state().players.get(victim).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = chooser;
    let tx = fire_end(tx, chooser);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "branch choice pending");
    state = drive_choice(state, chooser, vec![0]); // use a Sha
    assert!(state.has_pending_choice(), "Sha target choice pending");
    state = drive_choice(state, chooser, vec![victim.raw()]);

    // The virtual Sha hits the victim (no 闪 in hand).
    assert_eq!(
        state.players.get(victim).unwrap().hp,
        victim_hp_before - 1,
        "virtual Sha dealt 1 damage"
    );
}

#[test]
fn reset_branch_clears_anran_counter() {
    let package = content_compiler::compile_source_named("stdgaobian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, chooser, _victim) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(STDANRAN),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: engine_api::MarkDefId::new(ANRAN_MARK),
                delta: 2,
            },
            EffectRequest::DealDamage {
                source: owner,
                target: chooser,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            },
        ],
    )
    .unwrap();

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = chooser;
    let tx = fire_end(tx, chooser);
    let mut state = tx.commit().state;
    state = drive_choice(state, chooser, vec![1]); // reset Anran

    let owner_state = state.players.get(owner).unwrap();
    assert!(
        !owner_state
            .marks
            .iter()
            .any(|(mark, count)| *mark == ANRAN_MARK && *count > 0),
        "Anran counter cleared: {:?}",
        owner_state.marks
    );
}

#[test]
fn no_choice_when_two_players_damaged() {
    let package = content_compiler::compile_source_named("stdgaobian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, chooser, victim) = (seats[0], seats[1], seats[2]);
    setup(&mut tx, owner);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::DealDamage {
                source: owner,
                target: chooser,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            },
            EffectRequest::DealDamage {
                source: owner,
                target: victim,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().turn.damaged_player_count(), 2);

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = chooser;
    let tx = fire_end(tx, chooser);
    let state = tx.commit().state;
    assert!(
        !state.has_pending_choice(),
        "no choice when two players were damaged"
    );
}
}

mod original_sixiang_stdhuoshui {
//! Golden for 四象「祸水」按存活拥有者叠加的判定区受伤光环。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::damage_taken_amount_for;
use engine_core::state::ZoneKind;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const STDHUOSHUI: u32 = 3_517_180_722;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDHUOSHUI && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(STDHUOSHUI as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: STDHUOSHUI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(STDHUOSHUI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players[0], players[1], players[2])
}

fn add_judge_card(tx: &mut Transaction, player: PlayerId) {
    let card = tx.state().hand_cards(player)[0];
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, player).unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: judge,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn amount(tx: &Transaction, source: PlayerId, target: PlayerId) -> u32 {
    damage_taken_amount_for(tx.state(), source, target, damage_nature::NORMAL, 1)
}

#[test]
fn judge_zone_condition_applies_to_other_targets_and_sourceless_damage() {
    let (mut tx, owner, target, source) = isolated();
    assert_eq!(amount(&tx, source, target), 1);

    add_judge_card(&mut tx, target);
    assert_eq!(amount(&tx, source, target), 2);
    assert_eq!(amount(&tx, PlayerId::NONE, target), 2);

    add_judge_card(&mut tx, owner);
    assert_eq!(amount(&tx, source, owner), 1);
}

#[test]
fn living_owners_stack_and_dead_owners_stop_contributing() {
    let (mut tx, first_owner, target, second_owner) = isolated();
    add_judge_card(&mut tx, target);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: second_owner,
            skill_def: SkillDefId::new(STDHUOSHUI),
        }],
    )
    .unwrap();

    assert_eq!(amount(&tx, PlayerId::NONE, target), 3);
    tx.state_mut().players.get_mut(second_owner).unwrap().alive = false;
    assert_eq!(amount(&tx, PlayerId::NONE, target), 2);
    tx.state_mut().players.get_mut(first_owner).unwrap().alive = false;
    assert_eq!(amount(&tx, PlayerId::NONE, target), 1);
}

#[test]
fn aura_is_in_the_ordinary_incoming_damage_stage() {
    let (mut tx, _, target, source) = isolated();
    add_judge_card(&mut tx, target);

    assert_eq!(amount(&tx, source, target), 2);
    assert_eq!(
        engine_core::query::damage_taken_final_amount_for(
            tx.state(),
            target,
            damage_nature::NORMAL,
            1,
        ),
        1
    );
}
}

mod original_sixiang_stdjishe_stdwudu {
//! Golden for 标岑昏「极奢」「无度」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::hash::canonical_state_hash;
use engine_core::query::{query, QueryContext};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const STDJISHE: u32 = 2_148_577_926;
const STDWUDU: u32 = 4_151_205_179;
const STD_CENHUN: u32 = 3_739_444_871;
const JISHE_SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdjishe.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    source: PlayerId,
    jishe: engine_api::SkillId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(empty_target_hand: bool, seed: u64) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let source = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    if empty_target_hand {
        effects.push(EffectRequest::DiscardFromHand {
            player: target,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDJISHE),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDWUDU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.max_hp = 3;
    participant.hp = 3;
    let jishe = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == STDJISHE)
        .unwrap()
        .id;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        source,
        jishe,
    }
}

fn hand_limit(state: &GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::HandLimit { player },
        &mut QueryContext::default(),
    ) {
        QueryResponse::Int(limit) => limit,
        response => panic!("unexpected hand-limit response: {response:?}"),
    }
}

fn activate_jishe(setup: &Setup, state: GameState, key: u64) -> (GameState, CommandResult) {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.jishe,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    (state, result.result)
}

fn resolve_optional(state: GameState, owner: PlayerId, accept: bool, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
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

#[test]
fn package_publishes_abi143_and_complete_std_cenhun() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == STDJISHE)
        .unwrap();
    assert!(active.hand_limit_positive);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == STDJISHE
            && modifier.target == "hand_limit"
            && modifier.value_source.as_deref() == Some("owner_skill_use_count")
            && modifier.value == -1
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDWUDU
            && trigger.event == "damage_before"
            && trigger.filter == "owner_alive_event_player_hand_empty"
            && trigger.optional
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_CENHUN)
        .unwrap();
    assert_eq!(character.name_key, "std_cenhun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [STDJISHE, STDWUDU]);
    assert_eq!(
        content_compiler::compile_source_named("stdjishe", JISHE_SOURCE)
            .unwrap()
            .manifest
            .rule_vm_abi_version,
        143
    );
}

#[test]
fn stdjishe_draws_until_current_hand_limit_reaches_zero() {
    let setup = setup(false, 2_026_072_411);
    let initial_hand = setup.state.hand_count(setup.owner);
    let mut state = setup.state.clone();
    assert_eq!(hand_limit(&state, setup.owner), 3);
    for key in 1..=3 {
        let (next, result) = activate_jishe(&setup, state, key);
        assert!(matches!(result, CommandResult::Accepted { .. }));
        state = next;
        assert_eq!(hand_limit(&state, setup.owner), 3 - key as i32);
        assert_eq!(state.hand_count(setup.owner), initial_hand + key as u32);
    }
    let before = canonical_state_hash(&state);
    let (same, result) = activate_jishe(&setup, state, 4);
    assert!(matches!(result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}

#[test]
fn stdwudu_accept_prevents_empty_hand_target_damage_and_loses_owner_max_hp() {
    let setup = setup(true, 2_026_072_412);
    let target_hp = setup.state.players.get(setup.target).unwrap().hp;
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.source,
                target: setup.target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = resolve_optional(tx.commit().state, setup.owner, true, 10);
    assert_eq!(state.players.get(setup.owner).unwrap().max_hp, 2);
    assert_eq!(state.players.get(setup.target).unwrap().hp, target_hp);
}

#[test]
fn stdwudu_decline_or_nonempty_hand_allows_damage_without_cost() {
    let declined = setup(true, 2_026_072_413);
    let target_hp = declined.state.players.get(declined.target).unwrap().hp;
    let mut tx = Transaction::begin(declined.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: declined.source,
                target: declined.target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = resolve_optional(tx.commit().state, declined.owner, false, 20);
    assert_eq!(state.players.get(declined.owner).unwrap().max_hp, 3);
    assert_eq!(
        state.players.get(declined.target).unwrap().hp,
        target_hp - 1
    );

    let nonempty = setup(false, 2_026_072_414);
    let target_hp = nonempty.state.players.get(nonempty.target).unwrap().hp;
    let mut tx = Transaction::begin(nonempty.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: nonempty.source,
            target: nonempty.target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(nonempty.owner).unwrap().max_hp, 3);
    assert_eq!(
        tx.state().players.get(nonempty.target).unwrap().hp,
        target_hp - 1
    );
}
}

mod original_sixiang_stdjujian {
//! Golden coverage for 举荐（sixiang 标徐庶）无懈获牌（ABI 462）.
//!
//! 使用【无懈可击】后（每回合限一次），可令一名其他角色获得该牌。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDJUJIAN: u32 = 2_772_808_467;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdjujian.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selection: u32,
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
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted"
    );
    next
}

fn fire_card_used(
    tx: Transaction,
    player: PlayerId,
    card: CardId,
) -> Result<Transaction, EffectError> {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player,
            card,
            targets: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => Ok(tx),
            Err(EffectError::ChoiceRequested) => Ok(tx),
            Err(e) => Err(e),
        },
        Err(EffectError::ChoiceRequested) => Ok(tx),
        Err(e) => Err(e),
    }
}

#[test]
fn gives_wuxie_to_chosen_target_once_per_turn() {
    let package = content_compiler::compile_source_named("stdjujian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 462);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, _third) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDJUJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    // Create a wuxie card and place it in the discard pile (post-use state).
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .expect("discard pile");
    let wuxie = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_700),
        kind: CardKind::TrickWuxie,
        suit: engine_api::Suit::Spade,
        rank: 11,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .expect("discard zone")
        .cards
        .push(wuxie);

    // First wuxie use: optional confirm + target choice.
    let tx = fire_card_used(tx, owner, wuxie).unwrap();
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "optional confirm must be pending"
    );
    state = drive_choice(state, owner, 1); // owner confirms
    state = drive_choice(state, owner, target.raw()); // choose the recipient
    assert!(
        state.hand_cards(target).contains(&wuxie),
        "target gains the used wuxie"
    );

    // Second wuxie use in the same turn: once-per-turn, no choice.
    let mut tx = Transaction::begin(state);
    let wuxie2 = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_701),
        kind: CardKind::TrickWuxie,
        suit: engine_api::Suit::Club,
        rank: 12,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .expect("discard zone")
        .cards
        .push(wuxie2);
    let tx = fire_card_used(tx, owner, wuxie2).unwrap();
    let state = tx.commit().state;
    assert!(
        !state.has_pending_choice(),
        "second use in the same turn does not fire (once per turn)"
    );
}
}

mod original_sixiang_stdjujin {
//! Golden coverage for 标刘表「聚进」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_LIUBIAO: u32 = 2_470_672_462;
const STD_QUYI: u32 = 2_888_834_865;
const STDZISHOU: u32 = 2_683_293_404;
const ZONGSHI: u32 = 2_639_072_445;
const STDJUJIN: u32 = 3_408_182_272;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdjujin.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, output) = apply_command(
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
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    (state, output)
}

fn inject(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
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
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_and_runtime_preserve_exact_discard_then_recover_order() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 134);
    assert!(package.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "yijiang"
            && dependency.version == "0.1.0"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_LIUBIAO)
        .unwrap();
    assert_eq!(character.name_key, "std_liubiao");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [STDZISHOU, ZONGSHI, STDJUJIN]);
    let compiled = content_compiler::compile_source_named("stdjujin", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 134);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_074_402, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardCards {
        player: *player,
        cards: tx.state().hand_cards(*player),
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDJUJIN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().character_def = STD_LIUBIAO;
    tx.state_mut()
        .players
        .get_mut(source)
        .unwrap()
        .character_def = STD_QUYI;
    {
        let participant = tx.state_mut().players.get_mut(owner).unwrap();
        participant.hp = 2;
        participant.max_hp = 3;
    }
    let cards = [
        inject(&mut tx, owner, 134_001),
        inject(&mut tx, owner, 134_002),
        inject(&mut tx, owner, 134_003),
    ];

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::DamageDealt {
                source,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
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
    let (selecting, select_offer) = resolve(committed.state, owner, vec![1], 1);
    let choice = select_offer.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!((choice.min_count, choice.max_count), (2, 2));

    let (done, completed) = resolve(selecting, owner, vec![cards[0].raw(), cards[2].raw()], 2);
    assert_eq!(done.hand_count(owner), 1);
    assert_eq!(done.players.get(owner).unwrap().hp, 3);
    let moved = completed
        .events
        .iter()
        .position(|event| matches!(event, DomainEvent::CardsMoved { .. }))
        .unwrap();
    let recovered = completed
        .events
        .iter()
        .position(
            |event| matches!(event, DomainEvent::HpRecovered { player, .. } if *player == owner),
        )
        .unwrap();
    assert!(moved < recovered);
}
}

mod original_sixiang_stdkengqiang {
//! Golden for 文鸯「铿锵」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDKENGQIANG: u32 = 3_956_037_825;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdkengqiang.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
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

fn started(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let targets = players[1..].to_vec();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &players {
        effects.push(EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDKENGQIANG),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.max_hp = 15;
        participant.hp = 10;
    }
    (tx.commit().state, owner, targets)
}

fn play(
    state: GameState,
    owner: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve_optional(
    state: GameState,
    pipeline: PipelineResult,
    owner: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let choice = pipeline
        .choice
        .expect("铿锵 should ask whether to activate");
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice.choice_id.raw()),
                selections: vec![1],
                cancelled: false,
            },
        },
    )
}

fn resolve_branch(
    state: GameState,
    pipeline: PipelineResult,
    owner: PlayerId,
    branch: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VmResume
    );
    let choice = pipeline
        .choice
        .expect("铿锵 should ask which branch to use");
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice.choice_id.raw()),
                selections: vec![branch],
                cancelled: false,
            },
        },
    )
}

fn next_owner_turn(state: GameState, owner: PlayerId, other: PlayerId) -> GameState {
    let mut state = state;
    state.turn.phase = Phase::End;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, other);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_optional_damage_card_trigger_at_abi_141() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDKENGQIANG && !skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDKENGQIANG)
        .unwrap();
    assert_eq!(trigger.event, "card_used");
    assert_eq!(trigger.filter, "owner_source_damage_card_once_per_turn");
    assert!(trigger.optional);

    let compiled = content_compiler::compile_source_named("stdkengqiang", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 141);
}

#[test]
fn non_damage_card_does_not_consume_trigger_and_draw_branch_draws_two() {
    let (mut state, owner, targets) = started(2_026_072_401);
    let wuzhong = inject(&mut state, owner, CardKind::TrickWuzhong, 141_001);
    let (state_after_wuzhong, result) = play(state, owner, wuzhong, Vec::new(), 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state_after_wuzhong.has_pending_choice());

    state = state_after_wuzhong;
    let sha = inject(&mut state, owner, CardKind::Sha, 141_002);
    let hand_before = state.hand_count(owner);
    let target_hp = state.players.get(targets[0]).unwrap().hp;
    let (state, optional) = play(state, owner, sha, vec![targets[0]], 2);
    let (state, branch) = resolve_optional(state, optional, owner, 3);
    let (state, result) = resolve_branch(state, branch, owner, 1, 4);

    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert_eq!(state.players.get(targets[0]).unwrap().hp, target_hp - 1);
}

#[test]
fn lose_hp_branch_increases_current_damage_once_per_turn_and_resets_next_turn() {
    let (mut state, owner, targets) = started(2_026_072_402);
    let sha = inject(&mut state, owner, CardKind::Sha, 141_011);
    let owner_hp = state.players.get(owner).unwrap().hp;
    let target_hp = state.players.get(targets[0]).unwrap().hp;
    let (state, optional) = play(state, owner, sha, vec![targets[0]], 5);
    let (state, branch) = resolve_optional(state, optional, owner, 6);
    let (mut state, result) = resolve_branch(state, branch, owner, 2, 7);

    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(targets[0]).unwrap().hp, target_hp - 2);

    let nanman = inject(&mut state, owner, CardKind::TrickNanman, 141_012);
    let before: Vec<_> = targets
        .iter()
        .map(|target| state.players.get(*target).unwrap().hp)
        .collect();
    let (state, result) = play(state, owner, nanman, Vec::new(), 8);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    for (target, hp) in targets.iter().zip(before) {
        assert_eq!(state.players.get(*target).unwrap().hp, hp - 1);
    }

    let mut state = next_owner_turn(state, owner, targets[0]);
    let sha = inject(&mut state, owner, CardKind::Sha, 141_013);
    let (state, result) = play(state, owner, sha, vec![targets[0]], 9);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}
}

mod original_sixiang_stdmingfa {
//! Golden coverage for 标羊祜「明伐」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STD_DC_YANGHU: u32 = 3_430_968_343;
const STDMINGFA: u32 = 2_715_469_127;
const STDMINGFA_USED: u32 = 2_582_921_343;
const DAMAGE_BOOST: u32 = 4_000_000_001;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdmingfa.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
    skill: SkillId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
}

fn setup(boost_damage: bool) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_072_408, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    if boost_damage {
        let boost = content_compiler::compile_source(
            "skill stdmingfa_test_damage_boost(4000000001) locked { modifier damage_amount add 1; }",
        )
        .unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&boost).unwrap();
    }
    let players = tx.state().players.alive_by_seat();
    let (owner, target, other) = (players[0], players[1], players[2]);
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
        skill_def: SkillDefId::new(STDMINGFA),
    });
    if boost_damage {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DAMAGE_BOOST),
        });
    }
    effects.push(EffectRequest::SetTurnState {
        current: owner,
        phase: Phase::Play.to_u8(),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
        participant.dying = false;
    }
    let skill = skill_id(tx.state(), owner, STDMINGFA).unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        other,
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

fn resolve(state: GameState, selections: Vec<u32>, key: u64) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
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

fn run(state: GameState, effects: Vec<EffectRequest>) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn inject_tao(state: &mut GameState, owner: PlayerId) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(132_001),
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

fn assert_bound_target(state: &GameState, skill: SkillId, target: PlayerId) {
    let mut expected = b"BND1".to_vec();
    expected.extend_from_slice(&target.raw().to_le_bytes());
    assert_eq!(state.skills.get(skill).unwrap().state_blob, expected);
}

#[test]
fn package_publishes_abi132_skills_and_complete_std_dc_yanghu() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == STDMINGFA)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, None);
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self, targets.hp_gt),
        (1, 1, true, Some(1))
    );
    for event in ["hp_recovered", "death"] {
        assert!(package.trigger_index.iter().any(|trigger| {
            trigger.skill_def_id == STDMINGFA_USED
                && trigger.event == event
                && trigger.filter == "owner_alive_skill_bound_target_is_event_player"
                && !trigger.optional
        }));
    }
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_DC_YANGHU)
        .unwrap();
    assert_eq!(character.name_key, "std_dc_yanghu");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [STDMINGFA]);

    let compiled = content_compiler::compile_source_named("stdmingfa", SOURCE).unwrap();
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
fn surviving_target_disables_skill_until_that_target_recovers_or_dies() {
    let mut first = setup(false);
    first.state.players.get_mut(first.target).unwrap().hp = 3;
    first.state.players.get_mut(first.other).unwrap().hp = 3;
    let (state, result) = activate(first.state, first.owner, first.target, first.skill, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        } if *source == first.owner && *target == first.target
    )));
    assert_eq!(state.players.get(first.target).unwrap().hp, 2);
    assert!(skill_id(&state, first.owner, STDMINGFA).is_none());
    let helper = skill_id(&state, first.owner, STDMINGFA_USED).unwrap();
    assert_bound_target(&state, helper, first.target);

    let before_retry = canonical_state_hash(&state);
    let (state, retry) = activate(state, first.owner, first.target, first.skill, 2);
    assert!(matches!(retry.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before_retry);

    let state = run(
        state,
        vec![EffectRequest::RecoverHp {
            source: first.other,
            player: first.other,
            amount: 1,
        }],
    );
    assert!(skill_id(&state, first.owner, STDMINGFA).is_none());
    assert_eq!(skill_id(&state, first.owner, STDMINGFA_USED), Some(helper));

    let state = run(
        state,
        vec![EffectRequest::RecoverHp {
            source: first.owner,
            player: first.target,
            amount: 1,
        }],
    );
    assert!(skill_id(&state, first.owner, STDMINGFA).is_some());
    assert!(skill_id(&state, first.owner, STDMINGFA_USED).is_none());

    let mut second = setup(false);
    second.state.players.get_mut(second.target).unwrap().hp = 3;
    let (state, result) = activate(second.state, second.owner, second.target, second.skill, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let state = run(
        state,
        vec![EffectRequest::KillPlayer {
            player: second.target,
            killer: second.owner,
        }],
    );
    assert!(skill_id(&state, second.owner, STDMINGFA).is_some());
    assert!(skill_id(&state, second.owner, STDMINGFA_USED).is_none());
}

#[test]
fn literal_hp_threshold_rejects_one_hp_but_allows_self() {
    let mut illegal = setup(false);
    illegal.state.players.get_mut(illegal.target).unwrap().hp = 1;
    let before = canonical_state_hash(&illegal.state);
    let (same, result) = activate(
        illegal.state,
        illegal.owner,
        illegal.target,
        illegal.skill,
        10,
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let legal = setup(false);
    let owner_hp = legal.state.players.get(legal.owner).unwrap().hp;
    let (state, result) = activate(legal.state, legal.owner, legal.owner, legal.skill, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(legal.owner).unwrap().hp, owner_hp - 1);
    let helper = skill_id(&state, legal.owner, STDMINGFA_USED).unwrap();
    assert_bound_target(&state, helper, legal.owner);
}

#[test]
fn lethal_damage_keeps_skill_but_dying_save_finishes_before_disabling_it() {
    let mut lethal = setup(true);
    lethal.state.players.get_mut(lethal.target).unwrap().hp = 2;
    let (state, result) = activate(lethal.state, lethal.owner, lethal.target, lethal.skill, 20);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.players.get(lethal.target).unwrap().alive);
    assert!(skill_id(&state, lethal.owner, STDMINGFA).is_some());
    assert!(skill_id(&state, lethal.owner, STDMINGFA_USED).is_none());

    let mut saved = setup(true);
    saved.state.players.get_mut(saved.target).unwrap().hp = 2;
    inject_tao(&mut saved.state, saved.target);
    let (state, result) = activate(saved.state, saved.owner, saved.target, saved.skill, 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DyingSave
    );
    assert!(skill_id(&state, saved.owner, STDMINGFA).is_some());
    assert!(skill_id(&state, saved.owner, STDMINGFA_USED).is_none());
    assert!(result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == 1));

    let (state, result) = resolve(state, vec![1], 22);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let target = state.players.get(saved.target).unwrap();
    assert!(target.alive);
    assert!(!target.dying);
    assert_eq!(target.hp, 1);
    assert!(skill_id(&state, saved.owner, STDMINGFA).is_none());
    let helper = skill_id(&state, saved.owner, STDMINGFA_USED).unwrap();
    assert_bound_target(&state, helper, saved.target);
}
}

mod original_sixiang_stdmutao {
//! Golden coverage for 标鲍信「募讨」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDMUTAO: u32 = 3_513_095_743;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdmutao.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    sha_target: PlayerId,
    other_target: PlayerId,
    sha: CardId,
    other: CardId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn inject_hand(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_410, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let sha_target = players[1];
    let other_target = players[2];
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
        skill_def: SkillDefId::new(STDMUTAO),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let sha = inject_hand(tx.state_mut(), sha_target, CardKind::Sha, 137_001);
    let other = inject_hand(tx.state_mut(), other_target, CardKind::Shan, 137_002);
    Setup {
        tx,
        owner,
        sha_target,
        other_target,
        sha,
        other,
    }
}

fn phase_prepare(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::Prepare,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
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

fn trigger(setup: Setup) -> (GameState, PipelineResult) {
    let mut tx = setup.tx;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[phase_prepare(setup.owner)],
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
    resolve(committed.state, setup.owner, vec![1], 1)
}

#[test]
fn package_publishes_abi138_with_stdmutao_and_baoxin() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDMUTAO && skill.name == "stdmutao" && !skill.locked
    }));
    assert!(package
        .character_index
        .iter()
        .any(|character| character.name_key == "std_baoxin"));

    let compiled = content_compiler::compile_source_named("stdmutao", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 137);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn selected_sha_hand_is_publicly_revealed_before_owner_damage() {
    let setup = setup();
    let owner = setup.owner;
    let target = setup.sha_target;
    let other_target = setup.other_target;
    let sha = setup.sha;
    let before_hp = setup.tx.state().players.get(target).unwrap().hp;
    let (selecting, choice_output) = trigger(setup);
    assert!(matches!(
        choice_output.result,
        CommandResult::Accepted { .. }
    ));
    let choice = choice_output.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [target.raw(), other_target.raw()]
    );

    let (done, output) = resolve(selecting, owner, vec![target.raw()], 2);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(done.players.get(target).unwrap().hp, before_hp - 1);
    let revealed = output
        .events
        .iter()
        .position(|event| {
            matches!(event, DomainEvent::CardsRevealed { cards, to } if cards == &[sha] && to.is_empty())
        })
        .unwrap();
    let damaged = output
        .events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::DamageDealt {
                    source,
                    target: damaged,
                    amount: 1,
                    ..
                } if *source == owner && *damaged == target
            )
        })
        .unwrap();
    assert!(revealed < damaged);
}

#[test]
fn non_sha_hand_is_revealed_without_damage_and_empty_table_does_not_trigger() {
    let setup = setup();
    let owner = setup.owner;
    let target = setup.other_target;
    let other = setup.other;
    let before_hp = setup.tx.state().players.get(target).unwrap().hp;
    let (selecting, _) = trigger(setup);
    let (done, output) = resolve(selecting, owner, vec![target.raw()], 3);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(done.players.get(target).unwrap().hp, before_hp);
    assert!(output.events.iter().any(
        |event| matches!(event, DomainEvent::CardsRevealed { cards, to } if cards == &[other] && to.is_empty())
    ));
    assert!(!output.events.iter().any(
        |event| matches!(event, DomainEvent::DamageDealt { target: damaged, .. } if *damaged == target)
    ));

    let mut tx = Transaction::begin(done);
    let hands: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(|player| EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        })
        .collect();
    run_effects(&mut tx, hands).unwrap();
    let effects = dispatch_for_events(
        &mut tx,
        &[phase_prepare(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
}
}

mod original_sixiang_stdquedi {
//! Golden for 四象标文鸯「却敌」的实体【杀】视为【决斗】语义。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STD_DB_WENYANG: u32 = 2_396_957_062;
const STDQUEDI: u32 = 3_311_025_145;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

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
        def: CardDefId::new(97_000_000 + u32::from(rank)),
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

fn setup() -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_716, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    for player in [owner, target] {
        clear_hand(&mut tx, player);
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
        skill_def: SkillDefId::new(STDQUEDI),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == STDQUEDI)
        .unwrap()
        .id;
    (tx.state().clone(), owner, target, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    card: CardId,
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
fn package_contains_complete_std_db_wenyang_and_view_as_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_DB_WENYANG)
        .expect("committed 标文鸯 character");
    assert_eq!(character.name_key, "std_db_wenyang");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.initial_hp, 0, "omitted means start at max_hp");
    assert_eq!(character.skills, [STDQUEDI]);

    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == STDQUEDI)
        .expect("却敌 view_as entry");
    assert_eq!(view.as_kind, "juedou");
    assert_eq!(view.window, "play_phase");
    assert_eq!(view.filter, "card_sha");
    assert_eq!(view.source_zone.as_deref(), Some("hand_equip"));
    assert_eq!(view.uses_per_turn, None);
}

#[test]
fn stdquedi_rejects_non_sha_and_uses_hand_or_equipment_sha_as_duel() {
    let (state, owner, target, skill) = setup();
    let mut tx = Transaction::begin(state);
    let non_sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Heart,
        1,
    );
    let state = tx.state().clone();
    let before = canonical_state_hash(&state);
    let (same, result) = activate(state, owner, target, skill, non_sha, 1);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut tx = Transaction::begin(same);
    let shan = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Diamond,
        2,
    );
    let hand_sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        3,
    );
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let (state, result) = activate(tx.state().clone(), owner, target, skill, hand_sha, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(
        state.cards.get(shan).unwrap().zone,
        state.zones.find(ZoneKind::Hand, target).unwrap()
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(hand_sha).unwrap().zone, discard);

    let mut tx = Transaction::begin(state);
    let equip_sha = inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Sha,
        Suit::Club,
        4,
    );
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let (state, result) = activate(tx.state().clone(), owner, target, skill, equip_sha, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.cards.get(equip_sha).unwrap().zone, discard);

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    let late_sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Heart,
        5,
    );
    let state = tx.state().clone();
    let before = canonical_state_hash(&state);
    let (same, result) = activate(state, owner, target, skill, late_sha, 4);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
}

mod original_sixiang_stdrenzheng {
//! Golden coverage for 标骆统「仁政」rewritten from legacy JavaScript.

use engine_api::{DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDRENZHENG: u32 = 3_020_306_305;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdrenzheng.skill");
const SIXIANG: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup(extra_skill: &str, extra_def: u32) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let source = format!("{SOURCE}\n{extra_skill}");
    let package = content_compiler::compile_source_named("stdrenzheng-test", &source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_073_101, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let current = players[0];
    let damage_source = players[1];
    let target = players[2];
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: current,
                skill_def: SkillDefId::new(STDRENZHENG),
            },
            EffectRequest::AddSkill {
                player: if extra_def == 4_200_000_002 {
                    damage_source
                } else {
                    target
                },
                skill_def: SkillDefId::new(extra_def),
            },
            EffectRequest::SetTurnState {
                current,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    (tx, current, damage_source, target)
}

fn assert_prevention_draws(
    mut tx: Transaction,
    current: PlayerId,
    source: PlayerId,
    target: PlayerId,
) {
    let hand_before = tx.state().hand_count(current);
    let hp_before = tx.state().players.get(target).unwrap().hp;
    let events = run_effects(
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
    assert_eq!(tx.state().players.get(target).unwrap().hp, hp_before);
    assert_eq!(tx.state().hand_count(current), hand_before + 1);
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::DamagePrevented {
            source: event_source,
            target: event_target,
            current: event_current,
        } if *event_source == source && *event_target == target && *event_current == current
    )));
}

#[test]
fn package_publishes_forced_damage_prevented_trigger_at_abi337() {
    let package = content_schema::ContentPackage::from_json_bytes(SIXIANG).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDRENZHENG)
        .unwrap();
    assert_eq!(trigger.event, "damage_prevented");
    assert_eq!(trigger.filter, "owner_alive_event_player_alive");
    assert!(!trigger.optional);
}

#[test]
fn static_and_damage_before_prevention_both_make_current_player_draw() {
    let (tx, current, source, target) = setup(
        "skill pacifist(4200000002) locked { modifier damage_cancel override 1; }",
        4_200_000_002,
    );
    assert_prevention_draws(tx, current, source, target);

    let (tx, current, source, target) = setup(
        "skill shield(4200000003) locked {
            trigger damage_before owner_is_target {
                effect prevent_current_damage();
            }
        }",
        4_200_000_003,
    );
    assert_prevention_draws(tx, current, source, target);
}
}

mod original_sixiang_stdshangjian {
//! Golden coverage for 标王元姬「尚俭」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_WANGYUANJI: u32 = 3_012_016_618;
const STDQIANCHONG: u32 = 2_607_738_803;
const STDSHANGJIAN: u32 = 4_047_696_367;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdshangjian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn setup(hp: i32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdshangjian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 201);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_726, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, other) = (players[0], players[1]);
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
        skill_def: SkillDefId::new(STDSHANGJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.clear_per_player_card_counts();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp;
    (tx, owner, other)
}

fn inject(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn move_card(tx: &mut Transaction, card: CardId, from: u32, to: u32) {
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            from: Some(engine_api::ZoneId::new(from)),
            to: engine_api::ZoneId::new(to),
        }],
    )
    .unwrap();
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
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
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    (state, result)
}

#[test]
fn committed_package_publishes_complete_character_and_abi201_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDSHANGJIAN
            && trigger.event == "phase_end"
            && trigger.filter == "owner_is_target_lost_cards_at_most_hp_gainable"
            && trigger.optional
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_WANGYUANJI)
        .unwrap();
    assert_eq!(character.skills, [STDQIANCHONG, STDSHANGJIAN]);
}

#[test]
fn total_loss_threshold_and_discard_provenance_offer_only_gainable_cards() {
    let (mut tx, owner, other) = setup(3);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let gainable = inject(&mut tx, owner, 98_001);
    let transferred = inject(&mut tx, owner, 98_002);
    move_card(&mut tx, gainable, owner_hand.raw(), discard.raw());
    move_card(&mut tx, transferred, owner_hand.raw(), other_hand.raw());
    assert_eq!(tx.state().turn.card_loss_count(owner), 2);

    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new(),),
        Err(EffectError::ChoiceRequested)
    );
    let (state, offered) = resolve(tx.commit().state, owner, vec![1], 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseGainOwnLostDiscardCardThisTurn
    );
    let choice = offered.choice.unwrap();
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![gainable.raw()]
    );
    let (state, _) = resolve(state, owner, vec![gainable.raw()], 2);
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(gainable).unwrap().zone, owner_hand);
    assert_eq!(state.cards.get(transferred).unwrap().zone, other_hand);
}

#[test]
fn repeat_loss_counts_twice_but_offers_one_card_and_excess_loss_suppresses_trigger() {
    let (mut tx, owner, _) = setup(2);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = inject(&mut tx, owner, 98_003);
    move_card(&mut tx, card, hand.raw(), discard.raw());
    move_card(&mut tx, card, discard.raw(), hand.raw());
    move_card(&mut tx, card, hand.raw(), discard.raw());
    assert_eq!(tx.state().turn.card_loss_count(owner), 2);
    assert_eq!(
        tx.state().turn.lost_cards_for(owner).collect::<Vec<_>>(),
        vec![card]
    );
    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new(),),
        Err(EffectError::ChoiceRequested)
    );

    let (mut excessive, owner, _) = setup(1);
    let discard = excessive
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = excessive.state().zones.find(ZoneKind::Hand, owner).unwrap();
    for def in [98_004, 98_005] {
        let card = inject(&mut excessive, owner, def);
        move_card(&mut excessive, card, hand.raw(), discard.raw());
    }
    assert!(dispatch_for_events(
        &mut excessive,
        &[phase_end(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
}
}

mod original_sixiang_stdshangjue {
//! Golden for 标文鸳「殇决」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDKENGQIANG: u32 = 3_956_037_825;
const STDSHANGJUE: u32 = 3_835_885_608;
const STD_WENYUAN: u32 = 4_166_759_034;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdshangjue.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup(with_kengqiang: bool) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_072_402, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    if with_kengqiang {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDKENGQIANG),
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDSHANGJUE),
    });
    run_effects(&mut tx, effects).unwrap();
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.max_hp = 3;
    participant.hp = 0;
    participant.dying = true;
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
    state: GameState,
    owner: PlayerId,
    selection: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

#[test]
fn package_publishes_abi142_and_complete_std_wenyuan() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDSHANGJUE)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        ("dying", "owner_is_target", true)
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_WENYUAN)
        .unwrap();
    assert_eq!(character.name_key, "std_wenyuan");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [STDKENGQIANG, STDSHANGJUE]);
    assert_eq!(
        content_compiler::compile_source_named("stdshangjue", SOURCE)
            .unwrap()
            .manifest
            .rule_vm_abi_version,
        142
    );
}

#[test]
fn accepting_recovers_before_exchange_and_accepting_exchange_restores_the_limit() {
    let (mut tx, owner) = setup(true);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = resolve(tx.commit().state, owner, 1, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert!(!state.players.get(owner).unwrap().dying);
    assert!(!has_skill(&state, owner, STDSHANGJUE));
    assert!(has_skill(&state, owner, STDKENGQIANG));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RecoverToMaxThenOptionalExchangeSkill
    );
    assert_eq!(result.choice.unwrap().kind, ChoiceKind::Confirm);

    let (state, result) = resolve(state, owner, 1, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(!has_skill(&state, owner, STDKENGQIANG));
    assert!(has_skill(&state, owner, STDSHANGJUE));

    let mut tx = Transaction::begin(state);
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.hp = 0;
    participant.dying = true;
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}

#[test]
fn declining_or_missing_kengqiang_consumes_no_extra_skill_and_never_offers_exchange() {
    let (mut tx, owner) = setup(true);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = resolve(tx.commit().state, owner, 0, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 0);
    assert!(has_skill(&state, owner, STDKENGQIANG));
    assert!(has_skill(&state, owner, STDSHANGJUE));

    let (mut tx, owner) = setup(false);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = resolve(tx.commit().state, owner, 1, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert!(!state.has_pending_choice());
    assert!(!has_skill(&state, owner, STDKENGQIANG));
    assert!(!has_skill(&state, owner, STDSHANGJUE));
}

#[test]
fn declining_post_recovery_exchange_keeps_kengqiang_and_spends_shangjue() {
    let (mut tx, owner) = setup(true);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, _) = resolve(tx.commit().state, owner, 1, 5);
    let (state, result) = resolve(state, owner, 0, 6);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert!(!state.has_pending_choice());
    assert!(has_skill(&state, owner, STDKENGQIANG));
    assert!(!has_skill(&state, owner, STDSHANGJUE));
}
}

mod original_sixiang_stdshiren {
//! Golden coverage for 四象「士仁」.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDSHIREN: u32 = 2_834_902_877;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdshiren.skill");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = compile_source_named("stdshiren-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_715, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let old_skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old_skills).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDSHIREN),
        }],
    )
    .unwrap();
    (tx.commit().state, owner, source)
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_800 + u32::from(rank)),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selection: u32,
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
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

fn use_on(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    card: CardId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card,
            targets: vec![target],
        }],
    );
    (tx.commit().state, result)
}

#[test]
fn only_the_first_other_players_sha_each_turn_draws_two_then_gives_one() {
    let (mut state, owner, source) = setup();
    let non_sha = inject(&mut state, source, CardKind::Other, 5);
    let (mut state, result) = use_on(state, source, owner, non_sha);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());

    let old_card = state.hand_cards(owner)[0];
    let owner_hand = state.hand_count(owner);
    let source_hand = state.hand_count(source);
    let sha = inject(&mut state, source, CardKind::Sha, 6);
    let (state, result) = use_on(state, source, owner, sha);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, accepted) = resolve(state, owner, 1, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GiveOneHandOrEquip
    );
    assert_eq!(state.hand_count(owner), owner_hand + 2);

    let (mut state, given) = resolve(state, owner, old_card.raw(), 2);
    assert!(matches!(given.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.hand_count(source), source_hand + 1);

    let second_sha = inject(&mut state, source, CardKind::Sha, 7);
    let owner_after = state.hand_count(owner);
    let (state, result) = use_on(state, source, owner, second_sha);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_after);
}
}

mod original_sixiang_stdshushenx {
//! Golden coverage for 标甘夫人「淑慎」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardMoveReason, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDSHUSHENX: u32 = 3_633_500_345;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdshushenx.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_412, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let source = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDSHUSHENX),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.clear_per_player_card_counts();
    tx.state_mut().turn.current = source;
    tx.state_mut().turn.phase = engine_core::state::Phase::End;
    tx.state_mut().players.get_mut(target).unwrap().hp -= 1;
    (tx, owner, target, source)
}

fn phase_end(current: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current,
        phase: TurnPhaseTag::End,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
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

#[test]
fn package_publishes_abi140_and_complete_ganfuren() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDSHUSHENX && skill.name == "stdshushenx" && !skill.locked
    }));
    assert!(package
        .character_index
        .iter()
        .any(|character| character.name_key == "std_ganfuren"));

    let compiled = content_compiler::compile_source_named("stdshushenx", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 139);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn draw_and_gift_share_turn_gain_count_then_recover_other() {
    let (mut tx, owner, target, source) = setup();
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 2,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().turn.card_gain_count(owner), 2);
    assert!(
        dispatch_for_events(&mut tx, &[phase_end(source)], &mut TriggerFireRecord::new(),)
            .unwrap()
            .is_empty()
    );

    let card = tx.state().hand_cards(source)[0];
    let from = tx.state().cards.get(card).unwrap().zone;
    let to = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let moved = run_effects(
        &mut tx,
        vec![EffectRequest::GiveCards {
            cards: vec![card],
            to,
            from,
        }],
    )
    .unwrap();
    assert!(moved.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved {
            cards,
            reason: CardMoveReason::Give,
            ..
        } if cards == &vec![card]
    )));
    assert_eq!(tx.state().turn.card_gain_count(owner), 3);

    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end(source)], &mut TriggerFireRecord::new(),)
            .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = tx.commit();
    assert_eq!(
        committed.output.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );
    let (selecting, selected) = resolve(committed.state, owner, vec![1], 1);
    assert!(matches!(selected.result, CommandResult::Accepted { .. }));
    assert_eq!(
        selected.choice.as_ref().map(|choice| choice.kind),
        Some(ChoiceKind::SelectTargets)
    );
    let before = selecting.players.get(target).unwrap().hp;
    let (done, recovered) = resolve(selecting, owner, vec![target.raw()], 2);
    assert!(matches!(recovered.result, CommandResult::Accepted { .. }));
    assert_eq!(done.players.get(target).unwrap().hp, before + 1);
}
}

mod original_sixiang_stdsidi {
//! Golden for the first complete historical one-skill character: 标曹真「司敌」.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const STD_CAOZHEN: u32 = 2_623_502_245;
const STD_CAOJINYU: u32 = 2_785_864_584;
const STD_QUYI: u32 = 2_888_834_865;
const STD_NIUJIN: u32 = 3_712_372_828;
const STD_DB_WENYANG: u32 = 2_396_957_062;
const STDSIDI: u32 = 4_128_943_540;
const STDSHIBEI: u32 = 2_559_343_711;
const STDDULIANG: u32 = 2_231_518_236;
const STDHUANGONG: u32 = 2_543_694_356;
const STDQIANSU: u32 = 2_964_345_042;
const STDWUYAN: u32 = 3_164_633_937;
const STDSHANSHEN: u32 = 2_423_225_569;
const STDZHONGYAN: u32 = 3_353_809_882;
const STDANRAN: u32 = 2_536_614_283;
const STDYUQI: u32 = 3_561_403_597;
const STDQINGMAN: u32 = 3_965_321_062;
const STDJIAOZI: u32 = 3_194_342_337;
const STDFUQI: u32 = 2_567_338_825;
const STD_MENGDA: u32 = 3_789_552_174;
const STDZHUAN: u32 = 4_034_502_794;
const STDCUORUI: u32 = 2_683_321_015;
const STDQUEDI: u32 = 3_311_025_145;
const STD_LVFAN: u32 = 3_319_229_677;
const MBDIAODU: u32 = 2_440_438_931;
const STDDIANFENG: u32 = 3_414_143_685;
const STD_ZHANGYI: u32 = 3_037_060_961;
const STDZHIYI: u32 = 3_959_114_365;
const STDTIAOHE: u32 = 2_595_769_846;
const STD_FEIYI: u32 = 2_878_317_274;
const STD_ZHONGHUI: u32 = 2_397_163_146;
const STDXINGFA: u32 = 2_490_051_933;
const STD_LIUBA: u32 = 2_185_226_076;
const STDDUANBI: u32 = 3_811_377_323;
const STD_JIAKUI: u32 = 3_406_075_592;
const STDZHONGZUO: u32 = 3_465_971_059;
const STDWANLAN: u32 = 3_664_427_038;
const STD_WANGYUN: u32 = 3_976_039_480;
const STDYUNJI: u32 = 2_318_160_114;
const STDZONGJI: u32 = 3_394_363_597;
const STDDUANBING: u32 = 2_170_694_873;
const STD_DONGYUN: u32 = 2_711_327_184;
const STDBINGZHENG: u32 = 3_798_362_737;
const STD_WUKE: u32 = 4_182_138_442;
const STDANDA: u32 = 3_646_320_211;
const STDZHUGUO: u32 = 3_147_233_152;
const STD_DINGFENG: u32 = 3_454_509_574;
const STDFENXUN: u32 = 2_579_637_455;
const STD_LIUZAN: u32 = 4_100_887_402;
const STD_LIUBIAO: u32 = 2_470_672_462;
const STD_LIJUE: u32 = 2_852_318_654;
const STD_ZHANGFEN: u32 = 3_289_092_799;
const STD_BAOXIN: u32 = 2_419_830_352;
const STD_GANFUREN: u32 = 2_343_280_510;
const STD_WENYUAN: u32 = 4_166_759_034;
const STD_CENHUN: u32 = 3_739_444_871;
const STD_HUANGFUSONG: u32 = 3_128_139_054;
const STD_SUNHAO: u32 = 4_237_199_211;
const STDFENYIN: u32 = 2_225_023_788;
const STDZISHOU: u32 = 2_683_293_404;
const ZONGSHI: u32 = 2_639_072_445;
const STDJUJIN: u32 = 3_408_182_272;
const STDXIONGSUAN: u32 = 3_084_450_928;
const STDWANGLU: u32 = 3_938_702_468;
const STDMUTAO: u32 = 3_513_095_743;
const STDYIMOU: u32 = 4_237_148_705;
const STDSHUSHENX: u32 = 3_633_500_345;
const STDZHIJIE: u32 = 2_635_655_868;
const STDKENGQIANG: u32 = 3_956_037_825;
const STDSHANGJUE: u32 = 3_835_885_608;
const STDJISHE: u32 = 2_148_577_926;
const STDWUDU: u32 = 4_151_205_179;
const STDTAOLUAN: u32 = 3_364_139_751;
const STDCANSHI: u32 = 2_290_683_272;
const STDCANSHI_EFFECT: u32 = 2_335_115_700;
const CHOUHAI: u32 = 2_265_775_915;
const GUIMING: u32 = 3_828_399_214;
const STDANRAN_USED_MARK: u32 = 221_181_016;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

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

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    inject_into(tx, owner, ZoneKind::Hand, kind, suit, rank)
}

fn inject_into(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::new(9_200 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    };
    let id = tx.state_mut().cards.alloc(card);
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(id);
    id
}

#[test]
fn committed_package_matches_source_and_complete_character() {
    let committed = package();

    assert_eq!(
        committed.manifest.logic_hash,
        committed.compute_logic_hash()
    );
    assert_eq!(committed.skill_index.len(), 181);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDSIDI && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDSHIBEI && skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDDULIANG && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDHUANGONG && skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDQUEDI && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDTIAOHE && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDDUANBI && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDMUTAO && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDYIMOU && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDSHUSHENX && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDZHIJIE && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDKENGQIANG && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDSHANGJUE && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDJISHE && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDWUDU && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDTAOLUAN && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDCANSHI && skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDCANSHI_EFFECT && skill.locked));
    assert_eq!(committed.character_index.len(), 101);
    let find_char = |id: u32| committed.character_index.iter().find(|c| c.id == id);
    let caozhen = find_char(STD_CAOZHEN).unwrap();
    assert_eq!(caozhen.name_key, "std_caozhen");
    assert_eq!(caozhen.max_hp, 4);
    assert_eq!(caozhen.skills, vec![STDSIDI]);
    let caojinyu = find_char(STD_CAOJINYU).unwrap();
    assert_eq!(caojinyu.skills, vec![STDYUQI, STDSHANSHEN]);
    let mengda = find_char(STD_MENGDA).unwrap();
    assert_eq!(mengda.name_key, "std_mengda");
    assert_eq!(mengda.max_hp, 4);
    assert_eq!(mengda.skills, vec![STDZHUAN]);
    let quyi = find_char(STD_QUYI).unwrap();
    assert_eq!(quyi.name_key, "std_quyi");
    assert_eq!(quyi.max_hp, 4);
    assert_eq!(quyi.skills, vec![STDFUQI, STDJIAOZI]);
    let niujin = find_char(STD_NIUJIN).unwrap();
    assert_eq!(niujin.name_key, "std_niujin");
    assert_eq!(niujin.max_hp, 4);
    let niujin = find_char(STD_NIUJIN).unwrap();
    assert_eq!(niujin.skills, vec![STDCUORUI]);
    let db_wenyang = find_char(STD_DB_WENYANG).unwrap();
    assert_eq!(db_wenyang.name_key, "std_db_wenyang");
    assert_eq!(db_wenyang.max_hp, 4);
    assert_eq!(db_wenyang.skills, vec![STDQUEDI]);
    let lvfan = find_char(STD_LVFAN).unwrap();
    assert_eq!(lvfan.name_key, "std_lvfan");
    assert_eq!(lvfan.skills, vec![MBDIAODU, STDDIANFENG]);
    let zhangyi = find_char(STD_ZHANGYI).unwrap();
    assert_eq!(zhangyi.name_key, "std_zhangyì");
    assert_eq!(zhangyi.max_hp, 4);
    assert_eq!(zhangyi.skills, vec![STDZHIYI]);
    let feiyi = find_char(STD_FEIYI).unwrap();
    assert_eq!(feiyi.name_key, "std_feiyi");
    assert_eq!(feiyi.max_hp, 3);
    assert_eq!(feiyi.skills, vec![STDTIAOHE, STDQIANSU]);
    let zhonghui = find_char(STD_ZHONGHUI).unwrap();
    assert_eq!(zhonghui.name_key, "std_zhonghui");
    assert_eq!(zhonghui.max_hp, 4);
    let zhonghui_xingfa = find_char(STD_ZHONGHUI).unwrap();
    assert_eq!(zhonghui_xingfa.skills, vec![STDXINGFA]);
    let liuba = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_LIUBA)
        .unwrap();
    assert_eq!(liuba.name_key, "std_liuba");
    assert_eq!(liuba.max_hp, 3);
    assert_eq!(liuba.skills, vec![STDDUANBI]);
    let jiakui = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_JIAKUI)
        .unwrap();
    assert_eq!(jiakui.name_key, "std_jiakui");
    assert_eq!(jiakui.max_hp, 3);
    assert_eq!(jiakui.skills, vec![STDZHONGZUO, STDWANLAN]);
    let wangyun = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_WANGYUN)
        .unwrap();
    assert_eq!(wangyun.name_key, "std_wangyun");
    assert_eq!(wangyun.max_hp, 3);
    assert_eq!(wangyun.skills, vec![STDYUNJI, STDZONGJI]);
    let dongyun = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_DONGYUN)
        .unwrap();
    assert_eq!(dongyun.name_key, "std_dongyun");
    assert_eq!(dongyun.max_hp, 3);
    assert_eq!(dongyun.skills, vec![STDBINGZHENG, STDDULIANG]);
    let wuke = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_WUKE)
        .unwrap();
    assert_eq!(wuke.name_key, "std_wuke");
    assert_eq!(wuke.max_hp, 3);
    assert_eq!(wuke.skills, vec![STDANDA, STDZHUGUO]);
    let dingfeng = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_DINGFENG)
        .unwrap();
    assert_eq!(dingfeng.name_key, "std_dingfeng");
    assert_eq!(dingfeng.max_hp, 4);
    assert_eq!(dingfeng.skills, vec![STDDUANBING, STDFENXUN]);
    let liuzan = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_LIUZAN)
        .unwrap();
    assert_eq!(liuzan.name_key, "std_liuzan");
    assert_eq!(liuzan.max_hp, 4);
    assert_eq!(liuzan.skills, vec![STDFENYIN]);
    let liubiao = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_LIUBIAO)
        .unwrap();
    assert_eq!(liubiao.name_key, "std_liubiao");
    assert_eq!(liubiao.max_hp, 3);
    assert_eq!(liubiao.skills, vec![STDZISHOU, ZONGSHI, STDJUJIN]);
    let lijue = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_LIJUE)
        .unwrap();
    assert_eq!(lijue.name_key, "std_lijue");
    assert_eq!(lijue.max_hp, 5);
    assert_eq!(lijue.skills, vec![STDXIONGSUAN]);
    let zhangfen = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_ZHANGFEN)
        .unwrap();
    assert_eq!(zhangfen.name_key, "std_zhangfen");
    assert_eq!(zhangfen.max_hp, 3);
    assert_eq!(zhangfen.skills, vec![STDWANGLU]);
    let baoxin = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_BAOXIN)
        .unwrap();
    assert_eq!(baoxin.name_key, "std_baoxin");
    assert_eq!(baoxin.max_hp, 4);
    assert_eq!(baoxin.skills, vec![STDYIMOU, STDMUTAO]);
    let ganfuren = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_GANFUREN)
        .unwrap();
    assert_eq!(ganfuren.name_key, "std_ganfuren");
    assert_eq!(ganfuren.max_hp, 3);
    assert_eq!(ganfuren.skills, vec![STDZHIJIE, STDSHUSHENX]);
    let wenyuan = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_WENYUAN)
        .unwrap();
    assert_eq!(wenyuan.name_key, "std_wenyuan");
    assert_eq!(wenyuan.max_hp, 3);
    assert_eq!(wenyuan.skills, vec![STDKENGQIANG, STDSHANGJUE]);
    let cenhun = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_CENHUN)
        .unwrap();
    assert_eq!(cenhun.name_key, "std_cenhun");
    assert_eq!(cenhun.max_hp, 3);
    assert_eq!(cenhun.skills, vec![STDJISHE, STDWUDU]);
    let huangfusong = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_HUANGFUSONG)
        .unwrap();
    assert_eq!(huangfusong.name_key, "std_huangfusong");
    assert_eq!(huangfusong.max_hp, 4);
    assert_eq!(huangfusong.skills, vec![STDTAOLUAN]);
    let sunhao = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_SUNHAO)
        .unwrap();
    assert_eq!(sunhao.name_key, "std_sunhao");
    assert_eq!(sunhao.max_hp, 5);
    assert_eq!(sunhao.skills, vec![STDCANSHI, CHOUHAI, GUIMING]);
}

#[test]
fn stdyuqi_uses_event_player_to_owner_distance_and_owner_hp() {
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(20_260_716, 4));
    let owner = engine.state().turn.current;
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
            skill_def: SkillDefId::new(STDYUQI),
        }],
    )
    .unwrap();
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .max_by_key(|player| {
            match engine_core::query::query(
                tx.state(),
                &rule_api::QueryRequest::Distance {
                    from: *player,
                    to: owner,
                },
                &mut engine_core::query::QueryContext::new(),
            ) {
                rule_api::QueryResponse::Int(distance) => distance,
                _ => 0,
            }
        })
        .unwrap();
    let source = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner && *player != target)
        .unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let hand = tx.state().hand_count(target);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(20_260_716),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(target), hand + 1);

    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(target), hand + 1);
}

#[test]
fn stdqingman_draws_to_three_or_own_empty_equip_slots() {
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(20_260_717, 2));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDQINGMAN),
        },
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::SetTurnState {
            current: other,
            phase: Phase::End.to_u8(),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Prepare.to_u8(),
        },
    ]);
    run_effects(&mut tx, setup).unwrap();
    assert_eq!(tx.state().hand_count(owner), 3);

    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    inject_into(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Spade,
        5,
    );
    inject_into(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Heart,
        6,
    );
    inject_into(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Treasure,
        Suit::Club,
        7,
    );
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: other,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 2);
}

#[test]
fn stdwuyan_uses_wuxie_as_the_effective_name_for_physical_tricks() {
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(20_260_714, 2));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDWUYAN),
        }],
    )
    .unwrap();
    let card = inject(&mut tx, owner, CardKind::TrickJuedou, Suit::Spade, 1);
    let physical = tx.state().cards.get(card).unwrap();
    assert_eq!(physical.kind, CardKind::TrickJuedou);
    assert_eq!(
        engine_core::query::effective_card_kind(tx.state(), owner, physical),
        CardKind::TrickWuxie
    );
    assert_eq!(
        tx.state().cards.get(card).unwrap().kind,
        CardKind::TrickJuedou
    );
}

#[test]
fn stdhuangong_draws_only_after_the_last_field_card_leaves() {
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(20_260_715, 2));
    let owner = engine.state().turn.current;
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
            skill_def: SkillDefId::new(STDHUANGONG),
        }],
    )
    .unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let weapon = inject_into(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Spade,
        5,
    );
    let delayed = inject_into(
        &mut tx,
        owner,
        ZoneKind::Judge,
        CardKind::TrickLe,
        Suit::Heart,
        6,
    );
    let hand = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon],
            to: discard,
            from: Some(equip),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand);

    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![delayed],
            to: discard,
            from: Some(judge),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);
}

#[test]
fn stdshibei_recovers_only_after_first_damage() {
    let committed = package();
    let mut engine = Engine::new();
    engine.submit(start(20_260_713, 2));
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
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
            skill_def: SkillDefId::new(STDSHIBEI),
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(owner).unwrap().hp;

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
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);
    assert!(!tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.def.raw() == STDSHIBEI));

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
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

#[test]
fn stdduliang_checks_hand_count_after_drawing() {
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(20_260_714, 2));
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(engine.state().clone());
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.hp = 4;
        player.max_hp = 4;
    }
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDDULIANG),
        },
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::DrawCards {
            player: owner,
            count: 2,
        },
    ]);
    run_effects(&mut tx, setup).unwrap();
    let hp = tx.state().players.get(owner).unwrap().hp;
    let hand = tx.state().hand_count(owner);
    assert_eq!(hand, 2);

    let paused = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
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
    assert_eq!(state.hand_count(owner), hand + 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
}

#[test]
fn stdsidi_only_offers_draw_for_sha_response() {
    let committed = package();
    let mut engine = Engine::new();
    engine.submit(start(20_260_713, 2));
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;

    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDSIDI),
        }],
    )
    .unwrap();
    let hand_before = tx.state().hand_count(owner);
    let paused = run_effects(
        &mut tx,
        vec![EffectRequest::CardResponded {
            player: owner,
            respond_to: PlayerId::NONE,
            card: CardId::NONE,
            response_kind: CardKind::Sha.tag(),
        }],
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    assert!(tx.state().has_pending_choice());

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
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert!(!state.has_pending_choice());
    let mut non_sha = Transaction::begin(state);
    let hand_before = non_sha.state().hand_count(owner);
    run_effects(
        &mut non_sha,
        vec![EffectRequest::CardResponded {
            player: owner,
            respond_to: PlayerId::NONE,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }],
    )
    .unwrap();
    assert_eq!(non_sha.state().hand_count(owner), hand_before);
    assert!(!non_sha.state().has_pending_choice());
}

#[test]
fn optional_stdsidi_draw_resumes_juedou_exchange() {
    let committed = package();
    let mut engine = Engine::new();
    engine.submit(start(73, 4));
    engine.load_package(&committed).unwrap();
    let actor = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;

    // Isolate the response/continuation path from standard character triggers.
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
        player: actor,
        skill_def: SkillDefId::new(STDSIDI),
    });
    run_effects(&mut tx, effects).unwrap();

    // Remove random response/trick kinds while keeping the canonical zones intact.
    let hand_cards: Vec<_> = tx
        .state()
        .players
        .iter_live()
        .flat_map(|player| tx.state().hand_cards(player.id))
        .collect();
    for card in hand_cards {
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
    }

    let juedou = inject(&mut tx, actor, CardKind::TrickJuedou, Suit::Diamond, 1);
    let actor_sha = inject(&mut tx, actor, CardKind::Sha, Suit::Spade, 7);
    let target_sha = inject(&mut tx, target, CardKind::Sha, Suit::Heart, 8);

    let (state, played) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: juedou,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let response = played.choice.expect("target should be asked for 杀");
    assert_eq!(response.observer, target);

    let (state, responded) = apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: response.choice_id,
                selections: vec![target_sha.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(responded.result, CommandResult::Accepted { .. }));
    let confirm = responded.choice.expect("司敌 should ask its owner");
    assert_eq!(confirm.kind, ChoiceKind::Confirm);
    assert_eq!(confirm.observer, actor);
    let hand_before = state.hand_count(actor);

    let (state, resumed) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id: confirm.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(resumed.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(actor), hand_before + 1);
    let next = resumed
        .choice
        .expect("决斗 must resume and ask the other side for the next 杀");
    assert_eq!(next.observer, actor);
    assert!(next
        .options
        .iter()
        .any(|option| option.token == actor_sha.raw()));
}

#[test]
fn stdqiansu_only_draws_when_targeted_by_a_trick_with_empty_equip_zone() {
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(20_260_716, 2));
    let owner = engine.state().turn.current;
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
        skill_def: SkillDefId::new(STDQIANSU),
    });
    run_effects(&mut tx, effects).unwrap();

    let ignored = inject(&mut tx, source, CardKind::TrickWuzhong, Suit::Club, 1);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: ignored,
            targets: vec![source],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());

    let targeted = inject(&mut tx, source, CardKind::TrickWuzhong, Suit::Diamond, 2);
    let hand = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: source,
                card: targeted,
                targets: vec![owner],
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
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 1);

    let mut equipped = Transaction::begin(state);
    inject_into(
        &mut equipped,
        owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Spade,
        3,
    );
    let blocked = inject(
        &mut equipped,
        source,
        CardKind::TrickWuzhong,
        Suit::Heart,
        4,
    );
    let hand = equipped.state().hand_count(owner);
    run_effects(
        &mut equipped,
        vec![EffectRequest::UseCard {
            player: source,
            card: blocked,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(equipped.state().hand_count(owner), hand);
    assert!(!equipped.state().has_pending_choice());
}

fn death_recovery_case(
    skill_def: u32,
    victim_character: u32,
    owner_is_killer: bool,
) -> (
    Transaction,
    PlayerId,
    Result<Vec<engine_api::DomainEvent>, EffectError>,
) {
    let mut committed = package();
    committed.character_index.extend(
        [(9_216_001, "wei"), (9_216_002, "qun"), (9_216_003, "shu")]
            .into_iter()
            .map(|(id, group)| PackageCharacterEntry {
                id,
                name_key: format!("death_group_{id}"),
                max_hp: 4,
                initial_hp: 4,
                skills: vec![],
                sex: "male".into(),
                group: group.into(),
                is_lord: false,
                clans: vec![],
                tags: vec![],
                sort_key: String::new(),
                portrait_asset: None,
                skin_assets: vec![],
            }),
    );
    committed.manifest.logic_hash = committed.compute_logic_hash();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(skill_def as u64, 4));
    let owner = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.identity == Identity::Lord)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let victim = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    {
        let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
        owner_state.character_def = 9_216_001;
        owner_state.hp = 2;
        owner_state.max_hp = 4;
    }
    tx.state_mut()
        .players
        .get_mut(victim)
        .unwrap()
        .character_def = victim_character;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: if owner_is_killer {
                owner
            } else {
                PlayerId::NONE
            },
        }],
    );
    (tx, owner, result)
}

fn accept_optional_recovery(tx: Transaction, owner: PlayerId) -> engine_core::state::GameState {
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "death recovery choice rejected: {:?}",
        result.result
    );
    state
}

#[test]
fn stdshanshen_recovers_only_when_someone_else_caused_the_death() {
    let (tx, owner, result) = death_recovery_case(STDSHANSHEN, 9_216_002, false);
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    let state = accept_optional_recovery(tx, owner);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);

    let (tx, owner, result) = death_recovery_case(STDSHANSHEN, 9_216_002, true);
    result.unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
}

#[test]
fn stdzhongyan_is_lord_only_and_requires_a_qun_death() {
    let (tx, owner, result) = death_recovery_case(STDZHONGYAN, 9_216_002, false);
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    let state = accept_optional_recovery(tx, owner);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);

    let (tx, owner, result) = death_recovery_case(STDZHONGYAN, 9_216_003, false);
    result.unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
}

#[test]
fn stdanran_draws_one_to_four_and_then_stays_capped() {
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(20_260_824, 2));
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
            skill_def: SkillDefId::new(STDANRAN),
        }],
    )
    .unwrap();
    let initial_hand = tx.state().hand_count(owner);
    let mut expected_draws = 0;
    let mut state = tx.state().clone();

    for activation in 1..=5 {
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
        expected_draws += activation.min(4) as u32;
        assert_eq!(next.hand_count(owner), initial_hand + expected_draws);
        assert_eq!(
            next.players
                .get(owner)
                .unwrap()
                .marks
                .iter()
                .find(|(mark, _)| *mark == STDANRAN_USED_MARK)
                .map_or(0, |(_, count)| *count),
            activation.min(3) as i32
        );
        state = next;
    }
}

#[test]
fn stdjiaozi_only_boosts_the_first_damage_when_tied_for_max_hand() {
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(20_260_825, 3));
    let owner = engine.state().turn.current;
    let mut others = engine
        .state()
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id);
    let target = others.next().unwrap();
    let leader = others.next().unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    for player in [owner, target, leader] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    inject(&mut tx, owner, CardKind::Other, Suit::Spade, 1);
    inject(&mut tx, leader, CardKind::Other, Suit::Heart, 2);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDJIAOZI),
        }],
    )
    .unwrap();
    tx.state_mut().turn.current = target;
    let target_state = tx.state_mut().players.get_mut(target).unwrap();
    target_state.hp = 20;
    target_state.max_hp = 20;

    let damage = |tx: &mut Transaction| {
        run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .unwrap();
    };
    damage(&mut tx);
    assert_eq!(tx.state().players.get(target).unwrap().hp, 18);
    damage(&mut tx);
    assert_eq!(tx.state().players.get(target).unwrap().hp, 17);

    let skill_id = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == STDJIAOZI)
        .unwrap()
        .id;
    tx.state_mut()
        .skills
        .get_mut(skill_id)
        .unwrap()
        .turn_fire_count = 0;
    inject(&mut tx, leader, CardKind::Other, Suit::Club, 3);
    damage(&mut tx);
    assert_eq!(tx.state().players.get(target).unwrap().hp, 16);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: leader,
            count: u32::MAX,
        }],
    )
    .unwrap();
    damage(&mut tx);
    assert_eq!(
        tx.state().players.get(target).unwrap().hp,
        15,
        "首段伤害未满足手牌最多也应消耗本回合首次伤害窗口"
    );
}

#[test]
fn stdfuqi_locks_the_whole_sha_use_in_both_directions() {
    const MULTI_SHA: u32 = 4_290_000_001;
    let fixture = content_compiler::compile_source_named(
        "sixiang-test-fixture",
        "skill multi_sha(4290000001) { modifier card_distance_unlimited(sha) override 1; modifier sha_target_limit add 1; }",
    )
    .unwrap();
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.load_package(&fixture).unwrap();
    engine.submit(start(20_260_826, 4));
    let source = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != source)
        .collect();
    let distance_to_source =
        |state: &engine_core::state::GameState, player| match engine_core::query::query(
            state,
            &QueryRequest::Distance {
                from: player,
                to: source,
            },
            &mut engine_core::query::QueryContext::new(),
        ) {
            QueryResponse::Int(distance) => distance,
            other => panic!("unexpected distance response: {other:?}"),
        };
    let near = *others
        .iter()
        .min_by_key(|player| distance_to_source(tx.state(), **player))
        .unwrap();
    let far = *others
        .iter()
        .max_by_key(|player| distance_to_source(tx.state(), **player))
        .unwrap();
    assert_eq!(distance_to_source(tx.state(), near), 1);
    assert!(distance_to_source(tx.state(), far) > 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: source,
            skill_def: SkillDefId::new(STDFUQI),
        }],
    )
    .unwrap();
    assert!(engine_core::query::sha_use_unrespondable_within_distance(
        tx.state(),
        source,
        &[near]
    ));
    assert!(!engine_core::query::sha_use_unrespondable_within_distance(
        tx.state(),
        source,
        &[far]
    ));

    let source_skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == STDFUQI)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill {
                skill: source_skill,
            },
            EffectRequest::AddSkill {
                player: near,
                skill_def: SkillDefId::new(STDFUQI),
            },
        ],
    )
    .unwrap();
    assert!(engine_core::query::sha_use_unrespondable_within_distance(
        tx.state(),
        source,
        &[near]
    ));
    assert!(!engine_core::query::sha_use_unrespondable_within_distance(
        tx.state(),
        source,
        &[far]
    ));

    let target_skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == STDFUQI)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill {
                skill: target_skill,
            },
            EffectRequest::AddSkill {
                player: source,
                skill_def: SkillDefId::new(STDFUQI),
            },
            EffectRequest::AddSkill {
                player: source,
                skill_def: SkillDefId::new(MULTI_SHA),
            },
        ],
    )
    .unwrap();
    for player in [source, near, far] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    let sha = inject(&mut tx, source, CardKind::Sha, Suit::Spade, 7);
    inject(&mut tx, near, CardKind::Shan, Suit::Heart, 8);
    inject(&mut tx, far, CardKind::Shan, Suit::Diamond, 9);
    for target in [near, far] {
        let player = tx.state_mut().players.get_mut(target).unwrap();
        player.hp = 10;
        player.max_hp = 10;
    }
    assert!(engine_core::query::sha_use_unrespondable_within_distance(
        tx.state(),
        source,
        &[near, far]
    ));
    tx.state_mut().turn.phase = Phase::Play;
    let revision = tx.state().revision;
    let (next, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![near, far],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(!next.has_pending_choice());
    assert_eq!(next.players.get(near).unwrap().hp, 9);
    assert_eq!(next.players.get(far).unwrap().hp, 9);
}

#[test]
fn historical_character_joins_standard_roster_and_can_be_dealt() {
    let committed = package();
    let mut baseline = Engine::new();
    assert!(matches!(
        baseline.submit(start(1, 4)),
        CommandResult::Accepted { .. }
    ));
    let expected_character_count =
        baseline.content().characters().count() + committed.character_index.len();
    let mut selected_seed = None;
    for seed in 1..=128 {
        let mut engine = Engine::new();
        engine.load_package(&committed).unwrap();
        let result = engine.submit(start(seed, 4));
        assert!(matches!(result, CommandResult::Accepted { .. }));
        assert_eq!(
            engine.content().characters().count(),
            expected_character_count
        );
        if engine
            .state()
            .players
            .iter_live()
            .any(|player| player.character_def == STD_CAOZHEN)
        {
            selected_seed = Some(seed);
            break;
        }
    }
    assert!(
        selected_seed.is_some(),
        "std_caozhen must be reachable from the combined deterministic roster"
    );
}
}

mod original_sixiang_stdtaoluan {
//! Golden coverage for 标皇甫嵩「讨乱」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDTAOLUAN: u32 = 3_364_139_751;
const STD_HUANGFUSONG: u32 = 3_128_139_054;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdtaoluan.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    target: PlayerId,
    gift: CardId,
    shan: CardId,
    retained: CardId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn inject_hand(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let compiled = content_compiler::compile_source_named("stdtaoluan-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&compiled).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_072_414, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(STDTAOLUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = Phase::End;
    let gift = inject_hand(tx.state_mut(), owner, CardKind::Other, 144_001);
    let shan = inject_hand(tx.state_mut(), target, CardKind::Shan, 144_002);
    let retained = inject_hand(tx.state_mut(), target, CardKind::Sha, 144_003);
    Setup {
        tx,
        owner,
        target,
        gift,
        shan,
        retained,
    }
}

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selection: u32,
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
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_abi144_and_complete_std_huangfusong() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDTAOLUAN
            && trigger.event == "phase_end"
            && trigger.filter == "owner_alive_not_target_hand_equip_nonempty"
            && trigger.optional
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_HUANGFUSONG)
        .unwrap();
    assert_eq!(character.name_key, "std_huangfusong");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [STDTAOLUAN]);

    let compiled = content_compiler::compile_source_named("stdtaoluan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 144);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn accepted_gift_reveals_current_hand_then_discards_every_shan() {
    let setup = setup();
    let owner = setup.owner;
    let target = setup.target;
    let gift = setup.gift;
    let shan = setup.shan;
    let retained = setup.retained;
    let mut tx = setup.tx;
    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end(target)], &mut TriggerFireRecord::new(),),
        Err(EffectError::ChoiceRequested)
    );
    let offered = tx.commit();
    assert_eq!(
        offered.output.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );

    let (state, accepted) = resolve(offered.state, owner, 1, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GiveOneHandOrEquip
    );
    let (state, given) = resolve(state, owner, gift.raw(), 2);
    assert!(matches!(given.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(gift).unwrap().owner, target);
    assert_eq!(
        state.cards.get(gift).unwrap().zone,
        state.zones.find(ZoneKind::Hand, target).unwrap()
    );
    assert_eq!(
        state.cards.get(shan).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert_eq!(
        state.cards.get(retained).unwrap().zone,
        state.zones.find(ZoneKind::Hand, target).unwrap()
    );

    let revealed = given
        .events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::CardsRevealed { cards, to }
                    if to.is_empty()
                        && cards.contains(&gift)
                        && cards.contains(&shan)
                        && cards.contains(&retained)
            )
        })
        .unwrap_or_else(|| panic!("missing complete hand reveal in {:#?}", given.events));
    let discarded = given
        .events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::CardsMoved { cards, .. } if cards == &[shan]
            )
        })
        .unwrap();
    assert!(revealed < discarded);
}

#[test]
fn decline_self_phase_and_empty_owner_do_nothing() {
    let declined = setup();
    let owner = declined.owner;
    let target = declined.target;
    let target_hand = declined.tx.state().hand_cards(target);
    let mut tx = declined.tx;
    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end(target)], &mut TriggerFireRecord::new(),),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = resolve(tx.commit().state, owner, 0, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_cards(target), target_hand);

    let mut self_phase = Transaction::begin(state);
    assert!(dispatch_for_events(
        &mut self_phase,
        &[phase_end(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());

    let mut empty = Transaction::begin(self_phase.commit().state);
    run_effects(
        &mut empty,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    assert!(dispatch_for_events(
        &mut empty,
        &[phase_end(target)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
}
}

mod original_sixiang_stdtiaohe {
//! Golden for 四象费祎「调和」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const STDTIAOHE: u32 = 2_595_769_846;
const STDQIANSU: u32 = 2_964_345_042;
const STD_FEIYI: u32 = 2_878_317_274;
const PROTECT: u32 = 91_846;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/sixiang/stdtiaohe.skill"),
    "\nskill protect(91846) locked { modifier card_discard_protected(equip) override 1; }\n"
);
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup() -> (GameState, [PlayerId; 3]) {
    let package = content_compiler::compile_source_named("stdtiaohe-test", SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(STDTIAOHE as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_718,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players: [PlayerId; 3] = engine.state().players.alive_by_seat().try_into().unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(STDTIAOHE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players)
}

fn equip(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn activate(
    state: GameState,
    owner: PlayerId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == STDTIAOHE)
        .unwrap()
        .id;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
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

fn option_tokens(result: &PipelineResult) -> Vec<u32> {
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
fn committed_package_declares_typed_contract_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == STDTIAOHE)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (2, 2));
    assert!(targets.allow_self);
    assert!(targets.equipment_pair_with_armor);
    let feiyi = package
        .character_index
        .iter()
        .find(|character| character.id == STD_FEIYI)
        .unwrap();
    assert_eq!(feiyi.max_hp, 3);
    assert_eq!(feiyi.skills, vec![STDTIAOHE, STDQIANSU]);
}

#[test]
fn activation_rejects_duplicate_missing_equipment_and_pair_without_armor() {
    let (state, [owner, target0, target1]) = setup();
    let (_, duplicate) = activate(state.clone(), owner, vec![target0, target0], 1);
    assert!(matches!(duplicate.result, CommandResult::Rejected { .. }));
    let (_, empty) = activate(state.clone(), owner, vec![target0, target1], 2);
    assert!(matches!(empty.result, CommandResult::Rejected { .. }));

    let mut no_armor = state;
    equip(&mut no_armor, target0, CardKind::Weapon, 91_001);
    equip(&mut no_armor, target1, CardKind::HorseOff, 91_002);
    let (_, rejected) = activate(no_armor, owner, vec![target0, target1], 3);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}

#[test]
fn armor_on_first_target_is_forced_then_second_equipment_is_discarded() {
    let (mut state, [owner, target0, target1]) = setup();
    let weapon0 = equip(&mut state, target0, CardKind::Weapon, 91_010);
    let armor0 = equip(&mut state, target0, CardKind::Armor, 91_011);
    let weapon1 = equip(&mut state, target1, CardKind::Weapon, 91_012);

    let (state, offered) = activate(state, owner, vec![target0, target1], 10);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DiscardEquipmentPairWithArmor
    );
    assert_eq!(option_tokens(&offered), vec![armor0.raw()]);
    let (_, cancelled) = resolve(state.clone(), owner, Vec::new(), true, 11);
    assert!(matches!(cancelled.result, CommandResult::Rejected { .. }));

    let (state, first) = resolve(state, owner, vec![armor0.raw()], false, 12);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(option_tokens(&first), vec![weapon1.raw()]);
    let (state, completed) = resolve(state, owner, vec![weapon1.raw()], false, 13);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(armor0).unwrap().zone, discard);
    assert_eq!(state.cards.get(weapon1).unwrap().zone, discard);
    assert_ne!(state.cards.get(weapon0).unwrap().zone, discard);
    let moved = completed
        .events
        .iter()
        .filter(|event| matches!(event, engine_api::DomainEvent::CardsMoved { .. }))
        .count();
    assert_eq!(moved, 2);
    assert_eq!(
        state
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.def.raw() == STDTIAOHE)
            .unwrap()
            .use_count,
        1,
        "successful activation consumes its once-per-turn use exactly once"
    );
}

#[test]
fn non_armor_first_selection_forces_armor_on_second_target() {
    let (mut state, [owner, target0, target1]) = setup();
    let weapon0 = equip(&mut state, target0, CardKind::Weapon, 91_020);
    let armor1 = equip(&mut state, target1, CardKind::Armor, 91_021);
    let weapon1 = equip(&mut state, target1, CardKind::Weapon, 91_022);
    let (state, offered) = activate(state, owner, vec![target0, target1], 20);
    assert_eq!(option_tokens(&offered), vec![weapon0.raw()]);
    let (state, first) = resolve(state, owner, vec![weapon0.raw()], false, 21);
    assert_eq!(option_tokens(&first), vec![armor1.raw()]);
    assert!(!option_tokens(&first).contains(&weapon1.raw()));
    let (state, completed) = resolve(state, owner, vec![armor1.raw()], false, 22);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
}

#[test]
fn discard_protection_removes_equipment_from_the_pair_contract() {
    let (mut state, [owner, target0, target1]) = setup();
    equip(&mut state, target0, CardKind::Armor, 91_030);
    equip(&mut state, target1, CardKind::Weapon, 91_031);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: target0,
            skill_def: SkillDefId::new(PROTECT),
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    let (_, rejected) = activate(state, owner, vec![target0, target1], 30);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}
}

mod original_sixiang_stdwanglu {
//! Golden coverage for 标张奋「望橹」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STD_ZHANGFEN: u32 = 3_289_092_799;
const STDWANGLU: u32 = 3_938_702_468;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdwanglu.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    empty: PlayerId,
    skill: engine_api::SkillId,
    equipment: CardId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn skill_id(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == STDWANGLU)
        .unwrap()
}

fn inject_equipment(
    state: &mut GameState,
    owner: PlayerId,
    kind: CardKind,
    weapon_range: u8,
) -> CardId {
    let zone = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(136_001 + u32::from(kind.tag())),
        kind,
        suit: Suit::Spade,
        rank: 5,
        weapon_range,
        zone,
        owner,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(kind: CardKind, weapon_range: u8) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_072_409, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let empty = players[2];
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDWANGLU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let equipment = inject_equipment(tx.state_mut(), target, kind, weapon_range);
    let skill = skill_id(tx.state(), owner);
    Setup {
        state: tx.commit().state,
        owner,
        target,
        empty,
        skill,
        equipment,
    }
}

fn activate(
    setup: &Setup,
    state: GameState,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![card.raw()],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_abi136_and_complete_std_zhangfen() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == STDWANGLU)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max, targets.allow_self), (1, 1, true));
    assert!(targets.has_discardable_equip);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_ZHANGFEN)
        .unwrap();
    assert_eq!(character.name_key, "std_zhangfen");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [STDWANGLU]);

    let compiled = content_compiler::compile_source_named("stdwanglu", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 136);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn weapon_discard_draws_printed_range_after_card_movement() {
    let setup = setup(CardKind::Weapon, 3);
    let before = setup.state.hand_count(setup.target);
    let (offered, activation) = activate(&setup, setup.state.clone(), setup.target, 1);
    assert!(matches!(activation.result, CommandResult::Accepted { .. }));
    let choice = activation.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [setup.equipment.raw()]
    );

    let (done, result) = resolve(offered, setup.owner, setup.equipment, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(done.hand_count(setup.target), before + 3);
    let discard = done
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(done.cards.get(setup.equipment).unwrap().zone, discard);
    let moved = result
        .events
        .iter()
        .position(|event| {
            matches!(event, DomainEvent::CardsMoved { cards, .. } if cards.contains(&setup.equipment))
        })
        .unwrap();
    let drawn = result
        .events
        .iter()
        .position(
            |event| matches!(event, DomainEvent::CardsDrawn { player, .. } if *player == setup.target),
        )
        .unwrap();
    assert!(moved < drawn);
}

#[test]
fn non_weapon_draws_nothing_and_target_without_discardable_equipment_is_rejected() {
    let setup = setup(CardKind::Armor, 0);
    let before_hash = canonical_state_hash(&setup.state);
    let (same, rejected) = activate(&setup, setup.state.clone(), setup.empty, 10);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before_hash);

    let before_hand = setup.state.hand_count(setup.target);
    let (offered, activation) = activate(&setup, setup.state.clone(), setup.target, 11);
    assert!(matches!(activation.result, CommandResult::Accepted { .. }));
    let (done, result) = resolve(offered, setup.owner, setup.equipment, 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(done.hand_count(setup.target), before_hand);
    assert!(!result.events.iter().any(
        |event| matches!(event, DomainEvent::CardsDrawn { player, .. } if *player == setup.target)
    ));
}
}

mod original_sixiang_stdwanlan {
//! Golden coverage for 标贾逵「挽澜」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const STD_JIAKUI: u32 = 3_406_075_592;
const STDZHONGZUO: u32 = 3_465_971_059;
const STDWANLAN: u32 = 3_664_427_038;
const GIFT_PROBE: u32 = 4_200_000_125;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(with_probe: bool) -> (Transaction, PlayerId, PlayerId, Vec<CardId>) {
    let mut engine = Engine::new();
    engine.load_package(&package()).unwrap();
    if with_probe {
        let probe = content_compiler::compile_source_named(
            "stdwanlan-probe",
            r#"
skill gift_probe(4200000125) locked {
  trigger cards_moved owner_received_gift_from_other {
    effect lose_hp(self, 1);
  }
}
"#,
        )
        .unwrap();
        engine.load_package(&probe).unwrap();
    }
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 2_026_072_405,
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDWANLAN),
    });
    if with_probe {
        effects.push(EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(GIFT_PROBE),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    let cards = vec![
        inject(&mut tx, owner, CardKind::Sha, Suit::Heart, 7),
        inject(&mut tx, owner, CardKind::Shan, Suit::Club, 8),
    ];
    {
        let participant = tx.state_mut().players.get_mut(target).unwrap();
        participant.hp = -1;
        participant.dying = true;
    }
    (tx, owner, target, cards)
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(94_000 + rank as u32),
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

fn offer(tx: &mut Transaction, target: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player: target,
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
) -> (GameState, Vec<DomainEvent>) {
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

fn wanlan_instance(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == STDWANLAN)
        .unwrap()
        .id
}

#[test]
fn package_publishes_abi125_skill_and_complete_std_jiakui() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDWANLAN && skill.name == "stdwanlan" && !skill.locked
    }));
    assert!(package.character_index.iter().any(|character| {
        character.id == STD_JIAKUI
            && character.name_key == "std_jiakui"
            && character.max_hp == 3
            && character.skills == [STDZHONGZUO, STDWANLAN]
    }));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDWANLAN)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        (
            "dying",
            "owner_alive_other_dying_hand_nonempty_once_per_game",
            true
        )
    );
}

#[test]
fn accepting_gives_every_hand_card_then_recovers_after_nested_movement_trigger() {
    let (mut tx, owner, target, cards) = setup(true);
    assert_eq!(
        offer(&mut tx, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (saved, events) = resolve(&tx, owner, true, 1);
    assert_eq!(saved.hand_count(owner), 0);
    assert!(cards
        .iter()
        .all(|card| saved.cards.get(*card).unwrap().owner == target));
    assert_eq!(saved.players.get(target).unwrap().hp, 1);
    assert_eq!(
        saved
            .skills
            .get(wanlan_instance(&saved, owner))
            .unwrap()
            .lifetime_fire_count,
        1
    );
    let moved = events
        .iter()
        .position(|event| matches!(event, DomainEvent::CardsMoved { .. }))
        .unwrap();
    let nested_loss = events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::DamageDealt {
                    target: damaged,
                    amount: 1,
                    is_lose_hp: true,
                    ..
                } if *damaged == target
            )
        })
        .unwrap();
    let recovered = events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::HpRecovered {
                    player,
                    amount: 3,
                    ..
                } if *player == target
            )
        })
        .unwrap();
    assert!(moved < nested_loss && nested_loss < recovered);
}

#[test]
fn declining_does_not_consume_and_empty_self_or_spent_cases_do_not_offer() {
    let (mut tx, owner, target, _) = setup(false);
    let original_hand_count = tx.state().hand_count(owner);
    assert_eq!(
        offer(&mut tx, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (declined, _) = resolve(&tx, owner, false, 2);
    let skill = wanlan_instance(&declined, owner);
    assert_eq!(declined.skills.get(skill).unwrap().lifetime_fire_count, 0);
    assert_eq!(declined.hand_count(owner), original_hand_count);
    assert_eq!(declined.players.get(target).unwrap().hp, -1);

    let mut accepted_tx = Transaction::begin(declined);
    assert_eq!(
        offer(&mut accepted_tx, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (accepted, _) = resolve(&accepted_tx, owner, true, 3);
    let mut spent = Transaction::begin(accepted);
    let extra = inject(&mut spent, owner, CardKind::Tao, Suit::Heart, 9);
    spent.state_mut().players.get_mut(target).unwrap().hp = 0;
    spent.state_mut().players.get_mut(target).unwrap().dying = true;
    assert!(offer(&mut spent, target).unwrap().is_empty());
    assert!(!spent.state().has_pending_choice());
    assert_eq!(spent.state().cards.get(extra).unwrap().owner, owner);

    let (mut empty, empty_owner, empty_target, _) = setup(false);
    let cards = empty.state().hand_cards(empty_owner);
    let target_hand = empty
        .state()
        .zones
        .find(ZoneKind::Hand, empty_target)
        .unwrap();
    let owner_hand = empty
        .state()
        .zones
        .find(ZoneKind::Hand, empty_owner)
        .unwrap();
    run_effects(
        &mut empty,
        vec![EffectRequest::MoveCards {
            cards,
            to: target_hand,
            from: Some(owner_hand),
        }],
    )
    .unwrap();
    assert!(offer(&mut empty, empty_target).unwrap().is_empty());
    assert!(offer(&mut empty, empty_owner).unwrap().is_empty());
}
}

mod original_sixiang_stdxingfa {
//! Golden coverage for 标钟会「兴伐」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_ZHONGHUI: u32 = 2_397_163_146;
const STDXINGFA: u32 = 2_490_051_933;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdxingfa.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn phase_prepare(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::Prepare,
    }
}

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdxingfa-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_722, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDXINGFA),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, target)
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
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
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn committed_package_exposes_complete_character_and_abi76_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDXINGFA
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target_hand_at_least_hp_other_alive"
            && trigger.optional
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_ZHONGHUI)
        .unwrap();
    assert_eq!(character.name_key, "std_zhonghui");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, vec![STDXINGFA]);
}

#[test]
fn equality_offers_target_choice_and_selected_other_takes_one_damage() {
    let (mut tx, owner, target) = setup();
    let hand = tx.state().hand_count(owner);
    tx.state_mut().players.get_mut(owner).unwrap().hp = hand as i32;
    let target_hp = tx.state().players.get(target).unwrap().hp;

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[phase_prepare(owner)],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.state().clone(), owner, vec![1], 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDamage
    );
    assert_eq!(state.resolution.pending_observer, owner.raw());

    let state = resolve(state, owner, vec![target.raw()], 2);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
}

#[test]
fn fewer_hand_cards_or_no_other_living_player_suppresses_the_offer() {
    let (mut below, owner, _) = setup();
    run_effects(
        &mut below,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    run_effects(
        &mut below,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 3,
        }],
    )
    .unwrap();
    below.state_mut().players.get_mut(owner).unwrap().hp = 4;
    let effects = dispatch_for_events(
        &mut below,
        &[phase_prepare(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!below.state().has_pending_choice());

    let (mut alone, owner, target) = setup();
    let hand = alone.state().hand_count(owner);
    alone.state_mut().players.get_mut(owner).unwrap().hp = hand as i32;
    alone.state_mut().players.get_mut(target).unwrap().alive = false;
    let effects = dispatch_for_events(
        &mut alone,
        &[phase_prepare(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!alone.state().has_pending_choice());
}
}

mod original_sixiang_stdxiongsuan {
//! Golden coverage for 标李傕「凶算」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_LIJUE: u32 = 2_852_318_654;
const STDXIONGSUAN: u32 = 3_084_450_928;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdxiongsuan.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn phase_prepare(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::Prepare,
    }
}

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_074_403, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, tied, lower) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDXIONGSUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    for (player, hp) in [(owner, 4), (tied, 4), (lower, 3)] {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = hp;
        participant.max_hp = participant.max_hp.max(hp as u32);
    }
    (tx, owner, tied, lower)
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_and_runtime_force_same_hp_targets_then_damage_in_seat_order() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDXIONGSUAN && skill.name == "stdxiongsuan" && skill.locked
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_LIJUE)
        .unwrap();
    assert_eq!(character.name_key, "std_lijue");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 5);
    assert_eq!(character.skills, [STDXIONGSUAN]);

    let compiled = content_compiler::compile_source_named("stdxiongsuan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 135);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);

    let (mut tx, owner, tied, lower) = setup();
    let effects = dispatch_for_events(
        &mut tx,
        &[phase_prepare(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = tx.commit();
    let choice = committed.output.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!((choice.min_count, choice.max_count), (1, 2));
    let options: Vec<_> = choice.options.iter().map(|option| option.token).collect();
    assert_eq!(options, [owner.raw(), tied.raw()]);
    assert!(!options.contains(&lower.raw()));

    let (state, output) = resolve(committed.state, owner, vec![tied.raw(), owner.raw()]);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert_eq!(state.players.get(tied).unwrap().hp, 3);
    assert_eq!(state.players.get(lower).unwrap().hp, 3);
    let damaged: Vec<_> = output
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::DamageDealt { source, target, .. } if *source == owner => Some(*target),
            _ => None,
        })
        .collect();
    assert_eq!(damaged, [owner, tied]);
}

#[test]
fn trigger_does_not_fire_when_owner_is_below_the_table_maximum() {
    let (mut tx, owner, _, higher) = setup();
    tx.state_mut().players.get_mut(higher).unwrap().hp = 5;
    let effects = dispatch_for_events(
        &mut tx,
        &[phase_prepare(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
}
}

mod original_sixiang_stdyimou {
//! Golden coverage for 标鲍信「毅谋」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, CardMoveReason, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult,
    DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDYIMOU: u32 = 4_237_148_705;
const STDMUTAO: u32 = 3_513_095_743;
const STD_BAOXIN: u32 = 2_419_830_352;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdyimou.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn inject(state: &mut GameState, owner: PlayerId, zone_kind: ZoneKind, def: u32) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: if zone_kind == ZoneKind::Equip {
            CardKind::Weapon
        } else {
            CardKind::Shan
        },
        suit: Suit::Club,
        rank: 6,
        weapon_range: u8::from(zone_kind == ZoneKind::Equip),
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(
    card_zone: Option<ZoneKind>,
) -> (Transaction, PlayerId, PlayerId, PlayerId, Option<CardId>) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_411, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(STDYIMOU),
    });
    run_effects(&mut tx, effects).unwrap();
    let card = card_zone.map(|zone| inject(tx.state_mut(), owner, zone, 138_001));
    (tx, owner, target, other, card)
}

fn damage(owner: PlayerId, source: PlayerId, amount: u32) -> DomainEvent {
    DomainEvent::DamageDealt {
        source,
        target: owner,
        amount,
        is_lose_hp: false,
        nature: 0,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
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

fn trigger(mut tx: Transaction, owner: PlayerId, source: PlayerId) -> (GameState, PipelineResult) {
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[damage(owner, source, 1)],
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
    resolve(committed.state, owner, vec![1], 1)
}

#[test]
fn package_publishes_abi138_with_complete_baoxin() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDYIMOU && skill.name == "stdyimou" && !skill.locked
    }));
    let baoxin = package
        .character_index
        .iter()
        .find(|character| character.id == STD_BAOXIN)
        .unwrap();
    assert_eq!(baoxin.name_key, "std_baoxin");
    assert_eq!(baoxin.max_hp, 4);
    assert_eq!(baoxin.skills, [STDYIMOU, STDMUTAO]);

    let compiled = content_compiler::compile_source_named("stdyimou", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 138);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn damage_choice_selects_other_then_gives_current_hand_card() {
    let (tx, owner, target, other, card) = setup(Some(ZoneKind::Hand));
    let card = card.unwrap();
    let (selecting_target, target_output) = trigger(tx, owner, other);
    assert!(matches!(
        target_output.result,
        CommandResult::Accepted { .. }
    ));
    let target_choice = target_output.choice.as_ref().unwrap();
    assert_eq!(target_choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(
        target_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [target.raw(), other.raw()]
    );

    let (selecting_card, card_output) = resolve(selecting_target, owner, vec![target.raw()], 2);
    assert!(matches!(card_output.result, CommandResult::Accepted { .. }));
    assert_eq!(
        selecting_card.resolution.continuation_kind,
        ContinuationKind::GiveOneHandOrEquip
    );
    let card_choice = card_output.choice.as_ref().unwrap();
    assert_eq!(card_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        card_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [card.raw()]
    );

    let target_before = selecting_card.hand_count(target);
    let (done, given) = resolve(selecting_card, owner, vec![card.raw()], 3);
    assert!(matches!(given.result, CommandResult::Accepted { .. }));
    assert_eq!(done.cards.get(card).unwrap().owner, target);
    assert_eq!(done.hand_count(target), target_before + 1);
    assert!(given.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved {
            cards,
            reason: CardMoveReason::Give,
            ..
        } if cards == &vec![card]
    )));
}

#[test]
fn equipment_is_eligible_but_no_card_or_zero_damage_does_not_trigger() {
    let (tx, owner, target, other, equip) = setup(Some(ZoneKind::Equip));
    let equip = equip.unwrap();
    let (selecting_target, _) = trigger(tx, owner, other);
    let (selecting_card, card_output) = resolve(selecting_target, owner, vec![target.raw()], 4);
    assert_eq!(
        card_output
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [equip.raw()]
    );
    let (done, _) = resolve(selecting_card, owner, vec![equip.raw()], 5);
    assert_eq!(done.cards.get(equip).unwrap().owner, target);
    assert_eq!(
        done.cards.get(equip).unwrap().zone,
        done.zones.find(ZoneKind::Hand, target).unwrap()
    );

    let (mut empty, empty_owner, empty_target, _, _) = setup(None);
    assert!(dispatch_for_events(
        &mut empty,
        &[damage(empty_owner, empty_target, 1)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
    assert!(dispatch_for_events(
        &mut empty,
        &[damage(empty_owner, empty_target, 0)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
}
}

mod original_sixiang_stdyinzu {
//! Golden for 四象「荫族」按存活拥有者叠加的全场攻击范围光环。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const STDYINZU: u32 = 3_812_447_605;
const STDDUANBING: u32 = 2_170_694_873;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDYINZU && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(STDYINZU as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: STDYINZU as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(STDYINZU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players[0], players[1], players[2])
}

fn attack_range(tx: &Transaction, player: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::AttackRange { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

fn equip_long_weapon(tx: &mut Transaction, player: PlayerId) {
    let weapon = tx.state().hand_cards(player)[0];
    {
        let card = tx.state_mut().cards.get_mut(weapon).unwrap();
        card.kind = CardKind::Weapon;
        card.weapon_range = 5;
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
}

#[test]
fn every_player_uses_their_own_hand_hp_condition() {
    let (mut tx, owner, equal, _) = isolated();
    let owner_hand = tx.state().hand_count(owner) as i32;
    let equal_hand = tx.state().hand_count(equal) as i32;
    tx.state_mut().players.get_mut(owner).unwrap().hp = owner_hand - 1;
    tx.state_mut().players.get_mut(equal).unwrap().hp = equal_hand;

    assert_eq!(attack_range(&tx, owner), 2);
    assert_eq!(attack_range(&tx, equal), 0);

    equip_long_weapon(&mut tx, equal);
    assert_eq!(attack_range(&tx, equal), 4);
}

#[test]
fn living_owners_stack_and_dead_owners_stop_contributing() {
    let (mut tx, first_owner, target, second_owner) = isolated();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: second_owner,
            skill_def: SkillDefId::new(STDYINZU),
        }],
    )
    .unwrap();
    let target_hand = tx.state().hand_count(target) as i32;
    tx.state_mut().players.get_mut(target).unwrap().hp = target_hand - 1;

    assert_eq!(attack_range(&tx, target), 3);

    tx.state_mut().players.get_mut(second_owner).unwrap().alive = false;
    assert_eq!(attack_range(&tx, target), 2);

    tx.state_mut().players.get_mut(first_owner).unwrap().alive = false;
    assert_eq!(attack_range(&tx, target), 1);
}

#[test]
fn global_additions_compose_after_local_attack_range_override() {
    let (mut tx, _, target, second_owner) = isolated();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: second_owner,
                skill_def: SkillDefId::new(STDYINZU),
            },
            EffectRequest::AddSkill {
                player: target,
                skill_def: SkillDefId::new(STDDUANBING),
            },
        ],
    )
    .unwrap();
    let target_hand = tx.state().hand_count(target) as i32;
    tx.state_mut().players.get_mut(target).unwrap().hp = target_hand - 1;

    assert_eq!(attack_range(&tx, target), 3);

    tx.state_mut().players.get_mut(target).unwrap().hp = target_hand;
    assert_eq!(attack_range(&tx, target), 0);
}
}

mod original_sixiang_stdzaoli {
//! Golden coverage for 标孙翊「躁厉」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_SUNYI: u32 = 2_572_349_242;
const STDZAOLI: u32 = 2_735_843_050;
const MOVEMENT_HELPER: u32 = 4_000_000_011;
const DRAW_HELPER: u32 = 4_000_000_012;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdzaoli.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn phase_prepare(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::Prepare,
    }
}

fn setup(with_barrier_helpers: bool) -> (Transaction, PlayerId) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_072_410, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    if with_barrier_helpers {
        let helpers = content_compiler::compile_source(
            "skill stdzaoli_movement_test(4000000011) locked { trigger cards_moved owner_from_hand_empty { effect lose_hp(self, 1); } } skill stdzaoli_draw_test(4000000012) locked { trigger cards_drawn owner_is_target { effect recover_hp(self, 1); } }",
        )
        .unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&helpers).unwrap();
    }
    let owner = tx.state().players.alive_by_seat()[0];
    let players = tx.state().players.alive_by_seat();
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
        skill_def: SkillDefId::new(STDZAOLI),
    });
    if with_barrier_helpers {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(MOVEMENT_HELPER),
        });
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DRAW_HELPER),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.hp = 4;
    participant.max_hp = 4;
    (tx, owner)
}

fn inject(tx: &mut Transaction, owner: PlayerId, zone_kind: ZoneKind, def: u32) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: if zone_kind == ZoneKind::Equip {
            CardKind::Armor
        } else {
            CardKind::Other
        },
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn dispatch_prepare(
    tx: &mut Transaction,
    owner: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let effects =
        dispatch_for_events(tx, &[phase_prepare(owner)], &mut TriggerFireRecord::new()).unwrap();
    run_effects(tx, effects)
}

fn resolve_zone(state: GameState, owner: PlayerId, zone: u32) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![zone],
                cancelled: false,
            },
        },
    )
}

fn zone_cards(state: &GameState, owner: PlayerId, kind: ZoneKind) -> Vec<CardId> {
    let zone = state.zones.find(kind, owner).unwrap();
    state.zones.get(zone).unwrap().cards.clone()
}

#[test]
fn package_publishes_abi133_skill_and_complete_std_sunyi() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDZAOLI && skill.name == "stdzaoli" && skill.locked
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDZAOLI
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target"
            && !trigger.optional
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_SUNYI)
        .unwrap();
    assert_eq!(character.name_key, "std_sunyi");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [STDZAOLI]);

    let compiled = content_compiler::compile_source_named("stdzaoli", SOURCE).unwrap();
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
fn two_nonempty_zones_force_a_zone_choice_and_only_discard_that_zone() {
    let (mut tx, owner) = setup(false);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let hand_a = inject(&mut tx, owner, ZoneKind::Hand, 133_001);
    let hand_b = inject(&mut tx, owner, ZoneKind::Hand, 133_002);
    let armor = inject(&mut tx, owner, ZoneKind::Equip, 133_003);

    assert_eq!(
        dispatch_prepare(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp
    );
    let committed = tx.commit();
    let choice = committed.output.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert!(!choice.allow_cancel);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );

    let (state, result) = resolve_zone(committed.state, owner, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, 1);
    assert_eq!(zone_cards(&state, owner, ZoneKind::Hand).len(), 4);
    assert_eq!(zone_cards(&state, owner, ZoneKind::Equip), [armor]);
    let discard = zone_cards(&state, PlayerId::NONE, ZoneKind::DiscardPile);
    assert!(discard.contains(&hand_a));
    assert!(discard.contains(&hand_b));
}

#[test]
fn discard_and_draw_triggers_finish_before_lost_hp_is_counted_and_final_hp_is_lost() {
    let (mut tx, owner) = setup(true);
    let hand = inject(&mut tx, owner, ZoneKind::Hand, 133_011);

    let events = dispatch_prepare(&mut tx, owner).unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
    assert_eq!(zone_cards(tx.state(), owner, ZoneKind::Hand).len(), 2);
    assert!(zone_cards(tx.state(), PlayerId::NONE, ZoneKind::DiscardPile).contains(&hand));

    let movement_loss = events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::DamageDealt {
                    target,
                    is_lose_hp: true,
                    ..
                } if *target == owner
            )
        })
        .unwrap();
    let draw = events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::CardsDrawn { player, cards }
                    if *player == owner && cards.len() == 2
            )
        })
        .unwrap();
    let recovery = events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::HpRecovered { player, amount, .. }
                    if *player == owner && *amount == 1
            )
        })
        .unwrap();
    let final_loss = events
        .iter()
        .rposition(|event| {
            matches!(
                event,
                DomainEvent::DamageDealt {
                    target,
                    is_lose_hp: true,
                    ..
                } if *target == owner
            )
        })
        .unwrap();
    assert!(movement_loss < draw);
    assert!(draw < recovery);
    assert!(recovery < final_loss);
}

#[test]
fn no_discardable_hand_or_equipment_cards_is_a_noop() {
    let (mut tx, owner) = setup(false);
    let before_hp = tx.state().players.get(owner).unwrap().hp;
    let before_hand = zone_cards(tx.state(), owner, ZoneKind::Hand);
    let events = dispatch_prepare(&mut tx, owner).unwrap();
    assert!(!events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::CardsMoved { .. }
                | DomainEvent::CardsDrawn { .. }
                | DomainEvent::DamageDealt { .. }
        )
    }));
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(owner).unwrap().hp, before_hp);
    assert_eq!(zone_cards(tx.state(), owner, ZoneKind::Hand), before_hand);
}
}

mod original_sixiang_stdzhaoxin {
//! Golden coverage for 标司马昭「昭心」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_SIMAZHAO: u32 = 2_365_843_011;
const STDZHAOXIN: u32 = 2_955_343_028;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdzhaoxin.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn phase_prepare(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::Prepare,
    }
}

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdzhaoxin-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_722, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
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
        skill_def: SkillDefId::new(STDZHAOXIN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    (tx, owner, target)
}

fn inject(tx: &mut Transaction, owner: PlayerId, def: u32, suit: Suit) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
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
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, actor: PlayerId, selection: PlayerId) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_exact_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDZHAOXIN && skill.name == "stdzhaoxin" && skill.locked
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_SIMAZHAO)
        .unwrap();
    assert_eq!(character.name_key, "std_simazhao");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [STDZHAOXIN]);
}

#[test]
fn non_uniform_hand_is_revealed_without_damage_choice() {
    let (mut tx, owner, target) = setup();
    let red = inject(&mut tx, owner, 96_001, Suit::Heart);
    let black = inject(&mut tx, owner, 96_002, Suit::Spade);
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let effects = dispatch_for_events(
        &mut tx,
        &[phase_prepare(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let events = run_effects(&mut tx, effects).unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(target).unwrap().hp, target_hp);
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &vec![red, black] && to.is_empty()
    )));
}

#[test]
fn uniform_hand_forces_any_living_target_and_allows_self_damage() {
    let (mut tx, owner, target) = setup();
    let red_a = inject(&mut tx, owner, 96_011, Suit::Heart);
    let red_b = inject(&mut tx, owner, 96_012, Suit::Diamond);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let effects = dispatch_for_events(
        &mut tx,
        &[phase_prepare(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseOtherDamage
    );
    let committed = tx.commit();
    assert!(committed.output.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &vec![red_a, red_b] && to.is_empty()
    )));
    let choice = committed.output.choice.unwrap();
    assert!(!choice.allow_cancel);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    let options: Vec<_> = choice
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(options.contains(&owner.raw()));
    assert!(options.contains(&target.raw()));

    let state = resolve(committed.state, owner, owner);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
}
}

mod original_sixiang_stdzhijie {
//! Golden coverage for 标甘夫人「智诫」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_GANFUREN: u32 = 2_343_280_510;
const STDZHIJIE: u32 = 2_635_655_868;
const STDSHUSHENX: u32 = 3_633_500_345;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdzhijie.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

struct Setup {
    state: GameState,
    owner: PlayerId,
    phase_player: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_072_413, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let phase_player = players[1];
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
        skill_def: SkillDefId::new(STDZHIJIE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = phase_player;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
        phase_player,
        other,
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

fn skill_id(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == STDZHIJIE)
        .unwrap()
        .id
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
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

fn dispatch_card_used(
    mut state: GameState,
    player: PlayerId,
    card: CardId,
    target: PlayerId,
) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player,
            card,
            targets: vec![target],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    state = tx.commit().state;
    state
}

#[test]
fn package_publishes_abi140_and_complete_ganfuren() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDZHIJIE && skill.name == "stdzhijie" && !skill.locked
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_GANFUREN)
        .unwrap();
    assert_eq!(character.skills, [STDZHIJIE, STDSHUSHENX]);

    let compiled = content_compiler::compile_source_named("stdzhijie", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 140);
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
fn bound_category_draws_only_for_first_three_matching_uses_in_that_phase() {
    let mut setup = setup();
    let revealed = inject(
        &mut setup.state,
        setup.phase_player,
        CardKind::TrickWuzhong,
        99_401,
    );
    let matching: Vec<_> = (0..4)
        .map(|offset| {
            inject(
                &mut setup.state,
                setup.phase_player,
                CardKind::TrickWuxie,
                99_410 + offset,
            )
        })
        .collect();
    let basic = inject(&mut setup.state, setup.phase_player, CardKind::Sha, 99_420);

    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: setup.phase_player,
                phase: TurnPhaseTag::Play,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = tx.commit();
    assert_eq!(committed.output.choice.unwrap().kind, ChoiceKind::Confirm);
    let (state, confirmed) = resolve(committed.state, setup.owner, vec![1], 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseRevealOtherHandCardRememberCategory
    );
    assert_eq!(confirmed.choice.unwrap().kind, ChoiceKind::SelectCards);
    let (mut state, selected) = resolve(state, setup.owner, vec![revealed.raw()], 2);
    assert!(matches!(selected.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state
            .skills
            .get(skill_id(&state, setup.owner))
            .unwrap()
            .state_blob,
        [
            b"BCT1".as_slice(),
            &setup.phase_player.raw().to_le_bytes(),
            &[2]
        ]
        .concat()
    );

    let before = state.hand_count(setup.owner);
    state = dispatch_card_used(state, setup.phase_player, basic, setup.other);
    assert_eq!(state.hand_count(setup.owner), before);
    for (index, card) in matching.into_iter().enumerate() {
        state = dispatch_card_used(state, setup.phase_player, card, setup.other);
        assert_eq!(
            state.hand_count(setup.owner),
            before + u32::try_from((index + 1).min(3)).unwrap()
        );
    }

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: setup.phase_player,
            phase: Phase::Discard.to_u8(),
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .skills
        .get(skill_id(tx.state(), setup.owner))
        .unwrap()
        .state_blob
        .is_empty());
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: setup.phase_player,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.phase_player,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
}
}

mod original_sixiang_stdzhiyi {
//! Golden for 四象张翼「执义」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, CardKind, ContinuationKind, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDZHIYI: u32 = 3_959_114_365;
const STD_ZHANGYI: u32 = 3_037_060_961;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdzhiyi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdzhiyi-test", SOURCE).unwrap();
    let mut state = engine_core::state::GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, STDZHIYI as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, other) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDZHIYI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::End;
    (tx, owner, other)
}

fn remember_sha_use(tx: &mut Transaction, user: PlayerId, target: PlayerId) {
    let card = tx.state().hand_cards(user)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Sha;
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets: vec![target],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
}

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn fire(tx: &mut Transaction, ending_player: PlayerId) -> Result<(), EffectError> {
    let effects = dispatch_for_events(
        tx,
        &[phase_end(ending_player)],
        &mut TriggerFireRecord::new(),
    )?;
    run_effects(tx, effects).map(|_| ())
}

fn resolve(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> engine_core::state::GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let expected_revision = ExpectedRevision(state.revision);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision,
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn committed_package_declares_filter_effect_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDZHIYI
            && trigger.event == "phase_end"
            && trigger.filter == "owner_alive_used_sha_this_turn"
            && !trigger.optional
    }));
    assert!(package
        .character_index
        .iter()
        .any(|character| { character.id == STD_ZHANGYI && character.skills == vec![STDZHIYI] }));
}

#[test]
fn another_players_sha_does_not_trigger_for_the_owner() {
    let (mut tx, owner, other) = setup();
    remember_sha_use(&mut tx, other, owner);
    assert_eq!(
        tx.state().turn.flag(turn_flag::sha_used_by_player(owner)),
        0
    );
    fire(&mut tx, other).unwrap();
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn cancelling_virtual_sha_draws_one_and_does_not_consume_sha_count() {
    let (mut tx, owner, other) = setup();
    remember_sha_use(&mut tx, owner, other);
    let hand = tx.state().hand_count(owner);
    assert_eq!(fire(&mut tx, other), Err(EffectError::ChoiceRequested));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
    assert_eq!(tx.state().resolution.pending_observer, owner.raw());
    let state = resolve(tx.state().clone(), owner, Vec::new(), true, 1);
    assert_eq!(state.hand_count(owner), hand + 1);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 0);
}

#[test]
fn selected_virtual_sha_uses_normal_targeting_without_counting_and_no_target_draws() {
    let (mut use_tx, owner, other) = setup();
    remember_sha_use(&mut use_tx, owner, other);
    let hand = use_tx.state().hand_count(owner);
    assert_eq!(fire(&mut use_tx, other), Err(EffectError::ChoiceRequested));
    let state = resolve(use_tx.state().clone(), owner, vec![other.raw()], false, 2);
    assert_eq!(state.hand_count(owner), hand);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 0);

    let (mut no_target, owner, other) = setup();
    remember_sha_use(&mut no_target, owner, other);
    let hand = no_target.state().hand_count(owner);
    no_target.state_mut().players.get_mut(other).unwrap().alive = false;
    no_target.state_mut().players.get_mut(other).unwrap().dying = false;
    fire(&mut no_target, other).unwrap();
    assert!(!no_target.state().has_pending_choice());
    assert_eq!(no_target.state().hand_count(owner), hand + 1);
}
}

mod original_sixiang_stdzhongshi {
//! Golden for 四象「忠事」比较伤害来源与目标横置状态的加伤语义。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::intent::{resolve_damage_intent, DamageIntent};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const STDZHONGSHI: u32 = 3_843_853_491;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDZHONGSHI && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(STDZHONGSHI as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: STDZHONGSHI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(STDZHONGSHI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players[0], players[1], players[2])
}

fn set_linked(tx: &mut Transaction, player: PlayerId, linked: bool) {
    let marks = &mut tx.state_mut().players.get_mut(player).unwrap().marks;
    marks.retain(|(mark, _)| *mark != engine_core::content::MARK_CHAIN);
    if linked {
        marks.push((engine_core::content::MARK_CHAIN, 1));
        marks.sort_by_key(|(mark, _)| *mark);
    }
}

fn resolved_amount(tx: &Transaction, source: PlayerId, target: PlayerId) -> u32 {
    resolve_damage_intent(
        tx.state(),
        DamageIntent {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            cancelled: false,
        },
    )
    .amount
}

#[test]
fn bonus_applies_only_while_source_and_target_linked_states_differ() {
    let (mut tx, source, target, _) = isolated();

    assert_eq!(resolved_amount(&tx, source, target), 1);
    set_linked(&mut tx, target, true);
    assert_eq!(resolved_amount(&tx, source, target), 2);
    set_linked(&mut tx, source, true);
    assert_eq!(resolved_amount(&tx, source, target), 1);
    set_linked(&mut tx, target, false);
    assert_eq!(resolved_amount(&tx, source, target), 2);
}

#[test]
fn source_skill_does_not_modify_unrelated_or_sourceless_damage() {
    let (mut tx, source, target, third) = isolated();
    set_linked(&mut tx, target, true);
    set_linked(&mut tx, third, true);

    assert_eq!(resolved_amount(&tx, source, target), 2);
    assert_eq!(resolved_amount(&tx, third, target), 1);
    assert_eq!(resolved_amount(&tx, PlayerId::NONE, target), 1);
}

#[test]
fn owning_the_skill_on_the_damage_target_does_not_add_damage() {
    let (mut tx, source, target, _) = isolated();
    let source_skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == source && skill.def.raw() == STDZHONGSHI)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill {
                skill: source_skill,
            },
            EffectRequest::AddSkill {
                player: target,
                skill_def: SkillDefId::new(STDZHONGSHI),
            },
        ],
    )
    .unwrap();
    set_linked(&mut tx, target, true);

    assert_eq!(resolved_amount(&tx, source, target), 1);
}
}

mod original_sixiang_stdzhongzuo {
//! Golden coverage for 标贾逵「忠佐」turn-scoped damage history.

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDZHONGZUO: u32 = 3_465_971_059;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdzhongzuo.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdzhongzuo-test", SOURCE).unwrap();
    let mut state = engine_core::state::GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_717, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, ending_player, third) = (players[0], players[1], players[2]);
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
            skill_def: SkillDefId::new(STDZHONGZUO),
        },
        EffectRequest::SetTurnState {
            current: ending_player,
            phase: Phase::End.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, ending_player, third)
}

fn end_turn(tx: &mut Transaction, next: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::SetTurnState {
            current: next,
            phase: Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
}

#[test]
fn package_exposes_locked_turn_damage_participant_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDZHONGZUO
            && trigger.event == "turn_ended"
            && trigger.filter == "owner_alive_dealt_or_received_damage_this_turn"
            && !trigger.optional
    }));
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDZHONGZUO && skill.locked));
}

#[test]
fn noncurrent_owner_dealing_damage_draws_after_the_ending_player_in_seat_order() {
    let (mut tx, owner, ending_player, third) = setup();
    let owner_hand = tx.state().hand_count(owner);
    let ending_hand = tx.state().hand_count(ending_player);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: third,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(tx.state().turn.player_dealt_damage(owner));

    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let cards = &tx.state().zones.get(draw).unwrap().cards;
    let first = cards[cards.len() - 1];
    let second = cards[cards.len() - 2];
    end_turn(&mut tx, third);

    assert_eq!(tx.state().hand_count(ending_player), ending_hand + 1);
    assert_eq!(tx.state().hand_count(owner), owner_hand + 1);
    assert!(tx.state().hand_cards(ending_player).contains(&first));
    assert!(tx.state().hand_cards(owner).contains(&second));
    assert_eq!(tx.state().turn.flag(turn_flag::DAMAGE_SOURCES), 0);
    assert_eq!(tx.state().turn.flag(turn_flag::DAMAGED_PLAYERS), 0);
}

#[test]
fn receiving_damage_triggers_but_hp_loss_and_a_clean_following_turn_do_not() {
    let (mut tx, owner, ending_player, third) = setup();
    let owner_hand = tx.state().hand_count(owner);
    let ending_hand = tx.state().hand_count(ending_player);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: ending_player,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(tx.state().turn.player_received_damage(owner));
    end_turn(&mut tx, third);
    assert_eq!(tx.state().hand_count(owner), owner_hand + 1);
    assert_eq!(tx.state().hand_count(ending_player), ending_hand + 1);

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: third,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: owner,
                amount: 1,
                is_lose_hp: true,
                nature: 0,
            },
        ],
    )
    .unwrap();
    let clean_owner_hand = tx.state().hand_count(owner);
    let clean_ending_hand = tx.state().hand_count(third);
    end_turn(&mut tx, ending_player);
    assert_eq!(tx.state().hand_count(owner), clean_owner_hand);
    assert_eq!(tx.state().hand_count(third), clean_ending_hand);
}
}

mod original_sixiang_stdzhuan {
//! Golden coverage for 标孟达「赚」.

use content_compiler::compile_source_named;
use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDZHUAN: u32 = 4_034_502_794;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/sixiang/stdzhuan.skill"),
    r#"
skill draw_followup(999) locked {
  trigger cards_drawn owner_gains_multiple_outside_own_draw {
    effect draw_cards(self, 1);
  }
}
"#,
);

fn setup(extra_draw_trigger: bool) -> (GameState, PlayerId, PlayerId) {
    let package = compile_source_named("stdzhuan-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_715, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(STDZHUAN),
    });
    if extra_draw_trigger {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(999),
        });
        effects.push(EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Play.to_u8(),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, source)
}

fn damage(
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
            nature: 0,
        }],
    );
    (tx.commit().state, result)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selection: u32,
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
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

#[test]
fn first_damage_draws_three_then_source_takes_one_and_second_damage_does_not_fire() {
    let (state, owner, source) = setup(false);
    let old_card = state.hand_cards(owner)[0];
    let owner_hand = state.hand_count(owner);
    let source_hand = state.hand_count(source);
    let (state, first) = damage(state, source, owner);
    assert_eq!(first, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::PickTargetCard
    );
    assert_eq!(state.resolution.pending_observer, source.raw());
    assert_eq!(state.hand_count(owner), owner_hand + 3);

    let (state, resolved) = resolve(state, source, old_card.raw(), 1);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(old_card).unwrap().owner, source);
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    assert_eq!(state.hand_count(source), source_hand + 1);

    let hand_after_first = state.hand_count(owner);
    let (state, second) = damage(state, source, owner);
    assert!(second.is_ok());
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand_after_first);
}

#[test]
fn unsourced_damage_only_draws_three() {
    let (state, owner, _) = setup(false);
    let hand = state.hand_count(owner);
    let (state, result) = damage(state, PlayerId::NONE, owner);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand + 3);
}

#[test]
fn draw_triggers_finish_before_source_chooses_a_card() {
    let (state, owner, source) = setup(true);
    let hand = state.hand_count(owner);
    let (state, result) = damage(state, source, owner);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(state.hand_count(owner), hand + 4);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::PickTargetCard
    );
    assert_eq!(state.resolution.continuation_payload[0], owner.raw());
    assert_eq!(state.resolution.continuation_payload[1], source.raw());
}
}

mod original_sixiang_stdzhuguo {
//! Golden coverage for SIXIANG「助国」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDZHUGUO: u32 = 3_147_233_152;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdzhuguo.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_compiler::compile_source_named("stdzhuguo-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_724, "identity-standard").unwrap();
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
    effects.extend(
        players
            .into_iter()
            .map(|player| EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }),
    );
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDZHUGUO),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == STDZHUGUO)
        .unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, target, skill)
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
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

#[test]
fn package_publishes_abi128_literal_hand_target_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == STDZHUGUO)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (
            targets.min,
            targets.max,
            targets.allow_self,
            targets.hand_lt
        ),
        (1, 1, true, Some(2))
    );
}

#[test]
fn target_with_zero_or_one_card_draws_exactly_to_two() {
    let (state, owner, target, skill) = setup();
    let (resolved, result) = activate(state, owner, target, skill, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(resolved.hand_count(target), 2);
    assert!(!resolved.has_pending_choice());

    let (mut state, owner, target, skill) = setup();
    inject_hand(&mut state, target, 128_001);
    let (resolved, result) = activate(state, owner, target, skill, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(resolved.hand_count(target), 2);
}

#[test]
fn self_is_legal_but_two_card_target_and_second_use_are_rejected_atomically() {
    let (state, owner, _, skill) = setup();
    let (state, result) = activate(state, owner, owner, skill, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 2);
    let before_retry = canonical_state_hash(&state);
    let (same, result) = activate(state, owner, owner, skill, 11);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before_retry);

    let (mut state, owner, target, skill) = setup();
    inject_hand(&mut state, target, 128_010);
    inject_hand(&mut state, target, 128_011);
    let before = canonical_state_hash(&state);
    let (same, result) = activate(state, owner, target, skill, 12);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
}

mod original_sixiang_stdzhuying {
//! Golden tests for 四象「驻营」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const SIXIANG: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdzhuying.skill");
const STDZHUYING: u32 = 2_174_724_909;
const STDZHONGSHI: u32 = 3_843_853_491;
const STD_PANGLIN: u32 = 3_614_379_071;

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(SIXIANG).unwrap();
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
    let players = engine.state().players.alive_by_seat();
    let (owner, source, target) = (players[0], players[1], players[2]);
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
        skill_def: SkillDefId::new(STDZHUYING),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, source, target)
}

fn damage(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    nature: u8,
) -> Result<(), EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature,
        }],
    )
    .map(|_| ())
}

fn linked(tx: &Transaction, player: PlayerId) -> bool {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0)
}

fn set_linked(tx: &mut Transaction, player: PlayerId) {
    tx.state_mut()
        .players
        .get_mut(player)
        .unwrap()
        .marks
        .push((engine_core::content::MARK_CHAIN, 1));
}

#[test]
fn committed_package_declares_abi76_and_precise_filter_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(SIXIANG).unwrap();
    let compiled = content_compiler::compile_source_named("stdzhuying", SOURCE).unwrap();

    assert_eq!(compiled.manifest.rule_vm_abi_version, 73);
    let trigger = &compiled.trigger_index[0];
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_normal_damage_target_unlinked"
    );
    assert!(trigger.optional);
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDZHUYING && !skill.locked));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_PANGLIN)
        .unwrap();
    assert_eq!(character.name_key, "std_panglin");
    assert_eq!(character.skills, vec![STDZHUYING, STDZHONGSHI]);
}

#[test]
fn normal_damage_to_another_unlinked_player_offers_link_before_damage() {
    let (mut tx, owner, source, target) = setup(20_260_724);
    let hp = tx.state().players.get(target).unwrap().hp;

    assert_eq!(
        damage(&mut tx, source, target, damage_nature::NORMAL),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(tx.state().resolution.pending_observer, owner.raw());
    assert!(!linked(&tx, target));

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
    assert!(linked(&Transaction::begin(state.clone()), target));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn elemental_self_or_already_linked_damage_never_offers_the_skill() {
    let (mut elemental, _owner, source, target) = setup(20_260_725);
    damage(&mut elemental, source, target, damage_nature::FIRE).unwrap();
    assert!(!elemental.state().has_pending_choice());
    assert!(!linked(&elemental, target));

    let (mut self_damage, owner, source, _) = setup(20_260_726);
    damage(&mut self_damage, source, owner, damage_nature::NORMAL).unwrap();
    assert!(!self_damage.state().has_pending_choice());
    assert!(!linked(&self_damage, owner));

    let (mut already_linked, _owner, source, target) = setup(20_260_727);
    set_linked(&mut already_linked, target);
    damage(&mut already_linked, source, target, damage_nature::NORMAL).unwrap();
    assert!(!already_linked.state().has_pending_choice());
    assert!(linked(&already_linked, target));
}
}

mod original_sixiang_stdzishou {
//! Golden coverage for 标刘表「自守」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDZISHOU: u32 = 2_683_293_404;
const STD_CAOZHEN: u32 = 2_623_502_245;
const STD_QUYI: u32 = 2_888_834_865;
const STD_DONGYUN: u32 = 2_711_327_184;
const STD_LIUZAN: u32 = 4_100_887_402;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdzishou.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

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

#[test]
fn package_and_runtime_preserve_draw_then_skip_semantics() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    let compiled = content_compiler::compile_source_named("stdzishou", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, 2_026_074_401, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDZISHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    for (player, character) in players
        .iter()
        .zip([STD_CAOZHEN, STD_QUYI, STD_DONGYUN, STD_LIUZAN])
    {
        tx.state_mut()
            .players
            .get_mut(*player)
            .unwrap()
            .character_def = character;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    tx.state_mut().revision = 1;
    let state = tx.commit().state;
    let before = state.hand_count(owner);

    let (state, offered) = submit(state, owner, CommandKind::EndPhase, 1);
    let choice = offered.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(choice.choice_id.raw()),
            selections: vec![1],
            cancelled: false,
        },
        2,
    );

    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 4);
    assert_eq!(state.turn.phase, Phase::Discard);
    assert_eq!(state.turn.skipped_phase_count(), 1);
    assert!(accepted.events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Play,
        } if *current == owner
    )));
}
}

mod original_sixiang_stdzongji {
//! Golden coverage for SIXIANG「纵计」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDZONGJI: u32 = 3_394_363_597;
const STD_WANGYUN: u32 = 3_976_039_480;
const SIXIANG: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdzongji.skill");

fn setup() -> (GameState, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdzongji-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_724, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let source = seats[1];
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
        skill_def: SkillDefId::new(STDZONGJI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let state = tx.state_mut().players.get_mut(*player).unwrap();
        state.hp = 4;
        state.max_hp = 4;
    }
    tx.state_mut().turn.current = source;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, source, target)
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
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
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn play(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
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

fn assert_discarded(state: &GameState, card: CardId) {
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.get(card).unwrap();
    assert_eq!(card.zone, discard);
    assert_eq!(card.owner, PlayerId::NONE);
}

#[test]
fn package_publishes_exact_skill_and_closed_character() {
    let package = content_schema::ContentPackage::from_json_bytes(SIXIANG).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDZONGJI)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(
        trigger.filter,
        "owner_alive_sha_or_duel_damage_source_target_alive_hand_equip_nonempty"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_WANGYUN)
        .unwrap();
    assert_eq!(character.name_key, "std_wangyun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [2_318_160_114, STDZONGJI]);
}

#[test]
fn accepted_sha_damage_discards_target_then_source_cards() {
    let (state, owner, source, target) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, source, ZoneKind::Hand, CardKind::Sha, 126_000);
    let source_card = inject(
        &mut tx,
        source,
        ZoneKind::Equip,
        CardKind::HorseOff,
        126_001,
    );
    let target_card = inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, 126_002);

    let (offered, result) = play(tx.commit().state, source, sha, vec![target], 1);
    let confirm = result.choice.as_ref().unwrap();
    assert_eq!(confirm.kind, ChoiceKind::Confirm);
    assert_eq!(confirm.observer, owner);

    let (target_choice_state, result) = resolve(offered, owner, vec![1], false, 2);
    let target_choice = result.choice.as_ref().unwrap();
    assert_eq!(target_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(target_choice.observer, owner);
    assert_eq!(
        target_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [target_card.raw()]
    );

    let (source_choice_state, result) = resolve(
        target_choice_state,
        owner,
        vec![target_card.raw()],
        false,
        3,
    );
    let source_choice = result.choice.as_ref().unwrap();
    assert_eq!(source_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(source_choice.observer, owner);
    assert_eq!(
        source_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [source_card.raw()]
    );

    let (resolved, result) = resolve(
        source_choice_state,
        owner,
        vec![source_card.raw()],
        false,
        4,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(resolved.players.get(target).unwrap().hp, 3);
    assert_discarded(&resolved, target_card);
    assert_discarded(&resolved, source_card);
}

#[test]
fn duel_damage_uses_the_same_target_then_source_sequence() {
    let (state, owner, source, target) = setup();
    let mut tx = Transaction::begin(state);
    let duel = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        126_005,
    );
    let source_card = inject(&mut tx, source, ZoneKind::Hand, CardKind::Other, 126_006);
    let target_card = inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, 126_007);
    let (offered, result) = play(tx.commit().state, source, duel, vec![target], 5);
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);

    let (state, _) = resolve(offered, owner, vec![1], false, 6);
    let (state, _) = resolve(state, owner, vec![target_card.raw()], false, 7);
    let (resolved, result) = resolve(state, owner, vec![source_card.raw()], false, 8);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(resolved.players.get(target).unwrap().hp, 3);
    assert_discarded(&resolved, target_card);
    assert_discarded(&resolved, source_card);
}

#[test]
fn decline_preserves_both_participants_cards() {
    let (state, owner, source, target) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, source, ZoneKind::Hand, CardKind::Sha, 126_010);
    let source_card = inject(&mut tx, source, ZoneKind::Hand, CardKind::Other, 126_011);
    let target_card = inject(&mut tx, target, ZoneKind::Equip, CardKind::Armor, 126_012);
    let (offered, result) = play(tx.commit().state, source, sha, vec![target], 10);
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);

    let (declined, result) = resolve(offered, owner, vec![], true, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(declined.cards.get(source_card).unwrap().owner, source);
    assert_eq!(declined.cards.get(target_card).unwrap().owner, target);
}

#[test]
fn non_sha_or_duel_damage_and_empty_participants_do_not_prompt() {
    let (state, owner, source, target) = setup();
    let mut tx = Transaction::begin(state);
    let nanman = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        CardKind::TrickNanman,
        126_020,
    );
    inject(&mut tx, source, ZoneKind::Hand, CardKind::Other, 126_021);
    inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, 126_022);
    let (_, result) = play(tx.commit().state, source, nanman, vec![], 20);
    assert!(
        result
            .choice
            .as_ref()
            .is_none_or(|choice| choice.observer != owner),
        "ordinary trick damage must not offer stdzongji"
    );

    let (state, owner, source, target) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, source, ZoneKind::Hand, CardKind::Sha, 126_023);
    let (_, result) = play(tx.commit().state, source, sha, vec![target], 21);
    assert!(
        result
            .choice
            .as_ref()
            .is_none_or(|choice| choice.observer != owner),
        "empty source and target must not offer stdzongji"
    );
}
}

mod original_sixiang_stdzuici {
//! Golden coverage for 标孙邵「罪辞」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_SUNSHAO: u32 = 4_214_291_567;
const STDDINGYI: u32 = 2_968_274_563;
const STDZUICI: u32 = 2_244_504_256;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdzuici.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn damage(owner: PlayerId, source: PlayerId) -> DomainEvent {
    DomainEvent::DamageDealt {
        source,
        target: owner,
        amount: 1,
        is_lose_hp: false,
        nature: 0,
    }
}

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdzuici-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 202);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_726, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source, donor) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDZUICI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, source, donor)
}

fn inject_field(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selection: u32,
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
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

fn accept_trigger(
    mut tx: Transaction,
    owner: PlayerId,
    source: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[damage(owner, source)],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    resolve(tx.commit().state, owner, 1, key)
}

#[test]
fn committed_package_publishes_complete_sunshao_and_abi202_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDZUICI
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target_source_movable_field_card"
            && trigger.optional
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_SUNSHAO)
        .unwrap();
    assert_eq!(character.skills, [STDDINGYI, STDZUICI]);
}

#[test]
fn equipment_and_delayed_trick_are_moved_to_the_fixed_damage_source() {
    for (zone_kind, kind, def, key) in [
        (ZoneKind::Equip, CardKind::Weapon, 99_001, 1),
        (ZoneKind::Judge, CardKind::TrickLe, 99_002, 3),
    ] {
        let (mut tx, owner, source, donor) = setup();
        let moved = inject_field(&mut tx, donor, zone_kind, kind, def);
        let (state, offered) = accept_trigger(tx, owner, source, key);
        assert!(matches!(offered.result, CommandResult::Accepted { .. }));
        assert_eq!(
            state.resolution.continuation_kind,
            ContinuationKind::ChooseMoveFieldCardTo
        );
        assert_eq!(offered.choice.unwrap().options[0].token, moved.raw());
        let (state, completed) = resolve(state, owner, moved.raw(), key + 1);
        assert!(matches!(completed.result, CommandResult::Accepted { .. }));
        assert!(!state.has_pending_choice());
        let destination = state.zones.find(zone_kind, source).unwrap();
        assert_eq!(state.cards.get(moved).unwrap().zone, destination);
        assert_eq!(state.cards.get(moved).unwrap().owner, source);
    }
}

#[test]
fn occupied_equipment_slot_and_duplicate_delayed_trick_are_not_movable() {
    let (mut tx, owner, source, donor) = setup();
    inject_field(&mut tx, source, ZoneKind::Equip, CardKind::Armor, 99_003);
    inject_field(&mut tx, donor, ZoneKind::Equip, CardKind::Armor, 99_004);
    inject_field(
        &mut tx,
        source,
        ZoneKind::Judge,
        CardKind::TrickBingliang,
        99_005,
    );
    inject_field(
        &mut tx,
        donor,
        ZoneKind::Judge,
        CardKind::TrickBingliang,
        99_006,
    );
    assert!(dispatch_for_events(
        &mut tx,
        &[damage(owner, source)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
}
}
