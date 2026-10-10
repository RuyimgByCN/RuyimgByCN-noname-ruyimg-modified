//! Golden for DIY「对决」的颜色成本、阶段次数与临时技能生命周期。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const NSDUIJUE: u32 = 4_088_249_173;
const AFTER_RED: u32 = 3_607_483_734;
const AFTER_BLACK: u32 = 3_115_572_483;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(94_000_000 + u32::from(rank)),
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

fn skill_with_def(
    state: &engine_core::state::GameState,
    owner: PlayerId,
    def: u32,
) -> Option<SkillId> {
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
                .is_some_and(|skill| skill.def.raw() == def)
        })
}

fn play_phase_effects(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    )?;
    run_effects(tx, effects).map(|_| ())
}

#[test]
fn red_cost_grants_two_black_duels_then_expires_at_play_phase_end() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .character_index
        .iter()
        .any(|character| character.name_key == "ns_yanliang"
            && character.skills == [NSDUIJUE, 2_353_402_107]));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_715),
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
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NSDUIJUE),
    });
    run_effects(&mut tx, setup).unwrap();
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    tx.state_mut().turn.phase = Phase::Play;
    let red_cost = inject(&mut tx, owner, Suit::Heart, 1);
    let black_one = inject(&mut tx, owner, Suit::Spade, 2);
    let black_two = inject(&mut tx, owner, Suit::Club, 3);
    let black_three = inject(&mut tx, owner, Suit::Spade, 4);

    assert_eq!(
        play_phase_effects(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (mut state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![red_cost.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let temp = skill_with_def(&state, owner, AFTER_RED).expect("red cost temp skill");
    assert!(skill_with_def(&state, owner, AFTER_BLACK).is_none());

    for (key, card) in [(2, black_one), (3, black_two)] {
        let (next, result) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ActivateSkill {
                    skill: temp,
                    targets: vec![target],
                    card_targets: vec![card],
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        state = next;
    }
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let (same, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::ActivateSkill {
                skill: temp,
                targets: vec![target],
                card_targets: vec![black_three],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_ne!(same.cards.get(black_three).unwrap().zone, discard);

    let mut ending = Transaction::begin(same);
    let effects = dispatch_for_events(
        &mut ending,
        &[DomainEvent::PhaseEnded {
            current: owner,
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
    run_effects(&mut ending, effects).unwrap();
    assert!(skill_with_def(ending.state(), owner, AFTER_RED).is_none());

    ending.state_mut().turn.phase = Phase::Play;
    let black_cost = inject(&mut ending, owner, Suit::Club, 5);
    let red_duel = inject(&mut ending, owner, Suit::Diamond, 6);
    assert_eq!(
        play_phase_effects(&mut ending, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let choice_id = ChoiceId::new(ending.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        ending.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(ending.state().revision),
            idempotency_key: IdempotencyKey(5),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![black_cost.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let temp = skill_with_def(&state, owner, AFTER_BLACK).expect("black cost temp skill");
    let (_, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(6),
            kind: CommandKind::ActivateSkill {
                skill: temp,
                targets: vec![target],
                card_targets: vec![red_duel],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
}
