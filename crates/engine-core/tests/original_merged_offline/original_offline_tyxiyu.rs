//! Golden for OFFLINE 刘阿「戏语」的转化牌/虚拟牌首目标语义。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const TY_LIUE: u32 = 4_278_964_445;
const TYXIYU: u32 = 3_638_198_419;
const YUI_LIEYIN1: u32 = 3_727_853_068;
const OFFLINE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");
const KEY: &[u8] = include_bytes!("../../../../content/original/ported/key/key.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let offline = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    let key = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&offline).unwrap();
    engine.load_package(&key).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_716),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_716,
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
        skill_def: SkillDefId::new(TYXIYU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, others[0], others[1])
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, virtual_card: bool) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: if virtual_card {
            CardDefId::NONE
        } else {
            CardDefId::new(89_000 + kind.tag() as u32)
        },
        kind,
        suit: if virtual_card {
            Suit::None
        } else {
            Suit::Spade
        },
        rank: if virtual_card { 0 } else { 7 },
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn dispatch(
    tx: &mut Transaction,
    source: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: source,
            card,
            targets,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn confirm(tx: Transaction, owner: PlayerId, key: u64) -> engine_core::state::GameState {
    let choice = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn offline_package_contains_complete_ty_liue() {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == TY_LIUE)
        .expect("committed 刘阿 character");
    assert_eq!(character.name_key, "ty_liue");
    assert_eq!(character.max_hp, 5);
    assert_eq!(character.initial_hp, 0, "omitted means start at max_hp");
    assert_eq!(character.skills, [TYXIYU]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TYXIYU));
}

#[test]
fn tyxiyu_ignores_physical_cards_and_draws_once_for_each_converted_or_virtual_use() {
    let (mut tx, owner, source, other) = setup();
    let physical = inject(&mut tx, source, CardKind::Sha, false);
    assert!(dispatch(&mut tx, source, physical, vec![owner, other])
        .unwrap()
        .is_empty());
    assert!(!tx.state().has_pending_choice());

    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: source,
            skill_def: SkillDefId::new(YUI_LIEYIN1),
        }],
    )
    .unwrap();
    let converted = inject(&mut tx, source, CardKind::Sha, false);
    let before = tx.state().hand_count(owner);
    assert_eq!(
        dispatch(&mut tx, source, converted, vec![owner, other]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = confirm(tx, owner, 1);
    assert_eq!(state.hand_count(owner), before + 1);
    assert!(
        !state.has_pending_choice(),
        "multi-target use fires only once"
    );

    let mut tx = Transaction::begin(state);
    let lieyin = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == source && skill.def.raw() == YUI_LIEYIN1)
        .expect("converted-card modifier skill")
        .id;
    run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill: lieyin }]).unwrap();
    let view_as = inject(&mut tx, source, CardKind::Sha, false);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_CARD, view_as.raw() as i32);
    let before = tx.state().hand_count(owner);
    assert_eq!(
        dispatch(&mut tx, source, view_as, vec![owner]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_CARD, 0);
    let state = confirm(tx, owner, 2);
    assert_eq!(state.hand_count(owner), before + 1);
    assert!(!state.has_pending_choice());

    let mut tx = Transaction::begin(state);
    let virtual_card = inject(&mut tx, source, CardKind::Other, true);
    let before = tx.state().hand_count(owner);
    assert_eq!(
        dispatch(&mut tx, source, virtual_card, vec![other, owner]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = confirm(tx, owner, 3);
    assert_eq!(state.hand_count(owner), before + 1);
    assert!(!state.has_pending_choice());
}
