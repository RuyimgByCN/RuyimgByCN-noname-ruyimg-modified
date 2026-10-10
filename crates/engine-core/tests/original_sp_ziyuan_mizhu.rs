//! Golden coverage for SP 糜竺「资援」rewritten from legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, CardMoveReason, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::effective_card_number;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{choose_command, query, Engine, QueryContext, Snapshot};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const ZIYUAN: u32 = 2_303_983_371;
const JUGU: u32 = 2_684_521_577;
const MIZHU: u32 = 3_031_952_466;
const NUMBER_THIRTEEN: u32 = 4_200_000_131;
const NUMBER_TWELVE: u32 = 4_200_000_132;
const GIVER_DIES: u32 = 4_200_000_133;
const TARGET_DIES: u32 = 4_200_000_134;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/ziyuan.skill");
const JUGU_SOURCE: &str = include_str!("../../../content/original/ported/sp/jugu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const MODULE_LOGIC_HASH: [u8; 32] = [
    21, 26, 175, 88, 48, 123, 237, 245,
    208, 193, 63, 34, 142, 204, 22, 13,
    42, 95, 51, 235, 64, 164, 128, 170,
    160, 1, 57, 86, 247, 238, 217, 69,
];
const PACKAGE_LOGIC_HASH: [u8; 32] = [104,11,251,180,248,79,187,16,89,76,180,14,141,177,202,117,119,248,212,56,232,136,9,34,34,192,63,77,154,118,188,178];
const TEST_HELPERS: &str = r#"
skill number_thirteen(4200000131) locked {
  modifier card_number override 13;
}

skill number_twelve(4200000132) locked {
  modifier card_number override 12;
}

skill giver_dies(4200000133) locked {
  trigger cards_given owner_alive_gave_cards_to_other_alive {
    effect lose_hp(self, 99);
  }
}

skill target_dies(4200000134) locked {
  trigger cards_moved owner_received_gift_from_other {
    effect lose_hp(self, 99);
  }
}
"#;

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    cards: Vec<CardId>,
}

