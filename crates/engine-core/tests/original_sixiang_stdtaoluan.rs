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
