//! Golden for XIANDING（限定包）批次四十八：
//! 迅势（dcsbxunshi）、据谏（dcjujian）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCSBXUNSHI: u32 = 4_112_740_389;
const DCJUJIAN: u32 = 3_730_191_152;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn live_skill(tx: &Transaction, player: PlayerId, def: u32) -> engine_api::SkillId {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn inject_hand_card(tx: &mut Transaction, player: PlayerId) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind: CardKind::Shan,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_exposes_batch48_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let unlimited = package
        .modifier_index
        .iter()
        .find(|m| m.skill_def_id == DCSBXUNSHI && m.target == "card_use_unlimited")
        .expect("dcsbxunshi card_use_unlimited");
    assert_eq!(unlimited.selector.as_deref(), Some("sha_thunder"));

    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == DCSBXUNSHI)
        .expect("dcsbxunshi trigger");
    assert_eq!(trigger.event, "damage");

    let jujian = package
        .active_index
        .iter()
        .find(|a| a.skill_def_id == DCJUJIAN)
        .expect("dcjujian active");
    assert_eq!(jujian.window, "play_phase");
}

#[test]
fn dcsbxunshi_discards_and_draws_on_thunder_sha_damage() {
    let (mut tx, owner, first, _second) = setup(20_260_904);
    add_skill(&mut tx, owner, DCSBXUNSHI);
    let before = tx.state().hand_count(owner);
    inject_hand_card(&mut tx, first);

    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target: first,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::THUNDER,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let dispatched = run_effects(&mut tx, fired);
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::DiscardOneHandOrEquip
    );
    let hand_card = tx.state().hand_cards(first)[0];
    let state = resolve(tx.state().clone(), first, vec![hand_card.raw()], false, 1);
    assert_eq!(state.hand_count(owner), before + 2);
    assert_eq!(state.hand_count(first), 0);
}

#[test]
fn dcjujian_draws_one_for_target_and_bans_use_against_owner() {
    let (mut tx, owner, first, _second) = setup(20_260_905);
    add_skill(&mut tx, owner, DCJUJIAN);
    let before = tx.state().hand_count(first);
    let skill = live_skill(&tx, owner, DCJUJIAN);
    let revision = tx.state().revision + 1;
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![first],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(first), before + 1);
    assert!(state
        .turn
        .use_ban_pairs
        .contains(&(first, owner)));
}
