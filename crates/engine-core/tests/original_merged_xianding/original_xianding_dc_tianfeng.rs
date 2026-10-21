//! Golden for XIANDING 新杀田丰【随势】（ABI 596）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DC_TIANFENG: u32 = 3_051_387_474;
const DC_YANGBIAO: u32 = 3_963_716_631;
const DC_LINGCAO: u32 = 2_455_294_139;
const SIJIAN: u32 = 3_832_046_646;
const DCSUISHI: u32 = 2_514_713_423;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup() -> (GameState, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package())
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source, target) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCSUISHI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().character_def = DC_TIANFENG;
    tx.state_mut()
        .players
        .get_mut(source)
        .unwrap()
        .character_def = DC_YANGBIAO;
    tx.state_mut()
        .players
        .get_mut(target)
        .unwrap()
        .character_def = DC_LINGCAO;
    (tx.commit().state, owner, source, target)
}

#[test]
fn package_closes_dc_tianfeng_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 596);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DC_TIANFENG)
        .unwrap();
    assert_eq!(character.name_key, "dc_tianfeng");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [SIJIAN, DCSUISHI]);
}

#[test]
fn dcsuishi_draws_when_the_dying_damage_source_shares_the_owner_group() {
    let (state, owner, source, target) = setup();
    let mut tx = Transaction::begin(state);
    let hand = tx.state().hand_count(owner);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: target,
            source,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);
}

#[test]
fn dcsuishi_death_forces_one_to_all_hand_cards_and_revalidates_the_choice() {
    let (state, owner, _source, target) = setup();
    let mut tx = Transaction::begin(state);
    tx.state_mut()
        .players
        .get_mut(target)
        .unwrap()
        .character_def = DC_YANGBIAO;
    let before = tx.state().hand_count(owner);
    assert!(before >= 2);
    let selected: Vec<_> = tx.state().hand_cards(owner)[..2]
        .iter()
        .map(|card| card.raw())
        .collect();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: target,
            killer: PlayerId::NONE,
            hand_equip_cards: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: selected,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before - 2);

    let (state, owner, _source, target) = setup();
    let mut empty = Transaction::begin(state);
    empty
        .state_mut()
        .players
        .get_mut(target)
        .unwrap()
        .character_def = DC_YANGBIAO;
    run_effects(
        &mut empty,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let effects = dispatch_for_events(
        &mut empty,
        &[DomainEvent::PlayerKilled {
            player: target,
            killer: PlayerId::NONE,
            hand_equip_cards: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
}
