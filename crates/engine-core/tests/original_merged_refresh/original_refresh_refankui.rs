//! Golden coverage for 界反馈's per-damage-point card-taking contract.

use content_compiler::compile_source_named;
use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REFANKUI: u32 = 3_719_484_801;
const SOURCE: &str = include_str!("../../../../content/original/ported/refresh/refankui.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = compile_source_named("refankui-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_718, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(REFANKUI),
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

fn clear_personal_cards(tx: &mut Transaction, player: PlayerId) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for kind in [ZoneKind::Hand, ZoneKind::Equip] {
        let zone = tx.state().zones.find(kind, player).unwrap();
        let cards = tx.state().zones.get(zone).unwrap().cards.clone();
        if !cards.is_empty() {
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: discard,
                    from: Some(zone),
                }],
            )
            .unwrap();
        }
    }
}

#[test]
fn committed_package_declares_refankui_with_required_abi() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 22);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == REFANKUI
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target_source_alive_gainable_card_per_damage_point"
            && trigger.optional
    }));
}

#[test]
fn other_source_damage_offers_one_take_for_each_damage_point() {
    let (state, owner, source) = setup();
    let owner_hand = state.hand_count(owner);
    let source_hand = state.hand_count(source);
    let (state, result) = damage(state, source, owner, 2);
    assert_eq!(result, Err(EffectError::ChoiceRequested));

    let (state, accepted_first) = resolve(state, owner, vec![1], 1);
    assert!(matches!(
        accepted_first.result,
        CommandResult::Accepted { .. }
    ));
    // 第一次触发的选牌先于第二次触发的确认结算。
    assert_eq!(
        accepted_first.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let first = accepted_first.choice.as_ref().unwrap().options[0].token;
    let (state, first_taken) = resolve(state, owner, vec![first], 2);
    assert_eq!(
        first_taken.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );
    let (state, accepted_second) = resolve(state, owner, vec![1], 3);
    assert_eq!(
        accepted_second.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let second = accepted_second.choice.as_ref().unwrap().options[0].token;
    let (state, taken) = resolve(state, owner, vec![second], 4);
    assert!(matches!(taken.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    assert_eq!(state.hand_count(source), source_hand - 2);
}

#[test]
fn self_damage_only_offers_equipment_and_returns_it_to_hand() {
    let (state, owner, _) = setup();
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
    let hand_before = tx.state().hand_count(owner);
    let (state, result) = damage(tx.commit().state, owner, owner, 1);
    assert_eq!(result, Err(EffectError::ChoiceRequested));

    let (state, accepted) = resolve(state, owner, vec![1], 5);
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
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::PickTargetCard
    );

    let (state, taken) = resolve(state, owner, vec![card.raw()], 6);
    assert!(matches!(taken.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert_eq!(
        state.cards.get(card).unwrap().zone,
        state.zones.find(ZoneKind::Hand, owner).unwrap()
    );
}

#[test]
fn sourced_damage_with_no_gainable_card_opens_no_choice() {
    let (state, owner, source) = setup();
    let mut tx = Transaction::begin(state);
    clear_personal_cards(&mut tx, source);
    let (state, result) = damage(tx.commit().state, source, owner, 1);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());
}
