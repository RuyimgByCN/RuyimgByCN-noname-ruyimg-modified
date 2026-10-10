//! Golden coverage for SP「死谏」and the OFFLINE 田丰 cross-package character closure.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TIANFENG: u32 = 4_153_931_211;
const SIJIAN: u32 = 3_832_046_646;
const GZSUISHI: u32 = 4_271_491_554;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/sijian.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const OFFLINE_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("sijian-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_722, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, bystander) = (players[0], players[1], players[2]);
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
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SIJIAN),
        },
        EffectRequest::DrawCards {
            player: owner,
            count: 1,
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, target, bystander)
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

fn lose_last_hand_card(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state().hand_cards(owner)[0];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: discard,
            from: Some(hand),
        }],
    )
    .map(|_| ())
}

#[test]
fn committed_packages_expose_sijian_and_complete_tianfeng_closure() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 70);
    assert!(sp.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == SIJIAN
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_from_hand_empty_other_discardable_hand_equip"
            && trigger.optional
    }));

    let offline = content_schema::ContentPackage::from_json_bytes(OFFLINE_PACKAGE).unwrap();
    assert!(offline.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "sp"
            && dependency.version == "0.1.0"
    }));
    let character = offline
        .character_index
        .iter()
        .find(|character| character.id == TIANFENG)
        .unwrap();
    assert_eq!(character.name_key, "tianfeng");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, vec![SIJIAN, GZSUISHI]);

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&sp).unwrap();
    std::sync::Arc::make_mut(&mut state.content).load_package(&offline).unwrap();
}

#[test]
fn losing_the_last_hand_card_can_discard_another_players_hand_or_equip_card() {
    let (mut tx, owner, target, _) = setup();
    let target_card = tx.state().hand_cards(target)[0];
    assert_eq!(
        lose_last_hand_card(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );

    let state = resolve(tx.state().clone(), owner, vec![1], 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDiscardTargetCard
    );
    let state = resolve(state, owner, vec![target.raw()], 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCard
    );
    let state = resolve(state, owner, vec![target_card.raw()], 3);
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(target_card).unwrap().zone, discard);
}

#[test]
fn nonfinal_hand_loss_and_judgment_only_targets_do_not_offer_sijian() {
    let (mut nonfinal, owner, _, _) = setup();
    run_effects(
        &mut nonfinal,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    lose_last_hand_card(&mut nonfinal, owner).unwrap();
    assert_eq!(nonfinal.state().hand_count(owner), 1);
    assert!(!nonfinal.state().has_pending_choice());

    let (mut judgment_only, owner, target, bystander) = setup();
    for player in [target, bystander] {
        let hand = judgment_only
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .unwrap();
        let judge = judgment_only
            .state()
            .zones
            .find(ZoneKind::Judge, player)
            .unwrap();
        let cards = judgment_only.state().hand_cards(player);
        run_effects(
            &mut judgment_only,
            vec![EffectRequest::MoveCards {
                cards,
                to: judge,
                from: Some(hand),
            }],
        )
        .unwrap();
    }
    lose_last_hand_card(&mut judgment_only, owner).unwrap();
    assert!(!judgment_only.state().has_pending_choice());
}
