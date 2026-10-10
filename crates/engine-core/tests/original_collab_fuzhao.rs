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