fn inject(state: &mut GameState, owner: PlayerId, zone_kind: ZoneKind, rank: u8) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_100 + u32::from(rank)),
        kind: if zone_kind == ZoneKind::Equip {
            CardKind::Weapon
        } else {
            CardKind::Other
        },
        suit: Suit::Club,
        rank,
        weapon_range: u8::from(zone_kind == ZoneKind::Equip) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(ranks: &[u8], owner_helpers: &[u32], target_helpers: &[u32]) -> Setup {
    let package = compile_source_named(
        "sp-ziyuan-test",
        &format!("{SOURCE}\n{JUGU_SOURCE}\n{TEST_HELPERS}"),
    )
    .unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_073_001, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(ZIYUAN),
    });
    effects.extend(owner_helpers.iter().map(|skill| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(*skill),
    }));
    effects.extend(target_helpers.iter().map(|skill| EffectRequest::AddSkill {
        player: target,
        skill_def: SkillDefId::new(*skill),
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let max_hp = tx.state().players.get(target).unwrap().max_hp as i32;
    tx.state_mut().players.get_mut(target).unwrap().hp = max_hp - 1;
    tx.state_mut().revision = 1;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == ZIYUAN)
        .unwrap();
    let mut state = tx.commit().state;
    let cards = ranks
        .iter()
        .map(|rank| inject(&mut state, owner, ZoneKind::Hand, *rank))
        .collect();
    Setup {
        state,
        owner,
        target,
        skill,
        cards,
    }
}

fn activate(setup: Setup, selected: Vec<CardId>, key: u64) -> (GameState, PipelineResult) {
    command(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: selected,
        },
        key,
    )
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

#[test]
fn package_module_character_and_jugu_contract_are_stable() {
    let compiled = compile_source_named("sp-ziyuan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 301);
    let active = compiled.active_index.first().unwrap();
    assert_eq!(active.skill_def_id, ZIYUAN);
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_play_phase, Some(1));
    assert_eq!(
        active
            .selection
            .as_ref()
            .unwrap()
            .sum_effective_number_exact,
        Some(13)
    );
    let module = compiled
        .modules
        .iter()
        .find(|module| module.name == "ziyuan_active_play_phase_0")
        .unwrap();
    assert_eq!(module.code, [83, 85, 16, 1, 0, 0, 0, 90, 32, 1, 1]);
    assert_eq!(module.logic_hash, MODULE_LOGIC_HASH);
    assert_eq!(
        rule_vm::bytecode::effect_tag::GIVE_SELECTED_THEN_RECOVER_TARGET,
        288
    );

    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    assert_eq!(package.manifest.logic_hash, PACKAGE_LOGIC_HASH);
    assert_eq!(package.manifest.logic_hash, package.compute_logic_hash());
    assert_eq!(package.skill_index.len(), 728);
    assert_eq!(package.character_index.len(), 217);
    assert_eq!(package.modules.len(), 295);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == MIZHU)
        .unwrap();
    assert_eq!(character.name_key, "mizhu");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "shu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [ZIYUAN, JUGU]);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == JUGU
            && modifier.target == "initial_hand_count"
            && modifier.mode == "add"
            && modifier.value_source.as_deref() == Some("owner_max_hp")
            && modifier.value == 1
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == JUGU
            && modifier.target == "hand_limit"
            && modifier.mode == "add"
            && modifier.value_source.as_deref() == Some("owner_max_hp")
            && modifier.value == 1
    }));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn king_and_five_plus_eight_are_valid_and_full_target_is_allowed() {
    let mut king = setup(&[13], &[], &[]);
    let max_hp = king.state.players.get(king.target).unwrap().max_hp as i32;
    king.state.players.get_mut(king.target).unwrap().hp = max_hp;
    let card = king.cards[0];
    let target = king.target;
    let (state, result) = activate(king, vec![card], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(card).unwrap().owner, target);
    assert_eq!(state.players.get(target).unwrap().hp, max_hp);

    let pair = setup(&[5, 8], &[], &[]);
    let target = pair.target;
    let cards = pair.cards.clone();
    let hp = pair.state.players.get(target).unwrap().hp;
    let (state, result) = activate(pair, cards.clone(), 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(cards
        .iter()
        .all(|card| state.cards.get(*card).unwrap().owner == target));
    assert_eq!(state.players.get(target).unwrap().hp, hp + 1);
    let moves: Vec<_> = result
        .events
        .iter()
        .filter_map(|event| match event {
            engine_api::DomainEvent::CardsMoved { cards, reason, .. }
                if *reason == CardMoveReason::Give =>
            {
                Some(cards.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(moves, [cards]);
}

#[test]
fn jugu_still_adds_owner_max_hp_to_initial_hand_and_hand_limit() {
    let setup = setup(&[], &[JUGU], &[]);
    let player = setup.state.players.get(setup.owner).unwrap();
    let max_hp = u32::from(player.max_hp);
    assert_eq!(
        engine_core::query::initial_hand_count_for(&setup.state, setup.owner, 4),
        4 + max_hp
    );
    assert_eq!(
        query(
            &setup.state,
            &QueryRequest::HandLimit {
                player: setup.owner,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(player.hp + max_hp as i32)
    );
}

#[test]
fn invalid_sums_duplicates_and_non_hand_cards_are_rejected() {
    for (ranks, selected) in [(&[5, 7][..], vec![0, 1]), (&[6, 8][..], vec![0, 1])] {
        let setup = setup(ranks, &[], &[]);
        let cards: Vec<_> = selected.iter().map(|index| setup.cards[*index]).collect();
        let (_, result) = activate(setup, cards, 10 + ranks[0] as u64);
        assert!(matches!(result.result, CommandResult::Rejected { .. }));
    }

    let duplicate = setup(&[13], &[], &[]);
    let card = duplicate.cards[0];
    let (_, result) = activate(duplicate, vec![card, card], 30);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));

    let mut non_hand = setup(&[5], &[], &[]);
    let equipment = inject(&mut non_hand.state, non_hand.owner, ZoneKind::Equip, 8);
    let hand = non_hand.cards[0];
    let (_, result) = activate(non_hand, vec![hand, equipment], 31);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}

#[test]
fn effective_card_number_modifier_changes_exact_sum_legality() {
    let positive = setup(&[12], &[NUMBER_THIRTEEN], &[]);
    let card = positive.cards[0];
    assert_eq!(
        effective_card_number(
            &positive.state,
            positive.owner,
            positive.state.cards.get(card).unwrap()
        ),
        13
    );
    let (_, result) = activate(positive, vec![card], 40);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    let negative = setup(&[13], &[NUMBER_TWELVE], &[]);
    let card = negative.cards[0];
    assert_eq!(
        effective_card_number(
            &negative.state,
            negative.owner,
            negative.state.cards.get(card).unwrap()
        ),
        12
    );
    let (_, result) = activate(negative, vec![card], 41);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}

#[test]
fn use_limit_resets_on_each_play_phase_entry() {
    let setup = setup(&[13, 13], &[], &[]);
    let owner = setup.owner;
    let target = setup.target;
    let skill = setup.skill;
    let first = setup.cards[0];
    let second = setup.cards[1];
    let (state, first_result) = activate(setup, vec![first], 50);
    assert!(matches!(
        first_result.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.skills.get(skill).unwrap().play_phase_use_count, 1);

    let (state, repeated) = command(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![second],
        },
        51,
    );
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Discard.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(
        tx.state().skills.get(skill).unwrap().play_phase_use_count,
        0
    );
    let state = tx.commit().state;
    let (_, second_result) = command(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![second],
        },
        52,
    );
    assert!(matches!(
        second_result.result,
        CommandResult::Accepted { .. }
    ));
}

#[test]
fn runner_selects_an_exact_sum_subset() {
    let setup = setup(&[1, 5, 8], &[], &[]);
    let owner = setup.owner;
    let target = setup.target;
    let skill = setup.skill;
    let five = setup.cards[1];
    let eight = setup.cards[2];
    let (state, result) = command(
        setup.state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        60,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let choice = result.choice.unwrap();
    let mut snapshot = Snapshot::capture(&state);
    snapshot.pending_choice = Some(choice);
    let mut engine = Engine::new();
    engine.restore_snapshot(&snapshot).unwrap();
    let command = choose_command(&engine);
    assert!(matches!(
        &command.kind,
        CommandKind::ResolveChoice {
            selections,
            cancelled: false,
            ..
        } if selections == &vec![five.raw(), eight.raw()]
    ));
    assert!(matches!(
        engine.submit(command),
        CommandResult::Accepted { .. }
    ));
    assert_eq!(engine.state().cards.get(five).unwrap().owner, target);
    assert_eq!(engine.state().cards.get(eight).unwrap().owner, target);
}

#[test]
fn nested_giver_death_does_not_cancel_recovery_and_target_death_skips_it() {
    let giver_dies = setup(&[13], &[GIVER_DIES], &[]);
    let owner = giver_dies.owner;
    let target = giver_dies.target;
    let target_hp = giver_dies.state.players.get(target).unwrap().hp;
    let card = giver_dies.cards[0];
    let (state, result) = activate(giver_dies, vec![card], 70);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.players.get(owner).unwrap().alive);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp + 1);
    let killed = result
        .events
        .iter()
        .position(|event| {
            matches!(
                event,
                engine_api::DomainEvent::PlayerKilled { player, .. } if *player == owner
            )
        })
        .unwrap();
    let recovered = result
        .events
        .iter()
        .position(|event| {
            matches!(
                event,
                engine_api::DomainEvent::HpRecovered { player, .. } if *player == target
            )
        })
        .unwrap();
    assert!(killed < recovered);

    let target_dies = setup(&[13], &[], &[TARGET_DIES]);
    let target = target_dies.target;
    let card = target_dies.cards[0];
    let (state, result) = activate(target_dies, vec![card], 71);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.players.get(target).unwrap().alive);
    assert!(!state.has_pending_choice());
    assert!(result.events.iter().any(|event| {
        matches!(
            event,
            engine_api::DomainEvent::PlayerKilled { player, .. } if *player == target
        )
    }));
    assert!(!result.events.iter().any(|event| {
        matches!(
            event,
            engine_api::DomainEvent::HpRecovered { player, .. } if *player == target
        )
    }));
}
